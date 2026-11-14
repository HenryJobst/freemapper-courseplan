# 0092: Forest maps in ISOM 2017-2, by hand and with `check`

Status: Accepted, 2026-11-14. Replaces [0010](0010-sprint-and-schoolyard-maps-first.md) where it says that forest maps
come later, and takes up [0088](0088-mapped-area-limit-is-50-m.md) for forest maps. Design:
`docs/superpowers/specs/2026-11-13-forest-maps-design.md`.

## Context

ADR 0010 limited the first versions to sprint and schoolyard maps, and `init` refused every forest map. The user now
wants course setting on forest maps *(user, 2026-11-13)*: controls, courses, `check` and the exports, as on a sprint
map. The rule data already carried ISOM 2017-2 numbers for the rules that had a forest variant. What was missing was
a map to test them on, a way to tell ISOM 2017-2 from other forest symbol sets, limits for the edge of the mapped area
that make sense at 1:10000, and a refusal of everything that needs a route search.

## Decision

Marks: *(user)* is the user's decision, *(Claude's call)* a choice the user can change.

1. **Scope.** `init` accepts forest maps in ISOM 2017-2 at 1:5000 to 1:15000, and everything that works by hand on a
   sprint map works on them: controls, descriptions, individual courses, legs, numbers, special objects, classes,
   `check`, IOF XML, Purple Pen export, import and sync, editor, CLI and MCP *(user)*.
2. **Only ISOM 2017-2.** A map detected as a forest map whose symbol set is not `ISOM 2017-2` is refused, with the
   name that was found and the name that is accepted *(user)*. ISOM 2000 and unknown sets are covered by the same
   refusal. The test map `forest-sample.omap` is ISOM 2000 and now gives that refusal.
3. **No route search on forest maps.** `score suggest`, `score draft`, `score length`, their MCP tools and the editor's
   Score panel are refused, because the route search reads a raster of map symbols without elevation or vegetation
   speed, and its times would mislead *(user: first stage by hand and `check`)*. A score course set by hand, with its
   controls and links, works.
4. **Symbol numbers are confirmed before a rule reads them.** Every number the forest rules use is listed in
   [`docs/research/2026-11-14-isom-2017-2-symbols.md`](../research/2026-11-14-isom-2017-2-symbols.md) with its
   meaning and its source. All of them are confirmed. No rule dropped out for a missing number *(user)*.
5. **Licence of the test map.** `forest-sample.omap` serves tests only; no forest picture in the manual and no forest
   example under `examples/` until a map with a confirmed licence exists *(user)*.

## Rulings made while building

Each ruling is *(Claude's call)*, with its reason and the cost if it is wrong.

- Ruling: the accepted names are the one name `ISOM 2017-2`, compared without case and spaces — it is what Mapper and
  freemapper write, and the user asked for 2017-2 only — a person with a first-edition `ISOM2017` file is refused and
  has to convert it in Mapper (`ISOM2017-ISOM 2017-2.crt`); accepting the first edition later is one line.
- Ruling: the refusal applies only after the map is detected as a forest map by the old logic, so sprint and
  schoolyard detection and results stay byte-identical, and an ISOM-named map at 1:4000 stays a sprint map — the
  user asked for no change there — a sprint map that is really ISOM at 1:4000 still gets the sprint rules, which is what
  it got before.
- Ruling: a sidecar `freemapper.toml` with `map_kind = "forest"` on a map of another symbol set is refused like any
  other forest map — the sidecar is authoritative for the kind, and the forest rules would read wrong numbers —
  a person who set the key by mistake sees the refusal and fixes the sidecar.
- Ruling: the edge of the mapped area on a forest map is a distance on the sheet, `forest_sheet_mm = 10` in the rules
  `wkb2026.map.inside` and `wkb2026.map.margin`, which is 50 m at 1:5000, 100 m at 1:10000 and 150 m at 1:15000 — the
  50 m of ADR 0088 were tuned for 1:4000 sheets (12.5 mm there) and would be 5 mm on a 1:10000 forest sheet; 10 mm are the
  50 m of the smallest forest scale, so no forest map is stricter than a sprint map. The sample `forest-sample.omap` has no point
  further than 50 m from an object in its 550 x 440 m, so 100 m is not too strict for it. Sprint and schoolyard keep
  `metres = 50`; the sprint output does not change. — if 10 mm is too loose or too strict, one number in
  `wkb-2026.toml` changes it.
- Ruling: the setting `map_margin_m` keeps its default of 50 m on forest maps — it is the user's own setting, and the
  margin is about where a runner can run, not about the scale — a forest margin that should scale needs its own default.
- Ruling: "does not apply" is not a new list. A rule that is only for other map kinds is skipped with the existing
  reason `check-skip-map-kind` ("the rule is not for forest maps"), which `check --all` already lists with the
  reason — a new heading would change the output on sprint maps, where the forest rules are listed the same way —
  the list stays one list; someone who wants two headings changes the output of both kinds.
- Ruling: the rules that need data the program does not have stay "not checked" with their reason: the Lkm rule of the
  overlay `nor-bb-2021` needs the climb (reason `check-skip-climb-missing`), and `leg.through_forbidden` needs the
  route analysis. The overlay stays selectable on forest maps, since Lkm is a forest recommendation — a person who
  enters the climb by hand gets the rule checked.
- Ruling: `score_length_bands`, the one check rule that builds a cost surface for route times, is not checked on a forest
  map (new reason `check-skip-score-forest`) — it would report times from a model without elevation, which the design
  calls worse than a refusal — the other score rules run unchanged, since they read codes, values and positions only.
- Ruling: a 709 area is hidden in Mapper's ISOM 2017-2 symbol set (it is a course-setting symbol), and the map index
  leaves hidden symbols out. A control inside such an area is therefore not reported by
  `wkb2026.control.in_forbidden` — 520 is the map symbol for an out-of-bounds area, and a hidden object is not shown on
  the map — a mapper who hides a 709 area on purpose gets no finding; the out-of-bounds special object of the course
  covers that case.
- Ruling: `714` is not an ISOM 2017-2 symbol (ISSprOM only); the forest rule data already left it out and keeps doing so.
- Ruling: `numbers::overprint_sizes` keeps its own copy of the print values and a test compares it with
  `freemapper-printrules` for every kind (forest circle 5.0 mm, line 0.35 mm, number 4.0 mm) — the printrules call
  needs the whole rule set and the copy is three numbers — a change in printrules fails the test instead of passing
  silently.
- Ruling: the specification text was read in the copy of ISOM 2017-2 (adjusted version of January 2019, errata to
  23.01.2022) that the Italian federation hosts, next to Mapper's symbol set; revision 6 of January 2024 was not
  available as a file — the research file says so — a renumbering in revision 6 would show when someone compares the rows.
- Ruling: the conversion of `forest-sample.omap` with Mapper's `ISOM2000-ISOM 2017-2.crt` was not tried (it needs
  Mapper's GUI); the synthetic ISOM 2017-2 map carries the tests — real forest maps in ISOM 2017-2 stay untested
  until the user provides one.
- Ruling: the editor needs no change for the map itself. It draws the symbols from the definitions in the file, so every
  ISOM 2017-2 symbol in the file is drawn. Symbols that are hidden in the file (course-setting symbols such as 709 and 701
  to 703) are not drawn, and a map larger than one A3 sheet at map scale is clipped to its middle part (a forest map
  at 1:10000 passes that size at 4.2 x 3.0 km). The Score panel keeps its buttons, disabled, with the reason beside them.

## Consequences

- A forest map in ISOM 2017-2 can be set by hand end to end. `forest_chain.rs` runs it: `init`, special objects,
  controls, a course, numbers, `check`, IOF XML against the schema, Purple Pen (`standards map="2017"`, scale 10000),
  import, sync and the editor state.
- `check` on a forest map reports the sprint-only rules (`control.dead_end`, `course.start_finish_distance`) as not
  applying, and the elevation rules as not checked.
- The sprint and schoolyard behaviour is unchanged. The embedded rule snapshot changes by the new rule values only.

## Cost if wrong

- A forest map that is correct but made in ISOM 2017 (first edition) is refused; the person converts it in Mapper.
- The 10 mm edge distance is a proposal; too strict gives false errors at the edge of large forest maps, too loose
  misses a control in a blank corner.
- Forest maps with real ISOM 2017-2 symbols have not been tried here, only the synthetic map and the ISOM 2000
  sample's refusal.

Source: user, 2026-11-13 (forest maps, ISOM 2017-2 only, first stage by hand and `check`); builder check of the design;
rulings of the implementation of 2026-11-14.
