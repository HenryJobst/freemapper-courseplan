# Changes

[Deutsche Version](CHANGELOG.md) · [User manual](docs/manual/en.md)

This file lists all notable changes to freemapper-courseplan, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Commands for courses: `course add`, `course controls`, `course leg`, `course set`, `course numbers` and
  `course remove`. Adding and changing work for individual courses only. `course leg --after INDEX` counts from 1 as on the map, crossing points
  included. Give bends with a repeated `--bend X,Y`; without `--bend` the leg loses its bends, and without
  `--taped` it loses the marked-route flag. `course numbers` places the control numbers clear of legs and circles. Output uses the decimal comma in German, while input keeps the dot.
- `info` prints one line per course with the number of controls, the length in kilometres and the climb. Crossing
  points do not count as controls. The length follows WKB B 7.2.5: straight lines, detours as bends, marked routes
  along their bends. Team, relay and score courses show no length yet.
- Commands for controls, special objects and classes: `control add|move|remove|describe|number`,
  `special add|remove` and `class add|remove`. Each change is checked first and saved after; if the check fails,
  the file stays as it was. Points are in map millimetres with a dot as decimal separator. The program refuses
  points outside the map.
- Library: course legs, course length and control numbers. The program computes legs with bends, the length of a
  course at a map scale and the numbers of the controls (crossing points get none). It places a number beside the
  circle without touching legs, circles or other numbers. Overprint sizes follow the map kind and stay the same size
  on the map for enlarged prints.
- Map index: the program looks up each map symbol once per symbol instead of once per object. This speeds up
  checking points against the map; results are unchanged.
- Data model: crossing points, ids for special objects, bends and marked-route flag for the leg from the start,
  and a print area per course.
- Program texts come from Fluent files, one per language (`crates/freemapper-courseplan/i18n/`). German and
  English ship now. `--lang`, then `FREEMAPPER_LANG`, then the system language picks the language; help and
  messages follow it. Adding a language takes one new folder. Placeholders in the usage line and rare
  command-line parse errors stay English.
- Project file `*.courseplan.toml` with individual, team, relay and score courses. The program checks it on load
  and rejects unknown keys, so a typo shows up. When it saves the file, your comments and layout stay as they
  were.
- `freemapper-courseplan init <map>` starts a project from a sprint or schoolyard map. `--title` and `--project`
  set the title and the project file. Forest maps are refused for now.
- `freemapper-courseplan info <project>` prints the title, map, map kind, controls, courses by kind and classes.
- Reading OOM maps (`.omap`). The program refuses a map in the old binary format and tells you to save it again
  in OOM.
- Project skeleton modelled on freemapper-print: Rust workspace with a data model, a library and the
  `freemapper-courseplan` program, CI for Linux, macOS and Windows, release workflow.
- Draft design, glossary, the first architecture decisions and a collection of course-setting rules from German
  and international rule books.
