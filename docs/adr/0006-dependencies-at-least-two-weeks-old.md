# 0006. Dependencies at least two weeks old

Date: 2026-11-08
Status: Accepted

## Context

freemapper and freemapper-print only use crate versions that are at least two weeks old. The suite shares
tooling and many crates.

## Decision

Every dependency version must be at least two weeks old. Where a newer release is younger, the older version is
pinned exactly, in `Cargo.toml` or in `Cargo.lock`. The first `Cargo.lock` was taken from freemapper-print, so
shared crates start at versions already checked there.

## Consequences

- Every lock file change is checked against this rule.
- Cost if wrong: fixes from the newest releases arrive up to two weeks later.

Source: freemapper-print ADR 0006; freemapper ADR 0012.
