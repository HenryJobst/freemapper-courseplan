//! Spatial queries on an OOM map: which map features lie near a point.
//!
//! [`MapIndex`] flattens the objects of a [`MapDocument`] into points, lines and area rings in
//! millimetres (map scale, y down). It is a linear scan; there is no spatial index in M1.

use std::collections::HashMap;
use std::path::Path;

use freemapper_contract::{ObjectClass, Sidecar};
use freemapper_courseplan_model::MapPoint;

mod geometry;

use crate::omap::{MapDocument, MapObjectKind, SymbolSet};
pub(crate) use geometry::{distance_to_polyline, inside_area};
use geometry::{feature_distance, push_area, push_lines, sub_paths, to_point};

/// Kind of map, which decides the symbol norm and the check rules (glossary: Kartenart). The one
/// definition of the suite, from freemapper-core (ADR 0038).
pub use freemapper_contract::MapKind;

/// Geometry of one map feature in millimetres.
#[derive(Debug, Clone, PartialEq)]
pub enum FeatureGeometry {
    Point(MapPoint),
    Line(Vec<MapPoint>),
    /// Rings of an area: the first is the outer ring, the others are holes (even-odd rule).
    Area(Vec<Vec<MapPoint>>),
}

/// A terrain object of the map as seen by queries.
#[derive(Debug, Clone, PartialEq)]
pub struct MapFeature {
    pub symbol_code: Option<String>,
    pub part: String,
    pub geometry: FeatureGeometry,
}

/// All terrain features of a map, queryable by distance.
#[derive(Debug, Clone, Default)]
pub struct MapIndex {
    features: Vec<MapFeature>,
}

/// Scale 1:2000 or larger is a schoolyard map; ISSprOM or up to 1:5000 a sprint map; else forest.
pub fn map_kind(doc: &MapDocument) -> MapKind {
    let scale = doc.georef.scale_denominator;
    if scale <= 2000 {
        MapKind::Schoolyard
    } else if doc.symbol_set.name.contains("ISSprOM") || scale <= 5000 {
        MapKind::Sprint
    } else {
        MapKind::Forest
    }
}

/// The kind from `map_kind` in `freemapper.toml` next to `map_path`, read by the shared sidecar reader.
/// A missing or malformed file, a missing or mistyped key and an unknown value (a kind this version
/// does not know) all give `None`.
fn sidecar_kind(map_path: &Path) -> Option<MapKind> {
    Sidecar::next_to(map_path)
        .ok()
        .flatten()
        .and_then(|s| s.map_kind.known())
}

/// The kind from the symbol set name (case-insensitive) and the scale, as freemapper-print decides it.
/// `None` where the pair is ambiguous.
fn kind_from_name(doc: &MapDocument) -> Option<MapKind> {
    let name = doc.symbol_set.name.to_lowercase();
    let scale = doc.georef.scale_denominator;
    if name.contains("issprom") || name.contains("sprint") {
        return match scale {
            3000..=5000 => Some(MapKind::Sprint),
            1000 => Some(MapKind::Schoolyard),
            _ => None,
        };
    }
    if name.contains("isom") {
        return (5000..=15000).contains(&scale).then_some(MapKind::Forest);
    }
    (7500..=15000).contains(&scale).then_some(MapKind::Forest)
}

/// The kind of the map in `map_path`: `map_kind` in the `freemapper.toml` next to it first, then the
/// symbol set name and scale as freemapper-print reads them, then [`map_kind`] (ADR 0038).
pub fn map_kind_for_file(doc: &MapDocument, map_path: &Path) -> MapKind {
    sidecar_kind(map_path)
        .or_else(|| kind_from_name(doc))
        .unwrap_or_else(|| map_kind(doc))
}

/// Objects freemapper adds around the terrain: the sheet frame, the scale bar and the base area.
/// They are no map features and do not count for the map extent.
pub fn is_sheet_furniture(tags: &[(String, String)]) -> bool {
    !ObjectClass::of(tags).is_map_content()
}

/// What `from_document` needs to know about a symbol, looked up once per symbol id.
struct SymbolInfo {
    symbol_type: u32,
    hidden: bool,
    code: Option<String>,
}

impl SymbolInfo {
    fn lookup(set: &SymbolSet, id: u32) -> Option<SymbolInfo> {
        Some(SymbolInfo {
            symbol_type: set.symbol_type(id)?,
            hidden: set.is_hidden(id),
            code: set.code_of(id).map(str::to_owned),
        })
    }
}

impl MapIndex {
    pub fn from_document(doc: &MapDocument) -> MapIndex {
        let mut features = Vec::new();
        // One lookup per distinct symbol id; the symbol set scans its XML on every query.
        let mut symbols: HashMap<u32, Option<SymbolInfo>> = HashMap::new();
        for part in &doc.parts {
            for object in &part.objects {
                if is_sheet_furniture(&object.tags) {
                    continue;
                }
                let Some(symbol) = symbols
                    .entry(object.symbol_id)
                    .or_insert_with(|| SymbolInfo::lookup(&doc.symbol_set, object.symbol_id))
                else {
                    continue;
                };
                if symbol.hidden {
                    continue;
                }
                let (symbol_type, symbol_code) = (symbol.symbol_type, symbol.code.clone());
                let mut push = |geometry| {
                    features.push(MapFeature {
                        symbol_code: symbol_code.clone(),
                        part: part.name.clone(),
                        geometry,
                    });
                };
                if matches!(object.kind, MapObjectKind::Text { .. }) {
                    continue;
                }
                let paths = sub_paths(&object.coords);
                match symbol_type {
                    1 => {
                        if let Some(first) = object.coords.first() {
                            push(FeatureGeometry::Point(to_point(first)));
                        }
                    }
                    2 => push_lines(paths, &mut push),
                    4 => push_area(paths, &mut push),
                    // Combined symbols: decided per path. Closed paths form one area (first
                    // ring outer, the rest holes); open paths are lines, one feature each.
                    16 => {
                        let (closed, open): (Vec<_>, Vec<_>) =
                            paths.into_iter().partition(|(_, closed)| *closed);
                        push_area(closed, &mut push);
                        push_lines(open, &mut push);
                    }
                    _ => {}
                }
            }
        }
        MapIndex { features }
    }

    /// Features within `radius_mm` of `p`, nearest first, with their distance (0 inside an area).
    pub fn features_near(&self, p: MapPoint, radius_mm: f64) -> Vec<(&MapFeature, f64)> {
        let mut near: Vec<(&MapFeature, f64)> = self
            .features
            .iter()
            .map(|f| (f, feature_distance(&f.geometry, &p)))
            .filter(|(_, d)| *d <= radius_mm)
            .collect();
        near.sort_by(|a, b| a.1.total_cmp(&b.1));
        near
    }

    /// Smallest and largest corner over all feature points.
    pub fn extent(&self) -> Option<(MapPoint, MapPoint)> {
        let mut points = self.features.iter().flat_map(|f| match &f.geometry {
            FeatureGeometry::Point(p) => vec![p.clone()],
            FeatureGeometry::Line(l) => l.clone(),
            FeatureGeometry::Area(rings) => rings.concat(),
        });
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

    /// An index over given features, for tests that build a map by hand.
    #[cfg(test)]
    pub(crate) fn from_features(features: Vec<MapFeature>) -> MapIndex {
        MapIndex { features }
    }

    /// Every feature, in map order.
    pub fn features(&self) -> &[MapFeature] {
        &self.features
    }

    pub fn len(&self) -> usize {
        self.features.len()
    }

    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::omap::{
        MapCoord, MapGeoref, MapObject, MapObjectKind, MapPart, SymbolSet, closed_ring,
    };

    const SYMBOLS: &str = r#"<map><georeferencing scale="4000"/><colors count="0"></colors><symbols count="6" id="ISSprOM 2019"><symbol type="1" id="0" code="101" name="pt"><point_symbol/></symbol><symbol type="2" id="1" code="502" name="ln"><line_symbol/></symbol><symbol type="4" id="2" code="520" name="ar"><area_symbol/></symbol><symbol type="2" id="3" code="999" name="hid" is_hidden="true"><line_symbol/></symbol><symbol type="8" id="4" code="700" name="txt"><text_symbol/></symbol><symbol type="16" id="5" code="501" name="comb"><combined_symbol/></symbol></symbols><view><grid/></view></map>"#;

    fn p(x: f64, y: f64) -> MapPoint {
        MapPoint { x, y }
    }

    fn path(symbol_id: u32, coords: Vec<MapCoord>) -> MapObject {
        MapObject {
            symbol_id,
            kind: MapObjectKind::Path,
            coords,
            tags: vec![],
            pattern: None,
        }
    }

    fn document(scale: u32, name: &str, objects: Vec<MapObject>) -> MapDocument {
        let mut symbol_set = SymbolSet::from_omap_xml(SYMBOLS).unwrap();
        symbol_set.name = name.into();
        MapDocument {
            notes: String::new(),
            georef: MapGeoref {
                scale_denominator: scale,
                declination_deg: 0.0,
                grivation_deg: 0.0,
                ref_projected: None,
                crs_spec: None,
                combined_scale_factor: 1.0,
                map_ref_mm: (0.0, 0.0),
            },
            symbol_set,
            parts: vec![MapPart {
                name: "p".into(),
                objects,
            }],
            templates: vec![],
            unknown_objects: 0,
        }
    }

    fn square(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<MapCoord> {
        closed_ring(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
    }

    #[test]
    fn distance_to_point() {
        let doc = document(
            4000,
            "X",
            vec![MapObject {
                symbol_id: 0,
                kind: MapObjectKind::Point { rotation_rad: 0.0 },
                coords: vec![MapCoord::new(3000, 4000)],
                tags: vec![],
                pattern: None,
            }],
        );
        let index = MapIndex::from_document(&doc);
        let near = index.features_near(p(0.0, 0.0), 10.0);
        assert_eq!(near.len(), 1);
        assert!((near[0].1 - 5.0).abs() < 1e-9);
        assert_eq!(near[0].0.symbol_code.as_deref(), Some("101"));
        assert!(index.features_near(p(0.0, 0.0), 4.9).is_empty());
    }

    #[test]
    fn distance_to_line_segment() {
        let line = path(1, vec![MapCoord::new(0, 0), MapCoord::new(10_000, 0)]);
        let index = MapIndex::from_document(&document(4000, "X", vec![line]));
        let mid = index.features_near(p(5.0, 3.0), 10.0);
        assert!((mid[0].1 - 3.0).abs() < 1e-9);
        // Beyond the end the distance is to the end point, not to the infinite line.
        let past = index.features_near(p(13.0, 4.0), 10.0);
        assert!((past[0].1 - 5.0).abs() < 1e-9);
    }

    #[test]
    fn inside_area_is_zero() {
        let area = path(2, square(0, 0, 10_000, 10_000));
        let index = MapIndex::from_document(&document(4000, "X", vec![area]));
        assert_eq!(index.features_near(p(5.0, 5.0), 1.0)[0].1, 0.0);
        let outside = index.features_near(p(12.0, 5.0), 5.0);
        assert!((outside[0].1 - 2.0).abs() < 1e-9);
    }

    #[test]
    fn inside_hole_is_not_zero() {
        let mut coords = square(0, 0, 10_000, 10_000);
        coords.extend(square(4000, 4000, 6000, 6000));
        let index = MapIndex::from_document(&document(4000, "X", vec![path(2, coords)]));
        let in_hole = index.features_near(p(5.0, 5.0), 5.0);
        assert!((in_hole[0].1 - 1.0).abs() < 1e-9);
        assert_eq!(index.features_near(p(2.0, 5.0), 1.0)[0].1, 0.0);
    }

    #[test]
    fn curve_is_flattened() {
        // Quarter circle of radius 10 mm from (10, 0) to (0, 10) around the origin.
        let k = (0.552_284_749_8 * 10_000.0) as i32;
        let coords = vec![
            MapCoord {
                x: 10_000,
                y: 0,
                flags: 1,
            },
            MapCoord::new(10_000, k),
            MapCoord::new(k, 10_000),
            MapCoord::new(0, 10_000),
        ];
        let index = MapIndex::from_document(&document(4000, "X", vec![path(1, coords)]));
        let FeatureGeometry::Line(points) = &index.features[0].geometry else {
            panic!("expected a line");
        };
        assert_eq!(points.len(), 9);
        for q in points {
            let radius = (q.x * q.x + q.y * q.y).sqrt();
            assert!((radius - 10.0).abs() < 0.05, "{q:?} radius {radius}");
        }
    }

    #[test]
    fn features_near_sorted_and_bounded() {
        let objects = vec![
            path(1, vec![MapCoord::new(0, 8000), MapCoord::new(10_000, 8000)]),
            path(1, vec![MapCoord::new(0, 2000), MapCoord::new(10_000, 2000)]),
            path(
                1,
                vec![MapCoord::new(0, 30_000), MapCoord::new(10_000, 30_000)],
            ),
        ];
        let index = MapIndex::from_document(&document(4000, "X", objects));
        let near = index.features_near(p(5.0, 0.0), 10.0);
        let distances: Vec<f64> = near.iter().map(|(_, d)| *d).collect();
        assert_eq!(distances.len(), 2);
        assert!((distances[0] - 2.0).abs() < 1e-9);
        assert!((distances[1] - 8.0).abs() < 1e-9);
    }

    #[test]
    fn skips_hidden_text_and_sheet_objects() {
        let mut sheet = path(1, vec![MapCoord::new(0, 0), MapCoord::new(1000, 0)]);
        sheet.tags = vec![("freemapper:source".into(), "sheet".into())];
        let objects = vec![
            path(3, vec![MapCoord::new(0, 0), MapCoord::new(1000, 0)]),
            MapObject {
                symbol_id: 4,
                kind: MapObjectKind::Text {
                    text: "t".into(),
                    h_align: 0,
                    v_align: 0,
                    rotation_rad: 0.0,
                    box_size: None,
                },
                coords: vec![MapCoord::new(0, 0)],
                tags: vec![],
                pattern: None,
            },
            sheet,
            path(1, vec![MapCoord::new(0, 0), MapCoord::new(1000, 0)]),
        ];
        let index = MapIndex::from_document(&document(4000, "X", objects));
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn combined_symbol_is_line_when_open_and_area_when_closed() {
        let open = path(5, vec![MapCoord::new(0, 0), MapCoord::new(1000, 0)]);
        let closed = path(5, square(0, 0, 1000, 1000));
        let index = MapIndex::from_document(&document(4000, "X", vec![open, closed]));
        assert!(matches!(
            index.features[0].geometry,
            FeatureGeometry::Line(_)
        ));
        assert!(matches!(
            index.features[1].geometry,
            FeatureGeometry::Area(_)
        ));
    }

    #[test]
    fn closed_line_measures_closing_edge() {
        // Square 0..10 mm; the closing edge runs from (0, 10) back to (0, 0).
        let index = MapIndex::from_document(&document(
            4000,
            "X",
            vec![path(1, square(0, 0, 10_000, 10_000))],
        ));
        assert!(matches!(
            index.features[0].geometry,
            FeatureGeometry::Line(_)
        ));
        let near = index.features_near(p(-0.5, 5.0), 5.0);
        assert!((near[0].1 - 0.5).abs() < 1e-9, "{}", near[0].1);
    }

    #[test]
    fn extent_covers_all_points() {
        let objects = vec![path(
            1,
            vec![MapCoord::new(-2000, 3000), MapCoord::new(5000, 9000)],
        )];
        let index = MapIndex::from_document(&document(4000, "X", objects));
        let (min, max) = index.extent().unwrap();
        assert_eq!((min.x, min.y, max.x, max.y), (-2.0, 3.0, 5.0, 9.0));
        assert!(MapIndex::default().extent().is_none());
    }

    #[test]
    fn kinds_by_scale_and_symbol_set() {
        assert_eq!(
            map_kind(&document(1000, "ISSprOM 2019", vec![])),
            MapKind::Schoolyard
        );
        assert_eq!(
            map_kind(&document(4000, "ISSprOM 2019", vec![])),
            MapKind::Sprint
        );
        assert_eq!(
            map_kind(&document(10000, "ISSprOM 2019", vec![])),
            MapKind::Sprint
        );
        assert_eq!(
            map_kind(&document(4000, "ISOM 2017", vec![])),
            MapKind::Sprint
        );
        assert_eq!(
            map_kind(&document(10000, "ISOM 2017", vec![])),
            MapKind::Forest
        );
    }

    fn tagged(mut object: MapObject, source: &str) -> MapObject {
        object.tags = vec![("freemapper:source".into(), source.into())];
        object
    }

    #[test]
    fn scale_bar_and_base_are_no_features_and_keep_the_extent() {
        let terrain = path(1, vec![MapCoord::new(0, 0), MapCoord::new(10_000, 10_000)]);
        let bar = tagged(
            path(
                1,
                vec![MapCoord::new(0, 90_000), MapCoord::new(5000, 90_000)],
            ),
            "scale-bar",
        );
        let base = tagged(path(2, square(-5000, -5000, 100_000, 100_000)), "base");
        let index = MapIndex::from_document(&document(4000, "X", vec![terrain, bar, base]));
        assert_eq!(index.len(), 1);
        let (min, max) = index.extent().unwrap();
        assert_eq!((min.x, min.y, max.x, max.y), (0.0, 0.0, 10.0, 10.0));
    }

    fn with_sidecar(text: Option<&str>) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        if let Some(text) = text {
            std::fs::write(dir.path().join("freemapper.toml"), text).unwrap();
        }
        let map = dir.path().join("m.omap");
        (dir, map)
    }

    #[test]
    fn sidecar_decides_first() {
        let doc = document(2500, "ISSprOM 2019", vec![]);
        let (_d, map) = with_sidecar(Some("map_kind = \"schoolyard\"\n"));
        assert_eq!(map_kind_for_file(&doc, &map), MapKind::Schoolyard);
        assert_eq!(map_kind(&doc), MapKind::Sprint);
    }

    #[test]
    fn unknown_or_broken_sidecar_falls_through() {
        let doc = document(4000, "ISSprOM 2019", vec![]);
        for text in [
            Some("map_kind = \"gym\"\n"),
            Some("map_kind = [\n"),
            Some("title = \"x\"\n"),
            None,
        ] {
            let (_d, map) = with_sidecar(text);
            assert_eq!(map_kind_for_file(&doc, &map), MapKind::Sprint, "{text:?}");
        }
    }

    #[test]
    fn kind_names_and_order_are_pinned() {
        // Project files, MCP and CLI JSON and the Fluent texts spell the kinds like this.
        let names: Vec<&str> = MapKind::ALL.iter().map(|k| k.name()).collect();
        assert_eq!(names, ["sprint", "schoolyard", "forest"]);
        for kind in MapKind::ALL {
            #[derive(serde::Serialize)]
            struct Wrap {
                k: MapKind,
            }
            assert_eq!(
                toml::to_string(&Wrap { k: kind }).unwrap(),
                format!("k = \"{}\"\n", kind.name())
            );
            assert_eq!(MapKind::from_name(kind.name()), Some(kind));
        }
    }

    #[test]
    fn mistyped_sidecar_kind_counts_as_absent() {
        let doc = document(4000, "ISSprOM 2019", vec![]);
        let (_d, map) = with_sidecar(Some("map_kind = 3\n"));
        assert_eq!(map_kind_for_file(&doc, &map), MapKind::Sprint);
    }

    #[test]
    fn set_name_is_case_insensitive() {
        let (_d, map) = with_sidecar(None);
        assert_eq!(
            map_kind_for_file(&document(4000, "issprom 2019", vec![]), &map),
            MapKind::Sprint
        );
        assert_eq!(
            map_kind_for_file(&document(10000, "isom 2017", vec![]), &map),
            MapKind::Forest
        );
        assert_eq!(
            map_kind_for_file(&document(10000, "Custom", vec![]), &map),
            MapKind::Forest
        );
        // Print leaves ISOM at 1:4000 unknown; the old thresholds decide.
        assert_eq!(
            map_kind_for_file(&document(4000, "ISOM 2017", vec![]), &map),
            MapKind::Sprint
        );
    }

    #[test]
    fn symbol_table_matches_direct_lookups() {
        // Symbols: id 3 hidden; id 9 missing; ids 0 and 6 share code 101.
        let xml = SYMBOLS.replace(
            "</symbols>",
            r#"<symbol type="1" id="6" code="101" name="pt2"><point_symbol/></symbol></symbols>"#,
        );
        let mut doc = document(4000, "X", vec![]);
        doc.symbol_set = SymbolSet::from_omap_xml(&xml).unwrap();
        let line = |id| path(id, vec![MapCoord::new(0, 0), MapCoord::new(1000, 0)]);
        let pt = |id| MapObject {
            symbol_id: id,
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![MapCoord::new(1000, 2000)],
            tags: vec![],
            pattern: None,
        };
        let objects = vec![
            line(1),
            line(3),
            line(9),
            pt(0),
            pt(6),
            pt(0),
            path(2, square(0, 0, 1000, 1000)),
            line(1),
        ];
        doc.parts[0].objects = objects.clone();
        let index = MapIndex::from_document(&doc);

        let mut expected = Vec::new();
        for o in &objects {
            let set = &doc.symbol_set;
            if set.is_hidden(o.symbol_id) {
                continue;
            }
            let Some(ty) = set.symbol_type(o.symbol_id) else {
                continue;
            };
            let geometry = match ty {
                1 => FeatureGeometry::Point(to_point(&o.coords[0])),
                2 => FeatureGeometry::Line(sub_paths(&o.coords).remove(0).0),
                4 => FeatureGeometry::Area(vec![sub_paths(&o.coords).remove(0).0]),
                _ => continue,
            };
            expected.push(MapFeature {
                symbol_code: set.code_of(o.symbol_id).map(str::to_owned),
                part: "p".into(),
                geometry,
            });
        }
        assert_eq!(expected.len(), 6);
        assert_eq!(index.features, expected);
    }
}
