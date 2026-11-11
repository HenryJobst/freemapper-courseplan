//! Checks of score courses (ADR 0066): the value in the code, ambiguous codes, a ring of controls,
//! an obvious order, the length bands of both runner profiles, the limit and penalty, the
//! mandatory links and the mix of easy and hard controls.
//!
//! The rules read `CheckContext::score_courses`, never the individual courses. Findings sit at
//! the course (the whole set), at a control or at a pair of controls.

use freemapper_courseplan_model::{ControlCode, MapPoint, ScoreSettings, SpecialObject};

use super::geometry::{distance, polyline_meets_area, polylines_meet};
use super::{
    ArgValue, CheckContext, Finding, Location, RuleRun, SKIP_SCORE_NO_MAP, SKIP_SCORE_TRAVEL,
    SKIP_VALUE_MISSING, ScoreCourseContext, Skipped, value_number,
};
use crate::rules::Severity;
use crate::score::ScoreError;
use crate::score::code::{reads_alike_turned, value_of_code};
use crate::score::tour::{LengthIssue, ScoredControl, analyse_length};
use crate::score::travel::{CostSurface, LegRoutes, Pace, RouteParams, SharedRoutes, TravelTime};

/// Fewest controls from which the ring and the order tests judge a set; smaller sets have no
/// shape to speak of.
const MIN_SHAPED: usize = 6;
/// Fewest controls from which the mix is judged; with fewer, one control moves the share by more
/// than the tolerance.
const MIN_MIXED: usize = 5;

fn control_location(code: &ControlCode) -> Location {
    Location::Control { code: code.clone() }
}

fn text(s: &str) -> ArgValue {
    ArgValue::Text(s.to_owned())
}

fn missing(run: &RuleRun<'_>, name: &str, skipped: &mut Vec<Skipped>) {
    skipped.push(run.skipped(
        Location::Event,
        SKIP_VALUE_MISSING,
        vec![("name", text(name))],
    ));
}

/// `score_value_matches_code`: the value of a control equals the digit its code carries (the
/// second digit of a two-digit code, the last of a longer one), see ADR 0060. A code that carries
/// no value (letters, one digit, a last digit 0) is a finding of its own kind.
pub(super) fn value_matches_code(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    _skipped: &mut Vec<Skipped>,
) {
    for course in &ctx.score_courses {
        for control in &course.controls {
            let location = control_location(&control.code);
            let value = ArgValue::Int(i64::from(control.value));
            let name = text(&course.course.name);
            match value_of_code(&control.code.0) {
                Some(v) if u32::from(v) == control.value => {}
                Some(v) => findings.push(run.finding(
                    location,
                    "check-score-value-mismatch",
                    vec![
                        ("code", text(&control.code.0)),
                        ("expected", ArgValue::Int(i64::from(v))),
                        ("value", value),
                        ("course", name),
                    ],
                )),
                None => findings.push(run.finding(
                    location,
                    "check-score-value-no-code",
                    vec![
                        ("code", text(&control.code.0)),
                        ("value", value),
                        ("course", name),
                    ],
                )),
            }
        }
    }
}

/// `score_code_ambiguity`: two controls of one score course whose codes look alike turned upside
/// down or differ only by 6 and 9 (`reads_alike_turned`). A score code carries the value, so a
/// confused pair is a wrong punch with a wrong value. One finding per pair and course.
pub(super) fn code_ambiguity(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    _skipped: &mut Vec<Skipped>,
) {
    for course in &ctx.score_courses {
        let controls = &course.controls;
        for (i, a) in controls.iter().enumerate() {
            for b in &controls[i + 1..] {
                if reads_alike_turned(&a.code.0, &b.code.0) {
                    let (first, second) = if a.code.0 <= b.code.0 {
                        (&a.code, &b.code)
                    } else {
                        (&b.code, &a.code)
                    };
                    findings.push(run.finding(
                        Location::Controls {
                            codes: vec![first.clone(), second.clone()],
                        },
                        "check-score-code-alike",
                        vec![
                            ("first", text(&first.0)),
                            ("second", text(&second.0)),
                            ("course", text(&course.course.name)),
                        ],
                    ));
                }
            }
        }
    }
}

/// The largest angle in degrees between two neighbouring bearings of `points` seen from `centre`,
/// and the variation of their distances (standard deviation over the mean, in percent).
fn ring_measures(points: &[&MapPoint], centre: &MapPoint) -> (f64, f64) {
    let mut angles: Vec<f64> = points
        .iter()
        .map(|p| (p.y - centre.y).atan2(p.x - centre.x).to_degrees())
        .collect();
    angles.sort_by(f64::total_cmp);
    let mut gap = angles[0] + 360.0 - angles[angles.len() - 1];
    for pair in angles.windows(2) {
        gap = gap.max(pair[1] - pair[0]);
    }
    let radii: Vec<f64> = points.iter().map(|p| distance(p, centre)).collect();
    let mean = radii.iter().sum::<f64>() / radii.len() as f64;
    let variance = radii.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / radii.len() as f64;
    let variation = if mean > 0.0 {
        variance.sqrt() / mean * 100.0
    } else {
        0.0
    };
    (gap, variation)
}

/// `score_ring`: the controls stand in a ring around start and finish. Seen from the middle of
/// start and finish, the largest gap between the bearings of neighbouring controls is below
/// `max_gap_degrees` (the controls enclose the middle) and the distances to the middle vary by
/// less than `max_radius_variation_percent` (standard deviation over mean). Such a set has one
/// obvious way round. Needs at least six controls (BOF B12.3 and B12.6.4; the measure is this program's).
pub(super) fn ring(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(max_gap) = value_number(run.rule(), "max_gap_degrees") else {
        missing(run, "max_gap_degrees", skipped);
        return;
    };
    let Some(max_variation) = value_number(run.rule(), "max_radius_variation_percent") else {
        missing(run, "max_radius_variation_percent", skipped);
        return;
    };
    for course in &ctx.score_courses {
        if course.controls.len() < MIN_SHAPED {
            continue;
        }
        let centre = MapPoint {
            x: (course.start.x + course.finish.x) / 2.0,
            y: (course.start.y + course.finish.y) / 2.0,
        };
        let points: Vec<&MapPoint> = course.controls.iter().map(|c| &c.at).collect();
        let (gap, variation) = ring_measures(&points, &centre);
        if gap < max_gap && variation < max_variation {
            findings.push(run.finding(
                course.location(),
                "check-score-ring",
                vec![
                    ("course", text(&course.course.name)),
                    ("count", ArgValue::Int(points.len() as i64)),
                    ("gap", ArgValue::Number(gap)),
                    ("variation", ArgValue::Number(variation)),
                ],
            ));
        }
    }
}

fn cross(o: &MapPoint, a: &MapPoint, b: &MapPoint) -> f64 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// The indices of the points on the convex hull, collinear points left out.
fn hull(points: &[&MapPoint]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..points.len()).collect();
    order.sort_by(|&a, &b| {
        points[a]
            .x
            .total_cmp(&points[b].x)
            .then(points[a].y.total_cmp(&points[b].y))
    });
    let mut chain: Vec<usize> = Vec::new();
    let build = |chain: &mut Vec<usize>, i: usize, floor: usize| {
        while chain.len() >= floor
            && cross(
                points[chain[chain.len() - 2]],
                points[chain[chain.len() - 1]],
                points[i],
            ) <= 0.0
        {
            chain.pop();
        }
        chain.push(i);
    };
    for &i in &order {
        build(&mut chain, i, 2);
    }
    let lower = chain.len() + 1;
    for &i in order.iter().rev().skip(1) {
        build(&mut chain, i, lower);
    }
    chain.pop();
    chain
}

fn path_length(path: &[usize], at: &[&MapPoint]) -> f64 {
    path.windows(2).map(|w| distance(at[w[0]], at[w[1]])).sum()
}

/// The length of the path that always runs to the nearest control not yet taken, from the
/// start through all controls to the finish, and the length after 2-opt improvement (the shortest
/// order this program finds). Nodes 0..n are the controls, n the start, n+1 the finish.
fn chain_lengths(points: &[&MapPoint], start: &MapPoint, finish: &MapPoint) -> (f64, f64) {
    let n = points.len();
    let mut at: Vec<&MapPoint> = points.to_vec();
    at.push(start);
    at.push(finish);
    let mut path = vec![n];
    let mut left: Vec<usize> = (0..n).collect();
    while !left.is_empty() {
        let here = *path.last().expect("path starts with the start");
        let (pos, _) = left
            .iter()
            .enumerate()
            .min_by(|&(_, &a), &(_, &b)| {
                distance(at[here], at[a])
                    .total_cmp(&distance(at[here], at[b]))
                    .then(a.cmp(&b))
            })
            .expect("left is not empty");
        path.push(left.remove(pos));
    }
    path.push(n + 1);
    let nearest = path_length(&path, &at);
    let mut improved = true;
    while improved {
        improved = false;
        for i in 1..path.len() - 1 {
            for j in i + 1..path.len() - 1 {
                let before =
                    distance(at[path[i - 1]], at[path[i]]) + distance(at[path[j]], at[path[j + 1]]);
                let after =
                    distance(at[path[i - 1]], at[path[j]]) + distance(at[path[i]], at[path[j + 1]]);
                if after < before - 1e-9 {
                    path[i..=j].reverse();
                    improved = true;
                }
            }
        }
    }
    (nearest, path_length(&path, &at))
}

/// `score_obvious_order`: the set has an order everybody finds. Two measures must both hold: at
/// least `min_hull_percent` of the controls are corners of the convex hull of the controls (a
/// loop round the edge), and running always to the nearest control from the start ends at most
/// `max_chain_excess_percent` above the shortest order found by 2-opt. Needs at least six
/// controls (BOF B12.3 and B12.6.4; the measures are this program's).
pub(super) fn obvious_order(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(min_hull) = value_number(run.rule(), "min_hull_percent") else {
        missing(run, "min_hull_percent", skipped);
        return;
    };
    let Some(max_excess) = value_number(run.rule(), "max_chain_excess_percent") else {
        missing(run, "max_chain_excess_percent", skipped);
        return;
    };
    for course in &ctx.score_courses {
        if course.controls.len() < MIN_SHAPED {
            continue;
        }
        let points: Vec<&MapPoint> = course.controls.iter().map(|c| &c.at).collect();
        let hull_percent = hull(&points).len() as f64 / points.len() as f64 * 100.0;
        let (nearest, shortest) = chain_lengths(&points, &course.start, &course.finish);
        let excess = if shortest > 0.0 {
            (nearest / shortest - 1.0) * 100.0
        } else {
            0.0
        };
        if hull_percent >= min_hull && excess <= max_excess {
            findings.push(run.finding(
                course.location(),
                "check-score-obvious-order",
                vec![
                    ("course", text(&course.course.name)),
                    ("hull", ArgValue::Number(hull_percent)),
                    ("excess", ArgValue::Number(excess)),
                ],
            ));
        }
    }
}

/// The map polygons a runner of `course` cannot cross: the out-of-bounds and dangerous areas of
/// the event that apply to it.
fn blocked_areas(ctx: &CheckContext<'_>, course: &ScoreCourseContext<'_>) -> Vec<Vec<MapPoint>> {
    let name = &course.course.name;
    ctx.event
        .specials
        .iter()
        .filter_map(|s| match s {
            SpecialObject::OutOfBoundsArea {
                polygon, courses, ..
            }
            | SpecialObject::DangerousArea {
                polygon, courses, ..
            } if courses.is_empty() || courses.contains(name) => Some(polygon.clone()),
            _ => None,
        })
        .collect()
}

/// The cost surface of the map at the pace of the good runner, or the skip that says why none
/// can be built. The route search is this program's first part of the route analysis (ADR 0061).
fn surface_for(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    course: &ScoreCourseContext<'_>,
    settings: &ScoreSettings,
    params: &RouteParams,
) -> Result<CostSurface, Box<Skipped>> {
    if ctx.index.is_empty() {
        return Err(Box::new(run.skipped(
            course.location(),
            SKIP_SCORE_NO_MAP,
            Vec::new(),
        )));
    }
    CostSurface::build(
        &ctx.index,
        ctx.map_scale,
        &blocked_areas(ctx, course),
        Pace(settings.good_pace_s_per_km),
        params,
    )
    .map_err(|e| Box::new(travel_skip(run, course, &e)))
}

fn travel_skip(run: &RuleRun<'_>, course: &ScoreCourseContext<'_>, error: &ScoreError) -> Skipped {
    match error {
        ScoreError::MapEmpty => run.skipped(course.location(), SKIP_SCORE_NO_MAP, Vec::new()),
        other => run.skipped(
            course.location(),
            SKIP_SCORE_TRAVEL,
            vec![("error", text(&other.to_string()))],
        ),
    }
}

/// The links of a course as index pairs into its controls; links that name a control the course
/// lacks, join a control to itself or repeat another one are left out (`score_link_valid` reports
/// them).
fn link_indices(course: &ScoreCourseContext<'_>) -> Vec<(usize, usize)> {
    let index = |code: &ControlCode| course.controls.iter().position(|c| &c.code == code);
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (a, b) in &course.links {
        if let (Some(i), Some(j)) = (index(a), index(b))
            && i != j
            && !out.contains(&(i, j))
            && !out.contains(&(j, i))
        {
            out.push((i, j));
        }
    }
    out
}

/// `score_length_bands`: the best round trip of the good runner within the limit scores a share of
/// all points inside `good_band_percent` of the event's score settings, and the very good runner
/// reaches every control and finishes inside `very_good_band_percent` of the limit (user, design
/// §3 and §9 ruling 1). It uses `analyse_length` on a cost surface of the map built as the draft
/// does, at the grid edge of the rule's `cell_metres`. Skipped when the map has no objects or no
/// route search works on it. One search per pair of points serves both runners, so a course of 60
/// controls takes seconds, not minutes.
pub(super) fn length_bands(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(cell_m) = value_number(run.rule(), "cell_metres") else {
        missing(run, "cell_metres", skipped);
        return;
    };
    let settings = ctx.event.score_settings();
    let params = RouteParams {
        cell_m,
        ..RouteParams::default()
    };
    for course in &ctx.score_courses {
        if course.controls.is_empty() {
            continue;
        }
        let surface = match surface_for(ctx, run, course, &settings, &params) {
            Ok(surface) => surface,
            Err(skip) => {
                skipped.push(*skip);
                continue;
            }
        };
        let shared = SharedRoutes::new(surface);
        let good = shared.profile(1.0);
        let very_good = shared.profile(
            f64::from(settings.very_good_pace_s_per_km)
                / f64::from(settings.good_pace_s_per_km.max(1)),
        );
        let scored: Vec<ScoredControl> = course
            .controls
            .iter()
            .map(|c| ScoredControl {
                at: c.at.clone(),
                value: u8::try_from(c.value).unwrap_or(u8::MAX),
            })
            .collect();
        let report = analyse_length(
            &scored,
            &link_indices(course),
            &good,
            &very_good,
            course.start.clone(),
            course.finish.clone(),
            f64::from(course.time_limit_min) * 60.0,
            &settings,
        );
        let report = match report {
            Ok(report) => report,
            Err(error) => {
                skipped.push(travel_skip(run, course, &error));
                continue;
            }
        };
        let name = text(&course.course.name);
        let band = |b: (u8, u8)| {
            [
                ("low", ArgValue::Int(i64::from(b.0))),
                ("high", ArgValue::Int(i64::from(b.1))),
            ]
        };
        for issue in &report.issues {
            let (id, mut args) = match issue {
                LengthIssue::GoodBelow { percent, band: b } => {
                    let mut a = vec![("percent", ArgValue::Number(*percent))];
                    a.extend(band(*b));
                    ("check-score-length-good-below", a)
                }
                LengthIssue::GoodAbove { percent, band: b } => {
                    let mut a = vec![("percent", ArgValue::Number(*percent))];
                    a.extend(band(*b));
                    ("check-score-length-good-above", a)
                }
                LengthIssue::VeryGoodMissesControls { missing } => (
                    "check-score-length-very-good-misses",
                    vec![("missing", ArgValue::Int(i64::from(*missing)))],
                ),
                LengthIssue::VeryGoodOverLimit { percent } => (
                    "check-score-length-very-good-over",
                    vec![("percent", ArgValue::Number(*percent))],
                ),
                LengthIssue::VeryGoodTooEarly { percent, band: b } => {
                    let mut a = vec![("percent", ArgValue::Number(*percent))];
                    a.extend(band(*b));
                    ("check-score-length-very-good-early", a)
                }
            };
            args.push(("course", name.clone()));
            findings.push(run.finding(course.location(), id, args));
        }
    }
}

/// `score_limit_and_penalty`: the time limit lies in `min_minutes` to `max_minutes` (30 to 90
/// is usual), and the penalty per started minute is not below the highest control value of the
/// course; otherwise a minute over the limit can pay off (BOF B12.5).
pub(super) fn limit_and_penalty(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(min) = value_number(run.rule(), "min_minutes") else {
        missing(run, "min_minutes", skipped);
        return;
    };
    let Some(max) = value_number(run.rule(), "max_minutes") else {
        missing(run, "max_minutes", skipped);
        return;
    };
    for course in &ctx.score_courses {
        let name = text(&course.course.name);
        let limit = f64::from(course.time_limit_min);
        if limit < min - 1e-9 || limit > max + 1e-9 {
            findings.push(run.finding(
                course.location(),
                "check-score-limit-range",
                vec![
                    ("course", name.clone()),
                    ("limit", ArgValue::Int(i64::from(course.time_limit_min))),
                    ("min", ArgValue::Number(min)),
                    ("max", ArgValue::Number(max)),
                ],
            ));
        }
        let highest = course.controls.iter().map(|c| c.value).max().unwrap_or(0);
        if course.penalty_per_min < highest {
            findings.push(run.finding(
                course.location(),
                "check-score-penalty-pays",
                vec![
                    ("course", name),
                    ("penalty", ArgValue::Int(i64::from(course.penalty_per_min))),
                    ("highest", ArgValue::Int(i64::from(highest))),
                ],
            ));
        }
    }
}

fn pair_location(a: &ControlCode, b: &ControlCode) -> Location {
    let (first, second) = if a.0 <= b.0 { (a, b) } else { (b, a) };
    Location::Controls {
        codes: vec![first.clone(), second.clone()],
    }
}

/// `score_link_valid`: each mandatory link joins two different controls of the course, appears
/// once (A-B and B-A are one link), and its straight line crosses no out-of-bounds area and no
/// forbidden route of the course (a warning, as for a leg of an individual course); a leg without
/// a second route of similar length is a hint. The runner's route is not a straight line, so
/// both are hints about the link, not proof. There is no flag for "no route choice" in the
/// project file: the setter accepts that finding with a note. Needs a map with objects for the
/// route-choice part, which is skipped otherwise (ADR 0066).
pub(super) fn link_valid(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let settings = ctx.event.score_settings();
    let params = RouteParams::default();
    for course in &ctx.score_courses {
        let name = text(&course.course.name);
        let has = |code: &ControlCode| course.controls.iter().any(|c| &c.code == code);
        let at = |code: &ControlCode| {
            course
                .controls
                .iter()
                .find(|c| &c.code == code)
                .map(|c| c.at.clone())
        };
        let obstacles = obstacles(ctx, course);
        let mut seen: Vec<(&ControlCode, &ControlCode)> = Vec::new();
        let mut surface: Option<Result<CostSurface, ()>> = None;
        for (a, b) in &course.links {
            let args = |extra: Vec<(&'static str, ArgValue)>| {
                let mut v = vec![
                    ("first", text(&a.0)),
                    ("second", text(&b.0)),
                    ("course", name.clone()),
                ];
                v.extend(extra);
                v
            };
            if a == b {
                findings.push(run.finding(
                    control_location(a),
                    "check-score-link-self",
                    vec![("code", text(&a.0)), ("course", name.clone())],
                ));
                continue;
            }
            if let Some(gone) = [a, b].into_iter().find(|c| !has(c)) {
                findings.push(run.finding(
                    pair_location(a, b),
                    "check-score-link-not-in-course",
                    args(vec![("missing", text(&gone.0))]),
                ));
                continue;
            }
            if seen.contains(&(a, b)) || seen.contains(&(b, a)) {
                findings.push(run.finding(
                    pair_location(a, b),
                    "check-score-link-duplicate",
                    args(Vec::new()),
                ));
                continue;
            }
            seen.push((a, b));
            let (Some(pa), Some(pb)) = (at(a), at(b)) else {
                continue;
            };
            let line = [pa.clone(), pb.clone()];
            if let Some(hit) = obstacles.iter().find(|o| o.met_by(&line)) {
                findings.push(run.finding_with_severity(
                    Severity::Warning,
                    pair_location(a, b),
                    hit.message(),
                    args(vec![("id", text(hit.id()))]),
                ));
            }
            // The route choice of the leg, from one surface per course.
            let built = surface.get_or_insert_with(|| {
                match surface_for(ctx, run, course, &settings, &params) {
                    Ok(s) => Ok(s),
                    Err(skip) => {
                        skipped.push(*skip);
                        Err(())
                    }
                }
            });
            let Ok(surface) = built else {
                continue;
            };
            match surface.legs(pa, pb) {
                Ok(legs) => {
                    if !has_choice(&legs, params.max_length_ratio) {
                        findings.push(run.finding_with_severity(
                            Severity::Hint,
                            pair_location(a, b),
                            "check-score-link-no-choice",
                            args(Vec::new()),
                        ));
                    }
                }
                Err(error) => skipped.push(travel_skip(run, course, &error)),
            }
        }
    }
}

/// Whether a leg offers a second route no longer than `ratio` times the fastest.
fn has_choice(legs: &LegRoutes, ratio: f64) -> bool {
    let Some(best) = legs.routes.first() else {
        return false;
    };
    legs.routes
        .iter()
        .skip(1)
        .any(|r| r.length_m <= best.length_m * ratio)
}

/// An out-of-bounds area or forbidden route a link should not cross.
enum Obstacle<'a> {
    Area { id: &'a str, ring: Vec<MapPoint> },
    Route { id: &'a str, points: &'a [MapPoint] },
}

impl Obstacle<'_> {
    fn met_by(&self, line: &[MapPoint]) -> bool {
        match self {
            Obstacle::Area { ring, .. } => polyline_meets_area(line, std::slice::from_ref(ring)),
            Obstacle::Route { points, .. } => polylines_meet(line, points, false),
        }
    }

    fn id(&self) -> &str {
        match self {
            Obstacle::Area { id, .. } | Obstacle::Route { id, .. } => id,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Obstacle::Area { .. } => "check-score-link-out-of-bounds",
            Obstacle::Route { .. } => "check-score-link-forbidden-route",
        }
    }
}

fn obstacles<'a>(ctx: &'a CheckContext<'_>, course: &ScoreCourseContext<'_>) -> Vec<Obstacle<'a>> {
    let name = &course.course.name;
    let applies = |courses: &[String]| courses.is_empty() || courses.contains(name);
    ctx.event
        .specials
        .iter()
        .filter_map(|s| match s {
            SpecialObject::OutOfBoundsArea {
                id,
                polygon,
                courses,
                ..
            } if applies(courses) => Some(Obstacle::Area {
                id,
                ring: polygon.clone(),
            }),
            SpecialObject::ForbiddenRoute {
                id,
                points,
                courses,
                ..
            } if applies(courses) => Some(Obstacle::Route { id, points }),
            _ => None,
        })
        .collect()
}

/// Whether a control counts as hard for the mix: its value lies in the top two fifths of the range
/// 1 to `value_max` (`value × 5 > value_max × 3`: 4 and 5 of 5, 6 to 9 of 9). The draft's candidates carry a
/// difficulty, a saved course does not, so the value stands in for it.
fn is_hard(value: u32, value_max: u8) -> bool {
    u64::from(value) * 5 > u64::from(value_max) * 3
}

/// `score_mix`: the share of hard controls differs from `mix_hard_percent` of the score settings
/// by more than `tolerance_percent` points. Courses with fewer than five controls are not judged.
pub(super) fn mix(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(tolerance) = value_number(run.rule(), "tolerance_percent") else {
        missing(run, "tolerance_percent", skipped);
        return;
    };
    let settings = ctx.event.score_settings();
    let wanted = f64::from(settings.mix_hard_percent);
    for course in &ctx.score_courses {
        let count = course.controls.len();
        if count < MIN_MIXED {
            continue;
        }
        let hard = course
            .controls
            .iter()
            .filter(|c| is_hard(c.value, settings.value_max))
            .count();
        let actual = hard as f64 / count as f64 * 100.0;
        if (actual - wanted).abs() > tolerance + 1e-9 {
            findings.push(run.finding(
                course.location(),
                "check-score-mix",
                vec![
                    ("course", text(&course.course.name)),
                    ("hard", ArgValue::Int(hard as i64)),
                    ("count", ArgValue::Int(count as i64)),
                    ("actual", ArgValue::Number(actual)),
                    (
                        "wanted",
                        ArgValue::Int(i64::from(settings.mix_hard_percent)),
                    ),
                    ("tolerance", ArgValue::Number(tolerance)),
                ],
            ));
        }
    }
}

#[cfg(test)]
mod tests;
