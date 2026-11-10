//! The three Purple Pen views of a sync: base, ours and theirs, each as what the `.ppen` import makes of a
//! file (D3). Seen through the same reader, fit and rounding, Purple Pen's added defaults and the model's
//! lossy mapping cannot show up as changes.

use freemapper_course::KeptObject;
use freemapper_courseplan_model::Event;

use super::SyncError;
use crate::map::MapKind;
use crate::omap::MapDocument;
use crate::ppen_export::{ExportIds, ExportNote, Target, export_ppen_as};
use crate::ppen_import::{PpenImport, PpenImportError, import_ppen_as};

/// The three versions in the Purple Pen view.
#[derive(Debug, Clone, PartialEq)]
pub struct Views {
    pub base: PpenImport,
    pub ours: PpenImport,
    pub theirs: PpenImport,
    /// The Purple Pen ids the export gave the project's objects in the file `ours` was imported from.
    pub ours_ids: ExportIds,
    /// The `CourseSkipped` and `SpecialSkipped` notes of the ours export ("not written to Purple Pen").
    pub skipped: Vec<ExportNote>,
}

fn quiet(mut import: PpenImport) -> PpenImport {
    import.notes.clear();
    import
}

/// Removes an object the export kept from the ours view: the project does not hold it.
fn drop_kept(view: &mut PpenImport, kept: &KeptObject) {
    use freemapper_courseplan_model::SpecialObject;
    let is_end = |s: &SpecialObject| {
        matches!(
            s,
            SpecialObject::Start { .. } | SpecialObject::Finish { .. }
        )
    };
    match kept {
        KeptObject::Control { id, .. } => {
            view.controls.retain(|c| c.ppen_id != Some(*id));
            view.specials
                .retain(|s| !(is_end(s) && s.ppen_id() == Some(*id)));
        }
        KeptObject::Course { id, .. } => view.courses.retain(|c| c.ppen_id != Some(*id)),
        KeptObject::Special { id, .. } => view
            .specials
            .retain(|s| is_end(s) || s.ppen_id() != Some(*id)),
    }
}

/// Builds the three views. Ours is the import of the project exported onto `base` (linked, so the project's
/// Purple Pen ids count); with no base it is the import of a file made from scratch.
///
/// # Errors
/// [`SyncError::Base`] and [`SyncError::Theirs`] for a file that cannot be imported, [`SyncError::Export`]
/// when the project cannot be written.
pub fn views(
    project: &Event,
    base: Option<&[u8]>,
    theirs: &[u8],
    map: &MapDocument,
    kind: MapKind,
) -> Result<Views, SyncError> {
    let base_view = match base {
        Some(bytes) => quiet(import_ppen_as(bytes, map, kind).map_err(SyncError::Base)?),
        None => PpenImport {
            controls: Vec::new(),
            specials: Vec::new(),
            courses: Vec::new(),
            notes: Vec::new(),
        },
    };
    let theirs_view = quiet(import_ppen_as(theirs, map, kind).map_err(SyncError::Theirs)?);
    let exported = export_ppen_as(
        project,
        map,
        kind,
        Target {
            base,
            linked: true,
            // Only the ours view is read from this text. A file made from scratch needs some map path;
            // with a base the writer takes the base's.
            map_path: if base.is_some() {
                String::new()
            } else {
                "map.omap".to_owned()
            },
            lang: "en",
            delete: Default::default(),
        },
    )?;
    let mut ours_view = quiet(
        import_ppen_as(exported.text.as_bytes(), map, kind)
            .map_err(|e: PpenImportError| SyncError::Export(e.into()))?,
    );
    // An export deletes nothing: what the base holds and the project no longer mentions is still in the
    // file, but it is not in the project. In the ours view it is gone.
    for note in &exported.notes {
        if let ExportNote::Kept(kept) = note {
            drop_kept(&mut ours_view, kept);
        }
    }
    let skipped = exported
        .notes
        .into_iter()
        .filter(|n| {
            matches!(
                n,
                ExportNote::CourseSkipped { .. } | ExportNote::SpecialSkipped { .. }
            )
        })
        .collect();
    Ok(Views {
        base: base_view,
        ours: ours_view,
        theirs: theirs_view,
        ours_ids: exported.ids,
        skipped,
    })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use freemapper_courseplan_model::ProjectFile;

    use super::*;
    use crate::map::map_kind_for_file;
    use crate::omap::read::read_map_file;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn unchanged_project_views_equal_base() {
        let map_path = root().join("tests/fixtures/maps/luebeck.omap");
        let map = read_map_file(&map_path).unwrap();
        let kind = map_kind_for_file(&map, &map_path);
        let dir = root().join("examples/luebeck-sprint");
        let project = ProjectFile::load(&dir.join("luebeck-sprint.courseplan.toml")).unwrap();
        let ppen = std::fs::read(dir.join("luebeck-sprint.ppen")).unwrap();
        let v = views(&project.event, Some(&ppen), &ppen, &map, kind).unwrap();
        assert!(!v.base.controls.is_empty());
        assert_eq!(v.ours.controls, v.base.controls);
        assert_eq!(v.ours.specials, v.base.specials);
        assert_eq!(v.ours.courses, v.base.courses);
        assert_eq!(v.theirs, v.base);
    }
}
