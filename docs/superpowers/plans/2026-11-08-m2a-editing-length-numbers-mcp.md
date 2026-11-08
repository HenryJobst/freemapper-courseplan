# M2a: Editing courses, length, control numbers, MCP – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for
> tracking.

**Goal:** Set individual courses on a sprint map through the CLI and an AI agent (MCP): controls with
descriptions, special objects, courses with bends and taped legs, classes, course length after WKB B 7.2.5 and
automatic control number placement. Each command loads the project file, changes it and saves it with the
user's comments intact.

**Architecture:** The model crate gets the format additions that M2 needs (ADR 0019's gaps for crossing points,
special-object ids, legs from the start, print area). The library crate gets pure functions on `Event`: course
geometry and length (`course.rs`), edit operations that never leave an invalid event (`edit.rs`) and number
placement (`numbers.rs`). The CLI and a new MCP server are thin wrappers that load, call, save.

**Tech Stack:** Rust 1.99.1, serde, toml_edit `=0.25.17`, clap 4.6.7, rmcp `=3.6.0`, schemars 1.2.3, tokio
1.54.1, serde_json 1.0.154.

**Spec:** `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` (§2 first version, §4, §5.3,
§5.4, §5.8, §8 M2). Split: design milestone M2 is planned as M2a (this plan) and M2b (the `check` rules of §5.6,
separate plan). Follow-ups from M1: end of `docs/superpowers/plans/2026-11-08-m1-model-project-file-map.md`.

## Global Constraints

- Toolchain 1.99.1; `unsafe_code = "forbid"`; `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` pass after every task.
- Dependencies at least two weeks old (ADR 0006). New crates only at the versions in freemapper's
  `Cargo.lock` (`/Users/henryprivat/Documents/gitrepos/freemapper/Cargo.lock`): rmcp 3.6.0 and rmcp-macros
  3.6.0 (pin `rmcp = { version = "=3.6.0", features = ["server", "transport-io", "macros"] }`), schemars 1.2.3,
  tokio 1.54.1, serde_json 1.0.154. Every transitive crate cargo adds must match freemapper's lock version;
  use `cargo update -p <crate> --precise <version>` where needed.
- Identifiers follow `docs/glossary.md` and freemapper-print's glossary. New terms get glossary rows in the same
  task.
- Map coordinates in mm at map scale, y down (`MapPoint`). Overprint sizes are given in print mm at the map
  kind's base scale (sprint 1:4000, schoolyard 1:1000, forest 1:15000). Enlarged prints grow every symbol in
  proportion (ISSprOM 4.7: 133 % at 1:3000; ISOM: 150 % at 1:10000; freemapper-print, 2026-11-08), so on the
  map the overprint keeps its size: `map_mm = print_mm × base_scale / map_scale`, whatever the planned print
  scale.
- Number placement: `direction` in degrees, 0 = east, counter-clockwise; `gap_mm` in print mm.
- Only individual courses are edited and measured in M2a. Operations on other kinds return a clear
  "not supported yet" error.
- No AI in the program (ADR 0016). The MCP server lets an external agent drive it.
- Project file format stays `format = 1` (provisional until the first release, ADR 0019).
- Program texts in German and English (ADR 0021): every user-facing message, CLI help text and MCP text
  exists in both languages through the catalog of Task 6; the language comes from `--lang de|en`, else
  `FREEMAPPER_LANG`, else the system locale (`LC_ALL`, `LC_MESSAGES`, `LANG`), else English. Code, identifiers,
  logs and `Display` of error types stay English. Texts longer than a short phrase follow unslop/humanizer in
  both languages. Changelog entries in both changelogs.
- CLI and MCP tests set `LANG=C` and pass `--lang en` (or the MCP equivalent `FREEMAPPER_LANG=en`) unless
  they test German texts, so they do not depend on the machine's locale.
- Branch `m2a-editing`, worktree `.worktrees/m2a-editing`.
- Commit trailers: `Co-Authored-By: <model> <noreply@anthropic.com>` and
  `Claude-Session: https://claude.ai/code/session_01BW25YoqN46qrkRXtWogXH1`.

## Review Focus

1. A failed edit (unknown code, a removal that would break a course, an invalid value) leaves the project file
   byte-identical (Task 7, `failed_edit_leaves_file_unchanged`).
2. Removing a control that courses use takes it out of those courses and names them, rather than failing late
   or leaving a dangling code (Task 4, `remove_control_takes_it_out_of_courses`).
3. The same control twice in one course (a butterfly loop): numbering counts both visits, and replacing the
   sequence keeps each visit's bends and number (Task 3, `butterfly_numbers_both_visits`; Task 4,
   `set_course_controls_keeps_leg_data_per_visit`).
4. A position outside the map's extent: refused with the extent in the message (Task 4,
   `add_control_outside_map_is_refused`).
5. MCP with a relative project path: refused with a message, no file touched (Task 9,
   `relative_path_is_refused`).

---

### Task 1: Format additions for crossing points, special ids, start legs and print area

**Files:**
- Modify: `crates/freemapper-courseplan-model/src/types.rs`, `src/validate.rs`, `src/project_file.rs`
  (identity keys, omitted-default keys), `src/lib.rs`
- Modify: test fixtures in `crates/freemapper-courseplan-model/tests/fixtures/` that contain specials
- Create: `docs/adr/0020-format-1-crossing-points-special-ids-start-legs.md`; Modify: `docs/adr/0019-…`
  (errata only), `docs/adr/README.md`, `docs/glossary.md`
- Test: `types.rs` unit tests, `crates/freemapper-courseplan-model/tests/project_file.rs`

**Interfaces:**
- Produces:
  - `enum ControlKind { #[default] Control, CrossingPoint }` (snake_case). `Control` gets
    `kind: ControlKind` (TOML key `kind`, omitted when `Control`). A crossing point is a control that a course
    lists in its sequence but that gets no number (IOF XML `CrossingPoint`).
  - `SpecialObject::CrossingPoint` is removed. Every remaining variant gets `id: String`. Every variant except
    `Start` and `Finish` gets `courses: Vec<String>` (omitted when empty; empty means all courses).
  - `CourseControl` gets `taped: bool` (omitted when false): the leg leaving this control is a marked route.
  - `Course` gets `start_bends: Vec<MapPoint>` and `start_taped: bool` (both omitted when empty/false) for the
    leg from the start to the first control, and `print_area: Option<PrintArea>`.
  - `struct PrintArea { min: MapPoint, max: MapPoint }`.
  - `ModelError::UnknownCourseInSpecial { id: String, course: String }`. `DuplicateSpecialId` now covers all
    special objects.
  - Saving matches `specials` by `id` (identity key), like controls by `code`.
  - ADR 0020 (10–30 lines): the additions above and why: IOF XML and Purple Pen put crossing points into the
    course sequence; ids let courses and the later `.ppen` export refer to special objects; the start leg needs a
    home for its bends. ADR 0019 errata: identity matching holds for tables, an inline course `controls = [...]`
    array merges by index; "plain digits" means digits without a leading zero. Index row for 0020, inserted before the 0021 row.
  - Glossary rows: Querungsstelle as `ControlKind::CrossingPoint` (replace the old `CrossingPoint` row),
    markierte Strecke on a leg (`taped`), Druckbereich (`PrintArea`), and the M1 follow-up rows `RelayPart`,
    `ScoreControl`, `Sources`, `DangerousArea`, `FirstAid`, `Water`.

- [ ] **Step 1: Write the failing tests**:
  - `crossing_point_control_round_trips`: `kind = "crossing_point"` reads and writes; a control without `kind`
    is `ControlKind::Control` and is written without `kind`.
  - `special_objects_have_ids_and_courses`: every special kind with `id`, `courses = ["A"]` on a water point
    round-trips; `courses` omitted when empty.
  - `start_leg_and_taped_leg_round_trip`: `start_bends`, `start_taped = true`, a course control with
    `taped = true`, `print_area = { min = [0, 0], max = [180, 260] }`.
  - `special_with_unknown_course_is_reported` → `UnknownCourseInSpecial { id: "w1", course: "Z" }`.
  - `duplicate_special_id_across_kinds_is_reported` (a water point and a finish both `id = "F1"`).
  - `adding_a_special_keeps_comments_on_the_others` (identity match by `id`, like the control test from M1).
  - Update fixtures and existing tests to the new shape; `unchanged_save_keeps_bytes` stays green.
- [ ] **Step 2: Run** `cargo test -p freemapper-courseplan-model` → FAIL.
- [ ] **Step 3: Implement** the types, validation, identity key and omitted-default keys.
- [ ] **Step 4: Run** → PASS; fmt, clippy, workspace tests.
- [ ] **Step 5: Commit** `feat(model): crossing points, special-object ids, start legs and print area`.

### Task 2: Faster map index and curve tests

**Files:**
- Modify: `crates/freemapper-courseplan/src/map.rs`, `crates/freemapper-courseplan/src/map/geometry.rs`
- Test: unit tests in both files, `crates/freemapper-courseplan/tests/read_maps.rs`

**Interfaces:**
- Consumes: `omap::SymbolSet` (`symbol_type`, `is_hidden`, `code_for_id`).
- Produces: no API change. `MapIndex::from_document` builds a per-symbol table
  (`id → (type, hidden, code)`) once and looks objects up in it.

- [ ] **Step 1: Write the failing or pinning tests**:
  - `points_after_a_curve_are_kept`: a path curve-then-two-plain-points flattens to the curve's 9 points
    followed by both plain points.
  - `chained_curves_flatten`: two curves in a row give 17 points ending at the second curve's end.
  - `luebeck_index_builds_fast`: `MapIndex::from_document` on `luebeck.omap` finishes in under 5 s in a debug
    build (measure with `std::time::Instant`; generous for slow CI runners; it fails today if the
    lookup is quadratic — record the before/after times in the report).
- [ ] **Step 2: Run** `cargo test -p freemapper-courseplan` → record which tests fail or are slow.
- [ ] **Step 3: Implement** the per-symbol table in `from_document`.
- [ ] **Step 4: Run** → PASS; fmt, clippy.
- [ ] **Step 5: Commit** `perf(map): look up symbols once per symbol, not once per object`.

### Task 3: Course geometry, control numbers and length

**Files:**
- Create: `crates/freemapper-courseplan/src/course.rs`
- Modify: `crates/freemapper-courseplan/src/lib.rs`, `docs/glossary.md` (rows for `LegGeometry`,
  Bahnlänge / course length)
- Test: unit tests in `course.rs`

**Interfaces:**
- Consumes: `Event`, `Course`, `CourseKind`, `Control`, `ControlKind`, `SpecialObject` (Task 1).
- Produces:
  - `enum CourseError { NotSupported { course: String, kind: &'static str }, UnknownCourse(String) }`
    (thiserror; "course X is a team course; only individual courses are supported yet").
  - `struct LegGeometry { from: String, to: String, points: Vec<MapPoint>, taped: bool, length_mm: f64 }`;
    `from`/`to` are the start id, a control code or the finish id; `points` run from the `from` position
    through the bends to the `to` position.
  - `fn course_legs(event: &Event, course: &Course) -> Result<Vec<LegGeometry>, CourseError>`.
  - `fn control_numbers(event: &Event, course: &Course) -> Vec<Option<u32>>`, one entry per course control,
    `None` for crossing points, numbers from 1 upward in sequence (a control visited twice gets two numbers).
  - `fn course_length_m(event: &Event, course: &Course, map_scale: u32) -> Result<f64, CourseError>`: the sum of
    all leg polylines in map mm × `map_scale` / 1000 (WKB B 7.2.5: straight lines, plus the drawn detours as
    bends, plus marked routes, which are legs with `taped = true` and are measured along their bends).

- [ ] **Step 1: Write the failing tests** (map scale 4000; start S1 at [0, 0], 31 at [100, 0], 32 at
  [100, 100], finish F1 at [0, 100]):
  - `straight_course_length`: S1→31→32→F1 = 300 mm → 1200.0 m.
  - `bend_adds_detour`: a bend [150, 50] on the leg 31→32 → 100 + 2·√5000 + 100 mm → 1165.685… m
    (assert within 0.01 m).
  - `start_bends_count`: `start_bends = [[0, -50]]` → the first leg is 50 + √(100² + 50²) mm.
  - `crossing_point_is_a_vertex_without_number`: a crossing point 33 at [100, 50] between 31 and 32 → same
    length as straight, `control_numbers` = `[Some(1), None, Some(2)]`.
  - `butterfly_numbers_both_visits`: sequence 31, 32, 31 → numbers 1, 2, 3; three legs between controls.
  - `team_course_is_not_supported` → `CourseError::NotSupported`.
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** → PASS; fmt, clippy.
- [ ] **Step 5: Commit** `feat(course): legs, control numbers and course length`.

### Task 4: Edit operations

**Files:**
- Create: `crates/freemapper-courseplan/src/edit.rs`
- Modify: `crates/freemapper-courseplan/src/lib.rs`
- Test: unit tests in `edit.rs`

**Interfaces:**
- Consumes: model types and `validate`; `MapIndex::extent` (Task 2 unchanged API).
- Produces (each operation works on a clone of the event, validates it, and only on success writes it back
  into `event`; on error `event` is unchanged):
  - `enum EditError { Invalid(Vec<ModelError>), NotFound { what: &'static str, id: String }, CodeTaken(ControlCode), IdTaken(String), OutsideMap { at: MapPoint, min: MapPoint, max: MapPoint }, InUse { id: String, courses: Vec<String> }, NotSupported { course: String }, BadIndex { course: String, index: usize } }`
    (thiserror, one plain English sentence each).
  - `fn next_free_code(event: &Event) -> ControlCode`: the lowest integer ≥ 31 that no control uses.
  - `fn add_control(event: &mut Event, extent: Option<(MapPoint, MapPoint)>, at: MapPoint, code: Option<ControlCode>, kind: ControlKind) -> Result<ControlCode, EditError>`;
    `extent` from `MapIndex::extent`, `None` skips the check.
  - `fn move_control(event, extent, code: &ControlCode, at: MapPoint) -> Result<(), EditError>`.
  - `fn remove_control(event, code: &ControlCode) -> Result<Vec<String>, EditError>`: also removes every visit
    in individual courses and returns the course names; a team, relay or score course that uses the code →
    `InUse`.
  - `fn describe_control(event, code, description: ControlDescription) -> Result<(), EditError>`.
  - `fn set_central_number(event, code, number: Option<NumberPlacement>) -> Result<(), EditError>`.
  - `fn add_special(event, extent, special: SpecialObject) -> Result<(), EditError>` (id unique, every point
    inside the extent).
  - `fn remove_special(event, id: &str) -> Result<(), EditError>`: a start or finish a course uses → `InUse`.
  - `fn add_course(event, course: Course) -> Result<(), EditError>`; `fn remove_course(event, name: &str)`.
  - `fn set_course_controls(event, name: &str, codes: &[ControlCode]) -> Result<(), EditError>`: replaces an
    individual course's sequence. The n-th visit of a code in the new sequence keeps the number, bends and
    `taped` of the n-th visit of that code in the old sequence; new visits start empty.
  - `enum LegStart { Start, After(usize) }` (index into the course's controls).
  - `fn set_leg(event, name: &str, leg: LegStart, bends: Vec<MapPoint>, taped: bool) -> Result<(), EditError>`.
  - `fn set_course_number(event, name: &str, index: usize, number: Option<NumberPlacement>) -> Result<(), EditError>`.
  - `fn set_course_details(event, name: &str, classes: Option<Vec<String>>, climb_m: Option<Option<u32>>, planned_print_scale: Option<Option<u32>>) -> Result<(), EditError>`
    (`None` = leave as is).
  - `fn add_class(event, class: Class) -> Result<(), EditError>`; `fn remove_class(event, name: &str)` also
    removes the name from every course's `classes`.

- [ ] **Step 1: Write the failing tests**, among them:
  - `next_free_code_skips_used`: codes 31, 32, 34 used → 33.
  - `add_control_outside_map_is_refused` → `OutsideMap` whose message contains both extent corners.
  - `remove_control_takes_it_out_of_courses`: 32 used by courses A and B → returns `["A", "B"]`, both
    sequences lose 32, the control is gone.
  - `set_course_controls_keeps_leg_data_per_visit`: old 31(bends b1), 32, 31(bends b2) → new 31, 31, 33: first
    31 keeps b1, second keeps b2, 33 empty.
  - `remove_used_start_is_refused` → `InUse { id: "S1", courses: ["A"] }`.
  - `failed_operation_leaves_event_unchanged`: a `set_leg` with a bad index → `BadIndex`, event equal to before.
  - `remove_class_also_clears_course_classes`.
  - `team_course_edit_is_not_supported`.
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** → PASS; fmt, clippy.
- [ ] **Step 5: Commit** `feat(edit): edit controls, specials, courses and classes safely`.

### Task 5: Automatic control number placement

**Files:**
- Create: `crates/freemapper-courseplan/src/numbers.rs`
- Modify: `crates/freemapper-courseplan/src/lib.rs`
- Test: unit tests in `numbers.rs`

**Interfaces:**
- Consumes: `course_legs` (Task 3), `MapKind`.
- Produces:
  - `struct OverprintSizes { circle_diameter_mm: f64, line_width_mm: f64, number_height_mm: f64 }` in print mm;
    `fn overprint_sizes(kind: MapKind) -> OverprintSizes` and `fn base_scale(kind: MapKind) -> u32`: sprint
    (ISSprOM 2019-2) 6.0, 0.35, 4.0 at 1:4000; schoolyard the same print values at 1:1000 (assumption until
    the school set's overprint is checked; say so in a code comment); forest (ISOM 2017-2) 5.0, 0.35, 4.0 at
    1:15000.
  - `fn auto_place(event: &Event, course: &Course, map_scale: u32, kind: MapKind) -> Result<Vec<NumberPlacement>, CourseError>`:
    one placement per numbered course control, in sequence; crossing points are skipped.
  - Algorithm (deterministic, no map detail yet; the shared `freemapper-placement` module replaces it later,
    ADR 0013/0014):
    - gap 0.5 mm print; number box `0.6 × height × digits` wide and `height` high, in print mm at the base
      scale, converted to map mm as in the Global Constraints;
    - 16 candidate directions, every 22.5°, starting at 45°;
    - hard conflicts: the box overlaps another circle of the course, a leg polyline of the course (widened by
      half the line width), or a number already placed for this course;
    - choose the candidate with the fewest hard conflicts, then the one closest to the bisector of the widest
      gap between the legs at that control, then the lowest direction.
  - `fn place_numbers(event: &mut Event, course_name: &str, map_scale: u32, kind: MapKind, overwrite: bool) -> Result<usize, EditError>`
    in `edit.rs` or `numbers.rs` (implementer's choice, say which): writes `auto_place` results into the
    course controls that have no number (or all with `overwrite`), returns how many it set.

- [ ] **Step 1: Write the failing tests**:
  - `sizes_follow_the_map_kind`.
  - `number_avoids_both_legs`: control with incoming leg from the west and outgoing leg to the south → chosen
    direction lies in the north-east quadrant (between 0° and 90°).
  - `number_avoids_a_neighbour_circle`: a second control 8 mm (map, 1:4000 map at 1:4000) to the north-east →
    direction not within ±30° of 45°.
  - `numbers_do_not_overlap_each_other`: two close controls → their boxes do not overlap.
  - `crossing_points_get_no_number`.
  - `place_numbers_keeps_existing_unless_overwrite`.
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** → PASS; fmt, clippy.
- [ ] **Step 5: Commit** `feat(numbers): place control numbers clear of legs and circles`.

### Task 6: Program texts in German and English

**Files:**
- Create: `crates/freemapper-courseplan/src/text.rs` (language detection and the message catalog)
- Modify: `crates/freemapper-courseplan/src/lib.rs`, `crates/freemapper-courseplan/src/init.rs`,
  `crates/freemapper-courseplan-cli/src/main.rs`, `docs/glossary.md` (row Sprache / language `Lang`)
- Test: unit tests in `text.rs`, `crates/freemapper-courseplan-cli/tests/cli.rs`

**Interfaces:**
- Consumes: the error types `ProjectError`, `ModelError` (model crate), `InitError`, `CourseError`,
  `EditError`, `OmapError`.
- Produces:
  - `enum Lang { De, En }`; `fn detect_lang(arg: Option<&str>, env: &dyn Fn(&str) -> Option<String>) -> Lang`
    (order as in the Global Constraints; a locale starting with `de` → `De`; anything else → `En`).
  - `trait Localize { fn localize(&self, lang: Lang) -> String; }`, implemented for every error type above;
    the English text equals the type's `Display`. No new crate: the catalog is Rust code (a `match` per
    message), so a missing translation is a compile error. A later move to translation files for more
    languages is left open.
  - CLI: global option `--lang de|en`; help texts of all commands in both languages (build the clap command
    at start-up and set `about`/`help` from the catalog, e.g. with `Command::mut_subcommand`/`mut_arg`);
    `info` labels stay as in M1 in English and get German labels (`Titel`, `Karte`, `Kartenart`, `Posten`,
    `Bahnen (einzeln …, Mannschaft …, Staffel …, Score …)`, `Kategorien`) when the language is German.
  - The M1 messages (init, info, project file errors) are translated.

- [ ] **Step 1: Write the failing tests**:
  - `detects_language_in_order`: `--lang` wins over `FREEMAPPER_LANG`, which wins over `LANG=de_DE.UTF-8`;
    `LANG=C` → English.
  - `every_error_has_both_languages`: for one value of each error variant, `localize(De)` is non-empty and
    differs from `localize(En)`, and `localize(En) == to_string()`.
  - CLI: `init_refuses_forest_map_in_german` (`--lang de` → German stderr), `help_in_german`
    (`--lang de init --help` contains a German phrase), and the existing English tests run with `--lang en`
    or `LANG=C` so they do not depend on the machine.
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** → PASS; fmt, clippy.
- [ ] **Step 5: Commit** `feat(text): program texts in German and English`.

### Task 7: CLI for controls, special objects and classes

**Files:**
- Create: `crates/freemapper-courseplan-cli/src/edit_cmd.rs` (clap types and handlers for the edit commands)
- Modify: `crates/freemapper-courseplan-cli/src/main.rs`
- Test: `crates/freemapper-courseplan-cli/tests/edit.rs`

**Interfaces:**
- Consumes: `edit::*` (Task 4), `ProjectFile`, `read_map_file`, `MapIndex::extent`.
- Produces commands (every one: load the project, read the map when the operation needs the extent, apply,
  save, print one confirmation line from the catalog (Task 6) in the chosen language; on error print
  `error: …` (German `Fehler: …`), exit 1, file unchanged). Points are written
  `X,Y` in map mm; several points are separate arguments.
  - `control add <PROJECT> --at X,Y [--code CODE] [--crossing-point]` → `added control 33 at 12.5,-30`.
  - `control move <PROJECT> <CODE> --at X,Y`; `control remove <PROJECT> <CODE>` (prints the courses it left);
    `control describe <PROJECT> <CODE> [--c ID] [--d ID] [--e ID] [--f ID] [--g ID] [--h ID]` (unset columns
    stay; `--clear` empties all first); `control number <PROJECT> <CODE> --direction DEG --gap MM | --clear`.
  - `special add <PROJECT> --kind start|finish|marked-route|out-of-bounds|forbidden-route|dangerous-area|first-aid|water --id ID (--at X,Y | --points X,Y X,Y …) [--courses A,B]`;
    `special remove <PROJECT> <ID>`.
  - `class add <PROJECT> <NAME> [--rule-category CAT] [--starters N]`; `class remove <PROJECT> <NAME>`.

- [ ] **Step 1: Write the failing tests** (temp dir with `luebeck.omap` and a project made by `init`):
  - `control_add_move_describe_remove`: the full cycle; after each step `info` shows the expected count and the
    TOML contains the expected values.
  - `control_add_picks_next_free_code`.
  - `special_add_start_and_water_point`.
  - `failed_edit_leaves_file_unchanged`: `control move` with an unknown code → exit 1, file byte-identical.
  - `edit_keeps_hand_written_comments`: a comment added by hand to the project file survives `control add`.
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** → PASS; fmt, clippy.
- [ ] **Step 5: Commit** `feat(cli): edit controls, special objects and classes`.

### Task 8: CLI for courses, numbers and lengths

**Files:**
- Modify: `crates/freemapper-courseplan-cli/src/edit_cmd.rs`, `src/main.rs`, `CHANGELOG.md`,
  `CHANGELOG.en.md`, `README.md`, `README.en.md`
- Test: `crates/freemapper-courseplan-cli/tests/edit.rs`, `crates/freemapper-courseplan-cli/tests/cli.rs`

**Interfaces:**
- Consumes: Tasks 3–7.
- Produces:
  - `course add <PROJECT> <NAME> --start ID --finish ID [--controls 31,32,…] [--classes A,B] [--climb M] [--print-scale N]`.
  - `course controls <PROJECT> <NAME> 31,32,…` (replace the sequence); `course remove <PROJECT> <NAME>`.
  - `course leg <PROJECT> <NAME> (--from-start | --after INDEX) [--bends X,Y …] [--taped]` (INDEX counts
    controls from 1, as printed on the map; crossing points count as positions too).
  - `course set <PROJECT> <NAME> [--classes A,B] [--climb M | --no-climb] [--print-scale N | --no-print-scale]`.
  - `course numbers <PROJECT> <NAME> [--overwrite]` → `placed 5 control numbers on course A`.
  - `info` keeps its six M1 lines and adds one line per course:
    `course <NAME>: <n> controls, <km> km, climb <m> m` with km to one decimal (WKB B 7.2.5 format) and
    `climb –` when unset. Crossing points do not count as controls.
  - Changelog entries (both languages) and README status paragraph (what works now) for Tasks 1–8.

- [ ] **Step 1: Write the failing tests**:
  - `course_cycle`: add start, finish, three controls, a course, set its controls, add a bend, place numbers →
    `info` prints `course A: 3 controls, <expected> km, climb – m`; recompute `<expected>` in the test from the
    coordinates.
  - `course_leg_index_counts_from_one`.
  - `info_lines_from_m1_unchanged` (the first six lines keep the M1 format).
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** → PASS; fmt, clippy.
- [ ] **Step 5: Docs** (changelog, README; unslop/humanizer). **Step 6: Commit**
  `feat(cli): set courses, legs and control numbers`.

### Task 9: MCP server

**Files:**
- Create: `crates/freemapper-courseplan-cli/src/mcp.rs`, `crates/freemapper-courseplan-cli/tests/mcp.rs`,
  `docs/manual/de.md` and `docs/manual/en.md` chapter 3 ("Mit einem KI-Assistenten" / "With an AI assistant")
- Modify: `crates/freemapper-courseplan-cli/src/main.rs`, `crates/freemapper-courseplan-cli/Cargo.toml`,
  `Cargo.toml`, `CHANGELOG.md`, `CHANGELOG.en.md`

**Interfaces:**
- Consumes: Tasks 3–8; tool descriptions and server instructions come from the catalog (Task 6) in the language
  detected at server start. Pattern: freemapper's `crates/freemapper-cli/src/mcp.rs` and `tests/mcp.rs`
  (`/Users/henryprivat/Documents/gitrepos/freemapper`), same rmcp version and style.
- Produces: `freemapper-courseplan mcp` serves MCP over stdio (ADR 0004). Every tool takes `project` (absolute
  path or `~/…`; relative → error result "use an absolute path") and returns compact JSON text. Tools:
  - `event_info` (title, map, map kind, extent, counts, per course: controls, length_km, climb_m, numbers set),
  - `list_controls` (code, kind, map_mm, description, number), `list_courses` (name, kind, start, finish,
    controls, classes, length_km), `list_specials`, `list_classes`,
  - `add_control`, `move_control`, `remove_control`, `describe_control`, `set_control_number`,
  - `add_special`, `remove_special`, `add_class`, `remove_class`,
  - `add_course`, `remove_course`, `set_course_controls`, `set_leg`, `set_course_details`, `place_numbers`,
  - `map_features_near` (`x`, `y`, `radius_mm`, optional `limit` default 20 → symbol code, geometry kind,
    distance_mm, part) and `map_extent`.
  - Server instructions (one paragraph): what the program does, the coordinate system (map mm, y down), the
    workflow (event_info → map_extent/map_features_near → add start and finish → add controls → add course →
    place_numbers → event_info), and that only individual courses are supported yet.
  - `init_event` (map path, optional project path and title) wraps `init_project`.
  - Manual chapter 3 in both languages: how to register the server with an AI assistant (command
    `freemapper-courseplan mcp`) and a short example dialogue; follow the style of freemapper's manual
    chapter 4 (`/Users/henryprivat/Documents/gitrepos/freemapper/docs/manual/de.md`).

- [ ] **Step 1: Write the failing tests** in `tests/mcp.rs` (spawn the binary, speak JSON-RPC over stdio as
  freemapper's test does):
  - `lists_the_tools`: `tools/list` contains every tool name above.
  - `sets_a_course_end_to_end`: `init_event` on a copy of `luebeck.omap`, add start, finish, two controls, a
    course, `place_numbers`, `event_info` → one course with 2 controls and a length > 0.
  - `relative_path_is_refused`: `event_info` with `project = "x.toml"` → error result, no file created.
  - `map_features_near_returns_nearest_first`.
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** → PASS; fmt, clippy; check
  `git diff Cargo.lock` against freemapper's lock versions.
- [ ] **Step 5: Docs** (changelog, manual chapter 3; unslop/humanizer). **Step 6: Commit**
  `feat(mcp): serve course setting to AI agents over stdio`.

---

## After the last task

Final whole-branch review on the most capable model, then merge `--no-ff` into `main`, run the checks on
`main`, push to `origin` and `github`, watch CI, remove the worktree and run `cargo clean` there (CLAUDE.md).
Next plan: M2b, the `check` rules of design §5.6.
