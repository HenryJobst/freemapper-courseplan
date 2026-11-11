# 0068. Score courses: suggest, draft and length operations

Date: 2026-11-11
Status: Accepted
## Context
The library suggests, values and drafts (ADRs 0062 to 0065); ADR 0059 puts it in reach of the setter.

## Decision
- Three operations, `score_suggest`, `score_draft` and `score_length`, in `mcp/score.rs`. The CLI calls them through
  `call_json`; the MCP tools run them on `spawn_blocking`. `score_draft` is an editor operation, so undo works; the
  two reads are not *(Claude's call)*.
- Only `score_draft` with `apply: true` writes, through the edit operations. A replaced score course loses its links
  and the controls no other course uses, so their codes and sites are free again and a second run gives the same
  course. A course of another kind is refused. An existing course keeps its start and finish *(Claude's call)*.
- Two raster surfaces per call: 2 m for the way from the start, 4 m for the legs between controls (the
  `cell_metres` of the length rule, so a draft and `check` agree), with the fine one as fallback when 4 m cannot join
  start and finish. Out-of-bounds and dangerous areas are blocked as in `check` *(Claude's call)*.
- **Forbidden routes** *(Claude's call)*. A forbidden route is a polyline, and the surface blocks areas only. Routes
  may cross it; `check` warns about links and legs across it, as for individual courses. Candidates stay 4 m
  (`FORBIDDEN_LINE_MARGIN_M`, one fine cell) away from it. Design 4.1 ("a forbidden route blocks") holds for sites only.
- Defaults: 60 candidates for suggest and as the draft's pool, so link ends from `score suggest` lie in the pool; a
  minimum distance of 15 m on the ground at 1:4000 and larger scales, else 30 m (WKB26 B 7.2.1) *(Claude's call)*.
- A failed draft is an error result and exit code 1: a sentence per missed band, the closest attempt and the hint.
  Forest maps are refused as in `init`.
- `[profile.dev.package.freemapper-courseplan] opt-level = 2`: a draft took a minute unoptimised *(Claude's call)*.

## Consequences
On the Lübeck map (release build) `suggest` takes about 5 s and `draft` 5 to 10 s. `suggest` takes `course` for a
score course's start and finish (the editor sends it). A draft may fall back to the 2 m surface while `length` and
`check` stay at 4 m, so they can disagree on a narrow map.
