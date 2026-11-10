//! In-process tests of the editor routes and of the embedded page.

use std::collections::BTreeSet;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use freemapper_courseplan::text::{Lang, TEXTS, available};
use tokio::sync::{Mutex, Notify};
use tower::ServiceExt;

use super::{Assets, EditorState, router};

fn app(lang: &Lang) -> axum::Router {
    let state = EditorState::new(
        "/p/event.courseplan.toml".into(),
        lang.clone(),
        Arc::new(Notify::new()),
    );
    router("/tok", Arc::new(Mutex::new(state)))
}

async fn get_json(lang: &Lang) -> serde_json::Value {
    let response = app(lang)
        .oneshot(Request::get("/tok/api/texts").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn editor_ids() -> BTreeSet<String> {
    TEXTS
        .message_ids("en")
        .into_iter()
        .filter(|id| id.starts_with("editor-"))
        .collect()
}

#[tokio::test]
async fn texts_cover_every_editor_message_in_both_languages() {
    let expected = editor_ids();
    assert!(
        expected.len() > 10,
        "only {} editor messages",
        expected.len()
    );
    let mut codes = Vec::new();
    for lang in available() {
        let answer = get_json(&lang).await;
        assert_eq!(answer["lang"], lang.code());
        let keys: BTreeSet<String> = answer["texts"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, expected, "{}", lang.code());
        for (id, value) in answer["texts"].as_object().unwrap() {
            let value = value.as_str().unwrap();
            assert!(!value.is_empty() && value != id, "{}: {id}", lang.code());
        }
        codes.push(lang.code());
    }
    assert!(codes.iter().any(|c| c == "de") && codes.iter().any(|c| c == "en"));
}

/// The `.ftl` files, to read the messages as written.
const CATALOGS: [(&str, &str); 2] = [
    (
        "en",
        include_str!("../../../freemapper-courseplan/i18n/en/courseplan.ftl"),
    ),
    (
        "de",
        include_str!("../../../freemapper-courseplan/i18n/de/courseplan.ftl"),
    ),
];

#[test]
fn editor_messages_take_no_arguments() {
    for (code, source) in CATALOGS {
        let mut current: Option<&str> = None;
        let mut seen = 0;
        for line in source.lines() {
            if !line.starts_with([' ', '#']) && !line.is_empty() {
                current = line.split('=').next().map(str::trim);
            }
            if current.is_some_and(|id| id.starts_with("editor-")) {
                seen += 1;
                assert!(
                    !line.contains("{ $"),
                    "{code}: {current:?} takes an argument"
                );
            }
        }
        assert!(seen > 10, "{code}: only {seen} editor lines");
    }
}

fn embedded_texts() -> Vec<(String, String)> {
    Assets::iter()
        .map(|name| {
            let file = Assets::get(&name).unwrap();
            (
                name.to_string(),
                String::from_utf8(file.data.into_owned()).unwrap(),
            )
        })
        .collect()
}

#[test]
fn assets_have_no_inline_script_or_handler() {
    let mut pages = 0;
    for (name, content) in embedded_texts() {
        if !name.ends_with(".html") {
            continue;
        }
        pages += 1;
        for (at, _) in content.match_indices("<script") {
            let tag = &content[at..at + content[at..].find('>').unwrap()];
            assert!(tag.contains(" src="), "{name}: inline script {tag}");
        }
        assert!(!content.contains("javascript:"), "{name}: javascript: URL");
        for (at, _) in content.match_indices("on") {
            let before_ok = content[..at].ends_with(char::is_whitespace);
            let rest = &content[at + 2..];
            let letters = rest.chars().take_while(char::is_ascii_lowercase).count();
            assert!(
                !(before_ok && letters > 0 && rest[letters..].starts_with('=')),
                "{name}: event handler attribute near {:?}",
                &content[at..at + 12.min(content.len() - at)]
            );
        }
    }
    assert!(pages > 0);
}

#[test]
fn every_data_text_id_is_in_the_catalog() {
    let ids = editor_ids();
    let mut used = 0;
    for (name, content) in embedded_texts() {
        for attribute in ["data-text=\"", "data-label=\"", "data-title=\""] {
            for (at, _) in content.match_indices(attribute) {
                let rest = &content[at + attribute.len()..];
                let id = &rest[..rest.find('"').unwrap()];
                assert!(ids.contains(id), "{name}: {id} is not an editor message");
                used += 1;
            }
        }
    }
    assert!(used > 10, "only {used} ids found");
}

#[test]
fn assets_stay_small() {
    let total: usize = Assets::iter()
        .map(|name| Assets::get(&name).unwrap().data.len())
        .sum();
    assert!(total < 300 * 1024, "{total} bytes");
}

#[tokio::test]
async fn page_and_assets_are_served_from_the_embedded_set() {
    let lang = Lang::english();
    let get = |path: &'static str| {
        let app = app(&lang);
        async move {
            app.oneshot(Request::get(path).body(Body::empty()).unwrap())
                .await
                .unwrap()
        }
    };
    let index = get("/tok/").await;
    assert_eq!(index.status(), StatusCode::OK);
    assert!(
        index.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/html")
    );
    let script = get("/tok/assets/js/main.js").await;
    assert_eq!(script.status(), StatusCode::OK);
    assert!(
        script.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/javascript")
    );
    assert_eq!(
        get("/tok/assets/js/missing.js").await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get("/tok/assets/..%2fCargo.toml").await.status(),
        StatusCode::NOT_FOUND
    );
}

mod map {
    use super::*;
    use crate::editor::map_view::{MapCache, MapView, PageRect, PlaceholderMap};
    use freemapper_courseplan_model::MapPoint;

    fn placeholder() -> PlaceholderMap {
        PlaceholderMap::from_extent(
            Some((MapPoint { x: 0.0, y: 0.0 }, MapPoint { x: 200.0, y: 150.0 })),
            "Map <note>".into(),
        )
    }

    #[test]
    fn placeholder_mapping_round_trips() {
        let map = PlaceholderMap::from_extent(
            Some((MapPoint { x: -30.0, y: 5.5 }, MapPoint { x: 90.0, y: 70.0 })),
            String::new(),
        )
        .mapping();
        for (x, y) in [(0.0, 0.0), (-12.25, 33.5), (1234.5, -987.25)] {
            let (px, py) = map.to_page(MapPoint { x, y });
            let back = map.to_map(px, py);
            assert!((back.x - x).abs() < 1e-9 && (back.y - y).abs() < 1e-9);
        }
    }

    #[test]
    fn placeholder_page_box_is_extent_plus_10_mm() {
        let map = placeholder();
        let b = map.page_box();
        assert_eq!((b.x0, b.y0, b.x1, b.y1), (0.0, 0.0, 220.0, 170.0));
        let m = map.mapping();
        assert_eq!((m.k, m.dx, m.dy), (1.0, 10.0, 10.0));
    }

    #[test]
    fn placeholder_draws_no_map_content() {
        let map = placeholder();
        let svg = map.svg_for(map.page_box(), 1.0).unwrap();
        let paths = svg.matches("<path").count();
        let known = svg.matches("<path class=\"grid\"").count()
            + svg.matches("<path class=\"frame\"").count();
        assert!(paths >= 2 && paths == known, "{svg}");
        assert_eq!(svg.matches("<text").count(), 1);
        assert!(svg.contains("Map &lt;note&gt;"));
        assert_eq!(
            svg.matches("<image").count() + svg.matches("<circle").count(),
            0
        );
    }

    #[test]
    fn placeholder_svg_is_small_at_any_zoom() {
        let map = placeholder();
        for zoom in [0.01, 100.0] {
            let svg = map.svg_for(map.page_box(), zoom).unwrap();
            assert!(svg.len() < 64 * 1024, "{zoom}: {}", svg.len());
        }
        let huge = PlaceholderMap::from_extent(
            Some((MapPoint { x: 0.0, y: 0.0 }, MapPoint { x: 1e6, y: 1e6 })),
            String::new(),
        );
        assert!(huge.svg_for(huge.page_box(), 0.01).unwrap().len() < 64 * 1024);
        assert!(huge.svg_for(huge.page_box(), 1.0).is_err());
    }

    #[test]
    fn placeholder_refuses_hostile_rects() {
        let map = placeholder();
        for (x1, zoom) in [
            (f64::NAN, 1.0),
            (f64::INFINITY, 1.0),
            (100.0, 0.0),
            (100.0, f64::NAN),
        ] {
            let rect = PageRect {
                x0: 0.0,
                y0: 0.0,
                x1,
                y1: 10.0,
            };
            assert!(map.svg_for(rect, zoom).is_err());
        }
    }

    async fn status(query: &str) -> StatusCode {
        app(&Lang::english())
            .oneshot(
                Request::get(format!("/tok/api/map.svg{query}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn map_svg_rejects_bad_rects() {
        for query in [
            "",
            "?x0=0&y0=0&x1=10",
            "?x0=10&y0=0&x1=10&y1=10",
            "?x0=10&y0=0&x1=0&y1=10",
            "?x0=NaN&y0=0&x1=10&y1=10",
            "?x0=0&y0=0&x1=inf&y1=10",
            "?x0=0&y0=0&x1=1e300&y1=10",
            "?x0=0&y0=0&x1=10&y1=10&zoom=0",
            "?x0=0&y0=0&x1=10&y1=10&zoom=1e9",
            "?x0=0&y0=0&x1=10&y1=10&zoom=abc",
            "?x0=0&x0=1&y0=0&x1=10&y1=10",
            "?x0=0&y0=0&x1=10&y1=10&extra=1",
        ] {
            assert_eq!(status(query).await, StatusCode::BAD_REQUEST, "{query}");
        }
    }

    #[test]
    fn cache_rebuilds_when_the_map_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("map.omap");
        let source = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/maps/forest-sample.omap"
        ))
        .unwrap();
        std::fs::write(&path, &source).unwrap();
        let mut cache = MapCache::new("note".into());
        let first = cache.get(&path).unwrap().stamp;
        assert_eq!(cache.get(&path).unwrap().stamp, first);
        let mut changed = source;
        changed.extend_from_slice(b"\n\n");
        std::fs::write(&path, changed).unwrap();
        let second = cache.get(&path).unwrap();
        assert_ne!(second.stamp, first);
        assert!(second.view.is_placeholder());
    }
}

mod overlay {
    use crate::editor::overlay::{CourseOverlay, overlay};
    use freemapper_courseplan::map::{MapIndex, MapKind};
    use freemapper_courseplan::numbers::{GapRange, auto_place, number_centre};
    use freemapper_courseplan_model::{
        Control, ControlCode, ControlDescription, ControlKind, Course, CourseControl, CourseKind,
        Event, EventMeta, FORMAT_VERSION, MapPoint, NumberPlacement, Sources, SpecialObject,
    };

    fn p(x: f64, y: f64) -> MapPoint {
        MapPoint { x, y }
    }

    fn control(code: &str, at: MapPoint, number: Option<NumberPlacement>) -> Control {
        Control {
            ppen_id: None,
            code: ControlCode(code.to_owned()),
            kind: ControlKind::Control,
            map_mm: at,
            description: ControlDescription::default(),
            number,
        }
    }

    fn stop(code: &str, number: Option<NumberPlacement>) -> CourseControl {
        CourseControl {
            code: ControlCode(code.to_owned()),
            number,
            bends: vec![],
            taped: false,
        }
    }

    /// Start (0,0), controls 31 (0,-50) and 32 (100,100), finish (200,100); course "A" visits 31 and 32.
    fn event() -> Event {
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
            controls: vec![
                control("31", p(0.0, -50.0), None),
                control("32", p(100.0, 100.0), None),
            ],
            specials: vec![
                SpecialObject::Start {
                    id: "S1".to_owned(),
                    ppen_id: None,
                    map_mm: p(0.0, 0.0),
                },
                SpecialObject::Finish {
                    id: "F1".to_owned(),
                    ppen_id: None,
                    map_mm: p(200.0, 100.0),
                },
                SpecialObject::Water {
                    id: "W1".to_owned(),
                    ppen_id: None,
                    map_mm: p(5.0, 5.0),
                    courses: vec!["B".to_owned()],
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
                    controls: vec![stop("31", None), stop("32", None)],
                },
            }],
            accepted_findings: vec![],
            sync: None,
        }
    }

    fn run(event: &Event, course: Option<&str>, scale: u32) -> CourseOverlay {
        overlay(
            event,
            course,
            &MapIndex::default(),
            scale,
            MapKind::Sprint,
            GapRange::DEFAULT,
        )
        .unwrap()
    }

    fn near(got: f64, want: f64) {
        assert!((got - want).abs() < 1e-9, "{got} vs {want}");
    }

    #[test]
    fn sizes_at_1_4000() {
        let s = run(&event(), Some("A"), 4000).sizes;
        near(s.circle_r, 3.0);
        near(s.line, 0.35);
        near(s.number_h, 4.0);
    }

    #[test]
    fn sizes_at_1_3000() {
        // factor 4000 / 3000 = 4/3, values rounded to 0.001.
        let s = run(&event(), Some("A"), 3000).sizes;
        near(s.circle_r, 4.0);
        near(s.line, 0.467);
        near(s.number_h, 5.333);
    }

    #[test]
    fn number_box_follows_offset_from_placement() {
        let mut e = event();
        e.controls[0] = control(
            "31",
            p(100.0, 100.0),
            Some(NumberPlacement {
                direction: 0.0,
                gap_mm: 1.0,
            }),
        );
        let o = run(&e, Some("A"), 4000);
        let n = &o.numbers[0];
        // Outer radius 3.0 + 0.35 / 2 = 3.175; the box of "1" is 0.6 * 4.0 = 2.4 wide, so the centre lies at
        // 100 + 3.175 + 1.0 + 2.4 / 2 = 105.375.
        near(n.centre.x, 105.375);
        near(n.centre.y, 100.0);
        near(n.w, 2.4);
        near(n.h, 4.0);
        assert_eq!((n.index, n.code.as_str(), n.text.as_str()), (1, "31", "1"));
        assert_eq!(n.source, "central");
    }

    #[test]
    fn number_box_scales_with_the_map_scale() {
        let mut e = event();
        e.controls[0] = control(
            "31",
            p(100.0, 100.0),
            Some(NumberPlacement {
                direction: 0.0,
                gap_mm: 1.0,
            }),
        );
        // At 1:2000 everything is 2 times larger: 100 + 2 * (3.175 + 1.0 + 1.2) = 110.75.
        let n = &run(&e, Some("A"), 2000).numbers[0];
        near(n.centre.x, 110.75);
        near(n.w, 4.8);
        near(n.h, 8.0);
    }

    #[test]
    fn numbers_name_their_source() {
        let mut e = event();
        let own = NumberPlacement {
            direction: 90.0,
            gap_mm: 1.0,
        };
        if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
            controls[0].number = Some(own.clone());
        }
        let sources: Vec<_> = run(&e, Some("A"), 4000)
            .numbers
            .iter()
            .map(|n| n.source)
            .collect();
        assert_eq!(sources, ["course", "auto"]);
    }

    #[test]
    fn unplaced_number_is_auto_and_matches_auto_place() {
        let e = event();
        let o = run(&e, Some("A"), 4000);
        let placed = auto_place(
            &e,
            &e.courses[0],
            &MapIndex::default(),
            4000,
            MapKind::Sprint,
            GapRange::DEFAULT,
            true,
        )
        .unwrap();
        assert_eq!(o.numbers.len(), 2);
        for (n, (placement, control)) in o.numbers.iter().zip(placed.iter().zip(&e.controls)) {
            assert_eq!(n.source, "auto");
            let centre = number_centre(&control.map_mm, placement, 1, MapKind::Sprint, 4000);
            let want = (
                (centre.x * 1000.0).round() / 1000.0,
                (centre.y * 1000.0).round() / 1000.0,
            );
            near(n.centre.x, want.0);
            near(n.centre.y, want.1);
        }
        assert_eq!(o.numbers[1].text, "2");
    }

    #[test]
    fn legs_use_course_legs_drawn_part() {
        let mut e = event();
        e.controls[0] = control("31", p(50.0, 0.0), None);
        let o = run(&e, Some("A"), 4000);
        assert_eq!(o.legs.len(), 3);
        let first = &o.legs[0];
        assert_eq!(first.after, 0);
        assert_eq!(first.points, [p(0.0, 0.0), p(50.0, 0.0)]);
        // Cut by the circle radius 3.0 at both ends.
        assert_eq!(first.drawn, [p(3.0, 0.0), p(47.0, 0.0)]);
        assert_eq!(o.legs[1].after, 1);
        assert!(!first.taped);
    }

    #[test]
    fn start_triangle_points_along_the_first_leg() {
        // Start (0,0), first control (0,-50): up on the paper is 90 degrees.
        let o = run(&event(), Some("A"), 4000);
        assert_eq!(o.starts.len(), 1);
        near(o.starts[0].angle_deg, 90.0);
        assert_eq!(o.finishes[0].id, "F1");
        // Course A leaves out W1, which belongs to course B only.
        assert!(o.specials.is_empty());
    }

    #[test]
    fn all_controls_view_has_no_legs_or_numbers() {
        let o = run(&event(), None, 4000);
        assert!(o.legs.is_empty() && o.numbers.is_empty());
        let codes: Vec<_> = o.controls.iter().map(|c| c.code.as_str()).collect();
        assert_eq!(codes, ["31", "32"]);
        assert_eq!(o.specials.len(), 1);
        assert_eq!(o.specials[0].kind, "water");
    }

    #[test]
    fn unknown_course_is_an_error() {
        let r = overlay(
            &event(),
            Some("Z"),
            &MapIndex::default(),
            4000,
            MapKind::Sprint,
            GapRange::DEFAULT,
        );
        assert!(r.is_err());
    }
}

mod state {
    use super::*;
    use freemapper_courseplan_model::{Event, EventMeta, FORMAT_VERSION, ProjectFile, Sources};

    fn project(dir: &std::path::Path) -> std::path::PathBuf {
        std::fs::copy(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/fixtures/maps/forest-sample.omap"
            ),
            dir.join("map.omap"),
        )
        .unwrap();
        let path = dir.join("event.courseplan.toml");
        let event = Event {
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
                map: "map.omap".into(),
                ppen: None,
            },
            classes: vec![],
            controls: vec![],
            specials: vec![],
            courses: vec![],
            accepted_findings: vec![],
            sync: None,
        };
        ProjectFile::create(&path, event).unwrap();
        path
    }

    fn router_for(path: &std::path::Path) -> axum::Router {
        let state = EditorState::new(path.into(), Lang::english(), Arc::new(Notify::new()));
        router("/tok", Arc::new(Mutex::new(state)))
    }

    async fn get(app: &axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
        let response = app
            .clone()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
        )
    }

    #[tokio::test]
    async fn state_of_a_broken_file_has_only_error_and_revision() {
        let (status, body) = get(&app(&Lang::english()), "/tok/api/state").await;
        assert_eq!(status, StatusCode::OK);
        let mut keys: Vec<_> = body.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        assert_eq!(keys, ["error", "revision"]);
        assert!(!body["error"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn state_of_a_good_file_has_every_part() {
        let dir = tempfile::tempdir().unwrap();
        let (_, body) = get(&router_for(&project(dir.path())), "/tok/api/state").await;
        assert!(body["error"].is_null(), "{body}");
        assert_eq!(body["revision"].as_str().unwrap().len(), 16);
        assert_eq!(body["event"]["event"]["title"], "T");
        assert_eq!(body["map"]["scale"], 10000);
        assert_eq!(body["map"]["kind"], "forest");
        assert_eq!(body["courses"], serde_json::json!([]));
        assert!(body["undo"]["undo"].is_null() && body["link"]["ppen"].is_null());
    }

    #[tokio::test]
    async fn state_writes_a_date_as_plain_text() {
        let dir = tempfile::tempdir().unwrap();
        let path = project(dir.path());
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(
            &path,
            text.replacen("title = \"T\"", "title = \"T\"\ndate = 2026-11-11", 1),
        )
        .unwrap();
        let (_, body) = get(&router_for(&path), "/tok/api/state").await;
        assert_eq!(body["event"]["event"]["date"], "2026-11-11", "{body}");
    }

    #[tokio::test]
    async fn revision_changes_with_the_file_bytes_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = project(dir.path());
        let app = router_for(&path);
        let (_, first) = get(&app, "/tok/api/revision").await;
        // A save of the same bytes (new mtime) keeps the revision, and the state follows it.
        let bytes = std::fs::read(&path).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        let (_, same) = get(&app, "/tok/api/revision").await;
        assert_eq!(same["project"], first["project"]);
        let mut changed = bytes;
        changed.extend_from_slice(b"\n# note\n");
        std::fs::write(&path, changed).unwrap();
        let (_, other) = get(&app, "/tok/api/revision").await;
        assert_ne!(other["project"], first["project"]);
        let (_, state) = get(&app, "/tok/api/state").await;
        assert_eq!(state["revision"], other["project"]);
        assert_eq!(
            first["map"].as_str().unwrap(),
            other["map"].as_str().unwrap()
        );
    }

    #[tokio::test]
    async fn map_routes_use_the_state_without_reading_the_project_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = project(dir.path());
        let app = router_for(&path);
        std::fs::remove_file(&path).unwrap();
        let (status, body) = get(&app, "/tok/api/map/view").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["placeholder"], true);
    }

    #[tokio::test]
    async fn overlay_route_answers_for_all_controls_and_refuses_bad_queries() {
        let dir = tempfile::tempdir().unwrap();
        let app = router_for(&project(dir.path()));
        let (status, body) = get(&app, "/tok/api/overlay").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["legs"], serde_json::json!([]));
        assert_eq!(
            get(&app, "/tok/api/overlay?course=Nope").await.0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            get(&app, "/tok/api/overlay?x=1").await.0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            get(&app, "/tok/api/overlay?course=%zz").await.0,
            StatusCode::BAD_REQUEST
        );
    }
}
