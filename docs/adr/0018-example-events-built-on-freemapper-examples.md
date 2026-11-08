# 0018. Example events built on freemapper's examples

Date: 2026-11-08
Status: Accepted

## Context

freemapper ships one example per German state (its ADR 0020), and freemapper-print builds its examples on those
maps (its ADR 0013). Prospective users want to see finished courses, not only a feature list.

## Decision

`examples/` holds finished events built on maps from freemapper's examples: the project file, the IOF XML 3.0
export for OE, the `.ppen` export, the `check` findings and, once freemapper-print prints courses, the print PDF
and a preview image. The map itself is referenced or copied from freemapper. The first example comes once
courses can be set, checked and exported (after M3): a sprint training on freemapper's Lübeck old town map,
the map freemapper-print uses for its first example too.

Amendment (user, 2026-11-08: "Beispiele auch in der README darstellen"): the first example comes now, with M2a,
and both READMEs show it with a preview image. Until freemapper-print prints courses from a `.ppen`, the preview is
a sketch: the map rendered by freemapper-print v0.1.0 with the course drawn on top by the example's own script,
labelled as a sketch. The print PDF replaces it once both M3s exist.

## Consequences

- Each feature that changes what an event shows updates at least one example.
- Cost if wrong: the examples age when freemapper's maps change.

Source: user, 2026-11-08 ("wie in den Schwester-Projekten Beispielprojekte im Repo erstellen"); freemapper-print
ADR 0013; freemapper ADR 0020.
