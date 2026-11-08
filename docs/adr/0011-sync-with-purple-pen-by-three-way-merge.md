# 0011. Sync with Purple Pen by three-way merge

Date: 2026-11-08
Status: Accepted

## Context

The user moves over from Purple Pen once this program has everything needed. Until then the user runs both programs
side by side, and other course setters may keep using Purple Pen in the long run. `.ppen` is open XML with
stable ids.

## Decision

An event can link one `.ppen`. `sync` keeps a copy of the file from the last sync as the base and compares base,
Purple Pen now and courseplan now per object. Controls match by code, courses by name, with Purple Pen's ids as a
second key. `sync` takes over a change made on one side and lists different changes on both sides as conflicts that
wait for the user. Data Purple Pen cannot hold stays in the project file. `sync` never writes a `.ppen` of an
unknown version.

## Consequences

- Neither side loses data silently.
- Cost if wrong: Purple Pen format changes break the reader; this project tests Purple Pen 4 when it comes out.

Source: brainstorming 2026-11-08 (user: parallel use, long-term exchange); design §5.5a.
