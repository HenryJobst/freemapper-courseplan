# 0024. CLI editing conventions

Date: 2026-11-08
Status: Accepted

## Context

Until the browser editor exists (M4), courses are set through the CLI and MCP. Coordinates are map
millimetres (ADR 0019), German output uses a decimal comma, and options with negative numbers clash with
clap's option parsing.

## Decision

- A point is written `X,Y` in map millimetres with a dot as the decimal mark, in every language.
- Several points or bends are given as a repeatable single-value option (`--point X,Y`, `--bend X,Y`), never
  as one option with many values, so a negative coordinate cannot swallow the next option.
- `course leg --after N` counts course positions from 1, as on the map; crossing points count as positions.
- Every command loads the project file, applies one all-or-nothing edit and saves; a refused edit leaves the
  file byte-identical. Refusals exit with 1, usage errors with 2.
- Output follows the language: German prints lengths with a decimal comma, input keeps the dot.
- Removing a course takes its name out of special objects; if that would leave a special object with no
  course (which means all courses), the removal is refused with its own message, and `special courses` reassigns
  or widens the object first (ruling R5, amended). A special object lists each course once.
- A leg keeps its bends and taped flag only while it runs to the same next stop. When `course controls` or
  `control remove` changes the next stop, the leg loses both, and the command prints one line per such leg
  (ruling R12).
- `course number --after N` counts like `course leg`. A course number beats the control's central number, and
  that beats automatic placement; `course numbers` fills only the rest and runs only when called (ruling R11).
- A number direction is brought into [0, 360); a gap must be more than 0 and at most 10 print mm.
- Codes, ids, names and list items lose surrounding white space; empty or padded names in the file are refused.
- M2a edits and measures individual courses only; team, relay and score courses get a clear refusal.

## Consequences

- Command lines are a little longer, and behave the same in every shell and locale.
- Cost if wrong: a later editor may want other conventions; the library operations do not depend on these.

Source: plan M2a Tasks 4, 7, 8; rulings R5 (amended), R8, R11, R12; final review of M2a, 2026-11-08.

## Amendment 2026-11-09: checking and accepted findings

- `check` exits with 0 when no finding of severity `error` is left after the accepted ones are taken out, with
  1 when one is left or when the check is refused (project file, map or rule profile cannot be loaded), and
  with 2 for usage errors. The help of `check` states this. Warnings and hints never change the exit code.
- `finding accept` and `finding remove` name a finding by rule id and place: `--controls 31,32`, one
  `--leg COURSE:N` (the leg after position N, 0 the leg from the start, as in `course leg`) or `--course NAME`.
  Without any of them the finding is one of the whole event. A refused acceptance or removal exits with 1 and
  leaves the file byte-identical, like every other edit.
- The rule profile is set with `event profile <PROJECT> <BASE> [--overlay ID …]`; there is no `project set`.

Source: plan M2b Task 3; controller rulings for Task 3, 2026-11-09.

## Amendment 2026-11-09: start window and class settings

- `event start-window <PROJECT> <MINUTES>` sets the start window; 0 is refused, since the start capacity check
  reads 0 as no window. `class set <PROJECT> <CLASS> [--rule-category C] [--no-rule-category] [--starters N]
  [--no-starters]` changes a class and needs at least one option. A `--no-…` flag clears a value, as
  `--no-climb` does for `course set`, and an empty category is refused. The option is called
  `--rule-category` as in `class add`. MCP has the same as `set_start_window` and `set_class`
  (`no_rule_category`, `no_expected_starters`), with the checks of ADR 0023 (ruling R29).
- The skip reasons for a missing start window, missing expected starters and a class without rule category
  name these commands.
- In `finding accept --leg COURSE:N`, N is the position the leg leaves in the course's list of controls,
  counted from 1 with crossing points included, and 0 is the start. It is not the number printed on the map,
  and the help says so.

Source: final review of plan M2b; controller ruling R29, 2026-11-09.

## Amendment 2026-11-11: change a code, rename a course

- `control code <PROJECT> <FROM> <TO>` and `course rename <PROJECT> <FROM> <TO>` change the code of a control and
  the name of a course. MCP has the same as `set_control_code` and `rename_course` (`from`, `to`), and the editor
  runs both as operations. The library functions `change_code` and `change_course_name` check first and then call
  `rename_control` and `rename_course`, which the Purple Pen sync already used. So courses of every kind, the
  course lists of special objects, accepted findings and their legs, and the stored sync decisions follow the new
  name (ADR 0050).
- A code or name that another control or course has is refused (`CodeTaken`, `NameTaken`). A new value that is
  empty, padded with white space or longer than the project limits (64 characters for a code, 200 for a name) is
  refused with the localized validation errors. Naming the current value is no change. Each refusal leaves the
  file byte-identical.

Source: plan M4 Task 10, 2026-11-11.
