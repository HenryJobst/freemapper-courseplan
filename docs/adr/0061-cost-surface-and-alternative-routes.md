# 0061. Cost surface: raster, speed classes and alternative routes

Date: 2026-11-11
Status: Accepted

## Context
Score courses need the time between two points and the route choice on the leg (design §4.1, §9 ruling 5).
freemapper-core has no speed or passability classification, only impassable codes in the check rules.

## Decision
- **Raster** *(user: simple start; Claude's call: details)*. Cells of `cell_m` metres (default 2), each with a factor:
  time per metre relative to level open ground. The pace scales it.
- **Speed classes** *(Claude's call)* in `score/speeds.toml`, keyed by ISSprOM code; a code matches its sub-codes and
  the longest match wins. `block`: cliffs, water, buildings, walls, out-of-bounds and closed areas. `cross`: bridges,
  crossing points. `ground`: slowest cover wins (forest 2.0, rough open 1.5, marsh 2.5). `fast`: paths and paved
  ground, 0.8 to 0.9, replacing the ground. All else counts 1.0. Special objects come in as `blocked` polygons; a ring
  of fewer than three points blocks along a line. The numbers are first guesses, not tuned on race data.
- **Search**. Dijkstra on 8 neighbours, no diagonal step past a blocked cell, equal costs resolved by the lower cell
  index. A start or end in a blocked cell moves to the nearest free cell within 3 cells, else `ScoreError::Unreachable`.
- **Alternatives** *(Claude's call)* by the penalty method: cells within 10 m of each route found cost three times as
  much, then the search runs again (at most three searches per wanted route). A candidate is kept when it is at most
  `max_length_ratio` times as long as the best, at most `max_overlap` of its cells lie within 10 m of each accepted
  route (the overlap), and some cell lies beyond 20 m from it. The last test drops the first route moved sideways,
  which the penalty yields on open ground. Defaults: 4 routes, 0.6, 1.25.
- **Timing**, Lübeck map (522 by 413 cells): build 31 ms debug, 4 ms release; one corner-to-corner leg with four
  routes 0.4 s debug, 45 ms release.

## Consequences
- Narrow obstacles give one route only: two routes less than 20 m apart count as one choice.
- A wall thinner than a cell still blocks, since outlines of blocked areas are painted too. The grid skews times by
  a few percent; length is measured on the corner points.

Source: design 2026-11-11 §4.1, §9 ruling 5; plan task 3
