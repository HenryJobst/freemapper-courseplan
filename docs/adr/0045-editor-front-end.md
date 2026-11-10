# 0045. Editor front end: plain modules, one SVG, texts from the server

Date: 2026-11-11
Status: Accepted

## Context
The editor page runs in the user's browser against the local server (ADR 0042). It has to show the event as a
tree, the map with the course on top, and the properties of the selection. The sibling editors in the suite
(freemapper, freemapper-print) use plain JavaScript, and the server refuses anything but its own origin.

## Decision
- **Plain ES modules, hand-written CSS** *(Claude's call)*. `<script type="module" src>` only, no framework, no
  vendored library, no inline script or handler (a Rust test scans the page for them), nothing loaded from the
  network. The assets are embedded with rust-embed and stay below 300 KiB (a test checks it).
- **One root `<svg>` for map and overlay** *(Claude's call)*. The viewBox is a rectangle in page millimetres.
  The map is an `<image>` of `api/map.svg` placed at its page rectangle; the overlay sits in a group with one
  transform built from the server's mapping (`k`, `dx`, `dy`), so both always line up. Pan and zoom only change
  the viewBox. The image is requested again 150 ms after the view stops, for the visible part plus 50 % on every
  side; the old image stays until the new one has loaded.
- **Layout** *(Claude's call, as freemapper-print ADR 0005)*. Tree on the left, map with a toolbar in the
  centre, properties and findings on the right, an "Event" menu on top, a status bar below. In the document the
  menu comes after the panes, and CSS puts it on top, so Tab runs tree, map, properties, menu.
- **Texts** *(user for German and English in Fluent, ADR 0022; Claude's call for the delivery)*. Every visible
  word is an `editor-*` message without arguments, delivered by `GET api/texts` in the process language.
  Elements name their text with `data-text`, `data-label` or `data-title`. There is no language switch; restart
  with `--lang`. Names, numbers and findings come from the server already formatted.
- **The page draws, the server computes** *(see ADR 0046)*. `overlay.js` places only what `api/overlay` returns.
  Every drawn element has `data-kind` and `data-id`. A selection is marked by a dashed halo and, in the tree and
  the properties, by text; colour is never the only sign.
- **Calls** *(Claude's call)*. `api.js` answers `{ ok, status, data }`. A POST sends the JSON content type and,
  when given, the revision in `X-Courseplan-Revision`.
- **Polling** *(plan D8)*. `api/revision` every two seconds while the page is visible, and at once on focus. A
  new project revision reloads state and overlay; a new map stamp reloads the map layer; a changed `.ppen`
  shows a note in the status bar.
- **Accessibility basics**. The tree has `role="tree"` with arrow keys, Home, End and Enter, and one tab stop.
  The map takes focus: arrow keys pan, `+` and `-` zoom, `0` fits the page. Every control has a visible label or
  an `aria-label`, the focus ring is visible, the message area is `aria-live="polite"`, and animations are off
  under `prefers-reduced-motion`.

## Consequences
- Dragging, editing and the findings drawer come in later stages on the same modules.
- Fine lines are small click targets; legs are not selectable yet.
- The page has no browser test in CI; the stage's smoke test is manual (plan Task 5, step 4).

Source: browser editor plan M4, D9 to D11 (2026-11-11).
