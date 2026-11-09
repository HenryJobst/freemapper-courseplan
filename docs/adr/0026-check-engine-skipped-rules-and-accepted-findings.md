# 0026. Check engine: skipped rules and accepted findings

Date: 2026-11-09
Status: Accepted

## Context

ADR 0025 puts the rules in profile files and identifies an accepted finding by `(rule, course, controls,
legs)`. A rule has filters (map kinds, disciplines, print scales), and several rules can share a key, such as
the two distance rules for small and large print scales. A check that quietly does nothing looks like a pass.
Plan M2b implements the checks one task at a time.

## Decision

- The engine applies the filters before it calls a check. A check sees only the individual courses whose
  print scale its rule covers. Courses of the other kinds are reported once each as not checked yet, and so is
  a course whose legs cannot be built (it names a stop the event lacks).
- A rule that does not apply is reported as skipped only when no rule with the same key applies. For the
  discipline and the map kind that is decided once for the event, for the print scale once per course. Each
  rule of the key then gets one skip with its reason, such as "no rule of this kind for print scale 1:2500".
  A rule whose sibling checks the course stays silent, so `check --all` does not list a skip for every
  alternative. A training gets its own reason. A key whose check is not written yet gives one skip per rule.
- A finding's key comes from its location through `AcceptedFinding::key`: the event has no course, controls or
  legs; a course sets only `course`; one or several controls set only `controls`; one or several legs of a
  course set only `legs`, with the course inside each leg reference (ADR 0028). Keys are compared for
  equality, so an acceptance with a course and controls never matches.
- `accept_finding` refuses an unknown or inactive rule id, more than one place or legs of two courses, a
  course, control or leg position that does not exist, and an acceptance without place for a rule key whose
  findings never sit at the event. Each entry of the check table names the place kinds its findings sit at
  (`Check::places`: event, course, one control, two or more controls, one leg, two or more legs), and accept
  refuses the others with `FindingWrongPlace`, so `--course A` for the minimum distance is refused instead of
  stored where it never matches. Every check names its place kinds; a key that does not say allows every kind but the event.
  `start_capacity` sits at its start (a control place, ADR 0027). Stored controls and legs follow the key's order, and accepting the same key
  again replaces the note.
- The MCP `check` result gives each finding, accepted finding and skip with machine fields (rule, profile,
  severity, location as a tagged object, message id, arguments, source with document title and link) and the
  text in the server language. Accepted findings carry the course setter's note. The note is text the user
  wrote into a file that already passed the project checks of ADR 0023, like titles and names.

## Consequences

- Every line of the check table is `Check::at(key, run, places)`, which names the place kinds of the key (Task 4
  did this for the five distance keys, ADR 0027). After Task 8 no key is a stub.
- Cost if wrong: a reviewer who wants every filtered rule listed changes one condition in `run_key`.

Source: plan M2b Task 3; controller rulings R2 and R4 and the rulings for Task 3, 2026-11-09.
