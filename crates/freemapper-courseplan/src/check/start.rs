//! Start checks (plan M2b, Task 7): enough different first controls for the starters of one
//! start, and first legs that do not run in nearly the same direction.
//!
//! Courses share a start minute only when they use the same start, so both checks group the
//! courses by `Course.start` (ruling R21). The first control of a course is its first punched
//! control; a crossing point before it does not count (ruling R18).

use std::collections::{BTreeSet, HashMap};

use freemapper_courseplan_model::{ControlCode, MapPoint};

use super::distance::in_key_order;
use super::geometry::direction_difference;
use super::legs::degrees_below;
use super::stops::{Stop, StopKind, course_stops};
use super::{
    ArgValue, CheckContext, CourseContext, Finding, Location, RuleRun, SKIP_START_WINDOW_MISSING,
    SKIP_STARTERS_MISSING, Skipped, below, number,
};

/// The courses by start id, in order of first use.
fn by_start<'c>(run: &RuleRun<'c>) -> Vec<(&'c str, Vec<&'c CourseContext<'c>>)> {
    let mut groups: Vec<(&str, Vec<&CourseContext<'_>>)> = Vec::new();
    for course in run.courses.iter().copied() {
        let start = course.course.start.as_str();
        match groups.iter_mut().find(|(id, _)| *id == start) {
            Some((_, courses)) => courses.push(course),
            None => groups.push((start, vec![course])),
        }
    }
    groups
}

/// The first punched control of the course, if it has one.
fn first_control(ctx: &CheckContext<'_>, course: &CourseContext<'_>) -> Option<Stop> {
    course_stops(ctx, course)
        .into_iter()
        .find(|s| s.kind == StopKind::Control)
}

fn start_location(id: &str) -> Location {
    Location::Control {
        code: ControlCode(id.to_owned()),
    }
}

/// `start_capacity`: the courses at one start need as many different first controls as the
/// starters per minute require (SOFT, user criterion). Needed is the expected starters of all
/// classes on those courses divided by `start_window_min`, rounded up; available is the number of
/// distinct first controls among the courses that have starters. A course without classes has no
/// starters. A missing window skips the rule for the event; a class without expected starters
/// skips the start, with the class names. The finding sits at the start and names the first
/// controls that more than one course uses.
pub(super) fn start_capacity(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let window = match ctx.event.event.start_window_min {
        Some(w) if w > 0 => u64::from(w),
        _ => {
            skipped.push(run.skipped(Location::Event, SKIP_START_WINDOW_MISSING, Vec::new()));
            return;
        }
    };
    let starters_of: HashMap<&str, Option<u32>> = ctx
        .event
        .classes
        .iter()
        .map(|c| (c.name.as_str(), c.expected_starters))
        .collect();
    for (start, courses) in by_start(run) {
        // Classes the event does not define are refused elsewhere; they count nothing here.
        let mut unknown_starters: Vec<&str> = Vec::new();
        let mut per_course: Vec<(&CourseContext<'_>, u64)> = Vec::new();
        for course in &courses {
            let mut sum = 0u64;
            for name in &course.course.classes {
                match starters_of.get(name.as_str()) {
                    Some(Some(n)) => sum += u64::from(*n),
                    Some(None) if !unknown_starters.contains(&name.as_str()) => {
                        unknown_starters.push(name)
                    }
                    _ => {}
                }
            }
            per_course.push((course, sum));
        }
        if !unknown_starters.is_empty() {
            skipped.push(run.skipped(
                start_location(start),
                SKIP_STARTERS_MISSING,
                vec![
                    ("start", ArgValue::Text(start.to_owned())),
                    ("classes", ArgValue::Text(unknown_starters.join(", "))),
                ],
            ));
            continue;
        }
        let total: u64 = per_course.iter().map(|(_, n)| n).sum();
        let needed = total.div_ceil(window);
        // The first controls in order of first use, with the number of courses that use them.
        let mut firsts: Vec<(String, usize)> = Vec::new();
        for (course, _) in per_course.iter().filter(|(_, n)| *n > 0) {
            let Some(first) = first_control(ctx, course) else {
                continue;
            };
            match firsts.iter_mut().find(|(id, _)| *id == first.id) {
                Some((_, n)) => *n += 1,
                None => firsts.push((first.id, 1)),
            }
        }
        let available = firsts.len() as u64;
        if needed <= available {
            continue;
        }
        let shared: Vec<&str> = firsts
            .iter()
            .filter(|(_, n)| *n > 1)
            .map(|(id, _)| id.as_str())
            .collect();
        let mut args = vec![
            ("start", ArgValue::Text(start.to_owned())),
            ("needed", ArgValue::Int(needed as i64)),
            ("available", ArgValue::Int(available as i64)),
        ];
        let id = if shared.is_empty() {
            "check-start-capacity"
        } else {
            args.push(("shared", ArgValue::Text(shared.join(", "))));
            "check-start-capacity-shared"
        };
        findings.push(run.finding(start_location(start), id, args));
    }
}

/// `first_leg_angle`: two courses at the same start with different first controls whose first
/// legs differ in direction by less than `degrees` (SOFT 7.4.4 asks for different first controls;
/// this adds the direction). The direction runs from the start to the first start bend, or else
/// to the first stop, which may be a crossing point. Courses with the same first control are no
/// pair, that is `start_capacity`'s business. A pair of first controls is reported once, with
/// the smallest angle between any two of their courses; directions without length give no angle.
pub(super) fn first_leg_angle(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(limit) = number(run, "degrees", skipped) else {
        return;
    };
    let mut reported: BTreeSet<(String, String)> = BTreeSet::new();
    for (start, courses) in by_start(run) {
        // (first control, start point, first point of the first leg)
        let heads: Vec<(Stop, &MapPoint, &MapPoint)> = courses
            .iter()
            .filter_map(|course| {
                let first = first_control(ctx, course)?;
                let leg = course.legs.first()?;
                Some((first, &leg.points[0], leg.points.get(1)?))
            })
            .collect();
        // The smallest angle of each pair of first controls, in order of first use.
        let mut pairs: Vec<(&Stop, &Stop, f64)> = Vec::new();
        for (i, (a, from, to_a)) in heads.iter().enumerate() {
            for (b, _, to_b) in heads.iter().skip(i + 1) {
                if a.id == b.id {
                    continue;
                }
                let Some(angle) = direction_difference(from, to_a, from, to_b) else {
                    continue;
                };
                let (x, y) = in_key_order(a, b);
                match pairs
                    .iter_mut()
                    .find(|(p, q, _)| p.id == x.id && q.id == y.id)
                {
                    Some((_, _, best)) => *best = best.min(angle),
                    None => pairs.push((x, y, angle)),
                }
            }
        }
        for (first, second, angle) in pairs {
            if !below(angle, limit) || !reported.insert((first.id.clone(), second.id.clone())) {
                continue;
            }
            findings.push(run.finding(
                Location::Controls {
                    codes: vec![
                        ControlCode(first.id.clone()),
                        ControlCode(second.id.clone()),
                    ],
                },
                "check-first-leg-angle",
                vec![
                    ("start", ArgValue::Text(start.to_owned())),
                    ("first", ArgValue::Text(first.id.clone())),
                    ("second", ArgValue::Text(second.id.clone())),
                    ("degrees", degrees_below(angle)),
                    ("limit", ArgValue::Number(limit)),
                ],
            ));
        }
    }
}

#[cfg(test)]
mod tests;
