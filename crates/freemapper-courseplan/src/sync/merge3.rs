//! Pure three-way merge helpers: the pick rule for one field (D5) and diff3 over stop lists (D6).

/// Outcome of the three-way rule for one optional value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    /// Ours, theirs and base agree.
    Unchanged,
    /// Only ours changed: keep ours.
    Ours,
    /// Only theirs changed: take theirs.
    Theirs,
    /// Both sides made the same change.
    Both,
    /// Both sides changed differently.
    Conflict,
}

/// Applies the three-way rule to one field. `None` means absent (deleted or not yet created).
pub fn pick<T: PartialEq>(base: Option<&T>, ours: Option<&T>, theirs: Option<&T>) -> Pick {
    if ours == theirs {
        if ours == base {
            Pick::Unchanged
        } else {
            Pick::Both
        }
    } else if ours == base {
        Pick::Theirs
    } else if theirs == base {
        Pick::Ours
    } else {
        Pick::Conflict
    }
}

/// Result of [`diff3`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Merged3<T> {
    /// The merged list.
    Clean(Vec<T>),
    /// Overlapping changes that differ.
    Conflict,
}

/// For each base index, the index it matches in `other` (longest common subsequence). On a tie the
/// base advances first, so the alignment is deterministic.
fn align<T: Eq>(base: &[T], other: &[T]) -> Vec<Option<usize>> {
    let (n, m) = (base.len(), other.len());
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if base[i] == other[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut out = vec![None; n];
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if base[i] == other[j] {
            out[i] = Some(j);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

/// Three-way merge of identity sequences against their common base.
///
/// A clean result is accepted only if it is stable: merging it again with either side gives it back.
/// With repeated identities the LCS alignment does not guarantee that, and D13's crash recovery re-merges
/// a written result. An unstable merge is reported as a conflict.
pub fn diff3<T: Eq + Clone>(base: &[T], ours: &[T], theirs: &[T]) -> Merged3<T> {
    match merge_once(base, ours, theirs) {
        Merged3::Clean(m)
            if merge_once(base, ours, &m) == Merged3::Clean(m.clone())
                && merge_once(base, &m, theirs) == Merged3::Clean(m.clone()) =>
        {
            Merged3::Clean(m)
        }
        _ => Merged3::Conflict,
    }
}

fn merge_once<T: Eq + Clone>(base: &[T], ours: &[T], theirs: &[T]) -> Merged3<T> {
    let ao = align(base, ours);
    let at = align(base, theirs);
    let mut out: Vec<T> = Vec::new();
    // Start of the open chunk in base, ours and theirs.
    let (mut bs, mut os, mut ts) = (0, 0, 0);
    // Stable points, then a sentinel one past the end of each list.
    let stable = (0..base.len())
        .filter_map(|i| Some((i, ao[i]?, at[i]?)))
        .chain(std::iter::once((base.len(), ours.len(), theirs.len())));
    for (bi, oi, ti) in stable {
        let (b, o, t) = (&base[bs..bi], &ours[os..oi], &theirs[ts..ti]);
        if o == b {
            out.extend_from_slice(t);
        } else if t == b || o == t {
            out.extend_from_slice(o);
        } else {
            return Merged3::Conflict;
        }
        if bi < base.len() {
            out.push(base[bi].clone());
        }
        (bs, os, ts) = (bi + 1, oi + 1, ti + 1);
    }
    Merged3::Clean(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const S: u32 = 1000;
    const F: u32 = 2000;

    fn clean(v: &[u32]) -> Merged3<u32> {
        Merged3::Clean(v.to_vec())
    }

    fn swap(p: Pick) -> Pick {
        match p {
            Pick::Ours => Pick::Theirs,
            Pick::Theirs => Pick::Ours,
            other => other,
        }
    }

    fn values() -> [Option<u32>; 4] {
        [None, Some(1), Some(2), Some(3)]
    }

    #[test]
    fn pick_table() {
        for b in values() {
            for o in values() {
                for t in values() {
                    let want = if o == t {
                        if o == b { Pick::Unchanged } else { Pick::Both }
                    } else if o == b {
                        Pick::Theirs
                    } else if t == b {
                        Pick::Ours
                    } else {
                        Pick::Conflict
                    };
                    assert_eq!(
                        pick(b.as_ref(), o.as_ref(), t.as_ref()),
                        want,
                        "{b:?} {o:?} {t:?}"
                    );
                }
            }
        }
        assert_eq!(pick(Some(&1), None, Some(&2)), Pick::Conflict);
        assert_eq!(pick(None, Some(&1), None), Pick::Ours);
        assert_eq!(pick(Some(&1), None, None), Pick::Both);
        assert_eq!(pick(Some(&1), Some(&1), Some(&1)), Pick::Unchanged);
    }

    #[test]
    fn pick_is_symmetric() {
        for b in values() {
            for o in values() {
                for t in values() {
                    assert_eq!(
                        pick(b.as_ref(), t.as_ref(), o.as_ref()),
                        swap(pick(b.as_ref(), o.as_ref(), t.as_ref()))
                    );
                }
            }
        }
    }

    #[test]
    fn inserts_at_different_places_both_land() {
        let base = [S, 31, 32, 33, F];
        let ours = [S, 31, 40, 32, 33, F];
        let theirs = [S, 31, 32, 41, 33, F];
        assert_eq!(
            diff3(&base, &ours, &theirs),
            clean(&[S, 31, 40, 32, 41, 33, F])
        );
    }

    #[test]
    fn inserts_at_same_place_conflict() {
        let base = [S, 31, 32, 33, F];
        let ours = [S, 31, 40, 32, 33, F];
        let theirs = [S, 31, 41, 32, 33, F];
        assert_eq!(diff3(&base, &ours, &theirs), Merged3::Conflict);
    }

    #[test]
    fn same_insert_lands_once() {
        let base = [S, 31, 32, 33, F];
        let both = [S, 31, 40, 32, 33, F];
        assert_eq!(diff3(&base, &both, &both), clean(&both));
    }

    #[test]
    fn delete_and_far_insert_merge() {
        let base = [S, 31, 32, 33, F];
        let ours = [S, 31, 33, F];
        let theirs = [S, 31, 32, 33, 41, F];
        assert_eq!(diff3(&base, &ours, &theirs), clean(&[S, 31, 33, 41, F]));
    }

    #[test]
    fn delete_vs_replace_conflict() {
        let base = [S, 31, 32, 33, F];
        let ours = [S, 31, 33, F];
        let theirs = [S, 31, 42, 33, F];
        assert_eq!(diff3(&base, &ours, &theirs), Merged3::Conflict);
    }

    #[test]
    fn butterfly_visits_merge_by_position() {
        let base = [S, 50, 31, 50, 32, 50, F];
        let ours = [S, 50, 31, 33, 50, 32, 50, F];
        let theirs = [S, 50, 31, 50, 34, 50, F];
        assert_eq!(
            diff3(&base, &ours, &theirs),
            clean(&[S, 50, 31, 33, 50, 34, 50, F])
        );
    }

    fn lists() -> Vec<Vec<u8>> {
        let mut all = vec![vec![]];
        let mut frontier = vec![vec![]];
        for _ in 0..3 {
            let mut next = Vec::new();
            for l in &frontier {
                for x in *b"abc" {
                    let mut n: Vec<u8> = l.clone();
                    n.push(x);
                    next.push(n);
                }
            }
            all.extend(next.iter().cloned());
            frontier = next;
        }
        all
    }

    #[test]
    fn diff3_exhaustive_properties() {
        let all = lists();
        for b in &all {
            for x in &all {
                assert_eq!(diff3(b, b, x), Merged3::Clean(x.clone()), "{b:?} {x:?}");
                assert_eq!(diff3(b, x, b), Merged3::Clean(x.clone()), "{b:?} {x:?}");
                assert_eq!(diff3(b, x, x), Merged3::Clean(x.clone()), "{b:?} {x:?}");
                for t in &all {
                    assert_eq!(diff3(b, x, t), diff3(b, t, x), "{b:?} {x:?} {t:?}");
                    // Fixed point: re-merging a clean result with either side gives it back (D13).
                    if let Merged3::Clean(m) = diff3(b, x, t) {
                        let same = Merged3::Clean(m.clone());
                        assert_eq!(diff3(b, x, &m), same, "{b:?} {x:?} {t:?}");
                        assert_eq!(diff3(b, &m, t), same, "{b:?} {x:?} {t:?}");
                    }
                }
            }
        }
    }
}
