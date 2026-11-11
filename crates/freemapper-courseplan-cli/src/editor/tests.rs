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
        std::fs::write(&a, forest_sample()).unwrap();
        // A hard link has the same inode, times and bytes: only the path tells the two apart.
        std::fs::hard_link(&a, &b).unwrap();
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

    /// The overlay rounds positions and sizes to 1 micrometre.
    fn near_um(got: f64, want: f64) {
        assert!((got - want).abs() < 1e-3, "{got} vs {want}");
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
        // Outer radius 3.0 + 0.35 / 2 = 3.175; the box of "1" is as wide as the renderer draws it, so the
        // centre lies at 100 + 3.175 + 1.0 + w / 2.
        let (w, _) = freemapper_courseplan::numbers::number_box_mm(MapKind::Sprint, 1);
        near_um(n.centre.x, 100.0 + 3.175 + 1.0 + w / 2.0);
        near(n.centre.y, 100.0);
        near_um(n.w, w);
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
        // At 1:3000 everything is 4/3 times larger.
        let f = 4.0 / 3.0;
        let (w, _) = freemapper_courseplan::numbers::number_box_mm(MapKind::Sprint, 1);
        let n = &run(&e, Some("A"), 3000).numbers[0];
        near_um(n.centre.x, 100.0 + f * (3.175 + 1.0 + w / 2.0));
        near_um(n.w, f * w);
        near_um(n.h, 4.0 * f);
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
        // Cut by the start triangle's circumradius 7 / sqrt(3) at the start, by the circle radius 3.0 at the control.
        assert_eq!(first.drawn, [p(4.041, 0.0), p(47.0, 0.0)]);
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
    fn overlay_draws_every_special_kind() {
        let mut e = event();
        e.specials.truncate(2);
        let pts = |v: &[(f64, f64)]| v.iter().map(|&(x, y)| p(x, y)).collect::<Vec<_>>();
        e.specials.extend([
            SpecialObject::MarkedRoute {
                id: "marked-1".to_owned(),
                ppen_id: None,
                points: pts(&[(0.0, 10.0), (20.0, 10.0)]),
                courses: vec![],
            },
            SpecialObject::OutOfBoundsArea {
                id: "out-of-bounds-1".to_owned(),
                ppen_id: None,
                polygon: pts(&[(10.0, 10.0), (30.0, 10.0), (30.0, 30.0)]),
                courses: vec![],
            },
            SpecialObject::ForbiddenRoute {
                id: "forbidden-1".to_owned(),
                ppen_id: None,
                points: pts(&[(40.0, 40.0)]),
                courses: vec![],
            },
            SpecialObject::ForbiddenRoute {
                id: "forbidden-2".to_owned(),
                ppen_id: None,
                points: pts(&[(40.0, 0.0), (50.0, 5.0), (60.0, 0.0)]),
                courses: vec!["A".to_owned()],
            },
            SpecialObject::DangerousArea {
                id: "danger-1".to_owned(),
                ppen_id: None,
                polygon: pts(&[(0.0, 0.0), (5.0, 0.0), (5.0, 5.0)]),
                courses: vec![],
            },
            SpecialObject::FirstAid {
                id: "aid-1".to_owned(),
                ppen_id: None,
                map_mm: p(70.0, 70.0),
                courses: vec![],
            },
            SpecialObject::Water {
                id: "water-1".to_owned(),
                ppen_id: None,
                map_mm: p(80.0, 80.0),
                courses: vec![],
            },
        ]);
        for course in [Some("A"), None] {
            let o = run(&e, course, 4000);
            let drawn: Vec<(&str, &str, usize)> = o
                .specials
                .iter()
                .map(|s| (s.id.as_str(), s.kind, s.points.len()))
                .collect();
            assert_eq!(
                drawn,
                [
                    ("marked-1", "marked_route", 2),
                    ("out-of-bounds-1", "out_of_bounds_area", 3),
                    ("forbidden-1", "forbidden_route", 1),
                    ("forbidden-2", "forbidden_route", 3),
                    ("danger-1", "dangerous_area", 3),
                    ("aid-1", "first_aid", 1),
                    ("water-1", "water", 1),
                ]
            );
            assert_eq!((o.starts.len(), o.finishes.len()), (1, 1));
        }
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
    fn overlay_legs_list_their_bends_in_order() {
        let mut e = event();
        e.courses[0].start_bends = vec![p(1.0, -10.0)];
        e.courses[0].kind = CourseKind::Individual {
            controls: vec![
                CourseControl {
                    bends: vec![p(20.0, -50.0), p(50.123_456, 0.0)],
                    ..stop("31", None)
                },
                stop("32", None),
            ],
        };
        let o = run(&e, Some("A"), 4000);
        let bends: Vec<Vec<(f64, f64)>> = o
            .legs
            .iter()
            .map(|l| l.bends.iter().map(|b| (b.x, b.y)).collect())
            .collect();
        assert_eq!(
            bends,
            [
                vec![(1.0, -10.0)],
                vec![(20.0, -50.0), (50.123, 0.0)],
                vec![]
            ]
        );
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

    pub(super) fn project_with(
        dir: &std::path::Path,
        change: impl FnOnce(&mut Event),
    ) -> std::path::PathBuf {
        project_on(dir, "forest-sample.omap", change)
    }

    /// A project on the map fixture `map`, copied to `map.omap` next to it.
    pub(super) fn project_on(
        dir: &std::path::Path,
        map: &str,
        change: impl FnOnce(&mut Event),
    ) -> std::path::PathBuf {
        std::fs::copy(
            format!(
                "{}/../../tests/fixtures/maps/{map}",
                env!("CARGO_MANIFEST_DIR")
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
        assert_eq!(body["placeholder"], false);
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

    /// A project with an individual course "A" through control 31 at (100, 100) on the sample map.
    pub(super) fn numbered_project(dir: &std::path::Path) -> std::path::PathBuf {
        use freemapper_courseplan_model::{
            Control, ControlCode, ControlDescription, ControlKind, Course, CourseControl,
            CourseKind, MapPoint, SpecialObject,
        };
        project_with(dir, |event| {
            let at = |x, y| MapPoint { x, y };
            event.specials = vec![
                SpecialObject::Start {
                    id: "S1".to_owned(),
                    ppen_id: None,
                    map_mm: at(50.0, 50.0),
                },
                SpecialObject::Finish {
                    id: "F1".to_owned(),
                    ppen_id: None,
                    map_mm: at(150.0, 150.0),
                },
            ];
            event.controls = vec![Control {
                ppen_id: None,
                code: ControlCode("31".to_owned()),
                kind: ControlKind::Control,
                map_mm: at(100.0, 100.0),
                description: ControlDescription::default(),
                number: None,
            }];
            event.courses = vec![Course {
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
                    controls: vec![CourseControl {
                        code: ControlCode("31".to_owned()),
                        number: None,
                        bends: vec![],
                        taped: false,
                    }],
                },
            }];
        })
    }

    #[tokio::test]
    async fn number_at_inverts_the_overlay_box() {
        let dir = tempfile::tempdir().unwrap();
        let path = numbered_project(dir.path());
        let app = router_for(&path);
        // Whatever the overlay draws for the number is the point the server turns back into a placement.
        let (_, overlay) = get(&app, "/tok/api/overlay?course=A").await;
        let number = &overlay["numbers"][0];
        assert_eq!(number["index"], 1);
        let (cx, cy) = (
            number["centre"][0].as_f64().unwrap(),
            number["centre"][1].as_f64().unwrap(),
        );
        let (status, found) = get(
            &app,
            &format!("/tok/api/number-at?course=A&index=1&x={cx}&y={cy}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{found}");
        let (direction, gap) = (
            found["direction"].as_f64().unwrap(),
            found["gap_mm"].as_f64().unwrap(),
        );
        assert!((0.0..360.0).contains(&direction), "{direction}");
        // Store the answer and read the overlay again: the box lands on the same centre (the 0.01 degree
        // and 0.01 mm rounding of the answer allows a few hundredths of a millimetre).
        crate::mcp::CoursePlanServer::new(Lang::english())
            .call_json(
                &path,
                "set_course_number",
                serde_json::json!({ "name": "A", "after": 1, "direction": direction, "gap_mm": gap }),
            )
            .unwrap();
        let app = router_for(&path);
        let (_, again) = get(&app, "/tok/api/overlay?course=A").await;
        assert_eq!(again["numbers"][0]["source"], "course");
        for axis in [0, 1] {
            let a = again["numbers"][0]["centre"][axis].as_f64().unwrap();
            let b = number["centre"][axis].as_f64().unwrap();
            assert!((a - b).abs() < 0.05, "{axis}: {a} vs {b}");
        }
    }

    #[tokio::test]
    async fn number_at_answers_direction_and_gap_for_a_drop_point() {
        let dir = tempfile::tempdir().unwrap();
        let app = router_for(&numbered_project(dir.path()));
        let (_, overlay) = get(&app, "/tok/api/overlay?course=A").await;
        let number = &overlay["numbers"][0];
        let w = number["w"].as_f64().unwrap();
        let sizes = &overlay["sizes"];
        let outer = sizes["circle_r"].as_f64().unwrap() + sizes["line"].as_f64().unwrap() / 2.0;
        // The box of a drop 1 map mm beyond the circle edge; the exact gap in print mm depends on the map
        // kind (the library test pins 1.0 for the sprint case), so only direction and symmetry are checked.
        let reach = outer + 1.0 + w / 2.0;
        let (_, east) = get(
            &app,
            &format!(
                "/tok/api/number-at?course=A&index=1&x={}&y=100",
                100.0 + reach
            ),
        )
        .await;
        let (_, west) = get(
            &app,
            &format!(
                "/tok/api/number-at?course=A&index=1&x={}&y=100",
                100.0 - reach
            ),
        )
        .await;
        // Hand computation: the box edge is 1 map mm beyond the circle edge. The project's map is a forest
        // map at 1:10000, the overprint sizes apply at 1:15000, so a print mm is 1.5 map mm: 1 / 1.5 = 0.67.
        assert_eq!(east["direction"], 0.0);
        assert_eq!(west["direction"], 180.0);
        assert_eq!(east["gap_mm"], 0.67, "{east}");
        assert_eq!(west["gap_mm"], 0.67, "{west}");
        assert_eq!(east["clamped"], false);
        // Directions run counter-clockwise from east on the printed map; a larger map y lies to the south.
        let q = |x: f64, y: f64| format!("/tok/api/number-at?course=A&index=1&x={x}&y={y}");
        let h = number["h"].as_f64().unwrap();
        let (_, south) = get(&app, &q(100.0, 100.0 + outer + 2.0 + h / 2.0)).await;
        assert_eq!(south["direction"], 270.0, "{south}");
        assert_eq!(south["gap_mm"], 1.33, "{south}");
        // The 0 and 360 wrap: 0.004 degrees counter-clockwise (rounds to 0.0) and clockwise (359.996 rounds
        // to 360, which is 0).
        let dy = reach * 0.004_f64.to_radians().tan();
        let (_, above) = get(&app, &q(100.0 + reach, 100.0 - dy)).await;
        let (_, below) = get(&app, &q(100.0 + reach, 100.0 + dy)).await;
        assert_eq!(above["direction"], 0.0, "{above}");
        assert_eq!(below["direction"], 0.0, "{below}");
    }

    #[tokio::test]
    async fn number_at_keeps_the_gap_inside_the_stored_range() {
        let dir = tempfile::tempdir().unwrap();
        let app = router_for(&numbered_project(dir.path()));
        let q = |x: f64| format!("/tok/api/number-at?course=A&index=1&x={x}&y=100");
        // A box over the circle has a negative gap: the answer is the smallest storable gap, flagged.
        let (status, over) = get(&app, &q(100.5)).await;
        assert_eq!(status, StatusCode::OK, "{over}");
        assert_eq!(over["gap_mm"], 0.01, "{over}");
        assert_eq!(over["clamped"], true, "{over}");
        // Far away: the largest storable gap.
        let (_, far) = get(&app, &q(100.0 + 5000.0)).await;
        assert_eq!(far["gap_mm"], 10.0, "{far}");
        assert_eq!(far["clamped"], true, "{far}");
        // A positive gap that rounds to 0.00 (0.004 mm beyond the circle edge) is raised to 0.01.
        let (_, overlay) = get(&app, "/tok/api/overlay?course=A").await;
        let sizes = &overlay["sizes"];
        let outer = sizes["circle_r"].as_f64().unwrap() + sizes["line"].as_f64().unwrap() / 2.0;
        let w = overlay["numbers"][0]["w"].as_f64().unwrap();
        let (_, tiny) = get(&app, &q(100.0 + outer + 0.004 + w / 2.0)).await;
        assert_eq!(tiny["gap_mm"], 0.01, "{tiny}");
        assert_eq!(tiny["clamped"], true, "{tiny}");
    }

    #[tokio::test]
    async fn rules_route_lists_profiles_overlays_and_categories_without_file_names() {
        let dir = tempfile::tempdir().unwrap();
        let app = router_for(&numbered_project(dir.path()));
        let (status, rules) = get(&app, "/tok/api/rules").await;
        assert_eq!(status, StatusCode::OK, "{rules}");
        assert_eq!(rules["profiles"], serde_json::json!([{ "id": "wkb-2026" }]));
        assert_eq!(
            rules["overlays"],
            serde_json::json!([{ "id": "nor-bb-2021", "base": "wkb-2026" }])
        );
        let categories = rules["categories"]["wkb-2026"].as_array().unwrap();
        assert!(categories.iter().any(|c| c == "HE"), "{rules}");
        let text = rules.to_string();
        assert!(!text.contains(".toml") && !text.contains('/'), "{text}");
    }

    #[tokio::test]
    async fn number_at_unknown_course_is_404_and_bad_queries_are_400() {
        let dir = tempfile::tempdir().unwrap();
        let app = router_for(&numbered_project(dir.path()));
        for query in ["course=Nope&index=1&x=1&y=2", "course=A&index=2&x=1&y=2"] {
            assert_eq!(
                get(&app, &format!("/tok/api/number-at?{query}")).await.0,
                StatusCode::NOT_FOUND,
                "{query}"
            );
        }
        for query in [
            "",
            "course=A&index=1&x=1",
            "course=A&index=0&x=1&y=2",
            "course=A&index=-1&x=1&y=2",
            "course=A&index=1.5&x=1&y=2",
            "course=A&index=1&x=NaN&y=2",
            "course=A&index=1&x=inf&y=2",
            "course=A&index=1&x=1e9&y=2",
            "course=A&index=1&x=1&x=2&y=2",
            "course=A&course=A&index=1&x=1&y=2",
            "course=%zz&index=1&x=1&y=2",
            "course=A&index=1&x=1&y=2&r=3",
            "course=A&index=99999999999999999999&x=1&y=2",
        ] {
            assert_eq!(
                get(&app, &format!("/tok/api/number-at?{query}")).await.0,
                StatusCode::BAD_REQUEST,
                "{query}"
            );
        }
    }

    #[tokio::test]
    async fn near_route_lists_map_objects_and_refuses_bad_queries() {
        let dir = tempfile::tempdir().unwrap();
        let app = router_for(&project(dir.path()));
        let (_, state) = get(&app, "/tok/api/state").await;
        let e = &state["map"]["extent"];
        let (x, y) = (
            (e[0][0].as_f64().unwrap() + e[1][0].as_f64().unwrap()) / 2.0,
            (e[0][1].as_f64().unwrap() + e[1][1].as_f64().unwrap()) / 2.0,
        );
        let (status, body) = get(&app, &format!("/tok/api/map/near?x={x}&y={y}&r=50")).await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body["features"].is_array() && body["total"].is_u64(),
            "{body}"
        );
        let first = &body["features"][0];
        assert!(first["symbol_code"].is_string() && first["distance_mm"].is_number());
        // The same answer as the MCP tool's, which the route shares its code with.
        let tool = crate::mcp::CoursePlanServer::new(Lang::english())
            .call_json(
                &dir.path().join("event.courseplan.toml"),
                "map_features_near",
                serde_json::json!({ "x": x, "y": y, "radius_mm": 50.0 }),
            )
            .unwrap();
        assert_eq!(tool, body);
        for query in [
            "",
            "?x=1&y=2",
            "?x=1&y=2&r=0",
            "?x=1&y=2&r=-3",
            "?x=1&y=2&r=51",
            "?x=1&y=2&r=NaN",
            "?x=inf&y=2&r=3",
            "?x=1&x=2&y=2&r=3",
            "?x=1&y=2&r=3&limit=1",
            "?x=abc&y=2&r=3",
            "?x=1e9&y=2&r=3",
            "?path=/etc/passwd&x=1&y=2&r=3",
        ] {
            assert_eq!(
                get(&app, &format!("/tok/api/map/near{query}")).await.0,
                StatusCode::BAD_REQUEST,
                "{query}"
            );
        }
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
    async fn map_svg_with_a_course_paints_the_renderers_course_layer() {
        let dir = tempfile::tempdir().unwrap();
        let app = router_for(&numbered_project(dir.path()));
        let get = |query: &'static str| {
            let app = app.clone();
            async move {
                raw(
                    &app,
                    Request::get(format!(
                        "/tok/api/map.svg?x0=0&y0=0&x1=400&y1=300&zoom=2{query}"
                    ))
                    .body(Body::empty())
                    .unwrap(),
                )
                .await
            }
        };
        let (plain, _, plain_body) = get("").await;
        let (status, headers, with) = get("&course=A").await;
        assert_eq!((plain, status), (StatusCode::OK, StatusCode::OK), "{with}");
        assert_eq!(headers["content-type"], "image/svg+xml");
        assert!(
            with.len() > plain_body.len(),
            "no course layer in the image"
        );
        // The plain map is unchanged afterwards: the layer lives for one render.
        assert_eq!(get("").await.2, plain_body);
        assert_eq!(get("&course=Nope").await.0, StatusCode::NOT_FOUND);
        assert_eq!(get("&course=A&course=A").await.0, StatusCode::BAD_REQUEST);
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
    async fn a_same_length_same_mtime_rewrite_is_noticed() {
        let dir = tempfile::tempdir().unwrap();
        let path = project(dir.path());
        let app = router_for(&path);
        let (_, first) = get(&app, "/tok/api/revision").await;
        // Same length, same mtime, other bytes: the change time and the edge hash still give it away.
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
        assert_ne!(same["project"], first["project"]);
    }
}

mod history {
    use crate::editor::history::{HistoryError, MAX_BYTES, MAX_STEPS, UndoHistory};

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

    #[test]
    fn history_stays_within_its_byte_budget_and_keeps_the_newest_steps() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.toml");
        let budget = 10 * 1024;
        let mut history = UndoHistory::with_budget(budget);
        let state = |i: usize| format!("{i:04}{}", "x".repeat(1024 - 4));
        for i in 0..50 {
            history.record(format!("s{i}"), state(i), state(i + 1));
            assert!(history.bytes() <= budget, "{} bytes", history.bytes());
        }
        // One chain: about one state per step, not two.
        assert!(history.steps() >= 9, "{} steps", history.steps());
        std::fs::write(&path, state(50)).unwrap();
        let steps_before = history.steps();
        let mut undone = 0;
        while history.undo(&path).is_ok() {
            undone += 1;
            assert_eq!(std::fs::read_to_string(&path).unwrap(), state(50 - undone));
        }
        assert_eq!(undone, steps_before);
        assert!(undone >= 9);
        // Redo walks back to the newest text, byte for byte.
        while history.redo(&path).is_ok() {}
        assert_eq!(std::fs::read_to_string(&path).unwrap(), state(50));
    }

    #[test]
    fn a_step_larger_than_the_budget_is_still_kept() {
        let mut history = UndoHistory::with_budget(10);
        history.record("big".to_owned(), "a".repeat(100), "b".repeat(100));
        assert_eq!(history.steps(), 1);
    }

    #[test]
    fn a_hundred_steps_of_one_mib_stay_under_the_default_budget() {
        let mut history = UndoHistory::default();
        let big = |i: usize| format!("{i:08}{}", "x".repeat(1024 * 1024 - 8));
        for i in 0..100 {
            history.record(format!("s{i}"), big(i), big(i + 1));
        }
        assert!(history.bytes() <= MAX_BYTES, "{} bytes", history.bytes());
        assert!(history.steps() >= 1);
    }
}

mod ops {
    use super::state::{numbered_project, project};
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

    async fn history(
        app: &axum::Router,
        which: &str,
        revision: Option<&str>,
    ) -> (StatusCode, Value) {
        send(
            app,
            "POST",
            &format!("/tok/api/{which}"),
            revision,
            json!({}),
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
    async fn a_bend_outside_the_map_or_too_many_bends_snap_back_as_422() {
        let dir = tempfile::tempdir().unwrap();
        let path = numbered_project(dir.path());
        let app = router_for(&path);
        let first = state(&app).await;
        let revision = first["revision"].as_str().unwrap();
        let bytes = std::fs::read(&path).unwrap();
        for bends in [
            json!([{"x": 1.0e9, "y": 0.0}]),
            json!([{"x": -1000.0, "y": 100.0}]),
            Value::Array(vec![json!({"x": 90.0, "y": 60.0}); 201]),
        ] {
            let (status, body) = op(
                &app,
                "set_leg",
                revision,
                json!({"name": "A", "after": 1, "bends": bends}),
            )
            .await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
            assert!(!body["error"].as_str().unwrap().is_empty());
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
            if bends.as_array().is_some_and(|b| b.len() == 201) {
                assert!(body["error"].as_str().unwrap().contains("200"), "{body}");
            }
        }
        // Inside the map and within the cap, the same operation is stored.
        let (status, body) = op(
            &app,
            "set_leg",
            revision,
            json!({"name": "A", "after": 1, "bends": [{"x": 90.0, "y": 60.0}]}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
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
        let (status, undone) = history(&app, "undo", added["revision"].as_str()).await;
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
        let (status, redone) = history(&app, "redo", undone["revision"].as_str()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(std::fs::read(&path).unwrap(), after);
        assert_eq!(redone["revision"], added["revision"]);
        let (status, _) = history(&app, "redo", redone["revision"].as_str()).await;
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
        let now = state(&app).await;
        let (status, body) = history(&app, "undo", now["revision"].as_str()).await;
        assert_eq!(status, StatusCode::CONFLICT, "{body}");
        assert_eq!(std::fs::read(&path).unwrap(), other);
        let now = state(&app).await;
        assert!(now["undo"]["undo"].is_null() && now["undo"]["redo"].is_null());
        let (status, _) = history(&app, "undo", now["revision"].as_str()).await;
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

    #[tokio::test]
    async fn undo_and_redo_need_the_current_revision() {
        let (_dir, path, app, first) = setup().await;
        let first_rev = first["revision"].as_str().unwrap();
        let (_, added) = op(
            &app,
            "add_control",
            first_rev,
            json!({"at": centre(&first)}),
        )
        .await;
        let after = std::fs::read(&path).unwrap();
        // No header: the caller's fault.
        let (status, _) = history(&app, "undo", None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        // A second tab that still shows the old revision does not revert the first tab's edit.
        let (status, body) = history(&app, "undo", Some(first_rev)).await;
        assert_eq!(status, StatusCode::CONFLICT, "{body}");
        assert_eq!(body["revision"], added["revision"]);
        assert_eq!(std::fs::read(&path).unwrap(), after);
        assert!(state(&app).await["undo"]["undo"].is_string());
        // With the current revision it works, and so does redo, but not with a stale one.
        let (status, undone) = history(&app, "undo", added["revision"].as_str()).await;
        assert_eq!(status, StatusCode::OK);
        let (status, _) = history(&app, "redo", added["revision"].as_str()).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_ne!(std::fs::read(&path).unwrap(), after);
        let (status, _) = history(&app, "redo", undone["revision"].as_str()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(std::fs::read(&path).unwrap(), after);
    }

    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn a_write_between_load_and_save_is_409_and_nothing_is_recorded() {
        let (_dir, path, app, first) = setup().await;
        let before = std::fs::read_to_string(&path).unwrap();
        crate::edit_cmd::TEST_TOUCH
            .lock()
            .unwrap()
            .push(path.clone());
        let (status, body) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": centre(&first)}),
        )
        .await;
        crate::edit_cmd::TEST_TOUCH
            .lock()
            .unwrap()
            .retain(|p| p != &path);
        assert_eq!(status, StatusCode::CONFLICT, "{body}");
        assert!(body["error"].as_str().unwrap().contains("outside"));
        // Only the other program's write is in the file, and the page is told its revision.
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            format!("{before}\n# touched\n")
        );
        assert_eq!(body["revision"], state(&app).await["revision"]);
        assert!(state(&app).await["undo"]["undo"].is_null());
    }

    #[tokio::test]
    async fn a_write_between_the_first_read_and_the_tool_is_409_and_not_recorded() {
        let (_dir, path, app, first) = setup().await;
        let before = std::fs::read_to_string(&path).unwrap();
        super::super::api::RACE.lock().unwrap().push(path.clone());
        let (status, body) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": centre(&first)}),
        )
        .await;
        super::super::api::RACE
            .lock()
            .unwrap()
            .retain(|p| p != &path);
        assert_eq!(status, StatusCode::CONFLICT, "{body}");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            format!("{before}\n# raced\n")
        );
        assert_eq!(controls_in(&path), 0);
        assert!(state(&app).await["undo"]["undo"].is_null());
    }

    #[tokio::test]
    async fn a_failing_write_is_500_with_a_generic_text() {
        let (_dir, path, app, first) = setup().await;
        let bytes = std::fs::read(&path).unwrap();
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        std::fs::create_dir(&tmp).unwrap();
        let (status, body) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": centre(&first)}),
        )
        .await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{body}");
        let error = body["error"].as_str().unwrap();
        assert!(
            !error.contains("tmp") && !error.contains("Is a directory"),
            "{error}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(state(&app).await["undo"]["undo"].is_null());
    }

    #[tokio::test]
    async fn over_long_codes_and_names_are_refused_with_a_text() {
        let (_dir, path, app, first) = setup().await;
        let bytes = std::fs::read(&path).unwrap();
        let (status, body) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": centre(&first), "code": "7".repeat(65)}),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(
            body["error"].as_str().unwrap().contains("at most 64"),
            "{body}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let (status, body) = op(
            &app,
            "add_class",
            first["revision"].as_str().unwrap(),
            json!({"name": "c".repeat(201)}),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(
            body["error"].as_str().unwrap().contains("at most 200"),
            "{body}"
        );
        let (status, _) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"at": centre(&first), "code": "7".repeat(64)}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_field_errors_do_not_name_the_project_field() {
        let (_dir, _path, app, first) = setup().await;
        let (status, body) = op(
            &app,
            "add_control",
            first["revision"].as_str().unwrap(),
            json!({"nope": 1}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let error = body["error"].as_str().unwrap();
        assert!(error.contains("nope") && error.contains("`at`"), "{error}");
        assert!(!error.contains("project"), "{error}");
        let (_, body) = op(
            &app,
            "remove_control",
            first["revision"].as_str().unwrap(),
            json!({"nope": 1}),
        )
        .await;
        assert!(
            !body["error"].as_str().unwrap().contains("project"),
            "{body}"
        );
    }

    #[test]
    fn project_is_stripped_from_expected_field_lists() {
        use crate::editor::api::without_project_field;
        for (raw, want) in [
            (
                "unknown field `x`, expected one of `project`, `at`, `code`",
                "unknown field `x`, expected one of `at`, `code`",
            ),
            (
                "unknown field `x`, expected `project` or `code`",
                "unknown field `x`, expected `code`",
            ),
            (
                "unknown field `x`, expected one of `at`, `code`, `project`",
                "unknown field `x`, expected one of `at`, `code`",
            ),
            ("missing field `at`", "missing field `at`"),
        ] {
            assert_eq!(without_project_field(raw), want);
        }
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

    /// A copy of the numbered project with a second control 1 mm (10 m) from the first on the same course.
    async fn check_of(app: &axum::Router) -> (StatusCode, Value) {
        send(app, "GET", "/tok/api/check", None, json!({})).await
    }

    fn crowded_project(dir: &std::path::Path) -> std::path::PathBuf {
        let path = numbered_project(dir);
        let server = crate::mcp::CoursePlanServer::new(Lang::english());
        server
            .call_json(
                &path,
                "add_control",
                json!({"at": {"x": 90.0, "y": 60.0}, "code": "32"}),
            )
            .unwrap();
        server
            .call_json(
                &path,
                "move_control",
                json!({"code": "31", "at": {"x": 91.0, "y": 60.0}}),
            )
            .unwrap();
        server
            .call_json(
                &path,
                "set_course_controls",
                json!({"name": "A", "codes": ["31", "32"]}),
            )
            .unwrap();
        path
    }

    #[tokio::test]
    async fn check_route_equals_mcp_check() {
        let dir = tempfile::tempdir().unwrap();
        let path = crowded_project(dir.path());
        let app = router_for(&path);
        let (status, mut routed) = check_of(&app).await;
        assert_eq!(status, StatusCode::OK, "{routed}");
        let server = crate::mcp::CoursePlanServer::new(Lang::english());
        let direct = server.call_json(&path, "check", json!({})).unwrap();
        assert_eq!(routed["revision"], state(&app).await["revision"]);
        routed.as_object_mut().unwrap().remove("revision");
        assert_eq!(routed, direct);
        assert!(
            routed["summary"]["errors"].as_u64().unwrap() >= 1,
            "{}",
            routed["summary"]
        );
        // Asked again, the same answer comes from the cache.
        let (_, again) = check_of(&app).await;
        assert_eq!(again["findings"], routed["findings"]);
    }

    #[tokio::test]
    async fn check_of_a_stale_map_reads_the_new_map() {
        let dir = tempfile::tempdir().unwrap();
        let path = crowded_project(dir.path());
        let app = router_for(&path);
        let (status, first) = check_of(&app).await;
        assert_eq!(status, StatusCode::OK);
        let map = dir.path().join("map.omap");
        let good = std::fs::read(&map).unwrap();
        std::fs::write(&map, b"not a map").unwrap();
        let (status, _) = check_of(&app).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        std::fs::write(&map, good).unwrap();
        let (status, again) = check_of(&app).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(again, first);
    }

    #[tokio::test]
    async fn check_follows_an_accepted_finding() {
        let dir = tempfile::tempdir().unwrap();
        let path = crowded_project(dir.path());
        let app = router_for(&path);
        let (_, first) = check_of(&app).await;
        let finding = first["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["severity"] == "error")
            .unwrap()
            .clone();
        let loc = &finding["location"];
        let mut args = json!({"rule": finding["rule"], "note": "on purpose"});
        match loc["kind"].as_str().unwrap() {
            "controls" => args["controls"] = loc["codes"].clone(),
            kind => panic!("unexpected kind {kind}: {loc}"),
        }
        let (status, done) = op(
            &app,
            "accept_finding",
            first["revision"].as_str().unwrap(),
            args,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{done}");
        let (_, after) = check_of(&app).await;
        assert_ne!(after["revision"], first["revision"]);
        assert_eq!(after["accepted"].as_array().unwrap().len(), 1);
        assert_eq!(
            after["summary"]["errors"].as_u64().unwrap() + 1,
            first["summary"]["errors"].as_u64().unwrap()
        );
    }
}

mod exchange {
    use super::state::{numbered_project, project_on, project_with};
    use super::*;
    use axum::http::HeaderMap;
    use serde_json::{Value, json};

    const CHANGED_PPEN: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/ppen/luebeck-sprint-pp4-changed.ppen"
    );

    fn router_for(path: &std::path::Path) -> axum::Router {
        let state = EditorState::new(path.into(), Lang::english());
        router("/tok", Arc::new(Mutex::new(state)), Arc::new(Notify::new()))
    }

    struct Answer {
        status: StatusCode,
        headers: HeaderMap,
        body: Vec<u8>,
    }

    impl Answer {
        fn json(&self) -> Value {
            serde_json::from_slice(&self.body).unwrap_or(Value::Null)
        }
    }

    async fn call(
        app: &axum::Router,
        method: &str,
        uri: &str,
        revision: Option<&str>,
        content_type: &str,
        body: Vec<u8>,
    ) -> Answer {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", content_type);
        if let Some(revision) = revision {
            request = request.header("x-courseplan-revision", revision);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        let (parts, body) = response.into_parts();
        Answer {
            status: parts.status,
            headers: parts.headers,
            body: axum::body::to_bytes(body, usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        }
    }

    async fn revision(app: &axum::Router) -> String {
        let a = call(app, "GET", "/tok/api/state", None, "x/y", Vec::new()).await;
        a.json()["revision"].as_str().unwrap().to_owned()
    }

    async fn export(app: &axum::Router, to: &str, revision: Option<&str>) -> Answer {
        let body = json!({ "to": to }).to_string().into_bytes();
        call(
            app,
            "POST",
            "/tok/api/export/ppen",
            revision,
            "application/json",
            body,
        )
        .await
    }

    async fn upload(
        app: &axum::Router,
        query: &str,
        revision: Option<&str>,
        bytes: Vec<u8>,
    ) -> Answer {
        call(
            app,
            "POST",
            &format!("/tok/api/import/ppen{query}"),
            revision,
            "application/octet-stream",
            bytes,
        )
        .await
    }

    fn linked(event: &mut freemapper_courseplan_model::Event) {
        event.sources.ppen = Some("linked.ppen".into());
    }

    fn luebeck(dir: &std::path::Path) -> std::path::PathBuf {
        project_on(dir, "luebeck.omap", |_| {})
    }

    fn changed_ppen() -> Vec<u8> {
        std::fs::read(CHANGED_PPEN).unwrap()
    }

    fn created_blanked(xml: &str) -> String {
        let at = xml.find("createTime=\"").unwrap() + "createTime=\"".len();
        let end = at + xml[at..].find('"').unwrap();
        format!("{}{}", &xml[..at], &xml[end..])
    }

    #[test]
    fn download_names_are_safe() {
        use super::super::exchange::download_stem;
        let stem = |name: &str| download_stem(std::path::Path::new(&format!("/x/{name}")));
        assert_eq!(stem("event.courseplan.toml"), "event");
        assert_eq!(stem("Lübeck Sprint.toml"), "L_beck_Sprint");
        assert_eq!(stem("a\"b\r\n.toml"), "a_b__");
        assert_eq!(stem("..toml"), "_");
        assert_eq!(stem(".toml"), "event");
        assert_eq!(stem(&format!("{}.toml", "x".repeat(300))).len(), 80);
    }

    #[tokio::test]
    async fn iof_xml_download_equals_cli_export() {
        let dir = tempfile::tempdir().unwrap();
        let path = numbered_project(dir.path());
        let app = router_for(&path);
        let before = std::fs::read(&path).unwrap();
        let got = call(
            &app,
            "GET",
            "/tok/api/export/iof-xml",
            None,
            "x/y",
            Vec::new(),
        )
        .await;
        assert_eq!(
            got.status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&got.body)
        );
        assert_eq!(got.headers["content-type"], "application/xml");
        assert_eq!(
            got.headers["content-disposition"],
            "attachment; filename=\"event.xml\""
        );
        assert_eq!(got.headers["x-courseplan-courses"], "1");
        let target = dir.path().join("cli.xml");
        crate::export_cmd::export_iof_xml_file(&path, &target, "T").unwrap();
        let cli = std::fs::read_to_string(&target).unwrap();
        let served = String::from_utf8(got.body).unwrap();
        assert_eq!(created_blanked(&served), created_blanked(&cli));
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[tokio::test]
    async fn ppen_to_linked_equals_cli_export() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let link_it = |path: &std::path::Path| {
            let mut file = freemapper_courseplan_model::ProjectFile::load(path).unwrap();
            linked(&mut file.event);
            file.save().unwrap();
        };
        let (pa, pb) = (numbered_project(a.path()), numbered_project(b.path()));
        link_it(&pa);
        link_it(&pb);
        let app = router_for(&pa);
        // Both start from the same text.
        assert_eq!(std::fs::read(&pa).unwrap(), std::fs::read(&pb).unwrap());
        let rev = revision(&app).await;
        let got = export(&app, "linked", Some(&rev)).await;
        assert_eq!(got.status, StatusCode::OK, "{}", got.json());
        let report = crate::export_cmd::export_ppen_file(
            &pb,
            &b.path().join("linked.ppen"),
            false,
            &Lang::english(),
        )
        .unwrap();
        assert_eq!(
            std::fs::read(a.path().join("linked.ppen")).unwrap(),
            std::fs::read(b.path().join("linked.ppen")).unwrap()
        );
        assert_eq!(std::fs::read(&pa).unwrap(), std::fs::read(&pb).unwrap());
        let answer = got.json();
        assert_eq!(answer["courses"], report.courses);
        assert_eq!(answer["ids_stored"], report.ids_stored);
        assert!(report.ids_stored > 0);
        assert_ne!(answer["revision"].as_str().unwrap(), rev);
        assert_eq!(answer["revision"], revision(&app).await);
        assert!(answer["notes"].is_array());
    }

    #[tokio::test]
    async fn ppen_to_linked_patches_an_existing_file_and_keeps_ids() {
        let dir = tempfile::tempdir().unwrap();
        let path = project_with(dir.path(), linked);
        let app = router_for(&path);
        let rev = revision(&app).await;
        let first = export(&app, "linked", Some(&rev)).await;
        assert_eq!(first.status, StatusCode::OK);
        assert_eq!(first.json()["patched"], false);
        let again = export(&app, "linked", Some(&revision(&app).await)).await;
        assert_eq!(again.status, StatusCode::OK);
        assert_eq!(again.json()["patched"], true);
    }

    #[tokio::test]
    async fn ppen_download_writes_nothing_and_stores_no_ids() {
        let dir = tempfile::tempdir().unwrap();
        let path = numbered_project(dir.path());
        let app = router_for(&path);
        let names = |d: &std::path::Path| {
            let mut v: Vec<_> = std::fs::read_dir(d)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            v.sort();
            v
        };
        let (before, files) = (std::fs::read(&path).unwrap(), names(dir.path()));
        let got = export(&app, "download", None).await;
        assert_eq!(got.status, StatusCode::OK, "{}", got.json());
        let answer = got.json();
        assert_eq!(answer["file"], "event.ppen");
        assert_eq!(answer["ids_stored"], 0);
        assert_eq!(answer["linked_now"], false);
        let text = answer["text"].as_str().unwrap();
        assert!(
            text.contains("<course-scribe-event"),
            "{}",
            &text[..80.min(text.len())]
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(names(dir.path()), files);
        assert!(!std::fs::read_to_string(&path).unwrap().contains("ppen_id"));
    }

    #[tokio::test]
    async fn export_to_linked_without_a_link_is_422() {
        let dir = tempfile::tempdir().unwrap();
        let path = numbered_project(dir.path());
        let app = router_for(&path);
        let (before, rev) = (std::fs::read(&path).unwrap(), revision(&app).await);
        let got = export(&app, "linked", Some(&rev)).await;
        assert_eq!(got.status, StatusCode::UNPROCESSABLE_ENTITY);
        let want =
            freemapper_courseplan::text::text(&Lang::english(), "editor-export-no-link", &[]);
        assert_eq!(got.json()["error"], want);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
        // The browser names nothing but the target kind.
        let bad = export(&app, "/tmp/x.ppen", Some(&rev)).await;
        assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn export_to_linked_needs_the_current_revision_and_a_safe_link() {
        let dir = tempfile::tempdir().unwrap();
        let path = project_with(dir.path(), linked);
        let app = router_for(&path);
        let rev = revision(&app).await;
        assert_eq!(
            export(&app, "linked", None).await.status,
            StatusCode::BAD_REQUEST
        );
        let stale = export(&app, "linked", Some("0000000000000000")).await;
        assert_eq!(stale.status, StatusCode::CONFLICT);
        assert!(!dir.path().join("linked.ppen").exists());
        // A link that does not end in .ppen is refused, and so is a symbolic link.
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, text.replace("linked.ppen", "linked.txt")).unwrap();
        let rev2 = revision(&app).await;
        assert_ne!(rev, rev2);
        let got = export(&app, "linked", Some(&rev2)).await;
        assert_eq!(got.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(!dir.path().join("linked.txt").exists());
        #[cfg(unix)]
        {
            let other = tempfile::tempdir().unwrap();
            std::fs::write(&path, text).unwrap();
            let victim = other.path().join("victim.ppen");
            std::fs::write(&victim, "keep").unwrap();
            std::os::unix::fs::symlink(&victim, dir.path().join("linked.ppen")).unwrap();
            let rev3 = revision(&app).await;
            let got = export(&app, "linked", Some(&rev3)).await;
            assert_eq!(got.status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(std::fs::read_to_string(&victim).unwrap(), "keep");
        }
    }

    #[tokio::test]
    async fn import_upload_equals_cli_import_except_link() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let (pa, pb) = (luebeck(a.path()), luebeck(b.path()));
        std::fs::write(b.path().join("in.ppen"), changed_ppen()).unwrap();
        let app = router_for(&pa);
        let rev = revision(&app).await;
        let got = upload(&app, "", Some(&rev), changed_ppen()).await;
        assert_eq!(got.status, StatusCode::OK, "{}", got.json());
        let report =
            crate::import_cmd::import_ppen_file(&pb, &b.path().join("in.ppen"), false).unwrap();
        let answer = got.json();
        assert_eq!(answer["controls"], report.controls);
        assert_eq!(answer["courses"], report.courses);
        assert_eq!(
            answer["notes"].as_array().unwrap().len(),
            report.notes.len()
        );
        assert_eq!(answer["revision"], revision(&app).await);
        let fa = freemapper_courseplan_model::ProjectFile::load(&pa).unwrap();
        let mut fb = freemapper_courseplan_model::ProjectFile::load(&pb).unwrap();
        assert!(fa.event.sources.ppen.is_none());
        assert!(fb.event.sources.ppen.is_some());
        fb.event.sources.ppen = None;
        assert_eq!(fa.event, fb.event);
        assert!(!fa.event.controls.is_empty());
    }

    #[tokio::test]
    async fn import_keeps_an_existing_link() {
        let dir = tempfile::tempdir().unwrap();
        let path = project_on(dir.path(), "luebeck.omap", linked);
        let app = router_for(&path);
        let got = upload(&app, "", Some(&revision(&app).await), changed_ppen()).await;
        assert_eq!(got.status, StatusCode::OK, "{}", got.json());
        let file = freemapper_courseplan_model::ProjectFile::load(&path).unwrap();
        assert_eq!(
            file.event.sources.ppen.as_deref(),
            Some(std::path::Path::new("linked.ppen"))
        );
        assert!(!dir.path().join("linked.ppen").exists());
    }

    #[tokio::test]
    async fn import_without_replace_on_a_full_project_is_422() {
        let dir = tempfile::tempdir().unwrap();
        let path = luebeck(dir.path());
        let app = router_for(&path);
        let first = upload(
            &app,
            "?replace=false",
            Some(&revision(&app).await),
            changed_ppen(),
        )
        .await;
        assert_eq!(first.status, StatusCode::OK);
        let (before, rev) = (std::fs::read(&path).unwrap(), revision(&app).await);
        let got = upload(&app, "?replace=false", Some(&rev), changed_ppen()).await;
        assert_eq!(got.status, StatusCode::UNPROCESSABLE_ENTITY);
        let want =
            freemapper_courseplan::text::text(&Lang::english(), "editor-import-needs-replace", &[]);
        assert_eq!(got.json()["error"], want);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let replaced = upload(&app, "?replace=true", Some(&rev), changed_ppen()).await;
        assert_eq!(replaced.status, StatusCode::OK, "{}", replaced.json());
    }

    #[tokio::test]
    async fn import_requests_are_checked() {
        let dir = tempfile::tempdir().unwrap();
        let path = luebeck(dir.path());
        let app = router_for(&path);
        let (before, rev) = (std::fs::read(&path).unwrap(), revision(&app).await);
        assert_eq!(
            upload(&app, "", None, changed_ppen()).await.status,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            upload(&app, "", Some("0000000000000000"), changed_ppen())
                .await
                .status,
            StatusCode::CONFLICT
        );
        for query in [
            "?replace=yes",
            "?replace=true&replace=false",
            "?path=/etc/passwd",
            "?replace",
        ] {
            let got = upload(&app, query, Some(&rev), changed_ppen()).await;
            assert_eq!(got.status, StatusCode::BAD_REQUEST, "{query}");
        }
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[tokio::test]
    async fn upload_over_limit_is_413() {
        let dir = tempfile::tempdir().unwrap();
        let path = luebeck(dir.path());
        let app = router_for(&path);
        let (before, rev) = (std::fs::read(&path).unwrap(), revision(&app).await);
        let limit = freemapper_courseplan::ppen_import::MAX_COURSE_FILE_BYTES;
        let got = upload(&app, "", Some(&rev), vec![b' '; limit + 1]).await;
        assert_eq!(got.status, StatusCode::PAYLOAD_TOO_LARGE);
        let want =
            freemapper_courseplan::text::text(&Lang::english(), "editor-import-too-large", &[]);
        assert_eq!(got.json()["error"], want);
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[tokio::test]
    async fn a_hostile_upload_gives_a_clean_error_and_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = luebeck(dir.path());
        let app = router_for(&path);
        let (before, rev) = (std::fs::read(&path).unwrap(), revision(&app).await);
        let deep = format!("<course-scribe-event>{}", "<a>".repeat(200_000)).into_bytes();
        let png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR secret-marker".to_vec();
        let hostile: Vec<(&str, Vec<u8>)> = vec![
            (
                "garbage",
                b"\xff\xfe\x00 not xml at all secret-marker".to_vec(),
            ),
            ("deep", deep),
            ("png", png),
            ("empty", Vec::new()),
            (
                "doctype",
                b"<!DOCTYPE x [<!ENTITY a \"secret-marker\">]><x>&a;</x>".to_vec(),
            ),
            (
                "other xml",
                b"<html><body>secret-marker</body></html>".to_vec(),
            ),
        ];
        let want =
            freemapper_courseplan::text::text(&Lang::english(), "editor-import-invalid", &[]);
        for (what, bytes) in hostile {
            let got = upload(&app, "", Some(&rev), bytes).await;
            assert_eq!(got.status, StatusCode::UNPROCESSABLE_ENTITY, "{what}");
            let text = got.json()["error"].as_str().unwrap().to_owned();
            assert_eq!(text, want, "{what}");
            assert!(!text.contains("secret-marker"));
            assert_eq!(std::fs::read(&path).unwrap(), before, "{what}");
        }
    }

    #[tokio::test]
    async fn exchange_clears_history() {
        let dir = tempfile::tempdir().unwrap();
        let path = project_on(dir.path(), "luebeck.omap", linked);
        let app = router_for(&path);
        let undo_label = |app: &axum::Router| {
            let app = app.clone();
            async move {
                let a = call(&app, "GET", "/tok/api/state", None, "x/y", Vec::new()).await;
                a.json()["undo"]["undo"].clone()
            }
        };
        let step = |app: &axum::Router, code: &str| {
            let app = app.clone();
            let body = json!({"at": {"x": 1.0, "y": 2.0}, "code": code})
                .to_string()
                .into_bytes();
            async move {
                let rev = revision(&app).await;
                let a = call(
                    &app,
                    "POST",
                    "/tok/api/op/add_control",
                    Some(&rev),
                    "application/json",
                    body,
                )
                .await;
                assert_eq!(a.status, StatusCode::OK, "{}", a.json());
            }
        };
        step(&app, "41").await;
        assert!(undo_label(&app).await.is_string());
        let got = export(&app, "linked", Some(&revision(&app).await)).await;
        assert_eq!(got.status, StatusCode::OK, "{}", got.json());
        assert!(undo_label(&app).await.is_null());
        let undo = call(
            &app,
            "POST",
            "/tok/api/undo",
            Some(&revision(&app).await),
            "application/json",
            b"{}".to_vec(),
        )
        .await;
        assert_eq!(undo.status, StatusCode::CONFLICT);
        step(&app, "42").await;
        assert!(undo_label(&app).await.is_string());
        let got = upload(
            &app,
            "?replace=true",
            Some(&revision(&app).await),
            changed_ppen(),
        )
        .await;
        assert_eq!(got.status, StatusCode::OK, "{}", got.json());
        assert!(undo_label(&app).await.is_null());
    }

    #[tokio::test]
    async fn a_write_between_the_read_and_the_save_is_409() {
        let dir = tempfile::tempdir().unwrap();
        let path = luebeck(dir.path());
        let app = router_for(&path);
        let rev = revision(&app).await;
        // Another program writes after the page's revision was taken.
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("{text}\n# outside\n")).unwrap();
        let got = upload(&app, "", Some(&rev), changed_ppen()).await;
        assert_eq!(got.status, StatusCode::CONFLICT);
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .ends_with("# outside\n")
        );
    }
}

mod sync_panel {
    use super::*;
    use serde_json::{Value, json};
    use std::path::{Path, PathBuf};

    const P31: &str = "map_mm = [-50.1, -3.6]";
    const Q31: &str = "<location x=\"-50.1\" y=\"3.6\" />";

    fn router_for(path: &Path) -> axum::Router {
        let state = EditorState::new(path.into(), Lang::english());
        router("/tok", Arc::new(Mutex::new(state)), Arc::new(Notify::new()))
    }

    async fn call(
        app: &axum::Router,
        method: &str,
        uri: &str,
        revision: Option<&str>,
        body: Vec<u8>,
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
            .oneshot(request.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        let (parts, body) = response.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        (
            parts.status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn revision(app: &axum::Router) -> String {
        let (_, state) = call(app, "GET", "/tok/api/state", None, Vec::new()).await;
        state["revision"].as_str().unwrap().to_owned()
    }

    async fn sync(app: &axum::Router, body: Value, revision: Option<&str>) -> (StatusCode, Value) {
        call(
            app,
            "POST",
            "/tok/api/sync",
            revision,
            body.to_string().into_bytes(),
        )
        .await
    }

    async fn undo_label(app: &axum::Router) -> Value {
        call(app, "GET", "/tok/api/state", None, Vec::new()).await.1["undo"]["undo"].clone()
    }

    fn edit(path: &Path, from: &str, to: &str) {
        let text = std::fs::read_to_string(path).unwrap();
        assert!(text.contains(from), "{from}");
        std::fs::write(path, text.replacen(from, to, 1)).unwrap();
    }

    /// The Luebeck example, linked to `out.ppen` and synced once (the base copy exists).
    fn synced(dir: &Path) -> PathBuf {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        std::fs::copy(
            format!("{root}/tests/fixtures/maps/luebeck.omap"),
            dir.join("luebeck.omap"),
        )
        .unwrap();
        let text = std::fs::read_to_string(format!(
            "{root}/examples/luebeck-sprint/luebeck-sprint.courseplan.toml"
        ))
        .unwrap()
        .replace("../../tests/fixtures/maps/luebeck.omap", "luebeck.omap");
        let project = dir.join("event.courseplan.toml");
        std::fs::write(&project, text).unwrap();
        crate::export_cmd::export_ppen_file(
            &project,
            &dir.join("out.ppen"),
            false,
            &Lang::english(),
        )
        .unwrap();
        crate::sync_cmd::sync_file(&project, false, &[]).unwrap();
        project
    }

    /// Both sides move control 31: a conflict.
    fn conflict_pair(dir: &Path) -> PathBuf {
        let project = synced(dir);
        edit(&project, P31, "map_mm = [-45.0, -3.6]");
        edit(
            &dir.join("out.ppen"),
            Q31,
            "<location x=\"-55.0\" y=\"3.6\" />",
        );
        project
    }

    fn files(dir: &Path) -> [Option<Vec<u8>>; 3] {
        let project = dir.join("event.courseplan.toml");
        [
            std::fs::read(dir.join("out.ppen")).ok(),
            std::fs::read(&project).ok(),
            std::fs::read(freemapper_courseplan::sync::base::base_path(&project)).ok(),
        ]
    }

    #[tokio::test]
    async fn sync_dry_run_lists_conflicts_like_mcp() {
        let dir = tempfile::tempdir().unwrap();
        let project = conflict_pair(dir.path());
        let app = router_for(&project);
        let before = files(dir.path());
        let (status, mut got) = sync(&app, json!({"dry_run": true, "take": null}), None).await;
        assert_eq!(status, StatusCode::OK, "{got}");
        assert_eq!(files(dir.path()), before, "a dry run writes nothing");
        assert_eq!(got["revision"], revision(&app).await);
        got.as_object_mut().unwrap().remove("revision");
        let server = crate::mcp::CoursePlanServer::new(Lang::english());
        let mcp = server
            .call_json(&project, "sync_ppen", json!({"dry_run": true}))
            .unwrap();
        assert_eq!(got, mcp);
        assert_eq!(got["conflicts"].as_array().unwrap().len(), 1);
        assert_eq!(got["conflicts"][0]["key"], "control/31/position");
        assert_eq!(got["written"], false);
    }

    #[tokio::test]
    async fn resolve_then_sync_writes_all_three_files() {
        let dir = tempfile::tempdir().unwrap();
        let project = conflict_pair(dir.path());
        let app = router_for(&project);
        let before = files(dir.path());
        // Open conflicts: a real sync writes nothing.
        let rev = revision(&app).await;
        let (status, open) = sync(&app, json!({"dry_run": false, "take": null}), Some(&rev)).await;
        assert_eq!(status, StatusCode::OK, "{open}");
        assert_eq!(open["written"], false);
        assert_eq!(open["conflicts"].as_array().unwrap().len(), 1);
        assert_eq!(files(dir.path()), before);
        // Choose Purple Pen through the operation.
        let key = open["conflicts"][0]["key"].clone();
        let (status, chosen) = call(
            &app,
            "POST",
            "/tok/api/op/resolve_sync_conflict",
            Some(&rev),
            json!({"key": key, "take": "theirs"})
                .to_string()
                .into_bytes(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{chosen}");
        assert!(chosen["result"]["conflicts"].as_array().unwrap().is_empty());
        let rev = chosen["revision"].as_str().unwrap().to_owned();
        assert_eq!(rev, revision(&app).await);
        assert!(undo_label(&app).await.is_string());
        let (status, done) = sync(&app, json!({"dry_run": false, "take": null}), Some(&rev)).await;
        assert_eq!(status, StatusCode::OK, "{done}");
        assert_eq!(done["written"], true, "{done}");
        let after = files(dir.path());
        assert!(
            after[1] != before[1] && after[2].is_some(),
            "project and base"
        );
        let text = String::from_utf8(after[1].clone().unwrap()).unwrap();
        assert!(
            text.contains("map_mm = [-55.0, -3.6]"),
            "Purple Pen's position"
        );
        assert!(!text.contains("decisions"), "stored choices are gone");
        // The linked file keeps Purple Pen's position as well.
        let ppen = String::from_utf8(after[0].clone().unwrap()).unwrap();
        assert!(ppen.contains("<location x=\"-55.0\" y=\"3.6\" />"));
        assert_eq!(done["revision"], revision(&app).await);
        assert!(undo_label(&app).await.is_null(), "the sync clears history");
        // Nothing left to do.
        let (_, again) = sync(&app, json!({"dry_run": true, "take": null}), None).await;
        assert!(again["conflicts"].as_array().unwrap().is_empty());
        assert!(again["changes"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn take_applies_to_every_conflict_of_the_call() {
        let dir = tempfile::tempdir().unwrap();
        let project = conflict_pair(dir.path());
        let app = router_for(&project);
        let rev = revision(&app).await;
        let (status, done) =
            sync(&app, json!({"dry_run": false, "take": "ours"}), Some(&rev)).await;
        assert_eq!(status, StatusCode::OK, "{done}");
        assert_eq!(done["written"], true);
        let ppen = std::fs::read_to_string(dir.path().join("out.ppen")).unwrap();
        assert!(
            ppen.contains("<location x=\"-45\" y=\"3.6\" />"),
            "ours wins: {ppen}"
        );
    }

    #[tokio::test]
    async fn sync_without_link_is_422_with_text() {
        let dir = tempfile::tempdir().unwrap();
        let project = state::project(dir.path());
        let app = router_for(&project);
        let before = std::fs::read(&project).unwrap();
        let rev = revision(&app).await;
        for body in [
            json!({"dry_run": true, "take": null}),
            json!({"dry_run": false, "take": null}),
        ] {
            let (status, got) = sync(&app, body, Some(&rev)).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{got}");
            let want =
                freemapper_courseplan::text::text(&Lang::english(), "editor-sync-no-link", &[]);
            assert_eq!(got["error"], want);
        }
        assert_eq!(std::fs::read(&project).unwrap(), before);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn symlinked_link_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let project = conflict_pair(dir.path());
        let real = dir.path().join("real.ppen");
        std::fs::rename(dir.path().join("out.ppen"), &real).unwrap();
        std::os::unix::fs::symlink(&real, dir.path().join("out.ppen")).unwrap();
        let app = router_for(&project);
        let (before, rev) = (std::fs::read(&real).unwrap(), revision(&app).await);
        for body in [
            json!({"dry_run": true, "take": null}),
            json!({"dry_run": false, "take": "theirs"}),
        ] {
            let (status, got) = sync(&app, body, Some(&rev)).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{got}");
            let want = freemapper_courseplan::text::text(
                &Lang::english(),
                "editor-export-link-refused",
                &[],
            );
            assert_eq!(got["error"], want);
        }
        // The choice goes through the same check.
        let (status, _) = call(
            &app,
            "POST",
            "/tok/api/op/resolve_sync_conflict",
            Some(&rev),
            json!({"key": "control/31/position", "take": "theirs"})
                .to_string()
                .into_bytes(),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(std::fs::read(&real).unwrap(), before);
    }

    #[tokio::test]
    async fn bad_requests_are_400_and_a_real_sync_needs_the_revision() {
        let dir = tempfile::tempdir().unwrap();
        let project = conflict_pair(dir.path());
        let app = router_for(&project);
        let before = files(dir.path());
        let rev = revision(&app).await;
        for body in [
            "{}",
            "not json",
            r#"{"dry_run": true, "take": "both"}"#,
            r#"{"dry_run": true, "take": null, "path": "/etc/passwd"}"#,
            r#"{"dry_run": "yes", "take": null}"#,
        ] {
            let (status, _) = call(
                &app,
                "POST",
                "/tok/api/sync",
                Some(&rev),
                body.as_bytes().to_vec(),
            )
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        }
        let (status, _) = sync(&app, json!({"dry_run": false, "take": "ours"}), None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let (status, _) = sync(
            &app,
            json!({"dry_run": false, "take": "ours"}),
            Some("0000000000000000"),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(files(dir.path()), before);
    }

    #[tokio::test]
    async fn a_purple_pen_save_during_the_sync_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let project = conflict_pair(dir.path());
        let app = router_for(&project);
        let ppen = dir.path().join("out.ppen");
        let rev = revision(&app).await;
        super::super::super::sync_cmd::TOUCH
            .lock()
            .unwrap()
            .push(ppen.clone());
        let (status, got) = sync(
            &app,
            json!({"dry_run": false, "take": "theirs"}),
            Some(&rev),
        )
        .await;
        super::super::super::sync_cmd::TOUCH
            .lock()
            .unwrap()
            .retain(|p| p != &ppen);
        assert_eq!(status, StatusCode::CONFLICT, "{got}");
        assert!(got["error"].as_str().unwrap().contains("out.ppen"));
        assert!(
            !got["error"]
                .as_str()
                .unwrap()
                .contains(dir.path().to_str().unwrap())
        );
        let text = std::fs::read_to_string(&project).unwrap();
        assert!(text.contains("map_mm = [-45.0, -3.6]"), "project untouched");
    }

    #[tokio::test]
    async fn a_project_write_during_the_sync_is_409_and_the_revision_moves_on() {
        let dir = tempfile::tempdir().unwrap();
        let project = synced(dir.path());
        // Only Purple Pen moves control 31: a clean sync.
        edit(
            &dir.path().join("out.ppen"),
            Q31,
            "<location x=\"-55.0\" y=\"3.6\" />",
        );
        let app = router_for(&project);
        let rev = revision(&app).await;
        super::super::super::sync_cmd::TOUCH
            .lock()
            .unwrap()
            .push(project.clone());
        let (status, got) = sync(&app, json!({"dry_run": false, "take": null}), Some(&rev)).await;
        super::super::super::sync_cmd::TOUCH
            .lock()
            .unwrap()
            .retain(|p| p != &project);
        assert_eq!(status, StatusCode::CONFLICT, "{got}");
        let stale = freemapper_courseplan::text::text(&Lang::english(), "editor-op-stale", &[]);
        assert_eq!(got["error"], stale);
        assert_ne!(
            got["revision"], rev,
            "the answer carries the file as it is now"
        );
        assert_eq!(got["revision"], revision(&app).await);
    }

    /// The Luebeck example linked to `out.ppen`, synced once, then the base copy deleted.
    fn unsynced(dir: &Path) -> PathBuf {
        let project = synced(dir);
        std::fs::remove_file(freemapper_courseplan::sync::base::base_path(&project)).unwrap();
        project
    }

    #[tokio::test]
    async fn the_first_sync_writes_the_base_copy_and_keeps_the_undo_history() {
        let dir = tempfile::tempdir().unwrap();
        let project = unsynced(dir.path());
        let base = freemapper_courseplan::sync::base::base_path(&project);
        assert!(!base.exists());
        let app = router_for(&project);
        // An edit and its undo leave the project as it was and a redo step in the history.
        let rev = revision(&app).await;
        let (status, op) = call(
            &app,
            "POST",
            "/tok/api/op/add_control",
            Some(&rev),
            json!({"at": {"x": 1.0, "y": 2.0}, "code": "41"})
                .to_string()
                .into_bytes(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{op}");
        let rev = revision(&app).await;
        let (status, undone) =
            call(&app, "POST", "/tok/api/undo", Some(&rev), b"{}".to_vec()).await;
        assert_eq!(status, StatusCode::OK, "{undone}");
        let redo = |app: axum::Router| async move {
            call(&app, "GET", "/tok/api/state", None, Vec::new())
                .await
                .1["undo"]["redo"]
                .clone()
        };
        assert!(redo(app.clone()).await.is_string());
        let (_, dry) = sync(&app, json!({"dry_run": true, "take": null}), None).await;
        assert_ne!(dry["base"], "use", "{dry}");
        assert!(dry["changes"].as_array().unwrap().is_empty(), "{dry}");
        assert!(dry["conflicts"].as_array().unwrap().is_empty(), "{dry}");
        let before = std::fs::read(&project).unwrap();
        let rev = revision(&app).await;
        let (status, done) = sync(&app, json!({"dry_run": false, "take": null}), Some(&rev)).await;
        assert_eq!(status, StatusCode::OK, "{done}");
        assert_eq!(done["written"], true, "{done}");
        assert!(base.exists(), "the base copy is written");
        assert!(std::fs::read(&project).unwrap() == before, "base copy only");
        assert!(
            redo(app.clone()).await.is_string(),
            "the project text did not change: the history stays"
        );
    }

    #[tokio::test]
    async fn a_missing_link_target_has_a_fixed_text_without_a_path() {
        let dir = tempfile::tempdir().unwrap();
        let project = synced(dir.path());
        std::fs::remove_file(dir.path().join("out.ppen")).unwrap();
        let app = router_for(&project);
        let (status, got) = sync(&app, json!({"dry_run": true, "take": null}), None).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{got}");
        let want =
            freemapper_courseplan::text::text(&Lang::english(), "editor-sync-link-missing", &[]);
        assert_eq!(got["error"], want);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn a_symlinked_directory_in_the_link_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let project = synced(dir.path());
        let real = dir.path().join("real");
        std::fs::create_dir(&real).unwrap();
        std::fs::rename(dir.path().join("out.ppen"), real.join("out.ppen")).unwrap();
        std::os::unix::fs::symlink(&real, dir.path().join("sub")).unwrap();
        edit(&project, "out.ppen", "sub/out.ppen");
        let app = router_for(&project);
        let (status, got) = sync(&app, json!({"dry_run": true, "take": null}), None).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{got}");
        let want =
            freemapper_courseplan::text::text(&Lang::english(), "editor-export-link-refused", &[]);
        assert_eq!(got["error"], want);
    }
}

mod render {
    use super::*;
    use crate::editor::map_view::{FileStamp, MapView, PageRect};
    use crate::editor::render_map::RenderMap;
    use freemapper_courseplan::map::{FeatureGeometry, MapIndex};
    use freemapper_courseplan::omap::read::read_map_file;

    fn luebeck_path() -> String {
        format!(
            "{}/../../tests/fixtures/maps/luebeck.omap",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    fn luebeck() -> (freemapper_courseplan::omap::MapDocument, RenderMap) {
        let doc = read_map_file(std::path::Path::new(&luebeck_path())).unwrap();
        let map = RenderMap::new(&doc, &MapIndex::from_document(&doc)).unwrap();
        (doc, map)
    }

    fn rect_of(map: &RenderMap) -> PageRect {
        map.page_box()
    }

    #[test]
    fn render_mapping_round_trips() {
        let (_, map) = luebeck();
        let m = map.mapping();
        assert_eq!(m.k, 1.0);
        for (x, y) in [(0.0, 0.0), (-12.25, 33.5), (234.5, -87.25)] {
            let (px, py) = m.to_page(freemapper_courseplan_model::MapPoint { x, y });
            let back = m.to_map(px, py);
            assert!((back.x - x).abs() < 1e-9 && (back.y - y).abs() < 1e-9);
        }
    }

    #[test]
    fn render_overlay_aligns_with_map() {
        let (doc, map) = luebeck();
        let index = MapIndex::from_document(&doc);
        let mut checked = 0;
        for feature in index.features() {
            let (FeatureGeometry::Line(points), Some(code)) =
                (&feature.geometry, &feature.symbol_code)
            else {
                continue;
            };
            let p = &points[points.len() / 2];
            let at = map.mapping().to_page(p.clone());
            let hits = map.hit_at(at, 0.1);
            assert!(
                hits.iter().any(|h| &h.symbol_code == code),
                "line symbol {code} at {p:?} is not under its own page position {at:?}: {hits:?}"
            );
            checked += 1;
            if checked == 300 {
                break;
            }
        }
        assert!(checked >= 10, "only {checked} line objects");
    }

    /// The Lübeck map with every object coordinate multiplied by `factor`: a map that does not fit on A3.
    pub(super) fn scaled_luebeck_text(factor: i64) -> String {
        let text = std::fs::read_to_string(luebeck_path()).unwrap();
        let mut out = String::with_capacity(text.len());
        let mut rest = text.as_str();
        while let Some(at) = rest.find("<coords") {
            let open_end = at + rest[at..].find('>').unwrap() + 1;
            let close = open_end + rest[open_end..].find("</coords>").unwrap();
            out.push_str(&rest[..open_end]);
            for entry in rest[open_end..close].split_inclusive(';') {
                let mut parts = entry.split(' ');
                let (x, y) = (parts.next().unwrap(), parts.next());
                match (x.parse::<i64>(), y.map(|y| y.trim_end_matches(';'))) {
                    (Ok(x), Some(y)) if y.parse::<i64>().is_ok() => {
                        let y: i64 = y.parse().unwrap();
                        let tail: Vec<&str> = parts.collect();
                        out.push_str(&format!("{} {}", x * factor, y * factor));
                        if !tail.is_empty() {
                            out.push(' ');
                            out.push_str(&tail.join(" "));
                        }
                        if entry.ends_with(';') && !tail.last().is_some_and(|t| t.ends_with(';')) {
                            out.push(';');
                        }
                    }
                    _ => out.push_str(entry),
                }
            }
            rest = &rest[close..];
        }
        out.push_str(rest);
        out
    }

    #[test]
    fn a_map_larger_than_a3_is_reported_as_clipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.omap");
        std::fs::write(&path, scaled_luebeck_text(6)).unwrap();
        let doc = read_map_file(&path).unwrap();
        let map = RenderMap::new(&doc, &MapIndex::from_document(&doc)).unwrap();
        assert!(map.is_clipped());
        assert!(MapView::is_clipped(&map));
        let (_, plain) = luebeck();
        assert!(!MapView::is_clipped(&plain));
    }

    #[test]
    fn renderer_limits_set_the_incomplete_flag() {
        use freemapper_render::build::BuildReport;
        let report = |f: fn(&mut BuildReport)| {
            let mut r = BuildReport {
                page_mm: (420.0, 297.0),
                geometry_factor: 1.0,
                symbol_factor: 1.0,
                overflow_mm: None,
                skipped_objects: 7,
                unknown_type_objects: 0,
                truncated_objects: 0,
                objects_not_drawn_page_limit: 0,
                dropped_pattern_objects: 0,
                substituted_fonts: Vec::new(),
                missing_glyphs: Vec::new(),
            };
            f(&mut r);
            super::super::render_map::limits_of(&r)
        };
        // Hidden symbols are silent on purpose.
        assert_eq!(report(|_| {}), (false, false));
        assert_eq!(report(|r| r.truncated_objects = 1), (false, true));
        assert_eq!(
            report(|r| r.objects_not_drawn_page_limit = 3),
            (false, true)
        );
        assert_eq!(report(|r| r.dropped_pattern_objects = 1), (false, true));
        assert_eq!(report(|r| r.overflow_mm = Some((1.0, 0.0))), (true, false));
    }

    #[test]
    fn page_box_is_centred_on_the_page() {
        let (_, map) = luebeck();
        let (b, m) = (map.page_box(), map.mapping());
        let _ = m;
        assert!(
            (((b.x0 + b.x1) / 2.0) - 210.0).abs() < 1e-6
                || (((b.x0 + b.x1) / 2.0) - 148.5).abs() < 1e-6
        );
    }

    #[test]
    fn the_map_fits_the_page() {
        let (_, map) = luebeck();
        assert!(!map.is_clipped());
    }

    #[test]
    fn page_box_holds_the_extent_with_a_margin() {
        let (doc, map) = luebeck();
        let (min, max) = MapIndex::from_document(&doc).extent().unwrap();
        let (a, b) = (map.mapping().to_page(min), map.mapping().to_page(max));
        let r = map.page_box();
        // The box is symmetric around the page centre, so one side has exactly the margin, the other at least.
        assert!(a.0 - r.x0 >= 10.0 - 1e-6 && a.1 - r.y0 >= 10.0 - 1e-6);
        assert!(r.x1 - b.0 >= 10.0 - 1e-6 && r.y1 - b.1 >= 10.0 - 1e-6);
        assert!(r.x1 - r.x0 < 420.0 - 1.0, "the box is not the whole page");
    }

    #[test]
    fn same_view_same_svg() {
        let (_, map) = luebeck();
        let view = PageRect {
            x0: 100.0,
            y0: 160.0,
            x1: 200.0,
            y1: 240.0,
        };
        let a = map.svg_for(view, 4.0).unwrap();
        let (_, other) = luebeck();
        assert_eq!(a, other.svg_for(view, 4.0).unwrap());
        assert!(a.contains("<path"), "no map content");
        assert!(a.len() > 1000);
    }

    #[test]
    fn map_svg_is_cached_per_view() {
        let (_, map) = luebeck();
        let view = rect_of(&map);
        let first = map.svg_for(view, 1.0).unwrap();
        assert_eq!(map.render_count(), 1);
        assert_eq!(map.svg_for(view, 1.0).unwrap(), first);
        assert_eq!(map.render_count(), 1);
        map.svg_for(view, 2.0).unwrap();
        assert_eq!(map.render_count(), 2);
    }

    #[test]
    fn invalid_views_are_refused() {
        let (_, map) = luebeck();
        let bad = PageRect {
            x0: 0.0,
            y0: 0.0,
            x1: f64::NAN,
            y1: 10.0,
        };
        assert!(map.svg_for(bad, 1.0).is_err());
        assert!(map.svg_for(rect_of(&map), 0.0).is_err());
    }

    /// Timing on a big map, for the ADR 0046 addendum (not run by default; use `--release`):
    /// `MEASURE_MAP=/path/to/map.omap cargo test --release -p freemapper-courseplan-cli measure_render -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn measure_render() {
        use std::time::Instant;
        let path = std::env::var("MEASURE_MAP").expect("MEASURE_MAP");
        let t = Instant::now();
        let doc = read_map_file(std::path::Path::new(&path)).unwrap();
        let parse = t.elapsed();
        let t = Instant::now();
        let map = RenderMap::new(&doc, &MapIndex::from_document(&doc)).unwrap();
        let build = t.elapsed();
        let b = map.page_box();
        let t = Instant::now();
        let whole = map.svg_for(b, 2.0);
        let first = t.elapsed();
        let (cx, cy) = ((b.x0 + b.x1) / 2.0, (b.y0 + b.y1) / 2.0);
        let zoomed = PageRect {
            x0: cx - 30.0,
            y0: cy - 30.0,
            x1: cx + 30.0,
            y1: cy + 30.0,
        };
        let t = Instant::now();
        let z = map.svg_for(zoomed, 20.0);
        let zoom_time = t.elapsed();
        eprintln!(
            "MEASURE parse {parse:?} build_page {build:?} whole-page svg {first:?} ({:?} bytes) zoomed 60x60 mm at 20 px/mm {zoom_time:?} ({:?} bytes) clipped={}",
            whole.as_ref().map(String::len),
            z.as_ref().map(String::len),
            map.is_clipped()
        );
    }

    fn router_for(path: &std::path::Path) -> axum::Router {
        let state = EditorState::new(path.into(), Lang::english());
        router("/tok", Arc::new(Mutex::new(state)), Arc::new(Notify::new()))
    }

    async fn get(app: &axum::Router, uri: &str) -> (StatusCode, String) {
        let response = app
            .clone()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }

    #[tokio::test]
    async fn oversized_view_is_413_with_message() {
        // The page caps its own requests at 6 000 px a side, so this answer is for hand-made requests (ADR 0056).
        let dir = tempfile::tempdir().unwrap();
        let path = state::project_on(dir.path(), "luebeck.omap", |_| {});
        let app = router_for(&path);
        let (status, body) = get(&app, "/tok/api/map.svg?x0=0&y0=0&x1=300&y1=300&zoom=900").await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        let want = freemapper_courseplan::text::text(&Lang::english(), "editor-map-too-large", &[]);
        assert_eq!(body, want);
        let de = freemapper_courseplan::text::text(
            &freemapper_courseplan::text::parse_lang_arg("de").unwrap(),
            "editor-map-too-large",
            &[],
        );
        assert_ne!(de, want);
        // The map info still answers: the page keeps the overlay.
        let (status, info) = get(&app, "/tok/api/map/view").await;
        assert_eq!(status, StatusCode::OK, "{info}");
        assert!(info.contains("\"placeholder\":false"), "{info}");
        assert!(info.contains("\"clipped\":false"), "{info}");
        assert!(info.contains("\"incomplete\":false"), "{info}");
    }

    #[tokio::test]
    async fn map_info_says_when_the_map_is_clipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = state::project_on(dir.path(), "luebeck.omap", |_| {});
        std::fs::write(dir.path().join("map.omap"), scaled_luebeck_text(6)).unwrap();
        let app = router_for(&path);
        let (status, info) = get(&app, "/tok/api/map/view").await;
        assert_eq!(status, StatusCode::OK, "{info}");
        assert!(info.contains("\"clipped\":true"), "{info}");
        let en = freemapper_courseplan::text::text(&Lang::english(), "editor-map-clipped", &[]);
        let de = freemapper_courseplan::text::text(
            &freemapper_courseplan::text::parse_lang_arg("de").unwrap(),
            "editor-map-clipped",
            &[],
        );
        assert!(en.contains("A3") && de.contains("A3") && en != de);
        for key in ["editor-map-clipped", "editor-map-incomplete"] {
            let t = freemapper_courseplan::text::text(&Lang::english(), key, &[]);
            assert!(!t.is_empty() && !t.contains(key), "{key}: {t}");
        }
    }

    #[test]
    fn file_stamp_sees_a_same_length_rewrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("m.omap");
        let mut a = vec![b'a'; 20_000];
        std::fs::write(&path, &a).unwrap();
        let before = FileStamp::of(&path).unwrap();
        a[10] = b'b';
        a[19_990] = b'b';
        std::fs::write(&path, &a).unwrap();
        let mut after = FileStamp::of(&path).unwrap();
        // Same length and, as on a coarse clock, the same mtime.
        assert_eq!(before.len, after.len);
        after.modified = before.modified;
        assert_ne!(before, after);
        assert_ne!(before.token(), after.token());
    }

    #[tokio::test]
    async fn a_view_inside_the_budget_is_an_svg() {
        let dir = tempfile::tempdir().unwrap();
        let path = state::project_on(dir.path(), "luebeck.omap", |_| {});
        let app = router_for(&path);
        let (status, body) = get(&app, "/tok/api/map.svg?x0=50&y0=50&x1=150&y1=150&zoom=8").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.starts_with("<svg"));
        let _ = FileStamp::of(&path);
    }
}
