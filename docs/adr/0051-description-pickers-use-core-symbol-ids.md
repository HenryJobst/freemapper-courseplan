# 0051. Description pickers list the symbols from core

Date: 2026-11-11
Status: Accepted

## Context
The editor lets a user pick the symbols of columns C to H of a control. The project file stores IOF 2024 numbers
as text (`"1.3"`, `"12.1N"`). The drawings and texts are in freemapper-core's symbol database (core ADR 0017),
which is keyed by Purple Pen ids (the 2004 numbers with variant suffixes).

## Decision
- **The lists are derived** *(Claude's call)*. `GET api/symbols?column=C..H` takes core's `ids(Standard::Iof2018)`,
  keeps the ids that start with a digit, turns each into its IOF 2024 number with `iof_2024`, drops the
  `single` variants and lists each number once, sorted by section, item and direction. The drawing, text and
  column come from the Purple Pen id that `ppen_symbol` chooses for the number, so every listed number is one the
  `.ppen` export can write. Nothing is typed by hand.
- **Columns** *(Claude's call)*. A symbol is offered in the column named by its `symbol_kind`; column E also takes
  the D symbols, as Purple Pen and `ppen_export` do. Column C offers the "which of similar features" symbols only,
  not the instruction rows.
- **Text and drawing** *(Claude's call)*. The text is in the process language with English as fallback;
  placeholders such as `{0}` are shown as an ellipsis. The drawing is an SVG with `currentColor`, built only from
  the geometry's numbers, so no text from a client reaches it. The page parses it as SVG and imports the element;
  it is never inserted as markup text.
- **One pick is one operation** *(Claude's call)*. A pick sends `describe_control` with that column only. The
  operation has no way to empty a single column, so "No symbol" sends `clear` together with the other columns as
  they are, which is still one operation and one undo step.
- **Not decided here.** Proposing column D from the map object stays deferred (plan D16); the editor lists the map
  objects at the control next to the D picker.

## Consequences
Direction symbols exist only with a direction (`12.1N`, not `12.1`); the picker offers exactly those.
