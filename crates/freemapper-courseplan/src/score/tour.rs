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
    /// Position in map millimetres.
    pub at: MapPoint,
    /// Point value of the control.
    pub value: u8,
}

/// A round trip: control indices in running order, total time including start and finish legs.
/// With an empty order, `time_s` is the direct start-finish time, which can exceed the limit.
#[derive(Debug, Clone, PartialEq)]
pub struct Tour {
    /// Indices into the control list, in running order.
    pub order: Vec<usize>,
    /// Time of the whole trip in seconds.
    pub time_s: f64,
    /// Sum of the point values of the controls in `order`.
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
    /// Entry and exit control of every unit, forwards then reversed.
    ends: Vec<[(usize, usize); 2]>,
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
            ends: Vec::new(),
            links,
            limit: limit_s,
        };
        p.internal = p
            .units
            .iter()
            .map(|u| u.path.windows(2).map(|w| p.dist(w[0], w[1])).sum())
            .collect();
        p.ends = p
            .units
            .iter()
            .map(|u| {
                let (first, last) = (u.path[0], u.path[u.path.len() - 1]);
                [(first, last), (last, first)]
            })
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
        self.ends[u][usize::from(rev)].0
    }
    fn exit(&self, u: usize, rev: bool) -> usize {
        self.ends[u][usize::from(rev)].1
    }
    fn unit_points(&self, u: usize) -> u32 {
        self.units[u].path.iter().map(|&c| self.values[c]).sum()
    }
    fn has_mandatory(&self) -> bool {
        self.units.iter().any(|u| u.mandatory)
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

    /// No tour holds every link within the limit: name the link that fits worst. `proven` says the
    /// search was exact; otherwise a tour may exist that the heuristic did not find.
    fn link_error(&self, proven: bool) -> ScoreError {
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
        if proven {
            ScoreError::LinkDoesNotFit { a, b }
        } else {
            ScoreError::LinkTourNotFound { a, b }
        }
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
    match travel.fastest_time(from.clone(), to.clone()) {
        Ok(t) => Ok(t),
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
/// the shorter time. Equal ends of the exact table are then ordered by the lower index order, which
/// does not make the result unique among all tours. Every link is in the tour or the call fails
/// with `LinkDoesNotFit` (proven, exact solver) or `LinkTourNotFound` (heuristic, not proven). Exact up to [`EXACT_MAX`] controls, else a heuristic driven by `seed`.
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
    // Err holds whether the miss is proven: the exact solver proves it, the heuristic only
    // sometimes.
    let found = if p.n <= EXACT_MAX {
        exact::solve(p).ok_or(true)
    } else {
        heuristic::solve(p, seed)
    };
    match found {
        Ok(seq) => Ok(p.tour(&seq)),
        Err(true) if p.has_mandatory() => Err(p.link_error(true)),
        Err(false) if p.has_mandatory() => Err(p.link_error(false)),
        Err(_) => Ok(Tour {
            order: Vec::new(),
            time_s: p.direct(),
            points: 0,
        }),
    }
}
