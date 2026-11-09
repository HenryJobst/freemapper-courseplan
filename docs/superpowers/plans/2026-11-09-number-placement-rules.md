# Number placement by approach direction and map symbols – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for
> tracking.

**Goal:** `course numbers` / `place_numbers` put a control number behind the circle as seen from the incoming
line, or exactly outside at an almost straight angle. They never cover a point or line symbol of the map when a
free place exists. `check` gets a hint for almost straight angles at controls.

**Architecture:** The simple placement in `crates/freemapper-courseplan/src/numbers.rs` keeps its shape: it
tries candidate directions and ranks them. What changes is the preferred direction, the candidate grid and the
ranking, and the hard conflicts now include map symbols from `map::MapIndex`. The new check key
`straight_angle` sits beside `acute_angle` in `check/legs.rs`. freemapper-print builds the same rules into the
shared placement module (its M3 Task 10), which replaces this code later (ADR 0013, 0014).

**Tech Stack:** Rust 1.99.1, existing `map::MapIndex`, `course::course_legs`, `check` engine, Fluent catalog.
No new dependencies.

**Spec:** `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.3 (number placement) and §5.6
(checks), plus the user's decisions of 2026-11-09 below, which Task 4 writes into §5.3.

User's decisions (2026-11-09), binding:
1. The number covers no point or line symbol of the map in the control area (approach and departure). Area
   symbols may be covered. This wins over 2–4.
2. Preferred direction "behind": the continuation of the incoming connecting line past the control.
3. If "behind" is blocked by the outgoing line (almost straight angle), the preferred direction is exactly
   outside: the bisector of the larger sector between the two lines, the side the tip of the angle points to.
   Example: in from the left, out to the right, 170°, tip slightly up → 90°, straight above.
4. If the preferred place is blocked, take the free place that deviates least from it.
5. `check` reports an angle above 170° between incoming and outgoing line as a hint. Acute angles are already
   checked (`acute_angle`, 45° warning, 90° hint).

Agreed with freemapper-print (2026-11-09): "behind is blocked" means the box at the behind direction, at the
placement's gap, comes within half an overprint line width of the outgoing line. For the first control the
incoming line comes from the start triangle. Point and line symbols are hard conflicts, areas cost nothing
here (print gives them a small cost; this simple placement leaves that out).

## Global Constraints

- Toolchain 1.99.1; `unsafe_code = "forbid"`; `cargo fmt`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo test --workspace --locked` pass after every task. No new dependencies; Cargo.lock unchanged.
- Identifiers and comments English; program texts in `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl`,
  English the reference, every new message in both files in the same commit (ADR 0022).
- Glossary first: each new term gets its row in `docs/glossary.md` in the task that introduces it.
- User-facing text (help, MCP descriptions, manual, changelog, ADR, examples README) goes through the `unslop`
  and `humanizer` skills; a subagent reads both SKILL.md files and applies them.
- Conventional Commits with scope, ending with the session trailer lines.
- `gap_mm` keeps its meaning from ADR 0030 (clearance to the outer edge of the circle line); the automatic gap
  stays `GAP_PRINT_MM = 0.5`.
- Map symbol sizes are in map mm and do not scale with the overprint factor.

## Review Focus

1. A control whose incoming line has no length (start triangle on the control, or a bend on the control):
   placement must still return a direction, from the outer bisector rule with the outgoing line alone (Task 1
   test `incoming_without_length_falls_back_to_the_outgoing_line`).
2. Bends: "incoming line" is the last segment before the circle, from the last bend, not from the previous
   control, as in `acute_angle` (Task 1 test `behind_follows_the_last_bend`).
3. Dense maps where every candidate covers a symbol (forest contours): placement returns the candidate with
   the fewest conflicts and never fails (Task 2 test `every_direction_covered_takes_the_fewest_conflicts`).
4. A point symbol whose centre lies just outside the box but whose drawn symbol reaches under it counts as
   covered (Task 2 test `point_symbol_counts_with_its_radius`).
5. Crossing points get neither a number nor a `straight_angle` finding (Task 3 test
   `straight_angle_skips_crossing_points`).

---

### Task 1: Preferred direction behind the circle, outer bisector at almost straight angles

**Files:**
- Modify: `crates/freemapper-courseplan/src/numbers.rs` (module doc, `preferred_direction`, candidate loop in
  `auto_place`, constants, tests)
- Modify: `docs/glossary.md` (rows below)

**Interfaces:**
- Consumes: `number_rect`, `trim_start`, `direction_to`, `angle_diff`, `GAP_PRINT_MM` (existing, same file).
- Produces for Task 2: in `auto_place`, a per-candidate ranking key
  `(conflicts: usize, off_preferred: f64, off_outer: f64, direction: f64)`, smallest wins, with `conflicts`
  the count of hard conflicts. Task 2 adds map symbols to `conflicts`. Private helpers
  `fn outer_bisector(incoming: Option<f64>, outgoing: Option<f64>) -> f64` (today's `preferred_direction`,
  renamed, same body) and
  `fn preferred(incoming: Option<f64>, outgoing: Option<f64>, behind_blocked: bool) -> f64`.

Rules for the code:
- `incoming` = direction from the control to the last point before it on the incoming leg (as today's
  `in_dir`); `outgoing` = direction to the first point after it (as today's `out_dir`).
- behind = `incoming + 180` (normalised).
- `behind_blocked`: the box `number_rect(here, behind, number, GAP_PRINT_MM, …)` has
  `distance_to_segment ≤ half_line` to any segment of the outgoing leg trimmed by the circle radius (the same
  trimmed polyline the conflict test uses).
- `preferred`: behind if `incoming` is `Some` and not `behind_blocked`, else `outer_bisector(incoming,
  outgoing)`.
- Candidates: `preferred + 15° · k` for `k = 0, ±1, …, ±11` and `k = 12` (24 directions, each once),
  normalised to [0, 360). Remove `CANDIDATE_COUNT`, `CANDIDATE_START`, `CANDIDATE_STEP`; add
  `const CANDIDATE_STEP_DEG: f64 = 15.0;` and `const CANDIDATE_STEPS: i32 = 12;`.
- Ranking key as above; `off_preferred = angle_diff(direction, preferred)`, `off_outer = angle_diff(direction,
  outer_bisector(incoming, outgoing))`. Compare with `EPS` tolerance on the floats, as the current loop does.

- [ ] **Step 1: Write the failing tests** in the `numbers::tests` module, using the existing `event`, `n`,
  `place` helpers (map y is down; direction 90 is −y):

```rust
#[test]
fn preferred_is_behind_the_incoming_leg() {
    // In from the west, out to the south: behind is east (0°), free of the outgoing line.
    let e = event((70.0, 100.0), &[n("31", 100.0, 100.0)], (100.0, 140.0));
    assert!(angle_diff(place(&e, 4000)[0].direction, 0.0) < 1e-9);
}

#[test]
fn obtuse_angle_prefers_outer_bisector() {
    // The user's example: in from the left, out to the right, 170°, the tip slightly up
    // (the control lies above the line from start to finish). Number straight above: 90°.
    let rise = 30.0 * 5f64.to_radians().tan();
    let e = event((70.0, 100.0 + rise), &[n("31", 100.0, 100.0)], (130.0, 100.0 + rise));
    assert!(angle_diff(place(&e, 4000)[0].direction, 90.0) < 1e-9);
}

#[test]
fn blocked_preferred_takes_nearest_free_direction() {
    // Behind (east) is taken by the circle of control 33, 9 mm east of 31 and reached from the
    // south-east, so the number moves by the smallest step that clears it, towards the outer side.
    // Expected: angle_diff(direction, 0.0) is 15 or 30 (the smallest k that is free), and the box
    // keeps radius + half line away from 33's centre.
}

#[test]
fn ties_prefer_the_outer_side() {
    // Preferred 0° blocked symmetrically (+15 and −15 equally free): the candidate closer to the
    // outer bisector wins.
}

#[test]
fn behind_follows_the_last_bend() {
    // Previous control due west, but a bend south of the control: the incoming line arrives from
    // the south, so behind is north (90°).
}

#[test]
fn incoming_without_length_falls_back_to_the_outgoing_line() {
    // Start triangle on the control: no incoming direction; the number goes opposite the
    // outgoing line (outer_bisector with one direction).
}
```

  For the three tests given as comments, write the event and assertions the comment states; the expected
  values are the ones the comment names.

- [ ] **Step 2: Run them and see them fail**

Run: `cargo test -p freemapper-courseplan numbers::`
Expected: the new tests FAIL (directions from the old bisector rule); old tests pass.

- [ ] **Step 3: Implement** the rules above in `auto_place`. Update the module doc comment (first paragraph:
  behind the circle, outer bisector at almost straight angles, fewest conflicts first).

- [ ] **Step 4: Update the old tests** whose expectations depended on the old rule. Keep their purpose:
  - `ties_resolve_to_the_lowest_direction`: delete; `ties_prefer_the_outer_side` replaces it.
  - `number_avoids_both_legs`: expect 225° (behind the start at 45°) and keep the range assertion.
  - `number_avoids_a_neighbour_circle`: fix the comment (behind is 0°, not the bisector) and keep both
    assertions; adjust the neighbour position only if it no longer forces a move.
  - `numbers_do_not_overlap_each_other`, `new_numbers_avoid_kept_numbers`, `central_number_wins_over_automatic_placement`,
    `overwrite_does_not_seed_kept_numbers`: straight east–west course, behind blocked, outer bisector 90°.
    They should pass unchanged; fix comments that name the old rule.

- [ ] **Step 5: Run the whole workspace**

Run: `cargo test --workspace --locked`
Expected: PASS. A CLI/MCP test that pins a placed direction gets the new value; check it is the value rules
2–4 give before changing it.

- [ ] **Step 6: Glossary rows** in `docs/glossary.md`, next to "Nummernabstand":
  - `| bevorzugte Nummernrichtung | preferred number direction | Richtung, in der die automatische Platzierung die Postennummer zuerst versucht: hinter dem Kreis in Verlängerung des ankommenden Strichs, bei fast gestrecktem Winkel genau außen. | `preferred` |`
  - `| Außenseite des Winkels | outer side of the angle | Der größere der beiden Bereiche, in die ankommender und abgehender Strich den Raum um den Kreis teilen; dorthin zeigt die Spitze des Winkels. Ihre Mitte ist die Winkelhalbierende außen. | `outer_bisector` |`

- [ ] **Step 7: Commit**

```bash
git add crates/freemapper-courseplan/src/numbers.rs docs/glossary.md
git commit -m "feat(numbers): place numbers behind the circle, outside at straight angles"
```

### Task 2: Numbers do not cover point and line symbols

**Files:**
- Modify: `crates/freemapper-courseplan/src/numbers.rs` (`auto_place`, `place_numbers` signatures, conflicts,
  tests)
- Modify: `crates/freemapper-courseplan-cli/src/edit_cmd.rs` (`CourseCmd::Numbers`, ~line 1030)
- Modify: `crates/freemapper-courseplan-cli/src/mcp.rs` (`place_numbers` tool, ~line 1124)
- Modify: `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl` (`cli-help-course-numbers`,
  `mcp-tool-place-numbers`)

**Interfaces:**
- Consumes: Task 1's ranking key; `crate::map::{MapIndex, FeatureGeometry}` (`MapIndex::from_document`,
  `features_near(p, radius_mm) -> Vec<(&MapFeature, f64)>`).
- Produces:
  - `pub fn auto_place(event: &Event, course: &Course, map: &MapIndex, map_scale: u32, kind: MapKind,
    keep_existing: bool) -> Result<Vec<NumberPlacement>, CourseError>`
  - `pub fn place_numbers(event: &mut Event, course_name: &str, map: &MapIndex, map_scale: u32, kind: MapKind,
    overwrite: bool) -> Result<usize, EditError>`
  - CLI and MCP build the index with `MapIndex::from_document(&doc)` from the map they already read.

Rules for the code:
- Constants in map mm: `const POINT_SYMBOL_RADIUS_MM: f64 = 0.5;` (a point symbol is covered when the box
  comes within this distance of its position) and `const LINE_SYMBOL_HALF_WIDTH_MM: f64 = 0.15;` (a line is
  covered when any segment comes within this distance of the box). Areas, including their outlines, never
  count. Both values are Claude's call; Task 4 records them in the ADR.
- Per control, query `map.features_near(here, reach)` once, with `reach` = circle radius + half line + gap +
  box diagonal + `POINT_SYMBOL_RADIUS_MM` (all in map mm), and test only those features per candidate.
- Each covered point or line feature adds 1 to `conflicts`, the same tier as circles, legs and other numbers.
- Text objects and sheet content are already left out by `MapIndex`.

- [ ] **Step 1: Write the failing tests** in `numbers::tests`. Build maps with
  `crate::check::testing::{map, point_object, line_object, area_object, POINT_113, LINE_502, AREA_520}` and
  `MapIndex::from_document(&map(4000, objects))`; extend `place` to take the index (`MapIndex::default()` for
  the old tests).

```rust
#[test]
fn point_and_line_symbols_are_hard_area_symbols_soft() {
    // In from the west, out to the south; behind is east (0°).
    // (a) A 113 point 5.5 mm east of the control (under the 0° box): the number leaves 0°.
    // (b) A 502 line running north–south 5.5 mm east: the number leaves 0°.
    // (c) A 520 area covering the whole neighbourhood: the number stays at 0°.
}

#[test]
fn point_symbol_counts_with_its_radius() {
    // A 113 point 0.3 mm outside the 0° box (box from number_rect): still covered, the number moves.
}

#[test]
fn every_direction_covered_takes_the_fewest_conflicts() {
    // Concentric-ish 502 lines around the control so every candidate covers at least one; one
    // direction covers exactly one, all others two or more: that direction wins, and auto_place
    // returns Ok.
}
```

  Write the events and assertions the comments state.

- [ ] **Step 2: Run them and see them fail**

Run: `cargo test -p freemapper-courseplan numbers::`
Expected: compile errors, then FAIL until the map is used.

- [ ] **Step 3: Implement** the signatures and conflicts above; update both call sites in the CLI crate.

- [ ] **Step 4: Help texts.** Replace in both `.ftl` files:
  - `cli-help-course-numbers`: en `Place the control numbers of a course behind the circle as seen from the
    incoming leg, clear of legs, circles and point and line symbols of the map. Runs only when called, so call
    it again after changing legs; central and course numbers stay`; de with the same content in the style of
    the existing German line.
  - `mcp-tool-place-numbers`: same first sentence, rest unchanged.
  Pass both through `unslop`/`humanizer`.

- [ ] **Step 5: Run the whole workspace**

Run: `cargo test --workspace --locked`
Expected: PASS. CLI and MCP tests on `luebeck.omap` still place 7 numbers; a test that pins a direction gets
the new value after you have checked it against rules 1–4.

- [ ] **Step 6: Commit**

```bash
git add crates/freemapper-courseplan crates/freemapper-courseplan-cli
git commit -m "feat(numbers): keep control numbers off point and line symbols"
```

### Task 3: `straight_angle` check

**Files:**
- Modify: `crates/freemapper-courseplan/src/check/legs.rs` (new `straight_angle`, next to `acute_angle`)
- Modify: `crates/freemapper-courseplan/src/check/legs/tests.rs`
- Modify: `crates/freemapper-courseplan/src/check/mod.rs` (`CHECKS`)
- Modify: `crates/freemapper-courseplan/src/rules.rs` (`KNOWN_KEYS`)
- Modify: `crates/freemapper-courseplan/rules/wkb-2026.toml` (leg shapes section, after `acute_angle_hint`)
- Modify: `crates/freemapper-courseplan/rules/README.md`, `README.en.md` (key table)
- Modify: `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl`
- Modify: `docs/glossary.md`

**Interfaces:**
- Consumes: `angle_at`, `course_stops`, `RuleRun::finding`, `siblings_for`, `number`, `value_number`, `below`
  (as `acute_angle` uses them).
- Produces: key `straight_angle`, value `degrees`, message id `check-straight-angle` with args `stop`,
  `degrees`, `limit`; `Check::at("straight_angle", legs::straight_angle, &[Place::Leg])` right after
  `acute_angle` in `CHECKS`.

Rules for the code:
- Same angle as `acute_angle` (last bend or stop before, first bend or stop after; none without length).
- Reports when `angle > degrees`, at punched controls only: crossing points run straight on by nature, start
  and finish have no angle.
- Several rules of the key: a place gets only the rule with the largest limit it exceeds, so the most
  extreme angle carries the strictest rule (mirror of ruling R15).
- Shown angle: whole degrees rounded up (`degrees_above`, the counterpart of `degrees_below`), so an angle
  above its limit never shows as the limit.
- Finding at the incoming leg of the course (`course.leg_location(i)`), as `acute_angle`.

Profile entry (copy verbatim):

```toml
[[rules]]
id = "wkb2026.leg.straight_angle_hint"
key = "straight_angle"
severity = "hint"
values = { degrees = 170.0 }
source = { doc = "claude", section = "number placement plan, Task 3" }
note = { de = "Läuft die Bahn am Posten fast geradeaus weiter, ändert er die Laufrichtung kaum, und hinter dem Kreis ist kein Platz für die Nummer. Die 170° hat der Nutzer festgelegt.", en = "When the course runs almost straight on at a control, the control hardly changes the running direction, and there is no room for the number behind the circle. The user set the 170°." }
```

Texts:
- en: `check-straight-angle = the legs at control { $stop } form an angle of { $degrees }°, at most { $limit }° wanted`
- de: `check-straight-angle = Die Strecken am Posten { $stop } bilden einen Winkel von { $degrees }°, erwünscht sind höchstens { $limit }°`
- rules README row, de: `| `straight_angle` | Winkel zwischen den Strecken an einem Posten über `degrees`, fast geradeaus |`;
  en: `| `straight_angle` | Angle between the legs at a control above `degrees`, almost straight on |`.
- Glossary row: `| fast gestreckter Winkel | almost straight angle | Winkel zwischen ankommender und abgehender Strecke an einem Posten über dem Wert der Prüfregel (im Profil 170°): Die Bahn läuft fast geradeaus weiter. Gemessen wie der spitze Winkel. | `straight_angle` |`

- [ ] **Step 1: Write the failing tests** in `check/legs/tests.rs`, with `angle_event(degrees)` and a
  `straight_rule()` (`degrees = 170.0`, hint):

```rust
#[test]
fn a_straight_angle_at_a_control_is_reported_at_the_incoming_leg() {
    let r = report(&angle_event(175.0), &map(4000, vec![]), vec![straight_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(r.findings[0].location, leg("A", 0));
    assert_eq!(en(&r.findings[0]), "the legs at control 31 form an angle of 175°, at most 170° wanted");
    assert_eq!(de(&r.findings[0]), "Die Strecken am Posten 31 bilden einen Winkel von 175°, erwünscht sind höchstens 170°");
    let r = report(&angle_event(160.0), &map(4000, vec![]), vec![straight_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_straight_angle_shows_rounded_up() {
    // 170.4° shows as 171°, never as the limit.
}

#[test]
fn straight_angle_skips_crossing_points() {
    // Make 31 a crossing point (testing::crossing): no finding.
}

#[test]
fn only_the_most_extreme_straight_rule_reports() {
    // Rules at 170 (hint) and 175 (warning), angle 178°: one finding, the warning.
}

#[test]
fn the_wkb_profile_hints_at_straight_angles() {
    // load_rule_set("wkb-2026", &[]) on angle_event(175.0): finding with rule id
    // "wkb2026.leg.straight_angle_hint".
}
```

  Note on `angle_event`: in the existing helper the finish runs straight on from 32, so for 175° only the angle
  at 31 is above 170; check that the angle at 32 is 180° and either reposition the finish or assert on the
  finding at 31 only.

- [ ] **Step 2: Run them and see them fail**

Run: `cargo test -p freemapper-courseplan check::legs`
Expected: FAIL (unknown key / missing check).

- [ ] **Step 3: Implement** `straight_angle`, register it, add the profile entry, texts, README rows and the
  glossary row.

- [ ] **Step 4: Run the whole workspace**

Run: `cargo test --workspace --locked`
Expected: PASS, including `every_known_key_has_a_check`, `texts_are_localized` and the used-id scanners. The
Lübeck check transcripts may gain a hint; Task 4 regenerates them, so a test that pins the example output is
updated there, not here. If such a test fails now, update the expected text in this task.

- [ ] **Step 5: Commit**

```bash
git add crates/freemapper-courseplan docs/glossary.md
git commit -m "feat(check): hint at almost straight angles at controls"
```

### Task 4: Decision records, manual, changelog, example

**Files:**
- Create: `docs/adr/0031-number-placement-behind-the-circle-and-off-map-symbols.md`; index row in
  `docs/adr/README.md`
- Modify: `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.3
- Modify: `docs/manual/de.md`, `docs/manual/en.md` (chapter 3 "Was du wissen solltest" bullet on numbers;
  chapter 8 where the rules are named)
- Modify: `CHANGELOG.md`, `CHANGELOG.en.md` (under "Unveröffentlicht"/"Unreleased", which exists)
- Modify: `examples/luebeck-sprint/` (project file, `transcript.*.txt`, `check.*.txt`, `README*.md`,
  `make-preview.py`, `preview.jpg`)

- [ ] **Step 1: ADR 0031** (English): context (manual moving of numbers; the user's rules 1–5 with date);
  decision (preferred direction, behind blocked as agreed with freemapper-print, 15° grid, ranking key,
  point/line symbols hard with `POINT_SYMBOL_RADIUS_MM` 0.5 and `LINE_SYMBOL_HALF_WIDTH_MM` 0.15 as Claude's
  call, areas free; `straight_angle` 170° hint at punched controls); consequences (placed directions change;
  freemapper-print implements the same rules in the shared module and cites this ADR). Mark *(user)*,
  *(Claude's call)*, *(agreed with freemapper-print)*.

- [ ] **Step 2: Design §5.3.** Replace "It avoids the course's circles, legs and numbers and weighs covered
  map detail *(Claude's call)*." with the rules 1–4 marked *(user)*, the blocked-behind definition marked
  *(agreed with freemapper-print)*, and a pointer to ADR 0031. In §5.6, add `straight_angle` beside the angle
  check, marked *(user)*.

- [ ] **Step 3: Manual**, both languages: one or two sentences in chapter 3 on where the program puts numbers
  (behind the circle, outside at almost straight angles, off point and line symbols). In chapter 8, name the
  straight-angle hint where the acute angle is mentioned, if the chapter lists rules.

- [ ] **Step 4: Changelog**, both languages: under "Geändert"/"Changed" the new placement; under
  "Hinzugefügt"/"Added" the `straight_angle` hint (170°, `wkb2026.leg.straight_angle_hint`).

- [ ] **Step 5: Example.** In `examples/luebeck-sprint/`, rerun the commands of `transcript.de.txt` and
  `transcript.en.txt` with the built binary against a copy, take the new number placements into
  `luebeck-sprint.courseplan.toml` (keep `[[accepted_findings]]`), rerun the commands of `check.de.txt` and
  `check.en.txt`, and update the outputs. In `make-preview.py`, place the text with the ADR 0030 gap (outer
  edge of the circle line, `r_outer = 3.175`, plus the clearance to the box) instead of `r + gap + 2.2k`.
  Regenerate `preview.jpg` with the steps in the example README ("Skizze neu erzeugen"); if
  `freemapper-print render` is not available, keep the old image and say so in the report. Update the README
  text of the example where it quotes outputs that changed.

- [ ] **Step 6: Check both top-level READMEs** (`README.md`, `README.en.md`) against the branch: status
  paragraph and feature list mention the placement rules and the new hint if they list checks or placement;
  keep both languages equal.

- [ ] **Step 7: Run the whole workspace**

Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add docs CHANGELOG.md CHANGELOG.en.md examples README.md README.en.md
git commit -m "docs: record number placement rules, straight-angle hint and new example output"
```
