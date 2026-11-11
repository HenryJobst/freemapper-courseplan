import { get, post, setReadOnly } from "./api.js";
import { loadTexts, t } from "./texts.js";
import { createView, bindMapLayer } from "./view.js";
import * as overlay from "./overlay.js";
import { buildTree } from "./tree.js";
import * as props from "./props.js";
import * as status from "./status.js";
import * as tools from "./tools.js";
import * as keys from "./keys.js";
import * as course from "./course.js";
import * as bends from "./bends.js";
import * as numbers from "./numbers.js";
import * as specials from "./specials.js";
import * as findings from "./findings.js";
import * as exchange from "./exchange.js";
import * as sync from "./sync.js";
import * as score from "./score.js";

const POLL_MS = 2000;
const PAN_STEP = 0.1;
const DRAG_PIXELS = 4;
// A click within this many screen pixels of the last point of a drawing repeats it and adds nothing.
const DUPLICATE_PIXELS = 3;

// What the page holds. selection = { kind: "control"|"course"|"special"|"leg"|"number"|"bend"|"class", id, course?, index? }
const state = {
  data: null,
  overlay: null,
  course: null,
  selection: null,
  treeKey: "all",
  tree: null,
  view: null,
  mapLayer: null,
  seen: null,
  closed: false,
  revision: null,
  // The print preview is on: the map shows the renderer's course layer, the overlay is hidden, nothing is edited.
  preview: false,
  // The answer of api/rules (profiles, overlays, categories), or null while unknown: the panels then take text.
  rules: null,
};
const NEAR_RADIUS_MM = 3;
let pollTimer = null;

const svg = document.getElementById("map-svg");
const propsHost = document.getElementById("props");
overlay.attach(document.getElementById("layer-overlay"));
const findingsLayer = document.getElementById("layer-findings");
const scoreLayer = document.getElementById("layer-score");

// The overlay and the finding markers are in map mm: both sit under the view's transform.
function placeLayers() {
  const transform = state.view.transform();
  document.getElementById("layer-overlay").setAttribute("transform", transform);
  findingsLayer.setAttribute("transform", transform);
  scoreLayer.setAttribute("transform", transform);
}

// The rings, signs and dashed legs of the open findings over the drawn overlay.
function drawMarkers() {
  overlay.drawFindings(findingsLayer, state.overlay, findings.current()?.findings, state.course);
}

async function closeEditor() {
  try {
    const answer = await post("api/quit");
    if (answer.ok) {
      // The server is gone on purpose: stop asking it, so the message stays.
      state.closed = true;
      clearInterval(pollTimer);
      document.getElementById("menu-close").disabled = true;
    }
    status.say(answer.ok ? "editor-status-closed" : "editor-status-lost");
  } catch {
    status.say("editor-status-lost");
  }
}

document.getElementById("menu-close").addEventListener("click", closeEditor);

function treeGroups() {
  const event = state.data.event;
  const crossing = t("editor-kind-crossing-point");
  return [
    { id: "event", label: t("editor-tree-event"), items: [{ key: "event:settings", label: t("editor-tree-event-settings") }] },
    {
      id: "courses",
      label: t("editor-tree-courses"),
      items: [
        ...event.courses.map((c) => ({ key: `course:${c.name}`, label: c.name })),
        { key: "action:new-course", label: t("editor-tree-new-course") },
        { key: "action:new-score-course", label: t("editor-tree-new-score-course") },
      ],
    },
    {
      id: "classes",
      label: t("editor-tree-classes"),
      items: [
        ...event.classes.map((c) => ({ key: `class:${c.name}`, label: c.name })),
        { key: "action:new-class", label: t("editor-tree-new-class") },
      ],
    },
    {
      id: "controls",
      label: t("editor-tree-controls"),
      items: [
        { key: "all", label: t("editor-tree-all-controls") },
        ...event.controls.map((c) => ({
          key: `control:${c.code}`,
          label: c.kind === "crossing_point" ? `${c.code} (${crossing})` : String(c.code),
        })),
      ],
    },
    {
      id: "specials",
      label: t("editor-tree-specials"),
      items: event.specials.map((s) => ({ key: `special:${s.id}`, label: `${s.id} (${overlay.kindLabel(s.kind)})` })),
    },
  ];
}

function renderTree() {
  const root = document.getElementById("pane-tree");
  root.dataset.treeLabel = t("editor-tree-label");
  state.tree = buildTree(root, treeGroups(), selectKey);
  state.tree.setCurrent(state.treeKey);
}

function redraw() {
  overlay.draw(state.overlay, state.selection, state.course, extras());
  drawMarkers();
  score.drawMarkers(scoreLayer, state.overlay, state.course);
  findings.render();
  props.show(propsHost, state.data, state.selection, state.overlay, propActions);
  state.tree?.setCurrent(state.treeKey);
  loadNear();
}

// The line or area being drawn with the special tool, or null: { kind, shape, points (map mm), cursor }.
let draft = null;

// Course mode: the controls the course does not visit are drawn faint and can be clicked; a chosen leg is thick.
let pendingLeg = null;

function extras() {
  return { pool: tools.getTool() === "course" && !isScore() ? state.pool : [], pending: pendingLeg, draft };
}

// The selected course is a score course: no legs, no stop list, no course mode.
function isScore() {
  return state.data?.event.courses.find((c) => c.name === state.course)?.kind === "score";
}

function controlOf(code) {
  return state.data?.event.controls.find((c) => String(c.code) === String(code));
}

// The map objects at the selected control, listed in the properties (api/map/near, 3 mm).
async function loadNear() {
  const selected = state.selection;
  const control = selected?.kind === "control" ? controlOf(selected.id) : null;
  if (!control) return;
  try {
    const [x, y] = control.map_mm;
    const answer = await get(`api/map/near?x=${x}&y=${y}&r=${NEAR_RADIUS_MM}`);
    if (state.selection === selected) props.showNear(propsHost, answer.ok ? answer.data : null);
  } catch {
    // The list stays empty.
  }
}

// --- Operations ---------------------------------------------------------------------------------------------
// Each change is one POST api/op/{tool} with the revision the page last saw. The answer's revision becomes the
// page's own at once; the state and the overlay are loaded again afterwards, whatever the answer was, so the
// page always shows the file. A refusal or a conflict shows in the status bar and in the banner above the map.

let busy = false;
// The text of the last refusal or conflict, for a caller that has to undo a step and show it again.
let lastFailure = "";
// An operation that arrived while another ran is dropped; the page is loaded again when the running one ends,
// so a typed value or a moved circle does not stay although nothing was sent.
let refreshAfter = false;

function dropped() {
  refreshAfter = true;
  status.say("editor-status-busy");
}

async function idle() {
  busy = false;
  if (refreshAfter && !state.closed) {
    refreshAfter = false;
    try {
      await loadState(false);
    } catch {
      status.say("editor-status-lost");
    }
  }
}

function failure(answer) {
  const data = answer.data;
  return typeof data === "string" ? data : (data?.error ?? "");
}

// Runs one operation; the answer's data on success, or null (the banner says why).
async function runOp(tool, body) {
  if (state.closed) return null;
  if (busy) {
    dropped();
    return null;
  }
  busy = true;
  try {
    return await settle(await post(`api/op/${tool}`, body, state.revision));
  } catch {
    status.say("editor-status-lost");
    return null;
  } finally {
    await idle();
  }
}

// Runs a long score call (suggest, draft, length; api/score/{tool}): the page is marked busy, the Score buttons are
// disabled and the panel says it is working, while the server computes without holding its lock. The answer's data
// ({ revision, result }) on success, or null (the banner says why). A draft with `apply` writes, so it settles like
// an operation.
async function runScore(tool, body) {
  if (state.closed || state.preview) return null;
  if (busy) {
    dropped();
    return null;
  }
  busy = true;
  score.setWorking(true);
  propsHost.parentElement.setAttribute("aria-busy", "true");
  redraw();
  status.say("editor-score-working");
  try {
    const answer = await post(`api/score/${tool}`, body, state.revision);
    if (!answer.ok || body.apply) return await settle(answer);
    status.clearBanner();
    return answer.data;
  } catch {
    status.say("editor-status-lost");
    return null;
  } finally {
    score.setWorking(false);
    propsHost.parentElement.removeAttribute("aria-busy");
    await idle();
    redraw();
  }
}

score.bind({
  runOp,
  runScore,
  redraw: () => redraw(),
  say: (id) => status.say(id),
  state: () => state,
  focusAfter: props.focusAfter,
  lastFailure: () => lastFailure,
  showBanner: status.showBanner,
});

async function settle(answer) {
  if (answer.ok) {
    state.revision = answer.data.revision;
    // Our own write is not an outside change: the poll must not announce it.
    if (state.seen) state.seen = { ...state.seen, project: answer.data.revision };
    // Candidates and a draft survive a write of this page; one from outside discards them (loadState).
    score.rebase(answer.data.revision);
    status.clearBanner();
    await loadState(false);
    return answer.data;
  }
  const message = failure(answer);
  lastFailure = message;
  if (message) {
    // The banner is an alert region: a screen reader reads it, so the status bar does not repeat it.
    status.showBanner(message);
  }
  if (answer.status === 409 && typeof answer.data === "object") state.revision = answer.data.revision;
  await loadState(false);
  return null;
}

async function history(redo) {
  if (busy || state.closed || state.preview) return;
  const label = redo ? state.data?.undo.redo : state.data?.undo.undo;
  busy = true;
  try {
    const done = await settle(await post(redo ? "api/redo" : "api/undo", {}, state.revision));
    if (done) {
      status.say(`${t(redo ? "editor-status-redone" : "editor-status-undone")}${label ? `: ${label}` : ""}`, true);
    }
  } catch {
    status.say("editor-status-lost");
  } finally {
    await idle();
  }
}

const mmValue = (v) => Math.round(v * 100) / 100;

async function moveControl(code, x, y) {
  const done = await runOp("move_control", { code, at: { x, y } });
  if (done) status.say("editor-status-moved");
}

async function changeCode(from, to) {
  const done = await runOp("set_control_code", { from, to });
  if (!done) return;
  status.say("editor-status-code-changed");
  await selectKey(`control:${to}`);
}

async function renameCourse(from, to) {
  const done = await runOp("rename_course", { from, to });
  if (!done) return;
  status.say("editor-status-course-renamed");
  await selectKey(`course:${to}`);
}

async function placeControl(at, crossing) {
  const body = { at: { x: mmValue(at[0]), y: mmValue(at[1]) } };
  if (crossing) body.crossing_point = true;
  const done = await runOp("add_control", body);
  if (!done) return;
  status.say(crossing ? "editor-status-placed-crossing" : "editor-status-placed");
  await selectKey(`control:${done.result.code}`);
}

const confirmDialog = document.getElementById("confirm-dialog");
const helpDialog = document.getElementById("help-dialog");
const nameDialog = document.getElementById("name-dialog");

function coursesUsing(code) {
  return state.data.event.courses
    .filter((c) => (c.controls ?? []).some((x) => String(x.code) === String(code)))
    .map((c) => c.name);
}

async function deleteControl(code) {
  const done = await runOp("remove_control", { code });
  if (!done) return;
  const left = done.result.left_courses ?? [];
  status.say(left.length ? `${t("editor-status-deleted")}: ${left.join(", ")}` : t("editor-status-deleted"), true);
}

// Puts the courses that visit `code` in the confirm dialog; false when none does.
function fillConfirm(code) {
  const users = controlOf(code) ? coursesUsing(code) : [];
  if (users.length === 0) return false;
  const list = document.getElementById("confirm-courses");
  list.replaceChildren(...users.map((name) => {
    const li = document.createElement("li");
    li.textContent = name;
    return li;
  }));
  return true;
}

// The state changed under an open dialog: its course list follows the file, and it closes when the control is
// gone or no course visits it any more (the plain delete would not have asked).
function refreshConfirm() {
  if (confirmDialog.open && !fillConfirm(confirmDialog.dataset.code)) confirmDialog.close();
}

// Deleting asks first when a course visits the control.
function requestDelete() {
  if (state.preview) return;
  const selected = state.selection;
  if (draft) {
    // While a line or an area is drawn, Delete takes back its last point.
    draft.points.pop();
    if (draft.points.length === 0) draft = null;
    showOverlay();
    draftHint();
    return;
  }
  if (selected?.kind === "special" && !busy) {
    removeSpecial(selected.id);
    return;
  }
  if (selected?.kind === "bend") {
    removeBend();
    return;
  }
  if (selected?.kind !== "control" || !controlOf(selected.id) || busy) return;
  if (!fillConfirm(selected.id)) {
    deleteControl(selected.id);
    return;
  }
  confirmDialog.dataset.code = String(selected.id);
  confirmDialog.showModal();
}

document.getElementById("confirm-no").addEventListener("click", () => confirmDialog.close());
document.getElementById("confirm-yes").addEventListener("click", () => {
  confirmDialog.close();
  deleteControl(confirmDialog.dataset.code);
});

// --- Special objects -------------------------------------------------------------------------------------------
// Point kinds are placed by one click; lines and areas by clicks, finished with Enter or a double click. Each
// finished object is one add_special, so one undo step. Nothing moves a special object: delete and draw again.

const kindSelect = document.getElementById("special-kind");
const idInput = document.getElementById("special-id");

function takenIds() {
  return state.data?.event.specials.map((s) => s.id) ?? [];
}

function proposeId() {
  idInput.value = specials.nextId(kindSelect.value, takenIds());
}

function fillKinds() {
  kindSelect.replaceChildren(...specials.KINDS.map((k) => {
    const option = document.createElement("option");
    option.value = k.kind;
    option.textContent = overlay.kindLabel(k.kind);
    return option;
  }));
}

function draftHint() {
  if (!draft) {
    status.say("editor-status-special-tool");
    return;
  }
  status.say(`${draft.points.length} ${t("editor-status-draft-points")}. ${t("editor-status-draft-hint")}`, true);
}

function dropDraft() {
  if (!draft) return false;
  draft = null;
  showOverlay();
  status.say("editor-status-draft-cancelled");
  return true;
}

// Sends the object: the id in the toolbar is the one used; a refused id keeps the drawing.
async function addSpecial(kind, points) {
  const id = idInput.value;
  if (course.nameProblem(id, course.MAX_NAME_CHARS, takenIds())) {
    status.say("editor-error-special-id");
    idInput.focus();
    return;
  }
  const done = await runOp("add_special", specials.addBody(kind, id, points));
  if (!done) return;
  draft = null;
  status.say("editor-status-special-added");
  proposeId();
  await selectKey(`special:${done.result.id}`);
}

function specialClick(at) {
  const entry = specials.info(kindSelect.value);
  if (entry.shape === "point") {
    addSpecial(entry.kind, [at]);
    return;
  }
  draft ??= { kind: entry.kind, shape: entry.shape, points: [], cursor: null };
  // The second click of a double click lands on the point the first one set: it adds nothing.
  const last = draft.points.at(-1);
  if (last && Math.hypot(at[0] - last[0], at[1] - last[1]) < Math.max(DUPLICATE_PIXELS / state.view.scale, 0.01)) return;
  draft.points.push(at);
  showOverlay();
  draftHint();
}

function finishDraft() {
  if (!draft) return;
  if (!specials.enough(draft.kind, draft.points)) {
    status.say("editor-status-draft-short");
    return;
  }
  addSpecial(draft.kind, draft.points);
}

async function removeSpecial(id) {
  const done = await runOp("remove_special", { id });
  if (!done) return;
  status.say("editor-status-special-deleted");
}

async function setSpecialCourses(id, names) {
  const done = await runOp("set_special_courses", specials.coursesBody(id, names));
  if (done) status.say("editor-status-special-courses");
}

kindSelect.addEventListener("change", () => {
  dropDraft();
  proposeId();
});

// --- Courses and categories ------------------------------------------------------------------------------------

async function newCourse() {
  const specials = state.data.event.specials;
  const starts = specials.filter((s) => s.kind === "start");
  const finishes = specials.filter((s) => s.kind === "finish");
  if (starts.length !== 1 || finishes.length !== 1) {
    status.say("editor-status-need-start-finish");
    return;
  }
  const name = await course.askName({
    titleId: "editor-new-course-title",
    labelId: "editor-new-course-label",
    errorId: "editor-error-course-name",
    others: state.data.event.courses.map((c) => c.name),
  });
  if (name === null) return;
  const done = await runOp("add_course", { name, start: starts[0].id, finish: finishes[0].id });
  if (!done) return;
  status.say("editor-status-course-created");
  await selectKey(`course:${name}`);
}

// A score course gets the event's only start and finish and a time limit of 45 minutes, which the Score panel changes.
async function newScoreCourse() {
  const specials = state.data.event.specials;
  const starts = specials.filter((s) => s.kind === "start");
  const finishes = specials.filter((s) => s.kind === "finish");
  if (starts.length !== 1 || finishes.length !== 1) {
    status.say("editor-status-need-start-finish");
    return;
  }
  const name = await course.askName({
    titleId: "editor-new-course-title",
    labelId: "editor-new-course-label",
    errorId: "editor-error-course-name",
    others: state.data.event.courses.map((c) => c.name),
  });
  if (name === null) return;
  const done = await runOp("add_score_course", { name, start: starts[0].id, finish: finishes[0].id, time_limit_min: 45 });
  if (!done) return;
  status.say("editor-status-score-created");
  await selectKey(`course:${name}`);
}

async function newClass() {
  const name = await course.askName({
    titleId: "editor-new-class-title",
    labelId: "editor-new-class-label",
    errorId: "editor-error-class-name",
    others: state.data.event.classes.map((c) => c.name),
  });
  if (name === null) return;
  const done = await runOp("add_class", { name });
  if (!done) return;
  status.say("editor-status-class-added");
  await selectKey(`class:${name}`);
}

// One set_course_controls with the whole new list; `focusId` is the element to keep the focus on.
async function setStops(name, codes, focusId) {
  if (focusId) props.focusAfter(focusId);
  const done = await runOp("set_course_controls", { name, codes });
  if (!done) {
    props.focusAfter(null);
    return;
  }
  status.say(course.changedText(done.result.cleared_legs), true);
}

async function runAndSay(tool, body, messageId, after = null) {
  const done = await runOp(tool, body);
  if (!done) return;
  status.say(messageId);
  if (after) await after();
}

const setDetails = (name, body) => runAndSay("set_course_details", { name, ...body }, "editor-status-details-changed");
const removeCourse = (name) => runAndSay("remove_course", { name }, "editor-status-course-deleted", () => selectKey("all"));
const setClass = (name, body) => runAndSay("set_class", { name, ...body }, "editor-status-class-changed");
const removeClass = (name) => runAndSay("remove_class", { name }, "editor-status-class-deleted", () => selectKey("all"));
const setProfile = (base, overlays) => runAndSay("set_rule_profile", { base, overlays }, "editor-status-profile-changed");
const setWindow = (minutes) =>
  runAndSay("set_start_window", minutes === null ? { no_start_window: true } : { minutes }, "editor-status-window-changed");

// One set_leg per gesture: the leg's whole bend list and its taped flag.
async function setLeg(after, list, taped, messageId) {
  const done = await runOp("set_leg", bends.legBody(state.course, after, list, taped));
  if (done) status.say(messageId);
  return done;
}

const legOf = (after) => state.overlay?.legs.find((l) => l.after === after);

function setTaped(after, taped) {
  const leg = legOf(after);
  if (leg) setLeg(after, leg.bends, taped, "editor-status-taped-changed");
}

function removeBend() {
  const selected = state.selection;
  const leg = selected?.kind === "bend" ? legOf(selected.after) : null;
  if (!leg || busy) return;
  state.selection = { kind: "leg", id: String(leg.after), after: leg.after, course: state.course };
  setLeg(leg.after, bends.withoutBend(leg.bends, selected.index), leg.taped, "editor-status-bend-removed");
}

// Drops a dragged number box: the server turns the centre into direction and gap, then one operation stores it.
async function dropNumber(n, centre) {
  if (busy || state.closed) return;
  busy = true;
  let found = null;
  try {
    const answer = await get(numbers.numberAtQuery(state.course, n, centre));
    if (answer.ok) found = answer.data;
    else status.showBanner(failure(answer));
  } catch {
    status.say("editor-status-lost");
  } finally {
    busy = false;
  }
  if (!found) {
    await loadState(false);
    return;
  }
  const { tool, body } = numbers.dropOperation(state.course, n, found, numbers.options.forAll);
  const done = await runOp(tool, body);
  if (!done) return;
  // A number set for this course comes before the one on the control, so the box does not move.
  const shadowed = numbers.options.forAll && n.source === "course";
  const id = shadowed ? "editor-status-number-shadowed" : "editor-status-number-moved";
  // A box dropped over the circle or very far away is stored with the nearest gap that is allowed.
  status.say(found.clamped ? `${t(id)} ${t("editor-status-number-clamped")}` : id, !!found.clamped);
}

async function placeNumbers() {
  if (!state.course) return;
  const done = await runOp("place_numbers", numbers.placeBody(state.course, numbers.options.overwrite));
  if (done) status.say(`${t("editor-status-numbers-placed")} ${done.result.placed}`, true);
}

// One pick in a description picker: one describe_control operation, the focus stays on the picked symbol.
async function describeControl(body, focusId) {
  const done = await runOp("describe_control", body);
  if (!done) return;
  props.focusAfter(focusId);
  status.say("editor-status-described");
  redraw();
}

// --- Findings ----------------------------------------------------------------------------------------------------------
// Accepting and withdrawing are operations like any other: one accept_finding or remove_accepted_finding, one
// undo step. Clicking a finding selects its place and fits the map to it.

async function acceptFinding(finding, note) {
  const done = await runOp("accept_finding", findings.acceptBody(finding, note));
  if (done) status.say("editor-status-accepted");
}

async function removeAcceptance(accepted) {
  const done = await runOp("remove_accepted_finding", findings.removeBody(accepted));
  if (done) status.say("editor-status-acceptance-removed");
}

// Fits the map to map-mm points, with room around them (at least 20 mm on a side).
function fitPoints(points) {
  if (!state.view || points.length === 0) return;
  // A loop, not Math.max(...list): a very long list would overflow the argument limit.
  let x0 = Infinity;
  let y0 = Infinity;
  let x1 = -Infinity;
  let y1 = -Infinity;
  for (const [x, y] of points) {
    x0 = Math.min(x0, x);
    y0 = Math.min(y0, y);
    x1 = Math.max(x1, x);
    y1 = Math.max(y1, y);
  }
  const room = Math.max(20, 0.25 * Math.max(x1 - x0, y1 - y0));
  const a = state.view.toPage([x0 - room, y0 - room]);
  const b = state.view.toPage([x1 + room, y1 + room]);
  state.view.fit([Math.min(a[0], b[0]), Math.min(a[1], b[1]), Math.max(a[0], b[0]), Math.max(a[1], b[1])]);
}

function pointsOfCodes(codes) {
  const o = state.overlay;
  const known = [...(o?.controls ?? []), ...(o?.starts ?? []), ...(o?.finishes ?? [])];
  return codes.map((code) => known.find((c) => String(c.code ?? c.id) === String(code))?.at).filter(Boolean);
}

async function locateFinding(finding) {
  const loc = finding.location;
  if (loc.kind === "event") {
    await selectKey("event:settings");
  } else if (loc.kind === "course") {
    await selectKey(`course:${loc.course}`);
    const o = state.overlay;
    fitPoints([...(o?.controls ?? []), ...(o?.starts ?? []), ...(o?.finishes ?? [])].map((c) => c.at));
  } else if (loc.kind === "leg" || loc.kind === "legs") {
    if (state.course !== loc.course) await selectKey(`course:${loc.course}`);
    const afters = loc.kind === "leg" ? [loc.after] : loc.afters;
    const legs = afters.map(legOf).filter(Boolean);
    if (legs.length === 0) return;
    state.selection = legSelection(legs[0].after);
    state.treeKey = `course:${state.course}`;
    redraw();
    fitPoints(legs.flatMap((l) => l.points));
  } else {
    const codes = loc.kind === "control" ? [loc.code] : loc.codes;
    const first = codes.find((c) => controlOf(c));
    if (first === undefined) return;
    await selectKey(`control:${first}`);
    fitPoints(pointsOfCodes(codes));
  }
  status.say("editor-status-located");
}

findings.bind({
  selection: () => state.selection,
  event: () => state.data?.event ?? null,
  locate: locateFinding,
  accept: acceptFinding,
  remove: removeAcceptance,
  changed: () => {
    drawMarkers();
  },
});

// Export and import (ADR 0053). They run like an operation: one at a time, and the page loads the project
// again afterwards. `written` is called with the revision the server answered, also after a refusal.
let writes = 0;
async function exclusive(work) {
  if (state.closed) return;
  if (busy) {
    dropped();
    return;
  }
  busy = true;
  writes += 1;
  try {
    await work();
  } catch {
    status.say("editor-status-lost");
  } finally {
    await idle();
  }
}

async function exchangeWritten(revision) {
  if (revision) state.revision = revision;
  await loadState(false);
  // Our own write, also to the linked file, is not an outside change.
  const seen = await get("api/revision");
  if (seen.ok) state.seen = seen.data;
}

exchange.bind({
  revision: () => state.revision,
  link: () => state.data?.link?.ppen ?? null,
  hasContent: () => {
    const event = state.data?.event;
    return !!event && (event.controls.length > 0 || event.courses.length > 0 || (event.specials ?? []).length > 0);
  },
  exclusive,
  written: exchangeWritten,
});

// Sync with Purple Pen (ADR 0054). It uses the same hooks: a real sync and a conflict choice run one at a time.
sync.bind({
  revision: () => state.revision,
  link: () => state.data?.link?.ppen ?? null,
  exclusive,
  written: exchangeWritten,
});

const propActions = {
  describe: describeControl,
  rules: () => state.rules,
  placeNumbers,
  move: moveControl,
  remove: requestDelete,
  changeCode,
  renameCourse,
  setStops,
  setDetails,
  removeCourse,
  setClass,
  removeClass,
  setProfile,
  setWindow,
  setTaped,
  setSpecialCourses,
};

function updateHistoryButtons() {
  const u = state.data?.undo;
  for (const [id, label] of [["undo-btn", u?.undo], ["redo-btn", u?.redo]]) {
    const button = document.getElementById(id);
    button.disabled = !label;
    if (label) button.title = label;
    else button.removeAttribute("title");
  }
}

document.getElementById("undo-btn").addEventListener("click", () => history(false));
document.getElementById("redo-btn").addEventListener("click", () => history(true));
document.getElementById("help-btn").addEventListener("click", () => helpDialog.showModal());

// The course whose renderer layer the map image shows now, or null for the plain map.
let previewShown = null;

// Brings the print preview in line with the state: only a selected course can be previewed. `force` loads the map
// image again even when the course is the same (the project changed).
function syncPreview(force = false) {
  // A score course has no print preview yet: the renderer draws individual courses only.
  if (state.preview && (!state.course || isScore())) state.preview = false;
  const button = document.getElementById("preview-btn");
  button.disabled = !state.course || isScore();
  button.setAttribute("aria-pressed", String(state.preview));
  svg.dataset.preview = String(state.preview);
  setReadOnly(state.preview);
  // The preview shows the print, it edits nothing: the panes, the tools, undo and redo and the writing menu items
  // are dead while it is on. The preview button, zoom and help stay.
  for (const id of ["pane-tree", "pane-props", "tool-select", "tool-control", "tool-crossing", "tool-special", "tool-course", "undo-btn", "redo-btn", "special-options", "menu-import", "menu-sync"]) {
    document.getElementById(id).inert = state.preview;
  }
  for (const layer of [document.getElementById("layer-overlay"), findingsLayer, scoreLayer]) {
    layer.style.display = state.preview ? "none" : "";
  }
  const want = state.preview ? state.course : null;
  if (want !== previewShown || (force && want)) {
    previewShown = want;
    state.mapLayer?.setCourse(want);
    state.mapLayer?.refresh(true);
  }
}

function togglePreview() {
  if (!state.preview && (!state.course || isScore())) return;
  state.preview = !state.preview;
  if (state.preview) {
    pendingLeg = null;
    draft = null;
    tools.setTool("select");
  }
  syncPreview();
  redraw();
  status.say(state.preview ? "editor-status-preview-on" : "editor-status-preview-off");
}

document.getElementById("preview-btn").addEventListener("click", togglePreview);

async function loadOverlay(course) {
  const query = course ? `?course=${encodeURIComponent(course)}` : "";
  const answer = await get(`api/overlay${query}`);
  if (!answer.ok) {
    state.overlay = null;
    // 422: a score, team or relay course. Nothing is drawn; the status says why.
    if (answer.status === 422) status.say("editor-status-course-unsupported");
    else status.say(answer.data, true);
    return false;
  }
  state.course = course;
  score.forget(course);
  state.overlay = answer.data;
  state.pool = [];
  if (course) {
    // The controls the course does not visit, for course mode.
    const all = await get("api/overlay");
    if (all.ok) state.pool = all.data.controls.filter((c) => !answer.data.controls.some((x) => x.code === c.code));
  }
  return true;
}

function overlayHas(kind, id) {
  const o = state.overlay;
  if (!o) return false;
  if (kind === "control") return o.controls.some((c) => c.code === String(id));
  return o.specials.some((s) => s.id === id) || o.starts.some((s) => s.id === id) || o.finishes.some((f) => f.id === id);
}

async function selectKey(key) {
  const [kind, ...rest] = key.split(":");
  const id = rest.join(":");
  if (kind === "action") {
    // A list entry that starts a dialog; the selection stays as it is.
    pendingLeg = null;
    state.tree?.setCurrent(state.treeKey);
    await (id === "new-course" ? newCourse() : id === "new-score-course" ? newScoreCourse() : newClass());
    return;
  }
  pendingLeg = null;
  state.treeKey = key;
  if (key === "all") {
    state.selection = null;
    await loadOverlay(null);
  } else if (kind === "course") {
    state.selection = { kind: "course", id };
    await loadOverlay(id);
  } else if (kind === "class" || kind === "event") {
    state.selection = { kind, id };
  } else {
    state.selection = { kind, id };
    if (!overlayHas(kind, id)) await loadOverlay(null);
  }
  syncPreview();
  redraw();
}

// The control whose circle a map point (map mm) hits: within the circle's radius plus the line width.
function hitControl(at, extra = []) {
  const o = state.overlay;
  if (!o) return null;
  const reach = o.sizes.circle_r + o.sizes.line;
  let best = null;
  for (const c of [...o.controls, ...extra]) {
    const d = Math.hypot(c.at[0] - at[0], c.at[1] - at[1]);
    if (d <= reach && (!best || d < best.d)) best = { code: c.code, d };
  }
  return best?.code ?? null;
}

function selectControl(code) {
  state.selection = { kind: "control", id: code };
  state.treeKey = `control:${code}`;
  redraw();
}

const BEND_HIT_PX = 7;

// The leg selected, or null. A bend counts as its leg.
function selectedLeg() {
  const sel = state.selection;
  return sel?.kind === "leg" || sel?.kind === "bend" ? legOf(sel.after) : null;
}

// What a press at a map point hits on the selected leg: { bend } (index), { line } (segment index) or null.
function legTarget(at) {
  const leg = selectedLeg();
  if (!leg || !state.view) return null;
  const bend = bends.bendNear(leg.bends, at, BEND_HIT_PX / state.view.scale);
  if (bend !== null) return { leg, bend };
  const near = bends.nearestSegment(leg.points, at);
  return near && near.d <= LEG_HIT_PX / state.view.scale ? { leg, line: near.index } : null;
}

const legSelection = (after) => ({ kind: "leg", id: String(after), after, course: state.course });
const bendSelection = (after, index) => ({ kind: "bend", id: `${after}:${index}`, after, index, course: state.course });

// A click on a bend of the selected leg, or on any leg of the drawn course, selects it.
function selectLegAt(at) {
  const target = legTarget(at);
  if (target?.bend != null) {
    state.selection = bendSelection(target.leg.after, target.bend);
  } else {
    const after = hitLeg(at);
    if (after === null) return false;
    state.selection = legSelection(after);
    status.say(`${t("editor-leg-word")} ${course.legLabel(after)}. ${t("editor-status-leg-selected")}`, true);
  }
  redraw();
  return true;
}

// A click on a symbol selects it; the tree follows when the symbol is a control or a special object.
function selectDrawn(target, at) {
  const node = target.closest("[data-kind]");
  if (node?.dataset.kind === "number") {
    state.selection = { kind: "number", id: node.dataset.id, course: state.course, index: Number(node.dataset.id) };
    redraw();
    return;
  }
  const hit = hitControl(at);
  if (hit) {
    selectControl(hit);
  } else if ((!node || node.dataset.kind === "leg" || node.dataset.kind === "bend") && selectLegAt(at)) {
    // A bend or a leg of the course was hit.
  } else if (!node || node.dataset.kind === "leg" || node.dataset.kind === "control") {
    state.selection = null;
    state.treeKey = state.course ? `course:${state.course}` : "all";
    redraw();
  } else {
    state.selection = { kind: node.dataset.kind, id: node.dataset.id };
    state.treeKey = `${node.dataset.kind}:${node.dataset.id}`;
    redraw();
  }
}

async function loadState(announce) {
  const answer = await get("api/state");
  if (answer.ok && answer.data?.revision) state.revision = answer.data.revision;
  if (!answer.ok || !answer.data || answer.data.error) {
    status.say(answer.ok && answer.data?.error ? answer.data.error : "editor-status-state-failed", answer.ok && !!answer.data?.error);
    return false;
  }
  // The server leaves empty lists out of the event; the page counts on them.
  for (const list of ["classes", "controls", "specials", "courses", "accepted_findings"]) answer.data.event[list] ??= [];
  state.data = answer.data;
  if (score.checkRevision(state.revision)) status.say("editor-score-discarded");
  const m = answer.data.map;
  status.setMap(m.path, m.scale, m.kind);
  renderTree();
  const known = (name) => state.data.event.courses.some((c) => c.name === name);
  if (state.course && !known(state.course)) {
    state.course = null;
    state.selection = null;
    state.treeKey = "all";
  }
  if (state.selection?.kind === "special" && !state.data.event.specials.some((s) => s.id === state.selection.id)) {
    state.selection = null;
    state.treeKey = state.course ? `course:${state.course}` : "all";
  }
  if (!idInput.value || takenIds().includes(idInput.value)) proposeId();
  if (state.selection?.kind === "control" && !controlOf(state.selection.id)) {
    state.selection = null;
    state.treeKey = state.course ? `course:${state.course}` : "all";
  }
  updateHistoryButtons();
  refreshConfirm();
  await loadOverlay(state.course);
  syncPreview(true);
  keepLegSelection();
  redraw();
  findings.schedule();
  if (announce) status.say("editor-status-reloaded");
  return true;
}

// A selected leg or bend stays only while the overlay still has it; a bend that is gone falls back to its leg.
function keepLegSelection() {
  const sel = state.selection;
  if (sel?.kind !== "leg" && sel?.kind !== "bend") return;
  const leg = sel.course === state.course ? legOf(sel.after) : null;
  if (!leg) state.selection = null;
  else if (sel.kind === "bend" && sel.index >= leg.bends.length) {
    state.selection = { kind: "leg", id: String(sel.after), after: sel.after, course: sel.course };
  }
}

// The standing note about the map (cut off, incomplete), shown again when a refused section recovers.
let mapNote = null;

async function loadMapInfo(fit) {
  const answer = await get("api/map/view");
  if (!answer.ok) return;
  const info = answer.data;
  if (!state.view) {
    state.view = createView(svg, info.mapping);
    let mapRefused = false;
    state.mapLayer = bindMapLayer(document.getElementById("layer-map"), state.view, (text) => {
      // The overlay stays; the status bar says why the map area is empty.
      if (text) status.say(text, true);
      else if (mapRefused) mapNote ? status.say(mapNote) : status.say("", true);
      mapRefused = Boolean(text);
    });
    state.view.addEventListener("viewchange", () => {
      placeLayers();
    });
    bindMapInput();
    fit = true;
  }
  state.view.setMapping(info.mapping);
  placeLayers();
  state.pageBox = info.page_box;
  if (fit) state.view.fit(info.page_box);
  // A persistent note: the map area is cut off or lacks objects, which nothing else on the page shows.
  mapNote = info.clipped ? "editor-map-clipped" : info.incomplete ? "editor-map-incomplete" : null;
  if (mapNote) status.say(mapNote);
}

// A control being dragged, or null: { code, start (map mm under the pointer), orig (the circle's centre) }.
let drag = null;
// Arrow keys held: { code, orig, dx, dy } in map mm, committed as one move when the key is released.
let nudge = null;

function setCircle(code, at) {
  const c = state.overlay?.controls.find((x) => x.code === String(code));
  if (c) c.at = at;
}

function showOverlay() {
  overlay.draw(state.overlay, state.selection, state.course, extras());
  drawMarkers();
}

// A bend being dragged, or null: { after, index (null until the line gave one), at (the segment to split),
// orig (the bends before), start (map mm under the pointer), moved }.
let legDrag = null;

// A number box being dragged in course mode, or null: { number (the overlay entry), start, orig (its centre),
// moved }.
let numberDrag = null;

function cancelDrag() {
  if (numberDrag?.moved) {
    numberDrag.number.centre = numberDrag.orig;
    showOverlay();
  }
  numberDrag = null;
  if (drag?.moved) setCircle(drag.code, drag.orig);
  if (drag?.moved) showOverlay();
  drag = null;
  if (legDrag?.moved) {
    const leg = legOf(legDrag.after);
    if (leg) bends.setLegBends(leg, legDrag.orig);
    state.selection = legSelection(legDrag.after);
    redraw();
  }
  legDrag = null;
}

// The pointer moved with a bend in hand: insert it on the first move over a line, then follow the pointer.
function dragBend(at) {
  const leg = legOf(legDrag.after);
  if (!leg) return;
  if (!legDrag.moved) {
    legDrag.moved = true;
    if (legDrag.index === null) {
      legDrag.index = legDrag.at;
      legDrag.inserted = true;
      bends.setLegBends(leg, bends.withInserted(leg.bends, legDrag.at, { x: at[0], y: at[1] }));
    }
    state.selection = bendSelection(legDrag.after, legDrag.index);
  }
  const origin = legDrag.inserted ? at : [legDrag.orig[legDrag.index].x + at[0] - legDrag.start[0], legDrag.orig[legDrag.index].y + at[1] - legDrag.start[1]];
  bends.setLegBends(leg, bends.withMoved(leg.bends, legDrag.index, { x: origin[0], y: origin[1] }));
  showOverlay();
}

// The drop: one set_leg, or nothing when the bend is back where it was.
function dropBend() {
  const done = legDrag;
  legDrag = null;
  const leg = legOf(done.after);
  if (!leg) return;
  const rounded = leg.bends.map((b) => ({ x: mmValue(b.x), y: mmValue(b.y) }));
  if (!done.inserted && bends.sameBends(rounded, done.orig.map((b) => ({ x: mmValue(b.x), y: mmValue(b.y) })))) {
    bends.setLegBends(leg, done.orig);
    redraw();
    return;
  }
  setLeg(done.after, rounded, leg.taped, done.inserted ? "editor-status-bend-added" : "editor-status-bend-moved");
}

function startNudge(dx, dy) {
  const selected = state.selection;
  if (!selected || busy || state.preview) return;
  const c = state.overlay?.controls.find((x) => x.code === String(selected.id));
  if (!c) return;
  nudge ??= { code: c.code, orig: [...c.at], dx: 0, dy: 0 };
  nudge.dx += dx;
  nudge.dy += dy;
  setCircle(nudge.code, [nudge.orig[0] + nudge.dx, nudge.orig[1] + nudge.dy]);
  showOverlay();
}

async function commitNudge() {
  const done = nudge;
  nudge = null;
  if (!done) return;
  const x = mmValue(done.orig[0] + done.dx);
  const y = mmValue(done.orig[1] + done.dy);
  await moveControl(done.code, x, y);
}

// --- Course mode -------------------------------------------------------------------------------------------------
// A click on a control appends it before the finish. A click on a leg chooses that leg; the next click on a
// control inserts the control there. Esc drops the chosen leg.

const LEG_HIT_PX = 8;

// The `after` of the leg nearest to a map point within a few pixels, or null.
function hitLeg(at) {
  const reach = LEG_HIT_PX / state.view.scale;
  let best = null;
  for (const leg of state.overlay?.legs ?? []) {
    for (let i = 1; i < leg.drawn.length; i++) {
      const d = bends.distanceToSegment(at, leg.drawn[i - 1], leg.drawn[i]);
      if (d <= reach && (!best || d < best.d)) best = { after: leg.after, d };
    }
  }
  return best?.after ?? null;
}

function dropPendingLeg() {
  if (pendingLeg === null) return false;
  pendingLeg = null;
  showOverlay();
  status.say("editor-status-leg-cancelled");
  return true;
}

async function courseClick(at) {
  if (!state.course || !state.overlay) {
    status.say("editor-status-pick-course");
    return;
  }
  if (isScore()) {
    status.say("editor-score-no-course-mode");
    return;
  }
  const code = hitControl(at, state.pool);
  if (code) {
    const stops = state.data.event.courses.find((c) => c.name === state.course).controls.map((c) => String(c.code));
    const next = pendingLeg === null ? course.withAppended(stops, code) : course.withInsertedAt(stops, pendingLeg, code);
    pendingLeg = null;
    await setStops(state.course, next, null);
    return;
  }
  const leg = hitLeg(at);
  if (leg !== null) {
    pendingLeg = leg;
    showOverlay();
    status.say(`${t("editor-leg-word")} ${course.legLabel(leg)}. ${t("editor-status-leg-picked")}`, true);
  } else {
    dropPendingLeg();
  }
}

tools.onChange((name) => {
  pendingLeg = null;
  draft = null;
  if (name === "special") {
    proposeId();
    status.say("editor-status-special-tool");
  }
  if (name === "course" && isScore()) status.say("editor-score-no-course-mode");
  else if (name === "course") status.say(state.course ? "editor-status-course-mode" : "editor-status-pick-course");
  showOverlay();
});

function bindMapInput() {
  const view = state.view;
  // A press on the map: { id (the pointer), x, y (pixels), moved, target, at } or a control drag in `drag`.
  // Only the primary pointer counts: a second finger neither places, moves nor selects, and the events of
  // another pointer than the one that pressed are ignored.
  let press = null;
  const ours = (evt) => press !== null && evt.pointerId === press.id;

  svg.addEventListener("pointerdown", (evt) => {
    if (!evt.isPrimary || evt.button !== 0 || busy) return;
    const at = view.screenToMap(evt);
    const hit = tools.getTool() === "select" && !state.preview ? hitControl(at) : null;
    if (tools.getTool() === "course" && state.overlay && !state.preview) {
      const box = numbers.numberAt(state.overlay.numbers, at, 0);
      if (box) numberDrag = { number: box, start: at, orig: [...box.centre], moved: false };
    }
    press = { id: evt.pointerId, x: evt.clientX, y: evt.clientY, moved: false, target: evt.target, at };
    if (state.preview) {
      // Read-only: a press only pans.
      try {
        svg.setPointerCapture(evt.pointerId);
      } catch {
        // The pointer is gone already.
      }
      return;
    }
    if (hit) {
      const c = state.overlay.controls.find((x) => x.code === hit);
      drag = { code: hit, start: at, orig: [...c.at], moved: false };
    } else if (tools.getTool() === "select") {
      const target = legTarget(at);
      if (target) {
        legDrag = {
          after: target.leg.after,
          index: target.bend ?? null,
          at: target.line,
          orig: target.leg.bends.map((b) => ({ ...b })),
          start: at,
          inserted: false,
          moved: false,
        };
      }
    }
    try {
      svg.setPointerCapture(evt.pointerId);
    } catch {
      // The pointer is gone already (a synthetic or very short touch); the press still ends with its own events.
    }
  });
  svg.addEventListener("pointermove", (evt) => {
    if (!evt.isPrimary) return;
    status.setCursor(view.screenToMap(evt));
    if (draft) {
      draft.cursor = view.screenToMap(evt);
      showOverlay();
    }
    if (!ours(evt)) return;
    const dx = evt.clientX - press.x;
    const dy = evt.clientY - press.y;
    if (!press.moved && Math.hypot(dx, dy) < DRAG_PIXELS) return;
    press.moved = true;
    if (legDrag) {
      dragBend(view.screenToMap(evt));
      return;
    }
    if (numberDrag) {
      const at = view.screenToMap(evt);
      if (!numberDrag.moved) {
        numberDrag.moved = true;
        state.selection = { kind: "number", id: String(numberDrag.number.index), course: state.course, index: numberDrag.number.index };
      }
      numberDrag.number.centre = [numberDrag.orig[0] + at[0] - numberDrag.start[0], numberDrag.orig[1] + at[1] - numberDrag.start[1]];
      showOverlay();
      return;
    }
    if (drag) {
      const at = view.screenToMap(evt);
      if (!drag.moved) {
        drag.moved = true;
        state.selection = { kind: "control", id: drag.code };
        state.treeKey = `control:${drag.code}`;
      }
      setCircle(drag.code, [drag.orig[0] + at[0] - drag.start[0], drag.orig[1] + at[1] - drag.start[1]]);
      showOverlay();
      return;
    }
    view.panBy(-dx / view.scale, -dy / view.scale);
    press.x = evt.clientX;
    press.y = evt.clientY;
  });
  svg.addEventListener("pointerup", (evt) => {
    if (!ours(evt)) return;
    const finished = press;
    const dragged = drag;
    const bending = legDrag;
    const carried = numberDrag;
    press = null;
    drag = null;
    numberDrag = null;
    if (carried) {
      if (svg.hasPointerCapture(evt.pointerId)) svg.releasePointerCapture(evt.pointerId);
      if (carried.moved) {
        dropNumber(carried.number, [carried.number.centre[0], carried.number.centre[1]]);
      } else {
        // A click on a number box selects it.
        state.selection = { kind: "number", id: String(carried.number.index), course: state.course, index: carried.number.index };
        redraw();
        status.say("editor-status-number-selected");
      }
      return;
    }
    if (bending?.moved) {
      if (svg.hasPointerCapture(evt.pointerId)) svg.releasePointerCapture(evt.pointerId);
      dropBend();
      return;
    }
    legDrag = null;
    if (svg.hasPointerCapture(evt.pointerId)) svg.releasePointerCapture(evt.pointerId);
    if (!finished) return;
    if (dragged?.moved) {
      const c = state.overlay.controls.find((x) => x.code === dragged.code);
      const [x, y] = [mmValue(c.at[0]), mmValue(c.at[1])];
      if (x === mmValue(dragged.orig[0]) && y === mmValue(dragged.orig[1])) {
        setCircle(dragged.code, dragged.orig);
        redraw();
      } else {
        moveControl(dragged.code, x, y);
      }
      return;
    }
    if (finished.moved || state.preview) return;
    const tool = tools.getTool();
    if (tool === "course") courseClick(finished.at);
    else if (tool === "special") specialClick(finished.at);
    else if (tool !== "select" && !hitControl(finished.at)) placeControl(finished.at, tool === "crossing");
    else selectDrawn(finished.target, finished.at);
  });
  // A cancelled touch or a lost capture ends the press like a release, without selecting or moving anything.
  const abandon = () => {
    cancelDrag();
    press = null;
  };
  // A reload that waited for the gesture to end runs now.
  svg.addEventListener("pointerup", () => setTimeout(poll, 300));
  svg.addEventListener("pointercancel", (evt) => {
    if (ours(evt)) abandon();
  });
  svg.addEventListener("lostpointercapture", (evt) => {
    if (ours(evt)) abandon();
  });
  svg.addEventListener("pointerleave", () => status.setCursor(null));
  // A double click ends the drawing; its clicks added their point once (specialClick ignores a repeat).
  svg.addEventListener("dblclick", () => {
    if (draft) finishDraft();
  });
  svg.addEventListener("wheel", (evt) => {
    evt.preventDefault();
    view.zoomBy(evt.deltaY < 0 ? 1.2 : 1 / 1.2, view.screenToPage(evt));
  }, { passive: false });
  document.getElementById("zoom-in").addEventListener("click", () => view.zoomBy(1.25));
  document.getElementById("zoom-out").addEventListener("click", () => view.zoomBy(1 / 1.25));
  document.getElementById("zoom-fit").addEventListener("click", () => view.fit(state.pageBox));

  keys.bind({
    undo: () => history(false),
    redo: () => history(true),
    remove: requestDelete,
    cancel: () => {
      if (drag || nudge || legDrag || numberDrag) {
        if (nudge) {
          setCircle(nudge.code, nudge.orig);
          nudge = null;
          showOverlay();
        }
        cancelDrag();
        press = null;
      } else if (!dropDraft() && !dropPendingLeg()) {
        tools.setTool("select");
      }
    },
    tool: (name) => { if (!state.preview) tools.setTool(name); },
    preview: togglePreview,
    nudge: startNudge,
    commitNudge,
    canNudge: () => state.selection?.kind === "control",
    drafting: () => draft !== null,
    finish: () => { if (!state.preview) finishDraft(); },
    zoom: (f) => view.zoomBy(f),
    fit: () => view.fit(state.pageBox),
    pan: (dx, dy) => {
      const v = view.visible();
      view.panBy(dx * (v.x1 - v.x0) * PAN_STEP, dy * (v.y1 - v.y0) * PAN_STEP);
    },
    help: () => helpDialog.showModal(),
    dialogOpen: () => confirmDialog.open || helpDialog.open || nameDialog.open || findings.dialogOpen() || exchange.dialogOpen() || sync.dialogOpen(),
  });
}

// Polling (plan D8): every two seconds while the page is visible, and at once when it gets the focus.
let polling = false;
// Whether a press holds map data of the loaded state: a reload would replace what the gesture moves.
const gestureActive = () => !!(drag || nudge || legDrag || numberDrag);

async function poll() {
  if (polling || document.hidden || state.closed || gestureActive() || busy) return;
  polling = true;
  const epoch = writes;
  try {
    const answer = await get("api/revision");
    // The server answers a poll queued behind our own export, import or sync with the new revision before the
    // page has noted it: such an answer says nothing about outside changes.
    if (!answer.ok || busy || epoch !== writes) return;
    const now = answer.data;
    const before = state.seen;
    state.seen = now;
    if (!before) return;
    if (now.project !== before.project) {
      if (await loadState(true)) await loadMapInfo(false);
    }
    if (now.map !== before.map) {
      await loadMapInfo(false);
      state.mapLayer?.refresh(true);
    }
    if (now.ppen !== before.ppen && now.ppen !== "") status.say("editor-status-ppen-changed");
  } catch {
    if (!state.closed) status.say("editor-status-lost");
  } finally {
    polling = false;
  }
}

try {
  await loadTexts();
  status.setFindings(null);
  fillKinds();
  tools.bind();
  keys.fillHelp(document.getElementById("help-keys"));
  status.say("editor-status-ready");
  const known = await get("api/rules");
  if (known.ok && known.data?.profiles) state.rules = known.data;
  const first = await get("api/revision");
  state.seen = first.ok ? first.data : null;
  if (await loadState(false)) {
    await loadMapInfo(true);
    state.mapLayer.refresh(false);
  }
} catch {
  // Without texts the page stays as it is; the server console says why.
}

pollTimer = setInterval(poll, POLL_MS);
window.addEventListener("focus", poll);
document.addEventListener("visibilitychange", poll);
