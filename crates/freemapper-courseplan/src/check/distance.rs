//! Map-based distance checks (plan M2b, Task 4): controls too close to each other or to similar
//! features, controls near the map edge, legs through other controls and through forbidden
//! areas.
//!
//! Distances are measured in map mm and compared in metres on the ground. Measured values in the
//! texts are rounded down to whole metres, so a value below the limit never shows as the limit.
//! Controls are shared by courses, so `min_distance`, `similar_features` and `map_edge` report at
//! controls, once per rule even when several courses use them. Leg findings sit at the leg of
//! one course.

use std::collections::HashSet;

use freemapper_courseplan_model::SpecialObject;
use freemapper_courseplan_model::{AcceptedFinding, ControlCode, ControlKind, MapPoint};

use super::geometry::{distance, drawn_part, polyline_meets_area, polylines_meet};
use super::{
    ArgValue, Args, CheckContext, CourseContext, Finding, Location, RuleRun, SKIP_NO_MAP_EXTENT,
    SKIP_VALUE_MISSING, Skipped, symbol_matches,
};
use crate::map::{FeatureGeometry, MapFeature, distance_to_polyline};

/// The ISOM/ISSprOM code of an out-of-bounds route; a forbidden route special stands for it.
const FORBIDDEN_ROUTE_CODE: &str = "711";

/// What a stop of a course is; the selector of the finding texts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StopKind {
    Start,
    Control,
    CrossingPoint,
    Finish,
}

impl StopKind {
    fn word(self) -> &'static str {
        match self {
            StopKind::Start => "start",
            StopKind::Control => "control",
            StopKind::CrossingPoint => "crossing",
            StopKind::Finish => "finish",
        }
    }
}

/// A start, control, crossing point or finish of a course.
#[derive(Debug, Clone)]
struct Stop {
    /// Control code or special-object id; they share one namespace (`CodeClashesWithSpecial`).
    id: String,
    kind: StopKind,
    at: MapPoint,
}

/// The stops of a course in running order, from its legs: each leg leaves one stop, and the
/// last leg reaches the finish.
fn course_stops(ctx: &CheckContext<'_>, course: &CourseContext<'_>) -> Vec<Stop> {
    let mut stops = Vec::with_capacity(course.legs.len() + 1);
    for (i, leg) in course.legs.iter().enumerate() {
        let kind = if i == 0 {
            StopKind::Start
        } else {
            match ctx.control(&ControlCode(leg.from.clone())).map(|c| &c.kind) {
                Some(ControlKind::CrossingPoint) => StopKind::CrossingPoint,
                _ => StopKind::Control,
            }
        };
        stops.push(Stop {
            id: leg.from.clone(),
            kind,
            at: leg.points[0].clone(),
        });
    }
    if let Some(last) = course.legs.last() {
        stops.push(Stop {
            id: last.to.clone(),
            kind: StopKind::Finish,
            at: last.points[last.points.len() - 1].clone(),
        });
    }
    stops
}

/// The stops of these courses whose kind `keep` selects, each once, in order of first use.
fn stops_of<'c>(
    ctx: &CheckContext<'_>,
    courses: impl IntoIterator<Item = &'c CourseContext<'c>>,
    keep: impl Fn(StopKind) -> bool,
) -> Vec<Stop> {
    let mut seen = HashSet::new();
    let mut stops = Vec::new();
    for course in courses {
        for stop in course_stops(ctx, course) {
            if keep(stop.kind) && seen.insert(stop.id.clone()) {
                stops.push(stop);
            }
        }
    }
    stops
}

/// A number of the rule's values; a missing one skips the rule with the reason.
fn number(run: &RuleRun<'_>, name: &'static str, skipped: &mut Vec<Skipped>) -> Option<f64> {
    let value = run
        .rule()
        .values
        .get(name)
        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)));
    if value.is_none() {
        skipped.push(run.skipped(
            Location::Event,
            SKIP_VALUE_MISSING,
            vec![("name", ArgValue::Text(name.to_owned()))],
        ));
    }
    value
}

fn texts(value: &toml::Value) -> Vec<String> {
    value
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// A list of symbol codes of the rule's values; missing means none.
fn codes(run: &RuleRun<'_>, name: &str) -> Vec<String> {
    run.rule().values.get(name).map(texts).unwrap_or_default()
}

/// Whether a map code matches one of the rule codes.
fn matches_any(map_code: &str, rule_codes: &[String]) -> bool {
    rule_codes.iter().any(|c| symbol_matches(map_code, c))
}

/// Metres rounded down, for a measured value below its limit.
fn metres_below(metres: f64) -> ArgValue {
    ArgValue::Int(metres.floor() as i64)
}

/// Whether `metres` is below `limit`, with a little slack for rounding of map coordinates.
fn below(metres: f64, limit: f64) -> bool {
    metres < limit - 1e-9
}

/// Two stops in the order of their finding key, which is also the order the location shows.
fn in_key_order<'s>(a: &'s Stop, b: &'s Stop) -> (&'s Stop, &'s Stop) {
    let key = AcceptedFinding {
        rule: String::new(),
        controls: vec![ControlCode(a.id.clone()), ControlCode(b.id.clone())],
        course: None,
        legs: Vec::new(),
        note: String::new(),
    }
    .key();
    if key.controls[0].0 == a.id {
        (a, b)
    } else {
        (b, a)
    }
}

/// A finding at two stops, with their ids and kinds as `first`/`second` and the extra args.
fn pair_finding(
    run: &RuleRun<'_>,
    id: &'static str,
    a: &Stop,
    b: &Stop,
    mut args: Args,
) -> Finding {
    let (first, second) = in_key_order(a, b);
    let mut all: Args = vec![
        ("first", ArgValue::Text(first.id.clone())),
        ("first_kind", ArgValue::Text(first.kind.word().to_owned())),
        ("second", ArgValue::Text(second.id.clone())),
        ("second_kind", ArgValue::Text(second.kind.word().to_owned())),
    ];
    all.append(&mut args);
    let codes = vec![
        ControlCode(first.id.clone()),
        ControlCode(second.id.clone()),
    ];
    run.finding(Location::Controls { codes }, id, all)
}

/// Pairs of distinct stops of the event of which at least one belongs to the rule's courses.
/// `of_event` are the stops of every checked course, so a control of a course at another print
/// scale still counts as a neighbour.
fn pairs<'s>(of_event: &'s [Stop], own: &HashSet<&str>) -> Vec<(&'s Stop, &'s Stop)> {
    let mut out = Vec::new();
    for (i, a) in of_event.iter().enumerate() {
        for b in &of_event[i + 1..] {
            if own.contains(a.id.as_str()) || own.contains(b.id.as_str()) {
                out.push((a, b));
            }
        }
    }
    out
}

fn own_ids(stops: &[Stop]) -> HashSet<&str> {
    stops.iter().map(|s| s.id.as_str()).collect()
}

/// `min_distance`: the centres of two controls closer than `metres`. Crossing points count, and so
/// does the start (IOF-R 19.4 "start flag included"); the finish does not.
pub(super) fn min_distance(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(limit) = number(run, "metres", skipped) else {
        return;
    };
    let keep = |kind| kind != StopKind::Finish;
    let own = stops_of(ctx, run.courses.iter().copied(), keep);
    let of_event = stops_of(ctx, &ctx.courses, keep);
    for (a, b) in pairs(&of_event, &own_ids(&own)) {
        let metres = ctx.metres(distance(&a.at, &b.at));
        if below(metres, limit) {
            let args = vec![
                ("metres", metres_below(metres)),
                ("limit", ArgValue::Number(limit)),
            ];
            findings.push(pair_finding(run, "check-distance-min", a, b, args));
        }
    }
}

/// The symbol code of the feature a control stands at: the nearest map feature within
/// `search_mm` of its centre. An area counts by its outline, so the area a control merely stands
/// in (paved area, open land) is not its feature unless the edge is near.
fn control_feature<'m>(
    features: &'m [MapFeature],
    at: &MapPoint,
    search_mm: f64,
) -> Option<&'m str> {
    let mut best: Option<(&str, f64)> = None;
    for feature in features {
        let Some(code) = feature.symbol_code.as_deref() else {
            continue;
        };
        let d = match &feature.geometry {
            FeatureGeometry::Point(p) => distance(p, at),
            FeatureGeometry::Line(points) => distance_to_polyline(at, points, false),
            FeatureGeometry::Area(rings) => rings
                .iter()
                .map(|ring| distance_to_polyline(at, ring, true))
                .fold(f64::INFINITY, f64::min),
        };
        if d <= search_mm && best.is_none_or(|(_, b)| d < b) {
            best = Some((code, d));
        }
    }
    best.map(|(code, _)| code)
}

/// Whether two symbol codes are similar: identical, or both in one group.
fn similar(a: &str, b: &str, groups: &[Vec<String>]) -> bool {
    a == b
        || groups
            .iter()
            .any(|g| matches_any(a, g) && matches_any(b, g))
}

/// `similar_features`: two controls whose features are similar closer than `metres`
/// (WKB26 B 7.2.1). Crossing points, start and finish have no control feature.
pub(super) fn similar_features(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let (Some(limit), Some(search)) = (
        number(run, "metres", skipped),
        number(run, "feature_search_mm", skipped),
    ) else {
        return;
    };
    let groups: Vec<Vec<String>> = run
        .rule()
        .values
        .get("groups")
        .and_then(|g| g.as_array())
        .map(|groups| groups.iter().map(texts).collect())
        .unwrap_or_default();
    let keep = |kind| kind == StopKind::Control;
    let own = stops_of(ctx, run.courses.iter().copied(), keep);
    let of_event = stops_of(ctx, &ctx.courses, keep);
    let features = ctx.index.features();
    let feature_of: Vec<Option<&str>> = of_event
        .iter()
        .map(|s| control_feature(features, &s.at, search))
        .collect();
    let index_of = |stop: &Stop| of_event.iter().position(|s| s.id == stop.id);
    for (a, b) in pairs(&of_event, &own_ids(&own)) {
        let (Some(fa), Some(fb)) = (
            index_of(a).and_then(|i| feature_of[i]),
            index_of(b).and_then(|i| feature_of[i]),
        ) else {
            continue;
        };
        let metres = ctx.metres(distance(&a.at, &b.at));
        if similar(fa, fb, &groups) && below(metres, limit) {
            let (first, _) = in_key_order(a, b);
            let (first_symbol, second_symbol) = if first.id == a.id { (fa, fb) } else { (fb, fa) };
            let args = vec![
                ("first_symbol", ArgValue::Text(first_symbol.to_owned())),
                ("second_symbol", ArgValue::Text(second_symbol.to_owned())),
                ("metres", metres_below(metres)),
                ("limit", ArgValue::Number(limit)),
            ];
            findings.push(pair_finding(run, "check-distance-similar", a, b, args));
        }
    }
}

/// `map_edge`: a control, crossing point, start or finish closer than `metres` to the edge of the
/// map extent (the box around all map features). A point outside the map is 0 m from it.
pub(super) fn map_edge(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(limit) = number(run, "metres", skipped) else {
        return;
    };
    let Some((min, max)) = ctx.index.extent() else {
        skipped.push(run.skipped(Location::Event, SKIP_NO_MAP_EXTENT, Vec::new()));
        return;
    };
    for stop in stops_of(ctx, run.courses.iter().copied(), |_| true) {
        let p = &stop.at;
        let inside = (p.x - min.x)
            .min(max.x - p.x)
            .min(p.y - min.y)
            .min(max.y - p.y);
        let metres = ctx.metres(inside.max(0.0));
        if below(metres, limit) {
            findings.push(run.finding(
                Location::Control {
                    code: ControlCode(stop.id.clone()),
                },
                "check-map-edge",
                vec![
                    ("id", ArgValue::Text(stop.id.clone())),
                    ("kind", ArgValue::Text(stop.kind.word().to_owned())),
                    ("metres", metres_below(metres)),
                    ("limit", ArgValue::Number(limit)),
                ],
            ));
        }
    }
}

fn leg_location(course: &CourseContext<'_>, index: usize) -> Location {
    Location::Leg {
        course: course.course.name.clone(),
        after: index as u32,
    }
}

/// `leg_through_control`: the drawn part of a leg (outside the circles at its ends) passes
/// within one circle radius of another stop of the same course (BL02). One finding per leg,
/// naming the nearest stop.
pub(super) fn leg_through_control(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    _: &mut Vec<Skipped>,
) {
    let radius = ctx.circle_radius_mm();
    for course in &run.courses {
        let stops = stops_of(ctx, [*course], |_| true);
        for (i, leg) in course.legs.iter().enumerate() {
            let drawn = drawn_part(&leg.points, radius);
            if drawn.is_empty() {
                continue;
            }
            let nearest = stops
                .iter()
                .filter(|s| s.id != leg.from && s.id != leg.to)
                .map(|s| (s, distance_to_polyline(&s.at, &drawn, false)))
                .filter(|(_, d)| below(*d, radius))
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((stop, d)) = nearest {
                findings.push(run.finding(
                    leg_location(course, i),
                    "check-leg-through-control",
                    vec![
                        ("stop", ArgValue::Text(stop.id.clone())),
                        ("stop_kind", ArgValue::Text(stop.kind.word().to_owned())),
                        ("metres", metres_below(ctx.metres(d))),
                        ("limit", ArgValue::Number(ctx.metres(radius))),
                    ],
                ));
            }
        }
    }
}

/// Something a leg must not cross.
enum Obstacle<'a> {
    OutOfBounds {
        id: &'a str,
        /// The polygon as the one ring of an area.
        rings: Vec<Vec<MapPoint>>,
    },
    ForbiddenRoute {
        id: &'a str,
        points: &'a [MapPoint],
    },
    Symbol {
        code: &'a str,
        geometry: &'a FeatureGeometry,
    },
}

impl Obstacle<'_> {
    fn met_by(&self, leg: &[MapPoint]) -> bool {
        match self {
            Obstacle::OutOfBounds { rings, .. } => polyline_meets_area(leg, rings),
            Obstacle::ForbiddenRoute { points, .. } => polylines_meet(leg, points, false),
            Obstacle::Symbol { geometry, .. } => match geometry {
                // A point symbol has no extent a leg could cross.
                FeatureGeometry::Point(_) => false,
                FeatureGeometry::Line(points) => polylines_meet(leg, points, false),
                FeatureGeometry::Area(rings) => polyline_meets_area(leg, rings),
            },
        }
    }

    fn finding(&self, run: &RuleRun<'_>, location: Location) -> Finding {
        let (id, args) = match self {
            Obstacle::OutOfBounds { id, .. } => ("check-leg-out-of-bounds", ("id", id)),
            Obstacle::ForbiddenRoute { id, .. } => ("check-leg-forbidden-route", ("id", id)),
            Obstacle::Symbol { code, .. } => ("check-leg-impassable", ("code", code)),
        };
        run.finding(
            location,
            id,
            vec![(args.0, ArgValue::Text((*args.1).to_owned()))],
        )
    }
}

/// `leg_through_forbidden`: the drawn part of a leg crosses an out-of-bounds area or a forbidden
/// route of the course, or a map feature whose symbol is in `impassable` (WKB26 B 7.2.6,
/// IOF-R 17.2). Codes in `crossing_allowed` may be crossed; whether a leg runs along such a
/// feature is not checked, so they never give a finding. A forbidden route special counts as
/// symbol 711. Taped legs are exempt. One finding per leg, naming the first obstacle.
pub(super) fn leg_through_forbidden(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    _: &mut Vec<Skipped>,
) {
    let impassable = codes(run, "impassable");
    let allowed = codes(run, "crossing_allowed");
    let symbols: Vec<Obstacle<'_>> = ctx
        .index
        .features()
        .iter()
        .filter_map(|f| {
            let code = f.symbol_code.as_deref()?;
            (matches_any(code, &impassable) && !matches_any(code, &allowed)).then_some(
                Obstacle::Symbol {
                    code,
                    geometry: &f.geometry,
                },
            )
        })
        .collect();
    let routes_blocked = !matches_any(FORBIDDEN_ROUTE_CODE, &allowed);
    let radius = ctx.circle_radius_mm();
    for course in &run.courses {
        let name = &course.course.name;
        let for_course = |courses: &[String]| courses.is_empty() || courses.contains(name);
        let mut obstacles: Vec<Obstacle<'_>> = Vec::new();
        for special in &ctx.event.specials {
            match special {
                SpecialObject::OutOfBoundsArea {
                    id,
                    polygon,
                    courses,
                } if for_course(courses) => obstacles.push(Obstacle::OutOfBounds {
                    id,
                    rings: vec![polygon.clone()],
                }),
                SpecialObject::ForbiddenRoute {
                    id,
                    points,
                    courses,
                } if routes_blocked && for_course(courses) => {
                    obstacles.push(Obstacle::ForbiddenRoute { id, points })
                }
                _ => {}
            }
        }
        for (i, leg) in course.legs.iter().enumerate() {
            if leg.taped {
                continue;
            }
            let drawn = drawn_part(&leg.points, radius);
            if drawn.is_empty() {
                continue;
            }
            if let Some(hit) = obstacles.iter().chain(&symbols).find(|o| o.met_by(&drawn)) {
                findings.push(hit.finding(run, leg_location(course, i)));
            }
        }
    }
}

#[cfg(test)]
mod tests;
