# 0059. The command line runs the MCP operations

Date: 2026-11-11
Status: Accepted

## Context
`AGENTS.md` names the command line as the preferred interface for an assistant that can run commands, and lists what
it cannot do compared with the MCP tools: the tools `map_extent`, `map_features_near`, `list_controls`,
`list_courses`, `list_specials` and `list_classes` had no command, and `event start-window` could not remove the
window, which `set_start_window` can with `no_start_window`. The user asked to close these gaps
*(user: AGENTS.md CLI-first request)*.

## Decision
- **One operation, two doors** *(user: AGENTS.md CLI-first request)*. The read commands do not rebuild the answer.
  They call `CoursePlanServer::call_json`, the entry point the editor uses (ADR 0043), which runs the same
  `*_op` function as the MCP tool. A change to a tool changes the command.
- **Commands** *(Claude's call)*: `list controls|courses|specials|classes <PROJECT>`, `map extent <PROJECT>` and
  `map near <PROJECT> --at X,Y --radius MM [--limit N]`. The groups are `list` and `map`, and each command takes
  `--json`.
- **`--json` is the MCP answer** *(user: AGENTS.md CLI-first request)*. It prints the tool's value with
  `"ok": true` added, as the MCP result has it, and nothing else. Without `--json` the same value becomes a table
  or lines from the Fluent catalog, in both languages. The table reads only the JSON, so it cannot disagree with it.
- **Names follow the command line, not the tool** *(Claude's call)*. A point is `--at X,Y` in map millimetres, as in
  `control add`, not `x` and `y`; the radius is `--radius MM`, not `radius_mm`. Units are map millimetres in the
  frame of the `.omap` file.
- **Errors are the command line's** *(Claude's call)*. The command loads the project first, so a missing or broken
  file gives the message of every other command (`error: ...`, exit code 1). The tool's own messages stay
  redacted for the assistant (ADR 0023). A usage error, such as a bad point or a radius of 0 or less, exits with 2.
  The relative project path is made absolute before the call, which the tool requires.
- **Start window** *(Claude's call)*. `event start-window <PROJECT> <MINUTES>` or `event start-window <PROJECT>
  --none`; exactly one of them. The command calls `edit::clear_start_window`, which `set_start_window_op` calls
  too. Removing an absent window is not an error, as with the tool.

## Consequences
- `AGENTS.md` loses its list of gaps. An assistant with a shell can find usable positions without MCP.
- Each command starts the tool router to reach the operation, which costs a few milliseconds.
- A test per command compares `--json` with the answer of `freemapper-courseplan mcp` over stdio on the Luebeck
  example, so a divergence fails the build.

Source: ADR 0023, ADR 0043, `AGENTS.md`
