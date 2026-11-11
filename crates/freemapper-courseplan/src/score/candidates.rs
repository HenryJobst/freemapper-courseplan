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
/// A corner may lie in the blocked cell of its wall; it needs a free place at one of these distances (map mm).
const CORNER_PROBE_MM: [f64; 3] = [0.5, 1.0, 1.5];

struct Classes(Vec<(String, bool, f64)>);

impl Classes {
    /// Symbol code, corner (else point), clarity; read once from `candidates.toml`.
    fn embedded() -> Classes {
        let doc: toml::Table = include_str!("candidates.toml")
            .parse()
            .expect("candidates.toml is valid TOML");
        let mut rows = Vec::new();
        for class in doc["class"]
            .as_array()
            .expect("candidates.toml has classes")
        {
            let corner = class["kind"].as_str() == Some("corner");
            let clarity = class["clarity"].as_float().expect("class has a clarity");
            for code in class["codes"].as_array().expect("class has codes") {
                rows.push((
                    code.as_str().expect("code is text").to_owned(),
                    corner,
                    clarity,
                ));
            }
        }
        Classes(rows)
    }

    fn lookup(&self, symbol: &str, corner: bool) -> Option<f64> {
        self.0
            .iter()
            .filter(|(code, c, _)| *c == corner && symbol_matches(symbol, code))
            .max_by_key(|(code, _, _)| code.len())
            .map(|(_, _, clarity)| *clarity)
    }
}

#[derive(Debug, Clone)]
struct Site {
    at: MapPoint,
    symbol: String,
    clarity: f64,
    corner: bool,
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

/// Every site the map offers, with duplicate positions (0.01 mm) reduced to the clearest.
fn raw_sites(index: &MapIndex) -> Vec<Site> {
    let classes = Classes::embedded();
    let mut sites: Vec<Site> = Vec::new();
    for f in index.features() {
        let Some(code) = f.symbol_code.as_deref() else {
            continue;
        };
        let mut push = |at: MapPoint, clarity: f64, corner: bool| {
            sites.push(Site {
                at,
                symbol: code.to_owned(),
                clarity,
                corner,
            });
        };
        match &f.geometry {
            FeatureGeometry::Point(at) => {
                if let Some(c) = classes.lookup(code, false) {
                    push(at.clone(), c, false);
                }
            }
            FeatureGeometry::Line(points) => {
                if let Some(c) = classes.lookup(code, true) {
                    corners(points, false)
                        .into_iter()
                        .for_each(|at| push(at, c, true));
                }
            }
            FeatureGeometry::Area(rings) => {
                if let Some(c) = classes.lookup(code, true) {
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
    if !site.corner {
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
    // Site, route end, effort in seconds, route choice of the leg from the start.
    let max_ratio = RouteParams::default().max_length_ratio;
    let mut kept: Vec<(&Site, MapPoint, f64, f64)> = Vec::new();
    let mut used = vec![false; pool.len()];
    let mut tries = 0;
    while kept.len() < ctx.limit.saturating_mul(SITES_PER_LIMIT)
        && tries < ctx.limit.saturating_mul(ATTEMPTS_PER_LIMIT)
    {
        // Farthest point first, clear features preferred; the pool is in position order, so ties go to
        // the lower x, then y.
        let mut best: Option<(usize, f64)> = None;
        for (i, (site, _)) in pool.iter().enumerate() {
            if used[i] || near[i] < ctx.min_distance_mm {
                continue;
            }
            let value = near[i] * site.clarity;
            if best.is_none_or(|(_, v)| value > v) {
                best = Some((i, value));
            }
        }
        let Some((i, _)) = best else { break };
        tries += 1;
        let (site, end) = pool[i].clone();
        used[i] = true;
        // Reachable: from the start on the fine surface (effort), to the finish and from the start with
        // alternatives on the coarse one (route choice).
        let reachable = match fine.fastest(ctx.start.clone(), end.clone()) {
            Ok(route) => match coarse.fastest(end.clone(), ctx.finish.clone()) {
                Ok(_) => choice_of(coarse, &ctx.start, &end, max_ratio)?.map(|c| (route.time_s, c)),
                Err(ScoreError::Unreachable { .. }) => None,
                Err(e) => return Err(e),
            },
            Err(ScoreError::Unreachable { .. }) => None,
            Err(e) => return Err(e),
        };
        // A failed site still pushes the next picks away, so a walled-in yard is not tried corner by corner.
        for (j, (other, _)) in pool.iter().enumerate() {
            near[j] = near[j].min(dist(&other.at, &site.at));
        }
        if let Some((effort, choice)) = reachable {
            kept.push((site, end, effort, choice));
        }
    }
    let n = kept.len();
    let mut pairs: BTreeMap<(usize, usize), Option<f64>> = BTreeMap::new();
    let mut out = Vec::with_capacity(n);
    for (i, (site, _, effort_s, start_choice)) in kept.iter().enumerate() {
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
                    let v = choice_of(coarse, &kept[key.0].1, &kept[key.1].1, max_ratio)?;
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
