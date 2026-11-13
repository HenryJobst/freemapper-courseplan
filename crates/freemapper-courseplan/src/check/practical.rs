//! Checks of the practical sprint (ADR 0085, ADR 0086): a control deep in a dead end, and a start far from the
//! finish. Both are recommendations of the project owner, not rules of the WKB. The limits are the event
//! settings `dead_end_max_m` and `start_finish_max_m`; the generation of score courses keeps to the same
//! limits with the same functions, and a manual course may deviate.

use std::collections::HashSet;

use freemapper_courseplan_model::ControlCode;

use super::stops::{StopKind, course_stops, stops_of};
use super::{
    ArgValue, CheckContext, Finding, Location, RuleRun, SKIP_SCORE_NO_MAP, SKIP_SCORE_TRAVEL,
    Skipped,
};
use crate::score::ScoreError;
use crate::score::dead_end::{DeadEnds, ground_metres};
use crate::score::travel::HORIZON_MARGIN_M;

/// `control_dead_end`: a control of an individual or score course that lies deeper in a dead end than
/// `dead_end_max_m` of the event settings.
pub(super) fn control_dead_end(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let limit = ctx.event.score_settings().dead_end_max_m;
    let mut controls: Vec<(String, freemapper_courseplan_model::MapPoint)> =
        stops_of(ctx, run.courses.iter().copied(), |k| k == StopKind::Control)
            .into_iter()
            .map(|s| (s.id, s.at))
            .collect();
    let mut seen: HashSet<String> = controls.iter().map(|(id, _)| id.clone()).collect();
    for score in &ctx.score_courses {
        for c in &score.controls {
            if seen.insert(c.code.0.clone()) {
                controls.push((c.code.0.clone(), c.at.clone()));
            }
        }
    }
    if controls.is_empty() {
        return;
    }
    let analysis = match DeadEnds::build(&ctx.index, ctx.map_scale) {
        Ok(analysis) => analysis,
        Err(ScoreError::MapEmpty) => {
            skipped.push(run.skipped(Location::Event, SKIP_SCORE_NO_MAP, Vec::new()));
            return;
        }
        Err(other) => {
            skipped.push(run.skipped(
                Location::Event,
                SKIP_SCORE_TRAVEL,
                vec![("error", ArgValue::Text(other.to_string()))],
            ));
            return;
        }
    };
    let cap = f64::from(limit) + HORIZON_MARGIN_M;
    for (id, at) in controls {
        let Some(deep) = analysis.too_deep(&at, limit) else {
            continue;
        };
        let beyond = !deep.depth_m.is_finite();
        let metres = if beyond { cap } else { deep.depth_m };
        findings.push(run.finding(
            Location::Control {
                code: ControlCode(id.clone()),
            },
            "check-control-dead-end",
            vec![
                ("id", ArgValue::Text(id)),
                ("metres", ArgValue::Int(metres.floor() as i64)),
                (
                    "bound",
                    ArgValue::Text(if beyond { "atleast" } else { "exact" }.to_owned()),
                ),
                ("limit", ArgValue::Int(i64::from(limit))),
            ],
        ));
    }
}

/// `start_finish_distance`: start and finish of an individual or score course further apart in a straight line
/// than `start_finish_max_m` of the event settings.
pub(super) fn start_finish_distance(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    _skipped: &mut Vec<Skipped>,
) {
    let limit = ctx.event.score_settings().start_finish_max_m;
    let mut push = |course: &str, start: &str, finish: &str, metres: f64| {
        if metres > f64::from(limit) {
            findings.push(run.finding(
                Location::Course(course.to_owned()),
                "check-start-finish-distance",
                vec![
                    ("course", ArgValue::Text(course.to_owned())),
                    ("start", ArgValue::Text(start.to_owned())),
                    ("finish", ArgValue::Text(finish.to_owned())),
                    ("metres", ArgValue::Int(metres.round() as i64)),
                    ("limit", ArgValue::Int(i64::from(limit))),
                ],
            ));
        }
    };
    for course in &run.courses {
        let stops = course_stops(ctx, course);
        if let (Some(first), Some(last)) = (stops.first(), stops.last()) {
            push(
                &course.course.name,
                &first.id,
                &last.id,
                ground_metres(&first.at, &last.at, ctx.map_scale),
            );
        }
    }
    for score in &ctx.score_courses {
        push(
            &score.course.name,
            &score.course.start,
            &score.course.finish,
            ground_metres(&score.start, &score.finish, ctx.map_scale),
        );
    }
}

#[cfg(test)]
mod tests;
