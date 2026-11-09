//! Consistency checks on an [`Event`]. These reject files that cannot be a consistent event; they do not
//! judge course quality (that is `check`).

use std::collections::HashSet;

use crate::{ControlCode, CourseKind, Event, FORMAT_VERSION, ModelError, RelayPart, SpecialObject};

/// Finds every inconsistency of the event, in file order. An empty list means the event is consistent.
pub fn validate(event: &Event) -> Vec<ModelError> {
    let mut errors = Vec::new();

    if event.format > FORMAT_VERSION {
        errors.push(ModelError::UnsupportedFormat(event.format));
    }

    if let Some(date) = &event.event.date
        && (date.date.is_none() || date.time.is_some() || date.offset.is_some())
    {
        errors.push(ModelError::DateNotADate(date.to_string()));
    }

    for finding in &event.accepted_findings {
        if finding.note.trim().is_empty() {
            errors.push(ModelError::BlankNote {
                rule: finding.rule.clone(),
            });
        }
    }

    let mut codes: HashSet<&ControlCode> = HashSet::new();
    let mut reported: HashSet<&ControlCode> = HashSet::new();
    for control in &event.controls {
        check_name(&mut errors, "control code", &control.code.0);
        if !codes.insert(&control.code) && reported.insert(&control.code) {
            errors.push(ModelError::DuplicateControl(control.code.clone()));
        }
    }

    // Classes are matched by name when saving, and OE needs unique names.
    let mut class_names: HashSet<&str> = HashSet::new();
    let mut reported_classes: HashSet<&str> = HashSet::new();
    for class in &event.classes {
        let name = class.name.as_str();
        check_name(&mut errors, "class name", name);
        if !class_names.insert(name) && reported_classes.insert(name) {
            errors.push(ModelError::DuplicateClass(class.name.clone()));
        }
    }

    let mut starts: HashSet<&str> = HashSet::new();
    let mut finishes: HashSet<&str> = HashSet::new();
    let mut special_ids: HashSet<&str> = HashSet::new();
    let mut reported_ids: HashSet<&str> = HashSet::new();
    for special in &event.specials {
        let id = special.id();
        check_name(&mut errors, "special object id", id);
        if let Some((kind, needed, have)) = shape(special)
            && have < needed
        {
            errors.push(ModelError::TooFewPoints {
                id: id.to_owned(),
                kind,
                needed,
            });
        }
        let mut listed: HashSet<&str> = HashSet::new();
        let mut doubled: HashSet<&str> = HashSet::new();
        for course in special.courses() {
            if !listed.insert(course) && doubled.insert(course) {
                errors.push(ModelError::DuplicateCourseInSpecial {
                    id: id.to_owned(),
                    course: course.clone(),
                });
            }
        }
        match special {
            SpecialObject::Start { .. } => {
                starts.insert(id);
            }
            SpecialObject::Finish { .. } => {
                finishes.insert(id);
            }
            _ => {}
        }
        if !special_ids.insert(id) && reported_ids.insert(id) {
            errors.push(ModelError::DuplicateSpecialId(id.to_owned()));
        }
    }

    let mut clashes: HashSet<&ControlCode> = HashSet::new();
    for control in &event.controls {
        if special_ids.contains(control.code.0.as_str()) && clashes.insert(&control.code) {
            errors.push(ModelError::CodeClashesWithSpecial(control.code.clone()));
        }
    }

    let known_courses: HashSet<&str> = event.courses.iter().map(|c| c.name.as_str()).collect();
    for special in &event.specials {
        for course in special.courses() {
            if !known_courses.contains(course.as_str()) {
                errors.push(ModelError::UnknownCourseInSpecial {
                    id: special.id().to_owned(),
                    course: course.clone(),
                });
            }
        }
    }

    let mut course_names: HashSet<&str> = HashSet::new();
    for course in &event.courses {
        let name = &course.name;
        check_name(&mut errors, "course name", name);
        if !course_names.insert(name) {
            errors.push(ModelError::DuplicateCourse(name.clone()));
        }
        if !starts.contains(course.start.as_str()) {
            errors.push(ModelError::UnknownStart {
                course: name.clone(),
                id: course.start.clone(),
            });
        }
        if !finishes.contains(course.finish.as_str()) {
            errors.push(ModelError::UnknownFinish {
                course: name.clone(),
                id: course.finish.clone(),
            });
        }

        let mut used: Vec<&ControlCode> = Vec::new();
        match &course.kind {
            CourseKind::Individual { controls } => used.extend(controls.iter().map(|c| &c.code)),
            CourseKind::Team {
                team_size,
                mandatory,
                free,
            } => {
                if *team_size < 2 {
                    errors.push(ModelError::TeamTooSmall {
                        course: name.clone(),
                        team_size: *team_size,
                    });
                }
                used.extend(mandatory.iter().map(|c| &c.code));
                used.extend(free.iter());
                for code in free {
                    if mandatory.iter().any(|m| &m.code == code) {
                        errors.push(ModelError::FreeControlIsMandatory {
                            course: name.clone(),
                            code: code.clone(),
                        });
                    }
                }
            }
            CourseKind::Relay { parts, .. } => {
                for part in parts {
                    match part {
                        RelayPart::Common { controls } => {
                            used.extend(controls.iter().map(|c| &c.code))
                        }
                        RelayPart::Fork { variants } => {
                            for variant in variants {
                                used.extend(variant.iter().map(|c| &c.code));
                            }
                        }
                    }
                }
            }
            CourseKind::Score { controls, .. } => used.extend(controls.iter().map(|c| &c.code)),
        }

        let mut unknown_reported: HashSet<&ControlCode> = HashSet::new();
        for code in used {
            if !codes.contains(code) && unknown_reported.insert(code) {
                errors.push(ModelError::UnknownControl {
                    course: name.clone(),
                    code: code.clone(),
                });
            }
        }
    }

    errors
}

/// Reports a name that is empty or has white space at its start or end. Such names cannot be told
/// apart on the command line or in a list.
fn check_name(errors: &mut Vec<ModelError>, what: &'static str, value: &str) {
    if value.is_empty() || value.trim() != value {
        errors.push(ModelError::BlankName {
            what,
            value: value.to_owned(),
        });
    }
}

/// For a line or area kind: its English name, the fewest points it needs and the points it has. A
/// forbidden route of one point is a single cross, as Purple Pen places it.
fn shape(special: &SpecialObject) -> Option<(&'static str, usize, usize)> {
    match special {
        SpecialObject::MarkedRoute { points, .. } => Some(("marked route", 2, points.len())),
        SpecialObject::ForbiddenRoute { points, .. } => Some(("forbidden route", 1, points.len())),
        SpecialObject::OutOfBoundsArea { polygon, .. } => {
            Some(("out-of-bounds area", 3, polygon.len()))
        }
        SpecialObject::DangerousArea { polygon, .. } => Some(("dangerous area", 3, polygon.len())),
        _ => None,
    }
}
