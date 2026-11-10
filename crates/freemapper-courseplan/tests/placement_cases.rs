//! The shared number placement cases (`tests/fixtures/placement/cases.toml`, a byte-for-byte copy of
//! freemapper-core's file; the format is described in its header) run through courseplan's own
//! `numbers::auto_place`.
//!
//! `PLACEMENT_CASES_DUMP=<file>` writes the results of every case instead of asserting; the same variable
//! on `example_auto_placements_are_unchanged` writes `fixtures/placement/luebeck-sprint-auto.toml`.

use std::path::Path;

use freemapper_courseplan::map::{MapIndex, MapKind, map_kind};
use freemapper_courseplan::numbers::{GapRange, auto_place};
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::omap::{
    MapCoord, MapDocument, MapGeoref, MapObject, MapObjectKind, MapPart, SymbolSet,
};
use freemapper_courseplan_model::{
    Control, ControlCode, ControlDescription, ControlKind, Course, CourseControl, CourseKind,
    Event, EventMeta, FORMAT_VERSION, MapPoint, NumberPlacement, ProjectFile, Sources,
    SpecialObject,
};
use serde::Deserialize;

const CASES: &str = include_str!("fixtures/placement/cases.toml");
const TOLERANCE: f64 = 1e-9;

/// Cases whose expected values the code is known not to reproduce, each with the reason. The harness
/// still runs them and requires that they differ. Empty since the switch to freemapper-course (ADR 0036):
/// the four cases of core ADR 0016 (D2, D4) pass.
const KNOWN_DISAGREEMENTS: &[(&str, &str)] = &[];

type Pt = (f64, f64);

#[derive(Deserialize)]
struct File {
    format: u32,
    case: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    map_kind: String,
    map_scale: u32,
    start: Pt,
    finish: Pt,
    #[serde(default)]
    start_bends: Vec<Pt>,
    gap_min: Option<f64>,
    gap_max: Option<f64>,
    #[serde(default)]
    stops: Vec<StopSpec>,
    #[serde(default)]
    map_points: Vec<Pt>,
    #[serde(default)]
    map_lines: Vec<Vec<Pt>>,
    expected: Option<Vec<Expected>>,
}

#[derive(Deserialize)]
struct StopSpec {
    at: Pt,
    #[serde(default)]
    crossing: bool,
    kept: Option<Spot>,
    #[serde(default)]
    bends: Vec<Pt>,
    repeat: Option<usize>,
    step: Option<Pt>,
}

#[derive(Deserialize, Clone, Copy)]
struct Spot {
    direction: f64,
    gap: f64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Expected {
    Word(String),
    At(Spot),
}

/// One stop after the expansion of `repeat`.
struct Stop {
    at: Pt,
    crossing: bool,
    kept: Option<Spot>,
    bends: Vec<Pt>,
}

fn expand(case: &Case) -> Vec<Stop> {
    let mut stops = Vec::new();
    for spec in &case.stops {
        let n = spec.repeat.unwrap_or(1);
        assert!(n >= 1, "{}: repeat must be at least 1", case.name);
        if n > 1 {
            assert!(
                spec.kept.is_none() && spec.bends.is_empty(),
                "{}: kept and bends do not combine with repeat",
                case.name
            );
        }
        let step = spec.step.unwrap_or((0.0, 0.0));
        for i in 0..n {
            let f = i as f64;
            stops.push(Stop {
                at: (spec.at.0 + f * step.0, spec.at.1 + f * step.1),
                crossing: spec.crossing,
                kept: spec.kept,
                bends: spec.bends.clone(),
            });
        }
    }
    stops
}

const SYMBOLS: &str = r#"<map><georeferencing scale="4000"/><colors count="0"></colors><symbols count="2" id="ISSprOM 2019"><symbol type="1" id="0" code="999" name="point"><point_symbol/></symbol><symbol type="2" id="1" code="999" name="line"><line_symbol/></symbol></symbols><view><grid/></view></map>"#;
const POINT_SYMBOL: u32 = 0;
const LINE_SYMBOL: u32 = 1;

fn coord(p: Pt) -> MapCoord {
    MapCoord::new((p.0 * 1000.0).round() as i32, (p.1 * 1000.0).round() as i32)
}

fn object(symbol_id: u32, kind: MapObjectKind, points: &[Pt]) -> MapObject {
    MapObject {
        symbol_id,
        kind,
        coords: points.iter().map(|p| coord(*p)).collect(),
        tags: vec![],
        pattern: None,
    }
}

/// A sprint-symbol map document with `points` (point objects) and `lines` (open polylines).
fn map_document(scale: u32, points: &[Pt], lines: &[Vec<Pt>]) -> MapDocument {
    let mut objects = Vec::new();
    for p in points {
        objects.push(object(
            POINT_SYMBOL,
            MapObjectKind::Point { rotation_rad: 0.0 },
            &[*p],
        ));
    }
    for line in lines {
        objects.push(object(LINE_SYMBOL, MapObjectKind::Path, line));
    }
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
        symbol_set: SymbolSet::from_omap_xml(SYMBOLS).expect("symbols parse"),
        parts: vec![MapPart {
            name: "map".into(),
            objects,
        }],
        templates: vec![],
        unknown_objects: 0,
    }
}

fn mp(p: Pt) -> MapPoint {
    MapPoint { x: p.0, y: p.1 }
}

/// What `gap_range` makes of one `number_gap` rule with `min_gap_mm` / `max_gap_mm`.
fn gap_range_of_one_rule(min: Option<f64>, max: Option<f64>) -> GapRange {
    match GapRange::strictest([(min, max)]) {
        Some((min, max)) => GapRange::from_limits(min, max),
        None => GapRange::DEFAULT,
    }
}

fn event_of(stops: &[Stop], start: Pt, finish: Pt, start_bends: &[Pt]) -> Event {
    let code = |i: usize| ControlCode(format!("{}", 31 + i));
    Event {
        format: FORMAT_VERSION,
        event: EventMeta {
            title: "cases".to_owned(),
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
            .enumerate()
            .map(|(i, s)| Control {
                ppen_id: None,
                code: code(i),
                kind: if s.crossing {
                    ControlKind::CrossingPoint
                } else {
                    ControlKind::Control
                },
                map_mm: mp(s.at),
                description: ControlDescription::default(),
                number: None,
            })
            .collect(),
        specials: vec![
            SpecialObject::Start {
                id: "S1".to_owned(),
                ppen_id: None,
                map_mm: mp(start),
            },
            SpecialObject::Finish {
                id: "F1".to_owned(),
                ppen_id: None,
                map_mm: mp(finish),
            },
        ],
        courses: vec![Course {
            ppen_id: None,
            name: "A".to_owned(),
            classes: vec![],
            start: "S1".to_owned(),
            finish: "F1".to_owned(),
            start_bends: start_bends.iter().map(|p| mp(*p)).collect(),
            start_taped: false,
            print_area: None,
            climb_m: None,
            planned_print_scale: None,
            kind: CourseKind::Individual {
                controls: stops
                    .iter()
                    .enumerate()
                    .map(|(i, s)| CourseControl {
                        code: code(i),
                        number: s.kept.map(|k| NumberPlacement {
                            direction: k.direction,
                            gap_mm: k.gap,
                        }),
                        bends: s.bends.iter().map(|p| mp(*p)).collect(),
                        taped: false,
                    })
                    .collect(),
            },
        }],
        accepted_findings: vec![],
    }
}

fn kind_of(name: &str) -> MapKind {
    match name {
        "sprint" => MapKind::Sprint,
        "forest" => MapKind::Forest,
        other => panic!("unknown map_kind {other}"),
    }
}

/// The placements of a case, one per stop (crossing points `None`).
fn run(case: &Case) -> Vec<Option<NumberPlacement>> {
    let stops = expand(case);
    let event = event_of(&stops, case.start, case.finish, &case.start_bends);
    let index = MapIndex::from_document(&map_document(
        case.map_scale,
        &case.map_points,
        &case.map_lines,
    ));
    let gaps = gap_range_of_one_rule(case.gap_min, case.gap_max);
    let placed = auto_place(
        &event,
        &event.courses[0],
        &index,
        case.map_scale,
        kind_of(&case.map_kind),
        gaps,
        true,
    )
    .expect("auto_place");
    // auto_place returns the numbered controls only; crossing points get None. A wrong count means
    // auto_place numbered a crossing point or skipped a control.
    assert_eq!(
        placed.len(),
        stops.iter().filter(|s| !s.crossing).count(),
        "{}: one placement per numbered stop",
        case.name
    );
    let mut numbered = placed.into_iter();
    stops
        .iter()
        .map(|s| if s.crossing { None } else { numbered.next() })
        .collect()
}

fn load() -> File {
    let file: File = toml::from_str(CASES).expect("cases.toml parses");
    assert_eq!(file.format, 1, "unknown case format");
    file
}

fn spot_text(p: &Option<NumberPlacement>) -> String {
    match p {
        Some(p) => format!("{{ direction = {:?}, gap = {:?} }}", p.direction, p.gap_mm),
        None => "\"none\"".to_string(),
    }
}

/// Differences between the result of a case and its expected values, one text per differing stop.
fn differences(case: &Case, expected: &[Expected]) -> Vec<String> {
    let got = run(case);
    assert_eq!(
        got.len(),
        expected.len(),
        "{}: one expected entry per stop",
        case.name
    );
    let mut out = Vec::new();
    for (k, (g, e)) in got.iter().zip(expected).enumerate() {
        match (g, e) {
            (None, Expected::Word(w)) if w == "none" => {}
            (Some(g), Expected::At(e)) => {
                if (g.direction - e.direction).abs() > TOLERANCE
                    || (g.gap_mm - e.gap).abs() > TOLERANCE
                {
                    out.push(format!(
                        "{} stop {k}: got direction {} gap {}, expected {} {}",
                        case.name, g.direction, g.gap_mm, e.direction, e.gap
                    ));
                }
            }
            _ => out.push(format!(
                "{} stop {k}: number presence differs from expected",
                case.name
            )),
        }
    }
    out
}

#[test]
fn placement_cases_match_expected() {
    let file = load();
    if let Ok(path) = std::env::var("PLACEMENT_CASES_DUMP") {
        let mut text = String::from(
            "# Results of courseplan's auto_place per case; one entry per stop.\nformat = 1\n",
        );
        for case in &file.case {
            let placements: Vec<String> = run(case).iter().map(spot_text).collect();
            text.push_str(&format!(
                "\n[[result]]\nname = {:?}\nplacements = [{}]\n",
                case.name,
                placements.join(", ")
            ));
        }
        std::fs::write(&path, text).expect("dump written");
        return;
    }
    for case in &file.case {
        let Some(expected) = &case.expected else {
            panic!("{}: no expected values", case.name);
        };
        let diff = differences(case, expected);
        let known = KNOWN_DISAGREEMENTS.iter().find(|(n, _)| *n == case.name);
        match (diff.is_empty(), known) {
            (true, None) | (false, Some(_)) => {}
            (false, None) => panic!("{}", diff.join("\n")),
            (true, Some((name, reason))) => {
                panic!(
                    "{name} now agrees with its expected values; drop it from KNOWN_DISAGREEMENTS ({reason})"
                )
            }
        }
    }
}

/// Lists every case whose result differs from its expected values: `cargo test --test placement_cases --
/// --ignored --nocapture list_disagreements`.
#[test]
#[ignore = "prints the disagreements between expected values and courseplan"]
fn list_disagreements() {
    for case in &load().case {
        if let Some(expected) = &case.expected {
            for line in differences(case, expected) {
                println!("{line}");
            }
        }
    }
}

#[test]
fn cases_have_unique_names() {
    let file = load();
    let mut names: Vec<&str> = file.case.iter().map(|c| c.name.as_str()).collect();
    assert!(names.len() >= 24, "all scenarios present");
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "case names are unique");
    for (name, _) in KNOWN_DISAGREEMENTS {
        assert!(
            names.contains(name),
            "unknown case {name} in the allow-list"
        );
    }
}

#[test]
fn crossing_points_have_no_placement_and_the_rest_do() {
    for case in &load().case {
        let stops = expand(case);
        let got = run(case);
        assert_eq!(got.len(), stops.len(), "{}", case.name);
        for (s, g) in stops.iter().zip(&got) {
            assert_eq!(s.crossing, g.is_none(), "{}", case.name);
        }
    }
}

fn example_text() -> String {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/luebeck-sprint/luebeck-sprint.courseplan.toml");
    let file = ProjectFile::load(&project).expect("example loads");
    let doc = read_map_file(&file.map_path()).expect("example map reads");
    let index = MapIndex::from_document(&doc);
    let rules = freemapper_courseplan::rules::load_rule_set(
        &file.event.event.rule_profile,
        &file.event.event.rule_overlays,
    )
    .expect("rule set");
    let mut text = String::from(
        "# auto_place(keep_existing = false) of every course of examples/luebeck-sprint, today's code.\nformat = 1\n",
    );
    for course in &file.event.courses {
        let gaps = freemapper_courseplan::numbers::gap_range(
            &rules,
            map_kind(&doc),
            file.event.event.discipline,
            freemapper_courseplan::rules::print_scale_of(course, doc.georef.scale_denominator),
        );
        let placed = auto_place(
            &file.event,
            course,
            &index,
            doc.georef.scale_denominator,
            map_kind(&doc),
            gaps,
            false,
        )
        .expect("auto_place");
        let items: Vec<String> = placed
            .iter()
            .map(|p| format!("{{ direction = {:?}, gap = {:?} }}", p.direction, p.gap_mm))
            .collect();
        text.push_str(&format!(
            "\n[[course]]\nname = {:?}\nplacements = [{}]\n",
            course.name,
            items.join(", ")
        ));
    }
    text
}

#[test]
fn example_auto_placements_are_unchanged() {
    let baseline = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/placement/luebeck-sprint-auto.toml");
    let text = example_text();
    if std::env::var("PLACEMENT_CASES_DUMP").is_ok() {
        std::fs::write(&baseline, text).expect("baseline written");
        return;
    }
    let stored = std::fs::read_to_string(&baseline).expect("baseline exists");
    let parse = |t: &str| -> toml::Table { t.parse().expect("toml") };
    let (stored, now) = (parse(&stored), parse(&text));
    let courses = |t: &toml::Table| t["course"].as_array().expect("courses").clone();
    let (a, b) = (courses(&stored), courses(&now));
    assert_eq!(a.len(), b.len(), "same courses");
    for (a, b) in a.iter().zip(&b) {
        assert_eq!(a["name"], b["name"]);
        let (pa, pb) = (
            a["placements"].as_array().unwrap(),
            b["placements"].as_array().unwrap(),
        );
        assert_eq!(pa.len(), pb.len(), "{}", a["name"]);
        for (k, (x, y)) in pa.iter().zip(pb).enumerate() {
            for key in ["direction", "gap"] {
                let (x, y) = (x[key].as_float().unwrap(), y[key].as_float().unwrap());
                assert!(
                    (x - y).abs() <= TOLERANCE,
                    "course {} control {k} {key}: {x} vs {y}",
                    a["name"]
                );
            }
        }
    }
}

/// Stored directions are whole tenths of a degree, bit for bit: `102.4`, not `102.39999999999998`
/// (ADR 0031, ADR 0036). Core wraps a candidate past 360 degrees after rounding it, so courseplan
/// rounds the directions it finds. A kept number comes back untouched.
fn is_whole_tenth(d: f64) -> bool {
    (0.0..360.0).contains(&d) && (d * 10.0).round() / 10.0 == d
}

#[test]
fn automatic_directions_are_whole_tenths_in_the_cases() {
    let mut automatic = 0;
    for case in &load().case {
        let stops = expand(case);
        for (k, (stop, got)) in stops.iter().zip(run(case)).enumerate() {
            let Some(got) = got else { continue };
            if let Some(kept) = stop.kept {
                assert_eq!(
                    got.direction.to_bits(),
                    kept.direction.to_bits(),
                    "{} stop {k}: a kept direction stays as it is",
                    case.name
                );
            } else {
                assert!(
                    is_whole_tenth(got.direction),
                    "{} stop {k}: direction {:?}",
                    case.name,
                    got.direction
                );
                automatic += 1;
            }
        }
    }
    assert!(automatic > 50, "{automatic} automatic placements");
}

#[test]
fn automatic_directions_are_whole_tenths_in_generated_courses() {
    // Crowded courses, so that many candidates are blocked and the search walks around the circle,
    // past 360 degrees. A deterministic pseudo-random walk makes the stops and the map symbols.
    let mut seed: u64 = 987_654_321;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((seed >> 33) as f64) / f64::from(1u32 << 31)
    };
    let mut automatic = 0;
    let mut wrapped = 0;
    for round in 0..12 {
        let n = 4 + round % 6;
        let stops: Vec<Stop> = (0..n)
            .map(|_| Stop {
                at: (next() * 40.0, next() * 40.0),
                crossing: false,
                kept: None,
                bends: Vec::new(),
            })
            .collect();
        let points: Vec<Pt> = (0..80).map(|_| (next() * 40.0, next() * 40.0)).collect();
        let lines: Vec<Vec<Pt>> = (0..12)
            .map(|_| {
                let a = (next() * 40.0, next() * 40.0);
                vec![a, (a.0 + next() * 10.0 - 5.0, a.1 + next() * 10.0 - 5.0)]
            })
            .collect();
        let scale = if round % 2 == 0 { 4000 } else { 10_000 };
        let kind = if round % 2 == 0 {
            MapKind::Sprint
        } else {
            MapKind::Forest
        };
        let event = event_of(&stops, (0.0, 0.0), (40.0, 40.0), &[]);
        let index = MapIndex::from_document(&map_document(scale, &points, &lines));
        let placed = auto_place(
            &event,
            &event.courses[0],
            &index,
            scale,
            kind,
            gap_range_of_one_rule(None, None),
            false,
        )
        .expect("auto_place");
        for p in placed {
            assert!(
                is_whole_tenth(p.direction),
                "round {round}: {:?}",
                p.direction
            );
            wrapped += usize::from(p.direction < 90.0);
            automatic += 1;
        }
    }
    assert!(
        automatic > 50 && wrapped > 0,
        "{automatic} placements, {wrapped} low"
    );
}

#[test]
fn example_directions_are_whole_tenths() {
    let table: toml::Table = example_text().parse().expect("toml");
    let mut count = 0;
    for course in table["course"].as_array().expect("courses") {
        for p in course["placements"].as_array().expect("placements") {
            let d = p["direction"].as_float().expect("direction");
            assert!(is_whole_tenth(d), "{}: {d:?}", course["name"]);
            count += 1;
        }
    }
    assert!(count > 0);
}

/// Times the placement on a large map. Run in release mode:
/// `cargo test --release -p freemapper-courseplan --test placement_cases -- --ignored --nocapture large_map_timing`.
#[test]
#[ignore = "timing, run in release mode"]
fn large_map_timing() {
    // 1000 polylines of 100 segments each in a 400 x 400 mm square (a deterministic pseudo-random walk).
    let mut seed: u64 = 12345;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 33) as f64) / f64::from(1u32 << 31)
    };
    let mut lines = Vec::new();
    for _ in 0..1000 {
        let mut p = (next() * 400.0, next() * 400.0);
        let mut line = vec![p];
        for _ in 0..100 {
            p = (
                (p.0 + (next() - 0.5) * 8.0).clamp(0.0, 400.0),
                (p.1 + (next() - 0.5) * 8.0).clamp(0.0, 400.0),
            );
            line.push(p);
        }
        lines.push(line);
    }
    let index = MapIndex::from_document(&map_document(4000, &[], &lines));
    assert_eq!(index.len(), 1000);
    // 40 controls on a 8 x 5 grid, 45 mm apart.
    let stops: Vec<Stop> = (0..40)
        .map(|i| Stop {
            at: (
                30.0 + 45.0 * f64::from(i % 8),
                30.0 + 70.0 * f64::from(i / 8),
            ),
            crossing: false,
            kept: None,
            bends: vec![],
        })
        .collect();
    let event = event_of(&stops, (5.0, 5.0), (395.0, 395.0), &[]);
    let started = std::time::Instant::now();
    let placed = auto_place(
        &event,
        &event.courses[0],
        &index,
        4000,
        MapKind::Sprint,
        GapRange { min: 0.5, max: 5.0 },
        false,
    )
    .expect("auto_place");
    println!(
        "large_map_timing: {} placements, {} features, {:?}",
        placed.len(),
        index.len(),
        started.elapsed()
    );
}
