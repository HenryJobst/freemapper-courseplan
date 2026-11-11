// The tool in hand: "select", "control" (place a control) or "crossing" (place a crossing point).
// The toolbar buttons carry aria-pressed; the map element carries the tool in data-tool for its cursor.

const IDS = { select: "tool-select", control: "tool-control", crossing: "tool-crossing" };

let current = "select";

export function getTool() {
  return current;
}

export function setTool(name) {
  if (!(name in IDS)) return;
  current = name;
  for (const [tool, id] of Object.entries(IDS)) {
    document.getElementById(id).setAttribute("aria-pressed", String(tool === name));
  }
  document.getElementById("map-svg").dataset.tool = name;
}

export function bind() {
  for (const [tool, id] of Object.entries(IDS)) {
    document.getElementById(id).addEventListener("click", () => setTool(tool));
  }
  setTool("select");
}
