//! The map from the shared renderer (ADR 0046, D13): `freemapper-render` builds one page of the whole map at
//! map scale, draws parts of it as SVG and answers which map object lies under a point.
//!
//! The page is built once per state of the map file. Its coordinates are page millimetres; `PageMapping` shifts
//! map millimetres into them by one offset, which becomes the [`Affine`] the overlay uses.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, RwLock};

use freemapper_courseplan::map::MapIndex;
use freemapper_courseplan::omap::MapDocument;
use freemapper_render::build::{BuildReport, Orientation, PageSpec, Paper, build_page};
use freemapper_render::overprint::{IOF_PURPLE, on_map};
use freemapper_render::page::{Overprint, PageMapping, PaperBox, RenderedPage};
use freemapper_render::svg::{SvgError, SvgView, write_svg};

use super::map_view::{Affine, MapHit, MapView, MapViewError, PageRect};

/// Margin around the map's extent in the page box the editor fits to, in mm.
const BOX_MARGIN_MM: f64 = 10.0;

/// Largest SVG side in pixels; bigger requests are refused before the renderer is asked.
const MAX_PIXELS: f64 = 100_000.0;

/// Both factors are 1: the page shows the map at its own scale with the symbols as the file defines them.
const SYMBOL_FACTOR: f64 = 1.0;
const GEOMETRY_FACTOR: f64 = 1.0;

/// The page the renderer builds for the editor. The editor draws the whole map, never a print layout, so the
/// frame margin is 0 and the sheet objects stay out. A3 is the largest paper the renderer knows.
fn page_spec(symbol_factor: f64, geometry_factor: f64) -> Result<PageSpec, MapViewError> {
    let ok = |f: f64| f.is_finite() && f > 0.0;
    if !ok(symbol_factor) || !ok(geometry_factor) {
        return Err(MapViewError::Render(
            "the symbol and geometry factors must be finite and above 0".into(),
        ));
    }
    let mut spec = PageSpec::new(symbol_factor, geometry_factor, Paper::A3);
    spec.orientation = Orientation::Auto;
    spec.frame_margin_mm = Some(0.0);
    Ok(spec)
}

/// `page = k * map + d` from the renderer's mapping, checked at two points: `to_page` must move every point by
/// the same offset (scale 1), as the renderer promises. Anything else is an error, not a guess.
fn affine_of(mapping: PageMapping) -> Result<Affine, MapViewError> {
    let (ox, oy) = mapping.to_page((0.0, 0.0));
    let (px, py) = mapping.to_page((100.0, 50.0));
    let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
    if !(ox.is_finite() && oy.is_finite() && close(px - ox, 100.0) && close(py - oy, 50.0)) {
        return Err(MapViewError::Render(
            "the page mapping is not a scale of 1 plus an offset".into(),
        ));
    }
    Ok(Affine {
        k: 1.0,
        dx: ox,
        dy: oy,
    })
}

fn check_view(rect: PageRect, zoom: f64) -> Result<(), MapViewError> {
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
    Ok(())
}

fn svg_error(e: SvgError) -> MapViewError {
    match e {
        SvgError::TooLarge => MapViewError::TooLarge,
        other => MapViewError::Render(other.to_string()),
    }
}

type ViewKey = [u64; 5];

/// The map as the renderer draws it.
pub(crate) struct RenderMap {
    /// Behind a lock only because the print preview sets `overprint` for the length of one render.
    page: RwLock<RenderedPage>,
    affine: Affine,
    codes: HashMap<u32, String>,
    /// The last SVG: a repeated view (a reload, a second tab) does not render again. The lock is held only to
    /// look up and to store, never while the renderer writes.
    last: Mutex<Option<(ViewKey, String)>>,
    renders: AtomicUsize,
    /// What the editor fits to: the extent of the map's objects plus a margin, inside the page.
    view_box: PageRect,
    /// The map reaches beyond the page, so the renderer clipped it.
    clipped: bool,
    /// A renderer limit cut objects short or left them out.
    incomplete: bool,
}

/// What the renderer could not show: `(clipped, incomplete)`. `skipped_objects` (hidden symbols) is silent on
/// purpose; the three limits below lose map content the user wants to see.
pub(super) fn limits_of(report: &BuildReport) -> (bool, bool) {
    (
        report.overflow_mm.is_some(),
        report.truncated_objects > 0
            || report.objects_not_drawn_page_limit > 0
            || report.dropped_pattern_objects > 0,
    )
}

/// The page while a course is painted into it: the course is taken off again when this goes out of scope, also
/// when the render panics, so no later plain render carries a stale course.
struct WithCourse<'a>(std::sync::RwLockWriteGuard<'a, RenderedPage>);

impl Drop for WithCourse<'_> {
    fn drop(&mut self) {
        self.0.overprint = Overprint::default();
    }
}

impl RenderMap {
    pub(crate) fn new(doc: &MapDocument, index: &MapIndex) -> Result<Self, MapViewError> {
        let spec = page_spec(SYMBOL_FACTOR, GEOMETRY_FACTOR)?;
        let (page, report) =
            build_page(doc, &spec).map_err(|e| MapViewError::Render(e.to_string()))?;
        let affine = affine_of(page.mapping())?;
        let codes = doc
            .symbol_set
            .codes()
            .map(|(code, id)| (id, code.to_owned()))
            .collect();
        let whole = PageRect {
            x0: 0.0,
            y0: 0.0,
            x1: page.page_mm.0,
            y1: page.page_mm.1,
        };
        // The renderer centres the painted bounds on the page, so the box is symmetric around the page centre and
        // holds the extent of the objects plus a margin; an empty map shows the whole page.
        let view_box = index.extent().map_or(whole, |(min, max)| {
            let (a, b) = (affine.to_page(min), affine.to_page(max));
            let (cx, cy) = (whole.x1 / 2.0, whole.y1 / 2.0);
            let hx = (a.0 - cx).abs().max((b.0 - cx).abs()) + BOX_MARGIN_MM;
            let hy = (a.1 - cy).abs().max((b.1 - cy).abs()) + BOX_MARGIN_MM;
            PageRect {
                x0: (cx - hx).max(whole.x0),
                y0: (cy - hy).max(whole.y0),
                x1: (cx + hx).min(whole.x1),
                y1: (cy + hy).min(whole.y1),
            }
        });
        let (clipped, incomplete) = limits_of(&report);
        Ok(Self {
            view_box,
            page: RwLock::new(page),
            affine,
            codes,
            last: Mutex::new(None),
            renders: AtomicUsize::new(0),
            clipped,
            incomplete,
        })
    }

    /// How many times the renderer wrote an SVG (tests check the cache with it).
    pub(crate) fn render_count(&self) -> usize {
        self.renders.load(Ordering::Relaxed)
    }

    /// True when the map is larger than the page and the renderer cut it off.
    pub(crate) fn is_clipped(&self) -> bool {
        self.clipped
    }

    /// True when a renderer limit cut objects short or left them out.
    pub(crate) fn is_incomplete(&self) -> bool {
        self.incomplete
    }
}

impl MapView for RenderMap {
    fn mapping(&self) -> Affine {
        self.affine
    }

    fn page_box(&self) -> PageRect {
        self.view_box
    }

    fn is_clipped(&self) -> bool {
        self.clipped
    }

    fn is_incomplete(&self) -> bool {
        self.incomplete
    }

    fn svg_for(&self, rect: PageRect, zoom: f64) -> Result<String, MapViewError> {
        check_view(rect, zoom)?;
        let key: ViewKey = [rect.x0, rect.y0, rect.x1, rect.y1, zoom].map(f64::to_bits);
        {
            let last = self.last.lock().unwrap_or_else(|e| e.into_inner());
            if let Some((k, svg)) = last.as_ref()
                && *k == key
            {
                return Ok(svg.clone());
            }
        }
        let view = SvgView {
            rect: Some(PaperBox {
                min: (rect.x0, rect.y0),
                max: (rect.x1, rect.y1),
            }),
            zoom,
        };
        self.renders.fetch_add(1, Ordering::Relaxed);
        let svg = {
            let page = self.page.read().unwrap_or_else(|e| e.into_inner());
            write_svg(&page, &view).map_err(svg_error)?
        };
        *self.last.lock().unwrap_or_else(|e| e.into_inner()) = Some((key, svg.clone()));
        Ok(svg)
    }

    /// Not cached: the course changes with every edit. The lower layer is placed among the map colours by
    /// `on_map`; the page has no overprint again when the render is done.
    fn svg_with_course(
        &self,
        rect: PageRect,
        zoom: f64,
        overprint: Overprint,
    ) -> Result<String, MapViewError> {
        check_view(rect, zoom)?;
        let view = SvgView {
            rect: Some(PaperBox {
                min: (rect.x0, rect.y0),
                max: (rect.x1, rect.y1),
            }),
            zoom,
        };
        self.renders.fetch_add(1, Ordering::Relaxed);
        let mut page = WithCourse(self.page.write().unwrap_or_else(|e| e.into_inner()));
        page.0.overprint = on_map(overprint, IOF_PURPLE, &page.0.colours, &page.0.primitives);
        write_svg(&page.0, &view).map_err(svg_error)
    }

    /// The map objects under `page`, topmost first. `object` is the renderer's object number (the objects it
    /// draws, in the order of the map parts; ADR 0023 of freemapper-core), not an index into the file.
    fn hit_at(&self, page: (f64, f64), tolerance_mm: f64) -> Vec<MapHit> {
        self.page
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .hits_at(page, tolerance_mm)
            .into_iter()
            .map(|h| MapHit {
                symbol_code: self.codes.get(&h.symbol).cloned().unwrap_or_default(),
                object: h.object,
            })
            .collect()
    }

    fn is_placeholder(&self) -> bool {
        false
    }
}
