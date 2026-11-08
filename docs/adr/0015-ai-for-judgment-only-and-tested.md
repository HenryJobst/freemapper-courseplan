# 0015. AI for judgment only, and only after testing

Date: 2026-11-08
Status: Proposed

## Context

The user wants AI support where it is useful, after testing. About two thirds of the collected criteria are
machine-checkable; team splits, control-placer tours and route search are optimisation problems.

## Decision

Hard rules are code and data. Optimisation problems are algorithms. AI judges only what needs judgment: route
choice quality, difficulty for a class, unambiguous control sites, proposals. It starts after the route
analysis, uses TypeSafe typed judgments (the user's standing instruction), and ships per judgment only when it
agrees with a test basis, for example IOF's sprint leg scoring.

## Consequences

- Checks stay reproducible; AI adds advice on top.
- Cost if wrong: a judgment that would have helped waits for its test basis.

Source: brainstorming 2026-11-08 (user: "muss getestet werden"); design §6.
