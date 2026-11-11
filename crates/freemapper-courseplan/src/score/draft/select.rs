//! The order in which candidates enter the draft: forced ones first, then spatially spread, with the
//! easy/hard mix kept in every prefix.

use freemapper_courseplan_model::MapPoint;

use crate::score::candidates::Candidate;

/// Difficulty from which a candidate counts as hard for the mix.
pub const HARD_FROM: f64 = 0.5;

pub(super) fn is_hard(c: &Candidate) -> bool {
    c.difficulty >= HARD_FROM
}

fn dist(a: &MapPoint, b: &MapPoint) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// All of `pool` in the order they should be taken: `forced` first, then always the candidate
/// farthest from start, finish and everything taken so far (ties go to the lower index). With a
/// `hard_share` (0.0 to 1.0) every prefix holds about that share of hard candidates; a pool that
/// runs dry is filled from the other one. Deterministic: no hashing, index order breaks ties.
pub(super) fn priority_order(
    cands: &[Candidate],
    pool: &[usize],
    forced: &[usize],
    start: &MapPoint,
    finish: &MapPoint,
    hard_share: Option<f64>,
) -> Vec<usize> {
    let mut order: Vec<usize> = forced.to_vec();
    let mut left: Vec<usize> = pool
        .iter()
        .copied()
        .filter(|i| !forced.contains(i))
        .collect();
    let mut near: Vec<f64> = Vec::new();
    let anchor_dist = |i: usize| dist(&cands[i].at, start).min(dist(&cands[i].at, finish));
    let mut hard_count = forced.iter().filter(|&&i| is_hard(&cands[i])).count();
    // Distance of each remaining candidate to the nearest thing taken so far.
    for &i in &left {
        let mut d = anchor_dist(i);
        for &f in forced {
            d = d.min(dist(&cands[i].at, &cands[f].at));
        }
        near.push(d);
    }
    while !left.is_empty() {
        let want_hard = hard_share.map(|share| {
            let target = (share * (order.len() + 1) as f64).round() as usize;
            hard_count < target
        });
        let pick = |only: Option<bool>| -> Option<usize> {
            let mut best: Option<usize> = None;
            for (pos, &i) in left.iter().enumerate() {
                if only.is_some_and(|h| h != is_hard(&cands[i])) {
                    continue;
                }
                if best.is_none_or(|b| near[pos].total_cmp(&near[b]).is_gt()) {
                    best = Some(pos);
                }
            }
            best
        };
        let pos = pick(want_hard)
            .or_else(|| pick(None))
            .expect("left is not empty");
        let i = left.remove(pos);
        near.remove(pos);
        if is_hard(&cands[i]) {
            hard_count += 1;
        }
        for (p, &j) in left.iter().enumerate() {
            near[p] = near[p].min(dist(&cands[i].at, &cands[j].at));
        }
        order.push(i);
    }
    order
}
