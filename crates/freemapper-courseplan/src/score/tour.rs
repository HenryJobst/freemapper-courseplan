//! Round trips: the tour from start to finish that scores the most points within the time limit
//! (ADR 0064). Leg times come from [`TravelTime`] once per pair, so the solvers search no map.

use freemapper_courseplan_model::{MapPoint, ScoreSettings};

use crate::score::ScoreError;
use crate::score::links::{self, Unit};
use crate::score::travel::TravelTime;

mod exact;
mod heuristic;
mod length;
#[cfg(test)]
mod tests;

pub use length::{LengthIssue, LengthReport, analyse_length};

/// Up to this many controls the solver is exact, above it a seeded heuristic.
pub const EXACT_MAX: usize = 12;
/// Slack in seconds when comparing times.
const EPS: f64 = 1e-9;

/// A control with its point value and position in map millimetres.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredControl {
    pub at: MapPoint,
    pub value: u8,
}

/// A round trip: control indices in running order, total time including start and finish legs.
#[derive(Debug, Clone, PartialEq)]
pub struct Tour {
    pub order: Vec<usize>,
    pub time_s: f64,
    pub points: u32,
}

/// Units in running order, each with its direction (`true` is reversed).
type Seq = Vec<(usize, bool)>;

/// The solvers' input: a symmetric time matrix over the controls, the start and the finish.
pub(crate) struct Problem {
    n: usize,
    d: Vec<f64>,
    values: Vec<u32>,
    units: Vec<Unit>,
    internal: Vec<f64>,
    links: Vec<(usize, usize)>,
    limit: f64,
}

impl Problem {
    pub(crate) fn new(
        controls: &[ScoredControl],
        links: &[(usize, usize)],
        travel: &dyn TravelTime,
        start: &MapPoint,
        finish: &MapPoint,
        limit_s: f64,
    ) -> Result<Problem, ScoreError> {
        let n = controls.len();
        let links = links::normalise(n, links)?;
        let units = links::build_units(n, &links)?;
        let m = n + 2;
        let pos = |i: usize| -> &MapPoint {
            if i < n {
                &controls[i].at
            } else if i == n {
                start
            } else {
                finish
            }
        };
        let mut d = vec![f64::INFINITY; m * m];
        for i in 0..m {
            d[i * m + i] = 0.0;
            for j in (i + 1)..m {
                let is_link = links
                    .iter()
                    .any(|&(a, b)| (a, b) == (i, j) || (a, b) == (j, i));
                let mut t = leg(travel, pos(i), pos(j), i == n && j == n + 1)?;
                if is_link {
                    t = t.min(leg(travel, pos(j), pos(i), false)?);
                }
                d[i * m + j] = t;
                d[j * m + i] = t;
            }
        }
        let mut p = Problem {
            n,
            d,
            values: controls.iter().map(|c| u32::from(c.value)).collect(),
            units,
            internal: Vec::new(),
            links,
            limit: limit_s,
        };
        p.internal = p
            .units
            .iter()
            .map(|u| u.path.windows(2).map(|w| p.dist(w[0], w[1])).sum())
            .collect();
        Ok(p)
    }

    pub(crate) fn dist(&self, a: usize, b: usize) -> f64 {
        self.d[a * (self.n + 2) + b]
    }
    fn start(&self) -> usize {
        self.n
    }
    fn finish(&self) -> usize {
        self.n + 1
    }
    fn reversible(&self, u: usize) -> bool {
        self.units[u].path.len() > 1
    }
    fn entry(&self, u: usize, rev: bool) -> usize {
        let p = &self.units[u].path;
        if rev { p[p.len() - 1] } else { p[0] }
    }
    fn exit(&self, u: usize, rev: bool) -> usize {
        self.entry(u, !rev)
    }
    fn unit_points(&self, u: usize) -> u32 {
        self.units[u].path.iter().map(|&c| self.values[c]).sum()
    }
    fn mandatory_mask(&self) -> u64 {
        self.units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.mandatory)
            .fold(0, |m, (k, _)| m | 1 << k)
    }
    fn direct(&self) -> f64 {
        self.dist(self.start(), self.finish())
    }

    /// Time of a tour including the start and finish legs.
    fn eval(&self, seq: &[(usize, bool)]) -> f64 {
        let mut at = self.start();
        let mut t = 0.0;
        for &(u, rev) in seq {
            t += self.dist(at, self.entry(u, rev)) + self.internal[u];
            at = self.exit(u, rev);
        }
        t + self.dist(at, self.finish())
    }

    fn flatten(&self, seq: &[(usize, bool)]) -> Vec<usize> {
        let mut out = Vec::new();
        for &(u, rev) in seq {
            let p = &self.units[u].path;
            if rev {
                out.extend(p.iter().rev());
            } else {
                out.extend(p.iter());
            }
        }
        out
    }

    fn tour(&self, seq: &[(usize, bool)]) -> Tour {
        Tour {
            order: self.flatten(seq),
            time_s: self.eval(seq),
            points: seq.iter().map(|&(u, _)| self.unit_points(u)).sum(),
        }
    }

    /// No tour holds every link within the limit: name the link that fits worst.
    fn link_error(&self) -> ScoreError {
        let solo = |k: usize| {
            [false, true]
                .into_iter()
                .take(if self.reversible(k) { 2 } else { 1 })
                .map(|r| {
                    self.dist(self.start(), self.entry(k, r))
                        + self.internal[k]
                        + self.dist(self.exit(k, r), self.finish())
                })
                .fold(f64::INFINITY, f64::min)
        };
        let detour = |c: usize| self.dist(self.start(), c) + self.dist(c, self.finish());
        let (a, b) = links::least_fitting_link(&self.units, &self.links, solo, detour);
        ScoreError::LinkDoesNotFit { a, b }
    }
}

/// One leg's fastest time. A pair without a route costs infinity, so no tour uses it; only the
/// direct start-finish leg must exist.
fn leg(
    travel: &dyn TravelTime,
    from: &MapPoint,
    to: &MapPoint,
    required: bool,
) -> Result<f64, ScoreError> {
    match travel.fastest(from.clone(), to.clone()) {
        Ok(r) => Ok(r.time_s),
        Err(ScoreError::Unreachable { .. }) if !required => Ok(f64::INFINITY),
        Err(e) => Err(e),
    }
}

/// The best round trip with the seed of the default settings.
pub fn best_tour(
    controls: &[ScoredControl],
    links: &[(usize, usize)],
    travel: &dyn TravelTime,
    start: MapPoint,
    finish: MapPoint,
    limit_s: f64,
) -> Result<Tour, ScoreError> {
    best_tour_seeded(
        controls,
        links,
        travel,
        start,
        finish,
        limit_s,
        ScoreSettings::default().seed,
    )
}

/// The round trip with the most points within `limit_s`, start and finish legs included; ties go to
/// the shorter time, then to the lower index order. Every link is in the tour or the call fails
/// with `LinkDoesNotFit`. Exact up to [`EXACT_MAX`] controls, else a heuristic driven by `seed`.
pub fn best_tour_seeded(
    controls: &[ScoredControl],
    links: &[(usize, usize)],
    travel: &dyn TravelTime,
    start: MapPoint,
    finish: MapPoint,
    limit_s: f64,
    seed: u32,
) -> Result<Tour, ScoreError> {
    let p = Problem::new(controls, links, travel, &start, &finish, limit_s)?;
    solve(&p, seed)
}

fn solve(p: &Problem, seed: u32) -> Result<Tour, ScoreError> {
    let found = if p.n <= EXACT_MAX {
        exact::solve(p)
    } else {
        heuristic::solve(p, seed)
    };
    match found {
        Some(seq) => Ok(p.tour(&seq)),
        None if p.mandatory_mask() != 0 => Err(p.link_error()),
        None => Ok(Tour {
            order: Vec::new(),
            time_s: p.direct(),
            points: 0,
        }),
    }
}
