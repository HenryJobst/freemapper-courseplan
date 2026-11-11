//! Point values from effort and difficulty, and the codes that carry them (ADR 0063).

use freemapper_courseplan_model::{ControlCode, MapPoint, ScoreSettings};

use crate::score::ScoreError;
use crate::score::candidates::Candidate;
use crate::score::code::{code_for, code_space, reads_alike_turned};

/// Largest share of the score that difficulty can make up. The rest is travel time from the start.
/// With 0.3 a hard control at the start's doorstep still ranks below an easy one at the far end,
/// so only distance can lift a control into the top values by itself.
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

    fn limit(&self) -> usize {
        if self.tens_max < self.tens_min {
            return 0;
        }
        if self.tens_min == 3 && self.tens_max == 9 {
            return code_space(self.value_max);
        }
        usize::from(self.tens_max - self.tens_min + 1) * usize::from(self.value_max)
    }
}

/// A value from 1 to `value_max` for every candidate. The score is the travel time from the start,
/// relative to the farthest candidate, plus the difficulty, which makes up at most
/// [`DIFFICULTY_SHARE_MAX`] of it. The values are quantiles: every step gets the same number of
/// candidates (give or take one), and equal scores are ordered by index.
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
/// If the set is so full that two clashing codes cannot be avoided (63 controls at value 9 use
/// every code), the clash goes to the pair of controls that lie farthest apart. A clash with a
/// taken code is never accepted.
pub fn assign_codes(
    values: &[u8],
    taken: &[ControlCode],
    coords: &[MapPoint],
    rules: &CodeRules,
) -> Result<Vec<ControlCode>, ScoreError> {
    let limit = rules.limit();
    if values.len() > limit {
        return Err(ScoreError::CodeSpaceExhausted { limit });
    }
    let mut order: Vec<usize> = (0..values.len()).collect();
    let pos = |i: usize| coords.get(i).map_or((0.0, 0.0), |p| (p.y, p.x));
    order.sort_by(|&a, &b| {
        let (ay, ax) = pos(a);
        let (by, bx) = pos(b);
        ay.total_cmp(&by).then(ax.total_cmp(&bx)).then(a.cmp(&b))
    });
    let mut chosen: Vec<Option<String>> = vec![None; values.len()];
    for &i in &order {
        let value = values[i];
        if value == 0 || value > rules.value_max {
            return Err(ScoreError::CodeSpaceExhausted { limit });
        }
        let mut best: Option<(String, f64)> = None;
        for tens in rules.tens_min..=rules.tens_max {
            let code = code_for(tens, value);
            if taken.iter().any(|t| t.0 == code)
                || chosen.iter().flatten().any(|c| *c == code)
                || taken.iter().any(|t| reads_alike_turned(&t.0, &code))
            {
                continue;
            }
            // Distance to the nearest control whose code clashes with this one.
            let near = chosen
                .iter()
                .enumerate()
                .filter(|(_, c)| c.as_deref().is_some_and(|c| reads_alike_turned(c, &code)))
                .map(|(j, _)| distance(coords.get(i), coords.get(j)))
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
            None => return Err(ScoreError::CodeSpaceExhausted { limit }),
        }
    }
    Ok(chosen.into_iter().flatten().map(ControlCode).collect())
}

fn distance(a: Option<&MapPoint>, b: Option<&MapPoint>) -> f64 {
    match (a, b) {
        (Some(a), Some(b)) => (a.x - b.x).hypot(a.y - b.y),
        _ => 0.0,
    }
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
    fn difficulty_alone_does_not_beat_distance() {
        let set = [cand(100.0, 1.0), cand(600.0, 0.0), cand(300.0, 0.5)];
        let v = assign_values(&set, &settings(5));
        assert!(v[1] > v[0], "{v:?}");
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
        // Equal scores: ties by index, so the result is stable.
        let same: Vec<_> = (0..10).map(|_| cand(100.0, 0.5)).collect();
        assert_eq!(
            assign_values(&same, &settings(5)),
            assign_values(&same, &settings(5))
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

    #[test]
    fn code_space_exhausted_names_the_limit() {
        let rules = CodeRules::from_settings(&settings(5));
        let values: Vec<u8> = (0..36).map(|i| (i % 5 + 1) as u8).collect();
        let err = assign_codes(&values, &[], &pts(36), &rules).unwrap_err();
        assert_eq!(err, ScoreError::CodeSpaceExhausted { limit: 35 });
        // Seven controls of one value fit, an eighth does not.
        let err = assign_codes(&[1; 8], &[], &pts(8), &rules).unwrap_err();
        assert_eq!(err, ScoreError::CodeSpaceExhausted { limit: 35 });
    }

    #[test]
    fn sixty_three_controls_fit_at_value_nine() {
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
