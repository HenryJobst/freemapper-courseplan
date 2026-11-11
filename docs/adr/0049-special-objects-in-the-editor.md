# 0049. Editor: special objects, no move operation, redraw

Date: 2026-11-11
Status: Accepted

## Context
Special objects (start, finish, first aid post, water point, marked route, forbidden route, out-of-bounds area,
dangerous area) are drawn in the editor. The three interfaces have `add_special`, `remove_special` and
`set_special_courses`. None of them moves an object or changes its points.

## Decision
- **No move operation** *(Claude's call)*. The editor offers draw, assign to courses and delete. A wrong object is
  deleted and drawn again. A move operation would be new logic for the CLI, the MCP tools and the editor alike, with
  its own checks, and it is left out of this milestone.
- **Gestures** *(Claude's call)*. A point kind takes one click. A line or an area takes a click per point and ends
  with Enter or a double click. Esc cancels, Delete takes back the last point. An area needs three points, a marked
  route two, a forbidden route one or more. The finished object is one `add_special` and one undo step.
- **Ids** *(Claude's call)*. The page proposes `<kind>-<n>` with the first free `n`. The user may edit it, and the
  server refuses a taken id.
- **Courses** *(Claude's call)*. Check boxes in the properties send `set_special_courses`. With none ticked the
  object belongs to all courses, as the model says. Start and finish have no list.
- **Drawn from the overlay** *(Claude's call)*. The server's overlay already returns every kind with its points,
  so the page adds no geometry. Each kind also carries a letter or sign (M, OB, !, W, plus, cross) so that colour is
  not the only cue.
- **Double click** *(Claude's call, changed after the stage 3 review)*. A click within 3 screen pixels of the last
  point of a drawing adds nothing, so the clicks of a double click set their point once, and the `dblclick`
  event only finishes the drawing. `addBody` also drops a point closer than 0.05 mm to the one before it, which
  covers Enter after a double click. No near-duplicate point is left for the user to delete.

## Consequences
- Moving an object costs a delete and a redraw, and a second undo step.
- If users ask for a move, it becomes an operation in the library first (new ADR), then in all three interfaces.
- The duplicate last point after a double click is a known rough edge.

Source: browser editor plan M4, task 14 (2026-11-11).
