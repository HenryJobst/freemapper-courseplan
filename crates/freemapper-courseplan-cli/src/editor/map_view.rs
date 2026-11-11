//! The map as the editor page sees it: a coordinate mapping, a page box, an SVG for a rectangle and hit
//! testing (ADR 0046, D13). `PlaceholderMap` stands in until the shared renderer is wired in; it draws the
//! extent frame, a 10 mm grid and a note, never map content.
// The hit test, the index and the document are used by the next tasks.
#![allow(dead_code)]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use freemapper_courseplan::map::MapIndex;
use freemapper_courseplan::omap::MapDocument;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::MapPoint;
use tokio::sync::OnceCell;

use super::Shared;
use super::api::{file_name, tidy};

/// Margin around the extent in the placeholder's page box, in mm.
const MARGIN_MM: f64 = 10.0;
/// Grid spacing of the placeholder, in mm.
const GRID_MM: f64 = 10.0;
/// No more grid lines than this per axis; the spacing grows by factors of ten to stay below.
const MAX_GRID_LINES: f64 = 200.0;
/// Largest SVG side in pixels; bigger requests are refused.
const MAX_PIXELS: f64 = 100_000.0;

/// `page = k * map + d`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Affine {
    pub k: f64,
    pub dx: f64,
    pub dy: f64,
}

impl Affine {
    pub(crate) fn to_page(self, p: MapPoint) -> (f64, f64) {
        (self.k * p.x + self.dx, self.k * p.y + self.dy)
    }

    pub(crate) fn to_map(self, x: f64, y: f64) -> MapPoint {
        MapPoint {
            x: (x - self.dx) / self.k,
            y: (y - self.dy) / self.k,
        }
    }
}

/// A rectangle in page millimetres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PageRect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

/// A map object under a point.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MapHit {
    pub symbol_code: String,
    pub object: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MapViewError {
    TooLarge,
    Render(String),
}

pub(crate) trait MapView: Send + Sync {
    fn mapping(&self) -> Affine;
    fn page_box(&self) -> PageRect;
    fn svg_for(&self, rect: PageRect, zoom: f64) -> Result<String, MapViewError>;
    fn hit_at(&self, page: (f64, f64), tolerance_mm: f64) -> Vec<MapHit>;
    fn is_placeholder(&self) -> bool;
}

/// Frame, grid and a note; no map content.
pub(crate) struct PlaceholderMap {
    min: MapPoint,
    max: MapPoint,
    note: String,
}

impl PlaceholderMap {
    pub(crate) fn new(index: &MapIndex, note: String) -> Self {
        Self::from_extent(index.extent(), note)
    }

    /// An empty map gets a 100 x 100 mm box.
    pub(crate) fn from_extent(extent: Option<(MapPoint, MapPoint)>, note: String) -> Self {
        let (min, max) =
            extent.unwrap_or((MapPoint { x: 0.0, y: 0.0 }, MapPoint { x: 100.0, y: 100.0 }));
        Self { min, max, note }
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Multiples of `step` in `[lo, hi]`.
fn multiples(lo: f64, hi: f64, step: f64) -> impl Iterator<Item = f64> {
    let first = (lo / step).ceil() as i64;
    let last = (hi / step).floor() as i64;
    (first..=last).map(move |i| i as f64 * step)
}

impl MapView for PlaceholderMap {
    fn mapping(&self) -> Affine {
        Affine {
            k: 1.0,
            dx: MARGIN_MM - self.min.x,
            dy: MARGIN_MM - self.min.y,
        }
    }

    fn page_box(&self) -> PageRect {
        PageRect {
            x0: 0.0,
            y0: 0.0,
            x1: self.max.x - self.min.x + 2.0 * MARGIN_MM,
            y1: self.max.y - self.min.y + 2.0 * MARGIN_MM,
        }
    }

    fn svg_for(&self, rect: PageRect, zoom: f64) -> Result<String, MapViewError> {
        let (w, h) = (rect.x1 - rect.x0, rect.y1 - rect.y0);
        let valid = [rect.x0, rect.y0, rect.x1, rect.y1, zoom]
            .iter()
            .all(|v| v.is_finite())
            && w > 0.0
            && h > 0.0
            && zoom > 0.0;
        if !valid {
            return Err(MapViewError::Render("invalid rectangle or zoom".into()));
        }
        if w * zoom > MAX_PIXELS || h * zoom > MAX_PIXELS {
            return Err(MapViewError::TooLarge);
        }
        let map = self.mapping();
        let page = self.page_box();
        // The grid lies on map millimetres inside the page box and the requested rectangle.
        let (gx0, gx1) = (rect.x0.max(page.x0), rect.x1.min(page.x1));
        let (gy0, gy1) = (rect.y0.max(page.y0), rect.y1.min(page.y1));
        let mut step = GRID_MM;
        while (page.x1 - page.x0).max(page.y1 - page.y0) / step > MAX_GRID_LINES {
            step *= 10.0;
        }
        let mut grid = String::new();
        if gx0 < gx1 && gy0 < gy1 {
            for mx in multiples((gx0 - map.dx) / map.k, (gx1 - map.dx) / map.k, step) {
                let x = map.k * mx + map.dx;
                let _ = write!(grid, "M{x:.3} {gy0:.3}V{gy1:.3}");
            }
            for my in multiples((gy0 - map.dy) / map.k, (gy1 - map.dy) / map.k, step) {
                let y = map.k * my + map.dy;
                let _ = write!(grid, "M{gx0:.3} {y:.3}H{gx1:.3}");
            }
        }
        let (fx0, fy0) = (MARGIN_MM, MARGIN_MM);
        let (fx1, fy1) = (page.x1 - MARGIN_MM, page.y1 - MARGIN_MM);
        let mut svg = String::new();
        let _ = write!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.3} {:.3} {w:.3} {h:.3}\" \
             width=\"{:.1}\" height=\"{:.1}\">",
            rect.x0,
            rect.y0,
            w * zoom,
            h * zoom
        );
        if !grid.is_empty() {
            let _ = write!(
                svg,
                "<path class=\"grid\" fill=\"none\" stroke=\"#c8c8c8\" stroke-width=\"0.1\" d=\"{grid}\"/>"
            );
        }
        let _ = write!(
            svg,
            "<path class=\"frame\" fill=\"none\" stroke=\"#808080\" stroke-width=\"0.3\" \
             d=\"M{fx0:.3} {fy0:.3}H{fx1:.3}V{fy1:.3}H{fx0:.3}Z\"/>"
        );
        let _ = write!(
            svg,
            "<text x=\"{:.3}\" y=\"{:.3}\" text-anchor=\"middle\" font-size=\"4\" fill=\"#606060\">{}</text></svg>",
            page.x1 / 2.0,
            page.y1 / 2.0,
            escape(&self.note)
        );
        Ok(svg)
    }

    fn hit_at(&self, _page: (f64, f64), _tolerance_mm: f64) -> Vec<MapHit> {
        Vec::new()
    }

    fn is_placeholder(&self) -> bool {
        true
    }
}

/// Length and modification time of a file: when either changes, the cached map is stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileStamp {
    pub len: u64,
    pub modified: SystemTime,
}

impl FileStamp {
    /// The stamp as a string the page compares for equality.
    pub(crate) fn token(&self) -> String {
        let since = self
            .modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        format!(
            "{}-{}.{:09}",
            self.len,
            since.as_secs(),
            since.subsec_nanos()
        )
    }

    pub(crate) fn of(path: &Path) -> std::io::Result<Self> {
        let meta = std::fs::metadata(path)?;
        Ok(Self {
            len: meta.len(),
            modified: meta.modified()?,
        })
    }
}

/// What is built from one state of the map file.
pub(crate) struct MapEntry {
    pub stamp: FileStamp,
    pub view: Arc<dyn MapView>,
    pub index: Arc<MapIndex>,
    pub doc: Arc<MapDocument>,
}

/// What a map file yields: the entry, or the localized reason why not. Failures are cached like successes.
pub(crate) type MapResult = Result<Arc<MapEntry>, String>;

/// Called on the blocking thread right before a map file is parsed (tests slow the parse down with it).
pub(crate) type ParseHook = Arc<dyn Fn() + Send + Sync>;

struct Slot {
    path: PathBuf,
    stamp: FileStamp,
    cell: Arc<OnceCell<MapResult>>,
}

/// The server's map cache: one slot, keyed by the map file's path and stamp. Parsing happens outside the
/// state lock (see [`load_map`]); concurrent requests for the same key wait on one parse.
pub(crate) struct MapCache {
    note: String,
    slot: Option<Slot>,
    pub parse_hook: Option<ParseHook>,
}

impl MapCache {
    pub(crate) fn new(note: String) -> Self {
        Self {
            note,
            slot: None,
            parse_hook: None,
        }
    }

    /// The cell for this key; a new key replaces the old slot.
    fn cell(&mut self, path: &Path, stamp: FileStamp) -> Arc<OnceCell<MapResult>> {
        if let Some(slot) = &self.slot
            && slot.path == path
            && slot.stamp == stamp
        {
            return slot.cell.clone();
        }
        let cell = Arc::new(OnceCell::new());
        self.slot = Some(Slot {
            path: path.to_owned(),
            stamp,
            cell: cell.clone(),
        });
        cell
    }
}

/// Reads and indexes the map file. Errors are localized, without the absolute path.
fn build(path: &Path, stamp: FileStamp, note: String, lang: &Lang) -> MapResult {
    let doc = read_map_file(path).map_err(|e| {
        let reason = e.localize(lang);
        tidy(
            &text(
                lang,
                "error-editor-map",
                &[
                    ("file", FluentValue::from(file_name(path))),
                    ("reason", FluentValue::from(reason)),
                ],
            ),
            &[path],
        )
    })?;
    let index = MapIndex::from_document(&doc);
    let view = PlaceholderMap::new(&index, note);
    Ok(Arc::new(MapEntry {
        stamp,
        view: Arc::new(view),
        index: Arc::new(index),
        doc: Arc::new(doc),
    }))
}

/// The map at `map_path` through the cache. The state lock is held only to find the cache cell; the parse
/// runs on the blocking pool, so other requests (`quit`, `revision`) are not held up.
pub(crate) async fn load_map(state: &Shared, map_path: PathBuf) -> MapResult {
    let lang = state.lock().await.lang.clone();
    let stamp = FileStamp::of(&map_path).map_err(|e| {
        tidy(
            &text(
                &lang,
                "error-editor-map",
                &[
                    ("file", FluentValue::from(file_name(&map_path))),
                    ("reason", FluentValue::from(e.to_string())),
                ],
            ),
            &[&map_path],
        )
    })?;
    let (cell, note, hook) = {
        let mut guard = state.lock().await;
        let cell = guard.map.cell(&map_path, stamp);
        let hook = guard.map.parse_hook.clone();
        (cell, guard.map.note.clone(), hook)
    };
    cell.get_or_init(|| async move {
        tokio::task::spawn_blocking(move || {
            if let Some(hook) = hook {
                hook();
            }
            build(&map_path, stamp, note, &lang)
        })
        .await
        .unwrap_or_else(|e| Err(e.to_string()))
    })
    .await
    .clone()
}
