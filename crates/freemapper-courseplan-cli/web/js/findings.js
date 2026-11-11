// The findings of the check (ADR 0052). The page asks api/check 500 ms after each change and on load. The panel
// on the right lists the findings of the selection, the status bar counts the whole event, and a drawer lists
// them all, with the accepted ones and the rules that did not run. Every text a finding carries was written by
// the server in the page's language: it is put in with textContent only, and so is everything else here.

import { get } from "./api.js";
import { t } from "./texts.js";
import * as status from "./status.js";

const DELAY_MS = 500;
export const MAX_NOTE_CHARS = 200;

let report = null;
let failed = false;
let hooks = null;
let timer = null;
let ticket = 0;
let panelSignature = "";
let drawerSignature = "";

const host = document.getElementById("findings");
const drawer = document.getElementById("findings-drawer");
const drawerBody = document.getElementById("drawer-body");
const openButton = document.getElementById("status-findings");
const acceptDialog = document.getElementById("accept-dialog");

export const current = () => report;
export const dialogOpen = () => acceptDialog.open;

// hooks: selection() -> the page's selection, event() -> the event, locate(finding), accept(finding, note),
// remove(accepted), changed() -> the report changed.
export function bind(h) {
  hooks = h;
  openButton.addEventListener("click", () => (drawer.hidden ? openDrawer() : closeDrawer()));
  document.getElementById("drawer-close").addEventListener("click", closeDrawer);
  drawer.addEventListener("keydown", (evt) => {
    if (evt.key !== "Escape") return;
    evt.preventDefault();
    closeDrawer();
  });
}

function openDrawer() {
  drawer.hidden = false;
  openButton.setAttribute("aria-expanded", "true");
  drawerSignature = "";
  renderDrawer();
  drawer.focus();
}

function closeDrawer() {
  const inside = drawer.contains(document.activeElement);
  drawer.hidden = true;
  openButton.setAttribute("aria-expanded", "false");
  if (inside) openButton.focus();
}

// Asks for the check 500 ms from now; a request that comes in meanwhile replaces the waiting one.
export function schedule() {
  clearTimeout(timer);
  timer = setTimeout(refresh, DELAY_MS);
}

export async function refresh() {
  const mine = ++ticket;
  let answer = null;
  try {
    answer = await get("api/check");
  } catch {
    answer = null;
  }
  if (mine !== ticket) return;
  if (answer?.ok && answer.data && Array.isArray(answer.data.findings)) {
    report = answer.data;
    failed = false;
  } else {
    report = null;
    failed = true;
  }
  status.setFindings(report?.summary ?? null);
  render();
  hooks?.changed();
}

// --- Which findings belong to the selection ------------------------------------------------------------------------

const codesOf = (loc) => (loc.kind === "control" ? [loc.code] : loc.kind === "controls" ? loc.codes : []);
const aftersOf = (loc) => (loc.kind === "leg" ? [loc.after] : loc.kind === "legs" ? loc.afters : []);
const endsOf = (loc) => (loc.kind === "leg" ? [loc] : loc.kind === "legs" ? loc.legs : []);

// The control a selection stands for: a control itself, or the control a number box belongs to.
function controlCode(selection, event) {
  if (selection.kind === "control") return String(selection.id);
  if (selection.kind !== "number") return null;
  const course = event.courses.find((c) => c.name === selection.course);
  const stop = course?.controls?.[selection.index - 1];
  return stop ? String(stop.code) : null;
}

// Whether a finding's place is the selection's place (or part of it).
export function touches(loc, selection, event) {
  if (!selection || !loc) return false;
  const code = controlCode(selection, event);
  if (code !== null) {
    return codesOf(loc).includes(code) || endsOf(loc).some((end) => end.from === code || end.to === code);
  }
  if (selection.kind === "leg" || selection.kind === "bend") {
    return loc.course === selection.course && aftersOf(loc).includes(selection.after);
  }
  if (selection.kind === "course") {
    if (loc.course === selection.id) return true;
    const course = event.courses.find((c) => c.name === selection.id);
    const visited = (course?.controls ?? []).map((c) => String(c.code));
    return codesOf(loc).some((c) => visited.includes(c));
  }
  if (selection.kind === "event") return loc.kind === "event";
  return false;
}

// --- What the operations take ----------------------------------------------------------------------------------------

// The place of a finding as the arguments of accept_finding and remove_accepted_finding.
export function placeBody(loc) {
  switch (loc.kind) {
    case "control": return { controls: [loc.code] };
    case "controls": return { controls: loc.codes };
    case "course": return { course: loc.course };
    case "leg": return { legs: [{ course: loc.course, after: loc.after }] };
    case "legs": return { legs: loc.afters.map((after) => ({ course: loc.course, after })) };
    default: return {};
  }
}

export const acceptBody = (finding, note) => ({ rule: finding.rule, ...placeBody(finding.location), note });

// An acceptance that matches nothing keeps its stored fields; one that matches has its finding's place.
export function removeBody(accepted) {
  if (!Array.isArray(accepted.controls)) return { rule: accepted.rule, ...placeBody(accepted.location) };
  const body = { rule: accepted.rule, controls: accepted.controls };
  if (accepted.legs?.length) body.legs = accepted.legs.map((l) => ({ course: l.course, after: l.after }));
  if (accepted.course) body.course = accepted.course;
  return body;
}

export function noteProblem(text) {
  const length = [...text.trim()].length;
  return length < 1 || length > MAX_NOTE_CHARS;
}

// Asks for the note; resolves to the text, or null when the dialog is cancelled.
function askNote(finding) {
  const form = document.getElementById("accept-form");
  const input = document.getElementById("accept-input");
  const error = document.getElementById("accept-error");
  document.getElementById("accept-finding").textContent = finding.text;
  error.hidden = true;
  input.removeAttribute("aria-invalid");
  input.value = "";
  return new Promise((resolve) => {
    let result = null;
    form.onsubmit = (evt) => {
      evt.preventDefault();
      const bad = noteProblem(input.value);
      input.setAttribute("aria-invalid", String(bad));
      error.hidden = !bad;
      if (bad) return;
      result = input.value.trim();
      acceptDialog.close();
    };
    document.getElementById("accept-cancel").onclick = () => acceptDialog.close();
    acceptDialog.onclose = () => resolve(result);
    acceptDialog.showModal();
    input.focus();
  });
}

async function accept(finding) {
  const note = await askNote(finding);
  if (note !== null) hooks.accept(finding, note);
}

// --- Lists -----------------------------------------------------------------------------------------------------------

function el(name, className, text = null) {
  const node = document.createElement(name);
  if (className) node.className = className;
  if (text !== null) node.textContent = text;
  return node;
}

function badge(severity) {
  const box = el("span", `sev-badge sev-${severity}`);
  const icon = el("span", "", status.SEVERITY_ICON[severity] ?? "");
  icon.setAttribute("aria-hidden", "true");
  box.append(icon, el("span", "", ` ${t(`editor-severity-${severity}`)}`));
  return box;
}

function button(label, onClick) {
  const b = el("button", "finding-act", label);
  b.type = "button";
  b.addEventListener("click", onClick);
  return b;
}

// One finding. `mode` is "open" (can be accepted), "accepted" (shows its note) or "unmatched".
function item(finding, mode) {
  const li = el("li", `finding${finding.severity ? ` sev-${finding.severity}` : ""}`);
  const main = el("button", "finding-main");
  main.type = "button";
  main.addEventListener("click", () => hooks.locate(finding));
  if (finding.severity) main.append(badge(finding.severity), document.createTextNode(" "));
  main.append(el("span", "finding-text", finding.text));
  li.append(main, el("span", "finding-rule", finding.rule));
  if (mode !== "open") {
    li.append(el("p", "finding-note", `${t("editor-findings-note")}: ${finding.note}`));
    li.append(button(t("editor-findings-remove"), () => hooks.remove(finding)));
  } else {
    li.append(button(t("editor-findings-accept"), () => accept(finding)));
  }
  return li;
}

function list(entries, mode) {
  const ul = el("ul", "finding-list");
  ul.append(...entries.map((f) => item(f, mode)));
  return ul;
}

// Rebuilds `box` with `nodes`; the focus, if it was inside, goes to the box so a keyboard user is not lost.
function replace(box, nodes) {
  const kept = box.contains(document.activeElement);
  box.replaceChildren(...nodes);
  if (kept) box.focus({ preventScroll: true });
}

function renderPanel() {
  const selection = hooks.selection();
  const event = hooks.event();
  let open = [];
  let accepted = [];
  let note = null;
  if (failed) note = "editor-findings-failed";
  else if (!report || !event) note = "editor-findings-pending";
  else if (!selection) note = "editor-findings-pick";
  else {
    open = report.findings.filter((f) => touches(f.location, selection, event));
    accepted = report.accepted.filter((f) => touches(f.location, selection, event));
    if (open.length === 0 && accepted.length === 0) note = "editor-findings-empty";
  }
  const signature = JSON.stringify([note, open, accepted]);
  if (signature === panelSignature) return;
  panelSignature = signature;
  const nodes = [];
  if (note) nodes.push(el("p", "hint", t(note)));
  if (open.length) nodes.push(list(open, "open"));
  if (accepted.length) nodes.push(el("h3", "", t("editor-findings-accepted")), list(accepted, "accepted"));
  replace(host, nodes);
}

function section(titleId, body) {
  const box = el("section", "drawer-section");
  box.append(el("h3", "", t(titleId)), body);
  return box;
}

function skippedList(skipped) {
  const ul = el("ul", "finding-list skipped");
  for (const s of skipped) {
    const li = el("li", "finding");
    li.append(el("span", "finding-text", s.text), el("span", "finding-rule", s.rule ?? ""));
    ul.append(li);
  }
  return ul;
}

function renderDrawer() {
  if (drawer.hidden) return;
  const signature = JSON.stringify([report, failed]);
  if (signature === drawerSignature) return;
  drawerSignature = signature;
  if (!report) {
    replace(drawerBody, [el("p", "hint", t(failed ? "editor-findings-failed" : "editor-findings-pending"))]);
    return;
  }
  const nodes = [];
  nodes.push(report.findings.length
    ? section("editor-drawer-findings", list(report.findings, "open"))
    : section("editor-drawer-findings", el("p", "hint", t("editor-drawer-none"))));
  if (report.accepted.length) nodes.push(section("editor-drawer-accepted", list(report.accepted, "accepted")));
  if (report.unmatched.length) nodes.push(section("editor-drawer-unmatched", list(report.unmatched, "unmatched")));
  nodes.push(report.skipped.length
    ? section("editor-drawer-skipped", skippedList(report.skipped))
    : section("editor-drawer-skipped", el("p", "hint", t("editor-drawer-all-checked"))));
  replace(drawerBody, nodes);
}

// Draws the panel (selection) and the drawer again. The page calls it whenever the selection changes.
export function render() {
  if (!hooks) return;
  renderPanel();
  renderDrawer();
}
