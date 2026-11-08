# 0017. The .omap reader is copied from freemapper-print

Date: 2026-11-08
Status: Accepted

## Context

M2 checks need map objects: which symbol lies near a control, how far a leg runs through green. ADR 0013 puts one
renderer and map reader in a shared `freemapper-render` crate, but that crate does not exist yet. freemapper-print
has a reader, but on an unmerged branch.

## Decision

`crates/freemapper-courseplan/src/omap/` is a copy of the reader in freemapper-print at commit `8b0a0a6`
(itself copied from freemapper `743ce4b`), trimmed to reading: no enlargement, no built-in symbol sets, no
additions. Each file says so in its first line. `SymbolSet::code_for_id` is the only addition. `map.rs` builds
the queries (`MapIndex`, `map_kind`) on top. When `freemapper-render` is extracted, it replaces this copy.
Fixes found here are reported to freemapper-print.

## Consequences

- M2 does not wait for the shared crate.
- Cost if wrong: two copies drift until the extraction, and a fix made here must be carried over by hand.

Source: plan `docs/superpowers/plans/2026-11-08-m1-model-project-file-map.md`, Task 4; freemapper-print `8b0a0a6`.
