# 0014. Course content here, sheet presentation in freemapper-print

Date: 2026-11-08
Status: Accepted

## Context

Either this project or freemapper-print could set some course details, above all control number positions. Whether a
number hides a leg is course knowledge; whether it collides with a mask or logo shows only on the sheet.

## Decision

What is the same on every sheet of a course belongs here: controls, order, bends, special objects, control
descriptions, number positions (central per control, overridable per course), planned print scale. What depends
on the sheet belongs to freemapper-print. Number positions are a direction (degrees, 0 = east,
counter-clockwise) and a gap in print millimetres. freemapper-print prints control-placer tour sheets from tour
data this project delivers.

## Consequences

- freemapper-print never writes to course data; this project never decides sheet layout.
- Cost if wrong: corrections in freemapper-print go stale when a number moves here; its `check` reports them.

Source: agreed with the freemapper-print session 2026-11-08 (its ADR 0015, design §5.7, §5.8).
