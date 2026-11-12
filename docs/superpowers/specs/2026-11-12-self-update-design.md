# Self-update design

Status: approved by the user's request of 2026-11-12 · Terms follow [`docs/glossary.md`](../../glossary.md).
Decision record: ADR 0076. Source of the concept: freemapper's
[self-update design](https://github.com/HenryJobst/freemapper/blob/main/docs/superpowers/specs/2026-11-11-self-update-design.md)
and its ADR 0094. This document only lists what courseplan takes over and where it differs.

## 1. Intent

*(user, 2026-11-12)* "Take over freemapper's auto-update concept." People unpack freemapper-courseplan once and
register it with an assistant by its absolute path. Releases come every few days and nothing tells them. The
program notices a new release, says so in one line, and replaces itself in place on request, with a way back.

## 2. Taken over unchanged *(user)*

All of freemapper's design sections 1, 3.1 to 3.5, 3.9 and 3.10 apply with the names changed
(`freemapper-courseplan` for `freemapper`; manifest `freemapper-courseplan-update.json`, signature `.minisig`;
`freemapper-courseplan.previous`; environment `FREEMAPPER_COURSEPLAN_UPDATE`; settings file
`<config_dir>/freemapper-courseplan/settings.toml`):

- `update [--check|--yes|--version X|--rollback|--mode notify|install|off]`; exit 10 for `--check` with an update.
- Signed manifest over bare `.gz` executables, minisign, built-in key list, only the three GitHub hosts.
- Implicit check at most once a day, 3 s budget, one stderr line, mode `notify` by default, never for `mcp`
  stdout, `CI`, `--help`, `--version`, builds from source.
- Install: download, SHA-256 of `.gz` and executable, run `--version` before the swap, two renames,
  `.previous` kept, `stale-*` clean-up.
- MCP: tools `check_update` and `install_update` (`confirm: true`), a note in the first tool result; the server
  never installs on its own.
- Pure parts come from freemapper-core's `freemapper-update` (already in the pin v0.19.0); the network is
  courseplan's (core ADR 0007).

## 3. Where courseplan differs

- **Commands that check implicitly** *(Claude's call, D1)*: `init`, `import`, `export`, `check`, `edit` and
  `score suggest|draft|length` are offline; no command here uses the network. So the implicit check follows
  `edit` (the editor is the long-lived entry point; its page shows the notice from the cached check) and `info`,
  not the offline batch commands. Cost if wrong: someone who only scripts `check` never sees a notice; `update
  --check` stays.
- **Editor** *(Claude's call, D2)*: a one-line notice from the cached check, like freemapper's page; no install
  button. Texts in both languages through Fluent.
- **Signing key** *(user, D3)*: the public key of freemapper (`assets/update/minisign.pub`, key id
  9353842ABFFAC9FB) is reused, so the user only copies two secrets (`MINISIGN_SECRET_KEY`,
  `MINISIGN_PASSWORD`) into the environment `release` of the courseplan repository. The manifest's `app` field keeps a signed freemapper
  manifest from being accepted here. Cost if wrong: one key signs two repositories; a leak in one repository
  affects both; rotation through the key list covers it.
- **HTTP client** *(Claude's call, D4)*: `ureq` 3.4.2, the version freemapper uses, as the only new third-party
  dependency (ADR 0006: older than two weeks). `minisign-verify` 0.3.0 only as a dev-dependency to check the
  committed key, as in freemapper. Redirects off in the agent; the app follows at most three, each through
  `check_url`.
- **Release workflow** *(Claude's call, D5)*: builds with `FREEMAPPER_COURSEPLAN_DIST=github-release`, adds the
  `.gz` assets and the signed manifest, refuses to publish without a valid signature. Until the secrets exist a
  release fails at the signing step; no release is made before the user added them.
- **No `min_self_update_from` carry-over beyond freemapper's script**: the first release with the updater takes
  its own version; the pin lives in `[workspace.metadata.update]`.
- **Documentation**: both READMEs, manual chapter on updates (DE/EN), AGENTS.md (tools, rule: ask before
  installing, restart the MCP server), both changelogs, glossary rows, ADR 0076, i18n in English and German.

## 4. Human step

*(user)* The user adds `MINISIGN_SECRET_KEY` and `MINISIGN_PASSWORD` to the GitHub environment `release` of
freemapper-courseplan (the same values as in freemapper). An agent does not handle the secret key.
