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
  on the map, because nothing checks them against the rules yet.
- The program places control numbers only when the assistant asks for it. If the assistant then changes legs or
  the order, it has to have them placed again. A number placement set for a control or in a course stays.
- When the order of a course changes, every leg that now runs to another stop loses its bends and its taped
  flag. The assistant gets a list of these legs.
- Only individual courses can be edited and measured so far. For team, relay and score courses the program
  says so.
- The project file is plain TOML. `freemapper-courseplan info <project>` shows what the assistant did.

## 8. Checking: reading findings, giving reasons for exceptions

`check` holds every individual course against the rules the event is meant to follow and tells you what stands out.
It changes nothing in the project file. You decide whether to change the course or to give a reason for the finding
as an intended exception.

```sh
freemapper-courseplan check hall.courseplan.toml
freemapper-courseplan check hall.courseplan.toml --all
```

The program does not check team, relay and score courses yet. For each of them it reports once that it was not
checked.

### Reading a line

```text
course A:
  hint course A, leg after 6: the legs at control 37 form an angle of 85°, at least 90° (wkb2026.leg.acute_angle_hint, BL12 5. Bahnkonzeption)
```

From left to right the line holds:

- the severity, here "hint",
- the place, here the leg after the sixth position of course A (position 0 would be the leg from the start),
- the text with the measured value and the limit,
- in brackets the id of the rule and the source the limit comes from.

You need the id to give a reason for an exception. The source tells you where to read the rule yourself. After the
findings comes a summary, in the example "0 errors, 0 warnings, 1 hint; 0 accepted, 5 skipped".

### Severity and exit code

The rule profile sets how heavily a violation weighs for each rule:

- Error: the course breaks a rule that the profile treats as binding.
- Warning: the course deviates, and you should change it or give a reason for the deviation.
- Hint: something stands out that you should look at, such as an angle just under the guideline.

The exit code tells a script how the run went. It is 0 when no error finding is open, 1 with at least one error or
when the check is refused (project file or map unreadable), and 2 for a wrong call such as an unknown option.
Warnings and hints do not change the exit code, and accepted findings do not count.

Some checks are proposals of the program and do not appear word for word in the rules, for example that the halved
control distances also apply to schoolyard maps. The notes of the profile (`note` in the profile file, see
`crates/freemapper-courseplan/rules/README.en.md`) mark them with "Claude's call". If a straight leg crosses an
out-of-bounds area, the program only warns, because the best route might lead through it. Only the later route
analysis can say whether it does.

### Skipped rules

`check` counts the skipped rules in its summary, and `check --all` names each with its reason. Common reasons:

- At a training the program does not apply the length and difficulty rules of competitions.
- The rule applies only to another discipline, map kind or print scale.
- A value is missing, such as the start window of the event for the start capacity or the climb of a course.
- A class is not mapped to a category of the rule table, for example a name you chose yourself such as "Offen".

A skipped rule has not passed anything, it has just not checked. Read the list once before you rely on an empty
result.

### Giving a reason for an exception

Some findings are intended. You accept them with the id of the rule, the place as `check` shows it, and a note:

```sh
# two controls that stand close together on purpose
freemapper-courseplan finding accept hall.courseplan.toml wkb2026.distance.min_half --controls 31,32 --note "The controls are on two sides of the same building and belong to different courses."

# a leg
freemapper-courseplan finding accept hall.courseplan.toml wkb2026.leg.acute_angle --leg A:3 --note "The sharp angle is intended, the course turns at this control."

# a control and the start
freemapper-courseplan finding accept hall.courseplan.toml wkb2026.distance.min_half --controls 31,S1 --note "The start is close to the first control on purpose, so the training begins with a short run-in."
```

`--controls` takes control codes and the id of a start or finish, `--leg COURSE:N` the leg after position N (`--leg A:2 --leg A:5` for a finding at two legs), and `--course COURSE` a finding about a
whole course. A finding of the
whole event needs no place. If you give a place where the rule never reports anything, the program refuses it.
Accepting the same finding again replaces the old note with the new one.

The note is stored in the project file. `check` no longer lists accepted findings but counts them in the summary.
With `--all` they appear under "accepted", with your note. That way next year's course setter can see why the course
is the way it is.

To see a finding again, take the exception back with the same details:

```sh
freemapper-courseplan finding remove hall.courseplan.toml wkb2026.distance.min_half --controls 31,32
```

If the id of a rule changes, for example in a new program version, `check` reports its findings again.

### Choosing the rule profile and overlays

An event has one base profile and any number of overlays on it. The program ships two: `wkb-2026` (German
competition rules 2026) and `nor-bb-2021`, an overlay with the course recommendations of Berlin and Brandenburg. One
command sets both:

```sh
freemapper-courseplan event profile hall.courseplan.toml wkb-2026 --overlay nor-bb-2021
```

For several overlays, repeat `--overlay`; they apply in the order given. The program loads the profiles first and
refuses an unknown name with the list of available profiles. `info` shows the profile. In every finding of `check`, the
rule id starts with its profile (`wkb2026`, `norbb2021`). How a profile file is built and how to add one is described in
`crates/freemapper-courseplan/rules/README.en.md`.

### With the AI assistant

The assistant has four tools for this. `check` returns findings, accepted findings and skipped rules,
`accept_finding` and `remove_accepted_finding` match `finding accept` and `finding remove`, and `set_rule_profile`
matches `event profile`. Ask it, for example, to check the courses, explain each finding in one sentence and ask you
before it accepts one. You write the note or dictate it. The assistant should not invent one.
