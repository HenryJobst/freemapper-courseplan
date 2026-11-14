//! `check` on the synthetic ISOM 2017-2 forest map (`tests/fixtures/maps/isom2017-synthetic.omap`, ADR 0092).
//! The map is 1:10000, so 1 map mm is 10 m on the ground.

use freemapper_courseplan_model::{Discipline, SpecialObject};

use super::testing::*;
use super::*;
use crate::map::MapKind;
use crate::omap::read::read_map_file;
use crate::rules::load_rule_set;

fn synthetic() -> MapDocument {
    let path = format!(
        "{}/../../tests/fixtures/maps/isom2017-synthetic.omap",
        env!("CARGO_MANIFEST_DIR")
    );
    read_map_file(std::path::Path::new(&path)).expect("the synthetic map reads")
}

/// An event on the synthetic map: start at (1, 7), finish at (11, 7), one course A over `controls`.
fn forest_event(controls: &[(&str, f64, f64)]) -> Event {
    let codes: Vec<&str> = controls.iter().map(|c| c.0).collect();
    let mut e = event(controls, vec![course("A", &codes)]);
    e.specials = vec![
        SpecialObject::Start {
            id: "S1".to_owned(),
            ppen_id: None,
            map_mm: pt(1.0, 7.0),
        },
        SpecialObject::Finish {
            id: "F1".to_owned(),
            ppen_id: None,
            map_mm: pt(11.0, 7.0),
        },
    ];
    e
}

fn run_forest(event: &Event) -> CheckReport {
    run_as(event, &synthetic(), MapKind::Forest)
}

fn run_as(event: &Event, map: &MapDocument, kind: MapKind) -> CheckReport {
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    check_event_as(event, map, kind, &rules)
}

fn run_forest_with_nor_overlay(event: &Event) -> CheckReport {
    let rules = load_rule_set("wkb-2026", &["nor-bb-2021".to_owned()]).unwrap();
    check_event_as(event, &synthetic(), MapKind::Forest, &rules)
}

fn of_rule<'a>(report: &'a CheckReport, rule: &str) -> Vec<&'a Finding> {
    report.findings.iter().filter(|f| f.rule == rule).collect()
}

fn skip_of_rule<'a>(report: &'a CheckReport, rule: &str) -> Vec<&'a Skipped> {
    report
        .skipped
        .iter()
        .filter(|s| s.rule.as_deref() == Some(rule))
        .collect()
}

fn arg<'a>(finding: &'a Finding, name: &str) -> &'a ArgValue {
    &finding.args.iter().find(|(n, _)| *n == name).unwrap().1
}

#[test]
fn forest_min_distance_30m_fires() {
    // 2.5 mm are 25 m, 4 mm are 40 m.
    let e = forest_event(&[("31", 4.0, 9.5), ("32", 6.5, 9.5), ("33", 10.5, 9.5)]);
    let r = run_forest(&e);
    let found = of_rule(&r, "wkb2026.distance.min");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(
        found[0].location,
        Location::Controls {
            codes: vec![code("31"), code("32")]
        }
    );
    assert_eq!(found[0].severity, Severity::Error);
}

#[test]
fn forest_similar_features_60m_uses_isom_2017_groups() {
    // 31 stands on the fence (516), 32 on the ruined fence (517): one group in ISOM 2017-2, 31.6 m apart.
    let e = forest_event(&[("31", 3.0, 10.0), ("32", 6.0, 11.0)]);
    let r = run_forest(&e);
    let found = of_rule(&r, "wkb2026.distance.similar");
    assert_eq!(found.len(), 1, "{found:?}\n{:?}", r.findings);
    // 31 on the fence and 33 on the footpath (505) are in different groups.
    let e = forest_event(&[("31", 3.0, 10.0), ("33", 6.0, 8.4)]);
    let r = run_forest(&e);
    assert!(
        of_rule(&r, "wkb2026.distance.similar").is_empty(),
        "{:?}",
        r.findings
    );
    // The sprint variants of the rule do not run on a forest map.
    assert!(of_rule(&r, "wkb2026.distance.similar_sprint").is_empty());
}

#[test]
fn forest_control_in_out_of_bounds_fires() {
    let e = forest_event(&[("31", 9.5, 5.0)]);
    let r = run_forest(&e);
    let found = of_rule(&r, "wkb2026.control.in_forbidden");
    assert_eq!(found.len(), 1, "{:?}", r.findings);
    assert_eq!(found[0].severity, Severity::Error);
    // A control beside the area does not.
    let e = forest_event(&[("31", 7.0, 5.0)]);
    assert!(of_rule(&run_forest(&e), "wkb2026.control.in_forbidden").is_empty());
}

#[test]
fn forest_hidden_709_area_is_no_map_content() {
    // The 709 area of the synthetic map is hidden in Mapper's symbol set (a course-setting symbol), so the
    // index leaves it out and a control in it is not reported (ADR 0092).
    let e = forest_event(&[("31", 32.0, 32.0)]);
    let r = run_forest(&e);
    assert!(
        of_rule(&r, "wkb2026.control.in_forbidden").is_empty(),
        "{:?}",
        r.findings
    );
}

#[test]
fn forest_control_outside_mapped_area_fires() {
    // (35, 5) is 22 mm from the nearest object: far beyond the limit of 100 m on this map.
    let e = forest_event(&[("31", 35.0, 5.0), ("32", 8.0, 9.5)]);
    let r = run_forest(&e);
    let found = of_rule(&r, "wkb2026.map.inside");
    assert_eq!(found.len(), 1, "{:?}", r.findings);
    assert_eq!(found[0].location, Location::Control { code: code("31") });
    assert_eq!(arg(found[0], "limit"), &ArgValue::Number(100.0));
}

#[test]
fn forest_edge_limit_derives_from_scale() {
    for (scale, limit) in [(5000, 50.0), (7500, 75.0), (10000, 100.0), (15000, 150.0)] {
        let mut map = synthetic();
        map.georef.scale_denominator = scale;
        let e = forest_event(&[("31", 35.0, 5.0)]);
        let r = run_as(&e, &map, MapKind::Forest);
        let found = of_rule(&r, "wkb2026.map.inside");
        assert_eq!(found.len(), 1, "1:{scale}: {:?}", r.findings);
        assert_eq!(
            arg(found[0], "limit"),
            &ArgValue::Number(limit),
            "1:{scale}"
        );
    }
}

#[test]
fn sprint_edge_limits_unchanged() {
    // The same geometry read as a sprint map keeps the 50 m of sprint: (17, 2) is 61 m from the nearest
    // object, which a forest map accepts (limit 100 m) and a sprint map does not.
    let e = forest_event(&[("31", 17.0, 2.0)]);
    let forest = run_as(&e, &synthetic(), MapKind::Forest);
    assert!(
        of_rule(&forest, "wkb2026.map.inside").is_empty(),
        "{:?}",
        forest.findings
    );
    let sprint = run_as(&e, &synthetic(), MapKind::Sprint);
    let found = of_rule(&sprint, "wkb2026.map.inside");
    assert_eq!(found.len(), 1, "{:?}", sprint.findings);
    assert_eq!(arg(found[0], "limit"), &ArgValue::Number(50.0));
}

#[test]
fn forest_sprint_only_rules_do_not_apply() {
    let e = forest_event(&[("31", 4.0, 9.5), ("32", 10.5, 9.5)]);
    let r = run_forest(&e);
    for rule in [
        "wkb2026.control.dead_end",
        "wkb2026.course.start_finish_distance",
    ] {
        assert!(of_rule(&r, rule).is_empty(), "{rule}");
        let skips = skip_of_rule(&r, rule);
        assert_eq!(skips.len(), 1, "{rule}: {:?}", r.skipped);
        assert_eq!(skips[0].reason_id, SKIP_MAP_KIND);
        assert_eq!(
            skips[0].args,
            vec![("kind", ArgValue::Text("forest".into()))]
        );
    }
    // The text says so in both languages.
    let skip = skip_of_rule(&r, "wkb2026.control.dead_end")[0];
    assert!(skip.text(&lang_of("en")).contains("forest maps"));
    assert!(skip.text(&lang_of("de")).contains("Waldkarten"));
}

#[test]
fn forest_elevation_rules_are_not_checked() {
    // The Lkm rule of the overlay needs the climb of the course, which the program cannot work out without an
    // elevation model; the rule about running through out-of-bounds areas needs the route analysis. Both
    // are listed as not checked, each with its reason.
    let mut e = forest_event(&[("31", 4.0, 9.5), ("32", 10.5, 9.5)]);
    e.event.discipline = Discipline::Long;
    e.classes[0].rule_category = Some("H35".to_owned());
    e.courses[0].classes = vec!["H21".to_owned()];
    let r = run_forest_with_nor_overlay(&e);
    let lkm = skip_of_rule(&r, "norbb2021.lkm.long");
    assert_eq!(lkm.len(), 1, "{:?}", r.skipped);
    assert_eq!(lkm[0].reason_id, SKIP_CLIMB_MISSING);
    let through = skip_of_rule(&r, "wkb2026.leg.through_forbidden");
    assert!(!through.is_empty(), "{:?}", r.skipped);
    assert!(
        through
            .iter()
            .all(|s| s.reason_id == SKIP_NEEDS_ROUTE_ANALYSIS)
    );
    assert!(of_rule(&r, "wkb2026.leg.through_forbidden").is_empty());
    // The sprint variant does not apply on a forest map.
    assert!(
        skip_of_rule(&r, "wkb2026.leg.through_forbidden_sprint")
            .iter()
            .all(|s| s.reason_id == SKIP_MAP_KIND)
    );
}

#[test]
fn forest_map_margin_measures_from_the_forest_edge() {
    // (17, 2) is 61 m from the nearest object. The forest edge lies 100 m from the objects, so the point is
    // mapped and about 39 m inside the edge: closer than the 50 m margin.
    let e = forest_event(&[("31", 17.0, 2.0)]);
    let forest = run_forest(&e);
    let found = of_rule(&forest, "wkb2026.map.margin");
    assert_eq!(found.len(), 1, "{:?}", forest.findings);
    let metres = match arg(found[0], "metres") {
        ArgValue::Int(m) => *m,
        other => panic!("{other:?}"),
    };
    assert!((36..=40).contains(&metres), "{metres}");
    // Read as a sprint map the point lies outside the 50 m reach: map.inside speaks, map.margin does not.
    let sprint = run_as(&e, &synthetic(), MapKind::Sprint);
    assert!(of_rule(&sprint, "wkb2026.map.margin").is_empty());
}
