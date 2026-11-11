//! Mandatory links as chains. A link means two controls are run one directly after the other, either
//! way round. Links that share a control (A-B, B-C) form a chain; the solver treats a chain as one
//! unit that is run forwards or backwards (ADR 0064).

use crate::score::ScoreError;

/// Controls that must be run in this order or its reverse, or one free control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Unit {
    /// Control indices in running order. One entry for a free control.
    pub path: Vec<usize>,
    /// True when links tie the controls together, so the unit has to be in every tour.
    pub mandatory: bool,
}

/// The links without duplicates (A-B and B-A are one), in the order given.
pub(crate) fn normalise(
    n: usize,
    links: &[(usize, usize)],
) -> Result<Vec<(usize, usize)>, ScoreError> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for &(a, b) in links {
        if a >= n || b >= n || a == b {
            return Err(ScoreError::BadLink { a, b });
        }
        if !out
            .iter()
            .any(|&(x, y)| (x, y) == (a, b) || (x, y) == (b, a))
        {
            out.push((a, b));
        }
    }
    Ok(out)
}

/// Splits `n` controls into units. Chains come as they are met when walking the controls in index
/// order from the lower-indexed end; a control with more than two links, or a ring, is refused.
pub(crate) fn build_units(n: usize, links: &[(usize, usize)]) -> Result<Vec<Unit>, ScoreError> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(a, b) in links {
        adj[a].push(b);
        adj[b].push(a);
    }
    if let Some(control) = (0..n).find(|&c| adj[c].len() > 2) {
        return Err(ScoreError::LinksNotAChain { control });
    }
    let mut seen = vec![false; n];
    let mut units = Vec::new();
    for c in 0..n {
        if seen[c] || adj[c].len() > 1 {
            continue;
        }
        let mut path = vec![c];
        seen[c] = true;
        let mut cur = c;
        while let Some(&next) = adj[cur].iter().find(|&&x| !seen[x]) {
            seen[next] = true;
            path.push(next);
            cur = next;
        }
        units.push(Unit {
            mandatory: path.len() > 1,
            path,
        });
    }
    if let Some(control) = (0..n).find(|&c| !seen[c]) {
        return Err(ScoreError::LinksNotAChain { control });
    }
    Ok(units)
}

/// The link that fits worst, when no tour holds all links within the limit: the unit with the
/// largest cost for start, unit and finish alone, and in it the link whose nearer end has the
/// longest detour through start and finish. `solo(unit)` and `detour(control)` come from the
/// caller's time matrix.
pub(crate) fn least_fitting_link(
    units: &[Unit],
    links: &[(usize, usize)],
    solo: impl Fn(usize) -> f64,
    detour: impl Fn(usize) -> f64,
) -> (usize, usize) {
    let mut worst: Option<(usize, f64)> = None;
    for (k, _) in units.iter().enumerate().filter(|(_, u)| u.mandatory) {
        let cost = solo(k);
        if worst.is_none_or(|(_, w)| cost > w) {
            worst = Some((k, cost));
        }
    }
    let Some((k, _)) = worst else {
        return links.first().copied().unwrap_or((0, 0));
    };
    let path = &units[k].path;
    let mut best: Option<((usize, usize), f64)> = None;
    for &(a, b) in links {
        if path.contains(&a) && path.contains(&b) {
            let score = detour(a).min(detour(b));
            if best.is_none_or(|(_, s)| score > s) {
                best = Some(((a, b), score));
            }
        }
    }
    best.map_or((path[0], path[1]), |(l, _)| l)
}
