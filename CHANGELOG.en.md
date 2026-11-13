# Changes

[Deutsche Version](CHANGELOG.md) · [User manual](docs/manual/en.md)

This file lists all notable changes to freemapper-courseplan, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- The event has a date and an organizer (ADR 0090), requested by freemapper-print for its title block. `event set PROJECT --date YYYY-MM-DD --organizer TEXT` sets them, `--no-date` and `--no-organizer` remove them. The tool is `set_event_details`, the event settings of the editor have a "Date" and an "Organizer" field, and `info` and `event_info` show both. The project file gets the optional key `organizer` in `[event]` (format stays 1; a program from before this version refuses a file that has it). The IOF XML export writes a set date as `Event/StartTime/Date` and a set organizer as `Event/Organiser/Name`, and nothing for a field that is not set. Purple Pen has no field for either, so `.ppen` export, import and `sync` leave them alone.
- You can reject candidate sites (ADR 0089). `score reject --at X,Y` stores the position in the project file (new optional key `rejected_sites`, format stays 1), and `score suggest`, `score draft` and the candidate list of the editor leave out every site within 3 m of it. `score unreject --at X,Y` takes the rejection back. The tools are `reject_site` and `unreject_site`, the editor has a "Reject" button beside "Accept" in the candidate table. A program from before this version refuses a project file that has the new key.

### Changed

- The mapped area reaches 50 m beyond the map objects instead of 40 m (rule `wkb2026.map.inside`, `metres`, ADR 0088), the same number as the default margin. With the defaults, a control may stand at the outline of the objects; before it had to stand 10 m inside. Profiles that set `metres` themselves keep their value.

## [0.16.0] - 2026-11-13

### Added

- Three recommendations of the project owner, each with a setting in the `[score]` table (ADR 0085 to 0087). A manual course may deviate; `check` only warns, hints or reports, and `score suggest` and `score draft` keep to them.
  - Dead ends: `check` warns when a control stands deeper in a dead end than `dead_end_max_m` (default 60 m, 20 to 300; rule `wkb2026.control.dead_end`, sprint and schoolyard maps). The depth is the walk from the control to where the path network offers a choice again; open areas and courtyards are no corridors. A control beside a wall, fence or thin building is also judged from the free cells a few metres around it, and the deepest answer counts. The draft and the suggestion leave such sites out. On the Gleisdreieck map a quay spur of 208 m is the case; a control that stood 68 m deep on it set the default (ADR 0085).
  - Start and finish: `check` gives a hint when they are more than `start_finish_max_m` (default 400 m, 50 to 2000) apart in a straight line (rule `wkb2026.course.start_finish_distance`). `score draft`, and `score suggest --course`, refuse such a start and finish with a message that names both positions, the distance, the limit and the setting (ADR 0086).
  - Margin from the edge of the mapped area: `check` reports a start, finish, control or crossing point closer than `map_margin_m` (default 50 m, 0 to 200) to the edge as an error (rule `wkb2026.map.margin`). The edge is where no map object lies within 40 m any more; the distance is measured on a raster, to about 2 m. The draft leaves such sites out and refuses a start or finish that is too close (ADR 0087).
  - The settings come with `score settings --dead-end-max M --start-finish-max M --map-margin M`, the tool `set_score_settings` and three fields in the Score settings panel of the editor.
- `check` reports a start, finish, control or crossing point in the blank part of the sheet as an error. The new rule `wkb2026.map.inside` (source BL02, as the research file cites it) measures against the map objects themselves: a position counts as mapped when a map object lies within 40 m of it. The sheet frame (an axis-aligned rectangle line that spans 80 percent of the terrain) and layout symbols such as the north lines pattern (601, 602) do not count as objects. The message names the position, the nearest object and the limit. `map_edge` stays as it was; it measures against the box around the map objects and did not see a position in the blank corner of a diagonal map. Score courses are tested with start, controls and finish. See ADR 0084.
- The pictures of the examples: the examples page and both example READMEs show the course in the editor, with start and finish on the map. The Gleisdreieck picture is new (`docs/manual/images/gleisdreieck-course.jpg`).

### Changed

- The example "Gleisdreieck" is redrafted under the new limits: 32 minutes, 26 controls, 63 points, the very good runner inside the band and the good runner near the target (before: 34 minutes, 33 controls, 95 points). Control 54 of the old version stood at the tip of the quay spur, 208 m deep, and six more controls were 39 to 47 m from the edge of the mapped area. Project file, IOF XML, control list, sheets, transcripts, `check` output, READMEs, the manual and its four Gleisdreieck pictures follow; the files `score-34.*` became `score-32.*`. `check` is quiet apart from four accepted warnings (three alike code pairs and the share of hard controls).
- The score drafts on the Rummelsburger Bucht reach 11 minutes with both runners inside their bands and 12 near the target (before: 16 and 17), since sites keep 50 m from the edge of this narrow strip. Start, finish and the four controls of the example course already kept the margin and stay.
- The score draft keeps its sites off the sheet frame by the same test as the new rule, with the extent of the terrain objects instead of the extent of all objects. On the Rummelsburger Bucht the frame was not recognised before, so a draft could take sites at its corners. The largest draft with both runners in their bands on that map is now 16 minutes (17 near the target). See ADR 0084.
- The README of the Rummelsburger Bucht example and the manual (captions, section "Checking") follow the new course, and the manual pictures of the example are new.

### Fixed

- The example "Rummelsburger Bucht" had its finish in the blank part of the sheet, beside the map, and its start far from it. Start and finish now stand in the park in the middle, 34 m apart, and the course is a loop of four controls and 905 m (before: five controls, 1226 m). Project file, IOF XML, Purple Pen file, outputs, READMEs and all editor pictures of the example follow. `check` is clean. The example "Gleisdreieck" was checked: its start and finish (60 m apart) and all controls lie on the map.

## [0.15.0] - 2026-11-12

### Added

- `check` reports a control that stands in an out-of-bounds area as an error. The new rules `wkb2026.control.in_forbidden` (forest maps) and `wkb2026.control.in_forbidden_sprint` (sprint and schoolyard maps) cite WKB26 A 2.2. An area is a special object of the kind out-of-bounds area that belongs to the course of the control or names no course, and any map area with the symbol 520 or 709, on sprint and schoolyard maps also 714. A control on the edge stands in the area. The rule does not test start, finish, crossing points or controls that are on no course. See ADR 0081.
- The length bands of the score draft are targets with a tolerance. A runner is in the band, near the target (up to 3 percentage points outside, exactly 3 still counts) or misses the band. Near the target is a success: `score draft` and `score length` exit with 0 and name the deviation, and the JSON answers and the MCP tools carry `length.state`. Only a miss beyond the tolerance exits with 1 and is a finding of `check`. The draft still aims at the band; the tolerance only decides what it may return. See ADR 0083.
- The bands and the tolerance are settable: a new setting `band_tolerance_percent` (0 to 10, default 3), `score settings --good-band MIN-MAX --very-good-band MIN-MAX --band-tolerance N`, the parameters `good_band_percent`, `very_good_band_percent` and `band_tolerance_percent` of `set_score_settings`, and input fields in the Score panel of the editor, whose length report says "in the band", "near the target" or "missed". The very good band must not start below the good band.

### Changed

- `check` no longer tests the straight line between two controls against out-of-bounds areas. The warning "the straight leg crosses ..." is gone: WKB26 B 7.2.6 is about the running route, and a straight line over an out-of-bounds area is normal. The rules `wkb2026.leg.through_forbidden` and `wkb2026.leg.through_forbidden_sprint` stay in the rule profile and now appear in `check --all` among the rules that were not checked, until the route analysis exists. If you accepted a finding of these rules, the exception stays valid and shows at most among the acceptances that match no finding. See ADR 0080.
- The example "Gleisdreieck" is redrafted: 34 minutes, 33 controls, 95 points (before: 33 minutes, 27 controls, 83 points). Six controls of the old version stood in map areas with the symbol 520 (ADR 0082). The good runner is at 94.7 percent, near the target (ADR 0083); `check` shows four accepted warnings about alike codes. Files, outputs, pictures and the numbers in the READMEs and the manual follow.
- The example "Rummelsburger Bucht" now has five controls and only straight legs, with no bends and no marked leg. A leg is the straight line from control to control, and bends are the exception. IOF XML, the Purple Pen file, the outputs and both READMEs follow the new course.
- The screenshots in the manual show the new course. The picture for bends has bends for illustration only, and its caption says so. The findings pictures show three angle hints that came from moved controls in a working copy. The section "Legs and bends" now says that a leg is straight until you bend it.

### Fixed

- `score suggest` and `score draft` no longer propose control sites that lie inside or on the edge of a map area with the symbol 520, 709 or 714, nor on the edge of an out-of-bounds special object. Before, a draft on Gleisdreieck had six controls in "do not enter" areas, which `check` has reported as an error since ADR 0081. Draft and check use the same geometry and the same symbol list (ADR 0082).

## [0.14.1] - 2026-11-12

### Added

- Screenshots in the user manual, chapter 4. Fourteen pictures per language show the editor on the example maps by Remo Madella (Rummelsburger Bucht for the sprint course, Gleisdreieck for the score course): the whole page, a control, a course with its categories, a bent and taped leg, control numbers, control descriptions, a special object, findings, the Event menu, the Purple Pen export dialog, and the Score panel with candidates, a draft and the applied course. The German manual shows the German interface and the English manual the English one. Each caption names the map and points to the `ATTRIBUTION.txt` of the example.

### Fixed

- Editor: when you select a leg, "Length" now shows the length of that leg. It used to show the length of the whole course.
- Editor, Score panel: the candidate table fits the panel, and the Accept column is no longer cut off. The note "No free code" appears only when no candidate has a code. If only the rest of the list has none, the rows say so themselves.
- Editor: the dialog for a new score course is titled "New score course", not "New course".
- Editor: the status bar stays on one line. A long message ends in an ellipsis and shows in full as a tooltip, and the area above no longer jumps.
- Editor: the findings drawer covers the properties and map area completely. A strip of the properties pane used to show above it.
- `edit`: for undo, redo and quit the server now reads the request body before it answers. Otherwise a client on macOS could now and then see "Connection reset by peer" instead of the answer (ADR 0079).

## [0.14.0] - 2026-11-12

### Added

- Self-update (ADR 0076). `freemapper-courseplan update` replaces the program in place with the newest signed release, after asking (`--check` exits with 10 when a newer version exists, `--yes`, `--version X.Y.Z`, `--rollback`, `--mode notify|install|off`). The release is checked with minisign against a key built into the program, then by SHA-256 of the download and of the unpacked file, and the new file must answer `--version` before anything is swapped. The old file stays as `freemapper-courseplan.previous`. A release build looks for a newer version at most once a day, after `info` and when the editor starts, and prints one line on stderr; the editor shows it in its status bar. The MCP tools `check_update` and `install_update` (only with `confirm: true`) tell the assistant, which asks you first; the first tool result carries a note once. Builds from source never update themselves. The program sends only a fixed User-Agent to the GitHub release servers. The release workflow now signs a manifest with the secrets `MINISIGN_SECRET_KEY` and `MINISIGN_PASSWORD` of the environment `release`; the environment has a required reviewer, so a release asks for approval once, before it is signed and published. Unpacking by hand stays the way to reach the first release with the updater. User manual, chapter 2.
- Examples on two maps by Remo Madella (MOO Challenge Berlin 2025), who released them on 2026-11-12 for use within the freemapper suite; this is not an open licence (ADR 0077). `examples/gleisdreieck` holds a 33-minute score course (27 controls, 83 points; the good runner scores 92.8 percent, the very good runner finishes after 96.1 percent of the limit), and `examples/rummelsburger-bucht` a sprint course with four controls, bends, a marked leg, a first aid post and three accepted hints, with its IOF XML and Purple Pen export. Both come with transcripts and check output in English and German; each folder has an `ATTRIBUTION.txt`. 45 minutes does not work on either map: the READMEs give the tries and the real limits. The Purple Pen file of the new example has not been opened in Purple Pen.
- Wider pool of control sites for score courses (ADR 0073). `score suggest` now also uses the free ends of walls, fences, hedges, step lines, stairways and bridges, path ends and path junctions, bends and edges of paved ground, small buildings, canopies, water banks and more point symbols, each with a clarity that lowers the difficulty and never removes a site. A closed line gives all its corners. A site that cannot be routed no longer removes every site within 100 m, only those within 30 m. On the Lübeck map `suggest` finds 31 sites instead of 19; on a 1.8 x 2.0 km map of Berlin-Kreuzberg 183 instead of 137. Both maps hold mostly buildings, so the gain is modest; on the Kreuzberg map the draft ends at 13 controls for 30 minutes and 26 to 28 for 45, no longer bound by the pool but by the time of the very good runner.
- Four distance settings for score courses, in metres on the ground: `value1_min_m` (150) keeps every control away from start and finish, so value 1 starts there; `min_gap_m` (100, not lower) is the least distance between two controls and replaces the scale-derived default of `score suggest`; `max_reach_m` limits the distance from the start (automatic: a quarter of the ground the good runner covers in the time limit); `high_value_gap_m` (200) is the least distance between two high-valued controls, the values above 0.6 of the highest. `score settings` (`--value1-min`, `--min-gap`, `--max-reach`, `--auto-reach`, `--high-value-gap`), the tool `set_score_settings` and the editor's Score panel set them; older project files load with the defaults. The draft keeps the high-value gap, and `check` warns when a score course falls below either gap (`score_min_gap`, `score_high_gap`; ADR 0072).

### Fixed

- `score suggest` and `score draft` no longer take the corners of a map frame as sites (ADR 0078). The Gleisdreieck map has a closed rectangle of symbol 501.1 around its content, and a draft used its four corners as controls.
- `score draft` and `score length` (and `check`) now agree on a set. The same leg time came out a few digits apart depending on the search that made it, and the tour heuristic turned that into another tour: at the start 40,-40 of the Berlin example the draft promised 97 points and 99.8 percent, the applied course gave 96 points and 100.7 percent of the limit. Leg times are now rounded to a millisecond where they are made (ADR 0075).
- `score draft` finds a set that meets both bands where one exists, and runs faster (ADR 0074). After the prefix search it now searches locally: it swaps, adds or drops a control, or moves a value up to two steps off its quantile, for at most 150 tour evaluations with the seed of the settings. On a 1.8 x 2.0 km map of Berlin-Kreuzberg every start and limit we tried (30 and 45 minutes, three starts) now meets both bands; before, the good runner landed 0.3 to 3 points above his band every time. The values of a draft are therefore not always exact quantiles. A pool that is too small still fails: on the Lübeck map 30 and 45 minutes give the good runner every point.
- Faster route searches. The shortest-path search uses A*, one search per control reads its legs to all others, the route-choice searches of `score suggest` and `score draft` run on several threads, and the or-opt step of the tour heuristic no longer copies the tour. On the Kreuzberg map the CPU time of `score draft` fell from 44 to 63 s to 24 to 35 s; most of what is left is the route search of `score suggest` (ADR 0074).
- Score suggestions sit less at the rim of the map and closer to the start: the pick spaces sites more densely near the start and weighs down sites at the rim. On the Lübeck example the candidates within 48 m of the rim fell from 21 of 60 to 3, and the value-1 controls moved from 360 m to 230 m from the start (ADR 0071).

### Changed

- The documentation examples `examples/luebeck-sprint` and `examples/berlin-kreuzberg` are gone, replaced by the two examples above. The Lübeck project, its Purple Pen and IOF XML exports and its score project stay as test fixtures in `tests/fixtures/luebeck-sprint/`. The test `berlin_score_example_meets_its_bands_and_check_is_quiet` became `gleisdreieck_score_example_meets_its_bands_and_check_is_quiet`.

## [0.13.0] - 2026-11-11

### Added

- The Lübeck example has a score course in a second project file, `luebeck-score.courseplan.toml`: 9 controls, 25 points, 15 minutes, from `score draft`, with its transcript, check output, IOF XML, CSV list and evaluation sheet in English and German. A draft at 45 minutes fails because the free codes run out (30 controls at the highest value 5, 54 at 9), not because the map is small; the README says so.
- `score suggest` takes `--course NAME` (tool parameter `course`): start and finish of that score course. The editor's Suggest sends the selected course; before, it used the first start of the event, which can lie inside a building.
- A failed draft whose control count is capped by the free codes says so (hint "raise the highest value or free codes") instead of blaming the map.
- `info` names controls, points and time limit of a score course. It used to say that the length was not supported.
- Score courses in the exports. The IOF XML now holds score courses with the value of each control in `CourseControl/Score`; time limit, penalty and mandatory links sit in an `Extensions` element, because the schema has no place for them. New `export --csv FILE --course NAME` writes code, value and position of the controls, and `export --sheet FILE --course NAME` writes an evaluation sheet with the time limit, the penalty, the rule that the faster runner wins on equal points and the rule that a runner who skips a link is disqualified (tools `export_score_csv`, `export_score_sheet`). The Purple Pen export still skips score courses and says so. The IOF XML has not been tried in OE (ADR 0070).
- The editor draws score courses and has a Score panel: a suggestion with candidate markers and their values, "Accept" for a single candidate, a draft with a length report, "Apply" and a progress line while the program computes. Mandatory links appear as dashed lines between their controls. Candidates and drafts are not saved until you accept or apply them. A long run no longer stops the editor from noticing changes to the file (ADR 0069).
- `score suggest`, `score draft` and `score length` (tools `score_suggest`, `score_draft`, `score_length`): ranked control sites from the map, a draft of a score course whose length meets the bands of both runner profiles, and the length analysis of an existing course. Only `draft --apply` writes; it creates the course or replaces the controls of a score course, and the editor can undo it. A draft that misses a band exits with 1 and names each miss and a hint. Sites stay 4 m away from forbidden routes (ADR 0068).
- Editing score courses on the command line, over MCP and in the editor's operations: `course add --kind score --time-limit N`, `score controls`, `score value`, `score link add|remove`, `score settings` and `score course set`, with the tools `add_score_course`, `set_score_controls`, `set_control_value`, `add_score_link`, `remove_score_link`, `set_score_settings` and `set_score_course_details`. The code decides a control's value, so a different value is refused. Without a penalty a score course stores none and uses the default (the highest control value, at least `value_max`); `--no-penalty` goes back to it. Recoding a control gives it the value of its new code. `rename`, `remove`, classes and print scale work on score courses; legs, bends and numbers stay with individual courses (ADR 0067).
- Score courses: the project has a `[score]` table with the value range, runner profiles and seed, and a score course can name mandatory links between two controls. Loading a project refuses links to unknown controls, duplicate links and branching chains (ADR 0060).
- `check` now checks score courses: value in the code, codes, ring and order of the controls, length, time limit, penalty, mandatory links and mix. Team and relay courses stay skipped (ADR 0066).

### Changed

- Score fixes before the first release: a suggested control always has a route in the tours, a length report never says "inf percent", the speed table treats forest 405 as good running (1.1) and the railway helper 515.1 as no wall, and `score suggest` shows route choice and difficulty to two decimals and seconds to one. `score settings --value-max` refuses a value below that of an existing control.
- Score fixes before the first release: a suggested control always has a route in the tours, a length report never says "inf percent", the speed table treats forest 405 as good running (1.1) and the railway helper 515.1 as no wall, and `score suggest` shows route choice and difficulty to two decimals and seconds to one. `score settings --value-max` refuses a value below that of an existing control.
- The tool descriptions of `check` and `export_iof_xml` no longer say that score courses are not checked or written. README, manual, glossary, design and AGENTS.md describe score courses as built.
- Internal: the map kind, the sidecar reader and the object class now come from `freemapper-core` v0.19.0 (`freemapper-contract`) instead of local copies. Names and output stay the same.

## [0.12.0] - 2026-11-11

### Added

- `list controls|courses|specials|classes` list the details of a project as a table, and `map extent` and `map near --at X,Y --radius MM` report the map extent and the map objects near a point. With `--json` each prints the answer of the matching MCP tool unchanged (ADR 0059). `event start-window --none` removes the start window, as `set_start_window` with `no_start_window` does.

### Changed

- `AGENTS.md` puts the command line first and MCP second. An assistant that can run commands needs fewer turns; MCP stays for assistants without a shell, such as Claude Desktop. The command line section points to `--help`, names output and exit codes, says that `edit` blocks, and lists what the command line cannot do compared with the MCP tools.

## [0.11.1] - 2026-11-11

### Changed

- freemapper-core 0.16.1. The shared placement cases use the renderer's number box (0.8083747 x height x digits) and pass without exceptions.
- At scales the standard does not foresee (a sprint map at 1:5000, for example), the course symbols keep their base size, as in freemapper-print (ADR 0058). The overlay, the print preview, the placement and the checks measure with the same factor; circles used to shrink to 4.8 mm there. The check notes on such maps that the course is only fit for training.

## [0.11.0] - 2026-11-11

### Added

- The editor has a print preview (ADR 0057). The "Print preview" button or the P key shows the selected course the way freemapper-print draws it. The server hands the course to the renderer (freemapper-core 0.16.0), which paints start, controls, finish, legs, special objects and numbers as purple layers into the map image. The overlay is hidden and nothing can be edited meanwhile: the tree, the properties, undo and redo and the keys are switched off. A test compares the overlay's circles, legs, marked legs, crossing points and number boxes with the renderer's, for sprint, schoolyard and forest sizes.

### Fixed

- The number box is as wide as freemapper-print draws it (3.23 mm instead of 2.4 mm for one digit, ADR 0030). Stored number positions (direction and gap) are drawn about 0.4 mm differently, and the automatic placement, the checks and the positions in the Purple Pen and IOF XML exports use the new width. The example's Purple Pen file and the IOF XML are regenerated.
- The overlay's crossing point is the 3 mm symbol turned along the course, and a marked leg has the print's dashes. A map without a scale no longer gives infinite symbol sizes.
- The overlay draws start and finish at the IOF sizes and ends legs where the renderer ends them: at the start triangle's corner, at the outer finish circle, at the control circle, at the crossing point itself. Before, it cut every leg at both ends by the control circle's radius.

## [0.10.0] - 2026-11-11

### Added

- `AGENTS.md`, instructions for AI agents: what the program can and cannot do, installing it from the release archives, registering it over MCP (every tool with its parameters), the command line, troubleshooting. It is part of every release archive. The READMEs and chapter 3 of the manual explain how an assistant that can run commands installs and registers the program.
- The editor: `freemapper-courseplan edit <project>` shows the event in the browser (ADR 0042, 0045, 0046). On the left is the tree of courses, categories, controls and special objects. In the centre the view shows the selected course with circles, legs and numbers over the map; on the right are the properties. The status bar is at the bottom.

- The editor can place, move and delete controls, and undo and redo each step (ADR 0043, 0047). Click with the "Place control" tool to place one, drag a circle or change X and Y in the properties (a comma or a dot works) to move it, press Delete or Backspace to remove it. Arrow keys move the selected control by 0.1 mm, with Shift by 1 mm. Every change is saved at once through the same operations as the CLI and the MCP tools, so the checks and messages are the same. If the project file changed outside the editor, the editor refuses the change, shows a message and loads the current state. When a course visits the control, deleting it asks first. Ctrl/Cmd+Z undoes, Ctrl/Cmd+Shift+Z or Ctrl+Y redoes, and `?` lists all shortcuts. The undo history holds up to 100 steps and at most 64 MiB of file copies. If another program wrote the file, the next undo or redo refuses and clears the history. Undo and redo also refuse when the page shows an older state than the file, for example in a second tab.
- The editor lists the map objects near a selected control in the properties.
- The editor sets control descriptions (ADR 0051). The properties of a control list the columns C to H, each folded and showing its current symbol. Open a column to pick a symbol from a grid of icons with numbers and texts, by mouse, arrow keys or by typing a number such as `1.3`. Direction symbols are offered with their direction (`12.1N`), column E also offers the symbols of column D, and "No symbol" empties a column. Each pick is one step for undo. The number symbols 9.1 to 9.4 are not offered, because the project file holds no value for them. Setting a description now refuses a symbol that does not fit its column, on the command line, in the MCP tools and in the editor alike; symbols already in a file stay as they are.
- The editor builds courses, categories and event settings (ADR 0048, 0049). "New course..." makes a course with the event's start and finish. In course mode a click on a control appends it, and a click on a leg and then on a control inserts it there. The stop list reorders with Alt+Up and Alt+Down. Climb, planned print scale, categories, rule category, expected starters, rule profile with overlays and start window are fields in the properties.
- Legs get bends and tape in the editor: drag a leg's line to add a bend, drag a bend to move it, press Delete to remove it, and tick "Taped" for a taped leg. The course length follows each change.
- Control numbers can be dragged to a new place. The editor stores direction and gap for the course, or on the control when "for all courses" is ticked. "Place numbers" places the automatic ones and leaves set numbers alone, unless "also replace placed numbers" is ticked. The new route `api/number-at` turns a drop point into direction and gap.
- Special objects can be drawn in the editor (start, finish, first aid, water, marked route, forbidden route, out-of-bounds area, dangerous area), assigned to courses and deleted. Lines and areas end with Enter or a double click. The editor has no move for special objects; delete and redraw.
- A control gets a new code and a course a new name: `control code <project> <from> <to>`, `course rename <project> <from> <to>`, the MCP tools `set_control_code` and `rename_course`, and the "Code" and "Name" fields in the editor's properties (ADR 0024). Courses, special objects and accepted findings that name the control or course follow. A code or name that is taken, empty, padded with spaces or too long is refused and the file stays as it was.
- Control codes may have at most 64 characters, course and class names and the notes of accepted findings at most 200. Longer texts are refused with a message, in the editor, on the command line and by the MCP tools alike; existing project files stay valid.
- The editor shows the findings of the check (ADR 0052). Half a second after each change it runs the same check as `check`. The status bar counts errors, warnings and hints; a drawer lists open and accepted findings and the rules that did not run. The properties list the findings of the selection, and the map marks them with a ring or a dashed line and a sign per severity. A gap outside the rule range gets a warning sign at the number box. "Accept..." stores an exception with a note of up to 200 characters; "Remove acceptance" brings the finding back. Each is one step for undo.
- The Event menu exports and imports (ADR 0053). "Export IOF XML" downloads the file, "Export to Purple Pen" writes the linked `.ppen` or downloads a new one, and "Import Purple Pen file" reads an upload. The browser names no file; only the file `sources.ppen` names is written. Importing into a project that already has content needs "Replace". Writing to the linked file and importing clear the undo history. An empty event now shows correctly in the page.
- The Event menu syncs with Purple Pen (ADR 0054). A warning asks you to close the event in Purple Pen first; it can be silenced for the session. The panel lists the changes and the conflicts with the values from the base copy, courseplan and Purple Pen. You choose a side for each conflict, or "All courseplan" or "All Purple Pen" to sync at once. A sync that wrote files clears the undo history; if Purple Pen or another program saves a file during the sync, nothing is written and the editor loads the current state.
- The editor shows the real map (ADR 0056, 0046). The shared renderer freemapper-render draws it as SVG, and the course overlay lies on top. The page asks for the visible section 150 ms after the view stops moving; the server keeps the page per file state and the last section per view. A map larger than one A3 sheet at map scale is drawn clipped to the middle part, and the status bar says so. If the renderer leaves out or shortens objects, the status bar says the map area is incomplete. A section too large to draw is refused with a message, and the overlay stays. Not built yet: the print preview and the test that compares the overlay with the print overprint; both wait for freemapper-print's course layer.

### Changed

- Saving a project file now refuses when the file changed since the command read it (CLI, MCP tools and editor alike). Nothing is written, and the CLI exits with code 1 and prints `<file> changed since it was read; nothing was written`. Run the command again. This closes the lost-update limit of the sync (ADR 0044, 0040).
- The shared building blocks come from freemapper-core v0.14.1. The editor draws the map with `freemapper-render` and uses its list of description symbol ids (`description_symbols::ids`) and the drawings and texts of the symbols (`render`); the rest of the program does not change. The lock file pins `crc32fast` 1.5.3 and `flate2` 1.1.10 because newer releases were younger than two weeks (ADR 0006, 0056).
- Tests: every test run now reads four maps from freemapper (schoolyard 1:1000, sprint 1:4000, a map from freemapper 0.9.0 and one with an unknown map kind) and compares map kind, title, sheet objects, base area and map extent with the expected values from `freemapper-contract` (ADR 0041). If freemapper changes a tag or a key of `freemapper.toml`, these tests fail.

### Fixed

- The source check (clippy) failed on Windows because an error type of `sync` was too large there. The program behaves as before.

## [0.9.0] - 2026-11-10

### Added

- `sync` merges the project with its linked `.ppen` in both directions (ADR 0040). It compares, field by field, the base copy `X.courseplan.ppen-base` (belongs in git; `sync` removes the map's `absolute-path` from it), the file now and the project now. If both sides changed the same field differently, or one side deletes what the other edited, `sync` lists both values and writes nothing. `--take ours`, `--take theirs` or `--take KEY=SIDE` pick a side, `--dry-run` writes nothing, and `--json` prints the result for machines. The next run finishes an interrupted sync. The sync refuses files from an unknown Purple Pen version. Classes, accepted findings and team, relay and score courses stay in the project.
- New MCP tools `sync_ppen` and `resolve_sync_conflict`.
- The project file has an optional `[sync]` table with the link, a hash of the base copy and stored decisions. The format stays 1.

## [0.8.0] - 2026-11-10

### Added

- The IOF XML export writes latitude and longitude (`Position`, six decimals) for start, controls and finish when the map is georeferenced (ADR 0039). Without usable georeferencing the file keeps map positions only, and `export` says so. The MCP tool `export_iof_xml` also returns `geo_positions`.

## [0.7.5] - 2026-11-10

### Changed

- freemapper-core 0.6.1: automatically placed numbers store their direction in clean tenths of a degree again
  (ADR 0036). The rounding workaround in courseplan is gone.

## [0.7.4] - 2026-11-10

### Fixed

- The Purple Pen export writes the variant `single` into the right box. The same feature in columns D and E over
  a crossing or junction in column F now comes out as `single` in column F.
- The export leaves out description symbols that the standard of the file does not have, such as 2.10 in a 2004
  file, and names each symbol it left out (ADR 0037).
- The export normalises the directions `0.1` and `0.2`. It writes plain 9.1 and 9.3 as `9.1high` and `9.3high`
  and says so in a note.

### Changed

- Number placement and the description symbol tables now come from freemapper-core 0.6.0, the same code
  freemapper-print uses. Positions change only in two edge cases: a bend closer to the circle than one line
  width, and a rule minimum with three decimals. In the `luebeck-sprint` example no number moves (ADR 0036,
  ADR 0037).

## [0.7.3] - 2026-11-10

### Fixed

- The scale bar and the base area that freemapper 0.9.0 writes into the map (`freemapper:source=scale-bar` and
  `base`) no longer count as map objects and no longer move the map edge. The `map_edge` check finds controls at
  the south edge again.

### Changed

- The map kind comes first from `map_kind` in the `freemapper.toml` next to the map, then from the symbol set
  name (case does not matter) and the scale, as freemapper-print decides it. An unknown value or a broken file
  is skipped, not reported (ADR 0038).

## [0.7.2] - 2026-11-10

### Fixed

- Purple Pen 4.0 crashed while opening a `.ppen` written by `export --ppen`: a finish without a description symbol
  broke the description view. The export now gives every finish, crossing point and map issue point Purple Pen's
  own symbol (14.3, 13.3, 13.6), and an export onto an older file repairs the missing ones. freemapper-core 0.4.2
  checks the output against the loader's requirements in more depth. The example `luebeck-sprint.ppen` is
  regenerated.

## [0.7.1] - 2026-11-10

### Fixed

- `export --ppen` wrote courses that Purple Pen 4.0 refused to open: `<options>` lacked the required attribute
  `description-kind`. Files written by 0.7.0 load again after the next export onto them. freemapper-core 0.4.1
  checks everything the writer produces against the attributes Purple Pen's loader requires. The example
  `luebeck-sprint.ppen` is regenerated.

## [0.7.0] - 2026-11-10

### Added

- `freemapper-courseplan export <project> --ppen <file> [--new]` writes the individual courses as a Purple Pen
  event. It changes an existing file in place and leaves alone whatever the project does not know (gaps in circles
  and legs, map exchange, column F text, course length); `--new` writes the file from scratch. The export never
  deletes: what is in the file but not in the project stays and is reported as kept. What it writes differently or
  not at all (team, relay and score courses, symbols with no Purple Pen counterpart, partial flagging after
  changed bends) is named in a note. A newly created file is recorded in the project as `sources.ppen` when none
  is set yet; for that file the Purple Pen ids are stored in the project. The AI assistant has the tool
  `export_ppen`. The writer comes from freemapper-core v0.4.0 (`freemapper-course`). **Not yet opened in
  Purple Pen 4.0** (ADR 0035).
- The example [`luebeck-sprint.ppen`](examples/luebeck-sprint/luebeck-sprint.ppen); a test compares it with a
  fresh export, another reads it back and exports onto it.
- A test reads the IOF XML export (the Lübeck example and an event with a crossing point) back through the IOF
  reader of freemapper-course and compares controls, positions, courses, order of stops and number positions.

### Changed

- The documentation now states what is verified: the export passes the IOF schema and reads back. An import into
  OE, Purple Pen or Condes has not been tested, because nobody on the project has OE (ADR 0033, addendum).

## [0.6.0] - 2026-11-09

### Added

- `freemapper-courseplan init <map> --ppen <file>` and `import <project> --ppen <file> [--replace]` take over a
  Purple Pen event: controls, descriptions (renumbered to the 2024 standard), individual courses, bends, marked
  routes, number positions and special objects. The AI assistant has the tool `import_ppen`. What the model cannot
  hold is named in a note. Without `--replace`, `import` refuses to overwrite a project that has content (ADR 0034).
- Controls and courses remember their Purple Pen id (`ppen_id`) for the later sync.

### Changed

- The map is now read by `freemapper-omap` and the `.ppen` by `freemapper-course` from freemapper-core 0.3.0,
  instead of a local copy. `freemapper-text` and `freemapper-rules` are on the same tag.
- A forbidden route may consist of one point, as Purple Pen stores it.

## [0.5.0] - 2026-11-09

### Added

- `freemapper-courseplan export <project> --iof-xml <file>` writes the individual courses as IOF XML 3.0
  (`CourseData`) for scoring software. The AI assistant has the tool `export_iof_xml` for it. Team, relay and
  score courses are skipped and named. Geo positions and taped legs are missing for now (ADR 0033). The output is
  checked against the IOF schema; an import into OE is still to be tested. An example is in
  `examples/luebeck-sprint/course-data.xml`.

### Changed

- Program texts now come from `freemapper-text` (freemapper-core 0.1.0) instead of a copy of the code in this
  repository. Nothing changes for users.
- `freemapper-rules` (freemapper-core 0.2.0) now reads the rule profiles. The loader also checks value ranges by
  unit; the embedded profiles are unchanged.

## [0.4.0] - 2026-11-09

### Added

- `check` warns when the gap between a control number and its circle is outside 0.5 to 5 mm
  (`wkb2026.number.gap`, values `min_gap_mm` and `max_gap_mm`). No standard gives a value; the user set the
  limits (ADR 0032). Stored numbers stay where they are; old gaps above 5 mm are now reported.

### Changed

- Automatic placement tries gaps from the minimum of the rule profile upward in steps of 0.5 mm, up to the
  maximum. A larger gap in the preferred direction beats another direction at a smaller gap. Without an
  applicable rule it stays at 0.5 mm. Newly placed numbers can therefore stand farther out.

## [0.3.0] - 2026-11-09

### Added

- `check` hints at almost straight angles at controls: when the legs at a control form an angle above 170°,
  it gives a hint (`wkb2026.leg.straight_angle_hint`). Crossing points are left out.

### Changed

- Automatic placement puts a control number behind the circle, in line with the leg that leads to the control.
  When the next leg runs along that line (an almost straight angle), the number goes outside, at the tip of the
  angle. The number covers no point or line symbol of the map as long as a free place is left; it may cover
  areas and magnetic north lines (symbol 601). Legs, circles and other numbers weigh more than map symbols. If
  the preferred place is taken, placement picks the free place that deviates least, in steps of 15°; the
  direction is rounded to 0.1°. Newly placed numbers therefore often stand differently than before (ADR 0031). freemapper-print builds the
  same rules into its shared placement module.
- The gap of a control number (`--gap`, `gap_mm`) now starts at the outer edge of the circle line and means
  the shortest distance to the number box, in every direction. It used to start at the centre of the circle
  line, and on the diagonals the number stood farther away than given. freemapper-print computes it the same
  way from its M3 on. Stored number placements stay valid. Straight above, below or beside the circle a number
  moves 0.175 mm outwards; on the diagonals it moves up to 0.6 mm closer.

## [0.2.0] - 2026-11-09

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
  the limit, such as "control 31 and control 32 are 12 m apart, at least 15 m required". A pair or a control is
  reported once, even when several courses use it. Start and finish can be the place of a finding: `finding
  accept … --controls 31,S1`.
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
  10 / 1000) outside the target of the category row ±6 % give a warning, shown with up to two decimals; a
  course without climb is skipped. A category in no row of the table skips the course for that category, in
  the length table and in the NOR table alike. A course whose categories hit several rows gets one finding
  that names every violated row. A class without rule category skips the course and is named.
- User manual, chapter 8 "Checking: reading findings, giving reasons for exceptions": how to read a line of
  `check`, what severity, exit code and skipped rules mean, how to accept and take back exceptions, and how to
  choose the profile and overlays. The Lübeck example shows the output of `check` and one accepted hint in
  `check.en.txt` and `check.de.txt`; the project file holds it.
- `event start-window <PROJECT> <MINUTES>` sets the start window that the start capacity check needs; 0 is
  refused. `class set <PROJECT> <CLASS> [--rule-category C] [--starters N]` changes the rule category and the
  expected starters of a class, and `--no-rule-category` and `--no-starters` remove them. Over MCP the tools are
  `set_start_window` and `set_class`. When one of these values is missing, `check --all` names the command that
  sets it in the skipped rule.
- `check --all` lists under "not matched" the accepted findings that match no finding any more, for example
  because a control or a course was removed, with their notes and a pointer to `finding remove`. The summary
  counts them; they do not change the exit code. MCP `check` returns them in `unmatched`.
- A rule that an overlay switches off is listed among the skipped rules by `check --all`, with a reason that
  names the overlay: "rule profile nor-bb-2021 switches the rule off".
- `check --all` prints the note of each finding's rule in the chosen language below it. A value the program
  proposes because no source gives a number names its source as "proposal of freemapper-courseplan".

### Changed

- Output names a leg by its ends: "leg 36–37", "leg start–31", "leg 37–finish", and two legs as "legs 31–32
  and 35–36". As places, start and finish read "start S1" and "finish F1". Within a course the course comes
  first, then its legs in running order. Input stays `--leg A:6`, the leg from position 6 of the course with
  crossing points counted; the help says so.
- An accepted finding on a leg stores the leg's ends (`from`, `to`). When the leg later runs between other
  controls, `check` reports the finding again. `finding remove --leg A:6` removes the exception for the leg as it
  runs now; if there is none, it removes the older ones at that position.
- Texts: limits read "at least 15 m required", and a leg over a control "the leg runs over control 31" instead of
  "passes 0 m from control 31". Start and table findings no longer repeat their place in the text. An empty
  note is refused with a message of its own.

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
