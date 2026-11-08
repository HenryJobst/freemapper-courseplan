//! The project file: an event stored as TOML, loaded and validated.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{Event, validate};

/// An inconsistency inside an event.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ModelError {
    #[error("control {0} is defined more than once")]
    DuplicateControl(crate::ControlCode),
    #[error("course {0} is defined more than once")]
    DuplicateCourse(String),
    #[error("start or finish id {0} is used more than once")]
    DuplicateSpecialId(String),
    #[error("course {course} uses control {code}, which is not defined")]
    UnknownControl {
        course: String,
        code: crate::ControlCode,
    },
    #[error("course {course} has start {id}, which is not defined")]
    UnknownStart { course: String, id: String },
    #[error("course {course} has finish {id}, which is not defined")]
    UnknownFinish { course: String, id: String },
    #[error("course {course} has team size {team_size}, at least 2 is needed")]
    TeamTooSmall { course: String, team_size: u8 },
    #[error("course {course} lists control {code} as free and as mandatory")]
    FreeControlIsMandatory {
        course: String,
        code: crate::ControlCode,
    },
    #[error("project file format {0} is newer than this program supports")]
    UnsupportedFormat(u32),
}

fn list_errors(errors: &[ModelError]) -> String {
    errors
        .iter()
        .map(|e| format!("\n  {e}"))
        .collect::<String>()
}

/// Why a project file could not be loaded.
#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("cannot read {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// `message` already names the file.
    #[error("{message}")]
    Parse { path: PathBuf, message: String },
    #[error("{} is not a consistent event:{}", path.display(), list_errors(errors))]
    Invalid {
        path: PathBuf,
        errors: Vec<ModelError>,
    },
}

/// A loaded project file: the event and the parsed document it came from.
#[derive(Debug, Clone)]
pub struct ProjectFile {
    pub path: PathBuf,
    pub event: Event,
    /// Kept for writing changes back without losing comments and layout.
    #[allow(dead_code)]
    document: toml_edit::DocumentMut,
}

impl ProjectFile {
    /// Reads, parses and validates the project file at `path`.
    pub fn load(path: &Path) -> Result<ProjectFile, ProjectError> {
        let text = std::fs::read_to_string(path).map_err(|source| ProjectError::Io {
            path: path.to_owned(),
            source,
        })?;
        let parse_error = |message: String| ProjectError::Parse {
            path: path.to_owned(),
            message: format!("cannot parse {}: {message}", path.display()),
        };
        let document = text
            .parse::<toml_edit::DocumentMut>()
            .map_err(|e| parse_error(e.to_string()))?;
        let event: Event = toml_edit::de::from_document(document.clone())
            .map_err(|e| parse_error(e.to_string()))?;
        let errors = validate(&event);
        if !errors.is_empty() {
            return Err(ProjectError::Invalid {
                path: path.to_owned(),
                errors,
            });
        }
        Ok(ProjectFile {
            path: path.to_owned(),
            event,
            document,
        })
    }

    /// The map file: `sources.map`, relative to the folder of the project file unless absolute.
    pub fn map_path(&self) -> PathBuf {
        let folder = self.path.parent().unwrap_or_else(|| Path::new(""));
        folder.join(&self.event.sources.map)
    }
}
