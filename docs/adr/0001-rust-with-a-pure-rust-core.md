# 0001. Rust with a pure-Rust core

Date: 2026-11-08
Status: Accepted

## Context

freemapper-courseplan must run on macOS, Linux and Windows for club course setters without GIS or OCAD. Its
siblings freemapper and freemapper-print are Rust, and it will take over their `.omap` reader, renderer and
placement module.

## Decision

freemapper-courseplan uses Rust (edition 2024) with a pure-Rust stack, in three crates:
`freemapper-courseplan-model` (data model and project file, no map or rendering dependencies),
`freemapper-courseplan` (domain logic) and `freemapper-courseplan-cli` (CLI, MCP server, editor web server).
The release workflow, taken over from freemapper-print, builds one static binary per target.

## Consequences

- This project and its siblings share code without a language boundary.
- freemapper-print can later use the model crate to read courseplan events.
- Cost if wrong: this project needs its own code for any format no Rust crate covers.

Source: design §3, §5; freemapper-print ADR 0001; freemapper ADR 0001.
