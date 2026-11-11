//! Dijkstra on the 8-neighbour grid and the penalty method for alternatives.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::RouteParams;
use super::raster::Grid;

/// Largest corridor, in metres on the ground (ADR 0061). The corridor is 12 percent of the best
/// route, at least two cells.
const CORRIDOR_MAX_M: f64 = 10.0;
const CORRIDOR_SHARE: f64 = 0.12;
/// Extra cost factor inside the corridor of a route already found.
const PENALTY: f32 = 2.0;
/// A candidate is searched at most this many times per wanted route.
const ATTEMPTS_PER_ROUTE: usize = 3;

/// Neighbour offsets in the fixed order of the search: the four straight ones, then the diagonals.
const STEPS: [(i32, i32); 8] = [
    (0, -1),
    (1, 0),
    (0, 1),
    (-1, 0),
    (1, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
];

struct Entry {
    cost: f64,
    cell: u32,
}
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entry {
    /// Reversed, so the heap pops the cheapest entry; equal costs pop the lower cell first.
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .total_cmp(&self.cost)
            .then_with(|| other.cell.cmp(&self.cell))
    }
}

pub(super) struct Searcher<'g> {
    grid: &'g Grid,
    cell_m: f64,
    dist: Vec<f64>,
    prev: Vec<u32>,
}

impl<'g> Searcher<'g> {
    pub(super) fn new(grid: &'g Grid, cell_m: f64) -> Searcher<'g> {
        let n = grid.factor.len();
        Searcher {
            grid,
            cell_m,
            dist: vec![f64::INFINITY; n],
            prev: vec![u32::MAX; n],
        }
    }

    fn passable(&self, col: i64, row: i64) -> Option<usize> {
        let (w, h) = (self.grid.width as i64, self.grid.height as i64);
        if col < 0 || row < 0 || col >= w || row >= h {
            return None;
        }
        let cell = (row * w + col) as usize;
        self.grid.factor[cell].is_finite().then_some(cell)
    }

    /// Cost of one step between neighbouring cells, in factor-metres. `pen` multiplies it.
    fn step_cost(&self, a: usize, b: usize, diagonal: bool, pen: Option<&[f32]>) -> f64 {
        let len = if diagonal {
            std::f64::consts::SQRT_2
        } else {
            1.0
        } * self.cell_m;
        let mut factor = 0.5 * f64::from(self.grid.factor[a] + self.grid.factor[b]);
        if let Some(pen) = pen {
            factor *= 0.5 * f64::from(pen[a] + pen[b]);
        }
        len * factor
    }

    /// The cheapest cell path from `s` to `t`, both included.
    pub(super) fn shortest(
        &mut self,
        s: usize,
        t: usize,
        pen: Option<&[f32]>,
    ) -> Option<Vec<usize>> {
        self.dist.fill(f64::INFINITY);
        self.prev.fill(u32::MAX);
        let w = self.grid.width as i64;
        let mut heap = BinaryHeap::new();
        self.dist[s] = 0.0;
        heap.push(Entry {
            cost: 0.0,
            cell: s as u32,
        });
        while let Some(Entry { cost, cell }) = heap.pop() {
            let cell = cell as usize;
            if cost > self.dist[cell] {
                continue;
            }
            if cell == t {
                break;
            }
            let (col, row) = ((cell as i64) % w, (cell as i64) / w);
            for (dx, dy) in STEPS {
                let diagonal = dx != 0 && dy != 0;
                let Some(next) = self.passable(col + i64::from(dx), row + i64::from(dy)) else {
                    continue;
                };
                // No cutting a corner of a blocked cell.
                if diagonal
                    && (self.passable(col + i64::from(dx), row).is_none()
                        || self.passable(col, row + i64::from(dy)).is_none())
                {
                    continue;
                }
                let next_cost = cost + self.step_cost(cell, next, diagonal, pen);
                if next_cost < self.dist[next] {
                    self.dist[next] = next_cost;
                    self.prev[next] = cell as u32;
                    heap.push(Entry {
                        cost: next_cost,
                        cell: next as u32,
                    });
                }
            }
        }
        if self.dist[t].is_infinite() {
            return None;
        }
        let mut path = vec![t];
        while *path.last().expect("path starts at t") != s {
            path.push(self.prev[*path.last().expect("path starts at t")] as usize);
        }
        path.reverse();
        Some(path)
    }

    /// Length of a cell path in metres.
    pub(super) fn length_m(&self, path: &[usize]) -> f64 {
        path.windows(2).map(|p| self.step_len(p[0], p[1])).sum()
    }

    fn step_len(&self, a: usize, b: usize) -> f64 {
        let w = self.grid.width;
        let diagonal = a % w != b % w && a / w != b / w;
        if diagonal {
            std::f64::consts::SQRT_2 * self.cell_m
        } else {
            self.cell_m
        }
    }

    /// Cost of a cell path without penalties, in factor-metres.
    pub(super) fn cost(&self, path: &[usize]) -> f64 {
        path.windows(2)
            .map(|p| {
                let diagonal = self.step_len(p[0], p[1]) > self.cell_m;
                self.step_cost(p[0], p[1], diagonal, None)
            })
            .sum()
    }

    /// The fastest path and up to `max_routes - 1` clearly different ones, best first.
    pub(super) fn alternatives(
        &mut self,
        s: usize,
        t: usize,
        params: &RouteParams,
    ) -> Option<Vec<Vec<usize>>> {
        let best = self.shortest(s, t, None)?;
        let limit = self.length_m(&best) * params.max_length_ratio;
        let mut found = vec![best];
        if params.max_routes <= 1 || s == t {
            return Some(found);
        }
        let n = self.grid.factor.len();
        let corridor_m = (CORRIDOR_SHARE * self.length_m(&found[0]))
            .min(CORRIDOR_MAX_M)
            .max(2.0 * self.cell_m);
        let w = (corridor_m / self.cell_m).ceil() as i64;
        let mut pen = vec![1.0f32; n];
        let mut stamp = Marks {
            stamp: vec![0; n],
            now: 0,
        };
        let mut last = found[0].clone();
        for _ in 0..params.max_routes * ATTEMPTS_PER_ROUTE {
            if found.len() >= params.max_routes {
                break;
            }
            self.penalise(&mut pen, &last, w);
            let Some(candidate) = self.shortest(s, t, Some(&pen)) else {
                break;
            };
            // A candidate that is too long is dropped, but its penalty makes room for the next one.
            let distinct = self.length_m(&candidate) <= limit
                && found.iter().all(|accepted| {
                    self.is_distinct(&mut stamp, accepted, &candidate, w, params.max_overlap)
                });
            if distinct {
                found.push(candidate.clone());
            }
            last = candidate;
        }
        Some(found)
    }

    fn cells_within(&self, cell: usize, radius: i64, mut f: impl FnMut(usize)) {
        let w = self.grid.width as i64;
        let (col, row) = ((cell as i64) % w, (cell as i64) / w);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let (c, r) = (col + dx, row + dy);
                    if c >= 0 && r >= 0 && c < w && r < self.grid.height as i64 {
                        f((r * w + c) as usize);
                    }
                }
            }
        }
    }

    fn penalise(&self, pen: &mut [f32], path: &[usize], w: i64) {
        for &cell in path {
            self.cells_within(cell, w, |c| pen[c] = 1.0 + PENALTY);
        }
    }

    /// A candidate is distinct from an accepted route when at most `max_overlap` of its cells lie
    /// within `w` cells of that route, not counting the cells within `w` cells of the start and the
    /// end (every route shares those), and some cell lies beyond `2 * w` cells from it.
    fn is_distinct(
        &self,
        marks: &mut Marks,
        accepted: &[usize],
        candidate: &[usize],
        w: i64,
        max_overlap: f64,
    ) -> bool {
        marks.now += 1;
        let near = marks.now;
        for &cell in accepted {
            self.cells_within(cell, w, |c| marks.stamp[c] = near);
        }
        let width = self.grid.width as i64;
        let (first, last) = (candidate[0] as i64, candidate[candidate.len() - 1] as i64);
        let close = |a: i64, b: i64| {
            let (dx, dy) = (a % width - b % width, a / width - b / width);
            dx * dx + dy * dy <= w * w
        };
        let (mut counted, mut shared) = (0usize, 0usize);
        for &c in candidate {
            if close(c as i64, first) || close(c as i64, last) {
                continue;
            }
            counted += 1;
            shared += usize::from(marks.stamp[c] == near);
        }
        if counted > 0 && shared as f64 / counted as f64 > max_overlap {
            return false;
        }
        marks.now += 1;
        let wide = marks.now;
        for &cell in accepted {
            self.cells_within(cell, 2 * w, |c| marks.stamp[c] = wide);
        }
        candidate.iter().any(|&c| marks.stamp[c] != wide)
    }
}

struct Marks {
    stamp: Vec<u32>,
    now: u32,
}
