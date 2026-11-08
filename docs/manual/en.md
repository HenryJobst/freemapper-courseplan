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

Tell the assistant what you want, for example:

> Start an event from ~/maps/hall.omap. Put the start at the south door and the finish by the gym. Add three
> controls in the yard, make a course "Short" from them for class D10, place the numbers and tell me the length.

The assistant then works like this:

1. It starts the event. The project file lands next to the map, named `hall.courseplan.toml`.
2. It asks for the extent of the map and for the map objects around a spot, to find the door, the gym and
   good places for controls.
3. It adds the start, the finish and the controls.
4. It adds the course, places the control numbers and reads the length and the climb.

You can also continue in a later chat. Give the assistant the path of the project file and it picks up from
there.

### What to know

- Paths are absolute or start with `~`. A relative path is refused, because the assistant does not know the
  working folder of the program.
- Every change is saved at once. If the program refuses a change, for example a point outside the map or a
  code that is taken, the file stays as it was and the assistant gets the reason.
- Positions are in map millimetres at map scale, with y pointing down. The assistant gets them from the map
  objects. Look at every control position yourself on the map, because nothing checks them against the rules
  yet.
- Only individual courses can be edited and measured so far. For team, relay and score courses the program
  says so.
- The project file is plain TOML. `freemapper-courseplan info <project>` shows what the assistant did.
