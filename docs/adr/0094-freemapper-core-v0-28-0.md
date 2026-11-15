# 0094: The freemapper-core pin goes to v0.28.0

Status: Accepted, 2026-11-15. Follows [0091](0091-freemapper-core-v0-24-0.md) and [0093](0093-agpl-3.0-or-later.md).

## Context

freemapper-core v0.28.0 is licensed AGPL-3.0-or-later (core ADR 0035), as courseplan is since ADR 0093. The pin stood
at v0.24.0. v0.27.0 added `CourseSource::event_date` and `CourseSource::organiser`, which the core reads from IOF XML.

## Decision

- Raise the tag of all core crates in the workspace `Cargo.toml` to v0.28.0, so the app ships the AGPL core.
  `Cargo.lock` changes only in the core crates; ADR 0006 needs no exception.
- The only code change: the two places that build a `CourseSource` (Purple Pen export, print layer of the editor) set
  `event_date: None, organiser: None`. courseplan does not hand them to the core yet *(Claude's call)*.

## Consequences

- No behaviour change; all 1372 tests pass unchanged.

## Cost if wrong

- None known; the fields are only read by code that courseplan does not use for them.
