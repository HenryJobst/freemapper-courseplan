# 0095: cargo-deny checks the dependencies

Status: Accepted, 2026-11-15. Follows [0006](0006-dependencies-at-least-two-weeks-old.md) and
[0093](0093-agpl-3.0-or-later.md). Template: ADR 0068 of freemapper-print.

## Context

The licences of the dependencies change with every `cargo update`, and nothing checked them. ADR 0006 limits the age of
a dependency but says nothing about its licence, known vulnerabilities or source. The licence matters more since the
switch to AGPL-3.0-or-later. freemapper-print asked for the same check here *(agreed with freemapper-print)*.

## Decision

- `deny.toml` at the repository root configures `cargo deny check`. It allows the licences the dependencies use now
  (permissive ones, MPL-2.0 and AGPL-3.0-or-later), denies registries and git sources other than crates.io and
  freemapper-core, denies yanked crates, and reports crates present in two versions as a warning only
  (getrandom, miniz_oxide, syn and windows-sys today).
- Wildcard dependencies are allowed: the workspace members and the core crates are path or git dependencies without a
  version, and the check flags those.
- Three advisories are ignored, each with its reason in the file. RUSTSEC-2026-0457 and RUSTSEC-2026-0462 concern the
  OAuth and Streamable HTTP client of rmcp, which courseplan does not build (rmcp 3.6.0 with `server`, `transport-io`
  and `macros`: a server on stdin and stdout). RUSTSEC-2026-0192 says `ttf-parser` is unmaintained and offers no
  upgrade; replacing it is a decision for freemapper-core.
- CI runs the check as its own job `deny`, with the action pinned to a commit.

## Consequences

- A new dependency with a licence not on the list fails CI until someone decides and adds it. A new advisory fails CI
  as well, also on a day when nothing in the repository changed; the fix is an upgrade or a reasoned ignore.
- The rmcp ignores end when rmcp 3.9.0 or later is old enough to take (ADR 0006).

## Cost if wrong

- A false alarm blocks CI until an ignore with a reason is added. The first run passed with no findings that needed a
  change beyond the three ignores.
