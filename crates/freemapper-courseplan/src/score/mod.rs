//! Score courses: point values, codes that carry them, and the builder's helpers.

pub mod candidates;
pub mod code;
pub mod draft;
pub mod links;
pub mod points;
pub mod tour;
pub mod travel;

pub use draft::{Draft, DraftControl, DraftFailure, DraftRequest, Hint, draft};
pub use tour::{
    LengthIssue, LengthReport, ScoredControl, Tour, analyse_length, best_tour, best_tour_seeded,
};

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
    #[error("only {limit} free codes are left for the controls asked for")]
    CodeSpaceExhausted { limit: usize },
    #[error(
        "only {limit} free codes carry the value {value}; more controls of that value were asked for"
    )]
    ValueSpaceExhausted { value: u8, limit: usize },
    #[error("a point value must lie from 1 to {value_max}, not {value}")]
    BadValue { value: u32, value_max: u8 },
    #[error("{values} values but {coords} positions were given")]
    PositionsMismatch { values: usize, coords: usize },
    #[error("the link between controls {a} and {b} does not fit the time limit")]
    LinkDoesNotFit { a: usize, b: usize },
    #[error(
        "the link between controls {a} and {b} names a control that does not exist or the same control twice"
    )]
    BadLink { a: usize, b: usize },
    #[error("an end of the requested link {link} lies on no candidate")]
    LinkEndNotACandidate { link: usize },
    #[error(
        "the links at control {control} do not form a chain: a control may have at most two links and they may not close a ring"
    )]
    LinksNotAChain { control: usize },
    #[error(
        "no tour was found that holds every link within the time limit; the link between controls {a} and {b} fits worst. The search is not exhaustive for larger courses, so this is no proof"
    )]
    LinkTourNotFound { a: usize, b: usize },
}
