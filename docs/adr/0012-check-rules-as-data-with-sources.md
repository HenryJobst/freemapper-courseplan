# 0012. Check rules as data with sources, WKB 2026 first

Date: 2026-11-08
Status: Accepted

## Context

Course-setting rules come from the WKB 2026, the IOF rules and national guides, with differing values. A complete
revision of the WKB is due for 2027. A course setter needs to know why a check complains.

## Decision

Each check rule is data: an id, its rule profile, its values and its source (document and section). The first
version ships the WKB 2026 profile; an IOF profile comes later. The user can accept a finding with a note, and
`check` then stops reporting it.

## Consequences

- New rule versions change data, not code, and `check` can always name the source.
- Cost if wrong: rules that need logic beyond values still need code per rule.

Source: design §5.6; research `docs/research/2026-11-08-bahnlegung-kriterien.md`.
