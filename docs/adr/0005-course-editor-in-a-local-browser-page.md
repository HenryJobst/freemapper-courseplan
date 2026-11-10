# 0005. Course editor in a local browser page

Date: 2026-11-08
Status: Accepted

## Context

Setting courses means placing controls on a map and dragging bends and numbers. A native GUI needs a toolkit
per platform. freemapper's area selector and freemapper-print's layout editor run as local web pages served by
the binary.

## Decision

`freemapper-courseplan edit` starts a local web server and opens the editor in the browser: event on the left,
the map with the selected course in the centre, properties and check findings on the right. Security rules as
in freemapper-print ADR 0005: 127.0.0.1 only, a random 128-bit token in every path, a Host check, a CSP,
`X-Frame-Options: DENY`, `no-store`.

## Consequences

- The editor shows the shared renderer's output (ADR 0014), so the map looks as it prints.
- Cost if wrong: a browser page limits drawing tools compared with a native GUI.

Source: brainstorming 2026-11-08 (user: "Browser-Editor wie bei freemapper-print passt").

Amended by ADR 0042: the shell is the core crate `freemapper-web`, and every POST needs an Origin, a JSON content type and a revision.
