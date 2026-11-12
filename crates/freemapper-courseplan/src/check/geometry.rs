//! Geometry of the leg checks in map millimetres: the drawn part of a leg, whether it meets a
//! line or an area, angles between directions and stretches where two lines run against each
//! other.

use freemapper_courseplan_model::MapPoint;

use crate::map::{distance_to_polyline, inside_area};

pub(crate) fn distance(a: &MapPoint, b: &MapPoint) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// The point at `t` (0 to 1) between `a` and `b`.
fn between(a: &MapPoint, b: &MapPoint, t: f64) -> MapPoint {
    MapPoint {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

/// The part of a polyline that is drawn when its ends start and stop at circles: `cut` map mm
/// along the line are left out at either end. Empty when nothing is left.
pub fn drawn_part(points: &[MapPoint], cut: f64) -> Vec<MapPoint> {
    drawn_part_between(points, cut, cut)
}

/// Like [`drawn_part`], with `cut_from` map mm left out at the start and `cut_to` at the end: the legs of a
/// course leave a start triangle, a control circle, a finish or a crossing point at different distances.
pub fn drawn_part_between(points: &[MapPoint], cut_from: f64, cut_to: f64) -> Vec<MapPoint> {
    let length: f64 = points.windows(2).map(|w| distance(&w[0], &w[1])).sum();
    let (from, to) = (cut_from, length - cut_to);
    if points.len() < 2 || to <= from {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut walked = 0.0;
    for w in points.windows(2) {
        let step = distance(&w[0], &w[1]);
        let (start, end) = (walked, walked + step);
        walked = end;
        if step == 0.0 || end < from || start > to {
            continue;
        }
        let at = |s: f64| between(&w[0], &w[1], ((s - start) / step).clamp(0.0, 1.0));
        if out.is_empty() {
            out.push(at(from.max(start)));
        }
        out.push(at(to.min(end)));
    }
    out
}

fn cross(o: &MapPoint, a: &MapPoint, b: &MapPoint) -> f64 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

fn on_segment(p: &MapPoint, a: &MapPoint, b: &MapPoint) -> bool {
    p.x >= a.x.min(b.x) && p.x <= a.x.max(b.x) && p.y >= a.y.min(b.y) && p.y <= a.y.max(b.y)
}

/// Whether segments `a`–`b` and `c`–`d` meet; touching counts.
pub(crate) fn segments_meet(a: &MapPoint, b: &MapPoint, c: &MapPoint, d: &MapPoint) -> bool {
    let (d1, d2) = (cross(c, d, a), cross(c, d, b));
    let (d3, d4) = (cross(a, b, c), cross(a, b, d));
    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }
    (d1 == 0.0 && on_segment(a, c, d))
        || (d2 == 0.0 && on_segment(b, c, d))
        || (d3 == 0.0 && on_segment(c, a, b))
        || (d4 == 0.0 && on_segment(d, a, b))
}

/// Whether two polylines meet; `closed` closes the second one.
pub(crate) fn polylines_meet(leg: &[MapPoint], line: &[MapPoint], closed: bool) -> bool {
    let mut edges: Vec<(&MapPoint, &MapPoint)> = line.windows(2).map(|w| (&w[0], &w[1])).collect();
    if closed && let (Some(last), Some(first)) = (line.last(), line.first()) {
        edges.push((last, first));
    }
    leg.windows(2)
        .any(|s| edges.iter().any(|(c, d)| segments_meet(&s[0], &s[1], c, d)))
}

/// Whether a polyline enters an area: it crosses a ring, or a point of it lies inside.
pub(crate) fn polyline_meets_area(leg: &[MapPoint], rings: &[Vec<MapPoint>]) -> bool {
    leg.iter().any(|p| inside_area(p, rings))
        || rings.iter().any(|ring| polylines_meet(leg, ring, true))
}

/// Slack for a point that lies on an edge, in map mm: far below what a map or a click can place.
const ON_EDGE_MM: f64 = 1e-9;

/// Whether `p` lies inside an area or on the edge of one of its rings. The rings follow the even-odd
/// rule of the map (`FeatureGeometry::Area`): the first is the outer ring, the others are holes, and a
/// point in a hole is outside. The edge of a hole is part of the area, so it counts as inside.
pub(crate) fn point_in_area(p: &MapPoint, rings: &[Vec<MapPoint>]) -> bool {
    inside_area(p, rings)
        || rings
            .iter()
            .any(|ring| distance_to_polyline(p, ring, true) <= ON_EDGE_MM)
}

/// The angle in degrees between two directions, 0 to 180; `None` when one has no length.
fn angle_between(u: (f64, f64), v: (f64, f64)) -> Option<f64> {
    let (lu, lv) = (u.0.hypot(u.1), v.0.hypot(v.1));
    if lu == 0.0 || lv == 0.0 {
        return None;
    }
    let cos = ((u.0 * v.0 + u.1 * v.1) / (lu * lv)).clamp(-1.0, 1.0);
    Some(cos.acos().to_degrees())
}

/// The angle in degrees at `at` between the directions to `before` and to `after`: 180 for a
/// straight line through `at`, 0 for a line that turns back on itself. `None` when `at`
/// coincides with one of them.
pub(crate) fn angle_at(before: &MapPoint, at: &MapPoint, after: &MapPoint) -> Option<f64> {
    angle_between(
        (before.x - at.x, before.y - at.y),
        (after.x - at.x, after.y - at.y),
    )
}

/// The angle in degrees between the directions of the segments `a`–`b` and `c`–`d`: 0 for the
/// same direction, 180 for opposite ones. `None` for a segment without length.
pub(crate) fn direction_difference(
    a: &MapPoint,
    b: &MapPoint,
    c: &MapPoint,
    d: &MapPoint,
) -> Option<f64> {
    angle_between((b.x - a.x, b.y - a.y), (d.x - c.x, d.y - c.y))
}

/// A part of a segment as parameters from 0 to 1.
type Span = (f64, f64);

/// The part of `a`–`b` within `reach` of `centre`.
fn span_near_point(a: &MapPoint, b: &MapPoint, centre: &MapPoint, reach: f64) -> Option<Span> {
    let (wx, wy) = (b.x - a.x, b.y - a.y);
    let (fx, fy) = (a.x - centre.x, a.y - centre.y);
    let qa = wx * wx + wy * wy;
    let qb = 2.0 * (wx * fx + wy * fy);
    let qc = fx * fx + fy * fy - reach * reach;
    if qa == 0.0 {
        return (qc <= 0.0).then_some((0.0, 1.0));
    }
    let disc = qb * qb - 4.0 * qa * qc;
    if disc < 0.0 {
        return None;
    }
    let root = disc.sqrt();
    clip(((-qb - root) / (2.0 * qa), (-qb + root) / (2.0 * qa)))
}

/// The span limited to the segment, or `None` when nothing is left.
fn clip((from, to): Span) -> Option<Span> {
    let (from, to) = (from.max(0.0), to.min(1.0));
    (from <= to).then_some((from, to))
}

/// The part of `a`–`b` within `reach` of the segment `c`–`d`. That neighbourhood is convex
/// (a rectangle along `c`–`d` with a half disc at either end), so the part is one span: from
/// the first to the last parameter inside the rectangle or one of the discs.
fn span_near_segment(
    a: &MapPoint,
    b: &MapPoint,
    c: &MapPoint,
    d: &MapPoint,
    reach: f64,
) -> Option<Span> {
    let mut parts = vec![
        span_near_point(a, b, c, reach),
        span_near_point(a, b, d, reach),
    ];
    let length = distance(c, d);
    if length > 0.0 {
        // In coordinates along (u) and across (v) `c`–`d`.
        let (ux, uy) = ((d.x - c.x) / length, (d.y - c.y) / length);
        let along = |p: &MapPoint| (p.x - c.x) * ux + (p.y - c.y) * uy;
        let across = |p: &MapPoint| -(p.x - c.x) * uy + (p.y - c.y) * ux;
        let slab = |from: f64, to: f64, lo: f64, hi: f64| -> Option<Span> {
            let step = to - from;
            if step == 0.0 {
                return (lo <= from && from <= hi).then_some((0.0, 1.0));
            }
            let (t0, t1) = ((lo - from) / step, (hi - from) / step);
            clip((t0.min(t1), t0.max(t1)))
        };
        let rectangle = slab(along(a), along(b), 0.0, length)
            .zip(slab(across(a), across(b), -reach, reach))
            .and_then(|(s, t)| clip((s.0.max(t.0), s.1.min(t.1))));
        parts.push(rectangle);
    }
    let parts: Vec<Span> = parts.into_iter().flatten().collect();
    let from = parts.iter().map(|s| s.0).reduce(f64::min)?;
    let to = parts.iter().map(|s| s.1).reduce(f64::max)?;
    Some((from, to))
}

/// The spans merged into disjoint ones, in order.
fn merged(mut spans: Vec<Span>) -> Vec<Span> {
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out: Vec<Span> = Vec::with_capacity(spans.len());
    for span in spans {
        match out.last_mut() {
            Some(last) if span.0 <= last.1 => last.1 = last.1.max(span.1),
            _ => out.push(span),
        }
    }
    out
}

fn total(spans: &[Span]) -> f64 {
    spans.iter().map(|s| s.1 - s.0).sum()
}

/// The length of what two sets of disjoint spans share.
fn shared(a: &[Span], b: &[Span]) -> f64 {
    let mut sum = 0.0;
    for x in a {
        for y in b {
            sum += (x.1.min(y.1) - x.0.max(y.0)).max(0.0);
        }
    }
    sum
}

/// The length of the polyline `line` that runs within `reach` of a segment of `other` whose
/// direction differs from its own by more than `min_angle` degrees, that is, the stretch where
/// both run against each other. What lies within `reach` of a point of `exempt` does not count.
/// Segments without length are left out.
pub(crate) fn opposing_length(
    line: &[MapPoint],
    other: &[MapPoint],
    reach: f64,
    min_angle: f64,
    exempt: &[MapPoint],
) -> f64 {
    let mut sum = 0.0;
    for s in line.windows(2) {
        let (a, b) = (&s[0], &s[1]);
        let length = distance(a, b);
        if length == 0.0 {
            continue;
        }
        let near = merged(
            other
                .windows(2)
                .filter(|o| direction_difference(a, b, &o[0], &o[1]).is_some_and(|d| d > min_angle))
                .filter_map(|o| span_near_segment(a, b, &o[0], &o[1], reach))
                .collect(),
        );
        if near.is_empty() {
            continue;
        }
        let left_out = merged(
            exempt
                .iter()
                .filter_map(|p| span_near_point(a, b, p, reach))
                .collect(),
        );
        sum += (total(&near) - shared(&near, &left_out)) * length;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> MapPoint {
        MapPoint { x, y }
    }

    #[test]
    fn the_drawn_part_leaves_out_the_circles() {
        let line = [p(0.0, 0.0), p(10.0, 0.0)];
        assert_eq!(drawn_part(&line, 2.0), [p(2.0, 0.0), p(8.0, 0.0)]);
        assert!(drawn_part(&line, 5.0).is_empty());
        // Through a bend: the cut follows the line.
        let bent = [p(0.0, 0.0), p(1.0, 0.0), p(1.0, 10.0)];
        assert_eq!(drawn_part(&bent, 2.0), [p(1.0, 1.0), p(1.0, 8.0)]);
        let three = [p(0.0, 0.0), p(5.0, 0.0), p(5.0, 5.0), p(10.0, 5.0)];
        assert_eq!(
            drawn_part(&three, 1.0),
            [p(1.0, 0.0), p(5.0, 0.0), p(5.0, 5.0), p(9.0, 5.0)]
        );
    }

    #[test]
    fn segments_meet_when_they_cross_or_touch() {
        let (a, b) = (p(0.0, 0.0), p(10.0, 0.0));
        assert!(segments_meet(&a, &b, &p(5.0, -1.0), &p(5.0, 1.0)));
        assert!(segments_meet(&a, &b, &p(5.0, 0.0), &p(5.0, 1.0)));
        assert!(!segments_meet(&a, &b, &p(11.0, -1.0), &p(11.0, 1.0)));
        assert!(!segments_meet(&a, &b, &p(0.0, 1.0), &p(10.0, 1.0)));
        assert!(segments_meet(&a, &b, &p(8.0, 0.0), &p(12.0, 0.0)));
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn the_angle_at_a_point_runs_from_0_to_180_degrees() {
        let at = p(0.0, 0.0);
        let right = angle_at(&p(-1.0, 0.0), &at, &p(0.0, 5.0)).unwrap();
        assert!(close(right, 90.0), "{right}");
        let straight = angle_at(&p(-1.0, 0.0), &at, &p(3.0, 0.0)).unwrap();
        assert!(close(straight, 180.0), "{straight}");
        let back = angle_at(&p(-1.0, 0.0), &at, &p(-4.0, 0.0)).unwrap();
        assert!(close(back, 0.0), "{back}");
        let thirty = angle_at(&p(-2.0, 0.0), &at, &p(-(3f64.sqrt()), 1.0)).unwrap();
        assert!(close(thirty, 30.0), "{thirty}");
        assert_eq!(angle_at(&at, &at, &p(1.0, 0.0)), None);
        assert_eq!(angle_at(&p(1.0, 0.0), &at, &at), None);
    }

    #[test]
    fn directions_of_segments_differ_by_0_to_180_degrees() {
        let (a, b) = (p(0.0, 0.0), p(10.0, 0.0));
        let opposite = direction_difference(&a, &b, &p(5.0, 1.0), &p(1.0, 1.0)).unwrap();
        assert!(close(opposite, 180.0));
        let same = direction_difference(&a, &b, &p(5.0, 1.0), &p(6.0, 1.0)).unwrap();
        assert!(close(same, 0.0));
        let across = direction_difference(&a, &b, &p(5.0, 1.0), &p(5.0, 3.0)).unwrap();
        assert!(close(across, 90.0));
        assert_eq!(direction_difference(&a, &a, &a, &b), None);
    }

    #[test]
    fn opposing_length_counts_the_part_near_a_segment_running_the_other_way() {
        let line = [p(0.0, 0.0), p(10.0, 0.0)];
        let back = [p(10.0, 2.0), p(0.0, 2.0)];
        assert!(close(opposing_length(&line, &back, 2.5, 135.0, &[]), 10.0));
        // Same direction, or too far apart: nothing.
        let along = [p(0.0, 2.0), p(10.0, 2.0)];
        assert_eq!(opposing_length(&line, &along, 2.5, 135.0, &[]), 0.0);
        let far = [p(10.0, 3.0), p(0.0, 3.0)];
        assert_eq!(opposing_length(&line, &far, 2.5, 135.0, &[]), 0.0);
        // A shorter segment: the line is near it up to `reach` beyond its end.
        let short = [p(15.0, 0.0), p(5.0, 0.0)];
        assert!(close(opposing_length(&line, &short, 2.5, 135.0, &[]), 7.5));
        // The part within `reach` of an exempt point does not count.
        let exempt = [p(10.0, 0.0)];
        assert!(close(
            opposing_length(&line, &short, 2.5, 135.0, &exempt),
            5.0
        ));
        // A diagonal at 150° counts with 135°, not with 160°.
        let slant = [p(10.0, 1.0), p(10.0 - 10.0 * 3f64.sqrt(), 11.0)];
        assert!(opposing_length(&line, &slant, 2.5, 135.0, &[]) > 0.0);
        assert_eq!(opposing_length(&line, &slant, 2.5, 160.0, &[]), 0.0);
    }

    #[test]
    fn opposing_length_follows_bends_and_counts_overlaps_once() {
        let line = [p(0.0, 0.0), p(10.0, 0.0)];
        // Two segments in a row, both near the line: the part near both counts once.
        let bent = [p(10.0, 1.0), p(5.0, 1.5), p(0.0, 1.0)];
        assert!(close(opposing_length(&line, &bent, 2.5, 135.0, &[]), 10.0));
        // A segment across the line does not count.
        let turn = [p(10.0, 1.0), p(5.0, 1.0), p(5.0, 10.0)];
        let length = opposing_length(&line, &turn, 2.5, 135.0, &[]);
        assert!(
            close(length, 10.0 - (5.0 - (6.25f64 - 1.0).sqrt())),
            "{length}"
        );
        // Zero-length segments are skipped.
        let doubled = [p(10.0, 1.0), p(10.0, 1.0), p(0.0, 1.0)];
        assert!(close(
            opposing_length(&line, &doubled, 2.5, 135.0, &[]),
            10.0
        ));
        let dot = [p(3.0, 0.0), p(3.0, 0.0)];
        assert_eq!(opposing_length(&dot, &bent, 2.5, 135.0, &[]), 0.0);
    }

    #[test]
    fn a_point_in_an_area_is_inside_and_one_on_its_edge_too() {
        let square = vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)];
        let rings = [square];
        assert!(point_in_area(&p(5.0, 5.0), &rings));
        // Edges, corners and the closing edge from the last point to the first.
        assert!(point_in_area(&p(10.0, 5.0), &rings));
        assert!(point_in_area(&p(0.0, 5.0), &rings));
        assert!(point_in_area(&p(5.0, 0.0), &rings));
        assert!(point_in_area(&p(5.0, 10.0), &rings));
        assert!(point_in_area(&p(0.0, 0.0), &rings));
        assert!(point_in_area(&p(10.0, 10.0), &rings));
        assert!(!point_in_area(&p(10.001, 5.0), &rings));
        assert!(!point_in_area(&p(-1.0, -1.0), &rings));
        assert!(!point_in_area(&p(5.0, 10.5), &rings));
    }

    #[test]
    fn a_point_in_a_hole_is_outside_and_one_on_the_hole_edge_is_inside() {
        let outer = vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)];
        let hole = vec![p(4.0, 4.0), p(6.0, 4.0), p(6.0, 6.0), p(4.0, 6.0)];
        let rings = [outer, hole];
        assert!(!point_in_area(&p(5.0, 5.0), &rings));
        assert!(point_in_area(&p(4.0, 5.0), &rings));
        assert!(point_in_area(&p(5.0, 6.0), &rings));
        assert!(point_in_area(&p(2.0, 5.0), &rings));
    }

    #[test]
    fn a_point_in_a_concave_area_is_inside_only_in_the_arms() {
        // An L shape.
        let l = vec![
            p(0.0, 0.0),
            p(10.0, 0.0),
            p(10.0, 4.0),
            p(4.0, 4.0),
            p(4.0, 10.0),
            p(0.0, 10.0),
        ];
        let rings = [l];
        assert!(point_in_area(&p(2.0, 8.0), &rings));
        assert!(point_in_area(&p(8.0, 2.0), &rings));
        assert!(!point_in_area(&p(8.0, 8.0), &rings));
        assert!(point_in_area(&p(4.0, 8.0), &rings));
    }

    #[test]
    fn a_leg_inside_an_area_meets_it() {
        let square = vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)];
        let rings = [square];
        assert!(polyline_meets_area(&[p(2.0, 2.0), p(3.0, 3.0)], &rings));
        assert!(polyline_meets_area(&[p(-2.0, 5.0), p(3.0, 5.0)], &rings));
        assert!(polyline_meets_area(&[p(-2.0, 5.0), p(12.0, 5.0)], &rings));
        assert!(!polyline_meets_area(&[p(-2.0, 5.0), p(-1.0, 12.0)], &rings));
    }
}
