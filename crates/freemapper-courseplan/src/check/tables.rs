//! Table checks (plan M2b, Task 8): the length of a course against the HE course (WKB26
//! Anlage 5 Nr. 3 to 5), easy and hard categories on one course (Anlage 5 Nr. 2) and the
//! performance kilometres against the NOR recommendation (overlay nor-bb-2021).
//!
//! The tables work on rule categories (ruling R23): the `rule_category` of the classes of a
//! course. A class without a category skips the course, a course without classes gives nothing.
//! A course whose categories hit several table rows gets one finding that names every row it
//! violates.

use freemapper_courseplan_model::Class;
use toml::Value;
use toml::map::Map;

use super::{
    ArgValue, Args, CheckContext, CourseContext, Finding, Location, RuleRun, SKIP_CLASS_NOT_MAPPED,
    SKIP_CLIMB_MISSING, SKIP_COURSE_UNRESOLVED, SKIP_VALUE_MISSING, Skipped, number, value_number,
};
use crate::course::course_length_m;
use crate::rules::Severity;

/// Reason: no course carries the reference category; argument `category`.
const SKIP_NO_REFERENCE: &str = "check-skip-length-no-reference";
/// Reason: the reference course has length 0; argument `course`.
const SKIP_REFERENCE_EMPTY: &str = "check-skip-length-reference-empty";
/// Reason: categories of the course are in no row of the table; argument `categories`.
const SKIP_NOT_IN_TABLE: &str = "check-skip-category-not-in-table";

/// Slack for the comparison with a table bound, in the unit of the bound.
const SLACK: f64 = 1e-6;

fn course_location(course: &CourseContext<'_>) -> Location {
    Location::Course(course.course.name.clone())
}

fn text_arg(s: impl Into<String>) -> ArgValue {
    ArgValue::Text(s.into())
}

fn missing(run: &RuleRun<'_>, name: &str, skipped: &mut Vec<Skipped>) {
    skipped.push(run.skipped(
        Location::Event,
        SKIP_VALUE_MISSING,
        vec![("name", text_arg(name))],
    ));
}

/// The rule categories of the classes of a course, each once in order of first use, and the
/// names of the classes without a category. A class the event does not define is left out; the
/// project file refuses it elsewhere.
fn categories_of<'c>(
    ctx: &'c CheckContext<'_>,
    course: &CourseContext<'_>,
) -> (Vec<&'c str>, Vec<&'c str>) {
    let mut categories: Vec<&str> = Vec::new();
    let mut unmapped: Vec<&str> = Vec::new();
    for name in &course.course.classes {
        let Some(class) = ctx.event.classes.iter().find(|c: &&Class| &c.name == name) else {
            continue;
        };
        match class.rule_category.as_deref() {
            Some(category) if !categories.contains(&category) => categories.push(category),
            Some(_) => {}
            None => unmapped.push(&class.name),
        }
    }
    (categories, unmapped)
}

/// The categories of a course for a table check. A class without a category skips the course,
/// one skip per class; then the result is `None`. A course without classes gives an empty list.
fn course_categories<'c>(
    ctx: &'c CheckContext<'_>,
    run: &RuleRun<'_>,
    course: &CourseContext<'_>,
    skipped: &mut Vec<Skipped>,
) -> Option<Vec<&'c str>> {
    let (categories, unmapped) = categories_of(ctx, course);
    for name in &unmapped {
        skipped.push(run.skipped(
            course_location(course),
            SKIP_CLASS_NOT_MAPPED,
            vec![("name", text_arg(*name))],
        ));
    }
    unmapped.is_empty().then_some(categories)
}

/// A number of a table value, an integer or a float.
fn as_number(value: &Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|i| i as f64))
}

/// A row of the `courses` table: its label, its categories and the key's numbers.
struct Row {
    categories: Vec<String>,
    /// `he_length_percent` for `length_table`, `[lkm, lkm]` for `nor_lkm`.
    bounds: (f64, f64),
    target: f64,
}

/// The rows of the `courses` value. A malformed row is a missing value.
fn rows(run: &RuleRun<'_>, nor: bool) -> Option<Vec<Row>> {
    let array = run.rule().values.get("courses")?.as_array()?;
    array
        .iter()
        .map(|row| {
            let row = row.as_table()?;
            let categories = row
                .get("categories")?
                .as_array()?
                .iter()
                .map(|c| c.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()?;
            let (bounds, target) = if nor {
                let lkm = as_number(row.get("lkm")?)?;
                ((lkm, lkm), lkm)
            } else {
                let range = row.get("he_length_percent")?.as_array()?;
                let (min, max) = (as_number(range.first()?)?, as_number(range.get(1)?)?);
                ((min, max), max)
            };
            Some(Row {
                categories,
                bounds,
                target,
            })
        })
        .collect()
}

/// The table rows the categories hit, once each in table order, with the categories of the
/// course in each row, and the categories that are in no row.
fn hit_rows<'t, 'c>(
    table: &'t [Row],
    categories: &[&'c str],
) -> (Vec<(&'t Row, Vec<&'c str>)>, Vec<&'c str>) {
    let mut hit: Vec<(&Row, Vec<&str>)> = Vec::new();
    let mut unknown = Vec::new();
    for &category in categories {
        match table
            .iter()
            .find(|row| row.categories.iter().any(|c| c == category))
        {
            Some(row) => match hit.iter_mut().find(|(r, _)| std::ptr::eq(*r, row)) {
                Some((_, own)) => own.push(category),
                None => hit.push((row, vec![category])),
            },
            None => unknown.push(category),
        }
    }
    hit.sort_by_key(|(row, _)| table.iter().position(|r| std::ptr::eq(r, *row)));
    (hit, unknown)
}

/// `length_table`: the length of a course as a percentage of the length of the HE course lies in
/// the range of every table row its categories hit (WKB26 Anlage 5 Nr. 3 to 5, length by B 7.2.5).
/// The reference is the first course with a class of the category `reference_category`; with two
/// such courses a hint names both. The reference course itself is 100 % and never a finding. No
/// reference, or one of length 0, skips the rule for the event; categories in no row skip the
/// course with their names.
pub(super) fn length_table(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(table) = rows(run, false) else {
        missing(run, "courses", skipped);
        return;
    };
    let Some(reference) = run
        .rule()
        .values
        .get("reference_category")
        .and_then(Value::as_str)
    else {
        missing(run, "reference_category", skipped);
        return;
    };
    let carriers: Vec<&CourseContext<'_>> = ctx
        .courses
        .iter()
        .filter(|course| categories_of(ctx, course).0.contains(&reference))
        .collect();
    let Some(&he) = carriers.first() else {
        skipped.push(run.skipped(
            Location::Event,
            SKIP_NO_REFERENCE,
            vec![("category", text_arg(reference))],
        ));
        return;
    };
    if carriers.len() > 1 {
        let names: Vec<&str> = carriers.iter().map(|c| c.course.name.as_str()).collect();
        findings.push(run.finding_with_severity(
            Severity::Hint,
            Location::Event,
            "check-length-table-reference-twice",
            vec![
                ("category", text_arg(reference)),
                ("courses", text_arg(names.join(", "))),
                ("first", text_arg(he.course.name.as_str())),
            ],
        ));
    }
    let length_of = |course: &CourseContext<'_>, skipped: &mut Vec<Skipped>| {
        let length = course_length_m(ctx.event, course.course, ctx.map_scale).ok();
        if length.is_none() {
            skipped.push(run.skipped(
                course_location(course),
                SKIP_COURSE_UNRESOLVED,
                vec![("id", text_arg(course.course.name.as_str()))],
            ));
        }
        length
    };
    let Some(he_length) = length_of(he, skipped) else {
        return;
    };
    if he_length <= 0.0 {
        skipped.push(run.skipped(
            Location::Event,
            SKIP_REFERENCE_EMPTY,
            vec![("course", text_arg(he.course.name.as_str()))],
        ));
        return;
    }
    for course in run.courses.iter().copied() {
        if std::ptr::eq(course, he) {
            continue;
        }
        let Some(categories) = course_categories(ctx, run, course, skipped) else {
            continue;
        };
        if categories.is_empty() {
            continue;
        }
        let (hit, unknown) = hit_rows(&table, &categories);
        if !unknown.is_empty() {
            skipped.push(run.skipped(
                course_location(course),
                SKIP_NOT_IN_TABLE,
                vec![("categories", text_arg(unknown.join(", ")))],
            ));
        }
        if hit.is_empty() {
            continue;
        }
        let Some(length) = length_of(course, skipped) else {
            continue;
        };
        let percent = length / he_length * 100.0;
        let violated: Vec<Args> = hit
            .iter()
            .filter(|(row, _)| percent < row.bounds.0 - SLACK || percent > row.bounds.1 + SLACK)
            .map(|(row, own)| {
                vec![
                    ("categories", text_arg(own.join(", "))),
                    ("percent", ArgValue::Number(percent)),
                    ("reference", text_arg(reference)),
                    ("min", ArgValue::Number(row.bounds.0)),
                    ("max", ArgValue::Number(row.bounds.1)),
                ]
            })
            .collect();
        if violated.is_empty() {
            continue;
        }
        findings.push(run.finding(
            course_location(course),
            "check-length-table",
            vec![
                ("course", text_arg(course.course.name.as_str())),
                ("rows", ArgValue::List("check-length-table-row", violated)),
            ],
        ));
    }
}

/// The levels listed under `name` as integers.
fn levels(run: &RuleRun<'_>, name: &str) -> Option<Vec<i64>> {
    run.rule()
        .values
        .get(name)?
        .as_array()?
        .iter()
        .map(Value::as_integer)
        .collect()
}

/// `difficulty_mix`: a course with categories of an easy level and of a hard level is an error
/// (WKB26 Anlage 5 Nr. 2). Categories without a level, such as D-12b, are ignored.
pub(super) fn difficulty_mix(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let table: Option<&Map<String, Value>> = run
        .rule()
        .values
        .get("levels")
        .and_then(Value::as_table)
        .map(|t| t as &Map<String, Value>);
    let Some(table) = table else {
        missing(run, "levels", skipped);
        return;
    };
    let Some(easy) = levels(run, "easy_levels") else {
        missing(run, "easy_levels", skipped);
        return;
    };
    let Some(hard) = levels(run, "hard_levels") else {
        missing(run, "hard_levels", skipped);
        return;
    };
    for course in run.courses.iter().copied() {
        let Some(categories) = course_categories(ctx, run, course, skipped) else {
            continue;
        };
        let level_of = |category: &str| table.get(category).and_then(Value::as_integer);
        let on_side = |side: &[i64]| -> Vec<&str> {
            categories
                .iter()
                .copied()
                .filter(|c| level_of(c).is_some_and(|l| side.contains(&l)))
                .collect()
        };
        let (easy_found, hard_found) = (on_side(&easy), on_side(&hard));
        if easy_found.is_empty() || hard_found.is_empty() {
            continue;
        }
        findings.push(run.finding(
            course_location(course),
            "check-difficulty-mix",
            vec![
                ("course", text_arg(course.course.name.as_str())),
                ("easy", text_arg(easy_found.join(", "))),
                ("hard", text_arg(hard_found.join(", "))),
            ],
        ));
    }
}

/// `nor_lkm`: the performance kilometres of a course, `length_km + climb_km × climb_factor`, lie
/// within `tolerance_percent` of the target of every table row the categories hit (BB-Lkm21,
/// BB-Kombi14). A course without climb is skipped (a climb of 0 is a value); categories in no row
/// give a hint, since the recommendation does not cover them.
pub(super) fn nor_lkm(
    ctx: &CheckContext<'_>,
    run: &RuleRun<'_>,
    findings: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let Some(table) = rows(run, true) else {
        missing(run, "courses", skipped);
        return;
    };
    let Some(factor) = number(run, "climb_factor", skipped) else {
        return;
    };
    let Some(tolerance) = number(run, "tolerance_percent", skipped) else {
        return;
    };
    debug_assert!(value_number(run.rule(), "climb_factor").is_some());
    for course in run.courses.iter().copied() {
        let Some(categories) = course_categories(ctx, run, course, skipped) else {
            continue;
        };
        if categories.is_empty() {
            continue;
        }
        let (hit, unknown) = hit_rows(&table, &categories);
        if !unknown.is_empty() {
            findings.push(run.finding_with_severity(
                Severity::Hint,
                course_location(course),
                "check-nor-lkm-category-unknown",
                vec![
                    ("course", text_arg(course.course.name.as_str())),
                    ("categories", text_arg(unknown.join(", "))),
                ],
            ));
        }
        if hit.is_empty() {
            continue;
        }
        let Some(climb) = course.course.climb_m else {
            skipped.push(run.skipped(course_location(course), SKIP_CLIMB_MISSING, Vec::new()));
            continue;
        };
        let Ok(metres) = course_length_m(ctx.event, course.course, ctx.map_scale) else {
            skipped.push(run.skipped(
                course_location(course),
                SKIP_COURSE_UNRESOLVED,
                vec![("id", text_arg(course.course.name.as_str()))],
            ));
            continue;
        };
        let length_km = metres / 1000.0;
        let lkm = length_km + f64::from(climb) / 1000.0 * factor;
        let violated: Vec<Args> = hit
            .iter()
            .filter_map(|(row, own)| {
                let min = row.target * (1.0 - tolerance / 100.0);
                let max = row.target * (1.0 + tolerance / 100.0);
                (lkm < min - SLACK || lkm > max + SLACK).then(|| {
                    vec![
                        ("categories", text_arg(own.join(", "))),
                        ("target", ArgValue::Number(row.target)),
                        ("min", ArgValue::Number(min)),
                        ("max", ArgValue::Number(max)),
                    ]
                })
            })
            .collect();
        if violated.is_empty() {
            continue;
        }
        findings.push(run.finding(
            course_location(course),
            "check-nor-lkm",
            vec![
                ("course", text_arg(course.course.name.as_str())),
                ("actual", ArgValue::Number(lkm)),
                ("length", ArgValue::Number(length_km)),
                ("climb", ArgValue::Int(i64::from(climb))),
                ("rows", ArgValue::List("check-nor-lkm-row", violated)),
            ],
        ));
    }
}

#[cfg(test)]
mod tests;
