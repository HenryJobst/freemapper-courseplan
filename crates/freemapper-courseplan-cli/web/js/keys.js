// Keyboard shortcuts. The table below is the single source of the shortcut dialog and of the handler's keys.
// Ctrl/Cmd with + - 0 is the browser's zoom and is left alone (WCAG 1.4.4); typing in a field is never taken.

import { t } from "./texts.js";

export const SHORTCUTS = [
  { keys: "Ctrl/Cmd+Z", text: "editor-key-undo" },
  { keys: "Ctrl/Cmd+Shift+Z, Ctrl+Y", text: "editor-key-redo" },
  { keys: "Delete, Backspace", text: "editor-key-delete" },
  { keys: "Esc", text: "editor-key-cancel" },
  { keys: "V", text: "editor-key-select-tool" },
  { keys: "C", text: "editor-key-control-tool" },
  { keys: "← ↑ → ↓", text: "editor-key-nudge" },
  { keys: "Shift+← ↑ → ↓", text: "editor-key-nudge-big" },
  { keys: "+", text: "editor-key-zoom-in" },
  { keys: "-", text: "editor-key-zoom-out" },
  { keys: "0", text: "editor-key-fit" },
  { keys: "?", text: "editor-key-help" },
];

const NUDGE_MM = 0.1;
const NUDGE_BIG_MM = 1;
const ARROWS = {
  ArrowLeft: [-1, 0],
  ArrowRight: [1, 0],
  ArrowUp: [0, -1],
  ArrowDown: [0, 1],
};

function typing(target) {
  return target instanceof Element && target.closest("input, textarea, select, [contenteditable]") !== null;
}

// Fills the dialog's table from SHORTCUTS.
export function fillHelp(table) {
  table.replaceChildren();
  for (const row of SHORTCUTS) {
    const tr = document.createElement("tr");
    const key = document.createElement("th");
    key.scope = "row";
    key.textContent = row.keys;
    const what = document.createElement("td");
    what.textContent = t(row.text);
    tr.append(key, what);
    table.append(tr);
  }
}

// actions: undo, redo, remove, cancel, tool(name), nudge(dx, dy), commitNudge, zoom(factor), fit, pan(dx, dy),
// help, canNudge() (a control is selected), dialogOpen().
export function bind(actions) {
  document.addEventListener("keydown", (evt) => {
    if (evt.defaultPrevented || actions.dialogOpen() || typing(evt.target)) return;
    const mod = evt.ctrlKey || evt.metaKey;
    const key = evt.key;
    const inTree = evt.target instanceof Element && evt.target.closest("[role='tree']") !== null;
    let handled = true;
    if (mod && !evt.altKey) {
      const lower = key.toLowerCase();
      if (lower === "z" && evt.shiftKey) actions.redo();
      else if (lower === "z") actions.undo();
      else if (lower === "y" && !evt.metaKey) actions.redo();
      else handled = false;
    } else if (evt.altKey || mod) {
      handled = false;
    } else if (key === "Delete" || key === "Backspace") {
      actions.remove();
    } else if (key === "Escape") {
      actions.cancel();
    } else if (key === "v" || key === "V") {
      actions.tool("select");
    } else if (key === "c" || key === "C") {
      actions.tool("control");
    } else if (key === "?") {
      actions.help();
    } else if (key in ARROWS && !inTree && actions.canNudge()) {
      const [dx, dy] = ARROWS[key];
      const step = evt.shiftKey ? NUDGE_BIG_MM : NUDGE_MM;
      actions.nudge(dx * step, dy * step);
    } else if (key in ARROWS && !inTree && evt.target.id === "map-svg") {
      const [dx, dy] = ARROWS[key];
      actions.pan(dx, dy);
    } else if (key === "+" || key === "=") {
      actions.zoom(1.25);
    } else if (key === "-") {
      actions.zoom(1 / 1.25);
    } else if (key === "0") {
      actions.fit();
    } else {
      handled = false;
    }
    if (handled) evt.preventDefault();
  });
  // One operation per release, however many repeats the key made.
  document.addEventListener("keyup", (evt) => {
    if (evt.key in ARROWS) actions.commitNudge();
  });
  window.addEventListener("blur", () => actions.commitNudge());
}
