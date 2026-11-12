# 0080: The leg line is not checked against out-of-bounds areas

Status: Accepted, 2026-11-12. Amends [0027](0027-distance-checks-places-drawn-legs-symbol-codes.md).

## Context

The rules `wkb2026.leg.through_forbidden` (forest maps) and `wkb2026.leg.through_forbidden_sprint` (sprint and
schoolyard maps), key `leg_through_forbidden`, warned when the drawn part of a straight leg crossed an out-of-bounds
area, either a special object of the course or a map feature with symbol 520, 708, 709 or 714. They cite WKB26
B 7.2.6. According to the research file (`docs/research/2026-11-08-bahnlegung-kriterien.md`; we do not have the
original wording), that paragraph is about the best running route: it must not lead through out-of-bounds areas.

The line between two controls is not the running route. We used it only as a stand-in, because the program computes
no routes (Claude's call, plan M2b). A check of the straight line is not a check of the running route. A straight
line over an out-of-bounds area is a normal state; according to the research file, B 7.2.5 counts the length of a
course as straight lines plus the planned detours around out-of-bounds areas. On a sprint map the old check fired on
almost every leg, because out-of-bounds areas, buildings and water cut the map into lanes.

## Decision *(user, 2026-11-12)*

- The rules stay in the profile with id, key, source and the value `impassable`, for the route analysis. Their notes
  now say that the running route is checked, not the straight line.
- The check no longer tests any leg. It reports no finding, not even as an approximation. It adds the rule once to the
  rules that were not checked, with the reason "needs the route analysis" (`check-skip-needs-route-analysis`, English
  and German), so `check --all` shows it next to the other unchecked rules.
- The code that cut a leg with an out-of-bounds area, and its texts, are removed. The geometry the score link check
  uses stays.
- Legs through another control (`leg_through_control`) and the score link check (`score_link_valid`) stay as they
  are. The profile has no rule that reports a control inside an out-of-bounds area.
- An accepted finding of the two rules stays valid in the project file. It no longer matches any finding, so
  `check --all` lists it among the acceptances that match no finding, and the program does not fail on it.
- Tests: a leg across a 709 area and across an out-of-bounds special gives no finding, and both rules appear among
  the rules that were not checked; an acceptance of the rule shows as unmatched.

## Cost if wrong

A leg whose straight line is blocked and whose detour skews the route choice is not reported. The runner who has to
take the long way round loses time to one who does not, and a course setter who relies on `check` does not see it.
The requirement "the best route must not lead through out-of-bounds areas" stays in the research file as class A
(routing): only a route computation can test it. Until the route analysis exists, the setter judges such legs on the
map, and the rule in the unchecked list says so.

Source: the user, 2026-11-12.
