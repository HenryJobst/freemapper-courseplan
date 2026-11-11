# 0065. Score courses: the draft

Date: 2026-11-11
Status: Accepted
## Context
`score draft` picks controls so that both runner profiles land in their bands and writes nothing itself (design
2026-11-11 §4.4, §9 rulings 1 to 4 and 8).

## Decision
- Candidates with no route from the start are dropped; the rest get a fixed order: requested link ends first, then
  always the candidate farthest from start, finish and everything taken so far. Ties go to the lower index and nothing
  is hashed, so runs are reproducible *(Claude's call)*. A set of size k is the first k of that order.
- A link end must lie within half a map millimetre of a candidate, otherwise the draft fails with
  `LinkEndNotACandidate { link }`. Both ends join the set. A link that does not fit the limit fails with the tour
  error and `LimitUp`.
- The mix share steers the order: every prefix holds about `mix_hard_percent` hard candidates (difficulty from 0.5).
  If the bands are missed with the mix, the search runs once more without it. A draft found that way reports the
  missed share in `mix_missed_percent`; effort wins *(Claude's call, ruling 3)*.
- Values are the quantiles of the set of each count; codes are assigned per count after the set is chosen *(Claude's
  call, ruling 3)*. A count whose codes cannot be assigned is skipped, so no draft has empty codes.
- The count comes from bisection for the largest k the very good runner still covers, then a scan of up to ten
  counts below it for the first without issues *(Claude's call)*. The analysis is the one of ADR 0064.
- A miss returns `DraftFailure` with the attempt of smallest total miss, its issues and a hint: `LimitUp` when the
  very good runner cannot cover the set or needs more than the limit (`VeryGoodOverLimit`), `MoreCandidates` when all
  candidates are in and the set is still too short, `FasterProfile` when only the good runner is short, otherwise
  `LimitDown`. `Impossible` comes with a `ScoreError` for inputs that cannot work. A start-finish leg longer than the
  limit gives `LimitUp` at once.

## Consequences
- Prefixes of one order are nested, so a better set of the same size can exist.
