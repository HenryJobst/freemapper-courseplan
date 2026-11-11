// Properties of the selection. Values are put in with textContent or an input's value only.
// A control's position and code and a course's name are editable; the kind is shown as it is.

import { kindLabel } from "./overlay.js";
import { t } from "./texts.js";
import { legLabel } from "./course.js";
import {
  MAX_NAME_CHARS, nameProblem, courseFields, stopList, courseRemoveButton, classPanel, eventPanel,
} from "./course.js";

export { nameProblem };

let wantFocus = null;

// The element with this id gets the focus when the properties are built next (a moved stop keeps it).
export function focusAfter(id) {
  wantFocus = id;
}

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

const MAX_CODE_CHARS = 64;

// A text field for the code of a control or the name of a course. A refused text shows `errorId`'s message at
// the field; a good one different from `value` is passed to commit(text). Returns the message element.
function nameField(list, { id, labelId, value, limit, others, errorId, commit }) {
  const term = document.createElement("dt");
  const label = document.createElement("label");
  label.htmlFor = id;
  label.textContent = t(labelId);
  term.append(label);
  const detail = document.createElement("dd");
  const input = document.createElement("input");
  input.id = id;
  input.type = "text";
  input.autocomplete = "off";
  input.value = value;
  detail.append(input);
  list.append(term, detail);
  const error = document.createElement("p");
  error.id = `${id}-error`;
  error.className = "field-error";
  error.hidden = true;
  error.textContent = t(errorId);
  input.setAttribute("aria-describedby", error.id);
  input.addEventListener("input", () => {
    input.dataset.dirty = "1";
  });
  input.addEventListener("change", () => {
    delete input.dataset.dirty;
    const bad = nameProblem(input.value, limit, others);
    input.setAttribute("aria-invalid", String(bad));
    error.hidden = !bad;
    if (!bad && input.value !== value) commit(input.value);
  });
  return error;
}

function courseRows(host, list, state, name, actions) {
  const course = state.event.courses.find((c) => c.name === name);
  if (!course) return;
  const info = state.courses.find((c) => c.name === name);
  const error = nameField(list, {
    id: "prop-name",
    labelId: "editor-prop-name",
    value: course.name,
    limit: MAX_NAME_CHARS,
    others: state.event.courses.map((c) => c.name).filter((n) => n !== name),
    errorId: "editor-error-course-name",
    commit: (to) => actions.renameCourse(name, to),
  });
  host.append(list, error);
  row(list, "editor-prop-kind", t("editor-kind-course"));
  row(list, "editor-prop-length", info?.length_m == null ? null : `${Math.round(info.length_m)} m`);
  row(list, "editor-prop-start", course.start);
  row(list, "editor-prop-finish", course.finish);
  courseFields(list, state, course, actions);
  stopList(host, course, actions);
  courseRemoveButton(host, course, actions);
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
  // defaultValue keeps the text shown for the file's value, to tell an edited field from an untouched one.
  input.defaultValue = shown(value);
  input.value = input.defaultValue;
  detail.append(input);
  list.append(term, detail);
  return input;
}

// The value of a position field: the exact number from the file while the text is the one shown for it (the
// display is rounded, so editing the other axis must not round this one), else the number typed. Null when the
// text is no number.
export function positionOf(input, exact) {
  return input.value === input.defaultValue ? exact : parseMm(input.value);
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
    // Text typed but not yet committed must survive a reload of the page's data.
    input.addEventListener("input", () => {
      input.dataset.dirty = "1";
    });
    input.addEventListener("change", () => {
      delete input.dataset.dirty;
      const nx = positionOf(inputs[0], x);
      const ny = positionOf(inputs[1], y);
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
  const codeError = nameField(list, {
    id: "prop-code",
    labelId: "editor-prop-code",
    value: String(control.code),
    limit: MAX_CODE_CHARS,
    others: state.event.controls.map((c) => String(c.code)).filter((c) => c !== String(code)),
    errorId: "editor-error-code",
    commit: (to) => actions.changeCode(String(control.code), to),
  });
  row(list, "editor-prop-kind", kindLabel(control.kind ?? "control"));
  const error = positionFields(list, control, actions);
  // A crossing point has no control description.
  if (control.kind !== "crossing_point") {
    for (const column of COLUMNS) {
      row(list, `editor-desc-column-${column}`, control.description?.[column]);
    }
  }
  const users = state.event.courses
    .filter((c) => c.controls.some((x) => String(x.code) === String(code)))
    .map((c) => c.name);
  row(list, "editor-prop-used-in", users.join(", "));
  host.append(list, codeError, error);
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

// A selected leg or bend: the course and the leg, the number of bends and the taped box. The course length is
// shown here too, because every bend changes it.
function legRows(host, list, state, selection, overlay, actions) {
  const leg = overlay?.legs.find((l) => l.after === selection.after);
  if (!leg) return;
  const info = state.courses.find((c) => c.name === selection.course);
  row(list, "editor-prop-kind", t("editor-leg-word"));
  row(list, "editor-prop-name", selection.course);
  row(list, "editor-leg-word", legLabel(leg.after));
  row(list, "editor-prop-bends", leg.bends.length);
  if (selection.kind === "bend") row(list, "editor-prop-bend", `${selection.index + 1} / ${leg.bends.length}`);
  row(list, "editor-prop-length", info?.length_m == null ? null : `${Math.round(info.length_m)} m`);
  const term = document.createElement("dt");
  const label = document.createElement("label");
  label.htmlFor = "prop-taped";
  label.textContent = t("editor-prop-taped");
  term.append(label);
  const detail = document.createElement("dd");
  const box = document.createElement("input");
  box.type = "checkbox";
  box.id = "prop-taped";
  box.checked = leg.taped;
  box.addEventListener("change", () => actions.setTaped(leg.after, box.checked));
  detail.append(box);
  list.append(term, detail);
}

function specialRows(list, state, id) {
  const special = state.event.specials.find((s) => s.id === id);
  if (!special) return;
  row(list, "editor-prop-id", special.id);
  row(list, "editor-prop-kind", kindLabel(special.kind));
  const at = specialPosition(special);
  row(list, "editor-prop-position", at ? mm(at) : null);
}


// Fills `host` for the selection { kind, id, course? }, or the empty hint without one.
export function show(host, state, selection, overlay, actions) {
  // A field with text typed but not committed keeps it: nothing is rebuilt under the cursor.
  const key = selection ? `${selection.kind}:${selection.id}` : "";
  const active = document.activeElement;
  if (state && selection && host.dataset.key === key && host.contains(active) && active.dataset?.dirty) return;
  host.dataset.key = key;
  // Keep the focus in the field being edited: the page is rebuilt after each change.
  const focused = wantFocus ?? (host.contains(document.activeElement) ? document.activeElement.id : "");
  wantFocus = null;
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
    case "course": courseRows(host, list, state, selection.id, actions); break;
    case "control": controlRows(host, list, state, selection.id, actions); break;
    case "leg":
    case "bend": legRows(host, list, state, selection, overlay, actions); break;
    case "special": specialRows(list, state, selection.id); break;
    case "class": classPanel(host, list, state, selection.id, actions); break;
    case "event": eventPanel(host, list, state, actions); break;
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
  if (!["control", "course", "class", "event"].includes(selection.kind)) host.append(list);
  if (focused) host.querySelector(`#${focused}`)?.focus();
  if (selection.kind === "course" && overlay?.numbers.some((n) => n.source === "auto")) {
    const note = document.createElement("p");
    note.className = "hint";
    note.textContent = t("editor-legend-auto-number");
    host.append(note);
  }
}
