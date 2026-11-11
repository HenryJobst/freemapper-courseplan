// The tool in hand: "select", "control" (place a control), "crossing" (place a crossing point) or "course"
// (course mode: clicks build the selected course).
// The toolbar buttons carry aria-pressed; the map element carries the tool in data-tool for its cursor.

const IDS = { select: "tool-select", control: "tool-control", crossing: "tool-crossing", course: "tool-course" };

let listener = () => {};

// fn(name) runs after the tool changed.
export function onChange(fn) {
  listener = fn;
}

let current = "select";

export function getTool() {
  return current;
}

export function setTool(name) {
  if (!(name in IDS)) return;
  const changed = name !== current;
  current = name;
  for (const [tool, id] of Object.entries(IDS)) {
    document.getElementById(id).setAttribute("aria-pressed", String(tool === name));
  }
  document.getElementById("map-svg").dataset.tool = name;
  if (changed) listener(name);
}

export function bind() {
  for (const [tool, id] of Object.entries(IDS)) {
    document.getElementById(id).addEventListener("click", () => setTool(tool));
  }
  setTool("select");
}
