# freemapper-courseplan

[Deutsche Version](README.md) · [User manual](docs/manual/en.md) · [Glossary](docs/glossary.md) · [Changes](CHANGELOG.en.md)

freemapper-courseplan is the course-setting program of the freemapper suite. You set courses by hand in an
editor in the browser, on the finished map from OpenOrienteering Mapper (OOM). The program checks them against
the competition rules and hands them on to result software and to printing:

- [freemapper](https://github.com/HenryJobst/freemapper) produces the starter map,
- the mapper draws in OOM,
- freemapper-courseplan sets the courses,
- [freemapper-print](https://github.com/HenryJobst/freemapper-print) prints the map sheets.

Planned for the first version:

- controls with code and control description, individual courses with bends, marked routes and forbidden
  areas;
- control number positions set once per control and changeable per course, with automatic placement;
- checks against the German competition rules (WKB) 2026, such as control distances, acute angles and enough
  different first controls for a compact start list, each with its source;
- exchange with Purple Pen (`.ppen`) in both directions, so you can use both programs side by side;
- export to IOF XML 3.0 for result software, for example OE;
- sprint and schoolyard maps; forest maps come later;
- use through the editor in the browser, the command line or an AI assistant (MCP).

After that come a route analysis, team orienteering with mandatory and free controls, relays, score courses,
tours for the people who put out the controls, and AI support that judges courses and makes proposals.

The program needs no OCAD and runs on macOS, Linux and Windows.

## Status

freemapper-courseplan is in planning. There is no working program yet, only the project skeleton, the
[design](docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md), the [glossary](docs/glossary.md),
the [architecture decisions](docs/adr/README.md) and a
[collection of course-setting rules](docs/research/2026-11-08-bahnlegung-kriterien.md) (German).

## Building from source (for developers)

```sh
git clone https://github.com/HenryJobst/freemapper-courseplan
cd freemapper-courseplan
cargo build --release
```

`rust-toolchain.toml` fixes the Rust version; `rustup` installs it on the first build. Before each commit,
[lefthook](https://github.com/evilmartians/lefthook) checks formatting and clippy (`lefthook install`).

## Licence

freemapper-courseplan is licensed [GPL-3.0-or-later](LICENSE), like freemapper and freemapper-print (ADR 0002).
