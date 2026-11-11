# 0050. Edits keep accepted legs and sync decisions; bend limits; no control characters

Date: 2026-11-11
Status: Accepted

## Context
The stage 3 review of the browser editor found that edits a user now makes with the mouse left stored references
behind, accepted bends outside the map, and let a text field write characters that break the IOF XML export.

## Decision
- **Accepted legs follow stop edits** *(Claude's call)*. `set_course_controls` and `remove_control` move the
  `after` of an accepted leg to where the leg between the same two stops now runs (the n-th occurrence of a pair
  stays the n-th). A leg that no longer exists loses its acceptance, and the answer lists it: `dropped` in the
  library, `dropped_acceptances` in MCP, one line per leg in the CLI, a note in the editor status. A leg written
  without ends and a leg that did not match the course before the edit stay as they are. This replaces the
  earlier behaviour, where the acceptance came back as a hint once the leg ran between other stops (ruling R24 of
  plan M2b, ADR 0026); an acceptance that matches nothing can still arise from a file edited by hand.
- **Renames rewrite sync decisions** *(Claude's call)*. `rename_control` and `rename_course` rewrite the name
  segment of stored `sync.decisions` keys (`control/<code>/…`, `course/<name>/…`, the control of a number key and
  the ends of a leg key), with the escaping of `sync::decide`. A decision therefore keeps its conflict after a
  rename.
- **Bends** *(Claude's call)*. `set_leg` checks every bend against the map extent like `add_control` and refuses
  more than 200 bends on one leg (`TooManyBends`). The editor shows the refusal and snaps the bend back (HTTP 422).
- **No control characters in names** *(Claude's call)*. Codes, course and class names and special ids may not
  contain a control character (`char::is_control`), a zero-width or direction character (U+200B to U+200F, U+202A
  to U+202E, U+2066 to U+2069) or U+FEFF; validation refuses them with `InvalidCharacter`. The IOF XML writer
  also refuses any character that XML 1.0 forbids, whatever the text, as the `.ppen` writer does, so it can no
  longer write an ill-formed file.
- **Pickers for rules** *(Claude's call)*. `GET api/rules` lists the profile ids, the overlay ids with their base
  and the rule categories each profile names. It reads the embedded rules only and names no file. The page uses
  drop-down lists for profile, overlays and rule category. A start window can be removed (`no_start_window`).

## Consequences
- An accepted finding does not come back as a hint after a reorder; the user sees the note instead.
- A project file that holds a control character in a name no longer loads. Such a file is rare and was a
  problem for the exports before.

Source: stage 3 review of the browser editor, 2026-11-11.
