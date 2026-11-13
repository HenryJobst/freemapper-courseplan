//! Consistency checks on an [`Event`]. These reject files that cannot be a consistent event; they do not
//! judge course quality (that is `check`).

use std::collections::{HashMap, HashSet};

use crate::{
    ControlCode, CourseKind, Event, FORMAT_VERSION, ModelError, RelayPart, ScoreControl, ScoreLink,
    ScoreSettings, SpecialObject,
};

/// Longest control code or special object id, in characters (the limit of freemapper-course).
pub const MAX_CODE_CHARS: usize = 64;
/// Longest course or class name and longest note, in characters.
pub const MAX_NAME_CHARS: usize = 200;

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
        check_len(
            &mut errors,
            "accepted finding note",
            &finding.note,
            MAX_NAME_CHARS,
        );
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
        check_len(&mut errors, "control code", &control.code.0, MAX_CODE_CHARS);
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
        check_len(&mut errors, "class name", name, MAX_NAME_CHARS);
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
        check_len(&mut errors, "special object id", id, MAX_CODE_CHARS);
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

    if let Some(score) = &event.score {
        check_score_settings(&mut errors, score);
    }

    let mut course_names: HashSet<&str> = HashSet::new();
    for course in &event.courses {
        let name = &course.name;
        check_name(&mut errors, "course name", name);
        check_len(&mut errors, "course name", name, MAX_NAME_CHARS);
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
            CourseKind::Score {
                controls, links, ..
            } => {
                used.extend(controls.iter().map(|c| &c.code));
                check_links(&mut errors, name, controls, links);
            }
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

/// Reports a text of more than `limit` characters.
fn check_len(errors: &mut Vec<ModelError>, what: &'static str, value: &str, limit: usize) {
    let len = value.chars().count();
    if len > limit {
        errors.push(ModelError::TooLong { what, limit, len });
    }
}

/// Reports a name that is empty or has white space at its start or end. Such names cannot be told
/// apart on the command line or in a list.
fn check_name(errors: &mut Vec<ModelError>, what: &'static str, value: &str) {
    if value.is_empty() || value.trim() != value {
        errors.push(ModelError::BlankName {
            what,
            value: value.to_owned(),
        });
    } else if let Some(bad) = value.chars().find(|c| is_invisible_control(*c)) {
        errors.push(ModelError::InvalidCharacter {
            what,
            point: format!("U+{:04X}", u32::from(bad)),
        });
    }
}

/// A control character or an invisible formatting character that no name or code may contain: it
/// makes XML ill-formed (NUL, most C0 controls) or hides what a name says (direction overrides, zero
/// width characters).
fn is_invisible_control(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}')
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

/// Links join two different controls of the course, none twice (either direction), and at most two meet at
/// one control, so the links form chains a runner can follow.
fn check_links(
    errors: &mut Vec<ModelError>,
    course: &str,
    controls: &[ScoreControl],
    links: &[ScoreLink],
) {
    let mut seen: HashSet<(&ControlCode, &ControlCode)> = HashSet::new();
    let mut degree: HashMap<&ControlCode, usize> = HashMap::new();
    let mut parent: HashMap<&ControlCode, &ControlCode> = HashMap::new();
    for link in links {
        let missing = [&link.a, &link.b]
            .into_iter()
            .find(|code| !controls.iter().any(|c| &c.code == *code));
        let unknown = missing.is_some();
        if let Some(code) = missing {
            errors.push(ModelError::LinkToUnknownControl {
                course: course.to_owned(),
                a: link.a.clone(),
                b: link.b.clone(),
                code: code.clone(),
            });
        }
        if link.a == link.b {
            errors.push(ModelError::LinkToItself {
                course: course.to_owned(),
                code: link.a.clone(),
            });
            continue;
        }
        if unknown {
            continue;
        }
        let key = if link.a.0 <= link.b.0 {
            (&link.a, &link.b)
        } else {
            (&link.b, &link.a)
        };
        if !seen.insert(key) {
            errors.push(ModelError::DuplicateLink {
                course: course.to_owned(),
                a: link.a.clone(),
                b: link.b.clone(),
            });
            continue;
        }
        let (ra, rb) = (root(&mut parent, &link.a), root(&mut parent, &link.b));
        if ra == rb {
            errors.push(ModelError::LinkCycle {
                course: course.to_owned(),
                a: link.a.clone(),
                b: link.b.clone(),
            });
            continue;
        }
        parent.insert(ra, rb);
        for code in [&link.a, &link.b] {
            *degree.entry(code).or_default() += 1;
        }
    }
    let mut branching: Vec<&ControlCode> = degree
        .into_iter()
        .filter_map(|(code, n)| (n > 2).then_some(code))
        .collect();
    branching.sort_by(|x, y| x.0.cmp(&y.0));
    for code in branching {
        errors.push(ModelError::LinkBranches {
            course: course.to_owned(),
            code: code.clone(),
        });
    }
}

/// Union-find root of `code` among the controls joined so far.
fn root<'a>(
    parent: &mut HashMap<&'a ControlCode, &'a ControlCode>,
    code: &'a ControlCode,
) -> &'a ControlCode {
    let mut at = code;
    while let Some(next) = parent.get(at) {
        at = next;
    }
    at
}

/// Least value of `min_gap_m` in metres (ADR 0072).
const MIN_GAP_FLOOR_M: u32 = 100;

fn check_score_settings(errors: &mut Vec<ModelError>, s: &ScoreSettings) {
    let mut bad = |what: &'static str, reason: &'static str| {
        errors.push(ModelError::ScoreSettingOutOfRange { what, reason });
    };
    if !(1..=9).contains(&s.value_max) {
        bad("value_max", "must be 1 to 9");
    }
    if s.good_pace_s_per_km == 0 {
        bad("good_pace_s_per_km", "must be more than 0");
    }
    if s.very_good_pace_s_per_km == 0 {
        bad("very_good_pace_s_per_km", "must be more than 0");
    }
    if s.good_pace_s_per_km > 0
        && s.very_good_pace_s_per_km > 0
        && s.very_good_pace_s_per_km >= s.good_pace_s_per_km
    {
        bad("very_good_pace_s_per_km", "must be below the good pace");
    }
    for (what, (low, high)) in [
        ("good_band_percent", s.good_band_percent),
        ("very_good_band_percent", s.very_good_band_percent),
    ] {
        if low > high {
            bad(what, "low is above high");
        } else if high > 100 {
            bad(what, "must not exceed 100");
        }
    }
    if s.very_good_band_percent.0 < s.good_band_percent.0 {
        bad(
            "very_good_band_percent",
            "must not start below the good band",
        );
    }
    if s.band_tolerance_percent > crate::MAX_BAND_TOLERANCE_PERCENT {
        bad("band_tolerance_percent", "must be 0 to 10");
    }
    if s.mix_hard_percent > 100 {
        bad("mix_hard_percent", "must not exceed 100");
    }
    if s.value1_min_m == 0 {
        bad("value1_min_m", "must be more than 0");
    }
    if s.min_gap_m < MIN_GAP_FLOOR_M {
        bad("min_gap_m", "must be at least 100");
    }
    if s.max_reach_m == Some(0) {
        bad("max_reach_m", "must be more than 0");
    }
    if let Some(reach) = s.max_reach_m
        && reach > 0
        && s.value1_min_m > reach
    {
        bad("max_reach_m", "must not be below value1_min_m");
    }
    if s.high_value_gap_m < s.min_gap_m {
        bad("high_value_gap_m", "must not be below min_gap_m");
    }
    if !(20..=300).contains(&s.dead_end_max_m) {
        bad("dead_end_max_m", "must be 20 to 300");
    }
    if !(50..=2000).contains(&s.start_finish_max_m) {
        bad("start_finish_max_m", "must be 50 to 2000");
    }
    if s.map_margin_m > 200 {
        bad("map_margin_m", "must be 0 to 200");
    }
}
