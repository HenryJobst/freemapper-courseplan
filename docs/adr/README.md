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
| [0017](0017-omap-reader-copied-from-freemapper-print.md) | The .omap reader is copied from freemapper-print | Superseded by 0034 | 2026-11-08 |
| [0018](0018-example-events-built-on-freemapper-examples.md) | Example events built on freemapper's examples | Accepted | 2026-11-08 |
| [0019](0019-project-file-format-1.md) | Project file format 1 | Accepted | 2026-11-08 |
| [0020](0020-format-1-crossing-points-special-ids-start-legs.md) | Format 1: crossing points, special-object ids, start legs | Accepted | 2026-11-08 |
| [0021](0021-program-texts-in-german-and-english.md) | Program texts in German and English | Superseded by 0022 | 2026-11-08 |
| [0022](0022-program-texts-in-fluent-files.md) | Program texts in Fluent files, one per language | Accepted | 2026-11-08 |
| [0023](0023-mcp-server-file-checks-and-redaction.md) | MCP server: localized tool texts, file checks and redacted errors | Accepted | 2026-11-08 |
| [0024](0024-cli-editing-conventions.md) | CLI editing conventions | Accepted | 2026-11-08 |
| [0025](0025-rule-profiles-as-data.md) | Rule profiles as data files | Accepted | 2026-11-09 |
| [0026](0026-check-engine-skipped-rules-and-accepted-findings.md) | Check engine: skipped rules and accepted findings | Accepted | 2026-11-09 |
| [0027](0027-distance-checks-places-drawn-legs-symbol-codes.md) | Distance checks: places, drawn legs and symbol codes | Accepted | 2026-11-09 |
| [0028](0028-leg-shape-checks-angles-opposing-legs-two-leg-places.md) | Leg shape checks: angles, opposing legs and findings at two legs | Accepted | 2026-11-09 |
| [0029](0029-code-start-and-table-checks.md) | Code, start and table checks | Accepted | 2026-11-09 |
| [0030](0030-number-gap-is-the-clearance-to-the-circle-line.md) | The number gap is the clearance to the circle line | Accepted | 2026-11-09 |
| [0031](0031-number-placement-behind-the-circle-and-off-map-symbols.md) | Number placement behind the circle and off map symbols | Accepted | 2026-11-09 |
| [0032](0032-number-gap-range-from-the-rule-profile.md) | The number gap range comes from the rule profile | Accepted | 2026-11-09 |
| [0033](0033-iof-xml-export-writes-individual-courses-from-the-project.md) | IOF XML export writes individual courses from the project | Accepted | 2026-11-09 |
| [0034](0034-ppen-import-uses-the-core-course-crate.md) | The .ppen import uses the core course crate | Accepted | 2026-11-09 |
| [0035](0035-ppen-export-patches-the-linked-file.md) | The .ppen export patches the linked file | Accepted | 2026-11-10 |
| [0036](0036-number-placement-uses-the-core-module.md) | Number placement uses the core module | Accepted | 2026-11-10 |
| [0037](0037-description-symbols-from-the-core-crate.md) | Description symbols come from the core crate | Accepted | 2026-11-10 |
| [0038](0038-sheet-furniture-and-map-kind-from-the-sidecar.md) | Sheet furniture and the map kind from the sidecar | Accepted | 2026-11-10 |
| [0039](0039-iof-xml-positions-from-the-map-georeferencing.md) | IOF XML positions come from the map georeferencing | Accepted | 2026-11-10 |
| [0040](0040-ppen-sync-three-way-merge-on-the-purple-pen-view.md) | The Purple Pen sync is a three-way merge on the Purple Pen view | Accepted | 2026-11-10 |
| [0041](0041-reader-tests-against-the-suite-contract.md) | Reader tests against the suite contract | Accepted | 2026-11-10 |
| [0042](0042-editor-server-and-security.md) | Editor server: one process per project, a shared shell, and guarded requests | Accepted | 2026-11-11 |
| [0043](0043-editor-operations-revision-and-undo.md) | Editor operations, revision guard and undo | Accepted | 2026-11-11 |
| [0044](0044-save-refuses-a-changed-project-file.md) | Save refuses a project file that changed since it was read | Accepted | 2026-11-11 |
| [0045](0045-editor-front-end.md) | Editor front end: plain modules, one SVG, texts from the server | Accepted | 2026-11-11 |
| [0046](0046-editor-overlay-and-map-view.md) | Editor: course symbols from the server, map behind a view interface | Accepted | 2026-11-11 |
| [0047](0047-editor-controls-in-the-page.md) | Editor: selecting, placing, moving and deleting controls in the page | Accepted | 2026-11-11 |
| [0048](0048-number-drag-semantics-and-api-number-at.md) | Editor: number drag semantics and `api/number-at` | Accepted | 2026-11-11 |
| [0049](0049-special-objects-in-the-editor.md) | Editor: special objects, no move operation, redraw | Accepted | 2026-11-11 |
| [0050](0050-edits-keep-references-and-refuse-bad-text.md) | Edits keep accepted legs and sync decisions; bend limits; no control characters | Accepted | 2026-11-11 |
| [0051](0051-description-pickers-use-core-symbol-ids.md) | Description pickers list the symbols from core | Accepted | 2026-11-11 |
