# Score courses: design

Status: draft for approval, 2026-11-11. Extends §4.3 and "Later, in this order" of
`2026-11-08-freemapper-courseplan-design.md`. Team and relay courses follow in their own designs, in this order
*(user, 2026-11-11: score, relay, team)*.

Marks: *(user)* is a decision or statement of the user, *(Claude's call)* is my proposal.

## 1. Intent

A score course lets runners visit controls in any order within a time limit; each control is worth points. The
user sets such courses for one group of runners on sprint and schoolyard maps. Purple Pen and OE only record a
course the setter has already built. This program should help build it: it proposes control sites, assigns the
points, and computes how long the course has to be *(user)*.

## 2. Goals, non-goals, anti-goals

Goals *(user)*:

- Propose control sites, as a ranked candidate list and as a complete draft built from it. The setter can work at
  either level and edit by hand at both.
- Assign sensible points to the controls.
- Compute the optimal length of the course for the time limit.
- Mix easy and hard controls.

Non-goals *(Claude's call)*: several classes with different limits in one score course; team score and rogaining;
elevation (the first travel-time model has none); the results software; the full route analysis (§4.1 only
builds its first part); forest maps (ISOM is refused as before).

Anti-goals *(Claude's call, the user named none)*: a proposal that puts a control where the setter cannot go or
that the rules forbid (out of bounds, dangerous area, off the map); a draft that looks finished but reaches its
numbers through unstated assumptions; a changed project file without the setter asking for it.

## 3. Rules of the format *(user, unless marked)*

- One group of runners, one time limit: 45 or 60 minutes, set per map; any value can be entered.
- Penalty for exceeding the limit: a clear deduction of points per started minute. The size is a setting. The
  default is a value that makes a minute over the limit never pay off (Claude's call, §5.4).
- Ties in points are decided by the faster runner. This is a rule of the evaluation; the program records it in
  the course data and the evaluation sheet and does not use it itself.
- Point values 1 to 5 by default, up to 9 are allowed; setting the range is a project setting.
- The code carries the value: the second digit of a two-digit code is the value (31 → 1, 33 → 3, 49 → 9,
  55 → 5). For longer codes the last digit is the value *(Claude's call)*. The tens digit tells controls of equal
  value apart; with values 1 to 5 and tens 3 to 9 that allows 35 controls, with 1 to 9 up to 63.
- The map shows the controls with their number and no value; the value is read from the code *(Claude's call,
  confirmed in conversation)*. The number label of the map may add the code, as in §4.3 of the main design
  (`1-31`).
- Two runner profiles *(user)*: a good runner covers a kilometre in 4:00 to 4:15 and a very good one in under
  3:45. Defaults 4:10 and 3:45 min/km (Claude's call), both settings.
- Optional **mandatory links** *(user)*: a score course may name pairs of controls that have to be visited
  directly one after the other; either direction counts. A course has none, one or several. A link is a leg the
  runner must run, so route choice on that leg matters like on any other (§4.2). A runner who does not run a link
  directly is disqualified *(user)*; the evaluation sheet and the IOF XML course data name the links, so the
  results software or the person at the finish can check them. Whether links may share a control
  (A-B and B-C, so that B needs both neighbours) is the builder's decision; I allow it only if the set stays
  runnable (a chain, no branching at a control) *(Claude's call)*.
- Target for the length *(user)*: the good runner scores about 90 percent of all points within the limit; the very
  good runner may score all of them, just under the limit.

## 4. How it works

### 4.1 Travel time (first part of the route analysis) *(user: simple start; Claude's call: details)*

A cost surface over the map: a speed per map symbol, impassable symbols blocked, out-of-bounds and dangerous
areas of the event blocked. A forbidden route is a line, not an area: sites within about 4 m of it are rejected,
but a route may cross it, as `check` treats it for legs and links (ADR 0068, *(Claude's call)*). No elevation. The surface is a raster at a resolution
the builder chooses (a few metres on the ground). The builder decides how paths are searched.

The surface returns for two points: the fastest route and up to a few clearly different alternatives, each with
length and time. "Clearly different" means they share only a small part and are not one route shifted by a
pixel. Route choice is the central quality of a control for the user ("many route choices, similar in length, only
occasionally one a bit clearer"), so alternatives are part of the first step.

The travel-time interface sits behind a trait or a plain function boundary, so the later full route analysis (with
elevation and drawing on the map) replaces it without touching the score code *(Claude's call)*.

Speed table: a default per symbol class from freemapper-core, if core has one, otherwise a table in this project
as data (like the rule profiles). The runner profile (§3) scales it.

### 4.2 Control candidates

Candidate sites come from the map objects that make a clear, fair control: point and small area features such as
steps, wall corners, benches, fountains, corners of buildings *(Claude's call, the user named no features)*. The
builder checks what freemapper-core's object classes give. A candidate must lie on a passable place, not in an
out-of-bounds or dangerous area, and keep the minimum distances of the rule profile (`wkb-2026`) to other
controls, to start and to finish.

Each candidate gets a score for:

- **Route choice** *(user)*: for the legs to the other controls and to the start, how many clearly different
  routes of similar length exist, and how often one is somewhat clearer. A candidate is good when most of its legs
  offer several similar routes and a few are clearer.
- **Difficulty** *(Claude's call)*: from the same legs, plus the clarity of the feature and the neighbours in a
  radius (crowded areas are harder).
- **Effort**: the travel time from the start.

### 4.3 Points

Points follow the effort: travel time from the start plus difficulty. A hard control brings more than an easy one
at the same distance, but never so much that only hard controls pay off *(user: yes)*. The effort maps to five
steps (1 to 5 by default) by quantiles of the candidate set or fixed limits; the builder decides, the tests fix
the behaviour. The value then fixes the code (§3), so the setter sees both together.

### 4.4 Length and the draft

Input: the candidate list, the limit, the two profiles, the target (§3). The program searches a set of controls
and an order for the good runner, so that the best round trip within the limit scores 90 percent of the sum, and
the very good runner's best round trip reaches the sum just under the limit. If both cannot hold at once, it says
which of the two is missed and by how much, and what would help (limit, number of controls, profile). It does not
shrink the target by itself *(Claude's call)*.

The mix of easy and hard controls is a setting (a share per step, default evenly spread) and bounds the search
*(user: mix; Claude's call: share setting)*.

Mandatory links are constraints of the round trips: both runner profiles must be able to run every link, and the
trip time includes it. The draft can place links by request (the setter names the pair) or propose them, off by
default *(Claude's call)*; a proposed link is a leg with several similar route choices and a length that fits
both profiles.

The draft also keeps the checks of §6 as constraints. The result is a score course in the project file; the
setter can open it in the editor and move or replace controls as with other courses. Which of the two search
tasks (set selection, round trips) uses an exact or a heuristic method is the builder's decision, within the
size limits: 20 to 60 controls, run time in seconds *(Claude's call)*.

### 4.5 Interfaces

The same operations are available through CLI, MCP and the editor, following ADR 0059 (one op function per
tool). The builder names the commands; sketch *(Claude's call)*: `score suggest` (candidate list with scores),
`score draft` (complete course), `score length` (analysis of an existing course), `course add --kind score`.
Suggestions are returned as data and applied only on request; nothing writes to the project file unasked. In the
editor the candidates show as markers on the map with their scores, a course draft as a course.

## 5. Data model, project file, export

1. `CourseKind::Score` exists (`time_limit_min`, `penalty_per_min`, `controls: Vec<ScoreControl>`). It needs the
   point range (default 1 to 5), the profiles and the mix as event or course settings; the builder extends the
   project file with a format version step (ADR 0006 rules do not touch this).
2. The value is stored with the control (`ScoreControl::value`) and also derivable from the code; `check` reports
   a difference (§6).
3. IOF XML 3.0: `CourseControl` can carry `Score`; the export writes score courses. It was not tried in OE; the
   change log and AGENTS.md say so as for the individual export. The tie rule is not part of the schema; it goes
   into the evaluation sheet.
4. Penalty default *(Claude's call)*: points per minute `p = max value of the course`, so that one more started
   minute never makes up for the best control at the end. The setter can set any number.
5. Purple Pen: Purple Pen has its own score course type; the `.ppen` export and sync of score courses are not
   part of this step and keep skipping them with the message, until the builder checked how Purple Pen stores
   them *(Claude's call)*.
6. A CSV with code, value, position (map millimetres) and an evaluation sheet (printable table) for hand
   evaluation; the later results software reads IOF XML *(user: evaluation by hand or a possible freemapper-result;
   Claude's call: formats)*.

## 6. Checks

New rules in the check engine for score courses, each with a source (ADR 0026 form). Sources are BOF B12 and the
IRF rules unless marked *(user)*:

- Value and code agree (§3) *(user)*.
- Mandatory links: both controls exist in the course; no link joins a control to itself or repeats; a link is
  not blocked by an out-of-bounds area or a forbidden route; its leg has route choice or is flagged; shown on the
  map as a marked leg *(Claude's call)*.
- Code ambiguity (6/9 reversible, equal tens digit and value) *(Claude's call)*.
- No ring of controls around start and finish; no obvious order (convex hull and round-trip analysis).
- Nobody can score all points: the good runner's best round trip within the limit scores about 90 percent, the
  very good runner's lies just under the limit *(user)*. Both limits are settings.
- Time limit in the usual range and the penalty makes overrunning unprofitable.
- Every control keeps the distances of the rule profile; every control lies on the map.
- The mix of easy and hard controls matches the setting.

## 7. Process

1. Travel-time surface with alternatives, as a library with tests on small hand-built maps.
2. Candidates and their scores.
3. Points and codes.
4. Length search and the draft.
5. Checks, CLI, MCP, editor.
6. Export (IOF XML, CSV, sheet) and documentation (glossary rows `score course`, `control value`; ADRs).

Each step stays releasable on its own; step 1 and 2 are internal until step 4 ships *(Claude's call)*.

## 8. Left to the builder

The raster resolution and the path search; the default speed table; the object classes that count as candidates;
how difficulty and effort are weighted; quantile or fixed steps; the search method; the exact command names; the
format version step; translations of all texts through Fluent files, English and German.

## 9. Rulings from the builder check *(Claude's call, all)*

1. **Tolerances.** "About 90 percent" is a band: the good runner's best round trip within the limit scores 87 to
   93 percent of all points. "Just under the limit" for the very good runner: all controls, finishing in 95 to
   100 percent of the limit. Both bands are settings. The search treats them as hard constraints; a miss is
   reported with its size (§4.4). `check` uses the same bands as its warning limit.
2. **What the length search changes.** `score draft` chooses the set of controls from the candidate list and
   may add or drop candidates to meet the bands. `score length` only analyses an existing course and reports; it
   never changes it.
3. **Points and codes.** The value comes from the effort (§4.3). The mix share (§4.4) only bounds which
   candidates enter the set; if the two conflict, the effort wins and the report names the missed share. The order
   is: choose the set, set the values, then assign codes (tens digit per value) under the distance and ambiguity
   rules. Steps 3 and 4 of §7 therefore build on each other, but the code assignment stays a separate function.
4. **Profiles.** One course, one set; both profiles are checked on it. The search is two round-trip solves on one
   set, not two sets.
5. **Route-choice numbers.** Overlap limit and length ratio for "clearly different" and "similar" are project
   settings with defaults the builder sets. Tests use hand-built maps; for the acceptance of the candidates see
   the open question in §10.
6. **Project file.** `CourseKind::Score` already round-trips. The new fields are optional with defaults, so older
   files load unchanged and need no migration; the format version rises only if the builder adds a required
   field.
7. **Editor scope.** Step 5 shows candidates as markers with their scores, one panel to run a suggestion, accept
   single candidates, and run the draft with a progress line. No new editing tools beyond the ones for other
   courses.
8. **Determinism.** The search is reproducible: a fixed seed (a setting, default constant) and a stable order, so
   `check`, MCP and CLI give the same answer on every run.

## 10. Open for the user

- Answered *(user, 2026-11-11)*: missing a mandatory link means disqualification. The user will look at the
  proposed candidates on the Lübeck map once step 2 exists and name those they would never use.
- Answered *(Claude's call, 2026-11-11, ADR 0069)*: the editor draws a link as a dashed line between the two
  controls, in the marked-leg style, and it is no leg. The control description does not mention it.
- Acceptance of the candidates *(user: yes)*: the Lübeck sprint example (`examples/luebeck-sprint`); the proposals
  are shown once step 2 exists, the rejected ones become test cases.
- Whether the value should be shown on the map in a later event setting (the field exists, the default is off).
- Team and relay courses reuse the cost surface (§4.1); their designs follow.
