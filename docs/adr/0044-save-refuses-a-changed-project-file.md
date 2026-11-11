# 0044. Save refuses a project file that changed since it was read

Date: 2026-11-11
Status: Accepted

## Context
Every command loads the project file, changes the event and saves it. If another program wrote the file in
between (the editor, an MCP client, Purple Pen's sync, a text editor), the save replaced that work without a word.
ADR 0040 listed this as a known limit. With the browser editor and the MCP server running side by side it stops
being theoretical.

## Decision
- `ProjectFile` keeps the text it was loaded from, or wrote last (`create`, `save`). `save()` reads the file again
  before it writes. If the text differs, or the file is gone, it returns `ProjectError::ChangedMeanwhile` and
  writes nothing. After a successful write the kept text is the new one, so several saves in a row work.
- The CLI prints the localized `error-project-changed` and exits 1. The MCP server answers with
  `mcp-error-project-changed`, which names the file and quotes nothing from it.
- Commands that load more than once save the `ProjectFile` they loaded last. `sync` loads once to find the link and
  again in `open_inputs`; it saves the second. Export and import load once.
- Debug builds have a test hook, `FREEMAPPER_TEST_TOUCH_PROJECT`, that appends a comment to the project file just
  before the CLI edit commands save. `FREEMAPPER_SYNC_TEST_TOUCH` (the `.ppen`) stays for the sync tests. Neither
  exists in release builds.

## Consequences
- The lost update is refused for the CLI, MCP and editor alike. What is left is the window between the re-read and
  the rename, a few microseconds.
- Any byte change counts, also one that left the event the same. A user who saved the file in a text editor
  without changes gets the refusal once and runs the command again.
- A sync writes the `.ppen` before the project file. If the project file changed in between, the `.ppen` is already
  written, and the CLI reports a partial write. Running `sync` again finishes the job (ADR 0040).
- A file that cannot be read at the second look is a read error, not a changed file.
