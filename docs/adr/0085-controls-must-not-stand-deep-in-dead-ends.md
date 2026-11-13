# 0085: Controls must not stand deep in dead ends

Status: Accepted, 2026-11-12. Follows [0062](0062-score-candidates.md) and [0082](0082-score-candidates-keep-out-of-out-of-bounds-map-areas.md).

## Context

A control at the end of a long dead end gives the runner no route choice. Every route shares one corridor, and the
leg is one long run in and out. On the Gleisdreieck map the example course had such a control on the quay spur left
of the river bridge. Short stretches into courtyards are fine.

## Decision *(user, 2026-11-12)*

- Generation (`score suggest`, `score draft`) never proposes or drafts a control that stands deeper in a dead end
  than a limit. A manual course may deviate. `check` only warns.
- The limit is a setting in whole metres. The user named about 80 m and said: measure, and shorten to 70 m if
  necessary.
- `check` reports it as the warning `wkb2026.control.dead_end` for individual and score courses on sprint and
  schoolyard maps. It is a recommendation of the project owner, not a rule of the WKB.

## Measurement *(Claude's call)*

The control in question is "7" of a screenshot of the user's own course in the editor. It stands on a small x symbol
(ISSprOM 531) at (42.95, -137.60) mm on the spur, a little more than a third of the way from the mouth to the tip. The
tip carries the control 54 of the example.

- Depth of control 7: 68 m, measured in steps of 4 m, so 68 to 72 m. It is the walk on the 2 m raster from the control
  to where the path network offers a choice again. The control stands on a wall line, and the runner stands 1.9 m away.
- Length of the whole dead-end branch: 208 m, measured from control 54 at the tip.
- A crop of the screenshot, scaled against the branch, gives about 60 m to the mouth.

The default is **60 m**, not 70 m. At 70 m control 7 (68 m deep) would not be reported, and the user wants it reported.
So the chosen value is the largest round number below the measured depth. The range is 20 to 300 m.

## Design *(Claude's call)*

The depth is computed on the cost surface of ADR 0061 by `CostSurface::dead_end`:

1. Dijkstra from the place over passable cells, in metres of walking distance (speed factors do not count), out to a
   horizon of the limit plus 100 m. Blocked patches up to 14 m across (a kiosk, a bollard) count as free, so they do
   not split a corridor.
2. A cell is alive when the cells at least as far away still connect it to the far rim of the horizon. A spur, a
   cul-de-sac or a courtyard with one gate is not alive: it ends before the rim, and the way on leads back through
   cells nearer to the place. This is judged per band of 4 m of walking distance, so ragged cells along the walls of a
   corridor do not count as dead.
3. The alive cells of each band fall into fronts (8-connected). The first band with two fronts of at least 3 m (a
   junction) or one front of at least 60 m (open ground) is where the choice starts. Its distance is the depth. No such
   band within the horizon gives an infinite depth, shown as "at least" the horizon.
4. A place on a wall or in a blocked cell stands in the nearest free cell within 4 cells that has room around it. A
   region smaller than the horizon is not judged, so tiny maps flag nothing.

A large open area or courtyard is no corridor: its fronts are wide, so the depth is small. The analysis reads the map
objects only (no out-of-bounds special objects), so `check` and the draft see the same pockets. The suggestion tests
the depth lazily for each site it picks, and `check` tests each control, with the same function (`DeadEnds::too_deep`),
so a draft never holds a control that `check` warns about.

The setting is `dead_end_max_m` in the `[score]` table, with the other event settings, because that table already
serves every event (`Event::score_settings`). Individual courses read it too. `score settings --dead-end-max`, the tool
`set_score_settings` and the Score settings panel of the editor change it.

## Validation

On a lattice of 20 m over the passable cells, limit 60 m, release build:

| Map | Points | Deeper than 60 m | 40 to 60 m | Not judged |
|---|---|---|---|---|
| Gleisdreieck | 4064 | 20 (0.5 %) | 8 | 11 |
| Rummelsburger Bucht | 2706 | 3 (0.1 %) | 5 | 13 |
| Lübeck fixture | 1141 | 27 (2.4 %) | 17 | 109 |

On Gleisdreieck the flagged points lie on the quay spur beyond about 60 m from its mouth, plus a few pockets; the
street grid between the river arms is not flagged. Of the 33 controls of the old example only 54 (the tip of the spur)
was deeper than 60 m. Rummelsburg course A is clean (12, 12, 20 and 12 m).

## Consequences

The Gleisdreieck example is redrafted (changelog). A draft keeps to the limit by construction; a property test over
both example maps and several seeds compares draft and `check`.

## Cost if wrong

- A control that stands on a wall between two regions, such as the corner of a building at the end of a quay, takes the
  depth of the nearest free cell with room around it. If that cell lies on the open side, the control counts as open,
  although a runner can also approach it from the dead end. On the Gleisdreieck map the redraft has a control at the
  west corner of the quay end for this reason (the check agrees with the draft, since both ask the same function).
- A control in the middle of a dead end that goes on more than the limit plus 100 m behind it counts as not deep,
  because both directions reach the rim. Such long dead ends did not occur on the example maps.
- The raster is 2 m coarse and the depth comes in steps of 4 m. A control within 4 m of the limit may fall on either
  side.
- A passage the map draws as open but nobody uses (a gap in a fence) makes a pocket look like a through route. A
  narrow real passage under 3 m does not count as a way on.
- A limit of 60 m is stricter than the 80 m the user named first. A course that needs deeper controls accepts the
  warning with `finding accept`, or raises the setting.

Source: the user, 2026-11-12.
