//! `inside_map`: a start, finish, control or crossing point must lie on the mapped area (ADR 0084).
//!
//! `map_edge` measures against the bounding box of the map objects. A map is rarely a rectangle: a
//! diagonal strip of terrain leaves blank corners inside that box, and a position there passes
//! `map_edge`. This rule measures against the map objects themselves. A position is inside when a
//! map object lies within `metres` of it (0 inside an area). Not counted as objects:
//!
//! - the sheet furniture, which the map index leaves out anyway;
//! - the symbols of the rule value `ignore_symbols`, which are layout and no terrain, such as the
//!   north lines pattern (601) that mappers lay over the whole print area;
//! - a line that is an axis-aligned rectangle and spans at least `frame_percent` percent of the extent in both
//!   directions, which is the frame around the sheet. The extent is that of the objects not ignored.
//!
//! Which stops: the start, controls, crossing points and finish of the rule's individual courses,
//! and the start, controls and finish of every score course. Controls on no course are not tested,
//! like the other control checks.

use freemapper_courseplan_model::MapPoint;

use super::stops::{Stop, StopKind, stops_of};
use super::{
    ArgValue, CheckContext, Finding, Location, RuleRun, SKIP_NO_MAP_EXTENT, Skipped, number,
    symbol_matches, value_number,
};
use crate::map::{FeatureGeometry, MapFeature, MapIndex, MapKind};
use freemapper_courseplan_model::ControlCode;

fn ignored_symbols(run: &RuleRun<'_>) -> Vec<String> {
    run.rule()
        .values
        .get("ignore_symbols")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn bounds(points: impl Iterator<Item = MapPoint>) -> Option<(MapPoint, MapPoint)> {
    let mut points = points;
    let first = points.next()?;
    let (mut min, mut max) = (first.clone(), first);
    for p in points {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }
    Some((min, max))
}

/// Symbols of the profile value `ignore_symbols`: layout objects, no terrain. The score draft reads the
/// same list through [`terrain_extent`], and a test compares both with the rule `wkb2026.map.inside`.
pub(crate) const IGNORED_SYMBOLS: &[&str] = &["601", "602"];

/// The profile value `frame_percent`: a frame outline spans at least this percentage of the terrain extent in
/// both directions. Compared with the rule by a test, like [`IGNORED_SYMBOLS`].
pub(crate) const FRAME_PERCENT: f64 = 80.0;

/// Whether the outline is a sheet frame: a plain axis-aligned rectangle of four corners (a line may end
/// with its first point again) that spans at least `percent` of the extent in both directions. Its
/// corners lie on the edge of the sheet, not at terrain (ADR 0078, ADR 0084).
pub(crate) fn is_frame_outline(
    points: &[MapPoint],
    extent: &(MapPoint, MapPoint),
    percent: f64,
) -> bool {
    const FLAT_MM: f64 = 0.01;
    let mut pts = points.to_vec();
    if pts.len() > 1 && pts.first() == pts.last() {
        pts.pop();
    }
    if pts.len() != 4 {
        return false;
    }
    let axis_aligned = (0..4).all(|i| {
        let (a, b) = (&pts[i], &pts[(i + 1) % 4]);
        (a.x - b.x).abs() <= FLAT_MM || (a.y - b.y).abs() <= FLAT_MM
    });
    let Some((lo, hi)) = bounds(pts.iter().cloned()) else {
        return false;
    };
    let share = percent / 100.0;
    axis_aligned
        && (hi.x - lo.x) >= share * (extent.1.x - extent.0.x)
        && (hi.y - lo.y) >= share * (extent.1.y - extent.0.y)
}

/// The extent of the map objects that are no layout: all but the sheet furniture (which the index leaves
/// out) and the ignored symbols. A stray object or a pattern area larger than the sheet makes the extent of
/// all objects too large to find the frame by.
pub(crate) fn terrain_extent(index: &MapIndex, ignored: &[&str]) -> Option<(MapPoint, MapPoint)> {
    let kept: Vec<&MapFeature> = index
        .features()
        .iter()
        .filter(|f| {
            !f.symbol_code
                .as_deref()
                .is_some_and(|c| ignored.iter().any(|i| symbol_matches(c, i)))
        })
        .collect();
    extent_of(&kept)
}

/// The extent of all points of these features.
fn extent_of(features: &[&MapFeature]) -> Option<(MapPoint, MapPoint)> {
    bounds(features.iter().flat_map(|f| match &f.geometry {
        FeatureGeometry::Point(p) => vec![p.clone()],
        FeatureGeometry::Line(l) => l.clone(),
        FeatureGeometry::Area(rings) => rings.concat(),
    }))
}

/// The map objects that count as terrain: everything but the ignored symbols and the frame. The
/// frame is found against the extent of the objects that are not ignored, since an ignored area
/// such as the north lines pattern often reaches beyond the sheet.
pub(crate) fn terrain_features<'i, S: AsRef<str>>(
    index: &'i MapIndex,
    ignored: &[S],
    percent: f64,
) -> Vec<&'i MapFeature> {
    let kept: Vec<&MapFeature> = index
        .features()
        .iter()
        .filter(|f| {
            !f.symbol_code
                .as_deref()
                .is_some_and(|c| ignored.iter().any(|i| symbol_matches(c, i.as_ref())))
        })
        .collect();
    let Some(extent) = extent_of(&kept) else {
        return Vec::new();
    };
    kept.into_iter()
        .filter(|f| match &f.geometry {
            FeatureGeometry::Line(points) => !is_frame_outline(points, &extent, percent),
            _ => true,
        })
        .collect()
}

fn content<'i>(ctx: &'i CheckContext<'_>, ignored: &[String], percent: f64) -> Vec<&'i MapFeature> {
    terrain_features(&ctx.index, ignored, percent)
}

/// The nearest of these objects to `at`, with its distance in mm.
fn nearest<'i>(content: &[&'i MapFeature], at: &MapPoint) -> Option<(&'i MapFeature, f64)> {
    content
        .iter()
        .map(|f| (*f, MapIndex::distance_to(f, at)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

/// The stops to test, each once: individual courses first, then the score courses.
fn stops_to_test(ctx: &CheckContext<'_>, run: &RuleRun<'_>) -> Vec<Stop> {
    let mut stops = stops_of(ctx, run.courses.iter().copied(), |_| true);
    let mut add = |id: &str, kind: StopKind, at: &MapPoint| {
        if !stops.iter().any(|s| s.id == id) {
            stops.push(Stop {
                id: id.to_owned(),
                kind,
                at: at.clone(),
            });
        }
    };
    for score in &ctx.score_courses {
        add(&score.course.start, StopKind::Start, &score.start);
        for control in &score.controls {
            add(&control.code.0, StopKind::Control, &control.at);
        }
        add(&score.course.finish, StopKind::Finish, &score.finish);
    }
    stops
}

/// The reach of `inside_map` in ground metres on this map: the rule value `metres`, or on a forest map the
/// sheet distance `forest_sheet_mm` at the scale of the map (ADR 0092). `None` when the rule lacks the value.
fn reach_m(ctx: &CheckContext<'_>, run: &RuleRun<'_>) -> Option<f64> {
    if ctx.map_kind == MapKind::Forest
        && let Some(mm) = value_number(run.rule(), "forest_sheet_mm")
    {
        return Some(mm * f64::from(ctx.map_scale) / 1000.0);
    }
    value_number(run.rule(), "metres")
}

pub(super) fn inside_map(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(limit) = reach_m(ctx, run) else {
        number(run, "metres", skipped);
        return;
    };
    let Some(percent) = number(run, "frame_percent", skipped) else {
        return;
    };
    let content = content(ctx, &ignored_symbols(run), percent);
    if content.is_empty() {
        // No objects, or only ignored ones: nothing is mapped, so nothing can be inside.
        skipped.push(run.skipped(Location::Event, SKIP_NO_MAP_EXTENT, Vec::new()));
        return;
    }
    for stop in stops_to_test(ctx, run) {
        let Some((feature, mm)) = nearest(&content, &stop.at) else {
            continue;
        };
        let metres = ctx.metres(mm);
        if metres > limit {
            findings.push(run.finding(
                Location::Control {
                    code: ControlCode(stop.id.clone()),
                },
                "check-inside-map",
                vec![
                    ("id", ArgValue::Text(stop.id.clone())),
                    ("kind", ArgValue::Text(stop.kind.word().to_owned())),
                    ("x", ArgValue::Number(stop.at.x)),
                    ("y", ArgValue::Number(stop.at.y)),
                    (
                        "symbol",
                        ArgValue::Text(feature.symbol_code.clone().unwrap_or_else(|| "?".into())),
                    ),
                    ("metres", ArgValue::Int(metres.floor() as i64)),
                    ("limit", ArgValue::Number(limit)),
                ],
            ));
        }
    }
}

/// `map_margin`: a start, finish, control or crossing point closer than `map_margin_m` of the event settings
/// to the edge of the mapped area (ADR 0087). The edge is where no map object lies within the `metres` of
/// `inside_map` any more; the distance is measured on a raster of the mapped area, to about 2 m. A position in
/// the blank part is the business of `inside_map` and gets no second finding here.
pub(super) fn map_margin(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let limit = ctx.event.score_settings().map_margin_m;
    if limit == 0 {
        return;
    }
    // The reach of `inside_map` on this map: the same edge as that rule. The margin rule carries only
    // the forest sheet distance; a sprint or schoolyard map keeps the 50 m of the mapped area.
    let reach = reach_m(ctx, run).unwrap_or(crate::mapped::MAPPED_METRES);
    let Some(area) = crate::mapped::MappedArea::build_with_reach(&ctx.index, ctx.map_scale, reach)
    else {
        skipped.push(run.skipped(Location::Event, SKIP_NO_MAP_EXTENT, Vec::new()));
        return;
    };
    for stop in stops_to_test(ctx, run) {
        let margin = area.margin_m(&stop.at);
        if margin > 0.0 && margin < f64::from(limit) {
            findings.push(run.finding(
                Location::Control {
                    code: ControlCode(stop.id.clone()),
                },
                "check-map-margin",
                vec![
                    ("id", ArgValue::Text(stop.id.clone())),
                    ("kind", ArgValue::Text(stop.kind.word().to_owned())),
                    ("x", ArgValue::Number(stop.at.x)),
                    ("y", ArgValue::Number(stop.at.y)),
                    ("metres", ArgValue::Int(margin.floor() as i64)),
                    ("limit", ArgValue::Int(i64::from(limit))),
                ],
            ));
        }
    }
}

#[cfg(test)]
mod tests;
