// The map view: which part of the page (in page millimetres) the <svg> shows. The page box and the mapping from
// map to page millimetres come from the server (api/map/view); this module only pans and zooms.
// The scale is kept in pixels per millimetre and the top left corner in page mm; the viewBox follows.

const MIN_SCALE = 0.05;
const MAX_SCALE = 400;
const MAX_IMAGE_SIDE_PX = 6000;
const SETTLE_MS = 150;

export function createView(svgRoot, mapping) {
  const events = new EventTarget();
  let scale = 3;
  let left = 0;
  let top = 0;
  let current = mapping;

  function size() {
    const box = svgRoot.getBoundingClientRect();
    return { w: Math.max(box.width, 1), h: Math.max(box.height, 1), box };
  }

  function apply() {
    const { w, h } = size();
    svgRoot.setAttribute("viewBox", `${left} ${top} ${w / scale} ${h / scale}`);
    events.dispatchEvent(new Event("viewchange"));
  }

  const view = {
    addEventListener: (...args) => events.addEventListener(...args),
    get scale() { return scale; },
    get mapping() { return current; },
    setMapping(next) { current = next; },
    toPage: (p) => [current.k * p[0] + current.dx, current.k * p[1] + current.dy],
    toMap: (x, y) => [(x - current.dx) / current.k, (y - current.dy) / current.k],
    // A pointer event to map mm.
    screenToMap(evt) {
      const { box } = size();
      return view.toMap(left + (evt.clientX - box.left) / scale, top + (evt.clientY - box.top) / scale);
    },
    visible() {
      const { w, h } = size();
      return { x0: left, y0: top, x1: left + w / scale, y1: top + h / scale };
    },
    // Shows a page rectangle [x0, y0, x1, y1] with a small border, centred.
    fit(rect) {
      const { w, h } = size();
      const rw = Math.max(rect[2] - rect[0], 1e-6);
      const rh = Math.max(rect[3] - rect[1], 1e-6);
      scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, Math.min(w / (rw * 1.04), h / (rh * 1.04))));
      left = (rect[0] + rect[2]) / 2 - w / scale / 2;
      top = (rect[1] + rect[3]) / 2 - h / scale / 2;
      apply();
    },
    // Zooms by the factor f, keeping the page point `at` ([x, y]) where it is; without it the centre stays.
    zoomBy(f, at) {
      const { w, h } = size();
      const anchor = at ?? [left + w / scale / 2, top + h / scale / 2];
      const next = Math.min(MAX_SCALE, Math.max(MIN_SCALE, scale * f));
      left = anchor[0] - ((anchor[0] - left) * scale) / next;
      top = anchor[1] - ((anchor[1] - top) * scale) / next;
      scale = next;
      apply();
    },
    // Moves the view by page millimetres.
    panBy(dx, dy) {
      left += dx;
      top += dy;
      apply();
    },
    // The SVG transform that puts map mm into page mm.
    transform: () => `matrix(${current.k} 0 0 ${current.k} ${current.dx} ${current.dy})`,
    refresh: apply,
    // The page point under a pointer event.
    screenToPage(evt) {
      const { box } = size();
      return [left + (evt.clientX - box.left) / scale, top + (evt.clientY - box.top) / scale];
    },
  };
  new ResizeObserver(apply).observe(svgRoot);
  return view;
}

const SVG_NS = "http://www.w3.org/2000/svg";

// The map layer: one <image> of api/map.svg for the visible part plus 50 % margin on every side, requested
// 150 ms after the view stops moving, never while it moves. The old image stays until the new one has loaded.
// The image is an <image> element, never inline SVG: the clip ids of one SVG would clash with the page's own.
// When the server refuses the section (413), `onError` gets its localized text; a later success gets null.
export function bindMapLayer(group, view, onError = () => {}) {
  let timer = null;
  let ticket = 0;

  async function load(force) {
    const mine = ++ticket;
    const v = view.visible();
    const mx = (v.x1 - v.x0) / 2;
    const my = (v.y1 - v.y0) / 2;
    const rect = { x0: v.x0 - mx, y0: v.y0 - my, x1: v.x1 + mx, y1: v.y1 + my };
    const side = Math.max(rect.x1 - rect.x0, rect.y1 - rect.y0);
    const zoom = Math.min(view.scale * (window.devicePixelRatio || 1), MAX_IMAGE_SIDE_PX / side);
    const q = [rect.x0, rect.y0, rect.x1, rect.y1, zoom].map((n) => Number(n.toFixed(4)));
    const url = `api/map.svg?x0=${q[0]}&y0=${q[1]}&x1=${q[2]}&y1=${q[3]}&zoom=${q[4]}`;
    // One request: the body is the SVG (shown through a blob URL) or the server's reason for refusing.
    let blobUrl;
    try {
      const answer = await fetch(url, force ? { cache: "reload" } : {});
      if (mine !== ticket) return;
      if (!answer.ok) {
        onError(await answer.text());
        return;
      }
      blobUrl = URL.createObjectURL(await answer.blob());
    } catch {
      // The server is gone; the poll reports that.
      return;
    }
    if (mine !== ticket) {
      URL.revokeObjectURL(blobUrl);
      return;
    }
    const image = document.createElementNS(SVG_NS, "image");
    image.setAttribute("x", rect.x0);
    image.setAttribute("y", rect.y0);
    image.setAttribute("width", rect.x1 - rect.x0);
    image.setAttribute("height", rect.y1 - rect.y0);
    image.setAttribute("preserveAspectRatio", "none");
    image.addEventListener("load", () => {
      URL.revokeObjectURL(blobUrl);
      if (mine !== ticket) {
        image.remove();
        return;
      }
      for (const old of [...group.children]) if (old !== image) old.remove();
      onError(null);
    });
    image.addEventListener("error", () => {
      URL.revokeObjectURL(blobUrl);
      image.remove();
    });
    image.setAttribute("href", blobUrl);
    group.append(image);
  }

  view.addEventListener("viewchange", () => {
    clearTimeout(timer);
    timer = setTimeout(() => load(false), SETTLE_MS);
  });
  return { refresh: (force = false) => load(force) };
}
