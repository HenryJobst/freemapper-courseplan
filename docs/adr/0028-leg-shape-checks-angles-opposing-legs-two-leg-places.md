# 0028. Leg shape checks: angles, opposing legs and findings at two legs

Date: 2026-11-09
Status: Accepted

## Context
Plan M2b Task 5 adds `acute_angle` and `opposing_legs`. The WKB profile has two `acute_angle` rules, a
warning below 45° and a hint below 90°, so one sharp angle would break both. An opposing-legs finding is about
two legs, and a location could name only one.

## Decision
- An angle is measured at each control and crossing point of a course, not at start or finish, from the last
  bend before it (or the stop before) to the first bend after it (or the next stop). A direction without
  length gives no angle. The finding sits at the incoming leg and names the control and the angle, rounded
  down to whole degrees. Each visit of a butterfly control is measured on its own.
- A place gets only the strictest rule of its key (ruling R15). A rule of `acute_angle` stays silent where a
  sibling with a smaller `degrees` checks the course and reports the angle, so 30° gives the warning only, and
  accepting the warning does not bring up the hint. The engine hands every check the other rules of its key
  that apply to the event (`RuleRun::siblings`).
- `opposing_legs` compares every pair of legs of a course, also legs that do not follow each other, each as the
  polyline through its bends. A segment pair counts where its directions differ by more than
  `min_angle_degrees`. The length that counts is the part of each leg within `metres` of such a segment of the
  other, and the shorter of the two lengths must reach `min_length_metres`. Two legs that start or end at the
  same stop leave out the first `metres` around it. That is where legs in a row and the loops of a butterfly
  meet by design; opposing stretches further out still count. Taped legs count like any other.
- A finding at two legs has the location `Legs { course, afters }` and the place kind `Legs`. Its key holds
  both legs; `finding accept --leg A:2 --leg A:5` and MCP `legs` with two objects accept it, in any order.
  Accepting legs of two courses is refused as more than one place. MCP shows it as
  `{"kind": "legs", "course", "afters"}`.

## Consequences
Cost if wrong: one condition for the strictest rule, one location variant. The values 45°, 90°, 10 m, 30 m and
135° are proposals in the profile (Claude's call).

Source: plan M2b Task 5; controller rulings R15 and R16, 2026-11-09.
