//! Leg shape checks (plan M2b, Task 5): acute angles at controls and legs that run against
//! each other. Lengths are measured in map mm and compared in metres on the ground, angles in
//! degrees. Both checks look at the legs of one course at a time.

use freemapper_courseplan_model::MapPoint;

use super::geometry::{angle_at, opposing_length};
use super::stops::course_stops;
use super::{
    ArgValue, CheckContext, Finding, Location, RuleRun, Skipped, below, number, value_number,
};
use crate::course::LegGeometry;

/// Whole degrees, rounded down, so an angle below its limit never shows as the limit.
pub(super) fn degrees_below(degrees: f64) -> ArgValue {
    ArgValue::Int((degrees + 1e-9).floor() as i64)
}

/// `acute_angle`: the incoming and the outgoing leg at a control or crossing point form an angle
/// below `degrees` (BL12: below 90° the runners leaving lead those arriving to the control; SOA:
/// dog-legs). The incoming direction comes from the last bend before the control, or the stop
/// before it; the outgoing one goes to the first bend after it, or the next stop. Start and finish
/// have no angle, and a direction without length (a bend on the control, two stops at one
/// position) gives none. Taped legs count like any other.
///
/// A place gets only the strictest rule of the key it breaks: this rule stays silent where a
/// sibling with a smaller limit checks the course and reports the angle (ruling R15), so a 30°
/// angle gives the warning at 45° and no extra hint at 90°, also once the warning is accepted.
/// The finding sits at the incoming leg of the course.
pub(super) fn acute_angle(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(limit) = number(run, "degrees", skipped) else {
        return;
    };
    for course in &run.courses {
        let stricter: Vec<f64> = run
            .siblings_for(course)
            .filter_map(|rule| value_number(rule, "degrees"))
            .filter(|d| below(*d, limit))
            .collect();
        let stops = course_stops(ctx, course);
        for (i, pair) in course.legs.windows(2).enumerate() {
            let (incoming, outgoing) = (&pair[0], &pair[1]);
            let at = &outgoing.points[0];
            let before = &incoming.points[incoming.points.len() - 2];
            let Some(angle) = angle_at(before, at, &outgoing.points[1]) else {
                continue;
            };
            if !below(angle, limit) || stricter.iter().any(|d| below(angle, *d)) {
                continue;
            }
            let stop = &stops[i + 1];
            findings.push(run.finding(
                Location::Leg {
                    course: course.course.name.clone(),
                    after: i as u32,
                },
                "check-acute-angle",
                vec![
                    ("stop", ArgValue::Text(stop.id.clone())),
                    ("stop_kind", ArgValue::Text(stop.kind.word().to_owned())),
                    ("degrees", degrees_below(angle)),
                    ("limit", ArgValue::Number(limit)),
                ],
            ));
        }
    }
}

/// The ends of a leg: the stop it leaves and the stop it reaches, with their positions.
fn ends(leg: &LegGeometry) -> [(&str, &MapPoint); 2] {
    [
        (leg.from.as_str(), &leg.points[0]),
        (leg.to.as_str(), &leg.points[leg.points.len() - 1]),
    ]
}

/// The positions of the stops at which both legs start or end.
fn meeting_points(a: &LegGeometry, b: &LegGeometry) -> Vec<MapPoint> {
    let of_b = ends(b);
    ends(a)
        .into_iter()
        .filter(|(id, _)| of_b.iter().any(|(other, _)| other == id))
        .map(|(_, at)| at.clone())
        .collect()
}

/// `opposing_legs`: two legs of one course run against each other (WKB26 B 7.2.6 "längere
/// gegenläufige Teilstrecken"): along at least `min_length_metres` they are within `metres` of
/// each other and their directions differ by more than `min_angle_degrees`. Every pair of legs
/// of the course is compared, not only legs in a row, each leg as the polyline through its
/// bends; the shorter of the two stretches counts. Where two legs meet at a stop (one after the
/// other, or at the centre of a butterfly), the first `metres` around it do not count: that is
/// where they meet by design. Taped legs count like any other. One finding per pair of legs,
/// at both legs.
pub(super) fn opposing_legs(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let (Some(metres), Some(min_length), Some(min_angle)) = (
        number(run, "metres", skipped),
        number(run, "min_length_metres", skipped),
        number(run, "min_angle_degrees", skipped),
    ) else {
        return;
    };
    let reach = ctx.map_mm(metres);
    for course in &run.courses {
        for (i, a) in course.legs.iter().enumerate() {
            for (j, b) in course.legs.iter().enumerate().skip(i + 1) {
                let meet = meeting_points(a, b);
                let along_a = opposing_length(&a.points, &b.points, reach, min_angle, &meet);
                let along_b = opposing_length(&b.points, &a.points, reach, min_angle, &meet);
                let length = ctx.metres(along_a.min(along_b));
                if below(length, min_length) {
                    continue;
                }
                findings.push(run.finding(
                    Location::Legs {
                        course: course.course.name.clone(),
                        afters: vec![i as u32, j as u32],
                    },
                    "check-opposing-legs",
                    vec![
                        ("first", ArgValue::Int(i as i64)),
                        ("second", ArgValue::Int(j as i64)),
                        ("metres", ArgValue::Int(length.round() as i64)),
                        ("limit", ArgValue::Number(metres)),
                        ("min_length", ArgValue::Number(min_length)),
                    ],
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests;
