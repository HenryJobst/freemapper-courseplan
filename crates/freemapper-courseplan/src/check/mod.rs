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

mod context;
mod distance;
mod geometry;

pub use context::{CheckContext, CourseContext};

use freemapper_courseplan_model::{
    AcceptedFinding, ControlCode, CourseKind, Discipline, Event, FindingKey, LegRef,
};

use crate::omap::MapDocument;
use crate::rules::{Rule, RuleSet, SetRule, Severity, SourceRef};
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
    /// The leg of `course` that leaves position `after` (from 1 as in the CLI; 0 is the leg from
    /// the start).
    Leg { course: String, after: u32 },
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
}

impl Place {
    /// Every place kind, in declaration order.
    pub const ALL: [Place; 5] = [
        Place::Event,
        Place::Course,
        Place::Control,
        Place::Controls,
        Place::Leg,
    ];

    /// The name in English messages and the suffix of its `word-place-*` text.
    pub fn as_str(self) -> &'static str {
        match self {
            Place::Event => "event",
            Place::Course => "course",
            Place::Control => "control",
            Place::Controls => "controls",
            Place::Leg => "leg",
        }
    }

    /// The place kind of an accepted finding's key: what it names, or the event when it names
    /// nothing.
    pub fn of_key(key: &FindingKey) -> Place {
        if key.course.is_some() {
            Place::Course
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
        }
    }

    /// The course the location belongs to, for grouping the output.
    pub fn course(&self) -> Option<&str> {
        match self {
            Location::Course(name) | Location::Leg { course: name, .. } => Some(name),
            _ => None,
        }
    }

    /// The location as text, for example "control 31" or "course A, leg after 3".
    pub fn text(&self, lang: &Lang) -> String {
        let v = |s: &str| FluentValue::from(s.to_owned());
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
            Location::Leg { course, after: 0 } => {
                text(lang, "check-location-start-leg", &[("course", v(course))])
            }
            Location::Leg { course, after } => text(
                lang,
                "check-location-leg",
                &[("course", v(course)), ("index", FluentValue::from(*after))],
            ),
        }
    }
}

/// A value inside a finding text.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    Text(String),
    /// A measured value; shown with one decimal in the language's decimal mark.
    Number(f64),
    Int(i64),
}

/// The arguments of a finding or skip text: Fluent variable name and value.
pub type Args = Vec<(&'static str, ArgValue)>;

impl ArgValue {
    fn fluent(&self, lang: &Lang) -> FluentValue<'static> {
        match self {
            ArgValue::Text(s) => FluentValue::from(s.clone()),
            ArgValue::Int(n) => FluentValue::from(*n),
            ArgValue::Number(x) => FluentValue::from(format_number(lang, *x)),
        }
    }
}

/// One decimal, without a trailing `.0`, with the decimal mark of the language. Fluent's
/// `NUMBER()` does not localize separators in fluent-bundle 0.16.
fn format_number(lang: &Lang, x: f64) -> String {
    let rounded = (x * 10.0).round() / 10.0;
    let shown = if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    };
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
            Location::Leg { course, after } => {
                accepted.legs = vec![LegRef {
                    course: course.clone(),
                    after: *after,
                }]
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

/// Reason: the check of this key is not implemented yet (ruling R2 of plan M2b).
pub const SKIP_NOT_IMPLEMENTED: &str = "check-skip-not-implemented";
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
/// Reason: the map has no features, so it has no extent.
pub const SKIP_NO_MAP_EXTENT: &str = "check-skip-no-map-extent";

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

/// One rule as a check function sees it: the rule and the courses it applies to.
pub struct RuleRun<'r> {
    pub set_rule: &'r SetRule,
    /// The individual courses whose print scale the rule covers, in file order.
    pub courses: Vec<&'r CourseContext<'r>>,
}

impl RuleRun<'_> {
    pub fn rule(&self) -> &Rule {
        &self.set_rule.rule
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

    /// A skip of this rule.
    pub fn skipped(&self, location: Location, reason_id: &'static str, args: Args) -> Skipped {
        skip_of(self.set_rule, location, reason_id, args)
    }
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
const NOT_EVENT: &[Place] = &[Place::Course, Place::Control, Place::Controls, Place::Leg];

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

/// Every rule key with its check, one line per key (ruling R2 of plan M2b). Tasks 4 to 8 replace
/// the `not_implemented` stubs and name the place kinds of their findings with [`Check::at`].
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
    Check::new("acute_angle", not_implemented),
    Check::new("opposing_legs", not_implemented),
    Check::new("code_range", not_implemented),
    Check::new("code_pairs", not_implemented),
    Check::new("code_confusable_next", not_implemented),
    Check::new("same_objects_in_circle", not_implemented),
    Check::event_level("start_capacity", not_implemented),
    Check::new("first_leg_angle", not_implemented),
    Check::new("length_table", not_implemented),
    Check::new("difficulty_mix", not_implemented),
    Check::new("nor_lkm", not_implemented),
];

/// The check of a rule key.
pub fn check_of(key: &str) -> Option<&'static Check> {
    CHECKS.iter().find(|c| c.key == key)
}

fn not_implemented(
    _: &CheckContext<'_>,
    run: &RuleRun<'_>,
    _: &mut Vec<Finding>,
    skipped: &mut Vec<Skipped>,
) {
    skipped.push(run.skipped(Location::Event, SKIP_NOT_IMPLEMENTED, Vec::new()));
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
    for finding in found {
        let key = finding.key();
        match accepted_keys.iter().find(|(k, _)| *k == key) {
            Some((_, note)) => report.accepted.push((finding, (*note).to_owned())),
            None => report.findings.push(finding),
        }
    }
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
    let covers = |rule: &Rule, course: &CourseContext<'_>| {
        rule.print_scales.is_empty() || rule.print_scales.contains(&course.print_scale)
    };
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
    for set_rule in active {
        let run = RuleRun {
            set_rule,
            courses: ctx
                .courses
                .iter()
                .filter(|c| covers(&set_rule.rule, c))
                .collect(),
        };
        match checks.iter().find(|c| c.key == set_rule.rule.key) {
            Some(check) => (check.run)(ctx, &run, found, skipped),
            // Task 2 refuses profiles with unknown keys; a test table may still lack one.
            None => not_implemented(ctx, &run, found, skipped),
        }
    }
}

#[cfg(test)]
pub(crate) mod testing;

#[cfg(test)]
mod tests;
