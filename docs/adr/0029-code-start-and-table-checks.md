# 0029. Code, start and table checks

Date: 2026-11-09
Status: Accepted

## Context
Plan M2b Tasks 6 to 8 add the checks on control codes, on starts and on the WKB and NOR tables. The rulings
made while building them were only in the plan ledger.

## Decision
- Code checks look only at punched controls. Crossing points, start and finish have no code on the ground.
  "Consecutive" means consecutive punched controls of a course, so a crossing point between two controls does
  not break the pair. A code that is no number gets `non_numeric_severity` in `code_range` (R18).
- `same_objects_in_circle` counts point symbols only: the control stands at a point feature and another object
  with the same symbol code lies inside the circle. Line and area features are left out, since contours, fences
  and vegetation repeat everywhere (R19).
- `code_pairs` and `code_confusable_next` report once at the two codes (`Location::Controls`), even when several
  courses use them (R20).
- `start_capacity` works per start, since courses share a start minute only there. Needed is the expected
  starters of the classes on those courses divided by `start_window_min`, rounded up. Available is the number
  of different first controls of the courses with starters. The finding sits at the start. A missing window,
  or a window of 0, skips the rule for the event; a class without expected starters skips that start (R21).
- `first_leg_angle` compares courses at the same start whose first punched controls differ. The direction runs
  from the start to the first bend or stop. Each pair of first controls is reported once, at both (R22).
- The table checks read the rule categories of a course's classes. A class without one skips the course with
  its name. A course whose categories hit several rows gets one finding naming every violated row. A category
  in no row of the table is a skip for both `length_table` and `nor_lkm`, never a hint: the NOR table uses
  regional names and has no elite rows (R23, R26).
- Team, relay and score courses get one skip each, with their kind (R7). The plan's global constraint asked
  for an info finding; like a filtered rule (R4), such a course was not checked, and a skip says that.

## Consequences
Cost if wrong: one filter for crossing points, one location shape for code pairs, the grouping by start, and
one severity for categories outside the NOR table.

Source: plan M2b Tasks 3 and 6 to 8; controller rulings R7 and R18 to R23, R26, 2026-11-09.
