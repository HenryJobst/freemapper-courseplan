# 0057. Editor: print preview and overlay parity

Date: 2026-11-11
Status: Accepted

## Context
freemapper-print delivered the course layer in core v0.15.0: `freemapper_render::overprint::overprint` draws one
course as purple shapes in page millimetres, and `on_map` places its lower layer among the map colours. ADR 0046
promised a test that compares the editor's overlay with that drawing, and a read-only "print preview" layer.

## Decision
- **Core v0.15.0** *(Claude's call)*. All core crates move to the tag together. The lock file keeps `crc32fast`
  1.5.3 and `flate2` 1.1.10 (ADR 0056); only the core crates were updated.
- **The preview is a map image with the course painted in** *(Claude's call)*. `GET api/map.svg` takes an optional
  `course=NAME`. The server turns the course into a `FittedEvent` (the start and finish become controls with codes
  that no control code can have), takes the number centres from the overlay, calls `overprint` and `on_map`, and
  draws one SVG. The layer is set on the page for one render and removed afterwards, so the cached plain map is
  unchanged. A preview is not cached, because the course changes with each edit. The page sits behind an `RwLock`
  for that.
- **Sizes and factor in one place** *(Claude's call)*. `print_layer::course_symbol_sizes(kind)` takes circle,
  line and number height from `numbers::overprint_sizes` and the other values from freemapper-print's `iof-print`
  profile (ISSprOM 2019-2 section 4.7, ISOM 2017-2 section 3.7). `course_factor` is `base_scale / map_scale`,
  as for the overlay. Courseplan does not depend on freemapper-print, so the values are copied, not read.
  The course colour is the IOF purple of `on_map`; courseplan's rule profiles carry none.
- **Toolbar toggle** *(Claude's call)*. "Print preview" (key P) works with a course selected. It hides the overlay
  and the finding markers, selects the select tool and turns editing off: a press only pans, Delete and the arrow
  keys do nothing. A selection of another course loads that course's preview; selecting a non-course item ends it.
- **The overlay changed where it differed from the renderer** *(Claude's call)*. A leg's drawn part now starts at
  the start triangle's circumradius (side / sqrt 3), ends at the outer finish circle, and is not cut at a crossing
  point. Before, both ends were cut by the control circle's radius. `sizes` in `api/overlay` gains `start_r`,
  `finish_inner_r` and `finish_outer_r`, which the page uses to draw start and finish.
  `check::drawn_part` keeps its single radius, because the checks measure with it (ADR 0030).
- **Parity test** *(user, plan Task 20)*. `overlay_matches_renderer_overprint` takes every course of
  `examples/luebeck-sprint` and compares, within 0.01 mm, control circle centres and radii, start centre,
  circumradius and direction, finish centres and radii, the drawn part of every unmarked leg, and number box
  centres. A second case adds bends, a crossing point, a marked leg and special objects.
- **Rendering differences are findings, not local tweaks** *(user, plan Task 20)*. The number box differs: the
  renderer's box for one digit at 4 mm height is 3.233 mm wide (advance width in Arimo), the placement code's is
  0.6 x 4 = 2.4 mm. The test `number_boxes_have_the_renderers_size` is `#[ignore]` with that reason. The centres
  match because the renderer draws at the centre it is given.

## Consequences
- The preview is as fast as a map section plus the layer; a pan during the preview renders again.
- The overlay's crossing point is still two upright arcs, 6 mm high at 1:4000, while the print draws the 3 mm
  symbol turned along the course. The page does not draw the dashes of a marked leg like the print does either.
  Neither is compared yet.
- Marked legs are left out of the leg comparison, and so are the gaps of legs and circles, which the project
  file does not hold.

Source: browser editor plan M4, Task 20.
