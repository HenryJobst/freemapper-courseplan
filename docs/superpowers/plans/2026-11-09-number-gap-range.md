# Number Gap Range Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The number gap gets a minimum and a maximum from the rule profile. Automatic placement moves a
number outward up to the maximum before it turns away from the preferred direction, and `check` warns
about stored gaps outside the range.

**Architecture:** A new rule key `number_gap` with `min_gap_mm`/`max_gap_mm` in `wkb-2026.toml`. A
`GapRange` in `numbers.rs` is resolved from the event's rule set for one course and passed into
`auto_place`/`place_numbers`. A new check module `check/numbers.rs` reports stored placements outside the
range.

**Tech Stack:** Rust 2024, existing crates only.

**Spec:** `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.3, §5.6; ADR 0030, ADR 0031.

## Global Constraints

- User decisions (2026-11-09):
  - default range 0.5 to 5.0 mm print mm, measured as in ADR 0030;
  - both values are rule-profile values;
  - "gap before direction": the preferred direction at a larger gap beats another direction at a
    smaller gap, given equal conflicts;
  - a stored gap outside the range is a **warning**.
- Agreed with freemapper-print (2026-11-09):
  - rule key `number_gap` with values `min_gap_mm`, `max_gap_mm`;
  - gaps go from min to max in 0.5 mm steps, the last step being max;
  - ranking key `(overprint, map, off_preferred, gap, off_outer, direction)`;
  - "behind blocked" is tested at the min gap;
  - with no applicable rule (or the rule switched off), placement uses a fixed 0.5 mm;
  - when several rules apply, the strictest wins: the largest min and the smallest max;
  - input still accepts 0 < gap ≤ 10 mm (`MAX_GAP_MM`).
- English code and comments; program texts in both `.ftl` files in the same commit; no new dependencies;
  `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` pass.
- User-facing prose goes through the `unslop` and `humanizer` skills (read both SKILL.md files).

## Review Focus

1. A min above the max after merging overlays (for example overlay min 6, base max 5). Placement must not
   loop or return an empty candidate list. Expected: max is raised to min.
   - Test: `gap_range_with_min_above_max_uses_min` (Task 1).
2. A central number seen from several courses. It must be reported once per rule, at the control, not once
   per course.
   - Test: `central_number_outside_range_reported_once` (Task 1).
3. A kept or manual number at 8 mm. Placement keeps it unchanged, and only `check` warns.
   - Test: `kept_number_outside_range_is_not_moved` (Task 2).
4. A max that is not on the 0.5 mm grid (for example 1.2). The candidates end exactly at 1.2.
   - Test: `candidate_gaps_end_at_max` (Task 2).
5. The rule switched off by an overlay. Placement falls back to 0.5 mm fixed, and check reports nothing.
   - Tests: `gap_range_without_rule_is_fixed` (Task 1) and the existing switched-off machinery.

---

### Task 1: Rule `number_gap`, `GapRange` and the check

**Files:**
- Modify: `crates/freemapper-courseplan/rules/wkb-2026.toml`: a new rule after `straight_angle_hint`.
- Modify: `crates/freemapper-courseplan/src/rules.rs`: `KNOWN_KEYS`.
- Modify: `crates/freemapper-courseplan/src/numbers.rs`: `GapRange` and `gap_range`.
- Create: `crates/freemapper-courseplan/src/check/numbers.rs`, with tests in the same file, or in
  `check/numbers/tests.rs` like `legs`.
- Modify: `crates/freemapper-courseplan/src/check/mod.rs` (`mod numbers;`, `CHECKS`).
- Modify: `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl`.
- Modify: `crates/freemapper-courseplan/rules/README.md`, `README.en.md` (key table).
- Modify: `docs/glossary.md`.

**Interfaces:**
- Produces:
  - `pub struct GapRange { pub min_mm: f64, pub max_mm: f64 }` (`Debug, Clone, Copy, PartialEq`) with
    `pub const FIXED: GapRange = GapRange { min_mm: 0.5, max_mm: 0.5 }`.
  - `pub fn gap_range(rules: &RuleSet, map_kind: MapKind, discipline: Discipline, print_scale: u32) -> GapRange`
    in `numbers.rs`.
  - Check key `number_gap`.

- [ ] **Step 1: The profile rule.**

  ```toml
  [[rules]]
  id = "wkb2026.number.gap"
  key = "number_gap"
  severity = "warning"
  values = { min_gap_mm = 0.5, max_gap_mm = 5.0 }
  source = { doc = "claude", section = "number gap plan, Task 1" }
  ```

  The `note` (de/en) says three things:
  - ISOM/ISSprOM symbol 704 and WKB B 7.2.6 ask only that the number sits close to the circle and clearly
    belongs to it, without a value;
  - the number needs white space to the circle line;
  - the user set 0.5 and 5 mm.

  Add `"number_gap"` to `KNOWN_KEYS` after `"straight_angle"`.

- [ ] **Step 2: Failing tests for `gap_range`** (in `numbers.rs` tests):
  - `gap_range_reads_the_profile`: `load_rule_set("wkb-2026", &[])` for Sprint, training, 4000 gives
    `GapRange { min_mm: 0.5, max_mm: 5.0 }`.
  - `gap_range_without_rule_is_fixed`: the same set with the `number_gap` rule removed gives
    `GapRange::FIXED`.
  - `gap_range_takes_the_strictest_rule`: push a second rule with min 1.0 and max 3.0. Result:
    `{1.0, 3.0}`.
  - `gap_range_with_min_above_max_uses_min`: one rule with min 6.0 and max 5.0. Result: `{6.0, 6.0}`.

- [ ] **Step 3: Implement `gap_range`.**
  - It looks at the active rules with key `number_gap` whose `applies(map_kind, discipline, print_scale)`.
  - From those it takes the largest `min_gap_mm` and the smallest `max_gap_mm`.
  - A rule without a value leaves that side open.
  - No rule at all gives `FIXED`. Only a max gives min 0.5. Only a min gives max = min.
  - The result's max is `max(min, max)`.

- [ ] **Step 4: Failing check tests**, built like the `straight_angle` tests with `check/testing.rs` helpers:
  - `course_number_below_min_warns`: a course number at gap 0.3. One warning `check-number-gap-small` with
    `gap` 0.3 and `limit` 0.5, at the incoming leg of that control (`course.leg_location(k)`, where leg
    `k` arrives at course control `k`).
  - `course_number_above_max_warns`: gap 6.0 gives `check-number-gap-large` with `limit` 5.0.
  - `gap_inside_range_is_silent`: gaps 0.5 and 5.0 give no finding.
  - `central_number_outside_range_reported_once`: a central number at 7.0 on a control used by two
    courses without own numbers. Exactly one finding, at `Location::Control { code }`.
  - `course_number_overrides_central`: a central number at 7.0 and the course's own number at 1.0. No
    finding for that course. The other course without its own number still reports the central one.
  - `stricter_sibling_reports_alone`: two rules with min 0.5 and min 1.0, gap 0.3. Only the min-1.0 rule
    reports. Same for max: rules with 5.0 and 3.0, gap 6.0, only the 3.0 rule reports (mirror of R15).
  - `controls_without_placement_are_skipped`: no stored number means no finding.

- [ ] **Step 5: Implement `number_gap`** in `check/numbers.rs`.
  - It takes `pub(super) fn number_gap(ctx, run, findings, skipped)` with the signature of `acute_angle`.
  - It reads `min_gap_mm` and `max_gap_mm`. Each is optional, but at least one is required, otherwise
    the rule is skipped the same way a rule with a missing value is skipped.
  - Per run course, it goes through the course controls of an individual course. The effective placement
    is the course control's own `number`, else the event control's central `number`.
  - Central findings are deduplicated by code over all courses of this run.
  - A gap is outside the range when it is `below(gap, min)` or `above(gap, max)`, with the 1e-9
    tolerance the module already uses.
  - Register it as `Check::at("number_gap", numbers::number_gap, &[Place::Leg, Place::Control])`.

  Message texts (en / de), with `gap` and `limit` as `ArgValue::Number`:
  - `check-number-gap-small`
    - en: `the number of control { $stop } is { $gap } mm from the circle, at least { $limit } mm wanted`
    - de: `Die Nummer von Posten { $stop } steht { $gap } mm vom Kreis entfernt, erwünscht sind mindestens { $limit } mm`
  - `check-number-gap-large`
    - en: `… at most { $limit } mm wanted`
    - de: `… erwünscht sind höchstens { $limit } mm`
  - `stop` is the control code, as in `check-straight-angle`.

- [ ] **Step 6: Docs in the same task.**
  - Rules README de/en: a key-table row for `number_gap` with both values.
  - Glossary row "Abstandsbereich der Nummer | number gap range | kleinster und größter erwünschter
    Nummernabstand laut Prüfregel (im Profil 0,5 bis 5 mm) | `number_gap`, `GapRange`".

- [ ] **Step 7: Run** `cargo test -p freemapper-courseplan`, then fmt and clippy. All pass. The rule-file
  tests in `rules.rs` (value names carry units, keys known) also pass.

- [ ] **Step 8: Commit** `feat(check): number gap range rule and check`.

### Task 2: Placement within the gap range

**Files:**
- Modify: `crates/freemapper-courseplan/src/numbers.rs` (`auto_place`, `place_numbers`, `Rank`, candidate loop).
- Modify: `crates/freemapper-courseplan-cli/src/edit_cmd.rs` (`CourseCmd::Numbers`, about line 1034).
- Modify: `crates/freemapper-courseplan-cli/src/mcp.rs` (`place_numbers` tool).
- Modify: `crates/freemapper-courseplan-cli/tests/edit.rs`.
- Modify: `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl` (`cli-help-course-numbers`,
  `mcp-tool-place-numbers`).

**Interfaces:**
- Consumes: `GapRange`, `gap_range` (Task 1).
- Produces:
  - `auto_place(event, course, map: &MapIndex, map_scale, kind, gaps: GapRange, keep_existing)`.
  - `place_numbers(event, course_name, map: &MapIndex, map_scale, kind, gaps: GapRange, overwrite)`.
  - `fn candidate_gaps(gaps: GapRange) -> Vec<f64>`.

- [ ] **Step 1: Failing tests** (in `numbers.rs` tests):
  - `candidate_gaps_end_at_max`:
    - `{0.5, 5.0}` gives `[0.5, 1.0, …, 5.0]` (10 values);
    - `{0.5, 1.2}` gives `[0.5, 1.0, 1.2]`;
    - `FIXED` gives `[0.5]`.
    - Values are rounded to 0.01.
  - `blocked_at_min_gap_moves_outward_before_rotating`:
    - Put a point symbol (in the `MapIndex`) on the preferred ray, just outside the circle line, so that
      the box at 0.5 mm covers it.
    - With `{0.5, 5.0}`, the placement keeps the preferred direction and has a gap above 0.5 and at most
      5.0.
    - With `FIXED`, it turns away from the preferred direction, as before.
  - `gap_rank_prefers_smaller_gap`: `Rank::beats` with equal conflicts and equal `off_preferred` prefers
    the smaller gap. With smaller `off_preferred` and a larger gap, `off_preferred` wins.
  - `kept_number_outside_range_is_not_moved`: a course number at 8.0 with `keep_existing` comes back
    unchanged.
  - Existing tests: pass `GapRange::FIXED` where they call `auto_place`/`place_numbers`, so their
    results stay the same.

- [ ] **Step 2: Implement.**
  - `Rank` gets `gap: f64` between `off_preferred` and `off_outer` in `beats`, with the EPS tolerance
    of the other float fields.
  - The candidate loop runs over directions × `candidate_gaps(gaps)`.
  - `behind_blocked` uses `gaps.min_mm`, and `reach` uses `gaps.max_mm`.
  - The chosen gap goes into `placed` and into `NumberPlacement.gap_mm`.
  - `GAP_PRINT_MM` disappears, or stays only as `GapRange::FIXED`'s value.
  - Update the doc comments of `auto_place`/`place_numbers` and the module doc.

- [ ] **Step 3: Call sites.** CLI `CourseCmd::Numbers` and the MCP `place_numbers` tool:
  - load the event's rule set with the existing helpers (`check_cmd::rule_set`, `Mcp::rule_set`);
  - a profile that fails to load is the same error those helpers already return;
  - compute `gap_range(&rules, map_kind(&doc), event.event.discipline, print_scale_of(course, scale))`
    and pass it in;
  - the course must exist first, otherwise the existing not-found error stays.

- [ ] **Step 4: CLI test.** Extend `course_numbers_avoid_map_symbols` in `tests/edit.rs`, or add
  `course_numbers_move_outward_within_the_range`, where a point symbol right behind a circle makes the
  number keep the behind direction at a gap above 0.5 mm.

- [ ] **Step 5: Help texts.**
  - `cli-help-course-numbers` and `mcp-tool-place-numbers` (en/de) say that the number moves outward up
    to the profile's maximum gap before it turns.
  - Keep them short.

- [ ] **Step 6: Run** the whole workspace: tests, fmt and clippy. Run the example check:
  `cargo run -q -p freemapper-courseplan-cli -- check --all` in `examples/luebeck-sprint`. If its output
  changes, regenerate `check.*.txt`. The expected result is no change, because all gaps there are 0.5.

- [ ] **Step 7: Commit** `feat(numbers): place numbers within the profile's gap range`.

### Task 3: Decision record and user docs

**Files:**
- Create: `docs/adr/0032-number-gap-range-from-the-rule-profile.md`, with an index row in
  `docs/adr/README.md`.
- Modify: `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md`:
  - §5.3: a new bullet on the gap range, marked *(user, 2026-11-09)* and *(agreed with freemapper-print)*;
  - §5.6: a table row "Number gap outside the range (0.5 to 5 mm, warning) *(user, 2026-11-09)*";
  - §4.3: a paragraph on the number label, see below.
- Modify: `docs/glossary.md` (the number label row, see below).
- Modify: `docs/manual/de.md`, `docs/manual/en.md` (the numbers bullet).
- Modify: `README.md`, `README.en.md` (features bullet on numbers and checks).
- Modify: `CHANGELOG.md`, `CHANGELOG.en.md` (under Unveröffentlicht/Unreleased: Added the check, Changed
  the placement).

- [ ] **Step 1: ADR 0032.**
  - Context: no standard value exists. ISOM 2017-2 and ISSprOM 2019-2 symbol 704 and WKB B 7.2.6 are
    qualitative only.
  - Decision: the Global Constraints above, each marked user, agreed or Claude's call. The 0.5 mm grid is
    agreed.
  - Consequences:
    - stored placements are not moved;
    - `check` warns about old manual gaps above 5 mm;
    - placement can now return gaps above 0.5 mm.

- [ ] **Step 2: Number label, recorded now and built with the team/relay/score milestone.** In §4.3,
  *(user, 2026-11-09)* and *(agreed with freemapper-print)*:
  - On relay and score maps the control code may follow the number, as "1-31" or "1 (31)".
  - The setting is `number_label = "number" | "number_dash_code" | "number_paren_code"` on the event, with a
    per-course override.
  - The default is `number`. Of the two code forms, the dash form is the one ISSkiOM 2019 symbol 704
    uses. ISOM and ISSprOM do not standardise one.
  - The label text sets the number box width for placement and drawing.
  - Glossary row "Nummernbeschriftung | number label".

- [ ] **Step 3: Manual, READMEs and changelogs.** Short bullets in both languages. Pass them through
  `unslop` and `humanizer`.

- [ ] **Step 4: Commit** `docs: number gap range (ADR 0032) and number label`.
