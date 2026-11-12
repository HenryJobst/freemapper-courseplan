# 0078: No score sites on the map frame

Status: Accepted, 2026-11-12. Amends [0073](0073-score-candidates-wider-pool.md).

## Context

The Gleisdreieck map (ADR 0077) has a closed, axis-aligned rectangle drawn with symbol 501.1 around its content. It is
the line `501.1` ("step or edge of paved area") with four corners, 296.7 by 420 map millimetres inside an extent of
296.7 by 441.2. `score suggest` takes the corners of such outlines as sites, so the four corners of the frame became
candidates. A 40-minute draft used all four of them, one at each corner of the map. At those places `map near` finds
nothing but the frame line itself.

## Decision

- An outline that is a closed, axis-aligned rectangle of four corners and spans at least 90 percent of the extent of the
  mapped objects in both directions gives no sites, on a map with at least 50 objects (a lone rectangle is content, not a frame) *(Claude's call)*. Other rectangles, such as buildings, and large
  outlines with more corners are unchanged. The test is `is_map_frame` in `score/candidates.rs`.
- The Gleisdreieck example was redone without these sites. The 40-minute course it had is gone: without the corners the
  map supports 33 minutes (README of the example).

## Cost if wrong

A real feature that happens to be a large rectangle spanning the whole map loses its corners as sites. That is rare, and
a person can still set a control there by hand. The 90 percent span and the exact axis alignment (0.01 mm) were
chosen by looking at two maps; a third map with a rotated or slightly irregular frame would still give the corner sites.
