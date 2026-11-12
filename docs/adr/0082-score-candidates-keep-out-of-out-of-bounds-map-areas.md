# 0082: Score candidates keep out of out-of-bounds map areas

Status: Accepted, 2026-11-12. Amends [0062](0062-score-candidates-and-points.md) and follows
[0081](0081-a-control-must-not-stand-in-an-out-of-bounds-area.md).

## Context

ADR 0081 made `check` report a control that stands inside an out-of-bounds area. `score suggest` and `score draft` left
out sites only in out-of-bounds and dangerous special objects, never in map areas. The Gleisdreieck example showed the
gap: six of its 27 controls stood inside areas with symbol 520, on building outlines and in the grounds around them,
and `check` reported each as an error. A draft that the program itself finds wrong is a bug.

## Decision *(user, 2026-11-12)*

Controls must not stand in out-of-bounds areas, and that holds for proposed and drafted controls too.

## Design *(Claude's call)*

- The candidate pool drops every site that lies inside an area of the map with the symbol 520, 709 or 714, or on its
  edge. The area test is `check::point_in_area`, the one the check uses, so the edge counts as inside in both places
  and a draft cannot hold a control the check reports. No safety margin: the check has none, and a margin would leave
  sites that the rule allows. A site just outside the area stays.
- The out-of-bounds special objects use the same test now, so a site on the edge of one is also dropped. The draft
  drops them for every course, while the check applies a special object only to its own courses. This is stricter and
  safe.
- The symbols come from one constant, `check::SPRINT_AREA_SYMBOLS`. The profile cannot import it, so a test compares
  the constant with the values of the rule `wkb2026.control.in_forbidden_sprint`. Score courses exist on sprint and
  schoolyard maps only (ADR 0010), so the sprint list is the one that counts. If the profile changes and the constant
  does not, that test fails.
- Tests: a site on, in or beside a 520, 709 or 714 area, a hole, an area of another symbol, and a site on the edge of a
  special object. Besides these, the example maps Gleisdreieck and Rummelsburger Bucht get drafts with several seeds,
  and `check` must find no control in an out-of-bounds area in any of them.

## Cost if wrong

A control site just inside an area with a symbol that does not mean "do not enter" on some map is lost to the pool. On
the two example maps the exclusion removes sites on building outlines that lie inside 520 areas, which the check
rejects anyway. A map that draws 520 over open ground would lose those sites, and its drafts get fewer candidates. The
setter then sees fewer suggestions, and the drafts may miss the bands more often.

Source: the user, 2026-11-12.
