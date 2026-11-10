//! Starting an event from a map: `freemapper-courseplan init`.

use std::path::{Component, Path, PathBuf};

use freemapper_courseplan_model::{
    Discipline, Event, EventMeta, FORMAT_VERSION, ProjectError, ProjectFile, Sources,
};
use thiserror::Error;

use crate::map::{MapKind, map_kind_for_file};
use crate::omap::OmapError;
use crate::omap::read::read_map_file;

/// Why an event could not be started.
#[derive(Debug, Error)]
pub enum InitError {
    #[error("cannot read map {}: {source}", path.display())]
    Map { path: PathBuf, source: OmapError },
    #[error(
        "Forest maps (ISOM) are not supported yet; this version handles sprint and schoolyard maps (ADR 0010)."
    )]
    ForestNotSupported,
    #[error(transparent)]
    Project(#[from] ProjectError),
    #[error("cannot create folder {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Builds the first version of an event for `map_path`.
///
/// The title is `title`, else the `title` key of a `freemapper.toml` next to the map, else the
/// map file name without extension. `sources.map` is written relative to the folder of
/// `project_path`, with forward slashes.
pub fn new_event(
    map_path: &Path,
    project_path: &Path,
    title: Option<String>,
) -> Result<Event, InitError> {
    let map_error = |source: OmapError| InitError::Map {
        path: map_path.to_owned(),
        source,
    };
    let doc = read_map_file(map_path).map_err(map_error)?;
    if map_kind_for_file(&doc, map_path) == MapKind::Forest {
        return Err(InitError::ForestNotSupported);
    }
    let map_abs = map_path
        .canonicalize()
        .map_err(|e| map_error(OmapError::Io(e)))?;
    let map_dir = map_abs.parent().unwrap_or(Path::new(""));
    let title = title
        .or_else(|| title_from_config(map_dir))
        .unwrap_or_else(|| {
            map_abs
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        });
    let project_dir = absolute_dir(project_path.parent().unwrap_or(Path::new("")));
    Ok(Event {
        format: FORMAT_VERSION,
        event: EventMeta {
            title,
            date: None,
            discipline: Discipline::Training,
            rule_profile: "wkb-2026".to_owned(),
            rule_overlays: Vec::new(),
            start_window_min: None,
        },
        sources: Sources {
            map: relative_path(&project_dir, &map_abs),
            ppen: None,
        },
        classes: Vec::new(),
        controls: Vec::new(),
        specials: Vec::new(),
        courses: Vec::new(),
        accepted_findings: Vec::new(),
        sync: None,
    })
}

/// Starts an event: builds it with [`new_event`], creates the project folder if needed and
/// writes the project file. Nothing is written when an earlier step fails.
pub fn init_project(
    map_path: &Path,
    project_path: &Path,
    title: Option<String>,
) -> Result<ProjectFile, InitError> {
    let event = new_event(map_path, project_path, title)?;
    create_project(project_path, event)
}

/// Creates the project folder if needed and writes `event` as a new project file at `project_path`.
/// Nothing is left behind when the event is invalid or the file exists: a folder this call created is
/// removed again.
pub fn create_project(project_path: &Path, event: Event) -> Result<ProjectFile, InitError> {
    let mut created: Option<PathBuf> = None;
    if let Some(dir) = project_path.parent().filter(|d| !d.as_os_str().is_empty()) {
        // The topmost folder that does not exist yet; removing it undoes `create_dir_all`.
        created = dir
            .ancestors()
            .take_while(|d| !d.as_os_str().is_empty() && !d.exists())
            .last()
            .map(Path::to_owned);
        std::fs::create_dir_all(dir).map_err(|source| InitError::Io {
            path: dir.to_owned(),
            source,
        })?;
    }
    ProjectFile::create(project_path, event).map_err(|e| {
        if let Some(dir) = created {
            let _ = std::fs::remove_dir_all(dir);
        }
        e.into()
    })
}

/// `target` as a path relative to the folder of `project_path` (forward slashes where the platform
/// allows), for a link stored in the project file such as `sources.ppen`. `None` when `target` does
/// not exist.
pub fn link_from_project(project_path: &Path, target: &Path) -> Option<PathBuf> {
    let abs = target.canonicalize().ok()?;
    let dir = absolute_dir(project_path.parent().unwrap_or(Path::new("")));
    Some(relative_path(&dir, &abs))
}

/// The `title` key of `freemapper.toml` in `dir`. A missing or malformed file counts as none.
fn title_from_config(dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(dir.join("freemapper.toml")).ok()?;
    let table: toml::Table = text.parse().ok()?;
    table.get("title")?.as_str().map(str::to_owned)
}

/// Absolute form of a folder that may not exist yet: the nearest existing ancestor is
/// canonicalised and the missing parts are appended.
fn absolute_dir(dir: &Path) -> PathBuf {
    let dir = if dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        dir
    };
    let mut missing = Vec::new();
    let mut current = dir.to_owned();
    loop {
        if let Ok(base) = current.canonicalize() {
            let mut result = base;
            for part in missing.iter().rev() {
                if part == ".." {
                    result.pop();
                } else if part != "." {
                    result.push(part);
                }
            }
            return result;
        }
        match (current.file_name(), current.parent()) {
            (Some(name), Some(parent)) => {
                missing.push(name.to_owned());
                current = parent.to_owned();
            }
            _ => return dir.to_owned(),
        }
        if current.as_os_str().is_empty() {
            current = PathBuf::from(".");
        }
    }
}

/// `target` relative to the folder `from`, with forward slashes. Both must be absolute and
/// normalised. Without a relative path (another drive), the absolute `target` is returned.
fn relative_path(from: &Path, target: &Path) -> PathBuf {
    let from: Vec<_> = from.components().collect();
    let to: Vec<_> = target.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    if common == 0 || matches!(from.first(), Some(Component::Prefix(_))) && common < 2 {
        return slashed(target);
    }
    let mut parts: Vec<String> = Vec::new();
    parts.extend(from[common..].iter().map(|_| "..".to_owned()));
    parts.extend(
        to[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    PathBuf::from(parts.join("/"))
}

fn slashed(path: &Path) -> PathBuf {
    PathBuf::from(path.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_to_sibling_and_parent() {
        let map = Path::new("/a/b/luebeck.omap");
        assert_eq!(
            relative_path(Path::new("/a/b"), map),
            PathBuf::from("luebeck.omap")
        );
        assert_eq!(
            relative_path(Path::new("/a/b/ev"), map),
            PathBuf::from("../luebeck.omap")
        );
        assert_eq!(
            relative_path(Path::new("/a/c/d"), map),
            PathBuf::from("../../b/luebeck.omap")
        );
        assert_eq!(
            relative_path(Path::new("/a"), map),
            PathBuf::from("b/luebeck.omap")
        );
    }

    #[test]
    fn absolute_dir_handles_missing_folders() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        assert_eq!(absolute_dir(&base.join("x/y")), base.join("x").join("y"));
        assert_eq!(
            absolute_dir(&base.join("../x")),
            base.parent().unwrap().join("x")
        );
    }

    #[test]
    fn title_comes_from_config_and_ignores_garbage() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(title_from_config(tmp.path()), None);
        std::fs::write(tmp.path().join("freemapper.toml"), "title = \"Altstadt\"\n").unwrap();
        assert_eq!(title_from_config(tmp.path()).as_deref(), Some("Altstadt"));
        std::fs::write(tmp.path().join("freemapper.toml"), "title = [[[").unwrap();
        assert_eq!(title_from_config(tmp.path()), None);
    }
}
