// Draws the course overlay. Everything comes from api/overlay in map millimetres: the page puts the layer
// under one transform, so this module does no geometry beyond placing a symbol's own glyph at a point.
// Each drawn element carries data-kind and data-id. A selection is marked by a dashed halo as well as a colour.

import { t } from "./texts.js";
import { CUES, middle } from "./specials.js";

const NS = "http://www.w3.org/2000/svg";

let layer = null;

export function attach(group) {
  layer = group;
}

function el(name, attributes = {}, parent = null) {
  const node = document.createElementNS(NS, name);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, value);
  if (parent) parent.append(node);
  return node;
}

function pointList(points) {
  return points.map((p) => `${p[0]},${p[1]}`).join(" ");
}

export function kindLabel(kind) {
  return t(`editor-kind-${kind.replaceAll("_", "-")}`);
}

function isSelected(selection, kind, id) {
  return selection != null && selection.kind === kind && String(selection.id) === String(id);
}

// Marks a drawn element and, when it is selected, adds the halo.
function tag(node, kind, id, selection, halos, haloOf) {
  node.dataset.kind = kind;
  node.dataset.id = String(id);
  if (isSelected(selection, kind, id)) {
    node.classList.add("selected");
    halos.append(haloOf());
  }
  return node;
}

function drawControl(c, o, selection, parent, halos) {
  const r = o.sizes.circle_r;
  const group = el("g", { class: "sym" }, parent);
  if (c.kind === "crossing_point") {
    // Two arcs, ")(": the crossing point has no circle.
    const a = 0.9 * r;
    const b = 0.3 * r;
    el("path", {
      d: `M${c.at[0] - a},${c.at[1] - r} Q${c.at[0] - b},${c.at[1]} ${c.at[0] - a},${c.at[1] + r} ` +
        `M${c.at[0] + a},${c.at[1] - r} Q${c.at[0] + b},${c.at[1]} ${c.at[0] + a},${c.at[1] + r}`,
      "stroke-width": o.sizes.line,
      fill: "none",
    }, group);
    el("circle", { cx: c.at[0], cy: c.at[1], r, class: "hit" }, group);
  } else {
    el("circle", { cx: c.at[0], cy: c.at[1], r, "stroke-width": o.sizes.line, fill: "none" }, group);
  }
  return tag(group, "control", c.code, selection, halos, () =>
    el("circle", { cx: c.at[0], cy: c.at[1], r: r * 1.5, class: "halo", "stroke-width": o.sizes.line * 1.5 }));
}

function drawStart(s, o, selection, parent, halos) {
  const r = o.sizes.circle_r * 1.15;
  const group = el("g", { class: "sym" }, parent);
  // A triangle pointing east, turned counter-clockwise on the paper by angle_deg.
  el("polygon", {
    points: `${r},0 ${-r / 2},${r * 0.866} ${-r / 2},${-r * 0.866}`,
    transform: `translate(${s.at[0]} ${s.at[1]}) rotate(${-s.angle_deg})`,
    "stroke-width": o.sizes.line,
    fill: "none",
  }, group);
  return tag(group, "special", s.id, selection, halos, () =>
    el("circle", { cx: s.at[0], cy: s.at[1], r: r * 1.5, class: "halo", "stroke-width": o.sizes.line * 1.5 }));
}

function drawFinish(f, o, selection, parent, halos) {
  const r = o.sizes.circle_r;
  const group = el("g", { class: "sym" }, parent);
  el("circle", { cx: f.at[0], cy: f.at[1], r, "stroke-width": o.sizes.line, fill: "none" }, group);
  el("circle", { cx: f.at[0], cy: f.at[1], r: r * 0.7, "stroke-width": o.sizes.line, fill: "none" }, group);
  return tag(group, "special", f.id, selection, halos, () =>
    el("circle", { cx: f.at[0], cy: f.at[1], r: r * 1.5, class: "halo", "stroke-width": o.sizes.line * 1.5 }));
}

function drawNumber(n, o, course, selection, parent, halos) {
  const [cx, cy] = n.centre;
  const group = el("g", { class: `sym number source-${n.source}` }, parent);
  el("rect", {
    x: cx - n.w / 2,
    y: cy - n.h / 2,
    width: n.w,
    height: n.h,
    "stroke-width": o.sizes.line * 0.5,
    class: n.source === "auto" ? "auto" : "",
  }, group);
  const text = el("text", {
    x: cx,
    y: cy,
    "font-size": o.sizes.number_h,
    "text-anchor": "middle",
    "dominant-baseline": "central",
  }, group);
  text.textContent = n.text;
  group.dataset.course = course ?? "";
  return tag(group, "number", n.index, selection, halos, () =>
    el("rect", {
      x: cx - n.w / 2 - o.sizes.line * 2,
      y: cy - n.h / 2 - o.sizes.line * 2,
      width: n.w + o.sizes.line * 4,
      height: n.h + o.sizes.line * 4,
      class: "halo",
      "stroke-width": o.sizes.line * 1.5,
    }));
}

// A forbidden route of one point is a cross; first aid is a plus sign in a square, water a circle.
function cross(at, r, width, parent) {
  return el("path", {
    d: `M${at[0] - r},${at[1] - r}L${at[0] + r},${at[1] + r}M${at[0] - r},${at[1] + r}L${at[0] + r},${at[1] - r}`,
    "stroke-width": width,
    fill: "none",
  }, parent);
}

function drawSpecial(s, o, selection, parent, halos) {
  const group = el("g", { class: `sym special ${s.kind}` }, parent);
  const w = o.sizes.line * 1.5;
  const r = o.sizes.circle_r;
  const first = s.points[0];
  const line = (extra) => {
    // The wide transparent copy makes a thin line easy to hit.
    el("polyline", { points: pointList(s.points), class: "hit-line", "stroke-width": Math.max(w * 4, r) }, group);
    return el("polyline", { points: pointList(s.points), fill: "none", ...extra }, group);
  };
  const shape = {
    marked_route: () => line({ "stroke-width": w * 2 }),
    forbidden_route: () => {
      if (s.points.length === 1) cross(first, r, w * 2, group);
      else line({ "stroke-width": w, "stroke-dasharray": `${w * 4} ${w * 3}` });
    },
    out_of_bounds_area: () => el("polygon", { points: pointList(s.points), "stroke-width": w }, group),
    dangerous_area: () => el("polygon", {
      points: pointList(s.points), "stroke-width": w, "stroke-dasharray": `${w * 3} ${w * 2}`,
    }, group),
    first_aid: () => {
      el("rect", { x: first[0] - r, y: first[1] - r, width: r * 2, height: r * 2, "stroke-width": w, fill: "none" }, group);
      el("path", {
        d: `M${first[0] - r * 0.6},${first[1]}H${first[0] + r * 0.6}M${first[0]},${first[1] - r * 0.6}V${first[1] + r * 0.6}`,
        "stroke-width": w * 2,
      }, group);
    },
    water: () => el("circle", { cx: first[0], cy: first[1], r: r * 0.6, "stroke-width": w, fill: "none" }, group),
  }[s.kind];
  if (shape) shape();
  // The cue letter sits beside the middle of the object, so the kind is also told by text.
  const [mx, my] = middle(s.points);
  const point = s.points.length === 1;
  const cue = el("text", {
    x: point ? mx + r * 1.4 : mx,
    y: point ? my - r * 1.1 : my,
    "font-size": o.sizes.number_h * 0.9,
    "text-anchor": point ? "start" : "middle",
    "dominant-baseline": "central",
    class: "cue",
  }, group);
  cue.textContent = CUES[s.kind] ?? "";
  el("title", {}, group).textContent = `${kindLabel(s.kind)} ${s.id}`;
  return tag(group, "special", s.id, selection, halos, () =>
    el("circle", { cx: mx, cy: my, r: Math.max(r * 1.8, o.sizes.line * 6), class: "halo", "stroke-width": w }));
}

// The drawing in progress: the points clicked so far, joined, and a band to the pointer. A point kind needs no
// preview. The preview is dashed and marked by dots, so it differs from a finished object without colour.
function drawDraft(draft, o, parent) {
  const group = el("g", { class: "draft" }, parent);
  const line = [...draft.points, ...(draft.cursor ? [draft.cursor] : [])];
  const w = o.sizes.line * 1.5;
  if (line.length > 1) {
    el("polyline", { points: pointList(line), fill: "none", "stroke-width": w, "stroke-dasharray": `${w * 2} ${w * 2}` }, group);
  }
  if (draft.shape === "area" && draft.points.length > 2) {
    el("polygon", { points: pointList(draft.points), class: "draft-fill", "stroke-width": 0 }, group);
  }
  for (const p of draft.points) {
    el("circle", { cx: p[0], cy: p[1], r: o.sizes.circle_r * 0.25, "stroke-width": o.sizes.line }, group);
  }
}

// The handles of the bends of the selected leg, and a halo along the leg.
function drawBends(leg, o, selection, parent, halos) {
  el("polyline", {
    points: pointList(leg.drawn),
    class: "halo",
    "stroke-width": o.sizes.line * 3,
  }, halos);
  leg.bends.forEach((b, i) => {
    const chosen = selection.kind === "bend" && selection.index === i;
    const handle = el("circle", {
      cx: b.x,
      cy: b.y,
      r: o.sizes.circle_r * 0.3,
      class: chosen ? "bend selected" : "bend",
      "stroke-width": o.sizes.line,
    }, parent);
    handle.dataset.kind = "bend";
    handle.dataset.id = `${leg.after}:${i}`;
    if (chosen) el("circle", { cx: b.x, cy: b.y, r: o.sizes.circle_r * 0.6, class: "halo", "stroke-width": o.sizes.line }, halos);
  });
}

// Draws `overlay` (the answer of api/overlay) into the attached layer. `course` names the course drawn, or null.
// `extras` (course mode): { pool: controls not in the course, drawn faint and clickable; pending: the leg
// (its `after`) chosen for an insertion, drawn thick }.
export function draw(overlay, selection, course = null, extras = {}) {
  layer.replaceChildren();
  if (!overlay) return;
  const halos = el("g", { class: "halos" }, layer);
  const pool = el("g", { class: "pool" }, layer);
  for (const c of extras.pool ?? []) {
    drawControl(c, overlay, selection, pool, halos);
    const r = overlay.sizes.circle_r;
    const label = el("text", { x: c.at[0] + r * 1.4, y: c.at[1] - r * 1.1, "font-size": overlay.sizes.number_h * 0.9, class: "code" }, pool);
    label.textContent = c.code;
  }
  const body = el("g", { class: "course" }, layer);
  for (const s of overlay.specials) drawSpecial(s, overlay, selection, body, halos);
  for (const leg of overlay.legs) {
    const line = el("polyline", {
      points: pointList(leg.drawn),
      fill: "none",
      class: `${leg.taped ? "leg taped" : "leg"}${leg.after === extras.pending ? " pending" : ""}`,
      "stroke-width": overlay.sizes.line,
    }, body);
    if (leg.taped) line.setAttribute("stroke-dasharray", `${overlay.sizes.line * 4} ${overlay.sizes.line * 3}`);
    line.dataset.kind = "leg";
    line.dataset.id = `${course ?? ""}:${leg.after}`;
    if (selection && (selection.kind === "leg" || selection.kind === "bend") && selection.after === leg.after) {
      line.classList.add("selected");
      drawBends(leg, overlay, selection, body, halos);
    }
  }
  for (const s of overlay.starts) drawStart(s, overlay, selection, body, halos);
  for (const f of overlay.finishes) drawFinish(f, overlay, selection, body, halos);
  for (const c of overlay.controls) drawControl(c, overlay, selection, body, halos);
  for (const n of overlay.numbers) drawNumber(n, overlay, course, selection, body, halos);
  if (extras.draft) drawDraft(extras.draft, overlay, layer);
  if (overlay.numbers.length === 0) {
    // The "all controls" view has no numbers: each circle gets its code beside it.
    const r = overlay.sizes.circle_r;
    for (const c of overlay.controls) {
      const label = el("text", {
        x: c.at[0] + r * 1.4,
        y: c.at[1] - r * 1.1,
        "font-size": overlay.sizes.number_h * 0.9,
        class: "code",
      }, body);
      label.textContent = c.code;
    }
  }
}

// --- Findings on the map ---------------------------------------------------------------------------------------------
// A ring around each control with a finding, a thick dashed line along each leg, and a small sign for the
// severity at the place. The signs differ in shape (cross in a circle, triangle, letter i in a circle), so the
// colour is not the only thing that tells them apart. A gap outside the rule range puts its sign at the
// number box. The layer takes no pointer events: the map below stays clickable.

const GAP_RULE = "wkb2026.number.gap";

function sign(severity, at, size, width, parent) {
  const group = el("g", { class: `finding-sign sev-${severity}`, "stroke-width": width, fill: "none" }, parent);
  const [x, y] = at;
  if (severity === "error") {
    el("circle", { cx: x, cy: y, r: size }, group);
    const d = size * 0.5;
    el("path", { d: `M${x - d},${y - d}L${x + d},${y + d}M${x - d},${y + d}L${x + d},${y - d}` }, group);
  } else if (severity === "warning") {
    const h = size * 1.2;
    el("path", { d: `M${x},${y - h}L${x + h},${y + h * 0.8}L${x - h},${y + h * 0.8}Z` }, group);
    el("path", { d: `M${x},${y - h * 0.3}V${y + h * 0.25}M${x},${y + h * 0.5}V${y + h * 0.55}` }, group);
  } else {
    el("circle", { cx: x, cy: y, r: size }, group);
    el("path", { d: `M${x},${y - size * 0.15}V${y + size * 0.5}M${x},${y - size * 0.5}V${y - size * 0.45}` }, group);
  }
  return group;
}

function middleOf(points) {
  return points[Math.floor((points.length - 1) / 2)];
}

// Draws `findings` (the open ones of api/check) into `layer` over `overlay` (api/overlay of the drawn course, or
// of all controls). `course` names the course drawn, or null.
export function drawFindings(layer, overlay, findings, course) {
  layer.replaceChildren();
  if (!overlay || !findings?.length) return;
  const r = overlay.sizes.circle_r;
  const w = overlay.sizes.line * 1.2;
  const crowd = new Map();
  const next = (key) => {
    const n = crowd.get(key) ?? 0;
    crowd.set(key, n + 1);
    return n;
  };
  const place = (code) =>
    [...overlay.controls, ...overlay.starts, ...overlay.finishes].find((c) => String(c.code ?? c.id) === String(code));
  const ring = (code, severity) => {
    const c = place(code);
    if (!c) return;
    const n = next(`c:${code}`);
    el("circle", { cx: c.at[0], cy: c.at[1], r: r * (1.7 + 0.3 * n), class: `finding-ring sev-${severity}`, "stroke-width": w, fill: "none" }, layer);
    sign(severity, [c.at[0] + r * (1.9 + 0.3 * n), c.at[1] - r * (1.9 + 0.3 * n)], r * 0.75, w * 0.9, layer);
  };
  const onNumber = (box, severity) => {
    const n = next(`n:${box.index}`);
    el("rect", {
      x: box.centre[0] - box.w / 2 - w, y: box.centre[1] - box.h / 2 - w, width: box.w + 2 * w, height: box.h + 2 * w,
      class: `finding-ring sev-${severity}`, "stroke-width": w, fill: "none",
    }, layer);
    sign(severity, [box.centre[0] + box.w / 2 + r * (0.7 + 0.9 * n), box.centre[1] - box.h / 2 - r * 0.3], r * 0.75, w * 0.9, layer);
  };
  const leg = (after, severity) => {
    const l = overlay.legs.find((x) => x.after === after);
    if (!l) return;
    const n = next(`l:${after}`);
    // A leg shorter than the two circles has nothing left to draw: the marker then follows the whole leg.
    const line = l.drawn.length > 1 ? l.drawn : l.points;
    el("polyline", {
      points: pointList(line), fill: "none", class: `finding-leg sev-${severity}`,
      "stroke-width": overlay.sizes.line * 3, "stroke-dasharray": `${overlay.sizes.line * 5} ${overlay.sizes.line * 3}`,
    }, layer);
    const m = middleOf(line);
    sign(severity, [m[0] + r * 0.9 * n, m[1] - r * 1.2], r * 0.75, w * 0.9, layer);
  };
  for (const f of findings) {
    const loc = f.location;
    const gap = f.rule === GAP_RULE;
    if (loc.kind === "control" || loc.kind === "controls") {
      for (const code of loc.kind === "control" ? [loc.code] : loc.codes) {
        const box = gap ? overlay.numbers.find((n) => n.code === String(code)) : null;
        if (box) onNumber(box, f.severity);
        else ring(code, f.severity);
      }
    } else if ((loc.kind === "leg" || loc.kind === "legs") && loc.course === course) {
      for (const after of loc.kind === "leg" ? [loc.after] : loc.afters) {
        // The gap of an own number is reported at the leg that arrives at the control: its box has index after + 1.
        const box = gap ? overlay.numbers.find((n) => n.index === after + 1) : null;
        if (box) onNumber(box, f.severity);
        else leg(after, f.severity);
      }
    }
  }
}
