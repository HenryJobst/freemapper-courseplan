# 0066. Score courses: the checks

Date: 2026-11-11
Status: Accepted
## Context
`check` skipped score courses. Design 2026-11-11 §6 asks for rules with a source (ADR 0026).

## Decision
- Eight rule keys in `wkb-2026`, `score_value_matches_code` to `score_mix`. They read the score courses only; team and
  relay courses stay skipped *(Claude's call)*.
- Sources follow research note §3.7: ring and order BOF B12.3 and B12.6.4, length bands BOF B12.5.3 and the IRF, limit
  and penalty BOF B12.5. The 90 percent target and the value-in-code rule come from the project owner *(user)*.
  Ambiguity and links are *(Claude's call)*.
- The limit range is 30 to 90 minutes, wider than B12.5 ("usually 1 h, at least 45 min"), as any value goes *(Claude's call)*.
- Ring: the largest bearing gap seen from the middle of start and finish is under 120 degrees and the distances vary
  by under 25 percent. Order: 60 percent of the controls are hull corners and the nearest-control chain is at most
  10 percent above its 2-opt improvement. Both need six controls; the thresholds are rule values *(Claude's call)*.
- Length bands run `analyse_length` on a `CostSurface` with 4 m cells, blocked by out-of-bounds and dangerous areas.
  Both runners share one search per pair (their times differ by a constant factor); 60 controls took 2.7 s. A map
  without objects, or a failed search, skips the rule with a reason.
- Links need two different controls of the course and no repeat. A straight line across an out-of-bounds area or a
  forbidden route is a warning, since a runner takes a route and not the line. A leg without a second similar route is
  a hint. The file has no flag for that, so the setter accepts the hint with a note *(Claude's call)*.
- Mix: hard means a value in the top two fifths; judged from five controls, tolerance 15 points. A test shows that
  drafts pass at the default settings *(Claude's call)*.

## Consequences
- Rule ids are `wkb2026.score.*`; `finding accept` works at the course, a control or a pair of controls.
- Gaps: "hard" is difficulty in the draft and value in `check`, so the default mix passes. The profile's minimum
  distances do not apply to score controls (design §6).
