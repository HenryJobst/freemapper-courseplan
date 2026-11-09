# 0033. IOF XML export writes individual courses from the project

Date: 2026-11-09
Status: Accepted

## Context
OE12 and other evaluation software read courses as IOF XML 3.0 `CourseData`. The design (section 5.5b) asked for
such an export. The first version had to pick what to write, which parts of the standard to follow where its
schema and our data disagree, and how to make the output reproducible and safe to write.

The schema is `IOF.xsd` from the IOF datastandard-v3 repository (master, fetched 2026-11-09). The repository
states no licence, so the file is kept as a test fixture only (`tests/fixtures/iof-xml/IOF.xsd`) and the
fixtures README says so. The tests validate the output with `xmllint` against it. CI installs `libxml2-utils` on
Linux; where `xmllint` is missing, the schema test passes without checking.

## Decision
- **Individual courses only.** Team, relay and score courses are not written. `export` skips them, names each
  one on the terminal, and the MCP tool lists them under `skipped` *(Claude's call)*. The data model covers all
  kinds (ADR 0009), but `CourseData` has no representation we can fill for them yet.
- **Control types follow the schema.** Start and finish are `Control` elements with `type="Start"` and
  `type="Finish"`, crossing points are `CrossingPoint`, all other controls are `Control` *(Claude's call)*. All
  defined controls and all start and finish objects are written, not only those a course uses.
- **Id space.** Start and finish ids and control codes all go into the `Id` element of a `Control`. They share
  one id space in the file, as the schema requires. The code does not check for a collision between a start or
  finish id and a control code (a start called `31` next to control 31 would write two equal ids). Ids are
  unique within each group already, and the usual ids (`S1`, `F1`) cannot clash with codes in the code range.
  A cross-check stays open *(Claude's call)*.
- **y axis mirrored.** The project stores map millimetres with y pointing down, the standard has y pointing up.
  The writer negates every y, also in `MapTextPosition` *(Claude's call)*.
- **Map positions only.** Controls get `MapPosition` in millimetres. Geo positions (`Position`) are not written.
  They need the georeferencing of the map and wait for `freemapper-georef` in freemapper-core *(Claude's call)*.
- **Schema deviations from the plan** *(Claude's call, the schema wins)*:
  - `Map` requires `MapPositionTopLeft` and `MapPositionBottomRight`, not one position. The export does not know
    the map extent and writes both as 0/0 mm. That is valid but carries no information, so a reader that uses
    them gets nothing from us.
  - `MapTextPosition` is relative to the centre of the control circle, not absolute. The writer gives the offset
    of the centre of the number box, taken from the same `number_rect` that `check` uses, so check and export
    agree. A course override beats the control's own placement; a control with no placement gets neither
    `MapText` nor `MapTextPosition`.
  - Element order follows the schema: in `CourseControl` the order is `Control`, `MapText`, `MapTextPosition`,
    `LegLength`; in `Course` it is `Name`, `Length`, `Climb`, `CourseControl`. `Event` needs only `Name`.
- **Not written yet.** Taped legs (`specialInstruction="TapedRoute"`) are missing; forced and drawn routes have
  no schema element we use yet.
- **Overprint sizes need the map kind.** The text offset depends on the circle and number sizes, which depend on
  the kind of map. `export_iof_xml` takes a `MapKind` parameter and the callers read it from the map, as `check`
  does. The first draft guessed it from the scale; that guess would shift the text positions on a wrong match
  *(Claude's call, after review)*.
- **Timestamp injected, in UTC.** `export_iof_xml` takes `created` as an argument, so the output is
  deterministic and the snapshot test is stable. The CLI and MCP pass the current time as
  `YYYY-MM-DDTHH:MM:SS` in UTC, without a zone marker. The standard library has no local time and no dependency
  was added for it (ADR 0006). The standard accepts a time without zone; a reader will take it as local time and
  show a shift of one or two hours in Germany. Nothing evaluates this field *(Claude's call)*.
- **Atomic write.** The file is written with `write_atomic` of the model crate (temporary file, then rename),
  now public. A failed export leaves an existing target intact *(Claude's call; the project file is saved the same way)*. Nothing is written when a course cannot be measured, and the error names the course.
- **Interfaces.** `freemapper-courseplan export PROJECT --iof-xml FILE` and the MCP tool `export_iof_xml`
  (`project`, `path`; the path must end in `.xml`) share `export_iof_xml_file`. Without a target option the
  command exits with 2 and says that there is nothing to do, because more formats follow.

## Consequences
- The output validates against the schema in the tests and for the Lübeck example, which is committed as
  `examples/luebeck-sprint/course-data.xml`.
- **Open: no real OE12 import has happened.** Valid XML does not mean OE12 reads it the way we intend. The user
  imports the example into OE12 and reports what arrives (controls, course order, lengths, classes). Until then
  the OE claim in the README is a plan, not a result *(user, 2026-11-09)*.
- Names with characters that XML forbids (such as U+0001) would still produce a file that readers refuse.
  The project loader does not reject them yet.
- When `freemapper-georef` exists, geo positions are added in a new ADR; the map extent could then be filled in
  as well.
