//! Control candidates: clear sites on the map with route choice, difficulty and effort (ADR 0062).

use freemapper_courseplan_model::{MapPoint, SpecialObject};

use std::collections::BTreeMap;

use crate::check::symbol_matches;
use crate::map::{FeatureGeometry, MapIndex, inside_area};
use crate::score::ScoreError;
use crate::score::travel::{LegRoutes, Route, RouteParams, TravelTime};

/// A site where a control could stand, with the numbers the score builder works from.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub at: MapPoint,
    /// ISSprOM code of the feature the site belongs to.
    pub symbol_code: Option<String>,
    /// 0.0 to 1.0: how many of the legs to other candidates and to the start offer several similar routes.
    pub route_choice: f64,
    /// 0.0 to 1.0: harder for little route choice, an unclear feature and crowded surroundings.
    pub difficulty: f64,
    /// Seconds of the fastest route from the start.
    pub effort_s: f64,
}

/// What the suggestion needs besides the map and the travel time.
#[derive(Debug, Clone)]
pub struct SuggestContext<'a> {
    pub start: MapPoint,
    pub finish: MapPoint,
    pub specials: &'a [SpecialObject],
    /// Smallest distance in map millimetres between two controls and to start and finish.
    pub min_distance_mm: f64,
    /// Sites closer than this to the start or the finish are left out, in map millimetres, so value 1
    /// starts at that distance (ADR 0072). Zero keeps all.
    pub min_end_distance_mm: f64,
    /// Sites farther than this from the start are left out, in map millimetres; none keeps all.
    pub max_reach_mm: Option<f64>,
    /// Controls that stand already.
    pub existing: &'a [MapPoint],
    /// Most candidates returned.
    pub limit: usize,
    /// Sites closer than this to a forbidden-route line are left out, in map millimetres
    /// ([`FORBIDDEN_LINE_MARGIN_M`] on the ground).
    pub line_margin_mm: f64,
}

/// Distance in metres on the ground from a forbidden-route line within which a site is rejected: about
/// one cell of the fine cost surface. The surface itself blocks areas only (ADR 0068).
pub const FORBIDDEN_LINE_MARGIN_M: f64 = 4.0;

/// Weights of difficulty (sum 1): little route choice, an unclear feature, crowded surroundings.
const W_NO_CHOICE: f64 = 0.5;
const W_UNCLEAR: f64 = 0.3;
const W_DENSITY: f64 = 0.2;
/// Neighbours (other raw sites) within this many map millimetres make a site crowded.
const DENSITY_RADIUS_MM: f64 = 5.0;
/// This many neighbours count as fully crowded.
const DENSITY_FULL: f64 = 6.0;
/// Turn of an outline, in degrees, from which a vertex is a corner.
const CORNER_DEGREES: f64 = 45.0;
/// Routes that make the choice full: three similar routes.
const FULL_CHOICE_ROUTES: usize = 3;
/// Share of the route-choice value lost when the second route is as long as the limit allows.
const CLEARER_PENALTY: f64 = 0.5;
/// Reachable candidates wanted per `limit`, so the best `limit` can be picked from them.
const SITES_PER_LIMIT: usize = 2;
/// Sites tried per `limit` at most (unreachable ones are replaced by further sites).
const ATTEMPTS_PER_LIMIT: usize = 4;
/// Other candidates each candidate gets a leg to.
const NEAREST_LEGS: usize = 4;
/// How much sparser the picks get from the start to the farthest site: the spacing a site needs grows
/// linearly from 1 at the start to `1 + SPACING_GROWTH` at the far end, so the area around the start
/// gets close picks (value 1 stays near) and the rim gets few (ADR 0071).
const SPACING_GROWTH: f64 = 2.0;
/// Sites closer to the rim of the mapped area than this many minimum distances are weighed down
/// linearly, to a floor of `RIM_FLOOR` at the rim itself.
const RIM_MARGIN_DISTANCES: f64 = 3.0;
const RIM_FLOOR: f64 = 0.3;
/// Share of the minimum distance within which a site that could not be routed pushes other sites away.
const LOST_PUSH_SHARE: f64 = 0.3;
/// A corner may lie in the blocked cell of its wall; it needs a free place at one of these distances (map mm).
const CORNER_PROBE_MM: [f64; 3] = [0.5, 1.0, 1.5];

/// Where on a feature a site lies (ADR 0062, ADR 0073).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// A point symbol, at its position.
    Point,
    /// A vertex of a line or area outline with a turn of `CORNER_DEGREES` or more.
    Corner,
    /// A free end of an open line: no other line of the ends and junctions family touches it.
    End,
    /// A line end that meets another line of that family: a fork, or a branch onto a path.
    Junction,
}

struct Classes(Vec<(String, Kind, f64)>);

impl Classes {
    /// Symbol code, kind, clarity; read once from `candidates.toml`.
    fn embedded() -> Classes {
        let doc: toml::Table = include_str!("candidates.toml")
            .parse()
            .expect("candidates.toml is valid TOML");
        let mut rows = Vec::new();
        for class in doc["class"]
            .as_array()
            .expect("candidates.toml has classes")
        {
            let kind = match class["kind"].as_str().expect("class has a kind") {
                "point" => Kind::Point,
                "corner" => Kind::Corner,
                "end" => Kind::End,
                "junction" => Kind::Junction,
                other => panic!("candidates.toml: unknown kind {other}"),
            };
            let clarity = class["clarity"].as_float().expect("class has a clarity");
            for code in class["codes"].as_array().expect("class has codes") {
                rows.push((
                    code.as_str().expect("code is text").to_owned(),
                    kind,
                    clarity,
                ));
            }
        }
        Classes(rows)
    }

    fn lookup(&self, symbol: &str, kind: Kind) -> Option<f64> {
        self.0
            .iter()
            .filter(|(code, k, _)| *k == kind && symbol_matches(symbol, code))
            .max_by_key(|(code, _, _)| code.len())
            .map(|(_, _, clarity)| *clarity)
    }
}

#[derive(Debug, Clone)]
struct Site {
    at: MapPoint,
    symbol: String,
    clarity: f64,
    /// Lies on a line or outline, so the cell may be blocked and a free place nearby is probed.
    on_outline: bool,
}

fn dist(a: &MapPoint, b: &MapPoint) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn is_corner(prev: &MapPoint, at: &MapPoint, next: &MapPoint) -> bool {
    let a = (at.y - prev.y).atan2(at.x - prev.x);
    let b = (next.y - at.y).atan2(next.x - at.x);
    let mut turn = (b - a).abs();
    if turn > std::f64::consts::PI {
        turn = 2.0 * std::f64::consts::PI - turn;
    }
    turn.to_degrees() >= CORNER_DEGREES
}

fn corners(points: &[MapPoint], closed: bool) -> Vec<MapPoint> {
    let mut pts = points.to_vec();
    if closed && pts.len() > 1 && pts.first() == pts.last() {
        pts.pop();
    }
    let n = pts.len();
    let mut out = Vec::new();
    for i in 0..n {
        let (prev, next) = if closed {
            (&pts[(i + n - 1) % n], &pts[(i + 1) % n])
        } else if i > 0 && i + 1 < n {
            (&pts[i - 1], &pts[i + 1])
        } else {
            continue;
        };
        if n >= 3 && is_corner(prev, &pts[i], next) {
            out.push(pts[i].clone());
        }
    }
    out
}

/// Distance in map millimetres within which a line end counts as touching another line.
const TOUCH_MM: f64 = 0.3;

/// The open lines that have ends or junctions in `candidates.toml`, to tell free ends from forks.
struct LineFamily<'a> {
    lines: Vec<&'a [MapPoint]>,
}

/// How a line end stands to the other lines of the family.
enum EndKind {
    Free,
    /// Meets the end of one other line only: one path cut in two.
    Continuation,
    /// Meets the inside of another line, or two or more others.
    Fork,
}

impl LineFamily<'_> {
    fn classify(&self, own: usize, at: &MapPoint) -> EndKind {
        let (mut ends, mut inside) = (0, 0);
        for (i, line) in self.lines.iter().enumerate() {
            if i == own
                || !line
                    .windows(2)
                    .any(|w| segment_distance(at, &w[0], &w[1]) <= TOUCH_MM)
            {
                continue;
            }
            let at_its_end = [line.first(), line.last()]
                .into_iter()
                .flatten()
                .any(|e| dist(at, e) <= TOUCH_MM);
            if at_its_end {
                ends += 1;
            } else {
                inside += 1;
            }
        }
        match (ends, inside) {
            (0, 0) => EndKind::Free,
            (1, 0) => EndKind::Continuation,
            _ => EndKind::Fork,
        }
    }
}

/// Every site the map offers, with duplicate positions (0.01 mm) reduced to the clearest.
fn raw_sites(index: &MapIndex) -> Vec<Site> {
    let classes = Classes::embedded();
    let in_family = |code: &str| {
        classes.lookup(code, Kind::End).is_some() || classes.lookup(code, Kind::Junction).is_some()
    };
    let mut family = LineFamily { lines: Vec::new() };
    let mut family_of: Vec<Option<usize>> = Vec::new();
    for f in index.features() {
        match (&f.geometry, f.symbol_code.as_deref()) {
            (FeatureGeometry::Line(points), Some(code))
                if points.len() >= 2 && points.first() != points.last() && in_family(code) =>
            {
                family_of.push(Some(family.lines.len()));
                family.lines.push(points);
            }
            _ => family_of.push(None),
        }
    }
    let mut sites: Vec<Site> = Vec::new();
    for (n, f) in index.features().iter().enumerate() {
        let Some(code) = f.symbol_code.as_deref() else {
            continue;
        };
        let mut push = |at: MapPoint, clarity: f64, on_outline: bool| {
            sites.push(Site {
                at,
                symbol: code.to_owned(),
                clarity,
                on_outline,
            });
        };
        match &f.geometry {
            FeatureGeometry::Point(at) => {
                if let Some(c) = classes.lookup(code, Kind::Point) {
                    push(at.clone(), c, false);
                }
            }
            FeatureGeometry::Line(points) => {
                if let Some(c) = classes.lookup(code, Kind::Corner) {
                    corners(points, points.len() > 2 && points.first() == points.last())
                        .into_iter()
                        .for_each(|at| push(at, c, true));
                }
                if let Some(own) = family_of[n] {
                    for at in [&points[0], &points[points.len() - 1]] {
                        let kind = match family.classify(own, at) {
                            EndKind::Free => Kind::End,
                            EndKind::Fork => Kind::Junction,
                            EndKind::Continuation => continue,
                        };
                        if let Some(c) = classes.lookup(code, kind) {
                            push(at.clone(), c, true);
                        }
                    }
                }
            }
            FeatureGeometry::Area(rings) => {
                if let Some(c) = classes.lookup(code, Kind::Corner) {
                    for ring in rings {
                        corners(ring, true)
                            .into_iter()
                            .for_each(|at| push(at, c, true));
                    }
                }
            }
        }
    }
    let key = |s: &Site| {
        (
            (s.at.x * 100.0).round() as i64,
            (s.at.y * 100.0).round() as i64,
        )
    };
    sites.sort_by(|a, b| {
        key(a)
            .cmp(&key(b))
            .then(b.clarity.total_cmp(&a.clarity))
            .then(a.symbol.cmp(&b.symbol))
    });
    sites.dedup_by(|b, a| key(a) == key(b));
    sites
}

/// Distance from `p` to the segment `a` to `b`.
fn segment_distance(p: &MapPoint, a: &MapPoint, b: &MapPoint) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0)
    };
    dist(
        p,
        &MapPoint {
            x: a.x + t * dx,
            y: a.y + t * dy,
        },
    )
}

fn forbidden(specials: &[SpecialObject], at: &MapPoint, line_margin_mm: f64) -> bool {
    specials.iter().any(|s| match s {
        SpecialObject::OutOfBoundsArea { polygon, .. }
        | SpecialObject::DangerousArea { polygon, .. } => {
            inside_area(at, std::slice::from_ref(polygon))
        }
        SpecialObject::ForbiddenRoute { points, .. } => points
            .windows(2)
            .any(|w| segment_distance(at, &w[0], &w[1]) <= line_margin_mm),
        _ => false,
    })
}

/// 0.0 for one route; else the richness (two routes 0.5, three or more 1.0) less a penalty when the second
/// route is clearly longer than the first.
fn leg_choice(legs: &LegRoutes, max_ratio: f64) -> f64 {
    let Some(best) = legs.routes.first() else {
        return 0.0;
    };
    let similar: Vec<&Route> = legs
        .routes
        .iter()
        .filter(|r| r.length_m <= best.length_m * max_ratio)
        .collect();
    if similar.len() < 2 {
        return 0.0;
    }
    let richness =
        (similar.len().min(FULL_CHOICE_ROUTES) - 1) as f64 / (FULL_CHOICE_ROUTES - 1) as f64;
    let spread = if max_ratio > 1.0 && best.length_m > 0.0 {
        ((similar[1].length_m / best.length_m - 1.0) / (max_ratio - 1.0)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    richness * (1.0 - CLEARER_PENALTY * spread)
}

/// Where routes to and from a site end: the site itself when a runner can stand there, else (corners
/// only) the nearest free place within `CORNER_PROBE_MM`; none when there is no standing room.
fn route_end(travel: &dyn TravelTime, site: &Site) -> Option<MapPoint> {
    if travel.passable(site.at.clone()) {
        return Some(site.at.clone());
    }
    if !site.on_outline {
        return None;
    }
    CORNER_PROBE_MM.iter().find_map(|r| {
        (0..8).find_map(|k| {
            let a = f64::from(k) * std::f64::consts::FRAC_PI_4;
            let q = MapPoint {
                x: site.at.x + r * a.cos(),
                y: site.at.y + r * a.sin(),
            };
            travel.passable(q.clone()).then_some(q)
        })
    })
}

/// Start or finish must be reachable: a point in a blocked cell gets one probe leg, which snaps it.
fn check_end(
    travel: &dyn TravelTime,
    at: &MapPoint,
    error: fn(MapPoint) -> ScoreError,
) -> Result<(), ScoreError> {
    if travel.passable(at.clone()) {
        return Ok(());
    }
    match travel.legs(at.clone(), at.clone()) {
        Ok(_) => Ok(()),
        Err(ScoreError::Unreachable { .. }) => Err(error(at.clone())),
        Err(e) => Err(e),
    }
}

/// What a site is worth in the pick, per millimetre of distance to the nearest pick: clear features
/// first, sites far from the start need more room, sites at the rim count less.
fn site_weight(clarity: f64, spacing: f64, rim: f64) -> f64 {
    clarity * rim / spacing
}

/// Route choice of the leg between two points; none when the surface finds no route.
fn choice_of(
    travel: &dyn TravelTime,
    from: &MapPoint,
    to: &MapPoint,
    max_ratio: f64,
) -> Result<Option<f64>, ScoreError> {
    match travel.legs(from.clone(), to.clone()) {
        Ok(legs) => Ok(Some(leg_choice(&legs, max_ratio))),
        Err(ScoreError::Unreachable { .. }) => Ok(None),
        Err(e) => Err(e),
    }
}

/// The candidates for control sites, best route choice first, ties by position. See [`suggest_with`].
pub fn suggest(
    index: &MapIndex,
    travel: &dyn TravelTime,
    ctx: &SuggestContext<'_>,
) -> Result<Vec<Candidate>, ScoreError> {
    suggest_with(index, travel, travel, ctx)
}

/// Like [`suggest`] with two surfaces: `fine` for the time from the start (effort), `coarse` (a larger
/// cell) for the finish probe and every leg that gets alternatives, which dominate the run time.
pub fn suggest_with(
    index: &MapIndex,
    fine: &dyn TravelTime,
    coarse: &dyn TravelTime,
    ctx: &SuggestContext<'_>,
) -> Result<Vec<Candidate>, ScoreError> {
    if ctx.limit == 0 {
        return Ok(Vec::new());
    }
    check_end(fine, &ctx.start, |at| ScoreError::StartUnreachable { at })?;
    check_end(fine, &ctx.finish, |at| ScoreError::FinishUnreachable { at })?;
    // The coarse surface (large cells) can block an end that the fine one reaches, or cut start from
    // finish. Then the legs between candidates run on the fine surface too: slower, but not empty.
    let coarse = match check_end(coarse, &ctx.start, |at| ScoreError::StartUnreachable { at })
        .and_then(|()| {
            check_end(coarse, &ctx.finish, |at| ScoreError::FinishUnreachable {
                at,
            })
        })
        .and_then(|()| {
            coarse
                .fastest(ctx.start.clone(), ctx.finish.clone())
                .map(|_| ())
        }) {
        Ok(()) => coarse,
        Err(
            ScoreError::StartUnreachable { .. }
            | ScoreError::FinishUnreachable { .. }
            | ScoreError::Unreachable { .. },
        ) => fine,
        Err(e) => return Err(e),
    };
    let raw = raw_sites(index);
    // The pool: sites outside forbidden areas with standing room.
    let pool: Vec<(&Site, MapPoint)> = raw
        .iter()
        .filter(|s| !forbidden(ctx.specials, &s.at, ctx.line_margin_mm))
        .filter(|s| {
            dist(&s.at, &ctx.start) >= ctx.min_end_distance_mm
                && dist(&s.at, &ctx.finish) >= ctx.min_end_distance_mm
        })
        .filter(|s| {
            ctx.max_reach_mm
                .is_none_or(|reach| dist(&s.at, &ctx.start) <= reach)
        })
        .filter_map(|s| route_end(fine, s).map(|end| (s, end)))
        .collect();
    // Distance to the nearest of start, finish, existing and kept controls: the minimum distance rule and
    // the spread in one number.
    let mut near: Vec<f64> = pool
        .iter()
        .map(|(s, _)| {
            [&ctx.start, &ctx.finish]
                .into_iter()
                .chain(ctx.existing)
                .map(|p| dist(&s.at, p))
                .fold(f64::INFINITY, f64::min)
        })
        .collect();
    // Site, effort in seconds, route choice of the leg from the start.
    let max_ratio = RouteParams::default().max_length_ratio;
    let mut kept: Vec<(&Site, f64, f64)> = Vec::new();
    let mut used = vec![false; pool.len()];
    // Weights of the pick: the spacing grows with the distance from the start, and the rim of the area
    // the map objects span counts less.
    let far_from_start = pool
        .iter()
        .map(|(s, _)| dist(&s.at, &ctx.start))
        .fold(0.0_f64, f64::max)
        .max(f64::EPSILON);
    let (x0, x1, y0, y1) = raw.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(a, b, c, d), s| (a.min(s.at.x), b.max(s.at.x), c.min(s.at.y), d.max(s.at.y)),
    );
    let rim_margin = (RIM_MARGIN_DISTANCES * ctx.min_distance_mm).max(f64::EPSILON);
    let weight: Vec<f64> = pool
        .iter()
        .map(|(s, _)| {
            let spacing = 1.0 + SPACING_GROWTH * dist(&s.at, &ctx.start) / far_from_start;
            let rim = (s.at.x - x0)
                .min(x1 - s.at.x)
                .min(s.at.y - y0)
                .min(y1 - s.at.y)
                .max(0.0);
            let rim = RIM_FLOOR + (1.0 - RIM_FLOOR) * (rim / rim_margin).min(1.0);
            site_weight(s.clarity, spacing, rim)
        })
        .collect();
    let mut tries = 0;
    while kept.len() < ctx.limit.saturating_mul(SITES_PER_LIMIT)
        && tries < ctx.limit.saturating_mul(ATTEMPTS_PER_LIMIT)
    {
        // Farthest point first, clear features preferred; the pool is in position order, so ties go to
        // the lower x, then y.
        let mut best: Option<(usize, f64)> = None;
        for (i, _) in pool.iter().enumerate() {
            if used[i] || near[i] < ctx.min_distance_mm {
                continue;
            }
            let value = near[i] * weight[i];
            if best.is_none_or(|(_, v)| value > v) {
                best = Some((i, value));
            }
        }
        let Some((i, _)) = best else { break };
        tries += 1;
        let (site, end) = pool[i].clone();
        used[i] = true;
        // Reachable: from the start on the fine surface (effort, from the free place `end`), to the finish
        // and from the start with alternatives on the coarse one, routed from the site itself as the
        // tours do (route choice).
        let reachable = match fine.fastest(ctx.start.clone(), end.clone()) {
            Ok(route) => match coarse.fastest(site.at.clone(), ctx.finish.clone()) {
                Ok(_) => {
                    choice_of(coarse, &ctx.start, &site.at, max_ratio)?.map(|c| (route.time_s, c))
                }
                Err(ScoreError::Unreachable { .. }) => None,
                Err(e) => return Err(e),
            },
            Err(ScoreError::Unreachable { .. }) => None,
            Err(e) => return Err(e),
        };
        // A picked site pushes the next picks the full minimum distance away. A lost one only pushes its
        // close neighbours (`LOST_PUSH_SHARE` of the distance), so a walled-in yard is not tried corner by
        // corner but the free ground around it stays in the pool (ADR 0073).
        let push = if reachable.is_some() {
            f64::INFINITY
        } else {
            LOST_PUSH_SHARE * ctx.min_distance_mm
        };
        for (j, (other, _)) in pool.iter().enumerate() {
            let d = dist(&other.at, &site.at);
            if d <= push {
                near[j] = near[j].min(d);
            }
        }
        if let Some((effort, choice)) = reachable {
            kept.push((site, effort, choice));
        }
    }
    let n = kept.len();
    let mut pairs: BTreeMap<(usize, usize), Option<f64>> = BTreeMap::new();
    let mut out = Vec::with_capacity(n);
    for (i, (site, effort_s, start_choice)) in kept.iter().enumerate() {
        let mut values = vec![*start_choice];
        let mut others: Vec<usize> = (0..n).filter(|&j| j != i).collect();
        others.sort_by(|&a, &b| {
            dist(&site.at, &kept[a].0.at)
                .total_cmp(&dist(&site.at, &kept[b].0.at))
                .then(a.cmp(&b))
        });
        for j in others.into_iter().take(NEAREST_LEGS) {
            let key = (i.min(j), i.max(j));
            let value = match pairs.get(&key) {
                Some(v) => *v,
                None => {
                    let v = choice_of(coarse, &kept[key.0].0.at, &kept[key.1].0.at, max_ratio)?;
                    pairs.insert(key, v);
                    v
                }
            };
            values.extend(value);
        }
        let route_choice = values.iter().sum::<f64>() / values.len() as f64;
        let neighbours = raw
            .iter()
            .filter(|r| r.at != site.at && dist(&r.at, &site.at) <= DENSITY_RADIUS_MM)
            .count() as f64;
        let density = (neighbours / DENSITY_FULL).min(1.0);
        let difficulty = (W_NO_CHOICE * (1.0 - route_choice)
            + W_UNCLEAR * (1.0 - site.clarity)
            + W_DENSITY * density)
            .clamp(0.0, 1.0);
        out.push(Candidate {
            at: site.at.clone(),
            symbol_code: Some(site.symbol.clone()),
            route_choice,
            difficulty,
            effort_s: *effort_s,
        });
    }
    out.sort_by(|a, b| {
        b.route_choice
            .total_cmp(&a.route_choice)
            .then(a.at.x.total_cmp(&b.at.x))
            .then(a.at.y.total_cmp(&b.at.y))
    });
    out.truncate(ctx.limit);
    Ok(out)
}

#[cfg(test)]
mod tests;
