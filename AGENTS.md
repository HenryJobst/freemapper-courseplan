# freemapper-courseplan for AI agents

This file tells an AI agent how to install freemapper-courseplan and how to use it for a person who sets
orienteering courses. It ships in every release archive next to the program. People read
[README.en.md](README.en.md) (English) or [README.md](README.md) (German); the [user manual](docs/manual/en.md) has
the details.

## What it does and what it does not

freemapper-courseplan sets courses on a finished map from OpenOrienteering Mapper (OOM, `.omap`): controls with
code and control description, start, finish, classes, individual courses with bends and marked legs, control
numbers placed behind the circle, and special objects (marked routes, forbidden routes, out-of-bounds and
dangerous areas, first aid posts, water points). `check` tests the courses against a rule profile (built in:
`wkb-2026`, overlay `nor-bb-2021`) and names the source of each finding. Courses go out as IOF XML 3.0 for result
software and as Purple Pen events (`.ppen`), and a Purple Pen event can be taken over or merged back.

The program has no AI of its own. It does not see the map: it can only report the extent of the map and the map
objects near a position. Give it positions in map millimetres, or work them out from the extent.

Tell the person these limits before you start:

- Only sprint and schoolyard maps work. Forest maps (ISOM) are refused. The map must be an `.omap` or `.xmap`.
- Only individual courses can be edited, measured and checked. Team, relay and score courses are not built; the
  program skips them on export and says so.
- The IOF XML export has not been tried in OE, because nobody on the project has OE. The Purple Pen export opens in
  Purple Pen 4.0 (checked once by the author).
- Purple Pen must be closed for the event before a `sync`, and opened again afterwards. Otherwise Purple Pen may
  overwrite the merged file.
- The editor draws the real map, but a map larger than one A3 sheet at map scale is clipped to its middle part;
  courses outside it stay on the overlay.
- There is no print preview and no printing of courses yet; printing waits for freemapper-print.
- The route analysis is missing, so a straight leg through an out-of-bounds area is only a warning.
- OCAD is not needed and not tested.
- `check` tests distances, angles and codes, not whether a control stands at the right feature. The person looks
  at every position on the map.

## Install

freemapper-courseplan is one executable without an installer and without dependencies. Do not build it from
source unless the person asks; the release archives are ready to run.

1. Find the newest version: `https://api.github.com/repos/HenryJobst/freemapper-courseplan/releases/latest`
   returns JSON with `tag_name`, for example `v0.10.0`. The release page is
   <https://github.com/HenryJobst/freemapper-courseplan/releases/latest>.
2. Pick the archive for the system. The name is `freemapper-courseplan-<tag>-<platform>` plus the extension:

   | System (`uname -s` / `uname -m`) | Platform | Archive |
   |---|---|---|
   | macOS, `arm64` (Apple chip) | `macos-arm64` | `.tar.gz` |
   | macOS, `x86_64` (Intel) | `macos-x86_64` | `.tar.gz` |
   | Linux, `x86_64` | `linux-x86_64` | `.tar.gz` |
   | Linux, `aarch64` | `linux-arm64` | `.tar.gz` |
   | Windows, x64 | `windows-x86_64` | `.zip` |

   Each archive has a `<archive>.sha256` next to it. The Linux builds are static (musl).
3. Download, check and unpack. macOS and Linux, for example in `~/Applications` or `~/opt`:

   ```sh
   tag=v0.10.0; platform=macos-arm64
   base="https://github.com/HenryJobst/freemapper-courseplan/releases/download/$tag"
   curl -fLO "$base/freemapper-courseplan-$tag-$platform.tar.gz"
   curl -fLO "$base/freemapper-courseplan-$tag-$platform.tar.gz.sha256"
   shasum -a 256 -c "freemapper-courseplan-$tag-$platform.tar.gz.sha256"   # Linux: sha256sum -c
   tar -xzf "freemapper-courseplan-$tag-$platform.tar.gz"
   ```

   Windows (PowerShell):

   ```powershell
   $tag = "v0.10.0"; $name = "freemapper-courseplan-$tag-windows-x86_64.zip"
   $base = "https://github.com/HenryJobst/freemapper-courseplan/releases/download/$tag"
   Invoke-WebRequest "$base/$name" -OutFile $name
   Invoke-WebRequest "$base/$name.sha256" -OutFile "$name.sha256"
   (Get-FileHash $name -Algorithm SHA256).Hash.ToLower()   # compare with the first word of the .sha256 file
   Unblock-File $name
   Expand-Archive $name -DestinationPath .
   ```

4. The archive unpacks to a folder `freemapper-courseplan-<tag>-<platform>/` with `freemapper-courseplan`
   (Windows: `freemapper-courseplan.exe`), `README.md`, `README.en.md`, `LICENSE`, this file and `docs/manual/`.
   The program is not signed:
   - macOS refuses to start it until the quarantine flag is gone:
     `xattr -d com.apple.quarantine freemapper-courseplan-<tag>-<platform>/freemapper-courseplan`. Tell the person
     you did this and why.
   - Linux: `chmod +x freemapper-courseplan` if the executable bit got lost.
   - Windows may show "Windows protected your PC" on the first start. `Unblock-File` before `Expand-Archive`
     avoids it; otherwise the person confirms with *More info* > *Run anyway*.
5. Check: `<folder>/freemapper-courseplan --version` prints `freemapper-courseplan <version>`.

Remember the absolute path of the executable; the MCP registration needs it. Updating means unpacking a newer
archive and pointing the registration at the new path. Project files stay valid.

Do not put the person's e-mail address, name or other personal data into any request, file name, event title or
setting.

## Use it through MCP (preferred)

`freemapper-courseplan mcp` is an MCP server on stdin/stdout. Register it once with the absolute path:

```sh
claude mcp add freemapper-courseplan -- /absolute/path/to/freemapper-courseplan mcp          # Claude Code
```

```toml
# Codex: ~/.codex/config.toml
[mcp_servers.freemapper-courseplan]
command = "/absolute/path/to/freemapper-courseplan"
args = ["mcp"]
```

```json
{ "mcpServers": { "freemapper-courseplan": { "command": "/absolute/path/to/freemapper-courseplan", "args": ["mcp"] } } }
```

Optional: `--lang de|en` before or after `mcp` sets the language of the texts the tools return (default:
`FREEMAPPER_LANG`, then the system language, then English). Tool and field names stay English.

### Tools

Every tool except `init_event` takes `project`, the project file of the event (`*.courseplan.toml`), as an
absolute path or one starting with `~`. The table lists the other parameters; `?` marks an optional one. A
position `Point` is an object `{x, y}` in map millimetres at map scale, y pointing down. A `Code` is a string or a
whole number.

| Tool | Parameters besides `project` | What it does |
|---|---|---|
| `init_event` | `map`, `project?`, `title?` | Makes a project file from an OOM map (here `project` is where to write it). |
| `event_info` | none | Summary: title, map, extent, counts, and per course controls, length, climb. |
| `list_controls` | none | Controls with code, kind, position, description, number placement. |
| `list_courses` | none | Courses with start, finish, controls, classes, climb, print scale, length in km. |
| `list_specials` | none | Special objects with geometry and courses. |
| `list_classes` | none | Classes with rule category and expected starters. |
| `map_extent` | none | Smallest and largest corner of the map objects, scale, map kind. |
| `map_features_near` | `x`, `y`, `radius_mm`, `limit?` | Map objects near a position, nearest first (default 20, at most 200). |
| `add_control` | `at`, `code?`, `crossing_point?` | Adds a control; without `code` the lowest free one from 31. |
| `move_control` | `code`, `at` | Moves a control. |
| `set_control_code` | `from`, `to` | Changes a code; courses and accepted findings follow. |
| `remove_control` | `code` | Removes a control and its visits. |
| `describe_control` | `code`, `c?` to `h?`, `clear?` | Sets description symbols of columns C to H as IOF symbol ids. |
| `set_control_number` | `code`, `direction?`, `gap_mm?`, `clear?` | Central number placement: `direction` and `gap_mm` together, or `clear`. |
| `add_special` | `kind`, `id`, `at?`, `points?`, `courses?` | `kind`: `start`, `finish`, `marked_route`, `out_of_bounds`, `forbidden_route`, `dangerous_area`, `first_aid`, `water`. Single points take `at`, the others `points`. |
| `remove_special` | `id` | Removes a special object. |
| `set_special_courses` | `id`, `courses?`, `all?` | Courses a special object belongs to; not for start and finish. |
| `add_class` | `name`, `rule_category?`, `expected_starters?` | Adds a class. |
| `set_class` | `name`, `rule_category?`, `no_rule_category?`, `expected_starters?`, `no_expected_starters?` | Changes a class. |
| `remove_class` | `name` | Removes a class. |
| `add_course` | `name`, `start`, `finish`, `controls?`, `classes?`, `climb_m?`, `print_scale?` | Adds an individual course; `start` and `finish` are ids of special objects. |
| `remove_course` | `name` | Removes a course. |
| `rename_course` | `from`, `to` | Renames a course. |
| `set_course_controls` | `name`, `codes` | Replaces the controls in running order. |
| `set_leg` | `name`, `from_start?`, `after?`, `bends?`, `taped?` | Bends and marked-route flag of one leg: give `from_start` or `after`. |
| `set_course_details` | `name`, `classes?`, `climb_m?`, `no_climb?`, `print_scale?`, `no_print_scale?` | Changes classes, climb, print scale. |
| `place_numbers` | `name`, `overwrite?` | Places the control numbers automatically; call again after changing legs. |
| `set_course_number` | `name`, `after`, `direction?`, `gap_mm?`, `clear?` | Number placement of one control in one course. |
| `check` | none | Tests the individual courses against the rule profile. |
| `accept_finding` | `rule`, `note`, `controls?`, `legs?`, `course?` | Records an intended exception with its reason. `legs` holds `{course, after}` objects. |
| `remove_accepted_finding` | `rule`, `controls?`, `legs?`, `course?` | Takes an exception back. |
| `set_rule_profile` | `base`, `overlays?` | Sets the rule profile and overlays. |
| `set_start_window` | `minutes?`, `no_start_window?` | Start window in minutes, or removes it. |
| `export_iof_xml` | `path` | Writes IOF XML 3.0 CourseData (`.xml`). |
| `export_ppen` | `path`, `new?` | Writes or patches a Purple Pen event (`.ppen`) and links the project to it. |
| `import_ppen` | `path`, `replace?` | Takes over controls, courses and special objects from a `.ppen`. |
| `sync_ppen` | `dry_run?`, `take?` | Merges the project with its linked `.ppen`; `take` is `ours` or `theirs`. |
| `resolve_sync_conflict` | `key`, `take` | Stores the answer to one open conflict; `take` is `ours` or `theirs`. |

### Workflow

1. Ask for the map (`.omap`) and what the courses are for. `init_event` with its absolute path.
2. `event_info`, then `map_extent`. Work out positions from the extent, or ask the person for them. Use
   `map_features_near` to see what lies at a position.
3. `add_special` for the start and the finish, `add_control` for each control, `add_course`, then `place_numbers`.
   `event_info` again reads the length. Add `add_class` and `set_class` where the checks need a rule category or
   expected starters, and `set_start_window` for the start capacity check.
4. `check`. Relay the findings with their source. Fix what is wrong; a finding the person wants as it is gets
   `accept_finding` with their reason in `note`. Never invent a reason.
5. Export with `export_iof_xml` or `export_ppen`, or exchange with Purple Pen through `import_ppen` and
   `sync_ppen`.
6. Tell the person to open the editor (`freemapper-courseplan edit <project>`) to look at the result.

Rules:

- Paths are absolute or start with `~`. Relative paths are refused.
- Each call saves its change. A refused change leaves the file as it was. If the project file changed after the
  call read it (the person moved a control in the editor), nothing is written; read the project again and repeat.
- Positions are in map millimetres at map scale. Controls must lie on the map.
- Before `sync_ppen` ask the person to close the event in Purple Pen. If it reports conflicts, show both values
  and let the person choose; `take` settles all of them at once and the person should decide that.
- `export_iof_xml` replaces an existing file, and `export_ppen` with `new: true` does too. Ask before you pass
  either on a file that exists. `import_ppen` with `replace: true` swaps controls, courses and special objects;
  ask first.
- Errors come back as one sentence meant for the person. Relay it; do not retry the same call unchanged.

## Use it from the command line

Use the CLI when no MCP client is available or the person wants a script. Pass `--lang en` when you parse output.
Every command takes the project file as `<PROJECT>`, except `init` (takes the map) and `mcp`.

```sh
freemapper-courseplan init hall.omap --title "Training hall"
freemapper-courseplan special add hall.courseplan.toml --kind start --id S1 --at 40,60
freemapper-courseplan special add hall.courseplan.toml --kind finish --id F1 --at 120,60
freemapper-courseplan control add hall.courseplan.toml --at 70,40
freemapper-courseplan course add hall.courseplan.toml Short --start S1 --finish F1 --controls 31
freemapper-courseplan course numbers hall.courseplan.toml Short
freemapper-courseplan check hall.courseplan.toml
```

| Command | Switches and arguments |
|---|---|
| `init <MAP>` | `--project <FILE>`, `--title <TITLE>`, `--ppen <FILE>` (start from a Purple Pen event) |
| `info <PROJECT>` | none |
| `control add` | `--at <X,Y>` (required), `--code <CODE>`, `--crossing-point` |
| `control move <PROJECT> <CODE>` | `--at <X,Y>` |
| `control code <PROJECT> <FROM> <TO>` | none |
| `control remove <PROJECT> <CODE>` | none |
| `control describe <PROJECT> <CODE>` | `--c`, `--d`, `--e`, `--f`, `--g`, `--h <ID>` (one at least) or `--clear` |
| `control number <PROJECT> <CODE>` | `--direction <DEG>` with `--gap <MM>`, or `--clear` |
| `special add` | `--kind start\|finish\|marked-route\|out-of-bounds\|forbidden-route\|dangerous-area\|first-aid\|water`, `--id <ID>`, `--at <X,Y>`, `--point <X,Y>` (repeat), `--courses <A,B>` |
| `special remove <PROJECT> <ID>` | none |
| `special courses <PROJECT> <ID> <A,B>` | or `--all` |
| `class add <PROJECT> <NAME>` | `--rule-category <CAT>`, `--starters <N>` |
| `class set <PROJECT> <NAME>` | `--rule-category`, `--no-rule-category`, `--starters`, `--no-starters` |
| `class remove <PROJECT> <NAME>` | none |
| `course add <PROJECT> <NAME>` | `--start <ID>`, `--finish <ID>` (both required), `--controls <31,32>`, `--classes <A,B>`, `--climb <M>`, `--print-scale <N>` |
| `course controls <PROJECT> <NAME> <CODES>` | none |
| `course rename <PROJECT> <FROM> <TO>` | none |
| `course remove <PROJECT> <NAME>` | none |
| `course leg <PROJECT> <NAME>` | `--from-start` or `--after <N>`; `--bend <X,Y>` (repeat), `--taped` |
| `course set <PROJECT> <NAME>` | `--classes`, `--climb`, `--no-climb`, `--print-scale`, `--no-print-scale` |
| `course number <PROJECT> <NAME>` | `--after <N>`; `--direction <DEG>` with `--gap <MM>`, or `--clear` |
| `course numbers <PROJECT> <NAME>` | `--overwrite` |
| `event profile <PROJECT> <BASE>` | `--overlay <ID>` (repeat) |
| `event start-window <PROJECT> <MINUTES>` | none |
| `check <PROJECT>` | `--all` (also accepted, unmatched and skipped) |
| `finding accept <PROJECT> <RULE>` | `--note <NOTE>` (required), `--controls <31,32>`, `--leg <COURSE:N>`, `--course <NAME>` |
| `finding remove <PROJECT> <RULE>` | `--controls`, `--leg`, `--course` |
| `export <PROJECT>` | `--iof-xml <FILE>`, `--ppen <FILE>`, `--new` |
| `import <PROJECT>` | `--ppen <FILE>` (required), `--replace` |
| `sync <PROJECT>` | `--dry-run`, `--take ours\|theirs\|KEY=SIDE` (repeat), `--json` |
| `edit <PROJECT>` | `--port <PORT>`, `--no-browser` |
| `mcp` | none |

Global switch: `--lang <LANG>` (`de`, `en`; default `FREEMAPPER_LANG`, then the system language, then English).
`--version` prints the version.

`edit` starts the editor in the browser on `127.0.0.1`, prints its address with an access token on one line and
runs until Ctrl+C or "Close editor" in the page. A person has to look at the page; you cannot work in it. Use the
other commands yourself.

Output and exit codes: results and progress go to stdout. On failure the program prints one line `error: <message>`
to stderr (with `--lang de`: `Fehler: …`) and exits with code 1; a usage error exits with code 2. `check` exits
with 1 when a finding has the severity error; accepted findings do not count. `sync` exits with 1 while conflicts
stay open or when it was refused. Everything else exits with 0 on success.

## What lands on disk

- `<name>.courseplan.toml` next to the map (or at `--project`): the project file, plain TOML, saved atomically.
  The person may commit it to git.
- After a sync, the base copy next to the project file, for example `event.courseplan.ppen-base`: commit it
  together with the project file.
- Files you export: the `.xml` for result software and the `.ppen` for Purple Pen.
- The `.omap` and `freemapper.toml` stay untouched; courseplan only reads them.

## What to tell the person

- Look at every control position on the map yourself. `check` does not know the terrain.
- Findings come with a source (document and section of the rules). Accepted findings carry the person's reason;
  `check --all` lists them.
- The IOF XML is not tested in OE. Ask the person to report problems.
- Open the editor (`edit`) to look at the courses on the map. After a change from outside, the editor shows the
  new state after about two seconds.

## Troubleshooting

Texts as the English catalog has them:

- `Forest maps (ISOM) are not supported yet; this version handles sprint and schoolyard maps (ADR 0010).`:
  nothing to fix on your side; tell the person.
- `<path> is a relative path; use an absolute path or one starting with ~` (MCP): pass an absolute path.
- `<path> changed on disk after it was read, so nothing was written. Read the project again and repeat the change`
  (MCP; the CLI says `<path> changed since it was read; nothing was written`): read the project again and repeat.
- `point (x, y) is outside the map, which spans (…) to (…)`: use positions inside the span.
- `control <code> already exists`: choose another code.
- `course <name> is not an individual course; only individual courses can be edited yet`: the program cannot
  handle team, relay or score courses.
- `<path> has no link to a Purple Pen file. Link it first with export --ppen FILE or import --ppen FILE`: link
  the project before `sync`.
- `<path> changed while the sync ran, probably because Purple Pen saved it. Nothing was written. Run sync again`:
  ask the person to close the event in Purple Pen, then run `sync` again.
- `the Purple Pen event is drawn for 1:<n> and the map is 1:<m>; export the courses for the map's scale`: the
  `.ppen` belongs to another scale.
- `only <n> of <m> controls lie on the map; the event belongs to another map`: the `.ppen` was drawn for another
  map.
- `<path> already exists; choose another project file or remove it` (`init`): pick another `--project`.
- macOS "cannot be opened" or "killed": remove the quarantine flag (install step 4).

## The freemapper suite

courseplan reads the `.omap` and the `freemapper.toml` that freemapper writes (look in the map folder for the
title). Each program has its own AGENTS.md:

- [freemapper](https://github.com/HenryJobst/freemapper/blob/main/AGENTS.md): the starter map from official
  geodata and OpenStreetMap.

## Working on the freemapper-courseplan code

This section is for coding agents in a clone of the repository, not for using the program.

- Build and check: `cargo build --release --locked`, `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`. The toolchain is
  pinned in `rust-toolchain.toml`.
- Read `CLAUDE.md`, the [design](docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md),
  [`docs/glossary.md`](docs/glossary.md) and [`docs/adr/`](docs/adr/README.md) before changing code. Source
  code and technical docs are English; user docs are German and English. German text says "Bundesland" for a state.
- Program texts live in Fluent files under `crates/freemapper-courseplan/i18n/<language>/courseplan.ftl`;
  English is the reference and a new message gets every language in the same commit.
- Every change gets an entry in `CHANGELOG.md` and `CHANGELOG.en.md`.
- Shared code comes from [freemapper-core](https://github.com/HenryJobst/freemapper-core), pinned by tag.
