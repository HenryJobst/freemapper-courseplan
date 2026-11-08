# 0023. MCP server: localized tool texts, file checks and redacted errors

Date: 2026-11-08
Status: Accepted

## Context

The MCP server (ADR 0004) lets an AI assistant edit events. The assistant may be steered by text it read
elsewhere, so a tool call can name any file on the user's machine. rmcp's `#[tool]` attribute needs static
description strings, while program texts come from Fluent files (ADR 0022).

## Decision

- Tool and parameter descriptions carry English placeholders in the code; the server replaces them from the
  catalog in the language detected at start-up. Names of tools, parameters and JSON fields stay English.
- Tools open only project files ending in `.toml` and maps ending in `.omap` or `.xmap`. The check is lexical;
  symlinks are not resolved.
- Error results never quote file content: parse and schema errors give only line and column with a fixed
  reason, map format errors a fixed reason (the static hint for OOM's old binary format stays), and detailed
  consistency errors only for files that look like a project (an integer `format` and an `[event]` table).
- Parameters reject unknown fields. Each call loads, applies and saves without yielding, so one server process
  never loses an update; two processes on one file can still race.

## Consequences

- The CLI keeps its full error messages, because a person runs it on purpose.
- A crafted file with `format` and `[event]` can still surface its own key names to the assistant.
- Cost if wrong: an over-strict check refuses a real file; the user renames it to `*.toml`.

Source: plan M2a Task 9; rulings R1, R9, R10; two automated security reviews of the MCP commits, 2026-11-08.
