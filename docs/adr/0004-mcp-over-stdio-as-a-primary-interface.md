# 0004. MCP over stdio as a primary interface

Date: 2026-11-08
Status: Accepted

## Context

The user wants to set courses by hand and, where it proves useful, with AI support. No other program can
drive Purple Pen. freemapper and freemapper-print serve MCP over stdio with `rmcp`.

## Decision

`freemapper-courseplan mcp` serves MCP over stdio with `rmcp`. The tools mirror the CLI and add tools to list and
change controls, courses and classes, plus read-only map queries (objects and symbols near a point or in a
region, map extent). Results are compact JSON. The server refuses relative paths.

## Consequences

- An agent can set and check courses on the real map. Agent-made courses are possible in the first version but
  untested; the tested AI checker comes later (design §6).
- Cost if wrong: clients that need HTTP transport need another transport later.

Source: design §5.8; freemapper-print ADR 0004.
