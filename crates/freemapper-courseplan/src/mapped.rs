//! The mapped area as a raster, and the distance of a place to its edge (ADR 0087).
//!
//! A place counts as mapped when a map object lies within [`MAPPED_METRES`] of it, the test of the check
//! `inside_map` (ADR 0084); the sheet frame and layout symbols are no objects. The edge of the mapped area is
//! where that stops. [`MappedArea`] paints the objects on a raster, takes the distance transform to the
//! nearest object, cuts it at [`MAPPED_METRES`] and takes the distance transform again, now to the nearest
//! place that is no longer mapped. A runner standing at a place with a margin of 50 m can run 50 m in every
//! direction and still be on mapped ground. The raster is a few metres coarse, so a margin is right to
//! about one cell; a rule and the score draft read the same function, so they agree.

use freemapper_courseplan_model::MapPoint;

use crate::check::{FRAME_PERCENT, IGNORED_SYMBOLS, terrain_features};
use crate::map::{FeatureGeometry, MapIndex};

/// A place is mapped when a map object lies within this many metres of it. The value of the rule
/// `wkb2026.map.inside`; a test compares both.
pub const MAPPED_METRES: f64 = 50.0;

/// Edge of a raster cell in metres on the ground.
const CELL_M: f64 = 2.0;
/// Most cells the raster may have; a larger map gets coarser cells.
const MAX_CELLS: f64 = 6_000_000.0;
/// Margins beyond this many metres are not told apart (the largest setting is 200).
pub const MAX_MARGIN_M: f64 = 250.0;

/// The raster of the mapped area with the distance of each mapped cell to the edge.
pub struct MappedArea {
    origin: (f64, f64),
    cell_mm: f64,
    cell_m: f64,
    width: usize,
    height: usize,
    /// Distance in metres to the nearest unmapped cell; zero for an unmapped cell.
    margin: Vec<f32>,
}

impl std::fmt::Debug for MappedArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MappedArea({}x{} cells of {} m)",
            self.width, self.height, self.cell_m
        )
    }
}

impl MappedArea {
    /// The mapped area of the terrain objects of `index` at map scale 1:`scale`, or `None` when there are
    /// none (a map with no objects, or with layout symbols only).
    pub fn build(index: &MapIndex, scale: u32) -> Option<MappedArea> {
        Self::build_with_reach(index, scale, MAPPED_METRES)
    }

    /// Like [`MappedArea::build`], with a place counting as mapped when an object lies within `reach_m`
    /// metres of it. A forest map takes its reach from the sheet (`wkb2026.map.inside`, ADR 0092).
    pub fn build_with_reach(index: &MapIndex, scale: u32, reach_m: f64) -> Option<MappedArea> {
        let features = terrain_features(index, IGNORED_SYMBOLS, FRAME_PERCENT);
        if features.is_empty() || scale == 0 {
            return None;
        }
        let (mut lo, mut hi) = (
            MapPoint {
                x: f64::MAX,
                y: f64::MAX,
            },
            MapPoint {
                x: f64::MIN,
                y: f64::MIN,
            },
        );
        let mut take = |p: &MapPoint| {
            lo.x = lo.x.min(p.x);
            lo.y = lo.y.min(p.y);
            hi.x = hi.x.max(p.x);
            hi.y = hi.y.max(p.y);
        };
        for f in &features {
            match &f.geometry {
                FeatureGeometry::Point(p) => take(p),
                FeatureGeometry::Line(l) => l.iter().for_each(&mut take),
                FeatureGeometry::Area(rings) => rings.iter().flatten().for_each(&mut take),
            }
        }
        if !(lo.x.is_finite() && hi.x.is_finite() && lo.y.is_finite() && hi.y.is_finite()) {
            return None;
        }
        let mm_per_m = 1000.0 / f64::from(scale);
        let pad_mm = (reach_m + MAX_MARGIN_M + 2.0 * CELL_M) * mm_per_m;
        let span = ((hi.x - lo.x) + 2.0 * pad_mm, (hi.y - lo.y) + 2.0 * pad_mm);
        let mut cell_m = CELL_M;
        while (span.0 / (cell_m * mm_per_m)) * (span.1 / (cell_m * mm_per_m)) > MAX_CELLS {
            cell_m *= 1.5;
        }
        let cell_mm = cell_m * mm_per_m;
        let width = (span.0 / cell_mm).ceil() as usize + 1;
        let height = (span.1 / cell_mm).ceil() as usize + 1;
        let mut area = MappedArea {
            origin: (lo.x - pad_mm, lo.y - pad_mm),
            cell_mm,
            cell_m,
            width,
            height,
            margin: Vec::new(),
        };
        let mut object = vec![false; width * height];
        for f in &features {
            match &f.geometry {
                FeatureGeometry::Point(p) => area.mark(&mut object, p),
                FeatureGeometry::Line(l) => area.paint_line(&mut object, l),
                FeatureGeometry::Area(rings) => {
                    area.fill(&mut object, rings);
                    for ring in rings {
                        let mut closed = ring.clone();
                        closed.extend(ring.first().cloned());
                        area.paint_line(&mut object, &closed);
                    }
                }
            }
        }
        // Distance to the nearest object, in cells; mapped is within the reach.
        let to_object = squared_distance(&object, width, height);
        let reach_cells = reach_m / cell_m;
        let unmapped: Vec<bool> = to_object
            .iter()
            .map(|&d2| d2.sqrt() > reach_cells)
            .collect();
        let to_edge = squared_distance(&unmapped, width, height);
        area.margin = to_edge
            .iter()
            .zip(&unmapped)
            .map(|(&d2, &out)| {
                if out {
                    0.0
                } else {
                    // The edge runs half a cell before the centre of the first unmapped cell.
                    ((d2.sqrt() - 0.5).max(0.0) * cell_m) as f32
                }
            })
            .collect();
        Some(area)
    }

    fn cell_of(&self, p: &MapPoint) -> Option<usize> {
        let col = ((p.x - self.origin.0) / self.cell_mm).floor();
        let row = ((p.y - self.origin.1) / self.cell_mm).floor();
        (col >= 0.0 && row >= 0.0 && (col as usize) < self.width && (row as usize) < self.height)
            .then(|| row as usize * self.width + col as usize)
    }

    fn mark(&self, object: &mut [bool], p: &MapPoint) {
        if let Some(c) = self.cell_of(p) {
            object[c] = true;
        }
    }

    fn paint_line(&self, object: &mut [bool], points: &[MapPoint]) {
        if let [only] = points {
            self.mark(object, only);
        }
        for pair in points.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            let steps = ((a.x - b.x).hypot(a.y - b.y) / (self.cell_mm / 2.0))
                .ceil()
                .max(1.0) as usize;
            for k in 0..=steps {
                let t = k as f64 / steps as f64;
                self.mark(
                    object,
                    &MapPoint {
                        x: a.x + (b.x - a.x) * t,
                        y: a.y + (b.y - a.y) * t,
                    },
                );
            }
        }
    }

    /// Fills the cells whose centre lies inside the rings (even-odd rule).
    fn fill(&self, object: &mut [bool], rings: &[Vec<MapPoint>]) {
        let (mut y0, mut y1) = (f64::MAX, f64::MIN);
        for p in rings.iter().flatten() {
            y0 = y0.min(p.y);
            y1 = y1.max(p.y);
        }
        if y0 > y1 {
            return;
        }
        let row_of = |y: f64| ((y - self.origin.1) / self.cell_mm - 0.5).ceil();
        let first = row_of(y0).max(0.0) as usize;
        let last = (row_of(y1).max(0.0) as usize).min(self.height);
        let mut xs: Vec<f64> = Vec::new();
        for row in first..last {
            let y = self.origin.1 + (row as f64 + 0.5) * self.cell_mm;
            xs.clear();
            for ring in rings {
                for (i, a) in ring.iter().enumerate() {
                    let b = &ring[(i + 1) % ring.len()];
                    if (a.y <= y) != (b.y <= y) {
                        xs.push(a.x + (y - a.y) / (b.y - a.y) * (b.x - a.x));
                    }
                }
            }
            xs.sort_by(f64::total_cmp);
            for pair in xs.chunks(2) {
                let [x0, x1] = pair else { continue };
                let col_of = |x: f64| ((x - self.origin.0) / self.cell_mm - 0.5).ceil();
                let from = col_of(*x0).max(0.0) as usize;
                let to = (col_of(*x1).max(0.0) as usize).min(self.width);
                for col in from..to {
                    object[row * self.width + col] = true;
                }
            }
        }
    }

    /// Metres from `at` to the nearest place that is no longer mapped: zero in the blank part of the sheet,
    /// and at least [`MAX_MARGIN_M`] deep inside. The edge of the sheet counts as the edge of the mapped
    /// area, since the raster ends beyond it.
    pub fn margin_m(&self, at: &MapPoint) -> f64 {
        self.cell_of(at)
            .map_or(0.0, |c| f64::from(self.margin[c]))
            .min(MAX_MARGIN_M)
    }
}

/// Squared distance in cells from each cell to the nearest cell where `seed` is true (zero on a seed),
/// exact, by the two-pass transform of Felzenszwalb and Huttenlocher. Without a seed every value is huge.
fn squared_distance(seed: &[bool], width: usize, height: usize) -> Vec<f64> {
    const FAR: f64 = 1.0e18;
    let mut grid: Vec<f64> = seed.iter().map(|&s| if s { 0.0 } else { FAR }).collect();
    let mut column = vec![0.0_f64; height];
    for col in 0..width {
        for row in 0..height {
            column[row] = grid[row * width + col];
        }
        let out = transform_1d(&column);
        for row in 0..height {
            grid[row * width + col] = out[row];
        }
    }
    for row in 0..height {
        let out = transform_1d(&grid[row * width..(row + 1) * width]);
        grid[row * width..(row + 1) * width].copy_from_slice(&out);
    }
    grid
}

/// Lower envelope of the parabolas `(q - x)^2 + f[q]`.
fn transform_1d(f: &[f64]) -> Vec<f64> {
    let n = f.len();
    let mut v = vec![0_usize; n];
    let mut z = vec![0.0_f64; n + 1];
    let mut k = 0;
    z[0] = f64::NEG_INFINITY;
    z[1] = f64::INFINITY;
    let at = |i: usize| i as f64;
    for q in 1..n {
        let mut s;
        loop {
            let p = v[k];
            s = ((f[q] + at(q) * at(q)) - (f[p] + at(p) * at(p))) / (2.0 * at(q) - 2.0 * at(p));
            if s <= z[k] {
                k -= 1;
            } else {
                break;
            }
        }
        k += 1;
        v[k] = q;
        z[k] = s;
        z[k + 1] = f64::INFINITY;
    }
    let mut out = vec![0.0_f64; n];
    let mut k = 0;
    for (q, slot) in out.iter_mut().enumerate() {
        while z[k + 1] < at(q) {
            k += 1;
        }
        let d = at(q) - at(v[k]);
        *slot = d * d + f[v[k]];
    }
    out
}

#[cfg(test)]
mod tests;
