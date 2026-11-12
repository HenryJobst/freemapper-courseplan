//! Dead ends and the ends of a course: the rules of the practical sprint (ADR 0085, ADR 0086, ADR 0087).
//!
//! The check and the generation read the same functions, so a draft never holds a control that `check` warns
//! about, and never starts or finishes where `check` objects.

use freemapper_courseplan_model::{MapPoint, ScoreSettings};

use crate::map::MapIndex;
use crate::mapped::MappedArea;
use crate::score::ScoreError;
use crate::score::travel::{CostSurface, DeadEnd, Pace, RouteParams};

/// The walkable network of a map, analysed for dead ends (ADR 0085). Built from the map objects alone: out-of-bounds
/// special objects of an event do not change it, so the check of a manual course and the draft of a generated one see
/// the same pockets.
pub struct DeadEnds {
    surface: CostSurface,
}

impl std::fmt::Debug for DeadEnds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeadEnds")
    }
}

impl DeadEnds {
    /// The analysis of the map at scale 1:`scale`, on the fine raster of the cost surface.
    pub fn build(index: &MapIndex, scale: u32) -> Result<DeadEnds, ScoreError> {
        let surface = CostSurface::build(index, scale, &[], Pace(250), &RouteParams::default())?;
        Ok(DeadEnds { surface })
    }

    /// How far into a dead end `at` lies, judged against `limit_m`; `None` when nothing can be told there.
    pub fn depth(&self, at: &MapPoint, limit_m: f64) -> Option<DeadEnd> {
        self.surface.dead_end(at, limit_m)
    }

    /// The depth of `at` when it is more than `limit_m` (whole metres of the setting), else `None`.
    pub fn too_deep(&self, at: &MapPoint, limit_m: u32) -> Option<DeadEnd> {
        self.depth(at, f64::from(limit_m))
            .filter(|d| d.depth_m > f64::from(limit_m))
    }
}

/// Distance in metres on the ground between two points in map millimetres at scale 1:`scale`.
pub fn ground_metres(a: &MapPoint, b: &MapPoint, scale: u32) -> f64 {
    (a.x - b.x).hypot(a.y - b.y) * f64::from(scale) / 1000.0
}

/// Refuses a start and finish a draft cannot start from: further apart than `start_finish_max_m` (ADR 0086), or closer
/// than `map_margin_m` to the edge of the mapped area (ADR 0087). `mapped` is none for a map without objects, which
/// has no edge to measure against.
pub fn check_ends(
    start: &MapPoint,
    finish: &MapPoint,
    scale: u32,
    settings: &ScoreSettings,
    mapped: Option<&MappedArea>,
) -> Result<(), ScoreError> {
    let metres = ground_metres(start, finish, scale);
    if metres > f64::from(settings.start_finish_max_m) {
        return Err(ScoreError::StartFinishTooFar {
            start: start.clone(),
            finish: finish.clone(),
            metres,
            limit_m: settings.start_finish_max_m,
        });
    }
    if let Some(area) = mapped {
        for (kind, at) in [("start", start), ("finish", finish)] {
            let margin = area.margin_m(at);
            if margin < f64::from(settings.map_margin_m) {
                return Err(ScoreError::EndTooCloseToMapEdge {
                    kind,
                    at: at.clone(),
                    metres: margin,
                    limit_m: settings.map_margin_m,
                });
            }
        }
    }
    Ok(())
}
