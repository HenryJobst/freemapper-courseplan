//! The `edit` command: a local web server for the course editor (ADR 0042, ADR 0005).
//!
//! The shell around the routes (127.0.0.1 binding, token, Host and Origin checks, security headers) is
//! `freemapper-web`; this module brings the editor's own routes and the embedded page.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use freemapper_courseplan::text::{FluentValue, Lang, Localize, TEXTS, text};
use freemapper_courseplan_model::ProjectFile;
use freemapper_web::{ServeOptions, WebError, embedded, open_browser, serve};
use rust_embed::RustEmbed;
use tokio::sync::{Mutex, Notify};

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
}

impl EditorState {
    pub(crate) fn new(project: PathBuf, lang: Lang, shutdown: Arc<Notify>) -> Self {
        Self {
            project,
            lang,
            shutdown,
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
