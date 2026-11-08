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

    let mut codes: HashSet<&ControlCode> = HashSet::new();
    let mut reported: HashSet<&ControlCode> = HashSet::new();
    for control in &event.controls {
        if !codes.insert(&control.code) && reported.insert(&control.code) {
            errors.push(ModelError::DuplicateControl(control.code.clone()));
        }
    }

    // Classes are matched by name when saving, and OE needs unique names.
    let mut class_names: HashSet<&str> = HashSet::new();
    let mut reported_classes: HashSet<&str> = HashSet::new();
    for class in &event.classes {
        let name = class.name.as_str();
        if !class_names.insert(name) && reported_classes.insert(name) {
            errors.push(ModelError::DuplicateClass(class.name.clone()));
        }
    }

    let mut starts: HashSet<&str> = HashSet::new();
    let mut finishes: HashSet<&str> = HashSet::new();
    let mut special_ids: HashSet<&str> = HashSet::new();
    for special in &event.specials {
        let id = match special {
            SpecialObject::Start { id, .. } => {
                starts.insert(id);
                id
            }
            SpecialObject::Finish { id, .. } => {
                finishes.insert(id);
                id
            }
            _ => continue,
        };
        if !special_ids.insert(id) {
            errors.push(ModelError::DuplicateSpecialId(id.clone()));
        }
    }

    let mut course_names: HashSet<&str> = HashSet::new();
    for course in &event.courses {
        let name = &course.name;
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
