# 0061. Cost surface: raster, speed classes and alternative routes

Date: 2026-11-11
Status: Accepted

## Context
Score courses need travel time and route choice per leg (design §4.1, §9 ruling 5); the core has no speed table.

## Decision
- **Raster** *(user: simple start; Claude's call: details)*. Cells of `cell_m` m (default 2) with a time factor
  relative to level open ground; the pace scales it. Lübeck (522 by 413 cells): build 31 ms debug, 4 ms release.
- **Speed classes** *(Claude's call)* in `score/speeds.toml`, keyed by ISSprOM code, longest match wins: `solid`
  (cliffs, water, buildings), `block` (walls, fences, out-of-bounds lines), `cross` (opens `block` only), `bridge`
  (opens everything), `ground` (slowest wins, forest 1.1, walk 2.5), `fast` (paths, 0.8 to 0.9). Else 1.0. Special objects
  come in as `solid` polygons. Numbers are guesses, not tuned on race data.
- **Search**. Dijkstra on 8 neighbours, no diagonal step past a blocked cell, ties by the lower cell index. An end
  point in a blocked cell moves to the nearest free cell within 3 cells over free cells only, else
  `ScoreError::Unreachable`. `passable` does not snap. The longest listed code wins, so `515.1` (railway helper)
  is ground, not the wall `515`.
- **Alternatives** *(Claude's call)* by the penalty method: cells within one corridor of each route found cost three
  times as much, then the search runs again (at most three searches per wanted route). Corridor = 12 percent of
  the best route, 2 cells to 10 m. A candidate is kept when it is at most `max_length_ratio` times as long as the
  best, at most `max_overlap` of its cells (not counting those near start and end) lie in the corridor of each
  accepted route, and it strays more than two corridors from that route. Defaults: 4 routes, 0.6, 1.25.

## Consequences
- Routes less than two corridors apart are one choice (12 m on a 50 m leg). Eight neighbours overestimate
  off-45-degree directions by up to about 8 percent; a finer neighbourhood is a later option.
- Paved ground is faster than the setter's pace, since the fast factors are below 1.
Source: design 2026-11-11 §4.1, §9 ruling 5; plan task 3
