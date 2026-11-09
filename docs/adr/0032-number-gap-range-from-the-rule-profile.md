# 0032. The number gap range comes from the rule profile

Date: 2026-11-09
Status: Accepted

## Context
ADR 0030 defines the gap of a number placement as the distance between the outer edge of the circle line and the
number box. It says nothing about how large the gap should be. Automatic placement used a fixed 0.5 mm, and
`check` did not look at stored gaps.

No standard gives a value. ISOM 2017-2 and ISSprOM 2019-2 symbol 704 say the number is "placed close to the
control circle in such a way that it does not obscure important detail", oriented north. WKB 2026 B 7.2.6 asks
that number and code are clearly assigned to the circle. Both are qualitative.

## Decision
- **Range.** A gap is wanted between 0.5 and 5.0 mm in print mm, measured as in ADR 0030 *(user, 2026-11-09)*.
- **Rule profile.** Both values are rule-profile values: rule `wkb2026.number.gap`, key `number_gap`, values
  `min_gap_mm` and `max_gap_mm` *(user, 2026-11-09; the key and value names agreed with freemapper-print)*.
  A rule may leave out one value, which keeps that side open. A rule with neither value is skipped *(Claude's
  call)*.
- **Check.** A stored gap outside the range gives a warning *(user, 2026-11-09)*. A central number is reported
  once per rule, at the control, and a course's own number at the incoming leg *(Claude's call)*.
- **Candidate gaps.** Placement tries gaps from `min_gap_mm` upward in 0.5 mm steps. The last step is
  `max_gap_mm`, even when it is not a multiple of 0.5 *(agreed with freemapper-print)*.
- **Ranking.** Gap before direction: in the preferred direction a larger gap beats another direction at a smaller
  gap, given equal conflicts *(user, 2026-11-09)*. The key is (overprint conflicts, map conflicts, off preferred
  direction, gap, off outer bisector, direction) *(agreed with freemapper-print)*.
- **Behind blocked.** Tested at the minimum gap *(agreed with freemapper-print)*.
- **No rule.** Without an applicable rule, or with the rule switched off, the gap is a fixed 0.5 mm *(agreed with
  freemapper-print)*.
- **Several rules.** The strictest wins: the largest minimum and the smallest maximum. If that leaves the maximum
  below the minimum, the maximum is raised to the minimum *(agreed with freemapper-print)*.
- **Input.** `--gap` and `set_control_number` still accept 0 < gap <= 10 mm *(agreed with freemapper-print)*. The
  range is a wish, not a limit on input.
- **Overlays.** If an overlay applies the rule to only some courses, one control can get a finding from two
  rules, from a stricter overlay rule and the base rule. This is rare and left as it is *(Claude's call)*.

## Consequences
- Stored placements do not move. Only a new run of `course numbers` or `place_numbers` uses the range.
- `check` warns about old manual gaps above 5 mm, and about gaps below 0.5 mm.
- Automatic placement can now store gaps above 0.5 mm, up to the maximum, where the preferred direction is
  covered at the minimum gap.
- freemapper-print builds the same steps and ranking into the shared placement module.

Source: spec §5.3 and §5.6, plan 2026-11-09-number-gap-range, user rulings of 2026-11-09
