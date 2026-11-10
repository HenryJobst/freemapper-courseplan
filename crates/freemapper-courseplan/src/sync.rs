//! Three-way sync of a project with its linked `.ppen` (ADR 0040).

pub mod apply;
pub mod base;
pub mod decide;
pub mod fields;
pub mod matching;
pub mod merge3;
pub mod view;

use std::collections::{HashMap, HashSet};

use freemapper_course::{KeptObject, PatchOptions, PpenWriteError, write_ppen_patch};
use freemapper_courseplan_model::{Event, ModelError};

use self::apply::{Applied, SyncNote, apply};
use self::decide::{SyncConflict, Take, conflict_key, conflicts, render, resolve};
use self::fields::{FieldKey, FieldOutcome, FieldValue, ObjectKind, classify};
use self::matching::{Matching, match_sides, match_sides_with};
use self::merge3::Pick;
use self::view::{Views, views};
use crate::map::MapKind;
use crate::omap::MapDocument;
use crate::ppen_export::{ExportNote, Target, export_ppen_as, store_ids};
use crate::ppen_import::import_ppen_as;

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
    /// The written file does not read back as the merged project.
    #[error("the written Purple Pen file does not read back as the merged project: {}", .0.join(", "))]
    Verify(Vec<String>),
}

/// What a sync works on.
#[derive(Debug, Clone)]
pub struct SyncInput<'a> {
    pub project: &'a Event,
    /// The base copy (D1); `None` for a sync without a base.
    pub base: Option<&'a [u8]>,
    /// The linked `.ppen` as it is now.
    pub theirs: &'a [u8],
    pub map: &'a MapDocument,
    pub kind: MapKind,
    /// The sides the caller picks (`--take`).
    pub take: &'a [Take],
    /// A note the caller adds, such as `BaseIgnored`.
    pub base_note: Option<SyncNote>,
}

/// One change the sync makes, or would make.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncChange {
    pub key: String,
    /// Whose change it is: `Ours` goes to the `.ppen`, `Theirs` into the project, `Both` into both.
    pub side: Pick,
    pub value: String,
}

/// What a sync found.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SyncReport {
    pub changes: Vec<SyncChange>,
    /// The conflicts without a side.
    pub conflicts: Vec<SyncConflict>,
    pub notes: Vec<SyncNote>,
    /// Stored decisions that no longer fit.
    pub stale: Vec<String>,
    /// The keys of every conflict of this run, those settled by a decision or a take included.
    pub conflict_keys: Vec<String>,
}

/// The outcome of [`sync_ppen`].
// One plan per sync; boxing would only change the interface.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum SyncPlan {
    /// Conflicts without a side remain: nothing is written.
    Conflicts(SyncReport),
    Ready {
        /// The merged project, with the Purple Pen ids of the written file.
        project: Event,
        /// The new text of the `.ppen`; `None` when it equals the linked file.
        ppen: Option<String>,
        /// The new base copy: the text of the `.ppen` after the sync.
        base: String,
        report: SyncReport,
    },
}

/// What the sync changes: Purple Pen's and the project's own changes, what both sides changed the same way,
/// and the conflicts with the side picked for them.
fn changes(
    outcomes: &[FieldOutcome],
    sides: &HashMap<String, freemapper_courseplan_model::Side>,
    has_base: bool,
    v: &Views,
    m: &Matching,
) -> Vec<SyncChange> {
    use freemapper_courseplan_model::Side;
    outcomes
        .iter()
        .filter_map(|o| {
            let key = || conflict_key(o, v, m);
            let (side, shown) = match o.pick {
                Pick::Ours => (Pick::Ours, o.ours.as_ref()),
                Pick::Theirs => (Pick::Theirs, o.theirs.as_ref()),
                // Without a base everything both sides hold alike would count; it is no change.
                Pick::Both if !has_base => return None,
                Pick::Both => (Pick::Both, o.merged.as_ref().or(o.ours.as_ref())),
                Pick::Conflict => match sides.get(&key())? {
                    Side::Ours => (Pick::Ours, o.ours.as_ref()),
                    Side::Theirs => (Pick::Theirs, o.theirs.as_ref()),
                },
                _ => return None,
            };
            Some(SyncChange {
                key: key(),
                side,
                value: render(shown, v, m),
            })
        })
        .collect()
}

/// The course-controls and courses of a `.ppen` as written, whatever the import makes of them: the links
/// of every course-control (`next` and the branches of a variation), each course's name and first
/// course-control.
#[derive(Default)]
struct Chains {
    /// Course-control id to (control id, linked course-control ids).
    cc: std::collections::HashMap<u32, (u32, Vec<u32>)>,
    /// Course id, name and first course-control, in file order.
    courses: Vec<(Option<u32>, String, Option<u32>)>,
}

fn scan_chains(theirs: &[u8]) -> Chains {
    use quick_xml::XmlVersion;
    use quick_xml::events::{BytesStart, Event as Xml};
    fn num(e: &BytesStart<'_>, name: &str) -> Option<u32> {
        e.try_get_attribute(name).ok().flatten().and_then(|a| {
            a.normalized_value(XmlVersion::Implicit1_0)
                .ok()?
                .trim()
                .parse()
                .ok()
        })
    }
    let text = String::from_utf8_lossy(theirs);
    let mut reader = quick_xml::Reader::from_str(&text);
    let mut out = Chains::default();
    let mut in_course = false;
    let mut in_name = false;
    let mut cur_cc: Option<u32> = None;
    while let Ok(event) = reader.read_event() {
        let (e, empty) = match &event {
            Xml::Start(e) => (Some(e), false),
            Xml::Empty(e) => (Some(e), true),
            _ => (None, false),
        };
        if let Some(e) = e {
            match e.name().as_ref() {
                "course" => {
                    in_course = !empty;
                    out.courses.push((num(e, "id"), String::new(), None));
                }
                "name" if in_course => in_name = !empty,
                "first" if in_course => {
                    if let Some(c) = out.courses.last_mut() {
                        c.2 = num(e, "course-control");
                    }
                }
                "course-control" => {
                    if let (Some(id), Some(control)) = (num(e, "id"), num(e, "control")) {
                        out.cc.insert(id, (control, Vec::new()));
                        cur_cc = (!empty).then_some(id);
                    }
                }
                _ => {
                    if let (Some(id), Some(link)) = (cur_cc, num(e, "course-control"))
                        && let Some(entry) = out.cc.get_mut(&id)
                    {
                        entry.1.push(link);
                    }
                }
            }
        }
        match &event {
            Xml::Text(t) if in_name => {
                if let Some(c) = out.courses.last_mut() {
                    let t = t.xml_content(XmlVersion::Implicit1_0);
                    c.1.push_str(t.trim());
                }
            }
            Xml::End(e) => match e.name().as_ref() {
                "course" => in_course = false,
                "name" => in_name = false,
                "course-control" => cur_cc = None,
                _ => {}
            },
            Xml::Eof => break,
            _ => {}
        }
    }
    out
}

/// Of the controls with the Purple Pen ids `ids`, those that any course of the linked file visits, with the
/// first such course (D11). The scan reads the file itself: the import skips variations and score courses,
/// so `apply` cannot see them.
fn visited_in_file(
    theirs: &[u8],
    ids: &[u32],
    modeled: &[u32],
) -> Result<Vec<(u32, String, String)>, SyncError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let raw = crate::ppen_import::read(theirs).map_err(SyncError::Theirs)?;
    let chains = scan_chains(theirs);
    let mut visited: Vec<(u32, String)> = Vec::new();
    for (course_id, name, first) in &chains.courses {
        // A course the import models is rewritten by this sync, so its old stops do not count. It is
        // matched by its Purple Pen id: a skipped course may carry the name of a modeled one.
        if course_id.is_some_and(|i| modeled.contains(&i)) {
            continue;
        }
        let mut seen = HashSet::new();
        let mut todo: Vec<u32> = first.iter().copied().collect();
        while let Some(cc) = todo.pop() {
            if !seen.insert(cc) {
                continue;
            }
            if let Some((control, links)) = chains.cc.get(&cc) {
                if !visited.iter().any(|(c, _)| c == control) {
                    visited.push((*control, name.clone()));
                }
                todo.extend(links);
            }
        }
    }
    Ok(raw
        .controls
        .iter()
        .filter_map(|c| {
            let id = c.id.filter(|i| ids.contains(i))?;
            let (_, course) = visited.iter().find(|(v, _)| *v == id)?;
            Some((id, c.code.clone(), course.clone()))
        })
        .collect())
}

/// The kept objects of the writer that the theirs import modeled, except the controls kept on purpose.
/// The writer keeps what the event does not mention; a modeled object among them means that it did not
/// match the event's object to it (D8).
fn unmatched<'a>(
    kept: impl Iterator<Item = &'a KeptObject>,
    v: &Views,
    on_purpose: &HashSet<u32>,
) -> Vec<KeptObject> {
    use freemapper_courseplan_model::SpecialObject;
    let is_end = |s: &SpecialObject| {
        matches!(
            s,
            SpecialObject::Start { .. } | SpecialObject::Finish { .. }
        )
    };
    kept.filter(|k| match k {
        KeptObject::Control { id, .. } => {
            !on_purpose.contains(id)
                && (v.theirs.controls.iter().any(|c| c.ppen_id == Some(*id))
                    || v.theirs
                        .specials
                        .iter()
                        .any(|s| is_end(s) && s.ppen_id() == Some(*id)))
        }
        KeptObject::Course { id, .. } => v.theirs.courses.iter().any(|c| c.ppen_id == Some(*id)),
        KeptObject::Special { id, .. } => v
            .theirs
            .specials
            .iter()
            .any(|s| !is_end(s) && s.ppen_id() == Some(*id)),
    })
    .cloned()
    .collect()
}

/// The keys of the number placements taken from Purple Pen that the written file does not show again.
fn numbers_lost(
    outcomes: &[FieldOutcome],
    sides: &HashMap<String, freemapper_courseplan_model::Side>,
    names: &apply::AppliedNames,
    read_back: &crate::ppen_import::PpenImport,
    v: &Views,
    m: &Matching,
) -> Vec<String> {
    use freemapper_courseplan_model::{CourseKind, Side};
    let mut courses: HashMap<&str, usize> = HashMap::new();
    for (i, c) in read_back.courses.iter().enumerate() {
        courses.entry(c.name.as_str()).or_insert(i);
    }
    let mut lost = Vec::new();
    for o in outcomes {
        let FieldKey::Course(cid, fields::CourseField::Number { control, visit }) = o.key else {
            continue;
        };
        let taken = match o.pick {
            Pick::Theirs => true,
            Pick::Conflict => sides.get(&conflict_key(o, v, m)) == Some(&Side::Theirs),
            _ => false,
        };
        let Some(FieldValue::Number(expected)) = o.theirs.as_ref().filter(|_| taken) else {
            continue;
        };
        let (Some(Some(course)), Some(Some(code))) =
            (names.courses.get(cid), names.controls.get(control))
        else {
            continue;
        };
        let shown = courses
            .get(course.as_str())
            .map(|&i| &read_back.courses[i])
            .and_then(|c| match &c.kind {
                CourseKind::Individual { controls } => controls
                    .iter()
                    .filter(|cc| &cc.code.0 == code)
                    .nth((visit as usize).saturating_sub(1))
                    .map(|cc| {
                        let central = read_back
                            .controls
                            .iter()
                            .find(|k| &k.code.0 == code)
                            .and_then(|k| k.number.as_ref());
                        cc.number.as_ref().or(central).map(|n| {
                            (
                                (n.direction * 100.0).round() as i64,
                                (n.gap_mm * 100.0).round() as i64,
                            )
                        })
                    }),
                _ => None,
            });
        if shown.is_some_and(|got| got != *expected) {
            lost.push(conflict_key(o, v, m));
        }
    }
    lost
}

/// Merges the project with the linked `.ppen` by a three-way merge (D3 to D12), writes the result onto the
/// linked file with the patch export and reads it back to check it.
///
/// # Errors
/// [`SyncError::UnknownVersion`] before anything else when the linked file's shape is unknown;
/// [`SyncError::Unmatched`] when the writer did not match an object the merge modeled; the other variants
/// for files that do not import, projects that do not export, and a result that does not read back.
pub fn sync_ppen(input: SyncInput<'_>) -> Result<SyncPlan, SyncError> {
    let SyncInput {
        project,
        base,
        theirs,
        map,
        kind,
        take,
        base_note,
    } = input;
    // D12: the shape gate of the writer, with the identity patch.
    let raw = crate::ppen_import::read(theirs).map_err(SyncError::Theirs)?;
    match write_ppen_patch(theirs, &raw, &PatchOptions::default()) {
        Ok(_) => {}
        Err(PpenWriteError::UnknownVersion(why)) => return Err(SyncError::UnknownVersion(why)),
        Err(e) => return Err(SyncError::Export(e.into())),
    }

    let v = views(project, base, theirs, map, kind)?;
    // The project's Purple Pen ids belong to the linked file, unless the base copy was ignored because the
    // link changed: then they may come from another file (M12).
    let trust_ids = !matches!(base_note, Some(SyncNote::BaseIgnored { why: "other link" }));
    let m = match_sides_with(&v, base.is_some(), trust_ids);
    let outcomes = classify(&v, &m, project);
    let open_list = conflicts(&outcomes, &v, &m);
    let stored = project
        .sync
        .as_ref()
        .map_or(&[][..], |s| s.decisions.as_slice());
    let resolution = resolve(&open_list, stored, take);
    let conflict_keys: Vec<String> = open_list.iter().map(|c| c.key.clone()).collect();
    let mut report = SyncReport {
        changes: changes(&outcomes, &resolution.sides, base.is_some(), &v, &m),
        conflicts: open_list
            .into_iter()
            .filter(|c| resolution.open.contains(&c.key))
            .collect(),
        notes: base_note.iter().cloned().collect(),
        stale: resolution.stale.clone(),
        conflict_keys,
    };
    report
        .notes
        .extend(v.skipped.iter().cloned().map(SyncNote::NotWritten));
    if !resolution.open.is_empty() {
        return Ok(SyncPlan::Conflicts(report));
    }

    let Applied {
        mut event,
        mut delete,
        notes,
        names,
    } = apply(project, &v, &m, &outcomes, &resolution.sides)?;
    // `apply` already reported the skipped objects.
    report.notes.clear();
    report.notes.extend(base_note);
    report.notes.extend(notes);

    // D11 for the courses the import skips.
    let mut kept_ids: HashSet<u32> = HashSet::new();
    let mut kept_keys: HashSet<String> = HashSet::new();
    for (id, code, course) in visited_in_file(
        theirs,
        &delete.controls.clone(),
        &v.theirs
            .courses
            .iter()
            .filter_map(|c| c.ppen_id)
            .collect::<Vec<_>>(),
    )? {
        delete.controls.retain(|i| *i != id);
        kept_ids.insert(id);
        kept_keys.extend(outcomes.iter().filter_map(|o| {
            match o.key {
                FieldKey::Object(ObjectKind::Control, i)
                    if m.controls[i]
                        .theirs
                        .is_some_and(|ti| v.theirs.controls[ti].ppen_id == Some(id)) =>
                {
                    Some(conflict_key(o, &v, &m))
                }
                _ => None,
            }
        }));
        report.notes.push(SyncNote::DeletionKept {
            control: code,
            course,
        });
    }

    // A control kept on purpose is the note above; reporting its deletion would repeat on every sync (M8).
    report.changes.retain(|c| !kept_keys.contains(&c.key));

    // D8: exactly theirs' id for what matched theirs, none for the rest.
    for c in &mut event.controls {
        c.ppen_id = None;
    }
    for c in &mut event.courses {
        c.ppen_id = None;
    }
    for s in &mut event.specials {
        s.set_ppen_id(None);
    }
    let mut control_at: HashMap<&str, usize> = HashMap::new();
    for (i, c) in event.controls.iter().enumerate() {
        control_at.entry(c.code.0.as_str()).or_insert(i);
    }
    let mut course_at: HashMap<&str, usize> = HashMap::new();
    for (i, c) in event.courses.iter().enumerate() {
        course_at.entry(c.name.as_str()).or_insert(i);
    }
    let mut special_at: HashMap<&str, usize> = HashMap::new();
    for (i, s) in event.specials.iter().enumerate() {
        special_at.entry(s.id()).or_insert(i);
    }
    let control_ids: Vec<(usize, Option<u32>)> = m
        .controls
        .iter()
        .zip(&names.controls)
        .filter_map(|(t, name)| {
            Some((
                *control_at.get(name.as_deref()?)?,
                v.theirs.controls[t.theirs?].ppen_id,
            ))
        })
        .collect();
    let course_ids: Vec<(usize, Option<u32>)> = m
        .courses
        .iter()
        .zip(&names.courses)
        .filter_map(|(t, name)| {
            Some((
                *course_at.get(name.as_deref()?)?,
                v.theirs.courses[t.theirs?].ppen_id,
            ))
        })
        .collect();
    let special_ids: Vec<(usize, Option<u32>)> = m
        .specials
        .iter()
        .zip(&names.specials)
        .filter_map(|(t, name)| {
            Some((
                *special_at.get(name.as_deref()?)?,
                v.theirs.specials[t.theirs?].ppen_id(),
            ))
        })
        .collect();
    for (i, id) in control_ids {
        event.controls[i].ppen_id = id;
    }
    for (i, id) in course_ids {
        event.courses[i].ppen_id = id;
    }
    for (i, id) in special_ids {
        event.specials[i].set_ppen_id(id);
    }

    let written = export_ppen_as(
        &event,
        map,
        kind,
        Target {
            base: Some(theirs),
            linked: true,
            map_path: String::new(),
            lang: "en",
            delete,
        },
    )?;

    // The writer must have matched everything the merge modeled.
    let unmatched = unmatched(
        written.notes.iter().filter_map(|n| match n {
            ExportNote::Kept(k) => Some(k),
            _ => None,
        }),
        &v,
        &kept_ids,
    );
    if !unmatched.is_empty() {
        return Err(SyncError::Unmatched(unmatched));
    }

    store_ids(&mut event, &written.ids);

    // Verify: the result reads back, and the merged project and the written file agree.
    let read_back = import_ppen_as(written.text.as_bytes(), map, kind)
        .map_err(|e| SyncError::Verify(vec![e.to_string()]))?;
    // Every number placement taken from Purple Pen must read back from the written file as it was (I1).
    let lost = numbers_lost(&outcomes, &resolution.sides, &names, &read_back, &v, &m);
    if !lost.is_empty() {
        return Err(SyncError::Verify(lost));
    }
    let check = views(
        &event,
        Some(written.text.as_bytes()),
        written.text.as_bytes(),
        map,
        kind,
    )?;
    let cm = match_sides(&check, true);
    let on_purpose = |o: &FieldOutcome| match o.key {
        FieldKey::Object(ObjectKind::Control, i) => cm.controls[i]
            .base
            .and_then(|b| check.base.controls[b].ppen_id)
            .is_some_and(|id| kept_ids.contains(&id)),
        _ => false,
    };
    let left: Vec<String> = classify(&check, &cm, &event)
        .iter()
        .filter(|o| o.pick != Pick::Unchanged && !on_purpose(o))
        .map(|o| conflict_key(o, &check, &cm))
        .collect();
    if !left.is_empty() {
        return Err(SyncError::Verify(left));
    }

    let same = written.text.as_bytes() == theirs;
    Ok(SyncPlan::Ready {
        project: event,
        base: written.text.clone(),
        ppen: (!same).then_some(written.text),
        report,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_courseplan_model::ControlKind;

    fn views_of(ids: &[(u32, &str)]) -> Views {
        let empty = crate::ppen_import::PpenImport {
            controls: Vec::new(),
            specials: Vec::new(),
            courses: Vec::new(),
            notes: Vec::new(),
        };
        let mut theirs = empty.clone();
        for (id, code) in ids {
            theirs.controls.push(freemapper_courseplan_model::Control {
                code: freemapper_courseplan_model::ControlCode((*code).to_owned()),
                kind: ControlKind::Control,
                map_mm: freemapper_courseplan_model::MapPoint { x: 0.0, y: 0.0 },
                description: Default::default(),
                number: None,
                ppen_id: Some(*id),
            });
        }
        Views {
            base: empty.clone(),
            ours: empty,
            theirs,
            ours_ids: Default::default(),
            ours_names: Default::default(),
            skipped: Vec::new(),
        }
    }

    #[test]
    fn a_modeled_kept_object_is_unmatched() {
        let v = views_of(&[(3, "31"), (4, "32")]);
        let kept = [
            KeptObject::Control {
                id: 3,
                code: "31".to_owned(),
            },
            KeptObject::Control {
                id: 99,
                code: "x".to_owned(),
            },
        ];
        let got = unmatched(kept.iter(), &v, &HashSet::new());
        assert_eq!(got, vec![kept[0].clone()]);
        let purpose: HashSet<u32> = [3].into();
        assert!(unmatched(kept.iter(), &v, &purpose).is_empty());
    }
}
