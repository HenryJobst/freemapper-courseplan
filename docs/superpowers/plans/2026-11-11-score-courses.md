# Score Courses Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The program proposes control sites on a sprint or schoolyard map, assigns points and codes, computes the
length of a score course for its time limit, builds a complete draft, checks score courses, and exports them.

**Architecture:** A new module `score` in `freemapper-courseplan` holds the pure logic: a cost surface with
alternative routes (the first part of the later route analysis), candidate scoring, point and code assignment, and
the round-trip search for the two runner profiles. The model crate gets score settings and mandatory links. CLI,
MCP and editor reach it through the same op functions (ADR 0059). The cost surface sits behind a small trait so the
full route analysis can replace it.

**Tech Stack:** Rust 1.99.1, edition 2024, no new third-party dependency (ADR 0006); the map comes from
`MapIndex` (`map.rs`), freemapper-core crates as pinned (v0.19.0); Fluent texts; plain JS for the editor panel.

**Spec:** `docs/superpowers/specs/2026-11-11-score-courses-design.md` (approved 2026-11-11). Also main design §4.3,
§5.6 (check engine), ADR 0026, 0059.

## Global Constraints

- Edition 2024, `unsafe_code = "forbid"`, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked` pass. No new third-party dependency. Never plain `cargo update`.
- Source code English; texts through Fluent files `i18n/{en,de}/courseplan.ftl`, a new message in both languages
  in the same commit; user-facing prose through `unslop` and `humanizer`; "Bundesland" in German.
- Contract tests stay named `contract_*` in the lib (at least 4 run).
- Output deterministic: fixed seed, stable order, no hash-order dependence (spec §9 ruling 8).
- Nothing writes to a project file unasked; suggestion ops return data (spec §4.5).
- Value of a control = second digit of a two-digit code, last digit of longer codes; value 0 and digit 0 are
  never used; value range default 1 to 5, at most 9 (spec §3).
- Defaults: time limit 45 or 60 min; good runner 4:10 min/km (250 s), very good 3:45 min/km (225 s); good runner
  scores 87 to 93 percent of the points, very good runner finishes with everything at 95 to 100 percent of the
  limit (spec §3, §9).
- Missing a mandatory link means disqualification *(user)*; the program records and exports the links.
- ADR numbers start at 0060. Changelog entries in `CHANGELOG.md` and `CHANGELOG.en.md` per merged task.

## File Structure

- `crates/freemapper-courseplan-model/src/types.rs`, `project_file.rs`, `validate.rs`: `ScoreSettings`, `ScoreLink`.
- `crates/freemapper-courseplan/src/score/mod.rs`: re-exports, `Pace`, `ScoreError`.
- `score/code.rs`: value and code functions. `score/travel.rs`: cost surface, `Route`. `score/speeds.toml`:
  speed table. `score/candidates.rs`: candidates and their scores. `score/points.rs`: values and codes.
  `score/tour.rs`: round-trip solver. `score/draft.rs`: set selection and draft. `score/links.rs`: link helpers.
- `crates/freemapper-courseplan/src/check/score.rs`: score rules; rule data in the built-in profile files.
- `crates/freemapper-courseplan/src/{edit.rs,iof_xml.rs}`, `crates/freemapper-courseplan-cli/src/{score_cmd.rs,
  main.rs,mcp.rs}`, editor `web/js/score.js`: ops and interfaces.

## Review Focus

- A map with no passable ground, or start and finish enclosed by blocked areas: the ops answer with one sentence,
  no panic, no empty course written (Task 3, Task 6).
- Fewer candidates than the target needs, or a limit nobody can fill: the draft reports which band is missed and by
  how much and writes nothing (Task 7).
- More controls than the code space allows (35 for values 1 to 5, 63 for 1 to 9): refused with the limit named
  (Task 5).
- Mandatory links that branch at a control, repeat, or join a control to itself: refused by the model and by
  `check` (Task 1, Task 8).
- Two runs on the same map and settings give byte-equal output on every platform (Task 7).

---

### Task 1: Score settings and mandatory links in the model

**Files:**
- Modify: `crates/freemapper-courseplan-model/src/types.rs`, `project_file.rs`, `validate.rs`, and their tests
- Create: `docs/adr/0060-score-settings-links-and-code-value.md`; modify `docs/glossary.md`, `docs/adr/README.md`

**Interfaces:**
- Produces: `pub struct ScoreLink { pub a: ControlCode, pub b: ControlCode }`; `CourseKind::Score` gains
  `#[serde(default, skip_serializing_if = "Vec::is_empty")] links: Vec<ScoreLink>`;
  `pub struct ScoreSettings { value_max: u8 (5), good_pace_s_per_km: u32 (250), very_good_pace_s_per_km: u32 (225),
  good_band_percent: (u8, u8) ((87, 93)), very_good_band_percent: (u8, u8) ((95, 100)), mix_hard_percent: u8 (40),
  seed: u64 (1) }` with `Default`; `Event.score: Option<ScoreSettings>` (table `[score]`, absent until set) and
  `Event::score_settings(&self) -> ScoreSettings` (settings or default).

- [ ] **Step 1: Failing tests** in `project_file.rs` tests: `score_links_round_trip`, `score_settings_round_trip_and_default_absent`,
  `older_files_without_links_load`; in `validate.rs`: `link_to_unknown_control_is_refused`, `link_to_itself_is_refused`,
  `duplicate_link_in_either_direction_is_refused`, `link_chain_may_not_branch` (A-B, A-C, A-D refused: more than two
  links at one control), `settings_out_of_range_are_refused` (value_max 0 or 10, band low > high, pace 0).
- [ ] **Step 2: Run** `cargo test -p freemapper-courseplan-model` and see them fail.
- [ ] **Step 3: Implement** the types and `Event::validate` rules; error variants named in `ModelError` style of the
  file. A link joins two controls of that course; at most two links meet at one control.
- [ ] **Step 4: ADR 0060** (about 25 lines) *(Claude's call, with user's marks)*: settings table, links, the
  code-value rule, disqualification for a missed link. Glossary rows: `score course`, `control value`,
  `mandatory link`, `runner profile`. Both changelogs.
- [ ] **Step 5: Run** `cargo fmt`, clippy and `cargo test --workspace --locked`; commit
  `feat(model): score settings and mandatory links`.

### Task 2: Value and code functions

**Files:** Create `crates/freemapper-courseplan/src/score/{mod.rs,code.rs}`; modify `lib.rs`.

**Interfaces:**
- Produces: `pub fn value_of_code(code: &str) -> Option<u8>` (second digit of a two-digit code, last digit of a
  longer one, `None` if not all digits, shorter than two digits or the digit is 0);
  `pub fn code_for(tens: u8, value: u8) -> String` (tens 3 to 9, value 1 to 9: `"31"`, `"49"`);
  `pub fn reads_alike_turned(a: &str, b: &str) -> bool` (6/9 reversal pairs such as 69 and 96, 68 and 89);
  `pub fn code_space(value_max: u8) -> usize` (7 tens times value_max: 35 for 5, 63 for 9).

- [ ] **Step 1: Failing tests** (`score/code.rs`): `value_of_code_follows_the_second_digit` (31 gives 1, 33 gives 3,
  49 gives 9, 55 gives 5, 100 gives `None`? no: 105 gives 5, 30 gives `None`, "A1" gives `None`),
  `code_for_builds_two_digit_codes`, `turned_pairs_are_found` (69/96 true, 31/13 false), `code_space_is_35_and_63`.
- [ ] **Step 2: Run** and see them fail; **Step 3: implement**; **Step 4: run** the module tests; **Step 5: commit**
  `feat(score): value and code functions`.

### Task 3: Cost surface with alternative routes

**Files:** Create `score/travel.rs`, `score/speeds.toml`; modify `score/mod.rs`; ADR 0061.

**Interfaces:**
- Consumes: `MapIndex::features()`, `MapFeature { symbol_code, geometry }`, the map scale
  (`doc.georef.scale_denominator`), the event's special objects (`out_of_bounds`, `dangerous_area`,
  `forbidden_route` block; map content only, not the sheet furniture, see `is_sheet_furniture`).
- Produces:
  ```rust
  pub struct Route { pub points: Vec<MapPoint>, pub length_m: f64, pub time_s: f64 }
  pub struct LegRoutes { pub routes: Vec<Route> }   // best first, at most `max_routes`
  pub trait TravelTime {
      fn legs(&self, from: MapPoint, to: MapPoint) -> Result<LegRoutes, ScoreError>;
      fn passable(&self, p: MapPoint) -> bool;
  }
  pub struct CostSurface { /* raster */ }
  impl CostSurface {
      pub fn build(index: &MapIndex, scale: u32, blocked: &[Vec<MapPoint>], profile: Pace, p: &RouteParams)
          -> Result<CostSurface, ScoreError>;
  }
  pub struct RouteParams { pub cell_m: f64, pub max_routes: usize, pub max_overlap: f64, pub max_length_ratio: f64 }
  pub struct Pace(pub u32); // seconds per km on level open ground
  ```
  Defaults `cell_m` 2.0, `max_routes` 4, `max_overlap` 0.6, `max_length_ratio` 1.25 (all settings, spec §9 ruling 5).

- [ ] **Step 1: Failing tests** with hand-built `MapIndex` fixtures (no real maps): `straight_open_field_is_a_straight_route`,
  `a_building_in_between_forces_a_detour`, `two_sides_of_a_building_give_two_alternatives` (overlap below
  `max_overlap`, both within `max_length_ratio`), `a_shifted_copy_is_not_an_alternative`,
  `blocked_special_objects_are_avoided`, `faster_ground_wins` (road beats grass by the speed table),
  `enclosed_point_gives_a_sentence_not_a_panic` (`ScoreError::Unreachable`), `same_input_same_output`.
- [ ] **Step 2: Run** and see them fail.
- [ ] **Step 3: Implement** `build` and `TravelTime for CostSurface`: raster the map objects into cells with a
  speed class from `speeds.toml` (default per symbol class, as data; look at how freemapper-core classifies
  symbols first and reuse it if it does); search with Dijkstra on the 8-neighbour grid; alternatives by the
  penalty method (raise the cost around each found route, search again, keep a route if it passes the overlap
  and length-ratio tests). The pace scales the table.
- [ ] **Step 4: Run** the module tests; a timing test on the Lübeck map is `#[ignore]` and prints the build time;
  the build for the whole map stays in seconds (record the number in ADR 0061).
- [ ] **Step 5: ADR 0061**, changelog, commit `feat(score): cost surface with alternative routes`.

### Task 4: Candidates and their scores

**Files:** Create `score/candidates.rs`; modify `score/mod.rs`; ADR 0062 (shared with Task 5).

**Interfaces:**
- Consumes: `MapIndex`, `TravelTime`, the rule profile minimum distances (`check` rules), special objects.
- Produces:
  ```rust
  pub struct Candidate { pub at: MapPoint, pub symbol_code: Option<String>, pub route_choice: f64,
      pub difficulty: f64, pub effort_s: f64 }
  pub fn suggest(index: &MapIndex, travel: &dyn TravelTime, ctx: &SuggestContext) -> Result<Vec<Candidate>, ScoreError>;
  pub struct SuggestContext<'a> { pub start: MapPoint, pub finish: MapPoint, pub specials: &'a [SpecialObject],
      pub min_distance_mm: f64, pub existing: &'a [MapPoint], pub limit: usize }
  ```
  Candidate sites are point and small area features (steps, wall corners, benches, fountains, building corners);
  the builder reads freemapper-core's object classes and takes what it offers.

- [ ] **Step 1: Failing tests** with hand-built maps: `only_clear_features_are_candidates`, `blocked_and_out_of_bounds_are_excluded`,
  `minimum_distance_to_start_finish_and_other_controls_holds`, `many_similar_routes_score_higher_than_one_obvious`
  (route_choice), `crowded_surroundings_raise_difficulty`, `candidates_are_sorted_by_route_choice_then_position`
  (stable order), `limit_cuts_the_list`.
- [ ] **Step 2: Run**, fail; **Step 3: implement** (route choice = mean over the legs to the other candidates and
  to the start of the number of routes within `max_length_ratio`, minus a term when one route is clearly shorter;
  difficulty = a weighted sum of route choice, feature clarity and neighbour density; effort = travel time from
  start). **Step 4: run**; **Step 5: commit** `feat(score): control candidates with scores`.

### Task 5: Points and codes

**Files:** Create `score/points.rs`.

**Interfaces:**
- Produces: `pub fn assign_values(candidates: &[Candidate], settings: &ScoreSettings) -> Vec<u8>` (effort and
  difficulty to a value 1..=`value_max`, a hard control brings more than an easy one at the same distance, never
  so much that only hard controls pay); `pub fn assign_codes(values: &[u8], taken: &[ControlCode], coords: &[MapPoint], rules: &CodeRules) -> Result<Vec<ControlCode>, ScoreError>`
  (tens digit per value, skipping taken codes and turned-pair clashes); `ScoreError::CodeSpaceExhausted { limit }`.

- [ ] **Step 1: Failing tests:** `harder_control_at_same_distance_gets_more_points`, `values_stay_in_range` (also with
  `value_max` 9), `values_use_all_steps_on_a_wide_set` (quantiles), `codes_carry_their_value` (31, 42, 53 give 1, 2, 3),
  `taken_codes_are_skipped`, `turned_pairs_never_meet_in_one_course`, `code_space_exhausted_names_the_limit`
  (36 controls of value 1 to 5: error with 35), `sixty_three_controls_fit_at_value_nine`.
- [ ] **Step 2 to 4:** run, implement, run. **Step 5: ADR 0062** (candidates and points), changelog, commit
  `feat(score): point and code assignment`.

### Task 6: Round trips and length analysis

**Files:** Create `score/tour.rs`, `score/links.rs`.

**Interfaces:**
- Produces:
  ```rust
  pub struct Tour { pub order: Vec<usize>, pub time_s: f64, pub points: u32 }
  pub fn best_tour(controls: &[ScoredControl], links: &[(usize, usize)], travel: &dyn TravelTime,
      start: MapPoint, finish: MapPoint, limit_s: f64) -> Result<Tour, ScoreError>;
  pub struct LengthReport { pub good: Tour, pub very_good: Tour, pub good_percent: f64, pub very_good_finish_percent: f64,
      pub all_points: u32, pub issues: Vec<LengthIssue> }
  pub enum LengthIssue { GoodBelow{percent:f64,band:(u8,u8)}, GoodAbove{..}, VeryGoodMissesControls{missing:u32},
      VeryGoodOverLimit{percent:f64}, VeryGoodTooEarly{percent:f64,band:(u8,u8)} }
  pub fn analyse_length(controls: &[ScoredControl], links: &[(usize,usize)], travel_good: &dyn TravelTime,
      travel_very_good: &dyn TravelTime, start, finish, limit_s, settings: &ScoreSettings) -> Result<LengthReport, ScoreError>;
  ```
  Leg times of a link use the better of both directions' routes; a tour must contain every link as two
  consecutive controls in either order. The solver is exact up to 12 controls, else a seeded heuristic (nearest
  insertion plus 2-opt), deterministic (`settings.seed`).

- [ ] **Step 1: Failing tests** (small synthetic cost model, no map): `best_tour_takes_the_most_points_within_the_limit`,
  `links_stay_consecutive_in_either_direction`, `a_link_that_does_not_fit_the_limit_is_reported`,
  `report_flags_good_runner_outside_87_to_93`, `report_flags_very_good_runner_over_the_limit`,
  `same_seed_same_tour`, `heuristic_matches_exact_on_small_sets` (compare on 10 controls).
- [ ] **Step 2 to 4:** run, implement, run; the 60-control timing test is `#[ignore]` and must finish in seconds.
- [ ] **Step 5: Commit** `feat(score): round trips and length analysis`.

### Task 7: The draft

**Files:** Create `score/draft.rs`; ADR 0063.

**Interfaces:**
- Produces:
  ```rust
  pub struct DraftRequest<'a> { pub candidates: &'a [Candidate], pub travel_good: &'a dyn TravelTime,
      pub travel_very_good: &'a dyn TravelTime, pub start: MapPoint, pub finish: MapPoint, pub limit_min: u32,
      pub links: &'a [(MapPoint, MapPoint)], pub settings: &'a ScoreSettings, pub taken: &'a [ControlCode] }
  pub struct Draft { pub controls: Vec<DraftControl>, pub links: Vec<(usize,usize)>, pub report: LengthReport }
  pub fn draft(req: &DraftRequest<'_>) -> Result<Draft, DraftFailure>;
  pub struct DraftFailure { pub best: Draft, pub issues: Vec<LengthIssue>, pub hint: Hint }
  ```
  `Hint` names what would help (`LimitUp`, `LimitDown`, `MoreCandidates`, `FasterProfile`). A failure writes
  nothing; the best attempt is returned only for display.

- [ ] **Step 1: Failing tests:** `a_draft_meets_both_bands` (synthetic wide map), `mix_share_bounds_the_hard_controls`,
  `effort_wins_when_mix_and_effort_conflict_and_the_report_names_it`, `not_enough_candidates_fails_with_hint`,
  `impossible_limit_fails_with_hint_and_best_attempt`, `requested_links_are_in_the_draft_and_runnable`,
  `two_runs_are_byte_equal`, `no_draft_when_start_is_enclosed`.
- [ ] **Step 2 to 4:** run, implement (select candidates by the mix and route choice, order values, codes, tour
  search, adjust the number of controls by bisection until both bands hold or give up), run.
- [ ] **Step 5: ADR 0063**, commit `feat(score): draft builder`.

### Task 8: Score checks

**Files:** Create `check/score.rs`; modify `check/mod.rs`, built-in profile data (`rules/` files of `wkb-2026`), `i18n/{en,de}/courseplan.ftl`, check tests.

**Interfaces:**
- Consumes: `analyse_length`, `value_of_code`, `reads_alike_turned`, `ScoreSettings`.
- Produces rule ids `score_value_matches_code`, `score_code_ambiguity`, `score_ring`, `score_obvious_order`,
  `score_length_bands`, `score_limit_and_penalty`, `score_link_valid`, `score_mix`, with sources (BOF B12, IRF,
  *(user)* where the spec marks it), registered like the existing rules (read `check/mod.rs`, `check/codes.rs`,
  ADR 0026 before starting).

- [ ] **Step 1: Failing tests** using the check test helpers (`check/testing.rs`): one pass and one fail case per rule;
  `link_through_out_of_bounds_is_a_finding`, `penalty_that_pays_off_is_a_warning`, `ring_around_start_and_finish_is_found`,
  `a_score_course_is_no_longer_skipped_by_the_engine`.
- [ ] **Step 2 to 4:** run, implement the rules with texts in both languages, run `cargo test --workspace --locked`.
- [ ] **Step 5: Commit** `feat(check): rules for score courses`.

### Task 9: Editing score courses (CLI and MCP parity)

**Files:** Modify `edit.rs`, `crates/freemapper-courseplan-cli/src/{edit_cmd.rs,main.rs,mcp.rs}`, `tests/query.rs`, i18n, AGENTS.md tables, manual.

**Interfaces:**
- Produces ops (same function for CLI and MCP, ADR 0059): `add_score_course(name, start, finish, time_limit_min, penalty_per_min?)` (without a penalty the default is the
  highest control value of the course, at least `value_max`; spec §5.4),
  `set_score_controls(name, codes_with_values)`, `set_control_value(name, code, value)`, `add_score_link(name, a, b)`,
  `remove_score_link(name, a, b)`, `set_score_settings(...)`. CLI: `course add --kind score`, `score control`, `score link add|remove`,
  `score settings`. Existing individual-only guards (`course <name> is not an individual course`) accept score courses
  for these ops and keep refusing team and relay.

- [ ] **Step 1: Failing tests** in the edit tests and `tests/query.rs` (CLI-vs-MCP equality as for the other ops):
  `score_course_round_trips_through_cli`, `value_must_match_code_or_is_refused`, `link_ops_enforce_the_model_rules`,
  `individual_ops_still_refuse_team_and_relay`.
- [ ] **Step 2 to 4:** run, implement, run. **Step 5: Commit** `feat: edit score courses`.

### Task 10: suggest, draft, length

**Files:** Create `crates/freemapper-courseplan-cli/src/score_cmd.rs`; modify `main.rs`, `mcp.rs`, i18n, AGENTS.md, manual.

**Interfaces:**
- Produces ops `score_suggest(project, limit?)` returning candidates with scores as JSON, `score_draft(project, name, time_limit_min, links?, apply?)`
  (applies only with `apply: true`, otherwise returns the draft), `score_length(project, name)`. CLI: `score suggest`,
  `score draft`, `score length`; `--json` equals the MCP answer. Failures print the bands missed and the hint in one
  sentence each.

- [ ] **Step 1: Failing tests** on a small fixture map in `tests/`: `suggest_returns_ranked_candidates_and_writes_nothing`
  (project file bytes unchanged), `draft_without_apply_writes_nothing`, `draft_with_apply_creates_a_checked_course`
  (`check` passes the new rules), `length_reports_the_bands`, `failed_draft_exits_with_one_and_names_the_hint`,
  `cli_and_mcp_answers_are_equal`.
- [ ] **Step 2 to 4:** run, implement, run. **Step 5: Commit** `feat: score suggest, draft and length`.

### Task 11: Editor panel

**Files:** Modify `editor/{api.rs,mod.rs,overlay.rs}`, create `editor/web/js/score.js`; modify `main.js`, i18n, editor tests.

**Interfaces:**
- Consumes: the Task 10 ops through `call_json`. Produces: a panel with "Suggest", candidate markers with scores on
  the map, accept a single candidate, "Draft" with a progress line, a length report with the bands, links drawn as
  marked legs.

- [ ] **Step 1: Failing tests** (Rust server tests and the existing browser-test style): `suggest_through_the_api_returns_markers`,
  `accepting_a_candidate_adds_a_control_and_undo_removes_it`, `draft_progress_does_not_block_other_requests`,
  `the_panel_is_inert_in_print_preview`.
- [ ] **Step 2 to 4:** run, implement (plain ES modules, no new dependency), run; check both languages in a browser.
- [ ] **Step 5: Commit** `feat(editor): score panel`.

### Task 12: Export and sheets

**Files:** Modify `iof_xml.rs` and tests, snapshots; create `score/sheet.rs`; modify `export_cmd.rs`, `mcp.rs`, i18n.

**Interfaces:**
- Produces: IOF XML `CourseControl/Score` and the links in the course data (extension or documented element, builder
  checks the schema), `pub fn score_csv(event: &Event, course: &str) -> Result<String, ScoreError>` (code, value, x_mm, y_mm),
  `pub fn evaluation_sheet(event: &Event, course: &str, lang: &Lang) -> String` (printable table; time limit, penalty, tie rule
  "faster runner wins on equal points", links with the disqualification rule). `export --csv`, `export --sheet`.

- [ ] **Step 1: Failing tests:** `score_course_is_written_to_iof_xml_and_validates_against_the_schema`, `csv_is_stable`,
  `sheet_names_tie_rule_and_links`, `ppen_export_still_skips_score_courses_and_says_so`.
- [ ] **Step 2 to 4:** run, implement, run; `xmllint` the example. **Step 5: Commit** `feat(export): score courses`.

### Task 13: Example, documentation, release

**Files:** `examples/luebeck-sprint/` (a score course next to the individual ones), README.md and README.en.md,
manual de/en, `AGENTS.md` (limits: score courses now supported; tools and commands), both changelogs,
`docs/adr/README.md`, design §4.3 and "Later" list.

- [ ] **Step 1:** Build the example with `score suggest` and `score draft` on the Lübeck map; commit the project file and
  README rows. **Step 2:** Show the proposals to the user once, note the rejected ones as test cases in a new test
  `lubeck_candidates_exclude_user_rejected_sites` *(user, spec §10)*.
- [ ] **Step 3:** Update documents in German and English through `unslop` and `humanizer`; remove "score courses are not
  built" wherever it stands (AGENTS.md "Only individual courses can be edited ..." keeps team and relay).
- [ ] **Step 4:** Final review of the whole branch, merge `--no-ff`, `cargo clean`, remove the worktree, release 0.13.0
  per CLAUDE.md (both remotes, tag, watch the release workflow).
