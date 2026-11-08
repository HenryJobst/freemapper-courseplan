# M1: Data model, project file and map reading – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for
> tracking.

**Goal:** An event can be created from an OOM map, stored as a readable TOML project file that holds all four
course kinds, loaded with clear errors, saved without losing the user's comments, and its map can be queried
for objects near a point.

**Architecture:** `freemapper-courseplan-model` holds the domain types and the project file (serde + toml_edit),
with no map dependency. `freemapper-courseplan` gets the `.omap` reader, copied and trimmed from
freemapper-print, plus a `MapIndex` for spatial queries. `freemapper-courseplan-cli` gets `init` and `info`.

**Tech Stack:** Rust 1.99.1 (edition 2024), serde, toml 1.2.1, toml_edit, quick-xml 0.42.0, thiserror 2.0.21,
clap 4.6.7, tempfile 3.27.0 (tests).

**Spec:** `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` (§4, §5.1, §5.7, §8 M1). Terms:
`docs/glossary.md` and freemapper-print's glossary.

## Global Constraints

- Toolchain pinned to 1.99.1; `unsafe_code = "forbid"`; `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` pass after every task.
- Every dependency version at least two weeks old (ADR 0006). Use the suite's versions: serde 1 (lock 1.0.229),
  toml 1.2.1, quick-xml 0.42.0, thiserror 2.0.21, tempfile 3.27.0. For toml_edit take the newest release that is
  at least two weeks old on the day of the task and pin it exactly in `[workspace.dependencies]`.
- Code identifiers follow the glossaries: `Course`, `Control`, `ControlCode`, `CourseControl`, `CourseKind`,
  `NumberPlacement`, `MarkedRoute`, `SpecialObject`, `Discipline`, `Class`, `RuleCategory`, `AcceptedFinding`.
  New terms get a glossary row in the same task.
- Map coordinates: millimetres at map scale, y down, as `f64` (`MapPoint`). OOM files store 1/1000 mm as `i32`
  (`omap::MapCoord`); conversion is `mm = value / 1000.0`.
- Number placement: `direction` in degrees, 0 = east, counter-clockwise; `gap_mm` in print millimetres.
- The model does not judge course quality (no code ranges, no distances); that is `check` in M2. It only
  rejects files that cannot be a consistent event.
- No AI anywhere in M1 (ADR 0016).
- CLI and error messages in English; messages longer than a short phrase go through unslop/humanizer.
  Changelog entries in `CHANGELOG.md` and `CHANGELOG.en.md` under "Unveröffentlicht"/"Unreleased".
- Work in branch `m1-model-project-file-map`, worktree `.worktrees/m1-model-project-file-map`.

## Review Focus

1. A hand-edited project file with comments, saved after a change: comments in untouched tables survive
   (Task 3, `save_keeps_comments_outside_changed_values`).
2. A course that uses an undefined control, start or finish, or two controls with the same code: `load` fails
   with a message naming course and code, never a panic (Task 2, `load_reports_every_inconsistency`).
3. The program started from another folder: the map path resolves relative to the project file, not the
   working directory (Task 2, `map_path_is_relative_to_project_file`; Task 5, `init_from_other_folder`).
4. Umlauts and other non-ASCII text in titles, class and course names round-trip unchanged (Task 2,
   `round_trip_keeps_unicode`).
5. `init` on a forest map, an old binary `.omap` or a missing file: a clear refusal and no project file left
   behind (Task 5, `init_refuses_forest_map`, `init_reports_missing_map`).

---

### Task 1: Domain types for all course kinds

**Files:**
- Create: `crates/freemapper-courseplan-model/src/types.rs`
- Modify: `crates/freemapper-courseplan-model/src/lib.rs` (module and re-exports),
  `crates/freemapper-courseplan-model/Cargo.toml` (serde), `Cargo.toml` (workspace deps), `docs/glossary.md`
  (row "Kartenpunkt / map point / `MapPoint`": point in map mm, f64, y down; `omap::MapCoord` is the OOM file
  form in 1/1000 mm)
- Test: `crates/freemapper-courseplan-model/src/types.rs` (unit tests)

**Interfaces:**
- Produces (all `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`, public fields):
  - `MapPoint { x: f64, y: f64 }`, serialised as a two-element array `[x, y]`.
  - `ControlCode(pub String)`. Deserialises from a TOML integer or string; serialises as an integer when the
    string is all ASCII digits without a leading zero, else as a string. `Display` prints the inner string.
  - `NumberPlacement { direction: f64, gap_mm: f64 }`.
  - `ControlDescription { c, d, e, f, g, h: Option<String> }` (IOF 2024 symbol ids per column, e.g. `"1.8"`).
  - `Control { code: ControlCode, map_mm: MapPoint, description: ControlDescription, number: Option<NumberPlacement> }`.
  - `SpecialObject` with TOML field `kind` (snake_case): `Start { id: String, map_mm: MapPoint }`,
    `Finish { id, map_mm }`, `MarkedRoute { points: Vec<MapPoint> }`, `CrossingPoint { map_mm }`,
    `OutOfBoundsArea { polygon: Vec<MapPoint> }`, `ForbiddenRoute { points }`, `DangerousArea { polygon }`,
    `FirstAid { map_mm }`, `Water { map_mm }`.
  - `CourseControl { code: ControlCode, number: Option<NumberPlacement>, bends: Vec<MapPoint> }`; `bends` belong
    to the leg that leaves this control, default empty and omitted when empty.
  - `CourseKind` with TOML field `kind`: `Individual { controls: Vec<CourseControl> }`,
    `Team { team_size: u8, mandatory: Vec<CourseControl>, free: Vec<ControlCode> }`,
    `Relay { legs: u8, parts: Vec<RelayPart> }`, `Score { time_limit_min: u32, penalty_per_min: u32, controls: Vec<ScoreControl> }`.
  - `RelayPart`: `Common { controls: Vec<CourseControl> }` | `Fork { variants: Vec<Vec<CourseControl>> }`.
  - `ScoreControl { code: ControlCode, value: u32 }`.
  - `Course { name: String, classes: Vec<String>, start: String, finish: String, climb_m: Option<u32>, planned_print_scale: Option<u32>, kind: CourseKind }`; in TOML the kind's fields sit in the course table
    next to `kind = "individual"` etc. (flattened).
  - `Discipline`: `Sprint | Middle | Long | Ultralong | Training` (snake_case), default `Training`.
  - `Class { name: String, rule_category: Option<String>, expected_starters: Option<u32> }`.
  - `AcceptedFinding { rule: String, controls: Vec<ControlCode>, course: Option<String>, note: String }`.
  - `EventMeta { title: String, date: Option<String>, discipline: Discipline, rule_profile: String, start_window_min: Option<u32> }`; `rule_profile` default `"wkb-2026"`.
  - `Sources { map: PathBuf, ppen: Option<PathBuf> }`.
  - `Event { format: u32, event: EventMeta, sources: Sources, classes: Vec<Class>, controls: Vec<Control>, specials: Vec<SpecialObject>, courses: Vec<Course>, accepted_findings: Vec<AcceptedFinding> }`;
    `format` is `1`; empty lists are omitted on write and default on read.
  - `pub const FORMAT_VERSION: u32 = 1;`

- [ ] **Step 1: Write the failing tests** in `types.rs`:
  - `control_code_reads_integer_and_string`: `code = 31` and `code = "S31"` both deserialise;
    `ControlCode("31")` serialises to `31`, `ControlCode("031")` and `ControlCode("S31")` to strings.
  - `every_course_kind_round_trips`: one `Event` with one course of each kind (team: `team_size = 3`,
    mandatory `[31, 32, 33]`, free `[40, 41]`; relay: `legs = 3`, one common part and one fork with two
    variants; score: `time_limit_min = 60`, `penalty_per_min = 30`, values 10 and 40) serialises with
    `toml::to_string` and reads back equal.
  - `individual_course_toml_shape`: deserialising the design's §5.7 sketch (adapted: `start = "S1"`,
    `finish = "F1"`, `controls = [{ code = 31, number = { direction = 270, gap_mm = 1.0 } }, { code = 32, bends = [[20.0, -30.0]] }]`)
    gives the expected `Course`.
- [ ] **Step 2: Run** `cargo test -p freemapper-courseplan-model` → FAIL (types missing).
- [ ] **Step 3: Implement** the types in `types.rs`; serde attributes of the implementer's choice, as long as
  the TOML shape above holds.
- [ ] **Step 4: Run** `cargo test -p freemapper-courseplan-model` → PASS; then fmt and clippy.
- [ ] **Step 5: Commit** `feat(model): domain types for all four course kinds`.

### Task 2: Load and validate a project file

**Files:**
- Create: `crates/freemapper-courseplan-model/src/project_file.rs`, `crates/freemapper-courseplan-model/src/validate.rs`
- Modify: `crates/freemapper-courseplan-model/src/lib.rs`, `crates/freemapper-courseplan-model/Cargo.toml`
  (toml, thiserror; tempfile as dev-dependency)
- Test: `crates/freemapper-courseplan-model/tests/project_file.rs`, fixtures in
  `crates/freemapper-courseplan-model/tests/fixtures/` (hand-made; list them in `tests/fixtures/README.md`)

**Interfaces:**
- Consumes: `Event` and friends from Task 1.
- Produces:
  - `enum ModelError` (`Display` messages, one per inconsistency): `DuplicateControl(ControlCode)`,
    `DuplicateCourse(String)`, `DuplicateSpecialId(String)`, `UnknownControl { course: String, code: ControlCode }`,
    `UnknownStart { course: String, id: String }`, `UnknownFinish { course: String, id: String }`,
    `TeamTooSmall { course: String, team_size: u8 }` (below 2),
    `FreeControlIsMandatory { course: String, code: ControlCode }`,
    `UnsupportedFormat(u32)` (format newer than `FORMAT_VERSION`).
  - `fn validate(event: &Event) -> Vec<ModelError>` in `validate.rs`.
  - `enum ProjectError` (thiserror): `Io { path: PathBuf, source: std::io::Error }`,
    `Parse { path: PathBuf, message: String }`, `Invalid { path: PathBuf, errors: Vec<ModelError> }`.
    `Invalid`'s message lists every error on its own line.
  - `struct ProjectFile { path: PathBuf, event: Event, /* private: the parsed toml_edit document, used in Task 3 */ }`
    with `fn load(path: &Path) -> Result<ProjectFile, ProjectError>` (parse, then `validate`; any error →
    `Invalid`) and `fn map_path(&self) -> PathBuf` (`sources.map` joined to the project file's folder unless
    absolute).

- [ ] **Step 1: Write the failing tests** in `tests/project_file.rs`:
  - `loads_minimal_event`: fixture `minimal.toml` (format 1, title, map, one start, one finish, two controls,
    one individual course) loads; `event.courses[0].kind` is `Individual` with 2 controls.
  - `load_reports_every_inconsistency`: fixture `broken.toml` with duplicate code 31, course "A" using 44
    (undefined) and start "S9" (undefined), team course "T" with `team_size = 1` and free control 31 also
    mandatory → `ProjectError::Invalid` whose `errors` contains exactly those five `ModelError`s; its
    `to_string()` contains `"A"` and `"44"`.
  - `rejects_newer_format`: `format = 2` → `Invalid` with `UnsupportedFormat(2)`.
  - `parse_error_names_file`: invalid TOML → `Parse` whose message contains the file name.
  - `map_path_is_relative_to_project_file`: project in a temp dir `ev/` with `map = "maps/m.omap"` →
    `map_path()` is `ev/maps/m.omap`, independent of the current directory.
  - `round_trip_keeps_unicode`: title "Training Kröpeliner Tor", class "Anfänger", course "Weiß" survive
    `load` → (Task 3 not yet) `toml::to_string(&event)` → parse.
- [ ] **Step 2: Run** `cargo test -p freemapper-courseplan-model --test project_file` → FAIL.
- [ ] **Step 3: Implement** `validate` and `ProjectFile::load`, `map_path`. Keep the parsed `toml_edit`
  document in the struct for Task 3 (parse the text once with toml_edit and deserialise the `Event` from it).
- [ ] **Step 4: Run** the tests → PASS; fmt, clippy.
- [ ] **Step 5: Commit** `feat(model): load and validate project files`.

### Task 3: Save without losing comments

**Files:**
- Modify: `crates/freemapper-courseplan-model/src/project_file.rs`
- Test: `crates/freemapper-courseplan-model/tests/project_file.rs`

**Interfaces:**
- Consumes: `ProjectFile` from Task 2.
- Produces:
  - `fn ProjectFile::create(path: &Path, event: Event) -> Result<ProjectFile, ProjectError>`: validates, refuses
    an existing file (`Io` with `ErrorKind::AlreadyExists`), writes it.
  - `fn ProjectFile::save(&mut self) -> Result<(), ProjectError>`: validates `self.event`, then writes.
  - The write merges the freshly serialised event into the kept document:

    ```text
    merge(old, new):
      both tables        → for each key in new: merge(old[key], new[key]) or insert; remove keys not in new;
                           keep old's key order, append new keys at the end
      both arrays of tables, same length → merge element-wise by index
      anything else      → if old value == new value keep old (with its comments and formatting),
                           else replace by new and keep old's leading decor (comment lines above it)
    ```
    Write to `<file>.tmp` in the same folder, then rename over the original.

- [ ] **Step 1: Write the failing tests**:
  - `save_keeps_comments_outside_changed_values`: fixture `commented.toml` with a file-top comment, a comment
    above `[event]` and a trailing comment on a control's `code`; change `event.title`, save, re-read text: all
    three comments still present, new title written, `load` gives the changed event.
  - `save_after_removing_course_drops_it`: remove the only course, save, reload → no courses, the comment above
    `[event]` is still there.
  - `create_refuses_existing_file`: second `create` on the same path → `Io` with `AlreadyExists`, file content
    unchanged.
  - `save_refuses_invalid_event`: add a course with an unknown control, `save` → `Invalid`, file on disk
    unchanged.
- [ ] **Step 2: Run** → FAIL.
- [ ] **Step 3: Implement** `create`, `save` and the private `merge` in `project_file.rs`.
- [ ] **Step 4: Run** → PASS; fmt, clippy.
- [ ] **Step 5: Commit** `feat(model): save project files and keep the user's comments`.

### Task 4: Read OOM maps and query them

**Files:**
- Create: `crates/freemapper-courseplan/src/omap/{mod.rs,coord.rs,model.rs,read.rs,symbol_set.rs}` copied
  from freemapper-print commit `8b0a0a6` (branch `m1-map-rendering`, path `crates/freemapper-print/src/omap/`);
  `crates/freemapper-courseplan/src/map.rs`; `docs/adr/0017-omap-reader-copied-from-freemapper-print.md`
- Copy: `tests/fixtures/maps/luebeck.omap` and `tests/fixtures/maps/forest-sample.omap` from the same
  freemapper-print commit; add both entries, with source, commit and licence as in freemapper-print's
  `tests/fixtures/maps/README.md`, to `tests/fixtures/README.md`
- Modify: `crates/freemapper-courseplan/src/lib.rs`, `crates/freemapper-courseplan/Cargo.toml` (quick-xml,
  thiserror, model crate), `docs/adr/README.md` (row 0017)
- Test: `crates/freemapper-courseplan/tests/read_maps.rs`, unit tests in `map.rs`

**Interfaces:**
- Consumes: `MapPoint` from Task 1.
- Produces:
  - `omap` as in freemapper-print, trimmed: keep `read_map`, `read_map_file`, `MapDocument`, `MapObject`,
    `MapObjectKind`, `MapPart`, `MapGeoref`, `TemplateRef`, `MapCoord`, `SymbolSet::from_omap_xml`,
    `SymbolSet::id_for_code`, `SymbolSet::symbol_type`, `SymbolSet::is_hidden`, `OmapError`, and add
    `SymbolSet::code_for_id(&self, id: u32) -> Option<&SymbolCode>`. Drop `enlarge.rs`, the built-in symbol sets
    and `include_str!` assets, `with_additions`, `with_sheet_symbols`. Each file starts with
    `//! Copied from freemapper-print 8b0a0a6 (ADR 0017).` Keep its unit tests that still apply.
  - In `map.rs`:
    - `enum MapKind { Sprint, Schoolyard, Forest }` and
      `fn map_kind(doc: &MapDocument) -> MapKind`: scale ≤ 1:2000 → `Schoolyard`; symbol set name contains
      `"ISSprOM"` or scale ≤ 1:5000 → `Sprint`; otherwise `Forest`.
    - `enum FeatureGeometry { Point(MapPoint), Line(Vec<MapPoint>), Area(Vec<Vec<MapPoint>>) }` (area rings;
      first outer, others holes; even-odd).
    - `struct MapFeature { symbol_code: Option<String>, part: String, geometry: FeatureGeometry }`.
    - `struct MapIndex` with `fn from_document(doc: &MapDocument) -> MapIndex`,
      `fn features_near(&self, p: MapPoint, radius_mm: f64) -> Vec<(&MapFeature, f64)>` (sorted by distance,
      0 inside an area), `fn extent(&self) -> Option<(MapPoint, MapPoint)>` (min, max),
      `fn len(&self) -> usize`.
    - Rules: geometry type from `SymbolSet::symbol_type` (1 point, 2 line, 4 area; combined 16 → line if the
      path is open, area if closed; 8 text skipped); hidden symbols skipped; objects tagged
      `freemapper:source=sheet` skipped (sheet elements, not terrain). Cubic Béziers (flag
      `FLAG_CURVE_START`) are flattened into 8 segments each. Linear scan; no spatial index in M1.
  - ADR 0017 (10–30 lines): the `.omap` reader is copied now from freemapper-print `8b0a0a6`, trimmed to
    reading, because M2 checks need map objects before the shared `freemapper-render` crate exists; it is
    replaced by that crate when extracted (ADR 0013); fixes found here are reported to freemapper-print.

- [ ] **Step 1: Copy** the reader files and fixtures; adapt module paths; drop what is listed above.
- [ ] **Step 2: Write the failing tests**:
  - `read_maps.rs::luebeck_reads` and `forest_sample_reads`: same assertions as freemapper-print's
    `tests/read_maps.rs` (object count equals `<object ` count in `<parts>`, Lübeck scale 4000 and
    `+init=epsg:25832`, forest sample 539 objects and scale 10000).
  - `read_maps.rs::map_kinds`: Lübeck → `Sprint`, forest sample → `Forest`.
  - `read_maps.rs::luebeck_index_skips_sheet_objects`: `MapIndex::from_document` has fewer features than
    the map has objects, and no feature has `part == "peripheral information"` with a sheet tag.
  - `map.rs` unit tests on a hand-built `MapDocument`: `distance_to_point`, `distance_to_line_segment`,
    `inside_area_is_zero`, `inside_hole_is_not_zero`, `curve_is_flattened` (a quarter-circle Bézier's
    flattened points lie within 0.05 mm of the arc), `features_near_sorted_and_bounded`.
- [ ] **Step 3: Run** `cargo test -p freemapper-courseplan` → the copied reader tests PASS, the new ones FAIL.
- [ ] **Step 4: Implement** `map.rs`.
- [ ] **Step 5: Run** → PASS; fmt, clippy.
- [ ] **Step 6: Commit** `feat(map): read OOM maps and query objects near a point` (reader, fixtures, ADR 0017
  and index row in one commit).

### Task 5: CLI `init` and `info`

**Files:**
- Create: `crates/freemapper-courseplan/src/init.rs`, `crates/freemapper-courseplan-cli/tests/cli.rs`
- Modify: `crates/freemapper-courseplan-cli/src/main.rs`, `crates/freemapper-courseplan/src/lib.rs`,
  `crates/freemapper-courseplan-cli/Cargo.toml` (model crate; tempfile dev), `CHANGELOG.md`, `CHANGELOG.en.md`,
  `README.md`, `README.en.md` (status paragraph: what works now)
- Test: `crates/freemapper-courseplan-cli/tests/cli.rs` (runs the built binary via
  `env!("CARGO_BIN_EXE_freemapper-courseplan")`), unit tests in `init.rs`

**Interfaces:**
- Consumes: `ProjectFile::create`, `ProjectFile::load`, `map_path` (Tasks 2–3); `read_map_file`, `map_kind`,
  `MapIndex` (Task 4).
- Produces:
  - `fn new_event(map_path: &Path, project_path: &Path, title: Option<String>) -> Result<Event, InitError>` in
    `init.rs`: reads the map; refuses `MapKind::Forest` (`InitError::ForestNotSupported`, message: forest maps
    come in a later version, ADR 0010); title from `--title`, else `title` in a `freemapper.toml` next to the
    map, else the map file stem; `sources.map` relative to the project file's folder; discipline `Training`,
    rule profile `wkb-2026`, everything else empty.
  - `enum InitError` (thiserror): `Map(OmapError)`, `ForestNotSupported`, `Project(ProjectError)`.
  - CLI:
    - `freemapper-courseplan init <MAP> [--project <FILE>] [--title <TEXT>]`: default project file
      `<map folder>/<map stem>.courseplan.toml`; prints the project path; exit code 1 with the error on stderr
      on failure, and no file is written.
    - `freemapper-courseplan info <PROJECT>`: loads (with validation) and prints title, map path, map kind,
      number of controls, courses (per kind) and classes, one per line; exit code 1 and the error on failure.

- [ ] **Step 1: Write the failing tests** in `cli.rs` (each in a temp dir with a copy of a fixture map):
  - `init_creates_project_next_to_map`: `init luebeck.omap` → `luebeck.courseplan.toml` exists, `info` on it
    prints `map kind: sprint` and `controls: 0`.
  - `init_from_other_folder`: run from a different working directory with an absolute map path and
    `--project ev/x.toml` → `sources.map` in the file is relative (`../luebeck.omap` or equivalent) and `info`
    works from a third folder.
  - `init_takes_title_from_freemapper_toml`: a `freemapper.toml` with `title = "Altstadt Lübeck"` next to
    the map → title in the project file.
  - `init_refuses_forest_map`: forest sample → exit code 1, stderr mentions forest maps, no project file.
  - `init_reports_missing_map`: non-existent path → exit code 1, stderr names the path, no project file.
  - `init_refuses_existing_project`: second `init` → exit code 1, file unchanged.
  - `info_reports_invalid_project`: the `broken.toml` fixture from Task 2 next to a map → exit code 1, stderr
    contains `"44"`.
- [ ] **Step 2: Run** `cargo test -p freemapper-courseplan-cli` → FAIL.
- [ ] **Step 3: Implement** `init.rs` and the two subcommands with clap derive.
- [ ] **Step 4: Run** `cargo test --workspace` → PASS; fmt, clippy.
- [ ] **Step 5: Docs.** Changelog entries (German and English, "Hinzugefügt"/"Added"): project file with all
  course kinds, `init` and `info`. README status paragraph in both languages. Apply unslop and humanizer to the
  new text.
- [ ] **Step 6: Commit** `feat(cli): init an event from a map and show it`.

---

## After the last task

Final whole-branch review, then merge `--no-ff` into `main`, run the checks on `main`, push to `origin` and
`github`, watch CI, remove the worktree and run `cargo clean` there (CLAUDE.md).

## Follow-ups after M1 (from the reviews)

Carried into M2 planning; the SDD ledger is deleted after the merge.

- ADR 0019 wording: identity matching holds for the table forms; an inline course `controls = [...]` array
  merges by index, so a comment inside it stays at its position. "Plain digits" also means no leading zero.
- First M2 task: `MapIndex::from_document` looks up symbols per object (O(objects × symbol XML)); build a
  per-symbol table once.
- Tests for points after a Bézier curve and for chained curves, before M2 checks rely on the geometry.
- Close the format 1 gaps listed in ADR 0019 (leg and circle gaps, print area, ids and course assignment for
  special objects, team size per class, relay variant names and assignment).
- `init`: prefer `map_kind` from `freemapper.toml` when present; Windows absolute fallback keeps the `\\?\`
  prefix (`//?/C:/…`); empty folder left behind when `create` fails; an empty `title` wins over the map stem.
- `info` prints `sources.map` as stored (relative to the project file); print the resolved path or label it.
- Smaller: duplicate-reporting consistency in `validate`, start and finish sharing an id, `write_atomic` and
  an existing `<file>.tmp`, serialisation errors reported as parse errors, glossary rows for `RelayPart`,
  `ScoreControl`, `Sources`, `DangerousArea`, `FirstAid`, `Water`, doc comments on `Course`, `Discipline`,
  `EventMeta`.
