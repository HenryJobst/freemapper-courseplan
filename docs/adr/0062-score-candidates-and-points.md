# 0062. Score courses: control candidates and points

Date: 2026-11-11
Status: Accepted

## Context
The score builder needs sites where a control could stand, each with route choice, difficulty and effort
(design §4.2, §9 rulings 4, 5, 8). freemapper-core has no object class for a clear control site.

## Decision
Candidates *(user: features such as steps, wall corners, benches, fountains and building corners; Claude's call: the rest)*:
- Sites come from `score/candidates.toml`, ISSprOM codes with a clarity from 0 to 1. Point symbols give a site
  at their position. Corners (turn of 45 degrees or more) of walls, fences, stairs, step lines, buildings and
  canopies give the rest. ISSprOM has no bench symbol; benches show up as 530 or 531.
- A site keeps `min_distance_mm` (the caller takes it from the rule profile) to start, finish, existing controls
  and other candidates, and lies outside out-of-bounds and dangerous areas. A corner may sit in the blocked cell
  of its wall, so its routes end at the nearest free place within 1.5 map mm. That can be one snap radius away.
- A start or finish that the surface cannot reach is an error (`StartUnreachable`, `FinishUnreachable`). The
  surface snaps a blocked end point to a free cell within three cells, so only a point enclosed by blocked cells
  fails, such as the Lübeck example's start flag, which lies inside a building.
- Sites are picked by greedy farthest-point selection from start, finish and the sites already picked, clear
  features first, so they spread over the map. A site without a route from the start or to the finish is skipped
  and the next one is tried, up to `4 * limit` tries for `2 * limit` candidates. Each candidate gets legs to its
  4 nearest others and to the start. Legs with alternatives run on a coarser surface (8 m cells); the time from
  the start (`effort_s`, in seconds) runs on the fine one through `TravelTime::fastest`.
- Route choice (0 to 1) of a leg is 0 for one route; otherwise 0.5 for two routes within `max_length_ratio` of
  the best length and 1 for three or more, times `1 - 0.5 * s`, where `s` runs from 0 (second route as long as
  the best) to 1 (second route at the ratio limit). A candidate takes the mean over its legs.
- Difficulty (0 to 1) is 0.5 * (1 - route choice) + 0.3 * (1 - clarity) + 0.2 * density; density is the number
  of other sites within 5 mm over 6, at most 1.
- The list is sorted by route choice, then x, then y.

## Consequences
- Route choice is an estimate and shifts with `limit`. Clarity values and weights are guesses.
Source: design 2026-11-11 §4.2
