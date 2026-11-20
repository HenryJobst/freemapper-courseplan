# 0099. Mutation tests on the diff, property tests, nextest, SBOM and a complexity limit

Date: 2026-11-20
Status: Accepted

## Context

ADR 0098 added the housekeeping gates. The next candidates measure what the tests prove, how
fast they run and what ships. Each was measured here first (macOS, Apple silicon, warm `target`)
and is adopted or dropped by that number. The verdicts follow freemapper-print ADR 0072 and
freemapper-core ADR 0040, and differ where the numbers differ.

## Decision

- **Mutation tests on the diff: adopted.** `scripts/check.sh mutants` runs `cargo-mutants 27.1.0`
  with `--in-diff` over the Rust lines the branch changed against the base branch, in its own CI
  job on pull requests. A whole-workspace run is not a gate; the diff is. `cargo-mutants` copies
  the tree without `.git`: a test that reads the repository's history, or a path above the
  workspace, fails the baseline there (exit code 4) and is a bug in the test. A mutant that
  survives is fixed with a test, not with an exclusion (measured on the `ControlCode` code: 16
  mutants, 2 min, 10 caught, 5 unviable, 1 missed: the error text of the reader, now tested); `mutants.out/missed.txt` names it. A
  mutant that makes a loop endless ends in a timeout (exit code 3), which counts as caught.
- **Property tests: adopted at one place.** `proptest` checks `ControlCode`, whose `Serialize`
  chooses between a TOML integer and a string: any text a code holds (plain numbers, leading
  zeros, 20 digits, quotes, line breaks, any Unicode) is written and read back as the same
  code. The format's `toml` round trip of whole projects is covered by the existing fixtures;
  a generator for valid projects would be a second schema.
- **cargo-nextest: adopted.** `cargo test` took 1016 s, `cargo nextest run` 505 s on the whole
  workspace, because it runs the test binaries side by side. Another cargo process was running
  on the machine for both runs, so the ratio is the figure to trust, not the seconds. The
  `check` job runs `cargo nextest run` and `cargo test --doc`; the two stub-server test
  binaries keep their `cargo test` step. Pinned to 0.9.152. freemapper-print measured the
  opposite (11:33 min against 8:42 min) and keeps `cargo test`.
- **cargo-hack: adopted as a compile check.** `check --each-feature --no-dev-deps` over the
  features (`test-stub`, `page-rules`, `test-support`) in its own job, 156 s cold. The
  features exist for tests; the powerset would only repeat the combinations.
- **Complexity: adopted as a clippy threshold.** `clippy::cognitive_complexity` with the
  baseline `cognitive-complexity-threshold = 54` in `clippy.toml`; today's worst function
  scores 54 (ten are above clippy's default of 25). `scripts/check.sh complexity` fails on
  anything above. Lowering the number is welcome, raising it needs an ADR.
- **SBOM: adopted.** `scripts/check.sh sbom` renders a CycloneDX 1.5 list of the CLI with
  `cargo-cyclonedx` 0.5.10 and compares it with `docs/sbom.cdx.json`; `sbom --fix` rewrites it.
  The tool writes a random serial number, a time stamp, the build host and absolute paths; they
  are removed, so the same dependencies give the same bytes. `--target all` lists the
  dependencies of every platform: without it macOS and Linux disagree. The gate needs only
  `cargo metadata` and runs with the cheap ones.

## Consequences

- Three new CI jobs (`complexity`, `hack`, `mutants`), one new cheap gate (`sbom`), and the
  badge for the number of checks counts them.
- A change of `Cargo.lock` changes `docs/sbom.cdx.json` in the same commit.
- Tool versions are pinned in `ci.yml`: cargo-mutants 27.1.0, cargo-hack 0.6.46, cargo-nextest
  0.9.152, cargo-cyclonedx 0.5.10, installed with `cargo install --locked --version`.

Source: measured in this repository; freemapper-print ADR 0072 and freemapper-core ADR 0040 for
the same tools there.
