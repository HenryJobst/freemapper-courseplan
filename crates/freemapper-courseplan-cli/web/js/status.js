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

const banner = document.getElementById("banner");

// A refusal or a conflict stays above the map until it is dismissed or the next change succeeds.
// The container is a role="alert" region, so a screen reader reads the text when it appears.
export function showBanner(text) {
  const box = document.createElement("div");
  box.className = "banner-box";
  const words = document.createElement("span");
  words.textContent = text;
  const close = document.createElement("button");
  close.type = "button";
  close.textContent = t("editor-banner-dismiss");
  close.addEventListener("click", clearBanner);
  box.append(words, close);
  banner.replaceChildren(box);
}

export function clearBanner() {
  banner.replaceChildren();
}

// The severities in the order they are listed, with the glyph that tells them apart without colour.
export const SEVERITIES = ["error", "warning", "hint"];
export const SEVERITY_ICON = { error: "\u2716", warning: "\u25B2", hint: "\u2139" };
const COUNT_LABEL = { error: "editor-findings-errors", warning: "editor-findings-warnings", hint: "editor-findings-hints" };

// The counts of the check in the status bar: a glyph, the word and the number for each severity.
// `summary` is the check's summary ({ errors, warnings, hints }) or null while there is none.
export function setFindings(summary) {
  const counts = { error: summary?.errors, warning: summary?.warnings, hint: summary?.hints };
  for (const severity of SEVERITIES) {
    const box = document.getElementById(`count-${severity}`);
    box.className = `count sev-${severity}`;
    const icon = document.createElement("span");
    icon.setAttribute("aria-hidden", "true");
    icon.textContent = SEVERITY_ICON[severity];
    const words = document.createElement("span");
    words.textContent = ` ${t(COUNT_LABEL[severity])}: ${counts[severity] ?? "\u2013"}`;
    box.replaceChildren(icon, words);
  }
}
