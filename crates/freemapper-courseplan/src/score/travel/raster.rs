//! The raster of the cost surface: a speed factor per cell, painted from the map objects.

use std::collections::HashMap;

use freemapper_courseplan_model::MapPoint;

use crate::check::symbol_matches;
use crate::map::{FeatureGeometry, MapIndex};
use crate::score::ScoreError;

/// Most cells a raster may have (about 130 MB with the search arrays).
pub(super) const MAX_CELLS: u64 = 16_000_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Kind {
    /// Impassable; a crossing point on a line of it opens the cell.
    Block,
    /// Impassable whatever its geometry; nothing but a bridge opens it.
    Solid,
    /// A crossing point: opens cells blocked by a line, never a building or water area.
    Cross,
    /// A bridge: opens every blocked cell under it.
    Bridge,
    Ground(f32),
    Fast(f32),
}

/// The speed classes of `speeds.toml`: symbol code, kind. Parsed once per build.
pub(super) struct Table(pub(super) Vec<(String, Kind)>);

impl Table {
    pub(super) fn embedded() -> Table {
        let doc: toml::Table = include_str!("../speeds.toml")
            .parse()
            .expect("speeds.toml is valid TOML");
        let mut rows = Vec::new();
        for class in doc["class"].as_array().expect("speeds.toml has classes") {
            let factor = class
                .get("factor")
                .and_then(|f| f.as_float())
                .unwrap_or(1.0) as f32;
            let kind = match class["kind"].as_str().expect("class has a kind") {
                "block" => Kind::Block,
                "solid" => Kind::Solid,
                "cross" => Kind::Cross,
                "bridge" => Kind::Bridge,
                "ground" => Kind::Ground(factor),
                "fast" => Kind::Fast(factor),
                other => panic!("speeds.toml: unknown kind {other}"),
            };
            for code in class["codes"].as_array().expect("class has codes") {
                rows.push((code.as_str().expect("code is text").to_owned(), kind));
            }
        }
        Table(rows)
    }

    /// The class of the most specific (longest) code that matches `symbol`.
    pub(super) fn lookup(&self, symbol: &str) -> Option<Kind> {
        self.0
            .iter()
            .filter(|(code, _)| symbol_matches(symbol, code))
            .max_by_key(|(code, _)| code.len())
            .map(|(_, kind)| *kind)
    }
}

/// Cells of a raster over the map, row by row. A factor of infinity is a blocked cell.
pub(super) struct Grid {
    origin: (f64, f64),
    cell_mm: f64,
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) factor: Vec<f32>,
}

struct Layers {
    ground: Vec<f32>,
    fast: Vec<f32>,
    block_area: Vec<bool>,
    block_line: Vec<bool>,
    cross: Vec<bool>,
    bridge: Vec<bool>,
}

impl Grid {
    pub(super) fn build(
        index: &MapIndex,
        scale: u32,
        blocked: &[Vec<MapPoint>],
        cell_m: f64,
    ) -> Result<Grid, ScoreError> {
        let (mut lo, mut hi) = index.extent().ok_or(ScoreError::MapEmpty)?;
        for p in blocked.iter().flatten() {
            if !(p.x.is_finite() && p.y.is_finite()) {
                return Err(ScoreError::BadRouteSetting {
                    name: "blocked",
                    value: f64::NAN,
                });
            }
            lo = MapPoint {
                x: lo.x.min(p.x),
                y: lo.y.min(p.y),
            };
            hi = MapPoint {
                x: hi.x.max(p.x),
                y: hi.y.max(p.y),
            };
        }
        let cell_mm = cell_m * 1000.0 / f64::from(scale);
        let margin = 2.0 * cell_mm;
        let origin = (lo.x - margin, lo.y - margin);
        let width = ((hi.x + margin - origin.0) / cell_mm).ceil() + 1.0;
        let height = ((hi.y + margin - origin.1) / cell_mm).ceil() + 1.0;
        let cells = width * height;
        // NaN and infinity from absurd coordinates are refused too.
        if cells.is_nan() || cells > MAX_CELLS as f64 {
            return Err(ScoreError::GridTooLarge {
                cells: cells as u64,
                limit: MAX_CELLS,
            });
        }
        let (width, height) = (width as usize, height as usize);
        let n = width * height;
        let mut grid = Grid {
            origin,
            cell_mm,
            width,
            height,
            factor: Vec::new(),
        };
        let mut layers = Layers {
            ground: vec![1.0; n],
            fast: vec![f32::INFINITY; n],
            block_area: vec![false; n],
            block_line: vec![false; n],
            cross: vec![false; n],
            bridge: vec![false; n],
        };
        let table = Table::embedded();
        let mut cache: HashMap<&str, Option<Kind>> = HashMap::new();
        for feature in index.features() {
            let Some(code) = feature.symbol_code.as_deref() else {
                continue;
            };
            let Some(kind) = *cache.entry(code).or_insert_with(|| table.lookup(code)) else {
                continue;
            };
            grid.paint(&mut layers, &feature.geometry, kind);
        }
        for ring in blocked {
            // Fewer than three points is a line (a forbidden route), otherwise an area.
            let geometry = if ring.len() < 3 {
                FeatureGeometry::Line(ring.clone())
            } else {
                FeatureGeometry::Area(vec![ring.clone()])
            };
            grid.paint(&mut layers, &geometry, Kind::Solid);
        }
        grid.factor = (0..n)
            .map(|i| {
                let blocked = layers.block_area[i] || (layers.block_line[i] && !layers.cross[i]);
                if blocked && !layers.bridge[i] {
                    f32::INFINITY
                } else if layers.fast[i].is_finite() {
                    layers.fast[i]
                } else {
                    layers.ground[i]
                }
            })
            .collect();
        Ok(grid)
    }

    /// The cell index holding `p`, or `None` outside the raster.
    pub(super) fn cell_of(&self, p: &MapPoint) -> Option<usize> {
        let col = ((p.x - self.origin.0) / self.cell_mm).floor();
        let row = ((p.y - self.origin.1) / self.cell_mm).floor();
        let inside =
            col >= 0.0 && row >= 0.0 && (col as usize) < self.width && (row as usize) < self.height;
        inside.then(|| row as usize * self.width + col as usize)
    }

    pub(super) fn centre(&self, cell: usize) -> MapPoint {
        let (row, col) = (cell / self.width, cell % self.width);
        MapPoint {
            x: self.origin.0 + (col as f64 + 0.5) * self.cell_mm,
            y: self.origin.1 + (row as f64 + 0.5) * self.cell_mm,
        }
    }

    fn mark(&self, layers: &mut Layers, cell: usize, kind: Kind, line: bool) {
        match kind {
            Kind::Block if line => layers.block_line[cell] = true,
            Kind::Block | Kind::Solid => layers.block_area[cell] = true,
            Kind::Cross => layers.cross[cell] = true,
            Kind::Bridge => layers.bridge[cell] = true,
            Kind::Ground(f) => layers.ground[cell] = layers.ground[cell].max(f),
            Kind::Fast(f) => layers.fast[cell] = layers.fast[cell].min(f),
        }
    }

    fn paint(&self, layers: &mut Layers, geometry: &FeatureGeometry, kind: Kind) {
        match geometry {
            FeatureGeometry::Point(p) => {
                if let Some(cell) = self.cell_of(p) {
                    self.mark(layers, cell, kind, false);
                }
            }
            FeatureGeometry::Line(points) => self.paint_line(layers, points, kind, true),
            FeatureGeometry::Area(rings) => {
                self.fill(layers, rings, kind);
                if matches!(kind, Kind::Block | Kind::Solid) {
                    // A narrow wall or building must not vanish between cell centres.
                    for ring in rings {
                        let mut closed = ring.clone();
                        closed.extend(ring.first().cloned());
                        self.paint_line(layers, &closed, kind, false);
                    }
                }
            }
        }
    }

    /// Marks every cell a polyline touches, sampled at half a cell.
    fn paint_line(&self, layers: &mut Layers, points: &[MapPoint], kind: Kind, line: bool) {
        if let [only] = points
            && let Some(cell) = self.cell_of(only)
        {
            self.mark(layers, cell, kind, line);
        }
        for pair in points.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            let steps = ((a.x - b.x).hypot(a.y - b.y) / (self.cell_mm / 2.0))
                .ceil()
                .max(1.0) as usize;
            for k in 0..=steps {
                let t = k as f64 / steps as f64;
                let p = MapPoint {
                    x: a.x + (b.x - a.x) * t,
                    y: a.y + (b.y - a.y) * t,
                };
                if let Some(cell) = self.cell_of(&p) {
                    self.mark(layers, cell, kind, line);
                }
            }
        }
    }

    /// Fills the cells whose centre lies inside the rings (even-odd rule), row by row.
    fn fill(&self, layers: &mut Layers, rings: &[Vec<MapPoint>], kind: Kind) {
        let (mut y_lo, mut y_hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for p in rings.iter().flatten() {
            y_lo = y_lo.min(p.y);
            y_hi = y_hi.max(p.y);
        }
        if !y_lo.is_finite() {
            return;
        }
        let row_of = |y: f64| ((y - self.origin.1) / self.cell_mm - 0.5).ceil();
        let first = row_of(y_lo).max(0.0) as usize;
        let last = row_of(y_hi).min(self.height as f64) as usize;
        let mut xs: Vec<f64> = Vec::new();
        for row in first..last.min(self.height) {
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
                let to = col_of(*x1).min(self.width as f64) as usize;
                for col in from..to {
                    self.mark(layers, row * self.width + col, kind, false);
                }
            }
        }
    }
}
