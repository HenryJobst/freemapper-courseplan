# 0053. Editor: export and import

Date: 2026-11-11
Status: Accepted

## Context
The Event menu exports IOF XML and Purple Pen files and imports a Purple Pen file. ADR 0042 forbids any path from
the browser to the file system, so the page cannot name a source or a target.

## Decision
- **No names from the browser** *(user, ADR 0005; Claude's call for the routes)*. The server writes only the
  project's linked `.ppen`, which it takes from `sources.ppen`: the name must end in `.ppen` and must not be a
  symbolic link. Without a link, "to linked" is 422 with a localized text. Export and import use the CLI functions
  (`export_ppen_file`, `import_ppen_bytes`), so the same-file guards and the id write-back are the CLI's.
- **Downloads are made in memory** *(Claude's call)*. `GET api/export/iof-xml` answers `application/xml` with
  `Content-Disposition: attachment; filename="<stem>.xml"`. The stem is the project file name without
  `.courseplan.toml` or `.toml`, every character outside `[A-Za-z0-9._-]` replaced by `_`, at most 80 characters. The
  course counts come in `X-Courseplan-*` headers. `POST api/export/ppen {"to": "download"}` makes a new file as
  `export --new` would, without ids of the project, names the map by its absolute path and stores nothing. It
  answers JSON with the file text, because the page also needs the notes; the page makes the download from it.
- **Upload** *(Claude's call)*. `POST api/import/ppen?replace=true|false` takes the file as the body
  (`application/octet-stream`). A route-level `DefaultBodyLimit` of `MAX_COURSE_FILE_BYTES` replaces the shell's
  1 MiB for this route only (413 above). The revision header is required (409 when stale). Without `replace`, a
  project that has controls, courses or special objects is refused (422). The import leaves `sources.ppen` as it is.
  Any reader error becomes one fixed text, so an answer never quotes the file.
- **History** *(plan D7)*. Export to the linked file and import clear the undo history. Both save through the
  project's save guard, so a write by another program gives 409.

## Consequences
A project without a link can only download a Purple Pen file. A person who wants a link runs `export --ppen FILE`
once. The editor shows an empty event correctly now: the server leaves empty lists out of the event JSON, and the
page fills them in.
