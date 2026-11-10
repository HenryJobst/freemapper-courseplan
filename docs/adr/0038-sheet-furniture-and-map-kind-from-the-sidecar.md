# 0038. Sheet furniture and the map kind from the sidecar

Date: 2026-11-10
Status: Accepted

## Context
freemapper v0.9.0 tags more objects than the sheet frame: the scale bar (`freemapper:source=scale-bar`) and,
on sprint maps, a base area over the whole sheet (`freemapper:source=base`). The map index skipped only
`sheet`, so the scale bar grew the extent to the south and the `map_edge` check missed controls at the south
edge. Separately, `map_kind` ignored the `map_kind` key of `freemapper.toml`, which freemapper-print reads
first, so the two tools could disagree about one map.

## Decision
- **Sheet furniture.** `is_sheet_furniture` treats the tags `sheet`, `scale-bar` and `base` alike. Such objects
  are no map features and do not count for the extent *(Claude's call)*.
- **Kind from the file.** `map_kind_for_file(doc, map_path)` takes `map_kind` from the `freemapper.toml` next to
  the map first, then the symbol set name (case-insensitive) and the scale as freemapper-print decides them.
  Where that table has no answer, the earlier rule of `map_kind` applies as the last fallback *(Claude's call)*.
- **Tolerant sidecar.** A missing or malformed file, a missing key and an unknown value (a future kind such as
  `gym`) fall through to the name; they are never an error. freemapper-print reports a broken sidecar; here it
  would only block checks *(Claude's call)*.
- **Where it applies.** Everything that knows the map file: `init`, the CLI and MCP commands, check, number
  placement, and the exports and the import. The library functions that only get a document keep `map_kind`
  and gain `*_as` variants that take the kind (`check_event_as`, `export_ppen_as`, `import_ppen_as`).
- The `title` fallback of `init` stays as it is.
