# 0067. Score courses: editing operations

Date: 2026-11-11
Status: Accepted

## Context
Score courses load (ADR 0060) and `check` tests them (ADR 0066), but nothing edits them. ADR 0059 applies.

## Decision
- Seven operations in `edit/score.rs`: `add_score_course`, `set_score_controls`, `set_control_value`,
  `add_score_link`, `remove_score_link`, `set_score_settings`, `set_score_course_details`. The CLI calls them through
  `call_json`. The editor runs them as operations, so its whole-file snapshots give undo *(Claude's call)*.
- The code carries the value *(user, ADR 0060)*. A different value is refused, in `set_score_controls` and in
  `set_control_value`. The latter accepts only the value of the code (it repairs a drifted file), or any value from 1
  to the event maximum for a code that carries none, such as `A1`. To change a value, change the code, and
  `rename_control` gives the control the value of its new code. A new code without a value keeps the old value, and
  `check` reports the mismatch *(Claude's call)*.
- `Score::penalty_per_min` is optional. `None` means the default, the highest control value of the course and at
  least `value_max` (design §5.4), worked out on read by `default_penalty`. A stored number loads as before.
  `add_score_course` without a penalty stores none; `set_score_course_details` stores one or, with `no_penalty`, goes
  back to `None`; `set_score_controls` leaves it alone. Answers give the resolved value and `penalty_is_default`
  *(Claude's call)*.
- The model validation checks links (ADR 0060). A control that a link names cannot leave the course until the setter
  removes the link; a recoded control keeps its links *(Claude's call)*.
- Team and relay courses are refused as before. A score course gets its own refusal for legs, bends, numbers and
  `set_course_controls`. Details, rename and remove accept it. `place_numbers` stays individual-only until the number
  label of the main design exists *(Claude's call)*.

## Consequences
`list_courses` shows a score course in a `score` object. A test per operation compares CLI `--json` with MCP.
