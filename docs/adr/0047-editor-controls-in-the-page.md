# 0047. Editor: selecting, placing, moving and deleting controls in the page

Date: 2026-11-11
Status: Accepted

## Context
Stage 2 of the editor changes the project for the first time. The page needs a clear rule for what a click hits,
when a change is sent, what happens when the file moved under the page, and which keys do what.

## Decision
- **Hit radius** *(Claude's call)*. A click or press selects the control whose centre is nearest within
  `sizes.circle_r + sizes.line` (map mm) of the pointer. The test is geometric, not by the SVG element under the
  pointer, so a thin circle is easy to hit at any zoom. A number box or a special object under the pointer is
  selected by its element.
- **One gesture, one operation** *(Claude's call)*. Placing is `add_control`, a drop or a changed X/Y field is
  `move_control`, delete is `remove_control`. A drag moves only the circle live; legs and numbers follow after the
  drop, when the page loads the overlay again. Arrow keys move the circle live and send one `move_control` when the
  key is released, so holding a key never makes more than one undo step. Esc cancels a drag or a nudge.
- **The page always shows the file** *(Claude's call)*. After every answer, success or not, the page takes the
  returned revision, loads `api/state` and the overlay again and so snaps a refused drag back. A 409 means the
  file changed outside the editor: the banner and the status bar show the server's text, and the page shows the
  other writer's state. The page's own writes do not count as outside changes for the poll.
- **Messages** *(Claude's call)*. Refusals (400, 409, 422) show in the status bar and in a dismissible
  `role="alert"` banner at the bottom of the map pane. It sits over the map's lower edge, so the map does not
  move when it appears.
- **Fields** *(Claude's call)*. X and Y take a dot or a comma and are sent as JSON numbers. Anything else marks
  the field `aria-invalid` and shows a message the field points to with `aria-describedby`. The code is read-only
  until the code operation of stage 3; the kind has no operation.
- **Delete asks when a course visits the control** *(Claude's call)*, in a modal `<dialog>` listing the courses,
  with Cancel focused. The operation removes the control from those courses (ADR 0024).
- **`GET api/map/near?x&y&r`** *(Claude's call)*. Lists the map objects within `r` mm (at most 50) of a point, for
  the properties of a selected control (r = 3). It answers from the cached map index with the code of the MCP
  tool `map_features_near` (`features_near_json`), so the tool and the page cannot differ. Each parameter must
  appear once and be a finite number; the route takes no file names.
- **Shortcuts** *(plan task 9)*. Ctrl/Cmd+Z undo, Ctrl/Cmd+Shift+Z and Ctrl+Y redo, Delete and Backspace remove,
  Esc cancels, V and C pick the tool, arrows nudge by 0.1 mm (Shift 1 mm), `+`, `-`, `0` zoom, `?` lists them.
  Keys typed into a field are never taken. Ctrl/Cmd with `+`, `-` or `0` stays the browser's zoom. With a control
  selected, arrows nudge it; without one, they pan the map.

## Consequences
- Dragging a circle does not move its legs until the drop. A live redraw of legs would need the server's drawn
  parts per frame.
- Undo, redo and delete work from the keyboard, which makes a stray Backspace on the map delete a selected control.
  Delete asks only when a course uses it; undo takes it back.
- The map index is held once in memory, so the near list costs one index lookup per selection.
