//! Export of the project to a Purple Pen event (`.ppen`).
//!
//! - Descriptions: a `.ppen` stores Purple Pen's symbol ids, which are 2004 numbers with suffixes, while the
//!   project stores IOF Control Descriptions 2024 numbers. [`ppen_columns`] inverts the import's table
//!   ([`iof_2024`]). The inverse is built by enumerating every Purple Pen id through the import's table, so
//!   it cannot drift from it. A 2024 number that no Purple Pen id maps to has no counterpart in Purple Pen;
//!   it is left out of the exported description and reported, never replaced by a different symbol.
//!
//! Terms follow `docs/glossary.md`.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use freemapper_courseplan_model::ControlDescription;

use crate::ppen_import::{DIRECTIONS, iof_2024};

/// The Purple Pen symbol id of each IOF 2024 number the import can produce. Where two Purple Pen ids give
/// the same number (`10.1` and `10.1single`, `13.5` and `13.5control`), the plain id wins.
fn inverse() -> &'static BTreeMap<String, String> {
    static TABLE: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut suffixes = vec![""];
        suffixes.extend(DIRECTIONS);
        suffixes.extend(["high", "deep", "single", "control"]);
        let mut table: BTreeMap<String, String> = BTreeMap::new();
        for section in 0..=15 {
            for item in 1..=30 {
                for suffix in &suffixes {
                    let id = format!("{section}.{item}{suffix}");
                    let Some(mapped) = iof_2024(&id) else {
                        continue;
                    };
                    // The plain id comes first and is never replaced by a longer one.
                    table.entry(mapped.id).or_insert(id);
                }
            }
        }
        table
    })
}

/// 2024 numbers of "objects of the same kind" (crossing, junction, between): Purple Pen has one symbol
/// with the variant `single` for the pair of columns D and E.
const SINGLE_CAPABLE: [&str; 3] = ["10.1", "10.2", "12.14"];

/// Purple Pen symbol ids for boxes C..H of one courseplan description, and the (column, 2024 id) pairs
/// that have no Purple Pen id and are left out.
///
/// Two import rules run backwards: the bend `11.1` in column F goes to box G as `11.7` when G is empty
/// (Purple Pen keeps the bend in G; the import moves it to F); and D equal to E with a `single` variant
/// becomes `<id>single` in D with E empty.
#[allow(dead_code)] // used by the .ppen writer
pub(crate) fn ppen_columns(d: &ControlDescription) -> ([Option<String>; 6], Vec<(char, String)>) {
    let table = inverse();
    let mut cols: [Option<String>; 6] = Default::default();
    let mut left_out = Vec::new();
    let given = [&d.c, &d.d, &d.e, &d.f, &d.g, &d.h];
    for (i, id) in given.into_iter().enumerate() {
        let Some(id) = id else { continue };
        match table.get(id) {
            Some(pp) => cols[i] = Some(pp.clone()),
            None => left_out.push((char::from(b'C' + i as u8), id.clone())),
        }
    }
    // Bend: column F in the project, column G in Purple Pen.
    if d.f.as_deref() == Some("11.1") && cols[3].is_some() && d.g.is_none() {
        cols[4] = cols[3].take();
    }
    // Objects of the same kind: one symbol with the variant "single" in D, E empty.
    if let (Some(dd), Some(ee)) = (&d.d, &d.e)
        && dd == ee
        && SINGLE_CAPABLE.contains(&dd.as_str())
        && let Some(pp) = &cols[1]
    {
        cols[1] = Some(format!("{pp}single"));
        cols[2] = None;
    }
    (cols, left_out)
}

#[cfg(test)]
mod tests;
