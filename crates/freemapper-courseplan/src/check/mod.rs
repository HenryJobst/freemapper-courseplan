//! The check engine: runs the rules of a [`RuleSet`] on the individual courses of an event and
//! collects [`Finding`]s, accepted findings and [`Skipped`] rules (design §5.6, ADR 0025).
//!
//! Each rule names a `key`; [`CHECKS`] maps every key of [`KNOWN_KEYS`](crate::rules::KNOWN_KEYS)
//! to the function that implements it, one line per key. The engine applies the rule filters
//! (discipline, map kind, print scale) before it calls the function, so a check only sees the
//! courses its rule applies to. A key whose rules apply to nothing is reported as skipped with
//! the reason, never as a silent pass (ruling R4 of plan M2b).
//!
//! Findings and skip reasons are plain data with a Fluent message id and arguments; the CLI and
//! the MCP server localize them. Terms follow `docs/glossary.md`.

mod codes;
mod context;
mod distance;
mod geometry;
mod legs;
mod numbers;
mod start;
mod stops;
mod tables;

pub use context::{CheckContext, CourseContext};

use freemapper_courseplan_model::{
    AcceptedFinding, ControlCode, CourseKind, Discipline, Event, FindingKey, LegRef, SpecialObject,
};

use crate::omap::MapDocument;
use crate::rules::{PROPOSAL_DOC, Rule, RuleSet, SetRule, Severity, SourceRef};
use crate::text::{FluentValue, Lang, text};

/// Where a finding sits.
#[derive(Debug, Clone, PartialEq)]
pub enum Location {
    /// The event as a whole.
    Event,
    /// A course, by name.
    Course(String),
    /// One control; controls are shared by the courses that use them.
    Control { code: ControlCode },
    /// Several controls, for example two that stand too close.
    Controls { codes: Vec<ControlCode> },
    /// One leg of `course`, by the position it leaves and its end stops.
    Leg { course: String, leg: LegSpan },
    /// Two or more legs of one course, by the positions they leave in ascending order, for
    /// example two legs that run against each other.
    Legs { course: String, legs: Vec<LegSpan> },
}

/// One end of a leg: the start, a control or crossing point, or the finish, by id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegEnd {
    Start(String),
    /// A control or a crossing point, by code.
    Control(String),
    Finish(String),
}

impl LegEnd {
    /// The id of the stop: the control code, or the id of the start or finish.
    pub fn id(&self) -> &str {
        match self {
            LegEnd::Start(id) | LegEnd::Control(id) | LegEnd::Finish(id) => id,
        }
    }

    /// The end with this id in `event`: a start or a finish special object, otherwise a control.
    pub fn of_id(event: &Event, id: &str) -> LegEnd {
        let special = event.specials.iter().find(|s| s.id() == id);
        match special {
            Some(SpecialObject::Start { .. }) => LegEnd::Start(id.to_owned()),
            Some(SpecialObject::Finish { .. }) => LegEnd::Finish(id.to_owned()),
            _ => LegEnd::Control(id.to_owned()),
        }
    }

    /// Start and finish by their role, a control by its code (ruling R28 of plan M2b).
    fn text(&self, lang: &Lang) -> String {
        match self {
            LegEnd::Start(_) => text(lang, "check-leg-end-start", &[]),
            LegEnd::Finish(_) => text(lang, "check-leg-end-finish", &[]),
            LegEnd::Control(code) if code.is_empty() => "?".to_owned(),
            LegEnd::Control(code) => code.clone(),
        }
    }
}

/// A leg in a [`Location`]: the position of the course's stop list it leaves (from 1 as in the
/// CLI, crossing points included; 0 is the leg from the start) and the stops at its ends.
#[derive(Debug, Clone, PartialEq)]
pub struct LegSpan {
    pub after: u32,
    pub from: LegEnd,
    pub to: LegEnd,
}

impl LegSpan {
    /// The leg as output names it, by its ends: "36–37", "start–31", "37–finish".
    pub fn text(&self, lang: &Lang) -> String {
        format!("{}–{}", self.from.text(lang), self.to.text(lang))
    }

    /// The leg reference of an accepted finding on this leg of `course`.
    pub fn leg_ref(&self, course: &str) -> LegRef {
        LegRef {
            course: course.to_owned(),
            after: self.after,
            from: self.from.id().to_owned(),
            to: self.to.id().to_owned(),
        }
    }

    /// The leg a leg reference names in `event`, with the ends it stores.
    pub fn of_ref(event: &Event, leg: &LegRef) -> LegSpan {
        LegSpan {
            after: leg.after,
            from: LegEnd::of_id(event, &leg.from),
            to: LegEnd::of_id(event, &leg.to),
        }
    }
}

/// The kind of a [`Location`]; [`Check::places`] lists the kinds a rule key reports at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Event,
    Course,
    /// Exactly one control, start or finish.
    Control,
    /// Two or more controls, starts or finishes.
    Controls,
    Leg,
    /// Two or more legs of one course.
    Legs,
}

impl Place {
    /// Every place kind, in declaration order.
    pub const ALL: [Place; 6] = [
        Place::Event,
        Place::Course,
        Place::Control,
        Place::Controls,
        Place::Leg,
        Place::Legs,
    ];

    /// The name in English messages and the suffix of its `word-place-*` text.
    pub fn as_str(self) -> &'static str {
        match self {
            Place::Event => "event",
            Place::Course => "course",
            Place::Control => "control",
            Place::Controls => "controls",
            Place::Leg => "leg",
            Place::Legs => "legs",
        }
    }

    /// The place kind of an accepted finding's key: what it names, or the event when it names
    /// nothing.
    pub fn of_key(key: &FindingKey) -> Place {
        if key.course.is_some() {
            Place::Course
        } else if key.legs.len() > 1 {
            Place::Legs
        } else if !key.legs.is_empty() {
            Place::Leg
        } else {
            match key.controls.len() {
                0 => Place::Event,
                1 => Place::Control,
                _ => Place::Controls,
            }
        }
    }
}

impl Location {
    /// The kind of the location.
    pub fn place(&self) -> Place {
        match self {
            Location::Event => Place::Event,
            Location::Course(_) => Place::Course,
            Location::Control { .. } => Place::Control,
            Location::Controls { .. } => Place::Controls,
            Location::Leg { .. } => Place::Leg,
            Location::Legs { .. } => Place::Legs,
        }
    }

    /// The course the location belongs to, for grouping the output.
    pub fn course(&self) -> Option<&str> {
        match self {
            Location::Course(name)
            | Location::Leg { course: name, .. }
            | Location::Legs { course: name, .. } => Some(name),
            _ => None,
        }
    }

    /// Where an accepted finding sits in `event`: the place its key names, with the leg ends it
    /// stores.
    pub fn of_accepted(event: &Event, accepted: &AcceptedFinding) -> Location {
        if let Some(course) = &accepted.course {
            return Location::Course(course.clone());
        }
        if let Some(first) = accepted.legs.first() {
            let legs: Vec<LegSpan> = accepted
                .legs
                .iter()
                .map(|l| LegSpan::of_ref(event, l))
                .collect();
            return if legs.len() == 1 {
                Location::Leg {
                    course: first.course.clone(),
                    leg: legs.into_iter().next().expect("one leg"),
                }
            } else {
                Location::Legs {
                    course: first.course.clone(),
                    legs,
                }
            };
        }
        match accepted.controls.as_slice() {
            [] => Location::Event,
            [code] => Location::Control { code: code.clone() },
            codes => Location::Controls {
                codes: codes.to_vec(),
            },
        }
    }

    /// The location as text with starts and finishes named by their role, as the CLI and the MCP
    /// server show it: "start S1" instead of "control S1".
    pub fn text_in(&self, lang: &Lang, event: &Event) -> String {
        let one = |code: &ControlCode| match LegEnd::of_id(event, &code.0) {
            LegEnd::Start(id) => text(lang, "check-location-start", &[("id", v(&id))]),
            LegEnd::Finish(id) => text(lang, "check-location-finish", &[("id", v(&id))]),
            LegEnd::Control(code) => text(lang, "check-location-control", &[("code", v(&code))]),
        };
        let is_stop =
            |code: &ControlCode| !matches!(LegEnd::of_id(event, &code.0), LegEnd::Control(_));
        match self {
            Location::Control { code } => one(code),
            Location::Controls { codes } if codes.iter().any(is_stop) => {
                codes.iter().map(one).collect::<Vec<_>>().join(", ")
            }
            _ => self.text(lang),
        }
    }

    /// The location as text, for example "control 31", "course A, leg 36–37" or
    /// "course A, legs 31–32 and 35–36".
    pub fn text(&self, lang: &Lang) -> String {
        match self {
            Location::Event => text(lang, "check-location-event", &[]),
            Location::Course(name) => text(lang, "check-location-course", &[("course", v(name))]),
            Location::Control { code } => {
                text(lang, "check-location-control", &[("code", v(&code.0))])
            }
            Location::Controls { codes } => {
                let list = codes.iter().map(|c| c.0.as_str()).collect::<Vec<_>>();
                text(
                    lang,
                    "check-location-controls",
                    &[("codes", v(&list.join(", ")))],
                )
            }
            Location::Leg { course, leg } => text(
                lang,
                "check-location-leg",
                &[("course", v(course)), ("leg", v(&leg.text(lang)))],
            ),
            Location::Legs { course, legs } => {
                let names: Vec<String> = legs.iter().map(|l| l.text(lang)).collect();
                text(
                    lang,
                    "check-location-legs",
                    &[("course", v(course)), ("legs", v(&and_list(lang, &names)))],
                )
            }
        }
    }
}

fn v(s: &str) -> FluentValue<'static> {
    FluentValue::from(s.to_owned())
}

/// The source of a rule as output shows it: document key and section, such as "WKB26 B 7.2.1",
/// or for a value this program proposes (`doc = "claude"`) a localized "proposal of
/// freemapper-courseplan" without the plan section (ruling R27).
pub fn source_text(lang: &Lang, source: &SourceRef) -> String {
    if source.doc == PROPOSAL_DOC {
        text(lang, "check-source-claude", &[])
    } else {
        format!("{} {}", source.doc, source.section)
    }
}

/// "a", "a and b", "a, b and c".
pub fn and_list(lang: &Lang, items: &[String]) -> String {
    match items.split_last() {
        Some((last, rest)) if !rest.is_empty() => {
            format!("{} {} {last}", rest.join(", "), text(lang, "word-and", &[]))
        }
        _ => items.concat(),
    }
}

/// A value inside a finding text.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    Text(String),
    /// A measured value; shown with at most one decimal in the language's decimal mark.
    Number(f64),
    /// A value shown with at most this many decimals, such as performance kilometres.
    Decimals(f64, u8),
    Int(i64),
    /// Several items of one message, each rendered from its own message id and arguments and
    /// joined with "; ", such as the table rows a course violates.
    List(&'static str, Vec<Args>),
}

/// The arguments of a finding or skip text: Fluent variable name and value.
pub type Args = Vec<(&'static str, ArgValue)>;

impl ArgValue {
    fn fluent(&self, lang: &Lang) -> FluentValue<'static> {
        match self {
            ArgValue::Text(s) => FluentValue::from(s.clone()),
            ArgValue::Int(n) => FluentValue::from(*n),
            ArgValue::Number(x) => FluentValue::from(format_number(lang, *x)),
            ArgValue::Decimals(x, places) => FluentValue::from(format_decimals(lang, *x, *places)),
            ArgValue::List(id, items) => {
                let shown: Vec<String> =
                    items.iter().map(|args| localize(lang, id, args)).collect();
                FluentValue::from(shown.join("; "))
            }
        }
    }
}

/// At most one decimal (a trailing `.0` is dropped: 60 not 60.0), with the decimal mark of the
/// language. Fluent's `NUMBER()` does not localize separators in fluent-bundle 0.16.
fn format_number(lang: &Lang, x: f64) -> String {
    format_decimals(lang, x, 1)
}

/// At most `places` decimals, trailing zeros dropped (4.04, 4.1, 4), with the decimal mark of the
/// language.
fn format_decimals(lang: &Lang, x: f64, places: u8) -> String {
    let factor = 10f64.powi(i32::from(places));
    let rounded = (x * factor).round() / factor;
    let shown = format!("{rounded:.*}", usize::from(places));
    let shown = if shown.contains('.') {
        shown.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        shown
    };
    let shown = if shown == "-0" { "0".to_owned() } else { shown };
    shown.replace('.', &text(lang, "word-decimal-point", &[]))
}

fn localize(lang: &Lang, id: &str, args: &Args) -> String {
    let args: Vec<(&str, FluentValue)> = args
        .iter()
        .map(|(name, value)| (*name, value.fluent(lang)))
        .collect();
    text(lang, id, &args)
}

/// The result of a rule at one place.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// Id of the rule, such as `wkb2026.distance.min`.
    pub rule: String,
    /// The profile the rule came from; an overlay rule names the overlay.
    pub profile: String,
    pub severity: Severity,
    pub location: Location,
    /// Fluent id of the explanation.
    pub message_id: &'static str,
    pub args: Args,
    pub source: SourceRef,
}

impl Finding {
    /// The key an [`AcceptedFinding`] must have to accept this finding. It is built by
    /// [`AcceptedFinding::key`], so both sides share one normalization.
    pub fn key(&self) -> FindingKey {
        let mut accepted = AcceptedFinding {
            rule: self.rule.clone(),
            controls: Vec::new(),
            course: None,
            legs: Vec::new(),
            note: String::new(),
        };
        match &self.location {
            Location::Event => {}
            Location::Course(name) => accepted.course = Some(name.clone()),
            Location::Control { code } => accepted.controls = vec![code.clone()],
            Location::Controls { codes } => accepted.controls = codes.clone(),
            Location::Leg { course, leg } => accepted.legs = vec![leg.leg_ref(course)],
            Location::Legs { course, legs } => {
                accepted.legs = legs.iter().map(|l| l.leg_ref(course)).collect()
            }
        }
        accepted.key()
    }

    /// The explanation in `lang`.
    pub fn text(&self, lang: &Lang) -> String {
        localize(lang, self.message_id, &self.args)
    }
}

/// A rule, or a whole course, that was not checked, with the reason.
#[derive(Debug, Clone, PartialEq)]
pub struct Skipped {
    /// The rule id; `None` when a course is not checked at all (its kind is not individual).
    pub rule: Option<String>,
    /// The profile the rule came from.
    pub profile: Option<String>,
    pub location: Location,
    /// Fluent id of the reason, one of the `SKIP_*` constants or a check's own.
    pub reason_id: &'static str,
    pub args: Args,
}

impl Skipped {
    /// The reason in `lang`.
    pub fn text(&self, lang: &Lang) -> String {
        localize(lang, self.reason_id, &self.args)
    }
}

/// Reason: the event is a training, and the rule is for other disciplines.
pub const SKIP_TRAINING: &str = "check-skip-training";
/// Reason: the rule is for other disciplines; argument `discipline`.
pub const SKIP_DISCIPLINE: &str = "check-skip-discipline";
/// Reason: the rule is for other map kinds; argument `kind`.
pub const SKIP_MAP_KIND: &str = "check-skip-map-kind";
/// Reason: no rule of this key covers the course's print scale; argument `scale`.
pub const SKIP_PRINT_SCALE: &str = "check-skip-print-scale";
/// Reason: a course of another kind than individual; argument `kind`.
pub const SKIP_COURSE_KIND: &str = "check-skip-course-kind";
/// Reason: the course names a start, finish or control the event lacks; argument `id`.
pub const SKIP_COURSE_UNRESOLVED: &str = "check-skip-course-unresolved";
/// Reason: a class of the course has no rule category; argument `name`.
pub const SKIP_CLASS_NOT_MAPPED: &str = "check-skip-class-not-mapped";
/// Reason: the course has no climb.
pub const SKIP_CLIMB_MISSING: &str = "check-skip-climb-missing";
/// Reason: the rule lacks a value its check needs; argument `name`.
pub const SKIP_VALUE_MISSING: &str = "check-skip-value-missing";
/// Reason: the event has no start window, so the starters per minute cannot be computed.
pub const SKIP_START_WINDOW_MISSING: &str = "check-skip-start-window-missing";
/// Reason: a class at the start has no expected number of starters; arguments `start`, `classes`.
pub const SKIP_STARTERS_MISSING: &str = "check-skip-starters-missing";
/// Reason: the map has no features, so it has no extent.
pub const SKIP_NO_MAP_EXTENT: &str = "check-skip-no-map-extent";
/// Reason: a profile switched the rule off (`enabled = false`); argument `overlay`, the id of
/// that profile (ruling R27).
pub const SKIP_SWITCHED_OFF: &str = "check-skip-switched-off";

/// Whether the symbol code of a map feature matches a code of a rule: equal, or a sub-code of it
/// (`505.1` matches `505`; `5051` does not). Ruling R9 of plan M2b.
pub fn symbol_matches(map_code: &str, rule_code: &str) -> bool {
    map_code
        .strip_prefix(rule_code)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}

/// What `check` found.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CheckReport {
    /// Findings not accepted, in rule order.
    pub findings: Vec<Finding>,
    /// Findings an accepted finding of the event matches, with its note.
    pub accepted: Vec<(Finding, String)>,
    /// Accepted findings of the event that match no finding of this run, in file order: their
    /// control, leg or course changed or is gone, or their rule did not run (ruling R25). They
    /// never count as errors.
    pub unmatched: Vec<AcceptedFinding>,
    pub skipped: Vec<Skipped>,
}

impl CheckReport {
    /// The number of not accepted findings of this severity.
    pub fn count(&self, severity: Severity) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == severity)
            .count()
    }

    /// Whether a not accepted finding has severity `error`; `check` then exits with 1.
    pub fn has_errors(&self) -> bool {
        self.count(Severity::Error) > 0
    }
}

/// One rule as a check function sees it: the rule, the courses it applies to and the other
/// rules of its key.
pub struct RuleRun<'r> {
    pub set_rule: &'r SetRule,
    /// The individual courses whose print scale the rule covers, in file order.
    pub courses: Vec<&'r CourseContext<'r>>,
    /// The other rules of the same key that apply to the event (discipline, map kind), such as
    /// the hint beside a warning. A check that reports only the strictest rule at a place asks
    /// [`RuleRun::siblings_for`] which of them also check a course (ruling R15 of plan M2b).
    pub siblings: Vec<&'r SetRule>,
}

impl RuleRun<'_> {
    pub fn rule(&self) -> &Rule {
        &self.set_rule.rule
    }

    /// The sibling rules that also check this course.
    pub fn siblings_for<'s>(
        &'s self,
        course: &'s CourseContext<'_>,
    ) -> impl Iterator<Item = &'s Rule> + 's {
        self.siblings
            .iter()
            .map(|s| &s.rule)
            .filter(move |rule| covers(rule, course))
    }

    /// A finding of this rule, with its severity, profile and source.
    pub fn finding(&self, location: Location, message_id: &'static str, args: Args) -> Finding {
        Finding {
            rule: self.set_rule.rule.id.clone(),
            profile: self.set_rule.profile.clone(),
            severity: self.set_rule.rule.severity,
            location,
            message_id,
            args,
            source: self.set_rule.rule.source.clone(),
        }
    }

    /// A finding of this rule with a severity of its own, for a rule that grades its findings.
    pub fn finding_with_severity(
        &self,
        severity: Severity,
        location: Location,
        message_id: &'static str,
        args: Args,
    ) -> Finding {
        Finding {
            severity,
            ..self.finding(location, message_id, args)
        }
    }

    /// A skip of this rule.
    pub fn skipped(&self, location: Location, reason_id: &'static str, args: Args) -> Skipped {
        skip_of(self.set_rule, location, reason_id, args)
    }
}

/// A number of the rule's values; a missing one skips the rule with the reason.
fn number(run: &RuleRun<'_>, name: &'static str, skipped: &mut Vec<Skipped>) -> Option<f64> {
    let value = value_number(run.rule(), name);
    if value.is_none() {
        skipped.push(run.skipped(
            Location::Event,
            SKIP_VALUE_MISSING,
            vec![("name", ArgValue::Text(name.to_owned()))],
        ));
    }
    value
}

/// A number of a rule's values, an integer or a float.
pub(crate) fn value_number(rule: &Rule, name: &str) -> Option<f64> {
    rule.values
        .get(name)
        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
}

/// Whether a measured `value` is below `limit`, with a little slack for rounding of map
/// coordinates.
fn below(value: f64, limit: f64) -> bool {
    value < limit - 1e-9
}

/// Whether a rule's print scale filter covers the course.
fn covers(rule: &Rule, course: &CourseContext<'_>) -> bool {
    rule.print_scales.is_empty() || rule.print_scales.contains(&course.print_scale)
}

fn skip_of(rule: &SetRule, location: Location, reason_id: &'static str, args: Args) -> Skipped {
    Skipped {
        rule: Some(rule.rule.id.clone()),
        profile: Some(rule.profile.clone()),
        location,
        reason_id,
        args,
    }
}

/// A check function: reads the context and the rule, adds findings and skips.
pub type CheckFn = fn(&CheckContext<'_>, &RuleRun<'_>, &mut Vec<Finding>, &mut Vec<Skipped>);

/// The implementation of one rule key.
pub struct Check {
    pub key: &'static str,
    pub run: CheckFn,
    /// The place kinds the findings of this key sit at. `finding accept` refuses the others,
    /// since an accepted finding there could never match (ADR 0026).
    pub places: &'static [Place],
}

/// Place kinds of a key whose check does not say yet: anything but the event.
const NOT_EVENT: &[Place] = &[
    Place::Course,
    Place::Control,
    Place::Controls,
    Place::Leg,
    Place::Legs,
];

impl Check {
    /// A check that has not narrowed its place kinds yet; its findings never sit at the event.
    pub const fn new(key: &'static str, run: CheckFn) -> Check {
        Check {
            key,
            run,
            places: NOT_EVENT,
        }
    }

    /// A check whose findings can sit anywhere, the event as a whole included.
    pub const fn event_level(key: &'static str, run: CheckFn) -> Check {
        Check {
            key,
            run,
            places: &Place::ALL,
        }
    }

    /// A check whose findings sit only at these place kinds.
    pub const fn at(key: &'static str, run: CheckFn, places: &'static [Place]) -> Check {
        Check { key, run, places }
    }

    /// Whether a finding of this key can sit at this place kind.
    pub fn reports_at(&self, place: Place) -> bool {
        self.places.contains(&place)
    }
}

/// Every rule key with its check, one line per key (ruling R2 of plan M2b). Every check names the place kinds
/// of its findings with [`Check::at`].
pub const CHECKS: &[Check] = &[
    Check::at("min_distance", distance::min_distance, &[Place::Controls]),
    Check::at(
        "similar_features",
        distance::similar_features,
        &[Place::Controls],
    ),
    Check::at("map_edge", distance::map_edge, &[Place::Control]),
    Check::at(
        "leg_through_control",
        distance::leg_through_control,
        &[Place::Leg],
    ),
    Check::at(
        "leg_through_forbidden",
        distance::leg_through_forbidden,
        &[Place::Leg],
    ),
    Check::at("acute_angle", legs::acute_angle, &[Place::Leg]),
    Check::at("straight_angle", legs::straight_angle, &[Place::Leg]),
    Check::at(
        "number_gap",
        numbers::number_gap,
        &[Place::Leg, Place::Control],
    ),
    Check::at("opposing_legs", legs::opposing_legs, &[Place::Legs]),
    Check::at("code_range", codes::code_range, &[Place::Control]),
    Check::at("code_pairs", codes::code_pairs, &[Place::Controls]),
    Check::at(
        "code_confusable_next",
        codes::code_confusable_next,
        &[Place::Controls],
    ),
    Check::at(
        "same_objects_in_circle",
        codes::same_objects_in_circle,
        &[Place::Control],
    ),
    Check::at("start_capacity", start::start_capacity, &[Place::Control]),
    Check::at(
        "first_leg_angle",
        start::first_leg_angle,
        &[Place::Controls],
    ),
    Check::at(
        "length_table",
        tables::length_table,
        &[Place::Course, Place::Event],
    ),
    Check::at("difficulty_mix", tables::difficulty_mix, &[Place::Course]),
    Check::at("nor_lkm", tables::nor_lkm, &[Place::Course]),
];

/// The check of a rule key.
pub fn check_of(key: &str) -> Option<&'static Check> {
    CHECKS.iter().find(|c| c.key == key)
}

/// Checks the individual courses of `event` on `map` against `rules`.
pub fn check_event(event: &Event, map: &MapDocument, rules: &RuleSet) -> CheckReport {
    run_checks(&CheckContext::new(event, map), rules, CHECKS)
}

/// The engine behind [`check_event`], with the check table as a parameter for tests.
pub(crate) fn run_checks(ctx: &CheckContext<'_>, rules: &RuleSet, checks: &[Check]) -> CheckReport {
    let mut found = Vec::new();
    let mut skipped = Vec::new();
    for course in &ctx.event.courses {
        let kind = match course.kind {
            CourseKind::Individual { .. } => continue,
            CourseKind::Team { .. } => "team",
            CourseKind::Relay { .. } => "relay",
            CourseKind::Score { .. } => "score",
        };
        skipped.push(Skipped {
            rule: None,
            profile: None,
            location: Location::Course(course.name.clone()),
            reason_id: SKIP_COURSE_KIND,
            args: vec![("kind", ArgValue::Text(kind.to_owned()))],
        });
    }

    skipped.extend(ctx.unresolved.iter().cloned());
    for off in &rules.switched_off {
        skipped.push(Skipped {
            rule: Some(off.rule.id.clone()),
            profile: Some(off.profile.clone()),
            location: Location::Event,
            reason_id: SKIP_SWITCHED_OFF,
            args: vec![("overlay", ArgValue::Text(off.by.clone()))],
        });
    }

    let mut keys: Vec<&str> = Vec::new();
    for set_rule in &rules.rules {
        if !keys.contains(&set_rule.rule.key.as_str()) {
            keys.push(&set_rule.rule.key);
        }
    }
    for key in keys {
        let of_key: Vec<&SetRule> = rules.rules.iter().filter(|r| r.rule.key == key).collect();
        run_key(ctx, &of_key, checks, &mut found, &mut skipped);
    }

    let accepted_keys: Vec<(FindingKey, &str)> = ctx
        .event
        .accepted_findings
        .iter()
        .map(|a| (a.key(), a.note.as_str()))
        .collect();
    let mut report = CheckReport {
        skipped,
        ..CheckReport::default()
    };
    let mut matched = vec![false; accepted_keys.len()];
    for finding in found {
        let key = finding.key();
        match accepted_keys.iter().position(|(k, _)| *k == key) {
            Some(i) => {
                matched[i] = true;
                report
                    .accepted
                    .push((finding, accepted_keys[i].1.to_owned()));
            }
            None => report.findings.push(finding),
        }
    }
    report.unmatched = ctx
        .event
        .accepted_findings
        .iter()
        .zip(matched)
        .filter(|(_, m)| !m)
        .map(|(a, _)| a.clone())
        .collect();
    report
}

/// Why the event as a whole falls outside a rule's discipline or map kind filter.
fn event_filter(ctx: &CheckContext<'_>, rule: &Rule) -> Option<(&'static str, Args)> {
    let discipline = ctx.event.event.discipline;
    if !rule.disciplines.is_empty() && !rule.disciplines.contains(&discipline) {
        return Some(if discipline == Discipline::Training {
            (SKIP_TRAINING, Vec::new())
        } else {
            let name = discipline.as_str().to_owned();
            (SKIP_DISCIPLINE, vec![("discipline", ArgValue::Text(name))])
        });
    }
    if !rule.map_kinds.is_empty() && !rule.map_kinds.contains(&ctx.map_kind) {
        let name = ctx.map_kind.as_str().to_owned();
        return Some((SKIP_MAP_KIND, vec![("kind", ArgValue::Text(name))]));
    }
    None
}

/// Runs the rules of one key. A rule that does not apply is reported as skipped only where no
/// other rule of the key applies: for the whole event (discipline, map kind) or per course
/// (print scale).
fn run_key(
    ctx: &CheckContext<'_>,
    of_key: &[&SetRule],
    checks: &[Check],
    found: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    let (active, filtered): (Vec<&SetRule>, Vec<&SetRule>) = of_key
        .iter()
        .partition(|r| event_filter(ctx, &r.rule).is_none());
    if active.is_empty() {
        for set_rule in filtered {
            if let Some((reason, args)) = event_filter(ctx, &set_rule.rule) {
                skipped.push(skip_of(set_rule, Location::Event, reason, args));
            }
        }
        return;
    }
    for course in &ctx.courses {
        if !active.iter().any(|r| covers(&r.rule, course)) {
            for set_rule in &active {
                let scale = ArgValue::Int(i64::from(course.print_scale));
                skipped.push(skip_of(
                    set_rule,
                    Location::Course(course.course.name.clone()),
                    SKIP_PRINT_SCALE,
                    vec![("scale", scale)],
                ));
            }
        }
    }
    for &set_rule in &active {
        let run = RuleRun {
            set_rule,
            courses: ctx
                .courses
                .iter()
                .filter(|c| covers(&set_rule.rule, c))
                .collect(),
            siblings: active
                .iter()
                .copied()
                .filter(|r| !std::ptr::eq(*r, set_rule))
                .collect(),
        };
        // Task 2 refuses profiles with unknown keys; a test table may lack a key, and the rule
        // is then not run.
        if let Some(check) = checks.iter().find(|c| c.key == set_rule.rule.key) {
            (check.run)(ctx, &run, found, skipped);
        }
    }
}

#[cfg(test)]
pub(crate) mod testing;

#[cfg(test)]
mod tests;
