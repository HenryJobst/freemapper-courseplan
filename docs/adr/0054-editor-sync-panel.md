# 0054. Editor: sync panel with conflict choices

Date: 2026-11-11
Status: Accepted

## Context
`sync` merges the project with its linked Purple Pen file (ADR 0040). The editor needs the same from the Event
menu, including the conflict choices the CLI takes with `--take` and the MCP server with `resolve_sync_conflict`.
ADR 0042 forbids a path from the browser; ADR 0053 settled how exchange routes work.

## Decision
- **Same link, no names** *(user)*. The sync uses `sources.ppen` and nothing the page sends. The link checks are
  those of the MCP sync: the name ends in `.ppen`, and a symbolic link is refused. Without a link the answer is 422
  with a text that names `export --ppen FILE`. A missing linked file is 422 as well.
- **One route** *(Claude's call)*. `POST api/sync {"dry_run": bool, "take": "ours" | "theirs" | null}` runs
  `sync_file` and answers `sync_json` plus `"revision"`. Unknown fields, other values and a missing `dry_run` are
  400. A real sync needs the revision header (400 without, 409 when stale); a dry run does not, because it writes
  nothing. `take` is the CLI's `--take ours|theirs` for this call: every open conflict goes to that side.
- **Open conflicts write nothing** *(user, as in the CLI)*. A real sync with conflicts answers 200 with
  `written: false` and the conflict list.
- **Choices are an operation** *(Claude's call)*. A choice for one conflict goes through
  `POST api/op/resolve_sync_conflict {key, take}`, which stores the decision in the project file and answers the
  dry run with the decisions applied. It is a normal step with its own undo entry. The page shows the new
  conflict list from that answer.
- **History** *(plan D7)*. A sync that wrote files clears the undo history. A sync that wrote nothing keeps it.
- **Races** *(Claude's call)*. If Purple Pen saves the linked file during the sync, nothing is written and the answer
  is 409 with the CLI's text. If another program writes the project file, the save guard fails and the answer is 409
  with the stale-revision text; when the linked file was already written, the sync can be run again and finishes.
  Both answers carry the current revision, so the page loads the project again.
- **Errors do not quote the file** *(Claude's call)*. A Purple Pen file that does not parse or does not fit the map
  gets one fixed text, as in the MCP server. Paths shrink to file names.
- **Warning before the first sync** *(user)*. "Close this event in Purple Pen before you sync. A later save from
  Purple Pen drops what the sync added." The checkbox "Don't show again this session" is kept in
  `sessionStorage`; the page works when storage is blocked and then asks every time.
- **Panel** *(user for the choices; Claude's call for the layout)*. The dry run shows changes and conflicts with key,
  before, courseplan and Purple Pen. Each conflict has two buttons; "All courseplan" and "All Purple Pen" sync at
  once. "Sync" is enabled when no conflict is open and something changed. Notes and the result stay in the dialog.

## Consequences
A person who edits in Purple Pen and the editor at the same time still has to close the event in Purple Pen; the
warning is the only protection. Values in the conflict list are shown as the report holds them (text, numbers or
compact JSON), not worded like the CLI's lines.
