# 0031. Number placement behind the circle and off map symbols

Date: 2026-11-09
Status: Accepted

## Context
Until now the automatic placement (`place_numbers`, `course numbers`) avoided the course's own circles, legs and
numbers and weighed covered map detail, with no rule about where around the circle a number should stand. Course
setters moved many numbers by hand: a number belongs behind the circle, where the eye arrives after the incoming
leg, and it must not sit on a path crossing or a small object that the runner needs to read.

The user decided these rules on 2026-11-09 *(user)*:

1. The number covers no point or line symbol of the map in the control area (approach and departure). Area
   symbols may be covered. This wins over rules 2 to 4.
2. Preferred direction "behind": the continuation of the incoming connecting line past the control.
3. If "behind" is blocked by the outgoing line (an almost straight angle), the preferred direction is exactly
   outside: the bisector of the larger sector between the two lines, on the side the tip of the angle points to.
   Example: in from the left, out to the right at 170°, tip slightly up, gives 90°, straight above.
4. If the preferred place is blocked, the free place that deviates least from it wins.
5. `check` reports an angle above 170° between incoming and outgoing line as a hint.

freemapper-print builds the same rules into the shared placement module (its M3, ADR 0013 and 0014), so both
programs have to read them the same way.

## Decision
- **Preferred direction.** The incoming line is the last segment before the circle, counted from the last bend,
  as in `acute_angle`. The preferred direction is its continuation. For the first control the incoming line
  comes from the start triangle. Without an incoming line of any length, the outer bisector of the outgoing
  line alone is used.
- **Blocked behind.** "Behind is blocked" means the number box at the behind direction, at the placement's gap
  (0.5 mm), comes within half an overprint line width of the outgoing line *(agreed with freemapper-print,
  2026-11-09)*. Then the preferred direction is the outer bisector (rule 3).
- **Candidates.** The placement tries directions on a grid of 15 degrees around the preferred direction, up to
  the opposite one *(Claude's call)*. The preferred direction itself is always candidate zero, so it keeps its
  exact value.
- **Ranking.** Candidates compare as (conflicts, distance to the preferred direction, distance to the outer
  bisector, lowest direction value). Fewer conflicts win first, so a placement never fails even where every
  direction is covered *(Claude's call)*.
- **Map symbols.** Point and line symbols are hard conflicts, counted like the course's own circles, legs and
  numbers *(agreed with freemapper-print)*. A point symbol counts as covered when the box comes within
  `POINT_SYMBOL_RADIUS_MM` = 0.5 mm of its position; a line symbol when a segment comes within
  `LINE_SYMBOL_HALF_WIDTH_MM` = 0.15 mm of the box. Both are map mm and do not scale with the overprint factor
  *(Claude's call)*. Area symbols cost nothing here. freemapper-print gives them a small cost; this simple
  placement leaves that out.
- **Check.** A new check `straight_angle` hints at an angle above 170 degrees between incoming and outgoing
  line at a punched control. Crossing points get no finding. The rule id is
  `wkb2026.leg.straight_angle_hint` *(user)*; the 170 degrees are the user's value, the source is the user's
  criterion *(user)*.

## Consequences
- Placed directions change. A number now stands behind the circle on most legs, and at an almost straight angle
  outside at the tip of the angle. Numbers that a user placed by hand stay as they are.
- Automatic directions are no longer multiples of 15 degrees. The preferred direction comes from the legs
  and is stored as it is; only the fallback candidates step by 15 degrees from it.
- freemapper-print implements the same rules in the shared module and cites this ADR. When that module replaces
  this code, the placements it produces should match these rules except for the small cost it gives to areas.
- The example `examples/luebeck-sprint` has new number directions and a new preview.
