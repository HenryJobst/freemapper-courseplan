import { get, post } from "./api.js";
import { loadTexts, t } from "./texts.js";
import { createView, bindMapLayer } from "./view.js";
import * as overlay from "./overlay.js";
import { buildTree } from "./tree.js";
import * as props from "./props.js";
import * as status from "./status.js";

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
};

const svg = document.getElementById("map-svg");
const propsHost = document.getElementById("props");
overlay.attach(document.getElementById("layer-overlay"));

async function closeEditor() {
  try {
    const answer = await post("api/quit");
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
  props.show(propsHost, state.data, state.selection, state.overlay);
  state.tree?.setCurrent(state.treeKey);
}

async function loadOverlay(course) {
  const query = course ? `?course=${encodeURIComponent(course)}` : "";
  const answer = await get(`api/overlay${query}`);
  if (!answer.ok) {
    state.overlay = null;
    status.say(answer.data, true);
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

// A click on a symbol selects it; the tree follows when the symbol is a control or a special object.
function selectDrawn(target) {
  const node = target.closest("[data-kind]");
  if (!node || node.dataset.kind === "leg") {
    state.selection = null;
    state.treeKey = state.course ? `course:${state.course}` : "all";
  } else if (node.dataset.kind === "number") {
    state.selection = { kind: "number", id: node.dataset.id, course: state.course, index: Number(node.dataset.id) };
  } else {
    state.selection = { kind: node.dataset.kind, id: node.dataset.id };
    state.treeKey = `${node.dataset.kind}:${node.dataset.id}`;
  }
  redraw();
}

async function loadState(announce) {
  const answer = await get("api/state");
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
  if (!(await loadOverlay(state.course))) status.say("editor-status-overlay-failed");
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

function bindMapInput() {
  const view = state.view;
  let drag = null;

  svg.addEventListener("pointerdown", (evt) => {
    if (evt.button !== 0) return;
    drag = { x: evt.clientX, y: evt.clientY, moved: false, target: evt.target };
    svg.setPointerCapture(evt.pointerId);
  });
  svg.addEventListener("pointermove", (evt) => {
    status.setCursor(view.screenToMap(evt));
    if (!drag) return;
    const dx = evt.clientX - drag.x;
    const dy = evt.clientY - drag.y;
    if (!drag.moved && Math.hypot(dx, dy) < DRAG_PIXELS) return;
    drag.moved = true;
    view.panBy(-dx / view.scale, -dy / view.scale);
    drag.x = evt.clientX;
    drag.y = evt.clientY;
  });
  svg.addEventListener("pointerup", (evt) => {
    const finished = drag;
    drag = null;
    if (svg.hasPointerCapture(evt.pointerId)) svg.releasePointerCapture(evt.pointerId);
    if (finished && !finished.moved) selectDrawn(finished.target);
  });
  svg.addEventListener("pointerleave", () => status.setCursor(null));
  svg.addEventListener("wheel", (evt) => {
    evt.preventDefault();
    view.zoomBy(evt.deltaY < 0 ? 1.2 : 1 / 1.2, view.screenToPage(evt));
  }, { passive: false });
  svg.addEventListener("keydown", (evt) => {
    const v = view.visible();
    const stepX = (v.x1 - v.x0) * PAN_STEP;
    const stepY = (v.y1 - v.y0) * PAN_STEP;
    const actions = {
      ArrowLeft: () => view.panBy(-stepX, 0),
      ArrowRight: () => view.panBy(stepX, 0),
      ArrowUp: () => view.panBy(0, -stepY),
      ArrowDown: () => view.panBy(0, stepY),
      "+": () => view.zoomBy(1.25),
      "=": () => view.zoomBy(1.25),
      "-": () => view.zoomBy(1 / 1.25),
      0: () => view.fit(state.pageBox),
    };
    const action = actions[evt.key];
    if (action) {
      evt.preventDefault();
      action();
    }
  });
  document.getElementById("zoom-in").addEventListener("click", () => view.zoomBy(1.25));
  document.getElementById("zoom-out").addEventListener("click", () => view.zoomBy(1 / 1.25));
  document.getElementById("zoom-fit").addEventListener("click", () => view.fit(state.pageBox));
}

// Polling (plan D8): every two seconds while the page is visible, and at once when it gets the focus.
let polling = false;
async function poll() {
  if (polling || document.hidden) return;
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
    status.say("editor-status-lost");
  } finally {
    polling = false;
  }
}

try {
  await loadTexts();
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

setInterval(poll, POLL_MS);
window.addEventListener("focus", poll);
document.addEventListener("visibilitychange", poll);
