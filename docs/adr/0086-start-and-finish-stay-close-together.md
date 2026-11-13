# 0086: Start and finish stay close together

Status: Accepted, 2026-11-12. Follows [0084](0084-start-finish-and-controls-must-lie-on-the-mapped-area.md).

## Context

At sprint events start and finish are usually not far apart. Anything else is impractical for organisers and
competitors. Generated courses should keep to that. A manual course may deviate.

## Decision *(user, 2026-11-12)*

- The straight-line distance between start and finish is at most 400 m for a generated course. The limit is a setting.
- `score draft` refuses a start and finish that are further apart. They are inputs of the draft, not generated, so the
  draft cannot fix them. `score suggest --course` refuses the same way when it takes start and finish from a course.
- `check` gives a hint, not a warning, for individual and score courses on sprint and schoolyard maps.

## Design *(Claude's call)*

- The setting is `start_finish_max_m` in the `[score]` table, default 400, range 50 to 2000, next to `dead_end_max_m`
  (ADR 0085). `score settings --start-finish-max`, `set_score_settings` and the Score settings panel change it.
- The error names both positions, the distance, the limit and the setting to change. Exit code 1, as for the other
  refusals of a draft.
- The rule is `wkb2026.course.start_finish_distance`, severity hint, source the user. It is a recommendation from
  practice, no rule of the WKB. The finding sits at the course.
- The distance is the straight line in metres on the ground. A leg-based distance would need the route analysis.
- `check` and the draft use the same limit and the same function, `ground_metres`.

## Cost if wrong

- Events with a mass start away from the finish, or a relay-like arena run, trigger the hint. The setting raises the
  limit, or the person accepts the hint.
- The straight line ignores the route: start and finish across a river can be 100 m apart and 900 m by foot.

Source: the user, 2026-11-12.
