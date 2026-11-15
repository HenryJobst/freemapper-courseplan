# 0096: The freemapper-core pin goes to v0.29.0

Status: Accepted, 2026-11-15. Follows [0094](0094-freemapper-core-v0-28-0.md) and [0095](0095-cargo-deny.md).

## Context

freemapper-core v0.29.0 reads fonts with `skrifa` instead of the unmaintained `ttf-parser` (core ADR 0037, core PR #12;
the rendering goldens are unchanged). That ends the reason for the ignore of RUSTSEC-2026-0192 in `deny.toml`.

## Decision

- Raise the tag of all core crates in the workspace `Cargo.toml` to v0.29.0. No code change.
- New third-party crates in `Cargo.lock`: skrifa 0.49.0, read-fonts 0.46.0, font-types 0.12.7 (all October 2026),
  yoke 0.8.4 (2026-10-22), yoke-derive, zerofrom-derive, synstructure, bytemuck_derive, foldhash and stable_deref_trait.
  The youngest is two weeks old or older at the pin date, so ADR 0006 needs no exception. `ttf-parser` leaves.
- Remove the ignore of RUSTSEC-2026-0192 from `deny.toml`. Two ignores stay (rmcp, ADR 0095).

## Cost if wrong

- A font that the old parser read and `skrifa` does not would show in the editor map; the core's goldens say it does not.
