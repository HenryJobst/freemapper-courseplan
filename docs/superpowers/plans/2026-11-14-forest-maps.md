# Forest maps (ISOM 2017-2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A person can open an ISOM 2017-2 forest map, set courses by hand, run `check` with the forest rules and export, as on a sprint map.

**Architecture:** The map kind detection learns to tell ISOM 2017-2 from other forest symbol sets and refuses the others. Rule data (`wkb-2026.toml`) gets forest symbol numbers that are confirmed against the ISOM 2017-2 specification, and the map-edge limits derive from the map scale. Score commands refuse forest maps. Everything else (editor, export, Purple Pen) is checked with a forest map and fixed where it assumed sprint.

**Tech Stack:** Rust 1.99.1, edition 2024; crates `freemapper-courseplan-model`, `freemapper-courseplan`, `freemapper-courseplan-cli`; freemapper-core v0.24.0 (`MapKind::Forest`, `freemapper-printrules`).

**Spec:** `docs/superpowers/specs/2026-11-13-forest-maps-design.md`

## Global Constraints

- Only ISOM 2017-2 is accepted as a forest symbol set; ISOM 2000 and unknown sets are refused with a message that names what was found and what is accepted *(user)*.
- Sprint and schoolyard behaviour stays identical: all existing tests and examples pass unchanged, example files do not change.
- A symbol number enters a rule only after it is confirmed against the ISOM 2017-2 specification or Mapper's ISOM 2017-2 symbol set; section and number go into `docs/research/`.
- No score draft, suggest or length on forest maps (refused with a message); no elevation, no route analysis.
- No forest picture or example event in the manual or `examples/` until a map with a confirmed licence exists *(user)*.
- Program texts in Fluent, German and English, in the same commit (`i18n/<language>/courseplan.ftl`); German says "Bundesland" for a state.
- User-facing text passes the `unslop` and `humanizer` skills (read both SKILL.md files).
- Dependencies at least two weeks old (ADR 0006); no new dependency planned.
- ADR for the decision (next free number 0092), changelog entries under Unveröffentlicht/Unreleased in `CHANGELOG.md` and `CHANGELOG.en.md`, glossary rows first, READMEs equal in German and English.
- Every task: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, tests green. Disk is tight: `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0`, `df -h /` before long runs, `cargo clean` at the end.

## Review Focus

- A sprint map at 1:4000 that is ISOM-based (ISOM at 1:4000 counts as sprint today) must not become a refused forest map.
- A forest map in ISOM 2000 (the existing `forest-sample.omap`) must give the refusal message, not a panic and not a silent sprint treatment.
- A forest map with a `freemapper.toml` sidecar that says `map_kind = "forest"` but whose symbol set is not ISOM 2017-2.
- A forest course on which a sprint-only rule (`dead_end`, `start_finish_distance`) would have fired: it must be reported as "does not apply", not as a finding and not as "not checked".
- `score suggest|draft|length` and the editor's Score panel on a forest map: a clear refusal, no raster built.
- Purple Pen export, import and sync of a forest event keep `Isom2017` and the scale.
- Limits for `map.inside` and `map.margin` at 1:15000 versus 1:4000: derived from the scale, sprint values unchanged.

---

### Task 1: Confirm the ISOM 2017-2 symbol numbers

**Files:**
- Create: `docs/research/2026-11-14-isom-2017-2-symbols.md`

**Interfaces:**
- Produces: a table `symbol number | meaning in ISOM 2017-2 | source (specification section or Mapper symbol set file) | used by (rule or table)` that later tasks read.

- [ ] **Step 1:** Get the ISOM 2017-2 specification (IOF, CC BY-ND, link only, do not copy) and Mapper's ISOM 2017-2 symbol set from `https://github.com/OpenOrienteering/mapper` (`symbol sets/`, tag v0.9.6 or newer). Read `crates/freemapper-courseplan/rules/wkb-2026.toml`, `docs/research/2026-11-08-bahnlegung-kriterien.md` and `crates/freemapper-courseplan/src/numbers.rs`.
- [ ] **Step 2:** For every symbol number a forest rule or table uses (similar-feature groups, impassable and out-of-bounds symbols 520/528/709/714, layout symbols 601/602, north lines, column D proposals, first aid 712/713, crossing/boundary symbols 505-508, 516/517, 304-309, 109-112) write one row with the confirmed meaning. Mark a number "unconfirmed" when the sources do not settle it, and say why.
- [ ] **Step 3:** List every number that the current rule data uses with a different meaning in ISOM 2017-2 than in ISOM 2000 (the existing `forest-sample.omap` uses 2000 numbers).
- [ ] **Step 4:** Commit `docs(research): ISOM 2017-2 symbol numbers confirmed for forest rules`.

### Task 2: Synthetic ISOM 2017-2 test map

**Files:**
- Create: `tests/fixtures/maps/isom2017-synthetic.omap`
- Modify: `tests/fixtures/README.md` (source, date, licence: made for the project from Mapper's ISOM 2017-2 symbol set, GPL-3 note as Mapper states it)
- Test: `crates/freemapper-courseplan/tests/read_maps.rs`

**Interfaces:**
- Consumes: Task 1 table.
- Produces: a small `.omap` (about 300 x 300 m at 1:10000) with the symbol set id `ISOM 2017-2`, scale 1:10000, objects for: a path, a fence, a stone wall, a cliff, water, an out-of-bounds area, a prohibited area (528 or the 2017-2 equivalent), two similar boulders, a marsh, open land. Each rule that Task 4 changes has at least one object it can fire on and one it must not.

- [ ] **Step 1: Write the failing test** `isom2017_synthetic_reads_as_forest` in `read_maps.rs`: `map_kind_for_file(path)` returns `MapKind::Forest` and the scale is 10000.
- [ ] **Step 2:** Run it; expected FAIL (file missing).
- [ ] **Step 3:** Build the map: start from Mapper's ISOM 2017-2 symbol set file, add the objects above by writing the OOM XML (`.omap`) by hand or with a small script kept in `tests/fixtures/maps/` as `README` text only (no generator is committed); open it once with the program's own reader to check it parses.
- [ ] **Step 4:** Run the test; expected PASS. If the conversion of `forest-sample.omap` with Mapper's `ISOM2000-ISOM 2017-2.crt` was tried, note the result in the fixtures README; use it only if checked by eye.
- [ ] **Step 5:** Commit `test(fixtures): synthetic ISOM 2017-2 forest map`.

### Task 3: Detection and refusal

**Files:**
- Modify: `crates/freemapper-courseplan/src/map.rs` (`map_kind`, `kind_from_name`, `map_kind_for_file`), `crates/freemapper-courseplan/src/init.rs` (replace `ForestNotSupported`), `crates/freemapper-courseplan/src/text.rs`, `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl`
- Test: `crates/freemapper-courseplan/tests/init.rs` (or the existing init test file; find it with `grep -rn ForestNotSupported crates`)

**Interfaces:**
- Produces: `enum ForestSymbolSet { Isom2017_2, Isom2000, Unknown(String) }` returned by a new `pub fn forest_symbol_set(map: &Map) -> ForestSymbolSet`; `InitError::ForestSymbolSetNotSupported { found: String }` replaces `ForestNotSupported` (its `Display` stays English).
- Existing `map_kind(...)` signature stays; sprint detection logic is untouched.

- [ ] **Step 1: Write failing tests:** `init_accepts_isom_2017_2_forest_map` (the synthetic map initialises, project file records kind forest), `init_refuses_isom_2000_forest_map` (`forest-sample.omap`, error names `ISOM 2000` and says `ISOM 2017-2` is accepted), `init_refuses_forest_sidecar_with_unknown_symbol_set`, `isom_sprint_at_1_4000_is_still_sprint` (a 1:4000 ISOM-named map stays sprint; reuse a sprint fixture), `luebeck_and_gleisdreieck_still_init` (existing behaviour).
- [ ] **Step 2:** Run them; expected FAIL.
- [ ] **Step 3:** Implement `forest_symbol_set` (reads the symbol set id/name of the `.omap`; Task 1 settled which names Mapper and freemapper write) and use it in `init.rs`; the refusal applies only after the map is detected as forest. Add Fluent messages in both languages.
- [ ] **Step 4:** Run the tests and the whole `freemapper-courseplan` suite; expected PASS, sprint tests unchanged.
- [ ] **Step 5:** Commit `feat(init): accept ISOM 2017-2 forest maps, refuse other forest symbol sets`.

### Task 4: Forest rule data and map-edge limits

**Files:**
- Modify: `crates/freemapper-courseplan/rules/wkb-2026.toml` (forest symbol numbers confirmed in Task 1; `ignore_symbols` for layout symbols; edge limits per map kind), `crates/freemapper-courseplan/src/check/forbidden.rs`, `crates/freemapper-courseplan/src/check/inside.rs`, the rule loader (find with `grep -rn wkb-2026 crates`), `crates/freemapper-courseplan/src/check/` skip reporting (the "not checked" list)
- Test: `crates/freemapper-courseplan/tests/` (check tests; add `check_forest.rs`)

**Interfaces:**
- Consumes: Task 1 table, Task 2 map, Task 3 detection.
- Produces: rule data where `map.inside` and `map.margin` limits are a function of map kind and scale (sprint values 50 m / 50 m unchanged); a result category "does not apply" for sprint-only rules on a forest map, next to the existing "not checked".

- [ ] **Step 1: Write failing tests** on the synthetic map: `forest_min_distance_30m_fires`, `forest_similar_features_60m_uses_isom_2017_groups`, `forest_control_in_out_of_bounds_fires`, `forest_control_outside_mapped_area_fires`, `forest_sprint_only_rules_do_not_apply` (`dead_end` and `start_finish_distance` listed as not applying, not as findings), `forest_elevation_rules_are_not_checked` (reason named), `sprint_edge_limits_unchanged`, `forest_edge_limit_derives_from_scale` (1:15000 vs 1:10000 give the stated metres).
- [ ] **Step 2:** Run them; expected FAIL.
- [ ] **Step 3:** Edit the rule data with confirmed numbers only; an unconfirmed number leaves its rule out and the rule appears as "not checked" with the reason. Implement the limit function and the "does not apply" category; German and English texts for both.
- [ ] **Step 4:** Run all check tests plus the examples' `check` snapshots; expected PASS, sprint outputs byte-identical.
- [ ] **Step 5:** Commit `feat(check): forest rule data for ISOM 2017-2, map-edge limits from the scale`.

### Task 5: Score commands refuse forest maps

**Files:**
- Modify: `crates/freemapper-courseplan-cli/src/mcp/score.rs` (existing forest guard at about line 198), the CLI score commands, `crates/freemapper-courseplan-cli/src/editor/` (Score panel), Fluent files
- Test: `crates/freemapper-courseplan-cli/tests/` (CLI and MCP)

**Interfaces:**
- Consumes: the project's map kind from Task 3.
- Produces: one shared guard `fn ensure_not_forest(kind: MapKind) -> Result<(), String>` used by `score suggest`, `score draft`, `score length`, the MCP tools `score_suggest`, `score_draft`, `score_length`, and the editor's three Score actions.

- [ ] **Step 1: Write failing tests:** `score_suggest_refuses_forest_map` (exit 1, one-line `error:` message in English and German), `score_draft_refuses_forest_map_without_building_raster` (fast, no raster), the MCP variants, and `score_course_by_hand_works_on_forest_map` (creating and checking a score course by hand still works).
- [ ] **Step 2:** Run them; expected FAIL.
- [ ] **Step 3:** Add the guard, call it before any raster work, hide or disable the editor actions on a forest map with the message.
- [ ] **Step 4:** Run CLI and MCP suites; expected PASS.
- [ ] **Step 5:** Commit `feat(score): refuse suggest, draft and length on forest maps`.

### Task 6: Forest through the whole chain

**Files:**
- Modify: whatever the checks below show (editor `web/js/*.js`, `ppen_export.rs`, `iof_xml.rs`, `numbers.rs`)
- Test: `crates/freemapper-courseplan-cli/tests/forest_chain.rs`

**Interfaces:**
- Consumes: Tasks 2 to 5.
- Produces: an end-to-end test on the synthetic map plus a short list of renderer gaps (symbols the editor does not draw) in the plan's report.

- [ ] **Step 1: Write the test** `forest_course_by_hand_end_to_end`: `init`, `special add` start and finish, `control add` x3, `course add`, `course numbers`, `check` (no error on a clean course), `export --iof-xml` (validates against `tests/fixtures/iof-xml/IOF.xsd`), `export --ppen` (file says `Isom2017`, scale 10000), `import` and `sync --dry-run` round trip, and the editor's `/api/state` answers for the project.
- [ ] **Step 2:** Run it; expected FAIL where something assumed sprint (print values come from `freemapper-printrules` for forest: circle 5.0 mm, line 0.35 mm, number 4.0 mm at the forest scales; the duplicated table in `numbers.rs` must agree).
- [ ] **Step 3:** Fix each failure; where `numbers.rs` duplicates printrules values, take them from `freemapper-printrules` if the API allows, else keep and test equality.
- [ ] **Step 4:** Open the editor with the synthetic map with Playwright, screenshot to the scratchpad (not committed), and list symbols the core renderer does not draw. A needed core change is written up for freemapper-core, not done here.
- [ ] **Step 5:** Run the full workspace tests; commit `test(forest): course by hand end to end`.

### Task 7: ADR, documentation, texts

**Files:**
- Create: `docs/adr/0092-forest-maps-isom-2017-2.md`
- Modify: `docs/adr/README.md` (index), `docs/adr/0010-sprint-and-schoolyard-maps-first.md` (status line: "superseded in part by 0092"), `docs/glossary.md`, `docs/manual/de.md`, `docs/manual/en.md`, `AGENTS.md` (limits section, the forest refusal sentence at about lines 23 and 376, tool refusals, rule "does not apply"), `README.md`, `README.en.md`, `CHANGELOG.md`, `CHANGELOG.en.md`, `docs/research/2026-11-08-bahnlegung-kriterien.md` (pointer to the new research file), `tests/fixtures/README.md`

**Interfaces:** none produced for other tasks.

- [ ] **Step 1:** Write ADR 0092 with *(user)* and *(Claude's call)* marks as in the spec: scope, ISOM 2017-2 only, refusal, rule data and sources, edge limits from scale, score refusal, test map and the licence stance, "cost if wrong".
- [ ] **Step 2:** Update the glossary (new terms: forest symbol set, "does not apply"), manual chapters (de and en, no forest pictures), AGENTS.md, both READMEs (feature text and ADR range, equal content), both changelogs.
- [ ] **Step 3:** Apply `unslop` and `humanizer` to every text written.
- [ ] **Step 4:** Run the whole suite, fmt and clippy; commit `docs: ADR 0092, manual, AGENTS.md and READMEs for forest maps`.
