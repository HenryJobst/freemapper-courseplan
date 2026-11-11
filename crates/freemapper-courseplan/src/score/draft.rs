//! The draft: a set of controls, values and codes for a score course whose length meets both runner
//! bands (design 2026-11-11 §4.4, ADR 0065). Nothing is written; the caller applies the result.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use freemapper_courseplan_model::{ControlCode, MapPoint, ScoreSettings};

use crate::score::ScoreError;
use crate::score::candidates::Candidate;
use crate::score::code::code_space;
use crate::score::points::{CodeRules, assign_codes, assign_values};
use crate::score::tour::{LengthIssue, LengthReport, ScoredControl, Tour, analyse_length};
use crate::score::travel::TravelTime;

mod select;
#[cfg(test)]
mod tests;

use select::{is_hard, priority_order};

/// Largest distance in map millimetres between a requested link end and the candidate it stands for.
pub const LINK_MATCH_MM: f64 = 0.5;
/// How many control counts below the largest feasible one the search tries.
const SCAN_BELOW: usize = 10;

/// What the draft is built from.
pub struct DraftRequest<'a> {
    pub candidates: &'a [Candidate],
    pub travel_good: &'a dyn TravelTime,
    pub travel_very_good: &'a dyn TravelTime,
    pub start: MapPoint,
    pub finish: MapPoint,
    pub limit_min: u32,
    /// Pairs of positions the setter wants visited one directly after the other. Each end must lie
    /// on a candidate.
    pub links: &'a [(MapPoint, MapPoint)],
    pub settings: &'a ScoreSettings,
    /// Codes that stand already on the map.
    pub taken: &'a [ControlCode],
}

/// A control of the draft.
#[derive(Debug, Clone, PartialEq)]
pub struct DraftControl {
    /// Index into `DraftRequest::candidates`.
    pub candidate: usize,
    pub at: MapPoint,
    pub value: u8,
    /// Empty only in the best attempt of a failure whose codes could not be assigned.
    pub code: ControlCode,
    pub hard: bool,
}

/// A draft that meets both bands.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    pub controls: Vec<DraftControl>,
    /// The requested links as indices into `controls`.
    pub links: Vec<(usize, usize)>,
    pub report: LengthReport,
    /// Hard controls the mix asked for in a set of this size.
    pub hard_wanted: usize,
    pub hard_actual: usize,
    /// Percentage points between the wanted and the actual share of hard controls, when they
    /// differ; the effort bands won over the mix (design §9 ruling 3).
    pub mix_missed_percent: Option<f64>,
}

/// What would help when no draft meets both bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hint {
    LimitUp,
    LimitDown,
    MoreCandidates,
    FasterProfile,
    /// The inputs cannot work at all (no route from the start, links that are no chain); see `error`.
    Impossible,
}

/// The best attempt and why it does not do. Nothing was written.
#[derive(Debug, Clone, PartialEq)]
pub struct DraftFailure {
    pub best: Draft,
    pub issues: Vec<LengthIssue>,
    pub hint: Hint,
    pub error: Option<ScoreError>,
}

struct Trial {
    k: usize,
    report: LengthReport,
}

struct Search<'a> {
    req: &'a DraftRequest<'a>,
    limit_s: f64,
    links: Vec<(usize, usize)>,
    min_k: usize,
    max_k: usize,
}

struct Outcome {
    order: Vec<usize>,
    trial: Trial,
    ok: bool,
    /// With every candidate in, the very good runner still finishes early or the good runner
    /// still scores too much: the limit is long for this pool.
    pool_too_short: bool,
}

impl Search<'_> {
    fn attempt(&self, order: &[usize], k: usize) -> Result<Trial, ScoreError> {
        let picks = &order[..k];
        let report = {
            let (_, controls) = self.values_of(picks);
            analyse_length(
                &controls,
                &self.links,
                self.req.travel_good,
                self.req.travel_very_good,
                self.req.start.clone(),
                self.req.finish.clone(),
                self.limit_s,
                self.req.settings,
            )?
        };
        Ok(Trial { k, report })
    }

    fn values_of(&self, picks: &[usize]) -> (Vec<u8>, Vec<ScoredControl>) {
        let cands: Vec<Candidate> = picks
            .iter()
            .map(|&i| self.req.candidates[i].clone())
            .collect();
        let values = assign_values(&cands, self.req.settings);
        let controls = cands
            .iter()
            .zip(&values)
            .map(|(c, &value)| ScoredControl {
                at: c.at.clone(),
                value,
            })
            .collect();
        (values, controls)
    }

    /// Bisection for the largest count the very good runner still covers completely, then a scan
    /// downwards for a count with both bands met. Counts are evaluated once.
    fn search(&self, order: Vec<usize>) -> Result<Outcome, ScoreError> {
        let mut seen: BTreeMap<usize, Trial> = BTreeMap::new();
        let completes = |t: &Trial| {
            !t.report
                .issues
                .iter()
                .any(|i| matches!(i, LengthIssue::VeryGoodMissesControls { .. }))
        };
        // Whether the very good runner covers every control, and whether both bands hold.
        let mut eval = |k: usize| -> Result<(bool, bool), ScoreError> {
            if let Entry::Vacant(e) = seen.entry(k) {
                e.insert(self.attempt(&order, k)?);
            }
            let t = &seen[&k];
            Ok((completes(t), t.report.issues.is_empty()))
        };
        let kmax = if eval(self.max_k)?.0 {
            self.max_k
        } else if !eval(self.min_k)?.0 {
            self.min_k
        } else {
            let (mut lo, mut hi) = (self.min_k, self.max_k);
            while hi - lo > 1 {
                let mid = lo + (hi - lo) / 2;
                if eval(mid)?.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            lo
        };
        let from = kmax.saturating_sub(SCAN_BELOW).max(self.min_k);
        for k in (from..=kmax).rev() {
            if eval(k)?.1 {
                let trial = seen.remove(&k).expect("just evaluated");
                return Ok(Outcome {
                    order,
                    trial,
                    ok: true,
                    pool_too_short: false,
                });
            }
        }
        let pool_too_short = seen.get(&self.max_k).is_some_and(|t| {
            completes(t)
                && t.report.issues.iter().any(|i| {
                    matches!(
                        i,
                        LengthIssue::GoodAbove { .. } | LengthIssue::VeryGoodTooEarly { .. }
                    )
                })
        });
        let best = seen
            .into_values()
            .min_by(|a, b| {
                miss(&a.report)
                    .total_cmp(&miss(&b.report))
                    .then(a.k.cmp(&b.k))
            })
            .expect("at least one count was evaluated");
        Ok(Outcome {
            order,
            trial: best,
            ok: false,
            pool_too_short,
        })
    }
}

/// How far a report is from the bands, in percentage points (a missed control counts by its share).
fn miss(r: &LengthReport) -> f64 {
    r.issues
        .iter()
        .map(|i| match i {
            LengthIssue::GoodBelow { percent, band } => f64::from(band.0) - percent,
            LengthIssue::GoodAbove { percent, band } => percent - f64::from(band.1),
            LengthIssue::VeryGoodMissesControls { missing } => {
                100.0 * f64::from(*missing) / f64::from(r.all_points.max(1))
            }
            LengthIssue::VeryGoodOverLimit { percent } => percent - 100.0,
            LengthIssue::VeryGoodTooEarly { percent, band } => f64::from(band.0) - percent,
        })
        .sum()
}

fn empty_report() -> LengthReport {
    let none = Tour {
        order: Vec::new(),
        time_s: 0.0,
        points: 0,
    };
    LengthReport {
        good: none.clone(),
        very_good: none,
        good_percent: 0.0,
        very_good_finish_percent: 0.0,
        all_points: 0,
        issues: Vec::new(),
    }
}

fn empty_draft() -> Draft {
    Draft {
        controls: Vec::new(),
        links: Vec::new(),
        report: empty_report(),
        hard_wanted: 0,
        hard_actual: 0,
        mix_missed_percent: None,
    }
}

fn plain_failure(hint: Hint, error: Option<ScoreError>) -> DraftFailure {
    DraftFailure {
        best: empty_draft(),
        issues: Vec::new(),
        hint,
        error,
    }
}

fn dist(a: &MapPoint, b: &MapPoint) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// The candidate in `pool` nearest to `p` within [`LINK_MATCH_MM`]; the lower index wins a tie.
fn match_candidate(cands: &[Candidate], pool: &[usize], p: &MapPoint) -> Option<usize> {
    pool.iter()
        .copied()
        .map(|i| (i, dist(&cands[i].at, p)))
        .filter(|&(_, d)| d <= LINK_MATCH_MM)
        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
        .map(|(i, _)| i)
}

fn slot(forced: &mut Vec<usize>, c: usize) -> usize {
    forced.iter().position(|&f| f == c).unwrap_or_else(|| {
        forced.push(c);
        forced.len() - 1
    })
}

fn hint_for(r: &LengthReport, pool_too_short: bool) -> Hint {
    let has = |f: fn(&LengthIssue) -> bool| r.issues.iter().any(f);
    if has(|i| {
        matches!(
            i,
            LengthIssue::VeryGoodMissesControls { .. } | LengthIssue::VeryGoodOverLimit { .. }
        )
    }) {
        return Hint::LimitUp;
    }
    if has(|i| matches!(i, LengthIssue::GoodBelow { .. })) {
        return Hint::FasterProfile;
    }
    if pool_too_short {
        return Hint::MoreCandidates;
    }
    Hint::LimitDown
}

/// Builds the draft; see ADR 0065. On a miss the failure carries the best attempt, the issues of
/// its length analysis and a hint. The result is the same on every run.
#[allow(clippy::result_large_err)]
pub fn draft(req: &DraftRequest<'_>) -> Result<Draft, DraftFailure> {
    let s = req.settings;
    let limit_s = f64::from(req.limit_min) * 60.0;
    for travel in [req.travel_good, req.travel_very_good] {
        match travel.fastest(req.start.clone(), req.finish.clone()) {
            Ok(r) if r.time_s > limit_s => return Err(plain_failure(Hint::LimitUp, None)),
            Ok(_) => {}
            Err(e) => return Err(plain_failure(Hint::Impossible, Some(e))),
        }
    }
    let pool: Vec<usize> = (0..req.candidates.len())
        .filter(|&i| req.candidates[i].effort_s.is_finite())
        .collect();
    if pool.is_empty() {
        return Err(plain_failure(Hint::MoreCandidates, None));
    }
    let mut forced: Vec<usize> = Vec::new();
    let mut links = Vec::new();
    for (a, b) in req.links {
        let (Some(ca), Some(cb)) = (
            match_candidate(req.candidates, &pool, a),
            match_candidate(req.candidates, &pool, b),
        ) else {
            return Err(plain_failure(Hint::MoreCandidates, None));
        };
        let (ia, ib) = (slot(&mut forced, ca), slot(&mut forced, cb));
        if ia == ib {
            return Err(plain_failure(
                Hint::Impossible,
                Some(ScoreError::BadLink { a: ia, b: ib }),
            ));
        }
        links.push((ia, ib));
    }
    let max_k = pool.len().min(code_space(s.value_max));
    let min_k = forced.len().max(1);
    if min_k > max_k {
        return Err(plain_failure(Hint::MoreCandidates, None));
    }
    let search = Search {
        req,
        limit_s,
        links,
        min_k,
        max_k,
    };
    let share = f64::from(s.mix_hard_percent) / 100.0;
    let mut outcomes = Vec::new();
    for hard_share in [Some(share), None] {
        let order = priority_order(
            req.candidates,
            &pool,
            &forced,
            &req.start,
            &req.finish,
            hard_share,
        );
        let out = match search.search(order) {
            Ok(o) => o,
            Err(e) => {
                let hint = match e {
                    ScoreError::LinkDoesNotFit { .. } | ScoreError::LinkTourNotFound { .. } => {
                        Hint::LimitUp
                    }
                    _ => Hint::Impossible,
                };
                return Err(plain_failure(hint, Some(e)));
            }
        };
        let done = out.ok;
        outcomes.push(out);
        if done {
            break;
        }
    }
    let chosen = if let Some(i) = outcomes.iter().position(|o| o.ok) {
        outcomes.swap_remove(i)
    } else {
        outcomes
            .into_iter()
            .min_by(|a, b| {
                miss(&a.trial.report)
                    .total_cmp(&miss(&b.trial.report))
                    .then(a.trial.k.cmp(&b.trial.k))
            })
            .expect("one search ran")
    };
    let k = chosen.trial.k;
    let picks = &chosen.order[..k];
    let (values, _) = search.values_of(picks);
    let coords: Vec<MapPoint> = picks
        .iter()
        .map(|&i| req.candidates[i].at.clone())
        .collect();
    let codes = assign_codes(&values, req.taken, &coords, &CodeRules::from_settings(s));
    let hard_actual = picks
        .iter()
        .filter(|&&i| is_hard(&req.candidates[i]))
        .count();
    let hard_wanted = ((share * k as f64).round() as usize).min(k);
    let mix_missed_percent = (hard_actual != hard_wanted)
        .then(|| (hard_actual as f64 - hard_wanted as f64).abs() / k as f64 * 100.0);
    let draft = Draft {
        controls: picks
            .iter()
            .zip(&values)
            .enumerate()
            .map(|(n, (&i, &value))| DraftControl {
                candidate: i,
                at: req.candidates[i].at.clone(),
                value,
                code: codes
                    .as_ref()
                    .map(|c| c[n].clone())
                    .unwrap_or_else(|_| ControlCode(String::new())),
                hard: is_hard(&req.candidates[i]),
            })
            .collect(),
        links: search.links.clone(),
        report: chosen.trial.report,
        hard_wanted,
        hard_actual,
        mix_missed_percent,
    };
    if chosen.ok {
        return match codes {
            Ok(_) => Ok(draft),
            Err(e) => Err(DraftFailure {
                issues: Vec::new(),
                best: draft,
                hint: Hint::Impossible,
                error: Some(e),
            }),
        };
    }
    let hint = hint_for(&draft.report, chosen.pool_too_short);
    Err(DraftFailure {
        issues: draft.report.issues.clone(),
        best: draft,
        hint,
        error: None,
    })
}
