# 0066. Score courses: the checks

Date: 2026-11-11
Status: Accepted

## Context
Score courses were skipped by `check`. Design 2026-11-11 §6 asks for rules that name a source (ADR 0026).

## Decision
- Eight rule keys, all in `wkb-2026`: `score_value_matches_code`, `score_code_ambiguity`, `score_ring`, `score_obvious_order`,
  `score_length_bands`, `score_limit_and_penalty`, `score_link_valid`, `score_mix`. They read the score courses of
  the context only; individual courses never reach them, team and relay courses stay skipped *(Claude's call)*.
- Value and code agree, the length bands and the mix cite *(user)* (`[sources.user]`, shown as "decision of the project owner").
  Ambiguity and links are *(Claude's call)*. Ring and order cite BOF B12 as the design names it, the limit and penalty
  cite the IRF Rules of Rogaining without a section; no section number was verified.
- Ring: the controls enclose the middle of start and finish (largest bearing gap below 120 degrees) at distances that
  vary by less than 25 percent. Order: at least 60 percent of the controls are corners of their convex hull and the
  nearest-control chain is at most 10 percent longer than its 2-opt improvement. Both need six controls; the thresholds are
  rule values *(Claude's call)*.
- Length bands use `analyse_length` on a `CostSurface` of the map, blocked by out-of-bounds and dangerous areas, at a
  4 m cell (`cell_metres`). One search per pair of points serves both runners, as their times differ by a constant factor.
  60 controls on a 360 by 340 mm map took 2.7 s in a release build. A map without objects, or a failing route search,
  skips the rule with a reason.
- Limit 30 to 90 minutes is usual; a penalty below the highest control value of the course is a warning.
- Links: two different controls of the course, no repeat, the straight line crosses no out-of-bounds area or forbidden
  route (a warning: a runner takes a route, not the line; unlike for legs, the design lists forbidden routes here), and
  a second route of similar length exists. The project file has no "no route choice" flag; the setter accepts the hint
  with a note *(Claude's call)*.
- Mix: a control is hard when its value is in the top two fifths of the range (`value × 5 > value_max × 3`), judged
  from five controls, tolerance 15 percentage points *(Claude's call)*.

## Consequences
- Rule ids are `wkb2026.score.*`; `finding accept` works on them at the course, control or pair of controls.
- Hardness comes from the value, not from the candidate difficulty of the draft, which a course of the file lacks.
