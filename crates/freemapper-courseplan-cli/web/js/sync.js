// Sync with the linked Purple Pen file (ADR 0054). The page names no file: the server uses the project's own
// link. A dry run lists the changes and the conflicts; each conflict gets a choice (stored through the operation
// resolve_sync_conflict), and "Sync" runs when none is open. Everything the server says goes in as textContent.

import { post } from "./api.js";
import { t } from "./texts.js";
import * as status from "./status.js";

let hooks = null;
let done = false;
let ticket = 0;

const menu = document.querySelector(".menu");
const warnDialog = document.getElementById("sync-warn-dialog");
const dialog = document.getElementById("sync-dialog");
const hideWarning = document.getElementById("sync-warn-hide");
const runButton = document.getElementById("sync-run");
const allButtons = [document.getElementById("sync-all-ours"), document.getElementById("sync-all-theirs")];
const WARNED = "courseplan-sync-warned";

export const dialogOpen = () => warnDialog.open || dialog.open;

// The dialogs open from items of the closed menu; when they close the browser would return the focus to such a
// hidden item. It goes to the menu's button instead, unless another dialog took over.
function returnFocus() {
  if (document.querySelector("dialog[open]") === null) menu.querySelector("summary")?.focus();
}

const el = (tag, content, className) => {
  const node = document.createElement(tag);
  node.textContent = content;
  if (className) node.className = className;
  return node;
};
const show = (node, visible) => {
  node.hidden = !visible;
};

// The page must work without storage (private window, blocked site data).
function warned() {
  try {
    return sessionStorage.getItem(WARNED) === "1";
  } catch {
    return false;
  }
}
function rememberWarning() {
  try {
    sessionStorage.setItem(WARNED, "1");
  } catch {
    // Nothing to do: the warning comes again next time.
  }
}

// hooks: revision(), link() -> path of the linked .ppen or null, exclusive(work), written(revision).
export function bind(h) {
  hooks = h;
  document.getElementById("menu-sync").addEventListener("click", start);
  document.getElementById("sync-warn-cancel").addEventListener("click", () => warnDialog.close());
  document.getElementById("sync-warn-continue").addEventListener("click", () => {
    if (hideWarning.checked) rememberWarning();
    warnDialog.close();
    openPanel();
  });
  document.getElementById("sync-close").addEventListener("click", () => dialog.close());
  for (const each of [warnDialog, dialog]) each.addEventListener("close", returnFocus);
  // A dry run that answers after the dialog closed is dropped.
  dialog.addEventListener("close", () => {
    ticket += 1;
  });
  runButton.addEventListener("click", () => runSync(null));
  document.getElementById("sync-all-ours").addEventListener("click", () => runSync("ours"));
  document.getElementById("sync-all-theirs").addEventListener("click", () => runSync("theirs"));
}

function start() {
  menu.open = false;
  if (warned()) {
    openPanel();
    return;
  }
  hideWarning.checked = false;
  warnDialog.showModal();
}

function setError(words) {
  const node = document.getElementById("sync-error");
  node.textContent = words ?? "";
  show(node, !!words);
}

function failureText(answer) {
  const data = answer.data;
  const words = typeof data === "string" ? data : (data?.error ?? "");
  return words || t("editor-status-sync-failed");
}

function openPanel() {
  const link = hooks.link();
  document.getElementById("sync-file").textContent = link ? link.split(/[\\/]/).pop() : "–";
  done = false;
  setError(null);
  show(document.getElementById("sync-body"), false);
  runButton.disabled = true;
  runButton.textContent = t("editor-sync-run");
  show(document.getElementById("sync-first-note"), false);
  document.getElementById("sync-status").textContent = t("editor-sync-checking");
  dialog.showModal();
  dryRun();
}

async function dryRun() {
  ticket += 1;
  const mine = ticket;
  let answer;
  try {
    answer = await post("api/sync", { dry_run: true, take: null }, hooks.revision());
  } catch {
    if (mine === ticket) failed(t("editor-status-lost"));
    return;
  }
  if (mine !== ticket) return;
  if (!answer.ok) {
    failed(failureText(answer));
    if (answer.status === 409 && typeof answer.data === "object") await hooks.written(answer.data.revision);
    return;
  }
  render(answer.data);
}

function failed(words) {
  document.getElementById("sync-status").textContent = "";
  show(document.getElementById("sync-body"), false);
  runButton.disabled = true;
  setError(words);
}

// A value of the sync report: text as it is, a number as text, anything else as compact JSON.
function shown(value) {
  if (value === null || value === undefined) return "–";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return JSON.stringify(value);
}

const SIDE_TEXT = { ours: "editor-sync-side-ours", theirs: "editor-sync-side-theirs", both: "editor-sync-side-both" };

function conflictRow(conflict) {
  const row = document.createElement("div");
  row.className = "sync-conflict";
  row.append(el("strong", conflict.key));
  const grid = document.createElement("dl");
  for (const [label, value] of [
    ["editor-sync-col-base", conflict.base],
    ["editor-sync-col-ours", conflict.ours],
    ["editor-sync-col-theirs", conflict.theirs],
  ]) {
    grid.append(el("dt", t(label)), el("dd", shown(value)));
  }
  row.append(grid);
  const buttons = document.createElement("div");
  buttons.className = "dialog-buttons";
  const keep = el("button", t("editor-sync-keep-ours"));
  keep.type = "button";
  keep.addEventListener("click", () => choose(conflict.key, "ours"));
  const take = el("button", t("editor-sync-take-theirs"));
  take.type = "button";
  take.addEventListener("click", () => choose(conflict.key, "theirs"));
  buttons.append(keep, take);
  row.append(buttons);
  return row;
}

function render(data) {
  const conflicts = data.conflicts ?? [];
  const changes = data.changes ?? [];
  document.getElementById("sync-status").textContent =
    conflicts.length === 0 && changes.length === 0 ? t("editor-sync-nothing") : "";
  document.getElementById("sync-changes").replaceChildren(
    ...changes.map((c) => el("li", `${c.key}: ${shown(c.value)} (${t(SIDE_TEXT[c.side] ?? "editor-sync-side-both")})`)),
  );
  show(document.getElementById("sync-changes-heading"), changes.length > 0);
  show(document.getElementById("sync-changes"), changes.length > 0);
  document.getElementById("sync-conflict-count").textContent = String(conflicts.length);
  document.getElementById("sync-conflicts").replaceChildren(...conflicts.map(conflictRow));
  for (const id of ["sync-conflicts-heading", "sync-conflicts", "sync-all", "sync-all-note", "sync-resolve-first"]) {
    show(document.getElementById(id), conflicts.length > 0);
  }
  document.getElementById("sync-stale").replaceChildren(
    ...(data.stale ?? []).map((key) => el("li", `${t("editor-sync-stale")} ${key}`)),
  );
  document.getElementById("sync-result").replaceChildren();
  fillNotes(data.notes ?? []);
  show(document.getElementById("sync-history-note"), changes.length > 0);
  show(document.getElementById("sync-body"), true);
  // Without a base copy (first sync, or one that was lost) the files may agree and the sync still has a job:
  // it writes the base copy. Without it the next change on either side counts as a conflict.
  const baseOnly = conflicts.length === 0 && changes.length === 0 && (data.base ?? "use") !== "use";
  show(document.getElementById("sync-first-note"), baseOnly);
  runButton.textContent = t(baseOnly ? "editor-sync-run-first" : "editor-sync-run");
  runButton.disabled = !(conflicts.length === 0 && (changes.length > 0 || baseOnly));
  for (const button of allButtons) button.disabled = false;
}

function fillNotes(notes) {
  document.getElementById("sync-notes").replaceChildren(...notes.map((n) => el("li", n)));
  show(document.getElementById("sync-notes-heading"), notes.length > 0);
  show(document.getElementById("sync-notes"), notes.length > 0);
}

async function choose(key, take) {
  setError(null);
  await hooks.exclusive(async () => {
    let answer;
    try {
      answer = await post("api/op/resolve_sync_conflict", { key, take }, hooks.revision());
    } catch {
      setError(t("editor-status-lost"));
      return;
    }
    if (!answer.ok) {
      setError(failureText(answer));
      if (answer.status === 409 && typeof answer.data === "object") await hooks.written(answer.data.revision);
      return;
    }
    await hooks.written(answer.data.revision);
    status.say("editor-status-conflict-chosen");
    render(answer.data.result);
  });
}

async function runSync(take) {
  if (done) return;
  setError(null);
  await hooks.exclusive(async () => {
    let answer;
    try {
      answer = await post("api/sync", { dry_run: false, take }, hooks.revision());
    } catch {
      setError(t("editor-status-lost"));
      return;
    }
    if (!answer.ok) {
      setError(failureText(answer));
      if (answer.status === 409 && typeof answer.data === "object") await hooks.written(answer.data.revision);
      return;
    }
    const data = answer.data;
    await hooks.written(data.revision);
    if (!data.written) {
      // Conflicts remain, or there was nothing to write: the dialog shows the state as the server saw it.
      render(data);
      if ((data.conflicts ?? []).length > 0) setError(t("editor-sync-conflicts-left"));
      return;
    }
    done = true;
    status.say("editor-status-synced");
    render({ ...data, changes: [], conflicts: [], base: "use" });
    document.getElementById("sync-status").textContent = t("editor-sync-done");
    const lines = [];
    if (data.ppen_changed) lines.push(t("editor-sync-written-ppen"));
    lines.push(t("editor-sync-written-project"));
    document.getElementById("sync-result").replaceChildren(...lines.map((line) => el("li", line)));
    show(document.getElementById("sync-history-note"), false);
    runButton.disabled = true;
    for (const button of allButtons) button.disabled = true;
  });
}
