# 0025. Rule profiles as data files

Date: 2026-11-09
Status: Accepted

## Context
Course rules differ by country and year and change often. Rules in code need a release per change.

## Decision
- A rule profile is a TOML file embedded in the program. It starts with `format = 1`, then `[profile]` (`id`,
  `kind = "base" | "overlay"`, `base` for overlays, `name`, `valid_from`), `[sources.<doc>]` and `[[rules]]`.
- A rule has `id`, `key` (the implementation key, such as `min_distance`), `severity` (`error`, `warning`, `hint`),
  `values`, `source = { doc, section }` and an optional `note`. The filters `map_kinds`, `disciplines` and
  `print_scales` mean all when empty. A course's print scale is its `planned_print_scale`, else the map scale.
- A number in `values` names its unit, alone or as suffix: `metres`, `mm`, `degrees`, `percent`, `factor`. Texts,
  lists of texts and the tables documented per rule `key` in `rules.rs` are exempt; any other value key is invalid.
- An event names one base profile (`rule_profile`) and any overlays (`rule_overlays`, omitted when empty). An
  overlay rule with an existing `id` replaces the base rule; `enabled = false` (default `true`) switches it off.
- Rule ids are stable and dotted: `<profile>.<family>.<name>`. If an id changes, accepted findings come back.
- An accepted finding is identified by `(rule, course, controls, legs)`. Controls are deduplicated and sorted:
  all-digit codes by numeric value, then the other codes lexically. Legs are deduplicated and sorted by (course
  name, after). Absent and set `course` are different keys. A leg is `{ course = "A", after = 3 }`, with `after`
  counting from 1 as in the CLI and 0 the leg from the start. A finding whose key changes is reported again.
- freemapper-print will reuse this format for print rules (not before its M3); `freemapper-rules` may be extracted.

## Consequences
Rule changes are file edits, and every value names its source. Format 1 stays provisional (ADR 0019), so
`rule_overlays` and `legs` need no bump. Cost if wrong: renaming keys in the embedded files.

Source: agreed with freemapper-print, 2026-11-09; ADR 0019.

## Amendment 2026-11-09: a leg is named by its ends

- A leg reference stores the ids of the stops at its ends as well:
  `{ course = "A", after = 6, from = "36", to = "37" }`. `from` and `to` are control codes or the ids of the
  start and the finish. The key compares all four, so an acceptance on a leg comes back once the leg runs
  between other stops, even at the same position (ruling R24).
- `finding accept --leg A:6` and the MCP `accept_finding` name the leg by course and position; the program
  fills `from` and `to` from the course as it is when the finding is accepted, and returns the stored entry.
  `finding remove` and `remove_accepted_finding` first remove the entry whose ends are the leg's ends now. Only
  when there is none do they match legs by course and position, and then they remove every entry found there:
  none of them fits the course as it is, and the CLI could not name them apart. So an acceptance that no
  longer matches can still be removed.
- A file without `from` and `to` reads them as empty. Such an acceptance matches no finding and is listed as
  not matched (ADR 0026). No release has stored leg acceptances, so format 1 stays.

Source: final review of plan M2b; controller ruling R24, 2026-11-09.
