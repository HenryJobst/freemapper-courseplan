// Bends of a leg. Pure helpers, no DOM.
//
// The one place that maps the overlay's leg numbering onto the operation: the overlay's `legs[].after` is the
// number of course controls before the leg's origin, 0 for the leg from the start. `set_leg` takes the same
// number as `after` (counted from 1, crossing points included) and wants `from_start` for the leg from the start.

export function legBody(course, after, bends, taped) {
  const target = after === 0 ? { from_start: true } : { after };
  return { name: course, ...target, bends, taped };
}

export function distanceToSegment(p, a, b) {
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  const len2 = dx * dx + dy * dy;
  const u = len2 === 0 ? 0 : Math.max(0, Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2));
  return Math.hypot(p[0] - (a[0] + u * dx), p[1] - (a[1] + u * dy));
}

// The segment of a polyline (`points` as [x, y]) nearest to `at`: { index, d }. A bend put there gets this index
// in the bend list, because points[0] is the leg's origin and the bends follow.
export function nearestSegment(points, at) {
  let best = null;
  for (let i = 1; i < points.length; i++) {
    const d = distanceToSegment(at, points[i - 1], points[i]);
    if (!best || d < best.d) best = { index: i - 1, d };
  }
  return best;
}

export const bendNear = (bends, at, reach) => {
  let best = null;
  bends.forEach((b, index) => {
    const d = Math.hypot(b.x - at[0], b.y - at[1]);
    if (d <= reach && (!best || d < best.d)) best = { index, d };
  });
  return best?.index ?? null;
};

export const withInserted = (bends, index, bend) => [...bends.slice(0, index), bend, ...bends.slice(index)];
export const withMoved = (bends, index, bend) => bends.map((b, i) => (i === index ? bend : b));
export const withoutBend = (bends, index) => bends.filter((_, i) => i !== index);
export const sameBends = (a, b) => a.length === b.length && a.every((p, i) => p.x === b[i].x && p.y === b[i].y);

// Puts new bends into a leg of the overlay: the polyline follows. The clipped drawn part is the whole line until
// the page loads the overlay again.
export function setLegBends(leg, bends) {
  const first = leg.points[0];
  const last = leg.points[leg.points.length - 1];
  leg.bends = bends;
  leg.points = [first, ...bends.map((b) => [b.x, b.y]), last];
  leg.drawn = leg.points;
}
