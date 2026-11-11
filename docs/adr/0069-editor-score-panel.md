# 0069. Editor: the score panel, links on the map and long runs

Date: 2026-11-11
Status: Accepted

## Context
The editor drew no score courses. `score_suggest` and `score_draft` take 5 to 7 seconds, and an operation held the
state lock for its whole run (ADR 0043), so `api/revision` waited as long. Design §9 item 7 and §10 apply.
## Decision
- **Links** *(Claude's call; the user may overrule)*. The overlay of a score course has controls with `value`,
  start, finish, no legs, no numbers. A mandatory link is a `links` entry: a straight dashed line between its two
  controls in the marked-leg style. It is no leg, so nothing on it can be selected or bent. The control description
  does not mention it. This answers §10.
- **Panel** *(Claude's call, ruling 7)*. Time limit, penalty,
  controls, links, settings, and the buttons Suggest, Draft and Check length. Accept adds a candidate with
  `add_control` and `set_score_controls` (two undo steps; a refused second step removes the control again). A
  candidate has a free `code` as far as the code space reaches. Course mode and preview stay with individual courses.
- **Not stored** *(Claude's call)*. Candidates and drafts live in the page. Writes of this page keep them, but a change of the
  limit or the settings drops them. A revision from outside discards them, and the panel says so.
- **Long runs** *(Claude's call)*. `POST api/score/{tool}` (suggest, length, draft; apply is refused) takes the lock
  only to compare the revision, computes on the blocking pool, and compares the file again at the end: a change is a
  conflict and no result. The page is busy meanwhile and the buttons are disabled.
- **Apply writes what was shown** *(Claude's call)*. Apply sends the displayed draft to `score_apply_draft`, an
  editor-only operation (`EDITOR_ONLY`, no MCP tool or CLI command). It writes through the edit operations without
  a search, so taken codes, wrong values and bad links are refused, and the revision is checked. One undo step; the
  lock is held only for that short write.
## Consequences
- Tests slow a real draft and a suggestion and show that `api/revision` answers meanwhile. The page has no browser
  harness; panel, focus and apply were tried with Playwright on a synthetic map.
