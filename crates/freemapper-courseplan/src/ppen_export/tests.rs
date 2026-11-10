use freemapper_courseplan_model::ControlDescription;

use super::*;
use crate::ppen_import::import_columns;

fn some(s: &str) -> Option<String> {
    Some(s.to_owned())
}

/// Every Purple Pen id the import's table knows (the enumeration of the inverse, forwards).
fn all_ids() -> Vec<String> {
    let mut suffixes = vec![""];
    suffixes.extend(DIRECTIONS);
    suffixes.extend(["high", "deep", "single", "control"]);
    let mut ids = Vec::new();
    for section in 0..=15 {
        for item in 1..=30 {
            for suffix in &suffixes {
                let id = format!("{section}.{item}{suffix}");
                if iof_2024(&id).is_some() {
                    ids.push(id);
                }
            }
        }
    }
    ids
}

fn from_columns(columns: [Option<String>; 6]) -> ControlDescription {
    import_columns(columns)
}

fn round_trips(d: &ControlDescription) {
    let (cols, left_out) = ppen_columns(d);
    assert!(left_out.is_empty(), "{d:?}: {left_out:?}");
    assert_eq!(&from_columns(cols.clone()), d, "via {cols:?}");
}

#[test]
fn ppen_symbol_inverts_iof_2024() {
    let ids = all_ids();
    assert!(ids.len() > 150);
    // One id per column.
    for i in 0..6 {
        for id in &ids {
            let mut cols: [Option<String>; 6] = Default::default();
            cols[i] = Some(id.clone());
            round_trips(&from_columns(cols));
        }
    }
    // Two ids in neighbouring columns (D with E, F with G), which exercise the bend and `single` rules.
    for (a, b) in [(1, 2), (3, 4)] {
        for x in &ids {
            for y in &ids {
                let mut cols: [Option<String>; 6] = Default::default();
                cols[a] = Some(x.clone());
                cols[b] = Some(y.clone());
                round_trips(&from_columns(cols));
            }
        }
    }
}

#[test]
fn explicit_rows() {
    let col = |c: Option<&str>, f: Option<&str>| {
        let d = ControlDescription {
            c: c.and_then(some),
            d: None,
            e: None,
            f: f.and_then(some),
            ..Default::default()
        };
        ppen_columns(&d).0
    };
    let one = |d: ControlDescription| ppen_columns(&d).0;
    let d = |x: &str| ControlDescription {
        d: some(x),
        ..Default::default()
    };
    assert_eq!(one(d("5.21"))[1], some("5.23"));
    assert_eq!(
        one(ControlDescription {
            g: some("12.1NE"),
            ..Default::default()
        })[4],
        some("11.1NE")
    );
    assert_eq!(
        one(ControlDescription {
            h: some("13.1"),
            ..Default::default()
        })[5],
        some("12.1")
    );
    assert_eq!(col(Some("15.3"), None)[0], some("13.3"));
    assert_eq!(col(Some("16.3"), None)[0], some("14.3"));
    // Bend: F goes to G when G is empty, stays in F otherwise.
    let bend = col(None, Some("11.1"));
    assert_eq!((&bend[3], &bend[4]), (&None, &some("11.7")));
    let kept = one(ControlDescription {
        f: some("11.1"),
        g: some("12.1N"),
        ..Default::default()
    });
    assert_eq!((&kept[3], &kept[4]), (&some("11.7"), &some("11.1N")));
    // Objects of the same kind.
    let single = one(ControlDescription {
        d: some("10.1"),
        e: some("10.1"),
        ..Default::default()
    });
    assert_eq!((&single[1], &single[2]), (&some("10.1single"), &None));
    let single = one(ControlDescription {
        d: some("12.14"),
        e: some("12.14"),
        ..Default::default()
    });
    assert_eq!(single[1], some("11.15single"));
    // Equal ids without a `single` variant stay in both columns.
    let both = one(ControlDescription {
        d: some("5.2"),
        e: some("5.2"),
        ..Default::default()
    });
    assert_eq!((&both[1], &both[2]), (&some("5.2"), &some("5.2")));
}

#[test]
fn plain_id_wins_over_variants() {
    let t = inverse();
    assert_eq!(t["10.1"], "10.1");
    assert_eq!(t["15.5"], "13.5");
    assert_eq!(t["12.14"], "11.15");
    assert_eq!(t["9.1"], "9.1");
}

#[test]
fn symbol_without_purple_pen_id_is_reported() {
    // 2024 has symbols Purple Pen lacks; the column is left empty and named, not replaced.
    let d = ControlDescription {
        d: some("5.2"),
        e: some("99.9"),
        h: some("13.9"),
        ..Default::default()
    };
    let (cols, left_out) = ppen_columns(&d);
    assert_eq!(cols[1], some("5.2"));
    assert_eq!((&cols[2], &cols[5]), (&None, &None));
    assert_eq!(
        left_out,
        vec![('E', "99.9".to_owned()), ('H', "13.9".to_owned())]
    );
}
