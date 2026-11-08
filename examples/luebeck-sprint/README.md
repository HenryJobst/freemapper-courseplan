# Sprinttraining Lübecker Altstadt

[English](README.en.md) · [Alle Beispiele](../README.md)

Eine Sprintbahn mit sieben Posten auf der Karte der Lübecker Altstadt aus den
[Beispielen von freemapper](https://github.com/HenryJobst/freemapper/tree/main/examples/schleswig-holstein-luebeck)
(1:4000). Gelegt wurde sie nur mit der Kommandozeile. Die Karte liegt im Repository unter
`tests/fixtures/maps/luebeck.omap`.

<img src="preview.jpg" width="400" alt="Skizze der Bahn auf der Karte der Lübecker Altstadt">

Das Bild ist eine Skizze. Die Karte hat freemapper-print v0.1.0 gerendert, Kreise, Linien und Nummern hat
`make-preview.py` aus der Projektdatei darübergezeichnet. Den echten Bahnaufdruck liefert freemapper-print, sobald
es Bahnen aus einer `.ppen` drucken kann; dann ersetzt das Druck-PDF diese Skizze.

## Dateien

- `luebeck-sprint.courseplan.toml`: die Projektdatei
- `transcript.de.txt`, `transcript.en.txt`: die Befehle und ihre Ausgaben
- `preview.jpg`, `make-preview.py`: die Skizze und das Skript dazu

## So entstand die Bahn

```text
$ freemapper-courseplan init ../../tests/fixtures/maps/luebeck.omap --project luebeck-sprint.courseplan.toml --title "Sprinttraining Lübecker Altstadt"
luebeck-sprint.courseplan.toml

$ freemapper-courseplan special add luebeck-sprint.courseplan.toml --kind start --id S1 --at -97.4,-3.6
Start S1 bei -97.4,-3.6 angelegt

$ freemapper-courseplan special add luebeck-sprint.courseplan.toml --kind finish --id F1 --at -95.8,56.4
Ziel F1 bei -95.8,56.4 angelegt

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -50.1,-3.6
Posten 31 bei -50.1,-3.6 angelegt

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -47.8,-56.3
Posten 32 bei -47.8,-56.3 angelegt

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -17.8,-57.9
Posten 33 bei -17.8,-57.9 angelegt

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -1.2,-12.4
Posten 34 bei -1.2,-12.4 angelegt

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -28.2,10.2
Posten 35 bei -28.2,10.2 angelegt

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -52.4,47.6
Posten 36 bei -52.4,47.6 angelegt

$ freemapper-courseplan control add luebeck-sprint.courseplan.toml --at -88.2,32.9
Posten 37 bei -88.2,32.9 angelegt

$ freemapper-courseplan class add luebeck-sprint.courseplan.toml Offen
Kategorie Offen angelegt

$ freemapper-courseplan course add luebeck-sprint.courseplan.toml A --start S1 --finish F1 --controls 31,32,33,34,35,36,37 --classes Offen --climb 5
Bahn A angelegt

$ freemapper-courseplan course numbers luebeck-sprint.courseplan.toml A
7 Postennummern auf Bahn A gesetzt

$ freemapper-courseplan info luebeck-sprint.courseplan.toml
Titel: Sprinttraining Lübecker Altstadt
Karte: ../../tests/fixtures/maps/luebeck.omap
Kartenart: Sprintkarte
Posten: 7, Querungsstellen: 0
Bahnen: 1 (einzeln 1, Mannschaft 0, Staffel 0, Score 0)
Kategorien: 1
Bahn A: 7 Posten, 1,3 km, Anstieg 5 m
```

Die Koordinaten sind Kartenmillimeter, y zeigt nach unten. Die Postennummern hat `course numbers` platziert.

## Skizze neu erzeugen

```sh
freemapper-print render ../../tests/fixtures/maps/luebeck.omap --out map.pdf --paper a4 --scale 4000
pdftoppm -r 110 -png -singlefile map.pdf map
python3 make-preview.py .
```

Das Skript schreibt `sketch-crop.png`; die Bildunterschrift setzt `magick` dazu.
