//! Automatic placement of control numbers.
//!
//! A deterministic first version without map detail: a number goes where it touches the fewest
//! legs, circles and other numbers, and otherwise points into the widest gap between the legs. The
//! shared `freemapper-placement` module replaces this later (ADR 0013, ADR 0014).
//!
//! Conventions: map coordinates are millimetres with y down, so "north" on screen is -y. A
//! direction is in degrees, 0 = east, counter-clockwise, hence 90 degrees points to -y (north).
//! Overprint sizes are print millimetres at the base scale of the map kind and grow to map
//! millimetres by `map_mm = print_mm * base_scale / map_scale`. Terms follow `docs/glossary.md`.

use freemapper_courseplan_model::{CourseKind, Event, MapPoint, NumberPlacement};

use crate::course::{CourseError, control_numbers, course_legs};
use crate::edit::{EditError, apply, course_mut, individual_mut};
use crate::map::MapKind;

/// Gap between circle and number box in print mm.
const GAP_PRINT_MM: f64 = 0.5;
/// Width of one digit relative to the number height.
const DIGIT_WIDTH_FACTOR: f64 = 0.6;
/// Directions tried, in degrees: 16 steps of 22.5 degrees starting at 45.
const CANDIDATE_COUNT: u32 = 16;
const CANDIDATE_START: f64 = 45.0;
const CANDIDATE_STEP: f64 = 22.5;
const EPS: f64 = 1e-9;

/// Sizes of the course overprint in print mm at the base scale of the map kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverprintSizes {
    pub circle_diameter_mm: f64,
    pub line_width_mm: f64,
    pub number_height_mm: f64,
}

/// Overprint sizes of a map kind: ISSprOM 2019-2 for sprint, ISOM 2017-2 for forest. The schoolyard
/// uses the sprint print values; that is an assumption until the overprint of the school set is
/// checked.
pub fn overprint_sizes(kind: MapKind) -> OverprintSizes {
    match kind {
        MapKind::Sprint | MapKind::Schoolyard => OverprintSizes {
            circle_diameter_mm: 6.0,
            line_width_mm: 0.35,
            number_height_mm: 4.0,
        },
        MapKind::Forest => OverprintSizes {
            circle_diameter_mm: 5.0,
            line_width_mm: 0.35,
            number_height_mm: 4.0,
        },
    }
}

/// Scale denominator at which the print sizes of [`overprint_sizes`] apply.
pub fn base_scale(kind: MapKind) -> u32 {
    match kind {
        MapKind::Sprint => 4000,
        MapKind::Schoolyard => 1000,
        MapKind::Forest => 15000,
    }
}

/// Axis-aligned rectangle in map mm.
#[derive(Debug, Clone, Copy)]
struct Rect {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Rect {
    fn centred(cx: f64, cy: f64, w: f64, h: f64) -> Self {
        Self {
            min_x: cx - w / 2.0,
            min_y: cy - h / 2.0,
            max_x: cx + w / 2.0,
            max_y: cy + h / 2.0,
        }
    }

    fn overlaps(&self, other: &Rect) -> bool {
        self.min_x < other.max_x - EPS
            && other.min_x < self.max_x - EPS
            && self.min_y < other.max_y - EPS
            && other.min_y < self.max_y - EPS
    }

    fn contains(&self, x: f64, y: f64) -> bool {
        self.min_x <= x && x <= self.max_x && self.min_y <= y && y <= self.max_y
    }

    fn corners(&self) -> [(f64, f64); 4] {
        [
            (self.min_x, self.min_y),
            (self.max_x, self.min_y),
            (self.max_x, self.max_y),
            (self.min_x, self.max_y),
        ]
    }

    fn distance_to_point(&self, x: f64, y: f64) -> f64 {
        let dx = (self.min_x - x).max(0.0).max(x - self.max_x);
        let dy = (self.min_y - y).max(0.0).max(y - self.max_y);
        dx.hypot(dy)
    }

    /// Smallest distance between the rectangle and the segment, 0 when they intersect.
    fn distance_to_segment(&self, a: (f64, f64), b: (f64, f64)) -> f64 {
        if self.contains(a.0, a.1) || self.contains(b.0, b.1) {
            return 0.0;
        }
        let c = self.corners();
        for i in 0..4 {
            if segments_cross(a, b, c[i], c[(i + 1) % 4]) {
                return 0.0;
            }
        }
        let mut best = self
            .distance_to_point(a.0, a.1)
            .min(self.distance_to_point(b.0, b.1));
        for corner in c {
            best = best.min(point_segment_distance(corner, a, b));
        }
        best
    }
}

fn orient(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

/// True when the segments cross properly; touching cases are caught by the distance tests.
fn segments_cross(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    orient(a, b, c) * orient(a, b, d) < 0.0 && orient(c, d, a) * orient(c, d, b) < 0.0
}

fn point_segment_distance(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 < EPS {
        0.0
    } else {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    };
    (p.0 - (a.0 + t * dx)).hypot(p.1 - (a.1 + t * dy))
}

fn digits(n: u32) -> usize {
    n.to_string().len()
}

/// The rectangle a number occupies on the map. The centre lies along `direction` at the distance
/// circle radius + gap + half the extent of the box in that direction.
fn number_rect(
    centre: &MapPoint,
    direction: f64,
    number: u32,
    sizes: &OverprintSizes,
    factor: f64,
) -> Rect {
    let h = sizes.number_height_mm * factor;
    let w = DIGIT_WIDTH_FACTOR * h * digits(number) as f64;
    let rad = direction.to_radians();
    // y is down: 90 degrees (north) is -y.
    let (dx, dy) = (rad.cos(), -rad.sin());
    let extent = rad.cos().abs() * w / 2.0 + rad.sin().abs() * h / 2.0;
    let dist = sizes.circle_diameter_mm * factor / 2.0 + GAP_PRINT_MM * factor + extent;
    Rect::centred(centre.x + dx * dist, centre.y + dy * dist, w, h)
}

/// Direction in degrees [0, 360) from `from` to `to`, 0 = east, counter-clockwise, y down.
fn direction_to(from: &MapPoint, to: &MapPoint) -> Option<f64> {
    let (dx, dy) = (to.x - from.x, from.y - to.y);
    (dx.hypot(dy) > EPS).then(|| dy.atan2(dx).to_degrees().rem_euclid(360.0))
}

fn angle_diff(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// Bisector of the widest gap between the legs seen from the control.
fn preferred_direction(incoming: Option<f64>, outgoing: Option<f64>) -> f64 {
    match (incoming, outgoing) {
        (Some(a), Some(b)) => {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            let inner = hi - lo;
            if inner >= 360.0 - inner {
                (lo + inner / 2.0).rem_euclid(360.0)
            } else {
                (hi + (360.0 - inner) / 2.0).rem_euclid(360.0)
            }
        }
        (Some(a), None) | (None, Some(a)) => (a + 180.0).rem_euclid(360.0),
        (None, None) => 45.0,
    }
}

/// Removes the first `dist` mm of the polyline.
fn trim_start(points: &[MapPoint], dist: f64) -> Vec<MapPoint> {
    let mut left = dist;
    for (i, w) in points.windows(2).enumerate() {
        let len = (w[1].x - w[0].x).hypot(w[1].y - w[0].y);
        if len > left {
            let t = left / len;
            let mut out = vec![MapPoint {
                x: w[0].x + (w[1].x - w[0].x) * t,
                y: w[0].y + (w[1].y - w[0].y) * t,
            }];
            out.extend(points[i + 1..].iter().cloned());
            return out;
        }
        left -= len;
    }
    Vec::new()
}

/// Places the number of every numbered control of an individual course (crossing points get none),
/// in running order, as one placement per numbered control.
///
/// Number box: `0.6 * height * digits` wide and `height` high. Of 16 directions (every 22.5
/// degrees from 45) the one with the fewest hard conflicts wins (box on another circle, on a leg
/// widened by half the line width, or on a number placed before), then the one nearest to the
/// bisector of the widest gap between the legs at the control, then the lowest direction.
///
/// # Errors
/// As for [`course_legs`].
pub fn auto_place(
    event: &Event,
    course: &freemapper_courseplan_model::Course,
    map_scale: u32,
    kind: MapKind,
) -> Result<Vec<NumberPlacement>, CourseError> {
    let legs = course_legs(event, course)?;
    let CourseKind::Individual { controls } = &course.kind else {
        return Ok(Vec::new());
    };
    let sizes = overprint_sizes(kind);
    let factor = f64::from(base_scale(kind)) / f64::from(map_scale);
    let radius = sizes.circle_diameter_mm * factor / 2.0;
    let half_line = sizes.line_width_mm * factor / 2.0;
    let numbers = control_numbers(event, course);

    // Position of each course control; a control the event lacks cannot occur after course_legs.
    let positions: Vec<MapPoint> = controls
        .iter()
        .map(|cc| {
            event
                .controls
                .iter()
                .find(|c| c.code == cc.code)
                .map(|c| c.map_mm.clone())
                .unwrap_or(MapPoint { x: 0.0, y: 0.0 })
        })
        .collect();

    let mut placed: Vec<Rect> = Vec::new();
    let mut out = Vec::new();
    for (k, number) in numbers.iter().enumerate() {
        let Some(number) = *number else { continue };
        let here = &positions[k];
        // Leg k arrives at this control, leg k + 1 leaves it.
        let incoming = &legs[k];
        let outgoing = &legs[k + 1];
        let in_dir = incoming
            .points
            .len()
            .checked_sub(2)
            .and_then(|i| direction_to(here, &incoming.points[i]));
        let out_dir = outgoing.points.get(1).and_then(|p| direction_to(here, p));
        let preferred = preferred_direction(in_dir, out_dir);

        // Legs as polylines; the parts inside the own circle always touch it and are ignored.
        let mut lines: Vec<Vec<MapPoint>> = Vec::new();
        for (i, leg) in legs.iter().enumerate() {
            if i == k {
                let reversed: Vec<MapPoint> = leg.points.iter().rev().cloned().collect();
                lines.push(trim_start(&reversed, radius));
            } else if i == k + 1 {
                lines.push(trim_start(&leg.points, radius));
            } else {
                lines.push(leg.points.clone());
            }
        }

        let mut best: Option<(usize, f64, f64)> = None;
        for step in 0..CANDIDATE_COUNT {
            let direction = (CANDIDATE_START + CANDIDATE_STEP * f64::from(step)).rem_euclid(360.0);
            let rect = number_rect(here, direction, number, &sizes, factor);
            let mut conflicts = 0;
            for (j, pos) in positions.iter().enumerate() {
                let is_circle = numbers[j].is_some();
                let same_place = (pos.x - here.x).hypot(pos.y - here.y) < EPS;
                if is_circle
                    && j != k
                    && !same_place
                    && rect.distance_to_point(pos.x, pos.y) < radius + half_line
                {
                    conflicts += 1;
                }
            }
            for line in &lines {
                if line.windows(2).any(|w| {
                    rect.distance_to_segment((w[0].x, w[0].y), (w[1].x, w[1].y)) <= half_line
                }) {
                    conflicts += 1;
                }
            }
            conflicts += placed.iter().filter(|r| rect.overlaps(r)).count();

            let off = angle_diff(direction, preferred);
            let better = match best {
                None => true,
                Some((c, o, _)) => conflicts < c || (conflicts == c && off < o - EPS),
            };
            if better {
                best = Some((conflicts, off, direction));
            }
        }
        let direction = best.map_or(CANDIDATE_START, |(_, _, d)| d);
        placed.push(number_rect(here, direction, number, &sizes, factor));
        out.push(NumberPlacement {
            direction,
            gap_mm: GAP_PRINT_MM,
        });
    }
    Ok(out)
}

/// Writes [`auto_place`] results into the controls of the named individual course that have no
/// number, or into all of them with `overwrite`. Returns how many numbers it set. All or nothing,
/// like every edit.
///
/// # Errors
/// [`EditError::NotFound`] for an unknown course, [`EditError::NotSupported`] for other kinds,
/// [`EditError::Invalid`] when the result is inconsistent.
pub fn place_numbers(
    event: &mut Event,
    course_name: &str,
    map_scale: u32,
    kind: MapKind,
    overwrite: bool,
) -> Result<usize, EditError> {
    apply(event, |e| {
        let placements = {
            let course = e
                .courses
                .iter()
                .find(|c| c.name == course_name)
                .ok_or_else(|| EditError::NotFound {
                    what: "course",
                    id: course_name.to_owned(),
                })?;
            auto_place(e, course, map_scale, kind).map_err(|err| match err {
                CourseError::NotSupported { course, .. } => EditError::NotSupported { course },
                CourseError::UnknownCourse(id) => EditError::NotFound { what: "course", id },
                CourseError::UnknownReference { id, .. } => EditError::NotFound {
                    what: "control or special object",
                    id,
                },
            })?
        };
        let numbered: Vec<bool> = {
            let course = e.courses.iter().find(|c| c.name == course_name);
            course
                .map(|c| control_numbers(e, c).iter().map(Option::is_some).collect())
                .unwrap_or_default()
        };
        let course = course_mut(e, course_name)?;
        let mut next = placements.into_iter();
        let mut set = 0;
        for (cc, numbered) in individual_mut(course)?.iter_mut().zip(numbered) {
            if !numbered {
                continue;
            }
            let Some(placement) = next.next() else { break };
            if overwrite || cc.number.is_none() {
                cc.number = Some(placement);
                set += 1;
            }
        }
        Ok(set)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_courseplan_model::{
        Control, ControlCode, ControlDescription, ControlKind, Course, CourseControl, EventMeta,
        FORMAT_VERSION, Sources, SpecialObject,
    };

    fn p(x: f64, y: f64) -> MapPoint {
        MapPoint { x, y }
    }

    fn ctrl(code: &str, x: f64, y: f64, kind: ControlKind) -> Control {
        Control {
            code: ControlCode(code.to_owned()),
            kind,
            map_mm: p(x, y),
            description: ControlDescription::default(),
            number: None,
        }
    }

    fn cc(code: &str) -> CourseControl {
        CourseControl {
            code: ControlCode(code.to_owned()),
            number: None,
            bends: vec![],
            taped: false,
        }
    }

    /// Event with one course A from `start` over `stops` (code, x, y, kind) to `finish`.
    fn event(
        start: (f64, f64),
        stops: &[(&str, f64, f64, ControlKind)],
        finish: (f64, f64),
    ) -> Event {
        Event {
            format: FORMAT_VERSION,
            event: EventMeta {
                title: "T".to_owned(),
                date: None,
                discipline: Default::default(),
                rule_profile: "wkb-2026".to_owned(),
                start_window_min: None,
            },
            sources: Sources {
                map: "a.omap".into(),
                ppen: None,
            },
            classes: vec![],
            controls: stops
                .iter()
                .map(|(c, x, y, k)| ctrl(c, *x, *y, *k))
                .collect(),
            specials: vec![
                SpecialObject::Start {
                    id: "S1".to_owned(),
                    map_mm: p(start.0, start.1),
                },
                SpecialObject::Finish {
                    id: "F1".to_owned(),
                    map_mm: p(finish.0, finish.1),
                },
            ],
            courses: vec![Course {
                name: "A".to_owned(),
                classes: vec![],
                start: "S1".to_owned(),
                finish: "F1".to_owned(),
                start_bends: vec![],
                start_taped: false,
                print_area: None,
                climb_m: None,
                planned_print_scale: None,
                kind: CourseKind::Individual {
                    controls: stops.iter().map(|(c, ..)| cc(c)).collect(),
                },
            }],
            accepted_findings: vec![],
        }
    }

    fn n(code: &str, x: f64, y: f64) -> (&str, f64, f64, ControlKind) {
        (code, x, y, ControlKind::Control)
    }

    #[test]
    fn sizes_follow_the_map_kind() {
        let sprint = overprint_sizes(MapKind::Sprint);
        assert_eq!(
            (
                sprint.circle_diameter_mm,
                sprint.line_width_mm,
                sprint.number_height_mm
            ),
            (6.0, 0.35, 4.0)
        );
        assert_eq!(base_scale(MapKind::Sprint), 4000);
        assert_eq!(overprint_sizes(MapKind::Schoolyard), sprint);
        assert_eq!(base_scale(MapKind::Schoolyard), 1000);
        let forest = overprint_sizes(MapKind::Forest);
        assert_eq!(
            (
                forest.circle_diameter_mm,
                forest.line_width_mm,
                forest.number_height_mm
            ),
            (5.0, 0.35, 4.0)
        );
        assert_eq!(base_scale(MapKind::Forest), 15000);
    }

    #[test]
    fn number_avoids_both_legs() {
        // Map y is down: the start lies west, the finish south of the control. Direction 90 is
        // -y (north), so the free quadrant is north-east, between 0 and 90 degrees.
        let e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (100.0, 130.0));
        let placed = auto_place(&e, &e.courses[0], 4000, MapKind::Sprint).unwrap();
        assert_eq!(placed.len(), 1);
        assert!(
            placed[0].direction > 0.0 && placed[0].direction < 90.0,
            "direction {}",
            placed[0].direction
        );
        assert_eq!(placed[0].gap_mm, 0.5);
    }

    #[test]
    fn number_avoids_a_neighbour_circle() {
        // The second control lies 8 mm away at 45 degrees (north-east; -y is north).
        let d = 8.0 / 2f64.sqrt();
        let e = event(
            (70.0, 100.0),
            &[n("31", 100.0, 100.0), n("32", 100.0 + d, 100.0 - d)],
            (100.0, 140.0),
        );
        let placed = auto_place(&e, &e.courses[0], 4000, MapKind::Sprint).unwrap();
        assert_eq!(placed.len(), 2);
        assert!(
            angle_diff(placed[0].direction, 45.0) > 30.0,
            "direction {}",
            placed[0].direction
        );
    }

    #[test]
    fn numbers_do_not_overlap_each_other() {
        let e = event(
            (70.0, 100.0),
            &[n("31", 100.0, 100.0), n("32", 107.0, 100.0)],
            (140.0, 100.0),
        );
        let placed = auto_place(&e, &e.courses[0], 4000, MapKind::Sprint).unwrap();
        let sizes = overprint_sizes(MapKind::Sprint);
        let a = number_rect(&e.controls[0].map_mm, placed[0].direction, 1, &sizes, 1.0);
        let b = number_rect(&e.controls[1].map_mm, placed[1].direction, 2, &sizes, 1.0);
        assert!(!a.overlaps(&b), "{a:?} overlaps {b:?}");
    }

    #[test]
    fn crossing_points_get_no_number() {
        let e = event(
            (70.0, 100.0),
            &[
                n("31", 100.0, 100.0),
                ("32", 130.0, 100.0, ControlKind::CrossingPoint),
                n("33", 160.0, 100.0),
            ],
            (190.0, 100.0),
        );
        let placed = auto_place(&e, &e.courses[0], 4000, MapKind::Sprint).unwrap();
        assert_eq!(placed.len(), 2);
    }

    #[test]
    fn number_size_scales_with_the_map_scale() {
        let sizes = overprint_sizes(MapKind::Sprint);
        let at_base = number_rect(&p(0.0, 0.0), 0.0, 1, &sizes, 1.0);
        let at_double = number_rect(&p(0.0, 0.0), 0.0, 1, &sizes, 2.0);
        assert!(
            ((at_double.max_y - at_double.min_y) - 2.0 * (at_base.max_y - at_base.min_y)).abs()
                < 1e-9
        );
        // East: centre at radius 3 + gap 0.5 + half width 1.2.
        assert!(((at_base.min_x) - 3.5).abs() < 1e-9);
    }

    #[test]
    fn place_numbers_keeps_existing_unless_overwrite() {
        let mut e = event(
            (70.0, 100.0),
            &[n("31", 100.0, 100.0), n("32", 130.0, 100.0)],
            (160.0, 100.0),
        );
        let old = NumberPlacement {
            direction: 10.0,
            gap_mm: 1.0,
        };
        if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
            controls[0].number = Some(old.clone());
        }
        let set = place_numbers(&mut e, "A", 4000, MapKind::Sprint, false).unwrap();
        assert_eq!(set, 1);
        let CourseKind::Individual { controls } = &e.courses[0].kind else {
            unreachable!()
        };
        assert_eq!(controls[0].number, Some(old));
        assert!(controls[1].number.is_some());

        let set = place_numbers(&mut e, "A", 4000, MapKind::Sprint, true).unwrap();
        assert_eq!(set, 2);
        let CourseKind::Individual { controls } = &e.courses[0].kind else {
            unreachable!()
        };
        assert_eq!(controls[0].number.as_ref().unwrap().gap_mm, 0.5);

        assert!(matches!(
            place_numbers(&mut e, "Z", 4000, MapKind::Sprint, false),
            Err(EditError::NotFound { .. })
        ));
    }
}
