# 0102. Findings on running tests and mutation tests

Date: 2026-11-20
Status: Accepted

## Context

ADR 0099 adopted mutation tests on the diff, nextest, property tests, cargo-hack, a complexity
limit and the SBOM. Running them since then showed what the first measurement did not: how a
mutation gate ends, what a surviving mutant teaches, what the test run does to the coverage number
and in which order the badges are written. This ADR records those findings; it adds to ADR 0099
and replaces nothing in it.

## Findings

- **Exit codes of the `mutants` gate.** The gate runs on pull requests only and mutates the diff
  only. Exit code 2 (the tool's own error) and 4 (the unmutated baseline fails) fail the job;
  exit code 3 (a timeout) counts as caught, because a mutant that makes a loop endless is
  detected. ADR 0099 states the diff scope, exit code 3 and the missing `.git`; the other two are
  new here.
- **A changed line of arithmetic needs a test that fixes its value.** A test of the edge cases
  is not enough. In freemapper-core pull request #22 three mutants of the stretch calculation
  passed a test that covered only the edge cases. A survivor is closed with a test, never with an
  exclusion (ADR 0099); for arithmetic the test asserts the computed value.
- **Mutation probes check new tests.** For the test-gap pull request #9, 31 new tests were
  mutated by hand: 29 were confirmed, that is, a mutation of the code made them fail. The three
  tests of the sync error texts and two others were confirmed only in part. A new test for
  existing code is worth one such probe.
- **The test run.** The nextest figures (1016 s against 505 s) were measured while a foreign
  `cargo-mutants` process ran on the machine. The ratio of about 2 holds; the seconds do not.
  `cargo test --doc` stays a separate step, because nextest does not run doc tests.
- **A killed server writes no coverage.** An instrumented process writes its coverage data when
  it exits. A test server that the test ends with `kill()` writes nothing. The MCP test sessions
  now close stdin and wait for a clean exit (`tests/mcp_ops.rs`). `mcp_update.rs` and
  `editor_server.rs` still end their server with a kill: this is a known gap of the measurement.
  Part of the rise from 91.0 % to 94.8 % line coverage (1436 tests) is therefore an artefact of
  the measurement, not only a gain from tests. Not covered by tests: `editor/print_layer.rs`,
  `editor/exchange.rs` and `editor/map_view.rs`.
- **Order of the badge files.** `scripts/check.sh tests --fix` first, then `bash scripts/badges.sh`:
  the badges read `docs/tests.json`, and a count that is rewritten later leaves them stale.

## Decision

The decisions of ADR 0099 stand: nextest, complexity limit 54, SBOM, `proptest` for
`ControlCode`, `cargo hack --each-feature`, the mutation gate on the diff with a timeout counted
as caught. In addition:

- A measurement of test time, coverage or mutants does not run in parallel with a foreign
  mutation run on the same machine; a figure taken that way is quoted as a ratio, not in seconds.
- A server that a test starts is ended cleanly (stdin closed, exit awaited) wherever the test
  allows it; a kill is the exception and is named in the test file.

## Consequences

- The coverage number is read with the two `kill()` tests in mind; ending them cleanly would
  raise it without a new test line elsewhere.
- No gate changes. ADR 0099 stays the reference for the tools and their versions.

Source: measured in this repository and in freemapper-core pull request #22; ADR 0099.
