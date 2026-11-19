# 0098. The repository checks its own housekeeping

Date: 2026-11-19
Status: Accepted

## Context

CI ran the compiler and `cargo deny`. The documents, the ADR table, the changelog pair and the
dependency list were kept in order by hand. freemapper, freemapper-core and freemapper-print share
the same gap, and the gates are cut the same way in all four.

The first link check showed how much had drifted already: two ADRs linked to
`0062-score-candidates.md`, which had been renamed to `0062-score-candidates-and-points.md`; the
changelog linked to `examples/luebeck-sprint/luebeck-sprint.ppen`, an example that no longer
exists; and a design spec pointed at sibling checkouts (`../../../../freemapper`) that only resolve
on the author's machine. rustdoc was never held to `-D warnings`, so four doc comments linked to
private items, and nothing measured coverage or tested the `msrv` claim.

## Decision

- `scripts/check.sh` is the single entry point for the gates and has no dependencies beyond git and
  the tools named below. CI calls the same commands as a person does.
- The gates are `spdx`, `typos`, `links`, `secrets`, `machete`, `actions`, `adr`, `changelog`,
  `doc`, `coverage` and `msrv`. The first eight need no build and run on every commit
  (`lefthook.yml`); `secrets` runs before a push because it walks the history; `doc`, `coverage`
  and `msrv` build the workspace and each have their own CI job.
- Every Rust file starts with `// SPDX-License-Identifier: AGPL-3.0-or-later`. Documents and data
  do not repeat the line.
- `typos` checks the English documents only — `*.en.md`, `AGENTS.md`, `CLAUDE.md`, `docs/adr/`,
  `docs/superpowers/` and anything named `en.md`. German documents and fixtures stay out. The
  German words that stand as quotes in English text are listed in `_typos.toml` with their place.
- `lychee` checks links and anchors offline. The plans and specs under `docs/superpowers/` are
  excluded: they record how the code came to be and name files as they stood then, so their
  dangling references are history, not defects. Everything living — READMEs, manual, glossary,
  ADRs, example READMEs — is checked.
- The dead references in living documents are repaired: both ADR links name the renamed file, the
  changelog keeps the example's name without the hyperlink, and the design spec links the GitHub
  repositories instead of sibling checkouts.
- `gitleaks` walks the whole history with redacted findings; `cargo-machete` found nothing to
  remove here, and `cargo doc` with `RUSTDOCFLAGS=-D warnings` passes after four links to private
  items became code spans and the link to the model's constant took its full path.
- Coverage has a floor of 89.0 % in `scripts/check.sh` and the last measurement in
  `docs/coverage.json`. A run below either fails; a run above the snapshot rewrites it, and the
  commit belongs with the change that raised it. The first measurement is 91.0 %.
- The MSRV rises from 1.88 to 1.99. The workspace's own code does not need it; the manifest of the
  pinned freemapper-core v0.29.0 does, because cargo rejects a member overriding
  `default-features` of an inherited workspace dependency until 1.99. freemapper-core itself has
  the fix prepared, so the floor can fall with the next core tag.
- CI gives the gates their own jobs (`hygiene`, `docs`, `msrv`, `coverage`) beside `check` and
  `deny`. The tools are pinned by version and checksum in the workflow.

## Consequences

- A pull request now fails on a broken link or anchor, a missing ADR row, a changelog that misses a
  version, a declared but unused dependency, an action that hangs on a tag, a secret in the
  history, a rustdoc warning and a coverage drop.
- Historical plans may keep dangling references, but a link in a living document is a defect now.
- Raising coverage later means editing the floor in `scripts/check.sh` in the same commit as the
  better number.
- The next freemapper-core release carries the workspace-dependency fix; until this repository
  moves to that tag, its MSRV stays at 1.99.

## Amendment 2026-11-19
The MSRV paragraph above described the state under the pinned freemapper-core v0.29.0 tag. v0.29.1
carries the manifest fix: `default-features = false` stands on freemapper-core's workspace entry
instead of on one of its members, so cargo before 1.99 reads it again. courseplan moves to v0.29.1
and declares 1.88 again; the `msrv` job checks 1.88.0.

## Amendment 2026-11-19, badges
`scripts/badges.sh` writes the block between `<!-- badges:start -->` and `<!-- badges:end -->` in
both READMEs: line coverage, MSRV, ADRs, the number of gates and the licence. Every number has a
source in the repository, and `scripts/check.sh badges` fails when a block is out of date, so a
change that moves a number carries the badge in the same commit. The AI-cost badge beside them
belongs to its own service and keeps its own markers.
