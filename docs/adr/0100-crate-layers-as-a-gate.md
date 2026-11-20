# 0100. The crate layers are a gate

Date: 2026-11-20
Status: Accepted

## Context

The workspace has three crates: `freemapper-courseplan-model` (the course plan as data, read and
written as TOML), `freemapper-courseplan` (the library: maps, planning, rendering, built on the
model and the suite's core crates) and `freemapper-courseplan-cli` (arguments, the MCP and web
servers, the update client, the settings folders). The order held by habit only. A convenient
`quick-xml` or suite crate in the model would tie the plan format to the map reader; a `clap` or
`ureq` in the library would drag a command line or a network client into everything that builds
on it.

## Decision

- **`scripts/check.sh architecture`** reads the workspace with `cargo metadata --no-deps` and
  fails when
  - a workspace member is in no layer (a new crate is a decision, placed in `LAYERS`),
  - a crate depends on a workspace crate its layer may not use: the model on none, the library on
    the model only, nothing on the CLI,
  - the model or the library depends on a crate only the driver may use: `axum`, `clap`, `dirs`,
    `freemapper-update`, `freemapper-web`, `rmcp`, `rust-embed`, `schemars`, `serde_json`, `tokio`,
    `ureq`,
  - the model depends on `quick-xml` or any `freemapper-*` crate.
  Development dependencies are not counted.
- **Measured:** under a second, passes on today's tree. A probe that gave the library a dependency
  on `clap`, and one that gave the model a dependency on `quick-xml`, each failed it with a line
  naming the crate; both were reverted.
- **Cheap gate:** it runs in `hygiene` and before a commit; the badge for the number of checks
  counts it.

## Consequences

- A drive-by dependency against the layer order is a failing gate, not a review remark.
- Moving a crate between layers, or adding one, is an edit of the tables in `scripts/check.sh`
  with a reason in an ADR.
