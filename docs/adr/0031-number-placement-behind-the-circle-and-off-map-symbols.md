# 0031. Number placement behind the circle and off map symbols

Date: 2026-11-09
Status: Accepted

## Context
The earlier placement avoided the course's own circles, legs and numbers but did not weigh map detail, and it
had no rule about where around the circle a number stands. Course setters moved many numbers by hand: a number
belongs behind the circle, where the eye arrives after the incoming leg, and it must not sit on a path crossing
or a small object that the runner needs to read.

The user decided these rules on 2026-11-09 *(user)*: (1) the number covers no point or line symbol of the map in
the control area; area symbols may be covered; this wins over 2 to 4. (2) The preferred direction is "behind",
the continuation of the incoming line past the control. (3) If the outgoing line blocks it (an almost straight
angle), the preferred direction is exactly outside, the bisector of the larger sector between the two lines.
(4) If the preferred place is blocked, the free place that deviates least wins. (5) `check` hints at an angle
above 170° between incoming and outgoing line. freemapper-print builds the same rules into the shared module
(its ADR 0013 and 0014), so both programs read them the same way.

## Decision
- **Preferred direction.** The incoming line is the last segment before the circle, counted from the last bend,
  as in `acute_angle`; for the first control it comes from the start triangle. Without an incoming line, the
  outer bisector of the outgoing line alone is used.
- **Blocked behind.** The number box at the behind direction, at the gap of 0.5 mm, comes within half an overprint
  line width of the outgoing line *(agreed with freemapper-print)*. Then the preferred direction is the outer bisector.
- **Candidates.** 15° steps around the preferred direction, up to the opposite one *(Claude's call)*. Each
  candidate is rounded to 0.1° inside `auto_place` (R3), and that rounded value is evaluated, placed as an
  obstacle and stored. A candidate never comes out as 360°.
- **Ranking.** (overprint conflicts, map conflicts, distance to the preferred direction, distance to the outer
  bisector, lowest direction). Overprint conflicts are circles, legs and other numbers; map conflicts are covered
  point and line symbols. Overprint outweighs any number of map symbols (R4, *Claude's call*). Placement never
  fails, even where every direction is covered.
- **Map symbols.** A point symbol is covered when the box comes within 0.5 mm of it, a line symbol when a segment
  comes within 0.15 mm of the box (map mm, unscaled) *(Claude's call)*. Areas cost nothing. Magnetic north lines
  (601 and its sub-codes, matched like `symbol_matches`) never count, since they cross the whole map (R5,
  *Claude's call*).
- **Check.** `wkb2026.leg.straight_angle_hint` hints at an angle above 170° at a punched control; crossing points
  get none. The 170° were set by the user. The profile rule keeps `doc = "claude"`, which `check` shows as a
  proposal of freemapper-courseplan: the label marks a value that no published rule source gives.

## Consequences
- Placed directions change: a number stands behind the circle on most legs, and at an almost straight angle
  outside at the tip. Numbers placed by hand stay only without `--overwrite`; central numbers always stay.
- Automatic directions are no longer multiples of 15°. They come from the legs and are rounded to 0.1°.
- freemapper-print implements the same rules and cites this ADR; its placements should match these except for the
  small cost it gives to areas.
- `examples/luebeck-sprint` has new number directions and a new preview.

Source: spec §5.3, plan 2026-11-09-number-placement-rules, user rulings of 2026-11-09
