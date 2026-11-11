// Course editing: the list edits behind course mode and the stop list, the name dialog, and the property
// panels of a course's details, a category and the event. The page adds no rules of its own: every change is
// one operation the server checks. Values are put in with textContent or an input's value only.

import { t } from "./texts.js";

export const MAX_NAME_CHARS = 200;

// Whether a code or name is fit to commit: not empty, no white space at its ends, within the limit, and not
// the text of another entry. The server checks the same and has the last word.
// The rule categories of a profile and its overlays, sorted; `known` is the answer of api/rules.
export function categoriesOf(known, profile, overlays) {
  const found = new Set();
  for (const id of [profile, ...overlays]) for (const c of known.categories?.[id] ?? []) found.add(c);
  return found.size === 0 ? null : [...found].sort();
}

export function nameProblem(text, limit, others) {
  if (text === "" || text.trim() !== text) return true;
  return [...text].length > limit || others.includes(text);
}

// A whole number of at most nine digits; null for anything else.
export function parseCount(text) {
  const s = text.trim();
  return /^\d{1,9}$/.test(s) ? Number(s) : null;
}

// --- Stop lists: pure edits; each result is sent as one set_course_controls ----------------------------------
// `after` counts the stops before a leg: 0 is the leg from the start, n the leg leaving the n-th stop.

export const withAppended = (codes, code) => [...codes, code];

export function withInsertedAt(codes, after, code) {
  const at = Math.min(Math.max(after, 0), codes.length);
  return [...codes.slice(0, at), code, ...codes.slice(at)];
}

// The list with the stop at `from` moved by `delta` places, or null when it would leave the list.
export function withMoved(codes, from, delta) {
  const to = from + delta;
  if (from < 0 || from >= codes.length || to < 0 || to >= codes.length) return null;
  const next = [...codes];
  [next[from], next[to]] = [next[to], next[from]];
  return next;
}

export const withoutAt = (codes, index) => codes.filter((_, i) => i !== index);

export function legLabel(after) {
  return after === 0 ? t("editor-leg-from-start") : `${t("editor-leg-after")} ${after}`;
}

// The status text after set_course_controls: the legs that lost their bends or tape are named.
export function changedText(cleared) {
  const base = t("editor-status-course-changed");
  if (!cleared || cleared.length === 0) return base;
  const legs = cleared.map((leg) => `${leg.course} (${legLabel(leg.after)})`).join(", ");
  return `${base} ${t("editor-status-cleared")}: ${legs}`;
}

// --- The name dialog ---------------------------------------------------------------------------------------------

// Asks for a new name; resolves to the text, or null when the dialog is cancelled.
export function askName({ titleId, labelId, errorId, others }) {
  const dialog = document.getElementById("name-dialog");
  const form = document.getElementById("name-form");
  const input = document.getElementById("name-input");
  const error = document.getElementById("name-error");
  document.getElementById("name-title").textContent = t(titleId);
  document.getElementById("name-label").textContent = t(labelId);
  error.textContent = t(errorId);
  error.hidden = true;
  input.removeAttribute("aria-invalid");
  input.value = "";
  return new Promise((resolve) => {
    let result = null;
    form.onsubmit = (evt) => {
      evt.preventDefault();
      const bad = nameProblem(input.value, MAX_NAME_CHARS, others);
      input.setAttribute("aria-invalid", String(bad));
      error.hidden = !bad;
      if (bad) return;
      result = input.value;
      dialog.close();
    };
    document.getElementById("name-cancel").onclick = () => dialog.close();
    dialog.onclose = () => resolve(result);
    dialog.showModal();
    input.focus();
  });
}

// --- Property fields ---------------------------------------------------------------------------------------------

// A labelled text input in a definition list; with an `errorId` the cell also holds a hidden message the input
// points to, which the field's own check shows.
function inputRow(list, { id, labelId, value, numeric, errorId }) {
  const term = document.createElement("dt");
  const label = document.createElement("label");
  label.htmlFor = id;
  label.textContent = t(labelId);
  term.append(label);
  const detail = document.createElement("dd");
  const input = document.createElement("input");
  input.id = id;
  input.type = "text";
  if (numeric) input.inputMode = "numeric";
  input.autocomplete = "off";
  input.value = value;
  let error = null;
  if (errorId) {
    error = document.createElement("p");
    error.id = `${id}-error`;
    error.className = "field-error";
    error.hidden = true;
    error.textContent = t(errorId);
    input.setAttribute("aria-describedby", error.id);
  }
  // Text typed but not yet committed must survive a reload of the page's data.
  input.addEventListener("input", () => {
    input.dataset.dirty = "1";
  });
  detail.append(input);
  if (error) detail.append(error);
  list.append(term, detail);
  return { input, error };
}

// A labelled drop-down list. `options` is [{ value, label }]; `chosen` the selected values. A chosen value the
// list does not know (a file edited by hand) is kept as an entry of its own, so it shows and is not lost.
function selectRow(list, { id, labelId, options, chosen, multiple }) {
  const term = document.createElement("dt");
  const label = document.createElement("label");
  label.htmlFor = id;
  label.textContent = t(labelId);
  term.append(label);
  const detail = document.createElement("dd");
  const select = document.createElement("select");
  select.id = id;
  select.multiple = !!multiple;
  const known = new Set(options.map((o) => o.value));
  const all = [...options, ...chosen.filter((v) => v !== "" && !known.has(v)).map((v) => ({ value: v, label: v }))];
  for (const entry of all) {
    const option = document.createElement("option");
    option.value = entry.value;
    option.textContent = entry.label;
    option.selected = chosen.includes(entry.value);
    select.append(option);
  }
  if (multiple) select.size = Math.min(4, Math.max(2, all.length));
  detail.append(select);
  list.append(term, detail);
  return select;
}

// An optional whole number: empty means "none" (commit(null)), a number is committed when it differs.
function countRow(list, { id, labelId, value, commit }) {
  const { input, error } = inputRow(list, {
    id, labelId, value: value == null ? "" : String(value), numeric: true, errorId: "editor-error-count",
  });
  input.addEventListener("change", () => {
    delete input.dataset.dirty;
    const text = input.value.trim();
    const n = text === "" ? null : parseCount(text);
    const bad = text !== "" && n === null;
    input.setAttribute("aria-invalid", String(bad));
    error.hidden = !bad;
    if (!bad && n !== (value ?? null)) commit(n);
  });
}

function button(id, textId, onClick) {
  const b = document.createElement("button");
  b.type = "button";
  if (id) b.id = id;
  b.textContent = t(textId);
  b.addEventListener("click", onClick);
  return b;
}

function removeButton(id, textId, onClick) {
  const b = button(id, textId, onClick);
  b.className = "danger";
  return b;
}

// --- A course ------------------------------------------------------------------------------------------------------

// Climb, planned print scale and the categories of a course, as rows of the properties list.
export function courseFields(list, state, course, actions) {
  countRow(list, {
    id: "prop-climb",
    labelId: "editor-prop-climb",
    value: course.climb_m,
    commit: (n) => actions.setDetails(course.name, n === null ? { no_climb: true } : { climb_m: n }),
  });
  countRow(list, {
    id: "prop-print-scale",
    labelId: "editor-prop-print-scale",
    value: course.planned_print_scale,
    commit: (n) => actions.setDetails(course.name, n === null ? { no_print_scale: true } : { print_scale: n }),
  });
  const term = document.createElement("dt");
  term.textContent = t("editor-prop-classes");
  const detail = document.createElement("dd");
  const group = document.createElement("div");
  group.setAttribute("role", "group");
  group.setAttribute("aria-label", t("editor-prop-classes"));
  const chosen = course.classes ?? [];
  if (state.event.classes.length === 0) detail.textContent = t("editor-none");
  state.event.classes.forEach((cls, i) => {
    const label = document.createElement("label");
    label.className = "check";
    const box = document.createElement("input");
    box.type = "checkbox";
    box.id = `prop-class-${i}`;
    box.checked = chosen.includes(cls.name);
    box.addEventListener("change", () => {
      const names = state.event.classes
        .filter((c, j) => (j === i ? box.checked : chosen.includes(c.name)))
        .map((c) => c.name);
      actions.setDetails(course.name, { classes: names });
    });
    label.append(box, ` ${cls.name}`);
    group.append(label);
  });
  if (state.event.classes.length > 0) detail.append(group);
  list.append(term, detail);
}

// The ordered stops of an individual course. Each row has buttons to move it and to remove it; with the focus
// on a row, Alt+Up and Alt+Down move it. Every change is one actions.setStops(course, codes, focusId).
export function stopList(host, course, actions) {
  const heading = document.createElement("h3");
  heading.textContent = t("editor-prop-stops");
  host.append(heading);
  const codes = course.controls.map((c) => String(c.code));
  if (codes.length === 0) {
    const none = document.createElement("p");
    none.className = "hint";
    none.textContent = t("editor-stops-empty");
    host.append(none);
    return;
  }
  const hint = document.createElement("p");
  hint.className = "hint";
  hint.textContent = t("editor-stops-hint");
  const list = document.createElement("ol");
  list.className = "stops";
  list.setAttribute("aria-label", t("editor-prop-stops"));
  codes.forEach((code, i) => {
    const li = document.createElement("li");
    li.id = `stop-${i}`;
    li.tabIndex = 0;
    const name = document.createElement("span");
    name.className = "stop-code";
    name.textContent = code;
    li.append(name);
    const move = (delta) => {
      const next = withMoved(codes, i, delta);
      if (next) actions.setStops(course.name, next, `stop-${i + delta}`);
    };
    for (const [delta, id] of [[-1, "editor-stop-up"], [1, "editor-stop-down"]]) {
      const b = button(`stop-${i}-${delta < 0 ? "up" : "down"}`, id, () => move(delta));
      b.setAttribute("aria-label", `${t(id)} ${code}`);
      b.disabled = i + delta < 0 || i + delta >= codes.length;
      li.append(b);
    }
    const remove = button(`stop-${i}-remove`, "editor-stop-remove", () => {
      actions.setStops(course.name, withoutAt(codes, i), `stop-${Math.min(i, codes.length - 2)}`);
    });
    remove.setAttribute("aria-label", `${t("editor-stop-remove")} ${code}`);
    li.append(remove);
    li.addEventListener("keydown", (evt) => {
      if (evt.target !== li || !evt.altKey || evt.ctrlKey || evt.metaKey) return;
      if (evt.key !== "ArrowUp" && evt.key !== "ArrowDown") return;
      evt.preventDefault();
      move(evt.key === "ArrowUp" ? -1 : 1);
    });
    list.append(li);
  });
  host.append(list, hint);
}

export function courseRemoveButton(host, course, actions) {
  host.append(removeButton("prop-delete-course", "editor-prop-delete-course", () => actions.removeCourse(course.name)));
}

// --- A category ------------------------------------------------------------------------------------------------------

export function classPanel(host, list, state, name, actions) {
  const cls = state.event.classes.find((c) => c.name === name);
  if (!cls) return;
  const term = document.createElement("dt");
  term.textContent = t("editor-prop-name");
  const detail = document.createElement("dd");
  detail.textContent = cls.name;
  list.append(term, detail);
  const known = actions.rules?.() ?? null;
  const meta = state.event.event;
  const names = known ? categoriesOf(known, meta.rule_profile, meta.rule_overlays ?? []) : null;
  if (names) {
    const select = selectRow(list, {
      id: "prop-rule-category",
      labelId: "editor-prop-rule-category",
      options: [{ value: "", label: t("editor-none") }, ...names.map((n) => ({ value: n, label: n }))],
      chosen: [cls.rule_category ?? ""],
    });
    select.addEventListener("change", () => {
      if (select.value === (cls.rule_category ?? "")) return;
      actions.setClass(name, select.value === "" ? { no_rule_category: true } : { rule_category: select.value });
    });
  } else {
    const { input } = inputRow(list, {
      id: "prop-rule-category",
      labelId: "editor-prop-rule-category",
      value: cls.rule_category ?? "",
    });
    input.addEventListener("change", () => {
      delete input.dataset.dirty;
      const text = input.value.trim();
      if (text === (cls.rule_category ?? "")) return;
      actions.setClass(name, text === "" ? { no_rule_category: true } : { rule_category: text });
    });
  }
  countRow(list, {
    id: "prop-starters",
    labelId: "editor-prop-starters",
    value: cls.expected_starters,
    commit: (n) => actions.setClass(name, n === null ? { no_expected_starters: true } : { expected_starters: n }),
  });
  const users = state.event.courses.filter((c) => (c.classes ?? []).includes(name)).map((c) => c.name);
  const usedTerm = document.createElement("dt");
  usedTerm.textContent = t("editor-prop-used-in");
  const usedDetail = document.createElement("dd");
  usedDetail.textContent = users.length ? users.join(", ") : t("editor-none");
  list.append(usedTerm, usedDetail);
  host.append(list);
  host.append(removeButton("prop-delete-class", "editor-prop-delete-class", () => actions.removeClass(name)));
}

// --- The event -------------------------------------------------------------------------------------------------------

export function eventPanel(host, list, state, actions) {
  const meta = state.event.event;
  const term = document.createElement("dt");
  term.textContent = t("editor-prop-discipline");
  const detail = document.createElement("dd");
  detail.textContent = meta.discipline;
  list.append(term, detail);
  const known = actions.rules?.() ?? null;
  if (known) {
    const profile = selectRow(list, {
      id: "prop-rule-profile",
      labelId: "editor-prop-rule-profile",
      options: known.profiles.map((p) => ({ value: p.id, label: p.id })),
      chosen: [meta.rule_profile],
    });
    const overlays = selectRow(list, {
      id: "prop-rule-overlays",
      labelId: "editor-prop-rule-overlays",
      options: known.overlays.map((o) => ({ value: o.id, label: o.id })),
      chosen: meta.rule_overlays ?? [],
      multiple: true,
    });
    const commit = () => {
      const names = [...overlays.selectedOptions].map((o) => o.value);
      if (profile.value === meta.rule_profile && names.join(",") === (meta.rule_overlays ?? []).join(",")) return;
      actions.setProfile(profile.value, names);
    };
    profile.addEventListener("change", commit);
    overlays.addEventListener("change", commit);
  } else {
    const base = inputRow(list, { id: "prop-rule-profile", labelId: "editor-prop-rule-profile", value: meta.rule_profile });
    const overlays = inputRow(list, {
      id: "prop-rule-overlays",
      labelId: "editor-prop-rule-overlays",
      value: (meta.rule_overlays ?? []).join(", "),
    });
    const commitProfile = () => {
      delete base.input.dataset.dirty;
      delete overlays.input.dataset.dirty;
      const name = base.input.value.trim();
      const names = overlays.input.value.split(",").map((s) => s.trim()).filter((s) => s !== "");
      const same = name === meta.rule_profile && names.join(",") === (meta.rule_overlays ?? []).join(",");
      if (name === "" || same) return;
      actions.setProfile(name, names);
    };
    base.input.addEventListener("change", commitProfile);
    overlays.input.addEventListener("change", commitProfile);
  }
  const startWin = inputRow(list, {
    id: "prop-start-window",
    labelId: "editor-prop-start-window",
    value: meta.start_window_min == null ? "" : String(meta.start_window_min),
    numeric: true,
    errorId: "editor-error-count",
  });
  startWin.input.addEventListener("change", () => {
    delete startWin.input.dataset.dirty;
    // Empty removes the window; a number sets it.
    const text = startWin.input.value.trim();
    const n = text === "" ? null : parseCount(text);
    const bad = text !== "" && n === null;
    startWin.input.setAttribute("aria-invalid", String(bad));
    startWin.error.hidden = !bad;
    if (!bad && n !== (meta.start_window_min ?? null)) actions.setWindow(n);
  });
  host.append(list);
}
