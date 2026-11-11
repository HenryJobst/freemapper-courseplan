use freemapper_course::model::Offset;
use freemapper_course::{
    ControlKind as PKind, CourseFilter, ItemScaling, RawEvent, RawPoint, Special,
    placement_from_offset,
};
use freemapper_courseplan_model::{
    ControlDescription, ControlKind, CourseControl, CourseKind, Event, MapPoint, NumberPlacement,
    ScoreControl, SpecialObject,
};

use freemapper_course::description_symbols::{DIRECTIONS, iof_2024};

use super::*;
use crate::check::testing::{code, course, event, map, point_object, pt};
use crate::map::{MapKind, map_kind};
use crate::numbers::{circle_outer_radius_mm, number_box_mm};
use crate::omap::MapDocument;
use crate::ppen_import::{import_columns, import_ppen, read};

fn some(s: &str) -> Option<String> {
    Some(s.to_owned())
}

/// Every Purple Pen id the import's table knows, without the direction forms that 2024 has no number for
/// (`0.1NE`, `0.2N`: the core mapping writes the cardinal form as `0.1` and the diagonal form as `0.2`).
fn all_ids() -> Vec<String> {
    let mut suffixes = vec![""];
    suffixes.extend(DIRECTIONS);
    suffixes.extend(["high", "deep", "single", "control"]);
    let cardinal = |s: &str| matches!(s, "N" | "E" | "S" | "W");
    let mut ids = Vec::new();
    for section in 0..=15 {
        for item in 1..=30 {
            for suffix in &suffixes {
                let id = format!("{section}.{item}{suffix}");
                let wrong_form =
                    (section == 0 && item == 1 && !suffix.is_empty() && !cardinal(suffix))
                        || (section == 0 && item == 2 && cardinal(suffix));
                if !wrong_form && iof_2024(&id).is_some() {
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

/// The columns of a crossing point, which may keep an instruction row in column C where a normal control
/// may not (see `instruction_row_stays_out_of_column_c_of_a_normal_control`).
fn export_columns(d: &ControlDescription, standard: &str) -> PpenColumns {
    ppen_columns(d, standard, ControlKind::CrossingPoint)
}

fn refused(notes: &[ColumnNote]) -> bool {
    notes
        .iter()
        .any(|n| matches!(n, ColumnNote::Refused { .. }))
}

/// Both standards Purple Pen writes. Where the core mapping refuses a symbol (the file's standard lacks
/// it, or the column does not offer it), nothing is promised; otherwise the import gives the description
/// back. Returns whether the description was written whole.
fn round_trips(d: &ControlDescription) -> bool {
    let mut whole = true;
    for standard in ["2004", "2018"] {
        let out = export_columns(d, standard);
        if refused(&out.notes) {
            whole = false;
            continue;
        }
        assert_eq!(
            &from_columns(out.columns.clone()),
            d,
            "{standard} via {:?}",
            out.columns
        );
    }
    whole
}

#[test]
fn ppen_symbol_inverts_iof_2024() {
    let ids = all_ids();
    assert!(ids.len() > 150);
    // One id per column; every id fits some column in both standards or is a 2018 symbol.
    for id in &ids {
        let mut fits = false;
        for i in 0..6 {
            let mut cols: [Option<String>; 6] = Default::default();
            cols[i] = Some(id.clone());
            let d = from_columns(cols);
            fits |= export_columns(&d, "2018")
                .notes
                .iter()
                .all(|n| !matches!(n, ColumnNote::Refused { .. }));
            round_trips(&d);
        }
        assert!(fits, "{id} fits no column of a 2018 file");
    }
    // Two ids in neighbouring columns (D with E, F with G), which exercise the bend and `single` rules.
    let mut written = 0;
    for (a, b) in [(1, 2), (3, 4)] {
        for x in &ids {
            for y in &ids {
                let mut cols: [Option<String>; 6] = Default::default();
                cols[a] = Some(x.clone());
                cols[b] = Some(y.clone());
                written += usize::from(round_trips(&from_columns(cols)));
            }
        }
    }
    assert!(written > 1000, "{written}");
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
        ppen_columns(&d, "2004", ControlKind::CrossingPoint).columns
    };
    let one = |d: ControlDescription| export_columns(&d, "2004").columns;
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
    // Bend, 2004: F goes to G when G is empty, stays in F otherwise.
    let bend = col(None, Some("11.1"));
    assert_eq!((&bend[3], &bend[4]), (&None, &some("11.7")));
    // From 2018 on the bend is in box F (sampleevent8, standard 2018).
    let bend = export_columns(
        &ControlDescription {
            f: some("11.1"),
            ..Default::default()
        },
        "2018",
    )
    .columns;
    assert_eq!((&bend[3], &bend[4]), (&some("11.7"), &None));
    // 2004 with G taken: the core mapping refuses the bend in F (a bend belongs in G there). The old
    // inverse wrote 11.7 in F, which Purple Pen's 2004 popup does not offer.
    let kept = export_columns(
        &ControlDescription {
            f: some("11.1"),
            g: some("12.1N"),
            ..Default::default()
        },
        "2004",
    );
    assert_eq!(
        (&kept.columns[3], &kept.columns[4]),
        (&None, &some("11.1N"))
    );
    assert_eq!(
        kept.notes,
        vec![ColumnNote::Refused {
            column: 'F',
            id: "11.1".to_owned(),
            reason: Refusal::WrongColumn,
        }]
    );
    // Objects of the same kind: a combination in F with the same feature in D and E is the variant
    // `single` of the combination, and the feature stays in D (D never carries a description of its own).
    let single = one(ControlDescription {
        d: some("5.2"),
        e: some("5.2"),
        f: some("10.1"),
        ..Default::default()
    });
    assert_eq!(
        (&single[1], &single[2], &single[3]),
        (&some("5.2"), &None, &some("10.1single"))
    );
    // "Between" keeps both features and writes the plain symbol in G: Purple Pen rejects 11.15single.
    let between = one(ControlDescription {
        d: some("5.2"),
        e: some("5.2"),
        g: some("12.14"),
        ..Default::default()
    });
    assert_eq!(
        (&between[1], &between[2], &between[4]),
        (&some("5.2"), &some("5.2"), &some("11.15"))
    );
    // Equal ids without a combination in F stay in both columns.
    let both = one(ControlDescription {
        d: some("5.2"),
        e: some("5.2"),
        ..Default::default()
    });
    assert_eq!((&both[1], &both[2]), (&some("5.2"), &some("5.2")));
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
    let out = export_columns(&d, "2018");
    assert_eq!(out.columns[1], some("5.2"));
    assert_eq!((&out.columns[2], &out.columns[5]), (&None, &None));
    let refusal = |column, id: &str| ColumnNote::Refused {
        column,
        id: id.to_owned(),
        reason: Refusal::NoPurplePenId,
    };
    assert_eq!(out.notes, vec![refusal('E', "99.9"), refusal('H', "13.9")]);
}

#[test]
fn symbol_of_2018_in_a_2004_base_is_refused_and_noted() {
    let mut ev = event(&[("31", 10.0, 10.0)], vec![]);
    ev.controls[0].description = ControlDescription {
        d: some("2.10"),
        e: some("5.2"),
        ..Default::default()
    };
    let got = to_raw_in(&ev, MapKind::Sprint, 4000, ItemScaling::None, "2004", true);
    let d = raw_control(&got.event, "31").description.clone().unwrap();
    assert_eq!(d.columns, [None, None, some("5.2"), None, None, None]);
    assert!(got.notes.contains(&ExportNote::SymbolNotInStandard {
        control: "31".to_owned(),
        column: 'D',
        id: "2.10".to_owned(),
        standard: "2004".to_owned(),
    }));
    // The same symbol goes into a 2018 file.
    let got = sprint(&ev);
    let d = raw_control(&got.event, "31").description.clone().unwrap();
    assert_eq!(d.columns[1], some("2.10"));
}

#[test]
fn note_names_the_standard_the_export_used() {
    let refused = || ColumnNote::Refused {
        column: 'D',
        id: "2.10".to_owned(),
        reason: Refusal::NotInStandard(Standard::Iof2004),
    };
    let standard_of = |file: &str| match export_note("31", file, refused()) {
        ExportNote::SymbolNotInStandard { standard, .. } => standard,
        other => panic!("unexpected note {other:?}"),
    };
    assert_eq!(standard_of("2004"), "2004");
    assert_eq!(standard_of("2024"), "2024");
    // An unknown value is exported as 2018, so the note says 2018.
    assert_eq!(standard_of("1999"), "2018");
}

#[test]
fn plain_height_is_written_as_high_and_noted() {
    let mut ev = event(&[("31", 10.0, 10.0)], vec![]);
    ev.controls[0].description = ControlDescription {
        f: some("9.1"),
        ..Default::default()
    };
    let got = sprint(&ev);
    let d = raw_control(&got.event, "31").description.clone().unwrap();
    assert_eq!(d.columns[3], some("9.1high"));
    assert!(got.notes.contains(&ExportNote::SymbolVariantChosen {
        control: "31".to_owned(),
        column: 'F',
        id: "9.1".to_owned(),
        written: "9.1high".to_owned(),
    }));
}

#[test]
fn single_variant_round_trips_through_the_file() {
    let imported = from_columns([None, some("5.2"), None, some("10.1single"), None, None]);
    // The import names the feature in D and E, the combination in F.
    assert_eq!(
        (&imported.d, &imported.e, &imported.f),
        (&some("5.2"), &some("5.2"), &some("10.1"))
    );
    let out = export_columns(&imported, "2018");
    assert!(out.notes.is_empty(), "{:?}", out.notes);
    assert_eq!(
        out.columns,
        [None, some("5.2"), None, some("10.1single"), None, None]
    );
    // Through the writer and the reader of a whole file.
    let mut ev = event(&[("31", 10.0, 10.0)], vec![]);
    ev.controls[0].description = imported.clone();
    let got = sprint(&ev);
    let d = raw_control(&got.event, "31").description.clone().unwrap();
    assert_eq!(d.columns, out.columns);
}

#[test]
fn diagonal_column_c_is_written_as_0_2() {
    let c = |id: &str| {
        let d = ControlDescription {
            c: some(id),
            ..Default::default()
        };
        export_columns(&d, "2018").columns[0].clone()
    };
    assert_eq!(c("0.2NE"), some("0.2NE"));
    assert_eq!(c("0.1N"), some("0.1N"));
    // The cardinal form named with a diagonal direction (and the other way round) is normalised.
    assert_eq!(c("0.1NE"), some("0.2NE"));
    assert_eq!(c("0.2N"), some("0.1N"));
}

#[test]
fn instruction_row_stays_out_of_column_c_of_a_normal_control() {
    let d = ControlDescription {
        c: some("15.3"),
        ..Default::default()
    };
    let out = ppen_columns(&d, "2018", ControlKind::Control);
    assert_eq!(out.columns[0], None);
    assert_eq!(
        out.notes,
        vec![ColumnNote::Refused {
            column: 'C',
            id: "15.3".to_owned(),
            reason: Refusal::WrongColumn,
        }]
    );
    let out = ppen_columns(&d, "2018", ControlKind::CrossingPoint);
    assert_eq!(out.columns[0], some("13.3"));
}

// ---- to_raw ----

fn sprint(event: &Event) -> ToRaw {
    to_raw(event, MapKind::Sprint, 4000, ItemScaling::None, true)
}

fn raw_control<'a>(ev: &'a RawEvent, code: &str) -> &'a freemapper_course::Control<RawPoint> {
    ev.controls
        .iter()
        .find(|c| c.code == code)
        .unwrap_or_else(|| panic!("no control {code} in {:?}", ev.controls))
}

fn raw_course<'a>(ev: &'a RawEvent, name: &str) -> &'a freemapper_course::Course<RawPoint> {
    ev.courses
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no course {name} in {:?}", ev.courses))
}

fn rp(x_mm: f64, y_mm: f64) -> RawPoint {
    RawPoint { x_mm, y_mm }
}

fn individual(course: &mut freemapper_courseplan_model::Course) -> &mut Vec<CourseControl> {
    match &mut course.kind {
        CourseKind::Individual { controls } => controls,
        other => panic!("{other:?}"),
    }
}

fn water(id: &str, at: MapPoint, courses: &[&str]) -> SpecialObject {
    SpecialObject::Water {
        id: id.to_owned(),
        ppen_id: None,
        map_mm: at,
        courses: courses.iter().map(|c| (*c).to_owned()).collect(),
    }
}

#[test]
fn map_y_down_becomes_file_y_up() {
    let mut a = course("A", &["31"]);
    a.start_bends = vec![pt(1.0, 2.0)];
    individual(&mut a)[0].bends = vec![pt(20.5, 30.25)];
    let mut ev = event(&[("31", 12.345, 6.789)], vec![a]);
    ev.specials.push(water("W1", pt(5.0, 7.5), &[]));
    ev.specials.push(SpecialObject::OutOfBoundsArea {
        id: "OOB1".to_owned(),
        ppen_id: None,
        polygon: vec![pt(0.0, 1.0), pt(2.0, 3.0), pt(4.0, -5.0)],
        courses: vec![],
    });
    let out = sprint(&ev).event;
    assert_eq!(raw_control(&out, "31").position, Some(rp(12.345, -6.789)));
    assert_eq!(raw_control(&out, "F1").position, Some(rp(100.0, 0.0)));
    let legs = &raw_course(&out, "A").legs;
    assert_eq!(legs.len(), 2);
    assert_eq!(legs[0].bends, vec![rp(1.0, -2.0)]);
    assert_eq!(legs[1].bends, vec![rp(20.5, -30.25)]);
    assert_eq!(
        out.specials,
        vec![
            Special::Water {
                id: None,
                pt: rp(5.0, -7.5),
                courses: CourseFilter::All,
            },
            Special::OutOfBounds {
                id: None,
                rings: vec![vec![rp(0.0, -1.0), rp(2.0, -3.0), rp(4.0, 5.0)]],
                courses: CourseFilter::All,
            },
        ]
    );
}

#[test]
fn north_number_has_positive_file_dy() {
    let at = |direction: f64| {
        let mut ev = event(&[("31", 10.0, 10.0)], vec![course("A", &["31"])]);
        ev.controls[0].number = Some(NumberPlacement {
            direction,
            gap_mm: 1.0,
        });
        let out = sprint(&ev).event;
        let stops = &raw_course(&out, "A").stops;
        assert_eq!(stops[0].number_text_at, None, "the start has no number");
        stops[1].number_text_at.expect("a number offset")
    };
    let north = at(90.0);
    assert!(north.dy_mm > 0.0, "{north:?}");
    assert!(north.dx_mm.abs() < 1e-9, "{north:?}");
    let east = at(0.0);
    assert!(east.dx_mm > 0.0, "{east:?}");
    assert!(east.dy_mm.abs() < 1e-9, "{east:?}");
}

#[test]
fn number_round_trips_through_the_import() {
    // (scaling, print scale, the ratio Purple Pen gets for a 1:10000 map)
    let cases = [
        (ItemScaling::None, Some(7500), 0.75),
        (ItemScaling::None, Some(15000), 1.5),
        (ItemScaling::RelativeTo15000, None, 1.5),
    ];
    let codes: Vec<String> = (31..41).map(|c| c.to_string()).collect();
    let refs: Vec<&str> = codes.iter().map(String::as_str).collect();
    let controls: Vec<(&str, f64, f64)> = refs
        .iter()
        .enumerate()
        .map(|(i, c)| (*c, 10.0 * i as f64, 5.0))
        .collect();
    for (scaling, print, ratio) in cases {
        for direction in [0.0, 37.5, 90.0, 200.0, 321.73] {
            for gap_mm in [0.3, 1.0, 2.5] {
                let placement = NumberPlacement { direction, gap_mm };
                let mut a = course("A", &refs);
                a.planned_print_scale = print;
                // Number 2 from the control, number 10 (two digits) from the course control.
                individual(&mut a)[9].number = Some(placement.clone());
                let mut ev = event(&controls, vec![a]);
                ev.controls[1].number = Some(placement.clone());
                let out = to_raw(&ev, MapKind::Forest, 10000, scaling, false).event;
                let c = raw_course(&out, "A");
                assert_eq!(c.course_obj_ratio, Some(ratio));
                for (stop, n) in [(2, 2), (10, 10)] {
                    let o = c.stops[stop].number_text_at.expect("an offset");
                    let back = placement_from_offset(
                        Offset {
                            dx_mm: o.dx_mm,
                            dy_mm: -o.dy_mm,
                        },
                        circle_outer_radius_mm(MapKind::Forest),
                        number_box_mm(MapKind::Forest, n),
                        ratio,
                    );
                    let turn = (back.direction - direction).rem_euclid(360.0);
                    assert!(
                        turn.min(360.0 - turn) < 0.01 && (back.gap_mm - gap_mm).abs() < 0.01,
                        "{placement:?} at ratio {ratio}, number {n}: {back:?}"
                    );
                }
                // Stops without a placement leave the number to Purple Pen.
                assert_eq!(c.stops[1].number_text_at, None);
            }
        }
    }
}

#[test]
fn course_placement_wins_over_the_central_one() {
    let mut a = course("A", &["31"]);
    individual(&mut a)[0].number = Some(NumberPlacement {
        direction: 0.0,
        gap_mm: 1.0,
    });
    let mut ev = event(&[("31", 10.0, 10.0)], vec![a, course("B", &["31"])]);
    ev.controls[0].number = Some(NumberPlacement {
        direction: 180.0,
        gap_mm: 1.0,
    });
    let out = sprint(&ev).event;
    let dx = |name: &str| {
        raw_course(&out, name).stops[1]
            .number_text_at
            .unwrap()
            .dx_mm
    };
    assert!(dx("A") > 0.0);
    assert!(dx("B") < 0.0);
}

#[test]
fn team_relay_score_are_skipped_and_named() {
    let mut team = course("T", &[]);
    team.kind = CourseKind::Team {
        team_size: 2,
        mandatory: vec![],
        free: vec![code("31")],
    };
    let mut relay = course("R", &["31"]);
    relay.kind = CourseKind::Relay {
        legs: 3,
        parts: vec![],
    };
    let mut score = course("S", &[]);
    score.kind = CourseKind::Score {
        time_limit_min: 60,
        penalty_per_min: Some(10),
        controls: vec![ScoreControl {
            code: code("31"),
            value: 10,
        }],
        links: vec![],
    };
    let mut ev = event(
        &[("31", 10.0, 10.0)],
        vec![course("A", &["31"]), team, relay, score],
    );
    ev.specials.push(water("W1", pt(5.0, 5.0), &["T", "S"]));
    ev.specials.push(water("W2", pt(6.0, 6.0), &["S", "A"]));
    let got = sprint(&ev);
    let names: Vec<&str> = got.event.courses.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["A"]);
    let skipped = |course: &str, kind| ExportNote::CourseSkipped {
        course: course.to_owned(),
        kind,
    };
    assert_eq!(
        got.notes,
        vec![
            skipped("T", "team"),
            skipped("R", "relay"),
            skipped("S", "score"),
            ExportNote::SpecialSkipped {
                id: "W1".to_owned(),
                why: "no course left",
            },
        ]
    );
    // W2 keeps the written course only.
    assert_eq!(
        got.event.specials,
        vec![Special::Water {
            id: None,
            pt: rp(6.0, -6.0),
            courses: CourseFilter::Only(vec!["A".to_owned()]),
        }]
    );
}

#[test]
fn marked_route_special_is_skipped() {
    let mut ev = event(&[], vec![]);
    ev.specials.push(SpecialObject::MarkedRoute {
        id: "ROUTE1".to_owned(),
        ppen_id: Some(4),
        points: vec![pt(0.0, 0.0), pt(10.0, 0.0)],
        courses: vec![],
    });
    let got = sprint(&ev);
    assert_eq!(got.event.specials, vec![]);
    assert!(got.notes.contains(&ExportNote::SpecialSkipped {
        id: "ROUTE1".to_owned(),
        why: "marked route",
    }));
}

#[test]
fn two_point_forbidden_route_is_skipped() {
    let forbidden = |id: &str, points: Vec<MapPoint>| SpecialObject::ForbiddenRoute {
        id: id.to_owned(),
        ppen_id: None,
        points,
        courses: vec![],
    };
    let mut ev = event(&[], vec![]);
    ev.specials
        .push(forbidden("FORBID1", vec![pt(0.0, 0.0), pt(10.0, 0.0)]));
    ev.specials.push(forbidden("FORBID2", vec![pt(3.0, 4.0)]));
    let got = sprint(&ev);
    assert_eq!(
        got.event.specials,
        vec![Special::ForbiddenRoute {
            id: None,
            path: vec![rp(3.0, -4.0)],
            courses: CourseFilter::All,
        }]
    );
    assert!(got.notes.contains(&ExportNote::SpecialSkipped {
        id: "FORBID1".to_owned(),
        why: "forbidden route with several points",
    }));
}

/// An event in which every object carries a Purple Pen id.
fn with_ids() -> Event {
    let mut a = course("A", &["31"]);
    a.ppen_id = Some(7);
    let mut ev = event(&[("31", 10.0, 10.0)], vec![a]);
    ev.controls[0].ppen_id = Some(3);
    for (s, id) in ev.specials.iter_mut().zip([1, 2]) {
        if let SpecialObject::Start { ppen_id, .. } | SpecialObject::Finish { ppen_id, .. } = s {
            *ppen_id = Some(id);
        }
    }
    ev.specials.push(SpecialObject::Water {
        id: "W1".to_owned(),
        ppen_id: Some(9),
        map_mm: pt(5.0, 5.0),
        courses: vec![],
    });
    ev
}

fn object_ids(ev: &RawEvent) -> Vec<Option<u32>> {
    ev.controls
        .iter()
        .map(|c| c.id)
        .chain(ev.courses.iter().map(|c| c.id))
        .chain(ev.specials.iter().map(Special::id))
        .collect()
}

#[test]
fn ids_only_when_asked() {
    let ev = with_ids();
    let without = to_raw(&ev, MapKind::Sprint, 4000, ItemScaling::None, false).event;
    assert_eq!(object_ids(&without), vec![None; 5]);
    let with = to_raw(&ev, MapKind::Sprint, 4000, ItemScaling::None, true).event;
    // Starts and finishes come first, then the controls.
    assert_eq!(
        object_ids(&with),
        vec![Some(1), Some(2), Some(3), Some(7), Some(9)]
    );
}

#[test]
fn starts_finishes_and_crossing_points() {
    let mut ev = event(
        &[
            ("31", 10.0, 10.0),
            ("CROSS1", 20.0, 20.0),
            ("X7", 30.0, 30.0),
        ],
        vec![course("A", &["31", "CROSS1", "X7"])],
    );
    for c in &mut ev.controls[1..] {
        c.kind = ControlKind::CrossingPoint;
        c.description.c = Some("15.3".to_owned());
    }
    ev.controls[0].number = Some(NumberPlacement {
        direction: 0.0,
        gap_mm: 1.0,
    });
    ev.controls[1].number = Some(NumberPlacement {
        direction: 0.0,
        gap_mm: 1.0,
    });
    ev.specials[0] = SpecialObject::Start {
        id: "STA1".to_owned(),
        ppen_id: None,
        map_mm: pt(0.0, 0.0),
    };
    ev.courses[0].start = "STA1".to_owned();
    let got = sprint(&ev);
    let out = &got.event;
    let kinds: Vec<(&str, PKind)> = out
        .controls
        .iter()
        .map(|c| (c.code.as_str(), c.kind))
        .collect();
    assert_eq!(
        kinds,
        [
            ("STA1", PKind::Start),
            ("F1", PKind::Finish),
            ("31", PKind::Control),
            ("CROSS1", PKind::CrossingPoint),
            ("X7", PKind::CrossingPoint),
        ]
    );
    // A crossing point keeps its instruction row and gets no number, even with a placement.
    let cross = raw_control(out, "CROSS1");
    assert_eq!(cross.description.as_ref().unwrap().columns[0], some("13.3"));
    let stops = &raw_course(out, "A").stops;
    let codes: Vec<&str> = stops.iter().map(|s| s.control.as_str()).collect();
    assert_eq!(codes, ["STA1", "31", "CROSS1", "X7", "F1"]);
    assert!(stops[1].number_text_at.is_some());
    assert_eq!(stops[2].number_text_at, None);
    // A re-import names starts, finishes and crossing points STA1, FIN1, CROSS1 … in file order. That
    // is expected and not noted.
    assert_eq!(got.notes, vec![]);
}

#[test]
fn code_not_stored_only_for_a_normal_control() {
    // The reader trims a code, so a normal control whose code has blanks at an end comes back under
    // another one. Starts, finishes and crossing points have no stored code and get no note.
    let mut ev = event(
        &[("31", 10.0, 10.0), (" 33", 20.0, 20.0), ("CP", 30.0, 30.0)],
        vec![course("A", &["31", " 33", "CP"])],
    );
    ev.controls[2].kind = ControlKind::CrossingPoint;
    let got = sprint(&ev);
    assert_eq!(
        got.notes,
        vec![ExportNote::CodeNotStored {
            code: " 33".to_owned()
        }]
    );
}

#[test]
fn course_fields() {
    let mut a = course("A", &["31", "32"]);
    a.start_taped = true;
    individual(&mut a)[1].taped = true;
    a.climb_m = Some(35);
    let mut b = course("B", &["31"]);
    b.planned_print_scale = Some(5000);
    let ev = event(&[("31", 10.0, 10.0), ("32", 20.0, 10.0)], vec![a, b]);
    let out = sprint(&ev).event;
    let a = raw_course(&out, "A");
    let taped: Vec<bool> = a.legs.iter().map(|l| l.marked_route).collect();
    assert_eq!(taped, [true, false, true]);
    assert_eq!(a.climb_m, Some(35.0));
    assert_eq!(a.kind, freemapper_course::CourseKind::Normal);
    // Without a planned print scale the course prints at the map scale.
    assert_eq!(a.print_scale_hint, Some(4000));
    assert_eq!(a.course_obj_ratio, Some(1.0));
    let b = raw_course(&out, "B");
    assert_eq!(b.print_scale_hint, Some(5000));
    assert_eq!(b.course_obj_ratio, Some(1.25));
    assert_eq!(out.source.map_scale, Some(4000));
    assert_eq!(out.source.item_scaling, Some(ItemScaling::None));
}

#[test]
fn descriptions_go_through_ppen_columns() {
    let mut ev = event(&[("31", 10.0, 10.0), ("32", 20.0, 10.0)], vec![]);
    ev.controls[0].description = ControlDescription {
        d: some("5.2"),
        e: some("5.2"),
        f: some("10.1"),
        h: some("13.9"),
        ..Default::default()
    };
    let got = sprint(&ev);
    let d = raw_control(&got.event, "31").description.clone().unwrap();
    assert_eq!(
        d.columns,
        [None, some("5.2"), None, some("10.1single"), None, None]
    );
    assert_eq!(d.standard, DESCRIPTION_STANDARD);
    assert_eq!(d.column_f_text, None);
    assert_eq!(raw_control(&got.event, "32").description, None);
    assert!(got.notes.contains(&ExportNote::SymbolWithoutPpenId {
        control: "31".to_owned(),
        column: 'H',
        id: "13.9".to_owned(),
    }));
}

#[test]
fn crossing_point_keeps_column_c_only() {
    let mut ev = event(&[("CROSS1", 10.0, 10.0)], vec![]);
    ev.controls[0].kind = ControlKind::CrossingPoint;
    ev.controls[0].description = ControlDescription {
        c: some("15.3"),
        d: some("5.2"),
        ..Default::default()
    };
    let got = sprint(&ev);
    let d = raw_control(&got.event, "CROSS1")
        .description
        .clone()
        .unwrap();
    assert_eq!(d.columns, [some("13.3"), None, None, None, None, None]);
    assert!(got.notes.contains(&ExportNote::SymbolWithoutPpenId {
        control: "CROSS1".to_owned(),
        column: 'D',
        id: "5.2".to_owned(),
    }));
}

// ---- round trip through the import ----

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../tests/fixtures/ppen/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Every point of a raw event, in map mm with y down.
fn map_points(raw: &RawEvent) -> Vec<(f64, f64)> {
    let mut out: Vec<RawPoint> = raw.controls.iter().filter_map(|c| c.position).collect();
    for c in &raw.courses {
        out.extend(c.legs.iter().flat_map(|l| l.bends.iter().copied()));
    }
    for s in &raw.specials {
        match s {
            Special::OutOfBounds { rings, .. }
            | Special::Dangerous { rings, .. }
            | Special::WhiteOut { rings, .. } => out.extend(rings.iter().flatten().copied()),
            Special::ForbiddenRoute { path, .. }
            | Special::Boundary { path, .. }
            | Special::MarkedRoute { path, .. }
            | Special::Line { path, .. } => out.extend(path.iter().copied()),
            Special::FirstAid { pt, .. } | Special::Water { pt, .. } | Special::Text { pt, .. } => {
                out.push(*pt)
            }
        }
    }
    out.into_iter().map(|p| (p.x_mm, -p.y_mm)).collect()
}

/// A map at the file's scale with a point at two opposite corners of every point of the file.
fn frame_map(raw: &RawEvent) -> MapDocument {
    let points = map_points(raw);
    let fold = |f: fn(f64, f64) -> f64, start: f64, pick: fn(&(f64, f64)) -> f64| {
        points.iter().map(pick).fold(start, f)
    };
    let (x0, y0) = (
        fold(f64::min, f64::INFINITY, |p| p.0).floor(),
        fold(f64::min, f64::INFINITY, |p| p.1).floor(),
    );
    let (x1, y1) = (
        fold(f64::max, f64::NEG_INFINITY, |p| p.0).ceil(),
        fold(f64::max, f64::NEG_INFINITY, |p| p.1).ceil(),
    );
    let corners = vec![point_object(0, x0, y0), point_object(0, x1, y1)];
    let scale = raw.source.map_scale.expect("the fixture states its scale");
    if scale > 5000 {
        crate::check::testing::forest_map(scale, corners)
    } else {
        map(scale, corners)
    }
}

/// The fit rounds every point to the map's µm.
const POINT_TOLERANCE_MM: f64 = 0.001;
/// The import rounds a placement to 0.01° and 0.01 mm at the nominal size; at a ratio up to 1.5 that
/// moves the box centre by less than this.
const NUMBER_TOLERANCE_MM: f64 = 0.02;

fn near(a: RawPoint, b: RawPoint, tolerance: f64, what: &str) {
    assert!(
        (a.x_mm - b.x_mm).abs() <= tolerance && (a.y_mm - b.y_mm).abs() <= tolerance,
        "{what}: {a:?} against {b:?}"
    );
}

fn near_path(a: &[RawPoint], b: &[RawPoint], what: &str) {
    assert_eq!(a.len(), b.len(), "{what}: {a:?} against {b:?}");
    for (p, q) in a.iter().zip(b) {
        near(*p, *q, POINT_TOLERANCE_MM, what);
    }
}

/// The bends of a leg and whether it is a marked route.
type ExpectedLeg = (Vec<RawPoint>, bool);

/// A course of the original file as the model holds it: map exchange controls become bends of the leg
/// through them (the import's rule), so the stops and legs are those an export can give back.
fn expected_course(
    raw: &RawEvent,
    c: &freemapper_course::Course<RawPoint>,
) -> (Vec<String>, Vec<ExpectedLeg>, Vec<Option<Offset>>) {
    let control = |code: &str| raw.controls.iter().find(|x| x.code == code).unwrap();
    let mut stops = Vec::new();
    let mut legs: Vec<ExpectedLeg> = Vec::new();
    let mut numbers = Vec::new();
    for (i, s) in c.stops.iter().enumerate() {
        let leg = c.legs.get(i);
        let bends = leg.map(|l| l.bends.clone()).unwrap_or_default();
        let marked = leg.is_some_and(|l| l.marked_route);
        let x = control(&s.control);
        if x.kind == PKind::MapExchange {
            let last = legs.last_mut().unwrap();
            last.0.push(x.position.unwrap());
            last.0.extend(bends);
            last.1 &= marked;
            continue;
        }
        stops.push(s.control.clone());
        numbers.push(if x.kind == PKind::Control {
            s.number_text_at
        } else {
            None
        });
        if i + 1 < c.stops.len() {
            legs.push((bends, marked));
        }
    }
    (stops, legs, numbers)
}

/// Imports a fixture, maps the project back with ids and compares with what the reader gave. Allowed
/// differences, each from a rule of the import or of the model:
/// - Map exchange controls, controls without a position, score courses and courses the import skips, and
///   special objects of kinds the model lacks (boundary, text, line, white-out) are not in the project,
///   so not in the export; every other object is matched by its Purple Pen id.
/// - Order: starts and finishes come first, then the controls, in project order.
/// - Descriptions: start and finish descriptions, column F text and symbol ids without a 2024 number are
///   not in the model; the standard is [`DESCRIPTION_STANDARD`]. Only the columns are compared, for
///   controls and crossing points.
/// - Courses: circle gaps, leg gaps, secondary title and course length are not in the model; a map
///   exchange control is a bend of the leg through it; climb is whole metres; a course without a print
///   scale gets the map scale, with the ratio for it. Number offsets come back within
///   [`NUMBER_TOLERANCE_MM`] where the file gives a print scale.
/// - Points within [`POINT_TOLERANCE_MM`].
/// - A special object with several rings is several objects in the project, of which only the first keeps
///   the id; the others are left out of the comparison.
fn round_trip(name: &str) {
    let bytes = fixture(name);
    let raw = read(&bytes).unwrap();
    let map = frame_map(&raw);
    let import = import_ppen(&bytes, &map).unwrap_or_else(|e| panic!("{name}: {e}"));
    let mut ev = event(&[], vec![]);
    ev.controls = import.controls;
    ev.specials = import.specials;
    ev.courses = import.courses;
    let scale = raw.source.map_scale.unwrap();
    let scaling = raw.source.item_scaling.expect("a .ppen states its scaling");
    let got = to_raw(&ev, map_kind(&map), scale, scaling, true).event;
    assert_eq!(got.source.map_scale, raw.source.map_scale);
    assert_eq!(got.source.item_scaling, raw.source.item_scaling);

    let by_id = |id: Option<u32>| {
        raw.controls
            .iter()
            .find(|c| c.id.is_some() && c.id == id)
            .unwrap_or_else(|| panic!("{name}: no control with id {id:?}"))
    };
    let kept_controls = raw
        .controls
        .iter()
        .filter(|c| c.kind != PKind::MapExchange && c.position.is_some())
        .count();
    assert_eq!(got.controls.len(), kept_controls, "{name}");
    for c in &got.controls {
        let o = by_id(c.id);
        let what = format!("{name}: control {}", o.code);
        assert_eq!((&c.code, c.kind), (&o.code, o.kind), "{what}");
        near(
            c.position.unwrap(),
            o.position.unwrap(),
            POINT_TOLERANCE_MM,
            &what,
        );
        if matches!(o.kind, PKind::Start | PKind::Finish) {
            assert_eq!(c.description, None, "{what}");
            continue;
        }
        let want = o.description.as_ref().map(|d| {
            d.columns
                .iter()
                .map(|id| id.clone().filter(|id| iof_2024(id).is_some()))
                .collect::<Vec<_>>()
        });
        let want = want.filter(|cols| cols.iter().any(Option::is_some));
        // The same feature in D and E over a combination in F is written as the variant `single` (the
        // import reads both forms alike, so the export picks the one Purple Pen offers).
        let want = want.map(|mut cols| {
            if cols[1].is_some()
                && cols[1] == cols[2]
                && matches!(cols[3].as_deref(), Some("10.1" | "10.2"))
            {
                cols[2] = None;
                cols[3] = cols[3].take().map(|f| format!("{f}single"));
            }
            cols
        });
        let have = c.description.as_ref().map(|d| d.columns.to_vec());
        assert_eq!(have, want, "{what}");
    }

    for c in &got.courses {
        let o = raw
            .courses
            .iter()
            .find(|x| x.id.is_some() && x.id == c.id)
            .unwrap_or_else(|| panic!("{name}: no course {}", c.name));
        let what = format!("{name}: course {}", o.name);
        assert_eq!((&c.name, c.kind), (&o.name, o.kind), "{what}");
        let (stops, legs, numbers) = expected_course(&raw, o);
        let have: Vec<&String> = c.stops.iter().map(|s| &s.control).collect();
        assert_eq!(have, stops.iter().collect::<Vec<_>>(), "{what}");
        assert_eq!(c.legs.len(), legs.len(), "{what}");
        for (l, (bends, marked)) in c.legs.iter().zip(&legs) {
            near_path(&l.bends, bends, &what);
            assert_eq!(l.marked_route, *marked, "{what}");
            assert!(l.gaps.is_empty());
        }
        assert_eq!(c.climb_m, o.climb_m.map(f64::round), "{what}");
        assert_eq!(
            c.print_scale_hint,
            Some(o.print_scale_hint.unwrap_or(scale)),
            "{what}"
        );
        if o.print_scale_hint.is_some() {
            assert_eq!(c.course_obj_ratio, o.course_obj_ratio, "{what}");
            for (s, want) in c.stops.iter().zip(&numbers) {
                match (s.number_text_at, want) {
                    (None, None) => {}
                    (Some(a), Some(b)) => near(
                        rp(a.dx_mm, a.dy_mm),
                        rp(b.dx_mm, b.dy_mm),
                        NUMBER_TOLERANCE_MM,
                        &format!("{what}, number of {}", s.control),
                    ),
                    other => panic!("{what}, number of {}: {other:?}", s.control),
                }
            }
        }
        assert!(c.stops.iter().all(|s| s.circle_gaps.is_empty()));
    }

    for s in got.specials.iter().filter(|s| s.id().is_some()) {
        let o = raw
            .specials
            .iter()
            .find(|x| x.id() == s.id())
            .unwrap_or_else(|| panic!("{name}: no special {:?}", s.id()));
        let what = format!("{name}: special {:?}", s.id());
        assert_eq!(s.courses(), o.courses(), "{what}");
        match (s, o) {
            (Special::OutOfBounds { rings: a, .. }, Special::OutOfBounds { rings: b, .. })
            | (Special::Dangerous { rings: a, .. }, Special::Dangerous { rings: b, .. }) => {
                near_path(&a[0], &b[0], &what)
            }
            (Special::ForbiddenRoute { path: a, .. }, Special::ForbiddenRoute { path: b, .. }) => {
                near_path(a, b, &what)
            }
            (Special::FirstAid { pt: a, .. }, Special::FirstAid { pt: b, .. })
            | (Special::Water { pt: a, .. }, Special::Water { pt: b, .. }) => {
                near(*a, *b, POINT_TOLERANCE_MM, &what)
            }
            other => panic!("{what}: {other:?}"),
        }
    }
}

#[test]
fn round_trip_of_the_fixtures() {
    for name in ["lincoln-park.ppen", "overlapper.ppen", "sampleevent8.ppen"] {
        round_trip(name);
    }
}

#[test]
fn round_trip_keeps_every_course_and_special_the_model_holds() {
    // overlapper: two normal courses (the score course is not imported), a water and two forbidden
    // routes of one cross each.
    let bytes = fixture("overlapper.ppen");
    let raw = read(&bytes).unwrap();
    let map = frame_map(&raw);
    let import = import_ppen(&bytes, &map).unwrap();
    let mut ev = event(&[], vec![]);
    ev.controls = import.controls;
    ev.specials = import.specials;
    ev.courses = import.courses;
    let got = to_raw(&ev, map_kind(&map), 10000, ItemScaling::None, true);
    assert_eq!(got.event.courses.len(), 2);
    assert_eq!(got.event.specials.len(), 3);
}

#[test]
fn the_writer_accepts_the_mapped_fixtures() {
    use freemapper_course::{
        DescriptionStandard, LineEnding, MapStandard, PpenMap, ScratchOptions, write_ppen,
    };
    for name in ["lincoln-park.ppen", "overlapper.ppen", "sampleevent8.ppen"] {
        let bytes = fixture(name);
        let raw = read(&bytes).unwrap();
        let map = frame_map(&raw);
        let import = import_ppen(&bytes, &map).unwrap();
        let mut ev = event(&[], vec![]);
        ev.controls = import.controls;
        ev.specials = import.specials;
        ev.courses = import.courses;
        let scale = raw.source.map_scale.unwrap();
        // From scratch the writer scales course symbols by item scaling None.
        let mapped = to_raw(&ev, map_kind(&map), scale, ItemScaling::None, false);
        let options = ScratchOptions {
            map: PpenMap::None,
            map_standard: MapStandard::Isom2017,
            description_standard: DescriptionStandard::Iof2018,
            description_lang: "en".to_owned(),
            line_ending: LineEnding::Crlf,
        };
        assert_eq!(DescriptionStandard::Iof2018.as_str(), DESCRIPTION_STANDARD);
        let written = write_ppen(&mapped.event, &options).unwrap_or_else(|e| panic!("{name}: {e}"));
        let back = read(written.text.as_bytes()).unwrap();
        assert_eq!(back.courses.len(), mapped.event.courses.len(), "{name}");
        assert_eq!(back.controls.len(), mapped.event.controls.len(), "{name}");
    }
}

#[test]
fn base_standard_comes_from_descriptions_then_the_event() {
    let sample = String::from_utf8(fixture("sampleevent8.ppen")).unwrap();
    let raw = read(sample.as_bytes()).unwrap();
    assert_eq!(base_standard(&raw, sample.as_bytes()), "2018");
    // Without a single description the event's standards element tells.
    let bare: String = sample
        .split_inclusive('\n')
        .filter(|l| !l.contains("<description box"))
        .collect();
    let raw = read(bare.as_bytes()).unwrap();
    assert!(raw.controls.iter().all(|c| c.description.is_none()));
    assert_eq!(base_standard(&raw, bare.as_bytes()), "2018");
    let single = bare.replace("description=\"2018\"", "description='2004'");
    assert_eq!(base_standard(&raw, single.as_bytes()), "2004");
    // Lincoln Park states none: 2004, the reader's default.
    let lincoln = fixture("lincoln-park.ppen");
    let raw = read(&lincoln).unwrap();
    assert_eq!(base_standard(&raw, &lincoln), "2004");
    let none = bare.replace("<standards map=\"2000\" description=\"2018\" />", "");
    assert_eq!(base_standard(&raw, none.as_bytes()), "2004");
}

#[test]
fn base_standard_reads_the_element_not_the_text() {
    let sample = String::from_utf8(fixture("sampleevent8.ppen")).unwrap();
    let bare: String = sample
        .split_inclusive('\n')
        .filter(|l| !l.contains("<description box"))
        .collect();
    let raw = read(bare.as_bytes()).unwrap();
    let real = "<standards map=\"2000\" description=\"2018\" />";
    assert!(bare.contains(real));
    // A comment ahead of the root names the element with a multibyte character after `description=`.
    for comment in [
        "<!-- <standards description=\u{e9}> -->",
        "<!-- <standards description=\u{e4}\"2004\"> -->",
        "<!-- <standards description=\"2004\"> -->",
    ] {
        let hostile = bare.replacen(
            "<course-scribe-event",
            &format!("{comment}\n<course-scribe-event"),
            1,
        );
        assert_ne!(hostile, bare);
        assert_eq!(base_standard(&raw, hostile.as_bytes()), "2018", "{comment}");
    }
    // Blanks around `=` and a line break before the attribute are XML too.
    for spelled in [
        "<standards map=\"2000\" description = \"2004\" />",
        "<standards map=\"2000\"\r\n    description=\"2004\" />",
    ] {
        let text = bare.replace(real, spelled);
        assert_eq!(base_standard(&raw, text.as_bytes()), "2004", "{spelled}");
    }
}
