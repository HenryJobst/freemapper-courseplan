//! A small synthetic sprint map for the score tools: twelve buildings on a paved area at 1:4000, so that
//! suggestions, drafts and length analyses run in seconds. The symbol set is the one of the Luebeck
//! fixture, which gives the map the ISSprOM symbol codes the route search and the candidates read.

use std::path::{Path, PathBuf};

const BUILDING: u32 = 141;
const PAVED: u32 = 105;

fn square(symbol: u32, cx: f64, cy: f64, w: f64, h: f64) -> String {
    let c = |v: f64| (v * 1000.0).round() as i64;
    let (x0, y0, x1, y1) = (cx - w / 2.0, cy - h / 2.0, cx + w / 2.0, cy + h / 2.0);
    format!(
        "<object type=\"1\" symbol=\"{symbol}\"><coords count=\"5\">{} {};{} {};{} {};{} {};{} {} 18;</coords>\
         <pattern rotation=\"0\"><coord x=\"0\" y=\"0\"/></pattern></object>\n",
        c(x0),
        c(y0),
        c(x1),
        c(y0),
        c(x1),
        c(y1),
        c(x0),
        c(y1),
        c(x0),
        c(y0),
    )
}

/// Writes `synthetic.omap` into `dir`.
pub fn write_map(dir: &Path) -> PathBuf {
    let real = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/maps/luebeck.omap");
    let text = std::fs::read_to_string(real).unwrap();
    let head = &text[..text.find("<parts count").unwrap()];
    let tail = &text[text.find("</parts>").unwrap()..];
    let mut objects = square(PAVED, 0.0, 0.0, 200.0, 110.0);
    for cx in [-60.0, -20.0, 20.0, 60.0] {
        for cy in [-36.0, 0.0, 36.0] {
            objects.push_str(&square(BUILDING, cx, cy, 14.0, 10.0));
        }
    }
    let map = format!(
        "{head}<parts count=\"1\" current=\"0\">\n<part name=\"default part\"><objects count=\"13\">\n{objects}</objects></part>\n{tail}"
    );
    let path = dir.join("synthetic.omap");
    std::fs::write(&path, map).unwrap();
    path
}

/// Writes the map and a project with start S1 and finish F1 at the given positions; returns the project path.
/// The runner bands are wide open: the score settings keep controls 100 m apart, which leaves this small map
/// few sites, so a draft cannot hit the narrow default bands. The reach is wide too, so a draft sees the sites that
/// `score suggest` lists without a course. The tests here check the plumbing, not the bands.
pub fn write_project(dir: &Path, start: (f64, f64), finish: (f64, f64)) -> PathBuf {
    write_map(dir);
    let project = dir.join("event.courseplan.toml");
    std::fs::write(
        &project,
        format!(
            "format = 1\n\n[event]\ntitle = \"Synthetic\"\ndiscipline = \"training\"\nrule_profile = \"wkb-2026\"\n\n\
             [sources]\nmap = \"synthetic.omap\"\n\n\
             [[specials]]\nkind = \"start\"\nid = \"S1\"\nmap_mm = [{}, {}]\n\n\
             [[specials]]\nkind = \"finish\"\nid = \"F1\"\nmap_mm = [{}, {}]\n\n\
             [score]\ngood_band_percent = [0, 100]\nvery_good_band_percent = [0, 100]\nmax_reach_m = 5000\n",
            start.0, start.1, finish.0, finish.1
        ),
    )
    .unwrap();
    project
}

/// The usual start and finish, left of the buildings and between their rows.
pub fn project(dir: &Path) -> PathBuf {
    write_project(dir, (-92.0, -18.0), (-92.0, 18.0))
}
