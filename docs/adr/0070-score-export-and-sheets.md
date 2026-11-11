# 0070. Score courses: export, CSV and evaluation sheet

Date: 2026-11-11
Status: Accepted

## Context
Design §3 and §5.3 to §5.6 ask for score courses in the IOF XML, a CSV for hand evaluation and a printable
evaluation sheet. ADR 0033 limited the IOF XML export to individual courses. The IOF schema has `CourseControl/Score`
(a double, after `LegLength`). It has no flag for a score course, no time limit, no penalty and no link.
## Decision
- **IOF XML** *(user: write score courses; Claude's call: carriage)*. A score course is a `Course` with a
  `CourseControl` for start, each control with `Score` (the value) and finish. It has no `Length` and no leg lengths,
  because the order is free. Time limit, penalty and links go into the course's `Extensions`, as
  `fcp:ScoreCourse timeLimitMin penaltyPerMin` with one `fcp:Link a b` per pair, namespace
  `urn:freemapper-courseplan:score:1`. The schema allows foreign elements there, other programs ignore them, and the
  output still validates. The penalty is the resolved one, default included, so a reader needs no rule. Links are not
  a `CourseControl` `Type`: the enumeration has no fitting value and a wrong one would mislead result software.
- **Tie rule** *(user)*. "Equal points: the faster runner wins" and the disqualification for a skipped link stand on
  the sheet. The file carries the links, not the tie rule, which the schema cannot hold.
- **CSV** *(Claude's call)*. `code,value,x_mm,y_mm`, header row, ordered by code (numbers by value), positions in map
  millimetres at map scale with y down, two decimals with a dot, `\n`, UTF-8.
- **Sheet** *(Claude's call)*. Markdown, because it reads as plain text and prints through any viewer. It names course,
  event, limit, penalty (and whether it is the default), the tie rule, the points in all, the links with the rule, and
  one row per control. English and German through Fluent. No empty columns for results.
- **CLI and MCP** *(Claude's call)*. `export --csv FILE` and `--sheet FILE` with `--course NAME`; tools
  `export_score_csv` and `export_score_sheet` with `project`, `course`, `path`. Both call `export_score_file` (ADR 0059).
  A course that is missing or not a score course is refused with one sentence. An existing file is replaced
  atomically, like the other exports. The editor menu does not offer them yet.
- **Purple Pen** *(Claude's call, design §5.5)*. The `.ppen` export and sync still skip score courses and say so.
## Consequences
- The IOF XML export is as untried in OE as before; AGENTS.md and the changelog say so.
- Individual output is byte for byte unchanged; the snapshot test passes without edits.
