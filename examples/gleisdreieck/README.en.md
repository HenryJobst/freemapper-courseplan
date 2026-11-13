# Score course, Gleisdreieck

[Deutsch](README.md) · [All examples](../README.en.md)

<p align="center"><a href="../../docs/manual/images/gleisdreieck-course.jpg"><img src="../../docs/manual/images/gleisdreieck-course.jpg" alt="The score course on the map Gleisdreieck in the editor; start and finish at the top right" width="520"></a><br><sub>The course in the editor; start and finish at the top right, 60 m apart. Map: Remo Madella, MOO Challenge Berlin 2025.</sub></p>

A score course with a 31-minute time limit on the map "Gleisdreieck" by Remo Madella (MOO Challenge Berlin 2025,
1:4000, 297 by 441 map millimetres, about 1.2 by 1.8 km). The course has 30 controls worth 80 points. It was set with
the command line only, and the controls come from `score draft`. The map sits here as `gleisdreieck.omap`. Remo
Madella released it on 2026-11-12 for use within the freemapper suite; that is not an open licence. The details are in
[`ATTRIBUTION.txt`](ATTRIBUTION.txt) and below. The picture is a capture of the editor; a printed PDF from freemapper-print
will follow. You can see the course on the map in the editor (`freemapper-courseplan edit gleisdreieck-score.courseplan.toml`).

**The program cannot reach 45 minutes, the usual size for adults, on this map.** See "Why 31 and not 45 minutes".

## Files

- `gleisdreieck.omap`: the map, with `ATTRIBUTION.txt` naming the author and the permission
- `gleisdreieck-score.courseplan.toml`: the project file
- `transcript.de.txt`, `transcript.en.txt`: the commands and their output
- `check.de.txt`, `check.en.txt`: the check of the course, with four accepted warnings
- `course-data.xml`: the course as IOF XML 3.0
- `score-31.csv`, `score-31.sheet.de.md`, `score-31.sheet.md`: control list and evaluation sheet

## How it was made

```sh
freemapper-courseplan init gleisdreieck.omap --project gleisdreieck-score.courseplan.toml --title "Score course Gleisdreieck"
freemapper-courseplan special add gleisdreieck-score.courseplan.toml --kind start --id S-score --at 100,-290
freemapper-courseplan special add gleisdreieck-score.courseplan.toml --kind finish --id F-score --at 100,-275
freemapper-courseplan score suggest gleisdreieck-score.courseplan.toml --start S-score --finish F-score --limit 8
freemapper-courseplan score draft gleisdreieck-score.courseplan.toml "Score 31" --time-limit 31 --start S-score --finish F-score --apply
freemapper-courseplan score length gleisdreieck-score.courseplan.toml "Score 31"
freemapper-courseplan check gleisdreieck-score.courseplan.toml
freemapper-courseplan finding accept gleisdreieck-score.courseplan.toml wkb2026.score.code_ambiguity --controls 64,94 --note "..."
freemapper-courseplan export gleisdreieck-score.courseplan.toml --iof-xml course-data.xml
freemapper-courseplan export gleisdreieck-score.courseplan.toml --csv score-31.csv --course "Score 31"
freemapper-courseplan export gleisdreieck-score.courseplan.toml --sheet score-31.sheet.md --course "Score 31"
```

`map extent` gave an extent of -138.9 to 157.8 mm in x and -330.4 to 110.9 mm in y. Start and finish lie in the north of
the map, 15 mm (60 m) apart, so well within the 400 m that generated courses keep (ADR 0086). Both are at least 50 m
from the edge of the mapped area, like every control (ADR 0087), and `check` tests that. The draft reached both; in a
building or a blocked area it would stop with "cannot be reached". `map near` gives distances but does not say whether a
point lies inside an area; look at both positions on the map. `transcript.en.txt` has every command with its output.

## Numbers

| | |
|---|---|
| Controls, points | 30, 80 |
| Time limit | 31 minutes |
| Good runner | 74 of 80 points, 92.5 percent (band 87 to 93): inside the band |
| Very good runner | all 80 points, back after 97.8 percent of the limit (band 95 to 100): inside the band |
| Run time, release build, busy machine | `score suggest` about 2 s, `score draft` about 9 s, `score length` about 1 s |

Both runners are inside their bands (ADR 0083), so `score draft` and `score length` exit with 0 and name no deviation.
A test (`gleisdreieck_score_example_reaches_its_targets_and_check_is_quiet`) keeps it that way. I measured the run
times on a machine that other processes loaded heavily (load average between 25 and 80); an idle one is likely faster.

`check` reports no error and four warnings: the codes 64 and 94, 63 and 93, 62 and 92, and 61 and 91 look alike when
turned upside down. With 30 controls most free codes are in use, so such pairs occur. The project file accepts the four
findings with a note on how far apart the two controls of a pair stand (about 470, 390, 690 and 250 m; the test
recomputes the distances). No control stands deeper than 60 m in a dead end, and none closer than 50 m to the edge of
the mapped area, so `check` has nothing to say about those rules. The limit of 31 minutes lies in the usual range of 30
to 90 minutes, so there is no limit warning.

## Why 31 and not 45 minutes

I tried 45, 40, 35 and 30 minutes and every limit between 31 and 34, with the default settings and seed 1 and the start
in the north, as in the example. The result of the good and the very good runner:

| Limit | Good runner | Very good runner | State |
|---|---|---|---|
| 45 min | 100 percent of the points | back after 71.8 percent of the limit | missed |
| 40 min | 100 percent | back after 80.8 percent | missed |
| 35 min | 100 percent | no deviation named | missed |
| 34 min | 100 percent | no deviation named | missed |
| 33 min | 98.6 percent | no deviation named | missed |
| 32 min | 98.7 percent | back after 90.7 percent | missed |
| 31 min | 92.5 percent | back after 97.8 percent | inside the band (both) |
| 30 min | 94.0 percent | back after 95.1 percent | near the target (good), inside the band (very good) |

So 31 minutes is the largest limit at which both runners are at least near the target, and both are inside their
bands. From 32 minutes the good runner takes almost or all of the points. Above that the free codes run out: the
closest attempt has 28 controls worth 70 points (29 controls worth 75 points at 32 minutes), and the draft says so. The
search is not exhaustive, and the result can jump between neighbouring limits.

The first version of the example had 34 minutes and 33 controls. Three new limits changed that
(ADR 0085 to 0087). The map has a spur along a quay to the left of the river bridge, a dead end of 208 m. Control 54 of
the old version stood at its tip, 208 m deep, where the runner has no route choice at all (ADR 0085 has the
measurements). Six more controls stood between 39 and 47 m from the edge of the mapped area, closer than the 50 m. The
draft now leaves such sites out, so the same map offers fewer sites, and the code space runs out at 28 to 30 controls.
The result is 31 minutes with 30 controls. I moved no control by hand.

## Limits of this example

The routes come from a raster of the map symbols without elevation, and the runner paces (4:10 and 3:45 minutes per
kilometre) are a first version. The sites lie where the program found them. I did not look at each one on the map; look
at every position before you use the course. Many lie at building corners. Control 63 stands at a shifted place
(coordinates with many decimals); look at it with particular care. The IOF XML carries the time limit, penalty and
links in an extension element that has not been tried in OE. The file passes validation against the IOF schema
(`xmllint --noout --schema ../../tests/fixtures/iof-xml/IOF.xsd course-data.xml`); `createTime` is the time of the
run that made the file.

## Source and licence

Map and author are named in [`ATTRIBUTION.txt`](ATTRIBUTION.txt). **Remo Madella released the map for use within the
freemapper suite, not under an open licence.** If you want to use it outside the suite, ask him. The program and the
course data (project file, exports, outputs) are under GPL-3.0-or-later; the map content keeps its own situation.
Whether the permission also covers distributing the `.omap` in the public repository is not settled (ADR 0077).
