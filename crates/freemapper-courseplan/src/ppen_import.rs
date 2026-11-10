//! Import of a Purple Pen event (`.ppen`) into the courseplan model.
//!
//! `freemapper-course` reads the file and fits it to the map (core ADR 0013): after the fit every point is
//! in map coordinates (µm, y down) and every number offset in map mm with y down. This module maps that
//! fitted event onto the project's controls, special objects and courses. Content the model cannot hold is
//! never dropped silently; each case becomes an [`ImportNote`].
//!
//! - Normal controls and crossing points become [`Control`]s with their code, position, description and
//!   Purple Pen id. Starts (and map issue points, read as starts) and finishes become start and finish
//!   special objects named by the code the reader gives them (`STA1`, `FIN1`, …). A separate map exchange
//!   control has no counterpart: in a course it becomes a bend of the leg through it.
//! - Courses: a normal course from a start to a finish becomes an individual course with bends and taped
//!   legs. Score courses, courses with variations and courses without a start or finish are skipped and
//!   named. Purple Pen has no classes, so none are created.
//! - Number offsets become number placements with the circle and number box sizes of the number placement
//!   ([`crate::numbers`]), at the course's `CourseObjRatio`. When every course that numbers a control places
//!   its number the same way, the placement is the control's central one; otherwise each course keeps its
//!   own.
//! - Descriptions: a `.ppen` stores Purple Pen's symbol ids, which are the 2004 numbers with suffixes for
//!   direction and variant (`11.1NE`, `9.1high`, `10.1single`), also for 2018 descriptions. They map to the
//!   numbers of IOF Control Descriptions 2024 by [`iof_2024`]; a direction suffix stays. An id without a
//!   2024 number is left out of the description and noted ([`ImportNote::UnmappedSymbol`]), because the
//!   same number can mean another symbol in 2024. The instruction rows (such as 15.3 of a crossing point)
//!   are stored in description column c, which the model uses for them.
//!
//! Terms follow `docs/glossary.md`.

use std::collections::{HashMap, HashSet};

use freemapper_course::fit::FitOptions;
/// The largest course file the reader takes, for callers that read the file from disk.
pub use freemapper_course::model::MAX_COURSE_FILE_BYTES;
use freemapper_course::model::{Description, Offset};
use freemapper_course::{
    ControlKind as PKind, CourseError, CourseFilter, CourseKind as PCourseKind, CourseNote,
    FitError, FitNote, FittedEvent, RawEvent, Special, fit, placement_from_offset, read_ppen,
};
use freemapper_courseplan_model::{
    Control, ControlCode, ControlDescription, ControlKind, Course, CourseControl, CourseKind,
    MapPoint, NumberPlacement, SpecialObject,
};

use crate::map::{MapKind, map_kind};
use crate::numbers::{MAX_GAP_MM, circle_outer_radius_mm, normalize_direction, number_box_mm};
use crate::omap::{MapCoord, MapDocument};

/// What an import brings into the project, with the notes on what it could not bring.
#[derive(Debug, Clone, PartialEq)]
pub struct PpenImport {
    pub controls: Vec<Control>,
    /// Starts and finishes first, in file order, then the other special objects.
    pub specials: Vec<SpecialObject>,
    pub courses: Vec<Course>,
    pub notes: Vec<ImportNote>,
}

/// Something the import did not take over, or took over changed.
#[derive(Debug, Clone, PartialEq)]
pub enum ImportNote {
    /// An element or attribute value the reader does not know, by name.
    UnknownElement(String),
    /// The course was left out, and why: "variations", "alternative controls", "score course",
    /// "no start", "no finish", "start or finish inside the course", "no name".
    Skipped { course: String, why: &'static str },
    /// The course has circle gaps or leg gaps; the model has none, so its circles and legs are whole.
    GapsNotSupported { course: String },
    /// A description symbol id without an IOF 2024 number, left out of the description.
    UnmappedSymbol {
        control: String,
        column: char,
        id: String,
    },
    /// The number box touches or reaches into the circle (gap of 0 or less, which the editors reject); the
    /// placement is kept.
    NumberIntoCircle { control: String },
    /// A note of the fit worth reporting.
    Fit(FitNote),
    /// A map exchange control of the course became a bend of the leg through it.
    MapExchange { course: String, control: String },
    /// A map exchange at a normal control; the control stays, the exchange is not kept.
    MapExchangeAtControl { course: String, control: String },
    /// The course's secondary title, which the model has no field for.
    SecondaryTitle { course: String, title: String },
    /// Text in column F of a description, such as dimensions; the model keeps symbol ids only.
    ColumnFText { control: String, text: String },
    /// The description of a start or finish; special objects have none.
    StartFinishDescription { id: String },
    /// A map issue point, imported as a start.
    MapIssueAsStart { id: String },
    /// A special object of a kind the model does not have, by its Purple Pen kind.
    SpecialSkipped { kind: &'static str },
    /// A point of the special object lies outside the map's bounding box (grown by the margin the fit
    /// uses for controls), so the object is left out.
    SpecialOffMap { kind: &'static str },
    /// A special object for courses none of which was imported, left out so that it does not apply to
    /// all courses.
    SpecialWithoutCourse { kind: &'static str },
    /// The course name had white space at its start or end, or another course of the file has the same
    /// name; project course names are unique and trimmed.
    CourseRenamed { from: String, to: String },
    /// An unused control without a position, which cannot be placed on the map and is left out.
    ControlWithoutPosition { control: String },
    /// The number of the control lies farther from the circle than the editors accept; the gap was
    /// shortened to [`MAX_GAP_MM`]. Once per control.
    NumberGapClamped { control: String },
    /// A special object names a course whose name two or more courses of the file share. It applies to
    /// all of them, since the file does not say which one it meant.
    SpecialCourseAmbiguous { kind: &'static str, name: String },
    /// Courses of one family, the variants of a fork (IOF XML only).
    Forked {
        family: String,
        variants: Vec<String>,
    },
    /// The reader kept only its first notes.
    NotesTruncated,
}

/// Why a `.ppen` could not be imported.
#[derive(Debug, thiserror::Error)]
pub enum PpenImportError {
    #[error(transparent)]
    Read(#[from] CourseError),
    #[error(transparent)]
    Fit(#[from] FitError),
}

/// Reads the bytes of a `.ppen` (the first step of [`import_ppen`]).
///
/// # Errors
/// [`PpenImportError::Read`] when the file cannot be read.
pub fn read(bytes: &[u8]) -> Result<RawEvent, PpenImportError> {
    Ok(read_ppen(bytes)?)
}

/// Reads a `.ppen`, fits it to `map` and maps it onto the model.
///
/// # Errors
/// [`PpenImportError::Read`] for a file that cannot be read, [`PpenImportError::Fit`] for one that does not
/// fit the map (another scale, controls off the map).
pub fn import_ppen(bytes: &[u8], map: &MapDocument) -> Result<PpenImport, PpenImportError> {
    import_ppen_as(bytes, map, map_kind(map))
}

/// Like [`import_ppen`], with the map kind decided by the caller (see `map_kind_for_file`).
///
/// # Errors
/// As [`import_ppen`].
pub fn import_ppen_as(
    bytes: &[u8],
    map: &MapDocument,
    kind: MapKind,
) -> Result<PpenImport, PpenImportError> {
    let fitted = fit(read(bytes)?, map, &FitOptions::default())?;
    let mut import = from_fitted_in(fitted.event, kind, map_bounds(map));
    let fit_notes = fitted
        .notes
        .into_iter()
        .filter(worth_reporting)
        .map(ImportNote::Fit);
    import.notes.splice(0..0, fit_notes);
    Ok(import)
}

/// The box around every object coordinate of the map (map mm), grown by the margin the fit allows for
/// controls; `None` for a map without objects.
fn map_bounds(map: &MapDocument) -> Option<Bounds> {
    let mut coords = map
        .parts
        .iter()
        .flat_map(|p| &p.objects)
        .flat_map(|o| &o.coords);
    let first = coords.next()?;
    let (mut lo, mut hi) = ((first.x, first.y), (first.x, first.y));
    for c in coords {
        lo = (lo.0.min(c.x), lo.1.min(c.y));
        hi = (hi.0.max(c.x), hi.1.max(c.y));
    }
    let m = freemapper_course::fit::MARGIN_MM;
    let mm = |v: i32| f64::from(v) / 1000.0;
    Some(((mm(lo.0) - m, mm(lo.1) - m), (mm(hi.0) + m, mm(hi.1) + m)))
}

/// Lower left and upper right corner in map mm.
type Bounds = ((f64, f64), (f64, f64));

/// The fit notes an import reports: the unchecked scale and the warnings about orientation and axis. The
/// georeferencing notes (`PlacedByGeoref` and the like) are left out: a `.ppen` has no latitude and
/// longitude, so they cannot occur.
fn worth_reporting(note: &FitNote) -> bool {
    matches!(
        note,
        FitNote::ScaleUnchecked | FitNote::OrientationAssumed | FitNote::AxisAgainstFormat { .. }
    )
}

/// Maps a fitted event onto the model; `kind` gives the overprint sizes for the number placements.
pub fn from_fitted(event: FittedEvent, kind: MapKind) -> PpenImport {
    from_fitted_in(event, kind, None)
}

/// Like [`from_fitted`]; special objects with a point outside `bounds` are left out and noted.
fn from_fitted_in(event: FittedEvent, kind: MapKind, bounds: Option<Bounds>) -> PpenImport {
    let mut notes = Vec::new();
    for note in &event.notes {
        match note {
            CourseNote::UnknownElement(what) => {
                notes.push(ImportNote::UnknownElement(what.clone()))
            }
            CourseNote::Variations(course) => notes.push(ImportNote::Skipped {
                course: course.clone(),
                why: "variations",
            }),
            CourseNote::AlternativeControls(course) => notes.push(ImportNote::Skipped {
                course: course.clone(),
                why: "alternative controls",
            }),
            CourseNote::MapIssueAsStart(id) => {
                notes.push(ImportNote::MapIssueAsStart { id: id.clone() })
            }
            CourseNote::Forked { family, variants } => notes.push(ImportNote::Forked {
                family: family.clone(),
                variants: variants.clone(),
            }),
            CourseNote::NotesTruncated => notes.push(ImportNote::NotesTruncated),
            // The special objects below give their own note; a file without descriptions loses nothing.
            CourseNote::WhiteOutLaterInM4 | CourseNote::NoDescriptions => {}
        }
    }

    let mut out = Builder {
        notes,
        bounds,
        ..Builder::default()
    };
    for c in &event.controls {
        out.control(c);
    }
    let mut names: HashMap<&str, Vec<String>> = HashMap::new();
    for course in &event.courses {
        if let Some(name) = out.course(course, kind) {
            names.entry(course.name.as_str()).or_default().push(name);
        }
    }
    out.central_numbers();
    for special in event.specials {
        out.special(special, &names);
    }
    PpenImport {
        controls: out.controls,
        specials: out
            .starts_finishes
            .into_iter()
            .chain(out.specials)
            .collect(),
        courses: out.courses,
        notes: out.notes,
    }
}

/// A number placement read from a course: the control's place in the courses and the placement, `None`
/// when the course leaves the number to Purple Pen.
struct Numbered {
    course: usize,
    stop: usize,
    placement: Option<NumberPlacement>,
}

#[derive(Default)]
struct Builder {
    controls: Vec<Control>,
    starts_finishes: Vec<SpecialObject>,
    specials: Vec<SpecialObject>,
    courses: Vec<Course>,
    notes: Vec<ImportNote>,
    /// The kind and position of every control of the file, by code.
    kinds: HashMap<String, (PKind, MapPoint)>,
    /// Ids of special objects and control codes in use.
    used_ids: HashSet<String>,
    /// The number placements of each control code, in course order.
    numbers: HashMap<String, Vec<Numbered>>,
    /// Control codes in file order, for notes in a stable order.
    order: Vec<String>,
    /// Control codes whose clamped number gap was noted, so that each is noted once.
    clamped: HashSet<String>,
    /// The map's bounding box; special objects outside it are left out.
    bounds: Option<Bounds>,
}

fn point(c: MapCoord) -> MapPoint {
    MapPoint {
        x: f64::from(c.x) / 1000.0,
        y: f64::from(c.y) / 1000.0,
    }
}

fn points(v: Vec<MapCoord>) -> Vec<MapPoint> {
    v.into_iter().map(point).collect()
}

/// Rounds to 0.01, the precision of a stored placement.
fn hundredths(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// The courseplan description the import makes of Purple Pen symbol ids in columns C to H.
#[cfg(test)]
pub(crate) fn import_columns(columns: [Option<String>; 6]) -> ControlDescription {
    let d = Description {
        columns,
        standard: "2018".to_owned(),
        column_f_text: None,
    };
    Builder::default().description("1", &d)
}

impl Builder {
    fn control(&mut self, c: &freemapper_course::Control<MapCoord>) {
        // The fit places every control it can; a control no course uses may come without a position.
        let Some(at) = c.position.map(point) else {
            self.notes.push(ImportNote::ControlWithoutPosition {
                control: c.code.clone(),
            });
            return;
        };
        self.kinds.insert(c.code.clone(), (c.kind, at.clone()));
        self.used_ids.insert(c.code.clone());
        let described = c
            .description
            .as_ref()
            .is_some_and(|d| d.columns.iter().any(Option::is_some) || d.column_f_text.is_some());
        match c.kind {
            PKind::Start | PKind::Finish => {
                if described {
                    self.notes
                        .push(ImportNote::StartFinishDescription { id: c.code.clone() });
                }
                let id = c.code.clone();
                self.starts_finishes.push(if c.kind == PKind::Start {
                    SpecialObject::Start {
                        id,
                        ppen_id: c.id,
                        map_mm: at,
                    }
                } else {
                    SpecialObject::Finish {
                        id,
                        ppen_id: c.id,
                        map_mm: at,
                    }
                });
            }
            PKind::MapExchange => {}
            PKind::Control | PKind::CrossingPoint | PKind::EndOfMarkedRoute => {
                let description = match &c.description {
                    Some(d) => self.description(&c.code, d),
                    None => ControlDescription::default(),
                };
                self.order.push(c.code.clone());
                self.controls.push(Control {
                    code: ControlCode(c.code.clone()),
                    // The end of a marked route (IOF XML only) is a stop without number, as a crossing
                    // point is.
                    kind: if c.kind == PKind::Control {
                        ControlKind::Control
                    } else {
                        ControlKind::CrossingPoint
                    },
                    map_mm: at,
                    description,
                    number: None,
                    ppen_id: c.id,
                });
            }
        }
    }

    fn description(&mut self, code: &str, d: &Description) -> ControlDescription {
        let mut cols: [Option<String>; 6] = Default::default();
        let mut single = false;
        for (i, id) in d.columns.iter().enumerate() {
            let Some(id) = id else { continue };
            let column = char::from(b'C' + i as u8);
            match iof_2024(id) {
                Some(Mapped { id: new, single: s }) => {
                    single |= s;
                    // The bend was column G in 2004 and is column F since 2018.
                    if new == BEND_2024 && i == 4 && d.columns[3].is_none() {
                        cols[3] = Some(new);
                    } else {
                        cols[i] = Some(new);
                    }
                }
                None => {
                    self.notes.push(ImportNote::UnmappedSymbol {
                        control: code.to_owned(),
                        column,
                        id: id.clone(),
                    });
                    // Not kept: a 2004 number may mean another symbol in 2024 (12.3 is "West part").
                }
            }
        }
        // "Crossing", "junction" or "between" objects of the same kind: the 2024 standard names both
        // features, in columns D and E.
        if single && cols[2].is_none() {
            cols[2] = cols[1].clone();
        }
        if let Some(text) = &d.column_f_text {
            self.notes.push(ImportNote::ColumnFText {
                control: code.to_owned(),
                text: text.clone(),
            });
        }
        let [c, d, e, f, g, h] = cols;
        ControlDescription { c, d, e, f, g, h }
    }

    /// Imports one course; returns its name in the project, or `None` when it is skipped.
    fn course(
        &mut self,
        pc: &freemapper_course::Course<MapCoord>,
        kind: MapKind,
    ) -> Option<String> {
        let skip = |notes: &mut Vec<ImportNote>, why| {
            notes.push(ImportNote::Skipped {
                course: pc.name.clone(),
                why,
            });
            None
        };
        if pc.kind == PCourseKind::Score {
            return skip(&mut self.notes, "score course");
        }
        let trimmed = pc.name.trim();
        if trimmed.is_empty() {
            return skip(&mut self.notes, "no name");
        }
        let mut name = trimmed.to_owned();
        let mut n = 1;
        while self.courses.iter().any(|c| c.name == name) {
            n += 1;
            name = format!("{trimmed} ({n})");
        }
        let stop_kind = |s: &freemapper_course::Stop| {
            self.kinds
                .get(&s.control)
                .map(|(k, _)| *k)
                .unwrap_or(PKind::Control)
        };
        let kinds: Vec<PKind> = pc.stops.iter().map(stop_kind).collect();
        if kinds.first() != Some(&PKind::Start) {
            return skip(&mut self.notes, "no start");
        }
        if kinds.len() < 2 || kinds.last() != Some(&PKind::Finish) {
            return skip(&mut self.notes, "no finish");
        }
        let inner = &kinds[1..kinds.len() - 1];
        if inner
            .iter()
            .any(|k| matches!(k, PKind::Start | PKind::Finish))
        {
            return skip(&mut self.notes, "start or finish inside the course");
        }
        if name != pc.name {
            self.notes.push(ImportNote::CourseRenamed {
                from: pc.name.clone(),
                to: name.clone(),
            });
        }

        let course_index = self.courses.len();
        let ratio = pc.course_obj_ratio.unwrap_or(1.0);
        let mut ccs: Vec<CourseControl> = Vec::new();
        let mut start_bends = Vec::new();
        let mut start_taped = false;
        // The leg leaving the last stop taken over: its bends and whether it is taped.
        let leg = |i: usize| pc.legs.get(i);
        let mut bends: Vec<MapPoint> = leg(0).map(|l| points(l.bends.clone())).unwrap_or_default();
        let mut taped = leg(0).is_some_and(|l| l.marked_route);
        let mut number = 0;
        for (j, stop) in pc.stops.iter().enumerate().take(pc.stops.len() - 1).skip(1) {
            let out = leg(j);
            if kinds[j] == PKind::MapExchange {
                bends.push(self.kinds[&stop.control].1.clone());
                bends.extend(out.map(|l| points(l.bends.clone())).unwrap_or_default());
                taped &= out.is_some_and(|l| l.marked_route);
                self.notes.push(ImportNote::MapExchange {
                    course: name.clone(),
                    control: stop.control.clone(),
                });
                continue;
            }
            close_leg(&mut ccs, &mut start_bends, &mut start_taped, bends, taped);
            if stop.kind == Some(PKind::MapExchange) {
                self.notes.push(ImportNote::MapExchangeAtControl {
                    course: name.clone(),
                    control: stop.control.clone(),
                });
            }
            if kinds[j] == PKind::Control {
                number += 1;
                let placement = stop.number_text_at.map(|o| {
                    let (placement, clamped) = placement(o, kind, number, ratio);
                    if clamped && self.clamped.insert(stop.control.clone()) {
                        self.notes.push(ImportNote::NumberGapClamped {
                            control: stop.control.clone(),
                        });
                    }
                    placement
                });
                self.numbers
                    .entry(stop.control.clone())
                    .or_default()
                    .push(Numbered {
                        course: course_index,
                        stop: ccs.len(),
                        placement,
                    });
            }
            ccs.push(CourseControl {
                code: ControlCode(stop.control.clone()),
                number: None,
                bends: Vec::new(),
                taped: false,
            });
            bends = out.map(|l| points(l.bends.clone())).unwrap_or_default();
            taped = out.is_some_and(|l| l.marked_route);
        }
        close_leg(&mut ccs, &mut start_bends, &mut start_taped, bends, taped);

        let gaps = pc.legs.iter().any(|l| !l.gaps.is_empty())
            || pc.stops.iter().any(|s| !s.circle_gaps.is_empty());
        if gaps {
            self.notes.push(ImportNote::GapsNotSupported {
                course: name.clone(),
            });
        }
        if let Some(title) = &pc.secondary_title {
            self.notes.push(ImportNote::SecondaryTitle {
                course: name.clone(),
                title: title.clone(),
            });
        }
        let (Some(start), Some(finish)) = (pc.stops.first(), pc.stops.last()) else {
            return None;
        };
        self.courses.push(Course {
            name: name.clone(),
            classes: Vec::new(),
            start: start.control.clone(),
            finish: finish.control.clone(),
            start_bends,
            start_taped,
            print_area: None,
            climb_m: pc.climb_m.map(|c| c.round() as u32),
            planned_print_scale: pc.print_scale_hint,
            ppen_id: pc.id,
            kind: CourseKind::Individual { controls: ccs },
        });
        Some(name)
    }

    /// Puts each control's number placements into place: central when every course that numbers the
    /// control gives the same placement, else on the course controls.
    fn central_numbers(&mut self) {
        let index: HashMap<String, usize> = self
            .controls
            .iter()
            .enumerate()
            .map(|(i, c)| (c.code.0.clone(), i))
            .collect();
        for code in &self.order {
            let Some(list) = self.numbers.get(code) else {
                continue;
            };
            if list
                .iter()
                .any(|n| n.placement.as_ref().is_some_and(|p| p.gap_mm <= 0.0))
            {
                self.notes.push(ImportNote::NumberIntoCircle {
                    control: code.clone(),
                });
            }
            let first = list[0].placement.clone();
            if first.is_some() && list.iter().all(|n| n.placement == first) {
                if let Some(&i) = index.get(code) {
                    self.controls[i].number = first;
                }
                continue;
            }
            for n in list {
                if let CourseKind::Individual { controls } = &mut self.courses[n.course].kind {
                    controls[n.stop].number = n.placement.clone();
                }
            }
        }
    }

    /// A new special object id `prefix` plus a number, unused by controls and other specials.
    fn new_id(&mut self, prefix: &str) -> String {
        (1..)
            .map(|n| format!("{prefix}{n}"))
            .find(|id| !self.used_ids.contains(id))
            .inspect(|id| {
                self.used_ids.insert(id.clone());
            })
            .unwrap_or_default()
    }

    fn special(&mut self, special: Special<MapCoord>, names: &HashMap<&str, Vec<String>>) {
        let (pp_kind, filter) = match &special {
            Special::OutOfBounds { courses, .. } => ("out-of-bounds", courses),
            Special::Dangerous { courses, .. } => ("dangerous-area", courses),
            Special::ForbiddenRoute { courses, .. } => ("forbidden-route", courses),
            Special::Boundary { courses, .. } => ("boundary", courses),
            Special::MarkedRoute { courses, .. } => ("marked-route", courses),
            Special::FirstAid { courses, .. } => ("first-aid", courses),
            Special::Water { courses, .. } => ("water", courses),
            Special::Text { courses, .. } => ("text", courses),
            Special::Line { courses, .. } => ("line", courses),
            Special::WhiteOut { courses, .. } => ("white-out", courses),
        };
        if matches!(
            special,
            Special::Boundary { .. }
                | Special::Text { .. }
                | Special::Line { .. }
                | Special::WhiteOut { .. }
        ) {
            self.notes
                .push(ImportNote::SpecialSkipped { kind: pp_kind });
            return;
        }
        if let Some(((x0, y0), (x1, y1))) = self.bounds {
            let outside = |p: &MapCoord| {
                let p = point(*p);
                !(x0 <= p.x && p.x <= x1 && y0 <= p.y && p.y <= y1)
            };
            let off = match &special {
                Special::OutOfBounds { rings: r, .. } | Special::Dangerous { rings: r, .. } => {
                    r.iter().flatten().any(outside)
                }
                Special::ForbiddenRoute { path, .. } | Special::MarkedRoute { path, .. } => {
                    path.iter().any(outside)
                }
                Special::FirstAid { pt, .. } | Special::Water { pt, .. } => outside(pt),
                _ => false,
            };
            if off {
                self.notes.push(ImportNote::SpecialOffMap { kind: pp_kind });
                return;
            }
        }
        let courses: Vec<String> = match filter {
            CourseFilter::All => Vec::new(),
            CourseFilter::Only(list) => {
                let mut kept: Vec<String> = Vec::new();
                for n in list {
                    let Some(found) = names.get(n.as_str()) else {
                        continue;
                    };
                    if found.len() > 1 {
                        self.notes.push(ImportNote::SpecialCourseAmbiguous {
                            kind: pp_kind,
                            name: n.clone(),
                        });
                    }
                    kept.extend(
                        found
                            .iter()
                            .filter(|f| !kept.contains(f))
                            .cloned()
                            .collect::<Vec<_>>(),
                    );
                }
                if kept.is_empty() {
                    // An empty list would mean all courses.
                    self.notes
                        .push(ImportNote::SpecialWithoutCourse { kind: pp_kind });
                    return;
                }
                kept
            }
        };
        match special {
            Special::OutOfBounds { rings, id: sid, .. } => {
                // One special with several rings becomes several objects; the id goes to the first.
                for (n, ring) in rings.into_iter().enumerate() {
                    let ppen_id = if n == 0 { sid } else { None };
                    let id = self.new_id("OOB");
                    self.specials.push(SpecialObject::OutOfBoundsArea {
                        id,
                        ppen_id,
                        polygon: points(ring),
                        courses: courses.clone(),
                    });
                }
            }
            Special::Dangerous { rings, id: sid, .. } => {
                for (n, ring) in rings.into_iter().enumerate() {
                    let ppen_id = if n == 0 { sid } else { None };
                    let id = self.new_id("DANGER");
                    self.specials.push(SpecialObject::DangerousArea {
                        id,
                        ppen_id,
                        polygon: points(ring),
                        courses: courses.clone(),
                    });
                }
            }
            Special::ForbiddenRoute { path, id: sid, .. } => {
                let id = self.new_id("FORBID");
                self.specials.push(SpecialObject::ForbiddenRoute {
                    id,
                    ppen_id: sid,
                    points: points(path),
                    courses,
                });
            }
            Special::MarkedRoute { path, id: sid, .. } => {
                let id = self.new_id("ROUTE");
                self.specials.push(SpecialObject::MarkedRoute {
                    id,
                    ppen_id: sid,
                    points: points(path),
                    courses,
                });
            }
            Special::FirstAid { pt, id: sid, .. } => {
                let id = self.new_id("AID");
                self.specials.push(SpecialObject::FirstAid {
                    id,
                    ppen_id: sid,
                    map_mm: point(pt),
                    courses,
                });
            }
            Special::Water { pt, id: sid, .. } => {
                let id = self.new_id("WATER");
                self.specials.push(SpecialObject::Water {
                    id,
                    ppen_id: sid,
                    map_mm: point(pt),
                    courses,
                });
            }
            Special::Boundary { .. }
            | Special::Text { .. }
            | Special::Line { .. }
            | Special::WhiteOut { .. } => {}
        }
    }
}

/// Gives the leg leaving the last stop taken over its bends and tape: the start's when no control is
/// taken over yet.
fn close_leg(
    ccs: &mut [CourseControl],
    start_bends: &mut Vec<MapPoint>,
    start_taped: &mut bool,
    bends: Vec<MapPoint>,
    taped: bool,
) {
    match ccs.last_mut() {
        Some(cc) => {
            cc.bends = bends;
            cc.taped = taped;
        }
        None => {
            *start_bends = bends;
            *start_taped = taped;
        }
    }
}

/// The placement of number `number` whose box centre the file puts at `offset` (map mm, y down), with
/// the overprint sizes of `kind` at nominal size and `ratio` map mm per nominal mm.
/// A gap beyond [`MAX_GAP_MM`] is shortened to it; the flag says so.
fn placement(offset: Offset, kind: MapKind, number: u32, ratio: f64) -> (NumberPlacement, bool) {
    let p = placement_from_offset(
        offset,
        circle_outer_radius_mm(kind),
        number_box_mm(kind, number),
        ratio,
    );
    let clamped = p.gap_mm > MAX_GAP_MM;
    let placement = NumberPlacement {
        direction: hundredths(normalize_direction(p.direction)) % 360.0,
        gap_mm: hundredths(p.gap_mm.min(MAX_GAP_MM)),
    };
    (placement, clamped)
}

/// The IOF 2024 number of the bend (column F).
const BEND_2024: &str = "11.1";

/// A Purple Pen symbol id mapped to IOF Control Descriptions 2024.
#[derive(Debug, PartialEq)]
pub(crate) struct Mapped {
    pub(crate) id: String,
    /// The Purple Pen variant "objects of the same kind" (`10.1single`, `10.2single`, `11.15single`).
    pub(crate) single: bool,
}

pub(crate) const DIRECTIONS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];

/// The IOF Control Descriptions 2024 number of a Purple Pen symbol id, or `None` when 2024 has no such
/// symbol. Purple Pen stores the 2004 numbers (its `symbols.xml`: "Uses the 2004 ids only") with suffixes
/// for direction (`11.1NE`) and variant (`9.1high`, `10.1single`). Sections 0 to 10 kept their numbers in
/// 2018 and 2024, except that the four symbols Purple Pen numbers 5.23 to 5.26 (it has no 5.21 and 5.22)
/// are 5.21 to 5.24 in 2024; the bend (2004 11.7, column G) is 11.1 in column F; the locations of column G moved from section
/// 11 to 12, column H from 12 to 13, special instructions from 13 to 15 and the finish rows from 14 to 16.
/// Source: International Specification for Control Descriptions 2024 (IOF), compared with Purple Pen's
/// `symbols.xml` at `201417f9`. 2004 12.3 (radio control) and 13.6 (map issue point) have no 2024 number.
pub(crate) fn iof_2024(id: &str) -> Option<Mapped> {
    let split = id
        .char_indices()
        .find(|&(_, ch)| !(ch.is_ascii_digit() || ch == '.'))
        .map_or(id.len(), |(i, _)| i);
    let (base, suffix) = id.split_at(split);
    let (section, item) = base.split_once('.')?;
    let section: u32 = section.parse().ok()?;
    let item: u32 = item.parse().ok()?;
    let direction = DIRECTIONS.contains(&suffix);
    let plain = suffix.is_empty();
    let to = |s: u32, i: u32| format!("{s}.{i}");
    let new = match (section, item) {
        (0, 1 | 2) if direction => format!("{base}{suffix}"),
        (0, 3..=5) if plain => base.to_owned(),
        (1, 1..=16)
        | (2, 1..=10)
        | (3, 1..=11)
        | (4, 1..=10)
        | (5, 1..=20)
        | (6, 1 | 2)
        | (8, 1..=11)
            if plain =>
        {
            base.to_owned()
        }
        (5, 23..=26) if plain => to(5, item - 2),
        (9, 1 | 3) if plain || suffix == "high" || suffix == "deep" => base.to_owned(),
        (9, 2 | 4) if plain => base.to_owned(),
        (10, 1 | 2) if plain || suffix == "single" => base.to_owned(),
        (11, 7) if plain => BEND_2024.to_owned(),
        (11, 1..=6) if direction => format!("12.{item}{suffix}"),
        (11, 8) if direction => format!("12.7{suffix}"),
        (11, 14) if direction => format!("12.12{suffix}"),
        (11, 9) if plain => "12.8".to_owned(),
        (11, 10) if plain => "12.9".to_owned(),
        (11, 11) if plain => "12.10".to_owned(),
        (11, 12) if plain => "12.13".to_owned(),
        (11, 13) if plain => "12.11".to_owned(),
        (11, 15) if plain || suffix == "single" => "12.14".to_owned(),
        (12, 1 | 2) if plain => to(13, item),
        (12, 4) if plain => "13.3".to_owned(),
        (13, 1..=5) if plain => to(15, item),
        (13, 5) if suffix == "control" => "15.5".to_owned(),
        (15, 6) if plain => "15.6".to_owned(),
        (14, 1..=3) if plain => to(16, item),
        _ => return None,
    };
    Some(Mapped {
        id: new,
        single: suffix == "single",
    })
}

#[cfg(test)]
mod tests;
