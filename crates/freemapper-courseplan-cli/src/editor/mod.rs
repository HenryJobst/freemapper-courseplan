//! The `edit` command: a local web server for the course editor (ADR 0042, ADR 0005).
//!
//! The shell around the routes (127.0.0.1 binding, token, Host and Origin checks, security headers) is
//! `freemapper-web`; this module brings the editor's own routes and the embedded page.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, RawQuery, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use freemapper_courseplan::text::{FluentValue, Lang, Localize, TEXTS, text};
use freemapper_courseplan_model::ProjectFile;
use freemapper_web::{ServeOptions, WebError, embedded, open_browser, serve};
use rust_embed::RustEmbed;
use tokio::sync::{Mutex, Notify};

use map_view::{MapCache, MapViewError, PageRect};

mod map_view;
#[cfg(test)]
mod tests;

/// The page and its scripts and styles, compiled into the binary (`debug-embed`, so tests see the same set).
#[derive(RustEmbed)]
#[folder = "web/"]
pub(crate) struct Assets;

/// Scripts, styles, images and connections only from the page's own origin; inline styles stay allowed
/// because the renderer's SVG carries `style` attributes (ADR 0042).
const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
                   img-src 'self' data: blob:; connect-src 'self'; frame-ancestors 'none'";

/// What the routes share. One project per process; operations take the lock one after another.
pub(crate) struct EditorState {
    /// The project file, as a canonical path. The operations of the next tasks read it.
    #[allow(dead_code)]
    pub project: PathBuf,
    /// The language of the page's texts: the process language.
    pub lang: Lang,
    /// Notified to stop the server (`POST api/quit`).
    pub shutdown: Arc<Notify>,
    /// The map built from the project's map file.
    pub map: MapCache,
}

impl EditorState {
    pub(crate) fn new(project: PathBuf, lang: Lang, shutdown: Arc<Notify>) -> Self {
        let note = text(&lang, "editor-map-placeholder", &[]);
        Self {
            project,
            lang,
            shutdown,
            map: MapCache::new(note),
        }
    }
}

type Shared = Arc<Mutex<EditorState>>;

/// The editor's routes below `prefix` (`/{token}`).
pub(crate) fn router(prefix: &str, state: Shared) -> Router {
    Router::new()
        .route(&format!("{prefix}/"), get(index))
        .route(&format!("{prefix}/assets/{{*path}}"), get(asset))
        .route(&format!("{prefix}/api/texts"), get(texts))
        .route(&format!("{prefix}/api/map/view"), get(map_info))
        .route(&format!("{prefix}/api/map.svg"), get(map_svg))
        .route(&format!("{prefix}/api/quit"), post(quit))
        .with_state(state)
}

async fn index() -> Response {
    embedded::<Assets>("index.html")
}

async fn asset(Path(path): Path<String>) -> Response {
    embedded::<Assets>(&path)
}

/// Every `editor-*` message in the process language, for the page to put into its elements.
async fn texts(State(state): State<Shared>) -> Response {
    let lang = state.lock().await.lang.clone();
    let texts: BTreeMap<String, String> = TEXTS
        .message_ids("en")
        .into_iter()
        .filter(|id| id.starts_with("editor-"))
        .map(|id| {
            let message = text(&lang, &id, &[]);
            (id, message)
        })
        .collect();
    axum::Json(serde_json::json!({ "lang": lang.code(), "texts": texts })).into_response()
}

fn plain(status: StatusCode, message: String) -> Response {
    (status, message).into_response()
}

/// Loads the project's map through the cache.
fn map_entry(state: &mut EditorState) -> Result<&map_view::MapEntry, String> {
    let map_path = ProjectFile::load(&state.project)
        .map_err(|e| e.to_string())?
        .map_path();
    state.map.get(&map_path)
}

async fn map_info(State(state): State<Shared>) -> Response {
    let mut state = state.lock().await;
    let entry = match map_entry(&mut state) {
        Ok(entry) => entry,
        Err(e) => return plain(StatusCode::INTERNAL_SERVER_ERROR, e),
    };
    let (b, m) = (entry.view.page_box(), entry.view.mapping());
    axum::Json(serde_json::json!({
        "page_box": [b.x0, b.y0, b.x1, b.y1],
        "mapping": { "k": m.k, "dx": m.dx, "dy": m.dy },
        "placeholder": entry.view.is_placeholder(),
    }))
    .into_response()
}

/// The query of `api/map.svg`. A repeated or unknown parameter, or a value that is not a number, is refused.
#[derive(Default)]
struct SvgQuery {
    x0: Option<f64>,
    y0: Option<f64>,
    x1: Option<f64>,
    y1: Option<f64>,
    zoom: Option<f64>,
}

impl SvgQuery {
    fn parse(raw: &str) -> Option<Self> {
        let mut q = SvgQuery::default();
        for pair in raw.split('&').filter(|p| !p.is_empty()) {
            let (key, value) = pair.split_once('=')?;
            let slot = match key {
                "x0" => &mut q.x0,
                "y0" => &mut q.y0,
                "x1" => &mut q.x1,
                "y1" => &mut q.y1,
                "zoom" => &mut q.zoom,
                _ => return None,
            };
            if slot.replace(value.parse().ok()?).is_some() {
                return None;
            }
        }
        Some(q)
    }
}

/// Coordinates beyond this are not a page.
const MAX_COORDINATE: f64 = 1.0e7;
/// Largest accepted zoom (pixels per mm).
const MAX_ZOOM: f64 = 1000.0;

fn parse_rect(q: &SvgQuery) -> Option<(PageRect, f64)> {
    let rect = PageRect {
        x0: q.x0?,
        y0: q.y0?,
        x1: q.x1?,
        y1: q.y1?,
    };
    let zoom = q.zoom.unwrap_or(1.0);
    let coordinates = [rect.x0, rect.y0, rect.x1, rect.y1];
    let ok = coordinates
        .iter()
        .all(|v| v.is_finite() && v.abs() <= MAX_COORDINATE)
        && rect.x1 > rect.x0
        && rect.y1 > rect.y0
        && zoom.is_finite()
        && zoom > 0.0
        && zoom <= MAX_ZOOM;
    ok.then_some((rect, zoom))
}

async fn map_svg(State(state): State<Shared>, RawQuery(query): RawQuery) -> Response {
    let mut state = state.lock().await;
    let lang = state.lang.clone();
    let bad = || {
        plain(
            StatusCode::BAD_REQUEST,
            text(&lang, "editor-map-bad-request", &[]),
        )
    };
    let Some((rect, zoom)) = query
        .and_then(|raw| SvgQuery::parse(&raw))
        .and_then(|q| parse_rect(&q))
    else {
        return bad();
    };
    let entry = match map_entry(&mut state) {
        Ok(entry) => entry,
        Err(e) => return plain(StatusCode::INTERNAL_SERVER_ERROR, e),
    };
    match entry.view.svg_for(rect, zoom) {
        Ok(svg) => ([(header::CONTENT_TYPE, "image/svg+xml")], svg).into_response(),
        Err(MapViewError::TooLarge) => bad(),
        Err(MapViewError::Render(e)) => plain(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

async fn quit(State(state): State<Shared>) -> Response {
    state.lock().await.shutdown.notify_one();
    axum::Json(serde_json::json!({ "ok": true })).into_response()
}

/// Loads the project, serves the editor until Ctrl+C or `POST api/quit`, and prints the address as the only
/// line on stdout.
pub fn run(lang: Lang, project: PathBuf, port: u16, open: bool) -> Result<(), String> {
    ProjectFile::load(&project).map_err(|e| e.localize(&lang))?;
    let project = std::fs::canonicalize(&project).map_err(|e| e.to_string())?;
    let failed =
        |id: &str, reason: String| text(&lang, id, &[("reason", FluentValue::from(reason))]);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|e| failed("cli-edit-runtime-failed", e.to_string()))?;
    runtime.block_on(async {
        let options = ServeOptions {
            port,
            strict_post: true,
            ..ServeOptions::new(CSP)
        };
        let server_lang = lang.clone();
        let server = serve(
            move |prefix, shutdown| {
                let state = EditorState::new(project, server_lang, shutdown);
                router(prefix, Arc::new(Mutex::new(state)))
            },
            options,
        )
        .await
        .map_err(|e| match e {
            WebError::Bind { port, source } => text(
                &lang,
                "cli-edit-bind-failed",
                &[
                    ("port", FluentValue::from(port)),
                    ("reason", FluentValue::from(source.to_string())),
                ],
            ),
            other => failed("cli-edit-runtime-failed", other.to_string()),
        })?;
        println!("{}", server.url);
        if open && let Err(e) = open_browser(&server.url) {
            eprintln!("{}", failed("cli-edit-browser-failed", e.to_string()));
        }
        eprintln!("{}", text(&lang, "cli-edit-stop-hint", &[]));
        let shutdown = server.shutdown();
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                shutdown.notify_one();
            }
        });
        server
            .finished()
            .await
            .map_err(|e| failed("cli-edit-server-failed", e.to_string()))
    })
}
