//! Control code checks (plan M2b, Task 6): the range of codes, forbidden pairs, codes that
//! read alike next to each other, and identical objects in the control circle.
//!
//! Only punched controls are looked at (ruling R18): crossing points, start and finish have no
//! code on the ground. A crossing point between two controls does not break their pair. Codes
//! are shared by courses, so findings sit at the code or the pair of codes, once per rule even
//! when several courses use them (ruling R20).

use std::collections::HashSet;

use freemapper_courseplan_model::ControlCode;

use super::distance::{control_feature, in_key_order};
use super::stops::{Stop, StopKind, course_stops, stops_of};
use super::{ArgValue, CheckContext, Finding, Location, RuleRun, SKIP_VALUE_MISSING, Skipped};
use crate::rules::Severity;

/// A code as a number, if it consists of digits only.
fn numeric(code: &str) -> Option<u64> {
    (!code.is_empty() && code.bytes().all(|b| b.is_ascii_digit()))
        .then(|| code.parse().ok())
        .flatten()
}

fn control_location(code: &str) -> Location {
    Location::Control {
        code: ControlCode(code.to_owned()),
    }
}

/// `code_range`: a numeric code outside `range` (both bounds allowed) gets the rule's severity;
/// a code that is no number gets `non_numeric_severity`, the rule's severity if the value is
/// missing (WKB26 B 7.2.10).
pub(super) fn code_range(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let bounds = run
        .rule()
        .values
        .get("range")
        .and_then(|v| v.as_array())
        .and_then(|a| Some((a.first()?.as_integer()?, a.get(1)?.as_integer()?)));
    let Some((min, max)) = bounds else {
        skipped.push(run.skipped(
            Location::Event,
            SKIP_VALUE_MISSING,
            vec![("name", ArgValue::Text("range".to_owned()))],
        ));
        return;
    };
    let non_numeric = run
        .rule()
        .values
        .get("non_numeric_severity")
        .and_then(|v| v.as_str())
        .and_then(Severity::parse)
        .unwrap_or(run.rule().severity);
    let stops = stops_of(ctx, run.courses.iter().copied(), |k| k == StopKind::Control);
    for stop in stops {
        match numeric(&stop.id) {
            Some(n) if i128::from(n) >= i128::from(min) && i128::from(n) <= i128::from(max) => {}
            Some(_) => findings.push(run.finding(
                control_location(&stop.id),
                "check-code-range",
                vec![
                    ("code", ArgValue::Text(stop.id.clone())),
                    ("min", ArgValue::Int(min)),
                    ("max", ArgValue::Int(max)),
                ],
            )),
            None => findings.push(run.finding_with_severity(
                non_numeric,
                control_location(&stop.id),
                "check-code-not-numeric",
                vec![("code", ArgValue::Text(stop.id.clone()))],
            )),
        }
    }
}

/// A code of a rule value, written as a number or as text.
fn code_text(value: &toml::Value) -> Option<String> {
    value
        .as_integer()
        .map(|n| n.to_string())
        .or_else(|| value.as_str().map(str::to_owned))
}

/// A finding at a pair of punched controls, the codes in the order of their finding key.
fn pair_finding(run: &RuleRun<'_>, id: &'static str, a: &Stop, b: &Stop) -> Finding {
    let (first, second) = in_key_order(a, b);
    run.finding(
        Location::Controls {
            codes: vec![
                ControlCode(first.id.clone()),
                ControlCode(second.id.clone()),
            ],
        },
        id,
        vec![
            ("first", ArgValue::Text(first.id.clone())),
            ("second", ArgValue::Text(second.id.clone())),
        ],
    )
}

/// `code_pairs`: both codes of a forbidden pair are used by punched controls of the event's
/// courses, not necessarily in the same course (WKB26 B 7.2.10). A control that no course uses
/// does not count. The pair is reported when a course of the rule uses one of the two codes.
pub(super) fn code_pairs(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(pairs) = run.rule().values.get("pairs").and_then(|v| v.as_array()) else {
        skipped.push(run.skipped(
            Location::Event,
            SKIP_VALUE_MISSING,
            vec![("name", ArgValue::Text("pairs".to_owned()))],
        ));
        return;
    };
    let keep = |kind| kind == StopKind::Control;
    let own = stops_of(ctx, run.courses.iter().copied(), keep);
    let of_event = stops_of(ctx, &ctx.courses, keep);
    let stop = |code: &str| of_event.iter().find(|s| s.id == code);
    for pair in pairs {
        let codes: Vec<String> = pair
            .as_array()
            .map(|a| a.iter().filter_map(code_text).collect())
            .unwrap_or_default();
        let [a, b] = codes.as_slice() else {
            continue;
        };
        let (Some(a), Some(b)) = (stop(a), stop(b)) else {
            continue;
        };
        if own.iter().any(|s| s.id == a.id || s.id == b.id) {
            findings.push(pair_finding(run, "check-code-pair", a, b));
        }
    }
}

/// The code turned by 180°: every digit rotated, in reverse order. Only 0, 1, 6, 8 and 9 can
/// rotate.
fn rotated(code: &str) -> Option<String> {
    code.chars()
        .rev()
        .map(|c| match c {
            '0' | '1' | '8' => Some(c),
            '6' => Some('9'),
            '9' => Some('6'),
            _ => None,
        })
        .collect()
}

/// Whether two codes of the same length differ only where one has 6 and the other 9.
fn six_nine(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.chars()
            .zip(b.chars())
            .all(|(x, y)| x == y || matches!((x, y), ('6', '9') | ('9', '6')))
}

/// `code_confusable_next`: two consecutive punched controls of a course whose codes read the same
/// turned by 180° or differ only by 6 and 9 (IOF-SG, SOA). Identical codes (a butterfly) and
/// codes that are no numbers are left out. A pair is reported once for all courses.
pub(super) fn code_confusable_next(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    _: &mut Vec<Skipped>,
) {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for course in &run.courses {
        let punched: Vec<Stop> = course_stops(ctx, course)
            .into_iter()
            .filter(|s| s.kind == StopKind::Control)
            .collect();
        for pair in punched.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            if a.id == b.id || numeric(&a.id).is_none() || numeric(&b.id).is_none() {
                continue;
            }
            let id = if rotated(&a.id).as_deref() == Some(b.id.as_str()) {
                "check-code-confusable-rotated"
            } else if six_nine(&a.id, &b.id) {
                "check-code-confusable-six-nine"
            } else {
                continue;
            };
            let (first, second) = in_key_order(a, b);
            if seen.insert((first.id.clone(), second.id.clone())) {
                findings.push(pair_finding(run, id, a, b));
            }
        }
    }
}

/// `same_objects_in_circle`: the control feature is a point symbol that occurs again within the
/// control circle, and column C of the description is empty (TD11). Lines and areas are left out
/// (ruling R19), as are crossing points, start and finish.
pub(super) fn same_objects_in_circle(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(search) = super::number(run, "feature_search_mm", skipped) else {
        return;
    };
    let radius = ctx.circle_radius_mm();
    let features = ctx.index.features();
    for stop in stops_of(ctx, run.courses.iter().copied(), |k| k == StopKind::Control) {
        let Some(feature) = control_feature(features, &stop.at, search).filter(|f| f.point) else {
            continue;
        };
        let described = ctx
            .control(&ControlCode(stop.id.clone()))
            .and_then(|c| c.description.c.as_deref())
            .is_some_and(|c| !c.trim().is_empty());
        if described {
            continue;
        }
        // The control feature itself is one of the objects within the circle.
        let same = ctx
            .index
            .features_near(stop.at.clone(), radius)
            .into_iter()
            .filter(|(f, _)| {
                matches!(f.geometry, crate::map::FeatureGeometry::Point(_))
                    && f.symbol_code.as_deref() == Some(feature.code)
            })
            .count();
        if same >= 2 {
            findings.push(run.finding(
                control_location(&stop.id),
                "check-same-objects-in-circle",
                vec![
                    ("code", ArgValue::Text(stop.id.clone())),
                    ("symbol", ArgValue::Text(feature.code.to_owned())),
                ],
            ));
        }
    }
}

#[cfg(test)]
mod tests;
