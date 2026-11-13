//! Dead ends on the cost surface (ADR 0085): how far into a pocket of the walkable network a place lies.
//!
//! A runner who goes to a control in a dead end has no route choice: every route shares one corridor and
//! the leg ends in a long run in and out. The depth of a place is the walking distance, on the ground, to
//! the first point where the network offers a choice.
//!
//! The method looks outwards from the place. It settles the cells within a horizon by walking distance
//! (Dijkstra on the 8-neighbour raster, metres, the speed factors do not count) and calls a cell *alive*
//! when the cells at least as far away as it is still connect it to the far rim of the horizon. A pocket
//! such as a spur to a quay, a cul-de-sac or a courtyard with one gate is not alive: it ends before the rim,
//! and the only way on to the rim leads through cells nearer to the place. Then it cuts the alive cells
//! into bands of constant walking distance and looks at the fronts in each band. One narrow front is one
//! corridor, no choice yet. The first band with two or more fronts (a junction: the way on splits) or with
//! a front wider than [`OPEN_WIDTH_M`] (open ground: a square, a park) is where the choice starts, and its
//! distance is the depth. A place on a through street has two alive fronts at once, one to each side, so
//! its depth is the distance to the nearest alive neighbour, a few metres.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use freemapper_courseplan_model::MapPoint;

use super::CostSurface;

/// Thickness of a band of constant walking distance, in metres.
const BAND_M: f64 = 4.0;
/// The pocket behind a place starts with the cells that are not alive within this many metres of it (more than
/// half the width of the widest corridor, since near the place the two sides of a corridor are still joined), and
/// takes in what joins them through cells that are not alive either.
const POCKET_SEED_M: f64 = 20.0;
/// A front narrower than this, in metres, is speckle at a band edge, not a way on.
const MIN_FRONT_M: f64 = 3.0;
/// A front at least this wide (area of the band divided by its thickness, in metres) is open ground, where a
/// runner can choose among many ways. The widest streets and quays of the maps stay below it.
pub const OPEN_WIDTH_M: f64 = 60.0;
/// How far beyond the limit the horizon reaches, in metres: a branch that goes on this far past the
/// limit counts as a real way on.
pub const HORIZON_MARGIN_M: f64 = 100.0;

/// The answer of [`CostSurface::dead_end`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeadEnd {
    /// Walking distance in metres from the place to the first point with route choice; infinite when none
    /// lies within the horizon.
    pub depth_m: f64,
    /// Longest walk in metres from the place to a cell of the pocket behind it (not alive): the total
    /// length of the branch is about the depth plus this. Zero when no pocket lies behind the place.
    pub tip_m: f64,
}

struct Entry {
    cost: f32,
    cell: u32,
}
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .total_cmp(&self.cost)
            .then_with(|| other.cell.cmp(&self.cell))
    }
}

const STEPS: [(i32, i32); 8] = [
    (0, -1),
    (1, 0),
    (0, 1),
    (-1, 0),
    (1, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
];

/// Disjoint sets over the settled cells.
struct Sets {
    parent: Vec<u32>,
}

impl Sets {
    fn new(n: usize) -> Sets {
        Sets {
            parent: (0..n as u32).collect(),
        }
    }

    fn find(&mut self, mut x: u32) -> u32 {
        while self.parent[x as usize] != x {
            let up = self.parent[self.parent[x as usize] as usize];
            self.parent[x as usize] = up;
            x = up;
        }
        x
    }

    /// Joins the sets; the smaller index becomes the root, so the result does not depend on the order.
    fn join(&mut self, a: u32, b: u32) -> u32 {
        let (a, b) = (self.find(a), self.find(b));
        let (low, high) = (a.min(b), a.max(b));
        self.parent[high as usize] = low;
        low
    }
}

/// Largest extent in metres of a blocked patch that does not count as an obstacle: a kiosk, a bollard, a
/// tree pit. A runner passes it on either side without a real choice, so it must not split the way ahead.
pub const SMALL_OBSTACLE_M: f64 = 14.0;

/// The cells of the window that belong to a small blocked patch: an 8-connected group of blocked cells whose
/// bounding box is at most `max_cells` on each side and that does not touch the border of the window or the
/// edge of the raster (then it may be part of something large).
fn small_obstacles(side: usize, free: &dyn Fn(usize) -> bool, max_cells: usize) -> Vec<bool> {
    let n = side * side;
    let mut seen = vec![false; n];
    let mut small = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    for first in 0..n {
        if seen[first] || free(first) {
            continue;
        }
        let mut members = vec![first];
        seen[first] = true;
        stack.push(first);
        let (mut c0, mut c1, mut r0, mut r1) = (side, 0, side, 0);
        let mut touches = false;
        while let Some(at) = stack.pop() {
            let (col, row) = (at % side, at / side);
            c0 = c0.min(col);
            c1 = c1.max(col);
            r0 = r0.min(row);
            r1 = r1.max(row);
            if col == 0 || row == 0 || col + 1 == side || row + 1 == side {
                touches = true;
            }
            for (dx, dy) in STEPS {
                let (nc, nr) = (col as i64 + i64::from(dx), row as i64 + i64::from(dy));
                if nc < 0 || nr < 0 || nc >= side as i64 || nr >= side as i64 {
                    continue;
                }
                let next = nr as usize * side + nc as usize;
                if !seen[next] && !free(next) {
                    seen[next] = true;
                    members.push(next);
                    stack.push(next);
                }
            }
        }
        if !touches && c1 - c0 < max_cells && r1 - r0 < max_cells {
            for m in members {
                small[m] = true;
            }
        }
    }
    small
}

/// How far (in cells) a place on a wall or in a blocked cell looks for a free cell to stand in.
const SNAP_CELLS: i64 = 4;
/// Free cells tried for a place on a wall before it counts as unjudged.
const MAX_TRIES: usize = 40;

impl CostSurface {
    /// The cells to stand in for `p`: its own when free, else the free cells within [`SNAP_CELLS`], nearest first
    /// and whatever lies between: a control stands on the line of a wall or a kerb, and the runner stands beside
    /// it. The lower cell index wins a tie. A free cell may be a speck between two lines, so the caller takes the
    /// first one that has room around it.
    fn stand_cells(&self, p: &MapPoint) -> Vec<usize> {
        let Some(cell) = self.grid.cell_of(p) else {
            return Vec::new();
        };
        if self.grid.factor[cell].is_finite() {
            return vec![cell];
        }
        let w = self.grid.width as i64;
        let (col, row) = ((cell as i64) % w, (cell as i64) / w);
        let mut found: Vec<(i64, usize)> = Vec::new();
        for dy in -SNAP_CELLS..=SNAP_CELLS {
            for dx in -SNAP_CELLS..=SNAP_CELLS {
                let (c, r) = (col + dx, row + dy);
                if c < 0 || r < 0 || c >= w || r >= self.grid.height as i64 {
                    continue;
                }
                let idx = (r * w + c) as usize;
                if self.grid.factor[idx].is_finite() {
                    found.push((dx * dx + dy * dy, idx));
                }
            }
        }
        found.sort_unstable();
        found.into_iter().map(|(_, idx)| idx).collect()
    }

    /// The depth of `p` into a dead end, looking `limit_m + HORIZON_MARGIN_M` metres out. `None` when `p`
    /// is not on the surface or the walkable region around it is smaller than the horizon, so that nothing
    /// can be told (a tiny map, an island).
    pub fn dead_end(&self, p: &MapPoint, limit_m: f64) -> Option<DeadEnd> {
        self.stand_cells(p)
            .into_iter()
            .take(MAX_TRIES)
            .find_map(|start| self.dead_end_from(start, limit_m))
    }

    fn dead_end_from(&self, start: usize, limit_m: f64) -> Option<DeadEnd> {
        let cell_m = self.params.cell_m;
        let horizon = (limit_m + HORIZON_MARGIN_M) as f32;
        let reach = (f64::from(horizon) / cell_m).ceil() as i64 + 2;
        let side = (2 * reach + 1) as usize;
        let w = self.grid.width as i64;
        let (c0, r0) = ((start as i64) % w, (start as i64) / w);
        // The window is a square of cells around the start; local index = (row - r0 + reach) * side + ...
        let local = |col: i64, row: i64| -> Option<usize> {
            let (dc, dr) = (col - c0 + reach, row - r0 + reach);
            ((0..side as i64).contains(&dc) && (0..side as i64).contains(&dr))
                .then(|| dr as usize * side + dc as usize)
        };
        let global = |l: usize| -> Option<usize> {
            let (dc, dr) = ((l % side) as i64 - reach, (l / side) as i64 - reach);
            let (col, row) = (c0 + dc, r0 + dr);
            (col >= 0 && row >= 0 && col < w && row < self.grid.height as i64)
                .then(|| (row * w + col) as usize)
        };
        let raw_free = |l: usize| global(l).is_some_and(|g| self.grid.factor[g].is_finite());
        let small = small_obstacles(side, &raw_free, (SMALL_OBSTACLE_M / cell_m).ceil() as usize);
        let free = |l: usize| raw_free(l) || small[l];
        let step_ok = |from: usize, dx: i32, dy: i32| -> Option<usize> {
            let (col, row) = (
                (from % side) as i64 + i64::from(dx),
                (from / side) as i64 + i64::from(dy),
            );
            if col < 0 || row < 0 || col >= side as i64 || row >= side as i64 {
                return None;
            }
            let next = row as usize * side + col as usize;
            if !free(next) {
                return None;
            }
            // No cutting a corner of a blocked cell, as the route search does.
            if dx != 0 && dy != 0 {
                let a = from as i64 + i64::from(dx);
                let b = from as i64 + i64::from(dy) * side as i64;
                if !free(a as usize) || !free(b as usize) {
                    return None;
                }
            }
            Some(next)
        };
        let step_m = [cell_m as f32, (cell_m * std::f64::consts::SQRT_2) as f32];
        let mut dist = vec![f32::INFINITY; side * side];
        let s_local = local(c0, r0).expect("the start is in its own window");
        dist[s_local] = 0.0;
        let mut order: Vec<u32> = Vec::new();
        let mut heap = BinaryHeap::new();
        heap.push(Entry {
            cost: 0.0,
            cell: s_local as u32,
        });
        let mut settled = vec![false; side * side];
        while let Some(Entry { cost, cell }) = heap.pop() {
            let cell = cell as usize;
            if settled[cell] || cost > dist[cell] {
                continue;
            }
            settled[cell] = true;
            order.push(cell as u32);
            for (dx, dy) in STEPS {
                let Some(next) = step_ok(cell, dx, dy) else {
                    continue;
                };
                let next_cost = cost + step_m[usize::from(dx != 0 && dy != 0)];
                if next_cost <= horizon && next_cost < dist[next] {
                    dist[next] = next_cost;
                    heap.push(Entry {
                        cost: next_cost,
                        cell: next as u32,
                    });
                }
            }
        }
        // The far rim: cells in the last band below the horizon.
        let rim = horizon - BAND_M as f32;
        if dist.iter().all(|&d| !(rim..=horizon).contains(&d)) {
            return None;
        }
        // Alive: the cells at least as far away connect to the rim. Sweep from the far end inwards.
        let n = order.len();
        let mut pos = vec![u32::MAX; side * side];
        for (i, &c) in order.iter().enumerate() {
            pos[c as usize] = i as u32;
        }
        let band_of = |i: usize| (dist[order[i] as usize] / BAND_M as f32) as usize;
        let mut sets = Sets::new(n);
        let mut reaches = vec![false; n];
        let mut alive = vec![false; n];
        let mut inserted = vec![false; n];
        for i in (0..n).rev() {
            let cell = order[i] as usize;
            inserted[i] = true;
            let mut root = i as u32;
            reaches[i] = dist[cell] >= rim;
            for (dx, dy) in STEPS {
                let Some(next) = step_ok(cell, dx, dy) else {
                    continue;
                };
                let j = pos[next];
                if j == u32::MAX || !inserted[j as usize] {
                    continue;
                }
                let (ra, rb) = (sets.find(root), sets.find(j));
                if ra != rb {
                    let flag = reaches[ra as usize] || reaches[rb as usize];
                    root = sets.join(ra, rb);
                    reaches[root as usize] = flag;
                }
            }
            // A band is judged once every cell of it and beyond is in: its cells are alive when the
            // cells from the band's near edge outwards connect them to the rim. Judging cell by cell
            // would leave ragged slivers along the walls of a corridor, where a cell can be a local
            // maximum of the walking distance.
            if i == 0 || band_of(i - 1) != band_of(i) {
                let mut k = i;
                while k < n && band_of(k) == band_of(i) {
                    let root = sets.find(k as u32);
                    alive[k] = reaches[root as usize];
                    k += 1;
                }
            }
        }
        // Fronts: components of the alive cells within one band, cells counted for their area.
        let bands = band_of(n - 1) + 1;
        let mut by_band: Vec<Vec<usize>> = vec![Vec::new(); bands];
        for i in 0..n {
            if alive[i] {
                by_band[band_of(i)].push(i);
            }
        }
        let mut depth = f64::INFINITY;
        let mut sets = Sets::new(n);
        for (b, members) in by_band.iter().enumerate() {
            if members.is_empty() {
                continue;
            }
            let mut is_member = std::collections::HashSet::with_capacity(members.len());
            for &i in members {
                is_member.insert(i);
            }
            for &i in members {
                let cell = order[i] as usize;
                for (dx, dy) in STEPS {
                    let Some(next) = step_ok(cell, dx, dy) else {
                        continue;
                    };
                    let j = pos[next];
                    if j != u32::MAX && is_member.contains(&(j as usize)) {
                        sets.join(i as u32, j);
                    }
                }
            }
            let mut size: std::collections::BTreeMap<u32, usize> =
                std::collections::BTreeMap::new();
            for &i in members {
                *size.entry(sets.find(i as u32)).or_default() += 1;
            }
            let area = cell_m * cell_m;
            let widest = size
                .values()
                .map(|&c| c as f64 * area / f64::from(BAND_M as f32))
                .fold(0.0, f64::max);
            // Fronts of a single cell are speckle at a band edge, not a way on.
            let fronts = size.values().filter(|&&c| c as f64 >= MIN_FRONT_M).count();
            if fronts >= 2 || widest >= OPEN_WIDTH_M {
                depth = b as f64 * f64::from(BAND_M as f32);
                break;
            }
        }
        // The pocket behind the place: the cells that are not alive and touch its first few metres, with
        // the cells that join them through cells that are not alive either.
        let mut in_pocket = vec![false; n];
        let mut stack: Vec<usize> = (0..n)
            .filter(|&i| !alive[i] && f64::from(dist[order[i] as usize]) <= POCKET_SEED_M)
            .collect();
        for &i in &stack {
            in_pocket[i] = true;
        }
        let mut tip_m = 0.0_f64;
        while let Some(i) = stack.pop() {
            tip_m = tip_m.max(f64::from(dist[order[i] as usize]));
            let cell = order[i] as usize;
            for (dx, dy) in STEPS {
                let Some(next) = step_ok(cell, dx, dy) else {
                    continue;
                };
                let j = pos[next];
                if j != u32::MAX && !alive[j as usize] && !in_pocket[j as usize] {
                    in_pocket[j as usize] = true;
                    stack.push(j as usize);
                }
            }
        }
        Some(DeadEnd {
            depth_m: depth,
            tip_m,
        })
    }
}

#[cfg(test)]
mod tests;
