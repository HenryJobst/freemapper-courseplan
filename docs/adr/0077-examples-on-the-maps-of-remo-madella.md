# 0077: Examples on the maps of Remo Madella

Status: Accepted, 2026-11-12.

## Context

The documentation examples ran on two maps nobody on the project had drawn for orienteering: the Lübeck sprint map
from freemapper's examples, and a map of Berlin-Kreuzberg that freemapper generated from open data. Remo Madella made
two maps, "Gleisdreieck" and "Rummelsburger Bucht" (MOO Challenge Berlin 2025), and released both for use within the
freemapper suite. The project owner passed this on on 2026-11-12.

## Decision

- **All map examples in the documentation use these two maps** *(user, 2026-11-12)*. `examples/gleisdreieck` holds a
  score course, `examples/rummelsburger-bucht` a classic sprint course with bends, a marked leg, a first aid post and
  accepted hints. `examples/luebeck-sprint` and `examples/berlin-kreuzberg` are removed, with their previews,
  transcripts and attributions.
- **Which map suits what** *(Claude's call)*: Gleisdreieck (1.2 x 1.8 km) for the score course because it is the larger
  map; Rummelsburger Bucht (1.2 x 1.0 km) for the classic course. Neither map reaches a 45-minute score course. The
  largest limit reached on Gleisdreieck is 33 minutes (start and finish in the north, default seed); the Rummelsburger Bucht gives about 20. The README of the Gleisdreieck example says so and names
  the tries. The AGENTS.md statement that a 45-minute score course needs a larger map is now true of both.
- **Attribution** *(user, 2026-11-12)*: each example folder has an `ATTRIBUTION.txt` naming the author, the event, the
  date and that the permission is for the suite. It states plainly that this is not an open licence. No Creative
  Commons or other open licence is claimed anywhere.
- **The Lübeck project files stay as test fixtures** *(Claude's call)*. Fifteen test files read the Lübeck project, its
  Purple Pen export and its IOF XML. They moved to `tests/fixtures/luebeck-sprint/` with only the `map` path changed.
  The documentation around them (READMEs, transcripts, preview, `make-preview.py`) is gone. Earlier ADRs (0031, 0033,
  0035, 0057) still name `examples/luebeck-sprint`; read that as the fixture folder.
- The test `berlin_score_example_meets_its_bands_and_check_is_quiet` became
  `gleisdreieck_score_example_meets_its_bands_and_check_is_quiet`: both bands met on a fresh `score length`, `check`
  quiet apart from the accepted code pair, and the note of that acceptance (the two controls stand about 260 m apart)
  tested against the project's coordinates.

## Risk: the scope of the permission

*Open question, with cost if wrong.* The permission was given "for use within the freemapper suite". It is not known
whether that covers redistributing the `.omap` files in a public repository (GitHub, and the private GitLab mirror) and
in release archives. The release workflow packs the README, the licence, AGENTS.md and the manual, not `examples/`, so
archives do not carry the maps today; the public repository does.

Cost if wrong: Remo Madella could ask for removal. The maps would then have to go out of the repository, and the
history would still hold them unless rewritten, which the project does not do (no force-push). The examples would fall
back to maps made by freemapper from open data. The score and sprint examples are small to rebuild, but the numbers in
the READMEs and the test would have to be redone. The mitigation is to ask him, in writing, whether the public
repository is covered. Until he answers, nothing in the project calls the maps open.

## Consequences

- The Gleisdreieck map has a closed rectangle around its content, drawn with symbol 501.1. `score suggest` and
  `score draft` took its corners as sites (ADR 0078 fixes this).
- The Rummelsburger Bucht map is cut by areas that must not be entered. A straight leg between two controls almost
  always crosses one (`wkb2026.leg.through_forbidden_sprint`), so the sprint course uses bends. `check` does not test
  legs with bends against these areas; the route analysis is still missing. The bends were found with a throwaway script
  that tests building, water and closed-area outlines, and nobody has looked at them on the map.
- The Purple Pen export of the new example was not opened in Purple Pen. The earlier Lübeck export was (2026-11-10).
