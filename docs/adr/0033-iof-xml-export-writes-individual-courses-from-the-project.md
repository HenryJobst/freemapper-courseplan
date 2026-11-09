# 0033. IOF XML export writes individual courses from the project

Date: 2026-11-09
Status: Accepted

## Context
OE12 and other evaluation software read courses as IOF XML 3.0 `CourseData`. The design (section 5.5b) asks for
such an export. The schema is `IOF.xsd` from the IOF datastandard-v3 repository (commit `24eb108`). The
repository states no licence, so the file is a test fixture only (`tests/fixtures/iof-xml/IOF.xsd`). The tests
validate the output with `xmllint`; where it is missing, the schema tests pass without checking.

## Decision
- **Individual courses only.** Team, relay and score courses are skipped and named, on the terminal and in the
  MCP result under `skipped` *(Claude's call)*.
- **Control types follow the schema.** Start, finish and crossing points are `Control` or `CrossingPoint`
  elements. All defined controls and specials are written, not only those a course uses *(Claude's call)*.
- **One id space.** Start and finish ids and control codes share the `Id` element, as the schema requires. A
  clash between a start id and a control code is not checked yet.
- **y is written unmirrored.** The schema counts y in units below the centre, and the project stores y down, so
  no sign flips, also in `MapTextPosition`. The first draft mirrored y; that was wrong. Purple Pen, OCAD and
  Condes write y up against the standard (freemapper-print ADR 0010). Whether OE12 tolerates the standard's
  direction is unknown (see the addendum) *(Claude's call)*.
- **Map positions only.** No geo positions until `freemapper-georef` exists. `Map` needs two corner positions;
  they are written as 0/0 mm because the extent is unknown. Taped legs are not written yet.
- **Number positions** are the offset of the number box centre from the control, from the `number_rect` that
  `check` uses; a course override beats the control's placement. The caller passes the `MapKind`, read from the
  map, because the circle and number sizes depend on it.
- **`createTime` is injected** and written in UTC with a `Z`, so the output is deterministic and the snapshot
  stable. The standard library has no local time, and no dependency was added (ADR 0006) *(Claude's call)*.
- **Safe writing.** The file is written atomically (`write_atomic`). Nothing is written when a course cannot be
  measured, and the error names the course. The export refuses a target that is the project file or the map.
- **Interfaces.** `export PROJECT --iof-xml FILE` and the MCP tool `export_iof_xml` share one function.

## Consequences
- The output validates against the schema, also the committed example `examples/luebeck-sprint/course-data.xml`,
  which a test compares with the generated output.
- **No real OE12 import has happened** (see the addendum).
- Names with characters XML forbids would still yield a file that readers refuse.

## Addendum (2026-11-09): verification level
Nobody on the project has OE, so no import there will happen from our side; the earlier plan that the user imports
the example into OE12 is dropped *(user)*. What is verified: the output validates against `IOF.xsd`, and tests
read the export of the Lübeck example and of a crossing-point event back through `freemapper_course::read_iof_xml`
(freemapper-print's reader): codes, positions in file coordinates (y unmirrored), course names, order of stops,
leg count, number offsets from `MapTextPosition`, kinds, and zero courses when only non-individual courses exist.
Not verified: an import in OE, Purple Pen or Condes. The y axis stays uncertain: the schema counts y downward and we
follow it, while Purple Pen, OCAD and Condes write y upward against the schema, so a program that expects their
convention may show the controls mirrored. The reader above treats the direction as undetermined and leaves it to
the fit, so the read-back says nothing about it. Reports from people with OE are welcome.
