# M2b: `check` with rule profiles – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for
> tracking.

**Goal:** `freemapper-courseplan check` (CLI and MCP) checks the individual courses of an event against a rule
profile (WKB 2026) plus optional regional overlays (NOR Berlin/Brandenburg), reports each finding with location,
rule id, source and a localized explanation, and lets the user accept a deliberate exception with a note.

**Architecture:** Rule profiles are TOML files embedded in the library (`crates/freemapper-courseplan/rules/`),
loaded into a `RuleSet` (base plus overlays, an overlay replaces or adds rules by id). Each rule names a `check`
kind implemented once in Rust (`check/` module, one file per family); the values and sources come from the data.
Findings are plain data, localized through the Fluent catalog. Only class-A rules that need no routing
(design §5.6, research §3) are in scope.

**Tech Stack:** Rust 1.99.1, serde/toml (already present), the existing `map::MapIndex`, `course::course_legs`,
`text` catalog. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.6 (checks), §4.2a (discipline,
classes, starters), §1 (audience: profiles are data anyone can add). Rule sources:
`docs/research/2026-11-08-bahnlegung-kriterien.md` (§3.1–3.8, §3.3a NOR). Follow-ups from M2a: end of
`docs/superpowers/plans/2026-11-08-m2a-editing-length-numbers-mcp.md`.

## Global Constraints

- Toolchain 1.99.1; `unsafe_code = "forbid"`; fmt, `clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked` pass after every task. No new dependencies; Cargo.lock unchanged.
- Every rule value in a profile file carries `source = { doc = "<kürzel>", section = "<abschnitt>" }` using the
  document keys of the research file (WKB26, IOF-R, BOF, SOA, BL02, BL12, TD11, SOFT, BB-Lkm21, BB-Kombi14).
  Values marked *(Claude's call)* in this plan carry `source = { doc = "claude", section = "M2b plan" }` and a
  `note`.
- Rule ids are stable, dotted, lower-case: `<profile>.<family>.<name>`, e.g. `wkb2026.distance.min`.
- Findings never quote file content in MCP results beyond codes, ids and names of the event (ADR 0023).
- All user-facing texts (finding explanations, CLI and MCP texts) in German and English via the Fluent catalogs;
  terminology per the glossary (ruling R13 of M2a).
- Distances on the ground: `metres = map_mm × map_scale / 1000`. Overprint sizes per `numbers::overprint_sizes`
  and `base_scale` (ADR 0024).
- Project file format stays 1 (provisional, ADR 0019); new fields are documented in an ADR.
- Only individual courses are checked; other kinds give one "not checked yet" info finding per course.
- Branch `m2b-check`, worktree `.worktrees/m2b-check`. Trailers as in CLAUDE.md.

## Review Focus

1. An accepted finding stays silent after unrelated edits, but returns when its controls change
   (Task 3, `accepted_finding_matches_by_rule_and_ids`).
2. A rule that does not apply (wrong map kind, unmapped class, missing climb, training discipline) yields one
   "skipped, because …" info, never a wrong error (Task 3 skipped-rule reporting; Tasks 4–8 tests).
3. Overlay precedence: an overlay value replaces the base value by rule id, and `check` names the overlay's
   source (Task 2, `overlay_replaces_base_rule`).
4. A legless or one-control course, a butterfly, and crossing points do not crash or double-report a check
   (Tasks 4–5 edge tests).
5. Map objects tagged as sheet content (`freemapper:source=sheet`) never trigger map-based findings (Task 4).

---

### Task 1: Format additions: overlays and accepted-finding keys

**Files:** model `types.rs`, `validate.rs`, `project_file.rs` (default-omitted keys), fixtures; `docs/adr/0025-rule-profiles-as-data.md`
(also covers Task 2's file format), `docs/adr/README.md`, `docs/glossary.md`.

**Interfaces:**
- `EventMeta` gets `rule_overlays: Vec<String>` (omitted when empty).
- `AcceptedFinding` keeps `rule`, `controls`, `course`, `note`; add `legs: Vec<LegRef>` (omitted when empty),
  `LegRef { course: String, after: u32 }` (1-based like the CLI; 0 = start leg).
- Validation: a blank note is `ModelError::BlankName { what: "note", … }` (reuse the variant).
- ADR 0025: rule profiles are data files (format below), base plus overlays, rule ids stable, accepted findings
  keyed by rule id plus the sorted set of control codes and leg refs; a finding whose key changes is reported
  again. State that freemapper-print may reuse the profile format for print rules.

- [ ] Tests: round trip of `rule_overlays` and `legs`; blank note refused; `unchanged_save_keeps_bytes` stays.
- [ ] Implement, run, commit `feat(model): rule overlays and leg keys for accepted findings`.

### Task 2: Rule profile files and the rule set

**Files:** create `crates/freemapper-courseplan/rules/wkb-2026.toml`, `rules/nor-bb-2021.toml`,
`rules/README.md` (how to add a profile), `src/rules.rs`; modify `src/lib.rs`.

**Profile file format** (embedded with `include_str!`; listed in `rules::available()`):

```toml
[profile]
id = "wkb-2026"                # file stem
kind = "base"                  # "base" | "overlay"
base = "wkb-2026"              # overlays only: the base they extend
name = { de = "Wettkampfbestimmungen OL 2026", en = "German competition rules 2026" }
valid_from = 2026-03-01

[sources.WKB26]
title = "Wettkampfbestimmungen Orientierungslauf, 37. Änderung"
url = "https://o-sport.de/assets/dokumente/allgemein/wettkampfwesen/wkb2026.pdf"

[[rules]]
id = "wkb2026.distance.min"
check = "min_distance"         # implementation key, see the table below
severity = "error"             # "error" | "warning" | "hint"
map_kinds = ["sprint", "schoolyard"]   # optional filter; empty = all
values = { metres = 15.0 }
source = { doc = "WKB26", section = "B 7.2.1" }
note = { de = "…", en = "…" }  # optional, shown with the finding
```

`values` is a free table; each `check` kind documents the keys it reads. Tables (Anlage 5 percentages, category
→ difficulty level, NOR course combinations) are `values` with nested tables or arrays.

**Interfaces:**
- `struct RuleProfile { id, kind, base: Option<String>, name: LocalizedText, sources: BTreeMap<String, Source>, rules: Vec<Rule> }`;
  `struct Rule { id, check: String, severity: Severity, map_kinds: Vec<MapKind>, disciplines: Vec<Discipline>, values: toml::Table, source: SourceRef, note: Option<LocalizedText> }`.
- `fn load_rule_set(base: &str, overlays: &[String]) -> Result<RuleSet, RulesError>`: base must be `kind = "base"`;
  each overlay must name that base; an overlay rule with an existing id replaces it, a new id is added; the
  result keeps for each rule the profile it came from.
- `RulesError` (localized): unknown profile, overlay for another base, malformed file (only in tests: embedded
  files are checked at build time by a test), unknown `check` kind.
- Data, with exact values from the documents (implementer reads them; the WKB 2026 text is in the session
  scratchpad `…/scratchpad/dl/wkb2026.txt` if still present, else download from the URL in the research file):
  - `wkb-2026.toml`: every rule of Tasks 4–8 with the values given there, the Anlage 5 length percentages for
    Lang, Mittel, Ultralang (all rows), the category → difficulty level map (Anlage 5 Nr. 1–2), the similar-feature
    groups (Task 4).
  - `nor-bb-2021.toml` (overlay, base `wkb-2026`): Lkm formula (km + climb_m/100), tolerance ±6 %
    (BB-Kombi14), the three course-combination tables of BB-Lkm21 (Lang, Mittel, Ultralang: course letter, Lkm,
    O-value, categories) exactly as in research §3.3a, with a `note` that course D (D/H-14, 2.6 Lkm) looks like a
    transcription error in the source.
- [ ] Tests: `every_embedded_profile_parses`, `every_rule_has_a_known_check_kind_and_a_source_in_its_file`,
  `overlay_replaces_base_rule`, `overlay_for_another_base_is_refused`, `unknown_profile_is_refused`.
- [ ] Implement, run, commit `feat(rules): rule profiles as data, base plus overlays`.

### Task 3: Findings, the check engine, `check` in CLI and MCP, accepting findings

**Files:** create `src/check/mod.rs` (engine, `Finding`, accepted matching), `src/check/context.rs` (shared data:
map index, course legs, numbers, scale); modify CLI (`main.rs`, `edit_cmd.rs` or a new `check_cmd.rs`), `mcp.rs`,
catalogs.

**Interfaces:**
- `enum Location { Event, Course(String), Control { code }, Controls { codes: Vec<ControlCode> }, Leg { course, after: u32 } }`.
- `struct Finding { rule: String, profile: String, severity: Severity, location: Location, message_id: &'static str, args: Vec<(&'static str, ArgValue)>, source: SourceRef }`,
  `enum ArgValue { Text(String), Number(f64), Int(i64) }`.
- `fn check_event(event: &Event, map: &MapDocument, rules: &RuleSet) -> CheckReport`, `struct CheckReport { findings: Vec<Finding>, accepted: Vec<(Finding, String /* note */)>, skipped: Vec<Skipped> }`,
  `struct Skipped { rule: String, reason_id: &'static str, args }` (map kind filter, discipline training, class
  not mapped, climb missing, course kind not individual).
- Each check kind is a function `fn(&CheckContext, &Rule, &mut Vec<Finding>, &mut Vec<Skipped>)` registered in
  one table by its `check` key; unknown keys are rejected in Task 2.
- Accepted findings: a finding is accepted when an `AcceptedFinding` has the same rule id and the same sorted
  control codes and leg refs (course must match when set). Accepted ones move to `accepted`.
- CLI `check <PROJECT> [--all]`: prints findings grouped by course, each line `<severity> <location>: <text>
  (<rule id>, <doc> <section>)`, then a summary line; `--all` also prints accepted and skipped. Exit code 0 when
  no `error` finding, 1 when there is one (document it; a refused command is still 1, usage 2).
- CLI `finding accept <PROJECT> <RULE> [--controls 31,32] [--leg A:3] [--course A] --note TEXT` and
  `finding remove <PROJECT> <RULE> [same keys]`.
- MCP tools `check` (returns findings/accepted/skipped as JSON with localized texts and machine fields) and
  `accept_finding`, `remove_accepted_finding`.
- `event_info` (MCP) and `info` (CLI) gain the active profile and overlays.
- A `project set` sub-command or flags are not added; profile and overlays are set with
  `course`-independent `event profile <PROJECT> <BASE> [--overlay ID …]` and MCP `set_rule_profile`.
- [ ] Tests: engine with a fake rule kind; `accepted_finding_matches_by_rule_and_ids`;
  `accepted_finding_returns_when_its_controls_change`; skipped reporting; CLI exit codes; MCP JSON shape and
  German texts.
- [ ] Implement, run, commit `feat(check): findings, accepted exceptions, check in CLI and MCP`.

### Task 4: Map-based distance checks

**Files:** `src/check/distance.rs` (+ tests), profile data, catalogs.

**Check kinds and values (WKB 2026 profile):**
- `min_distance`: centres of any two controls of the event closer than `metres` → error. Values: 30 m; 15 m for
  map kinds sprint and schoolyard (two rules with `map_kinds` filters). Crossing points count. Start counts as a
  control (IOF-R 19.4 "Startflagge eingeschlossen"; for WKB the same reading, *(Claude's call)*).
- `similar_features`: two controls whose control features belong to the same similarity group closer than
  `metres` → warning. 60 m forest/default, 30 m sprint and schoolyard (WKB26 B 7.2.1). The control feature is the
  map feature nearest to the control centre within 1.0 map mm; no feature → no finding for that control. Groups
  are data: `values.groups = [["101","102"], …]` as ISSprOM/ISOM symbol codes, from BOF B6.1.3 examples
  (stream/ditch/linear marsh, depression/pit/shallow depression, knoll/spur, re-entrant/spur side, fence/ruined
  fence) mapped to ISSprOM and ISOM codes *(implementer lists the code mapping in the profile note; codes are
  checked against the symbol set in `tests/fixtures/maps/luebeck.omap` and the ISSprOM 2019-2 list)*; identical
  symbol codes are always similar.
- `map_edge`: control or start/finish closer than `metres` to the map extent boundary → warning; 10 m
  *(Claude's call; BL02 names the problem without a value)*.
- `leg_through_control`: a leg polyline passes within one circle radius (overprint size, map mm) of a control,
  start or finish of the same course other than its two ends → warning (BL02).
- `leg_through_forbidden`: a leg polyline intersects an out-of-bounds area special assigned to the course (or to
  all courses), a forbidden route special, or a map feature whose symbol code is in `values.impassable`
  (IOF-R 17.2 list for ISSprOM: 201, 301, 307, 411, 512.1, 515, 518, 520, 521, 529, 708, 709, 714; for ISOM: 520,
  708, 709, 711 crossing excepted) → error (WKB26 B 7.2.6, IOF-R 17.2). Taped legs are exempt (a marked route
  may lead through, the course setter decided).
- Sheet-tagged map objects are excluded by `MapIndex` already; add a test that proves it for these checks.
- [ ] Tests per kind (positive, negative, edge: one-control course, butterfly, crossing point, taped leg).
- [ ] Commit `feat(check): distances, similar features, map edge, legs through controls and forbidden areas`.

### Task 5: Leg shape checks

**Files:** `src/check/legs.rs`.

- `acute_angle`: angle between incoming and outgoing leg at a control (using the last bend before and the first
  bend after) below `degrees` → warning; 45° *(Claude's call; BL12 names < 90° as a risk, SOA dog-legs)*;
  a second hint rule at 90° (BL12).
- `opposing_legs`: two legs of the same course whose segments run within `metres` of each other over at least
  `min_length_m` with directions differing by more than 135° → warning (WKB26 B 7.2.6 "längere gegenläufige
  Teilstrecken"); values 10 m and 30 m *(Claude's call)*; the two legs meeting at a control are exempt for the
  first `metres` around it.
- [ ] Tests incl. butterfly loops (allowed by design: the shared control is a meeting point; opposing segments
  far from it still count).
- [ ] Commit `feat(check): acute angles and opposing legs`.

### Task 6: Code checks

**Files:** `src/check/codes.rs`.

- `code_range`: numeric code outside `min`..=`max` (31..=599 per WKB26 B 7.2.10; IOF ≥ 31) or non-numeric →
  error for numeric range, warning for non-numeric.
- `code_pairs`: both codes of a forbidden pair used in the event (`values.pairs = [[66,99],[68,89],[86,98]]`) →
  error (WKB26 B 7.2.10).
- `code_confusable_next`: two consecutive controls of a course whose codes read the same rotated by 180° or
  differ only by 6↔9 (IOF-SG, SOA) → warning.
- `same_objects_in_circle`: the control feature's symbol occurs again within the circle (overprint circle radius
  in map mm) and the description has no column C → warning (TD11).
- [ ] Tests. Commit `feat(check): control codes and same objects in the circle`.

### Task 7: Start checks

**Files:** `src/check/start.rs`.

- `start_capacity`: courses that start in the same minute need different first controls (user criterion,
  SOFT). With `start_window_min` and `expected_starters` per class: starters per minute needed
  `= ceil(Σ starters of all classes on courses / window)`; available = number of distinct first controls among
  courses that have starters. Needed > available → warning naming the shared first controls. Missing window or
  starters → skipped with reason.
- `first_leg_angle`: two different first controls whose first legs from the same start differ by less than
  `degrees` (30°, *(Claude's call)*, SOFT "nicht … gleicher erster Posten" extended to direction) → hint.
- [ ] Tests. Commit `feat(check): start capacity and first-leg directions`.

### Task 8: Table checks and the NOR overlay

**Files:** `src/check/tables.rs`.

- `length_table` (WKB26 Anlage 5): for disciplines long/middle/ultralong, each course whose classes map to rule
  categories: its length as a percentage of the HE course length must lie within the category's range. The HE
  reference is the course with category HE (or H21E as in the table) in the event; none → skipped with reason.
  Outside → warning with expected range and actual percentage.
- `difficulty_mix` (Anlage 5 Nr. 2): a course carries classes of level 1–2 and of level 3–5 → error.
- `nor_lkm` (overlay nor-bb-2021): Lkm = length_km + climb_m/100; the course's categories are looked up in the
  combination table of the discipline; Lkm outside target ±6 % → warning; course without climb → skipped; a
  category in no row → hint. Sprint has no NOR table → skipped.
- [ ] Tests with small tables. Commit `feat(check): length tables, difficulty levels and the NOR Lkm overlay`.

### Task 9: Docs and example

- Manual chapter 8 ("Prüfen: Hinweise lesen, Ausnahmen begründen" / "Checking") in DE and EN; changelog; README
  status (check exists; profiles; overlays); `rules/README.md` explains adding a profile; the Lübeck example gets
  `check.de.txt`/`check.en.txt` with the output of `check` and one accepted finding; the example README shows a
  short excerpt.
- [ ] Commit `docs: checking, rule profiles and the example's check output`.

---

## After the last task

Final whole-branch review (most capable model) including an end-to-end German run on the Lübeck example, one fix
wave, merge, checks on main, README check, push, CI, release v0.2.0 (new user-visible milestone), clean up.
Send the profile format (ADR 0025) to freemapper-print before Task 2 is merged (memory: sibling handoffs).
