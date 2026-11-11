# 0057. Editor: print preview and overlay parity

Date: 2026-11-11
Status: Accepted

## Context
freemapper-print delivered the course layer in core v0.15.0 (moved to v0.16.0 the same day, see below): `freemapper_render::overprint::overprint` draws one
course as purple shapes in page millimetres, and `on_map` places its lower layer among the map colours. ADR 0046
promised a test that compares the editor's overlay with that drawing, and a read-only "print preview" layer.

## Decision
- **Core v0.16.0** *(Claude's call)*. All core crates move to the tag together (v0.15.0 first, v0.16.0 on
  2026-11-11). The lock file keeps `crc32fast` 1.5.3 and `flate2` 1.1.10 (ADR 0056); only the core crates were
  updated.
- **The preview is a map image with the course painted in** *(Claude's call)*. `GET api/map.svg` takes an optional
  `course=NAME`. The server turns the course into a `FittedEvent` (the start and finish become controls with codes
  that no control code can have), takes the number centres from the overlay, calls `overprint` and `on_map`, and
  draws one SVG. The layer is set on the page for one render and removed afterwards, so the cached plain map is
  unchanged. A preview is not cached, because the course changes with each edit. The page sits behind an `RwLock`
  for that, and a guard takes the layer off again when the render ends or panics.
- **Sizes and factor in one place** *(Claude's call)*. `print_layer::course_symbol_sizes(kind)` takes the
  sizes from `freemapper_printrules::scale::course_symbols` (core v0.16.0, the `iof-print` profile: ISSprOM
  2019-2 section 4.7, ISOM 2017-2 section 3.7), the profile freemapper-print draws with. The copy of the
  values that v0.15.0 required is gone. A test keeps circle, line and number height equal to
  `numbers::overprint_sizes`, which the checks use. **Superseded by ADR 0058:** the factor was `base_scale / map_scale` at first; it is now
  `numbers::course_factor`, the table factor of `course_symbols`, which is 1.0 at a non-standard or missing row
  (sprint 1:5000, forest 1:20000). A map without a scale gets factor 1.0 instead of infinity. The course colour is the IOF purple of `on_map`; courseplan's rule profiles carry none.
- **Toolbar toggle** *(Claude's call)*. "Print preview" (key P) works with a course selected. It hides the overlay
  and the finding markers and selects the select tool. **It is read-only** *(review of Task 20)*: the page refuses
  every write while it is on (`api.js` answers a POST with 423 without sending it), the tree, the properties
  pane, the tool buttons, undo and redo and the import and sync menu items are `inert`, and the keys for undo,
  redo, tools, Enter, Delete and nudging do nothing. Zoom, pan, help and the preview button itself work. A
  selection of another course loads that course's preview; selecting a non-course item ends it.
- **The overlay changed where it differed from the renderer** *(Claude's call)*. A leg's drawn part now starts at
  the start triangle's circumradius (side / sqrt 3), ends at the outer finish circle, and is not cut at a crossing
  point. Before, both ends were cut by the control circle's radius. `sizes` in `api/overlay` gains `start_r`,
  `finish_inner_r` and `finish_outer_r`, which the page uses to draw start and finish.
  `check::drawn_part` keeps its single radius, because the checks measure with it (ADR 0030). **The two cuts
  differ on purpose**: the checks cut both ends at the circle radius (3.0 mm at 1:4000), the overlay at 4.04 mm
  at the start and at the outer finish circle, and the checks do not cut at a crossing point either. A finding
  about a leg therefore measures a slightly longer line than the page draws. The test
  `the_checks_cut_legs_at_the_circle_radius_the_overlay_does_not` pins both, so aligning the overlay cannot move
  findings unnoticed.
- **The overlay's crossing point and marked legs follow the print** *(Claude's call, review of Task 20)*. The
  crossing point is the 3 mm symbol turned along the course (`angle_deg` in `api/overlay`, `sizes.crossing`),
  and a marked leg has the renderer's dashes (`sizes.dash`, `sizes.dash_gap`: 2 mm and 0.5 mm at the base
  scale).
- **Number box** *(agreed with freemapper-print, 2026-11-11)*. The renderer's box is authoritative; see ADR
  0030. `numbers.rs` takes it from `number_size`, so the overlay's box, the placement and the checks have the
  renderer's width.
- **Parity test** *(user, plan Task 20)*. `overlay_matches_renderer_overprint` (and the variant with bends,
  a crossing point, a marked leg and special objects) takes every course of `examples/luebeck-sprint` for five
  setups (sprint 1:4000, 1:3000 and 1:5000, schoolyard 1:1000, forest 1:10000; the map is the sprint map, the
  kind and scale are what changes the symbol sizes) and compares, within 0.01 mm: control circle centres and
  radii, start centre, circumradius and direction, finish centres and radii, the drawn part of every unmarked
  leg, the dashes of every marked leg (start, length and count), crossing point strokes (length and
  direction), number box centres and number box sizes. A leg that is not compared or has fewer than two drawn
  points fails the test. Mutations (a dash length off by 10 %, a crossing angle with the wrong sign) make it
  fail.
- **Rendering differences are findings, not local tweaks** *(user, plan Task 20)*. Not compared: the gaps of
  legs and circles, because the project file holds none (the preview passes none).

## Consequences
- The preview is as fast as a map section plus the layer; a pan during the preview renders again.
- The preview is read-only for real; a pending drag or draft ends when it is switched on.
- Findings about legs measure the checks' cut, the page draws the overlay's (see above).

**Update 2026-11-11 (core v0.16.1):** the parity test covers sprint 1:5000 against the sheet's 6.0 mm circles (ADR 0058).

Source: browser editor plan M4, Task 20.

**Update 2026-11-11 (core v0.19.0):** courseplan and the print rules share one `MapKind` from `freemapper-contract`;
the conversion between the two types is gone.
