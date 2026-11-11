import { get, post } from "./api.js";
import { loadTexts, t } from "./texts.js";
import { createView, bindMapLayer } from "./view.js";
import * as overlay from "./overlay.js";
import { buildTree } from "./tree.js";
import * as props from "./props.js";
import * as status from "./status.js";
import * as tools from "./tools.js";
import * as keys from "./keys.js";

const POLL_MS = 2000;
const PAN_STEP = 0.1;
const DRAG_PIXELS = 4;

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
};
const NEAR_RADIUS_MM = 3;
let pollTimer = null;

const svg = document.getElementById("map-svg");
const propsHost = document.getElementById("props");
overlay.attach(document.getElementById("layer-overlay"));

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
    { id: "courses", label: t("editor-tree-courses"), items: event.courses.map((c) => ({ key: `course:${c.name}`, label: c.name })) },
    { id: "classes", label: t("editor-tree-classes"), items: event.classes.map((c) => ({ key: `class:${c.name}`, label: c.name })) },
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
  overlay.draw(state.overlay, state.selection, state.course);
  props.show(propsHost, state.data, state.selection, state.overlay, propActions);
  state.tree?.setCurrent(state.treeKey);
  loadNear();
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

function failure(answer) {
  const data = answer.data;
  return typeof data === "string" ? data : (data?.error ?? "");
}

// Runs one operation; the answer's data on success, or null (the banner says why).
async function runOp(tool, body) {
  if (busy || state.closed) return null;
  busy = true;
  try {
    return await settle(await post(`api/op/${tool}`, body, state.revision));
  } catch {
    status.say("editor-status-lost");
    return null;
  } finally {
    busy = false;
  }
}

async function settle(answer) {
  if (answer.ok) {
    state.revision = answer.data.revision;
    // Our own write is not an outside change: the poll must not announce it.
    if (state.seen) state.seen = { ...state.seen, project: answer.data.revision };
    status.clearBanner();
    await loadState(false);
    return answer.data;
  }
  const message = failure(answer);
  if (message) {
    status.showBanner(message);
    status.say(message, true);
  }
  if (answer.status === 409 && typeof answer.data === "object") state.revision = answer.data.revision;
  await loadState(false);
  return null;
}

async function history(redo) {
  if (busy || state.closed) return;
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
    busy = false;
  }
}

const mmValue = (v) => Math.round(v * 100) / 100;

async function moveControl(code, x, y) {
  const done = await runOp("move_control", { code, at: { x, y } });
  if (done) status.say("editor-status-moved");
}

async function placeControl(at, crossing) {
  const body = { at: { x: mmValue(at[0]), y: mmValue(at[1]) } };
  if (crossing) body.crossing_point = true;
  const done = await runOp("add_control", body);
  if (!done) return;
  status.say("editor-status-placed");
  await selectKey(`control:${done.result.code}`);
}

const confirmDialog = document.getElementById("confirm-dialog");
const helpDialog = document.getElementById("help-dialog");

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

// Deleting asks first when a course visits the control.
function requestDelete() {
  const selected = state.selection;
  if (selected?.kind !== "control" || !controlOf(selected.id) || busy) return;
  const users = coursesUsing(selected.id);
  if (users.length === 0) {
    deleteControl(selected.id);
    return;
  }
  const list = document.getElementById("confirm-courses");
  list.replaceChildren(...users.map((name) => {
    const li = document.createElement("li");
    li.textContent = name;
    return li;
  }));
  confirmDialog.dataset.code = String(selected.id);
  confirmDialog.showModal();
}

document.getElementById("confirm-no").addEventListener("click", () => confirmDialog.close());
document.getElementById("confirm-yes").addEventListener("click", () => {
  confirmDialog.close();
  deleteControl(confirmDialog.dataset.code);
});

const propActions = { move: moveControl, remove: requestDelete };

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
  state.overlay = answer.data;
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
  state.treeKey = key;
  if (key === "all") {
    state.selection = null;
    await loadOverlay(null);
  } else if (kind === "course") {
    state.selection = { kind: "course", id };
    await loadOverlay(id);
  } else if (kind === "class") {
    state.selection = { kind: "class", id };
  } else {
    state.selection = { kind, id };
    if (!overlayHas(kind, id)) await loadOverlay(null);
  }
  redraw();
}

// The control whose circle a map point (map mm) hits: within the circle's radius plus the line width.
function hitControl(at) {
  const o = state.overlay;
  if (!o) return null;
  const reach = o.sizes.circle_r + o.sizes.line;
  let best = null;
  for (const c of o.controls) {
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
  state.data = answer.data;
  const m = answer.data.map;
  status.setMap(m.path, m.scale, m.kind);
  renderTree();
  const known = (name) => state.data.event.courses.some((c) => c.name === name);
  if (state.course && !known(state.course)) {
    state.course = null;
    state.selection = null;
    state.treeKey = "all";
  }
  if (state.selection?.kind === "control" && !controlOf(state.selection.id)) {
    state.selection = null;
    state.treeKey = state.course ? `course:${state.course}` : "all";
  }
  updateHistoryButtons();
  await loadOverlay(state.course);
  redraw();
  if (announce) status.say("editor-status-reloaded");
  return true;
}

async function loadMapInfo(fit) {
  const answer = await get("api/map/view");
  if (!answer.ok) return;
  const info = answer.data;
  if (!state.view) {
    state.view = createView(svg, info.mapping);
    state.mapLayer = bindMapLayer(document.getElementById("layer-map"), state.view);
    state.view.addEventListener("viewchange", () => {
      document.getElementById("layer-overlay").setAttribute("transform", state.view.transform());
    });
    bindMapInput();
    fit = true;
  }
  state.view.setMapping(info.mapping);
  document.getElementById("layer-overlay").setAttribute("transform", state.view.transform());
  state.pageBox = info.page_box;
  if (fit) state.view.fit(info.page_box);
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
  overlay.draw(state.overlay, state.selection, state.course);
}

function cancelDrag() {
  if (drag?.moved) setCircle(drag.code, drag.orig);
  if (drag?.moved) showOverlay();
  drag = null;
}

function startNudge(dx, dy) {
  const selected = state.selection;
  if (!selected || busy) return;
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

function bindMapInput() {
  const view = state.view;
  // A press on the map: { x, y (pixels), moved, target, pan } or a control drag in `drag`.
  let press = null;

  svg.addEventListener("pointerdown", (evt) => {
    if (evt.button !== 0 || busy) return;
    const at = view.screenToMap(evt);
    const hit = tools.getTool() === "select" ? hitControl(at) : null;
    press = { x: evt.clientX, y: evt.clientY, moved: false, target: evt.target, at };
    if (hit) {
      const c = state.overlay.controls.find((x) => x.code === hit);
      drag = { code: hit, start: at, orig: [...c.at], moved: false };
    }
    svg.setPointerCapture(evt.pointerId);
  });
  svg.addEventListener("pointermove", (evt) => {
    status.setCursor(view.screenToMap(evt));
    if (!press) return;
    const dx = evt.clientX - press.x;
    const dy = evt.clientY - press.y;
    if (!press.moved && Math.hypot(dx, dy) < DRAG_PIXELS) return;
    press.moved = true;
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
    const finished = press;
    const dragged = drag;
    press = null;
    drag = null;
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
    if (finished.moved) return;
    const tool = tools.getTool();
    if (tool !== "select" && !hitControl(finished.at)) placeControl(finished.at, tool === "crossing");
    else selectDrawn(finished.target, finished.at);
  });
  // A cancelled touch or a lost capture ends the press like a release, without selecting or moving anything.
  const abandon = () => {
    cancelDrag();
    press = null;
  };
  svg.addEventListener("pointercancel", abandon);
  svg.addEventListener("lostpointercapture", () => {
    if (press) abandon();
  });
  svg.addEventListener("pointerleave", () => status.setCursor(null));
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
      if (drag || nudge) {
        if (nudge) {
          setCircle(nudge.code, nudge.orig);
          nudge = null;
          showOverlay();
        }
        cancelDrag();
        press = null;
      } else {
        tools.setTool("select");
      }
    },
    tool: tools.setTool,
    nudge: startNudge,
    commitNudge,
    canNudge: () => state.selection?.kind === "control",
    zoom: (f) => view.zoomBy(f),
    fit: () => view.fit(state.pageBox),
    pan: (dx, dy) => {
      const v = view.visible();
      view.panBy(dx * (v.x1 - v.x0) * PAN_STEP, dy * (v.y1 - v.y0) * PAN_STEP);
    },
    help: () => helpDialog.showModal(),
    dialogOpen: () => confirmDialog.open || helpDialog.open,
  });
}

// Polling (plan D8): every two seconds while the page is visible, and at once when it gets the focus.
let polling = false;
async function poll() {
  if (polling || document.hidden || state.closed) return;
  polling = true;
  try {
    const answer = await get("api/revision");
    if (!answer.ok) return;
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
  tools.bind();
  keys.fillHelp(document.getElementById("help-keys"));
  status.say("editor-status-ready");
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
