# 0089: The user can reject candidate sites, per project

Status: Accepted, 2026-11-13. Follows [0062](0062-score-candidates-and-points.md) and
[0065](0065-score-draft.md).

## Context

`score suggest` and `score draft` pick sites from the map by route choice. The user knows things the map does not
show: a site stands on a private yard, a corner is hard to see, a boulder was moved. Today the only way to keep such a
site out is to move the controls by hand after every draft, and the next draft offers it again.

## Decision

- The user can reject a candidate site *(user, 2026-11-13)*. The rejections belong to the project and stand in the
  project file; `score suggest`, `score draft` and the candidate list of the editor leave them out *(user)*.
- A rejected site is a position in map millimetres. A candidate within 3 m on the ground of a rejected position counts
  as the same site *(user: tolerance of about 3 m)*. The test is the distance to the rejected position, not a cell
  of a fixed grid, so two positions a metre apart never fall on different sides of a cell border *(Claude's call)*.
- The project file gets an optional top-level key `rejected_sites`, a list of `[x, y]` points. It is written only when
  the list is not empty. The format stays 1 and no older file needs a change, as for the score settings
  (ADR 0060) *(Claude's call)*. A program from before this version rejects a file that has the key, because it
  refuses unknown keys (ADR 0019).
- Rejecting a site that is rejected already changes nothing. Taking a rejection back removes every rejected position
  within 3 m of the given one and fails when there is none. Neither looks at the candidates: any position can be
  rejected, even one that no candidate has *(Claude's call)*.
- A control that stands at a rejected site stays. A link
  end at a rejected site fails like any position that is no candidate.
- Interfaces *(Claude's call)*: `score reject --at X,Y` and `score unreject --at X,Y` on the command line, the tools
  `reject_site` and `unreject_site` (parameters `project` and `at`) over MCP, and a "Reject" button beside "Accept" in
  the candidate table of the editor. The editor runs the tool, so undo takes it back. All three answer with `at`,
  `added` or `removed`, and `rejected_sites`, the number of rejected sites.
- No command lists the rejected sites. They are in the project file, a plain TOML list *(Claude's call)*.

## Consequences

- A draft is as repeatable as before, with the rejected sites out of its pool. A rejection can make a draft miss a
  band that it met before; the draft says so as it does for any other cause.
- The tool count of the MCP server goes from 52 to 54.
- Rejections are no findings: `check` does not know them.

## Cost if wrong

- Three metres may be too much on a dense map, where two real sites stand 2 m apart, and too little where the user
  clicked a few metres off. The value is a constant; the file keeps the exact positions, so it can change later.
- A list that only grows is hard to survey without a list command. Add `score rejected` when someone needs it.

Source: the user, 2026-11-13.
