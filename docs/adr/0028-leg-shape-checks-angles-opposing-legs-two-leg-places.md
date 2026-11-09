# 0028. Leg shape checks: angles, opposing legs and findings at two legs

Date: 2026-11-09
Status: Accepted

## Context
Plan M2b Task 5 adds `acute_angle` and `opposing_legs`. The WKB profile has two `acute_angle` rules (warning
below 45°, hint below 90°), and an opposing-legs finding concerns two legs, which no location could name.

## Decision
- Angles are measured at controls and crossing points, not at start or finish, from the last bend before to
  the first bend after. A direction without length gives no angle. The finding sits at the incoming leg.
- A place gets only the strictest rule of its key (R15): a rule stays silent where a sibling with a smaller
  `degrees` covers the course and reports the angle (`RuleRun::siblings`). Siblings with equal limits both
  report.
- `opposing_legs` compares every pair of legs of a course along their bends. The part of each leg within
  `metres` of a segment of the other whose direction differs by more than `min_angle_degrees` counts, and the
  shorter of the two parts must reach `min_length_metres`. The first `metres` around a stop both legs touch do
  not count; farther out, butterfly loops count like other legs, and so do taped legs.
- Legs that cross head-on at a shallow angle are opposing once their near stretch reaches the minimum length
  (R17): with 10 m and 30 m, below a crossing angle of about 41.8°.
- A sharp dog-leg can give both an `acute_angle` and an `opposing_legs` finding, since they are different keys.
- A finding at two legs has `Location::Legs { course, legs }` and place kind `Legs`, accepted with
  `--leg A:2 --leg A:5` or two MCP `legs` objects in any order; legs of two courses are refused. Each leg
  carries its position and its end stops (ADR 0025, amendment), and output names the pair as "legs 31–32
  and 35–36".

## Consequences
Cost if wrong: one condition for the strictest rule, one location variant. The values are proposals in the
profile (Claude's call).

Source: plan M2b Task 5; controller rulings R15, R16 and R17, 2026-11-09.
