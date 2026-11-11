# Sprint training, Lübeck old town

[Deutsch](README.md) · [All examples](../README.en.md)

A sprint course with seven controls on the map of Lübeck's old town from
[freemapper's examples](https://github.com/HenryJobst/freemapper/tree/main/examples/schleswig-holstein-luebeck)
(1:4000). It was set with the command line only. The map is in the repository at
`tests/fixtures/maps/luebeck.omap`.

<img src="preview.jpg" width="400" alt="Sketch of the course on the map of Lübeck's old town">

The image is a sketch. freemapper-print v0.2.0 rendered the map, and `make-preview.py` drew circles, lines and
numbers from the project file on top. freemapper-print will print the real course overprint once it can read
courses from a `.ppen`; its PDF then replaces this sketch.

## Files

- `luebeck-sprint.courseplan.toml`: the project file
- `transcript.en.txt`, `transcript.de.txt`: the commands and their output
- `check.en.txt`, `check.de.txt`: checking the course, with one accepted hint
- `course-data.xml`: the course as IOF XML 3.0, made by `export` (see below)
- `luebeck-sprint.ppen`: the course as a Purple Pen event, made by `export --ppen` (see below)
- `preview.jpg`, `make-preview.py`: the sketch and its script
- `luebeck-score.courseplan.toml`: a copy of the project with a score course (see below)
- `score-transcript.en.txt`, `score-transcript.de.txt`: the commands that added it, with their output
- `score-check.en.txt`, `score-check.de.txt`: checking the score course, with four accepted warnings
- `score-course-data.xml`, `score-15.csv`, `score-15.sheet.md`, `score-15.sheet.de.md`: the score course as IOF XML, control list and evaluation sheet

## How the course was set

```text
$ freemapper-courseplan init ../../tests/fixtures/maps/luebeck.omap --project luebeck-sprint.courseplan.toml --title "Sprint training Lübeck old town"
luebeck-sprint.courseplan.toml

$ freemapper-courseplan special add luebeck-sprint.courseplan.toml --kind start --id S1 --at -97.4,-3.6
added start S1 at -97.4,-3.6

$ freemapper-courseplan special add luebeck-sprint.courseplan.toml --kind finish --id F1 --at -95.8,56.4
added finish F1 at -95.8,56.4

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -50.1,-3.6
added control 31 at -50.1,-3.6

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -47.8,-56.3
added control 32 at -47.8,-56.3

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -17.8,-57.9
added control 33 at -17.8,-57.9

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -1.2,-12.4
added control 34 at -1.2,-12.4

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -28.2,10.2
added control 35 at -28.2,10.2

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -52.4,47.6
added control 36 at -52.4,47.6

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -88.2,32.9
added control 37 at -88.2,32.9

$ freemapper-courseplan class add luebeck-sprint.courseplan.toml Open
added class Open

$ freemapper-courseplan course add luebeck-sprint.courseplan.toml A --start S1 --finish F1 --controls 31,32,33,34,35,36,37 --classes Open --climb 5
added course A

$ freemapper-courseplan course numbers luebeck-sprint.courseplan.toml A
placed 7 control numbers on course A

$ freemapper-courseplan info luebeck-sprint.courseplan.toml
title: Sprint training Lübeck old town
map: ../../tests/fixtures/maps/luebeck.omap
map kind: sprint
rule profile: wkb-2026
controls: 7, crossing points: 0
courses: 1 (individual 1, team 0, relay 0, score 0)
classes: 1
course A: 7 controls, 1.3 km, climb 5 m
```

Coordinates are map millimetres, with y pointing down. `course numbers` placed the control numbers.

## Checking

`check` finds exactly one hint on the course. At control 37 the legs form an angle of 85°, and the guideline is 90°.
The course is a training, so the angle counts as intended here, and `finding accept` writes it into the project file
with a note.

```text
$ freemapper-courseplan check luebeck-sprint.courseplan.toml
course A:
  hint course A, leg 36–37: the legs at control 37 form an angle of 85°, at least 90° required (wkb2026.leg.acute_angle_hint, BL12 5. Bahnkonzeption)
0 errors, 0 warnings, 1 hint; 0 accepted, 5 skipped

$ freemapper-courseplan finding accept luebeck-sprint.courseplan.toml wkb2026.leg.acute_angle_hint --leg A:6 --note "Training: the 85° angle at the last control is intended."
accepted finding wkb2026.leg.acute_angle_hint at course A, leg 36–37
```

`--leg A:6` names the leg that leaves the sixth position of the course, from control 36 to control 37. The
project file stores the exception with both ends of the leg. After that, `check` reports nothing. The five skipped
rules are the length and difficulty rules of competitions, which do not apply to a training, and the start
capacity, because the event has no start window. All commands and the output of `check --all` are in
[`check.en.txt`](check.en.txt) and [`check.de.txt`](check.de.txt). The English file shows an English note; the
project file holds the German one.

## IOF XML export

```text
$ freemapper-courseplan export luebeck-sprint.courseplan.toml --iof-xml course-data.xml
```

The file validates against the IOF schema (`xmllint --noout --schema ../../tests/fixtures/iof-xml/IOF.xsd
course-data.xml`). Its `createTime` is fixed at `2026-11-09T12:00:00Z` so a test can compare the file; a run of your own writes the current time in UTC. A test also reads it back with the IOF reader of freemapper-course. The Lübeck map is georeferenced, so start, controls and finish carry latitude and longitude. An import into OE, Purple Pen or Condes has not been tested, because nobody on the project has OE. If you do it, please report what arrives.

## Purple Pen export

```text
$ freemapper-courseplan export luebeck-sprint.courseplan.toml --ppen luebeck-sprint.ppen
```

The file does not exist yet, so `export` writes it from scratch and records it in the project as `sources.ppen`. A test compares the file with a fresh export. Another reads it back and exports onto it a second time, with the same bytes. **The file has not been opened in Purple Pen 4.0.** If you have Purple Pen 4.0, please open it and report whether it loads and whether controls, course and numbers sit in the right place. The description texts are English because the example was made with the English interface.

## Score course

`luebeck-score.courseplan.toml` is a copy of the project above. It adds a start `S-score` and a finish `F-score` and
one score course, "Score 15", with 9 controls worth 25 points and a time limit of 15 minutes. The sprint project,
its Purple Pen event and its IOF XML stay as they were. The copy exists because the start `S1` of the sprint course
lies inside a building outline. There `score suggest` and `score draft` stop with the message that the start cannot
be reached, and a score course needs a start and a finish outside buildings. I did not move `S1`; the two new
objects stand on open map at `-60,-3.6` and `-30,10.2`, found with `map near`.

```sh
cp luebeck-sprint.courseplan.toml luebeck-score.courseplan.toml
freemapper-courseplan special add luebeck-score.courseplan.toml --kind start --id S-score --at -60,-3.6
freemapper-courseplan special add luebeck-score.courseplan.toml --kind finish --id F-score --at -30,10.2
freemapper-courseplan score suggest luebeck-score.courseplan.toml --start S-score --finish F-score --limit 8
freemapper-courseplan score draft luebeck-score.courseplan.toml "Score 15" --time-limit 15 --start S-score --finish F-score --apply
freemapper-courseplan check luebeck-score.courseplan.toml
freemapper-courseplan export luebeck-score.courseplan.toml --iof-xml score-course-data.xml
freemapper-courseplan export luebeck-score.courseplan.toml --csv score-15.csv --course "Score 15"
freemapper-courseplan export luebeck-score.courseplan.toml --sheet score-15.sheet.md --course "Score 15"
```

`score-transcript.en.txt` has every command with its output, `score draft` included.

**The draft fails at 45 minutes, and the map is not the reason.** I tried 45 first, the usual limit. The good runner
takes every control, 100 percent of the points against a band of 87 to 93, and the very good runner is back after
55 percent of the limit instead of 95 to 100. The draft stops at 30 controls because the free codes run out. A
two-digit code carries its value in the second digit, so a highest value of 5 leaves 35 codes, and the pairs that read
alike turned upside down (6 and 9) take away some of them. The last line of the message says so. With the highest value
raised to 9 the draft reaches 54 controls and still fails: the very good runner is back after 63 percent of the limit.
Other limits miss the bands as well, mostly by a narrow margin (tried at 14, 16 to 26, 30 and 45; the good runner lands 0.5 to 7
percentage points above the band, or the very good runner misses a control). At 15 minutes the draft succeeds in a few seconds: the good runner scores 23 of 25 points
(92.0 percent), the very good runner takes every control and finishes after 97.6 percent of the limit. The example uses
15 minutes, and a test (`luebeck_score_example_meets_its_bands_and_check_is_quiet`) keeps it there. That is a short
course with 9 controls; a longer one needs more codes than the default range gives.

**Since ADR 0072 a rerun of the draft misses the bands at 15 minutes.** The settings now keep controls 100 m apart (150 m
from start and finish, 200 m between high-valued ones). With the wider candidate pool of ADR 0073 this map, about 1 km
by 0.8 km, gives 31 sites (19 before; most of it is buildings, and about half of the tries fail inside courtyards). A
rerun at 15 minutes gives the good runner 93.5 percent of the points, half a point above the band of 87 to 93. The
limits 11 and 14 minutes meet both bands (6 controls with 16 points, 9 with 25). At 30 and 45 minutes the pool is still
too small: the draft stops at 20 to 22 controls, the good runner takes every point and the very good runner is back
after 78 and 52 percent of the limit. The score course in the project file is the earlier 15-minute draft. It keeps all
the new gaps, and `check` stays quiet.

`check` on this project gives no error and one warning, which the project file accepts with a reason:

- The limit of 15 minutes is below the 30 to 90 minutes that `wkb2026.score.limit_penalty` calls usual. The note says
  that the draft meets the bands at 15 minutes and that the free codes run out first at 45.

`score-check.en.txt` shows the run. The numbers `info` prints for the score course are its controls, points and
limit; it does not measure a length in kilometres.

Limits of this example. The routes behind the draft come from a raster of the map symbols without elevation, and the
runner paces (4:10 and 3:45 minutes per kilometre) are a first version. The sites sit where the program found them, and
I have not looked at each one on the map; before you use them, look at every position. The IOF XML carries the time
limit, penalty and links in an extension element that has not been tried in OE. `createTime` in
`score-course-data.xml` is the time of the run that made it, so a rerun changes that line only. The file validates
against the IOF schema. The sheet comes in English and German (`--lang de`).

## Making the sketch again

```sh
freemapper-print render ../../tests/fixtures/maps/luebeck.omap --out map.pdf --paper a4 --scale 4000
pdftoppm -r 110 -png -singlefile map.pdf map
python3 make-preview.py .
```

The script writes `sketch-crop.png`; `magick` adds the caption.
