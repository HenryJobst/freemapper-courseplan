//! The CSV list and the evaluation sheet of a score course (ADR 0070).
//!
//! Both are plain text made from the event alone. The CSV has one row per control with code, value and
//! position in map millimetres at map scale, y pointing down, as the project stores them. The sheet is a
//! Markdown text that prints well as it is: the rules of the course and a table of its controls.

use std::cmp::Ordering;

use freemapper_courseplan_model::{CourseKind, Event, ScoreControl, ScoreLink, score_penalty};

use crate::score::ScoreError;
use crate::text::{FluentValue, Lang, text};

/// The score course `name` of `event`: its controls and links, and the stored penalty.
struct Found<'a> {
    controls: &'a [ScoreControl],
    links: &'a [ScoreLink],
    time_limit_min: u32,
    penalty_per_min: Option<u32>,
}

fn find<'a>(event: &'a Event, name: &str) -> Result<Found<'a>, ScoreError> {
    let course = event
        .courses
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| ScoreError::UnknownCourse(name.to_owned()))?;
    match &course.kind {
        CourseKind::Score {
            time_limit_min,
            penalty_per_min,
            controls,
            links,
        } => Ok(Found {
            controls,
            links,
            time_limit_min: *time_limit_min,
            penalty_per_min: *penalty_per_min,
        }),
        _ => Err(ScoreError::NotAScoreCourse(name.to_owned())),
    }
}

/// Codes in number order where both are numbers, then as text.
fn by_code(a: &str, b: &str) -> Ordering {
    match (a.parse::<u64>(), b.parse::<u64>()) {
        (Ok(x), Ok(y)) => x.cmp(&y).then_with(|| a.cmp(b)),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}

/// Two decimals with a dot, never `-0.00`.
fn mm(value: f64) -> String {
    let s = format!("{value:.2}");
    if s == "-0.00" { "0.00".to_owned() } else { s }
}

/// Text for a Markdown table cell or line: `|` escaped, line breaks turned into blanks.
fn md(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace(['\r', '\n'], " ")
}

fn csv_field(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_owned()
    }
}

/// The controls of the score course `course` as CSV: a header row `code,value,x_mm,y_mm`, then one row per
/// control ordered by code. Positions are map millimetres at map scale, y down, with a dot as decimal
/// separator whatever the locale. Lines end in `\n`; the text is UTF-8.
///
/// # Errors
/// [`ScoreError::UnknownCourse`], [`ScoreError::NotAScoreCourse`], or [`ScoreError::UnknownControl`] when
/// the course names a control the event does not have.
pub fn score_csv(event: &Event, course: &str) -> Result<String, ScoreError> {
    let found = find(event, course)?;
    let mut rows = Vec::with_capacity(found.controls.len());
    for sc in found.controls {
        let control = event
            .controls
            .iter()
            .find(|c| c.code == sc.code)
            .ok_or_else(|| ScoreError::UnknownControl(sc.code.0.clone()))?;
        rows.push((sc.code.0.as_str(), sc.value, &control.map_mm));
    }
    rows.sort_by(|a, b| by_code(a.0, b.0));
    let mut out = String::from("code,value,x_mm,y_mm\n");
    for (code, value, at) in rows {
        out.push_str(&format!(
            "{},{value},{},{}\n",
            csv_field(code),
            mm(at.x),
            mm(at.y)
        ));
    }
    Ok(out)
}

/// The evaluation sheet of the score course `course` as Markdown in `lang`: the time limit, the penalty
/// per started minute (the stored one or the default), the tie rule, the mandatory links with the rule
/// that a runner who does not run a link directly is disqualified, and a table with code and value of every
/// control. Names are escaped for the Markdown table: `|` and line breaks.
///
/// # Errors
/// [`ScoreError::UnknownCourse`] or [`ScoreError::NotAScoreCourse`]. A link that names a control the course
/// does not hold is written as it stands; the project file refuses such a link on load.
pub fn evaluation_sheet(event: &Event, course: &str, lang: &Lang) -> Result<String, ScoreError> {
    let found = find(event, course)?;
    let t = |id: &str, args: &[(&str, FluentValue)]| text(lang, id, args);
    let n = |v: u32| FluentValue::from(i64::from(v));
    let penalty = score_penalty(
        found.penalty_per_min,
        found.controls,
        &event.score_settings(),
    );
    let mut codes: Vec<&ScoreControl> = found.controls.iter().collect();
    codes.sort_by(|a, b| by_code(&a.code.0, &b.code.0));
    let total: u32 = codes.iter().map(|c| c.value).sum();

    let mut out = String::new();
    let mut line = |s: String| {
        out.push_str(&s);
        out.push('\n');
    };
    line(format!(
        "# {}",
        t(
            "score-sheet-title",
            &[("course", FluentValue::from(md(course)))]
        )
    ));
    line(String::new());
    line(format!(
        "- {}",
        t(
            "score-sheet-event",
            &[("title", FluentValue::from(md(&event.event.title)))]
        )
    ));
    line(format!(
        "- {}",
        t(
            "score-sheet-time-limit",
            &[("minutes", n(found.time_limit_min))]
        )
    ));
    let id = if found.penalty_per_min.is_some() {
        "score-sheet-penalty"
    } else {
        "score-sheet-penalty-default"
    };
    line(format!("- {}", t(id, &[("points", n(penalty))])));
    line(format!("- {}", t("score-sheet-tie", &[])));
    line(format!(
        "- {}",
        t(
            "score-sheet-total",
            &[
                ("count", n(u32::try_from(codes.len()).unwrap_or(u32::MAX))),
                ("points", n(total))
            ]
        )
    ));
    line(String::new());
    line(format!("## {}", t("score-sheet-links-heading", &[])));
    line(String::new());
    if found.links.is_empty() {
        line(t("score-sheet-no-links", &[]));
    } else {
        line(t("score-sheet-links-rule", &[]));
        line(String::new());
        line(format!(
            "| {} | {} |",
            t("score-sheet-col-first", &[]),
            t("score-sheet-col-second", &[])
        ));
        line("|---|---|".to_owned());
        for link in found.links {
            line(format!("| {} | {} |", md(&link.a.0), md(&link.b.0)));
        }
    }
    line(String::new());
    line(format!("## {}", t("score-sheet-controls-heading", &[])));
    line(String::new());
    line(format!(
        "| {} | {} |",
        t("score-sheet-col-code", &[]),
        t("score-sheet-col-value", &[])
    ));
    line("|---|---|".to_owned());
    for c in codes {
        line(format!("| {} | {} |", md(&c.code.0), c.value));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_courseplan_model::{
        Control, ControlCode, ControlDescription, ControlKind, Course, EventMeta, FORMAT_VERSION,
        MapPoint, Sources,
    };

    fn event() -> Event {
        let control = |code: &str, x: f64, y: f64| Control {
            ppen_id: None,
            code: ControlCode(code.to_owned()),
            kind: ControlKind::Control,
            map_mm: MapPoint { x, y },
            description: ControlDescription::default(),
            number: None,
        };
        let sc = |code: &str, value| ScoreControl {
            code: ControlCode(code.to_owned()),
            value,
        };
        Event {
            format: FORMAT_VERSION,
            event: EventMeta {
                title: "Hall".to_owned(),
                date: None,
                discipline: Default::default(),
                rule_profile: "wkb-2026".to_owned(),
                rule_overlays: Vec::new(),
                start_window_min: None,
            },
            sources: Sources {
                map: "a.omap".into(),
                ppen: None,
            },
            classes: vec![],
            controls: vec![
                control("102", 5.0, -0.001),
                control("33", 12.345, 7.0),
                control("41", 1.0, 2.5),
            ],
            specials: vec![],
            courses: vec![
                Course {
                    ppen_id: None,
                    name: "Score".to_owned(),
                    classes: vec![],
                    start: "S1".to_owned(),
                    finish: "F1".to_owned(),
                    start_bends: vec![],
                    start_taped: false,
                    print_area: None,
                    climb_m: None,
                    planned_print_scale: None,
                    kind: CourseKind::Score {
                        time_limit_min: 45,
                        penalty_per_min: Some(7),
                        controls: vec![sc("102", 2), sc("33", 3), sc("41", 1)],
                        links: vec![ScoreLink {
                            a: ControlCode("33".to_owned()),
                            b: ControlCode("41".to_owned()),
                        }],
                    },
                },
                Course {
                    ppen_id: None,
                    name: "Indiv".to_owned(),
                    classes: vec![],
                    start: "S1".to_owned(),
                    finish: "F1".to_owned(),
                    start_bends: vec![],
                    start_taped: false,
                    print_area: None,
                    climb_m: None,
                    planned_print_scale: None,
                    kind: CourseKind::Individual { controls: vec![] },
                },
            ],
            accepted_findings: vec![],
            sync: None,
            score: None,
            rejected_sites: Vec::new(),
        }
    }

    #[test]
    fn csv_is_stable() {
        let csv = score_csv(&event(), "Score").unwrap();
        assert_eq!(
            csv,
            "code,value,x_mm,y_mm\n33,3,12.35,7.00\n41,1,1.00,2.50\n102,2,5.00,0.00\n"
        );
        assert_eq!(csv, score_csv(&event(), "Score").unwrap());
    }

    #[test]
    fn csv_positions_are_map_millimetres() {
        let mut e = event();
        e.controls[1].map_mm = MapPoint { x: 80.0, y: 60.5 };
        let csv = score_csv(&e, "Score").unwrap();
        assert!(csv.contains("33,3,80.00,60.50\n"), "{csv}");
    }

    #[test]
    fn csv_refuses_other_courses() {
        assert_eq!(
            score_csv(&event(), "Indiv"),
            Err(ScoreError::NotAScoreCourse("Indiv".to_owned()))
        );
        assert_eq!(
            score_csv(&event(), "Nope"),
            Err(ScoreError::UnknownCourse("Nope".to_owned()))
        );
    }

    #[test]
    fn sheet_names_tie_rule_and_links_in_en_and_de() {
        let en = evaluation_sheet(&event(), "Score", &Lang::english()).unwrap();
        for needle in [
            "Evaluation sheet for Score",
            "Time limit: 45 minutes",
            "7 points for each started minute",
            "the faster runner wins",
            "does not run a link directly is disqualified",
            "| 33 | 41 |",
            "| 33 | 3 |",
        ] {
            assert!(en.contains(needle), "{needle}\n{en}");
        }
        let de_lang = crate::text::parse_lang_arg("de").unwrap();
        let de = evaluation_sheet(&event(), "Score", &de_lang).unwrap();
        for needle in [
            "Auswertungsbogen für Score",
            "Zeitlimit: 45 Minuten",
            "Bei gleicher Punktzahl gewinnt, wer schneller ist.",
            "Abzug: 7 Punkte",
            "wird disqualifiziert",
            "| 33 | 41 |",
        ] {
            assert!(de.contains(needle), "{needle}\n{de}");
        }
        let mut e = event();
        if let CourseKind::Score {
            penalty_per_min,
            links,
            ..
        } = &mut e.courses[0].kind
        {
            *penalty_per_min = None;
            links.clear();
        }
        let text = evaluation_sheet(&e, "Score", &Lang::english()).unwrap();
        assert!(
            text.contains(
                "5 points for each started minute over the limit. The course sets no penalty"
            ),
            "{text}"
        );
        assert!(text.contains("no mandatory links"));
    }

    #[test]
    fn sheet_refuses_missing_and_other_courses() {
        let en = Lang::english();
        assert_eq!(
            evaluation_sheet(&event(), "Nope", &en),
            Err(ScoreError::UnknownCourse("Nope".to_owned()))
        );
        assert_eq!(
            evaluation_sheet(&event(), "Indiv", &en),
            Err(ScoreError::NotAScoreCourse("Indiv".to_owned()))
        );
    }

    #[test]
    fn sheet_escapes_pipes_and_line_breaks() {
        let mut e = event();
        e.courses[0].name = "A|B\nC".to_owned();
        e.event.title = "x|y".to_owned();
        let sheet = evaluation_sheet(&e, "A|B\nC", &Lang::english()).unwrap();
        assert!(sheet.contains("# Evaluation sheet for A\\|B C"), "{sheet}");
        assert!(sheet.contains("Event: x\\|y"), "{sheet}");
    }

    #[test]
    fn a_link_to_a_control_outside_the_course_does_not_panic() {
        let mut e = event();
        if let CourseKind::Score { links, .. } = &mut e.courses[0].kind {
            links.push(ScoreLink {
                a: ControlCode("99".to_owned()),
                b: ControlCode("33".to_owned()),
            });
        }
        // The model refuses such a link on load; the writers keep what they are given.
        assert!(
            evaluation_sheet(&e, "Score", &Lang::english())
                .unwrap()
                .contains("| 99 | 33 |")
        );
        assert!(!score_csv(&e, "Score").unwrap().contains("99"));
    }
}
