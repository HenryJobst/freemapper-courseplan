# 0040. The Purple Pen sync is a three-way merge on the Purple Pen view

Date: 2026-11-10
Status: Accepted

## Context
Design section 5.5a asks for `sync`: parallel use of courseplan and Purple Pen, changes flowing both ways, no silent
overwrite *(user)*. The mechanism is open. ADR 0034 (import) and ADR 0035 (patch export) exist; the project keeps
Purple Pen ids (`ppen_id`).

## Decision
- **One frame for all three versions** *(Claude's call)*. Base, theirs and ours are all what the import makes of a
  `.ppen`: base = import(base copy), theirs = import(linked file), ours = import(export(project onto the base copy)).
  Purple Pen's added defaults and the model's lossy mapping then never show up as changes. The merge compares on the
  import's own grid (µm, hundredths of a degree, whole metres of climb), with no tolerance on top.
- **Fields** *(Claude's call)*. Control: code, kind, position, each description column. Special: shape. Course:
  name, stop list, climb, print scale, and the shown number placement per course, control and visit. Leg: bends and
  marking, event-wide per ordered pair. A missing value is a value, so a deletion is a change.
- **Pick rule** *(user for the principle, Claude's call for the details)*. Unchanged, one side, the same change on
  both sides, or a conflict. A conflict lists base, ours and theirs. Delete against edit is one conflict on the object.
  Purple Pen deleting a course that has classes is a conflict.
- **Stop lists** *(Claude's call)*. A diff3 over the stop identities. Changes that do not overlap both land; overlapping
  ones that differ are one conflict.
- **Matching** *(user: code/name with ids as second key; Claude's call for the order)*. Purple Pen id, then code or
  name, always within one kind; new objects on both sides by code, name or shape. Ids that are not in the base are
  never trusted. The event sent to the writer carries exactly theirs' id per matched object. A modeled object the
  writer did not match refuses the sync (`Unmatched`).
- **Open conflicts write nothing, not even the clean part** *(Claude's call)*. A byte copy cannot express a
  half-synced state. `--take SIDE` settles every conflict for one call. `--take KEY=SIDE` and the MCP tool
  `resolve_sync_conflict` store a decision with a fingerprint of both values; it holds only while they stay the same
  and the next completed sync clears it.
- **Base copy** *(Claude's call)*. `X.courseplan.ppen-base` beside the project file, committed to git with it. Sync
  strips the `absolute-path` attribute of `<map>` from the copy, because it holds the author's home path and is no
  data the merge compares *(Claude's call, 2026-11-10)*. The `[sync]` table (`ppen`, `base_fnv1a64` over the stored
  bytes, `decisions`) is additive; the format stays 1.
- **Atomicity** *(Claude's call)*. Write order: `.ppen`, project, base copy. The `.ppen` is read again just before
  the write; if Purple Pen saved meanwhile, nothing is written. A crash after the `.ppen` write converges on the next
  sync. A crash after the project write is repaired (base := theirs). A missing, foreign or damaged base is ignored:
  nothing is deleted and every difference is a conflict. A first sync has no base and says so.
- **Deletions** *(Claude's call)*. A control that a merged course or any course of the linked file still visits is
  not deleted, and a note says so.
- **Format gate** *(user)*. An identity patch runs core's shape check first. An unknown Purple Pen version refuses
  the sync, also in a dry run.
- **Stays only in courseplan** *(user)*. Classes, starter counts, accepted findings, print areas, team, relay and
  score courses, marked-route specials and multi-point forbidden routes. The export names what it skips.
- **Interfaces** *(Claude's call for the flags)*. CLI `sync PROJECT [--dry-run] [--take ...] [--json]`; exit 0 when
  synced or nothing to do, 1 for open conflicts or a refusal. MCP `sync_ppen` and `resolve_sync_conflict`.

## Known limits
- The model has one central number placement per control. Sync writes it into every course that visits the
  control, so a sync can add `number-location` lines to courses nobody edited. Purple Pen's automatic placement
  cannot be represented.
- Courses the import skips (variations, score) are not merged. They stay in the file.
- Core's id guard drops all event ids when one code or name contradicts the base, for example two swapped codes. A
  course renamed in the same sync then becomes new and the old one stays; `Unmatched` turns that into a refusal.
  Proposed to freemapper-print, not yet agreed *(not agreed)*: `PatchOptions::ids_first`, "the caller vouches that
  the ids belong to this base". The sync does not depend on it; core stays at v0.7.0.
- The editor's conflict panel comes with the M4 editor plan. It will store a `SyncDecision` through the same
  `resolve` path as MCP.

## Consequences
- Every merge case has a test, among them a real Purple Pen 4.0 pair of files (`real_purple_pen_4_edits_merge`).
- The base copy is a new file in the repository of every user who syncs.

Source: spec §5.5a, ADR 0011, 0034, 0035, plan 2026-11-10-ppen-sync
