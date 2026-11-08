# 0021. Program texts in German and English

Date: 2026-11-08
Status: Accepted

## Context

freemapper-courseplan is meant for the whole German orienteering community and for course setters abroad where
it makes sense. Many club course setters prefer German; IOF, OE and international users work in English. Until
now only the README, manual and changelog were bilingual (ADR 0003).

## Decision

Every text the program shows a user exists in German and English: CLI messages and help, check findings, the
editor and the texts it gives AI assistants over MCP. The language comes from `--lang de|en`, else the
environment variable `FREEMAPPER_LANG`, else the system locale (`LC_ALL`, `LC_MESSAGES`, `LANG`), else English.
The catalog is Rust code in `crates/freemapper-courseplan/src/text.rs`, so a missing translation fails the
build. Code, identifiers, logs and the `Display` of error types stay English.

## Consequences

- Each new message is written twice, and reviews check both languages.
- More languages need a move from Rust code to translation files; that step is left open.
- Cost if wrong: if German is not wanted after all, the catalog can collapse to English without changing the
  callers.

Source: user, 2026-11-08 (Deutsch und Englisch); design §3.
