# 0073: Score candidates: a wider pool

Status: Accepted, 2026-11-11. Amends [0062](0062-score-candidates-and-points.md).

## Context

With the distances of ADR 0072 the candidate pool was the limit: 19 sites on the Lübeck map, 137 on a 1.8 x 2.0 km
map of Berlin-Kreuzberg. Both hold buildings, canopies and paths almost alone, and only building corners were
sites. A pick that could not be routed also pushed every site within 100 m away; on Lübeck 19 of 38 tries were lost
that way.

## Decision

- `candidates.toml` gets two kinds *(Claude's call)*: `end` (a free end of an open line) and `junction` (a line
  end that meets another line, or three that meet). A path cut in two is neither. Crossings stay out; bridges and
  tunnels make them ambiguous.
- New classes with a clarity: small buildings 0.9; canopy as a whole 0.9; prominent water features, landforms,
  crossing points, small cliffs 0.8; ends of walls, bridges, tunnels, stairways 0.8; ends of fences, hedges, step
  lines 0.7; water banks, earth banks 0.7; hedge and gully corners 0.6; path ends and junctions 0.6; path bends,
  edges of paved areas, cultivation boundaries 0.5. Clarity falls with what moves, grows or only changes the surface.
  Low clarity raises the difficulty and weighs the pick down; it never removes a site.
- A closed line gets all its vertices as corners (the first was lost).
- A site that cannot be routed pushes only sites within 0.3 of the minimum distance (`LOST_PUSH_SHARE`) *(Claude's
  call)*. A routed pick keeps the full distance.
- Checked, unchanged: gap, value-1 distance and reach convert as `m * 1000 / scale` (100 m at 1:4000 is 25 mm), and
  the graded spacing of 0071 only orders the picks.

## Consequences

`score suggest --limit 200` finds 31 sites on Lübeck (before 19) and 183 on Kreuzberg (before 137). Both maps are
still mostly buildings: of the Kreuzberg picks 152 are building corners. The draft there is no longer bound by the
pool: it ends at 13 controls for 30 minutes and 26 to 28 for 45, because the very good runner has no time for more,
and it misses the good runner's band by 1.6 to 3 points. That belongs to the draft's search (0063, 0065). The classes
are judged on paper; the user looks at the sites on the map. More weak sites raise the density term near paths. A
corner whose coarse cell is blocked is still lost (0062).
