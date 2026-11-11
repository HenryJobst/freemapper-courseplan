# 0063. Score courses: point values and codes

Date: 2026-11-11
Status: Accepted

## Context
Candidates carry an effort and a difficulty. The draft needs a value from 1 to `value_max` for each of them, and a
code that carries the value (ADR 0060) without clashing with codes already on the map (design §4.3, §9 rulings 3, 8).

## Decision
- A hard control brings more than an easy one at the same distance, but not so much that only hard controls pay
  *(user)*. The score is the travel time from the start, relative to the farthest candidate, weighted 0.7, plus the
  difficulty weighted 0.3. The 0.3 is the constant `DIFFICULTY_SHARE_MAX` *(Claude's call)*. A hard control next to the
  start therefore never outranks an easy one at the far end.
- The values are quantiles of the candidate set *(Claude's call)*. Candidates are ordered by score, equal scores by
  index, and cut into `value_max` bins of equal size. The result does not depend on the absolute times, so it works
  for a small park and for a large campus alike, and every run gives the same answer *(user, ruling 8)*.
- The tens digit is chosen per control, lowest first, from 3 to 9. A code is skipped when it is taken, used
  already in the set, or reads alike turned by 180 degrees or by 6 and 9 against a taken code or a chosen one. Controls
  are served top to bottom, then left to right, so low tens digits run in reading order.
- More controls than `code_space(value_max)`, or more of one value than it has tens digits, give
  `CodeSpaceExhausted { limit }`.
- A full set cannot avoid every turned pair: 63 controls at value 9 use all codes, 69 and 96 among them. Then
  the clash goes to the two controls that lie farthest apart *(Claude's call)*. A clash with a taken code is never accepted.

## Consequences
- Quantiles spread the values evenly even when the effort of all candidates is alike, so a flat map still gets
  the full range. A set smaller than `value_max` uses only the lower steps.
- The 0.3 share is a guess. It sits in one constant if trials on real maps ask for another.
Source: design 2026-11-11 §3, §4.3
