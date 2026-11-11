# 0064. Score courses: round trips and length analysis

Date: 2026-11-11
Status: Accepted
## Context
The length analysis (design 2026-11-11 §4.4) needs the best round trip within the time limit for two runner
profiles on one set of controls, with mandatory links (§3) and the same answer on every run (§9 rulings 1, 2, 4, 8).

## Decision
- Leg times are read once per pair of controls, start and finish with `TravelTime::fastest`, into a matrix. The
  solvers search no map. The matrix is symmetric, so a pair is asked in one direction only *(Claude's call)*. A link
  leg takes the faster of both directions *(user)*. A pair without a route costs infinity, and only a missing direct
  start-finish route is an error.
- A link means both controls are in the tour and run one directly after the other, either way round *(user)*.
  Links that share a control are merged into a chain that is run forwards or backwards as one unit. A control with
  three links, or a ring, is refused with `LinksNotAChain` *(Claude's call, design §3)*. A link naming a missing
  control or the same control twice gives `BadLink`.
- The tour maximises points within the limit, start and finish legs included. Ties go to the shorter time, then to
  the lower running order of control indices *(Claude's call)*.
- Up to 12 controls the solver is exact, a dynamic programme over subsets of units. Above that a heuristic builds a
  tour by cheapest insertion and shortens it with 2-opt and by moving single units. A last pass swaps one or two
  units for another one. A xorshift seeded by `ScoreSettings::seed` shuffles the order of a dozen restarts; no dependency is added
  *(Claude's call)*. `best_tour` takes the seed of the default settings, `best_tour_seeded` and `analyse_length` the
  one given.
- If no tour holds all links, the call fails with `LinkDoesNotFit { a, b }`. It names the link of the unit that
  costs most on its own and, in a chain, the link whose nearer end has the longest detour via start and finish.
- `LengthReport` gives the good runner's share of all points and the very good runner's finish as a share of the
  limit. Issues use the bands of `ScoreSettings` (87 to 93 and 95 to 100 by default). A very good runner who misses
  controls gets the missing points and, if a tour with every control exists, how far over the limit it runs.
## Consequences
- The heuristic can miss the optimum above 12 controls. On ten controls it matched the exact solver in the tests.
- Symmetry breaks if a later route analysis makes one-way legs; the matrix then needs both directions.
