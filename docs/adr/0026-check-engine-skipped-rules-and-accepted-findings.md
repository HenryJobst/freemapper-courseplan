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
  print scale its rule covers. Courses of the other kinds are reported once each as not checked yet.
- A rule that does not apply is reported as skipped only when no rule with the same key applies. For the
  discipline and the map kind that is decided once for the event, for the print scale once per course. Each
  rule of the key then gets one skip with its reason, such as "no rule of this kind for print scale 1:2500".
  A rule whose sibling checks the course stays silent, so `check --all` does not list a skip for every
  alternative. A training gets its own reason. A key whose check is not written yet gives one skip per rule.
- A finding's key comes from its location through `AcceptedFinding::key`: the event has no course, controls or
  legs; a course sets only `course`; one or several controls set only `controls`; a leg sets only `legs`, with
  its course inside the leg reference. Keys are compared for equality, so an acceptance with a course and
  controls never matches.
- `accept_finding` refuses what can never match: an unknown or inactive rule id, more than one place or more
  than one leg, a course, control or leg position that does not exist, and an acceptance without place for a
  rule key whose findings never sit at the event. The check table marks such keys; for now only
  `start_capacity`. Stored controls and legs follow the key's order, and accepting the same key again replaces
  the note.
- The MCP `check` result gives each finding, accepted finding and skip with machine fields (rule, profile,
  severity, location as a tagged object, message id, arguments, source with document title and link) and the
  text in the server language. Accepted findings carry the course setter's note. The note is text the user
  wrote into a file that already passed the project checks of ADR 0023, like titles and names.

## Consequences

- Tasks 4 to 8 replace one stub line each in the check table and set `event_level` where their findings can
  sit at the event.
- Cost if wrong: a reviewer who wants every filtered rule listed changes one condition in `run_key`.

Source: plan M2b Task 3; controller rulings R2 and R4 and the rulings for Task 3, 2026-11-09.
