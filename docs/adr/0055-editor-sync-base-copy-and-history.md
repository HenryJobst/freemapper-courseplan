# 0055. Editor: the sync can write the base copy alone

Date: 2026-11-11
Status: Accepted

## Context
ADR 0054 enabled "Sync" only when a change was waiting. A project linked to a Purple Pen file but never synced has
no base copy, and the two files may agree. The dry run then lists nothing, the button stayed disabled, and the
editor could never write the base copy. The next one-sided edit on either side then counted as a conflict. A real
sync in that state writes the base copy, and sometimes nothing else.

## Decision
- **Sync stays available without changes** *(Claude's call)*. With no open conflict, "Sync" is enabled when changes
  wait or when the dry run reports a base state other than `use`. In the second case the button reads "Create the
  base copy" and a note says what it does.
- **History follows the project text** *(Claude's call)*. A real sync clears the undo history only when the project
  file's revision changed. A sync that wrote only the base copy or the Purple Pen file keeps undo and redo. This
  refines the rule of ADR 0054 ("a sync that wrote files clears the history"); the history exists to undo project
  text, and that text is the same.
- **Link path** *(Claude's call)*. The symbolic-link check of ADR 0053 and 0054 covers every component of the path
  that `sources.ppen` names, not only the last one.
- **Error texts** *(Claude's call)*. A failed export to a Purple Pen file and a missing linked file get fixed texts, like
  the import and sync paths. Every file a message names, including the base copy and the map, shrinks to its file
  name.

## Consequences
The sync panel shows one more state. A symbolic link as a parent directory of the linked file is refused where it
was accepted before.
