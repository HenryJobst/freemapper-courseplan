use freemapper_courseplan_model::{AcceptedFinding, CourseKind, Discipline, LegRef};

use super::testing::*;
use super::*;
use crate::map::MapKind;
use crate::rules::KNOWN_KEYS;

/// A fake check: one warning per course it sees, at the course's first two controls.
fn first_two(
    _: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    _: &mut Vec<Skipped>,
) {
    for course in &run.courses {
        let codes = course.controls.iter().take(2).map(|c| c.code.clone());
        findings.push(run.finding(
            Location::Controls {
                codes: codes.collect(),
            },
            "check-location-event",
            vec![("course", ArgValue::Text(course.course.name.clone()))],
        ));
    }
}

const FAKE: &[Check] = &[Check::new("min_distance", first_two)];

fn sample_event() -> Event {
    event(
        &[("31", 10.0, 10.0), ("32", 20.0, 10.0), ("33", 30.0, 10.0)],
        vec![course("A", &["31", "32", "33"]), course("B", &["31", "33"])],
    )
}

fn run(event: &Event, rules: &RuleSet) -> CheckReport {
    let map = map(4000, vec![]);
    run_checks(&CheckContext::new(event, &map), rules, FAKE)
}

fn fake_rule() -> Rule {
    rule("t.fake.pair", "min_distance", Severity::Warning)
}

fn codes_of(location: &Location) -> Vec<&str> {
    match location {
        Location::Controls { codes } => codes.iter().map(|c| c.0.as_str()).collect(),
        other => panic!("not controls: {other:?}"),
    }
}

fn accepted(rule: &str, controls: &[&str], note: &str) -> AcceptedFinding {
    AcceptedFinding {
        rule: rule.to_owned(),
        controls: controls.iter().map(|c| code(c)).collect(),
        course: None,
        legs: vec![],
        note: note.to_owned(),
    }
}

#[test]
fn engine_runs_a_fake_rule_kind() {
    let mut overlay_rule = rule("o.fake.pair", "min_distance", Severity::Error);
    overlay_rule.source.section = "9".to_owned();
    let rules = rule_set(vec![(fake_rule(), "base"), (overlay_rule, "overlay")]);
    let report = run(&sample_event(), &rules);
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);
    assert_eq!(report.findings.len(), 4);
    let first = &report.findings[0];
    assert_eq!(first.rule, "t.fake.pair");
    assert_eq!(first.profile, "base");
    assert_eq!(first.severity, Severity::Warning);
    assert_eq!(first.source.doc, "TEST");
    assert_eq!(codes_of(&first.location), ["31", "32"]);
    assert_eq!(codes_of(&report.findings[1].location), ["31", "33"]);
    let overlay = &report.findings[2];
    assert_eq!(overlay.rule, "o.fake.pair");
    assert_eq!(overlay.profile, "overlay");
    assert_eq!(overlay.severity, Severity::Error);
    assert_eq!(overlay.source.section, "9");
    assert!(report.has_errors());
    assert_eq!(report.count(Severity::Warning), 2);
}

#[test]
fn accepted_finding_matches_by_rule_and_ids() {
    let mut event = sample_event();
    event.accepted_findings = vec![
        // Order and duplicates of the codes do not matter.
        accepted("t.fake.pair", &["32", "31", "32"], "wanted"),
        // Another rule with the same controls does not accept it.
        accepted("t.other.pair", &["31", "33"], "other rule"),
        // A course makes it another key.
        AcceptedFinding {
            course: Some("B".to_owned()),
            ..accepted("t.fake.pair", &["31", "33"], "with course")
        },
    ];
    let rules = rule_set(vec![(fake_rule(), "base")]);
    let report = run(&event, &rules);
    assert_eq!(report.accepted.len(), 1, "{:?}", report.accepted);
    let (finding, note) = &report.accepted[0];
    assert_eq!(codes_of(&finding.location), ["31", "32"]);
    assert_eq!(note, "wanted");
    assert_eq!(report.findings.len(), 1);
    assert_eq!(codes_of(&report.findings[0].location), ["31", "33"]);

    // An unrelated edit (a control moves, another course changes) keeps it silent.
    event.controls[2].map_mm = pt(50.0, 50.0);
    if let CourseKind::Individual { controls } = &mut event.courses[1].kind {
        controls.push(course_control("32"));
    }
    let report = run(&event, &rules);
    assert_eq!(report.accepted.len(), 1);
    assert_eq!(report.accepted[0].1, "wanted");
}

#[test]
fn accepted_finding_returns_when_its_controls_change() {
    let mut event = sample_event();
    event.accepted_findings = vec![accepted("t.fake.pair", &["31", "32"], "wanted")];
    let rules = rule_set(vec![(fake_rule(), "base")]);
    assert_eq!(run(&event, &rules).accepted.len(), 1);

    if let CourseKind::Individual { controls } = &mut event.courses[0].kind {
        controls.swap(1, 2);
    }
    let report = run(&event, &rules);
    assert!(report.accepted.is_empty(), "{:?}", report.accepted);
    assert_eq!(report.findings.len(), 2);
    assert_eq!(codes_of(&report.findings[0].location), ["31", "33"]);
}

#[test]
fn acceptances_that_match_no_finding_are_listed_as_unmatched() {
    // Ruling R25: one acceptance names a control that is gone, one a course that is gone.
    let mut event = sample_event();
    let gone_course = AcceptedFinding {
        course: None,
        legs: vec![LegRef {
            course: "Z".to_owned(),
            after: 1,
            from: "31".to_owned(),
            to: "32".to_owned(),
        }],
        ..accepted("t.fake.pair", &[], "old course")
    };
    event.accepted_findings = vec![
        accepted("t.fake.pair", &["31", "32"], "wanted"),
        accepted("t.fake.pair", &["31", "39"], "control 39 is gone"),
        gone_course.clone(),
    ];
    let rules = rule_set(vec![(fake_rule(), "base")]);
    let report = run(&event, &rules);
    assert_eq!(report.accepted.len(), 1);
    let notes: Vec<&str> = report.unmatched.iter().map(|a| a.note.as_str()).collect();
    assert_eq!(notes, ["control 39 is gone", "old course"]);
    // Unmatched acceptances never count as errors.
    assert!(!report.has_errors());
    let place = Location::of_accepted(&event, &gone_course);
    assert_eq!(place.text(&Lang::english()), "course Z, leg 31–32");
}

#[test]
fn rules_switched_off_by_an_overlay_are_skipped_with_its_id() {
    let mut rules = rule_set(vec![(fake_rule(), "base")]);
    rules.switched_off.push(crate::rules::SwitchedOff {
        rule: rule("t.code.range", "code_range", Severity::Error),
        profile: "base".to_owned(),
        by: "nor-bb-2021".to_owned(),
    });
    let report = run(&sample_event(), &rules);
    let off: Vec<&Skipped> = report
        .skipped
        .iter()
        .filter(|s| s.reason_id == SKIP_SWITCHED_OFF)
        .collect();
    assert_eq!(off.len(), 1, "{:?}", report.skipped);
    assert_eq!(off[0].rule.as_deref(), Some("t.code.range"));
    assert_eq!(off[0].profile.as_deref(), Some("base"));
    assert_eq!(off[0].location, Location::Event);
    let de = crate::text::parse_lang_arg("de").unwrap();
    assert_eq!(
        off[0].text(&de),
        "Das Regelprofil nor-bb-2021 schaltet die Prüfregel ab"
    );
    assert_eq!(
        off[0].text(&Lang::english()),
        "rule profile nor-bb-2021 switches the rule off"
    );
}

#[test]
fn starts_and_finishes_are_named_by_their_role() {
    let event = sample_event();
    let start = Location::Control { code: code("S1") };
    assert_eq!(start.text_in(&Lang::english(), &event), "start S1");
    let mixed = Location::Controls {
        codes: vec![code("31"), code("F1")],
    };
    let de = crate::text::parse_lang_arg("de").unwrap();
    assert_eq!(mixed.text_in(&de, &event), "Posten 31, Ziel F1");
    let plain = Location::Controls {
        codes: vec![code("31"), code("32")],
    };
    assert_eq!(plain.text_in(&de, &event), "Posten 31, 32");
}

#[test]
fn a_proposal_of_this_program_is_named_without_its_plan_section() {
    let de = crate::text::parse_lang_arg("de").unwrap();
    let proposal = SourceRef {
        doc: "claude".to_owned(),
        section: "M2b plan, Task 5".to_owned(),
    };
    assert_eq!(
        source_text(&de, &proposal),
        "Vorschlag von freemapper-courseplan"
    );
    assert_eq!(
        source_text(&Lang::english(), &proposal),
        "proposal of freemapper-courseplan"
    );
    let wkb = SourceRef {
        doc: "WKB26".to_owned(),
        section: "B 7.2.1".to_owned(),
    };
    assert_eq!(source_text(&de, &wkb), "WKB26 B 7.2.1");
}

#[test]
fn finding_keys_follow_the_location() {
    let rules = rule_set(vec![(fake_rule(), "base")]);
    let run = RuleRun {
        set_rule: &rules.rules[0],
        courses: vec![],
        siblings: vec![],
    };
    let key = |location| run.finding(location, "x", vec![]).key();
    let event = key(Location::Event);
    assert_eq!(event.rule, "t.fake.pair");
    assert_eq!(
        (event.course, event.controls, event.legs),
        (None, vec![], vec![])
    );
    let course = key(Location::Course("A".to_owned()));
    assert_eq!(course.course.as_deref(), Some("A"));
    let controls = key(Location::Controls {
        codes: vec![code("33"), code("S1"), code("31"), code("33")],
    });
    assert_eq!(controls.controls, [code("31"), code("33"), code("S1")]);
    assert_eq!(controls.course, None);
    let leg = key(Location::Leg {
        course: "A".to_owned(),
        leg: span(3, "33", "34"),
    });
    assert_eq!(
        leg.legs,
        [LegRef {
            course: "A".to_owned(),
            after: 3,
            from: "33".to_owned(),
            to: "34".to_owned(),
        }]
    );
    assert_eq!(leg.course, None);
    let one = key(Location::Control { code: code("31") });
    assert_eq!(one.controls, [code("31")]);
    let legs = key(Location::Legs {
        course: "A".to_owned(),
        legs: vec![span(5, "35", "F1"), span(0, "S1", "31")],
    });
    assert_eq!(Place::of_key(&legs), Place::Legs);
    assert_eq!(
        legs.legs,
        [
            span(0, "S1", "31").leg_ref("A"),
            span(5, "35", "F1").leg_ref("A")
        ]
    );
    assert_eq!((legs.course, legs.controls), (None, vec![]));
}

#[test]
fn rules_filtered_out_for_the_whole_event_are_skipped_with_the_reason() {
    let mut event = sample_event();
    let mut long = fake_rule();
    long.disciplines = vec![Discipline::Long];
    let mut forest = rule("t.fake.forest", "min_distance", Severity::Warning);
    forest.map_kinds = vec![MapKind::Forest];
    let rules = rule_set(vec![(long, "base"), (forest, "base")]);

    event.event.discipline = Discipline::Training;
    let report = run(&event, &rules);
    assert!(report.findings.is_empty());
    let reasons: Vec<_> = report
        .skipped
        .iter()
        .map(|s| (s.rule.as_deref(), s.reason_id, s.location.clone()))
        .collect();
    assert_eq!(
        reasons,
        [
            (Some("t.fake.pair"), SKIP_TRAINING, Location::Event),
            (Some("t.fake.forest"), SKIP_MAP_KIND, Location::Event),
        ]
    );
    assert_eq!(
        report.skipped[1].args,
        [("kind", ArgValue::Text("sprint".into()))]
    );
    assert_eq!(report.skipped[0].profile.as_deref(), Some("base"));

    event.event.discipline = Discipline::Middle;
    let report = run(&event, &rules);
    assert_eq!(report.skipped[0].reason_id, SKIP_DISCIPLINE);
    assert_eq!(
        report.skipped[0].args,
        [("discipline", ArgValue::Text("middle".into()))]
    );
}

#[test]
fn a_rule_is_not_skipped_while_another_rule_of_its_key_applies() {
    let mut event = sample_event();
    event.event.discipline = Discipline::Middle;
    let mut long = fake_rule();
    long.disciplines = vec![Discipline::Long];
    let mut middle = rule("t.fake.middle", "min_distance", Severity::Warning);
    middle.disciplines = vec![Discipline::Middle];
    let rules = rule_set(vec![(long, "base"), (middle, "base")]);
    let report = run(&event, &rules);
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);
    assert_eq!(report.findings.len(), 2);
    assert!(report.findings.iter().all(|f| f.rule == "t.fake.middle"));
}

#[test]
fn a_print_scale_without_rule_is_skipped_per_course() {
    let mut event = sample_event();
    event.courses[0].planned_print_scale = Some(4000);
    event.courses[1].planned_print_scale = Some(2500);
    let mut sprint = fake_rule();
    sprint.print_scales = vec![3000, 4000];
    let mut forest = rule("t.fake.forest", "min_distance", Severity::Warning);
    forest.print_scales = vec![10000, 15000];
    let rules = rule_set(vec![(sprint, "base"), (forest, "base")]);
    let report = run(&event, &rules);

    // Course A has a rule for 1:4000, so only the sprint rule checks it.
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].rule, "t.fake.pair");
    assert_eq!(codes_of(&report.findings[0].location), ["31", "32"]);
    // Course B at 1:2500 has none: each rule of the key says so, once.
    let skipped: Vec<_> = report
        .skipped
        .iter()
        .map(|s| (s.rule.as_deref().unwrap(), s.reason_id, &s.location))
        .collect();
    let b = Location::Course("B".to_owned());
    assert_eq!(
        skipped,
        [
            ("t.fake.pair", SKIP_PRINT_SCALE, &b),
            ("t.fake.forest", SKIP_PRINT_SCALE, &b),
        ]
    );
    assert_eq!(report.skipped[0].args, [("scale", ArgValue::Int(2500))]);
}

#[test]
fn courses_of_other_kinds_are_skipped_once() {
    let mut event = sample_event();
    event.courses[1].kind = CourseKind::Score {
        time_limit_min: 45,
        penalty_per_min: 10,
        controls: vec![],
    };
    let rules = rule_set(vec![(fake_rule(), "base")]);
    let report = run(&event, &rules);
    assert_eq!(report.findings.len(), 1, "only course A is checked");
    assert_eq!(
        report.skipped,
        [Skipped {
            rule: None,
            profile: None,
            location: Location::Course("B".to_owned()),
            reason_id: SKIP_COURSE_KIND,
            args: vec![("kind", ArgValue::Text("score".into()))],
        }]
    );
}

#[test]
fn every_known_key_has_a_check() {
    let keys: Vec<&str> = CHECKS.iter().map(|c| c.key).collect();
    assert_eq!(keys, KNOWN_KEYS);
}

#[test]
fn texts_are_localized() {
    let de = crate::text::parse_lang_arg("de").unwrap();
    let skipped = Skipped {
        rule: Some("r".to_owned()),
        profile: None,
        location: Location::Leg {
            course: "A".to_owned(),
            leg: span(3, "36", "37"),
        },
        reason_id: SKIP_PRINT_SCALE,
        args: vec![("scale", ArgValue::Int(2500))],
    };
    assert!(
        skipped.text(&de).contains("1:2500"),
        "{}",
        skipped.text(&de)
    );
    assert_eq!(skipped.location.text(&de), "Bahn A, Strecke 36–37");
    let start = Location::Leg {
        course: "A".to_owned(),
        leg: span(0, "S1", "31"),
    };
    assert_eq!(start.text(&Lang::english()), "course A, leg start–31");
    let finish = Location::Leg {
        course: "A".to_owned(),
        leg: span(7, "37", "F1"),
    };
    assert_eq!(finish.text(&de), "Bahn A, Strecke 37–Ziel");
    let two = Location::Legs {
        course: "A".to_owned(),
        legs: vec![span(0, "S1", "31"), span(4, "34", "35")],
    };
    assert_eq!(
        two.text(&Lang::english()),
        "course A, legs start–31 and 34–35"
    );
    assert_eq!(two.text(&de), "Bahn A, Strecken Start–31 und 34–35");
    assert_eq!(two.place(), Place::Legs);
    assert_eq!(two.course(), Some("A"));
    assert_eq!(format_number(&de, 12.34), "12,3");
    assert_eq!(format_decimals(&de, 4.042, 2), "4,04");
    assert_eq!(format_decimals(&Lang::english(), 3.999, 2), "4");
    assert_eq!(format_decimals(&Lang::english(), 3.67, 2), "3.67");
    assert_eq!(format_number(&Lang::english(), 30.0), "30");
}

#[test]
fn a_course_whose_legs_cannot_be_built_is_skipped() {
    let mut event = sample_event();
    event.courses.push(course("C", &["31", "99"]));
    let rules = rule_set(vec![(fake_rule(), "base")]);
    let report = run(&event, &rules);
    assert_eq!(
        report.findings.len(),
        2,
        "courses A and B are checked, C is not"
    );
    assert_eq!(
        report.skipped,
        [Skipped {
            rule: None,
            profile: None,
            location: Location::Course("C".to_owned()),
            reason_id: SKIP_COURSE_UNRESOLVED,
            args: vec![("id", ArgValue::Text("99".into()))],
        }]
    );
    let de = crate::text::parse_lang_arg("de").unwrap();
    assert_eq!(
        report.skipped[0].text(&de),
        "Die Bahn nennt 99, das die Veranstaltung nicht definiert; sie wird nicht geprüft"
    );
}
