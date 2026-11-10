// Read-only properties of the selection. Values are put in with textContent only.

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

function controlRows(list, state, code) {
  const control = state.event.controls.find((c) => String(c.code) === String(code));
  if (!control) return;
  row(list, "editor-prop-code", control.code);
  row(list, "editor-prop-kind", kindLabel(control.kind ?? "control"));
  row(list, "editor-prop-position", mm(control.map_mm));
  for (const column of COLUMNS) {
    row(list, `editor-desc-column-${column}`, control.description?.[column]);
  }
  const users = state.event.courses
    .filter((c) => c.controls.some((x) => String(x.code) === String(code)))
    .map((c) => c.name);
  row(list, "editor-prop-used-in", users.join(", "));
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
export function show(host, state, selection, overlay) {
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
    case "control": controlRows(list, state, selection.id); break;
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
  host.append(list);
  if (selection.kind === "course" && overlay?.numbers.some((n) => n.source === "auto")) {
    const note = document.createElement("p");
    note.className = "hint";
    note.textContent = t("editor-legend-auto-number");
    host.append(note);
  }
}
