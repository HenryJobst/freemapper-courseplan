//! Local search around the best prefix set (ADR 0074): swap, add or drop one control, or move one
//! value one step off its quantile, and keep what brings the length bands closer. Seeded and bounded by
//! a budget of tour evaluations, so the answer is the same on every run.

use super::{LengthIssue, Search, Trial, first_violation, is_hard, miss};
use crate::score::ScoreError;
use crate::score::points::{CodeRules, assign_codes, assign_values};

/// Most steps a value may lie from the quantile value of its candidate.
const MAX_SHIFT: i8 = 2;
/// Weight of one percentage point of missed hard share against one point of band miss. Small: the bands win
/// over the mix (design §9 ruling 3), the mix only orders equal choices.
const MIX_WEIGHT: f64 = 0.02;

/// A small xorshift; the same sequence for the same seed on every platform.
struct Rng(u32);

impl Rng {
    fn new(seed: u32) -> Rng {
        Rng(if seed == 0 { 0x9E37_79B9 } else { seed })
    }
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        self.next() as usize % n.max(1)
    }
}

/// A set of candidates with the shift of each value from its quantile value.
#[derive(Clone)]
struct State {
    picks: Vec<usize>,
    shift: Vec<i8>,
}

impl Search<'_> {
    fn values_shifted(&self, st: &State) -> Vec<u8> {
        let cands: Vec<_> = st
            .picks
            .iter()
            .map(|&i| self.req.candidates[i].clone())
            .collect();
        let base = assign_values(&cands, self.req.settings);
        let top = i16::from(self.req.settings.value_max);
        base.iter()
            .zip(&st.shift)
            .map(|(&b, &s)| (i16::from(b) + i16::from(s)).clamp(1, top) as u8)
            .collect()
    }

    /// The state as a trial, or `None` when a high-value pair is too close or the codes cannot be assigned.
    fn trial_of(&self, st: &State) -> Option<Result<Trial, ScoreError>> {
        let values = self.values_shifted(st);
        let gap_mm = f64::from(self.req.settings.high_value_gap_m) * 1000.0
            / f64::from(self.req.scale.max(1));
        if first_violation(
            &st.picks,
            &values,
            self.req.candidates,
            self.req.settings.high_from(),
            gap_mm,
            self.forced,
        )
        .is_some()
        {
            return None;
        }
        let coords: Vec<_> = st
            .picks
            .iter()
            .map(|&i| self.req.candidates[i].at.clone())
            .collect();
        assign_codes(
            &values,
            self.req.taken,
            &coords,
            &CodeRules::from_settings(self.req.settings),
        )
        .ok()?;
        Some(self.evaluate(st.picks.clone(), values))
    }

    fn mix_gap(&self, picks: &[usize]) -> f64 {
        if picks.is_empty() {
            return 0.0;
        }
        let share = f64::from(self.req.settings.mix_hard_percent) / 100.0;
        let hard = picks
            .iter()
            .filter(|&&i| is_hard(&self.req.candidates[i]))
            .count() as f64;
        (hard / picks.len() as f64 - share).abs() * 100.0
    }

    fn score(&self, t: &Trial) -> f64 {
        miss(&t.report) + MIX_WEIGHT * self.mix_gap(&t.picks)
    }

    /// Improves `from` with at most `budget` tour evaluations; returns the best trial seen, which is
    /// `from` itself when nothing was better. Stops at once when both bands hold.
    pub(super) fn refine(&self, from: Trial, budget: usize) -> Result<Trial, ScoreError> {
        let mut rng = Rng::new(self.req.settings.seed ^ 0x5bd1_e995);
        let base_values: Vec<u8> = {
            let st = State {
                picks: from.picks.clone(),
                shift: vec![0; from.picks.len()],
            };
            self.values_shifted(&st)
        };
        // A trial from the prefix search holds the quantile values; anything else starts from its own.
        let shift = from
            .values
            .iter()
            .zip(&base_values)
            .map(|(&v, &b)| {
                (i16::from(v) - i16::from(b)).clamp(-i16::from(MAX_SHIFT), i16::from(MAX_SHIFT))
                    as i8
            })
            .collect();
        let mut cur = State {
            picks: from.picks.clone(),
            shift,
        };
        let mut cur_trial = from;
        let mut cur_score = self.score(&cur_trial);
        if cur_trial.in_band() {
            return Ok(cur_trial);
        }
        let mut evals = 0;
        let mut tries = 0;
        while evals < budget && tries < budget * 20 {
            tries += 1;
            let Some(next) = self.neighbour(&cur, &cur_trial, &mut rng) else {
                continue;
            };
            let Some(result) = self.trial_of(&next) else {
                continue;
            };
            evals += 1;
            let trial = result?;
            let s = self.score(&trial);
            if s <= cur_score + 1e-12 {
                cur = next;
                cur_score = s;
                cur_trial = trial;
                if cur_trial.in_band() {
                    break;
                }
            }
        }
        Ok(cur_trial)
    }

    /// One random move, steered by the issue: when the good runner scores too much, value moves raise the
    /// controls his tour skips and lower the ones he takes, and the other way round when he scores too
    /// little.
    fn neighbour(&self, st: &State, t: &Trial, rng: &mut Rng) -> Option<State> {
        let n = st.picks.len();
        let free = n.saturating_sub(self.forced);
        let unused = |rng: &mut Rng| -> Option<usize> {
            let pool = &self.pool;
            (0..8).find_map(|_| {
                let c = pool[rng.below(pool.len())];
                (!st.picks.contains(&c)).then_some(c)
            })
        };
        let mut next = st.clone();
        match rng.below(8) {
            0..=3 => {
                // Value step. When the good runner scores too much, raise a control his tour skips or lower
                // one he takes; when he scores too little, the other way round.
                let above = t
                    .report
                    .issues
                    .iter()
                    .any(|x| matches!(x, LengthIssue::GoodAbove { .. }));
                let below = t
                    .report
                    .issues
                    .iter()
                    .any(|x| matches!(x, LengthIssue::GoodBelow { .. }));
                let up = rng.below(2) == 0;
                let want_visited = if above {
                    !up
                } else if below {
                    up
                } else {
                    rng.below(2) == 0
                };
                let from: Vec<usize> = (0..n)
                    .filter(|i| t.report.good.order.contains(i) == want_visited)
                    .collect();
                if from.is_empty() {
                    return None;
                }
                let i = from[rng.below(from.len())];
                let s = next.shift[i] + if up { 1 } else { -1 };
                if s.abs() > MAX_SHIFT {
                    return None;
                }
                next.shift[i] = s;
            }
            4..=5 => {
                // Swap one control for an unused candidate.
                if free == 0 {
                    return None;
                }
                let i = self.forced + rng.below(free);
                next.picks[i] = unused(rng)?;
                next.shift[i] = 0;
            }
            6 => {
                let c = unused(rng)?;
                next.picks.push(c);
                next.shift.push(0);
            }
            _ => {
                if free == 0 || n <= self.min_k {
                    return None;
                }
                let i = self.forced + rng.below(free);
                next.picks.remove(i);
                next.shift.remove(i);
            }
        }
        Some(next)
    }
}
