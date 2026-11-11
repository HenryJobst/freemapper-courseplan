# 0052. Editor: findings come from api/check

Date: 2026-11-11
Status: Accepted

## Context
The editor shows what `check` finds while the user works: in the panel of the selection, on the map, in the
status bar and in a drawer. It also lets the user accept a finding with a note.

## Decision
- **One check, one answer** *(Claude's call)*. `GET api/check` returns what the MCP tool `check` returns
  (`mcp::report_json`, now `pub(crate)`) plus `revision`. The editor builds it from the cached map and the loaded
  project, so no file is read twice. The result is cached on the project revision and the map stamp: repeated
  polls cost nothing, a changed map file is read again.
- **When it runs** *(user, plan)*. The page asks 500 ms after every `loadState`, which every successful operation,
  undo, redo and outside change goes through, and on load. A request that comes in meanwhile replaces the waiting one.
- **Where a finding shows** *(Claude's call)*. The panel lists the findings whose place is the selected control
  (also the control behind a selected number box and the legs that start or end at it), leg, course (its legs and
  its controls) or the event settings. The status bar counts the whole event. A separate SVG layer under the same
  transform draws a ring around controls and a thick dashed line along legs, with a sign per severity (cross in a
  circle, triangle, "i" in a circle), so colour is never the only cue. Findings of rule `wkb2026.number.gap` put
  a ring and the sign at the number box instead (the box with index `after + 1` for a leg, the box of the code for a
  control). The layer takes no pointer events.
- **Accepting** *(Claude's call)*. The note dialog takes 1 to 200 characters (counted as characters, trimmed).
  `accept_finding` gets the key the finding itself has (`Finding::key`): controls, course or legs by course and
  position. `remove_accepted_finding` takes the key of the finding, or the stored fields of an acceptance that
  matches nothing any more. Each is one operation and one undo step.
- **Texts** *(Claude's call)*. A finding's text, the skipped rules' text and their reasons come from the check
  engine in the server's language. The page puts them in with `textContent` and nothing else.

## Consequences
The page does not decide anything about rules. Findings of event, score or relay courses have no place on the map
and show in the drawer only.
