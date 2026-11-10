# 0042. Editor server: one process per project, a shared shell, and guarded requests

Date: 2026-11-11
Status: Accepted

## Context
ADR 0005 puts the course editor in a local browser page and names the base rules: 127.0.0.1 only, a random token,
a Host check, a CSP. The editor changes a project file, so a page from another site that reaches the port must not
be able to write. freemapper's area selector already has the token, Host and Origin checks, the headers, the
embedded assets and the browser opener, and freemapper-print's editor will need the same.

## Decision
- **Process model** *(Claude's call)*. `edit PROJECT [--port N] [--no-browser]` loads the project first. A broken
  file stops the command with the full CLI message and exit 1, and nothing reaches stdout. It binds
  `127.0.0.1:N` (0 takes a free port) and prints the address as the only line on stdout. It opens the browser
  unless `--no-browser` is given; if that fails it says so on stderr and keeps serving. It reads nothing from
  stdin and runs until Ctrl+C or `POST api/quit`, then exits with 0. A port that is taken is an error that names
  the port, exit 1. One project per process. Operations run one after another behind a lock, as the MCP server
  serialises its calls.
- **Shared shell in core** *(Claude's call)*. The checks, headers, token, embedded assets and `open_browser` live
  in the core crate `freemapper-web` (freemapper-core v0.11.0, core ADR 0020), because the suite keeps shared
  code in the core library. Each app brings its own routes. This crate uses `serve` with `strict_post`, an
  embedded asset set and its own routes below the token.
- **Security** *(user for the base rules in ADR 0005; Claude's call for the additions)*. The shell adds to the
  base rules:
  - every POST needs an `Origin` equal to the page's origin and a `Content-Type` of `application/json` or
    `application/octet-stream`. A cross-site form cannot send either, and a cross-site `fetch` with such a type
    needs a preflight that the server never answers;
  - the editing routes, which come with the API (ADR 0043), also require the header `X-Courseplan-Revision`;
  - no path from the browser reaches the file system. The API takes no file names, assets come only from the
    embedded set, and the server touches only the project file, its map, the linked `.ppen` and the sync base
    copy;
  - bodies are limited to 1 MiB.
- The CSP is `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:;
  connect-src 'self'; frame-ancestors 'none'`. Inline styles stay allowed because the renderer's SVG carries
  `style` attributes. The page has no inline script and no inline handler; a test reads the embedded files.

## Consequences
- The editor depends on the core tag that holds `freemapper-web`, and so on axum 0.8.9, rust-embed 8.13.0 and
  tokio features `net`, `time`, `signal`, `sync`, the versions freemapper already uses.
- The address is the access secret. While the browser opener runs, it shows in the process list of the same
  machine, and no log may contain it.
- Cost if wrong: if the shell proves too narrow for the editor's routes, the editor can build its own router
  around it, since the shell only wraps what it is given.

Source: plan `docs/superpowers/plans/2026-11-11-editor-m4.md`, decisions D1 to D3 *(Claude's call)*; ADR 0005.
