# Changes

[Deutsche Version](CHANGELOG.md) · [User manual](docs/manual/en.md)

This file lists all notable changes to freemapper-courseplan, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- `check <PROJECT>` checks the individual courses against the rule profile of the event and its overlays. The
  findings are grouped by course, each with severity, place, explanation, rule id and source, followed by a
  summary. `--all` also lists the accepted findings with their notes and the skipped rules with the reason. A
  rule whose filters (discipline, map kind, print scale) do not fit counts as skipped and is reported as such.
  Team, relay and score courses are not checked yet. The exit code is 0 without an error finding, 1 with one or
  when the check is refused, and 2 for a wrong call.
- `finding accept <PROJECT> <RULE> --note TEXT` accepts a finding as an intended exception. Give its place with
  `--controls 31,32`, `--leg A:3` or `--course A`, or none of them for a finding of the whole event. Accepting
  the same finding again replaces the note. `finding remove` takes it back and refuses an accepted finding that
  does not exist.
- `event profile <PROJECT> <BASE> [--overlay ID …]` sets the rule profile and the overlays after loading them.
  `info` shows both.
- MCP: the tools `check`, `accept_finding`, `remove_accepted_finding` and `set_rule_profile`. `check` returns
  findings, accepted findings and skipped rules as JSON, with machine fields and the text in the server's
  language. `event_info` names the rule profile and the overlays.
- `check` now measures distances on the map: controls closer together than the minimum distance (the start and
  crossing points count), controls at similar features too close together, controls, start and finish near the
  map edge, legs that pass through another control of the same course, and straight legs through an out-of-bounds
  area, as a warning that the best route may lead through it. Taped legs may lead through out-of-bounds areas.
  Buildings, walls, fences and water wait for the later route analysis. Each finding gives the measured value and
  the limit, such as "control 31 and control 32 are 12 m apart, at least 15 m". A pair or a control is reported
  once, even when several courses use it. Start and finish can be the place of a finding: `finding accept …
  --controls 31,S1`.
- `finding accept` refuses a place where the rule never reports a finding, such as `--course A` for the minimum
  distance, which always concerns two controls.
- `check` now looks at the shape of legs. Where the incoming and the outgoing leg meet at a control or crossing
  point at an acute angle, it reports a warning below 45° and a hint below 90°, but only the stricter of the
  two. Two legs of one course that run in opposite directions at most 10 m apart for 30 m or more give a
  warning that names both legs. The first 10 m around a control where both meet do not count, also at the
  centre of a butterfly loop. Accept such a finding with both legs: `finding accept … --leg A:2 --leg A:5`.
- `check` now looks at control codes: a numeric code outside 31 to 599 is an error and a code that is no number
  a warning; both codes of a forbidden pair (66 and 99, 68 and 89, 86 and 98) used anywhere in the event are an
  error, also in different courses; two consecutive controls whose codes look alike upside down or differ only
  by 6 and 9 give a warning. Crossing points, start and finish have no code and are left out, and a crossing
  point between two controls does not break their pair. A control at a point symbol that occurs again within
  its circle gives a warning when column C of the control description is empty; lines and areas are not
  compared. Each code or pair is reported once, even when several courses use it.
- `check` now looks at the start. Courses use the same start minute only at the same start. For each start, the
  expected starters of the classes on its courses, divided by the start window, give the number of different
  first controls needed per minute; if the courses have fewer, a warning at the start names the first controls
  shared by several courses. Without a start window the rule is skipped, and a start with a class that has no
  expected starters is skipped with the class names. As a hint, two courses at one start with different first
  controls whose first legs differ in direction by less than 30° are reported once per pair of first controls.
  The direction runs to the first start bend, or else to the first stop.
- `check` now looks at the course tables. At long, middle and ultralong distances, a course whose length is
  outside the range of its category row, as a percentage of the HE course, gives a warning (WKB Anlage 5);
  without an HE course, or with one of length 0, the rule is skipped, and with two the first serves as the
  reference and a hint says so. A course with easy and hard categories (levels 1 and 2 against 3 to 5) is an
  error that names both sides. With the NOR overlay, the performance kilometres (length plus climb ×
  10 / 1000) outside the target of the category row ±6 % give a warning; a course without climb is skipped,
  and a category in no row gives a hint. A course whose categories hit several rows gets one finding that
  names every violated row. A class without rule category skips the course and is named.

## [0.1.0] - 2026-11-09

### Added

- First example: a sprint course in Lübeck's old town in `examples/luebeck-sprint`, with project file, commands
  and a sketch that the README shows too.
- MCP server: `freemapper-courseplan mcp` serves course setting to AI assistants over MCP (stdin and stdout).
  25 tools read and change a project: `event_info`, `list_controls|courses|specials|classes`, `add_control`,
  `move_control`, `remove_control`, `describe_control`, `set_control_number`, `add_special`, `remove_special`,
  `set_special_courses`, `add_class`, `remove_class`, `add_course`, `remove_course`, `set_course_controls`,
  `set_leg`, `set_course_details`, `place_numbers`, `set_course_number`, `map_features_near`, `map_extent` and
  `init_event`. Positions are objects with `x` and `y` in map millimetres. A control code may be a string or a
  whole number. `set_course_controls` and `remove_control` list the legs that lost their bends in
  `cleared_legs`. Each call loads the project file, applies one change and saves only on
  success. The server refuses relative paths. Descriptions, hints for the assistant and error texts come from the
  catalog in the language of `--lang`, `FREEMAPPER_LANG` or the system; tool and field names stay English. The
  manual covers this in chapter 3.
- Commands for courses: `course add`, `course controls`, `course leg`, `course set`, `course number`,
  `course numbers` and `course remove`. Adding and changing work for individual courses only. `course leg --after
  N` counts from 1 as on the map, crossing points included. Give bends with a repeated `--bend X,Y`; without
  `--bend` the leg loses its bends, and without `--taped` it loses the marked-route flag. When `course controls`
  or `control remove` makes a leg run to another stop, the leg loses its bends and its taped flag, and the
  command prints a line for it, so old bends cannot change the length unseen. `course number --after N` sets or
  clears the number placement of one control in a course. `course numbers` places the other control numbers
  clear of legs and circles; it runs only when you call it, and it keeps central and course numbers. Number
  directions are brought into 0 to 360 degrees, and a gap must be more than 0 and at most 10 print mm. Output
  uses the decimal comma in German, while input keeps the dot.
- `info` prints one line per course with the number of controls, the length in kilometres and the climb. Crossing
  points do not count as controls, here and in the line with the totals, which counts them apart. The length follows WKB B 7.2.5: straight lines, detours as bends, marked routes
  along their bends. Team, relay and score courses show no length yet.
- Commands for controls, special objects and classes: `control add|move|remove|describe|number`,
  `special add|courses|remove` and `class add|remove`. Each change is checked first and saved after; if the check
  fails, the file stays as it was. Points are in map millimetres with a dot as decimal separator. The program
  refuses points outside the map. `special courses <PROJECT> <ID> (A,B | --all)` assigns a special object to
  courses. Removing the only course of a special object is refused with a message that says what to do. Codes,
  ids, names and the items of comma-separated lists lose surrounding spaces, so `--controls "31, 32"` works.
  `help control add` shows the help of a subcommand. German help shows the possible values in German words, and
  an invalid value lists the valid ones.
- Library: course legs, course length and control numbers. The program computes legs with bends, the length of a
  course at a map scale and the numbers of the controls (crossing points get none). It places a number beside the
  circle without touching legs, circles or other numbers. Overprint sizes follow the map kind and stay the same size
  on the map for enlarged prints.
- Map index: the program looks up each map symbol once per symbol instead of once per object. This speeds up
  checking points against the map; results are unchanged.
- Data model: crossing points, ids for special objects, bends and marked-route flag for the leg from the start,
  and a print area per course.
- The project file refuses empty or space-padded control codes, special ids, course and class names, a control
  code that is also a special id, a course listed twice in one special object, lines with fewer than two points
  and areas with fewer than three.
- German program texts use the terms of the glossary: Pflichtstrecke, abgeflaggte Strecke, Querungsstelle,
  Gefahrenbereich, Erste-Hilfe-Stelle, Verpflegungsstelle, verbotene Route, Nummernlage, Strecke and
  Knickpunkt.
- Program texts come from Fluent files, one per language (`crates/freemapper-courseplan/i18n/`). German and
  English ship now. `--lang`, then `FREEMAPPER_LANG`, then the system language picks the language; help and
  messages follow it. Adding a language takes one new folder. Placeholders in the usage line and rare
  command-line parse errors stay English.
- Project file `*.courseplan.toml` with individual, team, relay and score courses. The program checks it on load
  and rejects unknown keys, so a typo shows up. When it saves the file, your comments and layout stay as they
  were.
- `freemapper-courseplan init <map>` starts a project from a sprint or schoolyard map. `--title` and `--project`
  set the title and the project file. Forest maps are refused for now.
- `freemapper-courseplan info <project>` prints the title, map, map kind, controls, courses by kind and classes.
- Reading OOM maps (`.omap`). The program refuses a map in the old binary format and tells you to save it again
  in OOM.
- Project skeleton modelled on freemapper-print: Rust workspace with a data model, a library and the
  `freemapper-courseplan` program, CI for Linux, macOS and Windows, release workflow.
- Draft design, glossary, the first architecture decisions and a collection of course-setting rules from German
  and international rule books.
