//! The draft: a set of controls, values and codes for a score course whose length meets both runner
//! bands (design 2026-11-11 §4.4, ADR 0065). Nothing is written; the caller applies the result.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use freemapper_courseplan_model::{ControlCode, MapPoint, ScoreSettings};

use crate::score::ScoreError;
use crate::score::candidates::Candidate;
use crate::score::points::{CodeRules, assign_codes, assign_values, free_code_count};
use crate::score::tour::{LengthIssue, LengthReport, ScoredControl, Tour, analyse_length};
use crate::score::travel::TravelTime;

mod refine;
mod select;
#[cfg(test)]
mod tests;

use select::{is_hard, priority_order};

/// Largest distance in map millimetres between a requested link end and the candidate it stands for.
pub const LINK_MATCH_MM: f64 = 0.5;
/// How many control counts below the largest feasible one the search tries.
const SCAN_BELOW: usize = 10;
/// Tour evaluations the local search may spend on a draft that misses a band (ADR 0074).
const REFINE_BUDGET: usize = 150;

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
    /// Scale denominator of the map: the high-value gap is in metres on the ground, positions are in
    /// map millimetres.
    pub scale: u32,
}

/// A control of the draft.
#[derive(Debug, Clone, PartialEq)]
pub struct DraftControl {
    /// Index into `DraftRequest::candidates`.
    pub candidate: usize,
    pub at: MapPoint,
    pub value: u8,
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
    /// Every free code is in use before the bands are met: raise `value_max` or free codes on the map.
    CodeSpace,
    FasterProfile,
    /// The inputs cannot work at all (no route from the start, links that are no chain); see `error`.
    Impossible,
}

/// The best attempt and why it does not do. Nothing was written.
///
/// `hint` is `Impossible` exactly when `error` is `Some`, except that a tour error for a link that
/// does not fit also carries `LimitUp`. The other hints come from the issues and leave `error` at
/// `None`. `best` is the evaluated count with the smallest sum of misses and with assignable codes,
/// not the last bisection probe; it is empty when no count has codes.
#[derive(Debug, Clone, PartialEq)]
pub struct DraftFailure {
    pub best: Box<Draft>,
    pub issues: Vec<LengthIssue>,
    pub hint: Hint,
    pub error: Option<ScoreError>,
}

#[derive(Clone)]
struct Trial {
    k: usize,
    /// The candidates of this count in running order of the pool, after high-value repairs.
    picks: Vec<usize>,
    report: LengthReport,
    values: Vec<u8>,
    /// The codes of this count, or why none can be assigned; such a count is no draft.
    codes: Result<Vec<ControlCode>, ScoreError>,
}

impl Trial {
    fn ok(&self) -> bool {
        self.report.issues.is_empty() && self.codes.is_ok()
    }
}

struct Search<'a> {
    req: &'a DraftRequest<'a>,
    limit_s: f64,
    links: Vec<(usize, usize)>,
    /// How many of the first picks are forced (link ends) and must stay.
    forced: usize,
    min_k: usize,
    max_k: usize,
    /// Every candidate the draft may use, in index order.
    pool: Vec<usize>,
}

struct Outcome {
    trial: Trial,
    ok: bool,
    /// With every candidate in, the very good runner still finishes early or the good runner
    /// still scores too much: the limit is long for this pool.
    pool_too_short: bool,
}

impl Search<'_> {
    fn attempt(&self, order: &[usize], k: usize) -> Result<Trial, ScoreError> {
        let picks = self.repaired(order, k);
        let (values, _) = self.values_of(&picks);
        self.evaluate(picks, values)
    }

    /// The length analysis and the codes of a set with its values.
    fn evaluate(&self, picks: Vec<usize>, values: Vec<u8>) -> Result<Trial, ScoreError> {
        let controls: Vec<ScoredControl> = picks
            .iter()
            .zip(&values)
            .map(|(&i, &value)| ScoredControl {
                at: self.req.candidates[i].at.clone(),
                value,
            })
            .collect();
        let report = analyse_length(
            &controls,
            &self.links,
            self.req.travel_good,
            self.req.travel_very_good,
            self.req.start.clone(),
            self.req.finish.clone(),
            self.limit_s,
            self.req.settings,
        )?;
        let coords: Vec<MapPoint> = controls.iter().map(|c| c.at.clone()).collect();
        let codes = assign_codes(
            &values,
            self.req.taken,
            &coords,
            &CodeRules::from_settings(self.req.settings),
        );
        Ok(Trial {
            k: picks.len(),
            picks,
            report,
            values,
            codes,
        })
    }

    /// The first `k` of `order`, where of two high-valued controls closer than the high-value gap the
    /// later one gives way to the next unused candidate of the order (forced ones stay). When the order is
    /// used up the control is dropped, so the set may end up smaller than `k`.
    fn repaired(&self, order: &[usize], k: usize) -> Vec<usize> {
        let gap_mm = f64::from(self.req.settings.high_value_gap_m) * 1000.0
            / f64::from(self.req.scale.max(1));
        let high_from = self.req.settings.high_from();
        let mut picks: Vec<usize> = order[..k].to_vec();
        let mut next = k;
        while let Some(j) = {
            let (values, _) = self.values_of(&picks);
            first_violation(
                &picks,
                &values,
                self.req.candidates,
                high_from,
                gap_mm,
                self.forced,
            )
        } {
            if next < order.len() {
                picks[j] = order[next];
                next += 1;
            } else {
                picks.remove(j);
            }
        }
        picks
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
    /// downwards for a count with both bands met and codes that can be assigned; a count whose
    /// codes fail is skipped. Counts are evaluated once.
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
            Ok((completes(t), t.ok()))
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
        // The evaluated count with the smallest sum of misses (not the last bisection probe), among
        // those whose codes can be assigned.
        let code_error = seen.values().find_map(|t| t.codes.as_ref().err().cloned());
        let best = seen
            .into_values()
            .filter(|t| t.codes.is_ok())
            .min_by(|a, b| {
                miss(&a.report)
                    .total_cmp(&miss(&b.report))
                    .then(a.k.cmp(&b.k))
            })
            .ok_or_else(|| code_error.expect("a count without codes has an error"))?;
        Ok(Outcome {
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
        best: Box::new(empty_draft()),
        issues: Vec::new(),
        hint,
        error,
    }
}

/// The index of the later control of the first pair of high-valued controls closer than `gap_mm`; a
/// forced control (among the first `forced`) never gives way, so a pair of two forced ones is skipped.
fn first_violation(
    picks: &[usize],
    values: &[u8],
    cands: &[Candidate],
    high_from: u8,
    gap_mm: f64,
    forced: usize,
) -> Option<usize> {
    for j in 1..picks.len() {
        if j < forced || values[j] < high_from {
            continue;
        }
        for i in 0..j {
            if values[i] >= high_from && dist(&cands[picks[i]].at, &cands[picks[j]].at) < gap_mm {
                return Some(j);
            }
        }
    }
    None
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

fn hint_for(r: &LengthReport, pool_too_short: bool, code_bound: bool) -> Hint {
    let has = |f: fn(&LengthIssue) -> bool| r.issues.iter().any(f);
    if has(|i| {
        matches!(
            i,
            LengthIssue::VeryGoodMissesControls { .. } | LengthIssue::VeryGoodOverLimit { .. }
        )
    }) {
        return Hint::LimitUp;
    }
    if pool_too_short {
        return if code_bound {
            Hint::CodeSpace
        } else {
            Hint::MoreCandidates
        };
    }
    if has(|i| matches!(i, LengthIssue::GoodBelow { .. })) {
        return Hint::FasterProfile;
    }
    Hint::LimitDown
}

/// Builds the draft; see ADR 0065. On a miss the failure carries the best attempt, the issues of
/// its length analysis and a hint. The result is the same on every run.
pub fn draft(req: &DraftRequest<'_>) -> Result<Draft, DraftFailure> {
    draft_with_budget(req, REFINE_BUDGET)
}

/// [`draft`] with its own budget of local-search evaluations; 0 keeps the prefix search alone.
fn draft_with_budget(req: &DraftRequest<'_>, budget: usize) -> Result<Draft, DraftFailure> {
    let s = req.settings;
    let limit_s = f64::from(req.limit_min) * 60.0;
    for travel in [req.travel_good, req.travel_very_good] {
        match travel.fastest(req.start.clone(), req.finish.clone()) {
            Ok(r) if r.time_s > limit_s => return Err(plain_failure(Hint::LimitUp, None)),
            Ok(_) => {}
            Err(e) => return Err(plain_failure(Hint::Impossible, Some(e))),
        }
    }
    // One search per point reads the times to all later ones; the tours then find every leg ready.
    let mut points: Vec<MapPoint> = req.candidates.iter().map(|c| c.at.clone()).collect();
    points.push(req.start.clone());
    points.push(req.finish.clone());
    for (i, p) in points.iter().enumerate() {
        for travel in [req.travel_good, req.travel_very_good] {
            travel.prefetch(p, &points[i + 1..]);
        }
    }
    // A candidate whose every leg from its own position is infinite would block every larger count.
    let pool: Vec<usize> = (0..req.candidates.len())
        .filter(|&i| req.candidates[i].effort_s.is_finite())
        .filter(|&i| {
            req.travel_very_good
                .fastest_time(req.candidates[i].at.clone(), req.finish.clone())
                .is_ok()
        })
        .collect();
    if pool.is_empty() {
        return Err(plain_failure(Hint::MoreCandidates, None));
    }
    let mut forced: Vec<usize> = Vec::new();
    let mut links = Vec::new();
    for (n, (a, b)) in req.links.iter().enumerate() {
        let (Some(ca), Some(cb)) = (
            match_candidate(req.candidates, &pool, a),
            match_candidate(req.candidates, &pool, b),
        ) else {
            return Err(plain_failure(
                Hint::Impossible,
                Some(ScoreError::LinkEndNotACandidate { link: n }),
            ));
        };
        if ca == cb {
            // The indices are those of the candidates, as the controls are not chosen yet.
            return Err(plain_failure(
                Hint::Impossible,
                Some(ScoreError::BadLink { a: ca, b: cb }),
            ));
        }
        links.push((slot(&mut forced, ca), slot(&mut forced, cb)));
    }
    // Codes the map has already take room from the space (as `assign_codes` counts them); per-value room
    // is checked per count.
    let free_codes = free_code_count(req.taken, &CodeRules::from_settings(s));
    let code_bound = free_codes < pool.len();
    let max_k = pool.len().min(free_codes);
    let min_k = forced.len().max(1);
    if min_k > max_k {
        let hint = if code_bound {
            Hint::CodeSpace
        } else {
            Hint::MoreCandidates
        };
        return Err(plain_failure(hint, None));
    }
    let search = Search {
        req,
        limit_s,
        links,
        forced: forced.len(),
        min_k,
        max_k,
        pool: pool.clone(),
    };
    let share = f64::from(s.mix_hard_percent) / 100.0;
    let mut outcomes = Vec::new();
    let mut code_error = None;
    for hard_share in [Some(share), None] {
        let order = priority_order(
            req.candidates,
            &pool,
            &forced,
            &req.start,
            &req.finish,
            hard_share,
        );
        match search.search(order) {
            Ok(o) => {
                let done = o.ok;
                outcomes.push(o);
                if done {
                    break;
                }
            }
            Err(
                e
                @ (ScoreError::CodeSpaceExhausted { .. } | ScoreError::ValueSpaceExhausted { .. }),
            ) => code_error = Some(e),
            Err(e) => {
                let hint = match e {
                    ScoreError::LinkDoesNotFit { .. } | ScoreError::LinkTourNotFound { .. } => {
                        Hint::LimitUp
                    }
                    _ => Hint::Impossible,
                };
                return Err(plain_failure(hint, Some(e)));
            }
        }
    }
    let mut chosen = if let Some(i) = outcomes.iter().position(|o| o.ok) {
        outcomes.swap_remove(i)
    } else if let Some(best) = outcomes.into_iter().min_by(|a, b| {
        miss(&a.trial.report)
            .total_cmp(&miss(&b.trial.report))
            .then(a.trial.k.cmp(&b.trial.k))
    }) {
        best
    } else {
        return Err(plain_failure(Hint::Impossible, code_error));
    };
    if !chosen.ok
        && let Ok(better) = search.refine(chosen.trial.clone(), budget)
    {
        chosen.ok = better.ok();
        chosen.trial = better;
    }
    let picks = &chosen.trial.picks;
    let k = picks.len();
    let codes = chosen.trial.codes.expect("a chosen count has codes");
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
            .zip(chosen.trial.values.iter().zip(codes))
            .map(|(&i, (&value, code))| DraftControl {
                candidate: i,
                at: req.candidates[i].at.clone(),
                value,
                code,
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
        return Ok(draft);
    }
    let hint = hint_for(&draft.report, chosen.pool_too_short, code_bound);
    Err(DraftFailure {
        issues: draft.report.issues.clone(),
        best: Box::new(draft),
        hint,
        error: None,
    })
}
