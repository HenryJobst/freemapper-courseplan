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
}

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
/// Raw sites kept per wanted candidate before any route is computed.
const PREFILTER_FACTOR: usize = 2;
/// Legs per candidate besides the start: nearest and seeded-pseudo-random others.
const NEAREST_LEGS: usize = 4;
const RANDOM_LEGS: usize = 2;
const SAMPLE_SEED: u64 = 0x5EED_C0DE_2026_1111;

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

fn forbidden(specials: &[SpecialObject], at: &MapPoint) -> bool {
    specials.iter().any(|s| match s {
        SpecialObject::OutOfBoundsArea { polygon, .. }
        | SpecialObject::DangerousArea { polygon, .. } => {
            inside_area(at, std::slice::from_ref(polygon))
        }
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

/// The legs of candidate `i` that are looked at: the start (`n`), the nearest others, seeded-random others.
fn sample(i: usize, sites: &[Site]) -> Vec<usize> {
    let n = sites.len();
    let mut others: Vec<usize> = (0..n).filter(|&j| j != i).collect();
    others.sort_by(|&a, &b| {
        dist(&sites[i].at, &sites[a].at)
            .total_cmp(&dist(&sites[i].at, &sites[b].at))
            .then(a.cmp(&b))
    });
    let mut chosen: Vec<usize> = others.iter().copied().take(NEAREST_LEGS).collect();
    let rest = &others[chosen.len()..];
    if !rest.is_empty() {
        let mut state = SAMPLE_SEED ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut tries = 0;
        let want = RANDOM_LEGS.min(rest.len());
        let mut picked = 0;
        while picked < want && tries < want * 8 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let j = rest[((state >> 33) as usize) % rest.len()];
            if !chosen.contains(&j) {
                chosen.push(j);
                picked += 1;
            }
            tries += 1;
        }
    }
    chosen.push(n);
    chosen
}

/// The candidates for control sites, best route choice first, ties by position.
pub fn suggest(
    index: &MapIndex,
    travel: &dyn TravelTime,
    ctx: &SuggestContext<'_>,
) -> Result<Vec<Candidate>, ScoreError> {
    if ctx.limit == 0 {
        return Ok(Vec::new());
    }
    let raw = raw_sites(index);
    // Cheap rules: minimum distances, forbidden areas, standing room for point sites.
    let mut usable: Vec<Site> = raw
        .iter()
        .filter(|s| {
            let far = |p: &MapPoint| dist(&s.at, p) >= ctx.min_distance_mm;
            far(&ctx.start)
                && far(&ctx.finish)
                && ctx.existing.iter().all(far)
                && !forbidden(ctx.specials, &s.at)
                && (s.corner || travel.passable(s.at.clone()))
        })
        .cloned()
        .collect();
    // Clearest first, ties by position; thin out sites closer than the minimum distance to a kept one.
    usable.sort_by(|a, b| {
        b.clarity
            .total_cmp(&a.clarity)
            .then(a.at.x.total_cmp(&b.at.x))
            .then(a.at.y.total_cmp(&b.at.y))
    });
    let mut sites: Vec<Site> = Vec::new();
    for s in usable {
        if sites.len() >= ctx.limit.saturating_mul(PREFILTER_FACTOR) {
            break;
        }
        if sites
            .iter()
            .all(|k| dist(&k.at, &s.at) >= ctx.min_distance_mm)
        {
            sites.push(s);
        }
    }
    // Candidates the runner cannot reach from the start drop out; the rest carry their effort.
    let mut efforts = Vec::new();
    let mut reachable = Vec::new();
    for s in sites {
        match travel.legs(ctx.start.clone(), s.at.clone()) {
            Ok(legs) => {
                efforts.push(legs.routes.first().map_or(0.0, |r| r.time_s));
                reachable.push((s, legs));
            }
            Err(ScoreError::Unreachable { .. }) => {}
            Err(e) => return Err(e),
        }
    }
    let sites: Vec<Site> = reachable.iter().map(|(s, _)| s.clone()).collect();
    let n = sites.len();
    let max_ratio = RouteParams::default().max_length_ratio;
    let mut pairs: BTreeMap<(usize, usize), Option<f64>> = BTreeMap::new();
    let mut out = Vec::with_capacity(n);
    for (i, (site, start_legs)) in reachable.iter().enumerate() {
        let mut values = vec![leg_choice(start_legs, max_ratio)];
        for j in sample(i, &sites) {
            if j == n {
                continue;
            }
            let key = (i.min(j), i.max(j));
            let value = match pairs.get(&key) {
                Some(v) => *v,
                None => {
                    let v = match travel.legs(sites[key.0].at.clone(), sites[key.1].at.clone()) {
                        Ok(legs) => Some(leg_choice(&legs, max_ratio)),
                        Err(ScoreError::Unreachable { .. }) => None,
                        Err(e) => return Err(e),
                    };
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
            effort_s: efforts[i],
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
