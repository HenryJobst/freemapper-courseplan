// The Score panel of a score course (ADR 0069). The course itself is edited with the ordinary operations; what lives
// here is the part the file does not hold: the candidates of "Suggest" and the draft of "Draft". They stay in this
// module until they are accepted, applied or discarded, and a change of the project from outside discards them.
// Long runs go through ctx.runScore (api/score/{tool}); while one runs, the buttons are disabled and a progress line
// says so. Values are put in with textContent or an input's value only.

import { t } from "./texts.js";
import { parseCount } from "./course.js";

const NS = "http://www.w3.org/2000/svg";
const DASH = "–";

// { course, candidates, draft, length, revision }: what the page holds for one course, or nothing.
const empty = () => ({ course: null, candidates: [], draft: null, length: null, revision: null });
let results = empty();
let working = false;
// The id of the note that says why results are gone, or "" (shown in the panel until the next run).
let note = "";
let ctx = null;

// ctx = { runOp, runScore, redraw, say, state, focusAfter(id), lastFailure(), showBanner(text) } from main.js.
export function bind(context) {
  ctx = context;
}

export function setWorking(value) {
  working = value;
}

const hasResults = () => results.candidates.length > 0 || results.draft !== null || results.length !== null;

// A write of this page keeps the results; the revision they belong to moves along.
export function rebase(revision) {
  if (!hasResults()) return;
  results.revision = revision;
  // A length report describes the course as it was; any write makes it stale.
  results.length = null;
}

// True when results were dropped because the project changed under them.
export function checkRevision(revision) {
  if (!hasResults() || results.revision === revision) return false;
  results = empty();
  note = "editor-score-discarded";
  return true;
}

// Selecting another course (or none) drops the results of the one they were made for.
export function forget(course) {
  if (results.course !== null && results.course !== course) results = empty();
}

function node(name, attributes = {}, text = null) {
  const n = document.createElement(name);
  for (const [key, value] of Object.entries(attributes)) {
    if (key === "class") n.className = value;
    else n.setAttribute(key, value);
  }
  if (text !== null) n.textContent = text;
  return n;
}

function button(id, textId, onClick, { disabled = false, label = null } = {}) {
  const b = node("button", { type: "button", id }, t(textId));
  if (label) b.setAttribute("aria-label", label);
  b.disabled = disabled || working;
  b.addEventListener("click", onClick);
  return b;
}

function row(list, labelId, value) {
  list.append(node("dt", {}, t(labelId)), node("dd", {}, String(value)));
}

// A whole-number field: a good value is passed to commit(n), anything else is marked invalid.
function numberRow(list, id, labelId, value, commit) {
  const term = node("dt");
  const label = node("label", { for: id }, t(labelId));
  term.append(label);
  const detail = node("dd");
  const input = node("input", { id, type: "text", inputmode: "numeric", autocomplete: "off" });
  input.value = value === null || value === undefined ? "" : String(value);
  input.disabled = working;
  input.addEventListener("input", () => {
    input.dataset.dirty = "1";
  });
  input.addEventListener("change", () => {
    delete input.dataset.dirty;
    const n = parseCount(input.value);
    input.setAttribute("aria-invalid", String(n === null));
    if (n !== null && n !== value) commit(n);
  });
  detail.append(input);
  list.append(term, detail);
  return input;
}

function table(columns, rows) {
  const tbl = node("table", { class: "score-table" });
  const head = node("tr");
  for (const id of columns) head.append(node("th", { scope: "col" }, t(id)));
  const thead = node("thead");
  thead.append(head);
  tbl.append(thead);
  const body = node("tbody");
  for (const cells of rows) {
    const tr = node("tr");
    for (const cell of cells) {
      if (cell instanceof Node) {
        const td = node("td");
        td.append(cell);
        tr.append(td);
      } else {
        tr.append(node("td", {}, String(cell)));
      }
    }
    body.append(tr);
  }
  tbl.append(body);
  return tbl;
}

// --- Flows -----------------------------------------------------------------------------------------------------------

const courseOf = () => {
  const s = ctx.state();
  return s.data.event.courses.find((c) => c.name === s.course) ?? null;
};

// Redraws and puts the focus back on the control that started the action (the pane is rebuilt on every redraw).
function done(focusId) {
  ctx.focusAfter(focusId);
  ctx.redraw();
}

async function suggest() {
  const course = courseOf();
  if (!course) return;
  note = "";
  const answer = await ctx.runScore("score_suggest", { course: course.name });
  if (!answer || ctx.state().course !== course.name) return done("score-suggest");
  results = { ...empty(), course: course.name, candidates: answer.result.candidates, revision: answer.revision };
  ctx.say("editor-score-status-suggested");
  done("score-suggest");
}

async function draft() {
  const course = courseOf();
  if (!course) return;
  note = "";
  const body = { name: course.name, time_limit_min: course.time_limit_min };
  const answer = await ctx.runScore("score_draft", body);
  if (!answer || ctx.state().course !== course.name) return done("score-draft");
  results = { ...empty(), course: course.name, draft: answer.result, revision: answer.revision };
  ctx.say("editor-score-status-drafted");
  done("score-apply");
}

// Writes the draft the person sees, as it is: no new search, so nothing else can be written.
async function apply() {
  const course = courseOf();
  const shown = results.draft;
  if (!course || shown === null) return;
  const body = {
    name: course.name,
    time_limit_min: shown.time_limit_min,
    controls: shown.controls.map((c) => ({ code: c.code, value: c.value, at: c.at })),
    links: shown.links,
  };
  const written = await ctx.runOp("score_apply_draft", body);
  if (!written) return done("score-apply");
  results = empty();
  ctx.say("editor-score-status-applied");
  done("score-draft");
}

async function length() {
  const course = courseOf();
  if (!course) return;
  note = "";
  const answer = await ctx.runScore("score_length", { name: course.name });
  if (!answer || ctx.state().course !== course.name) return done("score-length");
  results = { ...empty(), course: course.name, length: answer.result.length, revision: answer.revision };
  ctx.say("editor-score-status-length");
  done("score-length");
}

function discard() {
  results = empty();
  note = "";
  done("score-suggest");
}

// A change of the limit or of the shared settings makes candidates and draft stale: they are dropped.
async function change(tool, body, focusId) {
  ctx.focusAfter(focusId);
  const written = await ctx.runOp(tool, body);
  if (written) results = empty();
  done(focusId);
}

// Two operations, so that each is one undo step: the control, then the course with the control in it. If the
// second one is refused, the control is taken out again, so a retry does not meet "code exists".
async function accept(candidate) {
  const name = results.course;
  if (!name || !candidate.code) return;
  const next = results.candidates[results.candidates.indexOf(candidate) + 1] ?? null;
  const after = next?.code ? `score-accept-${next.code}` : "score-suggest";
  ctx.focusAfter(after);
  const added = await ctx.runOp("add_control", { at: candidate.at, code: candidate.code });
  if (!added) return done(after);
  const course = ctx.state().data.event.courses.find((c) => c.name === name);
  if (!course) return done(after);
  const controls = course.controls.map((c) => ({ code: String(c.code), value: c.value }));
  controls.push({ code: candidate.code, value: candidate.value });
  const set = await ctx.runOp("set_score_controls", { name, controls });
  if (!set) {
    const reason = ctx.lastFailure();
    await ctx.runOp("remove_control", { code: candidate.code });
    if (reason) ctx.showBanner(reason);
    return done(`score-accept-${candidate.code}`);
  }
  results.candidates = results.candidates.filter((c) => c !== candidate);
  ctx.say("editor-score-status-accepted");
  done(after);
}

// --- Panel -----------------------------------------------------------------------------------------------------------

function reportRows(list, length) {
  const band = (b) => `${b[0]}${DASH}${b[1]} %`;
  if (length.good) {
    row(list, "editor-score-good", `${length.good.points} / ${length.all_points}`);
    row(list, "editor-score-good-share", `${length.good.percent} % (${band(length.good.band_percent)})`);
    row(list, "editor-score-very-good", length.very_good.points);
    row(list, "editor-score-very-good-share", `${length.very_good.finish_percent} % (${band(length.very_good.band_percent)})`);
  }
}

function reportBlock(length) {
  const block = node("div", { class: "score-report" });
  block.append(node("h4", {}, t("editor-score-length-heading")));
  const list = node("dl", { class: "props" });
  reportRows(list, length);
  block.append(list);
  block.append(node("p", {}, t(length.ok ? "editor-score-bands-ok" : "editor-score-bands-missed")));
  if (length.issues.length > 0) {
    const issues = node("ul");
    // The sentences come formatted from the server.
    for (const issue of length.issues) issues.append(node("li", {}, issue.text));
    block.append(issues);
  }
  return block;
}

function candidateTable(candidates) {
  const rows = candidates.map((c) => {
    const act = button(`score-accept-${c.code ?? "none"}`, "editor-score-accept", () => accept(c), {
      disabled: !c.code,
      label: `${t("editor-score-accept")} ${c.code ?? t("editor-score-no-code")}`,
    });
    return [c.code ?? t("editor-score-no-code"), c.value, c.route_choice, c.difficulty, Math.round(c.effort_s), act];
  });
  const tbl = table(
    ["editor-score-col-code", "editor-score-col-value", "editor-score-col-choice", "editor-score-col-difficulty", "editor-score-col-effort", "editor-score-accept"],
    rows,
  );
  return tbl;
}

function draftBlock(result) {
  const block = node("div", { class: "score-draft" });
  block.append(node("h4", {}, t("editor-score-draft-result")));
  const list = node("dl", { class: "props" });
  row(list, "editor-score-draft-hard", `${result.mix.hard_actual} / ${result.mix.hard_wanted}`);
  block.append(list);
  block.append(table(["editor-score-col-code", "editor-score-col-value"], result.controls.map((c) => [c.code, c.value])));
  block.append(reportBlock(result.length));
  const bar = node("p", { class: "score-actions" });
  bar.append(button("score-apply", "editor-score-apply", apply), button("score-discard-draft", "editor-score-discard", discard));
  block.append(bar);
  return block;
}

// Fills `host` with the panel of the score course `course` (an entry of the event's courses).
export function panel(host, state, course) {
  const name = course.name;
  const section = node("section", { class: "score", "aria-labelledby": "score-heading" });
  section.append(node("h3", { id: "score-heading" }, t("editor-score-heading")));

  const bar = node("p", { class: "score-actions" });
  bar.append(
    button("score-suggest", "editor-score-suggest", suggest),
    button("score-draft", "editor-score-draft", draft),
    button("score-length", "editor-score-length-run", length),
  );
  section.append(bar);
  section.append(node("p", { id: "score-progress", class: "hint", role: "status" }, working ? t("editor-score-working") : ""));
  if (note) section.append(node("p", { class: "hint" }, t(note)));

  const fields = node("dl", { class: "props" });
  numberRow(fields, "score-limit", "editor-score-limit", course.time_limit_min, (n) =>
    change("set_score_course_details", { name, time_limit_min: n }, "score-limit"));
  const stored = course.penalty_per_min ?? null;
  const term = node("dt");
  term.append(node("label", { for: "score-penalty-default" }, t("editor-score-penalty-default")));
  const detail = node("dd");
  const box = node("input", { id: "score-penalty-default", type: "checkbox" });
  box.checked = stored === null;
  box.disabled = working;
  detail.append(box);
  fields.append(term, detail);
  const penalty = numberRow(fields, "score-penalty", "editor-score-penalty", stored, (n) =>
    ctx.runOp("set_score_course_details", { name, penalty_per_min: n }));
  penalty.disabled = working || box.checked;
  box.addEventListener("change", () => {
    penalty.disabled = box.checked;
    if (box.checked) ctx.runOp("set_score_course_details", { name, no_penalty: true });
    else penalty.focus();
  });
  section.append(fields);

  section.append(node("h4", {}, t("editor-score-controls")));
  section.append(table(["editor-score-col-code", "editor-score-col-value"], course.controls.map((c) => [String(c.code), c.value])));

  section.append(node("h4", {}, t("editor-score-links")));
  const links = course.links ?? [];
  if (links.length === 0) {
    section.append(node("p", { class: "hint" }, t("editor-score-links-none")));
  } else {
    const list = node("ul", { class: "score-links" });
    for (const [i, link] of links.entries()) {
      const item = node("li", {}, `${link.a} ${DASH} ${link.b} `);
      item.append(button(`score-link-remove-${i}`, "editor-score-link-remove", () =>
        ctx.runOp("remove_score_link", { name, a: String(link.a), b: String(link.b) }), {
        label: `${t("editor-score-link-remove")} ${link.a} ${DASH} ${link.b}`,
      }));
      list.append(item);
    }
    section.append(list);
    section.append(node("p", { class: "hint" }, t("editor-score-link-hint")));
  }

  const s = state.score_settings;
  if (s) {
    section.append(node("h4", {}, t("editor-score-settings")));
    const settings = node("dl", { class: "props" });
    for (const [id, labelId, key] of [
      ["score-value-max", "editor-score-value-max", "value_max"],
      ["score-good-pace", "editor-score-good-pace", "good_pace_s_per_km"],
      ["score-very-good-pace", "editor-score-very-good-pace", "very_good_pace_s_per_km"],
      ["score-mix-hard", "editor-score-mix-hard", "mix_hard_percent"],
      ["score-seed", "editor-score-seed", "seed"],
    ]) {
      numberRow(settings, id, labelId, s[key], (n) => change("set_score_settings", { [key]: n }, id));
    }
    row(settings, "editor-score-good-band", `${s.good_band_percent[0]}${DASH}${s.good_band_percent[1]} %`);
    row(settings, "editor-score-very-good-band", `${s.very_good_band_percent[0]}${DASH}${s.very_good_band_percent[1]} %`);
    section.append(settings);
  }

  if (results.course === name) {
    if (results.candidates.length > 0 || results.draft) {
      section.append(node("p", { class: "hint" }, t("editor-score-candidates-hint")));
    }
    if (results.candidates.length > 0) {
      section.append(node("h4", {}, t("editor-score-candidates")));
      if (results.candidates.some((c) => !c.code)) {
        section.append(node("p", { class: "hint" }, t("editor-score-no-code-hint")));
      }
      section.append(candidateTable(results.candidates));
      const clear = node("p", { class: "score-actions" });
      clear.append(button("score-discard", "editor-score-discard", discard));
      section.append(clear);
    }
    if (results.draft) section.append(draftBlock(results.draft));
    if (results.length) section.append(reportBlock(results.length));
  }
  host.append(section);
}

// --- Markers on the map ----------------------------------------------------------------------------------------------

function svg(name, attributes, parent) {
  const n = document.createElementNS(NS, name);
  for (const [key, value] of Object.entries(attributes)) n.setAttribute(key, value);
  parent.append(n);
  return n;
}

// Candidates and draft controls as dashed circles with their value or code beside them, draft links as dashed
// lines. The dash and the text tell them from the course, so colour is not the only sign. `overlay` is the drawn
// course's overlay (it brings the sizes), `course` its name.
export function drawMarkers(layer, overlay, course) {
  layer.replaceChildren();
  if (!overlay || results.course !== course) return;
  const r = overlay.sizes.circle_r;
  const w = overlay.sizes.line;
  const mark = (at, text, cls, title) => {
    const group = svg("g", { class: cls }, layer);
    svg("circle", { cx: at.x, cy: at.y, r, "stroke-width": w, "stroke-dasharray": `${w * 2} ${w * 1.5}`, fill: "none" }, group);
    const label = svg("text", { x: at.x + r * 1.4, y: at.y - r * 1.1, "font-size": overlay.sizes.number_h * 0.9 }, group);
    label.textContent = text;
    svg("title", {}, group).textContent = title;
  };
  for (const c of results.candidates) mark(c.at, String(c.value), "cand", `${c.code ?? ""} ${c.value}`.trim());
  const result = results.draft;
  if (result) {
    const at = (code) => result.controls.find((c) => c.code === code)?.at;
    for (const link of result.links) {
      const a = at(link.a);
      const b = at(link.b);
      if (a && b) {
        svg("line", { x1: a.x, y1: a.y, x2: b.x, y2: b.y, class: "draft-link", "stroke-width": w, "stroke-dasharray": `${overlay.sizes.dash} ${overlay.sizes.dash_gap}` }, layer);
      }
    }
    for (const c of result.controls) mark(c.at, `${c.code}`, "draft-control", `${c.code} ${c.value}`);
  }
}
