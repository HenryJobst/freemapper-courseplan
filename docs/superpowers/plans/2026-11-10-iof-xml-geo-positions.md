# IOF XML Geo Positions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The IOF XML export writes latitude and longitude for every control (`Position`) when the map is georeferenced, as IOF XML 3.0 allows next to `MapPosition`.

**Architecture:** freemapper-core 0.7.0 offers `freemapper_course::MapGrid` (map mm, y down, in the `.omap` frame ↔ lat/lng). `export_iof_xml` takes an optional `&MapGrid`; the CLI and MCP build it from the map's `MapGeoref` and pass it, or pass `None` with a note when the map has no usable georeferencing. Project coordinates are already in the `.omap` frame (mm, y down), so there is no flip.

**Tech Stack:** Rust 1.99.1; `freemapper-course` v0.7.0 (git tag, feature `fit` on).

**Spec:** `docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md` §5.5b (`CourseData`: "controls with map and geo position"), ADR 0033.

## Global Constraints

- Edition 2024, `unsafe_code = "forbid"`, fmt/clippy `-D warnings`/tests pass; no new third-party dependency; no copy of core code.
- Texts in en and de Fluent; prose through `unslop` and `humanizer`; ADR 0039 *(Claude's call)*; changelog de+en; both READMEs equal.
- Output stays deterministic and valid against `tests/fixtures/iof-xml/IOF.xsd` (element order in `Control`: `Id`, `Position`, `MapPosition`; `lat`/`lng` attributes with 6 decimals).
- Crossing points and starts/finishes get a `Position` as well.

## Review Focus

- Map without georeferencing or with an unsupported CRS: the export still works, writes no `Position`, and says so once in the output.
- Round trip: the exported lat/lng, read back through `freemapper_course::read_iof_xml` and `MapGrid::map_mm_of`, lands within 0.01 mm of the control's position.
- Sign and frame: a control east/north of another has larger lng/lat (y down in the project).
- A project whose map changed since the file was made: positions follow the current map.
- Existing IOF tests, snapshot and example stay valid; the example file changes only by the added `Position` elements.

---

### Task 1: `Position` in `export_iof_xml`

**Files:**
- Modify: `Cargo.toml` (core crates `tag = "v0.7.0"`), `Cargo.lock`, `crates/freemapper-courseplan/src/iof_xml.rs`, `crates/freemapper-courseplan/src/iof_xml/tests.rs`, `crates/freemapper-courseplan-cli/src/export_cmd.rs`, `crates/freemapper-courseplan-cli/src/mcp.rs` (only if it builds the call itself), `crates/freemapper-courseplan/i18n/{en,de}/courseplan.ftl`, `crates/freemapper-courseplan/tests/snapshots/iof_xml_luebeck.xml`, `examples/luebeck-sprint/course-data.xml`

**Interfaces:**
- Consumes: `freemapper_course::MapGrid::{of(&MapGeoref) -> Result<MapGrid, SkipReason>, latlon_of((f64, f64)) -> (f64, f64)}`.
- Produces: `pub fn export_iof_xml(event: &Event, map_scale: u32, map_kind: MapKind, geo: Option<&MapGrid>, created: &str) -> Result<IofXmlExport, IofXmlError>`; `IofXmlExport` gains `pub geo_positions: bool`. Message `export-iof-xml-no-geo` (printed once when `geo` was `None`).

- [ ] **Step 1: Pin v0.7.0**, `cargo update -p freemapper-course` and the other core crates, `cargo test --workspace --locked`; expect PASS.
- [ ] **Step 2: Write failing tests** in `iof_xml/tests.rs`: `controls_carry_a_position_when_the_map_is_georeferenced` (Lübeck map: every `Control` element has `<Position lat=… lng=…/>` before `MapPosition`, six decimals), `geo_positions_read_back_to_the_map_position` (read the export with `read_iof_xml`, `MapGrid::map_mm_of(lat, lng)` within 0.01 mm of the project position), `east_and_north_increase_lng_and_lat` (two controls), `no_position_without_georef` (`geo = None`: no `Position`, `geo_positions == false`), `output_validates_against_schema` stays green with positions. Update call sites for the new parameter.
- [ ] **Step 3: Implement** the parameter and the element; in `export_cmd.rs` build `MapGrid::of(&doc.georef).ok()` once and print `export-iof-xml-no-geo` when `None`. Regenerate snapshot and example (`UPDATE_SNAPSHOT=1`).
- [ ] **Step 4: Run** fmt, clippy `--locked`, `cargo test --workspace --locked`; `xmllint` the example against the schema.
- [ ] **Step 5: Commit** `feat(iof): write Position for georeferenced maps`.

### Task 2: Documentation and release

**Files:** `docs/adr/0039-iof-xml-positions-from-the-map-georeferencing.md`, `docs/adr/README.md`, design §5.5b, `README.md` + `README.en.md`, manual de/en, examples READMEs, both changelogs.

- [ ] **Step 1: ADR 0039** (~25 lines): `Position` from `MapGrid`, no flip, six decimals, no georef → no `Position` and a one-line message, still untested in OE/OCAD/Condes.
- [ ] **Step 2: Update the documents** in German and English; remove "geo positions are not written yet" wherever it stands.
- [ ] **Step 3: Review, merge `--no-ff`, release 0.8.0** per CLAUDE.md.
