# 0043. Editor operations, revision guard and undo

Date: 2026-11-11
Status: Accepted

## Context
The page changes the project through the server. Other programs (the CLI, an MCP client, Purple Pen's sync, a
text editor) can write the same file while the page is open. The editor must not repeat the validation of the MCP
tools, must not overwrite a change it has not seen, and must let the user take steps back without losing
anything another program wrote.

## Decision
- **The operations are the MCP tools** *(Claude's call)*. `POST api/op/{tool}` takes the tool's arguments as a
  JSON object without `project`, and runs `CoursePlanServer::call_json` on the project file. Parameters, checks,
  results and texts are the tool's, redaction included. Only the tools in `EDITOR_TOOLS` run; any other name,
  including those that take file paths, answers 404. The API has no route that takes a file name.
- **Revision guard** *(Claude's call)*. The revision is the FNV-1a 64 fingerprint of the file's bytes. Each
  operation carries the revision the page saw in `X-Courseplan-Revision`. The server reads the file, compares,
  and answers 409 with the current revision when they differ; nothing is written. A save the tool makes can
  still fail with `ChangedMeanwhile` (ADR 0044) when a write lands in between; that is a 409 too.
- **Status codes**: 200 `{ revision, result }`; 400 for a body that is no JSON object, a missing revision header
  or arguments the tool does not take; 404 for a tool outside the list; 409 `{ error, revision }`; 422
  `{ error }` when the tool refused the edit, with the file byte-identical.
- **One operation at a time**. Each operation runs on the blocking pool while it holds the state lock: read
  bytes, compare, run the tool, read bytes, record the step. Concurrent posts queue; with the same revision, the
  first wins and the rest get 409. Right before the tool runs, the bytes are read once more and compared with
  those the step will record as "before"; a write in that gap answers 409. Undo and redo carry the revision
  header too and answer 409 when it is not the file's current revision, so a second tab cannot revert the first
  tab's edit unseen.
- **A broken file blocks edits** *(Claude's call)*. When the project file does not load, every operation answers
  409 with the load error and writes nothing, until the file loads again.
- **Undo by snapshots** *(Claude's call)*. The history is one chain of whole-file texts, a label from the catalog
  (`editor-op-<tool>`) between each pair, at most 100 steps and at most 64 MiB of text in all; the oldest steps go
  first, the newest always stays. Undo writes the previous text and redo the next one, atomically (the shared
  `write_atomic`: temporary file, then rename), and only while the file still equals the text the step started from. If it does
  not, nothing is written, the history is cleared and the answer is 409. Snapshots keep comments and layout
  exactly and need no inverse operation per tool. `api/state` carries the labels of the next undo and redo.
- **I/O errors** of a tool (the project cannot be read or written) answer 500 with a generic localized text;
  texts that are too long (codes over 64, names and notes over 200 characters) are refused by the model's
  validation, shared by CLI, MCP and editor.
- **External changes**: no watcher; the page polls `api/revision` (ADR 0046).

## Consequences
- Every operation parses the map for its checks, as the MCP tool does. A very large map makes each edit slower.
- The window between undo's comparison and its rename is not closed; another writer in that window is lost.
  The save guard of ADR 0044 has the same limit.
- Undo of a step restores bytes, so it also takes back changes the tool made to other parts of the file.
- Operations that write beyond the project file (sync, import, export to the linked `.ppen`) are separate routes
  in a later task and clear the history.

Source: browser editor plan M4, D4, D5, D7 and D8 (2026-11-11).
