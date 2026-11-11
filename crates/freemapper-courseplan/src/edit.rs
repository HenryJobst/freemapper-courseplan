//! Edit operations on an event: controls, special objects, courses and classes.
//!
//! Every operation works on a clone of the event and validates the clone. Only a consistent result
//! is written back, so a failed operation leaves the event as it was. Only individual courses are
//! edited in M2a. Terms follow `docs/glossary.md`.

use freemapper_courseplan_model::{
    AcceptedFinding, Class, Control, ControlCode, ControlDescription, ControlKind, Course,
    CourseControl, CourseKind, Event, FindingKey, LegRef, MapPoint, ModelError, NumberPlacement,
    SpecialObject, validate,
};

use crate::check::{Place, check_of};
use crate::rules::{RuleSet, RulesError, load_rule_set};

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
    /// Another course already has this name.
    #[error("course {0} already exists")]
    NameTaken(String),
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
    /// The position is beyond the controls of the course. The library passes its 0-based index;
    /// the CLI and the MCP server pass the position counted from 1, as the user gave it.
    #[error("course {course} has no control at position {index}")]
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
    /// The rule is not active in the event's rule profile and overlays.
    #[error("rule {rule} is not an active rule of rule profile {profile}")]
    UnknownRule { rule: String, profile: String },
    /// An accepted finding without course, controls and legs is a finding of the whole event,
    /// which this rule never reports.
    #[error(
        "findings of rule {rule} sit at {}; name that place",
        place_list(places)
    )]
    FindingNeedsPlace { rule: String, places: Vec<Place> },
    /// A finding sits at one place, so its key names at most one of course, controls and legs,
    /// and its legs belong to one course.
    #[error(
        "a finding of rule {rule} has one place: give a course, controls or legs of one course"
    )]
    FindingHasOnePlace { rule: String },
    /// The rule's findings never sit at this kind of place, so the acceptance could never match.
    #[error("findings of rule {rule} sit at {}, never at {}", place_list(places), place_words(*place))]
    FindingWrongPlace {
        rule: String,
        place: Place,
        places: Vec<Place>,
    },
    /// No accepted finding has this key.
    #[error("there is no accepted finding of rule {rule} at this place")]
    AcceptedNotFound { rule: String },
    /// A start window of 0 minutes means no window; the start capacity check needs at least 1.
    #[error("the start window must be at least 1 minute")]
    StartWindowZero,
    /// A rule category must not be empty; clear it instead.
    #[error("the rule category of class {class} is empty; give a category or clear it")]
    BlankRuleCategory { class: String },
    /// The rule profile or an overlay cannot be loaded.
    #[error(transparent)]
    Rules(#[from] RulesError),
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

fn place_words(place: Place) -> &'static str {
    match place {
        Place::Event => "the event as a whole",
        Place::Course => "a course",
        Place::Control => "one control",
        Place::Controls => "two or more controls",
        Place::Leg => "a leg",
        Place::Legs => "two or more legs of one course",
    }
}

fn place_list(places: &[Place]) -> String {
    let words: Vec<&str> = places.iter().map(|p| place_words(*p)).collect();
    match words.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} or {last}", rest.join(", ")),
        _ => words.join(""),
    }
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
            ppen_id: None,
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

/// Sets the start window of the event: the minutes from the first to the last start, at least 1.
pub fn set_start_window(event: &mut Event, minutes: u32) -> Result<(), EditError> {
    if minutes == 0 {
        return Err(EditError::StartWindowZero);
    }
    apply(event, |e| {
        e.event.start_window_min = Some(minutes);
        Ok(())
    })
}

/// Changes the rule category or the expected number of starters of a class. `None` leaves a
/// value as it is; `Some(None)` clears it. A category is trimmed, and an empty one is refused.
pub fn set_class(
    event: &mut Event,
    name: &str,
    rule_category: Option<Option<String>>,
    expected_starters: Option<Option<u32>>,
) -> Result<(), EditError> {
    apply(event, |e| {
        let class = e
            .classes
            .iter_mut()
            .find(|c| c.name == name)
            .ok_or_else(|| not_found("class", name))?;
        if let Some(category) = rule_category {
            let category = category.map(|c| c.trim().to_owned());
            if category.as_deref() == Some("") {
                return Err(EditError::BlankRuleCategory {
                    class: name.to_owned(),
                });
            }
            class.rule_category = category;
        }
        if let Some(starters) = expected_starters {
            class.expected_starters = starters;
        }
        Ok(())
    })
}

/// Sets `from` and `to` of each leg to the ends the leg has in its course now (ruling R24).
fn fill_leg_ends(event: &Event, legs: &mut [LegRef]) -> Result<(), EditError> {
    for leg in legs {
        let course = event
            .courses
            .iter()
            .find(|c| c.name == leg.course)
            .ok_or_else(|| not_found("course", leg.course.clone()))?;
        let CourseKind::Individual { controls } = &course.kind else {
            return Err(EditError::NotSupported {
                course: course.name.clone(),
            });
        };
        let after = leg.after as usize;
        if after > controls.len() {
            return Err(EditError::BadIndex {
                course: leg.course.clone(),
                index: after,
            });
        }
        leg.from = match after {
            0 => course.start.clone(),
            n => controls[n - 1].code.0.clone(),
        };
        leg.to = controls
            .get(after)
            .map_or_else(|| course.finish.clone(), |c| c.code.0.clone());
    }
    Ok(())
}

/// What [`accept_finding`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accepted {
    Added,
    /// An accepted finding with the same key existed; its note is replaced.
    NoteReplaced,
}

/// Marks a finding as an intended exception. The rule must be active in `rules`; the course,
/// controls and legs must exist. The key names one place: nothing for a finding of the whole
/// event (only for rules whose findings can sit there), a course, one control, several controls,
/// one leg or several legs of one course. A place kind the rule's check never reports is refused
/// (`Check::places`, ADR 0026). A control may also be a start or finish id, for every rule.
/// A leg is named by course and position; its ends (`LegRef::from`, `to`) are taken from the
/// course as it is now, whatever the caller passed (ruling R24). Controls and legs are stored in
/// the order of their key. Accepting a key again replaces the note. Returns what it did and the
/// stored entry, whose legs carry the ends; an older entry at the same position with other ends
/// is a different key and stays as it is.
pub fn accept_finding(
    event: &mut Event,
    rules: &RuleSet,
    mut finding: AcceptedFinding,
) -> Result<(Accepted, AcceptedFinding), EditError> {
    let Some(set_rule) = rules.rule(&finding.rule) else {
        let mut profile = vec![rules.base.as_str()];
        profile.extend(rules.overlays.iter().map(String::as_str));
        return Err(EditError::UnknownRule {
            rule: finding.rule,
            profile: profile.join(" + "),
        });
    };
    let places = usize::from(finding.course.is_some())
        + usize::from(!finding.controls.is_empty())
        + usize::from(!finding.legs.is_empty());
    let one_place = || EditError::FindingHasOnePlace {
        rule: finding.rule.clone(),
    };
    let leg_courses = finding.legs.iter().map(|l| &l.course);
    let mixed_courses = leg_courses
        .clone()
        .zip(leg_courses.skip(1))
        .any(|(a, b)| a != b);
    if places > 1 || mixed_courses {
        return Err(one_place());
    }
    fill_leg_ends(event, &mut finding.legs)?;
    let key = finding.key();
    if let Some(check) = check_of(&set_rule.rule.key) {
        let place = Place::of_key(&key);
        if place == Place::Event && !check.reports_at(place) {
            return Err(EditError::FindingNeedsPlace {
                rule: finding.rule,
                places: check.places.to_vec(),
            });
        }
        if !check.reports_at(place) {
            return Err(EditError::FindingWrongPlace {
                rule: finding.rule,
                place,
                places: check.places.to_vec(),
            });
        }
    }
    apply(event, |e| {
        if let Some(name) = &key.course {
            course_mut(e, name)?;
        }
        for code in &key.controls {
            // A start or finish can be one of the places of a distance finding.
            let stop = e.specials.iter().any(|s| {
                matches!(
                    s,
                    SpecialObject::Start { .. } | SpecialObject::Finish { .. }
                ) && s.id() == code.0
            });
            if !stop {
                control_mut(e, code)?;
            }
        }
        if let Some(existing) = e.accepted_findings.iter_mut().find(|a| a.key() == key) {
            existing.note = finding.note;
            return Ok((Accepted::NoteReplaced, existing.clone()));
        }
        let stored = AcceptedFinding {
            rule: key.rule.clone(),
            controls: key.controls.clone(),
            course: key.course.clone(),
            legs: key.legs.clone(),
            note: finding.note,
        };
        e.accepted_findings.push(stored.clone());
        Ok((Accepted::Added, stored))
    })
}

/// Removes the accepted findings with this key, so the finding is reported again, and returns
/// them. Legs are named by course and position. The entry whose leg ends are the leg's ends in
/// the course now is removed if there is one. Only without such an exact match do legs compare
/// by course and position ([`FindingKey::by_position`]), so an acceptance whose leg now runs
/// between other stops, or whose course is gone, can still be removed; if that finds several
/// stale entries, all of them go, since none of them matches the course as it is.
pub fn remove_accepted_finding(
    event: &mut Event,
    key: &FindingKey,
) -> Result<Vec<AcceptedFinding>, EditError> {
    let mut legs = key.legs.clone();
    let exact = fill_leg_ends(event, &mut legs).ok().map(|()| {
        AcceptedFinding {
            rule: key.rule.clone(),
            controls: key.controls.clone(),
            course: key.course.clone(),
            legs,
            note: String::new(),
        }
        .key()
    });
    let exact = exact.filter(|k| event.accepted_findings.iter().any(|a| &a.key() == k));
    let wanted = key.by_position();
    apply(event, |e| {
        let (removed, kept): (Vec<AcceptedFinding>, Vec<AcceptedFinding>) =
            std::mem::take(&mut e.accepted_findings)
                .into_iter()
                .partition(|a| match &exact {
                    Some(k) => &a.key() == k,
                    None => a.key().by_position() == wanted,
                });
        e.accepted_findings = kept;
        if removed.is_empty() {
            return Err(EditError::AcceptedNotFound {
                rule: key.rule.clone(),
            });
        }
        Ok(removed)
    })
}

/// Sets the base rule profile and the overlays of the event, after loading them together with
/// [`load_rule_set`]. An overlay named twice counts once. Returns the loaded rule set.
pub fn set_rule_profile(
    event: &mut Event,
    base: &str,
    overlays: &[String],
) -> Result<RuleSet, EditError> {
    let mut unique: Vec<String> = Vec::new();
    for overlay in overlays {
        if !unique.contains(overlay) {
            unique.push(overlay.clone());
        }
    }
    let set = load_rule_set(base, &unique)?;
    apply(event, |e| {
        e.event.rule_profile = base.to_owned();
        e.event.rule_overlays = unique;
        Ok(())
    })?;
    Ok(set)
}

/// Changes the code of a control, and with it every place that names the control (see
/// [`rename_control`]). The new code must be free and valid; the same code is no change.
pub fn change_code(
    event: &mut Event,
    from: &ControlCode,
    to: &ControlCode,
) -> Result<(), EditError> {
    apply(event, |e| {
        control_mut(e, from)?;
        if from != to && e.controls.iter().any(|c| &c.code == to) {
            return Err(EditError::CodeTaken(to.clone()));
        }
        rename_control(e, from, to);
        Ok(())
    })
}

/// Changes the name of a course, and with it every place that names the course (see
/// [`rename_course`]). The new name must be free, not empty, without white space at its ends and
/// within the length limit; the same name is no change.
pub fn change_course_name(event: &mut Event, from: &str, to: &str) -> Result<(), EditError> {
    apply(event, |e| {
        course_mut(e, from)?;
        if from != to && e.courses.iter().any(|c| c.name == to) {
            return Err(EditError::NameTaken(to.to_owned()));
        }
        rename_course(e, from, to);
        Ok(())
    })
}

/// Renames a control everywhere the project names it: the control, the stops of courses of every kind,
/// and the controls and legs of accepted findings. Used by the sync when Purple Pen recodes a control.
/// It does not check that `to` is free.
pub fn rename_control(event: &mut Event, from: &ControlCode, to: &ControlCode) {
    let fix = |code: &mut ControlCode| {
        if code == from {
            code.clone_from(to);
        }
    };
    for control in &mut event.controls {
        fix(&mut control.code);
    }
    for course in &mut event.courses {
        match &mut course.kind {
            CourseKind::Individual { controls } => {
                controls.iter_mut().for_each(|c| fix(&mut c.code))
            }
            CourseKind::Team {
                mandatory, free, ..
            } => {
                mandatory.iter_mut().for_each(|c| fix(&mut c.code));
                free.iter_mut().for_each(fix);
            }
            CourseKind::Relay { parts, .. } => {
                for part in parts {
                    match part {
                        freemapper_courseplan_model::RelayPart::Common { controls } => {
                            controls.iter_mut().for_each(|c| fix(&mut c.code));
                        }
                        freemapper_courseplan_model::RelayPart::Fork { variants } => {
                            variants.iter_mut().flatten().for_each(|c| fix(&mut c.code))
                        }
                    }
                }
            }
            CourseKind::Score { controls, .. } => {
                controls.iter_mut().for_each(|c| fix(&mut c.code));
            }
        }
    }
    for finding in &mut event.accepted_findings {
        finding.controls.iter_mut().for_each(fix);
        for leg in &mut finding.legs {
            if leg.from == from.0 {
                leg.from.clone_from(&to.0);
            }
            if leg.to == from.0 {
                leg.to.clone_from(&to.0);
            }
        }
    }
}

/// Renames a course everywhere the project names it: the course, the course lists of special objects, and
/// the course and legs of accepted findings. Classes stay with the course. It does not check that `to` is
/// free.
pub fn rename_course(event: &mut Event, from: &str, to: &str) {
    for course in &mut event.courses {
        if course.name == from {
            to.clone_into(&mut course.name);
        }
    }
    for special in &mut event.specials {
        if let Some(list) = special_courses_mut(special) {
            for name in list {
                if name == from {
                    to.clone_into(name);
                }
            }
        }
    }
    for finding in &mut event.accepted_findings {
        if finding.course.as_deref() == Some(from) {
            finding.course = Some(to.to_owned());
        }
        for leg in &mut finding.legs {
            if leg.course == from {
                to.clone_into(&mut leg.course);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use freemapper_courseplan_model::{LegRef, RelayPart, ScoreControl};

    use super::*;

    fn code(s: &str) -> ControlCode {
        ControlCode(s.to_owned())
    }

    fn cc(s: &str) -> CourseControl {
        CourseControl {
            code: code(s),
            number: None,
            bends: vec![],
            taped: false,
        }
    }

    fn course(name: &str, kind: CourseKind) -> Course {
        Course {
            name: name.to_owned(),
            classes: vec!["Offen".to_owned()],
            start: "S".to_owned(),
            finish: "F".to_owned(),
            start_bends: vec![],
            start_taped: false,
            print_area: None,
            climb_m: None,
            planned_print_scale: None,
            ppen_id: None,
            kind,
        }
    }

    fn event() -> Event {
        let mut e: Event = toml::from_str(
            r#"
format = 1
[event]
title = "t"
[sources]
map = "m.omap"
"#,
        )
        .unwrap();
        e.courses = vec![
            course(
                "A",
                CourseKind::Individual {
                    controls: vec![cc("36"), cc("37")],
                },
            ),
            course(
                "T",
                CourseKind::Team {
                    team_size: 2,
                    mandatory: vec![cc("36")],
                    free: vec![code("36"), code("37")],
                },
            ),
            course(
                "R",
                CourseKind::Relay {
                    legs: 2,
                    parts: vec![
                        RelayPart::Common {
                            controls: vec![cc("36")],
                        },
                        RelayPart::Fork {
                            variants: vec![vec![cc("36")], vec![cc("37")]],
                        },
                    ],
                },
            ),
            course(
                "Sc",
                CourseKind::Score {
                    time_limit_min: 30,
                    penalty_per_min: 1,
                    controls: vec![ScoreControl {
                        code: code("36"),
                        value: 10,
                    }],
                },
            ),
        ];
        e.accepted_findings = vec![freemapper_courseplan_model::AcceptedFinding {
            rule: "r".to_owned(),
            controls: vec![code("36")],
            course: Some("A".to_owned()),
            legs: vec![LegRef {
                course: "A".to_owned(),
                after: 1,
                from: "36".to_owned(),
                to: "37".to_owned(),
            }],
            note: "n".to_owned(),
        }];
        e
    }

    #[test]
    fn rename_control_follows_into_every_course_kind_and_finding() {
        let mut e = event();
        rename_control(&mut e, &code("36"), &code("46"));
        let CourseKind::Individual { controls } = &e.courses[0].kind else {
            unreachable!()
        };
        assert_eq!(controls[0].code, code("46"));
        assert_eq!(controls[1].code, code("37"));
        let CourseKind::Team {
            mandatory, free, ..
        } = &e.courses[1].kind
        else {
            unreachable!()
        };
        assert_eq!(mandatory[0].code, code("46"));
        assert_eq!(free, &vec![code("46"), code("37")]);
        let CourseKind::Relay { parts, .. } = &e.courses[2].kind else {
            unreachable!()
        };
        assert!(
            matches!(&parts[0], RelayPart::Common { controls } if controls[0].code == code("46"))
        );
        assert!(
            matches!(&parts[1], RelayPart::Fork { variants } if variants[0][0].code == code("46") && variants[1][0].code == code("37"))
        );
        let CourseKind::Score { controls, .. } = &e.courses[3].kind else {
            unreachable!()
        };
        assert_eq!(controls[0].code, code("46"));
        assert_eq!(e.accepted_findings[0].controls, vec![code("46")]);
        assert_eq!(e.accepted_findings[0].legs[0].from, "46");
        assert_eq!(e.accepted_findings[0].legs[0].to, "37");
    }

    #[test]
    fn rename_course_keeps_classes_and_follows_references() {
        let mut e = event();
        e.specials = vec![SpecialObject::Water {
            id: "W1".to_owned(),
            ppen_id: None,
            map_mm: MapPoint { x: 0.0, y: 0.0 },
            courses: vec!["A".to_owned(), "T".to_owned()],
        }];
        rename_course(&mut e, "A", "Lang");
        assert_eq!(e.courses[0].name, "Lang");
        assert_eq!(e.courses[0].classes, vec!["Offen".to_owned()]);
        assert_eq!(e.specials[0].courses(), ["Lang".to_owned(), "T".to_owned()]);
        assert_eq!(e.accepted_findings[0].course.as_deref(), Some("Lang"));
        assert_eq!(e.accepted_findings[0].legs[0].course, "Lang");
    }
}
