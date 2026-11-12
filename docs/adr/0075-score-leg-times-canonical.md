# 0075: Score legs: one number per leg time

Status: Accepted, 2026-11-12. Amends [0074](0074-score-draft-local-search.md) and [0065](0065-score-draft.md).

## Context

On the Berlin-Kreuzberg example with the start at 40,-40 and the finish at 60,-40, `score draft` at 45 minutes reported
33 controls, 97 points and the very good runner at 99.8 percent. After `--apply`, `score length` and `check` found 96
points reachable and 100.7 percent of the limit for all controls. Starts at 0,0 and -30,10 agreed.

Both runs used the same coarse surface, the same positions and the same control order. The leg times differed only in
the last digits (about 1e-12 s): the draft reads times that `prefetch` took from one search per source, `length` and
`check` ask `fastest` per pair, and the same cost summed along a path from the other end, or by A* instead of
Dijkstra, rounds differently. The tour heuristic (ADR 0064) compares sums with strict `<` and breaks ties by
position, so a change in the last digit picks another local optimum: 97 points in one run, 96 in the other. The
surface scale (2 m or 4 m) was not the cause.

## Decision

- Every leg time of the surface is rounded to a millisecond where it is made, in `fastest` and in the one-source
  search *(Claude's call)*. A pair then has one number whichever search, direction or cache produced it, and the
  draft's evaluation equals the later one bit for bit. Half a millisecond on a leg of minutes does not move any
  band.
- No second verification pass in the draft *(Claude's call)*. With equal numbers it would repeat the same
  computation; the parked item of 0074 (draft on the 2 m surface, check on 4 m) stays parked, because the draft
  uses the surface of `length` and `check`.
- Regression test: leg times from `times_from`, `fastest` forth and `fastest` back are equal as floats on a
  synthetic surface.

## Consequences

Results of `score draft` can change in single controls compared with 0.13.0, since tie-breaks shift once. The
three Berlin starts and the Lübeck 15-minute example now give identical numbers in the draft, `score length` and
`check`. The start 40,-40 yields 32 controls and 94 points with the good runner at 92.6 percent and the very good
runner at 99.9 percent. Run time does not change.
