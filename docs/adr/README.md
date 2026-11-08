# Architecture Decision Records

An ADR records one decision that shapes freemapper-courseplan: what was decided, why, and what it costs if it
turns out wrong. Each ADR ends with a `Source:` line that names where the decision was made. Several ADRs take
over a decision of freemapper-print or freemapper; their source line names that ADR.

`Proposed` marks a decision that stands in the design draft but that the user has not yet approved. It becomes
`Accepted` when the user approves the design. An `Accepted` decision comes from the user directly or from an
agreement with a sibling session that the user asked for.

## Numbering

ADR numbers start at 0001, follow the order of the decisions and never repeat. The file name is
`NNNN-short-kebab-title.md`. A decision that replaces an older one gets a new ADR, and the old one changes its
status to `Superseded by NNNN`. Its text stays as it was.

## Template

```markdown
# NNNN. Title

Date: YYYY-MM-DD
Status: Accepted

## Context

What forced a decision: the facts, limits and data that were known.

## Decision

What was decided, in the active voice.

## Consequences

What follows from it, good and bad, and what it costs if the decision is wrong.

Source: spec §…, plan …, commit …
```

Keep an ADR between 10 and 30 lines. Write it in English, like all technical docs, and use the terms from
`docs/glossary.md`.

## Index

| Number | Title | Status | Date |
|---|---|---|---|
| [0001](0001-rust-with-a-pure-rust-core.md) | Rust with a pure-Rust core | Accepted | 2026-11-08 |
| [0002](0002-gpl-3.0-or-later.md) | GPL-3.0-or-later | Accepted | 2026-11-08 |
| [0003](0003-glossary-first-english-code-bilingual-docs.md) | Glossary first, English code, bilingual user docs | Accepted | 2026-11-08 |
| [0004](0004-mcp-over-stdio-as-a-primary-interface.md) | MCP over stdio as a primary interface | Accepted | 2026-11-08 |
| [0005](0005-course-editor-in-a-local-browser-page.md) | Course editor in a local browser page | Accepted | 2026-11-08 |
| [0006](0006-dependencies-at-least-two-weeks-old.md) | Dependencies at least two weeks old | Accepted | 2026-11-08 |
| [0007](0007-rust-toolchain-pinned.md) | rust-toolchain.toml pins the Rust toolchain | Accepted | 2026-11-08 |
| [0008](0008-one-project-file-for-editor-cli-and-mcp.md) | One project file for editor, CLI and MCP | Accepted | 2026-11-08 |
| [0009](0009-data-model-for-all-course-kinds-individual-first.md) | Data model for all course kinds, individual courses first | Accepted | 2026-11-08 |
| [0010](0010-sprint-and-schoolyard-maps-first.md) | Sprint and schoolyard maps first | Accepted | 2026-11-08 |
| [0011](0011-sync-with-purple-pen-by-three-way-merge.md) | Sync with Purple Pen by three-way merge | Accepted | 2026-11-08 |
| [0012](0012-check-rules-as-data-with-sources.md) | Check rules as data with sources, WKB 2026 first | Accepted | 2026-11-08 |
| [0013](0013-one-renderer-in-the-suite.md) | One renderer in the suite, taken from freemapper-print | Accepted | 2026-11-08 |
| [0014](0014-course-content-here-presentation-in-freemapper-print.md) | Course content here, sheet presentation in freemapper-print | Accepted | 2026-11-08 |
| [0015](0015-ai-for-judgment-only-and-tested.md) | AI for judgment only, and only after testing | Accepted | 2026-11-08 |
| [0016](0016-ai-features-optional.md) | AI features are optional and need a configured model | Accepted | 2026-11-08 |
| [0017](0017-omap-reader-copied-from-freemapper-print.md) | The .omap reader is copied from freemapper-print | Accepted | 2026-11-08 |
| [0018](0018-example-events-built-on-freemapper-examples.md) | Example events built on freemapper's examples | Accepted | 2026-11-08 |
