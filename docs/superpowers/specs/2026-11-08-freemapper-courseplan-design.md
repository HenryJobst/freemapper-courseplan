# freemapper-courseplan – Design

Status: approved 2026-11-08 · Date: 2026-11-08

Statements marked *(user)* were decided by the user in the brainstorming on 2026-11-08. Statements marked
*(Claude's call)* are proposals the user has not confirmed yet. Statements marked *(agreed with
freemapper-print)* were settled between this session and the freemapper-print session on the user's request
and are recorded in freemapper-print's design (§2 "Later", §5.7, §5.8, ADR 0015). The research on course
planning criteria is in [`docs/research/2026-11-08-bahnlegung-kriterien.md`](../../research/2026-11-08-bahnlegung-kriterien.md)
(German); section numbers like "research §3.1" point there.

## 1. Intent

The freemapper suite covers the path from official geodata to a printed orienteering map:

- [freemapper](../../../../freemapper) produces an OOM starter map from official geodata.
- The mapper finishes the map in OpenOrienteering Mapper (OOM).
- [freemapper-print](../../../../freemapper-print) turns map and courses into print PDFs.

Course setting is the missing step. The author uses Purple Pen today *(user)*. Purple Pen has no API, no
scripting and no command line beyond opening a file, so it cannot be driven by another program or an AI agent.
Its macOS version (Avalonia, Purple Pen 4) exists only as unreleased work in its repository. OCAD does not run
on the author's MacBook *(user)*.

Current programs also handle German formats badly *(user)*, above all team orienteering ("Aufgeteilter
Mannschafts-OL mit Pflichtposten", WKB 2026 B 3.3.2.3). A guide from Berlin-Brandenburg (2025) describes
about 30 minutes of manual work per course in OCAD and warns that codes get mixed up easily (research §3).

**freemapper-courseplan** is a course-setting application for the suite. The author sets courses by hand in a
browser editor on top of the OOM map *(user)*. Where it helps, and only after it has been tested, AI supports
the work as a checker and as a source of proposals *(user)*. Later versions add relay, team and score formats,
pre-planned control-placer tours and a route analysis.

Users: course setters across the German orienteering community, and internationally where it makes sense
*(user, 2026-11-08)*; first the author and club members who set courses for training and events. They work
through the browser editor, the CLI or an AI agent via MCP. Consequences *(Claude's call)*: rule profiles are
data that anyone can add (national rules, regional recommendations such as the NOR tables for
Berlin/Brandenburg, IOF and other federations, research §3.3a); nothing in the code assumes one region; exports
use the international formats (IOF XML 3.0) besides `.ppen`.

## 2. Goals, non-goals, anti-goals

**First usable version: basic course setting for individual courses** *(user)*

- Map kinds: sprint (ISSprOM 2019-2) and schoolyard maps, the kinds freemapper produces today. Forest maps
  (ISOM 2017-2) come later *(user)*. Similarity groups, impassable symbol lists, column D proposals and
  overprint sizes therefore cover ISSprOM and the school set first; the rule data keeps room for ISOM.
- Open an event with an OOM map (`.omap`, `.xmap`) as background.
- Controls with position, code and control description (IOF 2024, columns A–H). One control can be used in
  several courses.
- Start, finish, marked routes, mandatory crossing points, forbidden areas and routes, out-of-bounds areas,
  dangerous areas, first aid and water points.
- Courses as an ordered list of controls from start to finish, with leg bends and gaps, assigned to classes.
- Course length as WKB B 7.2.5 measures it: straight lines start → controls → finish, plus marked routes and
  the planned detours around forbidden and impassable areas. Climb is entered by hand until the route analysis
  exists *(Claude's call)*.
- Control number positions: central per control, overridable per course, with automatic placement *(user,
  agreed with freemapper-print)*.
- Checks that need no routing (§5.6), each with its source.
- Exchange with Purple Pen (`.ppen`), repeatable in both directions (§5.5a). The author moves over once this
  application has everything needed; until then both programs are used in parallel, and in the long run
  other course setters who stay with Purple Pen should be able to work on the same event *(user)*.
- Export IOF XML 3.0 for result software, mainly OE (SportSoftware) *(user: results are usually done in OE)*.
- Export `.ppen` *(Claude's call)*. freemapper-print's first version reads only `.omap`, `.ppen` and IOF XML,
  and only `.ppen` carries control descriptions, bends and special objects. Without this export courses from
  freemapper-courseplan would print without descriptions.
- Three interfaces over one library and one project file: CLI, MCP server, local browser editor *(user)*.

**Later, in this order** *(user: route analysis soon after the first version; relay and team later. The order
after that is Claude's call)*

1. **Route analysis.** A cost surface from the map symbols (a speed per symbol, impassable symbols blocked)
   and the elevation model. Per leg the fastest route and clearly different alternatives with length, climb
   and estimated time, drawn on the map *(user: like the idea of OCAD's route analyzer, not a copy of it)*.
   It then feeds climb, winning time estimates and the routing checks.
2. **Team orienteering** (§4.3): mandatory controls labelled A, B, C … and connected, free controls split by
   the team, evaluation window between the first and the last mandatory control *(user)*. Plus an estimate of
   the best split for a team as a fairness check, which no rule and no program covers today (research §2).
3. **Relay and forking**: variants, combinatorics check (every team runs every part equally often), variant
   balance from the route analysis.
4. **Score**: control values, time limit, penalties (BOF B12 as the main rule source; WKB has no score format
   since 2025).
5. **Control-placer tours** *(user)*: planned beforehand by the course setter or split on site by the number
   of helpers present *(user: both cases)*. A routing optimisation over real walking times. Tour sheets are
   printed by freemapper-print (§5.8 there) *(agreed with freemapper-print)*.
6. **AI checker and proposal mode** *(user)*: the judgment criteria (class B in the research) and proposals
   for control sites or legs. Each judgment is tested against examples before it ships *(user: "must be
   tested")*.
7. Data model crate shared with freemapper-print, replacing the `.ppen` detour *(agreed with
   freemapper-print; user decision pending)*.
8. Forest maps (ISOM 2017-2) with their rule values, symbol groups and test maps *(user: later)*. The
   position in this order is open.
9. Indoor maps for gym training and indoor events *(user, 2026-11-08, agreed across the suite)*:
   - freemapper makes the starter project with an OOM indoor symbol set after the IOF draft ISInOM 6.6 (OCAD's
     "Gym" set cannot be copied) and the user's floor plan as template; the fixed gym map is drawn in OOM;
   - stage 1: each apparatus setup is a copy of the gym map in OOM, and each event names its copy;
   - courses and tape barriers (maze and labyrinth formats) belong here, as courses and special objects;
   - here: a map kind "indoor" (scales 1:100 to 1:1000), its own rule profile (WKB distances do not apply),
     overprint sizes after ISInOM as an assumption until it is final;
   - stage 2, optional and after the editor: an "apparatus setup" layer in the editor, stored as map objects
     with OOM symbol codes so freemapper-print draws them like map content.
   - Agreed with freemapper (2026-11-08): `freemapper.toml` says `map_kind = "gym"` for gym maps (changed only
     with notice), and course setting recognises `gym`; map parts `hall`, `courts`, `equipment`, one `.omap`
     per apparatus setup; equipment objects carry stable codes 1900–1989 (freemapper's extension of ISInOM
     6.6); column D descriptions for apparatus use ids `gym.<code>` with German and English names from
     freemapper's published list; a set-up danger area is map content (1980, orange), run-specific barriers
     and dangers are course-setting special objects (purple). Open for the user: course symbol size ×1.5
     (Claude's recommendation, as the map symbols) or ×1.0 (ISInOM size); apparatus dimensions.

**Non-goals**

- Editing the map. That stays in OOM.
- Print layout: passe-partout, logos, masks, print scale per class and sheet. That is freemapper-print
  (ADR 0015 there).
- Result evaluation and start list draw. That stays in OE. freemapper-courseplan delivers the data those
  programs need.
- A copy of OCAD's or Purple Pen's user interface.

**Anti-goals.** The user named none and expects every problem to be solvable, with the route analysis as the
hardest one *(user)*. Proposed *(Claude's call)*: the program fails even if it technically works when

- an exported code or control position differs from what the editor shows (the failure the Berlin guide
  warns about), or
- a length or climb on the map differs from what WKB B 7.2.5 would give, without the program saying why.

## 3. Constraints and decided choices

ADRs are written in M0 *(Claude's call: the same conventions as freemapper-print, so the suite stays
uniform)*.

| Topic | Choice | Source |
|---|---|---|
| Language | Rust, edition 2024, pure-Rust core, one static binary per platform | Claude's call, as freemapper-print ADR 0001 |
| Licence | GPL-3.0-or-later | Claude's call, as freemapper-print ADR 0002 |
| Terms and languages | glossary first, English code, bilingual user docs (German first) | Claude's call, as freemapper-print ADR 0003 |
| Interfaces | CLI, MCP over stdio, local browser editor | user |
| Editor | local browser page served by the binary, security rules as freemapper-print ADR 0005 | user |
| Dependencies | at least two weeks old; toolchain pinned | Claude's call, as freemapper-print ADR 0006/0007 |
| Project file | one readable file per event for editor, CLI and MCP | Claude's call, as freemapper-print ADR 0008 |
| Course content vs. sheet | course setting owns everything that is the same on every sheet | agreed with freemapper-print, ADR 0015 there |
| Number position | direction in degrees (0 = east, counter-clockwise) and gap in print mm | agreed with freemapper-print |
| Coordinates | OOM map coordinates (mm at map scale, y down), as freemapper-print | agreed with freemapper-print |
| Rule source | WKB 2026 in the first version; an IOF profile later, the rule data is shaped for it | Claude's call, narrowed after the review |
| Program language | German and English for every user-facing text (editor, CLI, findings, MCP texts), in Fluent files so further languages can be added; chosen by `--lang`, `FREEMAPPER_LANG` or the system locale | user, 2026-11-08; ADR 0021, 0022 |
| AI judgments | through the `typesafe-ai` skill's approach (TypeSafe), per the user's CLAUDE.md | user's standing instruction |

## 4. Concepts

```
Event (project file)
├── map source         sanssouci.omap (map scale 1:4000, map kind sprint)
├── rule profile       wkb-2026  |  iof-2026
├── controls           31, 32, … (position, code, description, number position)
├── special objects    start(s), finish, marked routes, crossing points, forbidden areas …
├── courses            "A", "B", … (kind, control sequence, bends, number overrides, climb)
├── classes            H21 → A, D21 → B, …
└── later: tours       control-placer tours
```

### 4.1 Control

A point in map coordinates with a code, an IOF control description (columns C–H; A and B follow from the
course and the code) and a central number position. Codes follow the active rule profile (WKB: > 30 and
< 600, not 66/99, 68/89, 86/98). A control used by several courses has one position and one description.

### 4.2 Course

A course has a kind and a sequence of course controls. Each course control references a control and may
override the number position and add leg bends and gaps.

**Planned print scale** *(Claude's call, after the review)*. Each course carries a planned print scale
(default: the map scale) and optionally a planned print area. Course setting needs them for the checks that
depend on the scale (15 m rule at 1:3000/1:4000, map edge), for overprint sizes in number placement and for
the `.ppen` export, which stores both per course. They are a planning value, not the print decision:
freemapper-print offers them as layout defaults and its layout decides (freemapper-print §5.3), as agreed in
ADR 0015 there.

### 4.2a Discipline and classes

*(Claude's call, after the review)*

- The event has a discipline: `sprint`, `middle`, `long`, `ultralong`, or `training` (no table checks).
- Classes are free names, so they match OE's class names exactly and club training can use names like
  "Anfänger". A class can be mapped to a WKB category (H21, D-12, …). The length table (Anlage 5) and the
  difficulty check run only for mapped classes and only for non-training disciplines; for the others
  `check` says which checks it skipped and why.
- For the first-control check (§5.6), the expected number of starters per class and the start window are
  typed in per event. Importing them from OE or an IOF XML EntryList is later.

### 4.3 Course kinds

The first version builds only `individual`. The data model contains all four kinds from the start, so the
project format and the later shared crate do not change shape *(Claude's call, user agreed to start with
individual)*.

| Kind | Structure | First version |
|---|---|---|
| `individual` | ordered controls, start to finish | yes |
| `team` | ordered mandatory controls (labelled A, B, C …) plus a set of free controls; free controls count only between the first and the last mandatory control; each free control by at least one member; team size per class (3 nationally, 2 in some states); team time is the last member's | model only |
| `relay` | a course family: shared parts and forks with variants, assigned to legs and teams | model only |
| `score` | a set of controls with values, a time limit and a penalty rule | model only |

Rules for `team` are in WKB 2026 B 3.3.2.3 and research §3.6. On the map and in the control description,
mandatory controls show letter and code, free controls show the code only, sorted by code, without a
sequence number.

## 5. Architecture

Workspace with three crates *(Claude's call)*:

- `crates/courseplan-model`: the event data model and the project file reader and writer, without map or
  rendering dependencies. This is the crate freemapper-print will use later instead of a third reader. No
  API stability promise until freemapper-print adopts it (user decision pending, §2 item 7) *(Claude's call)*.
- `crates/freemapper-courseplan`: library with readers (`.omap`, `.ppen`), writers (IOF XML 3.0, `.ppen`),
  geometry, checks, number placement and map rendering for the editor.
- `crates/freemapper-courseplan-cli`: binary `freemapper-courseplan` with CLI, MCP server and the editor's
  web server and page assets.

### 5.1 Map input and display

- `.omap`/`.xmap` reader for symbols, objects, colours and georeferencing. freemapper-print builds the same
  reader in its M1. *(Claude's call)*: take it from freemapper-print once it exists, the way freemapper-print
  copied its renderer from freemapper (ADR 0014 there). If freemapper-print's reader is not ready when this
  project starts M1, start from the MIT crate `omap` (omap-rs) or freemapper's `omap/model.rs` and align later.
- The editor shows the map as the renderer's own output. *(Claude's call, after the review)*: there is one
  renderer in the suite, freemapper-print's (copied from freemapper, ADR 0014 there). It writes PDF only and
  does not exist yet. freemapper-print's own editor needs a browser preview too, so the browser backend (SVG,
  or raster tiles at the editor's zoom levels; the builder chooses) is built once, inside that renderer, and
  this project takes it over like the `.omap` reader. No third copy of freemapper's renderer *(agreed with
  freemapper-print, its design §5.2 and M1 plan task 7)*. freemapper-print's M1 renders into a neutral
  `RenderedPage { primitives, clip, page_mm, colours }` in page millimetres; the PDF writer consumes only that,
  so an SVG or tile backend builds on it without touching the drawing code. freemapper-print builds the
  browser backend by its M6 at the latest and moves it forward if this project's M4 comes first; tell that
  session in time. Shared crate names *(agreed with freemapper-print)*: `freemapper-render` (renderer and
  `.omap` reader) and `freemapper-placement` (number placement, freemapper-print M3); until extraction they
  live in `crates/freemapper-print/src/{omap,render}`. Texts use the embedded font Arimo (freemapper-print ADR 0016); an SVG backend ships it via
  `@font-face`, so preview and print match.
- Consequence for the order: M1–M3 need no map rendering (model, reader, checks, exports work on data). The
  map display arrives with the editor in M4.
- The map stays a live link: the project file names it, and it is read again on every open, check and export.
  The course setter may still edit the map in OOM.

### 5.2 Control descriptions

- IOF Control Descriptions 2024. Symbol geometry from Purple Pen's `symbols.xml`, as freemapper-print plans
  (§5.3 there); its licence is confirmed before use.
- The editor offers the symbols per column. *(Claude's call)*: when a control is placed on a map object, the
  editor proposes column D from the object's ISOM/ISSprOM symbol and warns when the combination makes no
  sense (for example "edge" on a line object, research §3.1).

### 5.3 Number placement

- Levels, the more specific winning: automatic placement, central position per control, position per course
  *(agreed with freemapper-print)*.
- Automatic placement runs when a control is created, when a leg changes and on request. It avoids the
  course's circles, legs and numbers and weighs covered map detail *(Claude's call)*.
- freemapper-print develops the shared placement module (§5.7 there) and extracts it later. Until then this
  project uses a simple placement of its own behind an interface that the shared module can replace
  *(agreed with freemapper-print)*.

### 5.4 Length and climb

- Length per WKB B 7.2.5 and IOF-R 16.3 (research §3.3). The detours around forbidden and impassable areas
  are given by the leg bends the course setter draws in the first version; the route analysis computes them
  later.
- Climb: entered per course in the first version; computed on the predicted route once the route analysis
  exists.
- *Elevation data for later* (decided with the route analysis). Answer of the freemapper session
  (2026-11-08): DGM tile URLs are recorded only for some states and are no exchange format, so the route
  analysis does not reload heights from URLs. Good sources are the elevation model in the freemapper project
  folder (`data/dgm1.tif`, Float32 GeoTIFF in the map CRS, 1 m, NoData NaN or −9999; `data/dom1.tif` is the
  surface model) or calling freemapper as a library or by CLI/MCP. For heights outside the map area
  freemapper offers a stable command (for example `freemapper heights --bbox …`) on request. Stable fields of
  `freemapper.toml`: `map_kind`, `scale_denominator`, `crs`, `provider`, `title`, `[map_area]`.

### 5.5 Import and export

- **`.ppen` import**: controls, codes, descriptions, courses, bends, gaps, number positions, special objects.
  Purple Pen writes the y axis upward against the standard (found by freemapper-print); the reader detects
  it. Print area and print scale are kept only as hints for freemapper-print.
- **`.ppen` export**: everything the model can express in `.ppen`, so freemapper-print and Purple Pen read
  the event. Formats Purple Pen cannot represent (team) are refused with a clear message.
- **IOF XML 3.0 export**: see §5.5b.

### 5.5a Synchronisation with Purple Pen

*(user: parallel use until the move, exchange with Purple Pen users in the long run. The mechanism is
Claude's call.)*

- **Linked file.** An event can name one `.ppen` file as its Purple Pen partner. `sync` reads it, merges,
  and writes it back; `import` and `export` stay available as one-way steps.
- **Matching.** Controls match by code, courses by name, as freemapper-print does (§5.3 there). Purple Pen's
  internal ids are kept as a second key, so a renamed course or a changed code is recognised as a change,
  not as a deletion plus a new object.
- **Three-way merge.** After each sync the project keeps a copy of the `.ppen` as it was then (the sync
  base). On the next sync each object is compared in three versions: base, Purple Pen now, courseplan now.
  Changed on one side only: taken over. Changed on both sides differently: a conflict, listed with both
  values; nothing is overwritten until the user picks a side (editor, CLI or MCP).
- **What stays here.** Data Purple Pen cannot hold stays in the courseplan project and survives every sync:
  accepted findings, class mappings, starter counts, team courses, and later tours and route analysis
  settings. A team course is not written to the `.ppen`; `sync` says so.
- **Format changes.** The `.ppen` reader and writer follow Purple Pen 3.x. Before writing, `sync` checks
  that the file's version is one it knows and refuses otherwise, so a newer Purple Pen file is never damaged.
  Purple Pen 4 (4.0.0 beta) is installed on the author's Mac and already used for small courses *(user,
  2026-11-08)*; the reader and writer cover the 3.x and 4.0 files, and M3 tests the round trip with both.
- **Not in scope.** Live editing by several people at the same moment. Sync is a step someone runs.

### 5.5b IOF XML 3.0 export

- `CourseData`: controls with map and geo position, courses, control order,
  `MapTextPosition`, leg lengths, class assignments. Y axis as the standard defines it.
- **OE**: how OE12 imports courses, and later how it learns which controls are mandatory and which are free
  for "Bahnen mit Aufteilung", is not documented on SportSoftware's site. The builder tests the IOF XML
  export with a real OE import before M3 ends *(open, see §9)*.

### 5.6 Checks in the first version

Rules are data, not code: each rule has an id, the rule profile it belongs to, its values and its source
(document and section), so `check` can say why it complains and the values can be updated when the WKB change
(a complete revision is announced for 2027). Profiles stack *(Claude's call, 2026-11-08)*: a base profile
(WKB 2026, later IOF and others) plus optional regional overlays (for example the NOR recommendations for
Berlin/Brandenburg with Lkm targets, category combinations and ±6 % tolerance, research §3.3a, or Bavaria's
Lkm rules). An overlay adds or overrides values; the event names its base profile and its overlays. All rules here are class A (machine-checkable) and need no
routing *(Claude's call: selection)*.

| Check | Source |
|---|---|
| Minimum distance between controls (30 m; 15 m at 1:3000/1:4000) | WKB26 B 7.2.1 |
| Similar features too close (60 m; 30 m in sprint), similarity from ISOM/ISSprOM symbol and column D | WKB26 B 7.2.1, BOF B6.1.3 |
| Code range and forbidden pairs; confusable codes next to each other | WKB26 B 7.2.10, IOF-SG |
| Several identical objects in the circle without column C | TD11 |
| Column G "edge"/side on a line object | TD11 |
| Acute angle at a control (dog leg) and opposing traffic on overlapping legs | WKB26 B 7.2.6, BL12, SOA |
| Straight leg line through a forbidden or impassable area without bends | WKB26 B 7.2.6, IOF-R 17.2 |
| Leg line through start, finish or another control | BL02 |
| Control too close to the map edge | BL02 |
| Several courses share a first control: maximum starters per start minute against the planned start window *(user criterion)*, and the angle between first legs that share a direction | user, SOFT |
| Course length against the class table of the rule profile | WKB26 Anlage 5 |
| Classes of difficulty level 1–2 on a course of level 3–5 | WKB26 Anlage 5 |

`check` reports each finding with control or leg, rule id, source and a short explanation. Findings can be
accepted with a note, so a deliberate exception stops being reported *(Claude's call)*.

### 5.7 Project file

TOML, as in freemapper and freemapper-print *(Claude's call)*. A sketch, not a schema:

```toml
[event]
title = "Vereinstraining Sanssouci"
date = 2026-11-14
rule_profile = "wkb-2026"

[sources]
map = "maps/sanssouci.omap"

[[controls]]
code = 31
map_mm = [12.4, -33.1]          # OOM map coordinates, y down
description = { d = "1.8", g = "11.2" }   # IOF 2024 symbol ids per column
number = { direction = 45, gap_mm = 1.0 }

[[courses]]
name = "A"
kind = "individual"
classes = ["H21", "D21"]
climb_m = 35                    # entered by hand until the route analysis exists
controls = [
  { code = "S1" },
  { code = 31, number = { direction = 270 } },
  { code = 32, bends = [[20.0, -30.0]] },
  { code = "F1" },
]

[[accepted_findings]]
rule = "wkb26.b7.2.1.min_distance"
controls = [31, 44]
note = "Control 44 is on a different level, checked on site."
```

Paths are relative to the project file. The editor keeps comments it did not create.

### 5.8 Interfaces

- **CLI**: `init` (event from a map, or from a `.ppen`), `check`, `import`, `export` (`--iof-xml`, `--ppen`),
  `sync` (with the linked `.ppen`), `edit` (opens the editor).
- **MCP** (stdio): tools mirror the CLI, plus tools to list and change controls, courses and classes, and
  read-only map queries: objects and symbols near a point or in a region, the map extent *(Claude's call,
  after the review)*. The queries serve the column D proposal (§5.2) anyway; with them an agent can also
  pick control sites and set a course from a description. That use is possible in the first version but
  untested; the tested AI checker and proposal mode stay later (§6). Results are compact JSON.
- **Editor**: left the event (courses, classes, controls, special objects); centre the map with the selected
  course or all controls; right the properties of the selection and the check findings for it. Click on the
  map places a control, dragging moves controls, bends and numbers. *(Claude's call: layout follows
  freemapper-print's editor, so both feel the same.)*

## 6. AI support

- **Optional** *(user, 2026-11-08, for the whole suite)*: AI features inside the application are active only
  when a model is configured. Without one, every feature of the program works; AI findings and proposals are
  simply absent. Nothing in the editor, CLI or checks waits for or requires a model. The MCP server is not an
  AI feature in this sense: there an external assistant drives the program, and the program calls no model.
- Hard rules stay code (§5.6). Optimisation problems (team split, control-placer tours, route search) stay
  algorithms. AI is for judgment: route choice quality, difficulty for a class, whether a control site is
  unambiguous, and proposals *(Claude's call, consistent with the user's "test first")*.
- AI work starts after the route analysis, because the computed routes and times are the material the
  judgments rest on.
- Per the user's standing instruction, AI judgments are built with TypeSafe's typed judgments (`typesafe-ai`
  skill) instead of ad-hoc prompt-and-parse code.
- Test basis: courses with known assessments, for example legs scored with IOF's sprint leg scoring
  (IOF-SG App. 2: 0–3 points per leg) and TD reports. A judgment ships only when it agrees with the test
  basis well enough; the threshold is set when the test basis exists.

## 7. Testing and quality

- Readers and writers test-first with small fixtures and real files: `.ppen` from Purple Pen 3.x, IOF XML
  round trip, and an IOF XML import into OE. Real `.ppen` files from the author (Purple Pen 4.0.0) become
  fixtures only with the author's consent, with absolute paths removed.
- Sync: a `.ppen` written here opens in Purple Pen unchanged, is edited there and merges back; one test per
  merge case (one side changed, both sides changed, deleted on one side, renamed).
- Each check rule has a positive and a negative fixture.
- Length: fixed cases computed by hand against WKB B 7.2.5.
- Example events in `examples/`, built on freemapper's examples, as freemapper-print does (its ADR 0013):
  project file, IOF XML and `.ppen` exports, check findings, later the print PDF. The first one, a sprint
  training on the Lübeck old town map, comes after M3 *(user, 2026-11-08; ADR 0018)*.
- fmt, clippy `-D warnings` and tests in CI on Linux, macOS and Windows, lefthook pre-commit, as in the
  sibling projects.

## 8. Milestones *(Claude's call)*

1. **M0** Repository, docs, glossary, ADRs, CI.
2. **M1** Data model crate with all four course kinds, project file, `.omap` reader (objects and symbols, no
   rendering).
3. **M2** Controls, descriptions, special objects and individual courses through CLI and MCP; length;
   number positions with simple placement; `check` with the rules of §5.6.
4. **M3** `.ppen` import, export and sync with three-way merge (§5.5a), IOF XML 3.0 export, tested with
   Purple Pen, freemapper-print and OE.
5. **M4** Browser editor, with the map display from the shared renderer's browser backend (§5.1).
6. **Later** in the order of §2: route analysis, team, relay, score, control-placer tours, AI checker and
   proposals, shared data model crate.

The builder may start the editor earlier if placing controls through the CLI turns out too awkward.

## 9. Open points

- **OE import** (§5.5): how OE12 reads courses and, for team orienteering, mandatory and free controls.
  Answer by a test import; the user has OE *(user: results usually in OE)*.
- **Elevation data** for the route analysis (§5.4): decided with the route analysis.
- **Rule values to verify in the original documents** before they become data: overprint line widths of
  symbols 702, 709, 710; IOF texts were read from mirrors (research §5).

## 10. Left to the builder

- Module layout inside the crates, crate choices within the dependency rule, the exact project file schema.
- The map display format in the editor (SVG or raster tiles), within the shared renderer (§5.1).
- The simple number placement until the shared module exists.
- Whether the `.omap` reader comes from freemapper-print, omap-rs or freemapper (§5.1).
