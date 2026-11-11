//! The course overlay: the course symbols in map millimetres, computed here and only drawn by the page
//! (ADR 0046, D12). Sizes grow from the print sizes of `numbers.rs` by `base_scale / map_scale`, the number
//! boxes come from the same placement code the checks use, and numbers without a stored placement sit where
//! `auto_place` would put them.

use freemapper_courseplan::check::drawn_part;
use freemapper_courseplan::course::{CourseError, control_numbers, course_legs};
use freemapper_courseplan::map::{MapIndex, MapKind};
use freemapper_courseplan::numbers::{
    GapRange, auto_place, base_scale, number_box_mm, number_centre, overprint_sizes,
};
use freemapper_courseplan_model::{
    ControlKind, Course, CourseKind, Event, MapPoint, NumberPlacement, SpecialObject,
};
use serde::Serialize;

/// Rounds a JSON number to 0.001 mm.
pub(crate) fn r3(value: f64) -> f64 {
    let rounded = (value * 1000.0).round() / 1000.0;
    if rounded == 0.0 { 0.0 } else { rounded }
}

fn point(p: &MapPoint) -> MapPoint {
    MapPoint {
        x: r3(p.x),
        y: r3(p.y),
    }
}

fn points(list: &[MapPoint]) -> Vec<MapPoint> {
    list.iter().map(point).collect()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Sizes {
    pub circle_r: f64,
    pub line: f64,
    pub number_h: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct OverlayControl {
    pub code: String,
    pub kind: &'static str,
    pub at: MapPoint,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct OverlayStart {
    pub id: String,
    pub at: MapPoint,
    /// Where the triangle points: degrees, 0 east, counter-clockwise on the paper (90 is up, -y).
    pub angle_deg: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct OverlayFinish {
    pub id: String,
    pub at: MapPoint,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct OverlayLeg {
    /// Number of course controls before the leg's origin: 0 is the leg from the start, n the leg leaving the
    /// n-th control (as `after` of the MCP tool `set_leg`).
    pub after: usize,
    pub points: Vec<MapPoint>,
    pub drawn: Vec<MapPoint>,
    pub taped: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct OverlayNumber {
    /// Position in the course, counted from 1.
    pub index: usize,
    pub code: String,
    pub text: String,
    pub centre: MapPoint,
    pub w: f64,
    pub h: f64,
    /// Where the placement comes from: the course control, the control itself or the automatic placement.
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct OverlaySpecial {
    pub id: String,
    pub kind: &'static str,
    pub points: Vec<MapPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct CourseOverlay {
    pub sizes: Sizes,
    pub controls: Vec<OverlayControl>,
    pub starts: Vec<OverlayStart>,
    pub finishes: Vec<OverlayFinish>,
    pub legs: Vec<OverlayLeg>,
    pub numbers: Vec<OverlayNumber>,
    pub specials: Vec<OverlaySpecial>,
}

fn control_kind(kind: &ControlKind) -> &'static str {
    match kind {
        ControlKind::Control => "control",
        ControlKind::CrossingPoint => "crossing_point",
    }
}

/// The kind name and the points of a special object other than start and finish.
fn special_shape(special: &SpecialObject) -> Option<(&'static str, Vec<MapPoint>)> {
    Some(match special {
        SpecialObject::Start { .. } | SpecialObject::Finish { .. } => return None,
        SpecialObject::MarkedRoute { points, .. } => ("marked_route", points.clone()),
        SpecialObject::OutOfBoundsArea { polygon, .. } => ("out_of_bounds_area", polygon.clone()),
        SpecialObject::ForbiddenRoute { points, .. } => ("forbidden_route", points.clone()),
        SpecialObject::DangerousArea { polygon, .. } => ("dangerous_area", polygon.clone()),
        SpecialObject::FirstAid { map_mm, .. } => ("first_aid", vec![map_mm.clone()]),
        SpecialObject::Water { map_mm, .. } => ("water", vec![map_mm.clone()]),
    })
}

fn sizes_at(kind: MapKind, factor: f64) -> Sizes {
    let print = overprint_sizes(kind);
    Sizes {
        circle_r: r3(print.circle_diameter_mm * factor / 2.0),
        line: r3(print.line_width_mm * factor),
        number_h: r3(print.number_height_mm * factor),
    }
}

/// The overlay of the named course, or of all controls and special objects of the event without a name.
///
/// # Errors
/// [`CourseError`] for an unknown course, a course kind that is not an individual course, and a course that
/// names a start, finish or control the event lacks.
pub(crate) fn overlay(
    event: &Event,
    course: Option<&str>,
    index: &MapIndex,
    map_scale: u32,
    kind: MapKind,
    gaps: GapRange,
) -> Result<CourseOverlay, CourseError> {
    let factor = f64::from(base_scale(kind)) / f64::from(map_scale);
    let sizes = sizes_at(kind, factor);
    let Some(name) = course else {
        return Ok(all_controls(event, sizes));
    };
    let course = event
        .courses
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| CourseError::UnknownCourse(name.to_owned()))?;
    let geometry = course_legs(event, course)?;
    let CourseKind::Individual { controls: stops } = &course.kind else {
        unreachable!("course_legs refuses other kinds");
    };
    let circle_r = overprint_sizes(kind).circle_diameter_mm * factor / 2.0;

    let legs = geometry
        .iter()
        .enumerate()
        .map(|(after, leg)| OverlayLeg {
            after,
            points: points(&leg.points),
            drawn: points(&drawn_part(&leg.points, circle_r)),
            taped: leg.taped,
        })
        .collect();

    let mut controls: Vec<OverlayControl> = Vec::new();
    for stop in stops {
        let known = controls.iter().any(|c| c.code == stop.code.0);
        if let (false, Some(control)) = (known, event.controls.iter().find(|c| c.code == stop.code))
        {
            controls.push(OverlayControl {
                code: control.code.0.clone(),
                kind: control_kind(&control.kind),
                at: point(&control.map_mm),
            });
        }
    }

    let angle_deg = geometry
        .first()
        .and_then(|leg| Some((leg.points.first()?, leg.points.get(1)?)))
        .map_or(90.0, |(a, b)| (-(b.y - a.y)).atan2(b.x - a.x).to_degrees());
    let starts = event
        .specials
        .iter()
        .filter_map(|s| match s {
            SpecialObject::Start { id, map_mm, .. } if *id == course.start => Some(OverlayStart {
                id: id.clone(),
                at: point(map_mm),
                angle_deg: r3(angle_deg),
            }),
            _ => None,
        })
        .collect();
    let finishes = event
        .specials
        .iter()
        .filter_map(|s| match s {
            SpecialObject::Finish { id, map_mm, .. } if *id == course.finish => {
                Some(OverlayFinish {
                    id: id.clone(),
                    at: point(map_mm),
                })
            }
            _ => None,
        })
        .collect();

    let specials = event
        .specials
        .iter()
        .filter(|s| s.courses().is_empty() || s.courses().contains(&course.name))
        .filter_map(|s| {
            special_shape(s).map(|(kind, shape)| OverlaySpecial {
                id: s.id().to_owned(),
                kind,
                points: points(&shape),
            })
        })
        .collect();

    Ok(CourseOverlay {
        sizes,
        controls,
        starts,
        finishes,
        legs,
        numbers: numbers_of(event, course, index, map_scale, kind, gaps, factor)?,
        specials,
    })
}

/// The number boxes of the numbered controls of an individual course.
fn numbers_of(
    event: &Event,
    course: &Course,
    index: &MapIndex,
    map_scale: u32,
    kind: MapKind,
    gaps: GapRange,
    factor: f64,
) -> Result<Vec<OverlayNumber>, CourseError> {
    let CourseKind::Individual { controls: stops } = &course.kind else {
        return Ok(Vec::new());
    };
    // Kept placements come back unchanged, so this list holds every number's placement in order.
    let mut placed = auto_place(event, course, index, map_scale, kind, gaps, true)?.into_iter();
    let mut out = Vec::new();
    for (k, (stop, number)) in stops.iter().zip(control_numbers(event, course)).enumerate() {
        let (Some(number), Some(control)) =
            (number, event.controls.iter().find(|c| c.code == stop.code))
        else {
            continue;
        };
        let Some(placement) = placed.next() else {
            break;
        };
        let source = match (&control.number, &stop.number) {
            (_, Some(_)) => "course",
            (Some(_), None) => "central",
            (None, None) => "auto",
        };
        out.push(number_entry(
            k + 1,
            &control.code.0,
            number,
            &control.map_mm,
            &placement,
            kind,
            map_scale,
            factor,
            source,
        ));
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn number_entry(
    index: usize,
    code: &str,
    number: u32,
    at: &MapPoint,
    placement: &NumberPlacement,
    kind: MapKind,
    map_scale: u32,
    factor: f64,
    source: &'static str,
) -> OverlayNumber {
    let (w, h) = number_box_mm(kind, number);
    OverlayNumber {
        index,
        code: code.to_owned(),
        text: number.to_string(),
        centre: point(&number_centre(at, placement, number, kind, map_scale)),
        w: r3(w * factor),
        h: r3(h * factor),
        source,
    }
}

/// All controls of the event, its start and finish objects and the other special objects; no legs, no
/// numbers. The start triangles point up.
fn all_controls(event: &Event, sizes: Sizes) -> CourseOverlay {
    let mut starts = Vec::new();
    let mut finishes = Vec::new();
    let mut specials = Vec::new();
    for special in &event.specials {
        match special {
            SpecialObject::Start { id, map_mm, .. } => starts.push(OverlayStart {
                id: id.clone(),
                at: point(map_mm),
                angle_deg: 90.0,
            }),
            SpecialObject::Finish { id, map_mm, .. } => finishes.push(OverlayFinish {
                id: id.clone(),
                at: point(map_mm),
            }),
            other => specials.extend(special_shape(other).map(|(kind, shape)| OverlaySpecial {
                id: other.id().to_owned(),
                kind,
                points: points(&shape),
            })),
        }
    }
    CourseOverlay {
        sizes,
        controls: event
            .controls
            .iter()
            .map(|c| OverlayControl {
                code: c.code.0.clone(),
                kind: control_kind(&c.kind),
                at: point(&c.map_mm),
            })
            .collect(),
        starts,
        finishes,
        legs: Vec::new(),
        numbers: Vec::new(),
        specials,
    }
}
