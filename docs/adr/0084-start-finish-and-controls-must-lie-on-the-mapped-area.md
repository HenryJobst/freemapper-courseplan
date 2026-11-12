# 0084: Start, finish and controls must lie on the mapped area

Status: Accepted, 2026-11-12. Follows [0081](0081-a-control-must-not-stand-in-an-out-of-bounds-area.md) and amends
[0078](0078-no-score-sites-on-the-map-frame.md); `map_edge` stays as it is.

## Context

In the editor picture of the Rummelsburger Bucht example the finish lay in the white part of the sheet, beside the map.
The map is a diagonal strip of terrain, but the box around its objects covers the whole sheet. `map_edge` measures
against that box, so it saw nothing wrong. The same holds for the start and for controls. The user ruled that this is
not possible.

The research file (`docs/research/2026-11-08-bahnlegung-kriterien.md`, section 3.1 and 3.2) lists, under BL02, "controls
not too close to the map edge" and "route not outside the map". It has no wording on start and finish. We have only the
research file, so the source of the new rule is BL02 as the file cites it, and the note says so.

## Decision *(user, 2026-11-12)*

Start and finish must lie on the mapped area, and so must controls. A position in the blank part of the sheet is an
error.

## Design *(Claude's call)*

What the maps look like decides the definition. I measured the two Madella maps and the Lübeck map as rasters:

- The Madella maps hold, besides the terrain, a frame line around the sheet (one closed rectangle on Rummelsburg,
  two on Gleisdreieck), a logo and a point symbol in the blank part, and a large 601.6 area (north lines pattern, a
  blue hatching) that covers a hexagon bigger than the terrain. That area is a layout object, but it is an ordinary
  map area for the index, and it contains the finish F1 of the old example.
- Open land is mostly an explicit area (401) on these maps, so the strip has no holes. On the Lübeck map, which
  freemapper made, squares and courtyards have no object. The largest gap inside the terrain there is 7.5 mm across
  from the nearest object, 30 m on the ground.

The rule `wkb2026.map.inside` (key `inside_map`) works like this:

- A position is on the mapped area when a map object lies within `metres` of it, 0 inside an area. `metres` is 40 on
  the ground, so open squares between buildings stay inside and the blank part beside a strip does not. The old
  finish was 60 m from the nearest terrain object (63 m by the raster).
- Not counted as objects: the sheet furniture, which the map index already leaves out; the symbols in the rule value
  `ignore_symbols`, 601 and 602 (north lines pattern, registration mark), which are layout and no terrain; and the sheet
  frame, an axis-aligned rectangle of four corners drawn as a line that spans at least `frame_percent`, 80, percent of
  the terrain extent in both directions. Only lines count as frames, because an area that large may be a real base
  fill. The extent leaves out the ignored symbols, since the 601.6 area reaches beyond the sheet. It is not the extent
  of all objects either: Rummelsburg has a stray sliver of 403 at the south edge, and with it the frame spans only 89
  percent in height, so a limit of 90 percent missed the frame.
- It tests the start, controls, crossing points and finish of the individual courses of the rule, and the start,
  controls and finish of every score course. A control on no course gets no finding, as in the other control checks.
  One finding per position and rule, even when several courses use it.
- Severity is error, because the user wrote "not possible". The finding names the position in map millimetres, the
  nearest counted map object by symbol and its distance, and the limit. `finding accept` takes it like `map_edge`,
  with the control id or the id of the start or finish.
- `map_edge` is untouched: it still warns when a position is closer than 10 m to the edge of the box. A position in the
  blank corner of the box gets both findings only when it is also near the box edge.

- The score draft keeps its sites off the frame (ADR 0078) with the same test, `is_frame_outline`, and the same extent.
  On Rummelsburg the old test (90 percent of the extent of all objects) did not see the frame, so a draft could take
  its four corners, which lie in the blank part of the sheet. The new check reports such a control. The drafts of the
  example are redone: 16 minutes is the largest limit with both runners in their bands, 17 is near, 18 and above are
  missed (ADR 0083). The 20-minute course of the old README had controls at the frame corners.

## Consequences

`check` reports a start, finish or control in the blank part of the sheet as an error with the source BL02. The
Rummelsburg example had such a finish. Its start, finish and controls were set anew (changelog, "Fixed"), and so were
the pictures of the manual. The examples README says where start and finish stand. The message id is
`check-inside-map`, in English and German.

## Cost if wrong

- The limit of 40 m is too small on a map with large open areas and no object in them, for example a field of 150 m
  across drawn without a fill: a start in its middle gets a false error. The setter accepts it with a note, or the
  profile changes `metres`. A limit that is too large lets a position stand 40 m beside the terrain.
- A map that draws terrain objects far outside the real map, such as a legend or a logo, makes the blank part around
  them count as mapped. The Madella maps have a logo and a point symbol; a position within 40 m of them is not found.
- A map whose layout uses other symbols than 601 and 602 over the whole sheet needs them in `ignore_symbols`. A frame
  drawn as several short lines, or as a rectangle with more than four corners, is not recognised and counts as
  terrain; a real rectangle that is axis-aligned and spans 80 percent of the terrain is taken for a frame.
- The check does not know where the printed map area ends, only where objects are. A print area or a map border
  object would be exact, but the OOM files of these maps carry neither.

Source: the user, 2026-11-12.
