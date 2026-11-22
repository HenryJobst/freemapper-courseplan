# 0103: Issues of users are labelled

Status: Accepted, 2026-11-22 *(agreed with freemapper, ADR 0180 there)*.

## Context

The maintainer opens most issues of the repository, and so do the agents that work for them. An issue of a user stands
among them and can be overlooked, although it should be looked at first.

## Decision

- A workflow (`.github/workflows/user-issues.yml`) sets the label `von-nutzer` on every issue whose author is not the
  owner, a member, a collaborator (`author_association`) or a bot. It sets only the label and never reads the text of
  the issue, so an issue cannot steer it. The label is orange (FF6A13). A search `is:issue is:open label:von-nutzer`
  lists what is waiting.
- Two issue forms (bug, request) ask for the version, the map and what happened; they set `bug` or `enhancement`.
  Blank issues stay possible, and the issues of the project itself stay without the label.

## Consequences

- An issue of a user is marked at once, also when it was opened without a form.
- A collaborator who reports as a user is not marked; the maintainer may add the label by hand.
- The workflow needs `issues: write` and nothing else; it uses only the `gh` CLI of the runner, so the action pins
  do not change.
