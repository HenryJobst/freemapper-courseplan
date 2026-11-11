# 0071: Score candidates: graded spacing and rim weight

Status: Accepted, 2026-11-11. Amends [0062](0062-score-candidates.md).

## Context

The first review of the suggestions on the Lübeck map by the user *(user)*: too many candidates sit at the rim
of the map, and the controls with value 1 lie far from the start. The cause was the pick itself: always the
site farthest from everything taken so far. That fills the rim first, and the values come from the order of the
travel time, so even the nearest fifth was 360 m (median) from the start.

## Decision

- The pick still takes the site with the largest distance to the nearest start, finish or pick, weighted by
  clarity *(Claude's call)*. The weight now divides by a spacing that grows linearly from 1 at the start to 3 at
  the farthest site, so close to the start the picks may lie closer together (the minimum distance still holds).
- A site within three minimum distances of the rim of the area that the map objects span is weighed down
  linearly, to 0.3 at the rim.
- The constants are `SPACING_GROWTH` (2.0), `RIM_MARGIN_DISTANCES` (3.0) and `RIM_FLOOR` (0.3).

## Consequences

On the example (60 candidates): sites within 48 m of the rim fell from 21 to 3, the median distance of the
value-1 controls from the start from 90 mm to 57 mm (360 m to 230 m). The values themselves are still
quantiles of the travel time (0063). The constants are a first guess; the user judges them on the map.
