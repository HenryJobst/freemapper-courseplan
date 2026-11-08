# 0022. Program texts in Fluent files, one per language

Date: 2026-11-08
Status: Accepted

## Context

ADR 0021 put German and English texts into Rust code. The user wants further languages to be addable later.
Translators should not need Rust, and languages beyond German and English need proper plural and grammar
rules.

## Decision

Program texts live in Project Fluent files, one per language: `i18n/<language>/courseplan.ftl`, for example
`i18n/de/courseplan.ftl` and `i18n/en/courseplan.ftl`. The binary embeds them at build time and reads them with
the `fluent-bundle` crate (concurrent bundle, so the MCP server can share it). English is the reference and the
fallback: a message missing in another language falls back to English, and a test fails the build when a
language lacks a message id or has one English does not know. Adding a language means adding one folder.
The language comes from `--lang`, `FREEMAPPER_LANG`, `LC_ALL`, `LC_MESSAGES` and `LANG` in that order; the
first value whose language part matches an available language wins, else English. Code, identifiers, logs,
MCP tool and field names and the `Display` of error types stay English.

## Consequences

- Exception: the placeholders in clap's usage line (`[OPTIONS]`, `<COMMAND>`) and the text of rare command-line parse errors stay English; the common parse errors and all headings come from the catalog.
- Each new message is written in every language the project has, and reviews check them.
- Five small crates join the build (fluent-bundle, fluent-syntax, unic-langid, intl_pluralrules,
  fluent-langneg); all releases are older than two weeks.
- Cost if wrong: Fluent files are plain text and convert to any other catalog format.

Source: user, 2026-11-08 ("zukünftig sollen weitere Sprachen ergänzt werden können"); supersedes ADR 0021.
