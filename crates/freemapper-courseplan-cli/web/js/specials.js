// Special objects: the kinds the special tool can draw, the proposed ids and the operation bodies.
// Pure helpers; the gestures live in main.js. No operation moves a special object: delete it and draw it again.

// shape: "point" (one click), "line" or "area" (clicks, finished with Enter or a double click).
// `tool` is the name the operations use for the kind; `min` the fewest points the model accepts.
// A forbidden route may have a single point: it is drawn as a cross.
export const KINDS = [
  { kind: "start", tool: "start", shape: "point", min: 1 },
  { kind: "finish", tool: "finish", shape: "point", min: 1 },
  { kind: "first_aid", tool: "first_aid", shape: "point", min: 1 },
  { kind: "water", tool: "water", shape: "point", min: 1 },
  { kind: "marked_route", tool: "marked_route", shape: "line", min: 2 },
  { kind: "forbidden_route", tool: "forbidden_route", shape: "line", min: 1 },
  { kind: "out_of_bounds_area", tool: "out_of_bounds", shape: "area", min: 3 },
  { kind: "dangerous_area", tool: "dangerous_area", shape: "area", min: 3 },
];

// A letter or sign drawn next to each kind, so no kind is told apart by colour or dashes alone.
export const CUES = {
  start: "S",
  finish: "F",
  marked_route: "M",
  forbidden_route: "×",
  out_of_bounds_area: "OB",
  dangerous_area: "!",
  first_aid: "+",
  water: "W",
};

export function info(kind) {
  return KINDS.find((k) => k.kind === kind) ?? null;
}

// The first free id `<kind>-<n>`, n from 1, with underscores of the kind written as hyphens.
export function nextId(kind, taken) {
  const prefix = kind.replaceAll("_", "-");
  const used = new Set(taken.map(String));
  let n = 1;
  while (used.has(`${prefix}-${n}`)) n += 1;
  return `${prefix}-${n}`;
}

const round2 = (v) => Math.round(v * 100) / 100;

// The body of add_special for `points` (map mm pairs): `at` for a point kind, `points` for the others.
export function addBody(kind, id, points) {
  const entry = info(kind);
  const list = points.map((p) => ({ x: round2(p[0]), y: round2(p[1]) }));
  return entry.shape === "point" ? { kind: entry.tool, id, at: list[0] } : { kind: entry.tool, id, points: list };
}

// Whether `points` are enough to finish a drawing of this kind.
export function enough(kind, points) {
  return points.length >= info(kind).min;
}

// The body of set_special_courses for the ticked course names: none ticked means all courses.
export function coursesBody(id, ticked) {
  return ticked.length === 0 ? { id, all: true } : { id, courses: ticked };
}

// The middle of a point list, where the cue of a drawn kind is put.
export function middle(points) {
  const n = points.length;
  return [points.reduce((a, p) => a + p[0], 0) / n, points.reduce((a, p) => a + p[1], 0) / n];
}
