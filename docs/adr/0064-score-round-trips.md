# 0064. Score courses: round trips and length analysis

Date: 2026-11-11
Status: Accepted

## Context
The length analysis (design 2026-11-11 §4.4) needs the best round trip within the time limit for two runner
profiles on one set of controls, with mandatory links (§3) and the same answer on every run (§9 rulings 1, 2, 4, 8).

## Decision
- Leg times are read once per pair, start and finish included, with `TravelTime::fastest` into a symmetric matrix, so
  a pair is asked in one direction only *(Claude's call)*. A link leg takes the faster direction *(user)*. A pair
  without a route costs infinity.
- A link puts both controls in the tour, one directly after the other, either way round *(user)*. Links that share a
  control form a chain run as one unit. Three links at a control, or a ring, give `LinksNotAChain`; a missing control
  or the same control twice gives `BadLink` *(Claude's call, design §3)*.
- The tour maximises points within the limit, start and finish legs included, then takes the shorter time *(Claude's
  call)*. Equal tours of the exact table go to the lower index order; that is no promise across all tours.
- Up to 12 controls the solver is exact, a dynamic programme over subsets of units. Above that a heuristic inserts
  cheapest first, shortens with 2-opt, moves units and swaps one or two for another. A xorshift seeded by
  `ScoreSettings::seed` drives 16 restarts; no dependency is added *(Claude's call)*.
- With no tour for the links, `LinkDoesNotFit { a, b }` names the worst link. It is a proof when the search was
  exact, and also when at most 12 units are linked, since the heuristic solves those on their own first. With more,
  the heuristic may miss a tour and the error is `LinkTourNotFound { a, b }`, a possible false negative.
- `LengthReport` holds the good runner's share of all points and the very good runner's finish as a share of the
  limit. Issues use the bands of `ScoreSettings`. A very good runner who misses controls gets the missing points and
  how far over the limit every control would run.

## Consequences
- The heuristic can miss the optimum above 12 controls. The matrix needs both directions if legs become one-way.
