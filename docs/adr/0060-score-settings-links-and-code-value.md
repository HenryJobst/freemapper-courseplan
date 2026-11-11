# 0060. Score settings, mandatory links and the value in the code

Date: 2026-11-11
Status: Accepted

## Context
`CourseKind::Score` held a time limit, a penalty and controls with values, and nothing else. Score courses need
settings for the value range and the runner profiles, and the format allows pairs of controls that must be run
directly one after the other. The design is `docs/superpowers/specs/2026-11-11-score-courses-design.md` (§3, §5, §9).

## Decision
- **Settings table** *(Claude's call, values from the user's runner profiles)*. `[score]` in the project file,
  absent until set; `Event::score_settings()` returns the table or the defaults. A key left out takes its default.

  | Key | Default |
  |---|---|
  | `value_max` | 5 (1 to 9 allowed) |
  | `good_pace_s_per_km` | 250 (4:10) |
  | `very_good_pace_s_per_km` | 225 (3:45) |
  | `good_band_percent` | 87 to 93 |
  | `very_good_band_percent` | 95 to 100 |
  | `mix_hard_percent` | 40 |
  | `seed` | 1 |

  The validation refuses a `value_max` outside 1 to 9, a pace of 0, a band whose low end lies above its high end
  or whose high end exceeds 100, and a hard share above 100.
- **The code carries the value** *(user; the longer-code rule is Claude's call)*. For a two-digit code the second
  digit is the value (31 is 1, 49 is 9). For a longer code the last digit is. The map shows number and no value.
- **Mandatory links** *(user)*. `links = [{ a = 31, b = 32 }]` on a score course names controls to visit directly
  one after the other, in either direction. A runner who does not is disqualified. Older files have no `links`
  and load unchanged; an empty list is not written.
- **Links form chains** *(Claude's call)*. A link joins two different controls of the course, none twice in
  either direction, and at most two links meet at one control. A branching set could not be run.
- **No format version step** *(Claude's call)*. Every new field is optional with a default (design §9, item 6).

## Consequences
- Later tasks read the settings through `score_settings()` and never see an absent table.
- The check that compares the stored value with the one in the code comes with the score checks.
- The five new model errors have English and German texts in the catalogs.

Source: design 2026-11-11 §3, §5, §9
