//! `control_in_forbidden`: a control that stands inside an out-of-bounds area (WKB26 A 2.2, per the
//! research file; ADR 0081). Only the control is tested, not the legs and not start, finish or
//! crossing points.
//!
//! An area counts when it is an out-of-bounds special object of the event that belongs to a course
//! the control is on (or to all courses), or an area of the map with one of the rule's
//! `area_symbols`. A point on the edge of an area is inside it. A control that is on no course is not
//! reported, like the other control checks, which look at the stops of the courses only.

use freemapper_courseplan_model::{ControlCode, MapPoint, SpecialObject};

use super::geometry::point_in_area;
use super::stops::{StopKind, stops_of};
use super::{ArgValue, Args, CheckContext, Finding, Location, RuleRun, Skipped, symbol_matches};
use crate::map::FeatureGeometry;

/// The map symbols of out-of-bounds areas on sprint and schoolyard maps: the `area_symbols` of the rule
/// `wkb2026.control.in_forbidden_sprint`, which a test compares. The score draft keeps its sites out of
/// these areas with the same list and the same geometry (ADR 0082).
pub const SPRINT_AREA_SYMBOLS: &[&str] = &["520", "709", "714"];

/// A control on at least one course, with the names of the courses.
struct OnCourses {
    code: ControlCode,
    at: MapPoint,
    courses: Vec<String>,
}

/// The controls of the rule's individual courses and of all score courses, each once, in order of
/// first use. Crossing points, starts and finishes are left out.
fn controls_on_courses(ctx: &CheckContext<'_>, run: &RuleRun<'_>) -> Vec<OnCourses> {
    let mut out: Vec<OnCourses> = Vec::new();
    let mut add = |code: &ControlCode, at: &MapPoint, course: &str| match out
        .iter_mut()
        .find(|c| &c.code == code)
    {
        Some(known) => known.courses.push(course.to_owned()),
        None => out.push(OnCourses {
            code: code.clone(),
            at: at.clone(),
            courses: vec![course.to_owned()],
        }),
    };
    for course in &run.courses {
        for stop in stops_of(ctx, [*course], |kind| kind == StopKind::Control) {
            add(&ControlCode(stop.id), &stop.at, &course.course.name);
        }
    }
    for score in &ctx.score_courses {
        for control in &score.controls {
            add(&control.code, &control.at, &score.course.name);
        }
    }
    out
}

fn area_symbols(run: &RuleRun<'_>) -> Vec<String> {
    run.rule()
        .values
        .get("area_symbols")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// The id of the first out-of-bounds special object that covers the position for these courses.
fn special_hit<'e>(ctx: &'e CheckContext<'_>, control: &OnCourses) -> Option<&'e str> {
    ctx.event.specials.iter().find_map(|special| {
        let SpecialObject::OutOfBoundsArea { id, polygon, .. } = special else {
            return None;
        };
        let belongs = special.courses().is_empty()
            || special
                .courses()
                .iter()
                .any(|c| control.courses.contains(c));
        (belongs && point_in_area(&control.at, std::slice::from_ref(polygon)))
            .then_some(id.as_str())
    })
}

/// The symbol of the first map area with one of these symbols that covers the position.
fn map_hit<'e>(
    ctx: &'e CheckContext<'_>,
    symbols: &[String],
    control: &OnCourses,
) -> Option<&'e str> {
    ctx.index.features().iter().find_map(|feature| {
        let code = feature.symbol_code.as_deref()?;
        let FeatureGeometry::Area(rings) = &feature.geometry else {
            return None;
        };
        (symbols.iter().any(|s| symbol_matches(code, s)) && point_in_area(&control.at, rings))
            .then_some(code)
    })
}

pub(super) fn control_in_forbidden(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    _: &mut Vec<Skipped>,
) {
    // A rule without `area_symbols` tests the special objects only.
    let symbols = area_symbols(run);
    for control in controls_on_courses(ctx, run) {
        let place = |mut args: Args| -> Args {
            args.push(("x", ArgValue::Number(control.at.x)));
            args.push(("y", ArgValue::Number(control.at.y)));
            args
        };
        let code = ArgValue::Text(control.code.0.clone());
        let (id, args) = if let Some(area) = special_hit(ctx, &control) {
            (
                "check-control-in-forbidden-special",
                place(vec![
                    ("code", code),
                    ("id", ArgValue::Text(area.to_owned())),
                ]),
            )
        } else if let Some(symbol) = map_hit(ctx, &symbols, &control) {
            (
                "check-control-in-forbidden-map",
                place(vec![
                    ("code", code),
                    ("symbol", ArgValue::Text(symbol.to_owned())),
                ]),
            )
        } else {
            continue;
        };
        findings.push(run.finding(
            Location::Control {
                code: control.code.clone(),
            },
            id,
            args,
        ));
    }
}

#[cfg(test)]
mod tests;
