// The control description pickers, one per column C to H (ADR 0051). A column is a folded section whose summary
// shows the current symbol; opening it loads the symbols of the column (api/symbols) and shows them as a grid of
// icons with their text. One pick is one describe_control operation with that column only (the "none" entry
// clears the column and sends the other columns again, because the operation has no way to clear one column).
// The drawings come from the server as SVG text; they are parsed as SVG (DOMParser) and imported, never set
// as markup text. Keys: arrows move, Home/End jump, Enter picks, digits and dots type ahead by number.

import { get } from "./api.js";
import { t } from "./texts.js";

const COLUMNS = ["c", "d", "e", "f", "g", "h"];
const TYPE_AHEAD_MS = 800;
const SVG_NS = "http://www.w3.org/2000/svg";

// The symbols already loaded per column; they do not change while the program runs.
const loaded = new Map();
// The column that was open when the properties were built last, so it stays open across rebuilds.
let openColumn = null;

// An <svg> element for the server's SVG text, or null when the text is no SVG.
export function iconOf(svgText) {
  const doc = new DOMParser().parseFromString(svgText, "image/svg+xml");
  const root = doc.documentElement;
  if (!root || root.namespaceURI !== SVG_NS || root.localName !== "svg") return null;
  return document.importNode(root, true);
}

export function itemId(column, id) {
  return `pick-${column}-${id.replaceAll(".", "-")}`;
}

// The request body for one pick: just the column, or for "none" a clear plus the other columns as they are.
export function pickBody(code, description, column, id) {
  if (id !== null) return { code, [column]: id };
  const body = { code, clear: true };
  for (const other of COLUMNS) {
    if (other !== column && description?.[other]) body[other] = description[other];
  }
  return body;
}

// The next index for an arrow key in a grid of `count` items with `perRow` per row; null for other keys.
export function stepIndex(key, index, count, perRow) {
  const steps = { ArrowRight: 1, ArrowLeft: -1, ArrowDown: perRow, ArrowUp: -perRow };
  if (key === "Home") return 0;
  if (key === "End") return count - 1;
  if (!(key in steps)) return null;
  return Math.min(count - 1, Math.max(0, index + steps[key]));
}

function perRowOf(items) {
  if (items.length === 0) return 1;
  const top = items[0].offsetTop;
  const n = items.findIndex((el) => el.offsetTop !== top);
  return n === -1 ? items.length : n;
}

function setRoving(items, current) {
  items.forEach((el, i) => {
    el.tabIndex = i === current ? 0 : -1;
  });
}

function itemButton(column, entry, selected) {
  const button = document.createElement("button");
  button.type = "button";
  button.setAttribute("role", "option");
  button.setAttribute("aria-selected", String(selected));
  button.dataset.symbol = entry.id ?? "";
  button.className = "pick";
  if (entry.id !== null) button.id = itemId(column, entry.id);
  else button.id = `pick-${column}-none`;
  const icon = entry.id === null ? null : iconOf(entry.svg);
  if (icon) {
    icon.classList.add("pick-icon");
    button.append(icon);
  }
  const label = document.createElement("span");
  label.className = "pick-text";
  label.textContent = entry.id === null ? entry.text : `${entry.id} ${entry.text}`;
  button.append(label);
  if (selected) {
    const mark = document.createElement("span");
    mark.className = "pick-mark";
    mark.textContent = `✓ ${t("editor-desc-selected")}`;
    button.append(mark);
  }
  return button;
}

// Builds the grid for `entries` (the symbols of a column) into `list`.
function fillList(list, column, entries, current, pick) {
  const all = [{ id: null, text: t("editor-desc-none") }, ...entries];
  list.replaceChildren(...all.map((e) => itemButton(column, e, (e.id ?? null) === (current ?? null))));
  const items = [...list.children];
  let start = all.findIndex((e) => (e.id ?? null) === (current ?? null));
  if (start < 0) start = 0;
  setRoving(items, start);
  let typed = "";
  let timer = null;
  list.addEventListener("focusin", (evt) => {
    const at = items.indexOf(evt.target.closest("button"));
    if (at >= 0) setRoving(items, at);
  });
  list.addEventListener("click", (evt) => {
    const at = items.indexOf(evt.target.closest("button"));
    if (at >= 0) pick(all[at].id);
  });
  list.addEventListener("keydown", (evt) => {
    if (evt.ctrlKey || evt.metaKey || evt.altKey) return;
    const at = items.indexOf(evt.target.closest("button"));
    if (at < 0) return;
    let to = stepIndex(evt.key, at, items.length, perRowOf(items));
    if (evt.key === "Enter") {
      evt.preventDefault();
      pick(all[at].id);
      return;
    }
    if (to === null && /^[0-9A-Za-z.]$/.test(evt.key)) {
      typed += evt.key.toUpperCase();
      clearTimeout(timer);
      timer = setTimeout(() => { typed = ""; }, TYPE_AHEAD_MS);
      const found = all.findIndex((e) => e.id !== null && e.id.toUpperCase().startsWith(typed));
      to = found >= 0 ? found : at;
    } else if (to === null) {
      return;
    }
    evt.preventDefault();
    evt.stopPropagation();
    items[to].focus();
    items[to].scrollIntoView({ block: "nearest" });
  });
}

function section(host, control, column, actions) {
  const details = document.createElement("details");
  details.className = "pick-column";
  details.id = `desc-${column}`;
  const summary = document.createElement("summary");
  summary.id = `desc-sum-${column}`;
  const current = control.description?.[column] ?? null;
  const name = document.createElement("span");
  name.textContent = t(`editor-desc-column-${column}`);
  const value = document.createElement("span");
  value.className = "pick-value";
  value.textContent = current ?? t("editor-none");
  summary.append(name, " ", value);
  details.append(summary);
  const body = document.createElement("div");
  body.className = "pick-body";
  details.append(body);
  const code = String(control.code);
  const pick = (id) => {
    if (id === (current ?? null)) return;
    actions.describe(pickBody(code, control.description, column, id), itemId(column, id ?? "none"));
  };
  const show = (entries) => {
    const list = document.createElement("div");
    list.setAttribute("role", "listbox");
    list.setAttribute("aria-label", t(`editor-desc-column-${column}`));
    list.className = "pick-grid";
    list.id = `pick-list-${column}`;
    body.replaceChildren(list);
    fillList(list, column, entries, current, pick);
  };
  const load = async () => {
    if (body.querySelector("[role='listbox']")) return;
    if (loaded.has(column)) return show(loaded.get(column));
    body.textContent = t("editor-desc-loading");
    try {
      const answer = await get(`api/symbols?column=${column.toUpperCase()}`);
      if (!answer.ok) throw new Error(String(answer.status));
      loaded.set(column, answer.data);
      show(answer.data);
    } catch {
      body.textContent = t("editor-desc-failed");
    }
  };
  details.addEventListener("toggle", () => {
    if (details.open) {
      openColumn = column;
      load();
    } else if (openColumn === column) {
      openColumn = null;
    }
  });
  if (openColumn === column) {
    details.open = true;
    if (loaded.has(column)) show(loaded.get(column));
  }
  host.append(details);
  return details;
}

// The description pickers of `control` appended to `host`; `afterD(details)` can add something right after the
// D picker (the map objects at the control). actions.describe(body, focusId) runs the operation.
export function panel(host, control, actions, afterD) {
  const heading = document.createElement("h3");
  heading.textContent = t("editor-desc-heading");
  const hint = document.createElement("p");
  hint.className = "hint";
  hint.textContent = t("editor-desc-hint");
  host.append(heading, hint);
  for (const column of COLUMNS) {
    const details = section(host, control, column, actions);
    if (column === "d") afterD?.(details);
  }
}
