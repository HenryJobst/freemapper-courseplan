# freemapper-courseplan user manual

[Deutsch](de.md) · [README](../../README.en.md)

This manual takes shape along with the first usable version. It will lead step by step from the finished OOM map
to finished courses that OE evaluates and freemapper-print prints, even without command-line experience.

Planned chapters:

1. What freemapper-courseplan does
2. Installation
3. With an AI assistant
4. Without an assistant: editor and command line
5. Setting up an event: map, discipline, classes
6. Placing and describing controls
7. Setting courses: order, bends, control numbers
8. Checking: reading findings, giving reasons for exceptions
9. Exchanging with Purple Pen
10. Handing on: IOF XML for OE, printing with freemapper-print
11. Messages and what to do

## 3. With an AI assistant

The assistant sets the courses for you. For this, freemapper-courseplan is an "MCP server", a tool that AI
assistants can use. The program has no AI of its own. The assistant calls the same editing functions as the command
line, and each call saves into your project file.

### Set it up once

- **Claude Desktop:**
  1. Open *Settings > Developer > Edit Config*.
  2. Add this to the file `claude_desktop_config.json`, with your own path (on Windows with doubled backslashes
     `\\`):

     ```json
     {
       "mcpServers": {
         "freemapper-courseplan": {
           "command": "/Users/anna/Documents/freemapper-courseplan/freemapper-courseplan",
           "args": ["mcp"]
         }
       }
     }
     ```

  3. Restart Claude Desktop.
- **Claude Code:** type `claude mcp add freemapper-courseplan -- /path/to/freemapper-courseplan mcp` in a terminal.
- **Other assistants:** any assistant with MCP support takes the same entry: the program and the argument `mcp`.

The assistant reads the tool texts in the language of your system. To fix the language, write
`"--lang", "en"` before `"mcp"` in the arguments, or set `FREEMAPPER_LANG=en` in the environment of the
program. Tool names and field names stay English in every language.

### Set a course

The assistant does not see the map. It can ask for the extent of the map and, for a point, for the map objects
around it, each with its symbol number, its kind (point, line, area) and its distance. It knows no names such as
"south door" or "gym", and the query does not tell it where exactly an object lies. So give it positions, in map
millimetres or relative to the extent of the map, for example:

> Start an event from ~/maps/hall.omap. Put the start a quarter of the way in from the left edge, at half the
> height, and the finish 10 mm to the right of it. Add three controls spread over the map and tell me for each
> which map objects lie within 2 mm. Make a course "Short" from them for class D10, place the control numbers
> and tell me the length.

The assistant then works like this:

1. It starts the event. The project file lands next to the map, named `hall.courseplan.toml`.
2. It asks for the extent of the map, that is its smallest and largest corner in map millimetres, the scale and
   the map kind, and works out the positions from it.
3. It adds the start, the finish and the controls at these positions.
4. For each control it asks for the map objects nearby and tells you their symbol numbers and distances, such as
   "symbol 521, area, 0.4 mm". Look up what a symbol means in the symbol set of the map.
5. It adds the course, places the control numbers and reads the length and the climb.

You can also continue in a later chat. Give the assistant the path of the project file and it picks up from
there.

### What to know

- Paths are absolute or start with `~`. A relative path is refused, because the assistant does not know the
  working folder of the program.
- Every change is saved at once. If the program refuses a change, for example a point outside the map or a
  code that is taken, the file stays as it was and the assistant gets the reason.
- Positions are in map millimetres at map scale, with y pointing down. Look at every control position yourself
  on the map. `check` tests distances, angles and codes against the rules (chapter 8), but not whether the
  control stands at the right feature.
- The program places control numbers only when the assistant asks for it. If the assistant then changes legs or
  the order, it has to have them placed again. A number placement set for a control or in a course stays.
- Automatic placement puts a number behind the circle, in line with the leg that leads to the control. If the
  course hardly turns, the next leg runs there, and the number goes outside, at the tip of the angle. It stands on
  a point or line symbol of the map only when no free place is left; it may cover areas. If you dislike the
  position, set it yourself.
- The number stands 0.5 to 5 mm from the circle (distance from the circle line to the number box, rule
  `wkb2026.number.gap`). Automatic placement starts at 0.5 mm and moves outward in steps of 0.5 mm before it turns
  away from the preferred direction. `check` warns when a stored gap lies outside the range, for example a number
  set by hand at 6 mm.
- When the order of a course changes, every leg that now runs to another stop loses its bends and its taped
  flag. The assistant gets a list of these legs.
- Only individual courses can be edited and measured so far. For team, relay and score courses the program
  says so.
- The project file is plain TOML. `freemapper-courseplan info <project>` shows what the assistant did.

## 8. Checking: reading findings, giving reasons for exceptions

`check` holds every individual course against the rules the event is meant to follow and tells you what stands
out. It changes nothing in the project file. You decide whether to change the course or to give a reason for the
finding as an intended exception.

```sh
freemapper-courseplan check hall.courseplan.toml
freemapper-courseplan check hall.courseplan.toml --all
```

`check` tests angles in both directions. It reports an acute angle as a hint below 90° and as a warning below
45°. If the course runs almost straight at a control, with an angle above 170° between the legs, it gives a hint
(`wkb2026.leg.straight_angle_hint`). The control then hardly changes the running direction, and there is no room
for the number behind the circle.

The program does not check team, relay and score courses yet. For each of them it reports once that it was not
checked.

### Reading a line

```text
course A:
  hint course A, leg 36–37: the legs at control 37 form an angle of 85°, at least 90° required (wkb2026.leg.acute_angle_hint, BL12 5. Bahnkonzeption)
```

From left to right the line holds:

- the severity, here "hint",
- the place, here the leg from control 36 to control 37 of course A. A leg is named by its ends, such as
  "leg start–31" or "leg 37–finish"; as places, start and finish read "start S1" and "finish F1",
- the text with the measured value and the limit,
- in brackets the id of the rule and the source the limit comes from.

You need the id to give a reason for an exception. The source tells you where to read the rule yourself. Within a
course the findings about the whole course come first, then those of its legs in running order. The output ends
with one summary, in the example "0 errors, 0 warnings, 1 hint; 0 accepted, 5 skipped".

### Severity and exit code

The rule profile sets how heavily a violation weighs for each rule. `check` itself treats the three levels
differently only in the exit code:

- Error: the profile treats the rule as binding. An open error sets exit code 1.
- Warning: the profile expects you to change the course or give a reason for the deviation.
- Hint: something stands out that you should look at, such as an angle just under the guideline.

The exit code tells a script how the run went. It is 0 when no error finding is open, 1 with at least one error or
when the check is refused (project file or map unreadable), and 2 for a wrong call such as an unknown option.
Warnings and hints do not change the exit code, and accepted findings do not count.

Some limits are not written in the rules like that; the program proposes them, for example that the halved control
distances also apply to schoolyard maps. Their source then reads "proposal of freemapper-courseplan", and `check
--all` shows the rule's note below the finding, which explains the proposal. If a straight leg crosses an
out-of-bounds area, the program only warns, because the best route might lead through it. Only the later route
analysis can say whether it does.

### Skipped rules

`check` counts the skipped rules in its summary, and `check --all` names each with its reason. Common reasons:

- At a training the program does not apply the length and difficulty rules of competitions.
- The rule applies only to another discipline, map kind or print scale.
- An overlay switched the rule off; the reason names it, such as "rule profile nor-bb-2021 switches the rule
  off".
- A value is missing. `event start-window hall.courseplan.toml 90` sets the start window, `class set
  hall.courseplan.toml H21 --starters 40` the expected starters of a class, and `course set hall.courseplan.toml A
  --climb 120` the climb of a course.
- A class has no rule category, for example a name you chose yourself such as "Offen". `class set
  hall.courseplan.toml Offen --rule-category H21` gives it one. A rule category in no row of the table skips the
  course too.

A skipped rule has not checked anything. An empty result therefore does not mean that everything passes. Read the
list once before you rely on it.

### Giving a reason for an exception

Some findings are intended. You accept them with the id of the rule, the place and a note:

```sh
# two controls that stand close together on purpose
freemapper-courseplan finding accept hall.courseplan.toml wkb2026.distance.min_half --controls 31,32 --note "The controls are on two sides of the same building and belong to different courses."

# a leg
freemapper-courseplan finding accept hall.courseplan.toml wkb2026.leg.acute_angle --leg A:3 --note "The sharp angle is intended, the course turns at this control."

# a control and the start
freemapper-courseplan finding accept hall.courseplan.toml wkb2026.distance.min_half --controls 31,S1 --note "The start is close to the first control on purpose, so the training begins with a short run-in."
```

`--controls` takes control codes and the id of a start or finish, and `--course COURSE` a finding about a whole
course. `--leg COURSE:N` names the leg that leaves position N of the course. Positions count the controls of the
course from 1, crossing points included, and 0 is the start. So N is not the control number on the map: on course
A with controls 31 to 37, `--leg A:6` is the leg 36–37. A finding at two legs gets `--leg` twice, such as
`--leg A:2 --leg A:5`. A finding of the whole event needs no place. If you give a place where the rule never
reports anything, the program refuses it and names the places the rule reports at. Accepting the same finding
again replaces the old note with the new one.

The note is stored in the project file, for a leg together with its ends. `check` no longer lists accepted findings
but counts them in the summary. With `--all` they appear under "accepted", with your note. That way next year's
course setter can see why the course is the way it is.

When you change a course, an exception can lose its finding: the control was removed, the course deleted, or the
leg now runs between other controls. In the last case `check` reports the finding at the new leg again, because
the old note was about the old leg. `check --all` lists such exceptions under "not matched", the summary counts
them, and they do not change the exit code.

To see a finding again, or to drop an exception that is not matched, take it back with the rule and the place:

```sh
freemapper-courseplan finding remove hall.courseplan.toml wkb2026.distance.min_half --controls 31,32
freemapper-courseplan finding remove hall.courseplan.toml wkb2026.leg.acute_angle --leg A:3
```

`finding remove --leg` first removes the exception for the leg as it runs now. If there is none, it removes the
exceptions at that position whose leg has changed since. If the id of a rule changes, for example in a new program
version, `check` reports its findings again.

### Choosing the rule profile and overlays

An event has one base profile and any number of overlays on it. The program ships two: `wkb-2026` (German
competition rules 2026) and `nor-bb-2021`, an overlay with the course recommendations of Berlin and Brandenburg.
One command sets both:

```sh
freemapper-courseplan event profile hall.courseplan.toml wkb-2026 --overlay nor-bb-2021
```

For several overlays, repeat `--overlay`; they apply in the order given. The program loads the profiles first and
refuses an unknown name with the list of available profiles. `info` shows the profile. In every finding of
`check`, the rule id starts with its profile (`wkb2026`, `norbb2021`). How a profile file is built and how to add
one is described in `crates/freemapper-courseplan/rules/README.en.md`.

### With the AI assistant

The assistant has six tools for this. `check` returns findings, accepted and unmatched exceptions and skipped
rules. `accept_finding` and `remove_accepted_finding` match `finding accept` and `finding remove`,
`set_rule_profile` matches `event profile`, and `set_start_window` and `set_class` match `event start-window` and
`class set`. Ask it, for example, to check the courses, explain each finding in one sentence and ask you before it
accepts one. You write the note or dictate it. The assistant should not invent one.

## 9. Exchanging with Purple Pen

The way in is `import`; the way back is `export --ppen` (below). A sync that brings changes from Purple Pen back is still missing. `import` takes over a Purple Pen event (`.ppen`) drawn for the same map. The map
must be an OOM file at the same scale as the event.

```sh
freemapper-courseplan init map.omap --ppen event.ppen
freemapper-courseplan import project.courseplan.toml --ppen event.ppen
```

`init --ppen` creates a new project from the event. `import` fills an existing one. If the project already has
controls, courses or special objects, `import` stops. With `--replace` it swaps them for the content of the
event; classes and accepted findings stay. The project then stores the path of the `.ppen` relative to the project
file.

What arrives: controls with code, position and description (renumbered to the 2024 standard), individual courses
with bends and marked routes, the number positions, out-of-bounds areas, dangerous areas, forbidden routes, first
aid and water points. Purple Pen counts the y axis upward; the import flips it. If the controls then do not lie on
the map, the import stops. A special object with a point outside the map is left out and noted.

What does not arrive is listed as notes on the terminal after the import: variations and alternative controls,
score courses, column F text in a description, descriptions on start and finish, and gaps in circles and legs.
Boundaries, lines, texts, white-outs and the secondary title of a course do not arrive either. A map exchange becomes a bend. Courses with equal names get a number. The distance of a number from its circle is
only approximate, the direction is exact. A symbol with no counterpart in the 2024 standard, such as the radio
control, is missing from the description and named in the note.

The AI assistant has the tool `import_ppen` for this, with `project`, `path` and `replace`. The decisions behind
it are in ADR 0034.

### Export to Purple Pen

```sh
freemapper-courseplan export project.courseplan.toml --ppen event.ppen
freemapper-courseplan export project.courseplan.toml --ppen event.ppen --new
```

The export writes the individual courses as a Purple Pen event. If the file exists, the export changes it in place.
What the project does not know stays: gaps in circles and legs, map exchange, column F text of a description and the
length of a changed course. Only lines that differ are rewritten. With `--new` the file is written from scratch and
the old one is lost. If there is no file, the export creates it, with descriptions in the 2018 standard. An existing
event keeps its standard.

The export never deletes. If a control, course or special object is in the file but not in the project, it stays
there and the program reports it as kept. The project cannot tell whether it was deleted here or added in Purple
Pen. The later sync is to decide that.

The Purple Pen ids decide which object is which. If the file is the project's linked file (`sources.ppen`), the export
sends the stored ids and stores new ones in the project. That also holds when the export creates the file or replaces it
with `--new` and the project has no linked file yet; it then records the file. If the project already has another linked
file, that link stays. For any other target the export writes nothing back into the project. Without a link it pairs
starts, finishes and crossing points by code, else by order when the counts are equal (all or nothing per kind).

The program names on the terminal what it writes differently or not at all:

- Team, relay and score courses are left out, each with its kind.
- A symbol with no counterpart in Purple Pen is missing from the description.
- If the bends of a leg change, its partial flagging loses the marking.
- A changed event title does not reach an existing file.
- If Purple Pen flags a leg by itself that the project does not mark, a note says so.

The project file, the map and the `--iof-xml` target of the same call are not allowed as targets. The program reads a
base file only up to a fixed size limit; a broken or foreign file gives an error message. If the file does not fit the
map, the export writes nothing.

**Opened in Purple Pen 4.0.** The example [`luebeck-sprint.ppen`](../../examples/luebeck-sprint/luebeck-sprint.ppen)
opens there without an error, and its content looks right (checked by the author, 2026-11-10). The file also reads
back through the reader of freemapper-course, and exporting a file that was read gives the same bytes.

The AI assistant has the tool `export_ppen` for this, with `project`, `path` and `new`; the path must end in `.ppen`.
The writer lives in `freemapper-course` (freemapper-core); the decisions are in ADR 0035.

## 10. Handing on: IOF XML for OE

`export` passes the courses to scoring software that reads IOF XML 3.0:

```sh
freemapper-courseplan export luebeck-sprint.courseplan.toml --iof-xml course-data.xml
```

The program writes all individual courses of the event to the file: controls with map position, the control
order of each course, leg lengths, climb, the position of the control numbers and the assignment of courses to
classes. It replaces an existing file only once the new one is completely written. Without `--iof-xml`, `export`
does nothing and exits with code 2.

Things to know:

- Team, relay and score courses are not written. The program names each skipped course with its kind.
- If the target is the project file or the map, `export` stops with an error and leaves both untouched.
- If the map is georeferenced, start, controls and finish carry latitude and longitude with six decimals. Without
  georeferencing, or with a coordinate system the program does not support, the file has map positions only, and
  `export` says so. OE, OCAD and Condes have not been tested with these positions.
- Taped legs are missing for now. The map extent is written as 0/0 mm.
- The time `createTime` is UTC and carries the marker `Z`. It does not affect the courses.
- The example [`course-data.xml`](../../examples/luebeck-sprint/course-data.xml) validates against the IOF schema,
  and a test reads the export back through the IOF reader of freemapper-course (freemapper-print's). An import into
  OE, Purple Pen or Condes has not been tested, because nobody on the project has OE. If you import the file there,
  please report what arrives.
- Purple Pen, OCAD and Condes write the y axis upward, against the standard. This export follows the schema (y
  downward). If a program shows the controls mirrored, tell us which one.

The AI assistant has the tool `export_iof_xml` for this, with `project` and `path`; the path must end in `.xml`.
The decisions behind it are in ADR 0033.
