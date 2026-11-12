# Sprint training, Rummelsburger Bucht

[Deutsch](README.md) · [All examples](../README.en.md)

A sprint course with five controls on the map "Rummelsburger Bucht" by Remo Madella (MOO Challenge Berlin 2025,
1:4000, 308 by 257 map millimetres, about 1.2 by 1.0 km). It was set with the command line only. The map sits
here as `rummelsburg.omap`. Remo Madella released it on 2026-11-12 for use within the freemapper suite; that is not
an open licence. The details are in [`ATTRIBUTION.txt`](ATTRIBUTION.txt) and below.

There is no preview image yet. freemapper-print will supply the course print once it can print courses from a
`.ppen`; you can see the map itself in the editor (`freemapper-courseplan edit rummelsburg-sprint.courseplan.toml`).

## Files

- `rummelsburg.omap`: the map, with `ATTRIBUTION.txt` naming the author and the permission
- `rummelsburg-sprint.courseplan.toml`: the project file
- `transcript.de.txt`, `transcript.en.txt`: the commands and their output
- `check.de.txt`, `check.en.txt`: the check of the course, with no findings
- `course-data.xml`: the course as IOF XML 3.0, made with `export`
- `rummelsburg-sprint.ppen`: the course as a Purple Pen event, made with `export --ppen`

## How the course was made

```text
$ freemapper-courseplan init rummelsburg.omap --project rummelsburg-sprint.courseplan.toml --title "Sprint training Rummelsburger Bucht"
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind start --id S1 --at -85,-68
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind finish --id F1 --at 95,0
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind first-aid --id H1 --at -93,-70
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at -70.05,-40.78
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at -13.67,7.12
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at 23.39,76.2
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at 60.73,73.78
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at 67.69,27.2
$ freemapper-courseplan class add rummelsburg-sprint.courseplan.toml Open
$ freemapper-courseplan course add rummelsburg-sprint.courseplan.toml A --start S1 --finish F1 --controls 31,32,33,34,35 --classes Open
$ freemapper-courseplan course numbers rummelsburg-sprint.courseplan.toml A
$ freemapper-courseplan info rummelsburg-sprint.courseplan.toml
course A: 5 controls, 1.2 km, climb – m
```

The coordinates are map millimetres, y pointing down. `map extent` gives the extent of the map, -154.9 to 153.0 mm
in x and -126.6 to 130.2 mm in y. Every output is in [`transcript.en.txt`](transcript.en.txt). The project sets no
climb because I measured none; `info` then shows a dash.

## Straight legs

Every leg is the straight line from control to control, without bends. Bends are the exception: you set them only
when you want to steer a leg on purpose (see the manual, section "Legs and bends"), and a taped leg to the finish may
take a detour. This course needs neither.

Large parts of the map are buildings, water and areas that must not be entered (symbol 520), with narrow lanes
between them. The straight line between two controls therefore mostly runs over such areas. That is intended: the
runner goes around, and this choice between lanes and squares is the route choice of the course. `check` reports
nothing about it, because the line between two controls is not the running route (ADR 0080).

The controls stand at prominent map objects outside closed areas, each at least 0.8 mm from buildings, water and
closed areas: 31 and 32 at crosses (symbol 531), 33 and 35 at monuments (526), 34 at a ring (530). None of them has
an identical object inside its circle. The legs are 124 m (start to 31), 296 m, 314 m, 150 m, 188 m and 154 m (35 to
the finish) long, 1.2 km of straight line in all. I looked for the sites with a small script that works on the
outlines of the map, and I did not look at each one on the map. Look at every control in the editor before you use
the course.

## Checking

`check` finds nothing on this course: `0 errors, 0 warnings, 0 hints; 0 accepted, 6 skipped`. All angles at the
controls lie between 102° and 159°. Of the six skipped rules, four are the length and difficulty rules of a
competition, which do not apply at a training. The fifth is the start capacity, because the event has no start
window, and the sixth is the rule that the best running route must not lead through an out-of-bounds area: it needs
a route analysis, which the program does not have yet. All commands and the output of `check --all` are in
[`check.de.txt`](check.de.txt) and [`check.en.txt`](check.en.txt).

## Special objects

Besides start and finish the project has a first aid post `H1` near the start. The course has no marked leg.

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
whether controls, course and numbers sit in the right place.

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
