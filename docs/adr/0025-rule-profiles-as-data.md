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
- Keys in `values` carry their unit: `metres`, `mm`, `degrees`, `percent`, `factor`. A key without one is invalid.
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
