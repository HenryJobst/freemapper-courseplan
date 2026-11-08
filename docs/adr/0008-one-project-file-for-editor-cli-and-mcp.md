# 0008. One project file for editor, CLI and MCP

Date: 2026-11-08
Status: Proposed

## Context

An event is prepared in the editor, from the command line or by an AI agent, often mixed. IOF XML 3.0 and
`.ppen` cannot hold everything the program needs: team courses, accepted findings, class mappings, later tours.

## Decision

Each event has one readable TOML project file with controls, courses of every kind, classes, special objects and
accepted findings. It names the map source and the linked Purple Pen file by relative path. Editor, CLI and MCP
read and write only this file. IOF XML 3.0 and `.ppen` are exchange formats, not the store.

## Consequences

- An event can be copied, reviewed and versioned like any text file.
- Cost if wrong: two tools writing at the same moment can overwrite each other; there is no lock at first.

Source: design §5.7; freemapper-print ADR 0008.
