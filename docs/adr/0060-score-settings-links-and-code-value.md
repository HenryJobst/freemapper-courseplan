# 0060. Score settings, mandatory links and the value in the code

Date: 2026-11-11
Status: Accepted

## Context
`CourseKind::Score` held a time limit, a penalty and controls with values. Score courses also need a value range,
runner profiles and optional pairs of controls that must be run one after the other. The design is
`docs/superpowers/specs/2026-11-11-score-courses-design.md` (§3, §5, §9).

## Decision
- **Settings table** *(Claude's call, profiles from the user)*. `[score]` in the project file, absent until set;
  `Event::score_settings()` returns it or the defaults. Defaults: `value_max` 5 (1 to 9 allowed), paces 250 s/km
  (good) and 225 s/km (very good), bands 87 to 93 and 95 to 100 percent, `mix_hard_percent` 40, `seed` 1 (a `u32`).
  Loading refuses a `value_max` outside 1 to 9, a pace of 0, a very good pace that is not below the good one, a
  band with low above high or high above 100, and a hard share above 100.
- **The code carries the value** *(user; the longer-code rule is Claude's call)*. In a two-digit code the second
  digit is the value (31 is 1, 49 is 9). In a longer code it is the last digit. The map shows no value.
- **Mandatory links** *(user)*. `links = [{ a = 31, b = 32 }]` on a score course names controls to visit directly
  after each other, in either direction. A runner who does not is disqualified. Older files load unchanged, and an
  empty list is not written.
- **Links form chains** *(Claude's call)*. A link joins two different controls of the course, none twice in
  either direction, at most two links meet at a control, and no loop closes. Other sets could not be run.
- **No format version step** *(Claude's call)*. Every new field is optional with a default (design §9, item 6).

## Consequences
- Later tasks read the settings through `score_settings()` and never see an absent table.
- The check that compares the stored value with the code comes with the score checks.

Source: design 2026-11-11 §3, §5, §9
