# 0081: A control must not stand in an out-of-bounds area

Status: Accepted, 2026-11-12. Follows [0080](0080-the-leg-line-is-not-checked-against-out-of-bounds-areas.md).

## Context

ADR 0080 took the leg check against out-of-bounds areas out of `check`. The simplest case stayed untested: a control
that stands inside such an area. The research file (`docs/research/2026-11-08-bahnlegung-kriterien.md`, section 3.9)
lists "out-of-bounds areas and impassable symbols are forbidden, per a list of ISOM/ISSprOM numbers" under WKB26 A 2.2
and IOF-R 17.2. We do not have the original wording of either paragraph, so the research file is our only source. The
test needs no route analysis, only the position of the control and the area.

## Decision *(user, 2026-11-12)*

Only the control must not stand in an out-of-bounds area. `check` reports a control that does.

## Design *(Claude's call)*

- Two rules, like the leg rules before them. `wkb2026.control.in_forbidden` applies to forest maps and
  `wkb2026.control.in_forbidden_sprint` to sprint and schoolyard maps. Both have the key `control_in_forbidden`,
  severity error and the source WKB26 A 2.2. The note names IOF-R 17.2 and says that we know the wording only from the
  research file. If the paragraph number is wrong, the fix is one line in the profile.
- The severity is error, because the user wrote "must not". A setter who has a reason, for example an area that is
  lifted for the event, accepts the finding with a note. If error is too strict, a deliberate position costs one
  `finding accept`.
- An area is either an out-of-bounds special object of the event or an area of the map. A special object counts for a
  control when it names no course or names a course the control is on. A map area counts for every control, since all
  courses share the map. The map symbols are the rule value `area_symbols`: 520 and 709 on forest maps, 520, 709 and 714
  on sprint and schoolyard maps. The boundary line 708 is a line and is not tested. Dangerous areas and forbidden routes
  are not tested either. If the user wants them, they are further rules, and until then a control in a dangerous area
  gets no finding.
- The leg rules keep their value `impassable`, with 708, for the route analysis.
- A point on the edge of an area is inside, with a tolerance of 1e-9 map millimetres. A point in a hole of a map area
  is outside. The edge of the hole is the border of the area, so a point on it is inside. A control that sits exactly on
  a border line is reported once more than a setter might expect, and that is the safer side.
- Which controls. The check reads the controls of individual courses at a print scale the rule covers, and the
  controls of score courses, as `map_edge` does. A control on no course gets no finding. The other control checks skip
  it too, it is not printed, and spare controls often stay in the file while a setter plans. A stray control in an
  out-of-bounds area is then found when a course starts to use it.
- Not covered are start, finish and crossing points. A crossing point stands at a boundary on purpose. Start and finish
  are not controls, and no existing rule tests them against out-of-bounds areas, since `map_edge` tests only the map
  edge. The user did not ask for them, so nothing is added. This is an open gap.
- One finding per control, even when several courses or areas hit it. The finding sits at the control, so
  `finding accept` takes the control as for `map_edge`. When a special object and a map area both hit, the text names the
  special object. It gives the id or the symbol and the position in map millimetres.
- The point-in-area test is `point_in_area` in `check/geometry.rs`. It adds the edge rule to `map::inside_area`, which
  applies the even-odd rule to all rings. The map reader flattens curves, so a control within the flattening error of a
  curved edge can fall on the wrong side. No new dependency is needed (ADR 0006).

## Consequences

`check` reports a control inside an out-of-bounds area as an error with a source. The message ids are
`check-control-in-forbidden-special` and `check-control-in-forbidden-map`, in English and German. AGENTS.md no longer
says that `check` skips this case.

Source: the user, 2026-11-12.
