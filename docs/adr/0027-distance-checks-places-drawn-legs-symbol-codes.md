# 0027. Distance checks: places, drawn legs and symbol codes

Date: 2026-11-09
Status: Accepted

## Context

Plan M2b Task 4 implements five map-based checks: `min_distance`, `similar_features`, `map_edge`,
`leg_through_control` and `leg_through_forbidden`. The brief gives the rules and values. It leaves open where
a finding sits when a start is involved, which part of a leg is measured, how map symbol codes match the codes
of a profile, and what to do with features that may be crossed.

## Decision

- Controls are shared by courses, so `min_distance`, `similar_features` and `map_edge` report at the controls
  of the event, once per rule, even when several courses or a butterfly loop use them. A control never pairs
  with itself. A pair counts when at least one of the two stops belongs to a course the rule covers. The other
  may belong to any checked course, so a control of a course at another print scale is still a neighbour.
  Controls no course uses are not out in the terrain and are left out.
- `min_distance` counts crossing points and the start, not the finish (brief, IOF-R 19.4). `map_edge` counts
  every stop. `similar_features` counts controls only, because crossing points, start and finish have no
  control feature.
- A start or finish can be a place of a finding. Its id goes into `controls` of the location and of an accepted
  finding, next to control codes. Ids and codes share one namespace (`CodeClashesWithSpecial`), so the key
  stays unambiguous, and `finding accept` takes a start or finish id where it takes a control code. The project
  file format does not change.
- Leg findings sit at `Location::Leg { course, after }`, one per leg and rule, naming the nearest stop or the
  first obstacle. Legs run through their bends. Both leg checks measure only the drawn part of the leg: the
  polyline without one circle radius at either end. A leg that starts at the edge of an out-of-bounds area or
  passes its boundary at a crossing point therefore does not touch it, and a control just beside a leg end is the business of
  `min_distance`, not of `leg_through_control`.
- A map symbol code matches a profile code when it is equal or a sub-code of it: `505.1` matches `505`, `5051`
  does not (ruling R9). One helper, `check::symbol_matches`, does this for every check.
- `leg_through_forbidden` warns when the drawn part of a straight leg crosses an out-of-bounds area special
  of the course (its course list empty or naming the course) or a map feature whose symbol is in `impassable`:
  520, 708, 709, and 714 on sprint maps (ruling R11). The straight leg stands in for the best route, which
  WKB26 B 7.2.6 says must not tempt runners through out-of-bounds areas, so the severity is `warning`.
  Uncrossable lines and objects (cliffs, water, walls, fences, buildings, bridges) need the route analysis and
  are not checked; nor are forbidden routes and symbol 711, which may be crossed (IOF-R 17.2) and are only
  wrong to run along. The `crossing_allowed` value is gone with them. Point symbols never block a leg. Taped
  legs are exempt.
- The control feature is the nearest map feature within `feature_search_mm`, an area counted by its outline.
  Otherwise every control inside one large paved area would stand at the same symbol.
- The map edge is the box around all map features the map index holds; sheet objects are left out. A point
  outside it is 0 m from the edge. A map without features gives a skip.
- Measured values in the finding texts are rounded down to whole metres, so a value just below the limit never
  shows as the limit. Limits keep their decimals, with the decimal mark of the language.
- A rule without a value its check needs (`metres`, `feature_search_mm`) is skipped with the value's name.

## Consequences

- The print area of a course is not the map edge yet. A course printed on a smaller sheet can have a control
  close to its sheet edge without a finding.
- Running along a forbidden route, and legs through buildings, walls or water, go unreported until the route
  analysis exists.
- Cost if wrong: the place kinds are one line per key in the check table; the drawn part is one call per leg.

Source: plan M2b Task 4; controller rulings R4, R9, R10 and R11 and the rulings for Task 4, 2026-11-09.
