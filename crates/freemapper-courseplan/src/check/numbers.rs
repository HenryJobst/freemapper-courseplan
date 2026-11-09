//! Checks of the control numbers: the gap between circle and number (plan number gap range).

use std::collections::BTreeSet;

use freemapper_courseplan_model::NumberPlacement;

use super::{
    ArgValue, CheckContext, Finding, Location, RuleRun, SKIP_VALUE_MISSING, Skipped, below,
    value_number,
};

/// Whether `value` is above `limit`, with the tolerance of `below`.
fn above(value: f64, limit: f64) -> bool {
    value > limit + 1e-9
}

/// `number_gap`: the gap between the circle and the stored number is below `min_gap_mm` or above
/// `max_gap_mm`. Each value may be missing, at least one is needed. Controls without a stored
/// placement are skipped; placement itself keeps stored gaps (ADR 0032), so this check is what
/// tells about an old or manual gap outside the range.
///
/// The effective placement of a course control is its own number, else the central number of the
/// event control. An own number is reported at the incoming leg of the course. A central number
/// is reported once per rule at the control, however many courses use it. A place gets only the
/// strictest rule of each side (mirror of ruling R15): this rule stays silent where a sibling
/// with a larger min or a smaller max reports the gap.
pub(super) fn number_gap(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let min = value_number(run.rule(), "min_gap_mm");
    let max = value_number(run.rule(), "max_gap_mm");
    if min.is_none() && max.is_none() {
        skipped.push(run.skipped(
            Location::Event,
            SKIP_VALUE_MISSING,
            vec![("name", ArgValue::Text("min_gap_mm".to_owned()))],
        ));
        return;
    }
    let mut reported_central: BTreeSet<&str> = BTreeSet::new();
    for course in &run.courses {
        let siblings: Vec<_> = run.siblings_for(course).collect();
        let report = |gap: f64| -> Option<(&'static str, f64)> {
            if let Some(limit) = min.filter(|m| below(gap, *m)) {
                let beaten = siblings
                    .iter()
                    .filter_map(|r| value_number(r, "min_gap_mm"))
                    .any(|m| above(m, limit) && below(gap, m));
                return (!beaten).then_some(("check-number-gap-small", limit));
            }
            if let Some(limit) = max.filter(|m| above(gap, *m)) {
                let beaten = siblings
                    .iter()
                    .filter_map(|r| value_number(r, "max_gap_mm"))
                    .any(|m| below(m, limit) && above(gap, m));
                return (!beaten).then_some(("check-number-gap-large", limit));
            }
            None
        };
        for (k, course_control) in course.controls.iter().enumerate() {
            let own = course_control.number.as_ref();
            let central = || -> Option<&NumberPlacement> {
                ctx.event
                    .controls
                    .iter()
                    .find(|c| c.code == course_control.code)
                    .and_then(|c| c.number.as_ref())
            };
            let (placement, location) = match own {
                Some(n) => (n, course.leg_location(k)),
                None => {
                    let Some(n) = central() else { continue };
                    if reported_central.contains(course_control.code.0.as_str()) {
                        continue;
                    }
                    (
                        n,
                        Location::Control {
                            code: course_control.code.clone(),
                        },
                    )
                }
            };
            let Some((message_id, limit)) = report(placement.gap_mm) else {
                continue;
            };
            if own.is_none() {
                reported_central.insert(course_control.code.0.as_str());
            }
            findings.push(run.finding(
                location,
                message_id,
                vec![
                    ("stop", ArgValue::Text(course_control.code.0.clone())),
                    ("gap", ArgValue::Number(placement.gap_mm)),
                    ("limit", ArgValue::Number(limit)),
                ],
            ));
        }
    }
}

#[cfg(test)]
mod tests;
