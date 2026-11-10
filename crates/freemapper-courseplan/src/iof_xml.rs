//! IOF XML 3.0 `CourseData` export for result software such as OE.
//!
//! The writer turns an [`Event`] and the map scale into one XML string. Only individual courses are
//! written; other kinds are skipped and named in the result. Positions are map mm at the map scale
//! with the y axis pointing down (the schema counts y in units below the centre), as the project stores them, so y is written unmirrored. Geographic
//! positions (`Position`, latitude and longitude with six decimals) are written when the caller passes the
//! [`MapGrid`] of a georeferenced map; the grid gives them in the same map frame, so nothing is flipped.
//! Terms follow `docs/glossary.md`.

use std::io;

pub use freemapper_course::MapGrid;
use freemapper_courseplan_model::{
    Control, ControlKind, Course, CourseKind, Event, MapPoint, NumberPlacement, SpecialObject,
};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesText, Event as Xml};

use crate::course::{CourseError, control_numbers, course_legs, course_length_m};
use crate::map::MapKind;
use crate::numbers::number_centre;

#[cfg(test)]
mod tests;

const NAMESPACE: &str = "http://www.orienteering.org/datastandard/3.0";
const CREATOR: &str = "freemapper-courseplan";

/// The written file and the courses it leaves out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IofXmlExport {
    pub xml: String,
    /// The controls carry a geographic `Position` (the map was georeferenced).
    pub geo_positions: bool,
    pub skipped: Vec<SkippedCourse>,
}

/// A course that is not an individual course and therefore not exported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedCourse {
    pub name: String,
    /// `team`, `relay` or `score`.
    pub kind: &'static str,
}

/// Why the export failed.
#[derive(Debug, thiserror::Error)]
pub enum IofXmlError {
    /// A course cannot be measured, for example because it names an unknown control.
    #[error("course {course}: {source}")]
    Course { course: String, source: CourseError },
}

/// One stop of a course as the XML lists it.
struct Stop {
    code: String,
    kind: &'static str,
    leg_m: Option<f64>,
    /// Number text and the offset of its centre from the control, in IOF map mm (y down).
    map_text: Option<(u32, MapPoint)>,
}

struct CourseOut<'a> {
    course: &'a Course,
    length_m: f64,
    stops: Vec<Stop>,
}

/// Formats a number with at most two decimals, without trailing zeros and without `-0`.
fn fmt(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded == 0.0 {
        return "0".to_owned();
    }
    let mut s = format!("{rounded:.2}");
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

fn kind_label(kind: &CourseKind) -> &'static str {
    match kind {
        CourseKind::Individual { .. } => "individual",
        CourseKind::Team { .. } => "team",
        CourseKind::Relay { .. } => "relay",
        CourseKind::Score { .. } => "score",
    }
}

fn start_or_finish(special: &SpecialObject) -> Option<(&str, &str, &MapPoint)> {
    match special {
        SpecialObject::Start { id, map_mm, .. } => Some(("Start", id, map_mm)),
        SpecialObject::Finish { id, map_mm, .. } => Some(("Finish", id, map_mm)),
        _ => None,
    }
}

fn plan_course<'a>(
    event: &'a Event,
    course: &'a Course,
    map_scale: u32,
    kind: MapKind,
) -> Result<CourseOut<'a>, CourseError> {
    let CourseKind::Individual { controls } = &course.kind else {
        unreachable!("only individual courses are planned");
    };
    let legs = course_legs(event, course)?;
    let length_m = course_length_m(event, course, map_scale)?;
    let numbers = control_numbers(event, course);
    let leg_m = |i: usize| {
        legs.get(i)
            .map(|l| l.length_mm * f64::from(map_scale) / 1000.0)
    };

    let mut stops = vec![Stop {
        code: course.start.clone(),
        kind: "Start",
        leg_m: None,
        map_text: None,
    }];
    for (i, cc) in controls.iter().enumerate() {
        let control: &Control = event
            .controls
            .iter()
            .find(|c| c.code == cc.code)
            .expect("course_legs checked that the control exists");
        let number = numbers.get(i).copied().flatten();
        // A course override wins over the control's own placement.
        let placement: Option<&NumberPlacement> = cc.number.as_ref().or(control.number.as_ref());
        let map_text = number.zip(placement).map(|(n, p)| {
            let centre = number_centre(&control.map_mm, p, n, kind, map_scale);
            let offset = MapPoint {
                x: centre.x - control.map_mm.x,
                y: centre.y - control.map_mm.y,
            };
            (n, offset)
        });
        stops.push(Stop {
            code: cc.code.0.clone(),
            kind: match control.kind {
                ControlKind::Control => "Control",
                ControlKind::CrossingPoint => "CrossingPoint",
            },
            leg_m: leg_m(i),
            map_text,
        });
    }
    stops.push(Stop {
        code: course.finish.clone(),
        kind: "Finish",
        leg_m: leg_m(controls.len()),
        map_text: None,
    });
    Ok(CourseOut {
        course,
        length_m,
        stops,
    })
}

type Out = Writer<Vec<u8>>;

fn text_element(w: &mut Out, name: &str, text: &str) -> io::Result<()> {
    w.create_element(name)
        .write_text_content(BytesText::new(text))?;
    Ok(())
}

fn position(w: &mut Out, name: &str, x: f64, y: f64) -> io::Result<()> {
    w.create_element(name)
        .with_attribute(("x", fmt(x).as_str()))
        .with_attribute(("y", fmt(y).as_str()))
        .with_attribute(("unit", "mm"))
        .write_empty()?;
    Ok(())
}

/// Latitude and longitude of `at` as a `Position` element, six decimals.
fn geo_position(w: &mut Out, grid: &MapGrid, at: &MapPoint) -> io::Result<()> {
    let (lat, lng) = grid.latlon_of((at.x, at.y));
    w.create_element("Position")
        .with_attribute(("lat", format!("{lat:.6}").as_str()))
        .with_attribute(("lng", format!("{lng:.6}").as_str()))
        .write_empty()?;
    Ok(())
}

fn control_element(
    w: &mut Out,
    kind: &str,
    id: &str,
    at: &MapPoint,
    geo: Option<&MapGrid>,
) -> io::Result<()> {
    w.create_element("Control")
        .with_attribute(("type", kind))
        .write_inner_content(|w| {
            text_element(w, "Id", id)?;
            if let Some(grid) = geo {
                geo_position(w, grid, at)?;
            }
            position(w, "MapPosition", at.x, at.y)
        })?;
    Ok(())
}

fn course_element(w: &mut Out, out: &CourseOut<'_>) -> io::Result<()> {
    w.create_element("Course").write_inner_content(|w| {
        text_element(w, "Name", &out.course.name)?;
        text_element(w, "Length", &fmt(out.length_m))?;
        if let Some(climb) = out.course.climb_m {
            text_element(w, "Climb", &climb.to_string())?;
        }
        for stop in &out.stops {
            w.create_element("CourseControl")
                .with_attribute(("type", stop.kind))
                .write_inner_content(|w| {
                    text_element(w, "Control", &stop.code)?;
                    if let Some((number, offset)) = &stop.map_text {
                        text_element(w, "MapText", &number.to_string())?;
                        position(w, "MapTextPosition", offset.x, offset.y)?;
                    }
                    if let Some(leg) = stop.leg_m {
                        text_element(w, "LegLength", &fmt(leg))?;
                    }
                    Ok(())
                })?;
        }
        Ok(())
    })?;
    Ok(())
}

fn write_document(
    event: &Event,
    map_scale: u32,
    geo: Option<&MapGrid>,
    created: &str,
    courses: &[CourseOut<'_>],
) -> io::Result<Vec<u8>> {
    let mut w = Writer::new_with_indent(Vec::new(), b' ', 2);
    w.write_event(Xml::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    w.create_element("CourseData")
        .with_attribute(("xmlns", NAMESPACE))
        .with_attribute(("iofVersion", "3.0"))
        .with_attribute(("createTime", created))
        .with_attribute(("creator", CREATOR))
        .write_inner_content(|w| {
            w.create_element("Event")
                .write_inner_content(|w| text_element(w, "Name", &event.event.title))?;
            w.create_element("RaceCourseData")
                .write_inner_content(|w| {
                    w.create_element("Map").write_inner_content(|w| {
                        text_element(w, "Scale", &map_scale.to_string())?;
                        position(w, "MapPositionTopLeft", 0.0, 0.0)?;
                        position(w, "MapPositionBottomRight", 0.0, 0.0)
                    })?;
                    for special in &event.specials {
                        if let Some((kind, id, at)) = start_or_finish(special) {
                            control_element(w, kind, id, at, geo)?;
                        }
                    }
                    for control in &event.controls {
                        let kind = match control.kind {
                            ControlKind::Control => "Control",
                            ControlKind::CrossingPoint => "CrossingPoint",
                        };
                        control_element(w, kind, &control.code.0, &control.map_mm, geo)?;
                    }
                    for out in courses {
                        course_element(w, out)?;
                    }
                    for out in courses {
                        for class in &out.course.classes {
                            w.create_element("ClassCourseAssignment")
                                .write_inner_content(|w| {
                                    text_element(w, "ClassName", class)?;
                                    text_element(w, "CourseName", &out.course.name)
                                })?;
                        }
                    }
                    Ok(())
                })?;
            Ok(())
        })?;
    Ok(w.into_inner())
}

/// Writes the individual courses of `event` as IOF XML 3.0 `CourseData`.
///
/// `map_scale` is the scale denominator and `map_kind` the kind of the map (it sets the overprint sizes
/// for the number positions), `geo` the grid of the georeferenced map, which adds a geographic `Position`
/// to every control (none without it), `created` an ISO 8601 date and time such as
/// `2026-11-09T12:00:00Z` for `createTime`. Team, relay and score courses are left out and named in
/// [`IofXmlExport::skipped`].
///
/// # Errors
/// [`IofXmlError::Course`] when an individual course cannot be measured; nothing is written then.
pub fn export_iof_xml(
    event: &Event,
    map_scale: u32,
    map_kind: MapKind,
    geo: Option<&MapGrid>,
    created: &str,
) -> Result<IofXmlExport, IofXmlError> {
    let mut courses = Vec::new();
    let mut skipped = Vec::new();
    for course in &event.courses {
        if matches!(course.kind, CourseKind::Individual { .. }) {
            let out = plan_course(event, course, map_scale, map_kind).map_err(|source| {
                IofXmlError::Course {
                    course: course.name.clone(),
                    source,
                }
            })?;
            courses.push(out);
        } else {
            skipped.push(SkippedCourse {
                name: course.name.clone(),
                kind: kind_label(&course.kind),
            });
        }
    }
    let bytes = write_document(event, map_scale, geo, created, &courses)
        .expect("writing to a Vec<u8> cannot fail");
    let xml = String::from_utf8(bytes).expect("quick-xml writes UTF-8 from UTF-8 input");
    Ok(IofXmlExport {
        xml,
        geo_positions: geo.is_some(),
        skipped,
    })
}
