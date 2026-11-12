# Score course, Gleisdreieck

[Deutsch](README.md) · [All examples](../README.en.md)

A score course with a 33-minute time limit on the map "Gleisdreieck" by Remo Madella (MOO Challenge Berlin 2025,
1:4000, 297 by 441 map millimetres, about 1.2 by 1.8 km). The course has 27 controls worth 83 points. It was set with
the command line only, and the controls come from `score draft`. The map sits here as `gleisdreieck.omap`. Remo
Madella released it on 2026-11-12 for use within the freemapper suite; that is not an open licence. The details are in
[`ATTRIBUTION.txt`](ATTRIBUTION.txt) and below. There is no preview image; you can see the course on the map in the
editor (`freemapper-courseplan edit gleisdreieck-score.courseplan.toml`).

**The program cannot reach 45 minutes, the usual size for adults, on this map.** See "Why 33 and not 45 minutes".

## Files

- `gleisdreieck.omap`: the map, with `ATTRIBUTION.txt` naming the author and the permission
- `gleisdreieck-score.courseplan.toml`: the project file
- `transcript.de.txt`, `transcript.en.txt`: the commands and their output
- `check.de.txt`, `check.en.txt`: the check of the course, with one accepted warning
- `course-data.xml`: the course as IOF XML 3.0
- `score-33.csv`, `score-33.sheet.de.md`, `score-33.sheet.md`: control list and evaluation sheet

## How it was made

```sh
freemapper-courseplan init gleisdreieck.omap --project gleisdreieck-score.courseplan.toml --title "Score course Gleisdreieck"
freemapper-courseplan special add gleisdreieck-score.courseplan.toml --kind start --id S-score --at 100,-290
freemapper-courseplan special add gleisdreieck-score.courseplan.toml --kind finish --id F-score --at 100,-275
freemapper-courseplan score suggest gleisdreieck-score.courseplan.toml --start S-score --finish F-score --limit 8
freemapper-courseplan score draft gleisdreieck-score.courseplan.toml "Score 33" --time-limit 33 --start S-score --finish F-score --apply
freemapper-courseplan score length gleisdreieck-score.courseplan.toml "Score 33"
freemapper-courseplan check gleisdreieck-score.courseplan.toml
freemapper-courseplan finding accept gleisdreieck-score.courseplan.toml wkb2026.score.code_ambiguity --controls 65,95 --note "..."
freemapper-courseplan export gleisdreieck-score.courseplan.toml --iof-xml course-data.xml
freemapper-courseplan export gleisdreieck-score.courseplan.toml --csv score-33.csv --course "Score 33"
freemapper-courseplan export gleisdreieck-score.courseplan.toml --sheet score-33.sheet.md --course "Score 33"
```

`map extent` gave an extent of -138.9 to 157.8 mm in x and -330.4 to 110.9 mm in y. Start and finish lie in the north of
the map, 15 mm (60 m) apart. The draft reached both; in a building or a blocked area it would stop with "cannot be
reached". `map near` gives distances but does not say whether a point lies inside an area; look at both positions on
the map. `transcript.en.txt` has every command with its output.

## Numbers

| | |
|---|---|
| Controls, points | 27, 83 |
| Time limit | 33 minutes |
| Good runner | 77 of 83 points, 92.8 percent (band 87 to 93) |
| Very good runner | all 83 points, back after 96.1 percent of the limit (band 95 to 100) |
| Run time, release build, idle machine | `score suggest` about 4.5 s, `score draft` about 23 s, `score length` about 2.7 s |

Both bands are met. A test (`gleisdreieck_score_example_meets_its_bands_and_check_is_quiet`) keeps it that way.

`check` reports no error and one warning: codes 65 and 95 look alike when turned upside down. The draft picked both,
and the project file accepts the finding with the note that the two controls stand about 260 m apart. The limit of 33
minutes lies in the usual range of 30 to 90 minutes, so there is no limit warning.

## Why 33 and not 45 minutes

I tried limits from 20 to 45 minutes, several starts and several seeds (`score settings --seed`). With the start in
the north, 33 minutes works. At 34 and 35 minutes the good runner misses the band by 0.3 and 0.8 percentage points, at
36 and 38 by 2.8 and 3.8, at 40 by 5.1 (the very good runner is then back after 92.6 percent of the limit, 2.4 points
short of the band). At 45 minutes the good runner takes every point, and the very good runner is back after 80.8
percent of the limit instead of 95 to 100. With the highest value at 5 the free codes run out first; with the highest
value at 9 there are too few usable sites (83.3 percent). So 33 minutes is the largest limit I reached on this map;
another start or seed may add a minute or two, not twelve.

With start and finish in the middle of the map (`60,-60` and `90,-60`) drafts inside both bands work at 28, 30 and 31
minutes; at 32 the good runner misses the band by 4.6 percentage points, and at 45 the very good runner is back after
64.9 percent of the limit.

An earlier version of this example had 40 minutes, and the program delivered it. Four of its controls stood on the
corners of a rectangle the map's author drew around the map content; there is nothing there. That was a bug in the
program that this map brought to light: such a frame no longer yields control sites (ADR 0078). Since the fix, 40
minutes no longer works.

## Limits of this example

The routes come from a raster of the map symbols without elevation, and the runner paces (4:10 and 3:45 minutes per
kilometre) are a first version. The sites lie where the program found them. I did not look at each one on the map; look
at every position before you use the course. Many lie at building corners. Control 32 stands at a shifted place
(coordinates with many decimals); look at it with particular care. The IOF XML carries the time limit, penalty and
links in an extension element that has not been tried in OE. The file passes validation against the IOF schema
(`xmllint --noout --schema ../../tests/fixtures/iof-xml/IOF.xsd course-data.xml`); `createTime` is the time of the
run that made the file.

## Source and licence

Map and author are named in [`ATTRIBUTION.txt`](ATTRIBUTION.txt). **Remo Madella released the map for use within the
freemapper suite, not under an open licence.** If you want to use it outside the suite, ask him. The program and the
course data (project file, exports, outputs) are under GPL-3.0-or-later; the map content keeps its own situation.
Whether the permission also covers distributing the `.omap` in the public repository is not settled (ADR 0077).
