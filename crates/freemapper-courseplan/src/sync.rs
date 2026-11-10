//! Three-way sync of a project with its linked `.ppen` (ADR 0040).

pub mod base;
pub mod decide;
pub mod fields;
pub mod matching;
pub mod merge3;
pub mod view;

use freemapper_course::KeptObject;
use freemapper_courseplan_model::ModelError;

use crate::ppen_export::PpenExportError;
use crate::ppen_import::PpenImportError;

/// Why a sync was refused.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    /// The base copy cannot be read or does not fit the map.
    #[error("the sync base cannot be imported: {0}")]
    Base(#[source] PpenImportError),
    /// The linked `.ppen` cannot be read or does not fit the map.
    #[error("the linked Purple Pen file cannot be imported: {0}")]
    Theirs(#[source] PpenImportError),
    /// The project cannot be written as a Purple Pen event.
    #[error("the project cannot be written as a Purple Pen event: {0}")]
    Export(#[from] PpenExportError),
    /// The linked file was written by a Purple Pen version the writer does not know.
    #[error("the linked Purple Pen file has an unknown version: {0}")]
    UnknownVersion(String),
    /// The writer kept objects of the linked file that the merge modeled: it did not match them.
    #[error("the writer did not match {} object(s) of the linked Purple Pen file", .0.len())]
    Unmatched(Vec<KeptObject>),
    /// The merged event is not valid.
    #[error("the merged event is not valid: {} problem(s)", .0.len())]
    Invalid(Vec<ModelError>),
}
