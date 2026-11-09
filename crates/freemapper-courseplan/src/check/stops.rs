//! The stops of a course as the checks see them: start, controls, crossing points and finish,
//! with their position and kind, from the legs of `CourseContext`.

use std::collections::HashSet;

use freemapper_courseplan_model::{ControlCode, ControlKind, MapPoint};

use super::{CheckContext, CourseContext};

/// What a stop of a course is; the selector of the finding texts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StopKind {
    Start,
    Control,
    CrossingPoint,
    Finish,
}

impl StopKind {
    pub(super) fn word(self) -> &'static str {
        match self {
            StopKind::Start => "start",
            StopKind::Control => "control",
            StopKind::CrossingPoint => "crossing",
            StopKind::Finish => "finish",
        }
    }
}

/// A start, control, crossing point or finish of a course.
#[derive(Debug, Clone)]
pub(super) struct Stop {
    /// Control code or special-object id; they share one namespace (`CodeClashesWithSpecial`).
    pub(super) id: String,
    pub(super) kind: StopKind,
    pub(super) at: MapPoint,
}

/// The stops of a course in running order, from its legs: each leg leaves one stop, and the
/// last leg reaches the finish.
pub(super) fn course_stops(ctx: &CheckContext<'_>, course: &CourseContext<'_>) -> Vec<Stop> {
    let mut stops = Vec::with_capacity(course.legs.len() + 1);
    for (i, leg) in course.legs.iter().enumerate() {
        let kind = if i == 0 {
            StopKind::Start
        } else {
            match ctx.control(&ControlCode(leg.from.clone())).map(|c| &c.kind) {
                Some(ControlKind::CrossingPoint) => StopKind::CrossingPoint,
                _ => StopKind::Control,
            }
        };
        stops.push(Stop {
            id: leg.from.clone(),
            kind,
            at: leg.points[0].clone(),
        });
    }
    if let Some(last) = course.legs.last() {
        stops.push(Stop {
            id: last.to.clone(),
            kind: StopKind::Finish,
            at: last.points[last.points.len() - 1].clone(),
        });
    }
    stops
}

/// The stops of these courses whose kind `keep` selects, each once, in order of first use.
pub(super) fn stops_of<'c>(
    ctx: &CheckContext<'_>,
    courses: impl IntoIterator<Item = &'c CourseContext<'c>>,
    keep: impl Fn(StopKind) -> bool,
) -> Vec<Stop> {
    let mut seen = HashSet::new();
    let mut stops = Vec::new();
    for course in courses {
        for stop in course_stops(ctx, course) {
            if keep(stop.kind) && seen.insert(stop.id.clone()) {
                stops.push(stop);
            }
        }
    }
    stops
}
