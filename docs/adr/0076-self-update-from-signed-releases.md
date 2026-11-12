# 0076: Self-update from signed releases

Status: Accepted, 2026-11-12. Takes over freemapper's [ADR 0094](https://github.com/HenryJobst/freemapper/blob/main/docs/adr/0094-self-update-from-signed-releases.md).

## Context

People unpack freemapper-courseplan once and register it with an AI assistant by its absolute path. Releases come every
few days and nothing tells them. Updating means a new download, unpacking and a new registration. The checksums next to
the archives prove integrity, not origin. On 2026-11-12 the user asked for the same auto-update concept that freemapper
already has *(user)*. The design is [`2026-11-12-self-update-design.md`](../superpowers/specs/2026-11-12-self-update-design.md),
which lists only the differences from freemapper's design.

## Decision

- `freemapper-courseplan update` replaces the running executable in place with the newest release, after asking:
  download, SHA-256 of the `.gz` and of the executable, `--version` of the new file, two renames. The old file stays as
  `freemapper-courseplan.previous` for `update --rollback`. The path stays, so the MCP registration keeps working
  *(user)*. Flags: `--check` (exit 10 when a newer version exists), `--yes`, `--version X.Y.Z`, `--rollback`,
  `--mode notify|install|off`.
- Every release publishes the bare executables as `.gz` and one manifest `freemapper-courseplan-update.json` with a
  minisign signature. The manifest names its app, so a signed manifest of freemapper is refused here. Nothing unsigned is
  installed *(user)*.
- **Signing key** *(user)*: the public key of freemapper (`assets/update/minisign.pub`, key id
  9353842ABFFAC9FB) is reused, and the two secrets `MINISIGN_SECRET_KEY` and `MINISIGN_PASSWORD` go into the
  GitHub environment `release` of this repository. The user makes no second key. Cost if wrong: a leak in one repository exposes the key of both;
  the compiled-in list of keys lets a release rotate it.
- **When the program checks** *(Claude's call)*: courseplan has no command that uses the network. The daily check
  (at most once a day, 3 s budget, one line on stderr) therefore runs after `info` and `edit`, not after the offline
  batch commands. The editor page shows a one-line notice from the cached check and has no install button. Cost if
  wrong: someone who only runs `check` in scripts never sees a notice. `update --check` stays available.
- **MCP**: tools `check_update` and `install_update` (`confirm: true` required), a note in the first tool result, and
  no install by the server on its own, whatever the mode *(user)*. The tool count goes from 50 to 52.
- **Where things live** *(Claude's call)*: settings and the check record sit in `<config dir>/freemapper-courseplan/`
  (`settings.toml`, `update-check.json`); courseplan has no cache folder. `FREEMAPPER_COURSEPLAN_UPDATE` overrides the
  mode for one process, `FREEMAPPER_COURSEPLAN_CONFIG_DIR` moves the folder (the tests use it).
- Builds from source never update themselves. `release.yml` sets `FREEMAPPER_COURSEPLAN_DIST=github-release` for the
  build step only; the compile fails if a release build carries the test key.
- **HTTP client** *(Claude's call)*: `ureq` 3.4.2 (the version freemapper uses, older than two weeks, ADR 0006) with
  redirects off in the agent. The app follows at most three, each target through the host rule of `freemapper-update`.
  Only the three GitHub hosts are allowed. `minisign-verify` 0.3.0 is a dev-dependency that checks the committed key.
  The pure parts come from `freemapper-update` of freemapper-core (pin v0.19.0, unchanged); the network stays here
  (core ADR 0007).
- **Release workflow** *(Claude's call)*: a port of freemapper's. It builds with the dist variable, adds the `.gz`
  assets, writes and signs the manifest, verifies the signature with the key in the program and refuses to publish
  without one. `min_self_update_from` carries over from the previous release's signed manifest and is raised by hand
  through `[workspace.metadata.update]` in `Cargo.toml`; the first release with the updater takes its own version.
  A job after publishing lets the previous release update itself and roll back.

## Exception to ADR 0006

`flate2` 1.1.11 (published 2026-11-06) is in `Cargo.lock` and is younger than two weeks. `freemapper-update` of
freemapper-core v0.19.0, the pin we already use, requires `flate2 = "=1.1.11"`, and freemapper has the same entry. Going
older would need a different core pin, which this change does not touch. The version is two weeks old on 2026-11-20. No
release is needed before that date, so no release ships it younger than the rule allows. All other new entries are older
than two weeks; `rustls-pki-types` and `webpki-roots` were set back to 1.15.1 and 1.0.11 for that reason.

## Consequences

- The environment `release` holds the signing secrets, has a required reviewer and is limited to `main` and `v*`.
  Only the job `release`, after the builds, uses it, so a release asks once *(user, 2026-11-12)*. The protection
  sits in the environment, outside the workflow file: a changed workflow cannot reach the secrets without the
  reviewer. A gate job at the start (as in freemapper-print, ADR 0060 there) and a separate approval job were tried
  and dropped: a second job on the environment asks again, and an approval that only the `needs:` chain of the
  workflow enforces can be removed by anyone who may push to `main`. The price is that a release without the secret
  fails only after the builds, at the signing step.
- The first release with the updater cannot be reached by `update` from older versions, which have no updater. People
  install it by hand once.
- The program sends one kind of request, to the GitHub release hosts, with a fixed User-Agent that names the program,
  its version and the repository. No project data, no user name, no identifier.
- Cost if wrong: an update that breaks reaches everyone in mode `install` at once (rollback and the post-release job
  limit it); a lost signing key forces one manual update; a change of GitHub's redirects breaks the check until the
  program is adapted.

Source: user request of 2026-11-12; freemapper ADR 0094 and its design of 2026-11-11; freemapper-core ADRs 0005 to 0007
and 0009; ADR 0006 (dependency age).
