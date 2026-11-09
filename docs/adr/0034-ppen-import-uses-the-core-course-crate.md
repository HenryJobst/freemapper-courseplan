# 0034. The .ppen import uses the core course crate

Date: 2026-11-09
Status: Accepted

## Context
Design section 5.5 asks for a `.ppen` import. freemapper-core v0.3.0 holds the Purple Pen reader and the fit of an
event to a map (`freemapper-course`), and the `.omap` reader that ADR 0017 copied (`freemapper-omap`). Purple Pen
counts y upward and numbers descriptions as in 2004; the model counts y downward and uses the IOF 2024 ids.

## Decision
- **One core tag.** `freemapper-omap` and `freemapper-course` come from core v0.3.0, and `freemapper-text` and
  `freemapper-rules` move to it, so all four core crates share a version. The local `omap/` module is gone and
  ADR 0017 is superseded *(Claude's call)*.
- **Y axis.** Core says `.ppen` counts y upward, so the fit flips points and number offsets (core ADR 0013). A
  file whose controls then do not lie on the map is refused with `OutsideMap`. Special objects with a point
  outside the map are left out and noted.
- **Descriptions 2004 to 2024.** `iof_2024` maps the ids; the table and its source are in the code. An id without
  a 2024 number (2004 12.3 and 13.6) is left out and noted. Direction suffixes (`12.1NE`) are kept, so the id
  format allows them. Instruction rows that span the table go in column `c` *(Claude's call)*.
- **Forbidden route with one point.** Purple Pen stores a cross; the model accepts one point and `validate` is
  relaxed to match *(Claude's call)*.
- **Not imported, and named.** Variations, alternative controls, score courses, column F text, descriptions of
  start and finish, the secondary title, boundaries, lines, texts, white-outs, and gaps in circles and legs. A
  map exchange becomes a bend. Every omission gets a note *(Claude's call)*.
- **Number gap.** The direction is exact. The gap is the offset divided by the course object ratio, measured
  against courseplan's nominal circle and box, so it is close but not identical. A gap beyond the maximum is cut
  to it, and a number that touches or enters the circle is kept; both are noted *(Claude's call)*.
- **`ppen_id`.** `Control` and `Course` carry the optional Purple Pen id for the sync of design section 5.5a. The
  format version stays 1.
- **Replace and link.** `import` refuses a project with controls, courses or special objects unless `--replace`
  is given; classes and accepted findings stay. The `.ppen` path goes in `sources.ppen`, relative to the project.
  Equal course names get a number ("A (2)") and a note.

## Consequences
- The import is one way; `sync` and the `.ppen` export (section 5.5a) are still to build.
- freemapper-print and the editor must read `c = "15.x"` as a full-width row and accept direction suffixes.
- No file from the author's own events has been imported yet.

Source: spec §5.5 and §5.5a, plan 2026-11-09-ppen-import, user rulings of 2026-11-08 and 2026-11-09
