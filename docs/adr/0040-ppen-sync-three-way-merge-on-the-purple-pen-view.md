# 0040. The Purple Pen sync is a three-way merge on the Purple Pen view

Date: 2026-11-10
Status: Accepted

## Context
Design section 5.5a asks for `sync`: parallel use of courseplan and Purple Pen, changes flowing both ways, no silent
overwrite *(user)*. ADR 0034 (import) and ADR 0035 (patch export) exist; the project keeps Purple Pen ids (`ppen_id`).
The manual's sync chapter and the plan 2026-11-10-ppen-sync hold the details; this ADR keeps the decisions.

## Decision
- **One frame** *(Claude's call)*. Base, theirs and ours are all what the import makes of a `.ppen`: base = import of
  the base copy, theirs = import of the linked file, ours = import of the project exported onto the base copy. Fields
  are compared on the import's own grid, without tolerance.
- **Pick rule** *(user for the principle)*. Per field: unchanged, one side, the same change on both sides, or a
  conflict with base, ours and theirs. Delete against edit is one conflict. A leg removed on one side and bent on the
  other is no conflict: the removal wins and a note says so. Purple Pen deleting a course with classes is a conflict.
- **Stop lists** *(Claude's call)*. A diff3 over stop identities. A clean result counts only if it is a fixed point,
  meaning merging it again with either side returns it. Otherwise it is a conflict. Repeated identities (loops,
  butterflies) can make a diff3 unstable, and an unstable merge could add a loop after a crash. The rule also turns some merges that were clean before, those that combine a move
  with another edit, into conflicts. That is the safe direction; the reviewer counted 3,360 of 34.6M small triples.
- **Number placements** *(Claude's call)*. The model has one central placement per control, so "automatic in this
  course" is not a value. When Purple Pen leaves a visit automatic, the central placement moves onto the other visits
  of the control, in individual, team and relay courses, and the central one is dropped. A reset to automatic in Purple Pen therefore sticks. The verify step
  reads every number taken from Purple Pen back from the written file and refuses the sync on a mismatch.
- **Matching** *(user: code or name, ids second; Claude's call for the order)*. Purple Pen id, then code or name,
  within one kind. Project ids are distrusted after a base that belongs to another link, so `export --ppen NEW`
  followed by `sync` falls back to names. That gives conflicts, never a silent loss. A modeled object the writer did
  not match refuses the sync.
- **Open conflicts write nothing** *(Claude's call)*. `--take SIDE` settles all for one call. `--take KEY=SIDE` and
  `resolve_sync_conflict` store a decision with a fingerprint of both values; the next completed sync clears it.
- **Base copy and atomicity** *(Claude's call)*. `X.courseplan.ppen-base` beside the project file, without
  `absolute-path`, hashed in `[sync]` (format stays 1). Write order is `.ppen`, project, base copy, with a re-read of
  the `.ppen` before the first write. A missing, foreign or damaged base is ignored; every difference is a conflict.
- **Kept deletions** *(Claude's call)*. A control that a course of the merged project or of the linked file still
  visits is not deleted; a note says so.
- **Format gate** *(user)*. An unknown Purple Pen version refuses the sync, also in a dry run.
- **Only in courseplan** *(user)*. Classes, starters, accepted findings, print areas, team, relay and score courses,
  marked routes and multi-point forbidden routes.
- **Interfaces** *(Claude's call)*. CLI `sync PROJECT [--dry-run] [--take ...] [--json]`; exit 1 for conflicts or a
  refusal. MCP `sync_ppen` and `resolve_sync_conflict`. The MCP project link must end in `.ppen` and must not be a
  symlink.

## Known limits
- Purple Pen open during a sync: its later save drops what the sync added, and the next sync deletes those objects
  from the project without a conflict. The manual says to close the event first.
- Work is linear in the file size, except the pairing of many unmatched objects on both sides, which stays quadratic.
- The project file is not re-read before it is saved (a lost update if another tool wrote it meanwhile). Every
  command works this way; parked for the M4 editor plan.
- Core's id guard drops all event ids when one code or name contradicts the base, so a sync that swaps two codes and
  renames a course is refused. `PatchOptions::ids_first` is proposed to freemapper-print *(not agreed)*.
- The editor's conflict panel comes with the M4 editor plan and stores a decision through the MCP path.

## Consequences
- Every merge case has a test, among them a real Purple Pen 4.0 pair of files.
- The base copy is a new file in the repository of every user who syncs.

Source: spec §5.5a, ADR 0011, 0034, 0035, plan 2026-11-10-ppen-sync
