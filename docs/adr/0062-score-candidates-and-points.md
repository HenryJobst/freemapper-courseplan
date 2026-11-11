# 0062. Score courses: control candidates and points

Date: 2026-11-11
Status: Accepted

## Context
The score builder needs sites where a control could stand, each with route choice, difficulty and effort
(design §4.2, §9 rulings 4, 5, 8). freemapper-core has no object class for a clear control site.

## Candidates *(Claude's call)*
- **Sites** come from `score/candidates.toml`, keyed by ISSprOM code with a clarity from 0 to 1. Point symbols
  (boulders, fountain, tower, monument, pillar, prominent features) give a site at their position. Corner
  vertices (turn of 45 degrees or more) of walls, fences, stairs, step lines, buildings and canopies give the
  others. ISSprOM has no bench symbol; benches appear as prominent man-made features (530, 531).
- **Rules**: a site keeps `min_distance_mm` to start, finish and existing controls (the caller takes the number
  from the rule profile) and to a clearer site already kept. It lies outside out-of-bounds and dangerous areas.
  A point site must be passable; a corner may lie in the blocked cell of its wall, so it only needs a route from
  the start (`Unreachable` drops it).
- **Bound**: at most `limit * 2` sites, clearest first, ties by x then y, go on to routing. Each gets the leg to
  the start, to its 4 nearest others and to 2 others picked by a fixed-seed generator. Legs are cached per pair.
  Lübeck (1978 buildings, 9754 raw sites), limit 60, release build: 8.9 s. With `limit * 4` sites and 8 + 8 legs
  it took 45 s.
- **Route choice** per leg: 0 for one route; otherwise 0.5 for two routes within `max_length_ratio` of the best
  length, 1.0 for three or more, times `1 - 0.5 * s`, where `s` is how far the second route lies between the best
  length (0) and the ratio limit (1). Route choice of a candidate is the mean over its sampled legs.
- **Difficulty** = 0.5 * (1 - route choice) + 0.3 * (1 - clarity) + 0.2 * density, density being the other raw
  sites within 5 mm over 6. **Effort** is the time of the fastest route from the start.
- Sort: route choice descending, then x, then y.

## Consequences
- Sampling means a route-choice value is an estimate; it can change when `limit` changes.
- Clarity values and weights are guesses, not tuned on race data.
Source: design 2026-11-11 §4.2; plan task 4
