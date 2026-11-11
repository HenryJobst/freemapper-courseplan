# 0069. Editor: the score panel, links on the map and long runs

Date: 2026-11-11
Status: Accepted

## Context
Score courses could be edited and drafted through operations (ADRs 0067, 0068), but the editor drew nothing for
them and had no way to run a suggestion. `score_suggest` and `score_draft` take 5 to 7 seconds on a large map. The
editor held its state lock for a whole operation (ADR 0043), so `api/revision` waited as long. Design §9 item 7
and §10 apply.

## Decision
- **Overlay** *(Claude's call)*. A score course comes back as controls (with `value`), start and finish, no legs
  and no numbers. Each mandatory link is a straight `links` entry between its two controls, drawn like a marked leg
  (dashed, circles cut off). It is no leg: nothing on it can be selected, bent or taped. This settles §10: yes, a
  link is a line between the two controls. The description of a control does not mention it. Codes stand beside the
  circles as in the all-controls view.
- **Panel** *(user: panel, candidates, accept, draft with progress; Claude's call: details)*. Selecting a score
  course shows the Score panel in the properties pane: time limit, penalty with a "default" box, the controls and
  links (a link can be removed), the shared settings (the bands read-only), and the buttons "Suggest controls",
  "Draft course" and "Check length". "Accept" adds one candidate with two operations, `add_control` and
  `set_score_controls`, so undo takes each back. "New score course..." in the tree makes a course with a 45 minute
  limit. Course mode and the print preview stay with individual courses. Suggest answers carry a free `code` per
  candidate, as many as the code space holds, best first.
- **Not stored** *(user)*. Candidates and drafts live in the page until accepted, applied or discarded. A write of
  this page keeps them. A new project revision from outside discards them and the panel says so.
- **Long runs** *(Claude's call)*. `POST api/score/{tool}` (`score_suggest`, `score_length`, `score_draft`) takes
  the state lock only to compare the revision, then computes on the blocking pool without it, and compares the file
  again at the end: a changed file gives a conflict and no result. So `api/revision` and the reads answer while a
  run goes on. A draft with `apply` writes, so it takes the lock for the whole run, checks the revision and
  records its undo step; the page is busy then and polls nothing. While a call runs the page marks itself busy, the
  Score buttons are disabled and a status line says it is working.
- The panel sits in the properties pane, so the print preview's `inert` covers it and `api.js` refuses its writes.

## Consequences
- A test makes the long call sleep through a hook and shows that `api/revision` and `api/state` answer meanwhile.
- A draft that applies still blocks other requests for its run. Splitting compute and write would remove that.
- The page has no browser test harness; panel behaviour was tried by hand on the Lübeck example (its start lies
  in a building, so only the error path ran).

Source: design 2026-11-11 §9 item 7, §10
