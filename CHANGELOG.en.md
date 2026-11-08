# Changes

[Deutsche Version](CHANGELOG.md) · [User manual](docs/manual/en.md)

This file lists all notable changes to freemapper-courseplan, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Project file `*.courseplan.toml` with individual, team, relay and score courses. The program validates it on
  load and keeps your comments and layout when it saves.
- `freemapper-courseplan init <map>` starts a project from a sprint or schoolyard map. `--title` and `--project`
  set the title and the project file. Forest maps are refused for now.
- `freemapper-courseplan info <project>` prints the title, map, map kind, controls, courses by kind and classes.
- Reading OOM maps (`.omap`), including the objects near a point. The program rejects the old binary format with
  a clear message.
- Project skeleton modelled on freemapper-print: Rust workspace with a data model, a library and the
  `freemapper-courseplan` program (only `--version` and `--help` so far), CI for Linux, macOS and Windows,
  release workflow.
- Draft design, glossary, the first architecture decisions and a collection of course-setting rules from German
  and international rule books.
