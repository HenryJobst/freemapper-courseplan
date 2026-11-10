//! Automatic placement of control numbers.
//!
//! The search itself lives in `freemapper-course` (`placement`, shared with freemapper-print, ADR
//! 0036): a number goes where its box touches the fewest circles, legs and other numbers
//! (overprint), then the fewest point and line symbols of the map (areas and magnetic north lines
//! never count). Among equals it goes behind the circle, in line with the incoming leg, unless the
//! outgoing leg is there; then it takes the nearest free direction in steps of 15 degrees, rounded
//! to 0.1 degrees, and at an almost straight angle it points straight out of the bend (the outer
//! bisector). Before it turns away, a number moves outward within the gap range of the rules (ADR
//! 0032). This module keeps what belongs to the application: the overprint sizes, the gap range
//! from the rule profile, the kept-number levels (ruling R11) and the conversion of map features
//! into obstacles.
//!
//! Conventions: map coordinates are millimetres with y down, so "north" on screen is -y. A
//! direction is in degrees, 0 = east, counter-clockwise, hence 90 degrees points to -y (north).
//! Overprint sizes are print millimetres at the base scale of the map kind and grow to map
//! millimetres by `map_mm = print_mm * base_scale / map_scale`. Terms follow `docs/glossary.md`.

use freemapper_course::placement::{
    self, CourseNumbers, GAP_LIMITS_MM, NumberStop, Obstacle, ObstacleShape, Placement, Rect, Tier,
    number_box, place_course_numbers,
};
use freemapper_courseplan_model::{CourseKind, Discipline, Event, MapPoint, NumberPlacement};

use crate::check::{symbol_matches, value_number};
use crate::course::{CourseError, control_numbers, course_legs};
use crate::edit::{EditError, apply, course_mut, individual_mut};
use crate::map::{FeatureGeometry, MapFeature, MapIndex, MapKind};
use crate::rules::RuleSet;

pub use placement::{GapRange, normalize_direction};

/// The gap range of the active `number_gap` rules that apply to a course. The strictest wins:
/// the largest `min_gap_mm` and the smallest `max_gap_mm`; each rule is read the way the check
/// reads it (`rule_sides`). A value a rule leaves out keeps that side open. Without a rule the
/// range is [`GapRange::DEFAULT`]; with only a max the min is 0.5, with only a min the max equals
/// it (`GapRange::from_limits`, which also rounds the min up and the max down to 0.01 mm).
pub fn gap_range(
    rules: &RuleSet,
    map_kind: MapKind,
    discipline: Discipline,
    print_scale: u32,
) -> GapRange {
    let sides = rules
        .rules
        .iter()
        .filter(|r| {
            r.rule.key == "number_gap"
                && crate::rules::applies(&r.rule, map_kind, discipline, print_scale)
        })
        .map(|r| rule_sides(&r.rule));
    match GapRange::strictest(sides) {
        Some((min, max)) => GapRange::from_limits(min, max),
        None => GapRange::DEFAULT,
    }
}

/// Smallest gap a rule value may have, in print mm (the grid of the stored gaps).
const MIN_RULE_GAP_MM: f64 = GAP_LIMITS_MM.0;

/// A rule gap value made usable: a value that is not finite or not above 0 counts as missing,
/// the others are clamped into [0.01, [`MAX_GAP_MM`]].
fn clean_gap(value: Option<f64>) -> Option<f64> {
    value
        .filter(|v| v.is_finite() && *v > 0.0)
        .map(|v| v.clamp(MIN_RULE_GAP_MM, MAX_GAP_MM))
}

/// The sides of one `number_gap` rule, normalised the way placement and the check both read them:
/// invalid values are dropped, the others clamped, and a max below the min is raised to it.
pub(crate) fn normalise_sides(min: Option<f64>, max: Option<f64>) -> (Option<f64>, Option<f64>) {
    let min = clean_gap(min);
    let max = clean_gap(max);
    match (min, max) {
        (Some(min), Some(max)) => (Some(min), Some(max.max(min))),
        other => other,
    }
}

/// The normalised sides (min, max) of a `number_gap` rule, see [`normalise_sides`].
pub(crate) fn rule_sides(rule: &crate::rules::Rule) -> (Option<f64>, Option<f64>) {
    normalise_sides(
        value_number(rule, "min_gap_mm"),
        value_number(rule, "max_gap_mm"),
    )
}

/// Width of one digit relative to the number height.
const DIGIT_WIDTH_FACTOR: f64 = 0.6;
/// Symbol code of magnetic north lines, which never count as covered (ruling R5).
const NORTH_LINE_SYMBOL: &str = "601";
/// A point symbol is covered when the number box comes within this distance of its position
/// (map mm; map symbols do not scale with the overprint factor).
const POINT_SYMBOL_RADIUS_MM: f64 = 0.5;
/// A line symbol is covered when a segment of it comes within this distance of the number box
/// (map mm).
const LINE_SYMBOL_HALF_WIDTH_MM: f64 = 0.15;

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

/// Largest gap between circle and number that the CLI and the MCP server accept, in print mm.
pub const MAX_GAP_MM: f64 = GAP_LIMITS_MM.1;

/// Whether a gap in print mm is more than 0 and at most [`MAX_GAP_MM`].
pub fn gap_in_range(gap_mm: f64) -> bool {
    gap_mm > 0.0 && gap_mm <= MAX_GAP_MM
}

fn digits(n: u32) -> usize {
    n.to_string().len()
}

/// Radius of the control circle plus half its line width, in the unit of `sizes`.
fn outer_radius(sizes: &OverprintSizes) -> f64 {
    (sizes.circle_diameter_mm + sizes.line_width_mm) / 2.0
}

/// Width and height of the box of control number `number`, in the unit of `sizes`: `0.6 * height`
/// per digit wide and the number height high.
fn box_size(sizes: &OverprintSizes, number: u32) -> (f64, f64) {
    let h = sizes.number_height_mm;
    (DIGIT_WIDTH_FACTOR * h * digits(number) as f64, h)
}

/// Radius of the control circle plus half its line width, in print mm at the base scale of the
/// map kind: the circle the number gap is measured from.
pub fn circle_outer_radius_mm(kind: MapKind) -> f64 {
    outer_radius(&overprint_sizes(kind))
}

/// Width and height of the box of control number `number` in print mm at the base scale of the
/// map kind, as the placement and the check use it.
pub fn number_box_mm(kind: MapKind, number: u32) -> (f64, f64) {
    box_size(&overprint_sizes(kind), number)
}

/// The rectangle a number occupies on the map. The centre lies on the ray in `direction`, and
/// `gap_mm` (print mm) is the shortest distance between the outer edge of the circle line and the
/// rectangle (agreed with freemapper-print, ADR 0030).
#[cfg(test)]
fn number_rect(
    centre: &MapPoint,
    direction: f64,
    number: u32,
    gap_mm: f64,
    sizes: &OverprintSizes,
    factor: f64,
) -> Rect {
    let (w, h) = box_size(sizes, number);
    number_box(
        (centre.x, centre.y),
        outer_radius(sizes) * factor,
        (w * factor, h * factor),
        Placement {
            direction,
            gap_mm: gap_mm * factor,
        },
    )
}

/// Centre of the number box of a control at `at`, in map mm at `map_scale`, for the placement the
/// check and the overprint use (direction, gap, digit count of `number`).
pub(crate) fn number_centre(
    at: &MapPoint,
    placement: &NumberPlacement,
    number: u32,
    kind: MapKind,
    map_scale: u32,
) -> MapPoint {
    let factor = f64::from(base_scale(kind)) / f64::from(map_scale);
    let sizes = overprint_sizes(kind);
    let (w, h) = box_size(&sizes, number);
    let r = number_box(
        (at.x, at.y),
        outer_radius(&sizes) * factor,
        (w * factor, h * factor),
        Placement {
            direction: placement.direction,
            gap_mm: placement.gap_mm * factor,
        },
    );
    let (cx, cy) = r.centre();
    MapPoint { x: cx, y: cy }
}

/// The obstacle a map feature makes, or `None` for areas and magnetic north lines (601 and its
/// sub-codes, ruling R5). A point is a zero-length segment with the half width of a point symbol,
/// a line segment by segment with the half width of a line symbol; both rank as map symbols and
/// carry the feature index as label, so a feature counts once.
fn map_obstacle(index: usize, feature: &MapFeature) -> Vec<Obstacle> {
    if feature
        .symbol_code
        .as_deref()
        .is_some_and(|code| symbol_matches(code, NORTH_LINE_SYMBOL))
    {
        return Vec::new();
    }
    let obstacle = |a: (f64, f64), b: (f64, f64), half_width: f64| Obstacle {
        shape: ObstacleShape::Segment { a, b, half_width },
        weight: 0.0,
        tier: Tier::MapSymbol,
        label: format!("m{index}"),
    };
    match &feature.geometry {
        FeatureGeometry::Point(p) => {
            vec![obstacle((p.x, p.y), (p.x, p.y), POINT_SYMBOL_RADIUS_MM)]
        }
        FeatureGeometry::Line(points) => points
            .windows(2)
            .map(|w| {
                obstacle(
                    (w[0].x, w[0].y),
                    (w[1].x, w[1].y),
                    LINE_SYMBOL_HALF_WIDTH_MM,
                )
            })
            .collect(),
        FeatureGeometry::Area(_) => Vec::new(),
    }
}

/// All obstacles of the map, converted once per placement.
fn map_obstacles(map: &MapIndex) -> Vec<Obstacle> {
    map.features()
        .iter()
        .enumerate()
        .flat_map(|(i, f)| map_obstacle(i, f))
        .collect()
}

/// Places the number of every numbered control of an individual course (crossing points get none),
/// in running order, as one placement per numbered control.
///
/// Number box: `0.6 * height * digits` wide and `height` high. The search is
/// `freemapper_course::placement::place_course_numbers` (ADR 0036, core ADR 0016): the preferred
/// direction is behind the circle, in line with the incoming leg (its last segment, so after the
/// last bend); if the box there touches the outgoing leg, or there is no incoming leg, it is the
/// outer bisector (the middle of the larger area between the legs). Tried are the 24 directions
/// `preferred + 15 * k` degrees, each rounded to 0.1 degrees and returned as evaluated, crossed
/// with the gaps of `gaps` (from the minimum in steps of 0.5 mm up to the maximum) and compared as
/// (overprint conflicts, map conflicts, distance to the preferred direction, gap, distance to the
/// outer bisector, direction), so a larger gap in the preferred direction beats a smaller one in
/// another direction. "Behind is blocked" is tested at the minimum gap. Stored numbers keep their
/// gap even outside the range; only `check` warns. Overprint conflicts are a box on another
/// circle, on a leg widened by half the line width, or on a number placed before; map conflicts
/// are covered point or line symbols, each counting once. Areas and magnetic north lines (601 and
/// its sub-codes) never count. Overprint outweighs any number of map conflicts (rulings R3 to R5).
///
/// Kept numbers follow the levels of design §5.3, the more specific winning: automatic placement,
/// then the central number of the control (`Control.number`), then the number of the course
/// control (`CourseControl.number`) (ruling R11). A control with a central number always keeps the
/// number drawn for it, its course number if it has one, else the central one. With
/// `keep_existing`, a course number is kept as well. Kept boxes are obstacles for the others
/// (placed first), and the kept placement is returned unchanged. Without `keep_existing`, course
/// numbers of controls without a central number are placed afresh and are no obstacles.
///
/// # Errors
/// As for [`course_legs`].
pub fn auto_place(
    event: &Event,
    course: &freemapper_courseplan_model::Course,
    map: &MapIndex,
    map_scale: u32,
    kind: MapKind,
    gaps: GapRange,
    keep_existing: bool,
) -> Result<Vec<NumberPlacement>, CourseError> {
    let legs = course_legs(event, course)?;
    let CourseKind::Individual { controls } = &course.kind else {
        return Ok(Vec::new());
    };
    let sizes = overprint_sizes(kind);
    let factor = f64::from(base_scale(kind)) / f64::from(map_scale);
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

    let central: Vec<Option<NumberPlacement>> = controls
        .iter()
        .map(|cc| {
            event
                .controls
                .iter()
                .find(|c| c.code == cc.code)
                .and_then(|c| c.number.clone())
        })
        .collect();
    let kept = |k: usize| {
        let own = controls[k].number.clone();
        match &central[k] {
            Some(central) => Some(own.unwrap_or_else(|| central.clone())),
            None => own.filter(|_| keep_existing),
        }
    };
    let stops: Vec<NumberStop> = positions
        .iter()
        .zip(&numbers)
        .enumerate()
        .map(|(k, (pos, number))| NumberStop {
            centre: (pos.x, pos.y),
            size: number.map(|n| {
                let (w, h) = box_size(&sizes, n);
                (w * factor, h * factor)
            }),
            kept: number.and_then(|_| {
                kept(k).map(|p| Placement {
                    direction: p.direction,
                    gap_mm: p.gap_mm,
                })
            }),
        })
        .collect();
    let leg_points: Vec<Vec<(f64, f64)>> = legs
        .iter()
        .map(|leg| leg.points.iter().map(|p| (p.x, p.y)).collect())
        .collect();
    let placed = place_course_numbers(
        &CourseNumbers {
            stops: &stops,
            legs: &leg_points,
            circle_r: sizes.circle_diameter_mm * factor / 2.0,
            half_line: sizes.line_width_mm * factor / 2.0,
            unit: factor,
            gaps,
        },
        &map_obstacles(map),
        Rect::UNBOUNDED,
    );
    Ok(placed
        .into_iter()
        .flatten()
        .map(|p| NumberPlacement {
            direction: p.direction,
            gap_mm: p.gap_mm,
        })
        .collect())
}

/// Writes [`auto_place`] results into the course controls of the named individual course that have
/// neither a course number nor a central number. With `overwrite` it also replaces course
/// numbers, except at controls with a central number, and it never copies a central number into
/// the course (ruling R11). Returns how many numbers it set. All or nothing, like every edit.
/// Placement runs only on request; editing a leg does not move numbers.
///
/// # Errors
/// [`EditError::NotFound`] for an unknown course, [`EditError::NotSupported`] for other kinds,
/// [`EditError::Invalid`] when the result is inconsistent.
pub fn place_numbers(
    event: &mut Event,
    course_name: &str,
    map: &MapIndex,
    map_scale: u32,
    kind: MapKind,
    gaps: GapRange,
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
            auto_place(e, course, map, map_scale, kind, gaps, !overwrite).map_err(
                |err| match err {
                    CourseError::NotSupported { course, .. } => EditError::NotSupported { course },
                    CourseError::UnknownCourse(id) => EditError::NotFound { what: "course", id },
                    CourseError::UnknownReference { id, .. } => EditError::NotFound {
                        what: "control or special object",
                        id,
                    },
                },
            )?
        };
        let numbered: Vec<bool> = {
            let course = e.courses.iter().find(|c| c.name == course_name);
            course
                .map(|c| control_numbers(e, c).iter().map(Option::is_some).collect())
                .unwrap_or_default()
        };
        let centrally_numbered: Vec<bool> = {
            let course = e.courses.iter().find(|c| c.name == course_name);
            match course.map(|c| &c.kind) {
                Some(CourseKind::Individual { controls }) => controls
                    .iter()
                    .map(|cc| {
                        e.controls
                            .iter()
                            .any(|c| c.code == cc.code && c.number.is_some())
                    })
                    .collect(),
                _ => Vec::new(),
            }
        };
        let course = course_mut(e, course_name)?;
        let mut next = placements.into_iter();
        let mut set = 0;
        for ((cc, numbered), central) in individual_mut(course)?
            .iter_mut()
            .zip(numbered)
            .zip(centrally_numbered)
        {
            if !numbered {
                continue;
            }
            let Some(placement) = next.next() else { break };
            if !central && (overwrite || cc.number.is_none()) {
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
    use freemapper_course::placement::{angle_diff, candidate};

    // Shims over the core API for the scenarios that predate the switch (ADR 0036).
    fn candidate_direction(preferred: f64, step: i32) -> f64 {
        candidate(preferred, step.rem_euclid(24) as usize)
    }

    fn candidate_gaps(gaps: GapRange) -> Vec<f64> {
        // Rounding and clamping live in `from_limits` now; `steps` only walks the range.
        GapRange::from_limits(Some(gaps.min), Some(gaps.max)).steps()
    }

    fn feature_covered(r: &Rect, f: &MapFeature) -> bool {
        map_obstacle(0, f).iter().any(|o| o.shape.hits(r))
    }
    use crate::check::testing::{
        AREA_520, LINE_502, LINE_601, POINT_113, area_object, line_object, map, point_object, rule,
        with_values,
    };
    use crate::rules::{SetRule, Severity, load_rule_set};
    use freemapper_courseplan_model::{
        Control, ControlCode, ControlDescription, ControlKind, Course, CourseControl, EventMeta,
        FORMAT_VERSION, Sources, SpecialObject,
    };

    fn range_of(set: &RuleSet) -> GapRange {
        gap_range(set, MapKind::Sprint, Discipline::Sprint, 4000)
    }

    fn gap_rule(id: &str, values: &str) -> SetRule {
        SetRule {
            rule: with_values(rule(id, "number_gap", Severity::Warning), values),
            profile: "wkb-2026".to_owned(),
        }
    }

    #[test]
    fn gap_range_reads_the_profile() {
        let set = load_rule_set("wkb-2026", &[]).unwrap();
        assert_eq!(range_of(&set), GapRange { min: 0.5, max: 5.0 });
    }

    #[test]
    fn gap_range_without_rule_is_fixed() {
        let mut set = load_rule_set("wkb-2026", &[]).unwrap();
        set.rules.retain(|r| r.rule.key != "number_gap");
        assert_eq!(range_of(&set), GapRange::DEFAULT);
    }

    #[test]
    fn gap_range_takes_the_strictest_rule() {
        let mut set = load_rule_set("wkb-2026", &[]).unwrap();
        set.rules
            .push(gap_rule("t.gap", "min_gap_mm = 1.0\nmax_gap_mm = 3.0"));
        assert_eq!(range_of(&set), GapRange { min: 1.0, max: 3.0 });
    }

    #[test]
    fn gap_range_with_min_above_max_uses_min() {
        let mut set = load_rule_set("wkb-2026", &[]).unwrap();
        set.rules.retain(|r| r.rule.key != "number_gap");
        set.rules
            .push(gap_rule("t.gap", "min_gap_mm = 6.0\nmax_gap_mm = 5.0"));
        assert_eq!(range_of(&set), GapRange { min: 6.0, max: 6.0 });
    }

    #[test]
    fn gap_range_with_one_value_closes_the_other_side() {
        let mut set = load_rule_set("wkb-2026", &[]).unwrap();
        set.rules.retain(|r| r.rule.key != "number_gap");
        set.rules.push(gap_rule("t.max", "max_gap_mm = 3.0"));
        assert_eq!(range_of(&set), GapRange { min: 0.5, max: 3.0 });
        set.rules.clear();
        set.rules.push(gap_rule("t.min", "min_gap_mm = 2.0"));
        assert_eq!(range_of(&set), GapRange { min: 2.0, max: 2.0 });
    }

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
                rule_overlays: Vec::new(),
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
                    ppen_id: None,
                    map_mm: p(start.0, start.1),
                },
                SpecialObject::Finish {
                    id: "F1".to_owned(),
                    ppen_id: None,
                    map_mm: p(finish.0, finish.1),
                },
            ],
            courses: vec![Course {
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
    fn directions_are_normalised_and_gaps_bounded() {
        for (given, want) in [
            (-45.0, 315.0),
            (720.5, 0.5),
            (360.0, 0.0),
            (-0.0, 0.0),
            (-1e-20, 0.0),
            (90.0, 90.0),
        ] {
            let got = normalize_direction(given);
            assert_eq!(got, want, "{given}");
            assert!(got.is_sign_positive(), "{given}");
        }
        assert!(gap_in_range(0.5) && gap_in_range(MAX_GAP_MM));
        assert!(!gap_in_range(0.0) && !gap_in_range(-1.0) && !gap_in_range(10.01));
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

    fn place(e: &Event, scale: u32) -> Vec<NumberPlacement> {
        place_on(e, &MapIndex::default(), scale)
    }

    fn place_on(e: &Event, map: &MapIndex, scale: u32) -> Vec<NumberPlacement> {
        place_in(e, map, scale, GapRange::DEFAULT)
    }

    fn place_in(e: &Event, map: &MapIndex, scale: u32, gaps: GapRange) -> Vec<NumberPlacement> {
        auto_place(e, &e.courses[0], map, scale, MapKind::Sprint, gaps, false).unwrap()
    }

    fn sprint_rect(e: &Event, direction: f64) -> Rect {
        let sizes = overprint_sizes(MapKind::Sprint);
        number_rect(&e.controls[0].map_mm, direction, 1, 0.5, &sizes, 1.0)
    }

    /// The map index of a sprint map with the given objects.
    fn index(objects: Vec<crate::omap::MapObject>) -> MapIndex {
        MapIndex::from_document(&map(4000, objects))
    }

    #[test]
    fn map_features_are_converted_once() {
        let mut objects = Vec::new();
        for i in 0..3 {
            objects.push(point_object(POINT_113, f64::from(i) * 10.0, 0.0));
        }
        objects.push(line_object(LINE_502, &[(0.0, 5.0), (10.0, 5.0)]));
        objects.push(line_object(LINE_502, &[(0.0, 8.0), (10.0, 8.0)]));
        objects.push(area_object(
            AREA_520,
            &[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)],
        ));
        objects.push(line_object(LINE_601, &[(0.0, 20.0), (10.0, 20.0)]));
        let obstacles = map_obstacles(&index(objects));
        assert_eq!(obstacles.len(), 5);
        let points = obstacles.iter().filter(|o| {
            matches!(o.shape, ObstacleShape::Segment { a, b, half_width }
                if a == b && half_width == POINT_SYMBOL_RADIUS_MM)
        });
        assert_eq!(points.count(), 3);
        let lines = obstacles.iter().filter(|o| {
            matches!(o.shape, ObstacleShape::Segment { a, b, half_width }
                if a != b && half_width == LINE_SYMBOL_HALF_WIDTH_MM)
        });
        assert_eq!(lines.count(), 2);
        assert!(obstacles.iter().all(|o| o.tier == Tier::MapSymbol));
    }

    #[test]
    fn number_avoids_both_legs() {
        // Map y is down. The start lies north-east of the control (45 degrees), the finish
        // south-east (315 degrees); the widest gap is the west side, bisector 180. Direction 90
        // is -y (north), so the free half is 135..225. Behind the incoming line (from the start at
        // 45) is 225, and nothing is in the way there.
        let e = event((120.0, 80.0), &[n("31", 100.0, 100.0)], (120.0, 120.0));
        let placed = place(&e, 4000);
        assert_eq!(placed.len(), 1);
        let d = placed[0].direction;
        assert!((135.0..=225.0).contains(&d), "direction {d}");
        assert!(angle_diff(d, 225.0) < 1e-9, "direction {d}");
        assert_eq!(placed[0].gap_mm, 0.5);
    }

    #[test]
    fn number_avoids_a_neighbour_circle() {
        // Start west, second control south: behind the incoming line is 0 (east). The fourth stop
        // sits 8.5 mm away on that very direction and is reached from the north, so only the circle
        // check can move the number off 0 degrees.
        let d = 8.5;
        let mut e = event(
            (70.0, 100.0),
            &[
                n("31", 100.0, 100.0),
                n("32", 100.0, 130.0),
                n("33", 100.0 + d, 100.0),
            ],
            (130.0, 100.0),
        );
        if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
            controls[1].bends = vec![p(140.0, 130.0), p(140.0, 70.0), p(100.0 + d, 70.0)];
        }
        let placed = place(&e, 4000);
        let sizes = overprint_sizes(MapKind::Sprint);
        let rect = number_rect(
            &e.controls[0].map_mm,
            placed[0].direction,
            1,
            0.5,
            &sizes,
            1.0,
        );
        let other = &e.controls[2].map_mm;
        assert!(
            rect.distance_to_point((other.x, other.y)) >= 3.0 + 0.175,
            "direction {} touches the circle",
            placed[0].direction
        );
        assert!(angle_diff(placed[0].direction, 0.0) >= 15.0);
    }

    #[test]
    fn numbers_do_not_overlap_each_other() {
        // Two controls 2 mm apart on a straight east-west course: the boxes (2.4 mm wide) must not
        // overlap, whatever direction each one takes.
        let e = event(
            (70.0, 100.0),
            &[n("31", 100.0, 100.0), n("32", 102.0, 100.0)],
            (140.0, 100.0),
        );
        let placed = place(&e, 4000);
        let sizes = overprint_sizes(MapKind::Sprint);
        let a = number_rect(
            &e.controls[0].map_mm,
            placed[0].direction,
            1,
            0.5,
            &sizes,
            1.0,
        );
        let b = number_rect(
            &e.controls[1].map_mm,
            placed[1].direction,
            2,
            0.5,
            &sizes,
            1.0,
        );
        assert!(!a.overlaps(&b), "{a:?} overlaps {b:?}");
    }

    #[test]
    fn new_numbers_avoid_kept_numbers() {
        let mut e = event(
            (70.0, 100.0),
            &[n("31", 100.0, 100.0), n("32", 102.0, 100.0)],
            (140.0, 100.0),
        );
        // Kept number of 31 points north, exactly where 32 would like to put its own.
        let kept = NumberPlacement {
            direction: 90.0,
            gap_mm: 0.5,
        };
        if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
            controls[0].number = Some(kept.clone());
        }
        assert_eq!(
            place_numbers(
                &mut e,
                "A",
                &MapIndex::default(),
                4000,
                MapKind::Sprint,
                GapRange::DEFAULT,
                false
            ),
            Ok(1)
        );
        let CourseKind::Individual { controls } = &e.courses[0].kind else {
            unreachable!()
        };
        assert_eq!(controls[0].number, Some(kept));
        let new = controls[1].number.clone().unwrap();
        assert!(
            angle_diff(new.direction, 90.0) > 1.0,
            "direction {}",
            new.direction
        );
        let sizes = overprint_sizes(MapKind::Sprint);
        let a = number_rect(&e.controls[0].map_mm, 90.0, 1, 0.5, &sizes, 1.0);
        let b = number_rect(&e.controls[1].map_mm, new.direction, 2, 0.5, &sizes, 1.0);
        assert!(!a.overlaps(&b));
    }

    fn numbers_of(e: &Event) -> Vec<Option<NumberPlacement>> {
        let CourseKind::Individual { controls } = &e.courses[0].kind else {
            unreachable!()
        };
        controls.iter().map(|c| c.number.clone()).collect()
    }

    #[test]
    fn central_number_wins_over_automatic_placement() {
        // 31 has a central number pointing north, where 32 would like to put its own (as in
        // new_numbers_avoid_kept_numbers). Placement keeps it, copies nothing into the course and
        // steers 32 away from it, with and without overwrite.
        for overwrite in [false, true] {
            let mut e = event(
                (70.0, 100.0),
                &[n("31", 100.0, 100.0), n("32", 102.0, 100.0)],
                (140.0, 100.0),
            );
            let central = NumberPlacement {
                direction: 90.0,
                gap_mm: 0.5,
            };
            e.controls[0].number = Some(central.clone());
            assert_eq!(
                place_numbers(
                    &mut e,
                    "A",
                    &MapIndex::default(),
                    4000,
                    MapKind::Sprint,
                    GapRange::DEFAULT,
                    overwrite
                ),
                Ok(1),
                "overwrite {overwrite}"
            );
            let numbers = numbers_of(&e);
            assert_eq!(numbers[0], None, "overwrite {overwrite}");
            assert_eq!(e.controls[0].number, Some(central));
            let new = numbers[1].clone().unwrap();
            let sizes = overprint_sizes(MapKind::Sprint);
            let a = number_rect(&e.controls[0].map_mm, 90.0, 1, 0.5, &sizes, 1.0);
            let b = number_rect(&e.controls[1].map_mm, new.direction, 2, 0.5, &sizes, 1.0);
            assert!(!a.overlaps(&b), "overwrite {overwrite}: {}", new.direction);
        }
    }

    #[test]
    fn overwrite_keeps_a_course_number_of_a_control_with_central_number() {
        // 31 has both: the course number is the one drawn, so it stays and is the obstacle.
        let mut e = event(
            (70.0, 100.0),
            &[n("31", 100.0, 100.0), n("32", 102.0, 100.0)],
            (140.0, 100.0),
        );
        e.controls[0].number = Some(NumberPlacement {
            direction: 270.0,
            gap_mm: 0.5,
        });
        let own = NumberPlacement {
            direction: 90.0,
            gap_mm: 0.5,
        };
        if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
            controls[0].number = Some(own.clone());
        }
        assert_eq!(
            place_numbers(
                &mut e,
                "A",
                &MapIndex::default(),
                4000,
                MapKind::Sprint,
                GapRange::DEFAULT,
                true
            ),
            Ok(1)
        );
        let numbers = numbers_of(&e);
        assert_eq!(numbers[0], Some(own));
        assert!(angle_diff(numbers[1].as_ref().unwrap().direction, 90.0) > 1.0);
    }

    #[test]
    fn box_at_90_degrees_lies_above() {
        // y is down, so north (90 degrees) is smaller y.
        let sizes = overprint_sizes(MapKind::Sprint);
        let rect = number_rect(&p(100.0, 100.0), 90.0, 1, 0.5, &sizes, 1.0);
        assert!(rect.max.1 < 100.0);
        assert!((rect.max.1 - (100.0 - 3.675)).abs() < 1e-9);
    }

    #[test]
    fn gap_is_the_clearance_between_circle_line_and_box() {
        // Shared with freemapper-print: the gap is the shortest distance from the outer edge of the
        // circle line (3 + 0.175 on sprint) to the box, and the box is centred on the direction ray.
        let sizes = overprint_sizes(MapKind::Sprint);
        let r_outer = 3.175;
        for number in [1, 12] {
            for gap in [0.3, 1.0] {
                for direction in [0.0, 30.0, 45.0, 60.0, 90.0, 135.0, 200.0, 315.0] {
                    let rect = number_rect(&p(0.0, 0.0), direction, number, gap, &sizes, 1.0);
                    let clearance = rect.distance_to_point((0.0, 0.0)) - r_outer;
                    assert!(
                        (clearance - gap).abs() < 1e-9,
                        "{number} at {direction} with gap {gap}: clearance {clearance}"
                    );
                    let (cx, cy) = (
                        (rect.min.0 + rect.max.0) / 2.0,
                        (rect.min.1 + rect.max.1) / 2.0,
                    );
                    let rad = f64::to_radians(direction);
                    assert!(
                        (cx * -rad.sin() - cy * rad.cos()).abs() < 1e-9,
                        "centre off the ray"
                    );
                    assert!(
                        cx * rad.cos() - cy * rad.sin() > 0.0,
                        "centre behind the circle"
                    );
                }
            }
        }
    }

    #[test]
    fn placement_scales_with_the_map_scale() {
        // The neighbour-circle layout drawn at 1:5000 with all distances times 4000/5000 must
        // give the same direction as at 1:4000 (sizes in map mm shrink by the same factor).
        let layout = |f: f64| {
            let d = 8.0 * f / 2f64.sqrt();
            let mut e = event(
                (100.0 - 30.0 * f, 100.0),
                &[
                    n("31", 100.0, 100.0),
                    n("32", 100.0, 100.0 + 30.0 * f),
                    n("33", 100.0 + d, 100.0 - d),
                ],
                (100.0 + 30.0 * f, 100.0 - d),
            );
            if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
                controls[1].bends = vec![
                    p(100.0 + 40.0 * f, 100.0 + 30.0 * f),
                    p(100.0 + 40.0 * f, 100.0 - 30.0 * f),
                    p(100.0 + d, 100.0 - 30.0 * f),
                ];
            }
            e
        };
        let base = place(&layout(1.0), 4000);
        let small = place(&layout(0.8), 5000);
        assert_eq!(base[0].direction, small[0].direction);
        // A box at 1:5000 is 0.8 times as high: east box starts at 0.8 * (3.175 + 0.5).
        let sizes = overprint_sizes(MapKind::Sprint);
        let r = number_rect(&p(0.0, 0.0), 0.0, 1, 0.5, &sizes, 0.8);
        assert!((r.min.0 - 0.8 * 3.675).abs() < 1e-9);
        assert!(((r.max.1 - r.min.1) - 3.2).abs() < 1e-9);
    }

    #[test]
    fn preferred_is_behind_the_incoming_leg() {
        // In from the west, out to the south: behind is east (0), free of the outgoing line.
        let e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (100.0, 140.0));
        assert!(angle_diff(place(&e, 4000)[0].direction, 0.0) < 1e-9);
    }

    #[test]
    fn obtuse_angle_prefers_outer_bisector() {
        // In from the left, out to the right, 170 degrees, the tip slightly up (the control lies
        // above the line from start to finish). Behind is blocked by the outgoing line, so the
        // number goes straight above: 90.
        let rise = 30.0 * 5f64.to_radians().tan();
        let e = event(
            (70.0, 100.0 + rise),
            &[n("31", 100.0, 100.0)],
            (130.0, 100.0 + rise),
        );
        assert!(angle_diff(place(&e, 4000)[0].direction, 90.0) < 1e-9);
    }

    /// Course A: start (70, 100) -> 31 (100, 100) -> 32 at `second` -> 33 at (`x33`, 100) ->
    /// finish (130, 100); the leg to 33 may bend at `bend`.
    fn blocked_behind(second: (f64, f64), x33: f64, bend: Option<(f64, f64)>) -> Event {
        let mut e = event(
            (70.0, 100.0),
            &[
                n("31", 100.0, 100.0),
                n("32", second.0, second.1),
                n("33", x33, 100.0),
            ],
            (130.0, 100.0),
        );
        if let (Some(b), CourseKind::Individual { controls }) = (bend, &mut e.courses[0].kind) {
            controls[2].bends = vec![p(b.0, b.1)];
        }
        e
    }

    #[test]
    fn blocked_preferred_takes_nearest_free_direction() {
        // 31: in from the west, out to the south (32), so behind is east (0). The circle of 33,
        // 9.2 mm east of 31 and reached from the south-east, takes that direction: the box at 0 and
        // at 15 stays 3.12 mm from its centre (needs 3.175), at 30 it is 3.3 mm away. So the
        // smallest free step is 30; of 30 and 330 the one towards the outer side (45) wins.
        let e = blocked_behind((100.0, 140.0), 109.2, Some((120.0, 130.0)));
        let d = place(&e, 4000)[0].direction;
        let off = angle_diff(d, 0.0);
        assert!((off - 30.0).abs() < 1e-9 && d < 90.0, "direction {d}");
        let sizes = overprint_sizes(MapKind::Sprint);
        let rect = number_rect(&e.controls[0].map_mm, d, 1, 0.5, &sizes, 1.0);
        let other = &e.controls[2].map_mm;
        assert!(rect.distance_to_point((other.x, other.y)) >= 3.0 + 0.175);
        // Straight behind really is blocked.
        let behind = number_rect(&e.controls[0].map_mm, 0.0, 1, 0.5, &sizes, 1.0);
        assert!(behind.distance_to_point((other.x, other.y)) < 3.0 + 0.175);
    }

    #[test]
    fn ties_prefer_the_outer_side() {
        // 31: in from the west, out to the north (32), so behind is east (0) and the outer
        // bisector is 315. The circle of 33 lies on the east axis, 8.5 mm away, so it blocks 0 and
        // is mirror-symmetric for +k and -k: +15 and -15 (and every other pair) rank equal on
        // conflicts and on distance to the preferred direction. The one closer to the outer
        // bisector, below the axis, wins.
        let e = blocked_behind((100.0, 60.0), 108.5, Some((120.0, 70.0)));
        let d = place(&e, 4000)[0].direction;
        assert!(d > 180.0, "direction {d}");
        let sizes = overprint_sizes(MapKind::Sprint);
        let other = &e.controls[2].map_mm;
        for dir in [d, 360.0 - d] {
            let rect = number_rect(&e.controls[0].map_mm, dir, 1, 0.5, &sizes, 1.0);
            assert!(
                rect.distance_to_point((other.x, other.y)) >= 3.0 + 0.175,
                "{dir}"
            );
        }
    }

    #[test]
    fn behind_follows_the_last_bend() {
        // The previous point is due west, but the line bends south of the control and arrives
        // from the south, so behind is north (90). The outer bisector would be 135.
        let mut e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (140.0, 100.0));
        e.courses[0].start_bends = vec![p(100.0, 140.0)];
        assert!(angle_diff(place(&e, 4000)[0].direction, 90.0) < 1e-9);
    }

    #[test]
    fn incoming_without_length_falls_back_to_the_outgoing_line() {
        // Start triangle on the control: no incoming direction; the number goes opposite the
        // outgoing line (east), so west (180).
        let e = event((100.0, 100.0), &[n("31", 100.0, 100.0)], (140.0, 100.0));
        assert!(angle_diff(place(&e, 4000)[0].direction, 180.0) < 1e-9);
    }

    #[test]
    fn overwrite_does_not_seed_kept_numbers() {
        // 32 is so close that the outgoing leg of 31 vanishes inside the circle, so behind (east)
        // counts as free and the circle of 32 pushes 31 to 60. The existing number of 32 points
        // north; with `overwrite` it must not push 31 away from where it goes without it.
        let mut e = event(
            (70.0, 100.0),
            &[n("31", 100.0, 100.0), n("32", 102.0, 100.0)],
            (140.0, 100.0),
        );
        if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
            controls[1].number = Some(NumberPlacement {
                direction: 90.0,
                gap_mm: 0.5,
            });
        }
        let baseline = place(&e, 4000)[0].direction;
        let mut without = e.clone();
        if let CourseKind::Individual { controls } = &mut without.courses[0].kind {
            controls[1].number = None;
        }
        assert_eq!(place(&without, 4000)[0].direction, baseline);
        assert_eq!(
            place_numbers(
                &mut e,
                "A",
                &MapIndex::default(),
                4000,
                MapKind::Sprint,
                GapRange::DEFAULT,
                true
            ),
            Ok(2)
        );
        let CourseKind::Individual { controls } = &e.courses[0].kind else {
            unreachable!()
        };
        assert_eq!(controls[0].number.as_ref().unwrap().direction, baseline);

        // With the existing number of 32 kept, it does matter for 31: the direction changes.
        let mut kept = e.clone();
        if let CourseKind::Individual { controls } = &mut kept.courses[0].kind {
            controls[0].number = None;
            controls[1].number = Some(NumberPlacement {
                direction: 90.0,
                gap_mm: 0.5,
            });
        }
        let with_kept = auto_place(
            &kept,
            &kept.courses[0],
            &MapIndex::default(),
            4000,
            MapKind::Sprint,
            GapRange::DEFAULT,
            true,
        )
        .unwrap();
        assert!(angle_diff(with_kept[0].direction, baseline) > 1.0);
    }

    #[test]
    fn point_and_line_symbols_are_hard_area_symbols_soft() {
        // In from the west, out to the south; behind is east (0 degrees), and its box is free.
        let e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (100.0, 140.0));
        assert!(angle_diff(place(&e, 4000)[0].direction, 0.0) < 1e-9);
        // The 0 degree box spans x 103.675..106.075 around y 100; 5.5 mm east is inside it.
        // (a) A 113 point there: the number leaves 0 degrees.
        let point = index(vec![point_object(POINT_113, 105.5, 100.0)]);
        let d = place_on(&e, &point, 4000)[0].direction;
        assert!(angle_diff(d, 0.0) >= 15.0, "direction {d}");
        assert!(sprint_rect(&e, d).distance_to_point((105.5, 100.0)) > POINT_SYMBOL_RADIUS_MM);
        // (b) A 502 line running north-south there: the same.
        let line = index(vec![line_object(
            LINE_502,
            &[(105.5, 90.0), (105.5, 110.0)],
        )]);
        let d = place_on(&e, &line, 4000)[0].direction;
        assert!(angle_diff(d, 0.0) >= 15.0, "direction {d}");
        assert!(
            sprint_rect(&e, d).distance_to_segment((105.5, 90.0), (105.5, 110.0))
                > LINE_SYMBOL_HALF_WIDTH_MM
        );
        // (c) A 520 area over the whole neighbourhood does not count: the number stays at 0.
        let area = index(vec![area_object(
            AREA_520,
            &[(80.0, 80.0), (120.0, 80.0), (120.0, 120.0), (80.0, 120.0)],
        )]);
        assert!(angle_diff(place_on(&e, &area, 4000)[0].direction, 0.0) < 1e-9);
    }

    #[test]
    fn point_symbol_counts_with_its_radius() {
        let e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (100.0, 140.0));
        let east = sprint_rect(&e, 0.0);
        // 0.3 mm outside the box (less than the symbol radius): still covered.
        let near = index(vec![point_object(POINT_113, east.max.0 + 0.3, 100.0)]);
        assert!(angle_diff(place_on(&e, &near, 4000)[0].direction, 0.0) >= 15.0);
        // 0.7 mm outside: free.
        let far = index(vec![point_object(POINT_113, east.max.0 + 0.7, 100.0)]);
        assert!(angle_diff(place_on(&e, &far, 4000)[0].direction, 0.0) < 1e-9);
    }

    #[test]
    fn every_direction_covered_takes_the_fewest_conflicts() {
        // In from the west, out to the south: preferred is 0, the legs touch 180 and 270.
        // A ring of 502 line (radius 4.5) crosses every candidate box: one conflict each. A short
        // 502 piece in the middle of every candidate box except the one at 90 makes the others
        // cost two or more, so 90 wins although it is far from the preferred direction.
        let e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (100.0, 140.0));
        let mut objects = Vec::new();
        let ring: Vec<(f64, f64)> = (0..=72)
            .map(|i| {
                let a = f64::from(i) * 5f64.to_radians();
                (100.0 + 4.5 * a.cos(), 100.0 - 4.5 * a.sin())
            })
            .collect();
        objects.push(line_object(LINE_502, &ring));
        let target = 90.0;
        for k in 0..24 {
            let direction = f64::from(k) * 15.0;
            if direction == target {
                continue;
            }
            let r = sprint_rect(&e, direction);
            let (cx, cy) = ((r.min.0 + r.max.0) / 2.0, (r.min.1 + r.max.1) / 2.0);
            objects.push(line_object(LINE_502, &[(cx - 0.1, cy), (cx + 0.1, cy)]));
        }
        let map = index(objects);
        // Count by hand: the target is the only candidate with exactly one covered feature.
        let counts: Vec<usize> = (0..24)
            .map(|k| {
                let r = sprint_rect(&e, f64::from(k) * 15.0);
                map.features_near(e.controls[0].map_mm.clone(), 20.0)
                    .iter()
                    .filter(|(f, _)| feature_covered(&r, f))
                    .count()
            })
            .collect();
        assert_eq!(counts[6], 1, "{counts:?}");
        assert!(
            counts.iter().enumerate().all(|(k, c)| k == 6 || *c >= 2),
            "{counts:?}"
        );
        let placed = place_on(&e, &map, 4000);
        assert!(angle_diff(placed[0].direction, target) < 1e-9);
    }

    #[test]
    fn magnetic_north_lines_never_count_as_covered() {
        let e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (100.0, 140.0));
        let east = sprint_rect(&e, 0.0);
        let cy = (east.min.1 + east.max.1) / 2.0;
        let north_lines = index(vec![line_object(
            LINE_601,
            &[(east.min.0 - 3.0, cy), (east.max.0 + 3.0, cy)],
        )]);
        assert!(angle_diff(place_on(&e, &north_lines, 4000)[0].direction, 0.0) < 1e-9);
        let same_geometry = index(vec![line_object(
            LINE_502,
            &[(east.min.0 - 3.0, cy), (east.max.0 + 3.0, cy)],
        )]);
        assert!(angle_diff(place_on(&e, &same_geometry, 4000)[0].direction, 0.0) >= 15.0);
    }

    #[test]
    fn candidate_directions_are_rounded_to_a_tenth_and_never_360() {
        assert_eq!(candidate_direction(359.96, 0), 0.0);
        for step in -11..=12 {
            let d = candidate_direction(123.456_789, step);
            assert!((0.0..360.0).contains(&d), "{d}");
            assert!((d * 10.0 - (d * 10.0).round()).abs() < 1e-9, "{d}");
        }
        // Through auto_place: incoming from 3 degrees off west, so behind is not a round value.
        let e = event((70.0, 100.0), &[n("31", 100.0, 101.7)], (100.0, 140.0));
        let d = place(&e, 4000)[0].direction;
        assert!(
            d < 360.0 && (d * 10.0 - (d * 10.0).round()).abs() < 1e-9,
            "{d}"
        );
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
        let placed = place(&e, 4000);
        assert_eq!(placed.len(), 2);
    }

    #[test]
    fn number_size_scales_with_the_map_scale() {
        let sizes = overprint_sizes(MapKind::Sprint);
        let at_base = number_rect(&p(0.0, 0.0), 0.0, 1, 0.5, &sizes, 1.0);
        let at_double = number_rect(&p(0.0, 0.0), 0.0, 1, 0.5, &sizes, 2.0);
        assert!(
            ((at_double.max.1 - at_double.min.1) - 2.0 * (at_base.max.1 - at_base.min.1)).abs()
                < 1e-9
        );
        // East: the box starts at the outer edge of the circle line 3.175 + gap 0.5.
        assert!(((at_base.min.0) - 3.675).abs() < 1e-9);
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
        let set = place_numbers(
            &mut e,
            "A",
            &MapIndex::default(),
            4000,
            MapKind::Sprint,
            GapRange::DEFAULT,
            false,
        )
        .unwrap();
        assert_eq!(set, 1);
        let CourseKind::Individual { controls } = &e.courses[0].kind else {
            unreachable!()
        };
        assert_eq!(controls[0].number, Some(old));
        assert!(controls[1].number.is_some());

        let set = place_numbers(
            &mut e,
            "A",
            &MapIndex::default(),
            4000,
            MapKind::Sprint,
            GapRange::DEFAULT,
            true,
        )
        .unwrap();
        assert_eq!(set, 2);
        let CourseKind::Individual { controls } = &e.courses[0].kind else {
            unreachable!()
        };
        assert_eq!(controls[0].number.as_ref().unwrap().gap_mm, 0.5);

        assert!(matches!(
            place_numbers(
                &mut e,
                "Z",
                &MapIndex::default(),
                4000,
                MapKind::Sprint,
                GapRange::DEFAULT,
                false
            ),
            Err(EditError::NotFound { .. })
        ));
    }

    #[test]
    fn candidate_gaps_end_at_max() {
        let gaps = |min, max| candidate_gaps(GapRange { min, max });
        assert_eq!(
            gaps(0.5, 5.0),
            vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0]
        );
        assert_eq!(gaps(0.5, 1.2), vec![0.5, 1.0, 1.2]);
        assert_eq!(candidate_gaps(GapRange::DEFAULT), vec![0.5]);
    }

    #[test]
    fn candidate_gaps_round_inwards_and_always_end() {
        let gaps = |min, max| candidate_gaps(GapRange { min, max });
        // The minimum rounds up, so the first gap is never below it.
        assert_eq!(gaps(0.333, 0.333)[0], 0.34);
        // The maximum rounds down, so the last gap is never above it.
        assert_eq!(*gaps(0.5, 1.237).last().unwrap(), 1.23);
        for (min, max) in [
            (f64::NAN, f64::NAN),
            (f64::INFINITY, f64::INFINITY),
            (0.0, f64::INFINITY),
            (-3.0, 1e300),
            (50.0, 50.0),
        ] {
            let list = candidate_gaps(GapRange { min, max });
            assert!(!list.is_empty() && list.len() <= 1001, "{min} {max}");
            assert!(
                list.iter().all(|g| (0.01..=MAX_GAP_MM).contains(g)),
                "{list:?}"
            );
        }
    }

    #[test]
    fn gap_range_ignores_invalid_values_and_clamps() {
        let mut set = load_rule_set("wkb-2026", &[]).unwrap();
        set.rules.retain(|r| r.rule.key != "number_gap");
        // Not finite or not above 0 counts as missing.
        set.rules
            .push(gap_rule("t.bad", "min_gap_mm = nan\nmax_gap_mm = inf"));
        assert_eq!(range_of(&set), GapRange::DEFAULT);
        set.rules.clear();
        set.rules
            .push(gap_rule("t.neg", "min_gap_mm = -1.0\nmax_gap_mm = 0.0"));
        assert_eq!(range_of(&set), GapRange::DEFAULT);
        // 50 mm is clamped to the largest gap the CLI accepts.
        set.rules.clear();
        set.rules
            .push(gap_rule("t.big", "min_gap_mm = 50.0\nmax_gap_mm = 50.0"));
        assert_eq!(
            range_of(&set),
            GapRange {
                min: MAX_GAP_MM,
                max: MAX_GAP_MM
            }
        );
        // A tiny positive value is lifted to 0.01.
        set.rules.clear();
        set.rules.push(gap_rule("t.tiny", "min_gap_mm = 0.000001"));
        assert_eq!(range_of(&set).min, 0.01);
        // An invalid value of one rule leaves the valid one of another alone.
        set.rules.clear();
        set.rules
            .push(gap_rule("t.ok", "min_gap_mm = 1.0\nmax_gap_mm = 3.0"));
        set.rules.push(gap_rule("t.nan", "max_gap_mm = nan"));
        assert_eq!(range_of(&set), GapRange { min: 1.0, max: 3.0 });
    }

    #[test]
    fn line_along_the_preferred_direction_blocks_every_gap() {
        // In from the west, out to the south: preferred is east. A 502 line runs outward along
        // 0 degrees through all gaps from 0.5 to 5 mm, so the number turns away.
        let e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (100.0, 140.0));
        let line = index(vec![line_object(
            LINE_502,
            &[(103.0, 100.0), (130.0, 100.0)],
        )]);
        let range = GapRange { min: 0.5, max: 5.0 };
        let d = place_in(&e, &line, 4000, range)[0].direction;
        assert!(angle_diff(d, 0.0) >= 15.0, "direction {d}");
    }

    #[test]
    fn blocked_at_min_gap_moves_outward_before_rotating() {
        // In from the west, out to the south: preferred is east (0). A point symbol just outside
        // the circle line is covered by the box at 0.5 mm, but not by the box further out.
        let e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (100.0, 140.0));
        let east = sprint_rect(&e, 0.0);
        let blocker = index(vec![point_object(POINT_113, east.min.0 + 0.15, 100.0)]);
        let fixed = place_in(&e, &blocker, 4000, GapRange::DEFAULT)[0].clone();
        assert!(angle_diff(fixed.direction, 0.0) >= 15.0, "{fixed:?}");
        assert!((fixed.gap_mm - 0.5).abs() < 1e-9);
        let range = GapRange { min: 0.5, max: 5.0 };
        let wide = place_in(&e, &blocker, 4000, range)[0].clone();
        assert!(angle_diff(wide.direction, 0.0) < 1e-9, "{wide:?}");
        assert!(wide.gap_mm > 0.5 && wide.gap_mm <= 5.0, "{wide:?}");
    }

    #[test]
    fn kept_number_outside_range_is_not_moved() {
        let mut e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (100.0, 140.0));
        let far = NumberPlacement {
            direction: 90.0,
            gap_mm: 8.0,
        };
        if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
            controls[0].number = Some(far.clone());
        }
        let range = GapRange { min: 0.5, max: 5.0 };
        let out = auto_place(
            &e,
            &e.courses[0],
            &MapIndex::default(),
            4000,
            MapKind::Sprint,
            range,
            true,
        )
        .unwrap();
        assert_eq!(out, vec![far]);
    }
}
