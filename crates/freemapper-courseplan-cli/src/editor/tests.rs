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
