# 0037. Description symbols come from the core crate

Date: 2026-11-10
Status: Accepted

## Context
The import mapped Purple Pen symbol ids to IOF 2024 numbers (`iof_2024`) and the export inverted that table by
enumerating ids (`inverse`, `SINGLE_CAPABLE`). Both lived in courseplan and did not use the symbol
database. freemapper-print needed the same mapping, so it moved into `freemapper-course` as
`description_symbols` (core ADR 0017, v0.6.0): `iof_2024`, `ppen_symbol` and `ppen_columns`, built on the
symbol database that Purple Pen ships.

## Decision
- **One mapping, in core.** The import uses `description_symbols::iof_2024` (with `Mapped` and `BEND_2024`);
  `iof_2024`, `Mapped`, `DIRECTIONS` and `BEND_2024` are gone from `ppen_import.rs`, `inverse` and
  `SINGLE_CAPABLE` from `ppen_export.rs` *(agreed with freemapper-print)*.
- **`ppen_columns` is a thin wrapper.** It turns the `ControlDescription` into the column array, the file's
  standard string into a `Standard` (`from_description`, 2018 for anything else) and the core `ColumnNote`s into
  `ExportNote`s *(Claude's call)*.
- **Instruction rows.** The core mapping accepts the rows 15.x and 16.x in column C but does not know the
  control kind. The wrapper keeps them out of column C of a normal control and reports them; a crossing point
  keeps its one box as before *(Claude's call)*.
- **Notes.** `Refusal::NoPurplePenId` stays `SymbolWithoutPpenId`. New: `SymbolNotInStandard` (the file's
  standard lacks the symbol, such as 2.10 or 15.6 in a 2004 file), `SymbolVariantChosen` (Purple Pen has
  several ids, the export wrote `written`) and `SymbolWrongColumn` (Purple Pen has the symbol, but its popup
  does not offer it in this column; core's `WrongColumn` needs this third note)
  *(Claude's call)*. Fluent texts in English and German.
- **Export fixes that follow from the core rulings** *(agreed with freemapper-print)*:
  - The export writes only symbols the file's standard knows and the column offers.
  - The variant `single` is keyed on column F: the same feature in D and E over `10.1` or `10.2` in F becomes
    `10.1single` or `10.2single` in F, D keeps the feature, E stays empty. The old code tested the id in D,
    which no description has, and so never produced `single`.
  - `12.14` ("between") with D equal to E keeps D and E and writes the plain `11.15` in G, because Purple
    Pen throws on `11.15single` in G.
  - Direction forms are normalised: `0.1` is the cardinal and `0.2` the diagonal form, so `0.1NE` is written
    `0.2NE` and `0.2N` `0.1N`.
  - Plain 9.1 and 9.3 are written `9.1high` and `9.3high`, and the export says so.
  - The bend 11.1 in column F of a 2004 file goes to G when G is free; with G taken it is refused as
    `SymbolWrongColumn` (the old code wrote 11.7 in F, which Purple Pen's 2004 popup does not offer).
- **Tests adapted.** The scenarios stay; these assertions changed with the rulings:
  - `ppen_symbol_inverts_iof_2024`: the enumeration walks core's table and accepts a refusal where a
    symbol does not fit the column or the 2004 standard; it excludes the direction forms `0.1NE` and `0.2N`.
  - `explicit_rows`: `single` is tested with a combination in F; 2004 with the bend in F and G taken
    expects a refusal; `12.14` expects D, E and a plain `11.15` in G.
  - `plain_id_wins_over_variants` is replaced by the core database walk; `descriptions_go_through_ppen_columns`
    uses 5.2 in D and E over 10.1 in F.
  - `round_trip_of_the_fixtures` expects `single` where a fixture has the same feature in D and E over a plain
    10.1 or 10.2 (lincoln-park, control 32): both forms import alike.

## Consequences
- A patch onto a base file keeps the file's own columns where the project is unchanged (ADR 0035); a freshly
  written description can differ from the old file by `single` where the file wrote both columns.
- A change to the mapping is a change to core and needs the agreement of freemapper-print.
- The export reports three new cases instead of writing an id that the file's standard or column does not take.
