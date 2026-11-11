//! Program texts from Fluent files (ADR 0022).
//!
//! Provenance: the program independent part (languages, lookup with English fallback, language
//! detection, catalog checks) lives in `freemapper-text` of freemapper-core (ADR 0010 there). This
//! file holds the embedded catalogs and the `Localize` implementations of this program.
//!
//! Every user-facing text lives in `i18n/<language>/courseplan.ftl`. The files are embedded with
//! `include_str!`, so adding a language means adding a folder, listing it in `TEXTS` and
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

use freemapper_text::Catalog;
pub use freemapper_text::{FluentValue, Lang};

use freemapper_courseplan_model::{ModelError, ProjectError};

use crate::check::Place;
use crate::course::CourseError;
use crate::edit::EditError;
use crate::init::InitError;
use crate::numbers::MAX_GAP_MM;
use crate::omap::OmapError;
use crate::ppen_export::{ExportNote, PpenExportError};
use crate::ppen_import::{ImportNote, PpenImportError};
use crate::rules::RulesError;
use crate::sync::SyncError;
use crate::sync::apply::SyncNote;

/// Program texts for a value, for example an error.
///
/// Defined here and not taken from `freemapper_text::Localize`: this program implements it for
/// `ModelError` and `ProjectError` of the model crate, which the orphan rule forbids for a trait
/// from a third crate.
pub trait Localize {
    /// The text in `lang`. In English it equals `to_string()` for error types.
    fn localize(&self, lang: &Lang) -> String;
}

/// The embedded catalogs: language code and file content. English comes first.
pub static TEXTS: Catalog = Catalog::new(&[
    ("en", include_str!("../i18n/en/courseplan.ftl")),
    ("de", include_str!("../i18n/de/courseplan.ftl")),
]);

/// The languages the program has texts for, English first.
pub fn available() -> Vec<Lang> {
    TEXTS.available()
}

/// The `--lang` value as a language, or the list of available languages when it names none.
pub fn parse_lang_arg(arg: &str) -> Result<Lang, Vec<Lang>> {
    TEXTS.parse_lang_arg(arg)
}

/// Picks the language: `--lang`, else `FREEMAPPER_LANG`, else `LC_ALL`, `LC_MESSAGES`, `LANG`,
/// else English (see `freemapper_text::Catalog::detect_lang`).
pub fn detect_lang(arg: Option<&str>, env: &dyn Fn(&str) -> Option<String>) -> Lang {
    TEXTS.detect_lang(arg, env)
}

/// The text `id` in `lang`, falling back to English, and to the id itself as a last resort.
pub fn text(lang: &Lang, id: &str, args: &[(&str, FluentValue)]) -> String {
    TEXTS.text(lang, id, args)
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
            ModelError::BlankName { what, value } => text(
                lang,
                "error-model-blank-name",
                &[
                    ("what", v(word(lang, "word-name", what))),
                    ("value", v(value)),
                ],
            ),
            ModelError::InvalidCharacter { what, point } => text(
                lang,
                "error-model-invalid-character",
                &[
                    ("what", v(word(lang, "word-name", what))),
                    ("point", v(point)),
                ],
            ),
            ModelError::TooLong { what, limit, len } => text(
                lang,
                "error-model-too-long",
                &[
                    ("what", v(word(lang, "word-name", what))),
                    ("limit", v(limit)),
                    ("len", v(len)),
                ],
            ),
            ModelError::BlankNote { rule } => {
                text(lang, "error-model-blank-note", &[("name", v(rule))])
            }
            ModelError::CodeClashesWithSpecial(code) => text(
                lang,
                "error-model-code-clashes-with-special",
                &[("code", v(code))],
            ),
            ModelError::DuplicateCourseInSpecial { id, course } => text(
                lang,
                "error-model-duplicate-course-in-special",
                &[("id", v(id)), ("course", v(course))],
            ),
            ModelError::TooFewPoints { id, kind, needed } => {
                // The words of `special add`, which name the out-of-bounds area without "area".
                let kind_id = match *kind {
                    "out-of-bounds area" => "out-of-bounds",
                    other => other,
                };
                text(
                    lang,
                    "error-model-too-few-points",
                    &[
                        ("kind", v(word(lang, "word-special", kind_id))),
                        ("id", v(id)),
                        ("needed", FluentValue::from(*needed)),
                    ],
                )
            }
            ModelError::LinkToUnknownControl { course, a, b, code } => text(
                lang,
                "error-model-link-to-unknown-control",
                &[
                    ("course", v(course)),
                    ("a", v(a)),
                    ("b", v(b)),
                    ("code", v(code)),
                ],
            ),
            ModelError::LinkToItself { course, code } => text(
                lang,
                "error-model-link-to-itself",
                &[("course", v(course)), ("code", v(code))],
            ),
            ModelError::DuplicateLink { course, a, b } => text(
                lang,
                "error-model-duplicate-link",
                &[("course", v(course)), ("a", v(a)), ("b", v(b))],
            ),
            ModelError::LinkBranches { course, code } => text(
                lang,
                "error-model-link-branches",
                &[("course", v(course)), ("code", v(code))],
            ),
            ModelError::LinkCycle { course, a, b } => text(
                lang,
                "error-model-link-cycle",
                &[("course", v(course)), ("a", v(a)), ("b", v(b))],
            ),
            ModelError::ScoreSettingOutOfRange { what, reason } => text(
                lang,
                "error-model-score-setting-out-of-range",
                &[
                    ("what", v(what)),
                    ("reason", v(word(lang, "word-score-problem", reason))),
                ],
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
            ProjectError::ChangedMeanwhile { path } => text(
                lang,
                "error-project-changed",
                &[("path", v(path.display()))],
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

impl Localize for PpenImportError {
    fn localize(&self, lang: &Lang) -> String {
        use freemapper_course::FitError;
        match self {
            PpenImportError::Read(error) => {
                text(lang, "error-import-ppen-invalid", &[("source", v(error))])
            }
            PpenImportError::Fit(FitError::ScaleMismatch { file, map }) => text(
                lang,
                "error-import-ppen-scale-mismatch",
                &[("file", v(file)), ("map", v(map))],
            ),
            PpenImportError::Fit(FitError::MapScaleUnknown { file }) => text(
                lang,
                "error-import-ppen-map-scale-unknown",
                &[("file", v(file))],
            ),
            PpenImportError::Fit(FitError::OutsideMap { inside, total }) => text(
                lang,
                "error-import-ppen-outside-map",
                &[("inside", v(inside)), ("total", v(total))],
            ),
            PpenImportError::Fit(other) => {
                text(lang, "error-import-ppen-fit", &[("source", v(other))])
            }
        }
    }
}

impl Localize for ImportNote {
    fn localize(&self, lang: &Lang) -> String {
        let t = |id: &str, args: &[(&str, FluentValue)]| text(lang, id, args);
        match self {
            ImportNote::UnknownElement(what) => {
                t("import-ppen-note-unknown-element", &[("what", v(what))])
            }
            ImportNote::Skipped { course, why } => t(
                "import-ppen-note-skipped",
                &[
                    ("course", v(course)),
                    ("why", v(word(lang, "word-import-why", why))),
                ],
            ),
            ImportNote::GapsNotSupported { course } => {
                t("import-ppen-note-gaps", &[("course", v(course))])
            }
            ImportNote::UnmappedSymbol {
                control,
                column,
                id,
            } => t(
                "import-ppen-note-unmapped-symbol",
                &[
                    ("control", v(control)),
                    ("column", v(column)),
                    ("id", v(id)),
                ],
            ),
            ImportNote::NumberIntoCircle { control } => t(
                "import-ppen-note-number-into-circle",
                &[("control", v(control))],
            ),
            ImportNote::Fit(note) => {
                use freemapper_course::FitNote;
                match note {
                    FitNote::ScaleUnchecked => t("import-ppen-note-fit-scale-unchecked", &[]),
                    FitNote::OrientationAssumed => {
                        t("import-ppen-note-fit-orientation-assumed", &[])
                    }
                    FitNote::AxisAgainstFormat { .. } => {
                        t("import-ppen-note-fit-axis-against-format", &[])
                    }
                    // `worth_reporting` keeps these out of an import; a text is here for completeness.
                    FitNote::YUpDetected
                    | FitNote::GeorefChecked { .. }
                    | FitNote::GeorefSkipped(_)
                    | FitNote::GeorefCheckOff
                    | FitNote::PlacedByGeoref(_) => t("import-ppen-note-fit-other", &[]),
                }
            }
            ImportNote::MapExchange { course, control } => t(
                "import-ppen-note-map-exchange",
                &[("course", v(course)), ("control", v(control))],
            ),
            ImportNote::MapExchangeAtControl { course, control } => t(
                "import-ppen-note-map-exchange-at-control",
                &[("course", v(course)), ("control", v(control))],
            ),
            ImportNote::SecondaryTitle { course, title } => t(
                "import-ppen-note-secondary-title",
                &[("course", v(course)), ("title", v(title))],
            ),
            ImportNote::ColumnFText {
                control,
                text: what,
            } => t(
                "import-ppen-note-column-f-text",
                &[("control", v(control)), ("text", v(what))],
            ),
            ImportNote::StartFinishDescription { id } => t(
                "import-ppen-note-start-finish-description",
                &[("id", v(id))],
            ),
            ImportNote::MapIssueAsStart { id } => {
                t("import-ppen-note-map-issue-as-start", &[("id", v(id))])
            }
            ImportNote::SpecialSkipped { kind } => t(
                "import-ppen-note-special-skipped",
                &[("kind", v(word(lang, "word-import-special", kind)))],
            ),
            ImportNote::SpecialWithoutCourse { kind } => t(
                "import-ppen-note-special-without-course",
                &[("kind", v(word(lang, "word-import-special", kind)))],
            ),
            ImportNote::SpecialOffMap { kind } => t(
                "import-ppen-note-special-off-map",
                &[("kind", v(word(lang, "word-import-special", kind)))],
            ),
            ImportNote::CourseRenamed { from, to } => t(
                "import-ppen-note-course-renamed",
                &[("from", v(from)), ("to", v(to))],
            ),
            ImportNote::ControlWithoutPosition { control } => t(
                "import-ppen-note-control-without-position",
                &[("control", v(control))],
            ),
            ImportNote::NumberGapClamped { control } => t(
                "import-ppen-note-number-gap-clamped",
                &[("control", v(control)), ("max", v(MAX_GAP_MM))],
            ),
            ImportNote::SpecialCourseAmbiguous { kind, name } => t(
                "import-ppen-note-special-course-ambiguous",
                &[
                    ("kind", v(word(lang, "word-import-special", kind))),
                    ("name", v(name)),
                ],
            ),
            ImportNote::Forked { family, variants } => t(
                "import-ppen-note-forked",
                &[("family", v(family)), ("variants", v(variants.join(", ")))],
            ),
            ImportNote::NotesTruncated => t("import-ppen-note-notes-truncated", &[]),
        }
    }
}

impl Localize for ExportNote {
    fn localize(&self, lang: &Lang) -> String {
        use freemapper_course::KeptObject;
        let t = |id: &str, args: &[(&str, FluentValue)]| text(lang, id, args);
        match self {
            ExportNote::CourseSkipped { course, kind } => t(
                "export-ppen-note-course-skipped",
                &[
                    ("course", v(course)),
                    ("kind", v(word(lang, "word-kind", kind))),
                ],
            ),
            ExportNote::SpecialSkipped { id, why } => t(
                "export-ppen-note-special-skipped",
                &[
                    ("id", v(id)),
                    ("why", v(word(lang, "word-export-why", why))),
                ],
            ),
            ExportNote::SymbolWithoutPpenId {
                control,
                column,
                id,
            } => t(
                "export-ppen-note-symbol-without-id",
                &[
                    ("control", v(control)),
                    ("column", v(column)),
                    ("id", v(id)),
                ],
            ),
            ExportNote::SymbolNotInStandard {
                control,
                column,
                id,
                standard,
            } => t(
                "export-ppen-note-symbol-not-in-standard",
                &[
                    ("control", v(control)),
                    ("column", v(column)),
                    ("id", v(id)),
                    ("standard", v(standard)),
                ],
            ),
            ExportNote::SymbolWrongColumn {
                control,
                column,
                id,
            } => t(
                "export-ppen-note-symbol-wrong-column",
                &[
                    ("control", v(control)),
                    ("column", v(column)),
                    ("id", v(id)),
                ],
            ),
            ExportNote::SymbolVariantChosen {
                control,
                column,
                id,
                written,
            } => t(
                "export-ppen-note-symbol-variant-chosen",
                &[
                    ("control", v(control)),
                    ("column", v(column)),
                    ("id", v(id)),
                    ("written", v(written)),
                ],
            ),
            ExportNote::CodeNotStored { code } => {
                t("export-ppen-note-code-not-stored", &[("code", v(code))])
            }
            ExportNote::FlaggingImplied { course, from, to } => t(
                "export-ppen-note-flagging-implied",
                &[("course", v(course)), ("from", v(from)), ("to", v(to))],
            ),
            ExportNote::PartialFlaggingDropped { course, from, to } => t(
                "export-ppen-note-partial-flagging-dropped",
                &[("course", v(course)), ("from", v(from)), ("to", v(to))],
            ),
            ExportNote::Kept(KeptObject::Control { id, code }) => t(
                "export-ppen-kept-control",
                &[("id", v(id)), ("code", v(code))],
            ),
            ExportNote::Kept(KeptObject::Course { id, name }) => t(
                "export-ppen-kept-course",
                &[("id", v(id)), ("name", v(name))],
            ),
            ExportNote::Kept(KeptObject::Special { id, kind }) => t(
                "export-ppen-kept-special",
                &[
                    ("id", v(id)),
                    ("kind", v(word(lang, "word-import-special", kind))),
                ],
            ),
        }
    }
}

impl Localize for SyncNote {
    fn localize(&self, lang: &Lang) -> String {
        let t = |id: &str, args: &[(&str, FluentValue)]| text(lang, id, args);
        match self {
            SyncNote::DeletionKept { control, course } => t(
                "sync-note-deletion-kept",
                &[("control", v(control)), ("course", v(course))],
            ),
            SyncNote::LegChangeDropped { from, to } => t(
                "sync-note-leg-change-dropped",
                &[("from", v(from)), ("to", v(to))],
            ),
            SyncNote::NewNameTaken { from, to } => t(
                "sync-note-new-name-taken",
                &[("from", v(from)), ("to", v(to))],
            ),
            SyncNote::BaseIgnored { why } => t(
                "sync-note-base-ignored",
                &[("why", v(word(lang, "word-sync-why", why)))],
            ),
            SyncNote::FirstSync => t("sync-note-first-sync", &[]),
            SyncNote::BaseRepaired => t("sync-note-base-repaired", &[]),
            SyncNote::NotWritten(note) => {
                t("sync-note-not-written", &[("note", v(note.localize(lang)))])
            }
        }
    }
}

/// The words of a rendered sync value that a front end translates.
const SYNC_WORDS: [&str; 7] = [
    "deleted",
    "none",
    "empty",
    "control",
    "crossing point",
    "start",
    "finish",
];

/// Whether `raw` is one of the words [`sync_value`] translates. A name that is one is rendered in quotes.
pub fn is_sync_word(raw: &str) -> bool {
    SYNC_WORDS.contains(&raw)
}

/// A value of a sync conflict or change as the user reads it. The values are rendered in English; the
/// whole-value words (`deleted`, the control kinds, `none`, `empty`) are translated, in a value of several
/// parts (`35; control; empty`) each part, any other value (positions, stop lists) is language-neutral and
/// stays.
pub fn sync_value(lang: &Lang, raw: &str) -> String {
    raw.split("; ")
        .map(|part| {
            let prefix = if matches!(part, "deleted" | "none" | "empty") {
                "word-sync-value"
            } else {
                "word-sync-kind"
            };
            word(lang, prefix, part)
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// The text of a refused or failed sync of the linked file `path`.
pub fn sync_error(e: &SyncError, lang: &Lang, path: &str) -> String {
    let t = |id: &str, args: &[(&str, FluentValue)]| text(lang, id, args);
    match e {
        SyncError::Base(e) => t(
            "error-sync-base",
            &[("path", v(path)), ("source", v(e.localize(lang)))],
        ),
        SyncError::Theirs(e) => t(
            "error-sync-theirs",
            &[("path", v(path)), ("source", v(e.localize(lang)))],
        ),
        SyncError::Export(e) => t(
            "error-sync-export",
            &[
                ("path", v(path)),
                ("source", v(ppen_export_error(e, lang, path))),
            ],
        ),
        SyncError::UnknownVersion(version) => t(
            "error-sync-unknown-version",
            &[("path", v(path)), ("version", v(version))],
        ),
        SyncError::Unmatched(kept) => t(
            "error-sync-unmatched",
            &[
                ("count", v(kept.len())),
                ("path", v(path)),
                (
                    "list",
                    v(kept
                        .iter()
                        .map(|k| ExportNote::Kept(k.clone()).localize(lang))
                        .collect::<Vec<_>>()
                        .join("; ")),
                ),
            ],
        ),
        SyncError::Invalid(errors) => t(
            "error-sync-invalid",
            &[(
                "errors",
                v(errors
                    .iter()
                    .map(|e| e.localize(lang))
                    .collect::<Vec<_>>()
                    .join("; ")),
            )],
        ),
        SyncError::Verify(keys) => t("error-sync-verify", &[("list", v(keys.join(", ")))]),
    }
}

impl Localize for SyncError {
    fn localize(&self, lang: &Lang) -> String {
        sync_error(self, lang, "")
    }
}

/// The text of a failed `.ppen` export to `path`; an unusable base file names it, since one call can
/// write two files.
pub fn ppen_export_error(e: &PpenExportError, lang: &Lang, path: &str) -> String {
    use freemapper_course::PpenWriteError as W;
    let base = |source: String| {
        text(
            lang,
            "error-export-ppen-base",
            &[("path", v(path)), ("source", v(source))],
        )
    };
    match e {
        PpenExportError::Base(e) => base(e.localize(lang)),
        PpenExportError::Write(e) => match e {
            W::Base(e) => base(e.to_string()),
            W::UnknownVersion(version) => text(
                lang,
                "error-export-ppen-unknown-version",
                &[("version", v(version))],
            ),
            W::Unsupported(what) => {
                text(lang, "error-export-ppen-unsupported", &[("what", v(what))])
            }
            W::Invalid(what) => text(lang, "error-export-ppen-invalid", &[("what", v(what))]),
            W::SharedLeg {
                from,
                to,
                first,
                second,
            } => text(
                lang,
                "error-export-ppen-shared-leg",
                &[
                    ("from", v(from)),
                    ("to", v(to)),
                    ("first", v(first)),
                    ("second", v(second)),
                ],
            ),
            W::SharedCircleGaps {
                control,
                scale,
                first,
                second,
            } => text(
                lang,
                "error-export-ppen-shared-circle-gaps",
                &[
                    ("control", v(control)),
                    ("scale", v(scale)),
                    ("first", v(first)),
                    ("second", v(second)),
                ],
            ),
            W::Limit(what) => text(lang, "error-export-ppen-limit", &[("what", v(what))]),
        },
        PpenExportError::CircleGapLimit { limit } => text(
            lang,
            "error-export-ppen-circle-gap-limit",
            &[("limit", v(limit))],
        ),
    }
}

fn place_word(lang: &Lang, place: &Place) -> String {
    text(lang, &format!("word-place-{}", place.as_str()), &[])
}

/// "a, b or c" of place kinds.
fn place_list(lang: &Lang, places: &[Place]) -> String {
    let words: Vec<String> = places.iter().map(|p| place_word(lang, p)).collect();
    match words.split_last() {
        Some((last, rest)) if !rest.is_empty() => {
            let or = text(lang, "word-or", &[]);
            format!("{} {or} {last}", rest.join(", "))
        }
        _ => words.concat(),
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
            EditError::NameTaken(name) => text(lang, "error-edit-name-taken", &[("name", v(name))]),
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
            EditError::SpecialWouldCoverAllCourses { course, specials } => text(
                lang,
                "error-edit-special-would-cover-all-courses",
                &[
                    ("course", v(course)),
                    ("specials", v(specials.join(", "))),
                    ("count", FluentValue::from(specials.len())),
                ],
            ),
            EditError::StartOrFinishHasNoCourses(id) => text(
                lang,
                "error-edit-start-or-finish-has-no-courses",
                &[("id", v(id))],
            ),
            EditError::UnknownRule { rule, profile } => text(
                lang,
                "error-edit-unknown-rule",
                &[("name", v(rule)), ("id", v(profile))],
            ),
            EditError::FindingNeedsPlace { rule, places } => text(
                lang,
                "error-edit-finding-needs-place",
                &[("name", v(rule)), ("places", v(place_list(lang, places)))],
            ),
            EditError::FindingHasOnePlace { rule } => text(
                lang,
                "error-edit-finding-has-one-place",
                &[("name", v(rule))],
            ),
            EditError::FindingWrongPlace {
                rule,
                place,
                places,
            } => text(
                lang,
                "error-edit-finding-wrong-place",
                &[
                    ("name", v(rule)),
                    ("place", v(place_word(lang, place))),
                    ("places", v(place_list(lang, places))),
                ],
            ),
            EditError::AcceptedNotFound { rule } => {
                text(lang, "error-edit-accepted-not-found", &[("name", v(rule))])
            }
            EditError::StartWindowZero => text(lang, "error-edit-start-window-zero", &[]),
            EditError::TooManyBends { course, limit } => text(
                lang,
                "error-edit-too-many-bends",
                &[("course", v(course)), ("limit", v(limit))],
            ),
            EditError::BlankRuleCategory { class } => text(
                lang,
                "error-edit-blank-rule-category",
                &[("name", v(class))],
            ),
            EditError::BadSymbol { column, id } => text(
                lang,
                "error-edit-bad-symbol",
                &[("column", v(column)), ("id", v(id))],
            ),
            EditError::Rules(e) => e.localize(lang),
        }
    }
}

impl Localize for RulesError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            RulesError::UnknownProfile { id, available } => text(
                lang,
                "error-rules-unknown-profile",
                &[("id", v(id)), ("list", v(available))],
            ),
            RulesError::NotABase(id) => text(lang, "error-rules-not-a-base", &[("id", v(id))]),
            RulesError::NotAnOverlay(id) => {
                text(lang, "error-rules-not-an-overlay", &[("id", v(id))])
            }
            RulesError::OverlayForAnotherBase {
                overlay,
                extends,
                base,
            } => text(
                lang,
                "error-rules-overlay-for-another-base",
                &[("id", v(overlay)), ("other", v(extends)), ("name", v(base))],
            ),
            RulesError::Malformed { profile, message } => text(
                lang,
                "error-rules-malformed",
                &[("id", v(profile)), ("message", v(message))],
            ),
            RulesError::UnknownKey { profile, rule, key } => text(
                lang,
                "error-rules-unknown-key",
                &[("id", v(profile)), ("name", v(rule)), ("key", v(key))],
            ),
            RulesError::UnknownRule { overlay, rule } => text(
                lang,
                "error-rules-unknown-rule",
                &[("id", v(overlay)), ("name", v(rule))],
            ),
            RulesError::Ambiguous { key, value, rules } => text(
                lang,
                "error-rules-ambiguous",
                &[
                    ("key", v(key)),
                    ("value", v(value)),
                    ("list", v(rules.join(", "))),
                ],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::decide::KindWord;
    use freemapper_courseplan_model::{ControlCode, MapPoint};
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn de() -> Lang {
        parse_lang_arg("de").unwrap()
    }

    /// Sample values for every variable the catalogs use.
    fn sample_args() -> Vec<(&'static str, FluentValue<'static>)> {
        [
            "code",
            "name",
            "id",
            "error",
            "excess",
            "expected",
            "high",
            "highest",
            "hull",
            "low",
            "missing",
            "penalty",
            "tolerance",
            "variation",
            "wanted",
            "a",
            "b",
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
            "len",
            "limit",
            "version",
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
            "needed",
            "max",
            "specials",
            "values",
            "crossings",
            "discipline",
            "scale",
            "warnings",
            "hints",
            "accepted",
            "skipped",
            "note",
            "rule",
            "location",
            "reason",
            "doc",
            "section",
            "port",
            "first",
            "first_kind",
            "second",
            "second_kind",
            "first_symbol",
            "second_symbol",
            "metres",
            "limit",
            "stop",
            "stop_kind",
            "place",
            "places",
            "legs",
            "leg",
            "overlay",
            "unmatched",
            "minutes",
            "degrees",
            "min_length",
            "min",
            "max",
            "symbol",
            "start",
            "shared",
            "classes",
            "category",
            "categories",
            "percent",
            "reference",
            "rows",
            "easy",
            "hard",
            "actual",
            "length",
            "climb",
            "target",
            "file",
            "map",
            "inside",
            "total",
            "what",
            "why",
            "note",
            "from",
            "to",
            "title",
            "family",
            "variants",
            "name",
            "id",
            "column",
            "text",
            "controls",
            "courses",
            "course",
            "control",
            "point",
            "kind",
            "path",
            "source",
            "standard",
            "written",
            "ours",
            "theirs",
            "both",
            "side",
            "base",
            "base_scale",
        ]
        .iter()
        .map(|n| (*n, v("sample")))
        .collect()
    }

    #[test]
    fn sync_values_and_notes_speak_german() {
        let de = de();
        assert_eq!(sync_value(&de, "deleted"), "gelöscht");
        assert_eq!(
            sync_value(&de, "35; control; -1.2, 3 mm; empty"),
            "35; Posten; -1.2, 3 mm; leer"
        );
        // A name that reads like a word is quoted by the renderer and stays.
        assert_eq!(sync_value(&de, "\"none\""), "\"none\"");
        assert_eq!(sync_value(&de, "crossing point"), "Querungsstelle");
        assert_eq!(sync_value(&de, "-1.200, 12.400 mm"), "-1.200, 12.400 mm");
        let en = Lang::english();
        assert_eq!(sync_value(&en, "finish"), "finish");
        for note in [
            SyncNote::BaseRepaired,
            SyncNote::FirstSync,
            SyncNote::BaseIgnored { why: "hash" },
            SyncNote::DeletionKept {
                control: s("36"),
                course: s("A"),
            },
        ] {
            for lang in available() {
                let out = note.localize(&lang);
                assert!(!out.is_empty() && !out.starts_with("sync-"), "{out}");
            }
        }
        assert!(
            SyncNote::BaseIgnored { why: "hash" }
                .localize(&de)
                .contains("Hashwert")
        );
    }

    #[test]
    fn catalogs_have_no_problems() {
        let problems = TEXTS.problems(&sample_args());
        assert!(problems.is_empty(), "{problems:#?}");
    }

    fn ids_in_source(source: &str) -> BTreeSet<String> {
        let mut ids =
            freemapper_text::ids_in_source(source, &["error-", "cli-", "word-", "mcp-", "check-"]);
        for excluded in [
            "word-kind",
            "word-what",
            "word-name",
            "word-special",
            "word-score-problem",
            "word-import-why",
            "word-import-special",
            "word-export-why",
            "word-sync-why",
            "word-sync-value",
            "word-sync-kind",
        ] {
            ids.remove(excluded);
        }
        ids
    }

    /// Every non-test source file directly under `src/check/`; the guard test below fails when a
    /// new module is missing here, so its message ids are scanned.
    const CHECK_SOURCES: [(&str, &str); 13] = [
        ("mod.rs", include_str!("check/mod.rs")),
        ("codes.rs", include_str!("check/codes.rs")),
        ("context.rs", include_str!("check/context.rs")),
        ("distance.rs", include_str!("check/distance.rs")),
        ("geometry.rs", include_str!("check/geometry.rs")),
        ("legs.rs", include_str!("check/legs.rs")),
        ("numbers.rs", include_str!("check/numbers.rs")),
        ("score.rs", include_str!("check/score.rs")),
        ("start.rs", include_str!("check/start.rs")),
        ("stops.rs", include_str!("check/stops.rs")),
        ("tables.rs", include_str!("check/tables.rs")),
        ("tests.rs", include_str!("check/tests.rs")),
        ("testing.rs", include_str!("check/testing.rs")),
    ];

    #[test]
    fn every_check_module_is_scanned_for_message_ids() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/check");
        let mut missing = Vec::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "rs") {
                let name = path.file_name().unwrap().to_str().unwrap().to_owned();
                if !CHECK_SOURCES.iter().any(|(n, _)| *n == name) {
                    missing.push(name);
                }
            }
        }
        assert!(
            missing.is_empty(),
            "add to CHECK_SOURCES in text.rs: {missing:?}"
        );
    }

    #[test]
    fn list_and_table_check_texts_render_in_every_language() {
        let n = |x: f64| FluentValue::from(x);
        let row = |lang: &Lang| {
            text(
                lang,
                "check-nor-lkm-row",
                &[
                    ("categories", v("H-12")),
                    ("target", n(3.9)),
                    ("min", n(3.7)),
                    ("max", n(4.1)),
                ],
            )
        };
        let en = row(&parse_lang_arg("en").unwrap());
        assert!(
            en.contains("H-12") && en.contains("3.9") && en.contains("4.1"),
            "{en}"
        );
        let de = row(&parse_lang_arg("de").unwrap());
        assert!(de.contains("H-12") && de.contains("Ziel-Lkm"), "{de}");
        for lang in available() {
            let empty = text(
                &lang,
                "check-skip-length-reference-empty",
                &[("course", v("HE"))],
            );
            assert!(empty.contains("HE") && empty.contains('0'), "{empty}");
            let unknown = text(
                &lang,
                "check-skip-category-not-in-table",
                &[("categories", v("D-99"))],
            );
            assert!(unknown.contains("D-99"), "{unknown}");
        }
    }

    #[test]
    fn ids_used_in_code_exist() {
        let mut ids = ids_in_source(include_str!("text.rs"));
        for (_, source) in CHECK_SOURCES {
            ids.extend(ids_in_source(source));
        }
        assert!(
            ids.contains("check-skip-print-scale"),
            "the scanner sees the check module"
        );
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
        for what in [
            "control-code",
            "special-object-id",
            "course-name",
            "class-name",
        ] {
            ids.insert(format!("word-name-{what}"));
        }
        for problem in [
            "must-be-1-to-9",
            "must-be-more-than-0",
            "low-is-above-high",
            "must-not-exceed-100",
            "must-be-below-the-good-pace",
        ] {
            ids.insert(format!("word-score-problem-{problem}"));
        }
        for place in Place::ALL {
            ids.insert(format!("word-place-{}", place.as_str()));
        }
        for kind in [
            "marked-route",
            "forbidden-route",
            "out-of-bounds",
            "dangerous-area",
        ] {
            ids.insert(format!("word-special-{kind}"));
        }
        for why in [
            "variations",
            "alternative-controls",
            "score-course",
            "no-start",
            "no-finish",
            "start-or-finish-inside-the-course",
            "no-name",
        ] {
            ids.insert(format!("word-import-why-{why}"));
        }
        for why in [
            "marked-route",
            "forbidden-route-with-several-points",
            "no-course-left",
        ] {
            ids.insert(format!("word-export-why-{why}"));
        }
        for kind in [
            "out-of-bounds",
            "dangerous-area",
            "forbidden-route",
            "boundary",
            "marked-route",
            "first-aid",
            "water",
            "text",
            "line",
            "white-out",
        ] {
            ids.insert(format!("word-import-special-{kind}"));
        }
        for why in ["missing", "other-link", "hash"] {
            ids.insert(format!("word-sync-why-{why}"));
        }
        for value in ["deleted", "none", "empty"] {
            ids.insert(format!("word-sync-value-{value}"));
        }
        for kind in [
            KindWord::Control,
            KindWord::CrossingPoint,
            KindWord::Start,
            KindWord::Finish,
        ] {
            ids.insert(format!(
                "word-sync-kind-{}",
                kind.as_str().replace(' ', "-")
            ));
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
        let english = TEXTS.message_ids("en");
        for id in &ids {
            assert!(english.contains(id), "{id} not in the English file");
        }
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
            ModelError::BlankName {
                what: "control code",
                value: s(" 31"),
            },
            ModelError::InvalidCharacter {
                what: "control code",
                point: s("U+202E"),
            },
            ModelError::TooLong {
                what: "course name",
                limit: 200,
                len: 201,
            },
            ModelError::BlankNote { rule: s("r") },
            ModelError::CodeClashesWithSpecial(code("S1")),
            ModelError::DuplicateCourseInSpecial {
                id: s("w1"),
                course: s("A"),
            },
            ModelError::TooFewPoints {
                id: s("o1"),
                kind: "out-of-bounds area",
                needed: 3,
            },
            ModelError::TooFewPoints {
                id: s("r1"),
                kind: "forbidden route",
                needed: 1,
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
            ProjectError::ChangedMeanwhile { path: path() },
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
            EditError::NameTaken(s("A")),
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
            EditError::SpecialWouldCoverAllCourses {
                course: s("A"),
                specials: vec![s("W1")],
            },
            EditError::SpecialWouldCoverAllCourses {
                course: s("A"),
                specials: vec![s("W1"), s("M2")],
            },
            EditError::StartOrFinishHasNoCourses(s("S1")),
            EditError::UnknownRule {
                rule: s("wkb2026.no.such"),
                profile: s("wkb-2026 + nor-bb-2021"),
            },
            EditError::FindingNeedsPlace {
                rule: s("wkb2026.distance.min"),
                places: vec![Place::Controls],
            },
            EditError::FindingHasOnePlace {
                rule: s("wkb2026.distance.min"),
            },
            EditError::FindingWrongPlace {
                rule: s("wkb2026.distance.min"),
                place: Place::Course,
                places: vec![Place::Controls],
            },
            EditError::AcceptedNotFound {
                rule: s("wkb2026.distance.min"),
            },
            EditError::StartWindowZero,
            EditError::TooManyBends {
                course: s("A"),
                limit: 200,
            },
            EditError::BlankRuleCategory { class: s("H21") },
            EditError::BadSymbol {
                column: 'D',
                id: s("12.1N"),
            },
            EditError::Rules(RulesError::NotABase(s("nor-bb-2021"))),
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
        for e in [
            RulesError::UnknownProfile {
                id: s("wkb-2025"),
                available: s("wkb-2026, nor-bb-2021"),
            },
            RulesError::NotABase(s("nor-bb-2021")),
            RulesError::NotAnOverlay(s("wkb-2026")),
            RulesError::OverlayForAnotherBase {
                overlay: s("nor-bb-2021"),
                extends: s("wkb-2026"),
                base: s("iof-2026"),
            },
            RulesError::Malformed {
                profile: s("wkb-2026"),
                message: s("format = 1 is missing"),
            },
            RulesError::UnknownKey {
                profile: s("wkb-2026"),
                rule: s("wkb2026.x.y"),
                key: s("no_such_check"),
            },
            RulesError::UnknownRule {
                overlay: s("nor-bb-2021"),
                rule: s("wkb2026.x.y"),
            },
            RulesError::Ambiguous {
                key: s("min_distance"),
                value: s("metres"),
                rules: vec![s("wkb2026.distance.min"), s("wkb2026.distance.min_half")],
            },
        ] {
            check(&e);
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

    #[test]
    fn a_special_that_would_cover_all_courses_is_named_in_german() {
        let one = EditError::SpecialWouldCoverAllCourses {
            course: s("A"),
            specials: vec![s("W1")],
        };
        assert_eq!(
            one.localize(&de()),
            "Bahn A ist die einzige Bahn von W1; ohne sie gälte W1 für alle Bahnen. Ordne W1 \
             zuerst einer anderen Bahn zu oder lösche es"
        );
        let two = EditError::SpecialWouldCoverAllCourses {
            course: s("A"),
            specials: vec![s("W1"), s("M2")],
        };
        assert!(
            two.localize(&de()).contains("W1, M2"),
            "{}",
            two.localize(&de())
        );
        assert!(!two.localize(&de()).contains("gälte "));
    }

    /// The German catalog uses the glossary terms (ruling R13), not the older wording.
    #[test]
    fn german_catalog_uses_the_glossary_terms() {
        let german = include_str!("../i18n/de/courseplan.ftl");
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
