//! Score courses: point values, codes that carry them, and the builder's helpers.

pub mod candidates;
pub mod code;
pub mod travel;

/// Why the score code could not do what was asked. Matchable; a text per variant joins the Fluent
/// catalogs with the operations that raise it.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ScoreError {
    #[error("no route from ({}, {}) to ({}, {}) on this map", from.x, from.y, to.x, to.y)]
    Unreachable {
        from: freemapper_courseplan_model::MapPoint,
        to: freemapper_courseplan_model::MapPoint,
    },
    #[error("the start at ({}, {}) cannot be reached: it lies in a blocked area", at.x, at.y)]
    StartUnreachable {
        at: freemapper_courseplan_model::MapPoint,
    },
    #[error("the finish at ({}, {}) cannot be reached: it lies in a blocked area", at.x, at.y)]
    FinishUnreachable {
        at: freemapper_courseplan_model::MapPoint,
    },
    #[error("the map has no objects, so there is nothing to search routes on")]
    MapEmpty,
    #[error("route setting {name} is out of range: {value}")]
    BadRouteSetting { name: &'static str, value: f64 },
    #[error("the map would need {cells} raster cells, more than the limit of {limit}")]
    GridTooLarge { cells: u64, limit: u64 },
}
