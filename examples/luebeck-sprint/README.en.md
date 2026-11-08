# Sprint training, Lübeck old town

[Deutsch](README.md) · [All examples](../README.en.md)

A sprint course with seven controls on the map of Lübeck's old town from
[freemapper's examples](https://github.com/HenryJobst/freemapper/tree/main/examples/schleswig-holstein-luebeck)
(1:4000). It was set with the command line only. The map is in the repository at
`tests/fixtures/maps/luebeck.omap`.

<img src="preview.jpg" width="400" alt="Sketch of the course on the map of Lübeck's old town">

The image is a sketch. freemapper-print v0.1.0 rendered the map, and `make-preview.py` drew circles, lines and
numbers from the project file on top. freemapper-print will print the real course overprint once it can read
courses from a `.ppen`; its PDF then replaces this sketch.

## Files

- `luebeck-sprint.courseplan.toml`: the project file
- `transcript.en.txt`, `transcript.de.txt`: the commands and their output
- `preview.jpg`, `make-preview.py`: the sketch and its script

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
controls: 7, crossing points: 0
courses: 1 (individual 1, team 0, relay 0, score 0)
classes: 1
course A: 7 controls, 1.3 km, climb 5 m
```

Coordinates are map millimetres, with y pointing down. `course numbers` placed the control numbers.

## Making the sketch again

```sh
freemapper-print render ../../tests/fixtures/maps/luebeck.omap --out map.pdf --paper a4 --scale 4000
pdftoppm -r 110 -png -singlefile map.pdf map
python3 make-preview.py .
```

The script writes `sketch-crop.png`; `magick` adds the caption.
