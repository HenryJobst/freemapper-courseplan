# 0013. One renderer in the suite, taken from freemapper-print

Date: 2026-11-08
Status: Proposed

## Context

The editor needs the map in the browser. freemapper's renderer writes PDF only. freemapper-print copies it,
develops it further and needs a browser preview for its own editor.

## Decision

There is no third copy. freemapper-print's renderer produces a backend-neutral `RenderedPage`, and the browser
backend (SVG or tiles) is built once there. This project takes over the renderer and the `.omap` reader once they
are extracted. Texts use the embedded font Arimo; an SVG backend ships it via `@font-face`.

## Consequences

- Map display in the editor arrives with M4; M1 to M3 work on data only.
- Cost if wrong: this project waits on freemapper-print for the map display.

Source: agreed with the freemapper-print session 2026-11-08 (its design §5.2, M1 plan task 7); design §5.1.
