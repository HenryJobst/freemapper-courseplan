//! What every check reads: the event, the map index, the map kind and scale, and per individual
//! course its legs, control numbers and print scale. Built once per `check` run.

use freemapper_courseplan_model::{Control, ControlCode, Course, CourseControl, CourseKind, Event};

use super::{
    ArgValue, LegEnd, LegSpan, Location, NOTE_NON_STANDARD_SCALE, SKIP_COURSE_UNRESOLVED, Skipped,
};
use crate::course::{CourseError, LegGeometry, control_numbers, course_legs};
use crate::map::{MapIndex, MapKind, map_kind};
use crate::numbers::{
    OverprintSizes, base_scale, course_factor, is_standard_scale, overprint_sizes,
};
use crate::omap::MapDocument;
use crate::rules::print_scale_of;

/// Shared data of one check run.
pub struct CheckContext<'a> {
    pub event: &'a Event,
    pub map: &'a MapDocument,
    /// Terrain features of the map; sheet content is left out (`MapIndex`).
    pub index: MapIndex,
    pub map_kind: MapKind,
    /// Scale denominator of the map.
    pub map_scale: u32,
    /// Overprint sizes in print mm at the base scale of the map kind.
    pub sizes: OverprintSizes,
    /// The individual courses of the event, in file order. Other kinds are not checked.
    pub courses: Vec<CourseContext<'a>>,
    /// Individual courses left out of `courses` because their legs cannot be built, one skip each.
    pub unresolved: Vec<Skipped>,
}

/// One individual course as the checks see it.
pub struct CourseContext<'a> {
    pub course: &'a Course,
    /// The course controls in running order; crossing points included.
    pub controls: &'a [CourseControl],
    /// The legs from the start through every stop to the finish (`course::course_legs`).
    pub legs: Vec<LegGeometry>,
    /// The printed number of each course control; `None` for crossing points.
    pub numbers: Vec<Option<u32>>,
    /// The print scale the rule filters see (`rules::print_scale_of`).
    pub print_scale: u32,
}

impl CourseContext<'_> {
    /// The leg at `index` of [`CourseContext::legs`] (0 leaves the start), with its end stops.
    pub fn leg_span(&self, index: usize) -> LegSpan {
        let leg = &self.legs[index];
        LegSpan {
            after: index as u32,
            from: if index == 0 {
                LegEnd::Start(leg.from.clone())
            } else {
                LegEnd::Control(leg.from.clone())
            },
            to: if index + 1 == self.legs.len() {
                LegEnd::Finish(leg.to.clone())
            } else {
                LegEnd::Control(leg.to.clone())
            },
        }
    }

    /// The location of the leg at `index`.
    pub fn leg_location(&self, index: usize) -> Location {
        Location::Leg {
            course: self.course.name.clone(),
            leg: self.leg_span(index),
        }
    }
}

impl<'a> CheckContext<'a> {
    pub fn new(event: &'a Event, map: &'a MapDocument) -> CheckContext<'a> {
        CheckContext::with_kind(event, map, map_kind(map))
    }

    /// Like [`CheckContext::new`], with the map kind decided by the caller (see `map_kind_for_file`).
    pub fn with_kind(event: &'a Event, map: &'a MapDocument, kind: MapKind) -> CheckContext<'a> {
        let map_scale = map.georef.scale_denominator;
        let mut courses = Vec::new();
        let mut unresolved = Vec::new();
        for course in &event.courses {
            let CourseKind::Individual { controls } = &course.kind else {
                continue;
            };
            if !is_standard_scale(kind, map_scale) {
                unresolved.push(Skipped {
                    rule: None,
                    profile: None,
                    location: Location::Course(course.name.clone()),
                    reason_id: NOTE_NON_STANDARD_SCALE,
                    args: vec![
                        ("kind", ArgValue::Text(kind.name().to_owned())),
                        ("scale", ArgValue::Int(i64::from(map_scale))),
                        ("base_scale", ArgValue::Int(i64::from(base_scale(kind)))),
                    ],
                });
            }
            // A loaded project file is consistent, so this fails only for an event built in
            // code; the course is then reported as skipped, never as passed.
            match course_legs(event, course) {
                Ok(legs) => courses.push(CourseContext {
                    course,
                    controls,
                    legs,
                    numbers: control_numbers(event, course),
                    print_scale: print_scale_of(course, map_scale),
                }),
                Err(error) => {
                    let id = match error {
                        CourseError::UnknownReference { id, .. } => id,
                        _ => course.name.clone(),
                    };
                    unresolved.push(Skipped {
                        rule: None,
                        profile: None,
                        location: Location::Course(course.name.clone()),
                        reason_id: SKIP_COURSE_UNRESOLVED,
                        args: vec![("id", ArgValue::Text(id))],
                    });
                }
            }
        }
        CheckContext {
            event,
            map,
            index: MapIndex::from_document(map),
            map_kind: kind,
            map_scale,
            sizes: overprint_sizes(kind),
            courses,
            unresolved,
        }
    }

    /// The control with this code.
    pub fn control(&self, code: &ControlCode) -> Option<&'a Control> {
        self.event.controls.iter().find(|c| &c.code == code)
    }

    /// A length on the map in metres on the ground: `map_mm × map_scale / 1000`.
    pub fn metres(&self, map_mm: f64) -> f64 {
        map_mm * f64::from(self.map_scale) / 1000.0
    }

    /// A length on the ground in map millimetres.
    pub fn map_mm(&self, metres: f64) -> f64 {
        metres * 1000.0 / f64::from(self.map_scale)
    }

    /// A print size of the overprint in map millimetres: `print_mm × course_factor`
    /// (ADR 0024).
    pub fn print_to_map_mm(&self, print_mm: f64) -> f64 {
        print_mm * course_factor(self.map_kind, self.map_scale)
    }

    /// Radius of the control circle in map millimetres.
    pub fn circle_radius_mm(&self) -> f64 {
        self.print_to_map_mm(self.sizes.circle_diameter_mm) / 2.0
    }
}
