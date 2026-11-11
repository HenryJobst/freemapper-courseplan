//! The course overlay: the course symbols in map millimetres, computed here and only drawn by the page
//! (ADR 0046, D12). Sizes grow from the print sizes of `numbers.rs` by `numbers::course_factor` (ADR 0058), the number
//! boxes come from the same placement code the checks use, and numbers without a stored placement sit where
//! `auto_place` would put them.

use freemapper_courseplan::check::drawn_part_between;
use freemapper_courseplan::course::{CourseError, control_numbers, course_legs};
use freemapper_courseplan::map::{MapIndex, MapKind};
use freemapper_courseplan::numbers::{GapRange, auto_place, number_box_mm, number_centre};
use freemapper_courseplan_model::{
    ControlKind, Course, CourseKind, Event, MapPoint, NumberPlacement, ScoreControl, ScoreLink,
    SpecialObject,
};
use serde::Serialize;

use super::print_layer::course_symbol_sizes;

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
    /// Circumradius of the start triangle: where a leg leaves the start.
    pub start_r: f64,
    /// Radii of the two finish circles; a leg ends at the outer one.
    pub finish_inner_r: f64,
    pub finish_outer_r: f64,
    /// Length of a crossing point, as the print draws it.
    pub crossing: f64,
    /// Dash and gap of a marked (taped) leg.
    pub dash: f64,
    pub dash_gap: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct OverlayControl {
    pub code: String,
    pub kind: &'static str,
    pub at: MapPoint,
    /// Crossing points only: the direction of the way through, degrees as for a start (0 east, counter-clockwise
    /// on the paper). The print turns the symbol along the course.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle_deg: Option<f64>,
    /// Score courses only: the point value of the control in the course.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<u32>,
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
    /// The leg's bends in running order, as `{x, y}` like the `bends` argument of `set_leg`.
    pub bends: Vec<OverlayBend>,
    pub taped: bool,
}

/// A mandatory link of a score course: the straight line between the two controls, as a marked leg is drawn
/// (ADR 0069). It is no leg: nothing can be selected or bent on it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct OverlayLink {
    pub a: String,
    pub b: String,
    /// The line without the two circles.
    pub drawn: Vec<MapPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct OverlayBend {
    pub x: f64,
    pub y: f64,
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
    /// The mandatory links of a score course; empty for every other overlay.
    pub links: Vec<OverlayLink>,
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
    let print = course_symbol_sizes(kind);
    Sizes {
        circle_r: r3(print.circle_diameter_mm * factor / 2.0),
        line: r3(print.line_width_mm * factor),
        number_h: r3(print.number_height_mm * factor),
        start_r: r3(triangle_radius(print.start_side_mm * factor)),
        finish_inner_r: r3(print.finish_inner_mm * factor / 2.0),
        finish_outer_r: r3(print.finish_outer_mm * factor / 2.0),
        crossing: r3(print.crossing_point_mm * factor),
        dash: r3(print.marked_route_dash_mm * factor),
        dash_gap: r3(print.marked_route_gap_mm * factor),
    }
}

/// The direction the way through the crossing point at stop `stop` of the course runs, as the print turns the
/// symbol: from the point before it to the point after it, else along the one leg it has. Degrees, 0 east,
/// counter-clockwise on the paper.
fn crossing_angle(legs: &[freemapper_courseplan::course::LegGeometry], stop: usize) -> Option<f64> {
    let before = stop
        .checked_sub(1)
        .and_then(|i| legs.get(i))
        .and_then(|l| l.points.get(l.points.len().checked_sub(2)?));
    let after = legs.get(stop).and_then(|l| l.points.get(1));
    let (from, to) = match (before, after) {
        (Some(b), Some(a)) => (b, a),
        (Some(b), None) => (b, &legs.get(stop - 1)?.points.last()?.clone()),
        (None, Some(a)) => (&legs.get(stop)?.points.first()?.clone(), a),
        (None, None) => return None,
    };
    Some(r3((-(to.y - from.y)).atan2(to.x - from.x).to_degrees()))
}

/// Circumradius of an equilateral triangle of side `side`, as the renderer draws the start.
fn triangle_radius(side: f64) -> f64 {
    side / 3f64.sqrt()
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
    let factor = super::print_layer::course_factor(kind, map_scale);
    let sizes = sizes_at(kind, factor);
    let Some(name) = course else {
        return Ok(all_controls(event, sizes));
    };
    let course = event
        .courses
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| CourseError::UnknownCourse(name.to_owned()))?;
    if let CourseKind::Score {
        controls, links, ..
    } = &course.kind
    {
        return Ok(score_overlay(
            event, course, controls, links, kind, factor, sizes,
        ));
    }
    let geometry = course_legs(event, course)?;
    let CourseKind::Individual { controls: stops } = &course.kind else {
        unreachable!("course_legs refuses other kinds");
    };
    let print = course_symbol_sizes(kind);
    let circle_r = print.circle_diameter_mm * factor / 2.0;
    // Where a leg leaves the symbol of stop `i` (0 is the start, the last is the finish): the start triangle's
    // corner, the circle line, the outer finish circle, or the centre of a crossing point.
    let radius_of = |i: usize| -> f64 {
        if i == 0 {
            return triangle_radius(print.start_side_mm * factor);
        }
        let Some(stop) = stops.get(i - 1) else {
            return print.finish_outer_mm * factor / 2.0;
        };
        let crossing = event
            .controls
            .iter()
            .any(|c| c.code == stop.code && c.kind == ControlKind::CrossingPoint);
        if crossing { 0.0 } else { circle_r }
    };

    let legs = geometry
        .iter()
        .enumerate()
        .map(|(after, leg)| OverlayLeg {
            after,
            points: points(&leg.points),
            drawn: points(&drawn_part_between(
                &leg.points,
                radius_of(after),
                radius_of(after + 1),
            )),
            bends: leg
                .points
                .iter()
                .skip(1)
                .take(leg.points.len().saturating_sub(2))
                .map(|b| OverlayBend {
                    x: r3(b.x),
                    y: r3(b.y),
                })
                .collect(),
            taped: leg.taped,
        })
        .collect();

    let mut controls: Vec<OverlayControl> = Vec::new();
    for (k, stop) in stops.iter().enumerate() {
        let known = controls.iter().any(|c| c.code == stop.code.0);
        if let (false, Some(control)) = (known, event.controls.iter().find(|c| c.code == stop.code))
        {
            // The stop is number k + 1 of the course: the leg k arrives, the leg k + 1 leaves.
            let angle_deg = (control.kind == ControlKind::CrossingPoint)
                .then(|| crossing_angle(&geometry, k + 1))
                .flatten();
            controls.push(OverlayControl {
                code: control.code.0.clone(),
                kind: control_kind(&control.kind),
                at: point(&control.map_mm),
                angle_deg,
                value: None,
            });
        }
    }

    let angle_deg = geometry
        .first()
        .and_then(|leg| Some((leg.points.first()?, leg.points.get(1)?)))
        .map_or(90.0, |(a, b)| (-(b.y - a.y)).atan2(b.x - a.x).to_degrees());
    let starts = starts_of(event, course, angle_deg);
    let finishes = finishes_of(event, course);
    let specials = specials_of(event, course);

    Ok(CourseOverlay {
        sizes,
        controls,
        starts,
        finishes,
        legs,
        links: Vec::new(),
        numbers: numbers_of(event, course, index, map_scale, kind, gaps, factor)?,
        specials,
    })
}

fn starts_of(event: &Event, course: &Course, angle_deg: f64) -> Vec<OverlayStart> {
    event
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
        .collect()
}

fn finishes_of(event: &Event, course: &Course) -> Vec<OverlayFinish> {
    event
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
        .collect()
}

/// The special objects of the course: those that name it and those that belong to every course.
fn specials_of(event: &Event, course: &Course) -> Vec<OverlaySpecial> {
    event
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
        .collect()
}

/// The overlay of a score course: its controls with their values, start and finish, and the mandatory links as
/// straight lines between two controls. There are no legs and no numbers, because a score course has no order
/// (ADR 0069). The start triangle points up.
fn score_overlay(
    event: &Event,
    course: &Course,
    scored: &[ScoreControl],
    links: &[ScoreLink],
    kind: MapKind,
    factor: f64,
    sizes: Sizes,
) -> CourseOverlay {
    let circle_r = course_symbol_sizes(kind).circle_diameter_mm * factor / 2.0;
    let find = |code: &freemapper_courseplan_model::ControlCode| {
        event.controls.iter().find(|c| c.code == *code)
    };
    let controls = scored
        .iter()
        .filter_map(|s| {
            find(&s.code).map(|c| OverlayControl {
                code: c.code.0.clone(),
                kind: control_kind(&c.kind),
                at: point(&c.map_mm),
                angle_deg: None,
                value: Some(s.value),
            })
        })
        .collect();
    let links = links
        .iter()
        .filter_map(|l| {
            let (a, b) = (find(&l.a)?, find(&l.b)?);
            let line = [a.map_mm.clone(), b.map_mm.clone()];
            Some(OverlayLink {
                a: a.code.0.clone(),
                b: b.code.0.clone(),
                drawn: points(&drawn_part_between(&line, circle_r, circle_r)),
            })
        })
        .collect();
    CourseOverlay {
        sizes,
        controls,
        starts: starts_of(event, course, 90.0),
        finishes: finishes_of(event, course),
        legs: Vec::new(),
        links,
        numbers: Vec::new(),
        specials: specials_of(event, course),
    }
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
                angle_deg: None,
                value: None,
            })
            .collect(),
        starts,
        finishes,
        legs: Vec::new(),
        links: Vec::new(),
        numbers: Vec::new(),
        specials,
    }
}
