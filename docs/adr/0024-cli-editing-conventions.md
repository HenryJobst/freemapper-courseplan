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
