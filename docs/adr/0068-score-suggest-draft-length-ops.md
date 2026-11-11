# 0068. Score courses: suggest, draft and length operations

Date: 2026-11-11
Status: Accepted

## Context
The library can suggest candidates (ADR 0062), assign points and codes (ADR 0063) and draft a course (ADR 0065).
Nothing reached the setter. ADR 0059 applies: one operation, CLI and MCP.

## Decision
- Three operations, `score_suggest`, `score_draft` and `score_length`, in `mcp/score.rs`. The CLI calls them through
  `call_json`; the MCP tools run them on `spawn_blocking`. `score_draft` is an editor operation, so undo works; the
  two reads are not *(Claude's call)*.
- Only `score_draft` with `apply: true` writes, through the edit operations (`add_score_course`, `add_control`,
  `set_score_controls`, `add_score_link`). A replaced score course loses its links, and its controls that no other
  course uses; their codes and sites are free again, so a second run gives the same course *(Claude's call)*. A course
  of another kind is refused. An existing course keeps its own start and finish *(Claude's call)*.
- Two raster surfaces per call: 2 m for the way from the start (effort), 4 m for the legs between controls, the
  `cell_metres` of the length rule, so a draft and `check` agree. If 4 m cannot join start and finish, the draft uses
  the fine surface *(Claude's call)*. Blocked areas are the out-of-bounds and dangerous areas, as in `check`;
  forbidden routes stay out, because the surface blocks only polygons.
- Defaults: 60 candidates for suggest and as the pool of the draft, so link ends from `score suggest` are in the
  pool; a minimum distance of 15 m on the ground at 1:4000 and larger scales, else 30 m (WKB26 B 7.2.1) *(Claude's call)*.
- A failed draft is an error result and exit code 1: one sentence per missed band with its size, the closest attempt,
  and the hint. Forest maps are refused as in `init`.
- `[profile.dev.package.freemapper-courseplan] opt-level = 2`: the route search took a minute per draft unoptimised
  *(Claude's call)*.

## Consequences
On the Luebeck map in a release build, `score suggest` takes about 5 s and `score draft` about 7 s; the example's
start flag lies inside a building, so it needs a new position first.
