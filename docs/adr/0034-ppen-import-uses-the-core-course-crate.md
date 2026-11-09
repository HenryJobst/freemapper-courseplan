# 0034. The .ppen import uses the core course crate

Date: 2026-11-09
Status: Accepted

## Context
Design section 5.5 asks for a `.ppen` import. freemapper-core v0.3.0 now holds the Purple Pen reader and the fit
of an event to a map (`freemapper-course`), and the `.omap` reader that ADR 0017 copied (`freemapper-omap`).
Purple Pen writes the y axis upward and counts descriptions in the 2004 numbering; the model uses y down and the
IOF 2024 ids.

## Decision
- **One core tag.** `freemapper-omap` and `freemapper-course` come from freemapper-core v0.3.0, and
  `freemapper-text` and `freemapper-rules` move to the same tag, so all four core crates share one version. The
  local `omap/` module is gone; the copy of ADR 0017 is obsolete *(Claude's call)*.
- **The fit decides the y axis.** The reader tries the positions against the map with y up and y down and takes
  the one that lies on the map. If both fit, y up is taken and a note says so *(Claude's call)*.
- **Descriptions 2004 to 2024.** `iof_2024` maps Purple Pen's ids to the IOF 2024 ids, from the IOF PDF compared
  with Purple Pen's `symbols.xml`. Sections 0 to 10 keep their numbers, column G moves from 11.x to 12.x, column
  H from 12.x to 13.x, instruction rows from 13.x and 14.x to 15.x and 16.x. Ids without a counterpart (2004
  12.3 and 13.6) appear only in the note. Direction suffixes (`12.1NE`) are kept, so the id format of the model
  now allows them. Instruction rows that span the table live in description column `c` *(Claude's call)*.
- **Forbidden route with one point.** Purple Pen stores a forbidden route as a cross. The model accepts one
  point for `ForbiddenRoute` and `validate` is relaxed to match *(Claude's call)*.
- **Not imported, and named.** Variations and alternative controls, score courses, column F text, descriptions of
  start and finish, a second title, boundaries, lines, texts and white-outs. A map exchange control becomes a
  bend of the leg, and gaps in circles and legs are dropped. Nothing is left out without a note *(Claude's call)*.
- **Number gap approximate, direction exact.** The gap is the Purple Pen offset divided by the course object
  ratio, measured against courseplan's nominal circle and number box. Purple Pen's box differs, so the gap is
  close, not identical. A number that reaches into the circle is kept and noted *(Claude's call)*.
- **`ppen_id`.** `Control` and `Course` carry `ppen_id`, the Purple Pen id, optional, left out of the file when
  empty. It is the second key for the sync of design section 5.5a. The format version stays 1.
- **Replace rule.** `import` refuses a project that already has controls, courses or special objects, unless
  `--replace` is given. `--replace` swaps those and keeps classes and accepted findings *(Claude's call)*.
- **Linked file.** The `.ppen` path is stored in `sources.ppen` relative to the project file.
- **Names.** Equal course names get a number ("A (2)") and a note instead of being skipped.
- **Fixtures.** Three `.ppen` files from freemapper-core, BSD-3-Clause, with source and hash in
  `tests/fixtures/README.md`. `.gitattributes` marks `*.ppen -text`, as they have CRLF and a BOM.

## Consequences
- The import is one way; `sync` and the `.ppen` export (design section 5.5a) are still to build.
- freemapper-print and the editor must read `c = "15.x"` as a full-width row and accept direction suffixes.
- A real file from the author's events, on the Lübeck map, has not been imported yet.

Source: spec §5.5 and §5.5a, plan 2026-11-09-ppen-import, user rulings of 2026-11-08 and 2026-11-09
