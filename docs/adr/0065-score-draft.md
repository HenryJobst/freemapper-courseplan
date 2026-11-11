# 0065. Score courses: the draft

Date: 2026-11-11
Status: Accepted

## Context
`score draft` picks controls from the candidates so that both runner profiles land in their bands (design
2026-11-11 §4.4, §9 rulings 1 to 4 and 8) and writes nothing itself.

## Decision
- Candidates without a route from the start are left out. The rest are put in a fixed order: requested link ends
  first, then always the candidate farthest from start, finish and what is taken. A set of size k is the first k
  of that order *(Claude's call)*. Ties go to the lower index; no hashing is involved, so every run is the same.
- A link end must lie within half a map millimetre of a candidate, else the draft fails with `MoreCandidates`. Both
  ends are added to the set, and both runners must run the link, or the failure carries the tour error with `LimitUp`.
- The mix share steers that order: every prefix holds about `mix_hard_percent` hard candidates (difficulty from
  0.5). If the search does not meet the bands with the mix, it runs once more without it. A draft found that way
  names the missed share in `mix_missed_percent`; effort wins *(user, ruling 3)*.
- Per count k the values are the quantiles of that set (`assign_values`); codes come only after the set is final
  *(user, ruling 3)*.
- The count is found by bisection for the largest k that the very good runner still covers completely, then a scan
  of up to ten counts below it for the first with no issue *(Claude's call)*. The length analysis is the one of ADR 0064.
- A miss returns `DraftFailure` with the attempt of smallest total miss, its issues and a hint: `LimitUp` when the
  very good runner cannot cover the set, `FasterProfile` when only the good runner is short, `MoreCandidates` when
  all candidates are in and the set is still too short, else `LimitDown`. `Impossible` carries a `ScoreError` for
  inputs that cannot work (no route from the start, links that form no chain). A start-finish leg longer than the
  limit gives `LimitUp` at once.

## Consequences
- The sets are nested, so a count that fits is easy to explain, but a better set of the same size can exist.
- At most `code_space` controls are tried; more need more digits in the code.
