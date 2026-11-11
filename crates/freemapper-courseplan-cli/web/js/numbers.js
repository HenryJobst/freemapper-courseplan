// Control numbers: the box a drag moves, the operation a drop sends, and the panel with the placing options.
//
// The one place that maps the overlay's number numbering onto the operations: `numbers[].index` counts the
// stops of the course from 1 (crossing points counted, not numbered). `set_course_number` takes the same number
// as `after`; `set_control_number` names the control by its code.

import { t } from "./texts.js";

// The options of the next drop and of "Place numbers"; the panel and the page share this object.
export const options = { forAll: false, overwrite: false };

// Whether a map point (mm) lies within a number box, grown by `reach` mm. The nearest box wins.
export function numberAt(numbers, at, reach) {
  let best = null;
  for (const n of numbers) {
    const dx = Math.abs(at[0] - n.centre[0]);
    const dy = Math.abs(at[1] - n.centre[1]);
    if (dx > n.w / 2 + reach || dy > n.h / 2 + reach) continue;
    const d = Math.hypot(dx, dy);
    if (!best || d < best.d) best = { number: n, d };
  }
  return best?.number ?? null;
}

// The query of api/number-at for the number `n` of `course`, its box centred on `centre`.
export function numberAtQuery(course, n, centre) {
  const q = new URLSearchParams({ course, index: String(n.index), x: String(centre[0]), y: String(centre[1]) });
  return `api/number-at?${q}`;
}

// The operation of a drop: { tool, body }. With `forAll`, the central number of the control; else the course's.
export function dropOperation(course, n, found, forAll) {
  const placement = { direction: found.direction, gap_mm: found.gap_mm };
  if (forAll) return { tool: "set_control_number", body: { code: n.code, ...placement } };
  return { tool: "set_course_number", body: { name: course, after: n.index, ...placement } };
}

export const placeBody = (course, overwrite) => ({ name: course, overwrite });

function checkRow(host, id, labelId, key) {
  const label = document.createElement("label");
  label.className = "check";
  const box = document.createElement("input");
  box.type = "checkbox";
  box.id = id;
  box.checked = options[key];
  box.addEventListener("change", () => {
    options[key] = box.checked;
  });
  label.append(box, ` ${t(labelId)}`);
  host.append(label);
}

// The number controls in the properties of a course or a number: "for all courses", "Place numbers" and
// "also replace placed numbers". actions.placeNumbers() runs the operation.
export function panel(host, actions) {
  const group = document.createElement("div");
  group.className = "numbers-panel";
  group.setAttribute("role", "group");
  group.setAttribute("aria-label", t("editor-numbers-heading"));
  const heading = document.createElement("h3");
  heading.textContent = t("editor-numbers-heading");
  group.append(heading);
  const hint = document.createElement("p");
  hint.className = "hint";
  hint.textContent = t("editor-numbers-hint");
  group.append(hint);
  checkRow(group, "prop-number-all", "editor-numbers-for-all", "forAll");
  const place = document.createElement("button");
  place.type = "button";
  place.id = "prop-place-numbers";
  place.textContent = t("editor-numbers-place");
  place.addEventListener("click", () => actions.placeNumbers());
  group.append(place);
  checkRow(group, "prop-place-overwrite", "editor-numbers-overwrite", "overwrite");
  host.append(group);
}
