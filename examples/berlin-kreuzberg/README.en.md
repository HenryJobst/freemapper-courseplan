# Score course, Berlin Kreuzberg

[Deutsch](README.md) · [All examples](../README.en.md)

A score course with a time limit of 45 minutes, the usual size for adults (30 minutes is the sensible minimum). The
Lübeck map is too small for that, so this example uses our own map: Berlin-Kreuzberg and Schöneberg, 1.8 by 2.0 km,
1:4000, made with freemapper 0.12.0 from open data on 2026-11-11. The course has 27 controls worth 80 points. It was set
with the command line only, and the controls come from `score draft`. There is no preview image: `make-preview.py` of
the Lübeck example needs freemapper-print and was written for that one course.

## Files

- `kreuzberg.omap`: the map (1.5 MB), `freemapper.toml` and `ATTRIBUTION.txt` from the freemapper project that made it
- `kreuzberg.courseplan.toml`: the project file
- `transcript.en.txt`, `transcript.de.txt`: the commands and their output
- `check.en.txt`, `check.de.txt`: checking the course, with one accepted warning
- `course-data.xml`: the course as IOF XML 3.0
- `score-45.csv`, `score-45.sheet.md`, `score-45.sheet.de.md`: control list and evaluation sheet

## How it was made

```sh
freemapper-courseplan init kreuzberg.omap --title "Berlin Kreuzberg Schöneberg"
freemapper-courseplan special add kreuzberg.courseplan.toml --kind start --id S-score --at 0,0
freemapper-courseplan special add kreuzberg.courseplan.toml --kind finish --id F-score --at 30,0
freemapper-courseplan score suggest kreuzberg.courseplan.toml --start S-score --finish F-score --limit 8
freemapper-courseplan score draft kreuzberg.courseplan.toml "Score 45" --time-limit 45 --start S-score --finish F-score --apply
freemapper-courseplan score length kreuzberg.courseplan.toml "Score 45"
freemapper-courseplan check kreuzberg.courseplan.toml
freemapper-courseplan finding accept kreuzberg.courseplan.toml wkb2026.score.code_ambiguity --controls 63,93 --note "..."
freemapper-courseplan export kreuzberg.courseplan.toml --iof-xml course-data.xml
freemapper-courseplan export kreuzberg.courseplan.toml --csv score-45.csv --course "Score 45"
freemapper-courseplan export kreuzberg.courseplan.toml --sheet score-45.sheet.md --course "Score 45"
```

`map extent` gave a span of -226,-253 to 225,247 mm, so `0,0` is the middle of the map. `map near` showed paths and open
land around both points, with the nearest building 4 to 12 m away. The draft reached the start, which it would refuse
with "cannot be reached" inside a building or blocked area. `map near` lists distances but does not say whether a point
lies inside an area, so look at both positions on the map. `transcript.en.txt` has every command with its output.

## Numbers

| | |
|---|---|
| Controls, points | 27, 80 |
| Time limit | 45 minutes |
| Good runner | 74 of 80 points, 92.5 percent (band 87 to 93) |
| Very good runner | all 80 points, back after 99.2 percent of the limit (band 95 to 100) |
| Run time | about 30 s for `score draft`, release build |

Both bands are met. A test (`berlin_score_example_meets_its_bands_and_check_is_quiet`) keeps it that way.

`check` finds no error and one warning: the codes 63 and 93 look alike when turned upside down. The draft chose both,
and the project file accepts the finding with a note that the two controls stand about 1.3 km apart. The 45-minute limit
is inside the usual 30 to 90 minutes, so the limit warning of the Lübeck example does not occur.

## Other starts and limits

Three starts were tried at 45 minutes: `0,0`, `-30,10` and `40,-40`. The first two give 27 and 30 controls inside both
bands, and a rerun with `score length` agrees. At `40,-40` (finish `60,-40`) the draft reports 33 controls, 97 points
and 99.8 percent for the very good runner. After `--apply`, `score length` and `check` find 96 points reachable and
100.7 percent of the limit for all of them, just outside the band. The two evaluations disagree there, so the example
does not use that start.

## Limits of this example

The routes come from a raster of the map symbols without elevation, and the runner paces are a first version. The sites
sit where the program found them. I have not looked at each one on the map; before you use them, look at every position.
Most of the map is streets, courtyards and buildings. The IOF XML carries the time limit, penalty and links in an
extension element that has not been tried in OE. `createTime` in `course-data.xml` is the time of the run that made it.
The file validates against the IOF schema.

## Provenance and licences

The map was made with freemapper 0.12.0 on 2026-11-11 for the polygon lon 13.3640 to 13.3860, lat 52.4890 to 52.5040
(kind sprint, scale 1:4000). `freemapper.toml` lists the orthophoto, terrain model and surface model tiles with their
currency. `ATTRIBUTION.txt` holds the credits:

| Data | Licence |
|---|---|
| OpenStreetMap, extract of 2026-11-06 | © OpenStreetMap contributors, ODbL |
| Orthophoto, terrain and surface model, GeoBasis-DE/LGB 2026 | dl-de/by-2-0 (data changed) |
| Geoportal Berlin: ALKIS, Karte 1:5000 (K5), Baumbestand, Straßenbefahrung 2014 (changed), Öffentliche Beleuchtung, Zierbrunnen, Trinkwasserbrunnen, Gebäudehöhen | dl-de/zero-2-0 |

The `templates/` and `data/` folders of the freemapper project are not part of this repository. The program and the
course data are GPL-3.0-or-later; the map content keeps the licences above.
