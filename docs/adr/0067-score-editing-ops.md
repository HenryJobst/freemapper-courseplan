# 0067. Score courses: editing operations

Date: 2026-11-11
Status: Accepted

## Context
Score courses round-trip through the project file (ADR 0060) and `check` tests them (ADR 0066), but nothing edited
them. The operations have to follow ADR 0059: one function per tool, used by the CLI, MCP and the editor.

## Decision
- Seven operations: `add_score_course`, `set_score_controls`, `set_control_value`, `add_score_link`,
  `remove_score_link`, `set_score_settings` and `set_score_course_details`. They live in `edit/score.rs`, and the
  CLI commands call them through `call_json` like the read commands. The editor runs them as operations, so its
  whole-file snapshots give undo *(Claude's call)*.
- The code carries the value *(user, ADR 0060)*. A value that differs from the code is refused, in
  `set_score_controls` and in `set_control_value`. To give a control another value, change its code. So
  `set_control_value` only accepts the value of the code (it repairs a drifted file), or any value from 1 to the
  event maximum for a code that carries none, such as `A1` *(Claude's call)*.
- The penalty default is the highest control value of the course, at least `value_max` (design §5.4), stored in the
  file. `set_score_controls` recomputes it only while the stored penalty still equals the default of the old
  controls, so a penalty the setter chose stays. The file has no flag for this, so a chosen penalty that equals the
  default is recomputed too *(Claude's call)*.
- Links are checked by the model validation, which already knows the rules (ADR 0060). A control named by a link
  cannot be dropped from the course; the setter removes the link first. A recoded control keeps its links
  *(Claude's call)*.
- The guards stay. Team and relay courses are refused as before. A score course gets its own refusal for legs,
  bends, numbers and `set_course_controls`, with a pointer to the score tools. `set_course_details` (classes, climb,
  print scale), `rename_course` and `remove_course` accept score courses. `place_numbers` stays individual-only:
  numbers on score courses come with the number label of the main design, a later feature *(Claude's call)*.
- The Fluent texts of every `ScoreError` variant exist in both languages now, although the suggestion tools that
  raise most of them follow in the next task.

## Consequences
- `list_courses` shows a score course's controls, values, limit, penalty and links in a `score` object.
- CLI and MCP answers are equal by construction; a test per operation compares them.
