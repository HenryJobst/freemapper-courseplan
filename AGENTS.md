# freemapper-courseplan for AI agents

This file tells an AI agent how to install freemapper-courseplan and how to use it for a person who sets
orienteering courses. It ships in every release archive next to the program. People read
[README.en.md](README.en.md) (English) or [README.md](README.md) (German); the [user manual](docs/manual/en.md) has
the details.

## What it does and what it does not

freemapper-courseplan sets courses on a finished map from OpenOrienteering Mapper (OOM, `.omap`): controls with
code and control description, start, finish, classes, individual courses with bends and marked legs, control
numbers placed behind the circle, and special objects (marked routes, forbidden routes, out-of-bounds and
dangerous areas, first aid posts, water points). Score courses are also supported: controls with values (the code
carries the value), a time limit, a penalty per minute over the limit and optional mandatory links. `check` tests the courses against a rule profile (built in:
`wkb-2026`, overlay `nor-bb-2021`) and names the source of each finding. Courses go out as IOF XML 3.0 for result
software and as Purple Pen events (`.ppen`), and a Purple Pen event can be taken over or merged back.

The program has no AI of its own. It does not see the map: it can only report the extent of the map and the map
objects near a position. Give it positions in map millimetres, or work them out from the extent.

Tell the person these limits before you start:

- Sprint and schoolyard maps work, and so do forest maps in ISOM 2017-2 at 1:5000 to 1:15000 (ADR 0092). A forest map
  in another symbol set (ISOM 2000, the first edition ISOM2017) is refused; the person converts it in Mapper. The map
  must be an `.omap` or `.xmap`. On a forest map you set courses by hand and run `check`; `score suggest`, `score draft`
  and `score length`, their MCP tools and the editor's Score panel refuse it, because the route search has no elevation
  and no vegetation speed. A forest course needs no new commands. Forest rules in `check` are the variants of the same
  rules: 30 m between controls, similar features within 60 m, controls in areas with the symbol 520 or 709. Rules
  for sprints only (`control.dead_end`, `course.start_finish_distance`) are listed as not for forest maps, and rules
  that need elevation (the Lkm rules of `nor-bb-2021`, which need the climb) or the route analysis are listed as not
  checked. The limit for the mapped area (`map.inside`, `map.margin`) is 10 mm on the sheet on a forest map: 100 m at
  1:10000. Say so to the person; no forest map in ISOM 2017-2 has been tried except a synthetic test map.
- Individual and score courses can be edited and checked. `info` gives a length for individual courses only; a
  score course is measured with `score length`. Team and relay courses are not built. IOF XML export writes individual and score courses; it skips the others and says so. The Purple Pen export skips score courses too and says so.
- A score course has controls with values, a time limit, a penalty and optional mandatory links. `score suggest`,
  `score draft` and `score length` read the map and search routes, so they take seconds: about 1 to 4 s for a draft on the 1.2 x 1.0 km Rummelsburger Bucht map (`score suggest` 1.1 s, a 12-minute draft 2.8 s, a 15-minute draft that misses its targets 3.8 s), and about 2 s for `score suggest`, 10 s for a 32-minute `score draft` and 1 s for `score length` on the 1.2 x 1.8 km Gleisdreieck map (release build, measured 2026-11-13 on a busy shared machine with a load average between 25 and 80, so an idle one may be faster; a draft that misses the targets can take as long or longer). Before the three limits of ADR 0085 to 0087 the same drafts took 15 s on Gleisdreieck and 1 to 2 s on the Rummelsburger Bucht, on a machine with a different load, so the numbers do not show the cost of the limits alone. Neither map reaches the usual 45 minutes. The largest limit reached on Gleisdreieck is 32 minutes with the good runner near the target. The Rummelsburger Bucht reaches 11 minutes with both runners inside their bands and 12 near the target; the 50 m margin from the edge of the mapped area leaves less of that strip (16 and 17 minutes before). A 45-minute score course needs a larger map than either. `examples/gleisdreieck` holds the 32-minute course. A draft
  has to start and finish outside buildings and blocked areas, or the program says the start cannot be reached.
  The routes come from a raster of the map symbols, without elevation.
  The length bands are targets with a tolerance (ADR 0083): the good runner scores 87 to 93 percent of all points, the
  very good runner finishes after 95 to 100 percent of the limit, and 3 percentage points beyond either band still
  count as near the target. A draft or `score length` is then a success with exit 0 and names the deviation; the JSON
  answers say `in_band`, `near` or `missed` in `length.state`. The bands and the tolerance are settings (`score
  settings --good-band MIN-MAX --very-good-band MIN-MAX --band-tolerance N`, tool `set_score_settings`, the Score panel
  of the editor); change them only when the person asks. A draft can miss the targets beyond the tolerance, for
  example when the free codes run out before they are met (30 controls at the highest value 5, 54 at 9) or when the
  map has too few usable sites. The command then exits with 1, names each missed band and ends with a hint. Try
  another limit or a higher `score settings --value-max`, and tell the person what happened. Sites inside or on the
  edge of an out-of-bounds map area (symbols 520, 709, 714) are never suggested or drafted (ADR 0082).
  Three more limits apply to generated courses and to `check`, and each is a setting (ADR 0085 to 0087): no control
  deeper than 60 m in a dead end (`dead_end_max_m`, 20 to 300, `check` warns), start and finish at most 400 m apart in a
  straight line (`start_finish_max_m`, 50 to 2000, `check` gives a hint, and a draft refuses a start and finish that are
  further apart), and start, finish and controls at least 50 m from the edge of the mapped area (`map_margin_m`, 0 to 200,
  `check` reports an error, and a draft refuses a start or finish that is closer). They live in the `[score]` table, which
  every event reads, and `score settings --dead-end-max M --start-finish-max M --map-margin M`, the tool
  `set_score_settings` and the Score settings panel change them. A manual course may deviate; change a setting only when
  the person asks.
- The score extension in the IOF XML (time limit, penalty, links, in the namespace `https://github.com/HenryJobst/freemapper-courseplan/ns/score/1`) is not tried in OE or any other reader; a strict reader may ignore it. The `Score` values are standard.
- The IOF XML export has not been tried in OE, because nobody on the project has OE. The Purple Pen export opens in
  Purple Pen 4.0 (checked once by the author).
- Purple Pen must be closed for the event before a `sync`, and opened again afterwards. Otherwise Purple Pen may
  overwrite the merged file.
- Only releases that carry the updater can update themselves. A version before the first one with `update` is
  replaced by unpacking a newer archive once.
- The editor draws score courses with their mandatory links and has a Score panel for suggest, draft and length;
  candidates and drafts stay in the page until accepted or applied. The editor draws the real map, but a map
  larger than one A3 sheet at map scale is clipped to its middle part; courses outside it stay on the overlay.
- The editor has a read-only print preview of a course, but the program does not print courses yet; printing waits
  for freemapper-print.
- The route analysis is missing. The rule that the best route must not lead through an out-of-bounds area is not
  checked and shows among the rules that were not checked. A straight leg across such an area gets no finding: the
  line between controls is not the running route (ADR 0080). `check` does report a control that stands inside an
  out-of-bounds area, either a special object of its course or a map area with the symbol 520, 709 or 714 (709 and
  520 on forest maps), as an error (ADR 0081). It also reports a start, finish, control or crossing point that lies in the blank part of
  the sheet, with no map object within 50 m, as an error (rule `wkb2026.map.inside`, ADR 0084); the map frame and layout
  symbols such as the north lines pattern do not count as objects. It does not test start, finish or crossing points
  against out-of-bounds areas, nor controls that are on no course; look at those yourself. A start, finish, control or
  crossing point closer than 50 m to the edge of the mapped area is an error too (rule `wkb2026.map.margin`, ADR 0087), a
  control deeper than 60 m in a dead end a warning (`wkb2026.control.dead_end`, ADR 0085) and a start more than 400 m from
  the finish a hint (`wkb2026.course.start_finish_distance`, ADR 0086). The last two apply to sprint and schoolyard maps only.
- OCAD is not needed and not tested.
- `check` tests distances, angles, codes, out-of-bounds areas and the mapped area, not whether a control stands at the
  right feature.
  The person looks at every position on the map.

## Install

freemapper-courseplan is one executable without an installer and without dependencies. Do not build it from
source unless the person asks; the release archives are ready to run.

1. Find the newest version: `https://api.github.com/repos/HenryJobst/freemapper-courseplan/releases/latest`
   returns JSON with `tag_name`, for example `v0.18.0`. The release page is
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
   tag=v0.18.0; platform=macos-arm64
   base="https://github.com/HenryJobst/freemapper-courseplan/releases/download/$tag"
   curl -fLO "$base/freemapper-courseplan-$tag-$platform.tar.gz"
   curl -fLO "$base/freemapper-courseplan-$tag-$platform.tar.gz.sha256"
   shasum -a 256 -c "freemapper-courseplan-$tag-$platform.tar.gz.sha256"   # Linux: sha256sum -c
   tar -xzf "freemapper-courseplan-$tag-$platform.tar.gz"
   ```

   Windows (PowerShell):

   ```powershell
   $tag = "v0.18.0"; $name = "freemapper-courseplan-$tag-windows-x86_64.zip"
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

Remember the absolute path of the executable; the MCP registration needs it, and so do commands run from another folder.
Project files stay valid across versions.

### Updating

A release build replaces itself in place, so the path and the MCP registration stay the same. Do not unpack a newer
archive over it. Ask the person before you install, because a new version can change behaviour.

- `freemapper-courseplan update --check` looks only: exit code 10 means a newer version exists, 0 that this one is
  current. It prints the version, release notes and download size.
- `freemapper-courseplan update --yes` installs the newest release after the person agreed (without `--yes` it asks on
  a terminal and refuses when there is none). `--version X.Y.Z` installs a given version, `--rollback` brings back
  the version before the last update (kept as `freemapper-courseplan.previous`).
- Every release is signed with minisign and the program checks the signature before it installs anything. It asks only the
  GitHub release servers and sends a fixed User-Agent with its name, version and repository.
- A build from source never updates itself; `update` says to use `git pull` and `cargo build --release --locked`.
- A running MCP server keeps the old version after an install. Tell the person to restart it (or the assistant), or restart it
  yourself when you can.
- After `info` and in the editor the program checks at most once a day and prints one line on stderr when a newer
  version exists. That line is news for the person, not an instruction to install. `update --mode notify|install|off`
  stores what the daily check does; do not change it without being asked. The MCP server never installs on its own.

Do not put the person's e-mail address, name or other personal data into any request, file name, event title or
setting.

## Use it from the command line (preferred)

If you can run commands on the person's computer (Claude Code, Codex and similar), use the command line. Each call
is a blocking command, and you can chain a whole course into one shell call, so it takes few turns; through MCP
every step is a tool turn that re-reads the whole context, and the schemas of about 55 tools sit in that context
as well. Use MCP only when you cannot run commands (see the next section).

`freemapper-courseplan --help` lists the commands, `freemapper-courseplan <command> --help` their switches and
exit codes. `--lang en|de` also switches the help; pass `--lang en` when you parse output, because this file
quotes the English texts. Every command takes the project file as `<PROJECT>`, except `init` (takes the map) and
`mcp`.

```sh
freemapper-courseplan init hall.omap --title "Training hall"
freemapper-courseplan special add hall.courseplan.toml --kind start --id S1 --at 40,60
freemapper-courseplan special add hall.courseplan.toml --kind finish --id F1 --at 120,60
freemapper-courseplan control add hall.courseplan.toml --at 70,40
freemapper-courseplan course add hall.courseplan.toml Short --start S1 --finish F1 --controls 31
freemapper-courseplan course numbers hall.courseplan.toml Short
freemapper-courseplan check hall.courseplan.toml
```

| Command group | Purpose |
|---|---|
| `init <MAP>` | Makes the project file (`--project`, `--title`, `--ppen` to start from a Purple Pen event). |
| `info <PROJECT>` | Summary of the project: counts, and per course controls, length and climb (a score course: controls, points, time limit). |
| `list controls\|courses\|specials\|classes` | The details as a table; `--json` gives the MCP answer. |
| `map extent`, `map near --at X,Y --radius MM` | Extent, scale and kind of the map; the map objects near a point (`--limit`, `--json`). |
| `control`, `special`, `class`, `course` | Add, change and remove controls, special objects, classes and courses; `course numbers` places the control numbers; `course add --kind score --time-limit N [--penalty N]` adds a score course. Positions are `--at X,Y` in map millimetres. |
| `score controls`, `score value`, `score link add\|remove`, `score settings`, `score course set` | Edit score courses: controls as `31,42=2` (the code carries the value), the value of one control, mandatory links, the event's score settings (`--value-max`, `--good-pace`, `--very-good-pace`, `--good-band`, `--very-good-band`, `--band-tolerance`, `--mix-hard`, `--seed`, `--dead-end-max`, `--start-finish-max`, `--map-margin`), time limit and penalty (`--no-penalty` restores the default). Each takes `--json`. |
| `score reject`, `score unreject` | `reject --at X,Y` leaves a candidate site out of every later `suggest` and `draft`; the position is stored in the project file (`rejected_sites`) and a site within 3 m counts as the same. `unreject --at X,Y` takes it back. Use them when the person says a site is wrong; never reject on your own. Each takes `--json`. |
| `score suggest`, `score draft`, `score length` | `suggest` ranks control sites by route choice (`--limit`, `--min-distance MM`, `--course NAME` for the start and finish of a score course, `--start`, `--finish`) and writes nothing. `draft NAME --time-limit MIN` works out a set of controls with codes so that both runner profiles reach the target length bands, within a tolerance; it writes only with `--apply`, then creates the score course or replaces the controls of an existing one. `--link "X,Y;X,Y"` (repeatable) names two candidate positions to run one after the other. A draft that misses a band beyond the tolerance exits with 1 and names each miss and what would help; one near the target exits with 0 and names the deviation. `length NAME` analyses an existing course and changes nothing. Each takes `--json`. |
| `export --csv FILE --course NAME`, `export --sheet FILE --course NAME` | Control list (`code,value,x_mm,y_mm`, map millimetres) and Markdown evaluation sheet of a score course; other courses are refused. |
| `event profile`, `event start-window` | Rule profile with overlays, and the start window (`--none` removes it). |
| `event set` | The date (`--date YYYY-MM-DD`) and the organizer (`--organizer TEXT`) of the event; `--no-date` and `--no-organizer` remove them. At least one option. `info` shows both; the IOF XML export writes them as `Event/StartTime/Date` and `Event/Organiser/Name` (only when set). Purple Pen has no field for them. |
| `check` | Tests the courses; `--all` adds accepted findings, unmatched acceptances and skipped rules. |
| `finding accept`, `finding remove` | Record or take back an intended exception (`--note` is required). |
| `export`, `import`, `sync` | IOF XML and Purple Pen. `sync --dry-run` shows the merge first; `--json` prints the result as JSON. |
| `update` | Checks for a newer release and installs it (`--check`, `--yes`, `--version X.Y.Z`, `--rollback`, `--mode notify\|install\|off`); see Updating above. Exit code 10 with `--check` means an update exists. |
| `edit` | Editor for the person (see below). |

The commands follow the tool names of the MCP section: `add_control` is `control add`, `set_leg` is `course leg`,
and so on. The rules at the end of the MCP section apply here too: ask before `export` replaces a file (also
`export --ppen --new`) or `import --replace` swaps content, and let the person decide `sync` conflicts.

Reading commands, so you do not have to open the project file: `list controls|courses|specials|classes` print
tables (positions in map millimetres, descriptions and number placements only with `--json`), `map extent` prints
the extent of the map content, the scale and the map kind, and `map near --at X,Y --radius MM [--limit N]` lists
the map objects near a point, nearest first. Use them to find usable positions. With `--json` each prints exactly
what the MCP tool of the same name returns (`list_controls`, `map_features_near`, ...), including `"ok": true`.

Output and exit codes: results and progress go to stdout. On failure the program prints one line `error: <message>`
to stderr (with `--lang de`: `Fehler: …`) and exits with code 1; a usage error exits with code 2. `check` exits
with 1 when a finding has the severity error; accepted findings do not count. `sync` exits with 1 while conflicts
stay open or when it was refused. Everything else exits with 0 on success.

`edit <PROJECT>` starts the editor in the browser on `127.0.0.1` (`--port`, `--no-browser`), prints its address
with an access token on one line and blocks until Ctrl+C or "Close editor" in the page. A person has to look at
the page; you cannot work in it. If you start it for the person, run it in the background (or with
`--no-browser` and pass on the address) and stop it when they are done; a foreground call would sit there until
your command timeout. No other command is a long-running one (the score commands take seconds), so no timeout needs changing.

Workflow:

1. Ask for the map (`.omap`) and what the courses are for. `init <map>` with its path.
2. `info`, `map extent` and `map near`. Positions are map millimetres; ask the person for them, or take them from
   the map, or from a Purple Pen event.
3. `special add` for the start and the finish, `control add` for each control, `course add`, then
   `course numbers`. `info` again reads the length. Add `class add` and `class set` where the checks need a rule
   category or expected starters, and `event start-window` for the start capacity check.
4. `check`. Relay the findings with their source. Fix what is wrong; a finding the person wants as it is gets
   `finding accept --note` with their reason. Never invent a reason.
5. `export --iof-xml` or `export --ppen`, or exchange with Purple Pen through `import` and `sync`.
6. Tell the person to open the editor (`edit <project>`) to look at the result.

For a score course: `special add` for start and finish outside buildings, `score settings` if the paces differ,
`score suggest` to see the sites, then `score draft NAME --time-limit MIN` without `--apply` to read the draft,
and with `--apply` to write it. `score length NAME` checks a course the person changed by hand. `check` tests the
result.

## Use it through MCP

For assistants that cannot run commands (Claude Desktop, for example). `freemapper-courseplan mcp` is an MCP server on stdin/stdout. Register it once with the absolute path:

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

Every tool except `init_event`, `check_update` and `install_update` takes `project`, the project file of the event (`*.courseplan.toml`), as an
absolute path or one starting with `~`. The table lists the other parameters; `?` marks an optional one. A
position `Point` is an object `{x, y}` in map millimetres at map scale, y pointing down. A `Code` is a string or a
whole number.

| Tool | Parameters besides `project` | What it does |
|---|---|---|
| `init_event` | `map`, `project?`, `title?` | Makes a project file from an OOM map (here `project` is where to write it). |
| `event_info` | none | Summary: title, date, organizer, map, extent, counts, and per course controls, length, climb. |
| `list_controls` | none | Controls with code, kind, position, description, number placement. |
| `list_courses` | none | Courses with start, finish, controls, classes, climb, print scale, length in km. |
| `list_specials` | none | Special objects with geometry and courses. |
| `list_classes` | none | Classes with rule category and expected starters. |
| `map_extent` | none | Smallest and largest corner of the map objects, scale, map kind. |
| `map_features_near` | `x`, `y`, `radius_mm`, `limit?` | Map objects near a position, nearest first (default 20, at most 200). |
| `add_control` | `at`, `code?`, `crossing_point?` | Adds a control; without `code` the lowest free one from 31. |
| `move_control` | `code`, `at` | Moves a control. |
| `set_control_code` | `from`, `to` | Changes a code; courses and accepted findings follow. A score control takes the value its new code carries. |
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
| `set_course_details` | `name`, `classes?`, `climb_m?`, `no_climb?`, `print_scale?`, `no_print_scale?` | Changes classes, climb, print scale; works on score courses too. |
| `add_score_course` | `name`, `start`, `finish`, `time_limit_min`, `penalty_per_min?` | Adds a score course without controls. Without `penalty_per_min` the course stores none and the penalty defaults to the highest control value of the course, at least `value_max`; answers give `penalty_is_default`. |
| `set_score_controls` | `name`, `controls` | Replaces the controls: a list of `{code, value?}`. Without `value` the code gives it (second digit of a two-digit code, last digit of a longer one); a different value is refused. |
| `set_control_value` | `name`, `code`, `value` | Accepts only the value the code carries (repairs a drifted file); to change a value, change the code. |
| `add_score_link`, `remove_score_link` | `name`, `a`, `b` | Mandatory link between two controls of a score course, either direction. A runner who skips it is disqualified. At most two links per control, no ring. |
| `set_score_settings` | `value_max?`, `good_pace_s_per_km?`, `very_good_pace_s_per_km?`, `good_band_percent?`, `very_good_band_percent?`, `band_tolerance_percent?`, `mix_hard_percent?`, `seed?`, `dead_end_max_m?`, `start_finish_max_m?`, `map_margin_m?`, … | Event-wide score settings (no `name`, takes `project` only). The bands are `[low, high]` percent (the good runner scores that share of all points, the very good runner finishes after that share of the limit), the tolerance is 0 to 10 percentage points beyond a band that still count as near the target (default 3). |
| `set_score_course_details` | `name`, `time_limit_min?`, `penalty_per_min?`, `no_penalty?` | Time limit and penalty of a score course; `no_penalty` goes back to the default penalty. |
| `reject_site`, `unreject_site` | `at` | Reject a candidate site of `score_suggest` so that it and `score_draft` leave it out, or take the rejection back. A site within 3 m of a rejected one counts as the same. Answers `at`, `added` or `removed`, and `rejected_sites`. Only on the person's word. |
| `score_suggest` | `limit?`, `min_distance_mm?`, `start?`, `finish?` | Control site candidates, best route choice first: position, `symbol_code`, `route_choice`, `difficulty`, `effort_s`, suggested `value`, and a free `code` that carries that value (`null` when no free code is left; only the best-ranked candidates get one, so the code space is not overrun). Writes nothing. Takes seconds. |
| `score_draft` | `name`, `time_limit_min`, `start?`, `finish?`, `links?`, `apply?` | Draft of a score course: controls with code, value and position, and the length report. `links` is a list of `{a: {x, y}, b: {x, y}}`, each end a position from `score_suggest`. Writes only with `apply: true`. `length.state` is `in_band`, `near` (within the tolerance, a success) or `missed`. A draft that misses a band beyond the tolerance is an error result with one sentence per miss and the hint. |
| `score_length` | `name` | Round trips of both runner profiles on an existing score course, with the bands, `length.state` and each deviation. Reports only. |
| `place_numbers` | `name`, `overwrite?` | Places the control numbers automatically; call again after changing legs. |
| `set_course_number` | `name`, `after`, `direction?`, `gap_mm?`, `clear?` | Number placement of one control in one course. |
| `check` | none | Tests the individual and score courses against the rule profile. |
| `accept_finding` | `rule`, `note`, `controls?`, `legs?`, `course?` | Records an intended exception with its reason. `legs` holds `{course, after}` objects. |
| `remove_accepted_finding` | `rule`, `controls?`, `legs?`, `course?` | Takes an exception back. |
| `set_rule_profile` | `base`, `overlays?` | Sets the rule profile and overlays. |
| `set_event_details` | `date?`, `organizer?`, `no_date?`, `no_organizer?` | Date as `YYYY-MM-DD` and organizer text of the event; give at least one, and never `date` with `no_date` or `organizer` with `no_organizer`. `event_info` returns both. |
| `set_start_window` | `minutes?`, `no_start_window?` | Start window in minutes, or removes it. |
| `export_iof_xml` | `path` | Writes IOF XML 3.0 CourseData (`.xml`). |
| `export_score_csv`, `export_score_sheet` | `course`, `path` | CSV (`.csv`) or evaluation sheet (`.md`) of a score course. |
| `export_ppen` | `path`, `new?` | Writes or patches a Purple Pen event (`.ppen`) and links the project to it. |
| `import_ppen` | `path`, `replace?` | Takes over controls, courses and special objects from a `.ppen`. |
| `sync_ppen` | `dry_run?`, `take?` | Merges the project with its linked `.ppen`; `take` is `ours` or `theirs`. |
| `resolve_sync_conflict` | `key`, `take` | Stores the answer to one open conflict; `take` is `ours` or `theirs`. |
| `check_update` | none | Whether a newer release exists, with version, date, notes link and size. Installs nothing. Takes no `project`. |
| `install_update` | `confirm` | Installs the newest release over the running program. Only with `confirm: true`, and only after the person agreed. Takes no `project`. |

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

For a score course: `score_suggest`, then `score_draft` without `apply` to read the draft and with `apply: true` to
write it; `score_length` checks an existing course. Each takes seconds.

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
- The first tool result of a session can end with a note that a newer release exists. Pass it on to the person and ask
  before you call `install_update`. The server never installs by itself. After an install, the server still runs the old
  version: tell the person to restart it, which the answer of `install_update` also says.
- Errors come back as one sentence meant for the person. Relay it; do not retry the same call unchanged.

## What lands on disk

- `<name>.courseplan.toml` next to the map (or at `--project`): the project file, plain TOML, saved atomically.
  The person may commit it to git.
- After a sync, the base copy next to the project file, for example `event.courseplan.ppen-base`: commit it
  together with the project file.
- Files you export: the `.xml` for result software and the `.ppen` for Purple Pen. A score course also exports as a CSV control list and a Markdown evaluation sheet.
- The `.omap` and `freemapper.toml` stay untouched; courseplan only reads them.
- `settings.toml` (the update mode, only after `update --mode`) and `update-check.json` (the time of the last check) in
  the folder `freemapper-courseplan` of the system's configuration folder; `FREEMAPPER_COURSEPLAN_CONFIG_DIR` moves it.
- After an update, `freemapper-courseplan.previous` next to the program, until the next update replaces it.

## What to tell the person

- Look at every control position on the map yourself. `check` does not know the terrain.
- Findings come with a source (document and section of the rules). Accepted findings carry the person's reason;
  `check --all` lists them.
- The IOF XML is not tested in OE. Ask the person to report problems.
- Open the editor (`edit`) to look at the courses on the map. After a change from outside, the editor shows the
  new state after about two seconds.

## Troubleshooting

Texts as the English catalog has them:

- `This forest map uses the symbol set <name>, but only ISOM 2017-2 is supported for forest maps (ADR 0092).`
  (`init`): the map is ISOM 2000, the first edition ISOM2017 or another set. Nothing to fix on your side; tell the
  person to convert the map in Mapper (Mapper ships `ISOM2000-ISOM 2017-2.crt` and `ISOM2017-ISOM 2017-2.crt`). The same
  message appears when a `freemapper.toml` says `map_kind = "forest"` for a map of another symbol set.
- `Score suggest, draft and length do not work on forest maps: ...` (`score suggest|draft|length`, the MCP tools, the
  editor): do not retry. Set the score course by hand (`score controls`, `score link add`); `check` then lists its
  length bands as not checked.
- `<path> is a relative path; use an absolute path or one starting with ~` (MCP): pass an absolute path.
- `<path> changed on disk after it was read, so nothing was written. Read the project again and repeat the change`
  (MCP; the CLI says `<path> changed since it was read; nothing was written`): read the project again and repeat.
- `point (x, y) is outside the map, which spans (…) to (…)`: use positions inside the span.
- `control <code> already exists`: choose another code.
- `course <name> is not an individual course; only individual and score courses can be edited yet`: the program
  cannot handle team or relay courses.
- `course <name> is a score course; this change works on individual courses only`: legs, bends, control numbers and
  `course controls` belong to individual courses. Use the score commands for a score course.
- `control <code> carries the value <n> in its code, not <m>; change the code instead`: a score control's value is
  the one its code carries. Pick another code (second digit is the value), or drop the value.
- `the start at (x, y) cannot be reached: it lies in a blocked area` (the finish reads alike): `score suggest`,
  `score draft` and `score length` need a start and finish outside buildings and blocked areas. Add a new start or
  finish with `special add` at a free position (`map near` shows what lies there); do not move the person's own.
- `The good runner scores 100 percent of all points ...` or `The very good runner ...` (`score draft`, exit 1): the
  draft misses a length band beyond the tolerance and names the miss. The last line gives a hint, such as a shorter limit, too few
  suitable sites on the map, or free codes that run out (raise the highest value or free codes). Try another limit and tell the person which band was missed; never present such a draft as a
  success. A draft near the target is a success; say that it is near and by how much.
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
- `cannot write to the program folder <path>, and freemapper-courseplan does not ask for administrator rights` (`update`):
  the program sits where the person cannot write. Unpack the new archive by hand or move the program to a writable
  folder; do not use `sudo`.
- `the release could not be verified (...)` (`update`): nothing was installed. Tell the person; do not install the release
  some other way.
- `another update or rollback is running in the program folder`: wait and try again.

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
