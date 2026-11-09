//! `check`, `finding accept`/`remove` and `event profile`. `check` prints the findings grouped by
//! course and a summary; it exits with 1 when a finding that is not accepted is an error.

use std::path::PathBuf;

use clap::Subcommand;
use freemapper_courseplan::check::{CheckReport, Finding, Location, Skipped, check_event};
use freemapper_courseplan::edit::{self, Accepted};
use freemapper_courseplan::rules::{RuleSet, Severity, load_rule_set};
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::{AcceptedFinding, ControlCode, Event, LegRef, ProjectFile};

use crate::edit_cmd::{Failure, codes, load, read_map, save};

type Outcome = Result<(), Failure>;

// The help texts come from the catalog (see `edit_cmd::localize_groups`), not from doc comments.
#[derive(Subcommand)]
pub enum FindingCmd {
    /// Accept a finding as an intended exception
    Accept {
        project: PathBuf,
        rule: String,
        #[arg(long, value_name = "31,32", value_delimiter = ',')]
        controls: Vec<String>,
        #[arg(
            long,
            value_name = "COURSE:N",
            num_args = 1,
            action = clap::ArgAction::Append
        )]
        leg: Vec<String>,
        #[arg(long, value_name = "NAME")]
        course: Option<String>,
        #[arg(long)]
        note: String,
    },
    /// Remove an accepted finding
    Remove {
        project: PathBuf,
        rule: String,
        #[arg(long, value_name = "31,32", value_delimiter = ',')]
        controls: Vec<String>,
        #[arg(
            long,
            value_name = "COURSE:N",
            num_args = 1,
            action = clap::ArgAction::Append
        )]
        leg: Vec<String>,
        #[arg(long, value_name = "NAME")]
        course: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum EventCmd {
    /// Set the rule profile and its overlays
    Profile {
        project: PathBuf,
        base: String,
        #[arg(
            long,
            value_name = "ID",
            num_args = 1,
            action = clap::ArgAction::Append
        )]
        overlay: Vec<String>,
    },
}

fn v(value: impl Into<String>) -> FluentValue<'static> {
    FluentValue::from(value.into())
}

/// The rule set the event names.
pub fn rule_set(lang: &Lang, event: &Event) -> Result<RuleSet, Failure> {
    load_rule_set(&event.event.rule_profile, &event.event.rule_overlays)
        .map_err(|e| Failure::Error(e.localize(lang)))
}

/// `check <PROJECT> [--all]`.
pub fn check(lang: &Lang, project: PathBuf, all: bool) -> Outcome {
    let file = load(lang, &project)?;
    let rules = rule_set(lang, &file.event)?;
    let map = read_map(lang, &file)?;
    let report = check_event(&file.event, &map, &rules);
    for line in report_lines(lang, &file.event, &report, all) {
        println!("{line}");
    }
    exit_status(&report)
}

/// Exit code 1 (without a further message) when a finding that is not accepted is an error.
pub fn exit_status(report: &CheckReport) -> Outcome {
    if report.has_errors() {
        Err(Failure::Findings)
    } else {
        Ok(())
    }
}

fn severity_word(lang: &Lang, severity: Severity) -> String {
    let id = match severity {
        Severity::Error => "check-severity-error",
        Severity::Warning => "check-severity-warning",
        Severity::Hint => "check-severity-hint",
    };
    text(lang, id, &[])
}

/// `<severity> <location>: <text> (<rule id>, <doc> <section>)`.
fn finding_line(lang: &Lang, finding: &Finding) -> String {
    format!(
        "{} {}: {} ({}, {} {})",
        severity_word(lang, finding.severity),
        finding.location.text(lang),
        finding.text(lang),
        finding.rule,
        finding.source.doc,
        finding.source.section
    )
}

/// `<rule id>, <location>: <reason>`; a course that is not checked at all has no rule id.
fn skipped_line(lang: &Lang, skipped: &Skipped) -> String {
    let place = skipped.location.text(lang);
    match &skipped.rule {
        Some(rule) => format!("{rule}, {place}: {}", skipped.text(lang)),
        None => format!("{place}: {}", skipped.text(lang)),
    }
}

/// The groups of the output: the event first (findings at the event and at controls), then one
/// group per course in file order. Each group lists what `items` puts in it.
fn grouped<T>(
    lang: &Lang,
    event: &Event,
    items: &[T],
    location: impl Fn(&T) -> &Location,
    line: impl Fn(&T) -> String,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut group = |heading: String, of: &dyn Fn(Option<&str>) -> bool| {
        let lines: Vec<String> = items
            .iter()
            .filter(|i| of(location(i).course()))
            .map(|i| format!("  {}", line(i)))
            .collect();
        if !lines.is_empty() {
            out.push(heading);
            out.extend(lines);
        }
    };
    group(text(lang, "cli-check-group-event", &[]), &|c| c.is_none());
    for course in &event.courses {
        let name = course.name.as_str();
        group(
            text(lang, "cli-check-group-course", &[("course", v(name))]),
            &|c| c == Some(name),
        );
    }
    out
}

/// Every line `check` prints: findings by course, courses that are not checked, the summary;
/// with `all` also the accepted findings with their notes and the skipped rules.
pub fn report_lines(lang: &Lang, event: &Event, report: &CheckReport, all: bool) -> Vec<String> {
    let mut lines = grouped(
        lang,
        event,
        &report.findings,
        |f| &f.location,
        |f| finding_line(lang, f),
    );
    let unchecked: Vec<&Skipped> = report.skipped.iter().filter(|s| s.rule.is_none()).collect();
    for skipped in &unchecked {
        lines.push(skipped_line(lang, skipped));
    }
    if all {
        if !report.accepted.is_empty() {
            lines.push(text(lang, "cli-check-heading-accepted", &[]));
            for (finding, note) in &report.accepted {
                lines.push(format!("  {}", finding_line(lang, finding)));
                lines.push(format!(
                    "    {}",
                    text(lang, "cli-check-note", &[("note", v(note.as_str()))])
                ));
            }
        }
        let by_rule: Vec<&Skipped> = report.skipped.iter().filter(|s| s.rule.is_some()).collect();
        if !by_rule.is_empty() {
            lines.push(text(lang, "cli-check-heading-skipped", &[]));
            for skipped in by_rule {
                lines.push(format!("  {}", skipped_line(lang, skipped)));
            }
        }
    }
    let count = |n: usize| FluentValue::from(n);
    lines.push(text(
        lang,
        "cli-check-summary",
        &[
            ("errors", count(report.count(Severity::Error))),
            ("warnings", count(report.count(Severity::Warning))),
            ("hints", count(report.count(Severity::Hint))),
            ("accepted", count(report.accepted.len())),
            ("skipped", count(report.skipped.len())),
        ],
    ));
    if !all && (!report.accepted.is_empty() || unchecked.len() < report.skipped.len()) {
        lines.push(text(lang, "cli-check-show-all", &[]));
    }
    lines
}

/// `COURSE:N`, the leg after position N of a course (0 is the leg from the start).
fn parse_leg(lang: &Lang, value: &str) -> Result<LegRef, Failure> {
    let bad = || Failure::Usage(text(lang, "cli-finding-bad-leg", &[("value", v(value))]));
    let (course, after) = value.rsplit_once(':').ok_or_else(bad)?;
    let course = course.trim();
    let after = after.trim().parse::<u32>().map_err(|_| bad())?;
    if course.is_empty() {
        return Err(bad());
    }
    Ok(LegRef {
        course: course.to_owned(),
        after,
    })
}

/// The accepted finding the arguments name.
fn named(
    lang: &Lang,
    rule: String,
    controls: Vec<String>,
    leg: Vec<String>,
    course: Option<String>,
    note: String,
) -> Result<AcceptedFinding, Failure> {
    Ok(AcceptedFinding {
        rule,
        controls: codes(controls).into_iter().map(ControlCode).collect(),
        course: course.filter(|c| !c.is_empty()),
        legs: leg
            .iter()
            .map(|l| parse_leg(lang, l))
            .collect::<Result<_, _>>()?,
        note,
    })
}

/// Where an accepted finding sits, for the confirmation line.
fn place_text(lang: &Lang, finding: &AcceptedFinding) -> String {
    let location = if let Some(course) = &finding.course {
        Location::Course(course.clone())
    } else if let [leg] = finding.legs.as_slice() {
        Location::Leg {
            course: leg.course.clone(),
            after: leg.after,
        }
    } else if let Some(first) = finding.legs.first() {
        // `accept_finding` keeps several legs to one course.
        Location::Legs {
            course: first.course.clone(),
            afters: finding.legs.iter().map(|l| l.after).collect(),
        }
    } else if finding.controls.len() == 1 {
        Location::Control {
            code: finding.controls[0].clone(),
        }
    } else if !finding.controls.is_empty() {
        Location::Controls {
            codes: finding.controls.clone(),
        }
    } else {
        return text(lang, "cli-finding-at-event", &[]);
    };
    text(
        lang,
        "cli-finding-at",
        &[("location", v(location.text(lang)))],
    )
}

fn refused(lang: &Lang, e: edit::EditError) -> Failure {
    Failure::Error(e.localize(lang))
}

pub fn finding(lang: &Lang, cmd: FindingCmd) -> Outcome {
    match cmd {
        FindingCmd::Accept {
            project,
            rule,
            controls,
            leg,
            course,
            note,
        } => {
            let finding = named(lang, rule, controls, leg, course, note)?;
            let mut file: ProjectFile = load(lang, &project)?;
            let rules = rule_set(lang, &file.event)?;
            let key = finding.key();
            let done = edit::accept_finding(&mut file.event, &rules, finding)
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            let stored = file
                .event
                .accepted_findings
                .iter()
                .find(|a| a.key() == key)
                .expect("just accepted");
            let id = match done {
                Accepted::Added => "cli-finding-accepted",
                Accepted::NoteReplaced => "cli-finding-note-replaced",
            };
            say(
                lang,
                id,
                &[
                    ("rule", v(key.rule.as_str())),
                    ("location", v(place_text(lang, stored))),
                ],
            );
        }
        FindingCmd::Remove {
            project,
            rule,
            controls,
            leg,
            course,
        } => {
            let finding = named(lang, rule, controls, leg, course, String::new())?;
            let mut file = load(lang, &project)?;
            edit::remove_accepted_finding(&mut file.event, &finding.key())
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(
                lang,
                "cli-finding-removed",
                &[
                    ("rule", v(finding.rule.as_str())),
                    ("location", v(place_text(lang, &finding))),
                ],
            );
        }
    }
    Ok(())
}

pub fn event(lang: &Lang, cmd: EventCmd) -> Outcome {
    match cmd {
        EventCmd::Profile {
            project,
            base,
            overlay,
        } => {
            let overlays = codes(overlay);
            let mut file = load(lang, &project)?;
            let set = edit::set_rule_profile(&mut file.event, &base, &overlays)
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            if set.overlays.is_empty() {
                say(lang, "cli-event-profile-set", &[("value", v(set.base))]);
            } else {
                say(
                    lang,
                    "cli-event-profile-set-overlays",
                    &[("value", v(set.base)), ("list", v(set.overlays.join(", ")))],
                );
            }
        }
    }
    Ok(())
}

fn say(lang: &Lang, id: &str, args: &[(&str, FluentValue)]) {
    println!("{}", text(lang, id, args));
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_courseplan::rules::SourceRef;
    use freemapper_courseplan::text::available;

    fn finding(severity: Severity, location: Location) -> Finding {
        Finding {
            rule: "wkb2026.distance.min".to_owned(),
            profile: "wkb-2026".to_owned(),
            severity,
            location,
            message_id: "check-skip-not-implemented",
            args: vec![],
            source: SourceRef {
                doc: "WKB26".to_owned(),
                section: "B 7.2.1".to_owned(),
            },
        }
    }

    fn controls(list: &[&str]) -> Location {
        Location::Controls {
            codes: list.iter().map(|c| ControlCode((*c).to_owned())).collect(),
        }
    }

    #[test]
    fn exit_code_follows_the_errors_that_are_not_accepted() {
        let error = finding(Severity::Error, controls(&["31", "32"]));
        let warning = finding(Severity::Warning, Location::Event);
        let only_warnings = CheckReport {
            findings: vec![warning.clone()],
            ..CheckReport::default()
        };
        assert!(exit_status(&only_warnings).is_ok());
        let accepted_error = CheckReport {
            findings: vec![warning],
            accepted: vec![(error.clone(), "on purpose".to_owned())],
            skipped: vec![],
        };
        assert!(exit_status(&accepted_error).is_ok());
        let with_error = CheckReport {
            findings: vec![error],
            ..CheckReport::default()
        };
        assert!(matches!(exit_status(&with_error), Err(Failure::Findings)));
    }

    #[test]
    fn findings_are_grouped_by_course() {
        let lang = Lang::english();
        let mut event: Event =
            toml::from_str("format = 1\n[event]\ntitle = \"T\"\n[sources]\nmap = \"m.omap\"\n")
                .unwrap();
        for name in ["A", "B"] {
            event.courses.push(
                toml::from_str(&format!(
                    "name = \"{name}\"\nstart = \"S1\"\nfinish = \"F1\"\nkind = \"individual\"\ncontrols = []\n"
                ))
                .unwrap(),
            );
        }
        let report = CheckReport {
            findings: vec![
                finding(
                    Severity::Hint,
                    Location::Leg {
                        course: "B".to_owned(),
                        after: 2,
                    },
                ),
                finding(Severity::Error, controls(&["31", "32"])),
                finding(Severity::Warning, Location::Course("A".to_owned())),
            ],
            accepted: vec![],
            skipped: vec![],
        };
        let lines = report_lines(&lang, &event, &report, false);
        assert_eq!(
            lines,
            [
                "event:",
                "  error controls 31, 32: this check is not implemented yet \
                 (wkb2026.distance.min, WKB26 B 7.2.1)",
                "course A:",
                "  warning course A: this check is not implemented yet \
                 (wkb2026.distance.min, WKB26 B 7.2.1)",
                "course B:",
                "  hint course B, leg after 2: this check is not implemented yet \
                 (wkb2026.distance.min, WKB26 B 7.2.1)",
                "1 error, 1 warning, 1 hint; 0 accepted, 0 skipped",
            ]
        );
        let de = available().into_iter().find(|l| l.code() == "de").unwrap();
        let german = report_lines(&de, &event, &report, true);
        assert_eq!(german[0], "Veranstaltung:");
        assert!(
            german[1].starts_with("  Fehler Posten 31, 32: "),
            "{german:?}"
        );
        assert_eq!(
            german.last().unwrap(),
            "1 Fehler, 1 Warnung, 1 Hinweis; 0 akzeptiert, 0 übersprungen"
        );
    }

    #[test]
    fn legs_are_written_course_colon_position() {
        let lang = Lang::english();
        let leg = parse_leg(&lang, "A:3").unwrap();
        assert_eq!((leg.course.as_str(), leg.after), ("A", 3));
        let colon_in_name = parse_leg(&lang, "Lang: Ost:0").unwrap();
        assert_eq!(
            (colon_in_name.course.as_str(), colon_in_name.after),
            ("Lang: Ost", 0)
        );
        for bad in ["A", "A:", ":3", "A:-1", "A:x"] {
            assert!(
                matches!(parse_leg(&lang, bad), Err(Failure::Usage(_))),
                "{bad}"
            );
        }
    }
}
