# IOF XML 3.0 Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper-courseplan export --iof-xml FILE` (and the matching MCP tool) writes the event's individual courses as IOF XML 3.0 `CourseData` for result software, mainly OE.

**Architecture:** A new module `iof_xml` in `crates/freemapper-courseplan` turns an `Event` plus the map scale into an XML string with `quick-xml`; it uses `course::course_legs` for leg lengths and the control numbers. The CLI and MCP only choose the file and report. M3 starts with this export because it needs no `freemapper-course` crate; the `.ppen` parts follow when freemapper-print delivers it *(user, 2026-11-09)*.

**Tech Stack:** Rust 1.99.1, `quick-xml 0.42.0` (already a dependency), Fluent texts via `text.rs`.

**Spec:** `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.5b (IOF XML 3.0 export), §5.8 (interfaces).

## Global Constraints

- Edition 2024, `unsafe_code = "forbid"`, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` pass.
- No new third-party dependency. Source code, comments, ADRs, plans in English; READMEs, manual, changelog bilingual (de/en); program texts in `i18n/<lang>/courseplan.ftl` for every language in the same commit.
- Y axis as the IOF standard defines it: `MapPosition` in mm at map scale, y pointing up. Project coordinates are mm, y down, so write `y = -y`.
- Output is deterministic: the creation time is a parameter, never read inside the writer. Floats are written with at most 2 decimals.
- Only `individual` courses are exported. Other kinds are skipped and named in the result; they are not an error (design §5.5).
- User-facing text (README, manual, changelog, ADR, messages longer than a short phrase) goes through the `unslop` and `humanizer` skills.
- Glossary row before code for a new term (`IOF XML`, `Export`). ADR 0033 in the same branch. Changelog entries under "Unveröffentlicht"/"Unreleased".

## Review Focus

- A course whose leg cannot be computed (unknown code): the export fails with a message naming the course, and writes no file.
- Crossing points: written with `type="CrossingPoint"`, no number, no `MapTextPosition`.
- Control used by several courses: written once under `RaceCourseData`, referenced by each course.
- Text with `&`, `<`, umlauts in titles, course and class names: escaped, UTF-8.
- Event with no courses or only non-individual courses: valid file without `Course` elements, the skipped courses reported.
- Target file exists or the disk is full: written to a temp file and renamed; the old file stays intact on error.

---

### Task 1: The `iof_xml` writer

**Files:**
- Create: `crates/freemapper-courseplan/src/iof_xml.rs`, `crates/freemapper-courseplan/src/iof_xml/tests.rs`, `tests/fixtures/iof-xml/CourseData.xsd` (official schema, source/date/licence in `tests/fixtures/README.md`), `tests/snapshots/iof_xml_luebeck.xml`
- Modify: `crates/freemapper-courseplan/src/lib.rs` (`pub mod iof_xml;`), `docs/glossary.md`

**Interfaces:**
- Consumes: `freemapper_courseplan_model::{Event, Course, CourseKind, Control, ControlKind, CourseControl, SpecialObject, MapPoint}`; `crate::course::{course_legs, control_numbers, course_length_m, LegGeometry}`.
- Produces:
  ```rust
  pub struct IofXmlExport { pub xml: String, pub skipped: Vec<SkippedCourse> }
  pub struct SkippedCourse { pub name: String, pub kind: &'static str }
  #[derive(Debug, thiserror::Error)]
  pub enum IofXmlError { #[error("course {course}: {source}")] Course { course: String, source: crate::course::CourseError } }
  pub fn export_iof_xml(event: &Event, map_scale: u32, created: &str) -> Result<IofXmlExport, IofXmlError>
  ```
  `created` is an ISO 8601 string such as `2026-11-09T12:00:00` for `createTime`.

- [ ] **Step 1: Fetch the official IOF XML 3.0 `IOF.xsd`** (https://github.com/international-orienteering-federation/datastandard-v3) into `tests/fixtures/iof-xml/`, record URL, date and licence in `tests/fixtures/README.md`. Read the `CourseData`, `RaceCourseData`, `Control`, `Course`, `CourseControl` and `ClassCourseAssignment` definitions; the element order below must match the schema, and where it does not, the schema wins (note the deviation in ADR 0033).
- [ ] **Step 2: Write failing tests** in `iof_xml/tests.rs`:
  - `root_has_namespace_version_and_creator`: root `CourseData`, `xmlns="http://www.orienteering.org/datastandard/3.0"`, `iofVersion="3.0"`, `createTime` equals the parameter, `creator="freemapper-courseplan"`.
  - `controls_are_written_once_with_mirrored_y`: control at `[10.0, 20.0]` becomes `<MapPosition x="10" y="-20" unit="mm"/>`; used by two courses it appears once in `RaceCourseData`.
  - `course_lists_start_controls_finish_with_leg_lengths`: straight 1200 m course at 1:4000 → `<Length>1200</Length>`, `CourseControl type="Start"`, `type="Control"` with `<LegLength>`, `type="Finish"`; first stop has no `LegLength`.
  - `number_position_becomes_map_text_position`: a control with `NumberPlacement` gets `<MapTextPosition x y unit="mm"/>` at the number's centre (use the same position the check and numbers module compute; a course override wins over the control's).
  - `crossing_point_has_type_and_no_number`.
  - `climb_is_written_when_set`.
  - `classes_map_to_courses`: `ClassCourseAssignment` with `ClassName` and `CourseName` for each class named in `Course.classes`.
  - `names_are_escaped`: title `A & B <1>` and class `Anfänger` round-trip through an XML parse.
  - `non_individual_courses_are_skipped_and_named`.
  - `unknown_code_names_the_course`: error `IofXmlError::Course`.
  - `output_is_deterministic_and_matches_snapshot`: the Lübeck example event produces `tests/snapshots/iof_xml_luebeck.xml` (regenerate with `UPDATE_SNAPSHOT=1`, as the rule-set snapshot does).
  - `output_validates_against_schema`: runs `xmllint --noout --schema tests/fixtures/iof-xml/IOF.xsd` on the output when `xmllint` is on `PATH`, otherwise prints a note and passes (CI installs it on Linux).
- [ ] **Step 3: Run the tests; expect failures** (`cargo test -p freemapper-courseplan iof_xml`).
- [ ] **Step 4: Implement `export_iof_xml`.** Element order: `Event/Name`, `RaceCourseData` with `Map` (`Scale`, `MapPosition` 0/0 mm), `Control` elements (Start, Finish, Control, CrossingPoint types, `Id` = code or start/finish id, `MapPosition`), `Course` elements, `ClassCourseAssignment` elements. Use `quick_xml::Writer` with indentation; numbers via a helper that formats at most 2 decimals and no `-0`. Start and finish of each course are the `SpecialObject`s named in `Course.start`/`finish`; their positions come from the specials. Geo positions (`Position lat lng`) are not written: they need `freemapper-georef` (parked, ADR 0033).
- [ ] **Step 5: Run the tests; expect all pass** (xmllint test passes or prints its note). Run clippy.
- [ ] **Step 6: Commit** `feat(iof): write IOF XML 3.0 CourseData`.

---

### Task 2: `export --iof-xml` in CLI and MCP

**Files:**
- Modify: `crates/freemapper-courseplan-cli/src/main.rs` (command `Export { project, #[arg(long, value_name = "FILE")] iof_xml: Option<PathBuf> }`), `crates/freemapper-courseplan-cli/src/mcp.rs` (tool `export_iof_xml` with `path` argument), `i18n/en/courseplan.ftl`, `i18n/de/courseplan.ftl`
- Create: `crates/freemapper-courseplan-cli/src/export_cmd.rs`
- Test: `crates/freemapper-courseplan-cli/tests/export.rs` (or beside the existing CLI tests; follow their layout)

**Interfaces:**
- Consumes: `freemapper_courseplan::iof_xml::{export_iof_xml, IofXmlExport}`; the existing project loader and atomic write helper (use the one `edit_cmd.rs` uses for the project file; reuse, do not copy).
- Produces: `pub(crate) fn export_iof_xml_file(project: &Path, target: &Path, created: &str, lang: Lang) -> Result<ExportReport, CliError>` shared by CLI and MCP. Messages: `export-iof-xml-written` (path, course count), `export-iof-xml-skipped` (course name, kind), `export-iof-xml-nothing-to-do` (`export` without a format flag).

- [ ] **Step 1: Write failing tests:** `export --iof-xml out.xml` on the example project writes the file and prints the written message with the course count (en and de); a skipped team course is printed; without a flag the command exits non-zero with `export-iof-xml-nothing-to-do`; an existing target is replaced atomically; an error (unknown code) leaves an existing target unchanged and writes nothing; the MCP tool returns `{ "path", "courses", "skipped" }` as compact JSON.
- [ ] **Step 2: Run them; expect failures.**
- [ ] **Step 3: Implement** the shared function (current local time as `created`, formatted `YYYY-MM-DDTHH:MM:SS` without a new dependency: use `std::time` and the civil-date arithmetic already in the repo if present, otherwise a small tested helper in `export_cmd.rs`), the command, the MCP tool with a description in both languages, and the Fluent messages for every language.
- [ ] **Step 4: Run `cargo test --workspace --locked`; expect all pass.** Run `cargo run -p freemapper-courseplan-cli -- export examples/luebeck-sprint/*.toml --iof-xml /tmp/x.xml` once and check the file by eye.
- [ ] **Step 5: Commit** `feat(cli): export --iof-xml, MCP tool export_iof_xml`.

---

### Task 3: Documentation, ADR, example

**Files:**
- Create: `docs/adr/0033-iof-xml-export-writes-individual-courses-from-the-project.md`, `examples/luebeck-sprint/course-data.xml` (generated by the export, no live timestamp issues: regenerate with a fixed `createTime` via the snapshot helper)
- Modify: `docs/adr/README.md`, `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.5b (what is built; status; mark open points), `README.md` + `README.en.md` (feature, command, status), `docs/manual` (de/en: command `export`), `examples/README.md` + `.en.md`, `CHANGELOG.md` + `CHANGELOG.en.md`, `docs/glossary.md`

- [ ] **Step 1: ADR 0033** records: individual courses only, Control types per schema, y mirrored, map positions only (geo positions wait for `freemapper-georef`), skipped kinds reported, timestamp injected, atomic write, schema deviations found in Task 1. Mark each statement *(user)* or *(Claude's call)*.
- [ ] **Step 2: Update the design §5.5b and the other documents** in German and English; keep READMEs equal in both languages. State plainly that the real OE import test is still open: the user imports `examples/luebeck-sprint/course-data.xml` into OE12 and reports back.
- [ ] **Step 3: Run `unslop` and `humanizer` over the new prose.**
- [ ] **Step 4: Run fmt, clippy, tests.** Commit `docs: IOF XML export (ADR 0033, manual, readmes, changelog)`.
