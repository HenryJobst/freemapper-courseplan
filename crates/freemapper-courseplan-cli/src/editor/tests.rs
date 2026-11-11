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
    let state = EditorState::new("/p/event.courseplan.toml".into(), lang.clone());
    router("/tok", Arc::new(Mutex::new(state)), Arc::new(Notify::new()))
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
    let mut scripts = 0;
    for (name, content) in embedded_texts() {
        if name.ends_with(".js") {
            // Scripts build the page with textContent, setAttribute and dataset only.
            for banned in [
                "innerHTML",
                "outerHTML",
                "insertAdjacentHTML",
                "document.write",
                "eval(",
                "new Function",
                "setAttribute(\"on",
                "setAttribute('on",
                "javascript:",
            ] {
                assert!(!content.contains(banned), "{name}: {banned}");
            }
            scripts += 1;
            continue;
        }
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
    assert!(pages > 0 && scripts > 3);
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
    use crate::editor::Shared;
    use crate::editor::map_view::{MapView, PageRect, PlaceholderMap, load_map};
    use freemapper_courseplan_model::MapPoint;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

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

    fn forest_sample() -> Vec<u8> {
        std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/maps/forest-sample.omap"
        ))
        .unwrap()
    }

    /// A shared state whose map parse counts itself and, when `pause` is set, takes that long.
    fn counted(pause: Duration) -> (Shared, Arc<AtomicUsize>) {
        let parses = Arc::new(AtomicUsize::new(0));
        let seen = parses.clone();
        let mut state = EditorState::new("/p/event.courseplan.toml".into(), Lang::english());
        state.map.parse_hook = Some(Arc::new(move || {
            seen.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(pause);
        }));
        (Arc::new(Mutex::new(state)), parses)
    }

    #[tokio::test]
    async fn cache_rebuilds_when_the_map_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("map.omap");
        let source = forest_sample();
        std::fs::write(&path, &source).unwrap();
        let (state, parses) = counted(Duration::ZERO);
        let first = load_map(&state, path.clone()).await.unwrap();
        let again = load_map(&state, path.clone()).await.unwrap();
        assert!(Arc::ptr_eq(&first, &again));
        assert_eq!(parses.load(Ordering::SeqCst), 1);
        let mut changed = source;
        changed.extend_from_slice(b"\n\n");
        std::fs::write(&path, changed).unwrap();
        let second = load_map(&state, path).await.unwrap();
        assert_ne!(second.stamp, first.stamp);
        assert_eq!(parses.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_broken_map_is_parsed_once_and_its_error_has_no_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.omap");
        std::fs::write(&path, "<map><unclosed>").unwrap();
        let (state, parses) = counted(Duration::ZERO);
        let first = load_map(&state, path.clone()).await.err().unwrap();
        let second = load_map(&state, path.clone()).await.err().unwrap();
        assert_eq!(first, second);
        assert_eq!(parses.load(Ordering::SeqCst), 1);
        assert!(first.contains("broken.omap"), "{first}");
        let folder = dir.path().to_string_lossy().into_owned();
        assert!(!first.contains(&folder), "{first}");
        assert!(first.chars().count() <= 501);
    }

    #[tokio::test]
    async fn the_cache_key_includes_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b) = (dir.path().join("a.omap"), dir.path().join("b.omap"));
        let source = forest_sample();
        let time = std::time::SystemTime::now();
        for path in [&a, &b] {
            std::fs::write(path, &source).unwrap();
            std::fs::File::options()
                .write(true)
                .open(path)
                .unwrap()
                .set_modified(time)
                .unwrap();
        }
        let (state, parses) = counted(Duration::ZERO);
        let first = load_map(&state, a).await.unwrap();
        let second = load_map(&state, b).await.unwrap();
        assert_eq!(first.stamp, second.stamp);
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(parses.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn quit_and_revision_answer_while_a_map_is_parsed() {
        let dir = tempfile::tempdir().unwrap();
        let path = super::state::project(dir.path());
        let started = Instant::now();
        let stop = Arc::new(Notify::new());
        let mut state = EditorState::new(path, Lang::english());
        let parsing = Arc::new(AtomicUsize::new(0));
        let seen = parsing.clone();
        state.map.parse_hook = Some(Arc::new(move || {
            seen.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(1500));
        }));
        let app = router("/tok", Arc::new(Mutex::new(state)), stop.clone());
        let slow = tokio::spawn({
            let app = app.clone();
            async move {
                app.oneshot(Request::get("/tok/api/state").body(Body::empty()).unwrap())
                    .await
                    .unwrap()
                    .status()
            }
        });
        while parsing.load(Ordering::SeqCst) == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        let revision = app
            .clone()
            .oneshot(
                Request::get("/tok/api/revision")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(revision.status(), StatusCode::OK);
        let quit = app
            .oneshot(Request::post("/tok/api/quit").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(quit.status(), StatusCode::OK);
        assert!(started.elapsed() < Duration::from_millis(1000));
        // quit stored its signal without waiting for the parse.
        tokio::time::timeout(Duration::from_millis(100), stop.notified())
            .await
            .unwrap();
        assert_eq!(slow.await.unwrap(), StatusCode::OK);
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

    #[test]
    fn a_crossing_point_gets_no_number_but_counts_in_the_index() {
        let mut e = event();
        e.controls[0].kind = ControlKind::CrossingPoint;
        e.courses[0].kind = CourseKind::Individual {
            controls: vec![stop("31", None), stop("32", None)],
        };
        let o = run(&e, Some("A"), 4000);
        assert_eq!(o.controls[0].kind, "crossing_point");
        assert_eq!(o.controls[1].kind, "control");
        assert_eq!(o.numbers.len(), 1);
        assert_eq!(o.numbers[0].code, "32");
        assert_eq!(o.numbers[0].index, 2);
        assert_eq!(o.numbers[0].text, "1");
    }

    #[test]
    fn a_taped_leg_follows_the_control_that_leaves() {
        let mut e = event();
        e.courses[0].kind = CourseKind::Individual {
            controls: vec![
                CourseControl {
                    taped: true,
                    ..stop("31", None)
                },
                stop("32", None),
            ],
        };
        let o = run(&e, Some("A"), 4000);
        let taped: Vec<bool> = o.legs.iter().map(|l| l.taped).collect();
        assert_eq!(taped, [false, true, false]);
    }

    #[test]
    fn the_json_has_the_keys_and_point_arrays_the_page_reads() {
        let o = serde_json::to_value(run(&event(), Some("A"), 4000)).unwrap();
        let mut keys: Vec<&str> = o.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "controls", "finishes", "legs", "numbers", "sizes", "specials", "starts"
            ]
        );
        assert_eq!(o["sizes"]["circle_r"], 3.0);
        assert_eq!(o["controls"][0]["code"], "31");
        assert_eq!(o["controls"][0]["at"], serde_json::json!([0.0, -50.0]));
        assert_eq!(o["starts"][0]["at"], serde_json::json!([0.0, 0.0]));
        assert!(o["starts"][0]["angle_deg"].is_number());
        assert!(o["legs"][0]["points"][0].is_array());
        assert!(o["legs"][0]["drawn"][0].is_array());
        assert!(o["legs"][0]["taped"].is_boolean() && o["legs"][0]["after"].is_number());
        let number = &o["numbers"][0];
        assert!(number["centre"].is_array() && number["text"].is_string());
        assert!(number["w"].is_number() && number["h"].is_number());
        assert_eq!(number["source"], "auto");
        assert!(number["index"].is_number() && number["code"].is_string());
        assert!(o["specials"].is_array());
    }
}

mod state {
    use super::*;
    use freemapper_courseplan_model::{Event, EventMeta, FORMAT_VERSION, ProjectFile, Sources};

    pub(super) fn project(dir: &std::path::Path) -> std::path::PathBuf {
        project_with(dir, |_| {})
    }

    fn project_with(dir: &std::path::Path, change: impl FnOnce(&mut Event)) -> std::path::PathBuf {
        std::fs::copy(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/fixtures/maps/forest-sample.omap"
            ),
            dir.join("map.omap"),
        )
        .unwrap();
        let path = dir.join("event.courseplan.toml");
        let mut event = Event {
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
        change(&mut event);
        ProjectFile::create(&path, event).unwrap();
        path
    }

    fn router_for(path: &std::path::Path) -> axum::Router {
        let state = EditorState::new(path.into(), Lang::english());
        router("/tok", Arc::new(Mutex::new(state)), Arc::new(Notify::new()))
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

    async fn raw(
        app: &axum::Router,
        request: Request<Body>,
    ) -> (StatusCode, axum::http::HeaderMap, String) {
        let response = app.clone().oneshot(request).await.unwrap();
        let (status, headers) = (response.status(), response.headers().clone());
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (
            status,
            headers,
            String::from_utf8_lossy(&bytes).into_owned(),
        )
    }

    #[tokio::test]
    async fn a_team_course_gets_422_and_a_localized_message() {
        use freemapper_courseplan_model::{Course, CourseKind, SpecialObject};
        let dir = tempfile::tempdir().unwrap();
        let path = project_with(dir.path(), |event| {
            let at = freemapper_courseplan_model::MapPoint { x: 0.0, y: 0.0 };
            event.specials = vec![
                SpecialObject::Start {
                    id: "S1".to_owned(),
                    ppen_id: None,
                    map_mm: at.clone(),
                },
                SpecialObject::Finish {
                    id: "F1".to_owned(),
                    ppen_id: None,
                    map_mm: at,
                },
            ];
            event.courses = vec![Course {
                ppen_id: None,
                name: "Team".to_owned(),
                classes: vec![],
                start: "S1".to_owned(),
                finish: "F1".to_owned(),
                start_bends: vec![],
                start_taped: false,
                print_area: None,
                climb_m: None,
                planned_print_scale: None,
                kind: CourseKind::Team {
                    team_size: 2,
                    mandatory: vec![],
                    free: vec![],
                },
            }];
        });
        let (status, _, body) = raw(
            &router_for(&path),
            Request::get("/tok/api/overlay?course=Team")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn map_svg_answers_200_with_the_svg_content_type() {
        let dir = tempfile::tempdir().unwrap();
        let app = router_for(&project(dir.path()));
        let (status, headers, body) = raw(
            &app,
            Request::get("/tok/api/map.svg?x0=0&y0=0&x1=50&y1=50&zoom=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers["content-type"], "image/svg+xml");
        assert!(body.starts_with("<svg"), "{body}");
    }

    #[tokio::test]
    async fn a_project_file_over_the_limit_is_refused_with_a_message() {
        let dir = tempfile::tempdir().unwrap();
        let path = project(dir.path());
        let mut bytes = std::fs::read(&path).unwrap();
        bytes.resize(crate::editor::api::MAX_PROJECT_BYTES as usize + 1, b'#');
        std::fs::write(&path, bytes).unwrap();
        let (_, body) = get(&router_for(&path), "/tok/api/state").await;
        let error = body["error"].as_str().unwrap();
        assert!(
            error.contains("event.courseplan.toml") && error.contains("16 MiB"),
            "{error}"
        );
        assert!(!error.contains(dir.path().to_str().unwrap()), "{error}");
    }

    #[tokio::test]
    async fn a_project_error_is_cut_short_and_names_only_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = project(dir.path());
        std::fs::write(&path, format!("format = {}\n", "[".repeat(5000))).unwrap();
        let (_, body) = get(&router_for(&path), "/tok/api/state").await;
        let error = body["error"].as_str().unwrap();
        assert!(error.chars().count() <= 501, "{}", error.chars().count());
        assert!(!error.contains(dir.path().to_str().unwrap()), "{error}");
    }

    #[tokio::test]
    async fn an_unchanged_stamp_means_the_file_is_not_read_again() {
        let dir = tempfile::tempdir().unwrap();
        let path = project(dir.path());
        let app = router_for(&path);
        let (_, first) = get(&app, "/tok/api/revision").await;
        // Same length, same mtime, other bytes: only a re-read would notice.
        let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
        let mut bytes = std::fs::read(&path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] = if bytes[last] == b'#' { b' ' } else { b'#' };
        std::fs::write(&path, bytes).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(modified)
            .unwrap();
        let (_, same) = get(&app, "/tok/api/revision").await;
        assert_eq!(same["project"], first["project"]);
    }
}

mod history {
    use crate::editor::history::{HistoryError, MAX_STEPS, UndoHistory};

    #[test]
    fn history_keeps_100_steps() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.toml");
        let mut history = UndoHistory::default();
        for i in 0..MAX_STEPS + 20 {
            history.record(format!("s{i}"), format!("t{i}"), format!("t{}", i + 1));
        }
        std::fs::write(&path, format!("t{}", MAX_STEPS + 20)).unwrap();
        let mut undone = 0;
        while history.undo(&path).is_ok() {
            undone += 1;
            // The file follows the chain backwards.
            let want = format!("t{}", MAX_STEPS + 20 - undone);
            assert_eq!(std::fs::read_to_string(&path).unwrap(), want);
        }
        assert_eq!(undone, 100);
        assert_eq!(history.undo(&path), Err(HistoryError::Nothing));
        assert_eq!(history.redo(&path).as_deref(), Ok("s20"));
    }
}

mod ops {
    use super::state::project;
    use super::*;
    use serde_json::{Value, json};

    fn router_for(path: &std::path::Path) -> axum::Router {
        let state = EditorState::new(path.into(), Lang::english());
        router("/tok", Arc::new(Mutex::new(state)), Arc::new(Notify::new()))
    }

    async fn send(
        app: &axum::Router,
        method: &str,
        uri: &str,
        revision: Option<&str>,
        body: Value,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json");
        if let Some(revision) = revision {
            request = request.header("x-courseplan-revision", revision);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn state(app: &axum::Router) -> Value {
        send(app, "GET", "/tok/api/state", None, json!({})).await.1
    }

    async fn op(
        app: &axum::Router,
        tool: &str,
        revision: &str,
        body: Value,
    ) -> (StatusCode, Value) {
        send(
            app,
            "POST",
            &format!("/tok/api/op/{tool}"),
            Some(revision),
            body,
        )
        .await
    }

    fn centre(state: &Value) -> Value {
        let e = &state["map"]["extent"];
        json!({
            "x": (e[0][0].as_f64().unwrap() + e[1][0].as_f64().unwrap()) / 2.0,
            "y": (e[0][1].as_f64().unwrap() + e[1][1].as_f64().unwrap()) / 2.0,
        })
    }

    /// A project with a commented line, an app on it, and the first state.
    async fn setup() -> (tempfile::TempDir, std::path::PathBuf, axum::Router, Value) {
        let dir = tempfile::tempdir().unwrap();
        let path = project(dir.path());
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("# keep this comment\n{text}")).unwrap();
        let app = router_for(&path);
        let first = state(&app).await;
        (dir, path, app, first)
    }

    fn controls_in(path: &std::path::Path) -> usize {
        std::fs::read_to_string(path)
            .unwrap()
            .matches("[[controls]]")
            .count()
    }

    #[tokio::test]
    async fn op_moves_a_control_and_returns_the_new_revision() {
        let (_dir, path, app, first) = setup().await;
        let at = centre(&first);
        let (status, added) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": at}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{added}");
        let code = added["result"]["code"].as_str().unwrap().to_owned();
        let revision = added["revision"].as_str().unwrap().to_owned();
        assert_ne!(revision, first["revision"].as_str().unwrap());
        let to = json!({"x": at["x"].as_f64().unwrap() + 2.0, "y": at["y"]});
        let (status, moved) = op(
            &app,
            "move_control",
            &revision,
            json!({"code": code, "at": to}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{moved}");
        let now = state(&app).await;
        assert_eq!(now["revision"], moved["revision"]);
        assert_eq!(now["event"]["controls"][0]["map_mm"][0], to["x"]);
        assert_eq!(controls_in(&path), 1);
        assert!(now["undo"]["undo"].as_str().unwrap().contains("Move"));
    }

    #[tokio::test]
    async fn stale_revision_is_refused_and_file_unchanged() {
        let (_dir, path, app, first) = setup().await;
        let at = centre(&first);
        let (status, _) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": at}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let bytes = std::fs::read(&path).unwrap();
        let (status, body) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": at}),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["revision"], state(&app).await["revision"]);
        assert!(body["error"].as_str().unwrap().contains("outside"));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        // A change by another program is just as stale.
        let mut other = bytes.clone();
        other.extend_from_slice(b"\n# other\n");
        std::fs::write(&path, &other).unwrap();
        let (status, _) = op(
            &app,
            "add_control",
            state(&app).await["revision"].as_str().unwrap(),
            json!({"at": at}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let (status, _) = op(&app, "add_control", "0000000000000000", json!({"at": at})).await;
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn refused_edit_is_422_and_file_byte_identical() {
        let (_dir, path, app, first) = setup().await;
        let bytes = std::fs::read(&path).unwrap();
        let (status, body) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": {"x": 1.0e6, "y": 1.0e6}}),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(!body["error"].as_str().unwrap().is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(state(&app).await["undo"]["undo"].is_null());
        // Bad arguments are the caller's fault.
        let (status, _) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"nope": 1}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let (status, _) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": {"x": 1.0, "y": 1.0}, "project": "x"}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn unknown_or_excluded_tool_is_404() {
        let (_dir, path, app, first) = setup().await;
        let bytes = std::fs::read(&path).unwrap();
        for tool in ["init_event", "nope", "check", "export_ppen"] {
            let (status, _) = op(&app, tool, first["revision"].as_str().unwrap(), json!({})).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{tool}");
        }
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }

    #[tokio::test]
    async fn undo_restores_the_exact_bytes_and_redo_reapplies() {
        let (_dir, path, app, first) = setup().await;
        let original = std::fs::read(&path).unwrap();
        let at = centre(&first);
        let (_, added) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": at}),
        )
        .await;
        let after = std::fs::read(&path).unwrap();
        assert_ne!(after, original);
        let (status, undone) = send(&app, "POST", "/tok/api/undo", None, json!({})).await;
        assert_eq!(status, StatusCode::OK, "{undone}");
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .starts_with("# keep this comment")
        );
        assert_eq!(undone["revision"], first["revision"]);
        let now = state(&app).await;
        assert!(now["undo"]["undo"].is_null() && now["undo"]["redo"].is_string());
        let (status, redone) = send(&app, "POST", "/tok/api/redo", None, json!({})).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(std::fs::read(&path).unwrap(), after);
        assert_eq!(redone["revision"], added["revision"]);
        let (status, _) = send(&app, "POST", "/tok/api/redo", None, json!({})).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert!(!dir_has_tmp(&path));
    }

    fn dir_has_tmp(path: &std::path::Path) -> bool {
        std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .any(|e| e.unwrap().file_name().to_string_lossy().ends_with(".tmp"))
    }

    #[tokio::test]
    async fn undo_after_external_change_is_refused_and_clears_history() {
        let (_dir, path, app, first) = setup().await;
        let (_, _) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": centre(&first)}),
        )
        .await;
        let mut other = std::fs::read(&path).unwrap();
        other.extend_from_slice(b"\n# someone else\n");
        std::fs::write(&path, &other).unwrap();
        let (status, body) = send(&app, "POST", "/tok/api/undo", None, json!({})).await;
        assert_eq!(status, StatusCode::CONFLICT, "{body}");
        assert_eq!(std::fs::read(&path).unwrap(), other);
        let now = state(&app).await;
        assert!(now["undo"]["undo"].is_null() && now["undo"]["redo"].is_null());
        let (status, _) = send(&app, "POST", "/tok/api/undo", None, json!({})).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(std::fs::read(&path).unwrap(), other);
    }

    #[tokio::test]
    async fn broken_file_blocks_edits_until_fixed() {
        let (_dir, path, app, first) = setup().await;
        let good = std::fs::read(&path).unwrap();
        std::fs::write(&path, "this is [not toml").unwrap();
        let broken = state(&app).await;
        assert!(broken["error"].is_string());
        let (status, body) = op(
            &app,
            "add_control",
            broken["revision"].as_str().unwrap(),
            json!({"at": centre(&first)}),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "{body}");
        assert_eq!(body["error"], broken["error"]);
        assert_eq!(std::fs::read(&path).unwrap(), b"this is [not toml");
        std::fs::write(&path, &good).unwrap();
        let fixed = state(&app).await;
        assert!(fixed["error"].is_null());
        let (status, _) = op(
            &app,
            "add_control",
            fixed["revision"].as_str().unwrap(),
            json!({"at": centre(&first)}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn ops_run_one_at_a_time() {
        let (_dir, path, app, first) = setup().await;
        let revision = first["revision"].as_str().unwrap().to_owned();
        let at = centre(&first);
        let mut tasks = Vec::new();
        for _ in 0..20 {
            let (app, revision, at) = (app.clone(), revision.clone(), at.clone());
            tasks.push(tokio::spawn(async move {
                op(&app, "add_control", &revision, json!({"at": at}))
                    .await
                    .0
            }));
        }
        let mut ok = 0;
        for task in tasks {
            match task.await.unwrap() {
                StatusCode::OK => ok += 1,
                StatusCode::CONFLICT => {}
                other => panic!("{other}"),
            }
        }
        // All 20 saw the same revision, so exactly the first one in wins and no control is lost or doubled.
        assert_eq!(ok, 1);
        assert_eq!(controls_in(&path), ok);
        // The same with revisions that are fresh when sent: every success is in the file.
        let mut ok = 0;
        for _ in 0..5 {
            let mut tasks = Vec::new();
            let now = state(&app).await["revision"].as_str().unwrap().to_owned();
            for _ in 0..8 {
                let (app, now, at) = (app.clone(), now.clone(), at.clone());
                tasks.push(tokio::spawn(async move {
                    op(&app, "add_control", &now, json!({"at": at})).await.0
                }));
            }
            for task in tasks {
                if task.await.unwrap() == StatusCode::OK {
                    ok += 1;
                }
            }
        }
        assert_eq!(controls_in(&path), 1 + ok);
        assert!(ok >= 5);
    }

    #[test]
    fn every_editor_tool_has_an_undo_label_in_both_languages() {
        for lang in available() {
            for tool in crate::mcp::EDITOR_TOOLS {
                let id = format!("editor-op-{}", tool.replace('_', "-"));
                let label = freemapper_courseplan::text::text(&lang, &id, &[]);
                assert!(!label.is_empty() && label != id, "{}: {id}", lang.code());
            }
        }
    }
}
