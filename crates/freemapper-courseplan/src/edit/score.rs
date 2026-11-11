//! Edit operations of score courses (ADR 0067): the course, its controls with values, the mandatory
//! links, the event's score settings and the course details. They follow the rules of `edit`: work on
//! a clone, validate, write back only a consistent result.

use freemapper_courseplan_model::{
    ControlCode, Course, CourseKind, Event, ScoreControl, ScoreLink, ScoreSettings, score_penalty,
};

use super::{EditError, apply, course_mut, not_found};
use crate::score::ScoreError;
use crate::score::code::value_of_code;

/// One control of a score course as the caller gives it; without a value the code's value is taken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreControlInput {
    pub code: ControlCode,
    pub value: Option<u32>,
}

/// New settings for the event's `[score]` table; `None` leaves a field as it is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScoreSettingsChange {
    pub value_max: Option<u8>,
    pub good_pace_s_per_km: Option<u32>,
    pub very_good_pace_s_per_km: Option<u32>,
    pub mix_hard_percent: Option<u8>,
    pub seed: Option<u32>,
}

impl ScoreSettingsChange {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// The time limit and penalty of a score course as they read now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreLimits {
    pub time_limit_min: u32,
    /// The stored penalty, or the default while none is stored.
    pub penalty_per_min: u32,
    /// The penalty is the default, not a stored one.
    pub penalty_is_default: bool,
}

/// What [`set_score_controls`] stored.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreControlsEdit {
    pub controls: Vec<ScoreControl>,
    pub limits: ScoreLimits,
}

/// The limits of a score course under `settings`.
pub fn limits_of(kind: &CourseKind, settings: &ScoreSettings) -> Option<ScoreLimits> {
    match kind {
        CourseKind::Score {
            time_limit_min,
            penalty_per_min,
            controls,
            ..
        } => Some(ScoreLimits {
            time_limit_min: *time_limit_min,
            penalty_per_min: score_penalty(*penalty_per_min, controls, settings),
            penalty_is_default: penalty_per_min.is_none(),
        }),
        _ => None,
    }
}

/// What to do with the penalty of a score course.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PenaltyChange {
    #[default]
    Keep,
    Set(u32),
    /// Back to the default, worked out from the controls.
    Default,
}

/// The value of a score control: the one the code carries, or the given one for a code that carries
/// none. A given value that disagrees with the code is refused, and so is one outside 1 to `value_max`.
fn checked_value(
    code: &ControlCode,
    given: Option<u32>,
    settings: &ScoreSettings,
) -> Result<u32, EditError> {
    let carried = value_of_code(&code.0);
    let value = match (given, carried) {
        (Some(v), Some(c)) if v != u32::from(c) => {
            return Err(EditError::ValueDisagreesWithCode {
                code: code.clone(),
                value: v,
                expected: c,
            });
        }
        (Some(v), _) => v,
        (None, Some(c)) => u32::from(c),
        (None, None) => return Err(EditError::CodeCarriesNoValue(code.clone())),
    };
    if value == 0 || value > u32::from(settings.value_max) {
        return Err(ScoreError::BadValue {
            value,
            value_max: settings.value_max,
        }
        .into());
    }
    Ok(value)
}

type ScoreParts<'a> = (
    &'a mut u32,
    &'a mut Option<u32>,
    &'a mut Vec<ScoreControl>,
    &'a mut Vec<ScoreLink>,
);

fn score_mut(course: &mut Course) -> Result<ScoreParts<'_>, EditError> {
    match &mut course.kind {
        CourseKind::Score {
            time_limit_min,
            penalty_per_min,
            controls,
            links,
        } => Ok((time_limit_min, penalty_per_min, controls, links)),
        _ => Err(EditError::NotScoreCourse {
            course: course.name.clone(),
        }),
    }
}

/// Adds a score course without controls. Without a penalty the course stores none and uses the default.
/// Returns the limits as they read now.
pub fn add_score_course(
    event: &mut Event,
    name: &str,
    start: &str,
    finish: &str,
    time_limit_min: u32,
    penalty_per_min: Option<u32>,
) -> Result<ScoreLimits, EditError> {
    if time_limit_min == 0 {
        return Err(EditError::BadTimeLimit);
    }
    apply(event, |e| {
        e.courses.push(Course {
            ppen_id: None,
            name: name.to_owned(),
            classes: vec![],
            start: start.to_owned(),
            finish: finish.to_owned(),
            start_bends: vec![],
            start_taped: false,
            print_area: None,
            climb_m: None,
            planned_print_scale: None,
            kind: CourseKind::Score {
                time_limit_min,
                penalty_per_min,
                controls: vec![],
                links: vec![],
            },
        });
        let course = e.courses.last().expect("just added");
        Ok(limits_of(&course.kind, &e.score_settings()).expect("a score course"))
    })
}

/// Replaces the controls of a score course. A control without a value takes the one its code carries;
/// a value the code does not carry is refused. A control may appear once, and links keep naming only
/// controls of the course, so dropping a linked control is refused. A stored penalty stays; the default
/// follows the controls on its own.
pub fn set_score_controls(
    event: &mut Event,
    name: &str,
    controls: &[ScoreControlInput],
) -> Result<ScoreControlsEdit, EditError> {
    apply(event, |e| {
        let settings = e.score_settings();
        let mut stored = Vec::with_capacity(controls.len());
        for input in controls {
            if !e.controls.iter().any(|c| c.code == input.code) {
                return Err(not_found("control", input.code.0.clone()));
            }
            if stored.iter().any(|c: &ScoreControl| c.code == input.code) {
                return Err(EditError::DuplicateScoreControl(input.code.clone()));
            }
            stored.push(ScoreControl {
                code: input.code.clone(),
                value: checked_value(&input.code, input.value, &settings)?,
            });
        }
        let course = course_mut(e, name)?;
        let (_, _, old, _) = score_mut(course)?;
        *old = stored.clone();
        let limits = limits_of(&course.kind, &settings).expect("a score course");
        Ok(ScoreControlsEdit {
            controls: stored,
            limits,
        })
    })
}

/// Sets the value of one control of a score course. The code carries the value, so only the value of
/// the code is accepted (or any value in range for a code that carries none): to change the value of a
/// control, change its code (ADR 0067). The call repairs a stored value that has drifted from its code.
pub fn set_control_value(
    event: &mut Event,
    name: &str,
    code: &ControlCode,
    value: u32,
) -> Result<(), EditError> {
    apply(event, |e| {
        let settings = e.score_settings();
        let (_, _, controls, _) = score_mut(course_mut(e, name)?)?;
        let entry = controls
            .iter_mut()
            .find(|c| &c.code == code)
            .ok_or_else(|| not_found("control", code.0.clone()))?;
        entry.value = checked_value(code, Some(value), &settings)?;
        Ok(())
    })
}

/// Adds a mandatory link between two controls of a score course. The model's rules decide what is
/// allowed: both controls in the course, different, not linked yet, no ring, at most two links at one
/// control.
pub fn add_score_link(
    event: &mut Event,
    name: &str,
    a: &ControlCode,
    b: &ControlCode,
) -> Result<(), EditError> {
    apply(event, |e| {
        let (_, _, _, links) = score_mut(course_mut(e, name)?)?;
        links.push(ScoreLink {
            a: a.clone(),
            b: b.clone(),
        });
        Ok(())
    })
}

/// Removes the link between two controls; either direction matches.
pub fn remove_score_link(
    event: &mut Event,
    name: &str,
    a: &ControlCode,
    b: &ControlCode,
) -> Result<(), EditError> {
    apply(event, |e| {
        let (_, _, _, links) = score_mut(course_mut(e, name)?)?;
        let at = links
            .iter()
            .position(|l| (&l.a == a && &l.b == b) || (&l.a == b && &l.b == a))
            .ok_or_else(|| not_found("link", format!("{a}-{b}")))?;
        links.remove(at);
        Ok(())
    })
}

/// Changes the event's score settings and returns the stored ones.
pub fn set_score_settings(
    event: &mut Event,
    change: &ScoreSettingsChange,
) -> Result<ScoreSettings, EditError> {
    apply(event, |e| {
        let mut s = e.score_settings();
        if let Some(v) = change.value_max {
            // A control that already carries a higher value would be out of range.
            let highest = e
                .courses
                .iter()
                .filter_map(|c| match &c.kind {
                    CourseKind::Score { controls, .. } => Some(controls),
                    _ => None,
                })
                .flatten()
                .filter(|c| c.value > u32::from(v))
                .max_by_key(|c| c.value);
            if let Some(c) = highest {
                return Err(EditError::ValueMaxBelowControl {
                    value_max: v,
                    code: c.code.clone(),
                    value: c.value,
                });
            }
            s.value_max = v;
        }
        if let Some(v) = change.good_pace_s_per_km {
            s.good_pace_s_per_km = v;
        }
        if let Some(v) = change.very_good_pace_s_per_km {
            s.very_good_pace_s_per_km = v;
        }
        if let Some(v) = change.mix_hard_percent {
            s.mix_hard_percent = v;
        }
        if let Some(v) = change.seed {
            s.seed = v;
        }
        e.score = Some(s.clone());
        Ok(s)
    })
}

/// Changes the time limit or the penalty of a score course and returns both as they read now.
pub fn set_score_course_details(
    event: &mut Event,
    name: &str,
    time_limit_min: Option<u32>,
    penalty: PenaltyChange,
) -> Result<ScoreLimits, EditError> {
    if time_limit_min == Some(0) {
        return Err(EditError::BadTimeLimit);
    }
    apply(event, |e| {
        let settings = e.score_settings();
        let course = course_mut(e, name)?;
        let (limit, stored, _, _) = score_mut(course)?;
        if let Some(v) = time_limit_min {
            *limit = v;
        }
        match penalty {
            PenaltyChange::Keep => {}
            PenaltyChange::Set(v) => *stored = Some(v),
            PenaltyChange::Default => *stored = None,
        }
        Ok(limits_of(&course.kind, &settings).expect("a score course"))
    })
}
