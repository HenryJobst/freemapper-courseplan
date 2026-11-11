# 0058. The course factor comes from the print rules

Date: 2026-11-11
Status: Accepted

## Context
ADR 0057 kept the symbol factor at `base_scale / map_scale` and gave the reason: `course_symbols` returns 1.0 at a
non-standard row, which would split the preview from the overlay and the checks. freemapper-print and this
project then agreed that the print rules are authoritative (core ADR 0018, core v0.16.1). At a non-standard row,
such as a sprint map at 1:5000 or a reduction, the symbols keep their base size, and the sheet draws circles of
6.0 mm there. With `base_scale / map_scale` the editor drew 4.8 mm and the checks measured with that.

## Decision
- **One helper** *(agreed with freemapper-print)*. `numbers::course_factor(kind, map_scale)` returns
  `freemapper_printrules::scale::course_symbols(kind, map_scale).1`, and 1.0 for a map without a scale. Placement,
  `number_centre`, `placement_from_centre`, the overlay, the print layer and the checks (`CheckContext::print_to_map_mm`,
  so circle radius for distance, number gap and leg cut) all call it. `base_scale / map_scale` no longer appears.
- **A non-standard scale is a note** *(Claude's call)*. For every individual course on a map whose scale is not
  standard for its kind (`freemapper_printrules::scale::is_standard`), `check` adds a note
  (`check-note-non-standard-scale`, in `skipped` without a rule, like an unresolved course) that says the symbols
  keep their base size and the course is only for training. The text follows freemapper-print's
  `warning-non-standard-scale`.
- **Shared cases** *(Claude's call)*. The cases in `cases.toml` fix the factor at `base_scale / map_scale` for every
  scale; `mixed_course_3` is a sprint map at 1:15000. The harness therefore calls `auto_place_with_factor` with that
  factor, and `course_factor` has its own tests. `KNOWN_DISAGREEMENTS` is empty: since core v0.16.1 the cases use
  the renderer's number box (0.8083747 x height x digits).
- **Parity** *(user)*. A test checks that the overlay and the renderer both draw 3.0 mm circle radii on a sprint map at
  1:5000, and 4.0 mm at 1:3000.

## Consequences
- Scales off the table (sprint 1:2000, 1:5000) no longer shrink or grow the symbols; the factor there is 1.0.
- Positions of numbers stored in a project do not change; only sizes at non-standard scales do.

**Update 2026-11-11 (core v0.19.0):** `course_symbols` and `is_standard` take courseplan's `MapKind` directly, because
it is the print rules' type (core ADR 0027).
