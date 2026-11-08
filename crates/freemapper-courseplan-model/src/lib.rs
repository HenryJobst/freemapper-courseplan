//! The event data model of freemapper-courseplan and its project file: controls, courses of all kinds
//! (individual, team, relay, score), classes and special objects.
//!
//! This crate has no map or rendering dependencies, so freemapper-print can read courseplan events
//! through it later instead of a reader of its own (design §5). Terms follow `docs/glossary.md`.

mod types;

pub use types::*;
