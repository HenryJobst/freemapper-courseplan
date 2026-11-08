# 0019. Project file format 1

Date: 2026-11-08
Status: Accepted

## Context

M1 writes the first project files (ADR 0008). People edit them by hand, and the program saves them back.

## Decision

- The file is TOML with `format = 1`. The event date is an unquoted TOML local date (`date = 2026-11-14`), and a
  control code is an integer when it is plain digits, else a string.
- Start and finish are special objects. Each course names its start and finish by id, outside its control list.
- A save matches controls by `code` and courses and classes by `name`, and writes them in the event's new order.
  It matches special objects and accepted findings by index.
- Loading rejects unknown keys. Every field added later raises `format`.
- Format 1 is provisional and carries no compatibility promise until the first release. Before then it needs gaps
  on legs and circles, the planned print area, ids and course assignment for special objects other than start and
  finish (for mandatory crossing points in the WKB length, the `.ppen` export and freemapper-print), team size per
  class, and relay variant names with their assignment to legs and teams.

## Consequences

A typo in a hand-edited file stops the load and names the table and key, so no value goes missing on the next
save. Comments stay with a control, course or class that the user moves. A special object or accepted finding
loses its comments when its list changes length. Cost if wrong: format 1 files need a migration before the first
release, and none exist outside the tests yet.

Source: final review of M1, 2026-11-08; ledger rulings R5, R9 (amended), R11 and R12; spec §5.7.
