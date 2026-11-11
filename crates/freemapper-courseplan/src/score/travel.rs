//! Travel time over the map: a raster cost surface with the fastest route and clearly different
//! alternatives (ADR 0061). The score code talks to [`TravelTime`] only, so a fuller route analysis
//! can replace the surface later.

use freemapper_courseplan_model::MapPoint;

use crate::map::MapIndex;
use crate::score::ScoreError;

mod raster;
mod search;
#[cfg(test)]
mod tests;

use raster::Grid;
use search::Searcher;

/// Seconds per kilometre on level open ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace(pub u32);

/// Settings of the surface and of the alternatives (design §9, ruling 5).
#[derive(Debug, Clone, PartialEq)]
pub struct RouteParams {
    /// Edge of a raster cell in metres on the ground.
    pub cell_m: f64,
    /// Most routes per leg, the fastest included.
    pub max_routes: usize,
    /// Share of an alternative's cells that may lie within the corridor of another route.
    pub max_overlap: f64,
    /// An alternative may be this many times as long as the fastest route.
    pub max_length_ratio: f64,
}

impl Default for RouteParams {
    fn default() -> Self {
        RouteParams {
            cell_m: 2.0,
            max_routes: 4,
            max_overlap: 0.6,
            max_length_ratio: 1.25,
        }
    }
}

/// One route: corner points in map millimetres, length in metres and time in seconds.
#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub points: Vec<MapPoint>,
    pub length_m: f64,
    pub time_s: f64,
}

/// The routes of one leg, best first, at most `max_routes`.
#[derive(Debug, Clone, PartialEq)]
pub struct LegRoutes {
    pub routes: Vec<Route>,
}

/// How long it takes to get from one point to another. The first part of the route analysis.
pub trait TravelTime {
    /// The fastest route and clearly different alternatives from `from` to `to`.
    fn legs(&self, from: MapPoint, to: MapPoint) -> Result<LegRoutes, ScoreError>;
    /// Only the fastest route. The default takes it from [`TravelTime::legs`]; a surface that can find it
    /// without the alternatives answers faster.
    fn fastest(&self, from: MapPoint, to: MapPoint) -> Result<Route, ScoreError> {
        let unreachable = ScoreError::Unreachable {
            from: from.clone(),
            to: to.clone(),
        };
        self.legs(from, to)?
            .routes
            .into_iter()
            .next()
            .ok_or(unreachable)
    }
    /// Whether a runner can stand at `p`: inside the raster and not in a blocked cell. `legs`
    /// is more forgiving: an end point in a blocked cell moves to a free cell within three cells
    /// that a straight line over free cells reaches, and fails with `Unreachable` if there is none.
    fn passable(&self, p: MapPoint) -> bool;
}

/// The raster cost surface.
pub struct CostSurface {
    grid: Grid,
    scale: u32,
    pace: Pace,
    params: RouteParams,
}

/// How far (in cells) a point in a blocked cell looks for a passable cell.
const SNAP_CELLS: i64 = 3;

impl CostSurface {
    /// Rasters the map objects of `index` (symbol classes in `speeds.toml`) and the `blocked`
    /// polygons (map millimetres; fewer than three points block along a line).
    pub fn build(
        index: &MapIndex,
        scale: u32,
        blocked: &[Vec<MapPoint>],
        profile: Pace,
        params: &RouteParams,
    ) -> Result<CostSurface, ScoreError> {
        let bad = |name, value| Err(ScoreError::BadRouteSetting { name, value });
        if !(params.cell_m.is_finite() && params.cell_m > 0.0) {
            return bad("cell_m", params.cell_m);
        }
        if !(0.0..=1.0).contains(&params.max_overlap) {
            return bad("max_overlap", params.max_overlap);
        }
        if !(params.max_length_ratio.is_finite() && params.max_length_ratio >= 1.0) {
            return bad("max_length_ratio", params.max_length_ratio);
        }
        if scale == 0 {
            return bad("scale", 0.0);
        }
        let grid = Grid::build(index, scale, blocked, params.cell_m)?;
        Ok(CostSurface {
            grid,
            scale,
            pace: profile,
            params: params.clone(),
        })
    }

    fn metres(&self, mm: f64) -> f64 {
        mm * f64::from(self.scale) / 1000.0
    }

    /// The passable cell for `p`: its own, or the nearest free one within `SNAP_CELLS` that a
    /// straight line reaches over free cells only, so a point never snaps across a wall.
    fn cell_for(&self, p: &MapPoint) -> Option<usize> {
        let cell = self.grid.cell_of(p)?;
        if self.grid.factor[cell].is_finite() {
            return Some(cell);
        }
        let w = self.grid.width as i64;
        let (col, row) = ((cell as i64) % w, (cell as i64) / w);
        let mut best: Option<(i64, usize)> = None;
        for dy in -SNAP_CELLS..=SNAP_CELLS {
            for dx in -SNAP_CELLS..=SNAP_CELLS {
                let (c, r) = (col + dx, row + dy);
                if c < 0 || r < 0 || c >= w || r >= self.grid.height as i64 {
                    continue;
                }
                let idx = (r * w + c) as usize;
                let d = dx * dx + dy * dy;
                // Nearest first; equal distance falls to the lower cell index.
                if self.grid.factor[idx].is_finite()
                    && best.is_none_or(|b| (d, idx) < b)
                    && self.free_line(col, row, c, r)
                {
                    best = Some((d, idx));
                }
            }
        }
        best.map(|(_, idx)| idx)
    }

    /// Whether the cells strictly between two cells on the straight line are all free.
    fn free_line(&self, c0: i64, r0: i64, c1: i64, r1: i64) -> bool {
        let steps = (c1 - c0).abs().max((r1 - r0).abs()) * 2;
        (1..steps).all(|k| {
            let t = k as f64 / steps as f64;
            let c = (c0 as f64 + (c1 - c0) as f64 * t).round() as i64;
            let r = (r0 as f64 + (r1 - r0) as f64 * t).round() as i64;
            let same = |cc, rr| (c, r) == (cc, rr);
            same(c0, r0)
                || same(c1, r1)
                || self.grid.factor[(r * self.grid.width as i64 + c) as usize].is_finite()
        })
    }

    fn route(
        &self,
        search: &Searcher<'_>,
        path: &[usize],
        from: &MapPoint,
        to: &MapPoint,
    ) -> Route {
        // Corner points only: a cell stays on the polyline where the direction changes.
        let w = self.grid.width as i64;
        let dir = |a: usize, b: usize| {
            (
                (b as i64 % w) - (a as i64 % w),
                (b as i64 / w) - (a as i64 / w),
            )
        };
        let mut points = vec![from.clone()];
        for i in 1..path.len().saturating_sub(1) {
            if dir(path[i - 1], path[i]) != dir(path[i], path[i + 1]) {
                points.push(self.grid.centre(path[i]));
            }
        }
        points.push(to.clone());
        let hop = |a: &MapPoint, b: &MapPoint| self.metres((a.x - b.x).hypot(a.y - b.y));
        let length_m = points.windows(2).map(|p| hop(&p[0], &p[1])).sum();
        // Time: the cell path, plus the way from the point to its cell centre at that cell's speed.
        let ends = [(from, path[0]), (to, path[path.len() - 1])];
        let end_cost: f64 = ends
            .iter()
            .map(|(p, cell)| hop(p, &self.grid.centre(*cell)) * f64::from(self.grid.factor[*cell]))
            .sum();
        let time_s = (search.cost(path) + end_cost) * f64::from(self.pace.0) / 1000.0;
        Route {
            points,
            length_m,
            time_s,
        }
    }
}

impl TravelTime for CostSurface {
    fn legs(&self, from: MapPoint, to: MapPoint) -> Result<LegRoutes, ScoreError> {
        let unreachable = || ScoreError::Unreachable {
            from: from.clone(),
            to: to.clone(),
        };
        let (s, t) = (
            self.cell_for(&from).ok_or_else(unreachable)?,
            self.cell_for(&to).ok_or_else(unreachable)?,
        );
        let mut search = Searcher::new(&self.grid, self.params.cell_m);
        let paths = search
            .alternatives(s, t, &self.params)
            .ok_or_else(unreachable)?;
        let routes = paths
            .iter()
            .map(|path| self.route(&search, path, &from, &to))
            .collect();
        Ok(LegRoutes { routes })
    }

    fn fastest(&self, from: MapPoint, to: MapPoint) -> Result<Route, ScoreError> {
        let unreachable = || ScoreError::Unreachable {
            from: from.clone(),
            to: to.clone(),
        };
        let (s, t) = (
            self.cell_for(&from).ok_or_else(unreachable)?,
            self.cell_for(&to).ok_or_else(unreachable)?,
        );
        let mut search = Searcher::new(&self.grid, self.params.cell_m);
        let path = search.shortest(s, t, None).ok_or_else(unreachable)?;
        Ok(self.route(&search, &path, &from, &to))
    }

    fn passable(&self, p: MapPoint) -> bool {
        self.grid
            .cell_of(&p)
            .is_some_and(|c| self.grid.factor[c].is_finite())
    }
}
