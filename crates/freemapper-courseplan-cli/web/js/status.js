// The status bar: a message area (aria-live), the map's file, scale and kind, and the cursor in map mm.

import { t } from "./texts.js";

const message = document.getElementById("status-message");
const fields = {
  map: document.getElementById("status-map"),
  scale: document.getElementById("status-scale"),
  kind: document.getElementById("status-kind"),
  cursor: document.getElementById("status-cursor"),
};

// Shows a message by its text id; with raw = true the text is shown as it is (server answers already localized).
export function say(idOrText, raw = false) {
  message.textContent = raw ? idOrText : t(idOrText);
}

export function setMap(path, scale, kind) {
  fields.map.textContent = path.split(/[\\/]/).pop();
  fields.map.title = path;
  fields.scale.textContent = `1:${scale}`;
  fields.kind.textContent = t(`editor-mapkind-${kind}`);
}

export function setCursor(point) {
  fields.cursor.textContent = point ? `${point[0].toFixed(2)}; ${point[1].toFixed(2)}` : "";
}
