# 0083: Length bands are targets with a tolerance

Status: Accepted, 2026-11-12. Amends [0064](0064-score-round-trips.md), [0065](0065-score-draft.md) and
[0066](0066-score-checks.md).

## Context

A score draft had to land both runner profiles inside their bands: the good runner at 87 to 93 percent of all points,
the very good runner back after 95 to 100 percent of the limit. A draft outside either band failed with exit 1, and the
Gleisdreieck example had to meet both bands. On a map this size that capped the limit at 33 minutes (ADR 0077 and the
example README). The bands came from a rule of thumb, not from a competition rule.

## Decision *(user, 2026-11-12)*

The bands are target values, not hard limits. The values stay settable: the bands themselves and the tolerance below.

## Design *(Claude's call)*

- Each runner profile gets one of three states: *in the band*, *near the target* and *missed*. Near means outside the
  band by at most the tolerance. The tolerance is `band_tolerance_percent` in the `[score]` table, in whole percentage
  points, from 0 to 10, default 3. With the default bands the good runner is near at 84 to 96 percent and the very
  good runner at 92 to 100 percent. The upper end of the very good band is 100 anyway: a runner who needs more than the
  limit has not taken every control.
- Exactly the tolerance still counts as near; one tenth of a point more is a miss. A very good runner who cannot take
  every control within the limit, or needs more than the limit for all of them, is always a miss. No tolerance
  forgives missing controls.
- A near result is a success. `score length` and `score draft` exit with 0 and name the deviation, with the sentence
  that it lies within the tolerance. A miss exits with 1 as before, with the hint. The answers of the MCP tools and of
  `--json` carry `length.state` (`in_band`, `near`, `missed`), a `state` for each runner and each issue, and
  `tolerance_percent`. `length.ok` is true for the first two states.
- The draft search aims at the band as before. It scans the counts below the largest the very good runner covers and
  returns the first count inside both bands. Only when none is, it returns the count nearest to the bands among those
  within the tolerance, and the local search keeps trying for a count inside. The tolerance is the limit of what the
  search accepts and the point where it stops, not a new target.
- `check` reports a finding for a miss only. A deviation within the tolerance gives no finding, since the draft and
  `score length` already name it and a hint per near course would make the example's check noisy for nothing.
- Settable everywhere the other score settings are. `score settings` takes `--good-band MIN-MAX`,
  `--very-good-band MIN-MAX` and `--band-tolerance N`. The MCP tool `set_score_settings` takes `good_band_percent` and
  `very_good_band_percent` as `[low, high]` and `band_tolerance_percent`. The editor's Score panel has input fields for
  both ends of both bands and for the tolerance, with the defaults as placeholders; a change goes through the same
  operation as the other settings. The length report and the check read the settings, never constants.
- Validation: each band runs from 0 to 100 with low not above high, the tolerance from 0 to 10, and the band of the
  very good runner must not start below the band of the good runner. The last rule was asked for; it compares a share
  of points with a share of time, so it is a plausibility rule, not a physical one.

## Cost if wrong

A tolerance of 3 points lets a draft through that a runner would find a little too long or too short: the good runner
scores 84 percent of the points instead of 87, or the very good runner finishes after 92 percent of the limit and has
eight percent of the time left. A tolerance that is too wide makes courses feel unfair, since the whole point of the
bands is that both profiles have a comparable challenge. The setter can narrow it to 0, which restores the old
behaviour, or widen it to at most 10. A wider default would produce longer courses on small maps, a narrower one would
keep the old cap.

Source: the user, 2026-11-12.
