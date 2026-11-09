//! `check`, `finding accept`/`remove`, `event profile` and `event start-window`. `check` prints
//! the findings grouped by course and a summary; it exits with 1 when a finding that is not
//! accepted is an error.

use std::path::PathBuf;

use clap::Subcommand;
use freemapper_courseplan::check::{
    CheckReport, Finding, Location, Skipped, check_event, source_text,
};
use freemapper_courseplan::edit::{self, Accepted};
use freemapper_courseplan::rules::{RuleSet, Severity, load_rule_set};
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::{AcceptedFinding, ControlCode, Event, LegRef, ProjectFile};

use crate::edit_cmd::parse_count;

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
    /// Set the start window
    StartWindow { project: PathBuf, minutes: String },
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
    for line in report_lines(lang, &file.event, &rules, &report, all) {
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

/// `<severity> <location>: <text> (<rule id>, <doc> <section>)`; a value this program proposes
/// names its source as such (ruling R27).
fn finding_line(lang: &Lang, event: &Event, finding: &Finding) -> String {
    format!(
        "{} {}: {} ({}, {})",
        severity_word(lang, finding.severity),
        finding.location.text_in(lang, event),
        finding.text(lang),
        finding.rule,
        source_text(lang, &finding.source)
    )
}

/// The finding line and, with `all`, the note of its rule in `lang` below it, indented like the
/// note of an accepted finding.
fn finding_lines(
    lang: &Lang,
    event: &Event,
    rules: &RuleSet,
    finding: &Finding,
    all: bool,
) -> Vec<String> {
    let mut lines = vec![finding_line(lang, event, finding)];
    let note = rules
        .rule(&finding.rule)
        .and_then(|r| r.rule.note.as_ref())
        .map(|n| n.get(&lang.code()).to_owned());
    if let Some(note) = note.filter(|n| all && !n.is_empty()) {
        lines.push(format!(
            "  {}",
            text(lang, "cli-check-rule-note", &[("note", v(note))])
        ));
    }
    lines
}

/// `<rule id>, <location>: <reason>`; a course that is not checked at all has no rule id.
fn skipped_line(lang: &Lang, event: &Event, skipped: &Skipped) -> String {
    let place = skipped.location.text_in(lang, event);
    match &skipped.rule {
        Some(rule) => format!("{rule}, {place}: {}", skipped.text(lang)),
        None => format!("{place}: {}", skipped.text(lang)),
    }
}

/// Where a location sorts inside its group: the event before its controls; in a course the
/// course itself, then its legs by position.
fn order_in_group(location: &Location) -> u32 {
    match location {
        Location::Event | Location::Course(_) => 0,
        Location::Control { .. } | Location::Controls { .. } => 1,
        Location::Leg { leg, .. } => leg.after + 1,
        Location::Legs { legs, .. } => legs.first().map_or(0, |l| l.after) + 1,
    }
}

/// The groups of the output: the event first (findings at the event and at controls), then one
/// group per course in file order. Each group lists what `items` puts in it, the course before
/// its legs and the legs in running order; items at the same place keep rule order.
fn grouped<T>(
    lang: &Lang,
    event: &Event,
    items: &[T],
    location: impl Fn(&T) -> &Location,
    lines_of: impl Fn(&T) -> Vec<String>,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut group = |heading: String, of: &dyn Fn(Option<&str>) -> bool| {
        let mut members: Vec<&T> = items.iter().filter(|i| of(location(i).course())).collect();
        members.sort_by_key(|i| order_in_group(location(i)));
        let lines: Vec<String> = members
            .into_iter()
            .flat_map(&lines_of)
            .map(|l| format!("  {l}"))
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
/// with `all` also the note of each finding's rule, the accepted findings with their notes, the
/// acceptances that match no finding (ruling R25) and the skipped rules.
pub fn report_lines(
    lang: &Lang,
    event: &Event,
    rules: &RuleSet,
    report: &CheckReport,
    all: bool,
) -> Vec<String> {
    let mut lines = grouped(
        lang,
        event,
        &report.findings,
        |f| &f.location,
        |f| finding_lines(lang, event, rules, f, all),
    );
    let unchecked: Vec<&Skipped> = report.skipped.iter().filter(|s| s.rule.is_none()).collect();
    for skipped in &unchecked {
        lines.push(skipped_line(lang, event, skipped));
    }
    let note_line =
        |note: &str| format!("    {}", text(lang, "cli-check-note", &[("note", v(note))]));
    if all {
        if !report.accepted.is_empty() {
            lines.push(text(lang, "cli-check-heading-accepted", &[]));
            for (finding, note) in &report.accepted {
                let mut shown = finding_lines(lang, event, rules, finding, all).into_iter();
                lines.extend(shown.next().map(|l| format!("  {l}")));
                lines.push(note_line(note));
                lines.extend(shown.map(|l| format!("  {l}")));
            }
        }
        if !report.unmatched.is_empty() {
            lines.push(text(lang, "cli-check-heading-unmatched", &[]));
            for accepted in &report.unmatched {
                let place = Location::of_accepted(event, accepted).text_in(lang, event);
                lines.push(format!("  {}, {place}", accepted.rule));
                lines.push(note_line(&accepted.note));
            }
            lines.push(format!("  {}", text(lang, "cli-check-unmatched-hint", &[])));
        }
        let by_rule: Vec<&Skipped> = report.skipped.iter().filter(|s| s.rule.is_some()).collect();
        if !by_rule.is_empty() {
            lines.push(text(lang, "cli-check-heading-skipped", &[]));
            for skipped in by_rule {
                lines.push(format!("  {}", skipped_line(lang, event, skipped)));
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
            ("unmatched", count(report.unmatched.len())),
            ("skipped", count(report.skipped.len())),
        ],
    ));
    let hidden = !report.accepted.is_empty()
        || !report.unmatched.is_empty()
        || unchecked.len() < report.skipped.len();
    if !all && hidden {
        lines.push(text(lang, "cli-check-show-all", &[]));
    }
    lines
}

/// `COURSE:N`, the leg that leaves position N of a course's stop list (0 is the start). Its ends
/// are left empty; `accept_finding` fills them from the course.
fn parse_leg(lang: &Lang, value: &str) -> Result<LegRef, Failure> {
    let bad = || Failure::Usage(text(lang, "cli-finding-bad-leg", &[("value", v(value))]));
    let (course, after) = value.rsplit_once(':').ok_or_else(bad)?;
    let course = course.trim();
    let after = after.trim().parse::<u32>().map_err(|_| bad())?;
    if course.is_empty() {
        return Err(bad());
    }
    Ok(LegRef::at(course, after))
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

/// Where an accepted finding sits, for the confirmation line, with the leg ends it stores.
fn place_text(lang: &Lang, event: &Event, finding: &AcceptedFinding) -> String {
    match Location::of_accepted(event, finding) {
        Location::Event => text(lang, "cli-finding-at-event", &[]),
        location => text(
            lang,
            "cli-finding-at",
            &[("location", v(location.text_in(lang, event)))],
        ),
    }
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
            // The stored key has the leg ends `accept_finding` filled in.
            let stored = file
                .event
                .accepted_findings
                .iter()
                .find(|a| a.key().by_position() == key.by_position())
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
                    ("location", v(place_text(lang, &file.event, stored))),
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
            let removed = edit::remove_accepted_finding(&mut file.event, &finding.key())
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            for gone in &removed {
                say(
                    lang,
                    "cli-finding-removed",
                    &[
                        ("rule", v(gone.rule.as_str())),
                        ("location", v(place_text(lang, &file.event, gone))),
                    ],
                );
            }
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
                    &[
                        ("value", v(set.base)),
                        ("list", v(set.overlays.join(", "))),
                        ("count", FluentValue::from(set.overlays.len())),
                    ],
                );
            }
        }
        EventCmd::StartWindow { project, minutes } => {
            let minutes = parse_count(lang, &minutes)?;
            let mut file = load(lang, &project)?;
            edit::set_start_window(&mut file.event, minutes).map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(
                lang,
                "cli-event-start-window-set",
                &[("minutes", FluentValue::from(minutes))],
            );
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
    use freemapper_courseplan::check::{LegEnd, LegSpan};
    use freemapper_courseplan::rules::SourceRef;
    use freemapper_courseplan::text::available;

    fn finding(severity: Severity, location: Location) -> Finding {
        Finding {
            rule: "wkb2026.distance.min".to_owned(),
            profile: "wkb-2026".to_owned(),
            severity,
            location,
            message_id: "check-skip-climb-missing",
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
            ..CheckReport::default()
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
        let leg = |course: &str, after, from: &str, to: LegEnd| Location::Leg {
            course: course.to_owned(),
            leg: LegSpan {
                after,
                from: LegEnd::Control(from.to_owned()),
                to,
            },
        };
        let report = CheckReport {
            findings: vec![
                finding(
                    Severity::Hint,
                    leg("B", 2, "32", LegEnd::Finish("F1".to_owned())),
                ),
                finding(
                    Severity::Hint,
                    leg("A", 3, "33", LegEnd::Control("34".to_owned())),
                ),
                finding(
                    Severity::Hint,
                    leg("A", 1, "31", LegEnd::Control("32".to_owned())),
                ),
                finding(Severity::Error, controls(&["31", "32"])),
                finding(Severity::Warning, Location::Course("A".to_owned())),
            ],
            ..CheckReport::default()
        };
        let rules = load_rule_set("wkb-2026", &[]).unwrap();
        let lines = report_lines(&lang, &event, &rules, &report, false);
        let tail = "the course has no climb (wkb2026.distance.min, WKB26 B 7.2.1)";
        assert_eq!(
            lines,
            [
                "event:".to_owned(),
                format!("  error controls 31, 32: {tail}"),
                "course A:".to_owned(),
                format!("  warning course A: {tail}"),
                format!("  hint course A, leg 31–32: {tail}"),
                format!("  hint course A, leg 33–34: {tail}"),
                "course B:".to_owned(),
                format!("  hint course B, leg 32–finish: {tail}"),
                "1 error, 1 warning, 3 hints; 0 accepted, 0 skipped".to_owned(),
            ]
        );
        let de = available().into_iter().find(|l| l.code() == "de").unwrap();
        let german = report_lines(&de, &event, &rules, &report, true);
        assert_eq!(german[0], "Veranstaltung:");
        assert!(
            german[1].starts_with("  Fehler Posten 31, 32: "),
            "{german:?}"
        );
        assert_eq!(
            german.last().unwrap(),
            "1 Fehler, 1 Warnung, 3 Hinweise; 0 akzeptiert, 0 übersprungen"
        );
    }

    #[test]
    fn all_shows_rule_notes_unmatched_acceptances_and_proposals() {
        let de = available().into_iter().find(|l| l.code() == "de").unwrap();
        let mut event: Event = toml::from_str(
            "format = 1\n[event]\ntitle = \"T\"\n[sources]\nmap = \"m.omap\"\n\
             [[specials]]\nkind = \"start\"\nid = \"S1\"\nmap_mm = [0.0, 0.0]\n",
        )
        .unwrap();
        event.courses.push(
            toml::from_str(
                "name = \"A\"\nstart = \"S1\"\nfinish = \"F1\"\nkind = \"individual\"\n\
                 controls = []\n",
            )
            .unwrap(),
        );
        let rules = load_rule_set("wkb-2026", &[]).unwrap();
        // A rule with a note whose value is a proposal of this program.
        let proposed = rules
            .rules
            .iter()
            .find(|r| r.rule.source.doc == "claude" && r.rule.note.is_some())
            .expect("a proposed rule with a note");
        let found = Finding {
            rule: proposed.rule.id.clone(),
            source: proposed.rule.source.clone(),
            ..finding(
                Severity::Hint,
                Location::Control {
                    code: ControlCode("S1".to_owned()),
                },
            )
        };
        let gone = AcceptedFinding {
            rule: "wkb2026.distance.min".to_owned(),
            controls: vec![ControlCode("31".to_owned()), ControlCode("39".to_owned())],
            course: None,
            legs: vec![],
            note: "Absicht".to_owned(),
        };
        let report = CheckReport {
            findings: vec![found],
            unmatched: vec![gone],
            ..CheckReport::default()
        };
        let lines = report_lines(&de, &event, &rules, &report, true);
        let note = proposed.rule.note.as_ref().unwrap().get("de").to_owned();
        assert_eq!(
            lines[1],
            format!(
                "  Hinweis Start S1: Für die Bahn ist kein Anstieg angegeben ({}, Vorschlag von \
                 freemapper-courseplan)",
                proposed.rule.id
            )
        );
        assert_eq!(lines[2], format!("    Zur Prüfregel: {note}"));
        assert_eq!(
            &lines[3..7],
            [
                "nicht zugeordnet:",
                "  wkb2026.distance.min, Posten 31, 39",
                "    Begründung: Absicht",
                "  Diese Ausnahmen passen zu keinem Befund; finding remove entfernt eine, die nicht \
                 mehr gebraucht wird",
            ]
        );
        assert_eq!(
            lines.last().unwrap(),
            "0 Fehler, 0 Warnungen, 1 Hinweis; 0 akzeptiert, 1 nicht zugeordnet, 0 übersprungen"
        );
        // Without --all the notes and the unmatched list stay hidden, with the hint to see them.
        let short = report_lines(&de, &event, &rules, &report, false);
        assert_eq!(short.len(), 4, "{short:?}");
        assert!(short[3].starts_with("check --all"), "{short:?}");
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
