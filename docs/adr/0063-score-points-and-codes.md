# 0063. Score courses: point values and codes

Date: 2026-11-11
Status: Accepted
## Context
The draft needs a value from 1 to `value_max` for each candidate, and a code that carries it (ADR 0060) without
clashing with codes already on the map (design 2026-11-11 §3, §4.3, §9 rulings 3, 8).

## Decision
- A hard control brings more than an easy one at the same distance, but not so much that only hard controls pay
  *(user)*. The score is the travel time from the start, relative to the farthest candidate, weighted 0.7, plus the
  difficulty weighted 0.3. The 0.3 is the constant `DIFFICULTY_SHARE_MAX` *(Claude's call)*. A hard control next to the
  start therefore never outranks an easy one at the far end.
- The values are quantiles of the candidate set *(Claude's call)*. Candidates are ordered by score, equal scores by
  index, and cut into `value_max` bins of equal size. All-equal scores therefore spread over the values by index,
  and a set smaller than `value_max` uses the lower steps (3 candidates at 5 give 1, 2 and 4). The result does not
  depend on absolute times, and every run gives the same answer *(user, ruling 8)*.
- The tens digit is chosen per control, lowest first, from 3 to 9. A code is skipped when it is taken, used
  already in the set, or reads alike turned by 180 degrees or by 6 and 9 against a taken code or a chosen one. Controls
  are served top to bottom, then left to right, so low tens digits run in reading order.
- Too many controls give `CodeSpaceExhausted { limit }` with the number of free codes, or
  `ValueSpaceExhausted { value, limit }` when one value has fewer free tens digits than controls. A value outside
  1 to `value_max` or a position list of the wrong length is refused.
- Clash-free capacity is below `code_space`. The 6/9 rule makes 65 and 95 alike, so the tens digits 6 and 9 exclude
  each other for one value: at most 30 clash-free controls at `value_max` 5, and values 6 and 9 share a pool at 9.
  When a set needs a clash (63 controls use every code), it goes to the pair of controls that lie farthest apart
  *(Claude's call)*. A clash with a taken code is never accepted.
## Consequences
- Quantiles spread the values evenly even when all candidates have alike effort.
- The 0.3 share is a guess and sits in one constant.
