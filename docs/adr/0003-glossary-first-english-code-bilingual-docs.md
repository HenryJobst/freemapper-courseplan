# 0003. Glossary first, English code, bilingual user docs

Date: 2026-11-08
Status: Accepted

## Context

Users and most orienteering terms in daily use are German; code, specs and the IOF standards are English. The
siblings solved this with a glossary, and freemapper-print's glossary already names the course terms.

## Decision

- `docs/glossary.md` holds German term, English term, definition and code identifier for every new domain term.
  A new term gets its row before or with its code.
- Terms defined by freemapper's or freemapper-print's glossary keep their names and code identifiers (`Course`,
  `Control`, `ControlCode`, `NumberPlacement`, `MarkedRoute` …), and this glossary does not repeat them.
- Source code is English. READMEs, manual, examples README and changelog are German and English. Specs, plans
  and ADRs are English.

## Consequences

- One concept has one name across the suite.
- Cost if wrong: every user-facing change means writing it twice, and the languages can drift.

Source: design §3; freemapper-print ADR 0003; freemapper ADR 0007.
