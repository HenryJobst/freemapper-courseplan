# 0088: The mapped area reaches 50 m beyond the map objects

Status: Accepted, 2026-11-13. Changes a value of [0084](0084-start-finish-and-controls-must-lie-on-the-mapped-area.md) and
[0087](0087-start-finish-and-controls-keep-a-margin-from-the-edge-of-the-mapped-area.md).

## Context

ADR 0084 counts a position as mapped when a map object lies within 40 m. ADR 0087 asks for a margin of 50 m from the
edge of that area. Two numbers for what is one idea in the user's mind: 40 m and 50 m.

## Decision

- The limit of `wkb2026.map.inside` (`metres`) is 50 m, the same number as the default of `map_margin_m` *(user,
  2026-11-13)*. The constant `MAPPED_METRES` follows.
- Both rules stay *(Claude's call)*. `map.inside` says a position is on blank sheet and has its own switch in the rule
  profile. `map.margin` says it is too close to the edge and has the event setting `map_margin_m`, 0 switches it off.
  Their messages differ, and `map.margin` leaves a position in the blank part to `map.inside`.

## Consequences

- With the defaults, the edge lies 50 m beyond the outermost objects and the margin is 50 m. A position at the
  outline of the objects has exactly 50 m to the edge, one outside it has less and `map.margin` reports it. Before,
  the edge lay 40 m beyond the objects, so a position had to stand 10 m inside the outline. That strip of 10 m is
  usable again. `map.inside` alone (margin 0) now allows 50 m outside instead of 40.
- Examples Gleisdreieck and Rummelsburg pass `check` without errors.
- Written drafts keep the margin, so candidate sites in the outer 10 m of a map can now appear.

## Cost if wrong

- A map with a few outlying objects, such as a logo, counts a larger blank part as mapped. The rule profile `metres`
  lowers the limit.

Source: the user, 2026-11-13.
