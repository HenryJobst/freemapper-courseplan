//! What Purple Pen does to a `.ppen` when it saves it, for the sync tests.

use std::path::{Path, PathBuf};

use freemapper_courseplan::map::{MapKind, map_kind_for_file};
use freemapper_courseplan::omap::MapDocument;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::ppen_export::{Deletions, Target, export_ppen_as};
use freemapper_courseplan::ppen_import::import_ppen_as;
use freemapper_courseplan_model::Event;

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The Lübeck map and its kind.
pub fn luebeck_map() -> (MapDocument, MapKind) {
    let path = root().join("tests/fixtures/maps/luebeck.omap");
    let map = read_map_file(&path).unwrap();
    let kind = map_kind_for_file(&map, &path);
    (map, kind)
}

const PRINT_AREA: &str = r#"<print-area automatic="true" restrict-to-page-size="true" left="0" top="0" right="0" bottom="0" page-width="1169" page-height="1654" page-margins="28" page-landscape="true" />"#;

/// The file as Purple Pen 4.0 would save it after `edit` was made in Purple Pen: the base imported, edited,
/// and written onto the base with the ids linked (that allocates ids as Purple Pen does). With
/// `pp4_defaults` the elements the 2026-11-10 Purple Pen 4.0 diff added are inserted: the event-level
/// `print-area`, `numbering`, `punch-card`, `course-appearance`, `descriptions color`, `ocad`, the
/// course-level `labels`, `print-area`, `hide-from-reports` and the map's `absolute-path`.
pub fn purple_pen_saves(base: &str, edit: impl FnOnce(&mut Event), pp4_defaults: bool) -> String {
    let (map, kind) = luebeck_map();
    let import = import_ppen_as(base.as_bytes(), &map, kind).unwrap();
    let mut event = super::valid_event(&import);
    edit(&mut event);
    // Purple Pen really deletes: what the edit removed is deleted from the file, by id.
    let mut delete = Deletions::default();
    for c in &import.controls {
        if !event.controls.iter().any(|e| e.ppen_id == c.ppen_id) {
            delete.controls.extend(c.ppen_id);
        }
    }
    for c in &import.specials {
        if !event.specials.iter().any(|e| e.ppen_id() == c.ppen_id()) {
            delete.controls.extend(c.ppen_id());
        }
    }
    for c in &import.courses {
        if !event.courses.iter().any(|e| e.ppen_id == c.ppen_id) {
            delete.courses.extend(c.ppen_id);
        }
    }
    let text = export_ppen_as(
        &event,
        &map,
        kind,
        Target {
            base: Some(base.as_bytes()),
            linked: true,
            map_path: String::new(),
            lang: "en",
            delete,
        },
    )
    .unwrap()
    .text;
    if pp4_defaults {
        add_pp4_defaults(&text)
    } else {
        text
    }
}

fn add_pp4_defaults(text: &str) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = String::new();
    let mut in_course = false;
    for line in text.split_inclusive('\n') {
        let t = line.trim();
        let body = line.trim_end_matches(['\r', '\n']);
        let indent = &body[..body.len() - body.trim_start().len()];
        let add = |out: &mut String, s: &str| {
            out.push_str(indent);
            out.push_str(s);
            out.push_str(eol);
        };
        if t.starts_with("<map ") {
            out.push_str(&body.replacen(
                "<map ",
                "<map absolute-path=\"/home/author/maps/luebeck.omap\" ",
                1,
            ));
            out.push_str(eol);
        } else if t == "<course-appearance />" {
            add(&mut out, PRINT_AREA);
            add(
                &mut out,
                r#"<numbering start="31" disallow-invertible="true" />"#,
            );
            add(
                &mut out,
                r#"<punch-card rows="3" columns="8" left-to-right="true" top-to-bottom="false" />"#,
            );
            add(
                &mut out,
                r#"<course-appearance scale-sizes="None" scale-sizes-circle-gaps="true" number-font="Roboto" auto-leg-gap-size="3.5" blend-purple="false" />"#,
            );
        } else if t == r#"<descriptions lang="en" />"# {
            add(&mut out, r#"<descriptions lang="en" color="black" />"#);
            add(&mut out, r#"<ocad overprint-colors="false" />"#);
        } else if t.starts_with("<course ") {
            in_course = true;
            out.push_str(line);
        } else if in_course && t.starts_with("<first ") {
            add(&mut out, r#"<labels label-kind="sequence" />"#);
            out.push_str(line);
        } else if in_course && t.starts_with("<options ") {
            add(&mut out, PRINT_AREA);
            add(
                &mut out,
                &t.replace(
                    " description-kind=",
                    r#" hide-from-reports="false" description-kind="#,
                ),
            );
            in_course = false;
        } else {
            out.push_str(line);
        }
    }
    out
}
