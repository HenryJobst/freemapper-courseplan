//! freemapper-courseplan sets orienteering courses on an OOM map: controls, courses, control
//! descriptions, number positions and checks, with exchange to Purple Pen and IOF XML 3.0.
//!
//! All domain logic lives in this library and in `freemapper-courseplan-model`. The binary crate
//! `freemapper-courseplan-cli` holds the CLI, the MCP server and the course editor (ADR 0001). Terms
//! follow `docs/glossary.md`.

pub mod check;
pub mod course;
pub mod edit;
pub mod init;
pub mod iof_xml;
pub mod map;
pub mod numbers;
pub mod omap;
pub mod ppen_import;
pub mod rules;
pub mod text;
