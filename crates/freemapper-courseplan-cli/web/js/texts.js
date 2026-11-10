// The page's own words come from the server in the process language (api/texts).
// An element names its text by id: data-text (content), data-label (aria-label), data-title (title).

import { get } from "./api.js";

let catalog = {};

export function t(id) {
  return catalog[id] ?? id;
}

export async function loadTexts() {
  const answer = await get("api/texts");
  catalog = answer.texts;
  document.documentElement.lang = answer.lang;
  apply(document);
}

export function apply(root) {
  for (const [attribute, set] of [
    ["data-text", (el, text) => { el.textContent = text; }],
    ["data-label", (el, text) => el.setAttribute("aria-label", text)],
    ["data-title", (el, text) => el.setAttribute("title", text)],
  ]) {
    for (const el of root.querySelectorAll(`[${attribute}]`)) {
      set(el, t(el.getAttribute(attribute)));
    }
  }
  document.title = t("editor-page-title");
}
