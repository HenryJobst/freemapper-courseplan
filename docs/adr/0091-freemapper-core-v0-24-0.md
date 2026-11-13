# 0091: The freemapper-core pin goes to v0.24.0

Status: Accepted, 2026-11-13. Follows [0006](0006-dependencies-at-least-two-weeks-old.md).

## Context

freemapper-print moved to freemapper-core v0.24.0. courseplan pinned v0.19.0. The releases in between change:
v0.20.0 `PageSpec::page_mm` and `BuildReport::source_objects`; v0.21.x the suite page frame in freemapper-web;
v0.22.0 PDF and SVG leave out objects wholly outside the visible frame; v0.23.0 the map kind `forest` gets drawing
scales; v0.24.0 compact path coordinates in PDF.

## Decision

- Raise the tag of all core crates in the workspace `Cargo.toml` to v0.24.0. `Cargo.lock` changes only in the
  eleven core crates *(Claude's call)*. No third-party crate moved, so ADR 0006 needs no exception; the suite's own
  crates are exempt from the age rule anyway.
- The only code change is the new field `source_objects` in a test's `BuildReport` literal.
- Forest maps stay refused *(Claude's call)*: courseplan decides this itself from the map kind of the file
  (ADR 0010), not from the core's `is_written_by_freemapper`. The tests for it pass unchanged.
- The shared page frame (`suite/`) is not adopted here; that is a separate step.

## Consequences

- The editor map and the print preview look the same; no test that compares output needed new expected files.
- Anything that writes PDF or SVG through the core is smaller. courseplan writes none yet.

## Cost if wrong

- A later core change in rendering might show in the print preview; the editor tests would catch a geometry
  difference.

Source: freemapper-core CHANGELOG 0.20.0 to 0.24.0.
