# Core Placement and Description Symbols Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** One control number placement (`freemapper_course::placement`) and one description symbol table (`freemapper_course::description_symbols`) in freemapper-core v0.6.0, used by freemapper-courseplan, which then deletes its own copies.

**Architecture:** Core's `placement` already holds the box, the score, the ranking, `GapRange` and the behind test (print ADR 0038, ported from courseplan ADR 0030 to 0032). What it lacks is courseplan's course-level loop: kept numbers fixed and acting as obstacles, numbers placed in running order, circles and legs of the course as obstacles, 45° without legs, and the combination of several `number_gap` rules. These go into a new file `placement/course.rs`, with core input types. A shared table of cases runs against both old implementations first. Where they disagree, core ADR 0016 rules per courseplan ADR 0030 to 0032. The description symbols move from freemapper-print (symbols.xml parser, BSD) and from courseplan (2004 to 2024 forward mapping, inverse) into `description_symbols/`. The inverse is checked against symbols.xml and gets its missing ids. courseplan then pins v0.6.0 and keeps only what depends on its model: rule-profile lookup, `MapKind` sizes, check findings, import/export notes.

**Tech Stack:** Rust 1.99.1 (pinned in all repos), edition 2024. Core: `quick-xml =0.42.0`, `serde =1.0.229`, `toml =1.2.1` (new dev-dependency of `freemapper-course`; already a core workspace dependency). No new third-party crates.

**Spec:** The agreement with freemapper-print of 2026-11-10 (owner of `freemapper-course`), items (A) and (B), summarised in Global Constraints. Read also: core `CLAUDE.md`, core ADR 0005, 0006, 0013, 0014; core `crates/freemapper-course/src/placement/mod.rs`; courseplan ADR 0030, 0031, 0032, 0034, 0035; courseplan design `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.2, §5.3, §5.5a; print `crates/freemapper-print/src/describe/symbols.rs` and `data/README.md`; Purple Pen `src/PurplePenCore/symbols.xml` at `201417f97bb69302e3e0db123f428870e0914515`; IOF Control Descriptions 2024.

## Global Constraints

- Core public API of `placement` stays: no item removed or renamed, no signature changed, **no field added to an existing pub struct** (`Rect`, `Obstacle`, `Placement`, `Score`, `Legs`, `GapRange`, `Target`) and **no variant added to an existing enum** (`ObstacleShape`, `Tier`), because print builds them as literals and matches them. Only new items *(agreed with freemapper-print)*.
- The behaviour of `best`, `best_within`, `score`, `number_box`, `Target::directions`, `candidate`, `GapRange::*` stays. A ruling that would change one of them is sent to freemapper-print over SendMessage and waits for their answer *(agreed with freemapper-print)*.
- The readers and reader tests of `freemapper-course` (`ppen`, `iof_xml`, `fit`) and the writer `ppen/write.rs` stay unchanged.
- Core: GPL-3.0-or-later; `unsafe_code = "forbid"`; `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo clippy -p freemapper-course --no-default-features --all-targets -- -D warnings`, `cargo test --workspace` and `cargo test -p freemapper-course --no-default-features` pass. Error `Display` English, no Fluent in core (core ADR 0010). `freemapper-course` names no other suite crate than omap/georef (existing test).
- Core commits: Conventional Commits, scope `course` (`feat(course): …`, `test(course): …`, `docs(course): …`). Glossary row before or with its code. ADRs 0016 and 0017 in the same branch. Changelog entries in `CHANGELOG.md` and `CHANGELOG.en.md` under "Unveröffentlicht"/"Unreleased", named "freemapper-course:". ADRs, changelogs, READMEs and doc comments longer than a sentence go through the `unslop` and `humanizer` skills (read both SKILL.md files).
- One core release, v0.6.0 (minor; current version 0.5.1), tagged by this session after freemapper-print reviewed the branch. courseplan pins the tag, never a branch.
- courseplan: ADRs 0036 and 0037; texts in `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl` in the same commit; both changelogs, both READMEs equal before every push; design statements marked *(user)*, *(Claude's call)* or *(agreed with freemapper-print)*.
- courseplan's existing number placement tests (`crates/freemapper-courseplan/src/numbers.rs` test module), check tests (`src/check/numbers/tests.rs`), import/export tests and snapshots are the regression net: their scenarios and expected values stay. Allowed edits: imports, `GapRange { min_mm, max_mm }` → `GapRange { min, max }`, `GapRange::FIXED` → `GapRange::DEFAULT`, `.min_x/.min_y/.max_x/.max_y` → `.min.0/.min.1/.max.0/.max.1`, and `#[cfg(test)]` shims of the same name and signature for deleted private helpers. Any other change to a test is a behaviour drift: stop and handle it as in Task 4. All 546+ tests stay green.
- Worktrees: core `/Users/henryprivat/Documents/gitrepos/freemapper-core/.worktrees/course-placement-symbols` (branch `course-placement-symbols`); courseplan `.worktrees/core-placement-symbols` (branch `core-placement-symbols`). `df -h` before long runs; after each merge `cargo clean` in worktree and main checkout, then remove the worktree.
- Determinism: no `HashMap` iteration in results; placements depend only on the inputs.

## Review Focus

1. **Number positions drift between old and new.** A course placed by courseplan before and after the switch gets the same direction and gap at every control, except for the edge cases ruled in core ADR 0016. Tests: Task 2/3 `placement_cases_match_expected` (both repos), Task 10 `example_auto_placements_are_unchanged`.
2. **Gap range handling.** Several `number_gap` rules, one-sided rules, min above max, values with three decimals, invalid values, and gaps below half a line width give the same range and the same positions in both apps. Tests: Task 1 `strictest_equals_per_rule_normalising`; Task 2 cases `three_decimal_min`, `gaps_below_half_line`, `max_only`, `min_above_max`.
3. **Symbol id table errors.** Every id the inverse writes exists in symbols.xml under the file's standard and in a box that accepts its kind, and every Purple Pen id survives import then export. Tests: Task 6 `every_written_id_exists_in_its_standard_and_box`, `every_purple_pen_id_round_trips`.
4. **API breakage for print.** print compiles and its tests pass against the branch without a code change. Tests: Task 7 `print_call_shapes_still_compile`; Task 8 print's `cargo test --workspace` against the branch.
5. **Placement on large maps.** A course on a map with 100 000 line segments places in the same time order as courseplan does today, and map obstacles are converted once per course, not once per control. Tests: Task 4 `large_map_places_a_course_quickly` (core), Task 10 `map_features_are_converted_once` (courseplan).

---

## Part 1: freemapper-core (branch `course-placement-symbols`)

Before Task 1: create the core worktree from `main` at `d033348` (v0.5.1); run `cargo test --workspace` there as baseline.

### Task 1: Course-level placement in core

**Files:**
- Modify: `crates/freemapper-course/src/placement/mod.rs` (new consts and methods, `mod course; pub use course::…`)
- Create: `crates/freemapper-course/src/placement/course.rs`, `crates/freemapper-course/src/placement/course/tests.rs`
- Modify: `docs/glossary.md`

**Interfaces:**
- Consumes: `best`, `Target`, `Legs`, `Obstacle`, `ObstacleShape`, `Tier`, `Placement`, `GapRange`, `BoxGrid`, `Rect`, `direction_to`, private `trim_start` (all in `placement/mod.rs`).
- Produces (all `pub`, re-exported from `placement`):
  - `pub const NO_LEGS_DIRECTION_DEG: f64 = 45.0;`
  - `impl Rect { pub const UNBOUNDED: Rect }` (min `(f64::MIN, f64::MIN)`, max `(f64::MAX, f64::MAX)`)
  - `impl GapRange { pub fn strictest(rules: impl IntoIterator<Item = (Option<f64>, Option<f64>)>) -> Option<(Option<f64>, Option<f64>)> }`: largest usable min and smallest usable max over all rules, `None` when no rule gives a usable value. The caller passes the result to `from_limits` (placement) or `allowed` (check).
  - `pub struct NumberStop { pub centre: Pt, pub size: Option<(f64, f64)>, pub kept: Option<Placement> }`: `size: None` is a stop without number (start, finish, crossing point); it gets neither a number nor a circle obstacle.
  - `pub struct CourseNumbers<'a> { pub stops: &'a [NumberStop], pub legs: &'a [Vec<Pt>], pub circle_r: f64, pub half_line: f64, pub unit: f64, pub gaps: GapRange }`: `legs.len() == stops.len() + 1`, `legs[k]` arrives at `stops[k]`, `legs[k + 1]` leaves it; all lengths in coordinate mm, `gaps` in gap mm (× `unit`).
  - `pub fn place_course_numbers(course: &CourseNumbers<'_>, map: &[Obstacle], bounds: Rect) -> Vec<Option<Placement>>`: one entry per stop.

Behaviour of `place_course_numbers` (port of courseplan `auto_place`, numbers.rs:523–694):
- Kept placements come back unchanged; their boxes are overprint obstacles from the start (label `n{k}`).
- Numbered stops are placed in order. Each placed box is an obstacle for the later ones (`n{k}`).
- Obstacles of stop `k`: the circle of every other numbered stop that is not at the same point (`Circle`, radius `circle_r + half_line`, label `c{j}`); every leg as `Segment`s with `half_width = half_line` (label `l{i}`, one label per leg), with legs `k` and `k + 1` trimmed at the circle (trim length per ruling D2 in Task 4, start with `circle_r + half_line`); `map` as given.
- `Target`: `circle_outer_r = circle_r + half_line`; `legs.incoming` = direction from the stop to the second-to-last point of `legs[k]`; `legs.outgoing` = `legs[k + 1]`; `current = Placement { direction: NO_LEGS_DIRECTION_DEG, gap_mm: gaps.min }`.
- `map` goes into one `BoxGrid` per call (cell = largest number box side, `max_cells` 512, `wide_cells` 64). Each stop passes `best` only the obstacles the grid gives for its `Target::reach()` region.
- The module keeps `placement_module_has_no_crate_imports` green: `course.rs` uses only `super::`.

- [ ] **Step 1: Glossary rows** in `docs/glossary.md`: "gehaltene Nummer | kept number | Nummernlage, die die Platzierung nicht verschiebt; ihr Kasten ist Hindernis für die anderen | `NumberStop::kept`" and "Nummern einer Bahn platzieren | course number placement | automatische Platzierung aller Nummern einer Bahn in Laufreihenfolge, jede gesetzte Nummer Hindernis für die folgenden | `placement::place_course_numbers`, `CourseNumbers`, `NumberStop`".

- [ ] **Step 2: Write the failing tests** in `placement/course/tests.rs` (scenarios ported from courseplan's numbers.rs tests with the same names; sprint sizes: `circle_r` 3.0, `half_line` 0.175, box `(2.4 × digits, 4.0)`, `unit` 1.0):
  - `number_avoids_both_legs`: the box at the result hits neither leg (score has no `l…` label).
  - `number_avoids_a_neighbour_circle`, `numbers_do_not_overlap_each_other`: no `c…`/`n…` label in the result's score.
  - `new_numbers_avoid_kept_numbers`: stop 0 kept at `Placement { direction: 90.0, gap_mm: 0.5 }` comes back equal; stop 1's box does not overlap stop 0's.
  - `kept_number_outside_range_is_not_moved`: kept gap 7.0 with `gaps` 0.5..0.5 comes back as 7.0.
  - `crossing_points_get_no_number`: `size: None` gives `None`, and its position is no circle obstacle.
  - `a_control_without_legs_points_to_45_degrees`: one stop, `legs` two empty polylines, no obstacles → `direction == 45.0`, `gap_mm == 0.5`.
  - `the_same_control_twice_ignores_its_own_place`: two stops at the same point, both numbered → neither counts the other's circle.
  - `strictest_takes_largest_min_and_smallest_max`: `[(Some(0.5), Some(5.0)), (Some(1.0), None), (None, Some(3.0))]` → `Some((Some(1.0), Some(3.0)))`; `[(None, None)]` → `None`; `[(Some(f64::NAN), Some(-1.0))]` → `None`.
  - `strictest_equals_per_rule_normalising`: for every pair of rules drawn from `{None, 0.3, 1.0, 2.5}²`, `GapRange::from_limits` of `strictest` equals courseplan's old order (per rule: clamp and raise max to min, then strictest, then `from_limits`). This proves print's and courseplan's combining agree.

- [ ] **Step 3: Run them to verify they fail.** Run: `cargo test -p freemapper-course placement::course` from the core worktree. Expected: compile errors for the missing items.

- [ ] **Step 4: Implement** `NO_LEGS_DIRECTION_DEG`, `Rect::UNBOUNDED`, `GapRange::strictest` in `placement/mod.rs` (reuse private `usable`), and `NumberStop`, `CourseNumbers`, `place_course_numbers` in `placement/course.rs`, as described above.

- [ ] **Step 5: Run the crate tests.** Run: `cargo test -p freemapper-course && cargo test -p freemapper-course --no-default-features`. Expected: all pass, the existing `placement/tests.rs` untouched.

- [ ] **Step 6: Commit** `feat(course): place the numbers of a course in running order`.

### Task 2: Shared placement cases and the core harness

**Files:**
- Create: `crates/freemapper-course/tests/fixtures/placement/cases.toml`, `crates/freemapper-course/tests/placement_cases.rs`
- Modify: `crates/freemapper-course/Cargo.toml` (`[dev-dependencies] toml.workspace = true`)

**Interfaces:**
- Consumes: `place_course_numbers`, `CourseNumbers`, `NumberStop`, `GapRange::from_limits` (Task 1).
- Produces: the case format that Task 3 reads in courseplan, unchanged:
  - top level: `format = 1`, a header comment with source (this repository, commit), date and licence (GPL-3.0-or-later).
  - `[[case]]`: `name`, `map_kind` (`"sprint"` | `"forest"`), `map_scale` (u32), `start` `[x, y]`, `finish` `[x, y]`, `gap_min`, `gap_max` (optional floats, the rule limits before rounding), `stops` = array of `{ at = [x, y], number = u32 | absent for a crossing point, kept = { direction, gap } | absent, bends_before = [[x, y], …] | absent }`, `map_points` = `[[x, y], …]`, `map_lines` = `[[[x, y], …], …]`, `expected` = array of `{ direction, gap }` | `"none"` per stop (absent until Task 4).
  - coordinates are map mm, y down. Sizes come from the kind: sprint circle 6.0, line 0.35, number height 4.0, base scale 4000; forest 5.0, 0.35, 4.0, 15000; box `0.6 × height × digits` by `height`; factor `base / map_scale` (copied from courseplan `overprint_sizes`/`base_scale`, test-local).
  - map points become zero-length `Segment`s with `half_width` 0.5, map lines `Segment`s with `half_width` 0.15, both `Tier::MapSymbol`, one label per feature (ruling D3 below).

- [ ] **Step 1: Write the cases.** One `[[case]]` per scenario: `behind_straight_leg`, `obtuse_angle_outer_bisector`, `blocked_behind`, `neighbour_circle`, `numbers_avoid_each_other`, `kept_numbers_are_obstacles`, `kept_gap_outside_range`, `crossing_point`, `same_control_twice`, `no_legs` (start = finish = the control), `zero_length_incoming`, `bend_before_control`, `point_symbol_at_exactly_half_mm`, `line_symbol_at_exactly_0_15_mm`, `every_direction_covered`, `overprint_outranks_map`, `gap_range_moves_out_before_turning` (0.5..3.0), `three_decimal_min` (0.333..2.0), `gaps_below_half_line` (0.01..0.1), `max_only` (–..2.0), `min_above_max` (2.0..1.0), `bisector_tie_at_half_tenth` (incoming 0°, outgoing 74.5°, so the bisector is 217.25°), `forest_at_1_10000`, `two_and_three_digit_numbers`.

- [ ] **Step 2: Write the harness** `placement_cases_match_expected`: reads every case and runs `place_course_numbers` with `Rect::UNBOUNDED`. With `expected` present it asserts direction within 1e-9 and gap within 1e-9. With `PLACEMENT_CASES_DUMP=<path>` set, it writes the results as `[[result]] name, placements` to that path instead of asserting. A second test, `cases_have_unique_names`, checks the names.

- [ ] **Step 3: Dump core's results.** Run: `PLACEMENT_CASES_DUMP=<scratchpad>/core-results.toml cargo test -p freemapper-course --test placement_cases`. Expected: PASS, file written (cases have no `expected` yet).

- [ ] **Step 4: Commit** `test(course): shared number placement cases`.

### Task 3 (courseplan, old code): The same cases against courseplan's placement

Runs in the courseplan worktree on current `main` code; no production change.

**Files:**
- Create: `crates/freemapper-courseplan/tests/placement_cases.rs`, `crates/freemapper-courseplan/tests/fixtures/placement/cases.toml` (copy of core's, byte for byte), `crates/freemapper-courseplan/tests/fixtures/placement/luebeck-sprint-auto.toml`
- Modify: `tests/fixtures/README.md` (source: freemapper-core commit of Task 2, date, GPL-3.0-or-later)

**Interfaces:**
- Consumes: the case format of Task 2; courseplan `numbers::auto_place(event, course, map, map_scale, kind, gaps, keep_existing)`, `MapIndex`, `Event`.
- Produces: `<scratchpad>/courseplan-results.toml` in the same `[[result]]` format; `luebeck-sprint-auto.toml` (per course of `examples/luebeck-sprint`: name and the `auto_place(…, keep_existing = false)` placements of today's code).

- [ ] **Step 1: Write the harness.** `placement_cases_match_expected` turns each case into an `Event` with one individual course (controls with codes `31…`, the start, the finish, bends from `bends_before`, kept numbers as `CourseControl.number`) and a `MapIndex` with point and line features (symbol code `"999"`). It calls `auto_place` with `GapRange { min_mm, max_mm }` built the way `gap_range` builds it from one rule with `gap_min`/`gap_max`, and `keep_existing = true`. Dump and assert work as in Task 2. Add `example_auto_placements_are_unchanged`: it loads `examples/luebeck-sprint` and compares `auto_place` of every course with `luebeck-sprint-auto.toml`. With `PLACEMENT_CASES_DUMP` set, it writes that file instead.
- [ ] **Step 2: Dump.** Run: `PLACEMENT_CASES_DUMP=<scratchpad>/courseplan-results.toml cargo test -p freemapper-courseplan --test placement_cases`. Expected: PASS, both files written.
- [ ] **Step 3: Time the old placement on a large map.** An `#[ignore]` test `large_map_timing` builds 100 000 line segments in a 400 × 400 mm square and a 40-control course, then prints the `auto_place` time in release mode. Run: `cargo test --release -p freemapper-courseplan --test placement_cases -- --ignored --nocapture large_map_timing`. Record the time for Task 4.
- [ ] **Step 4: Commit** (courseplan branch) `test(numbers): shared placement cases and example baseline`.

### Task 4: Characterisation rulings (core ADR 0016)

**Files:**
- Create: `docs/adr/0016-course-number-placement-shared-with-courseplan.md`
- Modify: `docs/adr/README.md`, `crates/freemapper-course/tests/fixtures/placement/cases.toml` (`expected`), `crates/freemapper-course/src/placement/course.rs` (only if a ruling says so), `crates/freemapper-course/tests/placement_cases.rs`

**Interfaces:**
- Consumes: both result files (Tasks 2, 3).
- Produces: `expected` in every case; ADR 0016 with a table *case | core | courseplan | ruling | reason (ADR and rule)*.

Divergences known from reading the code. Proposed rulings are *(Claude's call)*; the results decide:

| | Where | courseplan (old) | core | Proposed ruling |
|---|---|---|---|---|
| D1 | candidate rounding | rounds `preferred + 15·s`, s = −11..12, before normalising | rounds `preferred + 15·k`, k = 0..23 | core; differs only when `preferred × 10` ends in exactly .5 |
| D2 | leg trim at own circle (behind test, own legs) | circle radius `r` | `r + half_line` | core (the leg starts outside the circle line); differs only when `gap × unit < half_line` |
| D3 | point symbol hit | distance `<= 0.5` | `Circle` is `< radius` | courseplan, by zero-length `Segment` in the caller; no core change |
| D4 | gap rounding | rounds only the candidates (tolerance 1e-9 mm), behind test at unrounded min | `from_limits` rounds (1e-6 of 0.01 mm), behind test at rounded min | core: the gap evaluated is the gap stored (ADR 0032) |
| D5 | no legs | 45° | `current` | same once `place_course_numbers` passes `NO_LEGS_DIRECTION_DEG` |
| D6 | tie order and last key | direction-major, plain `<` on direction | gap-major, EPS on every key | core; differs only for keys within 1e-9 |

- [ ] **Step 1: Diff** both result files per case and placement. Map every difference to D1–D6. A difference outside D1–D6, or one in a case with the default gap 0.5 mm and ordinary legs (Review Focus 1), is a real drift. Find its cause before ruling.
- [ ] **Step 2: Rule.** Decide each difference by courseplan ADR 0030–0032 and core's module rules 1–4. A ruling that needs a change in `best`, `Target::directions`, `candidate` or `GapRange` goes to freemapper-print over SendMessage first (Global Constraints). Wait for their answer before changing anything. A ruling for courseplan's side changes only `place_course_numbers` or the harness's obstacle building. Write ADR 0016: context, the table, the decisions, consequences (which courseplan positions move and why), and `Source:` (agreement of 2026-11-10, courseplan ADR 0030–0032, print ADR 0038).
- [ ] **Step 3: Fill `expected`** with the ruled values. Copy `cases.toml` to the courseplan fixture.
- [ ] **Step 4: Add the performance test** `large_map_places_a_course_quickly` to `placement_cases.rs`: the same synthetic map and course as Task 3 Step 3, `place_course_numbers` in debug, assert under `max(2 s, 30 × the recorded courseplan release time)` (3 × the old time, with a factor of 10 for debug over release). Record both times in ADR 0016.
- [ ] **Step 5: Run.** `cargo test -p freemapper-course --test placement_cases` (core) PASS. In courseplan: `cargo test -p freemapper-courseplan --test placement_cases`, which fails exactly in the cases ruled for core; list them in ADR 0016.
- [ ] **Step 6: Commit** `docs(course): ADR 0016 rulings on shared number placement` (with the `expected` values and any `course.rs` change).

### Task 5: Description symbol database moves to core

**Files:**
- Create: `crates/freemapper-course/data/purple-pen-symbols.xml` (byte copy of print's `crates/freemapper-print/data/purple-pen-symbols.xml`, SHA-256 `84b51153333961675cf2de5555e540d2e6360f3735a2ac3e61cdf248b724ad05`), `crates/freemapper-course/data/README.md` (print's provenance table and the full BSD notice, plus "moved from freemapper-print `<commit>`"), `crates/freemapper-course/src/description_symbols/mod.rs`, `…/description_symbols/db.rs`, `…/description_symbols/tests.rs`
- Modify: `crates/freemapper-course/src/lib.rs` (`pub mod description_symbols;`, crate doc), `crates/freemapper-course/Cargo.toml` (`include` gains `"data/**"`), `.gitattributes` (`crates/freemapper-course/data/purple-pen-symbols.xml -text`), `crates/freemapper-course/README.md` (third-party notice), `docs/glossary.md`

**Interfaces:**
- Produces (moved from print `describe/symbols.rs`, names unchanged): `pub enum Standard { Iof2004, Iof2018, Iof2024 }`, `Standard::from_description(&str) -> Option<Standard>`, `pub struct Stroke`, `pub struct Circle`, `pub struct SymbolGeometry` (with `is_empty()`), `pub fn symbol(id: &str, standard: Standard) -> Option<&'static SymbolGeometry>`, `pub const FLATNESS`, `MAX_POINTS_PER_SYMBOL`, `MAX_DEPTH`. Points are `(f64, f64)`, so print converts to its own `Pt`. Changed for core: `pub fn symbol_text(id: &str, lang: TextLang) -> Option<&'static str>` with new `pub enum TextLang { De, En }` (print's `SheetLang` stays in print). New: `pub fn symbol_kind(id: &str, standard: Standard) -> Option<char>` (the `kind` attribute) and `pub fn exists(id: &str, standard: Standard) -> bool`. Task 6 uses both.

- [ ] **Step 1: Glossary rows**, taken over from print's glossary lines 147–153: Beschreibungssymbol, Beschreibungsstandard, Symbolspalte, plus "Symbolart | symbol kind | Spalte, für die Purple Pen ein Symbol vorsieht (`kind` in symbols.xml) | `symbol_kind`".
- [ ] **Step 2: Move print's tests** `all_symbols_parse`, `known_symbol_geometry`, `every_symbol_stays_in_its_box_and_under_the_cap`, `standards_from_the_file`, `texts_in_de_and_en`, `parser_is_bounded` into `description_symbols/tests.rs` unchanged except paths and `TextLang`. Add `kinds_from_the_file`: `symbol_kind("11.7", Iof2004) == Some('G')`, `symbol_kind("11.7", Iof2018) == Some('F')`, `symbol_kind("10.1single", Iof2018) == Some('F')`, `exists("2.10", Iof2004) == false`, `exists("2.10", Iof2018) == true`, `exists("0.1NE", Iof2018) == false`.
- [ ] **Step 3: Run to verify they fail.** `cargo test -p freemapper-course description_symbols`. Expected: module missing.
- [ ] **Step 4: Move the parser** into `db.rs` (print's code, its provenance comment, `OnceLock` cache), add `TextLang`, `symbol_kind`, `exists`.
- [ ] **Step 5: Run** both feature sets and `cargo package -p freemapper-course --list | grep data/`. Expected: tests PASS, the XML and the README listed.
- [ ] **Step 6: Commit** `feat(course): description symbol database from freemapper-print`.

### Task 6: Forward and inverse symbol mapping (core ADR 0017)

**Files:**
- Create: `crates/freemapper-course/src/description_symbols/iof_2024.rs`, `…/description_symbols/ppen.rs`, `docs/adr/0017-description-symbols-in-core.md`
- Modify: `…/description_symbols/mod.rs`, `…/description_symbols/tests.rs`, `docs/adr/README.md`, `docs/glossary.md`

**Interfaces:**
- Consumes: `Standard`, `exists`, `symbol_kind` (Task 5).
- Produces:
  - `pub const BEND_2024: &str = "11.1"`, `pub const DIRECTIONS: [&str; 8]`, `pub struct Mapped { pub id: String, pub single: bool }`, `pub fn iof_2024(ppen_id: &str) -> Option<Mapped>`: courseplan's `ppen_import.rs:782–855`, moved with the same results.
  - `pub enum Refusal { NoPurplePenId, NotInStandard(Standard) }`
  - `pub struct Inverse { pub id: String, pub chosen: bool }`: `chosen` marks a mapping where Purple Pen has several ids for one 2024 symbol and the writer picked one.
  - `pub fn ppen_symbol(iof_2024_id: &str, standard: Standard) -> Result<Inverse, Refusal>`
  - `pub enum ColumnNote { Refused { column: char, id: String, reason: Refusal }, VariantChosen { column: char, id: String, written: String } }`
  - `pub struct PpenColumns { pub columns: [Option<String>; 6], pub notes: Vec<ColumnNote> }`
  - `pub fn ppen_columns(columns: &[Option<String>; 6], standard: Standard) -> PpenColumns`: columns C to H in 2024 ids in, Purple Pen ids out. Courseplan's rules stay: in 2004 the bend 11.1 in F goes to G as 11.7, and crossing-point handling stays in courseplan.

Decisions per id, to verify against symbols.xml and the IOF 2024 standard before coding. Each one becomes a row in ADR 0017:

| 2024 id (column) | Today | Proposed *(Claude's call)* |
|---|---|---|
| 0.1NE/NW/SE/SW (C) | written unchanged; Purple Pen has no such id | map to 0.2NE/NW/SE/SW: 0.1 is the cardinal, 0.2 the diagonal form of "which of similar features" |
| 0.2N/E/S/W (C) | same | map to 0.1N/E/S/W |
| plain 9.1, 9.3 (F) | written unchanged; Purple Pen has only `…high`/`…deep` | map to `9.1high`/`9.3high`, `chosen = true` (same drawing, Purple Pen's text says "height"); open question 1 |
| 2.10, 15.6, PP 5.25/5.26 (from 2024 5.23/5.24), 11.7 in F, in a 2004 file | written, but Purple Pen 2004 lacks them | `Refused(NotInStandard(Iof2004))` |
| 2024 5.25, 5.26 | left out, noted | `Refused(NoPurplePenId)` (unchanged) |
| `single` (D == E with 2024 10.1/10.2 in F, 12.14 in G) | `<id>single` written into D; box D does not accept kind F/G | D keeps the feature, E empty, the combination column gets `10.1single`/`10.2single`/`11.15single`; there is no `12.14single`: check what 2024 12.14 maps to, map only ids that exist |
| any other id | first hit of the reverse scan | unchanged; every written id passes `exists` and `symbol_kind` (box D accepts D, box E accepts E or D, as `purple_pen_strict.rs` `COLUMN_KINDS`) |

- [ ] **Step 1: Verify the table.** For every row, look up the ids in `data/purple-pen-symbols.xml` (kind, standard, names in en/de) and in IOF 2024. For every id in both PP standards (1.10, 11.7, 12.3, …), compare the English names. If a meaning differs between 2004 and 2018, `iof_2024` gains a `standard` parameter; record that in ADR 0017. Write ADR 0017: the database move (print ADR 0036 named as source), the forward mapping (courseplan ADR 0034), the table above with the verified rulings, BSD provenance, ~670 KB embedded data, and `Source:`.
- [ ] **Step 2: Write the failing tests** in `tests.rs`:
  - `forward_covers_every_purple_pen_id_but_12_3_and_13_6`: over all ids of the database, `iof_2024` is `None` only for `12.3` and `13.6` (courseplan's `every_purple_pen_id_has_a_2024_number_but_radio_control_and_map_issue`, now on the database instead of a list).
  - `every_purple_pen_id_round_trips`: for every id and both PP standards where it exists, `ppen_symbol(iof_2024(id).id, standard)` gives back `id`, or a variant whose `chosen` is set and that the ADR lists (`9.1deep` → `9.1high`).
  - `every_written_id_exists_in_its_standard_and_box`: for each 2024 id reachable from the forward mapping, plus the ids in the table, each in every column, `ppen_columns` writes only ids with `exists(id, standard)` and a `symbol_kind` accepted by the box.
  - `cardinal_and_diagonal_directions_are_normalised`: `ppen_symbol("0.1NE", Iof2018) == Ok(Inverse { id: "0.2NE", chosen: false })`, and `"0.2W"` → `"0.1W"`.
  - `plain_height_is_written_as_high`: `"9.1"` → `Inverse { id: "9.1high", chosen: true }`, and `ppen_columns` adds `VariantChosen { column: 'F', id: "9.1", written: "9.1high" }`.
  - `symbols_of_2018_are_refused_in_2004`: `"2.10"`, `"15.6"`, `"5.23"` in `Iof2004` give `Err(NotInStandard(Iof2004))`.
  - `single_variant_goes_to_the_combination_column`: D = E = `"5.2"`, F = `"10.1"` → D `"5.2"`, E `None`, F `"10.1single"`. Its inverse through `iof_2024` and courseplan's D→E copy gives the input back.
  - `bend_moves_to_g_in_2004` and `plain_id_wins_over_variants`: courseplan's existing export cases, moved.
- [ ] **Step 3: Run to verify they fail.** `cargo test -p freemapper-course description_symbols`. Expected: missing items.
- [ ] **Step 4: Implement** `iof_2024.rs` (courseplan's match, moved) and `ppen.rs`: build the inverse once in a `OnceLock<BTreeMap<String, String>>` by courseplan's scan, apply the table's rulings, and check each written id against `exists`/`symbol_kind`.
- [ ] **Step 5: Run** `cargo test -p freemapper-course` in both feature sets. Expected: PASS, reader and writer tests unchanged.
- [ ] **Step 6: Glossary** row "Rückabbildung | inverse mapping | IOF-2024-Kennung zur Purple-Pen-Kennung des Standards der Datei; verweigert, was Purple Pen in diesem Standard nicht kennt | `ppen_symbol`, `ppen_columns`, `Refusal`". **Commit** `feat(course): description symbol mapping between Purple Pen and IOF 2024`.

### Task 7: API guard, changelog, docs

**Files:**
- Create: `crates/freemapper-course/tests/placement_api.rs`
- Modify: `CHANGELOG.md`, `CHANGELOG.en.md`, `README.md`, `README.en.md` (crate description, if it lists modules), `crates/freemapper-course/README.md`, `crates/freemapper-course/src/lib.rs` (crate doc names the two new modules)

- [ ] **Step 1: Write `print_call_shapes_still_compile`.** It uses every placement item print uses, in print's shape (from print `numbers.rs` and `rules.rs`): struct literals with all fields of `GapRange { min, max }`, `Placement`, `Legs`, `Target`, `Obstacle`, `Rect`, and exhaustive `match`es on `Tier` and `ObstacleShape`. It also calls `best`, `best_within`, `score`, `evaluations`, `number_box`, `placement_of`, `BoxGrid::new`/`query`, `GapRange::from_limits`/`allowed`/`steps`/`contains`, `normalize_direction`, `direction_to`, `angle_diff`, `outer_bisector`, `candidate`, and reads `DEFAULT_GAP_MM`, `GAP_STEP_MM`, `GAP_LIMITS_MM`, `CANDIDATE_STEP_DEG`, `CANDIDATES`, `OUTSIDE`. It asserts `GapRange::DEFAULT == GapRange { min: 0.5, max: 0.5 }`. A field or variant added later fails to compile here first.
- [ ] **Step 2: Run** `cargo test -p freemapper-course --test placement_api`. Expected: PASS.
- [ ] **Step 3: Changelog** under "Hinzugefügt"/"Added": "freemapper-course: `placement::place_course_numbers` …, `GapRange::strictest`, `Rect::UNBOUNDED`, `NO_LEGS_DIRECTION_DEG` (ADR 0016)" and "freemapper-course: `description_symbols` …, symbols.xml from Purple Pen under BSD (ADR 0017)". No "Geändert" entry: nothing existing changes. Run `unslop` and `humanizer` on all prose.
- [ ] **Step 4: Full check.** `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo clippy -p freemapper-course --no-default-features --all-targets -- -D warnings && cargo test --workspace && cargo test -p freemapper-course --no-default-features`. Expected: all pass.
- [ ] **Step 5: Commit** `docs(course): changelog and API guard for v0.6.0`.

### Task 8: Review by freemapper-print

- [ ] **Step 1: Run print's tests against the branch** without touching print's files: in `/Users/henryprivat/Documents/gitrepos/freemapper-print`, `CARGO_TARGET_DIR=<scratchpad>/print-target cargo test --workspace --config 'patch."https://github.com/HenryJobst/freemapper-core".freemapper-course.path="<core worktree>/crates/freemapper-course"'`, with the same `--config` for `freemapper-omap`, `freemapper-georef`, `freemapper-rules`, `freemapper-text` (all patched together, so no core crate comes twice). Expected: PASS with no change to print. Delete `<scratchpad>/print-target` afterwards.
- [ ] **Step 2: Request the review** over SendMessage to the freemapper-print session. Send the branch name, the commit, a summary of what was added, ADR 0016 and 0017, Step 1's result, and the open rulings that touch `best`/`directions` (if any). Push the branch to `origin` only (no merge).
- [ ] **Step 3: Address findings** with one commit per finding (`fix(course): …`). Rerun Task 7 Step 4 and Step 1 here. Repeat until print approves; record the approval (date, commit) in ADR 0016 and 0017's `Source:` lines *(agreed with freemapper-print)*.
- [ ] **Step 4: Merge** `git merge --no-ff course-placement-symbols` into core `main`. Run Task 7 Step 4 on `main`, push `main` to `origin` and `github`.

### Task 9: Core release v0.6.0

- [ ] **Step 1:** Set `[workspace.package] version = "0.6.0"`, run `cargo update -w`, and turn "Unveröffentlicht"/"Unreleased" into `## [0.6.0] - YYYY-MM-DD` in both changelogs, keeping an empty "Unveröffentlicht"/"Unreleased" above.
- [ ] **Step 2:** Commit `release: v0.6.0 (course placement and description symbols)`, then `git tag -a v0.6.0 -m "v0.6.0"`, then push `main` and the tag to `origin` and `github`.
- [ ] **Step 3:** Watch CI on GitHub (`gh run watch` on the tag's run). Expected: green on Linux, macOS, Windows.
- [ ] **Step 4:** Tell freemapper-print and freemapper over SendMessage which version to pin and what print can now delete: `describe/symbols.rs`, `data/purple-pen-symbols.xml` and, if it wants, `rules.rs` `gap_limits` in favour of `GapRange::strictest`. Clean the core worktree (`cargo clean`) and remove it.

---

## Part 2: freemapper-courseplan (branch `core-placement-symbols`, after Task 9)

### Task 10: courseplan places numbers with the core module (ADR 0036)

**Files:**
- Modify: `Cargo.toml` (all five core crates `tag = "v0.6.0"`), `Cargo.lock`, `crates/freemapper-courseplan/src/numbers.rs`, `crates/freemapper-courseplan/src/check/numbers.rs` (imports), `crates/freemapper-courseplan-cli/src/edit_cmd.rs:1052-1061`, `crates/freemapper-courseplan-cli/src/mcp.rs:1159-1166` (`GapRange::DEFAULT`)
- Create: `docs/adr/0036-number-placement-uses-the-core-module.md`
- Modify: `docs/adr/README.md`, `docs/glossary.md` (row "Nummernabstand" names `placement::GapRange`)
- Test: `crates/freemapper-courseplan/src/numbers.rs` test module (allowed edits only), `crates/freemapper-courseplan/tests/placement_cases.rs`

**Interfaces:**
- Consumes: `freemapper_course::placement::{place_course_numbers, CourseNumbers, NumberStop, GapRange, Obstacle, ObstacleShape, Tier, Rect, Placement, number_box, normalize_direction, GAP_LIMITS_MM}` (v0.6.0).
- Keeps (app-specific, signatures unchanged except the `GapRange` type): `pub fn gap_range(rules: &RuleSet, map_kind: MapKind, discipline: Discipline, print_scale: u32) -> placement::GapRange`, which now applies the per-rule `rule_sides` and then `GapRange::strictest` and `GapRange::from_limits`, returning `GapRange::DEFAULT` without a rule. Also kept: `normalise_sides`, `rule_sides` (the check reads unrounded limits), `OverprintSizes`, `overprint_sizes`, `base_scale`, `circle_outer_radius_mm`, `number_box_mm`, `number_centre` (on core `number_box`), `pub const MAX_GAP_MM: f64 = GAP_LIMITS_MM.1`, `gap_in_range`, `pub use placement::normalize_direction`, `auto_place` and `place_numbers` (same signatures, `gaps: placement::GapRange`).
- Deletes: courseplan's `Rect`, `orient`, `segments_cross`, `point_segment_distance`, `Rank`, `candidate_gaps`, `candidate_direction`, `box_distance`, `direction_to`, `angle_diff`, `outer_bisector`, `preferred`, `trim_start`, `CANDIDATE_*`, `GAP_STEP_MM`, `MAX_GAP_STEPS`, `MIN_RULE_GAP_MM`, `EPS`, and the search loop of `auto_place`.
- New private: `fn map_obstacle(feature: &MapFeature) -> Option<Obstacle>` and `fn map_obstacles(map: &MapIndex) -> Vec<Obstacle>` (all features through `map_obstacle`; `MapIndex.features` is private, so add `pub(crate) fn features(&self) -> &[MapFeature]` in `map.rs` if no accessor exists). Point features become zero-length `Segment`s with `half_width` `POINT_SYMBOL_RADIUS_MM` (0.5); line features become `Segment`s with `LINE_SYMBOL_HALF_WIDTH_MM` (0.15). Both are `Tier::MapSymbol`, labelled by feature index. Areas and `601` (via `symbol_matches`) give none. It is called once per `auto_place`.

`auto_place` keeps its level logic (central number, course number, `keep_existing`; ruling R11). It builds `NumberStop`s (`size` from `number_box_mm × factor` for numbered stops; `kept` from those levels), the legs from `course_legs`, `circle_r = circle_diameter_mm × factor / 2`, `half_line = line_width_mm × factor / 2` and `unit = factor`. It calls `place_course_numbers(…, &map_obstacles(map), Rect::UNBOUNDED)` and returns the `Some` entries as `NumberPlacement`.

- [ ] **Step 1: Pin v0.6.0**, `cargo update -p freemapper-course` (and the other core crates), `cargo test --workspace`. Expected: PASS (the core changes are additive).
- [ ] **Step 2: Write the failing test** `map_features_are_converted_once` (numbers.rs test module): a `MapIndex` with 3 point, 2 line, 1 area and 1 `601` feature gives 5 obstacles: points with `half_width` 0.5 and zero length, lines with 0.15.
- [ ] **Step 3: Rewrite `numbers.rs`** as above. Re-point the test module with the allowed edits only. Shims under `#[cfg(test)]`: `number_rect` (returns core `Rect`), `candidate_direction(p, step)` → `candidate(p, step.rem_euclid(24) as usize)`, `candidate_gaps(g)` → `g.steps()`, `feature_covered(r, f)` → `map_obstacle(f).is_some_and(|o| o.shape.hits(r))`.
- [ ] **Step 4: Run** `cargo test --workspace`. Expected: all 546+ tests plus the new ones PASS. `placement_cases_match_expected` now passes the cases ruled for core in Task 4. `example_auto_placements_are_unchanged` passes, or differs only where ADR 0016 says so; if so, regenerate `luebeck-sprint-auto.toml` and name each changed control in ADR 0036. Any other failure is drift: go back to Task 4 (a core fix means v0.6.1, agreed with print).
- [ ] **Step 5: ADR 0036**: courseplan places with core `place_course_numbers` (agreement of 2026-11-10); what stays here; the rulings of core ADR 0016 that move courseplan positions. Mark them *(agreed with freemapper-print)* or *(Claude's call)*.
- [ ] **Step 6: Full check** `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`; **commit** `refactor(numbers): place numbers with freemapper-course placement`.

### Task 11: Import and export use the core symbol mapping (ADR 0037)

**Files:**
- Modify: `crates/freemapper-courseplan/src/ppen_import.rs` (delete `iof_2024`, `Mapped`, `DIRECTIONS`, `BEND_2024` at lines 782–855; use `description_symbols`), `crates/freemapper-courseplan/src/ppen_export.rs` (delete `inverse`, `SINGLE_CAPABLE`; `ppen_columns` becomes a thin wrapper that turns `ControlDescription` into `[Option<String>; 6]` and back, and turns `ColumnNote`s into `ExportNote`s), `crates/freemapper-courseplan/src/text.rs`, `crates/freemapper-courseplan/i18n/en/courseplan.ftl`, `crates/freemapper-courseplan/i18n/de/courseplan.ftl`
- Create: `docs/adr/0037-description-symbols-from-the-core-crate.md`
- Test: `crates/freemapper-courseplan/src/ppen_export/tests.rs`, `src/ppen_import/tests.rs`

**Interfaces:**
- Consumes: `freemapper_course::description_symbols::{iof_2024, Mapped, DIRECTIONS, BEND_2024, Standard, ppen_columns, PpenColumns, ColumnNote, Refusal}` (Task 6).
- Produces: `ExportNote::SymbolNotInStandard { control, column: char, id: String, standard: String }` and `ExportNote::SymbolVariantChosen { control, column: char, id: String, written: String }`. `ExportNote::SymbolWithoutPpenId` stays for `Refusal::NoPurplePenId`. `base_standard` results go through `Standard::from_description`.

- [ ] **Step 1: Write the failing tests** in `ppen_export/tests.rs`: `symbol_of_2018_in_a_2004_base_is_refused_and_noted` (a 2004 base, 2.10 in D: not written, `SymbolNotInStandard { id: "2.10", standard: "2004" }`), `plain_height_is_written_as_high_and_noted` (9.1 in F: file holds `9.1high`, note `SymbolVariantChosen`), `single_variant_round_trips_through_the_file` (import a `.ppen` with D = 5.2, F = `10.1single`, export, read back with `read_ppen`: F = `10.1single`, D = 5.2, E empty; the strict test `purple_pen_strict.rs` rules hold), `diagonal_column_c_is_written_as_0_2`.
- [ ] **Step 2: Run to verify they fail.** `cargo test -p freemapper-courseplan ppen_export`. Expected: FAIL on the old inverse.
- [ ] **Step 3: Switch** import and export to the core module and add the two notes with Fluent messages in `en` and `de`. Texts follow the style of the `SymbolWithoutPpenId` message and go through `unslop`/`humanizer`.
- [ ] **Step 4: Run** `cargo test --workspace`. Expected: PASS. Existing import/export tests unchanged; `ppen_symbol_inverts_iof_2024`, `plain_id_wins_over_variants` and `every_purple_pen_id_has_a_2024_number_but_radio_control_and_map_issue` now exercise the core module.
- [ ] **Step 5: ADR 0037**: the tables now live in core (ADR 0017), the export fixes (single variants, standard refusals, direction normalising, plain 9.1/9.3) and the new notes. **Commit** `fix(export): description symbols through freemapper-course; refuse ids the file's standard lacks`.

### Task 12: Docs, final review, merge, release

**Files:**
- Modify: `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.2 and §5.3 (lines 279–280: placement from core, *(agreed with freemapper-print, 2026-11-10)*), §10 (drop "The simple number placement until the shared module exists"), `CHANGELOG.md`, `CHANGELOG.en.md`, `README.md`, `README.en.md`, the ADRs 0030–0032 "Consequences" (one line each: implementation now in core, see ADR 0036)

- [ ] **Step 1: Changelog** under "Unveröffentlicht"/"Unreleased". "Behoben"/"Fixed": the export writes `single` variants into the right box and refuses 2018-only symbols in 2004 files with a note. "Geändert"/"Changed": number placement and the symbol tables come from freemapper-core 0.6.0, and positions move only in the cases ADR 0036 names. Both languages, through `unslop`/`humanizer`.
- [ ] **Step 2: READMEs** (status paragraph, core version, features) in both languages, kept equal.
- [ ] **Step 3: Final review** of the whole branch (superpowers:requesting-code-review), with ADR 0016 and this plan's Review Focus as the brief. Fix the findings or park them with rulings.
- [ ] **Step 4: Merge** `--no-ff` into `main`, run fmt, clippy and tests on `main`, and push `main` to `origin` and `github`. Then run `cargo clean` in the worktree and the main checkout and remove the worktree.
- [ ] **Step 5: Release v0.7.3** (patch: export fixes) once CI on `main` is green. Follow the CLAUDE.md release steps, watch the release workflow, report the URL. Tell freemapper-print over SendMessage that courseplan runs on v0.6.0 and that its own copies can go.

## Open questions for the user

1. **Plain 9.1/9.3 on export** (Task 6 table). The IOF 2024 standard has one symbol each for "height or depth". Purple Pen splits them into `…high`/`…deep`, and today's import folds both into the plain id. The proposal writes `…high` and adds a note. The alternative keeps `high`/`deep` as a suffix in the project, like the direction suffixes, so a depth survives import then export. That would change the import and older projects. The plan follows the proposal unless the user picks the alternative.
2. **Number positions that move.** If Task 4 finds a difference in ordinary cases (gap 0.5 mm, normal legs) rather than only the edge cases D1–D6, the work stops there, and the user decides whether courseplan accepts the core result or core must take courseplan's.
