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
  polyline without one circle radius at either end. A leg that starts at a fence end or passes a fence at a
  crossing point therefore does not touch the fence, and a control just beside a leg end is the business of
  `min_distance`, not of `leg_through_control`.
- A map symbol code matches a profile code when it is equal or a sub-code of it: `505.1` matches `505`, `5051`
  does not (ruling R9). One helper, `check::symbol_matches`, does this for every check.
- `leg_through_forbidden` cannot tell crossing a line from running along it. Codes in `crossing_allowed`
  therefore never give a finding. A forbidden route special counts as symbol 711, so it gives a finding only
  where 711 is not in `crossing_allowed`; in the WKB 2026 profile that is the sprint rule. Point symbols have no
  extent and never block a leg. Taped legs are exempt. Out-of-bounds areas and forbidden routes apply to a
  course when their course list is empty or names it.
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
- Running along a route that may be crossed goes unreported until a check can compare directions along a
  stretch.
- Cost if wrong: the place kinds are one line per key in the check table; the drawn part is one call per leg.

Source: plan M2b Task 4; controller rulings R4, R9 and R10 and the rulings for Task 4, 2026-11-09.
