# 0025. Rule profiles as data files

Date: 2026-11-09
Status: Accepted
## Context

M2b checks courses against rules that differ by country and year and change often. Rules in code need a release
for every change and hide where a value comes from.

## Decision

- A rule profile is a TOML file embedded in the program. It starts with `format = 1`, then `[profile]` (`id`,
  `kind = "base" | "overlay"`, `base` for overlays, `name`, `valid_from`), `[sources.<doc>]` and `[[rules]]`.
- A rule has `id`, `key` (the implementation key, such as `min_distance`), `severity` (`error`, `warning`, `hint`),
  `values`, `source = { doc, section }` and an optional `note`. The optional filters `map_kinds`, `disciplines`
  and `print_scales` mean all when empty. A course's print scale is its `planned_print_scale`, else the map scale.
- Keys in `values` carry their unit: `metres`, `mm`, `degrees`, `percent`, `factor`. A bare number is not allowed.
- An event names one base profile (`rule_profile`) and any overlays (`rule_overlays`, omitted when empty). An
  overlay rule with an existing `id` replaces the base rule; `enabled = false` (default `true`) switches it off.
- An accepted finding is keyed by rule id plus the sorted set of control codes and leg references. A leg reference
  is `{ course = "A", after = 3 }`; `after` counts from 1 as in the CLI, and 0 is the leg from the start. A finding
  whose key changes is reported again.
- freemapper-print will reuse this format for its print rules, not before its M3. A shared crate such as
  `freemapper-rules` may be extracted then.

## Consequences
Rule changes are file edits, and every value names its source. Format 1 stays provisional (ADR 0019), so the new
fields `rule_overlays` and `legs` need no format bump. Cost if wrong: renaming keys in the embedded files.

Source: plan M2b tasks 1-2, ruling R3; ADR 0019.
