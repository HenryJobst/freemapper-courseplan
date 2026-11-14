# 0010. Sprint and schoolyard maps first

Date: 2026-11-08
Status: Accepted; superseded in part by [0092](0092-forest-maps-isom-2017-2.md) (forest maps in ISOM 2017-2)

## Context

freemapper produces sprint and schoolyard maps today; forest maps come later. Each map kind needs its own
similarity groups, impassable symbols, column D proposals, overprint sizes and test maps.

## Decision

The first version supports sprint (ISSprOM 2019-2) and schoolyard maps. Forest maps (ISOM 2017-2) come later.
The rule data keeps room for ISOM values.

## Consequences

- The rule and symbol tables stay half as large in the first version.
- Cost if wrong: a forest event before forest support needs Purple Pen.

Source: brainstorming 2026-11-08 (user: "Waldkarten später"); builder check question 6.
