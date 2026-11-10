# 0039. IOF XML positions come from the map georeferencing

Date: 2026-11-10
Status: Accepted

## Context
ADR 0033 wrote map positions only and left the geographic `Position` of starts, controls and finishes for later,
because the grid conversion did not exist. freemapper-core v0.7.0 has it as `MapGrid` in `freemapper-course`. It
reads the georeferencing of the `.omap` file and converts a point on the map sheet to latitude and longitude.

## Decision
- **Source.** `export_iof_xml` takes an optional `MapGrid`. The CLI and the MCP tool build it once with
  `MapGrid::of(&doc.georef)` from the map document and pass it on *(Claude's call)*.
- **No y flip.** The project frame and the `.omap` frame are the same: millimetres on the sheet, y down. The grid
  takes points in that frame, so the converter gets the project coordinates unchanged *(Claude's call)*.
- **Six decimals.** `lat` and `lng` carry six decimals, about 0.1 m on the ground. The round trip through
  `MapGrid` stays within 0.01 mm on the map in a test *(Claude's call)*.
- **Element order.** `Position` comes before `MapPosition`, as the schema requires.
- **No usable georeferencing.** A map without georeferencing, or with a coordinate system the core does not
  support, gets no `Position`. The export still succeeds, `IofXmlExport.geo_positions` is false, the CLI prints
  `export-iof-xml-no-geo`, and the MCP result reports `geo_positions`.
- **Still open.** The `Map` corners stay 0/0 mm (ADR 0033). Taped legs are not written.

## Consequences
- The committed example `course-data.xml` carries positions, because the Lübeck map is georeferenced (EPSG:25832).
- Nobody has imported a file with positions into OE, OCAD or Condes. The positions are checked only against the
  schema and against the grid's own inverse.
- A new core version that changes `MapGrid` changes the numbers, and the snapshot test shows it.

Source: ADR 0033, freemapper-core v0.7.0, plan 2026-11-10-iof-xml-geo-positions
