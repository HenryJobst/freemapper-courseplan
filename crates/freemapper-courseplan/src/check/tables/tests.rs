//! Tests of the table checks. Maps are 1:10000 (not a standard scale; the note about it is dropped), so 1 mm on the map is 10 m on the ground. Every
//! course runs from the start S1 at (0, 0) over one control on the x axis to the finish F1 at
//! (100, 0); a course of `total` mm has its control at x = (total + 100) / 2.

use freemapper_courseplan_model::{Class, Discipline, Event, SpecialObject};

use super::super::testing::*;
use super::super::*;
use crate::rules::load_rule_set;

/// A course: name, total length in map mm, classes as (name, rule category).
type Spec<'a> = (&'a str, f64, &'a [(&'a str, Option<&'a str>)]);

fn build(discipline: Discipline, specs: &[Spec<'_>]) -> Event {
    let codes: Vec<String> = (0..specs.len()).map(|i| format!("{}", 31 + i)).collect();
    let controls: Vec<(&str, f64, f64)> = specs
        .iter()
        .zip(&codes)
        .map(|((_, total, _), code)| (code.as_str(), (total + 100.0) / 2.0, 0.0))
        .collect();
    let courses = specs
        .iter()
        .zip(&codes)
        .map(|((name, _, classes), code)| {
            let mut c = course(name, &[code]);
            c.classes = classes.iter().map(|(n, _)| (*n).to_owned()).collect();
            c
        })
        .collect();
    let mut e = event(&controls, courses);
    e.event.discipline = discipline;
    e.classes.clear();
    for (_, _, classes) in specs {
        for (name, category) in *classes {
            if !e.classes.iter().any(|c| c.name == *name) {
                e.classes.push(Class {
                    name: (*name).to_owned(),
                    rule_category: category.map(str::to_owned),
                    expected_starters: None,
                });
            }
        }
    }
    e
}

/// A class named like its category.
const fn c(category: &str) -> (&str, Option<&str>) {
    (category, Some(category))
}

const LENGTH_TABLE: &str = r#"reference_category = "HE"
courses = [
  { course = 1, printed = "HE", categories = ["HE"], he_length_percent = [100, 100] },
  { course = 2, printed = "H-18", categories = ["H-18"], he_length_percent = [50, 55] },
  { course = 3, printed = "D-16", categories = ["D-16"], he_length_percent = [30, 40] },
]"#;

fn length_rule() -> Rule {
    let mut r = with_values(
        rule("t.length", "length_table", Severity::Warning),
        LENGTH_TABLE,
    );
    crate::rules::set_disciplines(&mut r, &[Discipline::Long]);
    r
}

fn mix_rule() -> Rule {
    with_values(
        rule("t.mix", "difficulty_mix", Severity::Error),
        r#"easy_levels = [1, 2]
hard_levels = [3, 4, 5]
[levels]
"D-10" = 1
"H-12" = 2
"H-18" = 4
"HE" = 5"#,
    )
}

const NOR_TABLE: &str = r#"climb_factor = 10
tolerance_percent = 6
courses = [
  { course = "C", lkm = 3.9, o_value = 2, categories = ["D-12", "H-12"] },
  { course = "F", lkm = 7.1, o_value = 4, categories = ["H-18"] },
]"#;

fn nor_rule() -> Rule {
    let mut r = with_values(rule("t.nor", "nor_lkm", Severity::Warning), NOR_TABLE);
    crate::rules::set_disciplines(&mut r, &[Discipline::Long, Discipline::Middle]);
    r
}

/// The report without the note about a non-standard map scale, which these maps all raise.
fn without_scale_note(mut r: CheckReport) -> CheckReport {
    r.skipped.retain(|s| s.reason_id != NOTE_NON_STANDARD_SCALE);
    r
}

fn run(event: &Event, rules: Vec<Rule>) -> CheckReport {
    without_scale_note(report(event, &map(10000, vec![]), rules))
}

fn at_course(name: &str) -> Location {
    Location::Course(name.to_owned())
}

fn skip_ids(r: &CheckReport) -> Vec<&'static str> {
    r.skipped.iter().map(|s| s.reason_id).collect()
}

/// A long-distance event with a HE course of 300 mm (3 km) and a second course.
fn long(total: f64, classes: &[(&str, Option<&str>)]) -> Event {
    build(
        Discipline::Long,
        &[("HE", 300.0, &[c("HE")]), ("B", total, classes)],
    )
}

// ---- length_table ----

#[test]
fn a_course_inside_its_range_gives_no_finding() {
    let r = run(&long(159.0, &[c("H-18")]), vec![length_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert!(r.skipped.is_empty(), "{:?}", r.skipped);
}

#[test]
fn a_course_outside_its_range_is_a_warning_at_the_course() {
    let r = run(&long(169.5, &[c("H-18")]), vec![length_rule()]);
    assert_eq!(r.findings.len(), 1);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(f.location, at_course("B"));
    assert!(en(f).contains("56.5 %"), "{}", en(f));
    assert!(en(f).contains("50 to 55 %"), "{}", en(f));
    assert!(en(f).contains("H-18"), "{}", en(f));
    assert!(de(f).contains("56,5 %"), "{}", de(f));
    assert!(de(f).contains("50 bis 55 %"), "{}", de(f));
}

#[test]
fn the_he_course_is_never_a_finding() {
    let e = build(Discipline::Long, &[("HE", 300.0, &[c("HE")])]);
    let r = run(&e, vec![length_rule()]);
    assert!(r.findings.is_empty() && r.skipped.is_empty());
}

#[test]
fn without_an_he_course_the_rule_is_skipped_once() {
    let e = build(Discipline::Long, &[("B", 160.0, &[c("H-18")])]);
    let r = run(&e, vec![length_rule()]);
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].location, Location::Event);
    assert_eq!(r.skipped[0].reason_id, "check-skip-length-no-reference");
    assert!(r.skipped[0].text(&lang_of("en")).contains("HE"));
}

#[test]
fn two_he_courses_use_the_first_and_say_so() {
    let e = build(
        Discipline::Long,
        &[
            ("HE1", 300.0, &[c("HE")]),
            ("HE2", 600.0, &[("HE-b", Some("HE"))]),
            ("B", 159.0, &[c("H-18")]),
        ],
    );
    let r = run(&e, vec![length_rule()]);
    let notes: Vec<_> = r
        .findings
        .iter()
        .filter(|f| f.location == Location::Event)
        .collect();
    assert_eq!(notes.len(), 1, "{:?}", r.findings);
    assert_eq!(notes[0].severity, Severity::Hint);
    assert!(en(notes[0]).contains("HE1") && en(notes[0]).contains("HE2"));
    // B is 53 % of HE1; HE2 is 200 % of it and violates the HE row.
    assert!(!r.findings.iter().any(|f| f.location == at_course("B")));
    assert!(r.findings.iter().any(|f| f.location == at_course("HE2")));
}

#[test]
fn a_reference_of_length_zero_skips_the_rule() {
    let mut e = build(
        Discipline::Long,
        &[("HE", 0.0, &[c("HE")]), ("B", 100.0, &[c("H-18")])],
    );
    // A course of length 0 needs start, control and finish on one point.
    for s in &mut e.specials {
        if let SpecialObject::Finish { map_mm, .. } = s {
            *map_mm = pt(0.0, 0.0);
        }
    }
    e.controls[0].map_mm = pt(0.0, 0.0);
    let r = run(&e, vec![length_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert_eq!(skip_ids(&r), ["check-skip-length-reference-empty"]);
}

#[test]
fn a_class_without_category_skips_the_course_and_names_the_class() {
    let r = run(
        &long(160.0, &[("H18 open", None), c("H-18")]),
        vec![length_rule()],
    );
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].location, at_course("B"));
    assert_eq!(r.skipped[0].reason_id, SKIP_CLASS_NOT_MAPPED);
    assert!(r.skipped[0].text(&lang_of("en")).contains("H18 open"));
}

#[test]
fn a_course_without_classes_gives_nothing() {
    let r = run(&long(160.0, &[]), vec![length_rule()]);
    assert!(r.findings.is_empty() && r.skipped.is_empty());
}

#[test]
fn a_category_in_no_row_is_skipped_with_its_name() {
    let r = run(&long(160.0, &[c("D-99")]), vec![length_rule()]);
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].location, at_course("B"));
    assert_eq!(r.skipped[0].reason_id, "check-skip-category-not-in-table");
    assert!(r.skipped[0].text(&lang_of("de")).contains("D-99"));
}

#[test]
fn a_course_hitting_two_rows_gives_one_finding_naming_both() {
    // 60 % of HE: too long for H-18 (50 to 55) and for D-16 (30 to 40).
    let r = run(&long(180.0, &[c("H-18"), c("D-16")]), vec![length_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert!(
        en(f).contains("50 to 55 %") && en(f).contains("30 to 40 %"),
        "{}",
        en(f)
    );
    assert!(
        de(f).contains("50 bis 55 %") && de(f).contains("30 bis 40 %"),
        "{}",
        de(f)
    );
}

#[test]
fn one_satisfied_row_does_not_hide_a_violated_one() {
    // 35 % is inside D-16 and far below H-18.
    let r = run(&long(105.0, &[c("H-18"), c("D-16")]), vec![length_rule()]);
    assert_eq!(r.findings.len(), 1);
    assert!(en(&r.findings[0]).contains("H-18"));
    assert!(!en(&r.findings[0]).contains("D-16"));
}

#[test]
fn the_length_table_follows_the_discipline_filter() {
    let mut e = long(180.0, &[c("H-18")]);
    e.event.discipline = Discipline::Training;
    let r = run(&e, vec![length_rule()]);
    assert!(r.findings.is_empty());
    assert_eq!(skip_ids(&r), [SKIP_TRAINING]);
}

#[test]
fn the_wkb_long_table_with_real_data() {
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    for (total, outside) in [(159.0, false), (180.0, true)] {
        let mut e = long(total, &[c("H-18")]);
        e.event.discipline = Discipline::Long;
        let m = map(10000, vec![]);
        let r = without_scale_note(run_checks(&CheckContext::new(&e, &m), &rules, CHECKS));
        let found: Vec<_> = r
            .findings
            .iter()
            .filter(|f| f.rule == "wkb2026.length.long")
            .collect();
        assert_eq!(found.len(), usize::from(outside), "{total}: {found:?}");
        if let Some(f) = found.first() {
            assert!(
                en(f).contains("60 %") && en(f).contains("50 to 55 %"),
                "{}",
                en(f)
            );
            assert_eq!(f.location, at_course("B"));
        }
    }
}

// ---- difficulty_mix ----

#[test]
fn easy_and_hard_categories_on_one_course_are_an_error() {
    let e = build(
        Discipline::Long,
        &[("A", 100.0, &[c("D-10"), c("H-12"), c("H-18"), c("HE")])],
    );
    let r = run(&e, vec![mix_rule()]);
    assert_eq!(r.findings.len(), 1);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Error);
    assert_eq!(f.location, at_course("A"));
    let text = en(f);
    assert!(
        text.contains("D-10, H-12") && text.contains("H-18, HE"),
        "{text}"
    );
    assert!(de(f).contains("D-10, H-12") && de(f).contains("H-18, HE"));
}

#[test]
fn courses_of_one_side_and_categories_without_level_are_fine() {
    let e = build(
        Discipline::Long,
        &[
            ("A", 100.0, &[c("D-10"), c("H-12")]),
            ("B", 100.0, &[c("H-18"), c("HE")]),
            ("C", 100.0, &[c("D-10"), c("D-12b")]),
        ],
    );
    let r = run(&e, vec![mix_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert!(r.skipped.is_empty(), "{:?}", r.skipped);
}

#[test]
fn difficulty_mix_skips_a_class_without_category() {
    let e = build(
        Discipline::Long,
        &[("A", 100.0, &[("Open", None), c("D-10"), c("H-18")])],
    );
    let r = run(&e, vec![mix_rule()]);
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].reason_id, SKIP_CLASS_NOT_MAPPED);
}

#[test]
fn the_wkb_difficulty_mix_with_real_data() {
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    let e = build(
        Discipline::Long,
        &[("A", 100.0, &[c("D-10"), c("H-20"), c("H-12b")])],
    );
    let m = map(10000, vec![]);
    let r = without_scale_note(run_checks(&CheckContext::new(&e, &m), &rules, CHECKS));
    let found: Vec<_> = r
        .findings
        .iter()
        .filter(|f| f.rule == "wkb2026.difficulty.mix")
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].severity, Severity::Error);
    assert!(en(found[0]).contains("D-10") && en(found[0]).contains("H-20"));
    assert!(!en(found[0]).contains("H-12b"));
}

// ---- nor_lkm ----

/// A course of 3 km (300 mm) in category `H-12`, with `climb` metres.
fn nor_event(total: f64, climb: Option<u32>, classes: &[(&str, Option<&str>)]) -> Event {
    let mut e = build(Discipline::Long, &[("A", total, classes)]);
    e.courses[0].climb_m = climb;
    e
}

#[test]
fn a_course_inside_the_tolerance_gives_no_finding() {
    // 3 km + 90 m x 10 / 1000 = 3.9 Lkm.
    let r = run(&nor_event(300.0, Some(90), &[c("H-12")]), vec![nor_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert!(r.skipped.is_empty(), "{:?}", r.skipped);
}

#[test]
fn a_course_outside_the_tolerance_is_a_warning() {
    // 3 + 1.5 = 4.5 Lkm against 3.9 +- 6 % = 3.7 to 4.1.
    let r = run(&nor_event(300.0, Some(150), &[c("H-12")]), vec![nor_rule()]);
    assert_eq!(r.findings.len(), 1);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(f.location, at_course("A"));
    let text = en(f);
    assert!(text.contains("4.5") && text.contains("3.9"), "{text}");
    assert!(text.contains("3.67") && text.contains("4.13"), "{text}");
    assert!(de(f).contains("4,5") && de(f).contains("3,9"), "{}", de(f));
    assert!(de(f).contains("3,67 bis 4,13 Lkm"), "{}", de(f));
}

#[test]
fn lkm_values_show_two_decimals() {
    // 3.042 km and 150 m climb: 4.542 Lkm shows as 4.54, not as 4.5.
    let r = run(&nor_event(304.2, Some(150), &[c("H-12")]), vec![nor_rule()]);
    assert_eq!(r.findings.len(), 1, "{r:?}");
    let text = en(&r.findings[0]);
    assert!(text.contains("4.54 Lkm (3.04 km length"), "{text}");
}

#[test]
fn a_climb_of_zero_is_a_value_and_a_missing_climb_a_skip() {
    // 3.9 km without climb is exactly the target.
    let r = run(&nor_event(390.0, Some(0), &[c("H-12")]), vec![nor_rule()]);
    assert!(r.findings.is_empty() && r.skipped.is_empty(), "{r:?}");
    // 3 km without climb is 3.0 Lkm, too short.
    let r = run(&nor_event(300.0, Some(0), &[c("H-12")]), vec![nor_rule()]);
    assert_eq!(r.findings.len(), 1);
    let r = run(&nor_event(300.0, None, &[c("H-12")]), vec![nor_rule()]);
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].reason_id, SKIP_CLIMB_MISSING);
    assert_eq!(r.skipped[0].location, at_course("A"));
}

#[test]
fn a_category_in_no_row_is_a_skip() {
    // Ruling R26: like length_table, a category the NOR table does not name skips the course
    // for it, with no hint; the categories in a row are still checked (3.9 Lkm, on target).
    let r = run(
        &nor_event(300.0, Some(90), &[c("H-12"), c("D-99")]),
        vec![nor_rule()],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert_eq!(r.skipped.len(), 1);
    let s = &r.skipped[0];
    assert_eq!(s.location, at_course("A"));
    assert_eq!(s.reason_id, "check-skip-category-not-in-table");
    assert!(s.text(&lang_of("en")).contains("D-99"));
}

#[test]
fn a_course_hitting_two_rows_gives_one_finding() {
    // 7.1 Lkm for H-18 is far from 3.9 and 7.1; 3 + 4.1 = 7.1 hits H-18 and misses H-12.
    let r = run(
        &nor_event(300.0, Some(410), &[c("H-12"), c("H-18")]),
        vec![nor_rule()],
    );
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let text = en(&r.findings[0]);
    assert!(text.contains("H-12") && !text.contains("H-18"), "{text}");
    let r = run(
        &nor_event(300.0, Some(0), &[c("H-12"), c("H-18")]),
        vec![nor_rule()],
    );
    assert_eq!(r.findings.len(), 1);
    let text = en(&r.findings[0]);
    assert!(text.contains("H-12") && text.contains("H-18"), "{text}");
}

#[test]
fn nor_lkm_skips_a_class_without_category_and_a_course_without_classes() {
    let r = run(
        &nor_event(300.0, Some(0), &[("Open", None)]),
        vec![nor_rule()],
    );
    assert_eq!(skip_ids(&r), [SKIP_CLASS_NOT_MAPPED]);
    let r = run(&nor_event(300.0, Some(0), &[]), vec![nor_rule()]);
    assert!(r.findings.is_empty() && r.skipped.is_empty());
}

#[test]
fn a_training_and_a_sprint_are_skipped_not_silent() {
    for (discipline, reason) in [
        (Discipline::Training, SKIP_TRAINING),
        (Discipline::Sprint, SKIP_DISCIPLINE),
    ] {
        let mut e = nor_event(300.0, Some(150), &[c("H-12")]);
        e.event.discipline = discipline;
        let r = run(&e, vec![nor_rule()]);
        assert!(r.findings.is_empty());
        assert_eq!(skip_ids(&r), [reason]);
    }
}

#[test]
fn a_sprint_is_skipped_by_the_real_nor_profile() {
    let rules = load_rule_set("wkb-2026", &["nor-bb-2021".to_owned()]).unwrap();
    let mut e = nor_event(300.0, Some(150), &[c("H-12")]);
    e.event.discipline = Discipline::Sprint;
    let m = map(4000, vec![]);
    let r = without_scale_note(run_checks(&CheckContext::new(&e, &m), &rules, CHECKS));
    let skips: Vec<_> = r
        .skipped
        .iter()
        .filter(|s| {
            s.rule
                .as_deref()
                .is_some_and(|id| id.starts_with("norbb2021.lkm"))
        })
        .collect();
    assert_eq!(skips.len(), 3, "{skips:?}");
    assert!(skips.iter().all(|s| s.reason_id == SKIP_DISCIPLINE));
}

#[test]
fn the_nor_long_table_with_real_data() {
    let rules = load_rule_set("wkb-2026", &["nor-bb-2021".to_owned()]).unwrap();
    for (climb, outside) in [(90, false), (150, true)] {
        let e = nor_event(300.0, Some(climb), &[c("H-12")]);
        let m = map(10000, vec![]);
        let r = without_scale_note(run_checks(&CheckContext::new(&e, &m), &rules, CHECKS));
        let found: Vec<_> = r
            .findings
            .iter()
            .filter(|f| f.rule == "norbb2021.lkm.long")
            .collect();
        assert_eq!(found.len(), usize::from(outside), "{climb}: {found:?}");
    }
}
