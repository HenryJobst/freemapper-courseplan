# 0056. RenderMap: one A3 page at map scale, refusals as 413

Date: 2026-11-11
Status: Accepted

## Context
Task 19 wires freemapper-render (core v0.14.1) into the editor behind `MapView` (ADR 0046). The renderer builds
a print page, not an editor canvas, so a few choices were open.

## Decision
- **One page at map scale** *(Claude's call)*. `build_page` gets symbol factor 1 and geometry factor 1, A3,
  automatic orientation and a frame margin of 0, so the whole sheet is the map frame. The factors are checked
  (finite, above 0) before the call, because the builder does not check them. `PageMapping` becomes an `Affine`
  with scale 1 and the page offset; the code tests it at two points and refuses anything else.
- **Page box** *(Claude's call)*. `page_box` is the extent of the map's objects plus 10 mm, cut to the page, so
  fit-to-window does not show an A3 sheet that is mostly white. SVG and hit coordinates stay in page millimetres.
- **A map larger than A3 is cut off** *(Claude's call)*. `Paper` offers only A4 and A3. A map that does not fit
  is drawn clipped to the sheet. `RenderMap::is_clipped` reports it; the editor does not warn yet. Core would
  need a free page size to fix this. Lübeck and the two largest examples measured in ADR 0046 all fit.
- **Refusals** *(user: oversized view gives 413 with the message, the page keeps the overlay)*. A view over
  100 000 px on a side, or one the renderer refuses as too large, answers 413 with `editor-map-too-large`. The
  page shows that text in the status bar and keeps the overlay. A malformed query stays 400.
- **Cache** *(Claude's call)*. The page is built once per file stamp, with the parsed map. The last SVG is kept
  by view, so a repeated request does not render again.
- **Hits** *(Claude's call)*. `MapHit::object` is the renderer's object number, not an index into the file
  (core ADR 0023). `symbol_code` comes from the symbol set.
- **Dependencies** *(Claude's call)*. The lock file pins `crc32fast` 1.5.3 and `flate2` 1.1.10, because the
  newest releases were younger than two weeks (ADR 0006).

## Consequences
The program carries the render crate and its image, PDF and font dependencies; the binary grows, the web assets
do not. The placeholder stays for unit tests only.

Source: browser editor plan M4, Task 19.
