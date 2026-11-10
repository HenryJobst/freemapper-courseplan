//! Course geometry: the legs of a course, control numbers and course length.
//!
//! Only individual courses are supported in M2a. A leg runs from one stop of the course (start,
//! control, crossing point, finish) to the next, through the bends of the leg. Terms follow
//! `docs/glossary.md`.

use freemapper_courseplan_model::{
    Control, ControlKind, Course, CourseControl, CourseKind, Event, MapPoint, SpecialObject,
};

/// Why a course cannot be measured.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CourseError {
    /// The course kind is not handled yet.
    #[error("course {course} is a {kind} course; only individual courses are supported yet")]
    NotSupported { course: String, kind: &'static str },
    /// The course is not part of the event.
    #[error("course {0} does not exist")]
    UnknownCourse(String),
    /// The course names a control or special object that the event does not have.
    #[error("course {course} refers to {id}, which the event does not define")]
    UnknownReference { course: String, id: String },
}

/// One leg of a course: the polyline from one stop to the next.
#[derive(Debug, Clone, PartialEq)]
pub struct LegGeometry {
    /// Start id, control code or finish id of the stop the leg leaves.
    pub from: String,
    /// Start id, control code or finish id of the stop the leg reaches.
    pub to: String,
    /// From the position of `from` through the bends to the position of `to`, in map mm.
    pub points: Vec<MapPoint>,
    /// The leg is a marked route.
    pub taped: bool,
    /// Length of the polyline in map mm.
    pub length_mm: f64,
}

fn kind_name(kind: &CourseKind) -> &'static str {
    match kind {
        CourseKind::Individual { .. } => "individual",
        CourseKind::Team { .. } => "team",
        CourseKind::Relay { .. } => "relay",
        CourseKind::Score { .. } => "score",
    }
}

fn individual_controls(course: &Course) -> Result<&[CourseControl], CourseError> {
    match &course.kind {
        CourseKind::Individual { controls } => Ok(controls),
        other => Err(CourseError::NotSupported {
            course: course.name.clone(),
            kind: kind_name(other),
        }),
    }
}

fn polyline_length(points: &[MapPoint]) -> f64 {
    points
        .windows(2)
        .map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y))
        .sum()
}

/// The legs of an individual course in running order: start to the first control, control to
/// control (crossing points count as stops), last control to the finish. A course without controls
/// has one leg from start to finish.
///
/// # Errors
/// [`CourseError::NotSupported`] for other kinds, [`CourseError::UnknownReference`] when the course
/// names a start, finish or control the event lacks.
pub fn course_legs(event: &Event, course: &Course) -> Result<Vec<LegGeometry>, CourseError> {
    let controls = individual_controls(course)?;
    let unknown = |id: &str| CourseError::UnknownReference {
        course: course.name.clone(),
        id: id.to_owned(),
    };
    let special_pos = |id: &str, want_start: bool| -> Result<MapPoint, CourseError> {
        event
            .specials
            .iter()
            .find_map(|s| match s {
                SpecialObject::Start { id: i, map_mm, .. } if want_start && i == id => {
                    Some(map_mm.clone())
                }
                SpecialObject::Finish { id: i, map_mm, .. } if !want_start && i == id => {
                    Some(map_mm.clone())
                }
                _ => None,
            })
            .ok_or_else(|| unknown(id))
    };

    // Stops in order: (label, position, bends of the leg leaving, taped).
    let mut stops: Vec<(String, MapPoint, &[MapPoint], bool)> = vec![(
        course.start.clone(),
        special_pos(&course.start, true)?,
        &course.start_bends,
        course.start_taped,
    )];
    for cc in controls {
        let control: &Control = event
            .controls
            .iter()
            .find(|c| c.code == cc.code)
            .ok_or_else(|| unknown(&cc.code.0))?;
        stops.push((
            cc.code.0.clone(),
            control.map_mm.clone(),
            &cc.bends,
            cc.taped,
        ));
    }
    let finish_pos = special_pos(&course.finish, false)?;

    let mut legs = Vec::with_capacity(stops.len());
    for (i, (from, pos, bends, taped)) in stops.iter().enumerate() {
        let (to, to_pos) = match stops.get(i + 1) {
            Some((label, p, _, _)) => (label.clone(), p.clone()),
            None => (course.finish.clone(), finish_pos.clone()),
        };
        let mut points = Vec::with_capacity(bends.len() + 2);
        points.push(pos.clone());
        points.extend(bends.iter().cloned());
        points.push(to_pos);
        legs.push(LegGeometry {
            from: from.clone(),
            to,
            length_mm: polyline_length(&points),
            points,
            taped: *taped,
        });
    }
    Ok(legs)
}

/// The number printed at each course control, one entry per control of the course. Crossing points
/// get `None`; the others count up from 1 in running order, a control visited twice gets two
/// numbers. Only individual courses are numbered: other kinds yield an empty list. A control the
/// event does not define counts as a normal control.
pub fn control_numbers(event: &Event, course: &Course) -> Vec<Option<u32>> {
    let Ok(controls) = individual_controls(course) else {
        return Vec::new();
    };
    let mut next = 0;
    controls
        .iter()
        .map(|cc| {
            let crossing = event
                .controls
                .iter()
                .any(|c| c.code == cc.code && c.kind == ControlKind::CrossingPoint);
            (!crossing).then(|| {
                next += 1;
                next
            })
        })
        .collect()
}

/// Length of the course in metres at the given map scale: the sum of all leg polylines in map mm
/// times `map_scale` / 1000 (WKB B 7.2.5: straight lines, drawn detours as bends, marked routes
/// measured along their bends).
///
/// # Errors
/// As for [`course_legs`].
pub fn course_length_m(event: &Event, course: &Course, map_scale: u32) -> Result<f64, CourseError> {
    let mm: f64 = course_legs(event, course)?
        .iter()
        .map(|l| l.length_mm)
        .sum();
    Ok(mm * f64::from(map_scale) / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_courseplan_model::{
        ControlCode, ControlDescription, EventMeta, FORMAT_VERSION, Sources,
    };

    fn p(x: f64, y: f64) -> MapPoint {
        MapPoint { x, y }
    }

    fn ctrl(code: &str, x: f64, y: f64, kind: ControlKind) -> Control {
        Control {
            ppen_id: None,
            code: ControlCode(code.to_owned()),
            kind,
            map_mm: p(x, y),
            description: ControlDescription::default(),
            number: None,
        }
    }

    fn cc(code: &str, bends: Vec<MapPoint>) -> CourseControl {
        CourseControl {
            code: ControlCode(code.to_owned()),
            number: None,
            bends,
            taped: false,
        }
    }

    fn course(kind: CourseKind) -> Course {
        Course {
            ppen_id: None,
            name: "A".to_owned(),
            classes: vec![],
            start: "S1".to_owned(),
            finish: "F1".to_owned(),
            start_bends: vec![],
            start_taped: false,
            print_area: None,
            climb_m: None,
            planned_print_scale: None,
            kind,
        }
    }

    fn event() -> Event {
        Event {
            format: FORMAT_VERSION,
            event: EventMeta {
                title: "T".to_owned(),
                date: None,
                discipline: Default::default(),
                rule_profile: "wkb-2026".to_owned(),
                rule_overlays: Vec::new(),
                start_window_min: None,
            },
            sources: Sources {
                map: "a.omap".into(),
                ppen: None,
            },
            classes: vec![],
            controls: vec![
                ctrl("31", 100.0, 0.0, ControlKind::Control),
                ctrl("32", 100.0, 100.0, ControlKind::Control),
                ctrl("33", 100.0, 50.0, ControlKind::CrossingPoint),
            ],
            specials: vec![
                SpecialObject::Start {
                    id: "S1".to_owned(),
                    ppen_id: None,
                    map_mm: p(0.0, 0.0),
                },
                SpecialObject::Finish {
                    id: "F1".to_owned(),
                    ppen_id: None,
                    map_mm: p(0.0, 100.0),
                },
            ],
            courses: vec![],
            accepted_findings: vec![],
        }
    }

    fn individual(controls: Vec<CourseControl>) -> Course {
        course(CourseKind::Individual { controls })
    }

    fn assert_near(got: f64, want: f64) {
        assert!((got - want).abs() < 0.01, "{got} vs {want}");
    }

    #[test]
    fn straight_course_length() {
        let c = individual(vec![cc("31", vec![]), cc("32", vec![])]);
        assert_near(course_length_m(&event(), &c, 4000).unwrap(), 1200.0);
        let legs = course_legs(&event(), &c).unwrap();
        assert_eq!(legs.len(), 3);
        assert_eq!((legs[0].from.as_str(), legs[0].to.as_str()), ("S1", "31"));
        assert_eq!((legs[2].from.as_str(), legs[2].to.as_str()), ("32", "F1"));
    }

    #[test]
    fn bend_adds_detour() {
        let c = individual(vec![cc("31", vec![p(150.0, 50.0)]), cc("32", vec![])]);
        let want = (100.0 + 2.0 * 5000f64.sqrt() + 100.0) * 4.0;
        assert_near(course_length_m(&event(), &c, 4000).unwrap(), want);
        assert_near(want, 1365.685);
    }

    #[test]
    fn start_bends_count() {
        let mut c = individual(vec![cc("31", vec![]), cc("32", vec![])]);
        c.start_bends = vec![p(0.0, -50.0)];
        let legs = course_legs(&event(), &c).unwrap();
        assert_near(
            legs[0].length_mm,
            50.0 + (100.0f64 * 100.0 + 50.0 * 50.0).sqrt(),
        );
    }

    #[test]
    fn crossing_point_is_a_vertex_without_number() {
        let c = individual(vec![cc("31", vec![]), cc("33", vec![]), cc("32", vec![])]);
        assert_near(course_length_m(&event(), &c, 4000).unwrap(), 1200.0);
        assert_eq!(control_numbers(&event(), &c), [Some(1), None, Some(2)]);
    }

    #[test]
    fn butterfly_numbers_both_visits() {
        // Each visit of 31 carries the bends of its own leg: the first towards 32, the second
        // towards the finish.
        let first = vec![p(120.0, 50.0)];
        let second = vec![p(50.0, 20.0), p(20.0, 60.0)];
        let c = individual(vec![
            cc("31", first.clone()),
            cc("32", vec![]),
            cc("31", second.clone()),
        ]);
        assert_eq!(control_numbers(&event(), &c), [Some(1), Some(2), Some(3)]);
        let legs = course_legs(&event(), &c).unwrap();
        assert_eq!(
            legs.iter()
                .filter(|l| l.from != "S1" && l.to != "F1")
                .count(),
            2
        );
        assert_eq!(legs.len(), 4);
        let route = |i: usize| (legs[i].from.as_str(), legs[i].to.as_str());
        assert_eq!(route(1), ("31", "32"));
        assert_eq!(legs[1].points[1..legs[1].points.len() - 1], first[..]);
        assert_eq!(route(3), ("31", "F1"));
        assert_eq!(legs[3].points[1..legs[3].points.len() - 1], second[..]);
        assert_eq!(legs[2].points.len(), 2);
    }

    #[test]
    fn course_without_controls_is_start_to_finish() {
        let mut c = individual(vec![]);
        c.start_bends = vec![p(50.0, 50.0)];
        c.start_taped = true;
        let legs = course_legs(&event(), &c).unwrap();
        assert_eq!(legs.len(), 1);
        assert_eq!((legs[0].from.as_str(), legs[0].to.as_str()), ("S1", "F1"));
        assert!(legs[0].taped);
        assert_near(legs[0].length_mm, 2.0 * 5000f64.sqrt());
        assert!(control_numbers(&event(), &c).is_empty());
    }

    #[test]
    fn team_course_is_not_supported() {
        let c = course(CourseKind::Team {
            team_size: 3,
            mandatory: vec![],
            free: vec![],
        });
        assert!(matches!(
            course_legs(&event(), &c),
            Err(CourseError::NotSupported { kind: "team", .. })
        ));
        assert!(matches!(
            course_length_m(&event(), &c, 4000),
            Err(CourseError::NotSupported { .. })
        ));
        assert!(control_numbers(&event(), &c).is_empty());
        let msg = course_legs(&event(), &c).unwrap_err().to_string();
        assert_eq!(
            msg,
            "course A is a team course; only individual courses are supported yet"
        );
    }

    #[test]
    fn unknown_control_is_an_error_not_a_panic() {
        let c = individual(vec![cc("99", vec![])]);
        assert!(matches!(
            course_legs(&event(), &c),
            Err(CourseError::UnknownReference { .. })
        ));
    }
}
