# freemapper-courseplan

[Deutsche Version](README.md) · [User manual](docs/manual/en.md) · [Glossary](docs/glossary.md) · [Changes](CHANGELOG.en.md)

<!-- claude-cost:start -->
[![AI effort (Claude API): $246](https://img.shields.io/badge/AI%20effort%20%28Claude%20API%29-%24246-8A2BE2)](docs/ai-cost/history.csv)
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
`freemapper-text` for the program texts, `freemapper-omap` for reading the map and `freemapper-course` for reading
Purple Pen files. The programs use them in a fixed version instead of copying the code.

Planned for the first version:

- controls with code and control description, individual courses with bends, marked routes and forbidden
  areas;
- control number positions set once per control and changeable per course, with automatic placement behind the
  circle, away from point and line symbols of the map, at 0.5 to 5 mm from the circle (checked by `check`);
- checks against the German competition rules (WKB) 2026 with overlays for regional recommendations, such as
  control distances, acute and almost straight angles and enough different first controls for a compact start
  list, each with its source and with exceptions you can give reasons for (`check`, usable now);
- exchange with Purple Pen (`.ppen`) in both directions, so you can use both programs side by side (the import is
  usable now, `import`);
- export to IOF XML 3.0 for result software, for example OE (`export`, usable now; the import into OE is not tested yet);
- sprint and schoolyard maps; forest maps come later;
- use through the editor in the browser, the command line or an AI assistant (MCP).

After that come a route analysis, team orienteering with mandatory and free controls, relays, score courses,
tours for the people who put out the controls, and AI support that judges courses and makes proposals.

The program needs no OCAD and runs on macOS, Linux and Windows.

## Example

<a href="examples/luebeck-sprint/"><img src="examples/luebeck-sprint/preview.jpg" width="260" align="right" alt="Sketch of a sprint course in Lübeck's old town"></a>

A sprint course with seven controls in Lübeck's old town, set with the command line only. The image is a sketch:
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
start `freemapper-courseplan` in a terminal or register it with your AI assistant (user manual, chapter 3).

## Status

freemapper-courseplan is in early development. On a sprint or schoolyard map (`.omap`) you can already set whole
individual courses. `freemapper-courseplan init <map>` creates the project file. With `control`, `special`,
`class` and `course` you add controls, start, finish, classes and courses, set bends and have the control numbers
placed. `freemapper-courseplan info <project>` shows what the project holds, including the length of each course.
An AI assistant can do the same for you through the MCP server `freemapper-courseplan mcp`; chapter 3 of the
[user manual](docs/manual/en.md#3-with-an-ai-assistant) explains how to set it up. The project file is TOML, and
you can edit it by hand too. The program cannot edit or measure team, relay or score courses yet. It cannot print
courses either, and it does not read forest maps.

`freemapper-courseplan check <project>` checks the individual courses against a rule profile, each finding with
severity, place, rule and source. The built-in profile is `wkb-2026` (competition rules 2026) and the overlay
`nor-bb-2021` (course recommendations for Berlin and Brandenburg); `freemapper-courseplan event profile <project>
wkb-2026 --overlay nor-bb-2021` selects them. `finding accept <project> <rule> --note TEXT` records an intended
exception with its reason, `finding remove` takes it back, and `check --all` also lists the accepted findings, the
exceptions that match no finding any more and the skipped rules. `event start-window` and `class set` add the start
window, rule categories and expected starters that some checks need. Chapter 8 of the
[user manual](docs/manual/en.md#8-checking-reading-findings-giving-reasons-for-exceptions) explains it, and the AI
assistant has the tools `check`, `accept_finding`, `remove_accepted_finding`, `set_rule_profile`,
`set_start_window` and `set_class` for it. The program does not check team, relay or score courses yet, and the
route analysis is missing, so it reports a straight leg through an out-of-bounds area only as a warning.

`freemapper-courseplan export <project> --iof-xml <file>` writes the individual courses as IOF XML 3.0 for result
software; the AI assistant has the tool `export_iof_xml`. The output validates against the IOF schema, and a
finished example is [`course-data.xml`](examples/luebeck-sprint/course-data.xml). Whether OE12 reads it as intended
is still open: nobody has imported the file there yet. Geo positions and taped legs are not written yet, and team,
relay and score courses are skipped. Chapter 10 of the
[user manual](docs/manual/en.md#10-handing-on-iof-xml-for-oe) has the details.

`freemapper-courseplan init <map> --ppen <file>` and `freemapper-courseplan import <project> --ppen <file>`
take over controls, descriptions, courses, bends, marked routes, number positions and special objects from a Purple Pen event
drawn for the same map; the AI assistant has the tool `import_ppen`. What the model cannot hold, such as variations,
score courses or gaps, the program names in a note. `import` replaces a project that already has content only with
`--replace`; classes and accepted findings stay. The way back (export and sync) is still missing. Chapter 9 of the
[user manual](docs/manual/en.md#9-exchanging-with-purple-pen) explains it.

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
