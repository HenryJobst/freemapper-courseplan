# 0020. Format 1: crossing points, special-object ids, start legs

Date: 2026-11-08
Status: Accepted

## Context

ADR 0019 left format 1 provisional and listed what it still lacked. M2a needs four of those gaps. IOF XML and
Purple Pen put a crossing point into the course sequence, so a `CrossingPoint` special object outside the
control list cannot carry it. The `.ppen` export and the courses need to refer to special objects, which only
start and finish could be named by. The leg from the start to the first control had no place for bends.

## Decision

- A crossing point is a control with `kind = "crossing_point"`. A course lists it in its sequence, and it gets
  no number. `SpecialObject::CrossingPoint` is gone.
- Every special object has an `id`, and a save matches them by it. Every kind except start and finish takes
  `courses`, a list of course names; empty means all courses.
- A course control takes `taped = true` when the leg leaving it is a marked route. A course takes
  `start_bends` and `start_taped` for the leg from the start, and `print_area` with `min` and `max` corners: the planned print area of the course, which checks use and
  freemapper-print takes as a default.
- The writer leaves out `kind = "control"`, empty lists and `false`, and the loader accepts them.

## Consequences

Format 1 stays at `1`, although ADR 0019 says added fields raise `format`: the format is provisional until the
first release. Files with a `crossing_point` special or specials without `id` no longer load, and none
exist outside the tests. Cost if wrong: another edit of a provisional format before the first release.

Source: plan M2a task 1, 2026-11-08; ADR 0019.
