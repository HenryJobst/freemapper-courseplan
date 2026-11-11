//! The control description symbols for the column pickers (`GET api/symbols?column=C..H`, ADR 0051): the IOF
//! 2024 ids a column offers, their text in the process language and their drawing as SVG.
//!
//! The ids come from core's `ids(standard)`, mapped to IOF 2024 numbers with `iof_2024`, and are sorted into
//! columns by `symbol_kind`. Nothing is typed by hand.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

use freemapper_course::description_symbols::{
    DIRECTIONS, Standard, SymbolGeometry, TextLang, ids, iof_2024, ppen_symbol, symbol,
    symbol_kind, symbol_text,
};
use freemapper_courseplan::text::Lang;
use serde_json::{Value, json};

/// Purple Pen knows two standards; 2024 kept the symbols of 2018.
const STANDARD: Standard = Standard::Iof2018;

/// One offered symbol: the IOF 2024 id (what the project file stores) and the Purple Pen id that draws it.
struct Listed {
    id: String,
    ppen: String,
}

/// The column (`C` to `H`) as an index 0 to 5, or `None` for any other text.
pub(crate) fn column_index(column: &str) -> Option<usize> {
    let mut chars = column.chars();
    let letter = chars.next()?;
    (chars.next().is_none() && ('C'..='H').contains(&letter))
        .then(|| (letter as u8 - b'C') as usize)
}

/// Whether a symbol of Purple Pen's `kind` goes in column `index` (0 is C): the kind named for the column,
/// and column E takes the D symbols too (as `ppen_export` and Purple Pen do).
fn accepts(index: usize, kind: char) -> bool {
    let column = char::from(b'C' + index as u8);
    kind == column || (index == 2 && kind == 'D')
}

/// The sort key of an IOF 2024 id: section, item, direction (none first).
fn sort_key(id: &str) -> (u32, u32, usize) {
    let split = id
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(id.len());
    let (base, suffix) = id.split_at(split);
    let (section, item) = base.split_once('.').unwrap_or((base, "0"));
    let direction = DIRECTIONS
        .iter()
        .position(|d| *d == suffix)
        .map_or(0, |p| p + 1);
    (
        section.parse().unwrap_or(0),
        item.parse().unwrap_or(0),
        direction,
    )
}

/// The symbols of every column, found once: the ids core defines for the standard (Purple Pen's, which are
/// the 2004 numbers with variant suffixes), turned into IOF 2024 numbers by `iof_2024`. The variants
/// (`9.1high`, `10.1single`) share the 2024 number of the plain symbol and are listed once; the drawing and
/// the column are those of the Purple Pen id `ppen_symbol` chooses for the 2024 number.
fn columns() -> &'static [Vec<Listed>; 6] {
    static COLUMNS: OnceLock<[Vec<Listed>; 6]> = OnceLock::new();
    COLUMNS.get_or_init(|| {
        let mut found: BTreeMap<(u32, u32, usize), String> = BTreeMap::new();
        for id in ids(STANDARD).filter(|id| id.starts_with(|c: char| c.is_ascii_digit())) {
            if let Some(mapped) = iof_2024(id)
                && !mapped.single
            {
                found.insert(sort_key(&mapped.id), mapped.id);
            }
        }
        let mut out: [Vec<Listed>; 6] = Default::default();
        for id in found.into_values() {
            let Ok(inverse) = ppen_symbol(&id, STANDARD) else {
                continue;
            };
            let Some(kind) = symbol_kind(&inverse.id, STANDARD) else {
                continue;
            };
            for (index, list) in out.iter_mut().enumerate() {
                if accepts(index, kind) {
                    list.push(Listed {
                        id: id.clone(),
                        ppen: inverse.id.clone(),
                    });
                }
            }
        }
        out
    })
}

/// The IOF 2024 ids column `index` offers, in the order of the standard.
#[cfg(test)]
fn ids_of(index: usize) -> Vec<&'static str> {
    columns()[index].iter().map(|l| l.id.as_str()).collect()
}

/// The number as the shortest text with at most four decimals.
fn num(v: f64) -> String {
    if !v.is_finite() {
        return "0".to_owned();
    }
    let mut text = format!("{v:.4}");
    if text.contains('.') {
        text.truncate(text.trim_end_matches('0').trim_end_matches('.').len());
    }
    if text == "-0" { "0".to_owned() } else { text }
}

fn points(list: &[(f64, f64)]) -> String {
    list.iter()
        .map(|&(x, y)| format!("{},{}", num(x), num(y)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The drawing of a symbol as an SVG element: `viewBox` `0 0 {width} 1` in cells, strokes with their widths
/// in the text colour (`currentColor`), filled areas with the even-odd rule, circles. Only numbers of the
/// geometry go into the text, so there is nothing to escape; a number that is not finite is written as 0.
pub(crate) fn symbol_svg(g: &SymbolGeometry) -> String {
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} 1\" aria-hidden=\"true\" focusable=\"false\">",
        num(g.width)
    );
    for stroke in &g.strokes {
        let tag = if stroke.closed { "polygon" } else { "polyline" };
        let cap = if stroke.round_ends { "round" } else { "butt" };
        let join = if stroke.round_corners {
            "round"
        } else {
            "miter"
        };
        let _ = write!(
            s,
            "<{tag} points=\"{}\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"{}\" stroke-linecap=\"{cap}\" stroke-linejoin=\"{join}\"/>",
            points(&stroke.points),
            num(stroke.width)
        );
    }
    if !g.fills.is_empty() {
        let mut d = String::new();
        for polygon in &g.fills {
            for (i, &(x, y)) in polygon.iter().enumerate() {
                let _ = write!(d, "{}{} {}", if i == 0 { "M" } else { "L" }, num(x), num(y));
            }
            d.push('Z');
        }
        let _ = write!(
            s,
            "<path d=\"{d}\" fill=\"currentColor\" fill-rule=\"evenodd\"/>"
        );
    }
    for c in &g.circles {
        let (cx, cy, r) = (num(c.centre.0), num(c.centre.1), num(c.radius));
        match c.stroke {
            Some(width) => {
                let _ = write!(
                    s,
                    "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"{}\"/>",
                    num(width)
                );
            }
            None => {
                let _ = write!(
                    s,
                    "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"currentColor\"/>"
                );
            }
        }
    }
    s.push_str("</svg>");
    s
}

/// The text of a symbol in the process language, English when that language has none, the id as the last
/// resort.
fn text_of(ppen: &str, id: &str, lang: &Lang) -> String {
    let wanted = if lang.code() == "de" {
        TextLang::De
    } else {
        TextLang::En
    };
    let raw = symbol_text(ppen, wanted)
        .or_else(|| symbol_text(ppen, TextLang::En))
        .unwrap_or(id);
    blank_placeholders(raw)
}

/// The text with each placeholder (`{0}`, `{1}`) written as an ellipsis: the picker shows the symbol, not a
/// filled-in description.
fn blank_placeholders(raw: &str) -> String {
    let mut out = String::new();
    let mut rest = raw;
    while let Some(start) = rest.find('{') {
        let Some(len) = rest[start..].find('}') else {
            break;
        };
        out.push_str(&rest[..start]);
        out.push('\u{2026}');
        rest = &rest[start + len + 1..];
    }
    out.push_str(rest);
    out
}

/// The answer of `api/symbols` for column `index`: `[{ "id", "text", "svg" }]`.
pub(crate) fn symbols_json(index: usize, lang: &Lang) -> Value {
    let list = columns().get(index).map_or(&[][..], Vec::as_slice);
    Value::Array(
        list.iter()
            .filter_map(|l| {
                let geometry = symbol(&l.ppen, STANDARD)?;
                Some(json!({
                    "id": l.id,
                    "text": text_of(&l.ppen, &l.id, lang),
                    "svg": symbol_svg(geometry),
                }))
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use freemapper_course::description_symbols::{Circle, Stroke, exists};
    use freemapper_courseplan::text::parse_lang_arg;
    use tokio::sync::{Mutex, Notify};
    use tower::ServiceExt;

    use super::*;
    use crate::editor::{EditorState, router};

    async fn get(lang: &Lang, uri: &str) -> (StatusCode, Value) {
        let state = EditorState::new("/p/event.courseplan.toml".into(), lang.clone());
        let app = router("/tok", Arc::new(Mutex::new(state)), Arc::new(Notify::new()));
        let response = app
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
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

    fn ids(body: &Value) -> Vec<&str> {
        body.as_array()
            .unwrap()
            .iter()
            .map(|e| e["id"].as_str().unwrap())
            .collect()
    }

    #[test]
    fn symbol_svg_of_a_circle_and_a_line() {
        let g = SymbolGeometry {
            kind: 'D',
            width: 1.0,
            strokes: vec![Stroke {
                points: vec![(0.1, 0.5), (0.9, 0.5)],
                width: 0.05,
                closed: false,
                round_ends: false,
                round_corners: true,
            }],
            fills: vec![],
            circles: vec![
                Circle {
                    centre: (0.5, 0.5),
                    radius: 0.3,
                    stroke: Some(0.05),
                },
                Circle {
                    centre: (0.25, 0.75),
                    radius: 0.0625,
                    stroke: None,
                },
            ],
        };
        assert_eq!(
            symbol_svg(&g),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\" aria-hidden=\"true\" focusable=\"false\">\
             <polyline points=\"0.1,0.5 0.9,0.5\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"0.05\" \
             stroke-linecap=\"butt\" stroke-linejoin=\"round\"/>\
             <circle cx=\"0.5\" cy=\"0.5\" r=\"0.3\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"0.05\"/>\
             <circle cx=\"0.25\" cy=\"0.75\" r=\"0.0625\" fill=\"currentColor\"/></svg>"
        );
    }

    #[test]
    fn symbol_svg_of_fills_and_wide_boxes_and_odd_numbers() {
        let g = SymbolGeometry {
            kind: 'V',
            width: 8.0,
            strokes: vec![Stroke {
                points: vec![(0.0, 0.0), (1.0, f64::NAN), (1.0, 1.0)],
                width: 0.1,
                closed: true,
                round_ends: true,
                round_corners: false,
            }],
            fills: vec![
                vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)],
                vec![(0.25, 0.25), (0.5, 0.25), (0.5, 0.5)],
            ],
            circles: vec![],
        };
        let svg = symbol_svg(&g);
        assert!(svg.contains("viewBox=\"0 0 8 1\""), "{svg}");
        assert!(svg.contains("<polygon points=\"0,0 1,0 1,1\""), "{svg}");
        assert!(svg.contains("stroke-linecap=\"round\" stroke-linejoin=\"miter\""));
        assert!(svg.contains(
            "<path d=\"M0 0L1 0L1 1ZM0.25 0.25L0.5 0.25L0.5 0.5Z\" fill=\"currentColor\" fill-rule=\"evenodd\"/>"
        ));
    }

    #[test]
    fn every_listed_id_exists_in_its_column() {
        let mut total = 0;
        for index in 0..6 {
            let list = columns()[index].as_slice();
            assert!(!list.is_empty(), "column {index} is empty");
            for l in list {
                total += 1;
                let inverse = ppen_symbol(&l.id, Standard::Iof2018).unwrap();
                assert_eq!(inverse.id, l.ppen);
                assert!(exists(&inverse.id, Standard::Iof2018), "{}", l.id);
                let kind = symbol_kind(&inverse.id, Standard::Iof2018).unwrap();
                assert!(
                    accepts(index, kind),
                    "{} is kind {kind} in column {index}",
                    l.id
                );
                assert!(symbol_text(&inverse.id, TextLang::En).is_some(), "{}", l.id);
            }
        }
        assert!(total > 150, "{total} symbols");
    }

    #[test]
    fn every_column_is_sorted_by_number_without_duplicates() {
        let key = sort_key;
        for index in 0..6 {
            let ids = ids_of(index);
            assert!(
                ids.windows(2).all(|w| key(w[0]) < key(w[1])),
                "column {index}: {ids:?}"
            );
        }
    }

    #[tokio::test]
    async fn column_d_lists_1_3_and_not_11_1() {
        let (status, body) = get(&Lang::english(), "/tok/api/symbols?column=D").await;
        assert_eq!(status, StatusCode::OK);
        let list = ids(&body);
        assert!(list.contains(&"1.3"), "{list:?}");
        // 11.1 is the bend, a column F symbol.
        assert!(!list.contains(&"11.1"), "{list:?}");
        let (_, f) = get(&Lang::english(), "/tok/api/symbols?column=F").await;
        assert!(ids(&f).contains(&"11.1"));
        let g = body
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == "1.3")
            .unwrap();
        assert!(g["svg"].as_str().unwrap().starts_with("<svg "));
        assert!(!g["text"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn column_e_takes_d_symbols() {
        let (_, d) = get(&Lang::english(), "/tok/api/symbols?column=D").await;
        let (_, e) = get(&Lang::english(), "/tok/api/symbols?column=E").await;
        let (d, e) = (ids(&d), ids(&e));
        assert!(d.iter().all(|id| e.contains(id)), "{e:?}");
        assert!(e.len() > d.len(), "E has symbols of its own");
        let (_, h) = get(&Lang::english(), "/tok/api/symbols?column=H").await;
        assert!(!ids(&h).contains(&"1.3"));
    }

    #[tokio::test]
    async fn texts_follow_the_language() {
        let german = parse_lang_arg("de").unwrap();
        let find = |body: &Value| {
            body.as_array()
                .unwrap()
                .iter()
                .find(|e| e["id"] == "1.3")
                .unwrap()["text"]
                .as_str()
                .unwrap()
                .to_owned()
        };
        let (_, en) = get(&Lang::english(), "/tok/api/symbols?column=D").await;
        let (_, de) = get(&german, "/tok/api/symbols?column=D").await;
        assert_ne!(find(&en), find(&de));
        assert_eq!(
            find(&de),
            blank_placeholders(symbol_text("1.3", TextLang::De).unwrap())
        );
        assert_eq!(
            find(&en),
            blank_placeholders(symbol_text("1.3", TextLang::En).unwrap())
        );
    }

    #[test]
    fn placeholders_become_ellipses() {
        assert_eq!(blank_placeholders("{0}, N side"), "\u{2026}, N side");
        assert_eq!(blank_placeholders("{0} x {1}"), "\u{2026} x \u{2026}");
        assert_eq!(blank_placeholders("plain {open"), "plain {open");
    }

    #[tokio::test]
    async fn bad_column_is_400() {
        for query in [
            "",
            "?column=",
            "?column=A",
            "?column=B",
            "?column=I",
            "?column=d",
            "?column=CD",
            "?column=C&column=D",
            "?column=C&x=1",
            "?column=%3Cb%3E",
            "?x=C",
        ] {
            let (status, _) = get(&Lang::english(), &format!("/tok/api/symbols{query}")).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{query}");
        }
    }
}
