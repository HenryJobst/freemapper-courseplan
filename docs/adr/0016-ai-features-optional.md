# 0016. AI features are optional and need a configured model

Date: 2026-11-08
Status: Accepted

## Context

The AI checker and proposal mode (ADR 0015) need a model. Club course setters may have no model access, no
network at the event site, or may not want AI at all.

## Decision

AI features inside freemapper-courseplan are active only when a model is configured. Without a configured
model the program is fully usable: every check, export, sync and editor function works, and AI findings and
proposals are absent. No code path requires a model. The rule holds for the whole suite. The MCP server is not
affected: an external assistant drives the program there, and the program itself calls no model.

## Consequences

- Hard rules, optimisations and the route analysis never depend on AI.
- The model configuration is a separate, optional setting, outside the project file of an event.
- Cost if wrong: none for users without AI; with AI, one extra setting.

Source: user, 2026-11-08 ("KI-Funktionen innerhalb der Anwendungen sind optional, wenn Modell konfiguriert").
