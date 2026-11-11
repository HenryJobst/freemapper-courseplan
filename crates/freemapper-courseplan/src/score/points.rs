//! Point values from effort and difficulty, and the codes that carry them (ADR 0063).

use freemapper_courseplan_model::{ControlCode, MapPoint, ScoreSettings};

use crate::score::ScoreError;
use crate::score::candidates::Candidate;
use crate::score::code::{code_for, reads_alike_turned};

/// Largest share of the score that difficulty can make up. The rest is travel time from the start.
/// With 0.3 a hard control at the start's doorstep still ranks below an easy one at the far end,
/// and difficulty alone can lift a control past another only when their travel times differ by
/// less than three tenths of the farthest one.
pub const DIFFICULTY_SHARE_MAX: f64 = 0.3;

/// What the code assignment may use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeRules {
    /// Lowest tens digit allowed (3 by default).
    pub tens_min: u8,
    /// Highest tens digit allowed (9 by default).
    pub tens_max: u8,
    /// Highest value a code carries, as in the score settings.
    pub value_max: u8,
}

impl CodeRules {
    /// Tens 3 to 9 and values up to the settings' `value_max`.
    pub fn from_settings(settings: &ScoreSettings) -> Self {
        CodeRules {
            tens_min: 3,
            tens_max: 9,
            value_max: settings.value_max,
        }
    }

    fn tens(&self) -> std::ops::RangeInclusive<u8> {
        self.tens_min..=self.tens_max
    }
}

/// A value from 1 to `value_max` for every candidate. The score is the travel time from the start,
/// relative to the farthest candidate, plus the difficulty, which makes up at most
/// [`DIFFICULTY_SHARE_MAX`] of it. The values are quantiles: every step gets the same number of
/// candidates (give or take one), and equal scores are ordered by index (so all-equal scores
/// spread over the values by index). A set smaller than `value_max` uses the lower steps spread
/// over the range (3 candidates at `value_max` 5 give 1, 2 and 4).
///
/// An infinite or NaN `effort_s` counts as travel 0; candidates without a route from the start
/// must be left out by the caller.
pub fn assign_values(candidates: &[Candidate], settings: &ScoreSettings) -> Vec<u8> {
    let n = candidates.len();
    let value_max = usize::from(settings.value_max.max(1));
    let far = candidates
        .iter()
        .map(|c| c.effort_s)
        .filter(|e| e.is_finite())
        .fold(0.0_f64, f64::max);
    let score = |c: &Candidate| {
        let travel = if far > 0.0 && c.effort_s.is_finite() {
            (c.effort_s / far).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let hard = if c.difficulty.is_finite() {
            c.difficulty.clamp(0.0, 1.0)
        } else {
            0.0
        };
        (1.0 - DIFFICULTY_SHARE_MAX) * travel + DIFFICULTY_SHARE_MAX * hard
    };
    let scores: Vec<f64> = candidates.iter().map(score).collect();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| scores[a].total_cmp(&scores[b]).then(a.cmp(&b)));
    let mut values = vec![1u8; n];
    for (rank, &i) in order.iter().enumerate() {
        values[i] = (1 + rank * value_max / n) as u8;
    }
    values
}

/// A code for every value: the tens digit tells controls of equal value apart. Codes are unique,
/// avoid `taken`, and avoid codes that read alike turned by 180 degrees or by 6 and 9 against
/// `taken` and against each other. Low tens digits come first; controls are served in reading
/// order of `coords` (top to bottom, then left to right) and the result keeps the input order.
///
/// Clash-free sets are smaller than the code space: 6 and 9 count as alike in the same place, so
/// the tens digits 6 and 9 exclude each other for one value (30 clash-free controls at
/// `value_max` 5). If a set needs a clash, it goes to the pair of controls that lie farthest apart
/// (63 controls at value 9 use every code). A clash with a taken code is never accepted.
///
/// Errors: `PositionsMismatch` when `coords` and `values` differ in length, `BadValue` for a value
/// outside 1 to `value_max`, `ValueSpaceExhausted` when one value has fewer free codes than
/// controls, `CodeSpaceExhausted` when all free codes together are fewer than the controls.
pub fn assign_codes(
    values: &[u8],
    taken: &[ControlCode],
    coords: &[MapPoint],
    rules: &CodeRules,
) -> Result<Vec<ControlCode>, ScoreError> {
    if values.len() != coords.len() {
        return Err(ScoreError::PositionsMismatch {
            values: values.len(),
            coords: coords.len(),
        });
    }
    if let Some(&value) = values.iter().find(|&&v| v == 0 || v > rules.value_max) {
        return Err(ScoreError::BadValue {
            value: u32::from(value),
            value_max: rules.value_max,
        });
    }
    let usable = |tens: u8, value: u8| {
        let code = code_for(tens, value);
        !taken
            .iter()
            .any(|t| t.0 == code || reads_alike_turned(&t.0, &code))
    };
    let free_for = |value: u8| rules.tens().filter(|&t| usable(t, value)).count();
    let free_total: usize = (1..=rules.value_max).map(free_for).sum();
    if values.len() > free_total {
        return Err(ScoreError::CodeSpaceExhausted { limit: free_total });
    }
    for value in 1..=rules.value_max {
        let asked = values.iter().filter(|&&v| v == value).count();
        let limit = free_for(value);
        if asked > limit {
            return Err(ScoreError::ValueSpaceExhausted { value, limit });
        }
    }
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&a, &b| {
        coords[a]
            .y
            .total_cmp(&coords[b].y)
            .then(coords[a].x.total_cmp(&coords[b].x))
            .then(a.cmp(&b))
    });
    let mut chosen: Vec<Option<String>> = vec![None; values.len()];
    for &i in &order {
        let value = values[i];
        let mut best: Option<(String, f64)> = None;
        for tens in rules.tens() {
            let code = code_for(tens, value);
            if !usable(tens, value) || chosen.iter().flatten().any(|c| *c == code) {
                continue;
            }
            // Distance to the nearest control whose code clashes with this one.
            let near = chosen
                .iter()
                .enumerate()
                .filter(|(_, c)| c.as_deref().is_some_and(|c| reads_alike_turned(c, &code)))
                .map(|(j, _)| distance(&coords[i], &coords[j]))
                .fold(f64::INFINITY, f64::min);
            if near.is_infinite() {
                best = Some((code, near));
                break;
            }
            if best.as_ref().is_none_or(|(_, d)| near > *d) {
                best = Some((code, near));
            }
        }
        match best {
            Some((code, _)) => chosen[i] = Some(code),
            None => {
                return Err(ScoreError::ValueSpaceExhausted {
                    value,
                    limit: free_for(value),
                });
            }
        }
    }
    Ok(chosen.into_iter().flatten().map(ControlCode).collect())
}

fn distance(a: &MapPoint, b: &MapPoint) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::code::value_of_code;

    fn cand(effort_s: f64, difficulty: f64) -> Candidate {
        Candidate {
            at: MapPoint { x: 0.0, y: 0.0 },
            symbol_code: None,
            route_choice: 0.0,
            difficulty,
            effort_s,
        }
    }

    fn settings(value_max: u8) -> ScoreSettings {
        ScoreSettings {
            value_max,
            ..ScoreSettings::default()
        }
    }

    fn pts(n: usize) -> Vec<MapPoint> {
        (0..n)
            .map(|i| MapPoint {
                x: (i % 8) as f64 * 10.0,
                y: (i / 8) as f64 * 10.0,
            })
            .collect()
    }

    fn codes(v: &[ControlCode]) -> Vec<&str> {
        v.iter().map(|c| c.0.as_str()).collect()
    }

    #[test]
    fn harder_control_at_same_distance_gets_more_points() {
        let set = [cand(300.0, 0.1), cand(300.0, 0.9), cand(600.0, 0.5)];
        let v = assign_values(&set, &settings(5));
        assert!(v[1] > v[0], "{v:?}");
    }

    #[test]
    fn difficulty_share_is_capped_at_the_boundary() {
        // Hard at travel 0 scores 0.30, easy at travel 0.5 scores 0.35, the farthest 0.70.
        let set = [cand(0.0, 1.0), cand(500.0, 0.0), cand(1000.0, 0.0)];
        assert_eq!(assign_values(&set, &settings(5)), [1, 2, 4]);
    }

    #[test]
    fn values_follow_the_candidate_not_the_position() {
        let set: Vec<_> = [300.0, 100.0, 500.0, 200.0, 400.0]
            .iter()
            .map(|&e| cand(e, 0.0))
            .collect();
        assert_eq!(assign_values(&set, &settings(5)), [3, 1, 5, 2, 4]);
    }

    #[test]
    fn values_stay_in_range() {
        for max in [5u8, 9] {
            let set: Vec<_> = (0..40)
                .map(|i| cand(f64::from(i) * 17.0, f64::from(i % 5) / 4.0))
                .collect();
            let v = assign_values(&set, &settings(max));
            assert_eq!(v.len(), 40);
            assert!(v.iter().all(|&x| (1..=max).contains(&x)), "{v:?}");
        }
        assert!(assign_values(&[], &settings(5)).is_empty());
    }

    #[test]
    fn values_use_all_steps_on_a_wide_set() {
        let set: Vec<_> = (0..50).map(|i| cand(f64::from(i) * 10.0, 0.3)).collect();
        let v = assign_values(&set, &settings(5));
        for step in 1..=5u8 {
            assert_eq!(v.iter().filter(|&&x| x == step).count(), 10, "{v:?}");
        }
    }

    #[test]
    fn small_sets_and_equal_scores_have_exact_values() {
        assert_eq!(assign_values(&[cand(10.0, 0.5)], &settings(5)), [1]);
        let three: Vec<_> = (1..=3).map(|i| cand(f64::from(i), 0.0)).collect();
        assert_eq!(assign_values(&three, &settings(5)), [1, 2, 4]);
        let same: Vec<_> = (0..10).map(|_| cand(100.0, 0.5)).collect();
        assert_eq!(
            assign_values(&same, &settings(5)),
            [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]
        );
    }

    #[test]
    fn codes_carry_their_value() {
        let rules = CodeRules::from_settings(&settings(5));
        let c = assign_codes(&[1, 2, 3], &[], &pts(3), &rules).unwrap();
        assert_eq!(codes(&c), ["31", "32", "33"]);
        let c = assign_codes(&[1, 1, 1], &[], &pts(3), &rules).unwrap();
        assert_eq!(codes(&c), ["31", "41", "51"]);
        assert!(c.iter().all(|c| value_of_code(&c.0).is_some()));
    }

    #[test]
    fn taken_codes_are_skipped() {
        let rules = CodeRules::from_settings(&settings(5));
        let taken = [ControlCode("31".into()), ControlCode("41".into())];
        let c = assign_codes(&[1, 1], &taken, &pts(2), &rules).unwrap();
        assert_eq!(codes(&c), ["51", "61"]);
    }

    #[test]
    fn turned_pairs_never_meet_in_one_course() {
        let rules = CodeRules::from_settings(&settings(9));
        // 6 and 9 as values: 69 and 96 would clash, 68 and 89 too.
        let values = [9, 6, 9, 6, 8, 9, 8, 6];
        let c = assign_codes(&values, &[], &pts(8), &rules).unwrap();
        for (i, a) in c.iter().enumerate() {
            for b in &c[i + 1..] {
                assert_ne!(a, b);
                assert!(!reads_alike_turned(&a.0, &b.0), "{a} and {b}");
            }
        }
        // Against taken codes as well.
        let taken = [ControlCode("69".into())];
        let c = assign_codes(&[6], &taken, &pts(1), &rules).unwrap();
        assert!(!reads_alike_turned("69", &c[0].0));
    }

    fn clashes(c: &[ControlCode]) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for i in 0..c.len() {
            for j in i + 1..c.len() {
                if reads_alike_turned(&c[i].0, &c[j].0) {
                    out.push((i, j));
                }
            }
        }
        out
    }

    #[test]
    fn clash_free_sets_up_to_30_at_value_five() {
        let rules = CodeRules::from_settings(&settings(5));
        for n in [20usize, 30] {
            let values: Vec<u8> = (0..n).map(|i| (i % 5 + 1) as u8).collect();
            let c = assign_codes(&values, &[], &pts(n), &rules).unwrap();
            assert!(clashes(&c).is_empty(), "{n}: {:?}", codes(&c));
        }
    }

    #[test]
    fn clash_free_sets_of_28_at_value_nine() {
        let rules = CodeRules::from_settings(&settings(9));
        let values: Vec<u8> = (0..28).map(|i| (i % 9 + 1) as u8).collect();
        let c = assign_codes(&values, &[], &pts(28), &rules).unwrap();
        assert!(clashes(&c).is_empty(), "{:?}", codes(&c));
    }

    #[test]
    fn code_space_exhausted_names_the_free_codes() {
        let rules = CodeRules::from_settings(&settings(5));
        let values: Vec<u8> = (0..36).map(|i| (i % 5 + 1) as u8).collect();
        let err = assign_codes(&values, &[], &pts(36), &rules).unwrap_err();
        assert_eq!(err, ScoreError::CodeSpaceExhausted { limit: 35 });
        // Taken codes shrink the space.
        let taken: Vec<_> = ["31", "32"].map(|c| ControlCode(c.into())).to_vec();
        let err = assign_codes(&values[..34], &taken, &pts(34), &rules).unwrap_err();
        assert_eq!(err, ScoreError::CodeSpaceExhausted { limit: 33 });
    }

    #[test]
    fn value_space_exhausted_names_the_value_and_its_limit() {
        let rules = CodeRules::from_settings(&settings(5));
        let err = assign_codes(&[1; 8], &[], &pts(8), &rules).unwrap_err();
        assert_eq!(err, ScoreError::ValueSpaceExhausted { value: 1, limit: 7 });
        let taken = [ControlCode("31".into())];
        let err = assign_codes(&[1; 7], &taken, &pts(7), &rules).unwrap_err();
        assert_eq!(err, ScoreError::ValueSpaceExhausted { value: 1, limit: 6 });
    }

    #[test]
    fn bad_input_is_named() {
        let rules = CodeRules::from_settings(&settings(5));
        assert_eq!(
            assign_codes(&[6], &[], &pts(1), &rules).unwrap_err(),
            ScoreError::BadValue {
                value: 6,
                value_max: 5
            }
        );
        assert_eq!(
            assign_codes(&[0], &[], &pts(1), &rules).unwrap_err(),
            ScoreError::BadValue {
                value: 0,
                value_max: 5
            }
        );
        assert_eq!(
            assign_codes(&[1, 2], &[], &pts(1), &rules).unwrap_err(),
            ScoreError::PositionsMismatch {
                values: 2,
                coords: 1
            }
        );
    }

    #[test]
    fn codes_follow_the_input_order_not_the_reading_order() {
        let rules = CodeRules::from_settings(&settings(5));
        let values = [3u8, 1, 2, 1];
        let at = |x: f64, y: f64| MapPoint { x, y };
        let coords = [at(50.0, 0.0), at(10.0, 40.0), at(30.0, 0.0), at(0.0, 0.0)];
        let c = assign_codes(&values, &[], &coords, &rules).unwrap();
        for (code, v) in c.iter().zip(values) {
            assert_eq!(value_of_code(&code.0), Some(v), "{:?}", codes(&c));
        }
        // Reading order: index 3, 2, 0, 1; the two value-1 controls get 31 and 41 in that order.
        assert_eq!(codes(&c), ["33", "41", "32", "31"]);
    }

    #[test]
    fn full_code_space_puts_unavoidable_clashes_far_apart() {
        // Tens 6 and 7 only: 66 and 69, 76 and 79 clash, and all four codes are needed.
        let rules = CodeRules {
            tens_min: 6,
            tens_max: 7,
            value_max: 9,
        };
        let at = |x: f64| MapPoint { x, y: 0.0 };
        let coords = [at(0.0), at(100.0), at(1.0), at(101.0)];
        let c = assign_codes(&[6, 6, 9, 9], &[], &coords, &rules).unwrap();
        assert_eq!(clashes(&c).len(), 2, "{:?}", codes(&c));
        for (i, j) in clashes(&c) {
            assert!(distance(&coords[i], &coords[j]) > 50.0, "{:?}", codes(&c));
        }
    }

    #[test]
    fn sixty_three_controls_use_every_code_once() {
        let rules = CodeRules::from_settings(&settings(9));
        let values: Vec<u8> = (0..63).map(|i| (i % 9 + 1) as u8).collect();
        let c = assign_codes(&values, &[], &pts(63), &rules).unwrap();
        let mut seen: Vec<&str> = codes(&c);
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 63);
        for (code, v) in c.iter().zip(&values) {
            assert_eq!(value_of_code(&code.0), Some(*v));
        }
    }
}
