# 0046. Editor: course symbols from the server, map behind a view interface

Date: 2026-11-11
Status: Accepted

## Context
The editor shows the map and, on top of it, the course: circles, start triangle, finish, legs, number boxes and
special objects. Each symbol has to be its own element, so the page can select and drag it, and a drag must not
wait for a new render of the map. The shared renderer (freemapper-render) draws map content, but its course layer
arrives only after core v0.9.0 and print's overprint is not part of `RenderedPage` yet. A map request should not
read the project file again, so the server needs to hold the parsed state.

## Decision
- **The server computes the overlay, the page draws it** *(Claude's call)*. `GET api/overlay?course=NAME`
  returns the symbols of one course in map millimetres; without the parameter it returns all controls and special
  objects, with no legs and no numbers. The page does no geometry. Sizes are the print sizes of `numbers.rs`
  grown by `base_scale / map_scale` (ADR 0030). A leg's drawn part is `check::drawn_part` with the circle radius,
  the cut the checks use. A number box is centred where `numbers::number_centre` puts it, which is the placement
  code of core (`number_offset::offset_from_placement`) at the same scale. A number without a stored placement is
  shown where `auto_place(keep_existing = true)` puts it and is marked `"source": "auto"`; the other sources are
  `"course"` and `"central"`. JSON numbers are rounded to 0.001 mm.
- **Parity** *(Claude's call)*. Rust tests pin the geometry to hand-computed values. When print's overprint is
  part of `RenderedPage`, a test compares both, and a read-only "print preview" layer shows the renderer's own
  course symbols (plan M4, Task 20).
- **Map view behind a trait** *(user: the map comes from the shared renderer, no own minimal map; agreed with
  freemapper-print 2026-11-11 for the API)*. `MapView` gives the mapping from map to page millimetres, the page
  box, an SVG for a rectangle and hit testing. `PlaceholderMap` stands in with an extent frame, a 10 mm grid and
  a note; it draws no map content. `RenderMap` replaces it in Task 19.
- **State** *(Claude's call)*. The server keeps the project file as last read, parsed once per revision. The
  revision is the FNV-1a 64 fingerprint of the file's bytes, so a save that writes the same bytes keeps it.
  `GET api/state` and `GET api/revision` re-read the file when its bytes changed. The map routes and the overlay
  use the held state and read the project file no more. The page learns about outside changes from the
  `api/revision` poll (plan D8). A broken project file or map gives a state with only `revision` and `error`.

## Consequences
- Course symbols exist twice until the print preview layer arrives, here for editing and in the renderer for the
  printed page. The parity test keeps them from drifting.
- Between `api/revision` polls the map routes can serve data of the previous revision for up to two seconds.
- `check::drawn_part` and `numbers::number_centre` became public; they are now API of the library crate.

Source: browser editor plan M4, D12 and D13 (2026-11-11).
