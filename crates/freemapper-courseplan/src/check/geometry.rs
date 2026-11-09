//! Geometry of the leg checks in map millimetres: the drawn part of a leg and whether it meets a
//! line or an area.

use freemapper_courseplan_model::MapPoint;

use crate::map::inside_area;

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
pub(crate) fn drawn_part(points: &[MapPoint], cut: f64) -> Vec<MapPoint> {
    let length: f64 = points.windows(2).map(|w| distance(&w[0], &w[1])).sum();
    let (from, to) = (cut, length - cut);
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
