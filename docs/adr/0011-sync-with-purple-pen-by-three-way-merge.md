# 0011. Sync with Purple Pen by three-way merge

Date: 2026-11-08
Status: Accepted

## Context

The user moves over from Purple Pen once this program has everything needed. Until then both programs are used
side by side, and other course setters may keep using Purple Pen in the long run. `.ppen` is open XML with
stable ids.

## Decision

An event can link one `.ppen`. `sync` keeps a copy of the file from the last sync as the base and compares base,
Purple Pen now and courseplan now per object. Controls match by code, courses by name, with Purple Pen's ids as a
second key. A change on one side is taken over; different changes on both sides are listed as conflicts and
wait for the user. Data Purple Pen cannot hold stays in the project file. Unknown `.ppen` versions are never
written.

## Consequences

- No silent loss on either side.
- Cost if wrong: Purple Pen format changes break the reader; Purple Pen 4 is tested when released.

Source: brainstorming 2026-11-08 (user: parallel use, long-term exchange); design §5.5a.
