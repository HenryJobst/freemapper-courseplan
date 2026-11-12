# Sprint training, Rummelsburger Bucht

[Deutsch](README.md) · [All examples](../README.en.md)

A sprint course with four controls on the map "Rummelsburger Bucht" by Remo Madella (MOO Challenge Berlin 2025,
1:4000, 308 by 257 map millimetres, about 1.2 by 1.0 km). It was set with the command line only. The map sits
here as `rummelsburg.omap`. Remo Madella released it on 2026-11-12 for use within the freemapper suite; that is not
an open licence. The details are in [`ATTRIBUTION.txt`](ATTRIBUTION.txt) and below.

There is no preview image yet. freemapper-print will supply the course print once it can print courses from a
`.ppen`; you can see the map itself in the editor (`freemapper-courseplan edit rummelsburg-sprint.courseplan.toml`).

## Files

- `rummelsburg.omap`: the map, with `ATTRIBUTION.txt` naming the author and the permission
- `rummelsburg-sprint.courseplan.toml`: the project file
- `transcript.de.txt`, `transcript.en.txt`: the commands and their output
- `check.de.txt`, `check.en.txt`: the check of the course, with three accepted hints
- `course-data.xml`: the course as IOF XML 3.0, made with `export`
- `rummelsburg-sprint.ppen`: the course as a Purple Pen event, made with `export --ppen`

## How the course was made

```text
$ freemapper-courseplan init rummelsburg.omap --project rummelsburg-sprint.courseplan.toml --title "Sprint training Rummelsburger Bucht"
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind start --id S1 --at -120,-75
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind finish --id F1 --at 48,69
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind first-aid --id H1 --at -112,-68
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at -70.05,-40.78
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at 9,-6.85
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at -13.67,7.12
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at 6.42,37.92
$ freemapper-courseplan class add rummelsburg-sprint.courseplan.toml Open
$ freemapper-courseplan course add rummelsburg-sprint.courseplan.toml A --start S1 --finish F1 --controls 31,32,33,34 --classes Open
$ freemapper-courseplan course leg rummelsburg-sprint.courseplan.toml A --after 1 --bend=-57,-43 --bend=-50,-50
$ freemapper-courseplan course leg rummelsburg-sprint.courseplan.toml A --after 3 --bend 6,27
$ freemapper-courseplan course leg rummelsburg-sprint.courseplan.toml A --after 4 --bend 6,41 --bend 13,34 --bend 20,50 --taped
$ freemapper-courseplan course numbers rummelsburg-sprint.courseplan.toml A
$ freemapper-courseplan info rummelsburg-sprint.courseplan.toml
course A: 4 controls, 1.1 km, climb – m
```

The coordinates are map millimetres, y pointing down. `map extent` gives the extent of the map, -154.9 to 153.0 mm
in x and -126.6 to 130.2 mm in y. Every output is in [`transcript.en.txt`](transcript.en.txt). The project sets no
climb because I measured none; `info` then shows a dash.

## The map sets the limits

Large parts of the map are areas that must not be entered (symbol 520), buildings and water. Narrow lanes run
between them. A straight leg from control to control crosses a closed area almost every time: my first draft with
eight controls gave seven warnings, one for nearly every leg. The legs of this course therefore run through the
lanes with bends. The controls stand at prominent map objects (symbols 531 and 530, a cross and a ring), each at least 0.6 mm from water
and closed areas. A small script picked the bends. It tests the outlines of buildings, water and closed areas; the
program did not, and I did not look at them on the map. Look at every leg in the editor before you use the course.
`check` reports no closed areas for legs with bends, because the route analysis is still missing; it does not test
them.

The course has four controls and 1.1 km. I worked through courses with five and six controls; none that I found
stayed free of warnings, mostly because two legs run against each other on the same path.

## Checking

`check` finds three hints, all about angles. The legs at control 32 form 67° (at least 90° required), at control 33
76°, and at control 34 the course runs almost straight on at 171° (at most 170° wanted). The lanes dictate the
direction. Because the course is a training, `finding accept` records the three as intended in the project file, with
the reason that the angles follow from the lanes. After that `check` reports nothing:
`0 errors, 0 warnings, 0 hints; 3 accepted, 5 skipped`. The five skipped rules are the length and difficulty rules of
a competition, which do not apply at a training, and the start capacity, because the event has no start window. All
commands and the output of `check --all` are in [`check.de.txt`](check.de.txt) and [`check.en.txt`](check.en.txt); the
English file shows English reasons, the project file holds the German ones.

## Special objects and marked leg

Besides start and finish the project has a first aid post `H1` near the start. The last leg (from control 34 to the
finish) is flagged as a marked route (`--taped`).

## Export to IOF XML

```text
$ freemapper-courseplan export rummelsburg-sprint.courseplan.toml --iof-xml course-data.xml
```

The file passes validation against the IOF schema (`xmllint --noout --schema ../../tests/fixtures/iof-xml/IOF.xsd
course-data.xml`). The map is georeferenced, so start, controls and finish carry latitude and longitude. `createTime`
is the time of the run that made the file; a new run changes only that line. An import into OE, Purple Pen or Condes
has not been tried, because nobody on the project has OE. If you do, report what arrives.

## Export to Purple Pen

```text
$ freemapper-courseplan export rummelsburg-sprint.courseplan.toml --ppen rummelsburg-sprint.ppen
```

The file does not exist yet, so `export` writes it new and records it in the project as `sources.ppen`. **This file
has not been opened in Purple Pen yet.** An export of an earlier example (the Lübeck old town) opened in Purple Pen
4.0 without errors; the writer is the same. If you have Purple Pen 4.0, open the file and report whether it loads and
whether controls, course, bends and numbers sit in the right place.

## A score course on this map

The map is small for a score course. With start `S1` and finish `F1` a draft succeeds at 15 minutes (9 controls,
27 points; the good runner takes 24 of them, 88.9 percent, the very good runner all of them and finishes after 99.7
percent of the limit) and at 20 minutes (13 controls, 28 points, 92.9 and 95.1 percent). At 25, 30 and 45 minutes the
sites run out: the very good runner is back after 80, 67 and 44.5 percent of the limit instead of 95 to 100. For
longer courses use [`../gleisdreieck/`](../gleisdreieck/), the larger map. These were runs of `score draft` without
`--apply`; the project here holds no score course.

## Source and licence

Map and author are named in [`ATTRIBUTION.txt`](ATTRIBUTION.txt). **Remo Madella released the map for use within the
freemapper suite, not under an open licence.** If you want to use it outside the suite, ask him. The program and the
course data (project file, exports, outputs) are under GPL-3.0-or-later; the map content keeps its own situation.
Whether the permission also covers distributing the `.omap` in the public repository is not settled (ADR 0077).
