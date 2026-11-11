// Properties of the selection. Values are put in with textContent or an input's value only.
// A control's position is editable; its code and kind are shown as they are.

import { kindLabel } from "./overlay.js";
import { t } from "./texts.js";

const COLUMNS = ["c", "d", "e", "f", "g", "h"];

function mm(p) {
  return `${p[0].toFixed(2)}; ${p[1].toFixed(2)} mm`;
}

function row(list, labelId, value) {
  const term = document.createElement("dt");
  term.textContent = t(labelId);
  const detail = document.createElement("dd");
  detail.textContent = value === null || value === undefined || value === "" ? t("editor-none") : String(value);
  list.append(term, detail);
}

function specialPosition(s) {
  return s.map_mm ?? s.points?.[0] ?? s.polygon?.[0] ?? null;
}

function courseRows(list, state, name) {
  const course = state.event.courses.find((c) => c.name === name);
  if (!course) return;
  const info = state.courses.find((c) => c.name === name);
  row(list, "editor-prop-name", course.name);
  row(list, "editor-prop-kind", t("editor-kind-course"));
  row(list, "editor-prop-length", info?.length_m == null ? null : `${Math.round(info.length_m)} m`);
  row(list, "editor-prop-climb", course.climb_m == null ? null : `${course.climb_m} m`);
  row(list, "editor-prop-start", course.start);
  row(list, "editor-prop-finish", course.finish);
  row(list, "editor-prop-classes", (course.classes ?? []).join(", "));
  row(list, "editor-prop-stops", course.controls.map((c) => c.code).join(" → "));
}

// A decimal number in millimetres; a comma counts as the decimal separator. Null when it is not a number.
export function parseMm(text) {
  const s = text.trim().replace(",", ".");
  if (!/^[-+]?(\d+(\.\d+)?|\.\d+)$/.test(s)) return null;
  const value = Number(s);
  return Number.isFinite(value) ? value : null;
}

function shown(value) {
  return String(Math.round(value * 1000) / 1000);
}

function field(list, id, labelId, value) {
  const term = document.createElement("dt");
  const label = document.createElement("label");
  label.htmlFor = id;
  label.textContent = t(labelId);
  term.append(label);
  const detail = document.createElement("dd");
  const input = document.createElement("input");
  input.id = id;
  input.type = "text";
  input.inputMode = "decimal";
  input.autocomplete = "off";
  input.value = shown(value);
  detail.append(input);
  list.append(term, detail);
  return input;
}

// Editing X or Y moves the control: actions.move(code, x, y). Something that is no number is refused in the
// field, with a message the field points to.
function positionFields(list, control, actions) {
  const [x, y] = control.map_mm;
  const inputs = [field(list, "prop-x", "editor-prop-x", x), field(list, "prop-y", "editor-prop-y", y)];
  const error = document.createElement("p");
  error.id = "prop-error";
  error.className = "field-error";
  error.hidden = true;
  error.textContent = t("editor-error-number");
  for (const input of inputs) {
    input.setAttribute("aria-describedby", "prop-error");
    input.addEventListener("change", () => {
      const nx = parseMm(inputs[0].value);
      const ny = parseMm(inputs[1].value);
      for (const [i, v] of [[0, nx], [1, ny]]) inputs[i].setAttribute("aria-invalid", String(v === null));
      error.hidden = nx !== null && ny !== null;
      if (nx === null || ny === null) return;
      if (nx !== x || ny !== y) actions.move(String(control.code), nx, ny);
    });
  }
  return error;
}

function controlRows(host, list, state, code, actions) {
  const control = state.event.controls.find((c) => String(c.code) === String(code));
  if (!control) return;
  row(list, "editor-prop-code", control.code);
  row(list, "editor-prop-kind", kindLabel(control.kind ?? "control"));
  const error = positionFields(list, control, actions);
  for (const column of COLUMNS) {
    row(list, `editor-desc-column-${column}`, control.description?.[column]);
  }
  const users = state.event.courses
    .filter((c) => c.controls.some((x) => String(x.code) === String(code)))
    .map((c) => c.name);
  row(list, "editor-prop-used-in", users.join(", "));
  host.append(list, error);
  const remove = document.createElement("button");
  remove.type = "button";
  remove.id = "prop-delete";
  remove.textContent = t("editor-prop-delete");
  remove.addEventListener("click", () => actions.remove());
  host.append(remove);
  const heading = document.createElement("h3");
  heading.textContent = t("editor-prop-near-heading");
  const near = document.createElement("ul");
  near.id = "near-list";
  near.className = "near";
  host.append(heading, near);
}

// Lists the map objects of `answer` (api/map/near) in the properties; null leaves the list empty.
export function showNear(host, answer) {
  const list = host.querySelector("#near-list");
  if (!list) return;
  list.replaceChildren();
  const features = answer?.features ?? [];
  if (features.length === 0) {
    const none = document.createElement("li");
    none.textContent = t("editor-prop-near-none");
    list.append(none);
    return;
  }
  for (const f of features) {
    const li = document.createElement("li");
    li.textContent = `${f.symbol_code} (${t(`editor-geometry-${f.kind}`)}) ${f.distance_mm.toFixed(2)} mm`;
    list.append(li);
  }
  if (answer.total > features.length) {
    const more = document.createElement("li");
    more.textContent = "\u2026";
    list.append(more);
  }
}

function specialRows(list, state, id) {
  const special = state.event.specials.find((s) => s.id === id);
  if (!special) return;
  row(list, "editor-prop-id", special.id);
  row(list, "editor-prop-kind", kindLabel(special.kind));
  const at = specialPosition(special);
  row(list, "editor-prop-position", at ? mm(at) : null);
}

function classRows(list, state, name) {
  row(list, "editor-prop-name", name);
  const courses = state.event.courses.filter((c) => (c.classes ?? []).includes(name)).map((c) => c.name);
  row(list, "editor-prop-used-in", courses.join(", "));
}

// Fills `host` for the selection { kind, id, course? }, or the empty hint without one.
export function show(host, state, selection, overlay, actions) {
  // Keep the focus in the field being edited: the page is rebuilt after each change.
  const focused = host.contains(document.activeElement) ? document.activeElement.id : "";
  host.replaceChildren();
  if (!state || !selection) {
    const hint = document.createElement("p");
    hint.className = "hint";
    hint.textContent = t("editor-props-empty");
    host.append(hint);
    return;
  }
  const list = document.createElement("dl");
  list.className = "props";
  switch (selection.kind) {
    case "course": courseRows(list, state, selection.id); break;
    case "control": controlRows(host, list, state, selection.id, actions); break;
    case "special": specialRows(list, state, selection.id); break;
    case "class": classRows(list, state, selection.id); break;
    case "number": {
      const n = overlay?.numbers.find((x) => String(x.index) === String(selection.id));
      if (n) {
        row(list, "editor-prop-code", n.code);
        row(list, "editor-prop-name", n.text);
      }
      break;
    }
    default: break;
  }
  if (selection.kind !== "control") host.append(list);
  if (focused) host.querySelector(`#${focused}`)?.focus();
  if (selection.kind === "course" && overlay?.numbers.some((n) => n.source === "auto")) {
    const note = document.createElement("p");
    note.className = "hint";
    note.textContent = t("editor-legend-auto-number");
    host.append(note);
  }
}
