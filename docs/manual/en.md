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
- If the project file changed after the assistant read it, for example because you moved a control in the editor,
  the program writes nothing. The assistant gets "<file> changed on disk after it was read, so nothing was
  written. Read the project again and repeat the change" and repeats the change with the new state.
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

## 4. The editor (preview)

The editor shows the event in the browser. You can place, move and delete controls, build courses and categories,
bend and tape legs, place control numbers, draw special objects, choose control description symbols and take each step back.
The area for findings is empty, and the menu items for export, import and sync are greyed out. Both follow in later versions.

### Starting it

```sh
freemapper-courseplan edit luebeck-sprint.courseplan.toml
```

The command loads the project, opens the browser and prints the address on a single line, such as
`http://127.0.0.1:47123/3f9c…/`. If the project file is broken, it names the file and exits with code 1 without
printing anything. It runs until you press Ctrl+C or choose "Close editor" in the "Event" menu. Both end it with
exit code 0.

- `--port N` picks the port. Without it the program takes a free one. If the port is taken, the program says so and
  exits with code 1.
- `--no-browser` does not open the browser. You open the printed address yourself. If the browser cannot be started,
  the program prints a hint and keeps running.
- The page speaks the program's language (`--lang en`, `FREEMAPPER_LANG` or the system language setting). The page
  has no language switch; to change the language, restart the editor.

One editor belongs to one project. To open a second project, start a second editor.

### What the page shows

- On the left is the tree of the event with the groups Courses, Categories, Controls and Special objects. "All
  controls" shows every control with its code.
- In the centre is the map. When you select a course, the editor draws its circles, start triangle, finish, legs and
  control numbers on top. A number box with a dashed outline was placed automatically; the project stores no position
  for it yet.
- On the right are the properties of the selection, for example name, length and climb of a course, or code, kind,
  position and description of a control. For a control the list also names the map objects within 3 mm. The area for
  findings is still empty.
- The status bar at the bottom names the map, the scale, the map kind and the cursor position in map millimetres.
  Messages appear there too.

In this preview the map is a placeholder: a frame around the map extent, a 10 mm grid and a note. Circles, legs and
numbers sit at the right place on the map, but no map content is visible beneath them. freemapper-render will supply
that later.

### Controls

The toolbar has the tools "Select", "Place control", "Place crossing point" and "Special object". Every change is
saved at once, through the same operations as the command line and the AI assistant, so the same checks apply. If
the program refuses a change, for example a position outside the map, the file stays as it was and a message says
why.

- **Select.** With the "Select" tool, click a control in the map or in the tree. A click selects the control whose
  centre is nearest to the pointer, as long as the pointer is within the circle's radius plus the line width. The
  properties on the right show it.
- **Place.** Choose "Place control" and click on the map. The editor adds a control at that point.
- **Place a crossing point.** "Place crossing point" works the same way but adds a control of the kind "Crossing
  point". It takes the next free code, shares the numbering with the other controls and appears in the tree with
  "(Crossing point)". On the map it is drawn as two arcs without a circle. Its properties show code, kind and
  position, but no control description, since a crossing point has none. To add it to a course, use course mode
  like for any control.
- **Move.** Drag a circle, or type a new value into the fields X (mm) and Y (mm) in the properties. You may write
  a decimal comma or a decimal point, `12,5` or `12.5`. Anything else marks the field and shows a hint. While you
  drag, only the circle moves. Legs and numbers follow when you let go. Esc cancels the drag, and the control
  stays where it was.
- **Nudge.** With a control selected, the arrow keys move it by 0.1 mm, with Shift by 1 mm. The editor saves once
  when you release the key, so holding a key makes a single undo step. Without a selection, the arrow keys move the
  view.
- **Change the code.** Edit the "Code" field in the properties. The courses and accepted findings that name the
  control follow. The editor refuses a code that is empty, has spaces at its ends, is longer than 64 characters or
  belongs to another control. On the command line the same is `freemapper-courseplan control code
  hall.courseplan.toml 31 K7` (see "Changing a code or a name on the command line" below).
- **Delete.** Press Delete or Backspace while the focus is on the map or the tree, or use "Delete control" in the
  properties. In a field or on a button the keys do nothing. A control that no course visits is deleted at once;
  undo takes it back. If courses visit the control, a dialog lists them and asks. The focus is on "Cancel".
  "Delete" removes the control and takes it out of those courses. A stray Backspace on a selected, unused control
  therefore deletes it without a question; undo brings it back.

### Courses and categories

- **New course.** Choose "New course..." under "Courses" in the tree and type a name. The course gets the start and
  the finish of the event. If the event has not exactly one start and one finish, the editor says so and adds
  nothing. A name that is taken, empty or longer than 200 characters is refused in the dialog.
- **Rename a course.** Edit the "Name" field in the properties of the course. Special objects and accepted findings
  that name the course follow. On the command line use `freemapper-courseplan course rename hall.courseplan.toml A
  Long`.
- **Course mode.** Select a course, then choose "Course mode". The controls the course does not visit appear in
  grey. A click on a control adds it before the finish. To insert a control inside the course, first click the leg
  where it belongs, then the control. The chosen leg turns thick and blue, and the status bar says which one it is.
  Esc drops the chosen leg; a second Esc leaves course mode. Course mode has no key.
- **Stops.** The properties of a course list its stops. The buttons "Move up", "Move down" and "Remove" change the
  order. With the focus on a row, Alt+Up and Alt+Down move the stop. Each change is one step for undo and saves the
  whole list at once. If a change sends a leg to another stop, that leg loses its bends and tape. The status bar names
  those legs.
- **Course details.** Climb and planned print scale are fields in the properties; an empty field removes the value.
  The check boxes assign the course to categories.
- **Categories.** "New category..." adds one. Its properties hold the rule category and the expected starters; an
  empty field removes the value. "Delete category" removes it. The editor refuses this while a course still uses
  it, and says so above the map.
- **Event.** "Event settings" in the tree shows the discipline, which you cannot change here, the rule profile with
  its overlays, both as lists of the profiles the program knows, and the start window in minutes. An empty start
  window removes it.

### Legs and bends

A leg is the line between two stops. Click it with the "Select" tool to choose it.

- **Add a bend.** Drag the line of the chosen leg. A bend appears where you let go.
- **Move or delete a bend.** Drag a bend to move it. A click on a bend chooses it, and Delete removes it.
- **Tape a leg.** The "Taped" check box in the properties of the leg marks it as a taped leg.

Each gesture is one step for undo, and the course length in the properties changes with it. A leg that a change to
the stops sends to another stop loses its bends and tape.

### Control numbers

The number box of a control sits beside its circle. A dashed outline means the program placed it automatically and
the project stores no position for it yet.

- **Drag.** In course mode, drag a number box to another place. On release the editor turns the place into a
  direction and a gap from the circle and stores both for this course.
- **For all courses.** With "for all courses" ticked, the placement is stored on the control and applies to every
  course that uses it. A placement the course already has for itself comes before the one on the control. The box
  then snaps back, and the status bar says so.
- **Place numbers.** "Place numbers" places the numbers of the course automatically and leaves placed numbers alone.
  With "also replace placed numbers" everything is computed again, except the numbers set on the control for all
  courses.

One drag is one step for undo. The editor does not yet warn about a gap outside the rule range; `check` reports it.

### Control descriptions

Select a control. Its properties list the columns C to H, each folded and showing the current symbol. Open a column
to see its symbols as a grid of icons, each with its number and text in the language of the program. The map objects
within 3 mm of the control are listed below column D.

- **Pick.** Click a symbol, or move with the arrow keys (Home and End jump to the first and last) and press Enter.
  Typing a number such as `1.3` jumps to it. The symbol in use is marked, and the focus stays on the pick.
- **What a column offers.** Column C has the symbols 0.1 to 0.5 for which of several similar features is meant, with
  the compass directions 0.1N to 0.2NW; the instruction rows are not offered there.
  Column D has 1.1 to 6.2, column E has the symbols of column D and 8.1 to 8.11, column F has 10.1,
  10.2 and 11.1 (the number symbols 9.1 to 9.4 are not offered, because the project file holds no value for them), column G has the 12.x symbols, and column H has 13.1 to 13.3.
- **Directions.** A symbol with a direction exists only with one: choose `12.1N`, `12.1NE` and so on, there is no
  plain `12.1`.
- **A symbol the column does not list.** If the control holds one (an old or hand-written one), it appears at the top
  as "current value" and stays until you pick something else.
- **No symbol.** The entry "No symbol" empties the column.
- **Undo.** Each pick, and each emptied column, is one step for undo.

The list comes from freemapper-core, so every symbol you can pick can also be written to a `.ppen` file (ADR 0051).

### Special objects

- **Draw.** Choose "Special object", pick a kind (start, finish, first aid post, water point, marked route,
  forbidden route, out-of-bounds area, dangerous area) and check the id in the toolbar. It is proposed as
  `<kind>-<n>` and you may change it. A start, finish, first aid post or water point is placed by one click. A line
  or an area is drawn by clicking its points and finished with Enter or a double click. Esc cancels, Delete takes
  back the last point, and a dashed preview with dots shows the drawing. An area needs three points, a marked route
  two. A forbidden route may have a single point, which is drawn as a cross. Each object is one step for undo.
- **Signs on the map.** M marked route, a cross forbidden route, OB out-of-bounds area, ! dangerous area, plus first
  aid, W water.
- **Assign to courses.** In the properties of a selected object, tick the courses it belongs to. With none ticked it
  belongs to all courses. Start and finish have no course list.
- **Delete.** "Delete special object" or the Delete key removes the object; undo brings it back.
- **No move.** The editor cannot move a special object. Delete it and draw it again (ADR 0049).

### Undo and redo

The toolbar buttons "Undo" and "Redo" and the keys Ctrl/Cmd+Z and Ctrl/Cmd+Shift+Z (or Ctrl+Y) walk through the
steps of this editor session. One placing, one drop, one field change, one delete and one series of nudges is a
step each. The status bar reports "Undone" or "Redone".

- The editor keeps the last 100 steps, as long as their file copies together stay under 64 MiB; the oldest steps go
  first. They are lost when you close the editor.
- Undo and redo act on the state you see. If a second browser tab shows an older state than the file, they refuse
  and the tab loads the current state; they never revert what the other tab did without you seeing it.
- Undo restores the file as it was before the step, comments and layout included. That also takes back whatever
  else the step changed in the file, for example the removal of a control from a course.
- If another program changed the file after your last step, undo and redo refuse. The status bar says "The project
  file changed outside the editor, so undo and redo are no longer possible. Nothing was changed." and the history
  is empty from then on. The editor notices this at your next undo or redo, not at the moment of the outside change.

### Shortcuts

Press `?` or the "Keyboard shortcuts" button for this list in the page. A key typed into a field is never taken as
a shortcut.

| Keys | Effect |
| --- | --- |
| Ctrl/Cmd+Z | Undo |
| Ctrl/Cmd+Shift+Z, Ctrl+Y | Redo |
| Delete, Backspace | Delete the selected control, bend or special object; while drawing, the last point |
| Esc | Cancel a drag or a drawing, drop the chosen leg, leave course mode or go back to the select tool |
| Alt+Up, Alt+Down | Move the focused stop of a course |
| Enter | Finish the line or area being drawn |
| V | Select tool |
| C | Place-control tool |
| S | Special-object tool |
| Arrow keys | Move the selected control by 0.1 mm (without a selection: move the view) |
| Shift+arrow keys | Move the selected control by 1 mm |
| `+`, `-` | Zoom in, zoom out (Ctrl or Cmd with these keys still zooms the browser) |
| `0` | Fit the map to the window |
| `?` | Show the list |

### Mouse and tree

- In the tree, the arrow keys, Home and End move between entries and Enter selects.
- With the mouse you move the view by dragging on empty space and zoom with the wheel.
- Tab moves from the tree to the map, to the properties and to the menu.

### Changing a code or a name on the command line

```sh
freemapper-courseplan control code hall.courseplan.toml 31 K7
freemapper-courseplan course rename hall.courseplan.toml A Long
```

`control code` gives control 31 the code K7. `course rename` renames course A to Long. Courses, special objects and
accepted findings that name the control or course follow. A code or name that is taken, empty, padded with spaces
or too long is refused with a message, and the file stays as it was. The AI assistant has the tools
`set_control_code` and `rename_course` for the same.

### Changed outside the editor

If another program changes the project file, for example `freemapper-courseplan control move` in a terminal or the AI
assistant, the page reloads it after about two seconds and says so in the status bar. A tab in the background does
not ask again until you open it. If the map changes, the page reloads it. If the linked `.ppen` changes, the status
bar shows a note.

If you make a change in the page while the file is out of date, say between two polls, the editor refuses it. It
loads the current state, shows the banner "The project file changed outside the editor. The editor loaded the
current state; nothing was changed." at the bottom of the map and writes the same text in the status bar. Look at
what the other program did and make your change again if you still want it. "Dismiss" closes the banner.

If the project file does not load, for example because someone left it half edited, every change in the editor is
refused with the load error until the file is readable again.

### Commands that find the file changed

The editor is not the only program that saves the project file. Every command that changes it checks, right before
it saves, whether the file is still the one it read. If it is not, for example because the editor saved a move
while the command ran, the command writes nothing, prints

```
luebeck-sprint.courseplan.toml changed since it was read; nothing was written
```

and exits with code 1. Run the command again; it then reads the new state. The check counts any change in the file,
also a save from a text editor that changed nothing in the event. The AI assistant gets the same refusal in words
of its own (chapter 3). `sync` writes the `.ppen` before the project file. If the project file changes in between,
`sync` reports a partial write, and running it again finishes the job (chapter 9).

### Privacy and security

- The server listens on `127.0.0.1` only. Other computers on the network cannot reach it.
- The address contains a random access token (32 hex digits). Anyone who knows the full address can look at the event
  on your computer. Do not pass the address on.
- The token also shows in the process list while the program hands it to the browser. On a computer that several
  people share, another user can read it there. Start the editor with `--no-browser` on such a machine and open the
  address yourself, or work on your own computer.
- The page loads nothing from the network. The server reads only the project file, the map and the linked `.ppen`.
  It accepts no file names from the browser.
- The server also checks the origin and content type of requests that write, so a foreign web page in the same
  browser cannot trigger anything. The editor changes only the project file, and only through the operations of the
MCP tools. The address of an operation names no file.

The decisions behind this are in ADR 0042, 0045 and 0046.

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

The way in is `import`; the way back is `export --ppen` (below). `sync` brings changes made in Purple Pen back (below). `import` takes over a Purple Pen event (`.ppen`) drawn for the same map. The map
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
Pen. `sync` (below) decides that, using the base copy.

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

### Syncing with Purple Pen

```sh
freemapper-courseplan sync project.courseplan.toml --dry-run
freemapper-courseplan sync project.courseplan.toml
```

`sync` merges the project with its linked `.ppen` in both directions. The project must be linked first, through
`export --ppen FILE` or `import --ppen FILE`; without a link, `sync` stops and says so. Run it whenever you switch
between the two programs.

> **Warning: close the event in Purple Pen before you run `sync`, and reopen it afterwards.** Purple Pen keeps the
> event in memory. If it is still open when `sync` writes the `.ppen`, its next save drops everything `sync` added.
> The following `sync` then sees those objects as deleted in Purple Pen and deletes them from the project, without a
> conflict. When unsure, run `--dry-run` first; it writes nothing.

**The base copy.** `sync` compares three versions of every field: the file as the last sync left it, the file now and
the project now. It keeps the first one beside the project file as `X.courseplan.ppen-base` (for `event.courseplan.toml`
that is `event.courseplan.ppen-base`). Its extension is not `.ppen`, so nobody opens it in Purple Pen by mistake.
Commit it to git together with the project file. `sync` removes the `absolute-path` attribute of the map from the copy,
because it holds the path of your home folder, and the merge does not compare it. The project file gets a `[sync]`
table with the link and a hash of the base copy.

**First sync.** There is no base copy yet. `sync` says so, deletes nothing and treats every difference as a conflict.
It writes the base copy at the end. The same happens if the base copy is missing, belongs to another file or does
not match its hash; the note names the reason.

**Dry run.** `--dry-run` lists what the sync would change in each direction and which conflicts are open. It writes
nothing, not even decisions.

**What gets merged.** Each field on its own: a control's code, kind, position and each description column; a course's
name, controls, climb, print scale and the position of each number; the bends and marking of a leg; the shape of a
special object. A field changed on one side goes to the other. If both sides made the same change, it counts once.
If you moved control 34 in the project and someone moved 31 in Purple Pen, both moves land. A deletion is a change
too: deleted on one side and untouched on the other is deleted on both. If two people insert different controls into a
course at different places, both land.

**Conflicts.** A field that both sides changed differently is a conflict, and so is deleting an object on one side
while the other edited it. A course that Purple Pen deletes while the project has classes on it is a conflict as
well, because the classes would go with it. `sync` prints each conflict with its key and three values. Here control 34
was moved to the right in the project and to the left in Purple Pen:

```text
Sync with out.ppen.
Changes to Purple Pen: 0
Changes into the project: 0
Changed the same way on both sides: 0
control/34/position differs.
Base: -1.200, -12.400 mm
Project: -1.000, -12.400 mm
Purple Pen: -1.400, -12.400 mm
1 conflict is still open. Nothing was synced. Pick a side for all of them with --take ours or --take theirs, or for one conflict with --take KEY=ours or --take KEY=theirs.
```

While a conflict is open, `sync` writes nothing, not even the changes without conflict, and exits with 1. You settle
it with `--take`:

- `--take ours` keeps the project's value for every open conflict, `--take theirs` takes the Purple Pen file's.
  Nothing is stored; it holds for this call only.
- `--take control/34/position=theirs` decides one conflict and can be repeated. If other conflicts stay open, the
  decision is stored in the project file under `[[sync.decisions]]`, together with a fingerprint of both values. It
  holds while they stay as they are; if either changes, `sync` ignores the decision and says so. A completed sync
  clears all stored decisions.

The keys have the form `control/34/position`, `control/32/column/g`, `control/35` (the whole control),
`course/A/stops`, `course/A/number/31`, `leg/31-32` or `special/WATER1`. Run `sync --dry-run` to see the ones that are open.

**What is written.** When no conflict is open, `sync` writes in this order: the `.ppen`, the project file, the base
copy. Just before the first write it reads the `.ppen` again. If Purple Pen saved in the meantime, nothing is
written and you run `sync` again. If a crash or a full disk stops it in between, the next `sync` finishes the job:
changes that both sides already have are no conflicts, and a base copy that the project file shows as outdated is
repaired. A control stays in the `.ppen` when a course in it still visits the control, even if the project deleted it;
a note says so.

**What stays only in courseplan.** Classes, expected starters, accepted findings, print areas, team, relay and score
courses, marked routes in special objects and forbidden routes with several points never reach Purple Pen, and
`sync` never touches them. A note names each team, relay and score course, for example `note: not written to Purple
Pen: left out T: team courses are not exported to Purple Pen`. The reverse also holds: what the import skips, such as
variations and gaps in circles, stays in the `.ppen` as it is. A renamed control or course in Purple Pen is carried
along into accepted findings, legs and the course lists of special objects. If Purple Pen leaves a number position
automatic or resets it to automatic, that stays automatic in the project. If one side removes a leg while the other
bends it, the removal wins and a note says so.

**Refusals.** `sync` refuses, and writes nothing, if the `.ppen` comes from a Purple Pen version it does not know,
if the linked file does not exist, or if the writer could not match an object of the file to the merged project. The
last case can occur when you swap two control codes and rename a course in one sync; do it in two steps.

The AI assistant has two tools. `sync_ppen` takes `project`, `dry_run` and `take` (`ours` or `theirs` for the whole
call) and returns `written`, `conflicts`, `changes` and `notes`. `resolve_sync_conflict` takes `project`, `key` and
`take`, stores one decision and returns what is still open; when nothing is left, `sync_ppen` finishes the job. With
`sync --json` the command prints the same fields. The decisions behind all this are in ADR 0040.

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
