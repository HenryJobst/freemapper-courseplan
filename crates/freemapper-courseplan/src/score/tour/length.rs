//! Length analysis of a score course.

use freemapper_courseplan_model::{MapPoint, ScoreSettings};

use super::{ScoredControl, Tour, best_tour_seeded};
use crate::score::ScoreError;
use crate::score::travel::TravelTime;

#[derive(Debug, Clone, PartialEq)]
pub struct LengthReport {
    /// Best round trip of the good runner within the limit.
    pub good: Tour,
    /// Best round trip of the very good runner within the limit.
    pub very_good: Tour,
    /// Points of `good` as a share of all points, in percent; 0 for a set without points.
    pub good_percent: f64,
    /// Time of `very_good` as a share of the limit, in percent; 0 for a limit of zero or less.
    pub very_good_finish_percent: f64,
    /// Sum of the point values of all controls.
    pub all_points: u32,
    /// What lies outside the bands, near ones and misses alike; empty when both runners are in their bands.
    pub issues: Vec<LengthIssue>,
    /// Percentage points outside a band that still count as near the target (ADR 0083).
    pub tolerance_percent: u8,
}

/// How a runner's result stands against its band (ADR 0083).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandState {
    /// Inside the band.
    InBand,
    /// Outside the band by at most the tolerance: a success that the report names as a deviation.
    Near,
    /// Outside the tolerance, or controls out of reach: a miss.
    Missed,
}

impl LengthReport {
    /// The issues beyond the tolerance, which fail a draft and the length check.
    pub fn misses(&self) -> impl Iterator<Item = &LengthIssue> {
        self.issues
            .iter()
            .filter(|i| !i.is_near(self.tolerance_percent))
    }

    /// The issues within the tolerance: a deviation from the band, no failure.
    pub fn near(&self) -> impl Iterator<Item = &LengthIssue> {
        self.issues
            .iter()
            .filter(|i| i.is_near(self.tolerance_percent))
    }

    /// No issue is beyond the tolerance.
    pub fn is_ok(&self) -> bool {
        self.misses().next().is_none()
    }

    /// The state of the good runner, or of the very good runner.
    pub fn state(&self, good: bool) -> BandState {
        let mut state = BandState::InBand;
        for issue in self.issues.iter().filter(|i| i.is_good() == good) {
            if !issue.is_near(self.tolerance_percent) {
                return BandState::Missed;
            }
            state = BandState::Near;
        }
        state
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LengthIssue {
    /// The good runner scores a smaller share of all points than the band allows.
    GoodBelow { percent: f64, band: (u8, u8) },
    /// The good runner scores a larger share than the band allows.
    GoodAbove { percent: f64, band: (u8, u8) },
    /// The very good runner cannot take every control within the limit; `missing` points are left.
    VeryGoodMissesControls { missing: u32 },
    /// Every control would take the very good runner this share of the limit, over 100 percent.
    VeryGoodOverLimit { percent: f64 },
    /// The very good runner takes every control but finishes below the band, as a share of the limit.
    VeryGoodTooEarly { percent: f64, band: (u8, u8) },
}

impl LengthIssue {
    /// Percentage points outside the band, for the issues that are about a band; `None` for controls the very
    /// good runner cannot reach, which no tolerance forgives.
    pub fn band_deviation(&self) -> Option<f64> {
        match self {
            LengthIssue::GoodBelow { percent, band }
            | LengthIssue::VeryGoodTooEarly { percent, band } => Some(f64::from(band.0) - percent),
            LengthIssue::GoodAbove { percent, band } => Some(percent - f64::from(band.1)),
            LengthIssue::VeryGoodMissesControls { .. } | LengthIssue::VeryGoodOverLimit { .. } => {
                None
            }
        }
    }

    /// Whether the issue is within `tolerance` percentage points of its band; exactly the tolerance counts.
    pub fn is_near(&self, tolerance: u8) -> bool {
        self.band_deviation()
            .is_some_and(|d| d <= f64::from(tolerance) + 1e-9)
    }

    /// Whether the issue is about the good runner.
    pub fn is_good(&self) -> bool {
        matches!(
            self,
            LengthIssue::GoodBelow { .. } | LengthIssue::GoodAbove { .. }
        )
    }
}

/// Round trips of both runner profiles on one set of controls, with the bands of `settings`
/// (design 2026-11-11 §4.4, ruling 1). Nothing is changed; a miss is reported with its size.
#[allow(clippy::too_many_arguments)]
pub fn analyse_length(
    controls: &[ScoredControl],
    links: &[(usize, usize)],
    travel_good: &dyn TravelTime,
    travel_very_good: &dyn TravelTime,
    start: MapPoint,
    finish: MapPoint,
    limit_s: f64,
    settings: &ScoreSettings,
) -> Result<LengthReport, ScoreError> {
    let seed = settings.seed;
    let solve = |travel: &dyn TravelTime, limit: f64| {
        best_tour_seeded(
            controls,
            links,
            travel,
            start.clone(),
            finish.clone(),
            limit,
            seed,
        )
    };
    let good = solve(travel_good, limit_s)?;
    let very_good = solve(travel_very_good, limit_s)?;
    let all_points: u32 = controls.iter().map(|c| u32::from(c.value)).sum();
    let good_percent = if all_points == 0 {
        0.0
    } else {
        f64::from(good.points) / f64::from(all_points) * 100.0
    };
    let very_good_finish_percent = if limit_s > 0.0 {
        very_good.time_s / limit_s * 100.0
    } else {
        0.0
    };

    let mut issues = Vec::new();
    let band = settings.good_band_percent;
    if all_points == 0 {
        // An empty set has no length to judge.
    } else if good_percent < f64::from(band.0) {
        issues.push(LengthIssue::GoodBelow {
            percent: good_percent,
            band,
        });
    } else if good_percent > f64::from(band.1) {
        issues.push(LengthIssue::GoodAbove {
            percent: good_percent,
            band,
        });
    }
    if very_good.points < all_points {
        issues.push(LengthIssue::VeryGoodMissesControls {
            missing: all_points - very_good.points,
        });
        // How long the very good runner needs for every control, if there is a way at all.
        let all = solve(travel_very_good, f64::INFINITY)?;
        // Only a real excess: a heuristic that missed a control the limit allows is no excess.
        if all.points == all_points && all.time_s.is_finite() && all.time_s > limit_s {
            issues.push(LengthIssue::VeryGoodOverLimit {
                percent: all.time_s / limit_s * 100.0,
            });
        }
    } else if all_points > 0 {
        let band = settings.very_good_band_percent;
        if very_good_finish_percent < f64::from(band.0) {
            issues.push(LengthIssue::VeryGoodTooEarly {
                percent: very_good_finish_percent,
                band,
            });
        }
    }
    Ok(LengthReport {
        good,
        very_good,
        good_percent,
        very_good_finish_percent,
        all_points,
        issues,
        tolerance_percent: settings.band_tolerance_percent,
    })
}

#[cfg(test)]
mod tolerance_tests {
    use super::*;

    fn tour() -> Tour {
        Tour {
            order: Vec::new(),
            time_s: 0.0,
            points: 0,
        }
    }

    fn report(issues: Vec<LengthIssue>, tolerance: u8) -> LengthReport {
        LengthReport {
            good: tour(),
            very_good: tour(),
            good_percent: 0.0,
            very_good_finish_percent: 0.0,
            all_points: 100,
            issues,
            tolerance_percent: tolerance,
        }
    }

    const GOOD: (u8, u8) = (87, 93);
    const VERY_GOOD: (u8, u8) = (95, 100);

    #[test]
    fn exactly_the_tolerance_is_near_and_more_is_a_miss() {
        let below = |percent| LengthIssue::GoodBelow {
            percent,
            band: GOOD,
        };
        assert!(below(84.0).is_near(3));
        assert!(!below(83.9).is_near(3));
        let above = |percent| LengthIssue::GoodAbove {
            percent,
            band: GOOD,
        };
        assert!(above(96.0).is_near(3));
        assert!(!above(96.1).is_near(3));
        let early = |percent| LengthIssue::VeryGoodTooEarly {
            percent,
            band: VERY_GOOD,
        };
        assert!(early(92.0).is_near(3));
        assert!(!early(91.9).is_near(3));
        // Controls out of reach and a limit overrun are never forgiven.
        assert!(!LengthIssue::VeryGoodMissesControls { missing: 1 }.is_near(10));
        assert!(!LengthIssue::VeryGoodOverLimit { percent: 101.0 }.is_near(10));
        // A tolerance of 0 leaves no room.
        assert!(!below(86.9).is_near(0));
    }

    #[test]
    fn a_report_is_in_band_near_or_missed_per_runner() {
        let r = report(Vec::new(), 3);
        assert_eq!(r.state(true), BandState::InBand);
        assert_eq!(r.state(false), BandState::InBand);
        assert!(r.is_ok());

        let near = report(
            vec![LengthIssue::GoodAbove {
                percent: 96.0,
                band: GOOD,
            }],
            3,
        );
        assert_eq!(near.state(true), BandState::Near);
        assert_eq!(near.state(false), BandState::InBand);
        assert!(near.is_ok());
        assert_eq!(near.near().count(), 1);
        assert_eq!(near.misses().count(), 0);

        let missed = report(
            vec![
                LengthIssue::GoodAbove {
                    percent: 96.0,
                    band: GOOD,
                },
                LengthIssue::VeryGoodTooEarly {
                    percent: 91.9,
                    band: VERY_GOOD,
                },
            ],
            3,
        );
        assert_eq!(missed.state(true), BandState::Near);
        assert_eq!(missed.state(false), BandState::Missed);
        assert!(!missed.is_ok());
        assert_eq!(missed.misses().count(), 1);
    }

    #[test]
    fn a_wider_tolerance_turns_a_miss_into_near() {
        let issue = LengthIssue::GoodBelow {
            percent: 80.0,
            band: GOOD,
        };
        assert!(!report(vec![issue.clone()], 3).is_ok());
        assert!(report(vec![issue], 7).is_ok());
    }
}
