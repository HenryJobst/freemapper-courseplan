//! The score tools that need the map: `score_suggest`, `score_draft` and `score_length` (ADR 0068).
//! Each is one `*_op` function, run by the MCP tool, by `call_json` (CLI, editor) and nowhere else.
//! Suggestions and analyses write nothing; the draft writes only with `apply: true`, through the
//! edit operations of the other score tools, so validation, saving and undo behave as for them.

use freemapper_courseplan::edit;
use freemapper_courseplan::map::{MapIndex, MapKind, map_kind_for_file};
use freemapper_courseplan::score::candidates::{
    Candidate, FORBIDDEN_LINE_MARGIN_M, SuggestContext, suggest_with,
};
use freemapper_courseplan::score::points::assign_values;
use freemapper_courseplan::score::travel::{
    CostSurface, Pace, RouteParams, SharedRoutes, TravelTime,
};
use freemapper_courseplan::score::{
    Draft, DraftFailure, DraftRequest, Hint, LengthIssue, LengthReport, ScoreError, ScoredControl,
    Tour, analyse_length, draft,
};
use freemapper_courseplan::text::{FluentValue, Localize, text};
use freemapper_courseplan_model::{
    Control, ControlCode, ControlKind, CourseKind, Event, MapPoint, ProjectFile, ScoreSettings,
    SpecialObject,
};
use serde_json::{Value, json};

use super::params::{CourseRef, ScoreDraft, ScoreSuggest};
use super::{Answer, CoursePlanServer, flag, pt};

/// Grid edge in metres of the surface that serves the legs between controls; the one the length
/// rule of `check` uses (`cell_metres` in the rule profile), so the draft and `check` agree.
const COARSE_CELL_M: f64 = 4.0;
/// Candidates `score_suggest` returns without `limit`, and the pool the draft chooses from.
const DEFAULT_LIMIT: usize = 60;
/// Most candidates one call returns.
const LIMIT_MAX: usize = 200;
/// Distance in metres between controls that WKB26 B 7.2.1 asks at print scales of 1:5000 and
/// smaller, and the half of it from 1:4000 and larger.
const MIN_DISTANCE_M_COARSE: f64 = 30.0;

/// The map of a project as the route search needs it.
struct World {
    index: MapIndex,
    scale: u32,
}

/// Both cost surfaces of a call: the fine one for the way from the start (effort), the coarse one
/// for the legs between controls.
struct Surfaces {
    fine: CostSurface,
    coarse: CostSurface,
}

/// What a draft run worked out, before anything is written.
struct Computed {
    outcome: Result<Draft, Box<DraftFailure>>,
    candidates: Vec<Candidate>,
    start: String,
    finish: String,
    existed: bool,
    /// Controls only the replaced course used; they go when the draft is applied.
    leftovers: Vec<ControlCode>,
    extent: Option<(MapPoint, MapPoint)>,
    settings: ScoreSettings,
}

/// The forbidden-route margin of the candidates in map millimetres.
fn line_margin_mm(scale: u32) -> f64 {
    FORBIDDEN_LINE_MARGIN_M * 1000.0 / f64::from(scale.max(1))
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn v(value: impl Into<FluentValue<'static>>) -> FluentValue<'static> {
    value.into()
}

/// Whether the course kind mentions the control anywhere.
fn course_uses(kind: &CourseKind, code: &ControlCode) -> bool {
    use freemapper_courseplan_model::RelayPart;
    match kind {
        CourseKind::Individual { controls } => controls.iter().any(|c| &c.code == code),
        CourseKind::Team {
            mandatory, free, ..
        } => mandatory.iter().any(|c| &c.code == code) || free.contains(code),
        CourseKind::Relay { parts, .. } => parts.iter().any(|p| match p {
            RelayPart::Common { controls } => controls.iter().any(|c| &c.code == code),
            RelayPart::Fork { variants } => variants
                .iter()
                .any(|variant| variant.iter().any(|c| &c.code == code)),
        }),
        CourseKind::Score { controls, .. } => controls.iter().any(|c| &c.code == code),
    }
}

/// The out-of-bounds and dangerous areas, which the surface blocks, and the forbidden routes, which only
/// keep candidates away (ADR 0068), that apply to `course` (all of them without a course).
fn area_specials(event: &Event, course: Option<&str>) -> Vec<SpecialObject> {
    event
        .specials
        .iter()
        .filter(|s| match s {
            SpecialObject::OutOfBoundsArea { courses, .. }
            | SpecialObject::DangerousArea { courses, .. }
            | SpecialObject::ForbiddenRoute { courses, .. } => {
                course.is_none_or(|name| courses.is_empty() || courses.iter().any(|c| c == name))
            }
            _ => false,
        })
        .cloned()
        .collect()
}

fn polygons(specials: &[SpecialObject]) -> Vec<Vec<MapPoint>> {
    specials
        .iter()
        .filter_map(|s| match s {
            SpecialObject::OutOfBoundsArea { polygon, .. }
            | SpecialObject::DangerousArea { polygon, .. } => Some(polygon.clone()),
            _ => None,
        })
        .collect()
}

fn point_json(p: &MapPoint) -> Value {
    pt(p)
}

fn tour_json(tour: &Tour, codes: &[String]) -> Value {
    json!({
        "points": tour.points,
        "time_s": round1(tour.time_s),
        "order": tour.order.iter().map(|&i| codes.get(i).cloned().unwrap_or_default()).collect::<Vec<_>>(),
    })
}

impl CoursePlanServer {
    fn locs(&self, e: &ScoreError) -> String {
        e.localize(&self.lang)
    }

    /// The map of the project, refused for a forest map (ADR 0010) and for a map without objects.
    fn score_world(&self, file: &ProjectFile) -> Result<World, String> {
        let doc = self.read_map(file)?;
        if map_kind_for_file(&doc, &file.map_path()) == MapKind::Forest {
            return Err(text(&self.lang, "error-init-forest", &[]));
        }
        let index = MapIndex::from_document(&doc);
        if index.is_empty() {
            return Err(self.locs(&ScoreError::MapEmpty));
        }
        Ok(World {
            index,
            scale: doc.georef.scale_denominator,
        })
    }

    fn surfaces(
        &self,
        world: &World,
        settings: &ScoreSettings,
        blocked: &[Vec<MapPoint>],
    ) -> Result<Surfaces, String> {
        let pace = Pace(settings.good_pace_s_per_km);
        let build = |cell_m: f64| {
            let params = RouteParams {
                cell_m,
                ..RouteParams::default()
            };
            CostSurface::build(&world.index, world.scale, blocked, pace, &params)
                .map_err(|e| self.locs(&e))
        };
        Ok(Surfaces {
            fine: build(RouteParams::default().cell_m)?,
            coarse: build(COARSE_CELL_M)?,
        })
    }

    /// Start or finish: the special object with this id, else the first one of the kind.
    fn end_point(
        &self,
        event: &Event,
        id: Option<&str>,
        finish: bool,
    ) -> Result<(String, MapPoint), String> {
        let found = event.specials.iter().find_map(|s| match (s, finish) {
            (
                SpecialObject::Start {
                    id: sid, map_mm, ..
                },
                false,
            )
            | (
                SpecialObject::Finish {
                    id: sid, map_mm, ..
                },
                true,
            ) if id.is_none_or(|want| want == sid) => Some((sid.clone(), map_mm.clone())),
            _ => None,
        });
        found.ok_or_else(|| match id {
            Some(id) => self.refused(edit::EditError::NotFound {
                what: if finish { "finish" } else { "start" },
                id: id.to_owned(),
            }),
            None => text(
                &self.lang,
                if finish {
                    "mcp-error-score-no-finish"
                } else {
                    "mcp-error-score-no-start"
                },
                &[],
            ),
        })
    }

    fn min_distance_mm(&self, given: Option<f64>, scale: u32) -> Result<f64, String> {
        match given {
            Some(d) if !(d.is_finite() && d >= 0.0) => Err(text(
                &self.lang,
                "mcp-error-score-bad-distance",
                &[("value", v(d.to_string()))],
            )),
            Some(d) => Ok(d),
            None => {
                let metres = if scale <= 4000 {
                    MIN_DISTANCE_M_COARSE / 2.0
                } else {
                    MIN_DISTANCE_M_COARSE
                };
                Ok(metres * 1000.0 / f64::from(scale.max(1)))
            }
        }
    }

    pub(super) fn score_suggest_op(&self, req: ScoreSuggest) -> Answer {
        let file = self.load(&req.project)?;
        let world = self.score_world(&file)?;
        let event = &file.event;
        let settings = event.score_settings();
        let (_, start) = self.end_point(event, req.start.as_ref().map(|n| n.0.as_str()), false)?;
        let (_, finish) = self.end_point(event, req.finish.as_ref().map(|n| n.0.as_str()), true)?;
        let specials = area_specials(event, None);
        let surfaces = self.surfaces(&world, &settings, &polygons(&specials))?;
        let existing: Vec<MapPoint> = event.controls.iter().map(|c| c.map_mm.clone()).collect();
        let min_distance_mm = self.min_distance_mm(req.min_distance_mm, world.scale)?;
        let limit = req
            .limit
            .map_or(DEFAULT_LIMIT, |l| l as usize)
            .min(LIMIT_MAX);
        let candidates = suggest_with(
            &world.index,
            &surfaces.fine,
            &surfaces.coarse,
            &SuggestContext {
                start: start.clone(),
                finish: finish.clone(),
                specials: &specials,
                min_distance_mm,
                existing: &existing,
                limit,
                line_margin_mm: line_margin_mm(world.scale),
            },
        )
        .map_err(|e| self.locs(&e))?;
        let values = assign_values(&candidates, &settings);
        let list: Vec<Value> = candidates
            .iter()
            .zip(&values)
            .map(|(c, value)| {
                json!({
                    "at": point_json(&c.at),
                    "symbol_code": c.symbol_code,
                    "route_choice": c.route_choice,
                    "difficulty": c.difficulty,
                    "effort_s": c.effort_s,
                    "value": value,
                })
            })
            .collect();
        Ok(json!({
            "start": point_json(&start),
            "finish": point_json(&finish),
            "min_distance_mm": min_distance_mm,
            "count": list.len(),
            "candidates": list,
        }))
    }

    pub(super) fn score_length_op(&self, req: CourseRef) -> Answer {
        let file = self.load(&req.project)?;
        let event = &file.event;
        let name = &req.name.0;
        let course = event
            .courses
            .iter()
            .find(|c| &c.name == name)
            .ok_or_else(|| {
                self.refused(edit::EditError::NotFound {
                    what: "course",
                    id: name.clone(),
                })
            })?;
        let CourseKind::Score {
            time_limit_min,
            controls,
            links,
            ..
        } = &course.kind
        else {
            return Err(self.refused(edit::EditError::NotScoreCourse {
                course: name.clone(),
            }));
        };
        let world = self.score_world(&file)?;
        let settings = event.score_settings();
        let (_, start) = self.end_point(event, Some(&course.start), false)?;
        let (_, finish) = self.end_point(event, Some(&course.finish), true)?;
        let scored: Vec<ScoredControl> = controls
            .iter()
            .map(|c| {
                let at = event
                    .controls
                    .iter()
                    .find(|e| e.code == c.code)
                    .map(|e| e.map_mm.clone())
                    .ok_or_else(|| {
                        self.refused(edit::EditError::NotFound {
                            what: "control",
                            id: c.code.0.clone(),
                        })
                    })?;
                Ok(ScoredControl {
                    at,
                    value: u8::try_from(c.value).unwrap_or(u8::MAX),
                })
            })
            .collect::<Result<_, String>>()?;
        let codes: Vec<String> = controls.iter().map(|c| c.code.0.clone()).collect();
        let index_of = |code: &ControlCode| controls.iter().position(|c| &c.code == code);
        let mut pairs: Vec<(usize, usize)> = Vec::new();
        for link in links {
            if let (Some(i), Some(j)) = (index_of(&link.a), index_of(&link.b))
                && i != j
                && !pairs.contains(&(i, j))
                && !pairs.contains(&(j, i))
            {
                pairs.push((i, j));
            }
        }
        let report = if scored.is_empty() {
            None
        } else {
            let specials = area_specials(event, Some(name));
            let params = RouteParams {
                cell_m: COARSE_CELL_M,
                ..RouteParams::default()
            };
            let surface = CostSurface::build(
                &world.index,
                world.scale,
                &polygons(&specials),
                Pace(settings.good_pace_s_per_km),
                &params,
            )
            .map_err(|e| self.locs(&e))?;
            let shared = SharedRoutes::new(surface);
            let good = shared.profile(1.0);
            let very_good = shared.profile(
                f64::from(settings.very_good_pace_s_per_km)
                    / f64::from(settings.good_pace_s_per_km.max(1)),
            );
            Some(
                analyse_length(
                    &scored,
                    &pairs,
                    &good,
                    &very_good,
                    start,
                    finish,
                    f64::from(*time_limit_min) * 60.0,
                    &settings,
                )
                .map_err(|e| self.locs(&e))?,
            )
        };
        let length = match report {
            Some(report) => self.length_json(&report, &codes, &settings),
            None => json!({ "all_points": 0, "ok": true, "issues": [] }),
        };
        Ok(json!({
            "name": name,
            "time_limit_min": time_limit_min,
            "controls": controls.len(),
            "length": length,
        }))
    }

    /// The words of one miss and its size: percentage points, or points for a missed control.
    fn issue_json(&self, issue: &LengthIssue, all_points: u32) -> Value {
        let t = |id: &str, args: &[(&str, FluentValue<'static>)]| text(&self.lang, id, args);
        let band = |b: &(u8, u8)| [("low", v(b.0)), ("high", v(b.1))];
        let (kind, miss, words) = match issue {
            LengthIssue::GoodBelow { percent, band: b } => {
                let miss = round1(f64::from(b.0) - percent);
                let mut a = vec![("percent", v(round1(*percent))), ("miss", v(miss))];
                a.extend(band(b));
                ("good_below", miss, t("score-miss-good-below", &a))
            }
            LengthIssue::GoodAbove { percent, band: b } => {
                let miss = round1(percent - f64::from(b.1));
                let mut a = vec![("percent", v(round1(*percent))), ("miss", v(miss))];
                a.extend(band(b));
                ("good_above", miss, t("score-miss-good-above", &a))
            }
            LengthIssue::VeryGoodMissesControls { missing } => {
                let miss = round1(100.0 * f64::from(*missing) / f64::from(all_points.max(1)));
                (
                    "very_good_misses_controls",
                    miss,
                    t(
                        "score-miss-very-good-misses",
                        &[("missing", v(*missing)), ("miss", v(miss))],
                    ),
                )
            }
            LengthIssue::VeryGoodOverLimit { percent } => {
                let miss = round1(percent - 100.0);
                (
                    "very_good_over_limit",
                    miss,
                    t(
                        "score-miss-very-good-over",
                        &[("percent", v(round1(*percent))), ("miss", v(miss))],
                    ),
                )
            }
            LengthIssue::VeryGoodTooEarly { percent, band: b } => {
                let miss = round1(f64::from(b.0) - percent);
                let mut a = vec![("percent", v(round1(*percent))), ("miss", v(miss))];
                a.extend(band(b));
                (
                    "very_good_too_early",
                    miss,
                    t("score-miss-very-good-early", &a),
                )
            }
        };
        json!({ "kind": kind, "miss": miss, "text": words })
    }

    fn length_json(&self, r: &LengthReport, codes: &[String], s: &ScoreSettings) -> Value {
        json!({
            "all_points": r.all_points,
            "ok": r.issues.is_empty(),
            "good": {
                "points": r.good.points,
                "percent": round1(r.good_percent),
                "time_s": round1(r.good.time_s),
                "order": tour_json(&r.good, codes)["order"],
                "band_percent": [s.good_band_percent.0, s.good_band_percent.1],
            },
            "very_good": {
                "points": r.very_good.points,
                "finish_percent": round1(r.very_good_finish_percent),
                "time_s": round1(r.very_good.time_s),
                "order": tour_json(&r.very_good, codes)["order"],
                "band_percent": [s.very_good_band_percent.0, s.very_good_band_percent.1],
            },
            "issues": r.issues.iter().map(|i| self.issue_json(i, r.all_points)).collect::<Vec<_>>(),
        })
    }

    /// The sentences of a failed draft: one per missed band with its size, the error if the inputs
    /// cannot work, the closest attempt, and what would help.
    fn failure_text(&self, f: &DraftFailure) -> String {
        let t = |id: &str, args: &[(&str, FluentValue<'static>)]| text(&self.lang, id, args);
        let all = f.best.report.all_points;
        let mut lines: Vec<String> = f
            .issues
            .iter()
            .map(|i| {
                self.issue_json(i, all)["text"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect();
        if let Some(e) = &f.error {
            lines.push(self.locs(e));
        }
        if !f.best.controls.is_empty() {
            lines.push(t(
                "score-draft-best",
                &[
                    ("count", v(f.best.controls.len())),
                    ("points", v(f.best.report.all_points)),
                ],
            ));
        }
        lines.push(t(
            match f.hint {
                Hint::LimitUp => "score-draft-hint-limit-up",
                Hint::LimitDown => "score-draft-hint-limit-down",
                Hint::MoreCandidates => "score-draft-hint-more-candidates",
                Hint::FasterProfile => "score-draft-hint-faster-profile",
                Hint::Impossible => "score-draft-hint-impossible",
            },
            &[],
        ));
        lines.join("\n")
    }

    /// Runs the search for `req` on the project as it is. Nothing is written.
    fn compute_draft(&self, file: &ProjectFile, req: &ScoreDraft) -> Result<Computed, String> {
        let event = &file.event;
        let name = &req.name.0;
        if req.time_limit_min == 0 {
            return Err(self.refused(edit::EditError::BadTimeLimit));
        }
        let existing_course = event.courses.iter().find(|c| &c.name == name);
        let (start_id, start, finish_id, finish) = match existing_course {
            Some(course) => {
                if !matches!(course.kind, CourseKind::Score { .. }) {
                    return Err(self.refused(edit::EditError::NotScoreCourse {
                        course: name.clone(),
                    }));
                }
                for (given, own) in [(&req.start, &course.start), (&req.finish, &course.finish)] {
                    if given.as_ref().is_some_and(|g| &g.0 != own) {
                        return Err(text(
                            &self.lang,
                            "mcp-error-score-draft-ends",
                            &[("name", v(name.clone()))],
                        ));
                    }
                }
                let (sid, s) = self.end_point(event, Some(&course.start), false)?;
                let (fid, f) = self.end_point(event, Some(&course.finish), true)?;
                (sid, s, fid, f)
            }
            None => {
                let (sid, s) =
                    self.end_point(event, req.start.as_ref().map(|n| n.0.as_str()), false)?;
                let (fid, f) =
                    self.end_point(event, req.finish.as_ref().map(|n| n.0.as_str()), true)?;
                (sid, s, fid, f)
            }
        };
        // The controls only the replaced course used leave with it, so their sites and codes are free.
        let leftovers: Vec<ControlCode> = match existing_course.map(|c| &c.kind) {
            Some(CourseKind::Score { controls, .. }) => controls
                .iter()
                .map(|c| c.code.clone())
                .filter(|code| {
                    !event
                        .courses
                        .iter()
                        .filter(|c| &c.name != name)
                        .any(|c| course_uses(&c.kind, code))
                })
                .collect(),
            _ => Vec::new(),
        };
        let kept: Vec<&Control> = event
            .controls
            .iter()
            .filter(|c| !leftovers.contains(&c.code))
            .collect();
        let taken: Vec<ControlCode> = kept.iter().map(|c| c.code.clone()).collect();
        let existing: Vec<MapPoint> = kept.iter().map(|c| c.map_mm.clone()).collect();
        let world = self.score_world(file)?;
        let settings = event.score_settings();
        let specials = area_specials(event, Some(name));
        let surfaces = self.surfaces(&world, &settings, &polygons(&specials))?;
        let min_distance_mm = self.min_distance_mm(None, world.scale)?;
        let candidates = suggest_with(
            &world.index,
            &surfaces.fine,
            &surfaces.coarse,
            &SuggestContext {
                start: start.clone(),
                finish: finish.clone(),
                specials: &specials,
                min_distance_mm,
                existing: &existing,
                limit: DEFAULT_LIMIT,
                line_margin_mm: line_margin_mm(world.scale),
            },
        )
        .map_err(|e| self.locs(&e))?;
        // The legs between controls run on the coarse surface, as `check` computes them; where that
        // cannot join start and finish (narrow passages), on the fine one.
        let surface = if surfaces
            .coarse
            .fastest(start.clone(), finish.clone())
            .is_ok()
        {
            surfaces.coarse
        } else {
            surfaces.fine
        };
        let shared = SharedRoutes::new(surface);
        let good = shared.profile(1.0);
        let very_good = shared.profile(
            f64::from(settings.very_good_pace_s_per_km)
                / f64::from(settings.good_pace_s_per_km.max(1)),
        );
        let links: Vec<(MapPoint, MapPoint)> = req
            .links
            .iter()
            .flatten()
            .map(|l| (l.a.map_point(), l.b.map_point()))
            .collect();
        let outcome = draft(&DraftRequest {
            candidates: &candidates,
            travel_good: &good,
            travel_very_good: &very_good,
            start,
            finish,
            limit_min: req.time_limit_min,
            links: &links,
            settings: &settings,
            taken: &taken,
        })
        .map_err(Box::new);
        Ok(Computed {
            outcome,
            candidates,
            start: start_id,
            finish: finish_id,
            existed: existing_course.is_some(),
            leftovers,
            extent: world.index.extent(),
            settings,
        })
    }

    /// Writes the draft as the score course `name` through the edit operations.
    fn apply_draft(
        &self,
        file: &mut ProjectFile,
        req: &ScoreDraft,
        c: &Computed,
        d: &Draft,
    ) -> Result<(), String> {
        let name = req.name.0.as_str();
        let event = &mut file.event;
        let refused = |e| self.refused(e);
        if c.existed {
            let links: Vec<_> = event
                .courses
                .iter()
                .find(|x| x.name == name)
                .and_then(|x| match &x.kind {
                    CourseKind::Score { links, .. } => Some(links.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            for link in links {
                edit::remove_score_link(event, name, &link.a, &link.b).map_err(refused)?;
            }
            edit::set_score_controls(event, name, &[]).map_err(refused)?;
            edit::set_score_course_details(
                event,
                name,
                Some(req.time_limit_min),
                edit::PenaltyChange::Keep,
            )
            .map_err(refused)?;
            for code in &c.leftovers {
                edit::remove_control(event, code).map_err(refused)?;
            }
        } else {
            edit::add_score_course(event, name, &c.start, &c.finish, req.time_limit_min, None)
                .map_err(refused)?;
        }
        for control in &d.controls {
            edit::add_control(
                event,
                c.extent.clone(),
                control.at.clone(),
                Some(control.code.clone()),
                ControlKind::Control,
            )
            .map_err(refused)?;
        }
        let inputs: Vec<edit::ScoreControlInput> = d
            .controls
            .iter()
            .map(|x| edit::ScoreControlInput {
                code: x.code.clone(),
                value: Some(u32::from(x.value)),
            })
            .collect();
        edit::set_score_controls(event, name, &inputs).map_err(refused)?;
        for &(a, b) in &d.links {
            edit::add_score_link(event, name, &d.controls[a].code, &d.controls[b].code)
                .map_err(refused)?;
        }
        Ok(())
    }

    fn draft_json(&self, req: &ScoreDraft, c: &Computed, d: &Draft, applied: bool) -> Value {
        let codes: Vec<String> = d.controls.iter().map(|x| x.code.0.clone()).collect();
        let controls: Vec<Value> = d
            .controls
            .iter()
            .map(|x| {
                let cand = &c.candidates[x.candidate];
                json!({
                    "code": x.code.0,
                    "value": x.value,
                    "at": point_json(&x.at),
                    "hard": x.hard,
                    "symbol_code": cand.symbol_code,
                    "route_choice": cand.route_choice,
                    "difficulty": cand.difficulty,
                    "effort_s": cand.effort_s,
                })
            })
            .collect();
        json!({
            "name": req.name.0,
            "time_limit_min": req.time_limit_min,
            "start": c.start,
            "finish": c.finish,
            "applied": applied,
            "replaced": c.existed && applied,
            "controls": controls,
            "links": d.links.iter().map(|&(a, b)| json!({ "a": codes[a], "b": codes[b] })).collect::<Vec<_>>(),
            "length": self.length_json(&d.report, &codes, &c.settings),
            "mix": {
                "hard_wanted": d.hard_wanted,
                "hard_actual": d.hard_actual,
                "missed_percent": d.mix_missed_percent.map(round1),
            },
        })
    }

    pub(super) fn score_draft_op(&self, req: ScoreDraft) -> Answer {
        if flag(req.apply) {
            self.edit(&req.project, |file| {
                let c = self.compute_draft(file, &req)?;
                let d = c
                    .outcome
                    .as_ref()
                    .map_err(|f| self.failure_text(f))?
                    .clone();
                self.apply_draft(file, &req, &c, &d)?;
                Ok(self.draft_json(&req, &c, &d, true))
            })
        } else {
            let file = self.load(&req.project)?;
            let c = self.compute_draft(&file, &req)?;
            let d = c.outcome.as_ref().map_err(|f| self.failure_text(f))?;
            Ok(self.draft_json(&req, &c, d, false))
        }
    }
}
