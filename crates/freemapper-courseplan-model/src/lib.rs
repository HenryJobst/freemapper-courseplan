//! The event data model of freemapper-courseplan and its project file: controls, courses of all kinds
//! (individual, team, relay, score), classes and special objects.
//!
//! This crate has no map or rendering dependencies, so freemapper-print can read courseplan events
//! through it later instead of a reader of its own (design §5). Terms follow `docs/glossary.md`.

mod project_file;
mod types;
mod validate;

pub use project_file::{ModelError, ProjectError, ProjectFile, write_atomic};
pub use types::*;
pub use validate::{MAX_CODE_CHARS, MAX_NAME_CHARS, validate};
