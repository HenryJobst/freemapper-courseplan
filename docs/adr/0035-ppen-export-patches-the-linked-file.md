# 0035. The .ppen export patches the linked file

Date: 2026-11-10
Status: Accepted

## Context
Design section 5.5 asks for a `.ppen` export, and section 5.5a for parallel use with Purple Pen *(user)*. The
writer lives in freemapper-core v0.4.0 (`freemapper-course`, core ADR 0014). Purple Pen keeps data the model
does not hold: circle gaps, map exchange, column F text, course length. A user who exports onto a file they
also edit in Purple Pen must not lose those.

## Decision
- **Patch, not overwrite.** `export --ppen FILE` reads an existing FILE as the base and rewrites only what
  differs. Everything the project does not hold stays: gaps, map exchange, column F text, the length of a
  changed course, circle gaps on a moved control. `--new` writes the file from scratch *(Claude's call)*.
- **Never deletes.** An object that is in the file but not in the project stays and is reported as kept. A
  base-less export cannot tell "deleted here" from "added in Purple Pen"; the sync (M3c) will decide *(Claude's call)*.
- **Ids and the link.** An export that creates a file, or runs with `--new`, links the project only if it has
  no `sources.ppen` yet, and stores the Purple Pen ids in it. For the linked file the stored ids are used and new
  ones stored. For any other target nothing is written back and the link stays *(Claude's call)*.
- **Pairing without a link.** Objects pair by code or name. Starts, finishes and crossing points, which Purple
  Pen stores without a code, pair by code, else by order when the counts match; all or nothing per kind.
- **Losses, all named.** Team, relay and score courses are skipped. A symbol with no Purple Pen counterpart is
  left out. Partial flagging of a leg is dropped when its bends change. The title of the event is not written
  in a patch. Every case is a note.
- **Standard and map.** The description standard comes from the base (2018 for a new file). The map is written
  as `kind="OCAD"` with the `.omap` path, as the writer does for any OOM map.
- **Safe reading.** The base is read through the same size bound as the import (`MAX_COURSE_FILE_BYTES`). A
  hostile base gives an error, never a panic. The target may not be the project file, the map or the IOF XML
  target of the same call.

## Consequences
- Export onto the file an import read gives the same bytes; a test checks it on three Purple Pen files and
  on the Lübeck example, whose export is committed as `examples/luebeck-sprint/luebeck-sprint.ppen`.
- **Not tested in Purple Pen 4.0.** The export reads back through the core reader only. The author is asked to
  open the example in Purple Pen 4.0 and report; core plan Task 6 waits for a Purple Pen 4.0 file.
- A stale course length can remain in a changed course; the sync will tell.

Source: spec §5.5 and §5.5a, plan 2026-11-10-ppen-writer, core ADR 0014
