# .ppen Writer (M3b) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper-courseplan export PROJECT --ppen FILE` (and the MCP tool `export_ppen`) writes the project's courses as a Purple Pen event: a new file from scratch, or a patch of an existing `.ppen` that keeps everything the project does not hold, byte for byte where nothing changed.

**Architecture:** The writer lives in freemapper-core, crate `freemapper-course`, as module `ppen/write.rs` beside the reader (agreed with freemapper-print and the user, 2026-11-09). It is format-faithful like the reader: it takes a `RawEvent` in file frame (mm, y up), never flips, and compares exactly. Patch mode parses the base file losslessly, matches objects, keeps untouched objects as their original bytes and merges changed ones field by field, carrying every attribute and child it does not own. courseplan's new module `ppen_export` is the inverse of `ppen_import`: project model to `RawEvent` (y flip, 2024 to Purple Pen symbol ids, number placement to offset), plus an overlay that puts the base's own values back wherever the project still equals what the import made of them. The sync (M3c) builds on the patch mode, its explicit deletions and the returned ids; it is not part of this plan.

**Tech Stack:** Rust 1.99.1 (pinned in both repos), edition 2024; `quick-xml =0.42.0` (already a core dependency, no new crates); courseplan pins core by tag (`v0.3.0` → `v0.4.0`) with a local `[patch]` during development (core ADR 0006).

**Spec:** `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.5, §5.5a, §5.8, §7; core `docs/adr/0013-the-course-crate.md`; core `crates/freemapper-course/src/ppen.rs` doc header (format and sources); Purple Pen `src/PurplePenCore/EventDB.cs` at `201417f97bb69302e3e0db123f428870e0914515` (`WriteAttributesAndContent` of each class gives the element and attribute order; `EventDB.Save`/`Load`; `ChangeEvent.DeleteCourse` for specials).

## Global Constraints

- Core: GPL-3.0-or-later, `unsafe_code = "forbid"`, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` pass (CI on Linux, macOS, Windows). Error `Display` English, no Fluent in core (core ADR 0010). Commits `feat(course): …`.
- Core ownership: freemapper-print owns `freemapper-course`. Every change to the crate is agreed with that session over SendMessage before it is made (core ADR 0005). Core release v0.4.0 is made by this session (core CLAUDE.md "Releases").
- courseplan: program texts in `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl` in the same commit; user-facing prose through `unslop` and `humanizer`; glossary rows before code; ADR 0035; changelog entries in both languages; both READMEs equal before every push.
- Deterministic output: no clock, no `HashMap` iteration order in output (use `Vec`/`BTreeMap`), ids assigned in event order. Same input bytes and event give the same output on every OS.
- Number text: every number is written as `f32` (Purple Pen stores `float`), shortest round-trip form, no exponent, `-0` as `0`; a non-finite value or one outside `f32` is an error. Equality checks in patch mode compare `as f32`.
- The writer never writes a file its own reader refuses: the event is checked against the reader limits (`MAX_CONTROLS`, `MAX_COURSES`, `MAX_STOPS_PER_COURSE`, `MAX_POINTS_PER_PATH`, `MAX_SPECIALS`, `MAX_CODE_LEN`, `MAX_NAME_LEN`, `MAX_TOTAL_POINTS`, `MAX_TOTAL_TEXT_BYTES`) before any work, and the output against `MAX_COURSE_FILE_BYTES`.
- A base file goes through `read_ppen` (all reader budgets) and through the lossless parse with the same guards (size, DOCTYPE, entities, `MAX_XML_DEPTH`). Matching uses hash maps keyed by id/code/name, never a scan per object.
- Worktrees: core `../freemapper-core/.worktrees/ppen-writer` (branch `ppen-writer`), courseplan `.worktrees/ppen-export` (branch `ppen-export`). `cargo clean` in each worktree and checkout after its merge, then remove it. Check `df -h` before long runs.

## Review Focus

- **Y sign.** Map y down becomes file y up for points *and* for `number-location`: a number placed north of its circle on paper must have a positive file `y` offset. Tests: Task 9 `map_y_down_becomes_file_y_up`, `north_number_has_positive_file_dy`.
- **Ids from another file.** A project imported from `a.ppen` and exported onto `b.ppen` must not let its `ppen_id`s capture unrelated objects in `b.ppen`. Codes and names match first; courseplan sends ids only for the linked file. Tests: Task 5 `foreign_ids_do_not_capture_objects`, Task 11 `ids_only_for_the_linked_file`.
- **Shared legs and gap units.** Purple Pen keeps one leg per ordered control pair and circle gaps per control and scale; a leg gap is written as `start` and `length` (`to - from`), not `to`. Tests: Task 4 `leg_gap_written_as_start_and_length`, `shared_leg_with_different_bends_is_refused`, `shared_circle_gaps_differ_is_refused`.
- **Description standard mismatch.** `.ppen` stores 2004 ids for every standard; courseplan holds 2024 ids. The inverse must undo the import exactly (bend between G and F, `single` variants, direction suffixes), and `standards/@description` must match what is written. Tests: Task 9 `ppen_symbol_inverts_iof_2024`, Task 4 `description_standard_must_match_options`.
- **Line endings and BOM.** Purple Pen 3.x on Windows writes BOM and CRLF; 4.0 on macOS may write LF. A patch keeps the base's style everywhere, also in new and rewritten elements. Tests: Task 3 `style_detected_from_base`, Task 5 `patch_keeps_lf_and_no_bom`.

---

## Part A: freemapper-core (worktree `.worktrees/ppen-writer`)

### Task 1: Agree the writer with freemapper-print

**Files:** none in code. Afterwards modify the courseplan design §5.5 (one line *(agreed with freemapper-print)*).

- [ ] **Step 1: Send** freemapper-print (SendMessage) the proposal: module layout (`src/ppen.rs` → `src/ppen/mod.rs` unchanged, new `src/ppen/write.rs` with submodules `out.rs`, `base.rs`, `patch.rs`), the public API of Tasks 3–5, the model changes of Task 2 (breaking: `id` in every `Special` variant, `CourseSource::item_scaling`), version 0.4.0 tagged by courseplan, core ADR 0014. Ask whether print prefers a parallel `special_ids` vector over the variant field (only alternative considered).
- [ ] **Step 2: Wait for the answer.** Apply any change print asks for to Tasks 2–5 before starting them; record the agreement in the design with *(agreed with freemapper-print)* and in core ADR 0014's source line.

### Task 2: Model additions for the writer

**Files:**
- Modify: `crates/freemapper-course/src/model.rs`, `src/ppen.rs` → `git mv` to `src/ppen/mod.rs`, `src/lib.rs`, `src/iof_xml.rs` (literals), `docs/glossary.md`
- Test: `crates/freemapper-course/tests/stable_ids.rs`, `tests/read_ppen.rs`

**Interfaces:**
- Produces:
  ```rust
  // model.rs
  pub enum Special<P> { OutOfBounds { id: Option<u32>, rings, courses }, … /* `id` first in every variant */ }
  impl<P> Special<P> { pub fn id(&self) -> Option<u32>; pub fn courses(&self) -> &CourseFilter; }
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
  pub enum ItemScaling { #[default] None, RelativeToMap, RelativeTo15000 }   // moved from ppen.rs
  pub struct CourseSource { …, pub item_scaling: Option<ItemScaling> }      // None for IOF XML
  // ppen/mod.rs, re-exported from lib.rs
  pub fn course_obj_ratio(scaling: ItemScaling, map_scale: Option<f64>, print_scale: Option<f64>) -> f64
  ```
  `special-object/@id` outside `u32` reads as `None` (as for controls). `course_obj_ratio` is the body of today's `State::course_obj_ratio` (print scale default 15000, the 1e-5 rounding to 1); the reader calls it. `map_points` carries the ids.

- [ ] **Step 1: Write failing tests:** `reader_fills_special_ids` in `stable_ids.rs` (ids of `<special-object id=…>` in sampleevent8 and overlapper by a text scan, as `reader_fills_stable_ids` does for controls); `item_scaling_is_read` in `read_ppen.rs` (sampleevent8 → `Some(ItemScaling::None)`, `<course-appearance scale-sizes="RelativeToMap"/>` → `Some(RelativeToMap)`, IOF fixture → `None`); `course_obj_ratio_is_public` (`course_obj_ratio(ItemScaling::None, Some(10000.0), Some(7500.0)) == 0.75`, `RelativeTo15000` at 1:10000 → `1.5`, unknown map scale → `1.0`).
- [ ] **Step 2: Run `cargo test -p freemapper-course`; expect FAIL** (missing fields).
- [ ] **Step 3: Implement**; every existing test passes unchanged apart from literals gaining `id: None`.
- [ ] **Step 4: Run fmt, clippy, `cargo test --workspace`; expect PASS.**
- [ ] **Step 5: Commit** `feat(course): special ids, item scaling and course_obj_ratio for the ppen writer`.

### Task 3: Output style, number text and the base parse with version gate

**Files:**
- Create: `crates/freemapper-course/src/ppen/write.rs` (module root: errors, options, re-exports), `src/ppen/write/out.rs`, `src/ppen/write/base.rs`
- Test: unit tests in `out.rs` and `base.rs`

**Interfaces:**
- Produces (public, re-exported from `lib.rs`):
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum PpenWriteError {
      #[error(transparent)] Base(#[from] CourseError),             // base file unreadable or over a reader limit
      #[error("the .ppen comes from a Purple Pen version this writer does not know: {0}")] UnknownVersion(String),
      #[error("cannot be written to a .ppen: {0}")] Unsupported(String),
      #[error("invalid event: {0}")] Invalid(String),
      #[error("courses {first} and {second} draw the leg {from} to {to} differently; Purple Pen keeps one leg per pair of controls")]
      SharedLeg { from: String, to: String, first: String, second: String },
      #[error("courses {first} and {second} give control {control} different circle gaps at 1:{scale}")]
      SharedCircleGaps { control: String, scale: i64, first: String, second: String },
      #[error("too large for a .ppen: {0}")] Limit(String),
  }
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub enum LineEnding { #[default] Crlf, Lf }
  ```
- Produces (crate-internal):
  ```rust
  // out.rs
  pub(super) struct Style { pub bom: bool, pub newline: LineEnding, pub indent: String /* "  " */ }
  pub(super) struct Element { pub name: &'static str, pub attrs: Vec<(String, String)>, pub body: Body }
  pub(super) enum Body { Empty, Text(String), Children(Vec<Child>) }
  pub(super) enum Child { Element(Element), Raw(String) /* carried bytes, already indented */ }
  pub(super) fn num(v: f64, what: &str) -> Result<String, PpenWriteError>
  pub(super) fn text_ok(s: &str, what: &str) -> Result<(), PpenWriteError>   // XML 1.0 characters only
  pub(super) fn write_element(out: &mut String, e: &Element, depth: usize, style: &Style)
  // base.rs
  pub(super) struct Base<'a> { pub text: &'a str, pub style: Style, pub event: Span, pub objects: Vec<Object<'a>>, pub tail: usize }
  pub(super) struct Object<'a> { pub kind: Store, pub id: i64, pub span: Span /* incl. leading whitespace */, pub attrs: Vec<(&'a str, &'a str)>, pub children: Vec<Span> }
  #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)] pub(super) enum Store { Control, Course, CourseControl, Leg, Special }
  pub(super) type Span = std::ops::Range<usize>;
  pub(super) fn parse_base(text: &str) -> Result<Base<'_>, PpenWriteError>
  ```
  Serializer style is `XmlTextWriter` with `Formatting.Indented`: one element per line, `indent` per depth, attributes `name="value"` separated by one space, empty element `<x a="1" />` (space before `/>`), text-only element on one line, no XML declaration. Escaping: `&amp; &lt; &gt;` in text, plus `&quot;` and `&#xA; &#xD; &#x9;` in attributes. `parse_base` keeps every byte outside the top-level objects (BOM, root tags, `<event>`, comments, whitespace) for verbatim output. Version gate (Purple Pen writes no version number: `EventDB.Save` at `201417f`): root `course-scribe-event` with no attributes, at most one `event`, top-level children only `event`, `control`, `course`, `course-control`, `leg`, `special-object`, UTF-8; anything else is `UnknownVersion` naming the element or attribute. Unknown content below the top level passes the gate (Purple Pen's `XmlInput.FindSubElement` skips it; the patch carries it).

- [ ] **Step 1: Write failing tests:**
  - `num_is_shortest_f32`: `num(140.207184) == "140.20718"`, `num(69.6024857) == "69.602486"`, `num(-6.789) == "-6.789"`, `num(-0.0) == "0"`, `num(1e-7) == "0.0000001"`, `num(f64::NAN)` and `num(1e40)` are `Invalid`.
  - `escapes_text_and_attributes`: name `A<&"B` → text `A&lt;&amp;"B`, attribute `A&lt;&amp;&quot;B`; `text_ok("a\u{1}")` is `Invalid`.
  - `writes_purple_pen_layout`: lincoln-park control 1 built as an `Element` at depth 1 with CRLF and two-space indent gives `  <control id="1" kind="start">\r\n    <location x="140.20718" y="57.44655" />\r\n  </control>\r\n` (the f32 shortest form; the fixture's `140.207184` survives only through the patch's kept bytes).
  - `style_detected_from_base`: each fixture → `bom: true, newline: Crlf, indent: "  "`; the same text with `\r\n` → `\n` and BOM removed → `bom: false, Lf`.
  - `base_spans_cover_the_file`: for each fixture, concatenating the non-object bytes and object spans in order gives the file back; object counts per store equal a text count of `<control id=`, `<course id=`, …
  - `gate_refuses_unknown_versions`: `<course-scribe-event version="5">`, a top-level `<team>`, root `<event-file>`, a UTF-16 file → `UnknownVersion`/`Base`; a `<foo/>` inside a control passes.
  - `base_parse_has_reader_guards`: DOCTYPE, an undefined entity, nesting deeper than `MAX_XML_DEPTH`, a file over `MAX_COURSE_FILE_BYTES` → errors, no panic.
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement** `out.rs` and `base.rs`; `base.rs` runs quick-xml over the bytes with `buffer_position()` for spans and reuses `xml::drive`'s refusals (factor the DOCTYPE/entity checks into one shared function, do not copy them).
- [ ] **Step 4: Run; expect PASS.** fmt, clippy.
- [ ] **Step 5: Commit** `feat(course): ppen output style, number text and base parse with version gate`.

### Task 4: The writer from scratch

**Files:**
- Modify: `crates/freemapper-course/src/ppen/write.rs`, `src/lib.rs`
- Test: `crates/freemapper-course/tests/write_ppen.rs`

**Interfaces:**
- Consumes: Task 3 `Element`, `write_element`, `num`, `Style`; Task 2 `Special::id`, `course_obj_ratio`; the reader's private `gap_key` logic (call it, the writer is a child module of `ppen`).
- Produces:
  ```rust
  pub struct ScratchOptions {
      pub map: PpenMap,
      pub map_standard: MapStandard,
      pub description_standard: DescriptionStandard,
      pub description_lang: String,          // "de", "en"
      pub line_ending: LineEnding,
  }
  pub enum PpenMap { None, Ocad { path: String }, Bitmap { path: String, dpi: f64 }, Pdf { path: String } }
  pub enum MapStandard { Isom2000, Isom2017, Issprom2019 }          // "2000", "2017", "Spr2019"
  pub enum DescriptionStandard { Iof2004, Iof2018 }                  // "2004", "2018"
  pub struct Written { pub text: String, pub ids: AssignedIds, pub kept: Vec<KeptObject>, pub notes: Vec<WriteNote> }
  pub struct AssignedIds { pub controls: Vec<u32>, pub courses: Vec<u32>, pub specials: Vec<u32> } // parallel to the event's lists
  pub enum KeptObject { Control { id: u32, code: String }, Course { id: u32, name: String }, Special { id: u32, kind: String } }
  pub enum WriteNote { FlaggingImplied { course: String, from: String, to: String } }
  pub fn write_ppen(event: &RawEvent, options: &ScratchOptions) -> Result<Written, PpenWriteError>
  ```
- Rules the tests pin:
  - Points are written as given (file frame, y up); the writer never flips. Order of stores as `EventDB.Save`: `event`, controls, courses, course-controls, legs, specials; within a store in event order.
  - `<event id="1">`: `title` (from `source.event_name`, omitted if `None`), `map` (`kind` none/OCAD/bitmap/PDF, `scale` from `source.map_scale`, required; `dpi` for bitmap; `ignore-missing-fonts="false"` for OCAD; the path as text, no `absolute-path`), `standards map=… description=…`, `all-controls print-scale=<map scale> description-kind="symbols"`, `course-appearance` empty (item scaling none), `descriptions lang=…`.
  - Ids: an object's `id` is kept when given and unique within its store; a missing or repeated one gets `max(ids in the store) + 1` in event order. Course-controls are numbered 1.. in course order, legs 1.. in first-use order. Course `order` is 1..n in event order.
  - Controls: `kind` from `ControlKind` (`EndOfMarkedRoute` → `Unsupported`); `code` only for normal controls (`Stop.control` resolves by `Control::code`); `location` required (`None` → `Invalid`); description boxes: `all` for finish and crossing point (one symbol, as `EventDB.cs` writes `symbolIds.Length == 1`), else `C`..`H` for each `Some` column, `column_f_text` as text of box `F` (or `all`); every `Description::standard` must equal the option's string, else `Invalid`. `geo` is not written (no `.ppen` field). Circle gaps: one `<circle-gaps scale=K>` per gap key K that a course using the control has, text `start:stop` pairs joined by `,` (`CircleGap.EncodeGaps`); two courses with the same K and different gaps → `SharedCircleGaps`.
  - Courses: `kind` normal/score, `order`, `name`, `secondary-title`, `first course-control`, `options print-scale climb course-length` (each only when `Some`). `family: Some` → `Unsupported` (Purple Pen has no course families outside variations). A course without stops has no `first`.
  - Course-controls: `control`, `map-exchange="true"` when `Stop::kind == Some(MapExchange)` at a normal control, `map-flip="true"` with it, `next`, `number-location x=dx y=dy` from `number_text_at` as given. `Stop::number: Some` → `Unsupported` ("convert with offset_from_placement first"), so a placement is never dropped silently.
  - Legs: one per ordered pair of control ids that has bends, gaps or a flagging to write; `flagging kind="all"` when `marked_route` and Purple Pen does not flag the leg on its own (from map issue, into map exchange, into a finish whose column C is `14.1`); `marked_route == false` on such a leg gives `WriteNote::FlaggingImplied`. Gaps `<gap start=from length=to-from />`. Two courses with the same pair and different bends, gaps or flagging → `SharedLeg`.
  - Specials: `first-aid`, `water` (one location), `forbidden-route` (path of exactly one point, else `Unsupported`), `boundary`, `line` (path), `out-of-bounds`, `dangerous-area`, `white-out` (exactly one ring, else `Unsupported`). `Text` and `MarkedRoute` → `Unsupported` (the model keeps no text box width; Purple Pen has no marked-route special). Courses: `<courses all="true" />` or one `<course course=id />` per name; a name that matches no course or two courses → `Invalid`.

- [ ] **Step 1: Write failing tests** in `tests/write_ppen.rs`:
  - `round_trip_of_the_fixtures`: for each of the three fixtures, `read_ppen(write_ppen(read_ppen(f)))` equals `read_ppen(f)` in controls (code, kind, position as f32, description), courses (id, name, stops, legs, climb, length, print scale, secondary title), specials the writer supports, and `source.map_scale`; notes may differ. Options from the fixture's own event element.
  - `ids_are_stable`: writing twice gives identical text; given ids come back from the reader unchanged; a repeated control id gets `max + 1`.
  - `leg_gap_written_as_start_and_length`: Lincoln Park's leg gap reads back `(70.4939, 86.15942)` within 1e-4 and the text has `start="70.4939" length="15.66552"`.
  - `shared_leg_with_different_bends_is_refused` and `shared_circle_gaps_differ_is_refused` (names both courses).
  - `description_standard_must_match_options`: a `Description { standard: "2004" }` with `Iof2018` → `Invalid`.
  - `start_and_finish_have_no_code`, `crossing_point_description_box_all`, `map_exchange_and_flip_at_a_control`.
  - `unsupported_content_is_refused`: `Stop::number: Some`, `family: Some`, `EndOfMarkedRoute`, `Text` special, a two-point forbidden route, two rings → `Unsupported`.
  - `limits_are_checked_first`: an event with `MAX_CONTROLS + 1` controls, or one shared leg of `MAX_POINTS_PER_PATH` bends in 300 courses (over `MAX_TOTAL_POINTS`) → `Limit`, returned before any output is built.
  - `no_clock_and_no_map_paths_beyond_the_given_one`: output contains no `absolute-path` and is identical across two runs.
- [ ] **Step 2: Run `cargo test -p freemapper-course --test write_ppen`; expect FAIL.**
- [ ] **Step 3: Implement `write_ppen`.** Build `Element`s per object with one builder per store (`control_element`, `course_element`, `course_control_element`, `leg_element`, `special_element`), each returning the owned fields in Purple Pen's write order; Task 5 reuses them.
- [ ] **Step 4: Run; expect PASS.** fmt, clippy, `cargo test --workspace`.
- [ ] **Step 5: Commit** `feat(course): write a .ppen from scratch`.

### Task 5: Patch mode

**Files:**
- Create: `crates/freemapper-course/src/ppen/write/patch.rs`
- Modify: `src/ppen/write.rs`, `src/lib.rs`
- Test: `crates/freemapper-course/tests/patch_ppen.rs`

**Interfaces:**
- Consumes: Task 3 `parse_base`, `Base`, `Object`, `Style`; Task 4 element builders, `Written`, `AssignedIds`, `KeptObject`; the reader's `State` records (`PControl`, `PCourse`, `PCc`, `PLeg`, `PSpecial`, keyed by id) of the base, for field comparison.
- Produces:
  ```rust
  #[derive(Debug, Clone, Default)]
  pub struct PatchOptions { pub delete: Deletions }
  #[derive(Debug, Clone, Default)]
  pub struct Deletions { pub controls: Vec<u32>, pub courses: Vec<u32>, pub specials: Vec<u32> }
  pub fn write_ppen_patch(base: &[u8], event: &RawEvent, options: &PatchOptions) -> Result<Written, PpenWriteError>
  ```
- Matching, each base object at most once, in event order:
  - Normal controls: code, then id if the event's code is not in the base (a code change). Other control kinds (no code in the file): id, then the reader's derived code (`STA1`, `FIN1`, `CROSS1` …), then equal kind and position.
  - Courses: name, then id if the name is not in the base (a rename). Specials: id, then equal kind and points.
  - A given id that matched nothing and is free in its store is used for the new object; otherwise `max + 1` over base and event ids.
- Merge per matched object, field by field against the reader's value of the base object: an owned field whose value equals the base's (numbers compared as `f32`) keeps the base's bytes (attribute text or child elements); a changed one is regenerated by the Task 4 builder. Attributes and children the writer does not own (`orientation`, `punch-pattern`, `labels`, `print-area`, `relay`, `appearance`, `font`, `description-text-line`, unknown ones …) are carried in their base text. Children are emitted in Purple Pen's write order with unknown children last, in base order. An object with no changed field is written as its original span, byte for byte.
- Shared structure: changed circle gaps at scale K replace that control's `circle-gaps` and `gaps` children of scale K only; if the base control has the old `gaps` attribute (scale 0), it is dropped and its gaps are written as explicit `circle-gaps` for every key `FixControlPointGaps` would fill (map scale and every course print scale) except K. Legs keyed by (start id, end id); a changed course keeps its base course-control ids in chain order, takes `max + 1` for more and deletes the ones it no longer uses (Purple Pen's `Validate` refuses unused course-controls). A variation course in the base is never rewritten: an event course matching one → `Unsupported`.
- Unmatched base objects are kept as they are and listed in `Written::kept`. Only `Deletions` delete: a control (refused with `Invalid` if a course of the output still uses it) with its legs; a course with its course-controls, removed from specials' course lists, and a special whose list becomes empty is deleted (`ChangeEvent.DeleteCourse`).
- `order`: kept when the event courses keep their relative base order; otherwise every course gets 1..n, event courses in event order, then kept ones in base order. New objects go after the last base object of their store (or after the previous store's last); indentation and newline from `Base::style`. The `<event>` element is never changed in patch mode.

- [ ] **Step 1: Write failing tests** in `tests/patch_ppen.rs`:
  - `untouched_patch_is_byte_identical`: for each fixture, `write_ppen_patch(f, &read_ppen(f)?, &default)?.text.as_bytes() == f`, and `kept` lists what the reader left out (variation courses, `descriptions` specials).
  - `one_moved_control_changes_one_line`: move Lincoln Park control 31 by (1, 0) mm; the diff to the base is the one `<location …>` line of that control.
  - `unknown_content_survives`: base with `<foo a="1"/>` inside a control, `x-new="2"` on a course, a comment between objects; after changing that control's description and that course's name, all three are still in the output, in place.
  - `partial_flagging_and_map_issue_kept`: a leg with `flagging kind="beginning-part" x y` and a `map-issue` control stay byte-identical when their course's name changes.
  - `foreign_ids_do_not_capture_objects`: event read from lincoln-park with its ids, patched onto overlapper: no overlapper control or course changes its code or name; lincoln-park's controls come in as new ones with fresh ids.
  - `added_course_gets_fresh_ids_and_order`: new course in sampleevent8 gets course id `max + 1`, course-controls `max + 1 ..`, `order = max + 1`; the reader reads it back equal.
  - `shortened_course_deletes_its_course_controls`: remove a stop; no course-control in the output is unused.
  - `deletions_cascade`: deleting a course removes its course-controls and its entry in specials; a special listing only that course is gone; deleting a control still in use → `Invalid`.
  - `old_gaps_attribute_materialised`: control with `gaps="0111…10"` and a changed gap at 1:7500 reads back with the new gaps at 7500 and the old ones at every other course scale.
  - `patch_keeps_lf_and_no_bom`: overlapper with LF and no BOM, one control added: output has no `\r` and no BOM, the new control's lines use two spaces.
  - `hostile_base_is_refused_cheaply`: a base at `MAX_CONTROLS` controls and 1000 courses sharing one leg with `MAX_POINTS_PER_PATH` bends → `Base(Limit)` from the reader; a base with a DOCTYPE → error; neither panics.
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement `write_ppen_patch`** in `patch.rs`: gate and parse the base (Task 3), read it with the reader's `State` for per-id records, match, merge, emit.
- [ ] **Step 4: Run; expect PASS.** fmt, clippy, `cargo test --workspace`.
- [ ] **Step 5: Commit** `feat(course): patch an existing .ppen`.

### Task 6: Purple Pen 4.0 fixture (when the user's file arrives)

Independent of all other tasks; until the file exists nothing else waits for it.

**Files:**
- Create: `crates/freemapper-course/tests/fixtures/courses/ppen/pp4-<name>.ppen` (core), `tests/fixtures/ppen/pp4-<name>.ppen` (courseplan)
- Modify: both fixture `README.md`s (source: the user's Purple Pen 4.0 build and date, SHA-256, licence as the user states it, what was removed), `tests/patch_ppen.rs`, `tests/write_ppen.rs`, `src/ppen/write/base.rs` (vocabulary), and the courseplan `ppen_export` tests

- [ ] **Step 1: Ask the user's consent** to commit the file (design §7) and remove absolute paths: replace `absolute-path="…"` by nothing and note it in the README (the only change).
- [ ] **Step 2: Add the file to `untouched_patch_is_byte_identical`, `round_trip_of_the_fixtures` and the courseplan round trip of Task 11.** Run; a failure names the new element or attribute.
- [ ] **Step 3: Extend the vocabulary** (gate, owned and carried lists) with what 4.0 writes, and record in core ADR 0014: newline and BOM of 4.0 on macOS, how it names an `.omap` map (`kind`, path), whether it accepts `standards description="2024"`.
- [ ] **Step 4: If 4.0 writes an `.omap` map differently from `kind="OCAD"`, change `PpenMap` and courseplan's choice in Task 10 to match.**
- [ ] **Step 5: Commit in each repo** `test(course): Purple Pen 4.0 fixture` / `test(ppen): Purple Pen 4.0 fixture`.

---

## Part B: freemapper-courseplan (worktree `.worktrees/ppen-export`)

During Part B, `Cargo.toml` carries `[patch."https://github.com/HenryJobst/freemapper-core"]` entries pointing at `../freemapper-core/.worktrees/ppen-writer/crates/{freemapper-course,freemapper-omap,freemapper-text,freemapper-rules}`; Task 13 removes them.

### Task 7: Purple Pen ids on special objects

**Files:**
- Modify: `crates/freemapper-courseplan-model/src/types.rs` (`ppen_id: Option<u32>` in every `SpecialObject` variant, serde default and skipped when `None`; `SpecialObject::ppen_id()`), `crates/freemapper-courseplan/src/ppen_import.rs` (fill it: start/finish from `Control::id`, others from `Special::id`; several rings of one special: the id goes to the first only), `docs/glossary.md` (row "Purple-Pen-Kennung" extended to special objects), every `SpecialObject` literal
- Test: `crates/freemapper-courseplan-model/src/types.rs` tests, `crates/freemapper-courseplan/src/ppen_import/tests.rs`

- [ ] **Step 1: Write failing tests:** `special_ppen_id_round_trips_in_toml` (written as `ppen_id = 7`, absent when `None`, `FORMAT_VERSION` stays 1); `import_keeps_special_and_start_ids` (sampleevent8: start `STA1` has its control id, the water special its special id).
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run `cargo test --workspace --locked`; expect PASS.**
- [ ] **Step 5: Commit** `feat(model): Purple Pen ids on special objects`.

### Task 8: Inverse description ids

**Files:**
- Modify: `crates/freemapper-courseplan/src/ppen_import.rs` (`iof_2024` and `Mapped` become `pub(crate)`)
- Create: `crates/freemapper-courseplan/src/ppen_export.rs` (this function first), `crates/freemapper-courseplan/src/ppen_export/tests.rs`

**Interfaces:**
- Produces:
  ```rust
  /// Purple Pen symbol ids for boxes C..H of one courseplan description, and the (column, 2024 id) pairs
  /// that have no Purple Pen id and are left out.
  pub(crate) fn ppen_columns(d: &ControlDescription) -> ([Option<String>; 6], Vec<(char, String)>)
  ```
  Built once by enumeration so it cannot drift from the import: every candidate Purple Pen id (sections 0–15, items 1–30, suffixes none, `N`…`NW`, `high`, `deep`, `single`, `control`) through `iof_2024`, into a `BTreeMap<2024 id, Purple Pen id>` that prefers the plain id. Then the two import rules backwards: bend `11.1` in F goes to box G as `11.7` when G is empty (else stays in F); D equal to E with a `single` variant (`10.1`, `10.2`, `12.14`) becomes `<id>single` in D with E empty.

- [ ] **Step 1: Write failing tests:** `ppen_symbol_inverts_iof_2024`: for every courseplan description the import can produce from a single Purple Pen id per column, import(export(d)) == d; explicit rows `5.21` → `5.23`, `12.1NE` → `11.1NE`, `13.1` → `12.1`, `15.3` → `13.3`, `16.3` → `14.3`, F `11.1` → G `11.7`, D=E=`10.1` → D `10.1single`; `2024_only_symbol_is_reported` (an id with no Purple Pen counterpart lands in the second list with its column).
- [ ] **Step 2: Run; expect FAIL.** **Step 3: Implement.** **Step 4: Run; expect PASS.**
- [ ] **Step 5: Commit** `feat(ppen): inverse description ids for the .ppen export`.

### Task 9: Project to `RawEvent`

**Files:**
- Modify: `crates/freemapper-courseplan/src/ppen_export.rs`, `src/lib.rs`, `ppen_export/tests.rs`

**Interfaces:**
- Consumes: Task 8 `ppen_columns`; core `course_obj_ratio`, `offset_from_placement`, `ItemScaling`; courseplan `numbers::{circle_outer_radius_mm, number_box_mm}`, `map::MapKind`.
- Produces:
  ```rust
  pub struct ToRaw { pub event: RawEvent, pub notes: Vec<ExportNote> }
  pub enum ExportNote {
      CourseSkipped { course: String, kind: &'static str },        // "team", "relay", "score"
      SpecialSkipped { id: String, why: &'static str },            // "marked route", "forbidden route with several points", "no course left"
      SymbolWithoutPpenId { control: String, column: char, id: String },
      CodeNotStored { code: String },     // start, finish or crossing point whose code a re-import would not give back
      FlaggingImplied { course: String, from: String, to: String },
      Kept(KeptObject),
  }
  pub(crate) fn to_raw(event: &Event, kind: MapKind, map_scale: u32, scaling: ItemScaling, use_ids: bool) -> ToRaw
  ```
- Rules: point `MapPoint { x, y }` → `RawPoint { x_mm: x, y_mm: -y }` (mm, no µm step). Number per stop: course-control placement, else the control's central one, else none; offset from `offset_from_placement(p, circle_outer_radius_mm(kind), number_box_mm(kind, n), ratio)` with `n` the stop's sequence number and `ratio = course_obj_ratio(scaling, map_scale, print scale)`, then `dy_mm` negated. Print scale `planned_print_scale` or the map scale. Starts and finishes become controls of kind start/finish with their special id as code; crossing points keep `CrossingPoint`. Taped legs → `marked_route`. Climb `u32` → `f64`. Team, relay and score courses are skipped and named (like the IOF XML export and the planned sync) *(Claude's call)*; a special whose course list then names no written course is skipped. Ids only when `use_ids`.

- [ ] **Step 1: Write failing tests:**
  - `map_y_down_becomes_file_y_up`: control at `MapPoint(12.345, 6.789)` → `RawPoint(12.345, -6.789)`; bends and special points likewise.
  - `north_number_has_positive_file_dy`: placement direction 90° gap 1 mm → `number_text_at.dy_mm > 0`, `dx_mm` ≈ 0; direction 0° → `dx_mm > 0`, `dy_mm` ≈ 0.
  - `number_round_trips_through_the_import`: `placement_from_offset` of the produced offset (dy negated back) gives the placement within 0.01° and 0.01 mm, at ratio 0.75 and 1.5.
  - `team_relay_score_are_skipped_and_named`, `marked_route_special_is_skipped`, `two_point_forbidden_route_is_skipped`.
  - `ids_only_when_asked`: `use_ids = false` → every `id` is `None`.
- [ ] **Step 2: Run; expect FAIL.** **Step 3: Implement `to_raw`.** **Step 4: Run; expect PASS.**
- [ ] **Step 5: Commit** `feat(ppen): map the project onto a Purple Pen event`.

### Task 10: Overlay on the base and `export_ppen`

**Files:**
- Modify: `crates/freemapper-courseplan/src/ppen_export.rs`, `ppen_export/tests.rs`
- Create: `crates/freemapper-courseplan/tests/ppen_export.rs` (real fixtures, frame maps as in `tests/ppen_import.rs`; move `frame_map` into a shared `tests/common/mod.rs`)

**Interfaces:**
- Consumes: Task 9 `to_raw`; `ppen_import::{read, import_ppen, from_fitted}`; core `write_ppen`, `write_ppen_patch`, `ScratchOptions`, `PatchOptions`, `Written`.
- Produces:
  ```rust
  pub struct PpenExport { pub text: String, pub ids: ExportIds, pub notes: Vec<ExportNote>, pub courses: usize }
  pub struct ExportIds { pub controls: Vec<(ControlCode, u32)>, pub courses: Vec<(String, u32)>, pub specials: Vec<(String, u32)> }
  #[derive(Debug, thiserror::Error)]
  pub enum PpenExportError { #[error(transparent)] Base(#[from] PpenImportError), #[error(transparent)] Write(#[from] PpenWriteError) }
  pub struct Target<'a> { pub base: Option<&'a [u8]>, pub linked: bool, pub map_path: String, pub lang: &'a str }
  pub fn export_ppen(event: &Event, map: &MapDocument, target: Target<'_>) -> Result<PpenExport, PpenExportError>
  ```
- Overlay (only with a base): read the base (`RawEvent`) and import it onto the same map (`PpenImport`). For each project control, course and special that equals the import's object of the same `ppen_id` (else code or name), the base's raw object replaces the mapped one, so core sees no change and keeps the bytes. Courses compare with the effective number placement per stop. For changed objects, the fields the project does not hold come from the matched base object: column F text and start/finish descriptions; secondary title and course length; circle gaps of a stop whose control did not move; gaps and partial flagging of a leg whose bends did not change. Item scaling and ratio come from the base. Without a base: `ScratchOptions` with `PpenMap::Ocad { path: target.map_path }` for `.omap`/`.xmap` (verified in Task 6), `MapStandard::Issprom2019` for sprint and schoolyard maps, `Isom2017` for forest, `Iof2018` descriptions (Purple Pen 3.x knows 2004 and 2018; ids are 2004 numbers in both), CRLF.

- [ ] **Step 1: Write failing tests** in `tests/ppen_export.rs`:
  - `import_then_export_onto_the_same_file_is_byte_identical`: each fixture, imported onto its frame map, exported with itself as base → identical bytes.
  - `moved_control_changes_only_that_control`: after moving one control in the project, re-reading the export gives the same event except that control's position.
  - `kept_objects_are_reported`: the skipped score course of overlapper comes back as `ExportNote::Kept`.
  - `export_from_scratch_reads_back`: the Lübeck example project → `read_ppen` + import onto the Lübeck map gives controls, courses (stops, bends, taped legs, climb, print scale) and specials equal to the project (ppen ids aside), numbers within 0.01° and 0.01 mm.
  - `same_project_twice_same_bytes`.
- [ ] **Step 2: Run; expect FAIL.** **Step 3: Implement `export_ppen`.** **Step 4: Run; expect PASS.**
- [ ] **Step 5: Commit** `feat(ppen): export onto a base .ppen or from scratch`.

### Task 11: `export --ppen` and the MCP tool `export_ppen`

**Files:**
- Modify: `crates/freemapper-courseplan-cli/src/export_cmd.rs`, `src/main.rs` (`--ppen FILE`, `--new`), `src/mcp.rs`, `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl`
- Test: `crates/freemapper-courseplan-cli/tests/export.rs`, `tests/mcp.rs`

**Interfaces:**
- Consumes: Task 10 `export_ppen`, `Target`, `PpenExport`; `same_file`, `write_atomic`, `ProjectFile`.
- Produces: `pub(crate) fn export_ppen_file(project: &Path, target: &Path, new: bool) -> Result<PpenReport, ExportError>` shared by CLI and MCP; `ExportError::Ppen(PpenExportError)`; MCP tool `export_ppen { project, path, new? }` returning compact JSON `{ courses, skipped, kept, notes, linked }`.
- Behaviour: `--iof-xml` and `--ppen` may be given together; neither is a usage error (`export-nothing-to-do`, replacing `export-iof-xml-nothing-to-do`). The target must not be the project or the map (`error-export-overwrites-input`). An existing target is the base unless `--new`. The target is "linked" when it is `sources.ppen` or the project has none (then it becomes `sources.ppen`, relative to the project); only then are ids sent and the assigned ids written back into the project (`ppen_id` of controls, courses, specials). Write order: `.ppen` atomically, then the project atomically. Messages: `export-ppen-written` (count, path), `export-ppen-linked`, `export-ppen-kept` (per kept object), `export-ppen-skipped`, `export-ppen-note-*` per `ExportNote`, `error-export-ppen-*` per `PpenWriteError` variant (English `Display` stays the source text).

- [ ] **Step 1: Write failing tests:** `export_ppen_writes_and_links` (fresh target: file written, `sources.ppen` set, ids in the project); `ids_only_for_the_linked_file` (project linked to `a.ppen`, export onto a copy `b.ppen`: project file unchanged, `b.ppen` objects matched by code/name only); `export_ppen_new_ignores_the_existing_file`; `export_ppen_refuses_the_project_and_map_as_target`; `refused_course_and_team_named_in_de_and_en`; `unknown_version_base_exits_non_zero_and_writes_nothing` (base with `<course-scribe-event version="9">` stays unchanged); MCP `export_ppen` returns the JSON fields.
- [ ] **Step 2: Run; expect FAIL.** **Step 3: Implement**; texts in both languages, longer ones through `unslop`/`humanizer`. Run the real CLI once on the Lübeck example and read the output.
- [ ] **Step 4: Run `cargo test --workspace --locked`; expect PASS.**
- [ ] **Step 5: Commit** `feat(cli): export --ppen and MCP tool export_ppen`.

---

## Part C: Documentation, releases, integration

### Task 12: Core docs, merge and release v0.4.0

**Files (core worktree):**
- Create: `docs/adr/0014-the-ppen-writer.md`
- Modify: `docs/adr/README.md`, `docs/glossary.md` (patch mode, base file, kept object), `README.md` + `README.en.md` (crate list: the course crate also writes `.ppen`), `CHANGELOG.md` + `CHANGELOG.en.md` (`freemapper-course`: Added writer and patch mode; Changed: `Special` variants carry `id`, `CourseSource::item_scaling`, what print must change), `crates/freemapper-course/Cargo.toml` (`description`), `src/lib.rs` doc header, `Cargo.toml` (`version = "0.4.0"`)

- [ ] **Step 1: ADR 0014** (about 30 lines): writer in the course crate (agreed with freemapper-print), format-faithful frame, no version number in `.ppen` so the gate is the top-level vocabulary, exact field-by-field patch with carried content, explicit deletions only, id and order rules, what is refused, f32 number text, style taken from the base. Source line: courseplan design §5.5/§5.5a, Task 1 agreement, `EventDB.cs` at `201417f`.
- [ ] **Step 2: Other documents** in both languages; `unslop`, `humanizer`.
- [ ] **Step 3: fmt, clippy, `cargo test --workspace`; merge `--no-ff` into core `main`;** push to `origin` and `github`; CI green.
- [ ] **Step 4: Release:** `cargo update -w`, date the changelog sections `## [0.4.0] - YYYY-MM-DD`, commit `chore: release 0.4.0`, annotated tag `v0.4.0`, push `main` and the tag to both remotes; tell freemapper-print and freemapper over SendMessage which version to pin and what changed. `cargo clean` and remove the core worktree.

### Task 13: courseplan pin, docs, integration test, merge and release

**Files:**
- Modify: `Cargo.toml` (core crates `tag = "v0.4.0"`, `[patch]` removed), `Cargo.lock`
- Create: `docs/adr/0035-ppen-export-patches-the-linked-file.md`, `examples/luebeck-sprint/luebeck-sprint.ppen`
- Modify: `docs/adr/README.md`, design §5.5, §5.5a, §9 (what is built; marks *(user)*, *(Claude's call)*, *(agreed with freemapper-print)*), `README.md` + `README.en.md`, `docs/manual/{de,en}.md` (chapter `export --ppen`), `examples/README*.md`, `examples/luebeck-sprint/README*.md`, `CHANGELOG.md` + `CHANGELOG.en.md`, `docs/glossary.md`
- Test: `crates/freemapper-courseplan/tests/ppen_export.rs`

- [ ] **Step 1: Pin v0.4.0, drop the `[patch]`; `cargo test --workspace --locked`; expect PASS.**
- [ ] **Step 2: Integration test** `courseplan_reads_its_own_export_twice`: the Lübeck project → export (no base) → import into a new project on the Lübeck map → export again with the first file as base and `linked` → identical bytes, and the second project equals the first (ppen ids aside, numbers within 0.01° and 0.01 mm). Write the first export to `examples/luebeck-sprint/luebeck-sprint.ppen` and assert the committed file equals a fresh export.
- [ ] **Step 3: ADR 0035** (about 30 lines): mapping and its losses, the overlay on the base, patch onto an existing target unless `--new`, export never deletes (the sync will), ids and link written back only for the linked file, team/relay/score skipped and named, `.omap` as `kind="OCAD"`, description standard 2018, open: Purple Pen 4.0 check (Task 6).
- [ ] **Step 4: Other documents** in German and English, both READMEs equal; `unslop`, `humanizer`. Ask the user to open `examples/luebeck-sprint/luebeck-sprint.ppen` in Purple Pen 4.0 and note the result in ADR 0035.
- [ ] **Step 5: fmt, clippy, tests; final review; merge `--no-ff` into `main`; push to both remotes; CI green.** `cargo clean` in the worktree and the checkout, remove the worktree.
- [ ] **Step 6: Release 0.7.0** as CLAUDE.md describes; report the release URL. Tell freemapper-print over SendMessage that the `.ppen` export exists (memory note "Sibling handoffs").

---

## Open questions for the user

1. **Export onto an existing `.ppen`:** patch it, keeping what the project does not hold and never deleting, with `--new` to overwrite from scratch (planned default), or always overwrite?
2. **Export changes the project:** it links the target as `sources.ppen` when none is linked and writes the Purple Pen ids back, so the next export and the sync find the same objects. Acceptable, or only on an explicit flag?
3. **Purple Pen 4.0 file** (Task 6): your consent to commit it with absolute paths removed, and the file itself.
