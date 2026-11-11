# freemapper-courseplan

[Deutsche Version](README.md) · [User manual](docs/manual/en.md) · [Glossary](docs/glossary.md) · [Changes](CHANGELOG.en.md)

<!-- claude-cost:start -->
[![AI effort (Claude API): $521](https://img.shields.io/badge/AI%20effort%20%28Claude%20API%29-%24521-8A2BE2)](docs/ai-cost/history.csv)
<!-- claude-cost:end -->

freemapper-courseplan is the course-setting program of the freemapper suite. You set courses by hand in an
editor in the browser, on the finished map from OpenOrienteering Mapper (OOM). The program checks them against
the competition rules and hands them on to result software and to printing:

- [freemapper](https://github.com/HenryJobst/freemapper) produces the starter map,
- the mapper draws in OOM,
- freemapper-courseplan sets the courses,
- [freemapper-print](https://github.com/HenryJobst/freemapper-print) prints the map sheets.

Code that several of these programs need lives in a shared library:
[freemapper-core](https://github.com/HenryJobst/freemapper-core). It is a set of small Rust crates, for example
`freemapper-text` for the program texts, `freemapper-omap` for reading the map, `freemapper-course` for reading
Purple Pen files and `freemapper-web` for the editor's local server. The programs use them in a fixed version
(courseplan: 0.14.1) instead of copying the code. The number placement and the description symbol tables come from
there, so courseplan and freemapper-print place numbers the same way.

Planned for the first version:

- controls with code and control description, individual courses with bends, marked routes and forbidden
  areas;
- control number positions set once per control and changeable per course, with automatic placement behind the
  circle, away from point and line symbols of the map, at 0.5 to 5 mm from the circle (checked by `check`);
- checks against the German competition rules (WKB) 2026 with overlays for regional recommendations, such as
  control distances, acute and almost straight angles and enough different first controls for a compact start
  list, each with its source and with exceptions you can give reasons for (`check`, usable now);
- exchange with Purple Pen (`.ppen`) in both directions, so you can use both programs side by side (import and
  export and sync are usable now: `import`, `export --ppen` and `sync`; Purple Pen 4.0 opens the export);
- export to IOF XML 3.0 for result software, for example OE (`export`, usable now; the import into OE is not tested yet);
- score courses with point values carried by the control code, a time limit, a penalty and mandatory links, with
  suggested control sites, a draft that meets the length bands, checks and export (`score`, usable now);
- sprint and schoolyard maps; forest maps come later;
- use through the editor in the browser (`edit`, usable now: controls, courses, categories, bends, numbers and
  special objects and control descriptions, findings of the check, export, import and sync with Purple Pen, with undo and
  redo and a print preview; on the real map),
  the command line or an AI assistant (MCP).

After that come a route analysis, team orienteering with mandatory and free controls, relays,
tours for the people who put out the controls, and AI support that judges courses and makes proposals.

The program needs no OCAD and runs on macOS, Linux and Windows.

## Example

<a href="examples/luebeck-sprint/"><img src="examples/luebeck-sprint/preview.jpg" width="260" align="right" alt="Sketch of a sprint course in Lübeck's old town"></a>

A sprint course with seven controls in Lübeck's old town, set with the command line only. A second project file there adds a score course with 9 controls and a 15-minute limit. The image is a sketch:
the map comes from freemapper-print, the course is drawn on top. Commands, project file and output are in
[`examples/luebeck-sprint`](examples/luebeck-sprint/); [`examples/README.en.md`](examples/README.en.md) lists all
examples.

```text
$ freemapper-courseplan info luebeck-sprint.courseplan.toml
title: Sprint training Lübeck old town
map kind: sprint
controls: 7, crossing points: 0
course A: 7 controls, 1.3 km, climb 5 m
```

<br clear="right">

## Installing

Ready-made programs for Windows, macOS and Linux are on the
[releases page](https://github.com/HenryJobst/freemapper-courseplan/releases). Download the archive, unpack it and
start `freemapper-courseplan` in a terminal or register it with your AI assistant (user manual, chapter 3). An
assistant that can run commands (Claude Code, Codex) can also do the download itself and call the program directly; tell it
"Read AGENTS.md from github.com/HenryJobst/freemapper-courseplan and set up freemapper-courseplan." The file
[`AGENTS.md`](AGENTS.md) is in every archive.

## Status

freemapper-courseplan is in early development. On a sprint or schoolyard map (`.omap`) you can already set whole
individual courses. `freemapper-courseplan init <map>` creates the project file. With `control`, `special`,
`class` and `course` you add controls, start, finish, classes and courses, set bends and have the control numbers
placed. `freemapper-courseplan info <project>` shows what the project holds, including the length of each course; `list` and `map` report the details and the map, and `--json` gives the same answer as the MCP tools.
An AI assistant can do the same for you through the MCP server `freemapper-courseplan mcp`; chapter 3 of the
[user manual](docs/manual/en.md#3-with-an-ai-assistant) explains how to set it up, and [`AGENTS.md`](AGENTS.md) tells the assistant
how to install and use the program itself. The project file is TOML, and
you can edit it by hand too. `import`, `export --ppen` and `sync` exchange events with Purple Pen. The program cannot edit team or relay courses yet. It cannot print
courses either, and it does not read forest maps.

A score course has controls with points, a time limit and a penalty per minute over it, and optionally mandatory
links that the runner has to run directly after each other. The code carries the value: control 43 is worth 3
points. `course add --kind score` makes the course, and `score controls`, `score value`, `score link`, `score
settings` and `score course set` edit it. `score suggest` ranks control sites on the map by route choice,
`score draft` picks controls, values and codes so that a good runner scores about 90 percent of the points and a
very good runner just reaches all of them within the limit, and `score length` re-measures a course after you
changed it. The draft writes nothing until you add `--apply`. Ten rules of `check` read score courses (value and
code, alike codes, ring and obvious order, length bands, limit and penalty, links, mix of hard controls, gaps between controls), the editor
has a Score panel, and `export --csv` and `export --sheet` write the control list and the evaluation sheet. Routes
come from a raster of the map symbols without elevation, and the runner profiles are a first version, so treat the
length bands as a good starting point and check the result on the map. The AI assistant has the tools `score_suggest`,
`score_draft`, `score_length` and the tools for editing. Chapter 3 of the
[user manual](docs/manual/en.md#score-courses) explains it, the Lübeck example holds a finished course, and the
decisions are in ADR 0060 to 0070.

`freemapper-courseplan check <project>` checks the individual and score courses against a rule profile, each finding with
severity, place, rule and source. The built-in profile is `wkb-2026` (competition rules 2026) and the overlay
`nor-bb-2021` (course recommendations for Berlin and Brandenburg); `freemapper-courseplan event profile <project>
wkb-2026 --overlay nor-bb-2021` selects them. `finding accept <project> <rule> --note TEXT` records an intended
exception with its reason, `finding remove` takes it back, and `check --all` also lists the accepted findings, the
exceptions that match no finding any more and the skipped rules. `event start-window` and `class set` add the start
window, rule categories and expected starters that some checks need. Chapter 8 of the
[user manual](docs/manual/en.md#8-checking-reading-findings-giving-reasons-for-exceptions) explains it, and the AI
assistant has the tools `check`, `accept_finding`, `remove_accepted_finding`, `set_rule_profile`,
`set_start_window` and `set_class` for it. The program does not check team or relay courses yet, and the
route analysis is missing, so it reports a straight leg through an out-of-bounds area only as a warning.

`freemapper-courseplan export <project> --iof-xml <file>` writes the individual and score courses as IOF XML 3.0 for result
software; the AI assistant has the tool `export_iof_xml`. The output validates against the IOF schema and reads
back through the IOF reader of freemapper-course (freemapper-print's); a finished example is
[`course-data.xml`](examples/luebeck-sprint/course-data.xml). An import into OE, Purple Pen or Condes has not been
tested, because nobody on the project has OE. Reports from people with OE are welcome. If the map is georeferenced, the start, controls and finish also carry
latitude and longitude (ADR 0039). Taped legs are not written yet, and team and relay courses are skipped. A score course carries its time limit, penalty and links in an extension element of its own. Chapter 10 of the
[user manual](docs/manual/en.md#10-handing-on-iof-xml-for-oe) has the details.

`freemapper-courseplan init <map> --ppen <file>` and `freemapper-courseplan import <project> --ppen <file>`
take over controls, descriptions, courses, bends, marked routes, number positions and special objects from a Purple Pen event
drawn for the same map; the AI assistant has the tool `import_ppen`. What the model cannot hold, such as variations,
score courses or gaps, the program names in a note. `import` replaces a project that already has content only with
`--replace`; classes and accepted findings stay. Chapter 9 of the
[user manual](docs/manual/en.md#9-exchanging-with-purple-pen) explains it.

`freemapper-courseplan export <project> --ppen <file>` writes the individual courses as a Purple Pen event (score courses are left out, with a note); the AI
assistant has the tool `export_ppen`. It changes an existing file in place and leaves alone what the project does not
know, such as circle gaps or map exchange; with `--new` it writes the file from scratch. It never deletes anything and
reports what is in the file but not in the project. The writer is `freemapper-course` in freemapper-core. The example
[`luebeck-sprint.ppen`](examples/luebeck-sprint/luebeck-sprint.ppen) is such an export. **Purple Pen 4.0 opens
the file without an error, and its content looks right** (checked by the author, 2026-11-10).

`freemapper-courseplan sync <project>` merges the project with its linked `.ppen` in both directions. It compares the
file as the last sync left it (the base copy `X.courseplan.ppen-base`, to be committed to git), the file now and the
project now, field by field. A change on one side goes to the other. If both sides changed the same field differently,
`sync` lists both values and writes nothing until you pick a side with `--take ours`, `--take theirs` or
`--take KEY=SIDE`; `--dry-run` only reports. Classes, accepted findings and team, relay and score courses stay in the
project. The AI assistant has the tools `sync_ppen` and `resolve_sync_conflict`. The editor has a conflict panel for it (Event
menu). See chapter 9 of the [user manual](docs/manual/en.md#9-exchanging-with-purple-pen) and ADR 0040.

`freemapper-courseplan edit <project>` starts the editor and shows the event in the browser: on the left the tree of
courses, categories, controls and special objects, in the centre the selected course with circles, legs and numbers, on
the right the properties. You place, move and delete controls, build courses and categories, bend and tape legs, drag
or place control numbers, pick the description symbols of columns C to H and draw special objects with the mouse or the
keyboard; Ctrl+Z and Ctrl+Shift+Z undo and redo steps. Every change is saved at once through the same operations as the
command line and the AI assistant. You see outside changes in the browser after about two seconds; if the file changed
under the editor, it refuses your change and loads the new state. The findings of the check appear in the panel, on the map and in the status bar, and you can accept a finding with a
note. The Event menu exports IOF XML and Purple Pen files, imports a Purple Pen file and syncs with the linked one; close
the event in Purple Pen before you sync. The map is the real one, drawn as SVG by the renderer that freemapper-print
uses too (freemapper-render). A map larger than one A3 sheet at map scale is clipped to its middle part, and the
status bar says so. The "Print preview" button shows the selected course the way freemapper-print draws it, read-only and without the overlay (ADR 0057). The server listens on `127.0.0.1` only and needs an
access token in the address. See chapter 4 of the [user manual](docs/manual/en.md#4-the-editor) and ADR 0042,
0043, 0047, 0048, 0049, 0051, 0052, 0053, 0054 and 0056.

The repository also holds the [design](docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md), the
[glossary](docs/glossary.md), the [architecture decisions](docs/adr/README.md) and a
[collection of course-setting rules](docs/research/2026-11-08-bahnlegung-kriterien.md) (German).

## Building from source (for developers)

```sh
git clone https://github.com/HenryJobst/freemapper-courseplan
cd freemapper-courseplan
cargo build --release
```

`rust-toolchain.toml` fixes the Rust version; `rustup` installs it on the first build. Before each commit,
[lefthook](https://github.com/evilmartians/lefthook) checks formatting and clippy (`lefthook install`).

Whoever changes freemapper-core and this program together checks freemapper-core out next to freemapper-courseplan
and uses a local `[patch]` during development. The details are in the README of
[freemapper-core](https://github.com/HenryJobst/freemapper-core).

## Licence

freemapper-courseplan is licensed [GPL-3.0-or-later](LICENSE), like freemapper and freemapper-print (ADR 0002).
