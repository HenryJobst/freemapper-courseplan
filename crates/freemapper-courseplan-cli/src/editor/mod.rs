//! The `edit` command: a local web server for the course editor (ADR 0042, ADR 0005).
//!
//! The shell around the routes (127.0.0.1 binding, token, Host and Origin checks, security headers) is
//! `freemapper-web`; this module brings the editor's own routes and the embedded page.

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::rejection::BytesRejection;
use axum::extract::{DefaultBodyLimit, Path, RawQuery, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use freemapper_courseplan::ppen_import::MAX_COURSE_FILE_BYTES;
use freemapper_courseplan::text::{FluentValue, Lang, Localize, TEXTS, text};
use freemapper_courseplan_model::ProjectFile;
use freemapper_web::{ServeOptions, WebError, embedded, open_browser, serve};
use rust_embed::RustEmbed;
use tokio::sync::{Mutex, Notify};

use crate::mcp::CoursePlanServer;
use api::Loaded;
use map_view::{FileStamp, MapCache, MapResult, MapViewError, PageRect, load_map};

mod api;
mod exchange;
mod history;
mod map_view;
mod overlay;
mod render_map;
mod symbols;
mod sync_panel;
#[cfg(test)]
mod tests;

/// The page and its scripts and styles, compiled into the binary (`debug-embed`, so tests see the same set).
#[derive(RustEmbed)]
#[folder = "web/"]
pub(crate) struct Assets;

/// Scripts, styles, images and connections only from the page's own origin; inline styles stay allowed
/// because the renderer's SVG carries `style` attributes (ADR 0042).
const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
                   img-src 'self' data: blob:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; \
                   form-action 'none'";

/// What the routes share. One project per process. The lock guards only these fields and is never held
/// across an await or a blocking call: reading, hashing and parsing files runs on the blocking pool.
pub(crate) struct EditorState {
    /// The project file, as a canonical path.
    pub project: PathBuf,
    /// The project file as last read; `api/state` and `api/revision` refresh it, the map routes only use it.
    pub loaded: Arc<Loaded>,
    /// Length and modification time of the project file when `loaded` was read: an unchanged stamp means
    /// the file is not read or hashed again.
    pub stamp: Option<FileStamp>,
    /// The language of the page's texts: the process language.
    pub lang: Lang,
    /// The map built from the project's map file.
    pub map: MapCache,
    /// The steps undo and redo can take back (ADR 0043).
    pub history: history::UndoHistory,
    /// The MCP tools, which are the editor's operations.
    pub server: CoursePlanServer,
    /// The last check result, keyed by the project revision and the map stamp it was made from.
    pub checked: Option<(String, Arc<serde_json::Value>)>,
}

impl EditorState {
    pub(crate) fn new(project: PathBuf, lang: Lang) -> Self {
        let stamp = FileStamp::of(&project).ok();
        let loaded = Arc::new(Loaded::read(&project, &lang));
        Self {
            project,
            loaded,
            stamp,
            map: MapCache::new(),
            history: history::UndoHistory::default(),
            server: CoursePlanServer::new(lang.clone()),
            checked: None,
            lang,
        }
    }
}

pub(crate) type Shared = Arc<Mutex<EditorState>>;

/// The editor's routes below `prefix` (`/{token}`). `stop` ends the server (`POST api/quit`); it is not part
/// of the locked state, so quitting never waits for other work.
pub(crate) fn router(prefix: &str, state: Shared, stop: Arc<Notify>) -> Router {
    Router::new()
        .route(&format!("{prefix}/"), get(index))
        .route(&format!("{prefix}/assets/{{*path}}"), get(asset))
        .route(&format!("{prefix}/api/texts"), get(texts))
        .route(&format!("{prefix}/api/rules"), get(api_rules))
        .route(&format!("{prefix}/api/state"), get(api_state))
        .route(&format!("{prefix}/api/revision"), get(api_revision))
        .route(&format!("{prefix}/api/overlay"), get(api_overlay))
        .route(&format!("{prefix}/api/check"), get(api_check))
        .route(&format!("{prefix}/api/number-at"), get(api_number_at))
        .route(&format!("{prefix}/api/map/view"), get(map_info))
        .route(&format!("{prefix}/api/map.svg"), get(map_svg))
        .route(&format!("{prefix}/api/map/near"), get(map_near))
        .route(&format!("{prefix}/api/symbols"), get(api_symbols))
        .route(&format!("{prefix}/api/op/{{tool}}"), post(api_op))
        .route(&format!("{prefix}/api/export/iof-xml"), get(api_export_xml))
        .route(&format!("{prefix}/api/export/ppen"), post(api_export_ppen))
        .route(
            &format!("{prefix}/api/import/ppen"),
            // The shell limits bodies to 1 MiB; this inner layer wins for the upload (ADR 0053).
            post(api_import_ppen).layer(DefaultBodyLimit::max(MAX_COURSE_FILE_BYTES)),
        )
        .route(&format!("{prefix}/api/sync"), post(api_sync))
        .route(&format!("{prefix}/api/undo"), post(api_undo))
        .route(&format!("{prefix}/api/redo"), post(api_redo))
        .route(&format!("{prefix}/api/quit"), post(quit).with_state(stop))
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

/// The rule profiles, overlays and rule categories the program knows, for the page's pickers. Read-only; it
/// names ids, never files.
async fn api_rules() -> Response {
    use freemapper_courseplan::rules::{ProfileKind, available, categories, profile};
    let mut profiles = Vec::new();
    let mut overlays = Vec::new();
    let mut by_profile = serde_json::Map::new();
    for id in available() {
        let Ok(found) = profile(id) else { continue };
        by_profile.insert(id.to_owned(), serde_json::json!(categories(&found)));
        match found.kind {
            ProfileKind::Base => profiles.push(serde_json::json!({ "id": id })),
            ProfileKind::Overlay => {
                overlays.push(serde_json::json!({ "id": id, "base": found.base }));
            }
        }
    }
    axum::Json(serde_json::json!({
        "profiles": profiles,
        "overlays": overlays,
        "categories": by_profile,
    }))
    .into_response()
}

fn plain(status: StatusCode, message: String) -> Response {
    (status, message).into_response()
}

/// Runs blocking work (files, parsing, rendering) on the blocking pool.
async fn blocking<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    tokio::task::spawn_blocking(work).await.ok()
}

async fn blocking_response(work: impl FnOnce() -> Response + Send + 'static) -> Response {
    blocking(work).await.unwrap_or_else(|| {
        plain(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_owned(),
        )
    })
}

/// The project as last read; the file is not touched.
async fn current(state: &Shared) -> Arc<Loaded> {
    state.lock().await.loaded.clone()
}

/// The project, read again first when the file's stamp changed and its bytes hash differently.
async fn refreshed(state: &Shared) -> Arc<Loaded> {
    let (project, lang, prev, prev_stamp) = {
        let guard = state.lock().await;
        (
            guard.project.clone(),
            guard.lang.clone(),
            guard.loaded.clone(),
            guard.stamp,
        )
    };
    let before = prev.clone();
    let result = blocking(move || {
        let stamp = FileStamp::of(&project).ok();
        if stamp == prev_stamp {
            return (stamp, None);
        }
        (stamp, Loaded::changed_since(&prev, &project, &lang))
    })
    .await;
    let Some((stamp, changed)) = result else {
        return before;
    };
    let mut guard = state.lock().await;
    guard.stamp = stamp;
    if let Some(loaded) = changed {
        guard.loaded = Arc::new(loaded);
    }
    guard.loaded.clone()
}

/// The map of a readable project through the cache, or the project's error.
async fn map_of(state: &Shared, loaded: &Loaded) -> MapResult {
    match &loaded.file {
        Ok(file) => load_map(state, file.map_path()).await,
        Err(e) => Err(e.clone()),
    }
}

async fn api_state(State(state): State<Shared>) -> Response {
    let loaded = refreshed(&state).await;
    let map = map_of(&state, &loaded).await;
    let undo = {
        let guard = state.lock().await;
        let (undo, redo) = guard.history.labels();
        (undo.map(str::to_owned), redo.map(str::to_owned))
    };
    blocking_response(move || {
        let map = map.as_ref().map(|m| &**m).map_err(String::as_str);
        let undo = (undo.0.as_deref(), undo.1.as_deref());
        axum::Json(api::state_json(&loaded, map, undo)).into_response()
    })
    .await
}

/// Runs `work` on the blocking pool with the state locked for the whole operation: operations run one at a
/// time, and the file is read, changed and read again without anyone else's step in between.
async fn serialized(
    state: Shared,
    work: impl FnOnce(&mut EditorState) -> api::Reply + Send + 'static,
) -> Response {
    let mut guard = state.lock_owned().await;
    blocking_response(move || {
        let (status, body) = work(&mut guard);
        (status, axum::Json(body)).into_response()
    })
    .await
}

/// `POST api/op/{tool}`: one of the editor's operations. The body holds the tool's arguments without
/// `project`; the header `X-Courseplan-Revision` names the revision the page saw.
async fn api_op(
    State(state): State<Shared>,
    Path(tool): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let revision = page_revision(&headers);
    let args = if body.is_empty() {
        Some(serde_json::json!({}))
    } else {
        serde_json::from_slice(&body).ok()
    };
    serialized(state, move |st| {
        api::run_op(st, &tool, revision.as_deref(), args)
    })
    .await
}

/// `GET api/export/iof-xml`: the IOF XML of the project as a download. The counts of courses and of those
/// left out come in headers; the page words them.
async fn api_export_xml(State(state): State<Shared>) -> Response {
    let (project, lang) = {
        let guard = state.lock().await;
        (guard.project.clone(), guard.lang.clone())
    };
    blocking_response(move || match exchange::iof_xml(&project, &lang) {
        Ok(download) => {
            let disposition = format!("attachment; filename=\"{}\"", download.file_name);
            let count = |n: usize| n.to_string();
            (
                [
                    (header::CONTENT_TYPE, "application/xml".to_owned()),
                    (header::CONTENT_DISPOSITION, disposition),
                    (
                        header::HeaderName::from_static("x-courseplan-courses"),
                        count(download.report.courses),
                    ),
                    (
                        header::HeaderName::from_static("x-courseplan-skipped"),
                        count(download.report.skipped.len()),
                    ),
                    (
                        header::HeaderName::from_static("x-courseplan-geo"),
                        count(usize::from(download.report.geo_positions)),
                    ),
                ],
                download.xml,
            )
                .into_response()
        }
        Err((status, body)) => (status, axum::Json(body)).into_response(),
    })
    .await
}

/// `POST api/export/ppen {"to": "linked" | "download"}`.
async fn api_export_ppen(State(state): State<Shared>, headers: HeaderMap, body: Bytes) -> Response {
    let revision = page_revision(&headers);
    let to = serde_json::from_slice::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v.get("to").and_then(|t| t.as_str()).map(str::to_owned))
        .unwrap_or_default();
    serialized(state, move |st| {
        exchange::export_ppen(st, &to, revision.as_deref())
    })
    .await
}

/// `POST api/import/ppen?replace=true|false`: the body is the Purple Pen file.
async fn api_import_ppen(
    State(state): State<Shared>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    let lang = state.lock().await.lang.clone();
    let refuse = |status: StatusCode, id: &str| {
        (
            status,
            axum::Json(serde_json::json!({ "error": text(&lang, id, &[]) })),
        )
            .into_response()
    };
    let Some(replace) = exchange::import_query(query.as_deref()) else {
        return refuse(StatusCode::BAD_REQUEST, "editor-import-bad-request");
    };
    let body = match body {
        Ok(body) => body,
        Err(rejection) if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            return refuse(StatusCode::PAYLOAD_TOO_LARGE, "editor-import-too-large");
        }
        Err(_) => return refuse(StatusCode::BAD_REQUEST, "editor-import-bad-request"),
    };
    let revision = page_revision(&headers);
    serialized(state, move |st| {
        exchange::import_ppen(st, revision.as_deref(), replace, &body)
    })
    .await
}

/// `POST api/sync {dry_run, take}`: syncs the project with its linked Purple Pen file (ADR 0054).
async fn api_sync(State(state): State<Shared>, headers: HeaderMap, body: Bytes) -> Response {
    let revision = page_revision(&headers);
    serialized(state, move |st| {
        sync_panel::sync(st, revision.as_deref(), &body)
    })
    .await
}

/// The revision the page names in `X-Courseplan-Revision`.
fn page_revision(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-courseplan-revision")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

async fn api_undo(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let revision = page_revision(&headers);
    serialized(state, move |st| {
        api::run_history(st, false, revision.as_deref())
    })
    .await
}

async fn api_redo(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let revision = page_revision(&headers);
    serialized(state, move |st| {
        api::run_history(st, true, revision.as_deref())
    })
    .await
}

async fn api_revision(State(state): State<Shared>) -> Response {
    let loaded = refreshed(&state).await;
    blocking_response(move || axum::Json(api::revision_json(&loaded)).into_response()).await
}

/// `GET api/check`: the result of the MCP tool `check` plus `revision`. The result is cached on the project
/// revision and the map stamp, so repeated polls cost nothing; a changed map file is read again (ADR 0052).
async fn api_check(State(state): State<Shared>) -> Response {
    let loaded = refreshed(&state).await;
    let entry = match map_of(&state, &loaded).await {
        Ok(entry) => entry,
        Err(e) => return plain(StatusCode::INTERNAL_SERVER_ERROR, e),
    };
    let key = format!("{}/{}", loaded.revision.0, entry.stamp.token());
    let lang = {
        let guard = state.lock().await;
        if let Some((cached, value)) = &guard.checked
            && *cached == key
        {
            return axum::Json(value.as_ref()).into_response();
        }
        guard.lang.clone()
    };
    let work = {
        let loaded = loaded.clone();
        move || -> Result<serde_json::Value, String> {
            let file = loaded.file.as_ref().map_err(Clone::clone)?;
            let event = &file.event;
            let rules = freemapper_courseplan::rules::load_rule_set(
                &event.event.rule_profile,
                &event.event.rule_overlays,
            )
            .map_err(|e| api::tidy(&e.localize(&lang), &[]))?;
            let kind = freemapper_courseplan::map::map_kind_for_file(&entry.doc, &file.map_path());
            let report =
                freemapper_courseplan::check::check_event_as(event, &entry.doc, kind, &rules);
            let mut value = crate::mcp::report_json(&rules, event, &report, &lang);
            value["revision"] = serde_json::json!(loaded.revision.0);
            Ok(value)
        }
    };
    match blocking(work).await {
        Some(Ok(value)) => {
            let value = Arc::new(value);
            state.lock().await.checked = Some((key, value.clone()));
            axum::Json(value.as_ref()).into_response()
        }
        Some(Err(e)) => plain(StatusCode::INTERNAL_SERVER_ERROR, e),
        None => plain(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_owned(),
        ),
    }
}

/// `GET api/number-at?course&index&x&y`: direction and gap of a number whose box is centred on a point.
async fn api_number_at(State(state): State<Shared>, RawQuery(query): RawQuery) -> Response {
    let lang = state.lock().await.lang.clone();
    let Some((course, index, x, y)) = api::number_at_query(query.as_deref()) else {
        return plain(
            StatusCode::BAD_REQUEST,
            text(&lang, "editor-number-bad-request", &[]),
        );
    };
    let loaded = current(&state).await;
    let entry = match map_of(&state, &loaded).await {
        Ok(entry) => entry,
        Err(e) => return plain(StatusCode::INTERNAL_SERVER_ERROR, e),
    };
    blocking_response(move || {
        let Ok(file) = &loaded.file else {
            return plain(StatusCode::INTERNAL_SERVER_ERROR, String::new());
        };
        let kind = freemapper_courseplan::map::map_kind_for_file(&entry.doc, &file.map_path());
        let drop = freemapper_courseplan_model::MapPoint { x, y };
        let scale = entry.doc.georef.scale_denominator;
        match api::number_at(&file.event, &course, index, &drop, kind, scale) {
            Some(found) => axum::Json(found).into_response(),
            None => plain(
                StatusCode::NOT_FOUND,
                text(&lang, "editor-number-not-found", &[]),
            ),
        }
    })
    .await
}

/// The course symbols in map mm; `?course=NAME` for one course, no parameter for all controls.
async fn api_overlay(State(state): State<Shared>, RawQuery(query): RawQuery) -> Response {
    let lang = state.lock().await.lang.clone();
    let Ok(course) = api::overlay_query(query.as_deref()) else {
        return plain(
            StatusCode::BAD_REQUEST,
            text(&lang, "editor-map-bad-request", &[]),
        );
    };
    let loaded = current(&state).await;
    let entry = match map_of(&state, &loaded).await {
        Ok(entry) => entry,
        Err(e) => return plain(StatusCode::INTERNAL_SERVER_ERROR, e),
    };
    blocking_response(move || overlay_response(&loaded, &entry, course.as_deref(), &lang)).await
}

fn overlay_response(
    loaded: &Loaded,
    entry: &map_view::MapEntry,
    course: Option<&str>,
    lang: &Lang,
) -> Response {
    let Ok(file) = &loaded.file else {
        return plain(StatusCode::INTERNAL_SERVER_ERROR, String::new());
    };
    let scale = entry.doc.georef.scale_denominator;
    let kind = freemapper_courseplan::map::map_kind_for_file(&entry.doc, &file.map_path());
    let event = &file.event;
    let gaps = match course.and_then(|name| event.courses.iter().find(|c| c.name == name)) {
        Some(found) => match freemapper_courseplan::rules::load_rule_set(
            &event.event.rule_profile,
            &event.event.rule_overlays,
        ) {
            Ok(rules) => freemapper_courseplan::numbers::gap_range(
                &rules,
                kind,
                event.event.discipline,
                freemapper_courseplan::rules::print_scale_of(found, scale),
            ),
            Err(e) => {
                return plain(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    api::tidy(&e.localize(lang), &[]),
                );
            }
        },
        None => freemapper_courseplan::numbers::GapRange::DEFAULT,
    };
    match overlay::overlay(event, course, &entry.index, scale, kind, gaps) {
        Ok(overlay) => axum::Json(overlay).into_response(),
        // A score, team or relay course (or one naming something the event lacks) is not a missing
        // resource: the page says why nothing is drawn.
        Err(
            e @ (freemapper_courseplan::course::CourseError::NotSupported { .. }
            | freemapper_courseplan::course::CourseError::UnknownReference { .. }),
        ) => plain(
            StatusCode::UNPROCESSABLE_ENTITY,
            api::tidy(&e.localize(lang), &[]),
        ),
        Err(e) => plain(StatusCode::NOT_FOUND, api::tidy(&e.localize(lang), &[])),
    }
}

async fn map_info(State(state): State<Shared>) -> Response {
    let loaded = current(&state).await;
    let entry = match map_of(&state, &loaded).await {
        Ok(entry) => entry,
        Err(e) => return plain(StatusCode::INTERNAL_SERVER_ERROR, e),
    };
    let (b, m) = (entry.view.page_box(), entry.view.mapping());
    axum::Json(serde_json::json!({
        "page_box": [b.x0, b.y0, b.x1, b.y1],
        "mapping": { "k": m.k, "dx": m.dx, "dy": m.dy },
        "placeholder": entry.view.is_placeholder(),
        "clipped": entry.view.is_clipped(),
        "incomplete": entry.view.is_incomplete(),
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
    let lang = state.lock().await.lang.clone();
    let bad = |lang: &Lang| {
        plain(
            StatusCode::BAD_REQUEST,
            text(lang, "editor-map-bad-request", &[]),
        )
    };
    let Some((rect, zoom)) = query
        .and_then(|raw| SvgQuery::parse(&raw))
        .and_then(|q| parse_rect(&q))
    else {
        return bad(&lang);
    };
    let loaded = current(&state).await;
    let entry = match map_of(&state, &loaded).await {
        Ok(entry) => entry,
        Err(e) => return plain(StatusCode::INTERNAL_SERVER_ERROR, e),
    };
    blocking_response(move || match entry.view.svg_for(rect, zoom) {
        Ok(svg) => ([(header::CONTENT_TYPE, "image/svg+xml")], svg).into_response(),
        Err(MapViewError::TooLarge) => plain(
            StatusCode::PAYLOAD_TOO_LARGE,
            text(&lang, "editor-map-too-large", &[]),
        ),
        Err(MapViewError::Render(e)) => plain(StatusCode::INTERNAL_SERVER_ERROR, e),
    })
    .await
}

/// Largest radius `api/map/near` takes, in map millimetres: the page asks for a few.
const MAX_NEAR_RADIUS: f64 = 50.0;

/// The query of `api/map/near`: `x`, `y` and `r` (map mm), each exactly once, finite, in range.
fn parse_near(raw: &str) -> Option<(f64, f64, f64)> {
    let (mut x, mut y, mut r) = (None, None, None);
    for pair in raw.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=')?;
        let slot = match key {
            "x" => &mut x,
            "y" => &mut y,
            "r" => &mut r,
            _ => return None,
        };
        let value: f64 = value.parse().ok()?;
        if !value.is_finite() || slot.replace(value).is_some() {
            return None;
        }
    }
    let (x, y, r) = (x?, y?, r?);
    (x.abs() <= MAX_COORDINATE && y.abs() <= MAX_COORDINATE && r > 0.0 && r <= MAX_NEAR_RADIUS)
        .then_some((x, y, r))
}

/// `GET api/map/near?x&y&r`: the map objects within `r` mm of a point, from the cached map index.
async fn map_near(State(state): State<Shared>, RawQuery(query): RawQuery) -> Response {
    let lang = state.lock().await.lang.clone();
    let Some((x, y, r)) = query.as_deref().and_then(parse_near) else {
        return plain(
            StatusCode::BAD_REQUEST,
            text(&lang, "editor-map-bad-request", &[]),
        );
    };
    let loaded = current(&state).await;
    let entry = match map_of(&state, &loaded).await {
        Ok(entry) => entry,
        Err(e) => return plain(StatusCode::INTERNAL_SERVER_ERROR, e),
    };
    blocking_response(move || {
        let at = freemapper_courseplan_model::MapPoint { x, y };
        axum::Json(crate::mcp::features_near_json(&entry.index, at, r, None)).into_response()
    })
    .await
}

/// `GET api/symbols?column=C..H`: the description symbols of one column (ADR 0051). Only the column letter
/// is read from the query.
async fn api_symbols(State(state): State<Shared>, RawQuery(query): RawQuery) -> Response {
    let lang = state.lock().await.lang.clone();
    let index = query
        .as_deref()
        .and_then(|q| q.strip_prefix("column="))
        .and_then(symbols::column_index);
    match index {
        Some(index) => axum::Json(symbols::symbols_json(index, &lang)).into_response(),
        None => plain(
            StatusCode::BAD_REQUEST,
            text(&lang, "editor-symbols-bad-request", &[]),
        ),
    }
}

async fn quit(State(stop): State<Arc<Notify>>) -> Response {
    stop.notify_one();
    axum::Json(serde_json::json!({ "ok": true })).into_response()
}

/// After the first stop signal, open connections get this long to finish; then the process exits. Every
/// write is atomic, so nothing is half-written.
const GRACE: Duration = Duration::from_secs(2);

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
    let outcome = runtime.block_on(async {
        let options = ServeOptions {
            port,
            strict_post: true,
            ..ServeOptions::new(CSP)
        };
        let server_lang = lang.clone();
        let stop = Arc::new(Notify::new());
        let router_stop = stop.clone();
        let server = serve(
            move |prefix, _shutdown| {
                let state = EditorState::new(project, server_lang);
                router(prefix, Arc::new(Mutex::new(state)), router_stop)
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
        // A closed stdout must not stop the editor.
        let _ = writeln!(std::io::stdout(), "{}", server.url);
        if open && let Err(e) = open_browser(&server.url) {
            eprintln!("{}", failed("cli-edit-browser-failed", e.to_string()));
        }
        eprintln!("{}", text(&lang, "cli-edit-stop-hint", &[]));
        let shutdown = server.shutdown();
        let stopping = Arc::new(Notify::new());
        let signalled = stopping.clone();
        tokio::spawn(async move {
            tokio::select! {
                () = async {
                    if tokio::signal::ctrl_c().await.is_err() {
                        std::future::pending::<()>().await;
                    }
                } => {}
                () = stop.notified() => {}
            }
            shutdown.notify_one();
            signalled.notify_one();
        });
        let grace = async {
            stopping.notified().await;
            tokio::time::sleep(GRACE).await;
        };
        tokio::select! {
            result = server.finished() => {
                result.map_err(|e| failed("cli-edit-server-failed", e.to_string()))
            }
            () = grace => Ok(()),
        }
    });
    // A map still being parsed on the blocking pool must not hold the process back either.
    runtime.shutdown_timeout(Duration::from_millis(200));
    outcome
}
