//! The print preview: the course as `freemapper-render` draws it (ADR 0057, ADR 0046 D12/D13).
//!
//! The editor's own overlay (`overlay.rs`) is what the page edits; this module hands the same course to the
//! renderer's `overprint` and lets it paint the purple layers into the map image, read-only. Sizes and the
//! course factor are decided here, once: the renderer decides no scale policy (core ADR 0024).

use freemapper_course::{
    Control, ControlKind as FitKind, Course as FitCourse, CourseFilter,
    CourseKind as FitCourseKind, CourseSource, FittedEvent, Leg, SourceFormat, Special, Stop,
};
use freemapper_courseplan::course::{CourseError, course_legs};
use freemapper_courseplan::map::{MapIndex, MapKind};
use freemapper_courseplan::numbers::GapRange;
pub(super) use freemapper_courseplan::numbers::course_factor;
use freemapper_courseplan::omap::MapCoord;
use freemapper_courseplan_model::{
    ControlKind, Course, CourseKind, Event, MapPoint, SpecialObject,
};
use freemapper_printrules::scale::{MapKind as PrintKind, course_symbols};
use freemapper_render::overprint::{CourseSymbolSizes, overprint};
use freemapper_render::page::Overprint;

use super::map_view::Affine;
use super::overlay::overlay;

/// The sizes of the course symbols in print mm at the base scale of `kind`: the `iof-print` values of
/// freemapper-printrules (ISSprOM 2019-2 section 4.7 for sprint and schoolyard maps, ISOM 2017-2 section 3.7 for
/// forest maps), the ones freemapper-print draws with. The checks and the overlay take circle, line and number
/// height from `numbers::overprint_sizes`; a test keeps the two equal.
pub(super) fn course_symbol_sizes(kind: MapKind) -> CourseSymbolSizes {
    let kind = match kind {
        MapKind::Sprint => PrintKind::Sprint,
        MapKind::Schoolyard => PrintKind::Schoolyard,
        MapKind::Forest => PrintKind::Forest,
    };
    course_symbols(kind, kind.base_scale()).0
}

fn coord(p: &MapPoint) -> MapCoord {
    let mm = |v: f64| (v * 1000.0).round() as i32;
    MapCoord::new(mm(p.x), mm(p.y))
}

fn coords(list: &[MapPoint]) -> Vec<MapCoord> {
    list.iter().map(coord).collect()
}

fn filter(courses: &[String]) -> CourseFilter {
    if courses.is_empty() {
        CourseFilter::All
    } else {
        CourseFilter::Only(courses.to_vec())
    }
}

/// The code the start and the finish of a course get as controls of the fitted event. It starts with a control
/// character, which no control code has, so it cannot clash with one.
fn terminal_code(role: &str, id: &str) -> String {
    format!("\u{1}{role}:{id}")
}

fn special(special: &SpecialObject) -> Option<Special<MapCoord>> {
    let id = None;
    Some(match special {
        SpecialObject::Start { .. } | SpecialObject::Finish { .. } => return None,
        SpecialObject::MarkedRoute {
            points, courses, ..
        } => Special::MarkedRoute {
            id,
            path: coords(points),
            courses: filter(courses),
        },
        SpecialObject::OutOfBoundsArea {
            polygon, courses, ..
        } => Special::OutOfBounds {
            id,
            rings: vec![coords(polygon)],
            courses: filter(courses),
        },
        SpecialObject::ForbiddenRoute {
            points, courses, ..
        } => Special::ForbiddenRoute {
            id,
            path: coords(points),
            courses: filter(courses),
        },
        SpecialObject::DangerousArea {
            polygon, courses, ..
        } => Special::Dangerous {
            id,
            rings: vec![coords(polygon)],
            courses: filter(courses),
        },
        SpecialObject::FirstAid {
            map_mm, courses, ..
        } => Special::FirstAid {
            id,
            pt: coord(map_mm),
            courses: filter(courses),
        },
        SpecialObject::Water {
            map_mm, courses, ..
        } => Special::Water {
            id,
            pt: coord(map_mm),
            courses: filter(courses),
        },
    })
}

/// The event as the renderer takes it: the controls, the start and finish of `course` as controls of their own,
/// the course with its legs, and the special objects. Only individual courses; the caller has checked that with
/// `course_legs`.
fn fitted(
    event: &Event,
    course: &Course,
    legs: &[freemapper_courseplan::course::LegGeometry],
) -> FittedEvent {
    let CourseKind::Individual { controls: visits } = &course.kind else {
        unreachable!("course_legs refuses other kinds");
    };
    let at = |want_start: bool| {
        event.specials.iter().find_map(|s| match s {
            SpecialObject::Start { id, map_mm, .. } if want_start && *id == course.start => {
                Some(coord(map_mm))
            }
            SpecialObject::Finish { id, map_mm, .. } if !want_start && *id == course.finish => {
                Some(coord(map_mm))
            }
            _ => None,
        })
    };
    let (start, finish) = (
        terminal_code("start", &course.start),
        terminal_code("finish", &course.finish),
    );
    let mut controls: Vec<Control<MapCoord>> = event
        .controls
        .iter()
        .map(|c| Control {
            id: None,
            code: c.code.0.clone(),
            kind: match c.kind {
                ControlKind::Control => FitKind::Control,
                ControlKind::CrossingPoint => FitKind::CrossingPoint,
            },
            position: Some(coord(&c.map_mm)),
            geo: None,
            description: None,
        })
        .collect();
    for (code, kind, position) in [
        (&start, FitKind::Start, at(true)),
        (&finish, FitKind::Finish, at(false)),
    ] {
        controls.push(Control {
            id: None,
            code: code.clone(),
            kind,
            position,
            geo: None,
            description: None,
        });
    }
    let stop = |code: &str| Stop {
        control: code.to_owned(),
        kind: None,
        number: None,
        number_text_at: None,
        circle_gaps: Vec::new(),
        map_flip: false,
    };
    let stops = std::iter::once(stop(&start))
        .chain(visits.iter().map(|v| stop(&v.code.0)))
        .chain(std::iter::once(stop(&finish)))
        .collect();
    let fit_legs = legs
        .iter()
        .map(|leg| Leg {
            bends: coords(&leg.points[1..leg.points.len().saturating_sub(1).max(1)]),
            gaps: Vec::new(),
            marked_route: leg.taped,
        })
        .collect();
    let fit_course = FitCourse {
        id: None,
        name: course.name.clone(),
        family: None,
        kind: FitCourseKind::Normal,
        legs: fit_legs,
        stops,
        climb_m: None,
        length_m: None,
        print_scale_hint: None,
        course_obj_ratio: None,
        secondary_title: None,
    };
    FittedEvent {
        controls,
        courses: vec![fit_course],
        specials: event.specials.iter().filter_map(special).collect(),
        source: CourseSource {
            format: SourceFormat::IofXml,
            event_name: None,
            map_scale: None,
            item_scaling: None,
        },
        notes: Vec::new(),
    }
}

/// The renderer's overprint of the named course in page mm, as `RenderedPage::overprint` takes it after
/// `overprint::on_map`. The number centres are those of the overlay, which come from the same placement code as
/// the checks.
///
/// # Errors
/// [`CourseError`] for an unknown course, a course kind that is not an individual course, and a course that
/// names a start, finish or control the event lacks.
pub(super) fn course_overprint(
    event: &Event,
    name: &str,
    index: &MapIndex,
    map_scale: u32,
    kind: MapKind,
    gaps: GapRange,
    mapping: Affine,
) -> Result<Overprint, CourseError> {
    let course = event
        .courses
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| CourseError::UnknownCourse(name.to_owned()))?;
    let legs = course_legs(event, course)?;
    let numbers: Vec<(String, (f64, f64))> =
        overlay(event, Some(name), index, map_scale, kind, gaps)?
            .numbers
            .into_iter()
            .map(|n| (n.code, mapping.to_page(n.centre)))
            .collect();
    let fit = fitted(event, course, &legs);
    let to_page = |c: MapCoord| {
        mapping.to_page(MapPoint {
            x: f64::from(c.x) / 1000.0,
            y: f64::from(c.y) / 1000.0,
        })
    };
    Ok(overprint(
        &fit.courses[0],
        &fit,
        &numbers,
        to_page,
        &course_symbol_sizes(kind),
        course_factor(kind, map_scale),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::map_view::MapView;
    use crate::editor::overlay::CourseOverlay;
    use crate::editor::render_map::RenderMap;
    use freemapper_courseplan::omap::read::read_map_file;
    use freemapper_courseplan_model::{ControlCode, CourseControl, ProjectFile};
    use freemapper_render::overprint::number_size;
    use freemapper_render::page::Shape;

    const TOLERANCE_MM: f64 = 0.01;

    type Pt = (f64, f64);

    struct Setup {
        event: Event,
        index: MapIndex,
        map: RenderMap,
        scale: u32,
        kind: MapKind,
    }

    fn setup(change: impl FnOnce(&mut Event)) -> Setup {
        let project = format!(
            "{}/../../examples/luebeck-sprint/luebeck-sprint.courseplan.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        let file = ProjectFile::load(std::path::Path::new(&project)).unwrap();
        let doc = read_map_file(&file.map_path()).unwrap();
        let index = MapIndex::from_document(&doc);
        let kind = freemapper_courseplan::map::map_kind_for_file(&doc, &file.map_path());
        let mut event = file.event;
        change(&mut event);
        Setup {
            map: RenderMap::new(&doc, &index).unwrap(),
            scale: doc.georef.scale_denominator,
            event,
            index,
            kind,
        }
    }

    /// The richer event: a crossing point, bends, a marked leg and special objects.
    fn rich(event: &mut Event) {
        let p = |x: f64, y: f64| MapPoint { x, y };
        event.controls.push(freemapper_courseplan_model::Control {
            code: ControlCode("X1".into()),
            kind: ControlKind::CrossingPoint,
            map_mm: p(-40.0, -30.0),
            description: Default::default(),
            number: None,
            ppen_id: None,
        });
        let CourseKind::Individual { controls } = &mut event.courses[0].kind else {
            unreachable!()
        };
        controls.insert(
            1,
            CourseControl {
                code: ControlCode("X1".into()),
                number: None,
                bends: vec![p(-45.0, -20.0)],
                taped: false,
            },
        );
        controls[0].bends = vec![p(-60.0, -20.0), p(-55.0, -30.0)];
        controls[3].taped = true;
        event.courses[0].start_bends = vec![p(-80.0, 0.0)];
        for s in [
            SpecialObject::MarkedRoute {
                id: "M1".into(),
                ppen_id: None,
                points: vec![p(-30.0, 0.0), p(-20.0, 5.0)],
                courses: vec![],
            },
            SpecialObject::OutOfBoundsArea {
                id: "O1".into(),
                ppen_id: None,
                polygon: vec![p(-10.0, 20.0), p(0.0, 20.0), p(0.0, 30.0)],
                courses: vec!["A".into()],
            },
            SpecialObject::Water {
                id: "W1".into(),
                ppen_id: None,
                map_mm: p(-70.0, 20.0),
                courses: vec![],
            },
        ] {
            event.specials.push(s);
        }
    }

    fn dist(a: Pt, b: Pt) -> f64 {
        (a.0 - b.0).hypot(a.1 - b.1)
    }

    fn mean(path: &[Pt]) -> Pt {
        let n = path.len() as f64;
        (
            path.iter().map(|p| p.0).sum::<f64>() / n,
            path.iter().map(|p| p.1).sum::<f64>() / n,
        )
    }

    struct Parts {
        /// Centre and radius of every closed stroke of more than three points.
        circles: Vec<(Pt, f64)>,
        /// Centre and first corner of every closed three-point stroke.
        triangles: Vec<(Pt, Pt)>,
        /// Every open stroke.
        lines: Vec<Vec<Pt>>,
        /// Text and centre of the number boxes.
        numbers: Vec<(String, Pt)>,
    }

    fn parts(o: &Overprint, number_h: f64) -> Parts {
        let mut p = Parts {
            circles: vec![],
            triangles: vec![],
            lines: vec![],
            numbers: vec![],
        };
        for shape in o.lower.iter().chain(&o.upper) {
            match shape {
                Shape::Stroke {
                    path, closed: true, ..
                } if path.len() == 3 => p.triangles.push((mean(path), path[0])),
                Shape::Stroke {
                    path, closed: true, ..
                } => {
                    let c = mean(path);
                    p.circles.push((c, dist(c, path[0])));
                }
                Shape::Stroke {
                    path,
                    closed: false,
                    ..
                } => p.lines.push(path.clone()),
                Shape::Text { origin, text, .. } => {
                    let (w, h) = number_size(text, number_h);
                    p.numbers
                        .push((text.clone(), (origin.0 + w / 2.0, origin.1 - h / 2.0)));
                }
                _ => {}
            }
        }
        p
    }

    fn path_length(path: &[Pt]) -> f64 {
        path.windows(2).map(|w| dist(w[0], w[1])).sum()
    }

    /// The point `along` mm from the start of `path`.
    fn point_at(path: &[Pt], along: f64) -> Pt {
        let mut left = along;
        for w in path.windows(2) {
            let seg = dist(w[0], w[1]);
            if left <= seg {
                let t = if seg == 0.0 { 0.0 } else { left / seg };
                return (
                    w[0].0 + t * (w[1].0 - w[0].0),
                    w[0].1 + t * (w[1].1 - w[0].1),
                );
            }
            left -= seg;
        }
        *path.last().unwrap()
    }

    fn on_path(p: Pt, path: &[Pt]) -> bool {
        path.windows(2).any(|w| {
            let (d, f) = (
                (w[1].0 - w[0].0, w[1].1 - w[0].1),
                (p.0 - w[0].0, p.1 - w[0].1),
            );
            let len2 = d.0 * d.0 + d.1 * d.1;
            let t = if len2 == 0.0 {
                0.0
            } else {
                ((f.0 * d.0 + f.1 * d.1) / len2).clamp(0.0, 1.0)
            };
            dist(p, (w[0].0 + t * d.0, w[0].1 + t * d.1)) <= TOLERANCE_MM
        })
    }

    fn near(a: Pt, b: Pt) -> bool {
        dist(a, b) <= TOLERANCE_MM
    }

    fn compare(s: &Setup, name: &str, mismatches: &mut Vec<String>) -> CourseOverlay {
        let map = s.map.mapping();
        let page = |p: &MapPoint| map.to_page(p.clone());
        let course = s.event.courses.iter().find(|c| c.name == name).unwrap();
        let overlay = overlay(
            &s.event,
            Some(name),
            &s.index,
            s.scale,
            s.kind,
            GapRange::DEFAULT,
        )
        .unwrap();
        let o = course_overprint(
            &s.event,
            name,
            &s.index,
            s.scale,
            s.kind,
            GapRange::DEFAULT,
            map,
        )
        .unwrap();
        let got = parts(&o, overlay.sizes.number_h);
        assert!(
            !overlay.controls.is_empty() && !overlay.legs.is_empty() && !overlay.numbers.is_empty(),
            "the course {name} has nothing to compare"
        );
        let mut miss = |what: String| mismatches.push(format!("{name}: {what}"));

        // Control circles: centre and radius.
        for c in overlay.controls.iter().filter(|c| c.kind == "control") {
            let at = page(&c.at);
            match got.circles.iter().find(|(m, _)| near(*m, at)) {
                None => miss(format!("control {}: no circle at {at:?}", c.code)),
                Some((m, r)) => {
                    if (r - overlay.sizes.circle_r).abs() > TOLERANCE_MM {
                        miss(format!(
                            "control {}: circle radius {r:.3}, overlay {:.3} (centre {m:?})",
                            c.code, overlay.sizes.circle_r
                        ));
                    }
                }
            }
        }
        // Start: centre, circumradius, direction.
        for st in &overlay.starts {
            let at = page(&st.at);
            match got.triangles.iter().find(|(m, _)| near(*m, at)) {
                None => miss(format!("start {}: no triangle at {at:?}", st.id)),
                Some((m, corner)) => {
                    let r = dist(*m, *corner);
                    if (r - overlay.sizes.start_r).abs() > TOLERANCE_MM {
                        miss(format!(
                            "start {}: circumradius {r:.3}, overlay {:.3}",
                            st.id, overlay.sizes.start_r
                        ));
                    }
                    let angle = (-(corner.1 - m.1)).atan2(corner.0 - m.0).to_degrees();
                    if (angle - st.angle_deg).abs() > TOLERANCE_MM {
                        miss(format!(
                            "start {}: direction {angle:.3} deg, overlay {:.3}",
                            st.id, st.angle_deg
                        ));
                    }
                }
            }
        }
        // Finish: two circles at the centre.
        for f in &overlay.finishes {
            let at = page(&f.at);
            let mut radii: Vec<f64> = got
                .circles
                .iter()
                .filter(|(m, _)| near(*m, at))
                .map(|c| c.1)
                .collect();
            radii.sort_by(f64::total_cmp);
            let want = [overlay.sizes.finish_inner_r, overlay.sizes.finish_outer_r];
            if radii.len() != 2
                || radii
                    .iter()
                    .zip(want)
                    .any(|(r, w)| (r - w).abs() > TOLERANCE_MM)
            {
                miss(format!(
                    "finish {}: radii {radii:?}, overlay {want:?}",
                    f.id
                ));
            }
        }
        // Legs: the drawn part. An unmarked leg is one open stroke whose points are the drawn part's; a marked leg is
        // dashes, the first of each period starting where the overlay's dashes start. Every leg is compared.
        for leg in &overlay.legs {
            let want: Vec<Pt> = leg.drawn.iter().map(page).collect();
            if want.len() < 2 {
                miss(format!(
                    "leg after {}: the overlay draws {want:?}",
                    leg.after
                ));
                continue;
            }
            if leg.taped {
                let (dash, gap) = (overlay.sizes.dash, overlay.sizes.dash_gap);
                let total = path_length(&want);
                let mut expected = 0;
                let mut start = 0.0;
                while start < total - 1e-9 {
                    expected += 1;
                    let at = point_at(&want, start);
                    let length = dash.min(total - start);
                    let found = got
                        .lines
                        .iter()
                        .any(|l| near(l[0], at) && (path_length(l) - length).abs() <= TOLERANCE_MM);
                    if !found {
                        miss(format!(
                            "taped leg after {}: no dash of {length:.3} at {at:?}",
                            leg.after
                        ));
                    }
                    start += dash + gap;
                }
                let dashes = got
                    .lines
                    .iter()
                    .filter(|l| path_length(l) <= dash + TOLERANCE_MM && on_path(l[0], &want))
                    .count();
                if dashes != expected {
                    miss(format!(
                        "taped leg after {}: {dashes} dashes, overlay {expected}",
                        leg.after
                    ));
                }
                continue;
            }
            let found = got
                .lines
                .iter()
                .find(|l| l.len() == want.len() && l.iter().zip(&want).all(|(a, b)| near(*a, *b)));
            if found.is_none() {
                let near_start = got.lines.iter().find(|l| near(l[0], want[0]));
                miss(format!(
                    "leg after {}: drawn part {want:?}, renderer {near_start:?}",
                    leg.after
                ));
            }
        }
        // Crossing points: the way through, the length of the symbol.
        for c in overlay
            .controls
            .iter()
            .filter(|c| c.kind == "crossing_point")
        {
            let at = page(&c.at);
            let strokes: Vec<&Vec<Pt>> = got
                .lines
                .iter()
                .filter(|l| l.len() > 3 && dist(mean(l), at) <= overlay.sizes.crossing / 2.0)
                .collect();
            if strokes.len() != 2 {
                miss(format!(
                    "crossing point {}: {} strokes",
                    c.code,
                    strokes.len()
                ));
                continue;
            }
            let angle = c.angle_deg.unwrap_or(f64::NAN);
            for l in strokes {
                let (first, last) = (l[0], l[l.len() - 1]);
                let chord = dist(first, last);
                let turned = (last.1 - first.1).atan2(last.0 - first.0).to_degrees();
                let off = ((turned + angle + 180.0).rem_euclid(360.0) - 180.0).abs();
                if (chord - overlay.sizes.crossing).abs() > TOLERANCE_MM || off > 0.1 {
                    miss(format!(
                        "crossing point {}: length {chord:.3} turned {turned:.2}, overlay {:.3} turned {:.2}",
                        c.code, overlay.sizes.crossing, -angle
                    ));
                }
            }
        }
        // Numbers: the centre and the size of the box. The renderer's size is the one the placement uses.
        for n in &overlay.numbers {
            let at = page(&MapPoint {
                x: n.centre.x,
                y: n.centre.y,
            });
            let found = got
                .numbers
                .iter()
                .any(|(t, c)| *t == n.text && near(*c, at));
            if !found {
                miss(format!(
                    "number {} ({}): no box centred at {at:?}",
                    n.index, n.text
                ));
            }
            let (w, h) = number_size(&n.text, overlay.sizes.number_h);
            if (w - n.w).abs() > TOLERANCE_MM || (h - n.h).abs() > TOLERANCE_MM {
                miss(format!(
                    "number {} ({}): overlay box {:.3} x {:.3}, renderer {w:.3} x {h:.3}",
                    n.index, n.text, n.w, n.h
                ));
            }
        }
        let _ = course;
        overlay
    }

    /// The mismatches of every course of the example, as a map of `kind` at 1:`scale`. The map file is the
    /// sprint map of the example; only the kind and the scale the symbols are sized for change, which is what
    /// a forest or schoolyard project differs by in the overlay and the print layer.
    fn all_mismatches(change: impl FnOnce(&mut Event), kind: MapKind, scale: u32) -> Vec<String> {
        let mut s = setup(change);
        s.kind = kind;
        s.scale = scale;
        let mut out = Vec::new();
        let names: Vec<String> = s.event.courses.iter().map(|c| c.name.clone()).collect();
        for name in names {
            compare(&s, &name, &mut out);
        }
        out
    }

    const CASES: [(MapKind, u32); 5] = [
        (MapKind::Sprint, 4000),
        (MapKind::Sprint, 3000),
        (MapKind::Sprint, 5000),
        (MapKind::Schoolyard, 1000),
        (MapKind::Forest, 10000),
    ];

    #[test]
    fn overlay_matches_renderer_overprint() {
        for (kind, scale) in CASES {
            let out = all_mismatches(|_| {}, kind, scale);
            assert!(out.is_empty(), "{kind:?} 1:{scale}\n{}", out.join("\n"));
        }
    }

    /// Sprint at 1:5000 is no standard row: the sheet draws circles of 6.0 mm (diameter) there, so the overlay and
    /// the renderer both have a radius of 3.0 map mm, not 3.0 * 4000 / 5000 (ADR 0058).
    #[test]
    fn a_non_standard_scale_keeps_the_circles_at_the_sheets_6_mm() {
        let mut s = setup(|_| {});
        s.kind = MapKind::Sprint;
        s.scale = 5000;
        let (sheet, factor) = course_symbols(PrintKind::Sprint, 5000);
        assert_eq!((sheet.circle_diameter_mm, factor), (6.0, 1.0));
        let mut out = Vec::new();
        let name = s.event.courses[0].name.clone();
        let o = compare(&s, &name, &mut out);
        assert!(out.is_empty(), "{}", out.join("\n"));
        assert!(
            (o.sizes.circle_r - 3.0).abs() < 1e-9,
            "{}",
            o.sizes.circle_r
        );
        let drawn = course_overprint(
            &s.event,
            &name,
            &s.index,
            5000,
            s.kind,
            GapRange::DEFAULT,
            s.map.mapping(),
        )
        .unwrap();
        let radii = parts(&drawn, o.sizes.number_h).circles;
        assert!(
            radii.iter().any(|(_, r)| (r - 3.0).abs() < 1e-3),
            "{radii:?}"
        );
        // 1:3000 is a standard row and grows by 4/3.
        s.scale = 3000;
        let mut out = Vec::new();
        let o = compare(&s, &name, &mut out);
        assert!(
            (o.sizes.circle_r - 4.0).abs() < 1e-9,
            "{}",
            o.sizes.circle_r
        );
    }

    #[test]
    fn overlay_matches_renderer_overprint_with_bends_crossing_points_and_specials() {
        for (kind, scale) in CASES {
            let out = all_mismatches(rich, kind, scale);
            assert!(out.is_empty(), "{kind:?} 1:{scale}\n{}", out.join("\n"));
        }
    }

    /// The test would pass trivially if the rich event lost its marked leg or its crossing point.
    #[test]
    fn the_rich_event_has_what_the_comparison_needs() {
        let s = setup(rich);
        let o = overlay(
            &s.event,
            Some("A"),
            &s.index,
            s.scale,
            s.kind,
            GapRange::DEFAULT,
        )
        .unwrap();
        assert!(o.legs.iter().any(|l| l.taped));
        assert!(o.controls.iter().any(|c| c.kind == "crossing_point"));
        assert!(
            o.controls
                .iter()
                .filter(|c| c.kind == "crossing_point")
                .all(|c| c.angle_deg.is_some())
        );
        assert!(o.legs.iter().all(|l| l.drawn.len() >= 2));
    }

    /// ADR 0057: the overlay cuts a leg at the start triangle's corner and the outer finish circle, the checks
    /// (ADR 0030) cut both ends at the circle radius. The two differ on purpose; this keeps the checks' cut where
    /// it is, so a later alignment of the overlay cannot move findings unnoticed.
    #[test]
    fn the_checks_cut_legs_at_the_circle_radius_the_overlay_does_not() {
        let s = setup(|_| {});
        let o = overlay(
            &s.event,
            Some("A"),
            &s.index,
            s.scale,
            s.kind,
            GapRange::DEFAULT,
        )
        .unwrap();
        let first = &o.legs[0];
        let start = first.points[0].clone();
        let by_checks = freemapper_courseplan::check::drawn_part(&first.points, o.sizes.circle_r);
        let at = |p: &MapPoint| (p.x - start.x).hypot(p.y - start.y);
        assert!((at(&by_checks[0]) - o.sizes.circle_r).abs() < 1e-6);
        assert!((at(&first.drawn[0]) - o.sizes.start_r).abs() < 2e-3);
        assert!(o.sizes.start_r > o.sizes.circle_r);
    }

    #[test]
    fn symbol_sizes_agree_with_the_numbers_module() {
        for kind in [MapKind::Sprint, MapKind::Schoolyard, MapKind::Forest] {
            let (sizes, base) = (
                course_symbol_sizes(kind),
                freemapper_courseplan::numbers::overprint_sizes(kind),
            );
            assert_eq!(sizes.circle_diameter_mm, base.circle_diameter_mm);
            assert_eq!(sizes.line_width_mm, base.line_width_mm);
            assert_eq!(sizes.number_height_mm, base.number_height_mm);
        }
    }

    #[test]
    fn a_map_without_scale_draws_at_base_size() {
        assert_eq!(course_factor(MapKind::Sprint, 0), 1.0);
        assert!((course_factor(MapKind::Sprint, 3000) - 4.0 / 3.0).abs() < 1e-12);
        // A non-standard row keeps the base size (ADR 0058), not 4000 / 5000 or 4000 / 2000.
        assert_eq!(course_factor(MapKind::Sprint, 5000), 1.0);
        assert_eq!(course_factor(MapKind::Sprint, 2000), 1.0);
    }

    #[test]
    fn the_print_layer_has_the_courses_symbols() {
        let s = setup(rich);
        let o = course_overprint(
            &s.event,
            "A",
            &s.index,
            s.scale,
            s.kind,
            GapRange::DEFAULT,
            s.map.mapping(),
        )
        .unwrap();
        assert!(!o.lower.is_empty() && !o.upper.is_empty());
        assert!(matches!(
            course_overprint(
                &s.event,
                "nope",
                &s.index,
                s.scale,
                s.kind,
                GapRange::DEFAULT,
                s.map.mapping()
            ),
            Err(CourseError::UnknownCourse(_))
        ));
    }
}
