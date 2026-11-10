//! Matching of the objects of the three views (D7, D14).
//!
//! Each object belongs to at most one [`Triple`]. The order is: theirs to base, ours to base, then what is
//! new on both sides. Purple Pen ids count only where the base holds the same id for an object of the same
//! kind, so an id that is new, or comes from another file, never pairs two objects (D7).

use freemapper_courseplan_model::{Control, ControlKind, Course, MapPoint, SpecialObject};

use super::view::Views;

/// One object in up to three views; the indices point into the vectors of the respective `PpenImport`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triple {
    pub base: Option<usize>,
    pub ours: Option<usize>,
    pub theirs: Option<usize>,
}

/// The matched controls, special objects (starts and finishes among them) and courses.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Matching {
    pub controls: Vec<Triple>,
    pub specials: Vec<Triple>,
    pub courses: Vec<Triple>,
}

type Shape = Vec<(i64, i64)>;

/// What matching needs to know about an object.
struct Item {
    id: Option<u32>,
    /// Objects of different classes never match (normal control or crossing point, start, finish, …).
    class: u8,
    /// Code, special object id or course name.
    name: String,
    shape: Shape,
    /// New objects pair by shape, not by name (starts, finishes, crossing points, special objects).
    by_shape: bool,
    /// Without a base, the k-th unpaired object of the class pairs with the k-th on the other side.
    ordered: bool,
}

fn quantize(points: &[MapPoint]) -> Shape {
    points
        .iter()
        .map(|p| ((p.x * 1000.0).round() as i64, (p.y * 1000.0).round() as i64))
        .collect()
}

fn control_item(c: &Control) -> Item {
    let crossing = c.kind == ControlKind::CrossingPoint;
    Item {
        id: c.ppen_id,
        class: u8::from(crossing),
        name: c.code.0.clone(),
        shape: quantize(std::slice::from_ref(&c.map_mm)),
        by_shape: crossing,
        ordered: crossing,
    }
}

fn special_item(s: &SpecialObject) -> Item {
    let (class, shape) = match s {
        SpecialObject::Start { map_mm, .. } => (0, quantize(std::slice::from_ref(map_mm))),
        SpecialObject::Finish { map_mm, .. } => (1, quantize(std::slice::from_ref(map_mm))),
        SpecialObject::MarkedRoute { points, .. } => (2, quantize(points)),
        SpecialObject::OutOfBoundsArea { polygon, .. } => (3, quantize(polygon)),
        SpecialObject::ForbiddenRoute { points, .. } => (4, quantize(points)),
        SpecialObject::DangerousArea { polygon, .. } => (5, quantize(polygon)),
        SpecialObject::FirstAid { map_mm, .. } => (6, quantize(std::slice::from_ref(map_mm))),
        SpecialObject::Water { map_mm, .. } => (7, quantize(std::slice::from_ref(map_mm))),
    };
    Item {
        id: s.ppen_id(),
        class,
        name: s.id().to_owned(),
        shape,
        by_shape: true,
        ordered: class <= 1,
    }
}

fn course_item(c: &Course) -> Item {
    Item {
        id: c.ppen_id,
        class: 0,
        name: c.name.clone(),
        shape: Vec::new(),
        by_shape: false,
        ordered: false,
    }
}

/// Matches `side` to the base objects whose slot is still free: by id first, then by name.
fn claim(side: &[Item], base: &[Item], slots: &mut [Option<usize>]) {
    let mut done = vec![false; side.len()];
    for pass in 0..2 {
        for (i, s) in side.iter().enumerate() {
            if done[i] {
                continue;
            }
            let found = (0..base.len()).find(|&b| {
                slots[b].is_none()
                    && base[b].class == s.class
                    && if pass == 0 {
                        s.id.is_some() && base[b].id == s.id
                    } else {
                        base[b].name == s.name
                    }
            });
            if let Some(b) = found {
                slots[b] = Some(i);
                done[i] = true;
            }
        }
    }
}

/// Pairs the unmatched objects of the two sides with each other by `same`; returns the pairs.
fn pair(
    ours: &[Item],
    theirs: &[Item],
    free_ours: &mut Vec<usize>,
    free_theirs: &mut Vec<usize>,
    same: impl Fn(&Item, &Item) -> bool,
) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for &o in free_ours.clone().iter() {
        let found = free_theirs
            .iter()
            .position(|&t| ours[o].class == theirs[t].class && same(&ours[o], &theirs[t]));
        if let Some(pos) = found {
            pairs.push((o, free_theirs.remove(pos)));
            free_ours.retain(|&x| x != o);
        }
    }
    pairs
}

fn match_items(base: &[Item], ours: &[Item], theirs: &[Item], has_base: bool) -> Vec<Triple> {
    let mut theirs_of = vec![None; base.len()];
    let mut ours_of = vec![None; base.len()];
    claim(theirs, base, &mut theirs_of);
    claim(ours, base, &mut ours_of);
    let mut out: Vec<Triple> = (0..base.len())
        .map(|b| Triple {
            base: Some(b),
            ours: ours_of[b],
            theirs: theirs_of[b],
        })
        .collect();

    let mut free_ours: Vec<usize> = (0..ours.len())
        .filter(|i| !ours_of.contains(&Some(*i)))
        .collect();
    let mut free_theirs: Vec<usize> = (0..theirs.len())
        .filter(|i| !theirs_of.contains(&Some(*i)))
        .collect();
    let mut pairs = Vec::new();
    if !has_base {
        // The project's ids belong to the linked file; they count where theirs holds the same id.
        pairs.extend(pair(
            ours,
            theirs,
            &mut free_ours,
            &mut free_theirs,
            |o, t| o.id.is_some() && o.id == t.id,
        ));
    }
    pairs.extend(pair(
        ours,
        theirs,
        &mut free_ours,
        &mut free_theirs,
        |o, t| !o.by_shape && o.name == t.name,
    ));
    pairs.extend(pair(
        ours,
        theirs,
        &mut free_ours,
        &mut free_theirs,
        |o, t| o.by_shape && o.shape == t.shape,
    ));
    if !has_base {
        // The order fallback of the export (`kth_renames`), per class and only when the counts agree.
        for class in 0..=u8::MAX {
            let o_k: Vec<usize> = free_ours
                .iter()
                .copied()
                .filter(|&i| ours[i].class == class && ours[i].ordered)
                .collect();
            let t_k: Vec<usize> = free_theirs
                .iter()
                .copied()
                .filter(|&i| theirs[i].class == class && theirs[i].ordered)
                .collect();
            if !o_k.is_empty() && o_k.len() == t_k.len() {
                for (&o, &t) in o_k.iter().zip(&t_k) {
                    pairs.push((o, t));
                    free_ours.retain(|&x| x != o);
                    free_theirs.retain(|&x| x != t);
                }
            }
        }
    }
    pairs.sort_unstable();
    out.extend(pairs.into_iter().map(|(o, t)| Triple {
        base: None,
        ours: Some(o),
        theirs: Some(t),
    }));
    out.extend(free_ours.into_iter().map(|o| Triple {
        base: None,
        ours: Some(o),
        theirs: None,
    }));
    out.extend(free_theirs.into_iter().map(|t| Triple {
        base: None,
        ours: None,
        theirs: Some(t),
    }));
    out
}

/// Matches the objects of the three views. `has_base` is false for a sync without a base copy; then the
/// project's Purple Pen ids that the linked file holds too count, and starts, finishes and crossing points
/// pair by their order (D14).
pub fn match_sides(v: &Views, has_base: bool) -> Matching {
    let items = |i: &super::super::ppen_import::PpenImport| {
        (
            i.controls.iter().map(control_item).collect::<Vec<_>>(),
            i.specials.iter().map(special_item).collect::<Vec<_>>(),
            i.courses.iter().map(course_item).collect::<Vec<_>>(),
        )
    };
    let (bc, bs, bk) = items(&v.base);
    let (oc, os, ok) = items(&v.ours);
    let (tc, ts, tk) = items(&v.theirs);
    Matching {
        controls: match_items(&bc, &oc, &tc, has_base),
        specials: match_items(&bs, &os, &ts, has_base),
        courses: match_items(&bk, &ok, &tk, has_base),
    }
}

#[cfg(test)]
mod tests {
    use freemapper_courseplan_model::{ControlCode, ControlDescription};

    use super::*;
    use crate::ppen_import::PpenImport;

    fn control(code: &str, id: Option<u32>, kind: ControlKind) -> Control {
        Control {
            code: ControlCode(code.to_owned()),
            kind,
            map_mm: MapPoint { x: 1.0, y: 2.0 },
            description: ControlDescription::default(),
            number: None,
            ppen_id: id,
        }
    }

    fn start(id: &str, ppen_id: Option<u32>, x: f64) -> SpecialObject {
        SpecialObject::Start {
            id: id.to_owned(),
            ppen_id,
            map_mm: MapPoint { x, y: 0.0 },
        }
    }

    fn import(controls: Vec<Control>, specials: Vec<SpecialObject>) -> PpenImport {
        PpenImport {
            controls,
            specials,
            courses: Vec::new(),
            notes: Vec::new(),
        }
    }

    fn views(base: PpenImport, ours: PpenImport, theirs: PpenImport) -> Views {
        Views {
            base,
            ours,
            theirs,
            ours_ids: Default::default(),
            skipped: Vec::new(),
        }
    }

    fn n(code: &str, id: u32) -> Control {
        control(code, Some(id), ControlKind::Control)
    }

    #[test]
    fn theirs_matches_base_by_id_across_a_recode() {
        let v = views(
            import(vec![n("31", 5)], vec![]),
            import(vec![], vec![]),
            import(vec![n("41", 5)], vec![]),
        );
        let m = match_sides(&v, true);
        assert_eq!(
            m.controls,
            vec![Triple {
                base: Some(0),
                ours: None,
                theirs: Some(0)
            }]
        );
    }

    #[test]
    fn id_match_needs_the_same_kind() {
        let v = views(
            import(
                vec![control("CROSS1", Some(7), ControlKind::CrossingPoint)],
                vec![],
            ),
            import(vec![], vec![]),
            import(vec![n("45", 7)], vec![]),
        );
        let m = match_sides(&v, true);
        assert_eq!(m.controls.len(), 2);
        assert!(
            m.controls
                .iter()
                .all(|t| t.base.is_some() != t.theirs.is_some())
        );
    }

    #[test]
    fn ids_outside_the_base_are_not_trusted() {
        let v = views(
            import(vec![n("31", 1)], vec![]),
            import(vec![n("31", 1), n("40", 12)], vec![]),
            import(vec![n("31", 1), n("39", 12)], vec![]),
        );
        let m = match_sides(&v, true);
        assert_eq!(m.controls.len(), 3);
        assert!(m.controls.contains(&Triple {
            base: None,
            ours: Some(1),
            theirs: None
        }));
        assert!(m.controls.contains(&Triple {
            base: None,
            ours: None,
            theirs: Some(1)
        }));
    }

    #[test]
    fn new_on_both_sides_match_by_code() {
        let v = views(
            import(vec![], vec![]),
            import(vec![n("38", 3)], vec![]),
            import(vec![n("38", 9)], vec![]),
        );
        let m = match_sides(&v, true);
        assert_eq!(
            m.controls,
            vec![Triple {
                base: None,
                ours: Some(0),
                theirs: Some(0)
            }]
        );
    }

    #[test]
    fn new_starts_match_by_shape() {
        let v = views(
            import(vec![], vec![]),
            import(vec![], vec![start("S1", Some(1), 5.0)]),
            import(vec![], vec![start("STA7", Some(9), 5.0)]),
        );
        let m = match_sides(&v, true);
        assert_eq!(
            m.specials,
            vec![Triple {
                base: None,
                ours: Some(0),
                theirs: Some(0)
            }]
        );
    }

    #[test]
    fn first_sync_uses_project_ids_found_in_theirs() {
        let v = views(
            import(vec![], vec![]),
            import(vec![], vec![start("S1", Some(1), 5.0)]),
            import(vec![], vec![start("STA1", Some(1), 8.0)]),
        );
        let m = match_sides(&v, false);
        assert_eq!(
            m.specials,
            vec![Triple {
                base: None,
                ours: Some(0),
                theirs: Some(0)
            }]
        );
        // With a base, the same ids are not trusted and the moved start does not pair.
        assert_eq!(match_sides(&v, true).specials.len(), 2);
    }
}
