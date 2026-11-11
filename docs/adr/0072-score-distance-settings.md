# 0072: Score courses: distance settings

Status: Accepted, 2026-11-11. Amends [0062](0062-score-candidates-and-points.md) and [0068](0068-score-suggest-draft-length-ops.md).

## Context

After ADR 0071 the value-1 controls still lay 230 m from the start, and a draft could put two controls of high
value next to each other. The user asked for distances the setter can set *(user)*.

## Decision

Four settings in `[score]`, whole metres on the ground, each optional in the file with its default, so older
project files load unchanged. They convert to map millimetres with the map scale.

- `value1_min_m` (150) *(user)*: no candidate lies closer than this to the start or the finish. Value 1 goes to the
  nearest candidates (0063), so it starts at this distance. `suggest_with` filters the pool; the draft gets its
  candidates from the same call.
- `min_gap_m` (100, not lower) *(user)*: least distance between two controls. It replaces the scale-derived 15 or
  30 m default of `suggest`; `--min-distance` still overrides it. `check` warns (`score_min_gap`).
- `max_reach_m` (none = automatic) *(user)*: no candidate lies farther from the start. Automatic is a quarter of
  the ground the good runner covers in the time limit, `limit * 60 / good_pace * 1000 / 4` metres (900 m for 15
  minutes at 250 s/km) *(Claude's call)*. `suggest` without `--course` has no limit and no maximum.
- `high_value_gap_m` (200, at least `min_gap_m`) *(user)*: least distance between two high-valued controls. High
  is a value above 0.6 of `value_max` (4 and 5 at 5, 6 to 9 at 9), the same cut as the mix check *(Claude's call)*.
  The draft assigns values after it picks a prefix, so it repairs the prefix: of two close high controls the later
  one gives way to the next unused candidate of the spread order, or drops out when none is left. Forced link ends
  stay. `check` warns (`score_high_gap`).
- Validation: all positive, `min_gap_m >= 100`, `high_value_gap_m >= min_gap_m`, and `value1_min_m <= max_reach_m`
  when the reach is set. The values are integers (*Claude's call*), which keeps `ScoreSettings` comparable.
- Both checks are warnings with rule data in `wkb-2026.toml`; the distances come from the settings, not the rule.

## Consequences

The control count of a draft falls, because 100 m and 200 m at 1:4000 are 25 and 50 map millimetres. The measured
effect on the Lübeck example is in its README. The editor panel and the CLI edit all four values; an empty reach
field and `--auto-reach` return the reach to automatic.
