# Forest maps (ISOM 2017-2): course setting by hand and `check`

Date: 2026-11-13
Status: Draft, waiting for approval by the user (builder check done)

Marks: *(user)* is a decision the user made, *(Claude's call)* is a choice I made and the user can change.

## Intent

ADR 0010 limited the first versions to sprint and schoolyard maps and said "forest maps later" *(user, 2026-11-08)*. The user now wants
course setting on forest maps *(user, 2026-11-13)*. Today `init` refuses an ISOM map with `ForestNotSupported`. After this work a person can
open a forest map, place start, finish and controls by hand, build courses, let `check` test them against the forest rules and
export them, as on a sprint map.

## Goals

- `init` accepts forest maps in ISOM 2017-2 at the scales 1:5000, 1:7500, 1:10000 and 1:15000 *(user: "Nur ISOM 2017-2")*.
- Everything that works by hand on a sprint map works on a forest map: controls, descriptions, individual courses, legs and bends,
  control numbers, special objects, classes, IOF XML export, Purple Pen export, import and sync, editor, CLI, MCP.
- `check` applies the forest rules *(user: "Vorhandene + Wald-Basis")*: the rules that already carry a forest variant
  (minimum distance 30 m, similar features 60 m with the ISOM groups, out-of-bounds areas 520/709, angles, codes, number spacing) and
  the base rules that make sense on any map: a control inside an out-of-bounds area (ADR 0081) and a start, finish or control
  outside the mapped area (ADR 0084, 0087, with limits suited to the scale; see below).
- Rules that need elevation (climb, running kilometres) or distance-specific knowledge (long legs, changes of direction on middle and
  long distance) are not checked and appear among the rules that were not checked, with the reason.
- A map that is not ISOM 2017-2 is refused with a message that names what was found (ISOM 2000, unknown symbol set) and what is accepted.

## Non-goals

- No score draft on forest maps (`score suggest`, `score draft`, `score length`). The route search reads a raster of map symbols
  without elevation and without vegetation speed; its times would mislead in the forest *(user: first stage manual and `check`)*.
  The score commands refuse a forest map with a message that says so.
- No ISOM 2000 and no mixed symbol sets *(user)*.
- No elevation model, no climb, no route analysis.
- No new example event or manual pictures from a forest map until the licence of the test map is settled (see Test map).

## Anti-goals

- A forest `check` that gives wrong findings because it reads sprint numbers: worse than a refusal. A symbol number whose meaning in
  ISOM 2017-2 is not confirmed against the specification must not feed a rule.
- Weakening the sprint behaviour. Sprint and schoolyard results stay byte-identical (all existing tests and examples pass unchanged).

## Constraints

- ADR 0006 (dependencies two weeks old), 0007 (pinned toolchain), the project's rules in `CLAUDE.md`. Shared code goes to freemapper-core.
- freemapper-core v0.24.0 already knows `MapKind::Forest`, the forest print values and `MapStandard::Isom2017`. It has no embedded ISOM symbol
  set; the symbol groups live in courseplan's rule data.
- Program texts go through the Fluent files in German and English.

## Decisions

1. **Detection.** The map kind comes from the map as today (`map_kind_for_file`: `freemapper.toml` sidecar first, then symbol set name and scale).
   *(Claude's call)* ISOM at 1:4000 stays sprint-like as today; a person can force the kind with `map_kind = "forest"` in `freemapper.toml`.
   The symbol set must identify itself as ISOM 2017-2; a map with `ISOM2000` or without a recognisable symbol set is refused with the message above.
2. **Symbol numbers.** Every symbol number that a forest rule or table uses (similar-feature groups, impassable symbols, out-of-bounds
   symbols, column D proposals, layout symbols 601/602, north lines) is checked against the ISOM 2017-2 specification before it is used,
   and the source (section and number) goes into the rule data and the research file. Numbers that cannot be confirmed are left out and
   the rule that needs them is listed as not checked. The existing `forest-sample.omap` is ISOM 2000 and cannot confirm 2017-2 numbers.
3. **Limits that were tuned for sprint.** The 40/50 m limits of `map.inside` and `map.margin` and the frame heuristic were chosen for
   1:4000 sheets. *(Claude's call)* On forest maps they scale with the map scale: the rule data holds the limit in metres per map kind,
   starting from the sprint value for sprint and a value to be set in the plan for forest after measuring the test map. The sprint
   values do not change.
4. **Sprint-only rules.** `control.dead_end`, `course.start_finish_distance` and the `_sprint` variants stay sprint and schoolyard only.
   They are listed as not applying, not as not checked, on a forest map.
5. **Score on forest maps.** `score suggest`, `score draft`, `score length` and the editor's Score panel are refused on a forest map with a
   clear message; creating a score course by hand and `check` on it keep working *(Claude's call)*.
6. **ADR.** A new ADR replaces the "forest maps later" decision of ADR 0010 and records the points above.

## Test map

*(user)* The licence of the best candidate is unclear: OpenOrienteering Mapper's `forest sample.omap` (GPL-3 for the repository, no own statement for the
example files; ISOM 2000; already in `tests/fixtures/maps/forest-sample.omap`). The user chose: use it for tests only, and take pictures and an example
event only after a confirmed release (an issue with the Mapper project, or an association or Remo Madella).
- *(Claude's call)* For the rule tests the plan builds a small synthetic ISOM 2017-2 map (a few symbols: paths, fence, stone wall,
  out-of-bounds area, cliff, water) so each rule has a map it can really read. A conversion of the sample with Mapper's
  `ISOM2000-ISOM 2017-2.crt` is tried, and used only if its result is checked by eye.
- No forest example event and no manual picture from a forest map ship until a map with a confirmed licence exists.

## Rulings from the builder check *(all Claude's call)*

- **Source for symbol numbers.** The authority is the ISOM 2017-2 specification of the IOF (the specification is CC BY-ND, so it is read,
  not copied into the repository) and the ISOM 2017-2 symbol set that OpenOrienteering Mapper ships (`symbol sets/`). The builder fetches both
  and records section and number in the research file. A partial result is acceptable: a rule with confirmed numbers ships, a rule with an
  unconfirmed number is listed as not checked and does not block the merge.
- **Synthetic test map.** It is built from Mapper's ISOM 2017-2 symbol set (the map file carries the symbol definitions; the fixtures README
  names the source, date and licence). It is made before the rule work, because the rule tests need it.
- **Base rules.** Every rule that does not depend on the map kind (angles, codes, descriptions, number spacing, minimum distance and so on)
  applies to forest maps unchanged. A rule that applies only to sprint and schoolyard maps is reported as "does not apply" on a forest map;
  a rule that needs data courseplan does not have (elevation, route analysis) is reported as "not checked", with the reason.
- **Profile and overlay.** `wkb-2026` is the base profile on every map kind, with rule data scoped by map kind as today. The overlay
  `nor-bb-2021` stays selectable; where one of its rules has no forest meaning the rule data says so and the rule is reported as not applying.
- **Refusal scope.** The new refusal (ISOM 2000, unknown symbol set) applies only to maps that are detected as forest maps. How a map is
  detected as sprint or schoolyard does not change, so sprint and schoolyard results stay identical.
- **Editor.** The editor draws the map with freemapper-core. Symbols the renderer does not draw yet are listed in the report of the work; a
  change in freemapper-core is a request to that repository (own tag, then a pin step here) and does not block the first stage.
- **Limits for `map.inside` and `map.margin` on forest maps.** Derived from the map scale by one rule in the rule data (metres on the
  ground per map kind), checked on the synthetic map and on the ISOM 2000 sample for plausibility; no tuning to one map.

## Open questions for the plan (the builder can settle them)

- Which ISOM 2017-2 symbol sets does `kind_from_name` have to recognise (names as written by Mapper and by freemapper)?
- The exact forest limit for `map.inside` and `map.margin` (a value in metres per scale).
- Whether the print values duplicated in `numbers.rs` should now come from `freemapper-printrules` (they match today).

## Out of scope for this design

Forest score drafts, elevation, climb, route analysis, relay and team courses.
