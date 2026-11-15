# 0097: The release workflow starts the rebuild of freemapper.de

Status: Accepted, 2026-11-15. Follows freemapper's PR #166 *(agreed with freemapper)*.

## Context

The suite has a website, https://freemapper.de (repository freemapper-site, GitHub Pages). It shows the program page
with the download table, the manual and the changes of freemapper-courseplan, all read from the newest release tag at
build time. It builds daily; a release should show up sooner.

## Decision

- The release workflow gets a job `website` (needs `release`, skipped in a dry run, `permissions: {}`). It runs
  `gh workflow run deploy.yml -R HenryJobst/freemapper-site` with the secret `SITE_DISPATCH_TOKEN`, a fine-grained
  token that may only run workflows of freemapper-site. The user creates the secret; nothing here handles its value.
- Without the secret the job ends with exit 0. A failed start gives a warning only. A release never fails because
  of the website.
- README, manual and AGENTS.md link to the website.

## Consequences

- Until the secret exists, the website follows with its daily build.
- The job sits outside the environment `release`, so it has no access to the signing secrets.
