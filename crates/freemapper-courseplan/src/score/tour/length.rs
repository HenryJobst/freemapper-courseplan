//! Length analysis of a score course.

use freemapper_courseplan_model::{MapPoint, ScoreSettings};

use super::{ScoredControl, Tour, best_tour_seeded};
use crate::score::ScoreError;
use crate::score::travel::TravelTime;

#[derive(Debug, Clone, PartialEq)]
pub struct LengthReport {
    pub good: Tour,
    pub very_good: Tour,
    pub good_percent: f64,
    pub very_good_finish_percent: f64,
    pub all_points: u32,
    pub issues: Vec<LengthIssue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LengthIssue {
    GoodBelow { percent: f64, band: (u8, u8) },
    GoodAbove { percent: f64, band: (u8, u8) },
    VeryGoodMissesControls { missing: u32 },
    VeryGoodOverLimit { percent: f64 },
    VeryGoodTooEarly { percent: f64, band: (u8, u8) },
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
    let very_good_finish_percent = very_good.time_s / limit_s * 100.0;

    let mut issues = Vec::new();
    let band = settings.good_band_percent;
    if good_percent < f64::from(band.0) {
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
        if all.points == all_points {
            issues.push(LengthIssue::VeryGoodOverLimit {
                percent: all.time_s / limit_s * 100.0,
            });
        }
    } else {
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
    })
}
