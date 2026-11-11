# 0062. Score courses: control candidates and points

Date: 2026-11-11
Status: Accepted
## Context
The score builder needs sites where a control could stand, each with route choice, difficulty and effort
(design §4.2, §9 rulings 4, 5, 8). freemapper-core has no class for a clear control site.

## Decision
Candidates *(Claude's call)*:
- Sites come from `score/candidates.toml`: ISSprOM point symbols at their position, and corners (turn of 45
  degrees or more) of walls, fences, stairs, step lines, buildings and canopies. Each has a clarity from 0 to 1.
  ISSprOM has no bench symbol; benches show up as 530 or 531.
- A site keeps `min_distance_mm` (the caller takes it from the rule profile) to start, finish, existing controls
  and other candidates, and lies outside out-of-bounds and dangerous areas. A corner in the blocked cell of its
  wall routes from the nearest free place within 1.5 map mm. An unreachable start or finish is an error
  (`StartUnreachable`, `FinishUnreachable`), for example the Lübeck example's start flag inside a building.
- Sites are picked farthest-point first from start, finish and earlier picks, clear features preferred. A pick
  without a route is skipped, checked from the site itself on the coarse surface as the tours route it: at most `4 * limit` tries for `2 * limit` candidates. Each gets legs to the start
  and its 4 nearest others. Legs with alternatives run on a coarser surface (4 m cells; the fine one if the
  coarse one cannot join start and finish); `effort_s` in seconds runs on the fine one through `TravelTime::fastest`.
- Route choice (0 to 1) of a leg is 0 for one route; otherwise 0.5 for two routes within `max_length_ratio` of
  the best length and 1 for three or more, times `1 - 0.5 * s`, where `s` runs from 0 (second route as long as
  the best) to 1 (second route at the ratio limit). A candidate takes the mean over its legs, and the list is
  sorted by it, then by x, then by y.
- Difficulty (0 to 1) is 0.5 * (1 - route choice) + 0.3 * (1 - clarity) + 0.2 * density; density is the number
  of other sites within 5 mm over 6, at most 1.

## Consequences
- Route choice is an estimate and shifts with `limit`. Clarity values and weights are guesses.
