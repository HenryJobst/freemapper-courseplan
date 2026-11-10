# 0036. Number placement uses the core module

Date: 2026-11-10
Status: Accepted

## Context
ADRs 0030 to 0032 describe the number placement that `numbers::auto_place` carried out itself. freemapper-print
needed the same search for its sheets, so the search moved into `freemapper-course` as `placement`
(`best`, `GapRange`, `Obstacle`) and, with v0.6.0, `place_course_numbers` for a whole course (core ADR 0016).
Both sides ran the same 33 cases before the switch.

## Decision
- **One search, in core.** `auto_place` builds the stops (number box size, kept placement), the legs and the
  map obstacles and calls `place_course_numbers` with unbounded bounds. The search loop, the box and distance
  geometry (`Rect`, `segments_cross`, `box_distance`, ...), the candidate lists and the ranking are gone here
  *(agreed with freemapper-print, 2026-11-10)*.
- **What stays in courseplan.** The overprint sizes per map kind, the base scale, `circle_outer_radius_mm` and
  `number_box_mm`, the kept-number levels (central number, course number, `keep_existing`, ruling R11), the
  rule profile to gap range step (`rule_sides`, `normalise_sides`, then `GapRange::strictest` and
  `GapRange::from_limits`) and the conversion of map features into obstacles *(Claude's call)*.
- **Map features.** A point becomes a zero-length `Segment` with half width 0.5 mm, a line one `Segment` per
  piece with half width 0.15 mm. All are tier `MapSymbol` and carry the index of their feature as label, so a
  feature counts once however many pieces it has, as before. Areas and magnetic north lines (601 and its
  sub-codes) give none (ruling R5). The conversion runs once per `auto_place`, not per control
  *(Claude's call)*. That follows from the code structure: `map_obstacles` is called once, before the call
  into core, and no test would catch a call per control.
- **Types.** `numbers::GapRange` is core's type (fields `min` and `max`, `GapRange::DEFAULT` replaces
  `GapRange::FIXED`); `normalize_direction` is core's function. `MAX_GAP_MM` is the upper limit of
  `GAP_LIMITS_MM` *(Claude's call)*.
- **Rulings of core ADR 0016 that move positions** *(agreed with freemapper-print)*:
  - D2: a leg loses the part within the circle radius plus half the line width, not only within the radius. A
    bend within a line width of the circle no longer blocks a box at a gap below that width
    (`bend_inside_circle_line_gap_0_01`, `_0_1`: 0 degrees instead of 90).
  - D4: "behind is blocked" is tested at the stored gap. A rule minimum of 0.331 or 0.333 mm is stored as
    0.34 mm; the old code tested at the unrounded value and turned the number away for no reason
    (`rounded_min_behind_test_0_331`, `_0_333`: 0 degrees instead of 286.1).
  - D1, D3, D5 and D6 give no difference in practice.
- **Positions in the example.** `example_auto_placements_are_unchanged` passes without regeneration. No control
  of `luebeck-sprint` moves.
- **Directions are tenths.** Core 0.6.0's `candidate` rounded before it wrapped at 360 degrees, so about one
  placement in seven carried float noise (102.39999999999998 for 102.4). courseplan rounded newly placed
  directions itself for a day. Core 0.6.1 wraps first and rounds second (agreed with freemapper-print), and the
  workaround is gone. The tests that check every automatic direction for an exact tenth stay.
- **Tests adapted.** Two tests died with the private `Rank` type. `overprint_conflicts_weigh_more_than_map_symbols`
  is covered in core by `overprint_outranks_map_symbols`. `gap_rank_prefers_smaller_gap` is covered by the
  shared case `gap_range_moves_out_before_turning`, which runs here too.
- **Harness.** The allow-list `KNOWN_DISAGREEMENTS` in `tests/placement_cases.rs` is empty. The harness still
  fails when a listed case agrees.

## Consequences
- Placements move only in the two cases above: a bend within a line width of a circle with a gap below that
  width, and a rule minimum with three decimals. Whole and half millimetre gaps are unaffected.
- A change to the search is a change to core and needs the agreement of freemapper-print.
- ADRs 0030 to 0032 still hold the rules. The code lives in core.
