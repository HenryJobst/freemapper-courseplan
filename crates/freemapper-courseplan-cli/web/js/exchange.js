// Export and import from the Event menu (ADR 0053). The page never names a file: the server writes only the
// project's linked Purple Pen file, hands the other exports over as downloads and reads an uploaded file from
// the request body. Everything the server says is put in with textContent.

import { getFile, post, postBytes } from "./api.js";
import { t } from "./texts.js";
import * as status from "./status.js";

let hooks = null;

const menu = document.querySelector(".menu");
const exportDialog = document.getElementById("export-dialog");
const importDialog = document.getElementById("import-dialog");
const reportDialog = document.getElementById("report-dialog");
const importFile = document.getElementById("import-file");
const importReplace = document.getElementById("import-replace");

export const dialogOpen = () => exportDialog.open || importDialog.open || reportDialog.open;

const text = (id) => t(id);
const el = (tag, content, className) => {
  const node = document.createElement(tag);
  node.textContent = content;
  if (className) node.className = className;
  return node;
};

function show(node, visible) {
  node.hidden = !visible;
}

function message(answer) {
  const data = answer.data;
  const words = typeof data === "string" ? data : (data?.error ?? "");
  return words || text("editor-status-exchange-failed");
}

// hooks: revision() -> the page's revision, link() -> path of the linked .ppen or null, hasContent(),
// exclusive(work) -> runs work unless another change is running, written(revision) -> the project changed on
// the server: load it again.
export function bind(h) {
  hooks = h;
  document.getElementById("menu-export-xml").addEventListener("click", exportXml);
  document.getElementById("menu-export-ppen").addEventListener("click", openExport);
  document.getElementById("menu-import").addEventListener("click", openImport);
  document.getElementById("export-cancel").addEventListener("click", () => exportDialog.close());
  document.getElementById("export-linked").addEventListener("click", () => exportPpen("linked"));
  document.getElementById("export-download").addEventListener("click", () => exportPpen("download"));
  document.getElementById("import-cancel").addEventListener("click", () => importDialog.close());
  document.getElementById("import-form").addEventListener("submit", (evt) => {
    evt.preventDefault();
    importPpen();
  });
  importReplace.addEventListener("change", showImportHints);
  for (const dialog of [exportDialog, importDialog, reportDialog]) dialog.addEventListener("close", returnFocus);
}

// The dialogs open from items of the closed menu; when they close the browser would return the focus to such a
// hidden item. It goes to the menu's button instead, unless another dialog took over.
function returnFocus() {
  if (document.querySelector("dialog[open]") === null) menu.querySelector("summary")?.focus();
}

function closeMenu() {
  menu.open = false;
}

// --- Downloads ----------------------------------------------------------------------------------------------

// The server sends a safe name; the page checks it again before it gives it to the browser.
function safeName(name, fallback) {
  return typeof name === "string" && /^[A-Za-z0-9._-]{1,100}$/.test(name) && !name.startsWith(".") ? name : fallback;
}

function save(blob, name) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  link.hidden = true;
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10000);
}

async function exportXml() {
  closeMenu();
  let answer;
  try {
    answer = await getFile("api/export/iof-xml");
  } catch {
    status.say("editor-status-lost");
    return;
  }
  if (!answer.ok) {
    status.showBanner(message(answer));
    return;
  }
  const disposition = answer.headers.get("content-disposition") ?? "";
  const name = safeName(/filename="([^"]*)"/.exec(disposition)?.[1], "event.xml");
  save(answer.blob, name);
  const courses = Number(answer.headers.get("x-courseplan-courses") ?? 0);
  const skipped = Number(answer.headers.get("x-courseplan-skipped") ?? 0);
  const geo = answer.headers.get("x-courseplan-geo") !== "0";
  if (skipped === 0 && geo) {
    status.say("editor-status-iof-downloaded");
    return;
  }
  const rows = [["editor-report-courses", courses]];
  if (skipped > 0) rows.push(["editor-report-skipped", skipped]);
  const lines = [];
  if (skipped > 0) lines.push(text("editor-report-xml-skipped"));
  if (!geo) lines.push(text("editor-report-xml-no-geo"));
  report(text("editor-export-xml-title"), rows, lines, []);
  status.say("editor-status-iof-downloaded");
}

// --- Export to Purple Pen -----------------------------------------------------------------------------------

function openExport() {
  closeMenu();
  const link = hooks.link();
  document.getElementById("export-linked-file").textContent = link ? link.split(/[\\/]/).pop() : "–";
  document.getElementById("export-linked").disabled = !link;
  show(document.getElementById("export-no-link"), !link);
  show(document.getElementById("export-linked-note"), !!link);
  show(document.getElementById("export-error"), false);
  exportDialog.showModal();
}

async function exportPpen(to) {
  const failure = document.getElementById("export-error");
  show(failure, false);
  await hooks.exclusive(async () => {
    let answer;
    try {
      answer = await post("api/export/ppen", { to }, hooks.revision());
    } catch {
      failure.textContent = text("editor-status-lost");
      show(failure, true);
      return;
    }
    if (!answer.ok) {
      failure.textContent = message(answer);
      show(failure, true);
      if (answer.status === 409 && typeof answer.data === "object") await hooks.written(answer.data.revision);
      return;
    }
    const data = answer.data;
    exportDialog.close();
    if (to === "linked") {
      await hooks.written(data.revision);
      status.say("editor-status-ppen-exported");
    } else {
      save(new Blob([data.text], { type: "application/octet-stream" }), safeName(data.file, "event.ppen"));
      status.say("editor-status-ppen-downloaded");
    }
    const lines = [];
    if (to === "linked") lines.push(text(data.patched ? "editor-report-patched" : "editor-report-created"));
    if (data.linked_now) lines.push(text("editor-report-linked-now"));
    const rows = [["editor-report-courses", data.courses]];
    if (data.skipped.length > 0) rows.push(["editor-report-skipped", data.skipped.map((s) => s.name).join(", ")]);
    if (data.ids_stored > 0) rows.push(["editor-report-ids", data.ids_stored]);
    report(text("editor-report-title-ppen"), rows, lines, data.notes);
  });
}

// --- Import -------------------------------------------------------------------------------------------------

function showImportHints() {
  const replace = importReplace.checked;
  show(document.getElementById("import-replace-warning"), replace);
  show(document.getElementById("import-has-content"), !replace && hooks.hasContent());
}

function openImport() {
  closeMenu();
  importFile.value = "";
  importReplace.checked = false;
  show(document.getElementById("import-error"), false);
  showImportHints();
  importDialog.showModal();
}

async function importPpen() {
  const failure = document.getElementById("import-error");
  const file = importFile.files[0];
  if (!file) {
    failure.textContent = text("editor-import-no-file");
    show(failure, true);
    return;
  }
  show(failure, false);
  await hooks.exclusive(async () => {
    let answer;
    try {
      const bytes = await file.arrayBuffer();
      answer = await postBytes(`api/import/ppen?replace=${importReplace.checked}`, bytes, hooks.revision());
    } catch {
      failure.textContent = text("editor-status-lost");
      show(failure, true);
      return;
    }
    if (!answer.ok) {
      failure.textContent = message(answer);
      show(failure, true);
      // The answer says it already; the hint above it would only repeat.
      show(document.getElementById("import-has-content"), false);
      if (answer.status === 409 && typeof answer.data === "object") await hooks.written(answer.data.revision);
      return;
    }
    importDialog.close();
    await hooks.written(answer.data.revision);
    status.say("editor-status-imported");
    report(
      text("editor-report-title-import"),
      [["editor-report-controls", answer.data.controls], ["editor-report-courses", answer.data.courses]],
      [],
      answer.data.notes,
    );
  });
}

// --- The report -----------------------------------------------------------------------------------------------

function report(title, rows, lines, notes) {
  document.getElementById("report-title").textContent = title;
  document.getElementById("report-rows").replaceChildren(
    ...rows.flatMap(([label, value]) => [el("dt", text(label)), el("dd", String(value))]),
  );
  document.getElementById("report-lines").replaceChildren(...lines.map((line) => el("li", line)));
  document.getElementById("report-notes").replaceChildren(...notes.map((note) => el("li", note)));
  show(document.getElementById("report-notes-heading"), notes.length > 0);
  show(document.getElementById("report-notes"), notes.length > 0);
  show(document.getElementById("report-no-notes"), notes.length === 0);
  reportDialog.showModal();
}
