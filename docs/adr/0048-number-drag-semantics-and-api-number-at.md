# 0048. Editor: number drag semantics and `api/number-at`

Date: 2026-11-11
Status: Accepted

## Context
A control number sits at a direction and a gap from its circle (ADR 0030, 0036). The editor lets the user drag a
number box. The page knows only where the box was dropped, and the model stores a direction and a gap, so something
must convert one into the other. It must not differ from what the CLI and the MCP tools store.

## Decision
- **Which number a drag sets** *(Claude's call)*. In course mode a drag sets the number of that course
  (`set_course_number`). With "for all courses" ticked it sets the number on the control (`set_control_number`).
  A placement the course already has comes before the one on the control, so a central drag on such a number changes
  nothing visible. The box snaps back and the status bar says why.
- **The server does the conversion** *(Claude's call)*. `GET api/number-at?course&index&x&y` answers
  `{ "direction", "gap_mm" }` for a box centred on the point (map mm). The page never repeats the geometry, so the
  stored value is the one the overlay draws again. The route takes no file names.
- **One inverse in the library** *(Claude's call)*. `numbers::placement_from_centre` inverts `number_centre`, next to
  it. The plan named `number_offset::placement_from_offset`; the library has no such function, and a second module
  for one inverse would split the placement code.
- **Rounding** *(Claude's call)*. The direction is normalised to [0, 360) and rounded to 0.01 degrees, the gap to
  0.01 mm. A drop that is not exactly west therefore stores 180.13, not 180.
- **`index`** *(Claude's call)*. It counts positions from 1, crossing points included, like `numbers[].index` in the
  overlay and `after` in the MCP tools. The mapping sits in `web/js/numbers.js` only.
- **The gap stays in the storable range** *(Claude's call, changed after the stage 3 review)*. The operations
  store a gap of more than 0 and at most 10 mm; a gap outside the rule range of the profile is still stored and
  left to `check`, but one outside 0 to 10 is refused. A box dropped over the circle has a negative gap, a far drop
  a huge one, and 0.004 rounds to 0.00. `api/number-at` therefore keeps the gap in 0.01 to 10 mm and adds
  `"clamped": true` when it moved it. The page says so in the status bar instead of showing the refusal. The
  editor shows no warning marker before the findings panel exists.
- **Place numbers** *(Claude's call)*. The button runs `place_numbers`. It leaves placed numbers alone unless
  "also replace placed numbers" sets `overwrite`.

## Consequences
- One drag is one operation and one undo step.
- A drag to "all courses" that a course placement shadows looks like a failure. The status text is the only hint.
- A change to the box size or to the circle geometry changes `number_centre` and `placement_from_centre` together,
  and their round-trip test catches a mismatch.

Source: browser editor plan M4, D14 (2026-11-11).
