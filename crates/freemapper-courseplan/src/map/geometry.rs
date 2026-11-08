//! Geometry helpers of [`super::MapIndex`]: curve flattening and distances in millimetres.

use freemapper_courseplan_model::MapPoint;

use super::FeatureGeometry;
use crate::omap::MapCoord;
use crate::omap::coord::{FLAG_CLOSE, FLAG_CURVE_START, FLAG_HOLE};

/// Segments per cubic Bézier curve.
const CURVE_SEGMENTS: usize = 8;

pub(super) fn push_area(paths: Vec<(Vec<MapPoint>, bool)>, push: &mut impl FnMut(FeatureGeometry)) {
    let rings: Vec<Vec<MapPoint>> = paths
        .into_iter()
        .map(|(points, _)| points)
        .filter(|points| points.len() >= 3)
        .collect();
    if !rings.is_empty() {
        push(FeatureGeometry::Area(rings));
    }
}

/// One line feature per path; a closed path ends with its first point again, so the closing
/// edge is part of the polyline.
pub(super) fn push_lines(
    paths: Vec<(Vec<MapPoint>, bool)>,
    push: &mut impl FnMut(FeatureGeometry),
) {
    for (mut points, closed) in paths {
        if closed && let Some(first) = points.first().cloned() {
            points.push(first);
        }
        push(FeatureGeometry::Line(points));
    }
}

pub(super) fn to_point(c: &MapCoord) -> MapPoint {
    MapPoint {
        x: f64::from(c.x) / 1000.0,
        y: f64::from(c.y) / 1000.0,
    }
}

/// Splits coordinates at the hole flag into sub-paths, flattens curves, and reports whether each
/// sub-path is closed (close flag, or first point repeated). A closing duplicate is dropped.
pub(super) fn sub_paths(coords: &[MapCoord]) -> Vec<(Vec<MapPoint>, bool)> {
    let mut result = Vec::new();
    let mut start = 0;
    for (i, c) in coords.iter().enumerate() {
        if c.flags & FLAG_HOLE != 0 || i + 1 == coords.len() {
            result.push(flatten(&coords[start..=i]));
            start = i + 1;
        }
    }
    result
}

fn flatten(coords: &[MapCoord]) -> (Vec<MapPoint>, bool) {
    let mut points: Vec<MapPoint> = Vec::new();
    let mut i = 0;
    // The end point of a curve is already in `points`.
    let mut after_curve = false;
    while i < coords.len() {
        let start = to_point(&coords[i]);
        if coords[i].flags & FLAG_CURVE_START != 0 && i + 3 < coords.len() {
            let (c1, c2, end) = (
                to_point(&coords[i + 1]),
                to_point(&coords[i + 2]),
                to_point(&coords[i + 3]),
            );
            if points.is_empty() {
                points.push(start.clone());
            }
            for step in 1..=CURVE_SEGMENTS {
                points.push(bezier(
                    &start,
                    &c1,
                    &c2,
                    &end,
                    step as f64 / CURVE_SEGMENTS as f64,
                ));
            }
            i += 3;
            after_curve = true;
        } else {
            if !after_curve {
                points.push(start);
            }
            after_curve = false;
            i += 1;
        }
    }
    let flagged = coords.last().is_some_and(|c| c.flags & FLAG_CLOSE != 0);
    let repeated = points.len() > 1 && points.first() == points.last();
    if repeated {
        points.pop();
    }
    (points, flagged || repeated)
}

fn bezier(a: &MapPoint, b: &MapPoint, c: &MapPoint, d: &MapPoint, t: f64) -> MapPoint {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    MapPoint {
        x: w0 * a.x + w1 * b.x + w2 * c.x + w3 * d.x,
        y: w0 * a.y + w1 * b.y + w2 * c.y + w3 * d.y,
    }
}

pub(super) fn feature_distance(geometry: &FeatureGeometry, p: &MapPoint) -> f64 {
    match geometry {
        FeatureGeometry::Point(q) => (p.x - q.x).hypot(p.y - q.y),
        FeatureGeometry::Line(points) => distance_to_polyline(p, points, false),
        FeatureGeometry::Area(rings) => {
            if inside_area(p, rings) {
                0.0
            } else {
                rings
                    .iter()
                    .map(|ring| distance_to_polyline(p, ring, true))
                    .fold(f64::INFINITY, f64::min)
            }
        }
    }
}

fn distance_to_polyline(p: &MapPoint, points: &[MapPoint], closed: bool) -> f64 {
    if let [only] = points {
        return (p.x - only.x).hypot(p.y - only.y);
    }
    let wrap = closed.then(|| (points.last(), points.first()));
    let mut best = points
        .windows(2)
        .map(|w| distance_to_segment(p, &w[0], &w[1]))
        .fold(f64::INFINITY, f64::min);
    if let Some((Some(last), Some(first))) = wrap {
        best = best.min(distance_to_segment(p, last, first));
    }
    best
}

fn distance_to_segment(p: &MapPoint, a: &MapPoint, b: &MapPoint) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length_sq = dx * dx + dy * dy;
    let t = if length_sq == 0.0 {
        0.0
    } else {
        (((p.x - a.x) * dx + (p.y - a.y) * dy) / length_sq).clamp(0.0, 1.0)
    };
    (p.x - (a.x + t * dx)).hypot(p.y - (a.y + t * dy))
}

/// Even-odd rule over all rings, so holes are outside.
pub(super) fn inside_area(p: &MapPoint, rings: &[Vec<MapPoint>]) -> bool {
    let mut inside = false;
    for ring in rings {
        let mut previous = ring.last();
        for current in ring {
            if let Some(prev) = previous
                && (current.y > p.y) != (prev.y > p.y)
                && p.x < (prev.x - current.x) * (p.y - current.y) / (prev.y - current.y) + current.x
            {
                inside = !inside;
            }
            previous = Some(current);
        }
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve_start(x: i32, y: i32) -> MapCoord {
        MapCoord {
            x,
            y,
            flags: FLAG_CURVE_START,
        }
    }

    #[test]
    fn points_after_a_curve_are_kept() {
        let coords = vec![
            curve_start(0, 0),
            MapCoord::new(0, 5000),
            MapCoord::new(5000, 10_000),
            MapCoord::new(10_000, 10_000),
            MapCoord::new(20_000, 10_000),
            MapCoord::new(30_000, 10_000),
        ];
        let (points, closed) = flatten(&coords);
        assert!(!closed);
        assert_eq!(points.len(), 11);
        assert_eq!(points[8], MapPoint { x: 10.0, y: 10.0 });
        assert_eq!(points[9], MapPoint { x: 20.0, y: 10.0 });
        assert_eq!(points[10], MapPoint { x: 30.0, y: 10.0 });
    }

    #[test]
    fn chained_curves_flatten() {
        let coords = vec![
            curve_start(0, 0),
            MapCoord::new(0, 5000),
            MapCoord::new(5000, 10_000),
            curve_start(10_000, 10_000),
            MapCoord::new(15_000, 10_000),
            MapCoord::new(20_000, 5000),
            MapCoord::new(20_000, 0),
        ];
        let (points, _) = flatten(&coords);
        assert_eq!(points.len(), 17);
        assert_eq!(points[8], MapPoint { x: 10.0, y: 10.0 });
        assert_eq!(points[16], MapPoint { x: 20.0, y: 0.0 });
    }
}
