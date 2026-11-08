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
