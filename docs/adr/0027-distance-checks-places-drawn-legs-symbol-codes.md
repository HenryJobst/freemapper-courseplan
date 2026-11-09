# 0027. Distance checks: places, drawn legs and symbol codes

Date: 2026-11-09
Status: Accepted

## Context
Plan M2b Task 4 adds `min_distance`, `similar_features`, `map_edge`, `leg_through_control` and
`leg_through_forbidden`. The brief leaves open where a finding with a start sits, which part of a leg counts
and how map symbols match profile codes.

## Decision
- Pair and control checks report at the controls of the event, once per rule, also for shared or butterfly
  controls; a control never pairs with itself. A pair counts if one stop is on a course the rule covers.
- `min_distance` counts start and crossing points, not the finish; `map_edge` counts every stop.
- Start and finish ids go into `controls` of a location and an accepted finding (ids and codes share one
  namespace). `finding accept` takes them for every rule, also where they can never match.
- Leg checks measure the drawn part: the polyline through its bends, without one circle radius at either end.
- A map code matches a profile code when equal or a sub-code (`505.1` matches `505`, `5051` does not).
- `leg_through_forbidden` warns when a straight leg crosses an out-of-bounds special of the course or a symbol
  in `impassable` (520, 708, 709; 714 on sprint maps), as a hint for the best route (WKB26 B 7.2.6, ruling
  R11). Uncrossable objects and forbidden routes wait for the route analysis. Taped legs are exempt.
- The control feature is the nearest feature within `feature_search_mm`, an area by its outline. The same
  point symbol is similar; lines and areas only through a group (ruling R14).
- Measured metres are rounded down; limits keep decimals in the language's mark, such as "37,5 m" for the
  forest circle radius at 1:10000. A missing required value skips the rule with the value's name.

## Consequences
The map edge ignores the print area (R12). Cost if wrong: one line per key in the check table.

Source: plan M2b Task 4; controller rulings R4, R9, R10, R11 and R14, 2026-11-09.
