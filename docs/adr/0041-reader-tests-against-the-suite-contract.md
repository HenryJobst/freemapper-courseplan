# 0041. Reader tests against the suite contract

Date: 2026-11-10
Status: Accepted

## Context
freemapper-courseplan reads maps and `freemapper.toml` files that freemapper writes: the map kind, the title, and
which objects are sheet objects or base area and therefore no map features. No test checked that against a file
freemapper had really written. freemapper v0.9.0 tagged the scale bar `freemapper:source=scale-bar`, and the bar
counted as map extent here until `is_sheet_furniture` learned the tag. freemapper-core v0.8.0 holds the contract
as a crate, `freemapper-contract` (core ADR 0018), with two maps exactly as freemapper wrote them and the values
a reader must derive from each. The user asked the freemapper session to write the reader tests here
*(user, 2026-11-10)*.

## Decision
- `freemapper-contract` is a dev-dependency with the feature `fixtures`, on the same core tag as the other core
  crates *(Claude's call)*.
- `crates/freemapper-courseplan/src/contract.rs` holds the reader tests. Each writes every fixture into a
  temporary folder, reads it with courseplan's own code and compares with `expected.toml`:
  - `contract_kind`: `map_kind_for_file` names the kind of the sidecar. `map_kind`, which `export_ppen` uses
    without a path, agrees for both fixtures.
  - `contract_title`: `title_from_config` returns the title.
  - `contract_classes`: `is_sheet_furniture` accepts as many objects as the fixture counts as sheet objects
    and base area together.
  - `contract_extent`: no feature of `MapIndex` lies in the parts `peripheral information` or `base`, and the
    extent stays inside the box of the map content and north of the sheet objects.
- The names start with `contract_`. The gate planned for core's CI runs them by that prefix against a new core
  before it is released *(Claude's call)*.
- `title_from_config` is `pub(crate)` for these tests.
- A failing contract test is a finding about courseplan or about the contract. It is settled with the freemapper
  session and never by loosening the test here.

## Consequences
- With `sheet` removed from `is_sheet_furniture`, `contract_classes` and `contract_extent` fail. That was tried
  before the merge.
- The test build embeds about 540 kB of fixture text. Release binaries are unchanged.
- Moving the core tag now also moves the fixtures. A core release that changes the contract fails these tests
  until courseplan follows.
- `map_kind` without the sidecar agrees only for today's two fixtures. A fixture with a kind the scale does not
  give away (stage 3 of the contract design) will show whether `export_ppen` needs the path too.

Source: core ADR 0018, ADR 0038, the review of freemapper `0f25841`
