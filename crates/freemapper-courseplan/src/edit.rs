//! Edit operations on an event: controls, special objects, courses and classes.
//!
//! Every operation works on a clone of the event and validates the clone. Only a consistent result
//! is written back, so a failed operation leaves the event as it was. Only individual courses are
//! edited in M2a. Terms follow `docs/glossary.md`.

use freemapper_courseplan_model::{
    Class, Control, ControlCode, ControlDescription, ControlKind, Course, CourseControl,
    CourseKind, Event, MapPoint, ModelError, NumberPlacement, SpecialObject, validate,
};

/// Why an edit was refused.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EditError {
    /// The edited event would be inconsistent.
    #[error("the change would make the event inconsistent: {}", join_errors(.0))]
    Invalid(Vec<ModelError>),
    /// The named control, special object, course or class does not exist.
    #[error("there is no {what} {id}")]
    NotFound { what: &'static str, id: String },
    /// Another control already has this code.
    #[error("control {0} already exists")]
    CodeTaken(ControlCode),
    /// Another special object already has this id.
    #[error("special object {0} already exists")]
    IdTaken(String),
    /// A point lies outside the map.
    #[error(
        "point ({:.1}, {:.1}) is outside the map, which spans ({:.1}, {:.1}) to ({:.1}, {:.1})",
        at.x, at.y, min.x, min.y, max.x, max.y
    )]
    OutsideMap {
        at: MapPoint,
        min: MapPoint,
        max: MapPoint,
    },
    /// Courses still use the object.
    #[error("{id} is still used by {}; change those first", courses.join(", "))]
    InUse { id: String, courses: Vec<String> },
    /// The course kind cannot be edited yet.
    #[error(
        "course {course} is not an individual course; only individual courses can be edited yet"
    )]
    NotSupported { course: String },
    /// The index is beyond the controls of the course.
    #[error("course {course} has no control at index {index}")]
    BadIndex { course: String, index: usize },
    /// Removing the course would empty the course list of these special objects, and an empty
    /// list means all courses (ruling R5, amended).
    #[error("{}", cover_all_text(course, specials))]
    SpecialWouldCoverAllCourses {
        course: String,
        specials: Vec<String>,
    },
    /// Start and finish have no course list; each course names its own.
    #[error(
        "special object {0} is a start or finish; each course names its start and finish itself"
    )]
    StartOrFinishHasNoCourses(String),
}

fn cover_all_text(course: &str, specials: &[String]) -> String {
    let list = specials.join(", ");
    if specials.len() == 1 {
        format!(
            "course {course} is the only course of {list}; without it, {list} would apply to all \
             courses. Assign {list} to another course first or remove it"
        )
    } else {
        format!(
            "course {course} is the only course of {list}; without it, they would apply to all \
             courses. Assign them to another course first or remove them"
        )
    }
}

fn join_errors(errors: &[ModelError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

/// Where a leg begins: at the start, or after the control at the given 0-based index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegStart {
    Start,
    /// Index into the controls of the course. The leg after the last control runs to the finish.
    After(usize),
}

/// A leg whose bends and taped flag an edit removed, because the leg now runs to another stop
/// (ruling R12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearedLeg {
    pub course: String,
    /// Position of the stop the leg leaves, counted from 1 as on the map; 0 is the leg from the
    /// start.
    pub after: usize,
}

/// What [`remove_control`] changed besides the control list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemovedControl {
    /// The courses that lost a visit.
    pub courses: Vec<String>,
    /// The legs that lost their bends and taped flag.
    pub cleared: Vec<ClearedLeg>,
}

/// After a change of an individual course's sequence: clears the bends and taped flag of every
/// leg whose next stop changed and returns the legs that had any (ruling R12). `old_first` is the
/// first stop before the change; `old_next[j]` is the stop that the leg leaving new position `j`
/// ran to before. `None` stands for the finish, and also for new visits, which carry no leg data.
fn clear_changed_legs(
    course: &mut Course,
    old_first: Option<ControlCode>,
    old_next: &[Option<ControlCode>],
) -> Vec<ClearedLeg> {
    let Course {
        name,
        start_bends,
        start_taped,
        kind,
        ..
    } = course;
    let CourseKind::Individual { controls } = kind else {
        return Vec::new();
    };
    let mut cleared = Vec::new();
    let mut clear = |bends: &mut Vec<MapPoint>, taped: &mut bool, after: usize| {
        if !bends.is_empty() || *taped {
            bends.clear();
            *taped = false;
            cleared.push(ClearedLeg {
                course: name.clone(),
                after,
            });
        }
    };
    if controls.first().map(|c| &c.code) != old_first.as_ref() {
        clear(start_bends, start_taped, 0);
    }
    let next: Vec<Option<ControlCode>> = (0..controls.len())
        .map(|j| controls.get(j + 1).map(|c| c.code.clone()))
        .collect();
    for (j, cc) in controls.iter_mut().enumerate() {
        if next[j] != old_next.get(j).cloned().flatten() {
            clear(&mut cc.bends, &mut cc.taped, j + 1);
        }
    }
    cleared
}

/// Runs `change` on a clone of the event and writes the clone back when it is consistent.
pub(crate) fn apply<T>(
    event: &mut Event,
    change: impl FnOnce(&mut Event) -> Result<T, EditError>,
) -> Result<T, EditError> {
    let mut draft = event.clone();
    let out = change(&mut draft)?;
    let errors = validate(&draft);
    if !errors.is_empty() {
        return Err(EditError::Invalid(errors));
    }
    *event = draft;
    Ok(out)
}

fn check_inside(extent: &Option<(MapPoint, MapPoint)>, at: &MapPoint) -> Result<(), EditError> {
    if let Some((min, max)) = extent
        && !(min.x <= at.x && at.x <= max.x && min.y <= at.y && at.y <= max.y)
    {
        return Err(EditError::OutsideMap {
            at: at.clone(),
            min: min.clone(),
            max: max.clone(),
        });
    }
    Ok(())
}

fn not_found(what: &'static str, id: impl Into<String>) -> EditError {
    EditError::NotFound {
        what,
        id: id.into(),
    }
}

fn control_mut<'a>(event: &'a mut Event, code: &ControlCode) -> Result<&'a mut Control, EditError> {
    event
        .controls
        .iter_mut()
        .find(|c| &c.code == code)
        .ok_or_else(|| not_found("control", code.0.clone()))
}

pub(crate) fn course_mut<'a>(
    event: &'a mut Event,
    name: &str,
) -> Result<&'a mut Course, EditError> {
    event
        .courses
        .iter_mut()
        .find(|c| c.name == name)
        .ok_or_else(|| not_found("course", name))
}

pub(crate) fn individual_mut(course: &mut Course) -> Result<&mut Vec<CourseControl>, EditError> {
    match &mut course.kind {
        CourseKind::Individual { controls } => Ok(controls),
        _ => Err(EditError::NotSupported {
            course: course.name.clone(),
        }),
    }
}

fn special_points(special: &SpecialObject) -> Vec<&MapPoint> {
    match special {
        SpecialObject::Start { map_mm, .. }
        | SpecialObject::Finish { map_mm, .. }
        | SpecialObject::FirstAid { map_mm, .. }
        | SpecialObject::Water { map_mm, .. } => vec![map_mm],
        SpecialObject::MarkedRoute { points, .. }
        | SpecialObject::ForbiddenRoute { points, .. } => points.iter().collect(),
        SpecialObject::OutOfBoundsArea { polygon, .. }
        | SpecialObject::DangerousArea { polygon, .. } => polygon.iter().collect(),
    }
}

fn special_courses_mut(special: &mut SpecialObject) -> Option<&mut Vec<String>> {
    match special {
        SpecialObject::Start { .. } | SpecialObject::Finish { .. } => None,
        SpecialObject::MarkedRoute { courses, .. }
        | SpecialObject::OutOfBoundsArea { courses, .. }
        | SpecialObject::ForbiddenRoute { courses, .. }
        | SpecialObject::DangerousArea { courses, .. }
        | SpecialObject::FirstAid { courses, .. }
        | SpecialObject::Water { courses, .. } => Some(courses),
    }
}

/// The lowest integer code from 31 up that no control uses. Codes that are not numbers are ignored.
pub fn next_free_code(event: &Event) -> ControlCode {
    let used: std::collections::HashSet<u64> = event
        .controls
        .iter()
        .filter_map(|c| c.code.0.parse().ok())
        .collect();
    let free = (31u64..).find(|n| !used.contains(n)).unwrap_or(31);
    ControlCode(free.to_string())
}

/// Adds a control at `at`. Without a code the next free code is taken. `extent` comes from
/// `MapIndex::extent`; `None` skips the map check. Returns the code of the new control.
pub fn add_control(
    event: &mut Event,
    extent: Option<(MapPoint, MapPoint)>,
    at: MapPoint,
    code: Option<ControlCode>,
    kind: ControlKind,
) -> Result<ControlCode, EditError> {
    apply(event, |e| {
        check_inside(&extent, &at)?;
        let code = code.unwrap_or_else(|| next_free_code(e));
        if e.controls.iter().any(|c| c.code == code) {
            return Err(EditError::CodeTaken(code));
        }
        e.controls.push(Control {
            code: code.clone(),
            kind,
            map_mm: at,
            description: ControlDescription::default(),
            number: None,
        });
        Ok(code)
    })
}

/// Moves a control to `at`.
pub fn move_control(
    event: &mut Event,
    extent: Option<(MapPoint, MapPoint)>,
    code: &ControlCode,
    at: MapPoint,
) -> Result<(), EditError> {
    apply(event, |e| {
        let control = control_mut(e, code)?;
        check_inside(&extent, &at)?;
        control.map_mm = at;
        Ok(())
    })
}

/// Removes a control and every visit of it in individual courses. A leg that ran to a removed
/// visit now runs to the next stop, so it loses its bends and taped flag (ruling R12). Returns the
/// courses that lost a visit and the cleared legs. A team, relay or score course that uses the
/// control refuses the removal.
pub fn remove_control(event: &mut Event, code: &ControlCode) -> Result<RemovedControl, EditError> {
    apply(event, |e| {
        if !e.controls.iter().any(|c| &c.code == code) {
            return Err(not_found("control", code.0.clone()));
        }
        let uses = |controls: &[CourseControl]| controls.iter().any(|c| &c.code == code);
        let blocked: Vec<String> = e
            .courses
            .iter()
            .filter(|course| match &course.kind {
                CourseKind::Individual { .. } => false,
                CourseKind::Team {
                    mandatory, free, ..
                } => uses(mandatory) || free.contains(code),
                CourseKind::Relay { parts, .. } => parts.iter().any(|p| match p {
                    freemapper_courseplan_model::RelayPart::Common { controls } => uses(controls),
                    freemapper_courseplan_model::RelayPart::Fork { variants } => {
                        variants.iter().any(|v| uses(v))
                    }
                }),
                CourseKind::Score { controls, .. } => controls.iter().any(|c| &c.code == code),
            })
            .map(|c| c.name.clone())
            .collect();
        if !blocked.is_empty() {
            return Err(EditError::InUse {
                id: code.0.clone(),
                courses: blocked,
            });
        }
        let mut removed = RemovedControl::default();
        for course in &mut e.courses {
            let CourseKind::Individual { controls } = &mut course.kind else {
                continue;
            };
            if !uses(controls) {
                continue;
            }
            let old_first = controls.first().map(|c| c.code.clone());
            let old_next: Vec<Option<ControlCode>> = (0..controls.len())
                .filter(|&k| &controls[k].code != code)
                .map(|k| controls.get(k + 1).map(|c| c.code.clone()))
                .collect();
            controls.retain(|c| &c.code != code);
            removed.courses.push(course.name.clone());
            removed
                .cleared
                .extend(clear_changed_legs(course, old_first, &old_next));
        }
        e.controls.retain(|c| &c.code != code);
        Ok(removed)
    })
}

/// Replaces the description of a control.
pub fn describe_control(
    event: &mut Event,
    code: &ControlCode,
    description: ControlDescription,
) -> Result<(), EditError> {
    apply(event, |e| {
        control_mut(e, code)?.description = description;
        Ok(())
    })
}

/// Sets or clears the default number placement of a control.
pub fn set_central_number(
    event: &mut Event,
    code: &ControlCode,
    number: Option<NumberPlacement>,
) -> Result<(), EditError> {
    apply(event, |e| {
        control_mut(e, code)?.number = number;
        Ok(())
    })
}

/// Adds a special object. Its id must be new and all its points must lie on the map.
pub fn add_special(
    event: &mut Event,
    extent: Option<(MapPoint, MapPoint)>,
    special: SpecialObject,
) -> Result<(), EditError> {
    apply(event, |e| {
        if e.specials.iter().any(|s| s.id() == special.id()) {
            return Err(EditError::IdTaken(special.id().to_owned()));
        }
        for p in special_points(&special) {
            check_inside(&extent, p)?;
        }
        e.specials.push(special);
        Ok(())
    })
}

/// Removes a special object. A start or finish that a course uses stays.
pub fn remove_special(event: &mut Event, id: &str) -> Result<(), EditError> {
    apply(event, |e| {
        if !e.specials.iter().any(|s| s.id() == id) {
            return Err(not_found("special object", id));
        }
        let courses: Vec<String> = e
            .courses
            .iter()
            .filter(|c| c.start == id || c.finish == id)
            .map(|c| c.name.clone())
            .collect();
        if !courses.is_empty() {
            return Err(EditError::InUse {
                id: id.to_owned(),
                courses,
            });
        }
        e.specials.retain(|s| s.id() != id);
        Ok(())
    })
}

/// Sets the courses a special object belongs to; an empty list means all courses. Start and
/// finish have no course list. Unknown and repeated course names make the event inconsistent.
pub fn set_special_courses(
    event: &mut Event,
    id: &str,
    courses: Vec<String>,
) -> Result<(), EditError> {
    apply(event, |e| {
        let special = e
            .specials
            .iter_mut()
            .find(|s| s.id() == id)
            .ok_or_else(|| not_found("special object", id))?;
        let list = special_courses_mut(special)
            .ok_or_else(|| EditError::StartOrFinishHasNoCourses(id.to_owned()))?;
        *list = courses;
        Ok(())
    })
}

/// Adds an individual course.
pub fn add_course(event: &mut Event, course: Course) -> Result<(), EditError> {
    apply(event, |e| {
        if !matches!(course.kind, CourseKind::Individual { .. }) {
            return Err(EditError::NotSupported {
                course: course.name,
            });
        }
        e.courses.push(course);
        Ok(())
    })
}

/// Removes a course of any kind and its name from the course lists of special objects. A special
/// object that would be left with an empty list (meaning all courses) stays and refuses the removal.
pub fn remove_course(event: &mut Event, name: &str) -> Result<(), EditError> {
    apply(event, |e| {
        course_mut(e, name)?;
        let widened: Vec<String> = e
            .specials
            .iter()
            .filter(|s| !s.courses().is_empty() && s.courses().iter().all(|c| c == name))
            .map(|s| s.id().to_owned())
            .collect();
        if !widened.is_empty() {
            return Err(EditError::SpecialWouldCoverAllCourses {
                course: name.to_owned(),
                specials: widened,
            });
        }
        for special in &mut e.specials {
            if let Some(list) = special_courses_mut(special) {
                list.retain(|c| c != name);
            }
        }
        e.courses.retain(|c| c.name != name);
        Ok(())
    })
}

/// Replaces the sequence of an individual course. The n-th visit of a code in the new sequence keeps
/// the number of the n-th visit of that code in the old sequence, and its bends and `taped` flag
/// only while its leg runs to the same next stop as before (ruling R12); new visits start empty.
/// The leg from the start keeps its bends only while the first stop stays. Returns the legs that
/// lost bends or the taped flag.
pub fn set_course_controls(
    event: &mut Event,
    name: &str,
    codes: &[ControlCode],
) -> Result<Vec<ClearedLeg>, EditError> {
    apply(event, |e| {
        let known: Vec<ControlCode> = e.controls.iter().map(|c| c.code.clone()).collect();
        let controls = individual_mut(course_mut(e, name)?)?;
        if let Some(unknown) = codes.iter().find(|code| !known.contains(code)) {
            return Err(not_found("control", unknown.0.clone()));
        }
        let old = std::mem::take(controls);
        let old_first = old.first().map(|c| c.code.clone());
        let mut old_next = Vec::with_capacity(codes.len());
        let mut seen: std::collections::HashMap<&ControlCode, usize> = Default::default();
        for code in codes {
            let n = seen.entry(code).or_insert(0);
            let previous = old
                .iter()
                .enumerate()
                .filter(|(_, c)| &c.code == code)
                .nth(*n);
            *n += 1;
            old_next.push(previous.and_then(|(k, _)| old.get(k + 1).map(|c| c.code.clone())));
            controls.push(
                previous
                    .map(|(_, c)| c.clone())
                    .unwrap_or_else(|| CourseControl {
                        code: code.clone(),
                        number: None,
                        bends: vec![],
                        taped: false,
                    }),
            );
        }
        let course = course_mut(e, name)?;
        Ok(clear_changed_legs(course, old_first, &old_next))
    })
}

/// Sets the bends and the marked-route flag of one leg of an individual course.
pub fn set_leg(
    event: &mut Event,
    name: &str,
    leg: LegStart,
    bends: Vec<MapPoint>,
    taped: bool,
) -> Result<(), EditError> {
    apply(event, |e| {
        let course = course_mut(e, name)?;
        match leg {
            LegStart::Start => {
                individual_mut(course)?;
                course.start_bends = bends;
                course.start_taped = taped;
            }
            LegStart::After(index) => {
                let course_name = course.name.clone();
                let cc = individual_mut(course)?
                    .get_mut(index)
                    .ok_or(EditError::BadIndex {
                        course: course_name,
                        index,
                    })?;
                cc.bends = bends;
                cc.taped = taped;
            }
        }
        Ok(())
    })
}

/// Sets or clears the number placement of the control at `index` in an individual course.
pub fn set_course_number(
    event: &mut Event,
    name: &str,
    index: usize,
    number: Option<NumberPlacement>,
) -> Result<(), EditError> {
    apply(event, |e| {
        let course = course_mut(e, name)?;
        let course_name = course.name.clone();
        let cc = individual_mut(course)?
            .get_mut(index)
            .ok_or(EditError::BadIndex {
                course: course_name,
                index,
            })?;
        cc.number = number;
        Ok(())
    })
}

/// Changes class list, climb or planned print scale of an individual course. `None` leaves a value
/// as it is; `Some(None)` clears climb or print scale.
pub fn set_course_details(
    event: &mut Event,
    name: &str,
    classes: Option<Vec<String>>,
    climb_m: Option<Option<u32>>,
    planned_print_scale: Option<Option<u32>>,
) -> Result<(), EditError> {
    apply(event, |e| {
        let course = course_mut(e, name)?;
        individual_mut(course)?;
        if let Some(classes) = classes {
            course.classes = classes;
        }
        if let Some(climb) = climb_m {
            course.climb_m = climb;
        }
        if let Some(scale) = planned_print_scale {
            course.planned_print_scale = scale;
        }
        Ok(())
    })
}

/// Adds a class.
pub fn add_class(event: &mut Event, class: Class) -> Result<(), EditError> {
    apply(event, |e| {
        e.classes.push(class);
        Ok(())
    })
}

/// Removes a class and its name from every course.
pub fn remove_class(event: &mut Event, name: &str) -> Result<(), EditError> {
    apply(event, |e| {
        if !e.classes.iter().any(|c| c.name == name) {
            return Err(not_found("class", name));
        }
        e.classes.retain(|c| c.name != name);
        for course in &mut e.courses {
            course.classes.retain(|c| c != name);
        }
        Ok(())
    })
}
