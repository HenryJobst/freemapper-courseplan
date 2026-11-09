# 0030. The number gap is the clearance to the circle line

Date: 2026-11-09
Status: Accepted

## Context
A number placement (`NumberPlacement`) holds a direction and a gap `gap_mm`. This project lays the numbers
out, and freemapper-print draws them (ADR 0014), so both must read the gap the same way. Before its M3,
freemapper-print asked whether the two definitions match. They did not:

- this project measured from the centre of the circle line, and put the box beyond the line at
  `gap` from the circle along the direction (the box's projection onto the ray);
- freemapper-print measured from the outer edge of the circle line, along the ray to the point where the ray
  enters the box.

With the second definition the box can reach into the circle on the diagonals. Sprint, one digit, 45 degrees,
gap 0.3 mm: the box covers the circle line by 0.21 mm. With two digits the visible clearance is 0.03 mm.

## Decision
`gap_mm` is the shortest distance, in print millimetres, between the outer edge of the circle line
(`r_outer = circle diameter / 2 + line width / 2`) and the number box. The box sits centred on the ray in the
direction of the placement. The direction stays as it was: degrees, 0 = east, counter-clockwise *(agreed with
freemapper-print, 2026-11-09)*.

Forward, the distance from the circle centre to the box centre has a closed form with three cases: the nearest
point of the box is on its near vertical edge, on its near horizontal edge or at the corner between them
(`box_distance` in `numbers.rs`). The inverse, needed when reading positions from other programs such as
Purple Pen, is `gap = distance(circle centre, box) − r_outer`.

The box size comes from freemapper-print's `number_box`, which uses the real font metrics. This project
keeps its stand-in of 0.6 × number height per digit until it takes over the shared placement module.

## Consequences
- A user who types `--gap 1` gets 1 mm of white between circle and number in every direction, and the box
  never covers the circle.
- Placements stored before this change stay valid but are drawn a little differently: 0.175 mm farther out
  straight above, below or beside the circle, up to 0.6 mm closer on the diagonals.
- The automatic placement keeps its gap of 0.5 mm; its conflict test uses the new box position.
