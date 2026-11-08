//! Program texts from Fluent files (ADR 0022).
//!
//! Provenance: written in freemapper-courseplan; freemapper and freemapper-print copy this file
//! (everything above the `Localize` implementations is program independent).
//!
//! Every user-facing text lives in `i18n/<language>/courseplan.ftl`. The files are embedded with
//! `include_str!`, so adding a language means adding a folder, listing it in `EMBEDDED` and
//! running the tests. English is the reference and the fallback: a message missing in another
//! language is taken from English, and a message missing everywhere comes back as its id.
//!
//! # Language detection
//!
//! [`detect_lang`] looks at these sources in order and takes the first one that names an
//! available language:
//!
//! 1. the `--lang` argument,
//! 2. `FREEMAPPER_LANG`,
//! 3. `LC_ALL`, 4. `LC_MESSAGES`, 5. `LANG`.
//!
//! Only the language part counts, case-insensitively: `de`, `de-DE`, `de_DE.UTF-8` and `de_AT`
//! all mean German. An unset or empty variable, `C`, `POSIX` and a language the program does not
//! have (`fr`) pass to the next source, so `FREEMAPPER_LANG=fr` with `LANG=de_DE.UTF-8` gives
//! German. If nothing matches, the result is English. A `--lang` value that names no available
//! language is a usage error for the caller: [`parse_lang_arg`] reports it with the available
//! languages, while [`detect_lang`] alone skips it.
//!
//! # What is not translated
//!
//! Only the sentence around a foreign text is localized. Parser messages of the `toml` crate
//! (inside `ProjectError::Parse`), XML parser messages and operating system error texts from
//! `std::io::Error` arrive as arguments and stay English. `Display` of every error type stays
//! English too (logs, tests); `Localize::localize(&Lang::english())` returns exactly that text.

use std::sync::OnceLock;

pub use fluent_bundle::FluentValue;
use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use unic_langid::LanguageIdentifier;

use freemapper_courseplan_model::{ModelError, ProjectError};

use crate::course::CourseError;
use crate::edit::EditError;
use crate::init::InitError;
use crate::omap::OmapError;

/// A language of the program texts, identified by its language part (`en`, `de`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lang(pub LanguageIdentifier);

impl Lang {
    /// English, the reference language and the fallback.
    pub fn english() -> Lang {
        Lang(LanguageIdentifier::from_parts(
            "en".parse().expect("valid language"),
            None,
            None,
            &[],
        ))
    }

    /// The language part as text, for example `de`.
    pub fn code(&self) -> String {
        self.0.language.to_string()
    }
}

/// The embedded catalogs: language code and file content. English comes first.
const EMBEDDED: &[(&str, &str)] = &[
    ("en", include_str!("../i18n/en/courseplan.ftl")),
    ("de", include_str!("../i18n/de/courseplan.ftl")),
];

type Bundle = FluentBundle<FluentResource>;

fn lang_of(code: &str) -> Lang {
    Lang(LanguageIdentifier::from_parts(
        code.parse().expect("valid language code"),
        None,
        None,
        &[],
    ))
}

/// The languages the program has texts for, English first.
pub fn available() -> Vec<Lang> {
    EMBEDDED.iter().map(|(code, _)| lang_of(code)).collect()
}

fn build_bundle(code: &str, source: &str) -> Bundle {
    // A file with syntax errors still yields its valid messages; the tests refuse such files.
    let resource = match FluentResource::try_new(source.to_owned()) {
        Ok(resource) => resource,
        Err((resource, _)) => resource,
    };
    let mut bundle = FluentBundle::new_concurrent(vec![lang_of(code).0]);
    // Isolation marks would show up as stray characters in terminals.
    bundle.set_use_isolating(false);
    // Overriding is impossible with a single resource; ignore the (empty) error list.
    let _ = bundle.add_resource(resource);
    bundle
}

fn bundles() -> &'static [(String, Bundle)] {
    static BUNDLES: OnceLock<Vec<(String, Bundle)>> = OnceLock::new();
    BUNDLES.get_or_init(|| {
        EMBEDDED
            .iter()
            .map(|(code, source)| ((*code).to_owned(), build_bundle(code, source)))
            .collect()
    })
}

fn bundle_of(lang: &Lang) -> Option<&'static Bundle> {
    let code = lang.code();
    bundles().iter().find(|(c, _)| *c == code).map(|(_, b)| b)
}

fn english_bundle() -> &'static Bundle {
    bundle_of(&Lang::english()).expect("English is embedded")
}

/// The language part of a locale value such as `de_DE.UTF-8`, `de-AT` or `en`; `None` for an
/// empty value, `C` and `POSIX`.
fn language_part(value: &str) -> Option<String> {
    let head = value.split(['.', '@']).next().unwrap_or("");
    let language = head.split(['_', '-']).next().unwrap_or("").trim();
    if language.is_empty()
        || language.eq_ignore_ascii_case("c")
        || language.eq_ignore_ascii_case("posix")
    {
        None
    } else {
        Some(language.to_ascii_lowercase())
    }
}

fn match_available(value: &str) -> Option<Lang> {
    let language = language_part(value)?;
    available().into_iter().find(|l| l.code() == language)
}

/// The `--lang` value as a language, or the list of available languages when it names none.
pub fn parse_lang_arg(arg: &str) -> Result<Lang, Vec<Lang>> {
    match_available(arg.trim()).ok_or_else(available)
}

/// Picks the language: `--lang`, else `FREEMAPPER_LANG`, else `LC_ALL`, `LC_MESSAGES`, `LANG`,
/// else English. Values naming no available language pass to the next source. `env` reads one
/// environment variable (`|name| std::env::var(name).ok()` in the programs).
pub fn detect_lang(arg: Option<&str>, env: &dyn Fn(&str) -> Option<String>) -> Lang {
    let from_arg = arg.and_then(match_available);
    let from_env = || {
        ["FREEMAPPER_LANG", "LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .find_map(|name| env(name).and_then(|v| match_available(&v)))
    };
    from_arg.or_else(from_env).unwrap_or_else(Lang::english)
}

fn format_in(bundle: &Bundle, id: &str, args: &FluentArgs) -> Option<String> {
    let pattern = bundle.get_message(id)?.value()?;
    let mut errors = Vec::new();
    Some(
        bundle
            .format_pattern(pattern, Some(args), &mut errors)
            .into_owned(),
    )
}

/// Looks `id` up in the bundles in order; the id itself when none has it.
fn lookup(chain: &[&Bundle], id: &str, args: &[(&str, FluentValue)]) -> String {
    let mut fluent_args = FluentArgs::new();
    for (name, value) in args {
        fluent_args.set(*name, value.clone());
    }
    chain
        .iter()
        .find_map(|bundle| format_in(bundle, id, &fluent_args))
        .unwrap_or_else(|| id.to_owned())
}

/// The text `id` in `lang`, falling back to English, and to the id itself as a last resort.
pub fn text(lang: &Lang, id: &str, args: &[(&str, FluentValue)]) -> String {
    match bundle_of(lang) {
        Some(bundle) => lookup(&[bundle, english_bundle()], id, args),
        None => lookup(&[english_bundle()], id, args),
    }
}

/// Program texts for a value, for example an error.
pub trait Localize {
    /// The text in `lang`. In English it equals `to_string()` for error types.
    fn localize(&self, lang: &Lang) -> String;
}

// ---------------------------------------------------------------------------------------------
// Program specific part: the error types of this program.

fn v(value: impl ToString) -> FluentValue<'static> {
    FluentValue::from(value.to_string())
}

/// A word inside a message, such as a course kind. `raw` is the English word used in the error
/// values; a word without catalog entry stays as it is.
fn word(lang: &Lang, prefix: &str, raw: &str) -> String {
    let id = format!("{prefix}-{}", raw.replace(' ', "-"));
    let found = text(lang, &id, &[]);
    if found == id { raw.to_owned() } else { found }
}

impl Localize for ModelError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            ModelError::DuplicateControl(code) => {
                text(lang, "error-model-duplicate-control", &[("code", v(code))])
            }
            ModelError::DuplicateCourse(name) => {
                text(lang, "error-model-duplicate-course", &[("name", v(name))])
            }
            ModelError::DuplicateClass(name) => {
                text(lang, "error-model-duplicate-class", &[("name", v(name))])
            }
            ModelError::DuplicateSpecialId(id) => {
                text(lang, "error-model-duplicate-special-id", &[("id", v(id))])
            }
            ModelError::UnknownCourseInSpecial { id, course } => text(
                lang,
                "error-model-unknown-course-in-special",
                &[("id", v(id)), ("course", v(course))],
            ),
            ModelError::UnknownControl { course, code } => text(
                lang,
                "error-model-unknown-control",
                &[("course", v(course)), ("code", v(code))],
            ),
            ModelError::UnknownStart { course, id } => text(
                lang,
                "error-model-unknown-start",
                &[("course", v(course)), ("id", v(id))],
            ),
            ModelError::UnknownFinish { course, id } => text(
                lang,
                "error-model-unknown-finish",
                &[("course", v(course)), ("id", v(id))],
            ),
            ModelError::TeamTooSmall { course, team_size } => text(
                lang,
                "error-model-team-too-small",
                &[("course", v(course)), ("team_size", v(team_size))],
            ),
            ModelError::FreeControlIsMandatory { course, code } => text(
                lang,
                "error-model-free-control-is-mandatory",
                &[("course", v(course)), ("code", v(code))],
            ),
            ModelError::DateNotADate(date) => {
                text(lang, "error-model-date-not-a-date", &[("date", v(date))])
            }
            ModelError::UnsupportedFormat(format) => text(
                lang,
                "error-model-unsupported-format",
                &[("format", v(format))],
            ),
            ModelError::UnknownKey { table, key } if table.is_empty() => {
                text(lang, "error-model-unknown-key-top", &[("key", v(key))])
            }
            ModelError::UnknownKey { table, key } => text(
                lang,
                "error-model-unknown-key-in",
                &[("key", v(key)), ("table", v(table))],
            ),
        }
    }
}

impl Localize for ProjectError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            ProjectError::Read { path, source } => text(
                lang,
                "error-project-read",
                &[("path", v(path.display())), ("source", v(source))],
            ),
            ProjectError::Write { path, source } => text(
                lang,
                "error-project-write",
                &[("path", v(path.display())), ("source", v(source))],
            ),
            ProjectError::AlreadyExists { path } => text(
                lang,
                "error-project-already-exists",
                &[("path", v(path.display()))],
            ),
            ProjectError::Parse { path, message } => {
                // The model crate builds `message` as "cannot parse <path>: <parser text>" (or
                // "cannot serialise"); the parser text is split off and stays English.
                let shown = path.display().to_string();
                let split = |verb: &str| {
                    message
                        .strip_prefix(&format!("{verb} {shown}: "))
                        .map(str::to_owned)
                };
                let (id, rest) = if let Some(rest) = split("cannot parse") {
                    ("error-project-parse", rest)
                } else if let Some(rest) = split("cannot serialise") {
                    ("error-project-serialise", rest)
                } else {
                    return message.clone();
                };
                text(lang, id, &[("path", v(&shown)), ("message", v(rest))])
            }
            ProjectError::Invalid { path, errors } => {
                let list: String = errors
                    .iter()
                    .map(|e| format!("\n  {}", e.localize(lang)))
                    .collect();
                text(
                    lang,
                    "error-project-invalid",
                    &[("path", v(path.display())), ("list", v(list))],
                )
            }
        }
    }
}

impl Localize for OmapError {
    fn localize(&self, lang: &Lang) -> String {
        /// The text of the binary-format refusal in `omap/read.rs` (copied code, ADR 0017).
        const BINARY_FORMAT_MESSAGE: &str = "This file is in the old binary format of OpenOrienteering Mapper, which this program cannot read. Open it in Mapper and save it again as a current .omap or .xmap file.";
        match self {
            OmapError::Xml(message) => text(lang, "error-omap-xml", &[("message", v(message))]),
            OmapError::MissingElement(name) => {
                text(lang, "error-omap-missing-element", &[("name", v(name))])
            }
            OmapError::Io(source) => text(lang, "error-omap-io", &[("message", v(source))]),
            OmapError::Unsupported(message) if message == BINARY_FORMAT_MESSAGE => {
                text(lang, "error-omap-binary-format", &[])
            }
            OmapError::Unsupported(message) => {
                text(lang, "error-omap-unsupported", &[("message", v(message))])
            }
        }
    }
}

impl Localize for InitError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            InitError::Map { path, source } => text(
                lang,
                "error-init-map",
                &[
                    ("path", v(path.display())),
                    ("source", v(source.localize(lang))),
                ],
            ),
            InitError::ForestNotSupported => text(lang, "error-init-forest", &[]),
            InitError::Project(error) => error.localize(lang),
            InitError::Io { path, source } => text(
                lang,
                "error-init-io",
                &[("path", v(path.display())), ("source", v(source))],
            ),
        }
    }
}

impl Localize for CourseError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            CourseError::NotSupported { course, kind } => text(
                lang,
                "error-course-not-supported",
                &[
                    ("course", v(course)),
                    ("kind", v(word(lang, "word-kind", kind))),
                ],
            ),
            CourseError::UnknownCourse(course) => text(
                lang,
                "error-course-unknown-course",
                &[("course", v(course))],
            ),
            CourseError::UnknownReference { course, id } => text(
                lang,
                "error-course-unknown-reference",
                &[("course", v(course)), ("id", v(id))],
            ),
        }
    }
}

impl Localize for EditError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            EditError::Invalid(errors) => {
                let list = errors
                    .iter()
                    .map(|e| e.localize(lang))
                    .collect::<Vec<_>>()
                    .join("; ");
                text(lang, "error-edit-invalid", &[("errors", v(list))])
            }
            EditError::NotFound { what, id } => text(
                lang,
                "error-edit-not-found",
                &[("what", v(word(lang, "word-what", what))), ("id", v(id))],
            ),
            EditError::CodeTaken(code) => text(lang, "error-edit-code-taken", &[("code", v(code))]),
            EditError::IdTaken(id) => text(lang, "error-edit-id-taken", &[("id", v(id))]),
            EditError::OutsideMap { at, min, max } => text(
                lang,
                "error-edit-outside-map",
                &[
                    ("x", v(format!("{:.1}", at.x))),
                    ("y", v(format!("{:.1}", at.y))),
                    ("min_x", v(format!("{:.1}", min.x))),
                    ("min_y", v(format!("{:.1}", min.y))),
                    ("max_x", v(format!("{:.1}", max.x))),
                    ("max_y", v(format!("{:.1}", max.y))),
                ],
            ),
            EditError::InUse { id, courses } => text(
                lang,
                "error-edit-in-use",
                &[("id", v(id)), ("courses", v(courses.join(", ")))],
            ),
            EditError::NotSupported { course } => {
                text(lang, "error-edit-not-supported", &[("course", v(course))])
            }
            EditError::BadIndex { course, index } => text(
                lang,
                "error-edit-bad-index",
                &[("course", v(course)), ("index", v(index))],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_courseplan_model::{ControlCode, MapPoint};
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn env_of<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    fn de() -> Lang {
        lang_of("de")
    }

    #[test]
    fn detects_language_in_order() {
        let env = env_of(&[("FREEMAPPER_LANG", "en"), ("LANG", "de_DE.UTF-8")]);
        assert_eq!(detect_lang(Some("de"), &env), de());
        assert_eq!(detect_lang(Some("DE"), &env), de());
        assert_eq!(detect_lang(None, &env), Lang::english());
        let env = env_of(&[("LANG", "de_DE.UTF-8")]);
        assert_eq!(detect_lang(None, &env), de());
        assert_eq!(detect_lang(Some("en"), &env), Lang::english());
        let env = env_of(&[("LANG", "C")]);
        assert_eq!(detect_lang(None, &env), Lang::english());
        let env = env_of(&[("FREEMAPPER_LANG", "fr"), ("LANG", "de_DE.UTF-8")]);
        assert_eq!(detect_lang(None, &env), de());
        // LC_ALL before LC_MESSAGES before LANG; empty values and POSIX pass on.
        let env = env_of(&[("LC_ALL", ""), ("LC_MESSAGES", "POSIX"), ("LANG", "de_AT")]);
        assert_eq!(detect_lang(None, &env), de());
        let env = env_of(&[("LC_ALL", "en_US.UTF-8"), ("LANG", "de_DE")]);
        assert_eq!(detect_lang(None, &env), Lang::english());
        let env = env_of(&[("LC_MESSAGES", "de-DE")]);
        assert_eq!(detect_lang(None, &env), de());
        assert_eq!(detect_lang(None, &env_of(&[])), Lang::english());
        // An unknown --lang passes to the next source.
        let env = env_of(&[("LANG", "de_DE")]);
        assert_eq!(detect_lang(Some("fr"), &env), de());
    }

    #[test]
    fn unknown_lang_argument_is_refused() {
        assert_eq!(parse_lang_arg("de"), Ok(de()));
        assert_eq!(parse_lang_arg("EN"), Ok(Lang::english()));
        let available = parse_lang_arg("fr").unwrap_err();
        assert_eq!(available, vec![Lang::english(), de()]);
        assert!(parse_lang_arg("").is_err());
        assert!(parse_lang_arg("C").is_err());
    }

    fn message_ids(source: &str) -> BTreeSet<String> {
        use fluent_syntax::ast::Entry;
        let resource = fluent_syntax::parser::parse(source).expect("file parses without errors");
        resource
            .body
            .iter()
            .filter_map(|entry| match entry {
                Entry::Message(m) => Some(m.id.name.to_owned()),
                _ => None,
            })
            .collect()
    }

    /// Sample values for every variable the catalogs use.
    fn sample_args() -> Vec<(&'static str, FluentValue<'static>)> {
        [
            "code",
            "name",
            "id",
            "course",
            "team_size",
            "date",
            "format",
            "key",
            "table",
            "path",
            "source",
            "message",
            "list",
            "kind",
            "errors",
            "what",
            "x",
            "y",
            "min_x",
            "min_y",
            "max_x",
            "max_y",
            "courses",
            "index",
            "value",
            "count",
            "total",
            "individual",
            "team",
            "relay",
            "score",
            "available",
            "arg",
            "args",
            "other",
            "columns",
            "direction",
            "gap",
            "codes",
            "bends",
            "taped",
            "km",
            "controls",
            "climb",
            "line",
            "column",
        ]
        .iter()
        .map(|n| (*n, v("sample")))
        .collect()
    }

    #[test]
    fn every_language_has_every_message() {
        let english = message_ids(EMBEDDED[0].1);
        assert!(!english.is_empty());
        for (code, source) in EMBEDDED {
            let ids = message_ids(source);
            assert_eq!(ids, english, "message ids of {code} differ from English");
            let lang = lang_of(code);
            let bundle = bundle_of(&lang).unwrap();
            let args = sample_args();
            let mut fluent_args = FluentArgs::new();
            for (n, value) in &args {
                fluent_args.set(*n, value.clone());
            }
            for id in &ids {
                let pattern = bundle.get_message(id).unwrap().value().unwrap();
                let mut errors = Vec::new();
                let out = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
                assert!(errors.is_empty(), "{code}/{id}: {errors:?}");
                assert!(!out.trim().is_empty(), "{code}/{id} is empty");
            }
        }
    }

    /// The variables a pattern refers to, including those inside selectors and variants.
    fn variables_of(source: &str) -> std::collections::BTreeMap<String, BTreeSet<String>> {
        use fluent_syntax::ast::{
            CallArguments, Entry, Expression, InlineExpression, Pattern, PatternElement,
        };
        fn in_inline(e: &InlineExpression<&str>, out: &mut BTreeSet<String>) {
            match e {
                InlineExpression::VariableReference { id } => {
                    out.insert(id.name.to_owned());
                }
                InlineExpression::FunctionReference { arguments, .. } => in_args(arguments, out),
                InlineExpression::TermReference {
                    arguments: Some(arguments),
                    ..
                } => in_args(arguments, out),
                InlineExpression::Placeable { expression } => in_expression(expression, out),
                _ => {}
            }
        }
        fn in_args(a: &CallArguments<&str>, out: &mut BTreeSet<String>) {
            a.positional.iter().for_each(|e| in_inline(e, out));
            a.named.iter().for_each(|n| in_inline(&n.value, out));
        }
        fn in_expression(e: &Expression<&str>, out: &mut BTreeSet<String>) {
            match e {
                Expression::Inline(i) => in_inline(i, out),
                Expression::Select { selector, variants } => {
                    in_inline(selector, out);
                    variants.iter().for_each(|v| in_pattern(&v.value, out));
                }
            }
        }
        fn in_pattern(p: &Pattern<&str>, out: &mut BTreeSet<String>) {
            for element in &p.elements {
                if let PatternElement::Placeable { expression } = element {
                    in_expression(expression, out);
                }
            }
        }
        let resource = fluent_syntax::parser::parse(source).expect("file parses");
        let mut map = std::collections::BTreeMap::new();
        for entry in &resource.body {
            if let Entry::Message(m) = entry {
                let mut vars = BTreeSet::new();
                if let Some(pattern) = &m.value {
                    in_pattern(pattern, &mut vars);
                }
                map.insert(m.id.name.to_owned(), vars);
            }
        }
        map
    }

    #[test]
    fn every_language_uses_the_same_variables() {
        let english = variables_of(EMBEDDED[0].1);
        for (code, source) in EMBEDDED {
            let vars = variables_of(source);
            for (id, expected) in &english {
                assert_eq!(
                    vars.get(id),
                    Some(expected),
                    "{code}/{id}: variables differ from English"
                );
            }
        }
    }

    /// Message ids written as string literals in the code above the test module.
    fn ids_in_source(source: &str) -> BTreeSet<String> {
        let code = source.split("#[cfg(test)]").next().unwrap();
        code.split('"')
            .skip(1)
            .step_by(2)
            .filter(|lit| {
                ["error-", "cli-", "word-", "mcp-"]
                    .iter()
                    .any(|p| lit.starts_with(p))
                    && lit.chars().all(|c| c.is_ascii_lowercase() || c == '-')
                    && !lit.ends_with('-')
                    && !["word-kind", "word-what"].contains(lit)
            })
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn ids_used_in_code_exist() {
        let mut ids = ids_in_source(include_str!("text.rs"));
        assert!(ids.len() > 30, "scanner found only {}", ids.len());
        for kind in ["individual", "team", "relay", "score"] {
            ids.insert(format!("word-kind-{kind}"));
        }
        for what in [
            "control",
            "course",
            "special-object",
            "class",
            "control-or-special-object",
        ] {
            ids.insert(format!("word-what-{what}"));
        }
        let args = sample_args();
        for lang in available() {
            for id in &ids {
                let out = text(&lang, id, &args);
                assert_ne!(&out, id, "{}: {id} is missing", lang.code());
                assert!(
                    !out.contains(['\u{2068}', '\u{2069}']),
                    "{}/{id} has isolation marks",
                    lang.code()
                );
            }
        }
        let english = message_ids(EMBEDDED[0].1);
        for id in &ids {
            assert!(english.contains(id), "{id} not in the English file");
        }
    }

    #[test]
    fn missing_message_falls_back_to_english() {
        let partial = build_bundle("de", "other = Anderes\n");
        let english = english_bundle();
        let args = [("code", v("31"))];
        assert_eq!(
            lookup(&[&partial, english], "error-edit-code-taken", &args),
            "control 31 already exists"
        );
        assert_eq!(lookup(&[&partial, english], "other", &[]), "Anderes");
        assert_eq!(
            lookup(&[&partial, english], "no-such-id", &[]),
            "no-such-id"
        );
    }

    fn code(s: &str) -> ControlCode {
        ControlCode(s.to_owned())
    }

    fn s(x: &str) -> String {
        x.to_owned()
    }

    fn model_errors() -> Vec<ModelError> {
        vec![
            ModelError::DuplicateControl(code("31")),
            ModelError::DuplicateCourse(s("A")),
            ModelError::DuplicateClass(s("H21")),
            ModelError::DuplicateSpecialId(s("w1")),
            ModelError::UnknownCourseInSpecial {
                id: s("w1"),
                course: s("A"),
            },
            ModelError::UnknownControl {
                course: s("A"),
                code: code("31"),
            },
            ModelError::UnknownStart {
                course: s("A"),
                id: s("s1"),
            },
            ModelError::UnknownFinish {
                course: s("A"),
                id: s("f1"),
            },
            ModelError::TeamTooSmall {
                course: s("A"),
                team_size: 1,
            },
            ModelError::FreeControlIsMandatory {
                course: s("A"),
                code: code("31"),
            },
            ModelError::DateNotADate(s("2026-13-45")),
            ModelError::UnsupportedFormat(7),
            ModelError::UnknownKey {
                table: s(""),
                key: s("colour"),
            },
            ModelError::UnknownKey {
                table: s("courses[0]"),
                key: s("colour"),
            },
        ]
    }

    /// Checks one value: German is not empty and differs, English equals `Display`.
    fn check<E: Localize + std::fmt::Display + std::fmt::Debug>(e: &E) {
        let en = e.localize(&Lang::english());
        assert_eq!(en, e.to_string(), "{e:?}");
        let german = e.localize(&de());
        assert!(!german.is_empty(), "{e:?}");
        assert_ne!(german, en, "{e:?}");
    }

    #[test]
    fn every_error_has_both_languages() {
        let path = || PathBuf::from("ev/x.toml");
        let io = || std::io::Error::other("disk on fire");
        for e in model_errors() {
            check(&e);
        }
        let project = vec![
            ProjectError::Read {
                path: path(),
                source: io(),
            },
            ProjectError::Write {
                path: path(),
                source: io(),
            },
            ProjectError::AlreadyExists { path: path() },
            ProjectError::Parse {
                path: path(),
                message: "cannot parse ev/x.toml: TOML parse error at line 3".into(),
            },
            ProjectError::Parse {
                path: path(),
                message: "cannot serialise ev/x.toml: unsupported value".into(),
            },
            ProjectError::Invalid {
                path: path(),
                errors: model_errors(),
            },
        ];
        for e in &project {
            check(e);
        }
        let omap = || {
            vec![
                OmapError::Xml("unexpected end".into()),
                OmapError::MissingElement("map"),
                OmapError::Io(io()),
                OmapError::Unsupported("This file is in the old binary format of OpenOrienteering Mapper, which this program cannot read. Open it in Mapper and save it again as a current .omap or .xmap file.".into()),
            ]
        };
        for e in omap() {
            check(&e);
        }
        // A free-form Unsupported text is passed through in both languages.
        let other = OmapError::Unsupported("something else".into());
        assert_eq!(other.localize(&de()), other.to_string());
        for source in omap() {
            check(&InitError::Map {
                path: path(),
                source,
            });
        }
        check(&InitError::ForestNotSupported);
        check(&InitError::Project(ProjectError::AlreadyExists {
            path: path(),
        }));
        check(&InitError::Io {
            path: path(),
            source: io(),
        });
        for e in [
            CourseError::NotSupported {
                course: s("T"),
                kind: "team",
            },
            CourseError::NotSupported {
                course: s("T"),
                kind: "relay",
            },
            CourseError::UnknownCourse(s("T")),
            CourseError::UnknownReference {
                course: s("T"),
                id: s("31"),
            },
        ] {
            check(&e);
        }
        let at = MapPoint { x: 1.25, y: -3.0 };
        let min = MapPoint { x: 0.0, y: 0.0 };
        let max = MapPoint { x: 100.0, y: 80.5 };
        let mut edits = vec![
            EditError::Invalid(model_errors()),
            EditError::CodeTaken(code("31")),
            EditError::IdTaken(s("w1")),
            EditError::OutsideMap { at, min, max },
            EditError::InUse {
                id: s("31"),
                courses: vec![s("A"), s("B")],
            },
            EditError::NotSupported { course: s("T") },
            EditError::BadIndex {
                course: s("A"),
                index: 9,
            },
        ];
        for what in [
            "control",
            "course",
            "special object",
            "class",
            "control or special object",
        ] {
            edits.push(EditError::NotFound { what, id: s("x") });
        }
        for e in &edits {
            check(e);
        }
    }

    #[test]
    fn german_texts_use_the_domain_words() {
        let e = EditError::NotFound {
            what: "special object",
            id: s("w1"),
        };
        assert_eq!(e.localize(&de()), "Sonderobjekt w1 gibt es nicht");
        let e = CourseError::NotSupported {
            course: s("M"),
            kind: "team",
        };
        assert!(e.localize(&de()).contains("Mannschaftsbahn"));
    }

    /// The German catalog uses the glossary terms (ruling R13), not the older wording.
    #[test]
    fn german_catalog_uses_the_glossary_terms() {
        let german = EMBEDDED.iter().find(|(code, _)| *code == "de").unwrap().1;
        for old in [
            "Querungspunkt",
            "Nummernplatz",
            "Erste-Hilfe-Posten",
            "Verbotene Strecke",
            "Teilstrecke",
            "Markierte Strecke",
        ] {
            assert!(!german.contains(old), "the German catalog still says {old}");
        }
    }

    #[test]
    fn parser_text_stays_english_inside_a_german_sentence() {
        let e = ProjectError::Parse {
            path: PathBuf::from("a.toml"),
            message: "cannot parse a.toml: TOML parse error at line 3".into(),
        };
        assert_eq!(
            e.localize(&de()),
            "a.toml lässt sich nicht einlesen: TOML parse error at line 3"
        );
    }
}
