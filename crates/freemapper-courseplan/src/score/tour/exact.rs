//! Exact round trips by dynamic programming over subsets of units. A state is the set of units
//! visited and the unit run last with its direction; the table holds the shortest time. The
//! subsets are bitmasks, so this only runs on at most [`super::EXACT_MAX`] units.

use super::{EPS, Problem, Seq};

/// The best tour over all units; `None` when no tour holds every link within the limit.
pub(super) fn solve(p: &Problem) -> Option<Seq> {
    let all: Vec<usize> = (0..p.units.len()).collect();
    solve_units(p, &all)
}

/// The best tour over the units in `active` only (at most `EXACT_MAX` of them), with every
/// mandatory one among them in the tour.
pub(super) fn solve_units(p: &Problem, active: &[usize]) -> Option<Seq> {
    debug_assert!(active.len() <= super::EXACT_MAX);
    let u = active.len();
    let states = 2 * u;
    let full = 1usize << u;
    let mut dp = vec![f64::INFINITY; full * states.max(1)];
    let mut prev = vec![u32::MAX; full * states.max(1)];
    let fits = |t: f64| t.is_finite() && t <= p.limit + EPS;
    let dirs = |k: usize| 0..=usize::from(p.reversible(active[k]));

    for k in 0..u {
        for o in dirs(k) {
            let cost = p.dist(p.start(), p.entry(active[k], o == 1)) + p.internal[active[k]];
            if fits(cost) {
                dp[(1 << k) * states + k * 2 + o] = cost;
            }
        }
    }
    for mask in 1..full {
        for s in 0..states {
            let val = dp[mask * states + s];
            if !val.is_finite() {
                continue;
            }
            let (k, o) = (s / 2, s % 2 == 1);
            for j in (0..u).filter(|j| mask & (1 << j) == 0) {
                for o2 in dirs(j) {
                    let nd = val
                        + p.dist(p.exit(active[k], o), p.entry(active[j], o2 == 1))
                        + p.internal[active[j]];
                    let ni = (mask | 1 << j) * states + j * 2 + o2;
                    if fits(nd) && nd < dp[ni] {
                        dp[ni] = nd;
                        prev[ni] = s as u32;
                    }
                }
            }
        }
    }

    let mandatory = (0..u)
        .filter(|&k| p.units[active[k]].mandatory)
        .fold(0usize, |m, k| m | 1 << k);
    let mut best: Best = None;
    if mandatory == 0 && fits(p.direct()) {
        consider(p, &mut best, 0, p.direct(), Vec::new());
    }
    for mask in (1..full).filter(|m| m & mandatory == mandatory) {
        let points: u32 = (0..u)
            .filter(|k| mask & (1 << k) != 0)
            .map(|k| p.unit_points(active[k]))
            .sum();
        for s in 0..states {
            let val = dp[mask * states + s];
            if !val.is_finite() {
                continue;
            }
            let total = val + p.dist(p.exit(active[s / 2], s % 2 == 1), p.finish());
            if !fits(total) {
                continue;
            }
            if best
                .as_ref()
                .is_some_and(|(bp, bt, _, _)| points < *bp || (points == *bp && total > bt + EPS))
            {
                continue;
            }
            let mut seq = Vec::new();
            let (mut m, mut st) = (mask, s);
            loop {
                seq.push((active[st / 2], st % 2 == 1));
                let pv = prev[m * states + st];
                m &= !(1 << (st / 2));
                if pv == u32::MAX {
                    break;
                }
                st = pv as usize;
            }
            seq.reverse();
            consider(p, &mut best, points, total, seq);
        }
    }
    best.map(|(_, _, _, seq)| seq)
}

/// Points, time, running order and sequence of the best tour so far.
type Best = Option<(u32, f64, Vec<usize>, Seq)>;

/// Keeps the candidate when it has more points, or the same points in less time, or the same
/// time with the lower running order. The last holds among tours that end in different states
/// of one table, not among every tour of equal points and time.
fn consider(p: &Problem, best: &mut Best, points: u32, time: f64, seq: Seq) {
    let flat = p.flatten(&seq);
    let better = match best {
        None => true,
        Some((bp, bt, bf, _)) => {
            points > *bp
                || (points == *bp && time < *bt - EPS)
                || (points == *bp && (time - *bt).abs() <= EPS && flat < *bf)
        }
    };
    if better {
        *best = Some((points, time, flat, seq));
    }
}
