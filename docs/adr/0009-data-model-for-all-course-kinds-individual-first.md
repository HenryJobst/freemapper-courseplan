# 0009. Data model for all course kinds, individual courses first

Date: 2026-11-08
Status: Proposed

## Context

The user starts with individual courses; relay and team orienteering come later. Team orienteering (mandatory
plus free controls, WKB B 3.3.2.3) is the format current programs handle worst. freemapper-print will read the
data model later.

## Decision

The data model holds four course kinds from the start: individual, team, relay, score. The first version builds
and edits only individual courses. The model lives in its own crate without map or rendering dependencies.

## Consequences

- Adding team, relay and score later changes the editor and checks, not the project format.
- The model crate makes no API stability promise until freemapper-print adopts it.
- Cost if wrong: an early model of a later kind may need rework when it is built.

Source: brainstorming 2026-11-08 (user: "mit Einzel-OL starten, Staffel und Mannschaft später"); design §4.3.
