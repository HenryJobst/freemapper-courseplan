//! Seeded heuristic for many controls: cheapest insertion by points per second, 2-opt, and a
//! remove-and-refill pass, repeated from several seeded orders; the best result wins.

use super::{EPS, Problem, Seq};

/// Restarts after the plain run.
const RESTARTS: usize = 12;
/// Passes of remove-and-refill per run.
const IMPROVE_PASSES: usize = 3;

/// A small xorshift; no dependency, same sequence for the same seed on every platform.
struct Rng(u32);

impl Rng {
    fn new(seed: u32) -> Rng {
        Rng(if seed == 0 { 0x9E37_79B9 } else { seed })
    }
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn unit(&mut self) -> f64 {
        f64::from(self.next()) / f64::from(u32::MAX)
    }
    fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.next() as usize % (i + 1);
            v.swap(i, j);
        }
    }
}

pub(super) fn solve(p: &Problem, seed: u32) -> Option<Seq> {
    let mut rng = Rng::new(seed);
    let mut best: Option<(u32, f64, Seq)> = None;
    for run in 0..=RESTARTS {
        let Some(seq) = build(p, &mut rng, run > 0) else {
            continue;
        };
        let (pts, time) = (points(p, &seq), p.eval(&seq));
        if best
            .as_ref()
            .is_none_or(|(bp, bt, _)| better(pts, time, *bp, *bt))
        {
            best = Some((pts, time, seq));
        }
    }
    let (_, _, mut seq) = best?;
    polish(p, &mut seq, &mut rng);
    Some(seq)
}

/// Takes one or two optional units out, puts another in, refills, and keeps what scores more,
/// until nothing improves.
fn polish(p: &Problem, seq: &mut Seq, rng: &mut Rng) {
    loop {
        let mut improved = false;
        let members: Vec<usize> = seq
            .iter()
            .map(|&(u, _)| u)
            .filter(|&u| !p.units[u].mandatory)
            .collect();
        let mut removals: Vec<Vec<usize>> = members.iter().map(|&u| vec![u]).collect();
        for (i, &u) in members.iter().enumerate() {
            for &w in &members[i + 1..] {
                removals.push(vec![u, w]);
            }
        }
        for removal in removals {
            if removal.iter().any(|&u| !seq.iter().any(|&(k, _)| k == u)) {
                continue;
            }
            for v in 0..p.units.len() {
                if removal.contains(&v) || p.units[v].mandatory || seq.iter().any(|&(k, _)| k == v)
                {
                    continue;
                }
                let mut trial: Seq = seq
                    .iter()
                    .copied()
                    .filter(|&(k, _)| !removal.contains(&k))
                    .collect();
                let (pos, rev, _) = best_insertion(p, &trial, v);
                trial.insert(pos, (v, rev));
                two_opt(p, &mut trial);
                if p.eval(&trial) > p.limit + EPS {
                    continue;
                }
                fill(p, &mut trial, rng, false);
                if better(
                    points(p, &trial),
                    p.eval(&trial),
                    points(p, seq),
                    p.eval(seq),
                ) {
                    *seq = trial;
                    improved = true;
                    break;
                }
            }
            if improved {
                break;
            }
        }
        if !improved {
            return;
        }
    }
}

fn points(p: &Problem, seq: &Seq) -> u32 {
    seq.iter().map(|&(u, _)| p.unit_points(u)).sum()
}

fn better(pts: u32, time: f64, bp: u32, bt: f64) -> bool {
    pts > bp || (pts == bp && time < bt - EPS)
}

fn build(p: &Problem, rng: &mut Rng, noisy: bool) -> Option<Seq> {
    let mut seq: Seq = Vec::new();
    let mut must: Vec<usize> = (0..p.units.len())
        .filter(|&k| p.units[k].mandatory)
        .collect();
    if noisy {
        rng.shuffle(&mut must);
    }
    for k in must {
        let (pos, rev, _) = best_insertion(p, &seq, k);
        seq.insert(pos, (k, rev));
    }
    two_opt(p, &mut seq);
    if p.eval(&seq) > p.limit + EPS {
        return None;
    }
    fill(p, &mut seq, rng, noisy);
    for _ in 0..IMPROVE_PASSES {
        let mut improved = false;
        let members: Vec<usize> = seq
            .iter()
            .map(|&(u, _)| u)
            .filter(|&u| !p.units[u].mandatory)
            .collect();
        for u in members {
            let Some(at) = seq.iter().position(|&(k, _)| k == u) else {
                continue;
            };
            let mut trial = seq.clone();
            trial.remove(at);
            fill(p, &mut trial, rng, false);
            if better(
                points(p, &trial),
                p.eval(&trial),
                points(p, &seq),
                p.eval(&seq),
            ) {
                seq = trial;
                improved = true;
            }
        }
        if !improved {
            break;
        }
    }
    Some(seq)
}

/// Where unit `k` costs the least time: position, direction, added seconds.
fn best_insertion(p: &Problem, seq: &Seq, k: usize) -> (usize, bool, f64) {
    let mut best = (0, false, f64::INFINITY);
    for pos in 0..=seq.len() {
        let prev = if pos == 0 {
            p.start()
        } else {
            p.exit(seq[pos - 1].0, seq[pos - 1].1)
        };
        let next = if pos == seq.len() {
            p.finish()
        } else {
            p.entry(seq[pos].0, seq[pos].1)
        };
        for rev in [false, true] {
            if rev && !p.reversible(k) {
                continue;
            }
            let added =
                p.dist(prev, p.entry(k, rev)) + p.internal[k] + p.dist(p.exit(k, rev), next)
                    - p.dist(prev, next);
            if added < best.2 {
                best = (pos, rev, added);
            }
        }
    }
    best
}

/// Adds optional units while some fit, the one with the most points per added second first.
fn fill(p: &Problem, seq: &mut Seq, rng: &mut Rng, noisy: bool) {
    loop {
        let now = p.eval(seq);
        let mut pick: Option<(usize, usize, bool, f64)> = None;
        for k in 0..p.units.len() {
            if p.units[k].mandatory || seq.iter().any(|&(u, _)| u == k) {
                continue;
            }
            let (pos, rev, added) = best_insertion(p, seq, k);
            if !added.is_finite() || now + added > p.limit + EPS {
                continue;
            }
            let mut score = f64::from(p.unit_points(k)) / added.max(1e-6);
            if noisy {
                score *= 1.0 + 2.0 * rng.unit();
            }
            if pick.is_none_or(|(.., s)| score > s) {
                pick = Some((k, pos, rev, score));
            }
        }
        let Some((k, pos, rev, _)) = pick else { return };
        seq.insert(pos, (k, rev));
        two_opt(p, seq);
    }
}

/// Reverses stretches of the tour and moves single units while that saves time. The matrix is symmetric, so only the two
/// edges at the ends of a stretch change.
fn two_opt(p: &Problem, seq: &mut Seq) {
    loop {
        let mut improved = false;
        for i in 0..seq.len() {
            for j in i..seq.len() {
                let prev = if i == 0 {
                    p.start()
                } else {
                    p.exit(seq[i - 1].0, seq[i - 1].1)
                };
                let next = if j + 1 == seq.len() {
                    p.finish()
                } else {
                    p.entry(seq[j + 1].0, seq[j + 1].1)
                };
                let (first, last) = (seq[i], seq[j]);
                let old =
                    p.dist(prev, p.entry(first.0, first.1)) + p.dist(p.exit(last.0, last.1), next);
                let new =
                    p.dist(prev, p.exit(last.0, last.1)) + p.dist(p.entry(first.0, first.1), next);
                if new < old - 1e-9 {
                    seq[i..=j].reverse();
                    for e in &mut seq[i..=j] {
                        if p.reversible(e.0) {
                            e.1 = !e.1;
                        }
                    }
                    improved = true;
                }
            }
        }
        // Or-opt: move one unit to its cheapest place.
        for i in 0..seq.len() {
            let now = p.eval(seq);
            let mut trial = seq.clone();
            let (k, _) = trial.remove(i);
            let (pos, rev, _) = best_insertion(p, &trial, k);
            trial.insert(pos, (k, rev));
            if p.eval(&trial) < now - 1e-9 {
                *seq = trial;
                improved = true;
            }
        }
        if !improved {
            return;
        }
    }
}
