# freemapper-courseplan: working rules

Course-setting application of the freemapper suite. Siblings: freemapper (`../freemapper`,
https://github.com/HenryJobst/freemapper) and freemapper-print (`../freemapper-print`). Read
`docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md`, `docs/glossary.md` and
`docs/adr/README.md` before changing code. The rule sources for checks are collected in
`docs/research/2026-11-08-bahnlegung-kriterien.md`.

- Rust, edition 2024, pure-Rust core. Data model and project file in `crates/freemapper-courseplan-model`,
  domain logic in `crates/freemapper-courseplan`, CLI/MCP/editor in `crates/freemapper-courseplan-cli`
  (ADR 0001).
- Source code (identifiers, comments) is English. Specs, plans and ADRs are English. READMEs, manual, examples
  README and changelog are bilingual, German and English (ADR 0003). With the user, speak German.
- Program texts (CLI messages and help, findings, editor, MCP texts) live in Fluent files, one per language,
  under `i18n/<language>/courseplan.ftl` (ADR 0022). English is the reference; a new message gets its text in
  every language the project has, in the same commit. `Display` of error types stays English.
- Glossary first: a new domain term gets its row in `docs/glossary.md` before or with its code. Terms that
  freemapper's or freemapper-print's glossary defines keep their names and code identifiers (`Course`,
  `Control`, `ControlCode`, `NumberPlacement`, `MarkedRoute` …).
- User-facing text goes through the `unslop` and `humanizer` skills before it is committed: READMEs, manual,
  examples README, changelog, release notes, ADRs, and program messages longer than a short phrase (warnings,
  errors, CLI help, MCP tool descriptions). A subagent that writes such text reads and applies both skills'
  SKILL.md files.
- Every significant decision gets an ADR in `docs/adr/` in the same branch. Every merged feature or fix gets an
  entry under "Unveröffentlicht"/"Unreleased" in `CHANGELOG.md` and `CHANGELOG.en.md`.
- In the design, mark statements *(user)* or *(Claude's call)*, so the user sees what was decided and what is
  only proposed.
- Dependencies must be at least two weeks old (ADR 0006). The toolchain is pinned (ADR 0007), together with
  freemapper and freemapper-print.
- `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` must pass;
  hooks in `.claude/` and `lefthook.yml` run them.
- Work on without asking for confirmation as long as the next step is clear *(user, 2026-11-08)*: the next
  plan from the approved design, its execution, merges and releases. Ask only for decisions that are the
  user's to make.
- Superpowers process: brainstorming, design approved by the user, writing-plans, then subagent-driven
  development; TDD for readers, writers, geometry and check rules. Rulings made during implementation go into
  ADRs before the ledger is deleted.
- Git: `main`; one branch and one worktree under `.worktrees/<branch>` per plan; Conventional Commits in
  English (`feat(scope): …`, `fix`, `docs`, `test`, `chore`); merge with `--no-ff` after the final review;
  never force-push. Merge feature branches on your own judgment, without asking *(user, 2026-11-08)*: when the
  final review is clean or its residuals are parked with rulings, and fmt, clippy and tests pass on the merged
  `main`.
- Disk space (the Mac is shared by the suite; freemapper's `target/` once reached 31 GB): after a merge run
  `cargo clean` in the worktree and in the main checkout, then remove the worktree; check `df -h` before long
  runs. The program itself writes only small files and saves atomically (tmp file, then rename), so a full
  disk leaves the old project file intact and the error names the file.
- Before every push *(user, 2026-11-08)*: check that `README.md` and `README.en.md` describe the pushed state
  (status paragraph, features, commands, links), fix them in the same push if not, and keep both languages equal.
- Remotes as in freemapper *(user, 2026-11-08)*: `origin` is the private GitLab
  (https://gitlab.jobst-server.de/henry/freemapper-courseplan, HTTPS), `github` the public GitHub repository
  (https://github.com/HenryJobst/freemapper-courseplan, push over SSH). CI runs on GitHub. Push `main` to both
  after a merge.
- Releases *(user, 2026-11-08)*: publish a release on your own at sensible points, without asking: after a
  milestone with new user-visible functionality is merged and CI on `main` is green, or after an important
  fix. Version 0.x: minor step per milestone, patch step for fixes. Steps: set `[workspace.package] version`,
  `cargo update -w`, turn "Unveröffentlicht"/"Unreleased" into `## [x.y.z] - YYYY-MM-DD` in both changelogs,
  set the version and date in the first lines of the "Status" section of both READMEs, commit, annotated tag `vX.Y.Z`, push `main` and the tag to both remotes, watch the release workflow, and
  report the release URL.
- Test fixtures carry source, date and licence in `tests/fixtures/README.md`.
- Coordination with the sibling sessions goes through SendMessage. Agreements with them are recorded in the
  design with *(agreed with freemapper-print)* or *(agreed with freemapper)*.
