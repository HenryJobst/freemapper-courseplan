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
