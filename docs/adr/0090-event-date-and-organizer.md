# 0090: The event has a date and an organizer, and the IOF XML export carries them

Status: Accepted, 2026-11-13. Follows [0060](0060-score-settings-links-and-code-value.md) and
[0089](0089-rejected-sites-per-project.md) in how a field is added.

## Context

freemapper-print prints a title block with the date and the organizer of the event and asked for both
*(request from freemapper-print, 2026-11-13)*. The project file had `date` in `[event]` since the first format, but
nothing set it, and no output used it. There was no organizer at all, so the IOF XML export wrote only
`Event/Name`.

## Decision

- The event gets an optional organizer, a text, and a date that a person can set. Both are shown in `info` and
  `event_info` and go into the IOF XML export *(user, 2026-11-13)*.
- Project file *(Claude's call)*: `[event] date = 2026-11-14` (the existing unquoted TOML date, `YYYY-MM-DD`, no
  time and no offset) and the new `[event] organizer = "..."`. Both are written only when set. The organizer is
  not empty, has no white space at its start or end, no invisible control characters and at most 200 characters,
  like a course name. The format stays 1 and no older file needs a change, as for the score settings (ADR 0060) and
  the rejected sites (ADR 0089). A program from before this version refuses a file with `organizer`, because it
  refuses unknown keys (ADR 0019).
- IOF XML 3.0 *(Claude's call)*: `Event/Name` stays. A set date becomes `Event/StartTime/Date` (`YYYY-MM-DD`, no
  time of day), a set organizer becomes `Event/Organiser/Name`. The children of `Event` keep the order of the
  schema: `Name`, `StartTime`, `Organiser`. Nothing is written for a field that is not set.
- Purple Pen *(Claude's call)*: a `.ppen` event has a title but no date and no organizer element. Export, import
  and `sync` leave both fields alone, so a sync never overwrites or clears them.
- Interfaces *(Claude's call)*: `event set PROJECT [--date YYYY-MM-DD] [--organizer TEXT] [--no-date]
  [--no-organizer]` on the command line (at least one option; `--date` with `--no-date`, or `--organizer` with
  `--no-organizer`, is a usage error), the tool `set_event_details` over MCP with `date`, `organizer`, `no_date` and
  `no_organizer`, and two text fields "Date" and "Organizer" in the event settings of the editor. The organizer is
  trimmed. A wrong date or an empty organizer is refused and leaves the file as it was.
- The score evaluation sheet and the other exports do not change.

## Consequences

- The tool count of the MCP server goes from 54 to 55.
- The examples keep no date and no organizer: both would have to be invented for a training map that has no
  organizer.
- A reader of the IOF XML sees a date without a time of day. Time of day and time zone are the business of the
  start list, not of the course data.

## Cost if wrong

- OE or another reader may want `StartTime/Time` too. It is optional in the schema; adding it later needs a
  field for it.
- Two events on different days of one project are not possible; one project is one event.

Source: freemapper-print's request and the user's decision, 2026-11-13.
