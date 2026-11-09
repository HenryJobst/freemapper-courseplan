# .ppen Import Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper-courseplan init <map> --ppen <file>` and `import <project> --ppen <file>` (and the MCP tool `import_ppen`) read a Purple Pen event into a courseplan project: controls, descriptions, courses, bends, taped legs, number positions, special objects.

**Architecture:** courseplan stops keeping its own copy of the OOM reader and uses `freemapper-omap` from freemapper-core; the `.ppen` is read and fitted to the map by `freemapper-course` (core v0.3.0, owner freemapper-print), then a new module `ppen_import` in `crates/freemapper-courseplan` maps the `FittedEvent` onto the courseplan model. The `.ppen` writer and the sync follow in later plans (M3b, M3c); the writer is mine but lives in `freemapper-course` (agreed with freemapper-print, 2026-11-09).

**Tech Stack:** Rust 1.99.1; git dependencies `freemapper-omap` and `freemapper-course` pinned to tag `v0.3.0` plus the local `[patch]` workflow (core ADR 0006).

**Spec:** `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.5 (`.ppen` import), §5.5a, §5.8; core ADR 0013 (`docs/adr/0013-the-course-crate.md` in freemapper-core).

## Global Constraints

- Edition 2024, `unsafe_code = "forbid"`; `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` pass. Own crates are exempt from the two-week rule (ADR 0006 note).
- Source, comments, ADRs, plans English; READMEs, manual, changelog bilingual; program texts in `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl` in the same commit. User-facing prose goes through `unslop` and `humanizer`. German texts: "Bundesland", "abgeflaggte Strecke".
- Glossary row before code (`Import`, `Purple Pen id`). ADR 0034 in the same branch. Changelog under "Unveröffentlicht"/"Unreleased".
- Project coordinates: `MapPoint` mm, y down. `freemapper-course` returns `MapCoord` in 1/1000 mm, y down after `fit`; a `.ppen` stores `number-location` with y up, so `fit` has already flipped offsets (core ADR 0013).
- Never copy code from `freemapper-course`; if it lacks something, ask freemapper-print over SendMessage.
- Unsupported Purple Pen content (variations, alternative controls, gaps, special kinds without a counterpart) is never silent: each case gives an import note listed in the command output.

## Review Focus

- Map scale in the file differs from the map's: `fit` refuses (`ScaleMismatch`); the command exits non-zero, names both scales, writes nothing.
- A `.ppen` written by Purple Pen 4.0 or with a newer format: unknown elements arrive as notes, not as errors.
- Control codes that already exist in the project (`import` into a non-empty project): refused without `--replace`; with it the whole event content is replaced atomically.
- Course with variations/forks: skipped and named, other courses still import.
- Circle gaps and leg gaps (not in the courseplan model yet): noted per course, geometry otherwise intact.
- Descriptions with standard 2004: ids map to IOF 2024 ids or are kept and noted as unmapped; a control without description imports with an empty description.
- Number offset that points into the circle (negative gap): imported with the computed placement and a note.

---

### Task 1: Use `freemapper-omap` and `freemapper-course` from core

**Files:**
- Modify: `Cargo.toml` (workspace dependencies `freemapper-omap`, `freemapper-course` = git tag `v0.3.0`, `default-features` kept so `fit` is on; local `[patch]` block commented as in the existing pattern), `crates/freemapper-courseplan/Cargo.toml`, every `use crate::omap::…` / `freemapper_courseplan::omap` site in the three crates and tests
- Delete: `crates/freemapper-courseplan/src/omap/` (our copy of print's reader), after moving what is ours (`SymbolSet` extras only if the core crate lacks them: check first)
- Modify: `docs/adr/0006-*.md` note unchanged; new ADR text goes into ADR 0034 (Task 4)

**Interfaces:**
- Consumes: `freemapper_omap::{MapDocument, MapGeoref, MapObject, MapObjectKind, MapPart, SymbolSet, OmapError, coord::MapCoord, read…}` (read the crate's `lib.rs` for the exact reader entry point).
- Produces: the same public paths courseplan code used before, re-exported from `crates/freemapper-courseplan/src/omap.rs` (`pub use freemapper_omap::*;`) so call sites change in one place, and the field `combined_scale_factor`/`map_ref_mm` additions of core's `MapGeoref` handled in `check/testing.rs` and wherever `MapGeoref` literals are built.

- [ ] **Step 1: Write a failing test** `omap_comes_from_core`: `std::any::type_name::<crate::omap::MapDocument>()` starts with `freemapper_omap::` (it fails while the local copy exists).
- [ ] **Step 2: Run it; expect FAIL.**
- [ ] **Step 3: Switch.** Add the dependencies, replace the local module by the re-export, fix compile errors (new `MapGeoref` fields, `box_size` in text objects, any renamed reader function). Behaviour of every existing test stays; do not edit expectations except for the type changes.
- [ ] **Step 4: Run `cargo test --workspace --locked`; expect all 506+ pass**, clippy clean.
- [ ] **Step 5: Commit** `refactor(omap): use freemapper-omap from freemapper-core`.

---

### Task 2: The `ppen_import` module

**Files:**
- Create: `crates/freemapper-courseplan/src/ppen_import.rs`, `crates/freemapper-courseplan/src/ppen_import/tests.rs`, `crates/freemapper-courseplan/tests/ppen_import.rs` (real files)
- Modify: `crates/freemapper-courseplan/src/lib.rs`, `crates/freemapper-courseplan-model/src/types.rs` (fields `ppen_id: Option<u32>` on `Control` and `Course`, serde-skipped when `None`; `FORMAT_VERSION` unchanged because it is additive), `docs/glossary.md`
- Fixtures: use the three Purple Pen files from `freemapper-core/crates/freemapper-course/tests/fixtures/` (copy into `tests/fixtures/ppen/` with source, commit and SHA-256 in `tests/fixtures/README.md`, BSD-3 licence text kept) and the Lübeck map already in `examples/luebeck-sprint/`

**Interfaces:**
- Consumes: `freemapper_course::{read_ppen, fit, FitOptions, FittedEvent, Control as PControl, Course as PCourse, Special, ControlKind, offset_from_placement/placement_from_offset, CourseNote, FitNote}`; courseplan `numbers` for the circle outer radius and number box size at the course's print scale (reuse the functions the number placement uses; do not duplicate constants).
- Produces:
  ```rust
  pub struct PpenImport { pub controls: Vec<Control>, pub specials: Vec<SpecialObject>, pub courses: Vec<Course>, pub notes: Vec<ImportNote> }
  pub enum ImportNote { UnknownElement(String), Skipped { course: String, why: &'static str }, GapsNotSupported { course: String }, UnmappedSymbol { control: String, column: char, id: String }, NumberIntoCircle { control: String }, Fit(String), Other(String) }
  #[derive(Debug, thiserror::Error)]
  pub enum PpenImportError { Read(freemapper_course::CourseError), Fit(freemapper_course::FitError) }
  pub fn import_ppen(bytes: &[u8], map: &MapDocument) -> Result<PpenImport, PpenImportError>
  ```

- [ ] **Step 1: Write failing unit tests** in `ppen_import/tests.rs` on small inline `.ppen` strings:
  - `normal_control_keeps_code_position_and_ppen_id` (position in mm, y down, `ppen_id` set).
  - `start_and_finish_become_special_objects` (kinds start/finish → `SpecialObject::Start`/`Finish` with ids such as `STA1`, `FIN1`; `Course.start`/`finish` refer to them).
  - `crossing_point_keeps_kind`.
  - `course_stops_follow_the_chain_with_bends_and_taped_legs` (`flagging all` → `taped`, bends kept in order).
  - `number_location_becomes_direction_and_gap` (a known offset → direction/gap within 0.01; central placement when all courses agree, otherwise per-course override).
  - `description_columns_c_to_h_map_to_iof_2024_ids` and `unmapped_2004_symbol_is_kept_and_noted`.
  - `climb_print_scale_and_secondary_title` (`climb_m`, `planned_print_scale`; secondary title noted if the model has no field).
  - `special_objects_map_to_their_counterparts`: out-of-bounds, dangerous, forbidden route (single cross), marked route, first aid, water; boundary/line/text/white-out give `Skipped`-style notes naming the kind; course filter `All` → empty `courses`, `Only` → course names.
  - `variation_course_is_skipped_and_named`, `circle_gaps_are_noted_per_course`.
  - `scale_mismatch_is_an_error`.
- [ ] **Step 2: Real-file test** `tests/ppen_import.rs`: each of the three fixtures imports against its matching map without error (use the map each fixture belongs to; if a fixture has no map in our repo, fit against the Lübeck map only for the one that matches, and for the others assert the `Read` stage via `freemapper_course::read_ppen` plus a unit-level mapping call on the `RawEvent`; split `import_ppen` into `read` and `from_fitted(FittedEvent)` for this) and the result passes `Event` validation (`freemapper_courseplan_model::validate`).
- [ ] **Step 3: Run; expect FAIL.**
- [ ] **Step 4: Implement `import_ppen` and `from_fitted`.** Sort out the description-id mapping from the IOF control description standards (2004 → 2024); keep unknown ids and emit `UnmappedSymbol`. Names of courses stay as they are; classes are not created (Purple Pen has none).
- [ ] **Step 5: Run `cargo test --workspace --locked`; expect pass.** Clippy clean.
- [ ] **Step 6: Commit** `feat(ppen): import a Purple Pen event`.

---

### Task 3: `init --ppen`, `import --ppen`, MCP tool

**Files:**
- Modify: `crates/freemapper-courseplan-cli/src/main.rs` (`init … --ppen FILE`; new command `Import { project, #[arg(long)] ppen: PathBuf, #[arg(long)] replace: bool }`), `crates/freemapper-courseplan-cli/src/mcp.rs` (tool `import_ppen` with `path`, `replace`), `crates/freemapper-courseplan/src/init.rs` as needed, `i18n/{en,de}/courseplan.ftl`
- Create: `crates/freemapper-courseplan-cli/src/import_cmd.rs`
- Test: `crates/freemapper-courseplan-cli/tests/import.rs`

**Interfaces:**
- Consumes: `ppen_import::{import_ppen, PpenImport, ImportNote, PpenImportError}`, the project loader and `write_atomic` (reuse).
- Produces: `pub(crate) fn import_ppen_file(project: &Path, ppen: &Path, replace: bool) -> Result<ImportReport, ImportError>` shared by CLI and MCP; `Event.sources.ppen = Some(path relative to the project file)` is set (design §5.5a: the linked file). Messages: `import-ppen-done` (controls, courses), `import-ppen-note-*` per `ImportNote`, `import-ppen-needs-replace`, `error-import-ppen-*` for read and fit errors (names both scales for a mismatch).

- [ ] **Step 1: Write failing tests:** `init --ppen` on the example map and a fixture `.ppen` writes a project with controls and courses and the linked `ppen` path; `import` into an empty project works; into a project with controls it exits non-zero with `import-ppen-needs-replace` and leaves the file unchanged; `--replace` swaps the content and keeps classes and accepted findings; notes are printed in en and de; a scale mismatch exits non-zero and names both scales; the MCP tool returns compact JSON `{controls, courses, notes}`.
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement** the shared function, commands and MCP tool; new texts in both languages (unslop/humanizer for messages longer than a phrase).
- [ ] **Step 4: Run `cargo test --workspace --locked`; expect pass.** Run the real CLI once on a fixture and look at the output.
- [ ] **Step 5: Commit** `feat(cli): init --ppen, import --ppen, MCP tool import_ppen`.

---

### Task 4: Documentation

**Files:**
- Create: `docs/adr/0034-ppen-import-uses-the-core-course-crate.md`
- Modify: `docs/adr/README.md`, design §5.5/§5.5a/§9 (what is built, what is open, mark *(user)*/*(Claude's call)*/*(agreed with freemapper-print)*), `README.md` + `README.en.md` (features, commands, status; mention that omap and course come from freemapper-core), `docs/manual/{de,en}.md` (chapter for `import`), `examples/README*.md`, `CHANGELOG.md` + `CHANGELOG.en.md`, `docs/glossary.md`

- [ ] **Step 1: ADR 0034** (about 30 lines): omap and course from core, the fit decides the y axis, descriptions mapping and its gaps, what is not imported and noted, `ppen_id` field, `--replace` rule, the linked file path. Statements marked.
- [ ] **Step 2: Update the other documents** in German and English; both READMEs equal.
- [ ] **Step 3: Run `unslop` and `humanizer`** over new prose.
- [ ] **Step 4: fmt, clippy, tests; commit** `docs: .ppen import (ADR 0034, manual, readmes, changelog)`.
