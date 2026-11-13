# 0087: Start, finish and controls keep a margin from the edge of the mapped area

Status: Accepted, 2026-11-12. Follows [0084](0084-start-finish-and-controls-must-lie-on-the-mapped-area.md).

## Context

ADR 0084 requires start, finish and controls to lie on the mapped area: a map object within 40 m. A position right at
that limit leaves the runner no ground to run on beyond it. The user wants the runner always able to run on mapped
ground: start, finish and controls at least 50 m from the edge of the mapped area.

## What the existing rules measure

- `map_edge` (warning, 10 m): the distance to the rectangle around all map objects. A diagonal strip leaves blank
  corners in that rectangle, so it sees nothing there.
- `inside_map` (error, 40 m): whether a map object lies within 40 m. It says nothing about how far inside a place is.

## Decision *(user, 2026-11-12)*

Start, finish, controls and crossing points keep at least 50 m from the edge of the mapped area. The edge is the border
of the mapped area, not the box around the objects. The value is a setting.

## Design *(Claude's call)*

- The mapped area is the one of `inside_map`: every place within 40 m of a map object, without the sheet frame and the
  layout symbols (601, 602). The margin of a place is its distance to the nearest place that is no longer mapped.
- It is measured on a raster of the mapped area (2 m cells, coarser for very large maps): one exact distance transform
  to the nearest object, cut at 40 m, and a second to the nearest unmapped cell. The result is right to about 2 m. The
  tests put a place just under, on and just over the limit, and the last one allows 2.5 m of slack.
- A margin of 50 m measured from an edge that lies 40 m beyond the objects means a place at least 10 m inside the
  objects. That makes `map_edge` (10 m to the box) mostly redundant on strips, but it stays untouched.
- No double reporting: a place outside the mapped area is the business of `inside_map`; `map_margin` reports only places
  inside it that are too close to the edge.
- The setting is `map_margin_m` in the `[score]` table, default 50, range 0 to 200, next to the settings of ADR 0085 and
  0086. 0 switches the rule off. The rule values of a profile are not settable by the user, so the event setting is the
  way to change it.
- The rule is `wkb2026.map.margin`, severity error like `inside_map`, for start, finish, controls and crossing points
  of individual courses and for start, finish and controls of score courses, by the same pattern as `inside_map`. The
  source is the user. `finding accept` takes it like any other finding.
- Generation: `score suggest` and `score draft` leave out sites closer than the margin. A start or finish that is too
  close is refused with a message that gives the place, the distance, the limit and the setting (`score draft`, and
  `score suggest --course`).

## Consequences

The Gleisdreieck and Rummelsburg examples keep the margin (changelog). A property test over both example maps and
several seeds checks that written drafts have no `map_margin` finding.

## Cost if wrong

- Narrow strips of map leave little room. The Rummelsburger Bucht strip is the case: start, finish and controls had to
  stay near its middle.
- The raster is 2 m coarse, so a place within 2 m of the limit may be judged either way.
- The 40 m that define "mapped" come from ADR 0084. A map with open land drawn without objects shows a smaller mapped
  area than it has, and the margin shrinks the usable area further.

Source: the user, 2026-11-12.
