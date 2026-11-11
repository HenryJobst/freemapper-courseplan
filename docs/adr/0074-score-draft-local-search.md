# 0074: Score draft: local search and faster legs

Status: Accepted, 2026-11-12. Amends [0065](0065-score-draft.md).

## Context

On a 1.8 x 2.0 km map of Berlin-Kreuzberg (183 sites) the draft of 0065 missed the good runner's band by 0.3 to 3.8
points at five of six starts and limits, and took 52 to 117 s. It tries only prefixes of one spread order with
quantile values, so it never sees a set of the same size that fits. A profile put the time in the route searches of
`score suggest`, one shortest-path search per pair of controls, and the tour heuristic.

## Decision

- Local search after the prefix search *(Claude's call)*. From the attempt with the smallest miss, a move swaps one
  control for an unused candidate, adds or drops one, or shifts one value. Moves are seeded by the settings' seed,
  bounded by 150 tour evaluations (`REFINE_BUDGET`), and kept when the band miss does not grow. The hard share only
  breaks ties (0.02 per point), so the bands still win over the mix (ruling 3).
- A value moves at most two steps off the quantile of its candidate *(Claude's call)*. Far controls stay worth more
  than near ones, but value can move to the controls the good runner skips, which lowers his share without changing
  the very good runner's time. A move that breaks the high-value gap or the codes is dropped before the tours run.
  Link ends stay. A draft that already meets the bands is returned unchanged.
- Legs: one search per point reads the times to all later points (`TravelTime::prefetch`), and the tours read them
  with `fastest_time`. The surface is symmetric, so one entry serves both directions *(Claude's call)*. `shortest`
  is A* with the straight line at the least cell factor as bound: same cost, equal-cost routes may differ.
- The nearest-leg searches of `suggest_with` run on up to eight threads, joined in key order, so the result does not
  change. `suggest` and `suggest_with` ask for `TravelTime + Sync`. The or-opt step of the heuristic (0064) no longer
  copies the tour; exact solving up to 12 controls stays.

## Consequences

Release build on a busy machine, CPU seconds per `score draft`: all six Kreuzberg drafts (three starts, 30 and 45
minutes) now meet both bands at 24 to 35 s; before, five missed at 44 to 63 s. The draft itself takes 1.4 to 5 s of
that, the rest is `score suggest`. The 15-minute Lübeck draft meets its bands. A pool too small stays a failure: on
Lübeck (31 sites) 30 and 45 minutes give the good runner every point. Values are no longer exact quantiles.
