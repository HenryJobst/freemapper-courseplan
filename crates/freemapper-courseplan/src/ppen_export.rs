//! Export of the project to a Purple Pen event (`.ppen`), the inverse of [`crate::ppen_import`].
//!
//! `freemapper-course` writes the file (core ADR 0014). Its writer takes a `RawEvent` in the file's frame
//! and never flips; [`to_raw`] maps the project onto one:
//!
//! - Frame: the project holds map mm with y down, a `.ppen` file mm with y up, both at the map scale. A
//!   point `(x, y)` becomes `(x, -y)`; there is no translation and no µm step (a `RawPoint` is mm).
//! - Controls: starts and finishes become controls of kind start and finish, coded by their special
//!   object id, first and in the project's order; then the project's controls and crossing points. Purple
//!   Pen stores a code only for normal controls; a re-import names the others `STA1`, `FIN1`, `CROSS1` …
//!   in file order, and a code that does not come back so is noted ([`ExportNote::CodeNotStored`]).
//! - Courses: individual courses become normal courses with one leg per pair of stops, its bends and
//!   whether it is taped (a marked route). Team, relay and score courses are skipped and named, as the IOF
//!   XML export does *(Claude's call)*. The print scale is the planned one, else the map scale, which is
//!   what the project prints at.
//! - Numbers: a stop's placement is the course control's own, else the control's central one, else none
//!   (Purple Pen places it). The offset is [`offset_from_placement`] with the circle and box sizes of the
//!   map kind ([`crate::numbers`]) at the course's `CourseObjRatio`, which [`course_obj_ratio`] gives from
//!   the item scaling, map scale and print scale written: the print scale over the map scale for item
//!   scaling `None`, which the writer uses from scratch. The core function returns the offset in the map
//!   frame (y down), so `dy_mm` changes sign. The import divides by the same ratio, so a placement comes
//!   back within its rounding.
//! - Special objects: out-of-bounds and dangerous areas, single-cross forbidden routes, first aid and water
//!   points. A marked route special and a forbidden route of several points have no Purple Pen
//!   counterpart and are skipped; so is an object whose course list names only courses not written.
//! - Ids: the Purple Pen ids of the project only when asked, that is when the target is the file the project
//!   is linked to; ids from another file could capture unrelated objects there.
//! - Descriptions: a `.ppen` stores Purple Pen's symbol ids, which are 2004 numbers with suffixes, while the
//!   project stores IOF Control Descriptions 2024 numbers. [`ppen_columns`] inverts the import's table
//!   ([`iof_2024`]). The inverse is built by enumerating every Purple Pen id through the import's table, so
//!   it cannot drift from it. A 2024 number that no Purple Pen id maps to has no counterpart in Purple Pen;
//!   it is left out of the exported description and reported, never replaced by a different symbol. A
//!   crossing point has one symbol in Purple Pen (column C); its other columns are left out and reported
//!   the same way. The bend's box depends on the file's description standard, so a patch maps the
//!   descriptions in the base file's standard.
//!
//! [`export_ppen`] writes the mapped event as a new file, or into an existing one (the base) through the
//! writer's patch. Before the patch, the `overlay` submodule puts the base file's own values back wherever
//! the project still equals what the import made of the base file, so that an unchanged project gives the
//! base file back byte for byte and a changed object keeps what the project does not hold. An export
//! deletes nothing: an object of the base file the project does not mention is kept and noted.
//!
//! Terms follow `docs/glossary.md`.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::OnceLock;

use freemapper_course::{
    Control as PControl, ControlKind as PKind, Course as PCourse, CourseFilter,
    CourseKind as PCourseKind, CourseSource, Description, DescriptionStandard, ItemScaling,
    KeptObject, Leg, LineEnding, MapStandard, PatchOptions, PpenMap, PpenWriteError, RawEvent,
    RawPoint, ScratchOptions, SourceFormat, Special, Stop, WriteNote, course_obj_ratio,
    offset_from_placement, write_ppen, write_ppen_patch,
};
use freemapper_courseplan_model::{
    Control, ControlCode, ControlDescription, ControlKind, Course, CourseKind, Event, MapPoint,
    NumberPlacement, SpecialObject,
};

use crate::map::{MapKind, map_kind};
use crate::numbers::{circle_outer_radius_mm, number_box_mm};
use crate::omap::MapDocument;
use crate::ppen_import::{DIRECTIONS, iof_2024};
use crate::ppen_import::{PpenImportError, import_ppen};

/// The project mapped onto a Purple Pen event, with what the mapping left out or changed.
#[derive(Debug, Clone, PartialEq)]
pub struct ToRaw {
    pub event: RawEvent,
    pub notes: Vec<ExportNote>,
    /// For each special object of `event`, the index of the project's special object it comes from.
    pub(crate) specials_from: Vec<usize>,
}

/// Something the export did not write, or wrote differently from the project.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportNote {
    /// A course of a kind Purple Pen's normal courses cannot hold: "team", "relay" or "score".
    CourseSkipped { course: String, kind: &'static str },
    /// A special object left out, and why: "marked route", "forbidden route with several points", "no
    /// course left" (it belongs only to courses that were not written).
    SpecialSkipped { id: String, why: &'static str },
    /// A description symbol left out: a 2024 id without a Purple Pen id, or a symbol in columns D to H of
    /// a crossing point, which has one symbol (column C) in Purple Pen.
    SymbolWithoutPpenId {
        control: String,
        column: char,
        id: String,
    },
    /// A start, finish or crossing point whose code a re-import would not give back: Purple Pen stores no
    /// code for them, and the import names them `STA1`, `FIN1`, `CROSS1` … in file order.
    CodeNotStored { code: String },
    /// Purple Pen flags the leg on its own (into a map exchange or a finish whose symbol is 14.1), although
    /// the project's leg is not taped.
    FlaggingImplied {
        course: String,
        from: String,
        to: String,
    },
    /// The bends of a leg changed that the base file flags only in part (from or up to a bend), and the
    /// point where the flagging starts or stops is no longer a bend; the partial flagging was dropped.
    PartialFlaggingDropped {
        course: String,
        from: String,
        to: String,
    },
    /// An object of the base file the project does not mention, kept as it was. An export deletes
    /// nothing.
    Kept(KeptObject),
}

/// A project written as a Purple Pen event.
#[derive(Debug, Clone, PartialEq)]
pub struct PpenExport {
    /// The file, byte order mark included.
    pub text: String,
    /// The Purple Pen id each written project object has in the file.
    pub ids: ExportIds,
    pub notes: Vec<ExportNote>,
    /// The number of courses written.
    pub courses: usize,
}

/// The Purple Pen ids of the written project objects: controls by code, courses by name, special
/// objects (starts and finishes too) by id.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExportIds {
    pub controls: Vec<(ControlCode, u32)>,
    pub courses: Vec<(String, u32)>,
    pub specials: Vec<(String, u32)>,
}

/// Why a project could not be exported to a `.ppen`.
#[derive(Debug, thiserror::Error)]
pub enum PpenExportError {
    /// The base file cannot be read or does not fit the project's map.
    #[error(transparent)]
    Base(#[from] PpenImportError),
    #[error(transparent)]
    Write(#[from] PpenWriteError),
}

/// Where an export goes.
#[derive(Debug, Clone)]
pub struct Target<'a> {
    /// The existing `.ppen` to write into, `None` for a new file.
    pub base: Option<&'a [u8]>,
    /// The base is the file the project is linked to (`sources.ppen`), or the new file becomes it: the
    /// project's Purple Pen ids belong to it and are sent.
    pub linked: bool,
    /// The map path a new file names, as Purple Pen stores it.
    pub map_path: String,
    /// The language of the descriptions of a new file, such as "de".
    pub lang: &'a str,
}

/// Writes the project's courses as a Purple Pen event: into `target.base`, keeping everything the
/// project does not change byte for byte, or as a new file.
///
/// A patch reads the base file, imports it onto `map` and puts the base file's own values back wherever the
/// project still equals what the import made of them (see the `overlay` module), so that an unchanged
/// project gives the base file back. Objects of the base file the project does not mention are kept and
/// noted; nothing is deleted. The project's Purple Pen ids are used only when `target.linked`.
///
/// A new file names `target.map_path` as an OCAD map (Purple Pen reads `.omap` and `.xmap` maps so), states
/// ISSprOM 2019 for sprint and schoolyard maps and ISOM 2017 for forest maps, IOF 2018 descriptions in
/// `target.lang`, and has CRLF line ends, as Purple Pen 3.x on Windows writes.
///
/// # Errors
/// [`PpenExportError::Base`] for a base file that cannot be read or does not fit the map,
/// [`PpenExportError::Write`] for an event the writer refuses (a base file of an unknown Purple Pen
/// version, two courses drawing one leg differently, a limit).
pub fn export_ppen(
    event: &Event,
    map: &MapDocument,
    target: Target<'_>,
) -> Result<PpenExport, PpenExportError> {
    let kind = map_kind(map);
    let map_scale = map.georef.scale_denominator;
    let (overlaid, written, mut notes) = match target.base {
        None => {
            let mapped = to_raw_in(
                event,
                kind,
                map_scale,
                ItemScaling::None,
                DESCRIPTION_STANDARD,
                target.linked,
            );
            let (controls, courses, specials) = overlay::origins(event, &mapped);
            let options = ScratchOptions {
                map: PpenMap::Ocad {
                    path: target.map_path.clone(),
                },
                map_standard: match kind {
                    MapKind::Sprint | MapKind::Schoolyard => MapStandard::Issprom2019,
                    MapKind::Forest => MapStandard::Isom2017,
                },
                description_standard: DescriptionStandard::Iof2018,
                description_lang: target.lang.to_owned(),
                line_ending: LineEnding::Crlf,
            };
            let written = write_ppen(&mapped.event, &options)?;
            let overlaid = overlay::Overlaid {
                event: mapped.event,
                controls,
                courses,
                specials,
            };
            (overlaid, written, mapped.notes)
        }
        Some(bytes) => {
            let base = crate::ppen_import::read(bytes)?;
            let import = import_ppen(bytes, map)?;
            let standard = base_standard(&base, bytes);
            let scaling = base.source.item_scaling.unwrap_or_default();
            let mapped = to_raw_in(event, kind, map_scale, scaling, &standard, target.linked);
            let notes = mapped.notes.clone();
            let frame = overlay::Frame {
                kind,
                linked: target.linked,
            };
            let overlaid = overlay::overlay(mapped, event, &base, &import, &frame);
            let options = PatchOptions {
                delete: Default::default(),
                ids_from_base: target.linked,
            };
            let written = write_ppen_patch(bytes, &overlaid.event, &options)?;
            (overlaid, written, notes)
        }
    };

    for note in &written.notes {
        notes.push(match note.clone() {
            WriteNote::FlaggingImplied { course, from, to } => {
                ExportNote::FlaggingImplied { course, from, to }
            }
            WriteNote::PartialFlaggingDropped { course, from, to } => {
                ExportNote::PartialFlaggingDropped { course, from, to }
            }
        });
    }
    notes.extend(written.kept.iter().cloned().map(ExportNote::Kept));

    let mut ids = ExportIds::default();
    for (origin, id) in overlaid.controls.iter().zip(&written.ids.controls) {
        match origin {
            overlay::Origin::Control(code) => ids.controls.push((code.clone(), *id)),
            overlay::Origin::Special(s) => ids.specials.push((s.clone(), *id)),
            overlay::Origin::Base => {}
        }
    }
    ids.courses = overlaid
        .courses
        .iter()
        .cloned()
        .zip(written.ids.courses.iter().copied())
        .collect();
    ids.specials.extend(
        overlaid
            .specials
            .iter()
            .cloned()
            .zip(written.ids.specials.iter().copied()),
    );
    Ok(PpenExport {
        text: written.text,
        ids,
        notes,
        courses: overlaid.event.courses.len(),
    })
}

/// The description standard of a base file: that of its descriptions, else the `description` attribute
/// of its first `standards` element, else 2004, the reader's default.
///
/// The attribute comes from an XML parse, so comments, blanks around `=` and line breaks inside the tag
/// are read as XML reads them. The bytes have passed the reader's size, DOCTYPE and depth guards already;
/// a parse error ends the search with the default.
fn base_standard(base: &RawEvent, bytes: &[u8]) -> String {
    use quick_xml::events::Event as Xml;
    if let Some(d) = base.controls.iter().find_map(|c| c.description.as_ref()) {
        return d.standard.clone();
    }
    let text = String::from_utf8_lossy(bytes);
    let mut reader = quick_xml::Reader::from_str(&text);
    loop {
        match reader.read_event() {
            Ok(Xml::Start(e) | Xml::Empty(e)) if e.name().as_ref() == "standards" => {
                return match e.try_get_attribute("description") {
                    Ok(Some(a)) => a.value.trim().to_owned(),
                    _ => "2004".to_owned(),
                };
            }
            Ok(Xml::Eof) | Err(_) => return "2004".to_owned(),
            Ok(_) => {}
        }
    }
}

/// The description standard the export states. Purple Pen 3.x knows 2004 and 2018 and stores 2004 ids
/// for both; courseplan's descriptions are 2024, closest to 2018.
pub(crate) const DESCRIPTION_STANDARD: &str = "2018";

/// Maps `event` onto a Purple Pen event at map scale 1:`map_scale` with the course symbols scaled by
/// `scaling`, for the overprint sizes of `kind`. `use_ids` sends the project's Purple Pen ids; without it
/// every id is `None` and the writer assigns new ones. See the module for the rules.
#[cfg(test)]
pub(crate) fn to_raw(
    event: &Event,
    kind: MapKind,
    map_scale: u32,
    scaling: ItemScaling,
    use_ids: bool,
) -> ToRaw {
    to_raw_in(
        event,
        kind,
        map_scale,
        scaling,
        DESCRIPTION_STANDARD,
        use_ids,
    )
}

/// [`to_raw`] for descriptions of the given standard (`"2004"`, `"2018"`), the base file's in a patch.
fn to_raw_in(
    event: &Event,
    kind: MapKind,
    map_scale: u32,
    scaling: ItemScaling,
    standard: &str,
    use_ids: bool,
) -> ToRaw {
    let id = |id: Option<u32>| id.filter(|_| use_ids);
    let mut notes = Vec::new();
    let mut controls = Vec::with_capacity(event.specials.len() + event.controls.len());
    for s in &event.specials {
        let (pk, code, ppen_id, at) = match s {
            SpecialObject::Start {
                id,
                ppen_id,
                map_mm,
            } => (PKind::Start, id, ppen_id, map_mm),
            SpecialObject::Finish {
                id,
                ppen_id,
                map_mm,
            } => (PKind::Finish, id, ppen_id, map_mm),
            _ => continue,
        };
        controls.push(PControl {
            id: id(*ppen_id),
            code: code.clone(),
            kind: pk,
            position: Some(raw_point(at)),
            geo: None,
            description: None,
        });
    }
    for c in &event.controls {
        controls.push(PControl {
            id: id(c.ppen_id),
            code: c.code.0.clone(),
            kind: match c.kind {
                ControlKind::Control => PKind::Control,
                ControlKind::CrossingPoint => PKind::CrossingPoint,
            },
            position: Some(raw_point(&c.map_mm)),
            geo: None,
            description: description(c, standard, &mut notes),
        });
    }
    notes.extend(
        codes_not_stored(&controls)
            .into_iter()
            .map(|code| ExportNote::CodeNotStored { code }),
    );

    let by_code: HashMap<&str, &Control> = event
        .controls
        .iter()
        .map(|c| (c.code.0.as_str(), c))
        .collect();
    let mut courses = Vec::with_capacity(event.courses.len());
    for course in &event.courses {
        let label = match &course.kind {
            CourseKind::Individual { .. } => None,
            CourseKind::Team { .. } => Some("team"),
            CourseKind::Relay { .. } => Some("relay"),
            CourseKind::Score { .. } => Some("score"),
        };
        if let Some(kind) = label {
            notes.push(ExportNote::CourseSkipped {
                course: course.name.clone(),
                kind,
            });
            continue;
        }
        let print = PrintScale::of(course, map_scale, scaling);
        courses.push(raw_course(
            &by_code,
            course,
            kind,
            print,
            id(course.ppen_id),
        ));
    }

    let written: HashSet<&str> = courses.iter().map(|c| c.name.as_str()).collect();
    let mut specials = Vec::new();
    let mut specials_from = Vec::new();
    for (i, s) in event.specials.iter().enumerate() {
        if let Some(special) = raw_special(s, &written, id(s.ppen_id()), &mut notes) {
            specials.push(special);
            specials_from.push(i);
        }
    }

    ToRaw {
        event: RawEvent {
            controls,
            courses,
            specials,
            source: CourseSource {
                format: SourceFormat::PurplePen,
                event_name: Some(event.event.title.clone()),
                map_scale: Some(map_scale),
                item_scaling: Some(scaling),
            },
            notes: Vec::new(),
        },
        notes,
        specials_from,
    }
}

/// Map mm with y down to file mm with y up.
fn raw_point(p: &MapPoint) -> RawPoint {
    RawPoint {
        x_mm: p.x,
        y_mm: -p.y,
    }
}

fn raw_points(points: &[MapPoint]) -> Vec<RawPoint> {
    points.iter().map(raw_point).collect()
}

/// The Purple Pen description of a control, `None` when no column has a symbol Purple Pen knows.
fn description(c: &Control, standard: &str, notes: &mut Vec<ExportNote>) -> Option<Description> {
    let (mut columns, mut left_out) = ppen_columns(&c.description, standard);
    if c.kind == ControlKind::CrossingPoint {
        // Purple Pen keeps one symbol for a crossing point: box "all", read back as column C.
        let given = [
            &c.description.d,
            &c.description.e,
            &c.description.f,
            &c.description.g,
            &c.description.h,
        ];
        for (i, id) in given.into_iter().enumerate() {
            let column = char::from(b'D' + i as u8);
            if let Some(id) = id
                && !left_out.iter().any(|(col, _)| *col == column)
            {
                left_out.push((column, id.clone()));
            }
        }
        left_out.sort();
        columns[1..].fill(None);
    }
    notes.extend(
        left_out
            .into_iter()
            .map(|(column, id)| ExportNote::SymbolWithoutPpenId {
                control: c.code.0.clone(),
                column,
                id,
            }),
    );
    columns.iter().any(Option::is_some).then(|| Description {
        columns,
        standard: standard.to_owned(),
        column_f_text: None,
    })
}

/// The codes of starts, finishes and crossing points that a re-import would name otherwise. The reader
/// gives them the codes of Purple Pen's IOF XML export: prefix and a count per kind in file order, skipping
/// codes in use.
fn codes_not_stored(controls: &[PControl<RawPoint>]) -> Vec<String> {
    let mut used: HashSet<String> = controls
        .iter()
        .filter(|c| c.kind == PKind::Control && !c.code.is_empty())
        .map(|c| c.code.clone())
        .collect();
    let mut counters: BTreeMap<&str, u32> = BTreeMap::new();
    let mut out = Vec::new();
    for c in controls {
        let prefix = match c.kind {
            PKind::Start => "STA",
            PKind::Finish => "FIN",
            PKind::CrossingPoint => "CROSS",
            _ => continue,
        };
        let n = counters.entry(prefix).or_insert(0);
        let given = loop {
            *n += 1;
            let code = format!("{prefix}{n}");
            if used.insert(code.clone()) {
                break code;
            }
        };
        if given != c.code {
            out.push(c.code.clone());
        }
    }
    out
}

/// The print scale a course states in the file and the `CourseObjRatio` the reader computes from it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PrintScale {
    hint: Option<u32>,
    ratio: f64,
}

impl PrintScale {
    /// The project's: the planned print scale, else the map scale, which is what the project prints at.
    fn of(course: &Course, map_scale: u32, scaling: ItemScaling) -> PrintScale {
        let hint = course.planned_print_scale.unwrap_or(map_scale);
        PrintScale {
            hint: Some(hint),
            ratio: course_obj_ratio(scaling, Some(f64::from(map_scale)), Some(f64::from(hint))),
        }
    }
}

/// An individual course as a normal Purple Pen course, its number offsets at `print`.
fn raw_course(
    by_code: &HashMap<&str, &Control>,
    course: &Course,
    kind: MapKind,
    print: PrintScale,
    id: Option<u32>,
) -> PCourse<RawPoint> {
    let CourseKind::Individual { controls: ccs } = &course.kind else {
        unreachable!("only individual courses are written");
    };
    let ratio = print.ratio;
    let stop = |control: &str, number_text_at| Stop {
        control: control.to_owned(),
        kind: None,
        number: None,
        number_text_at,
        circle_gaps: Vec::new(),
        map_flip: false,
    };
    let mut stops = vec![stop(&course.start, None)];
    let mut legs = vec![Leg {
        bends: raw_points(&course.start_bends),
        gaps: Vec::new(),
        marked_route: course.start_taped,
    }];
    let mut number = 0;
    for cc in ccs {
        let offset = match by_code.get(cc.code.0.as_str()) {
            Some(c) if c.kind == ControlKind::Control => {
                number += 1;
                cc.number
                    .as_ref()
                    .or(c.number.as_ref())
                    .map(|p| file_offset(p, kind, number, ratio))
            }
            _ => None,
        };
        stops.push(stop(&cc.code.0, offset));
        legs.push(Leg {
            bends: raw_points(&cc.bends),
            gaps: Vec::new(),
            marked_route: cc.taped,
        });
    }
    stops.push(stop(&course.finish, None));
    PCourse {
        id,
        name: course.name.clone(),
        family: None,
        kind: PCourseKind::Normal,
        legs,
        stops,
        climb_m: course.climb_m.map(f64::from),
        length_m: None,
        print_scale_hint: print.hint,
        course_obj_ratio: Some(ratio),
        secondary_title: None,
    }
}

/// The file offset (mm, y up) of number `number` at `placement`, at `ratio` map mm per nominal mm.
fn file_offset(
    placement: &NumberPlacement,
    kind: MapKind,
    number: u32,
    ratio: f64,
) -> freemapper_course::model::Offset {
    let o = offset_from_placement(
        freemapper_course::NumberPlacement {
            direction: placement.direction,
            gap_mm: placement.gap_mm,
        },
        circle_outer_radius_mm(kind),
        number_box_mm(kind, number),
        ratio,
    );
    // offset_from_placement works in the map frame, y down; the file has y up.
    freemapper_course::model::Offset {
        dx_mm: o.dx_mm,
        dy_mm: -o.dy_mm,
    }
}

/// A special object other than start and finish as a Purple Pen special, `None` when it is skipped.
fn raw_special(
    s: &SpecialObject,
    written: &HashSet<&str>,
    id: Option<u32>,
    notes: &mut Vec<ExportNote>,
) -> Option<Special<RawPoint>> {
    if matches!(
        s,
        SpecialObject::Start { .. } | SpecialObject::Finish { .. }
    ) {
        // Written as controls.
        return None;
    }
    let mut skip = |why| {
        notes.push(ExportNote::SpecialSkipped {
            id: s.id().to_owned(),
            why,
        });
        None
    };
    let courses = match s.courses() {
        [] => CourseFilter::All,
        list => {
            let kept: Vec<String> = list
                .iter()
                .filter(|c| written.contains(c.as_str()))
                .cloned()
                .collect();
            if kept.is_empty() {
                // An empty list would mean all courses.
                return skip("no course left");
            }
            CourseFilter::Only(kept)
        }
    };
    Some(match s {
        SpecialObject::Start { .. } | SpecialObject::Finish { .. } => unreachable!("handled above"),
        SpecialObject::MarkedRoute { .. } => return skip("marked route"),
        SpecialObject::ForbiddenRoute { points, .. } => match points.as_slice() {
            [p] => Special::ForbiddenRoute {
                id,
                path: vec![raw_point(p)],
                courses,
            },
            _ => return skip("forbidden route with several points"),
        },
        SpecialObject::OutOfBoundsArea { polygon, .. } => Special::OutOfBounds {
            id,
            rings: vec![raw_points(polygon)],
            courses,
        },
        SpecialObject::DangerousArea { polygon, .. } => Special::Dangerous {
            id,
            rings: vec![raw_points(polygon)],
            courses,
        },
        SpecialObject::FirstAid { map_mm, .. } => Special::FirstAid {
            id,
            pt: raw_point(map_mm),
            courses,
        },
        SpecialObject::Water { map_mm, .. } => Special::Water {
            id,
            pt: raw_point(map_mm),
            courses,
        },
    })
}

/// The Purple Pen symbol id of each IOF 2024 number the import can produce. Where two Purple Pen ids give
/// the same number (`10.1` and `10.1single`, `13.5` and `13.5control`), the plain id wins.
fn inverse() -> &'static BTreeMap<String, String> {
    static TABLE: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut suffixes = vec![""];
        suffixes.extend(DIRECTIONS);
        suffixes.extend(["high", "deep", "single", "control"]);
        let mut table: BTreeMap<String, String> = BTreeMap::new();
        for section in 0..=15 {
            for item in 1..=30 {
                for suffix in &suffixes {
                    let id = format!("{section}.{item}{suffix}");
                    let Some(mapped) = iof_2024(&id) else {
                        continue;
                    };
                    // The plain id comes first and is never replaced by a longer one.
                    table.entry(mapped.id).or_insert(id);
                }
            }
        }
        table
    })
}

/// 2024 numbers of "objects of the same kind" (crossing, junction, between): Purple Pen has one symbol
/// with the variant `single` for the pair of columns D and E.
const SINGLE_CAPABLE: [&str; 3] = ["10.1", "10.2", "12.14"];

/// Purple Pen symbol ids for boxes C..H of one courseplan description, and the (column, 2024 id) pairs
/// that have no Purple Pen id and are left out.
///
/// Two import rules run backwards: in a file of description standard `"2004"`, the bend `11.1` in column F
/// goes to box G as `11.7` when G is empty (2004 has the bend in G; the import moves it to F); from 2018 on
/// it stays in box F, where Purple Pen keeps it (sampleevent8, standard 2018, box F). And D equal to E with
/// a `single` variant becomes `<id>single` in D with E empty.
pub(crate) fn ppen_columns(
    d: &ControlDescription,
    standard: &str,
) -> ([Option<String>; 6], Vec<(char, String)>) {
    let table = inverse();
    let mut cols: [Option<String>; 6] = Default::default();
    let mut left_out = Vec::new();
    let given = [&d.c, &d.d, &d.e, &d.f, &d.g, &d.h];
    for (i, id) in given.into_iter().enumerate() {
        let Some(id) = id else { continue };
        match table.get(id) {
            Some(pp) => cols[i] = Some(pp.clone()),
            None => left_out.push((char::from(b'C' + i as u8), id.clone())),
        }
    }
    // Bend: column F in the project, column G in a 2004 file.
    if standard == "2004" && d.f.as_deref() == Some("11.1") && cols[3].is_some() && d.g.is_none() {
        cols[4] = cols[3].take();
    }
    // Objects of the same kind: one symbol with the variant "single" in D, E empty.
    if let (Some(dd), Some(ee)) = (&d.d, &d.e)
        && dd == ee
        && SINGLE_CAPABLE.contains(&dd.as_str())
        && let Some(pp) = &cols[1]
    {
        cols[1] = Some(format!("{pp}single"));
        cols[2] = None;
    }
    (cols, left_out)
}

mod overlay;

#[cfg(test)]
mod tests;
