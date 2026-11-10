# Sprinttraining Lübecker Altstadt

[English](README.en.md) · [Alle Beispiele](../README.md)

Eine Sprintbahn mit sieben Posten auf der Karte der Lübecker Altstadt aus den
[Beispielen von freemapper](https://github.com/HenryJobst/freemapper/tree/main/examples/schleswig-holstein-luebeck)
(1:4000). Gelegt wurde sie nur mit der Kommandozeile. Die Karte liegt im Repository unter
`tests/fixtures/maps/luebeck.omap`.

<img src="preview.jpg" width="400" alt="Skizze der Bahn auf der Karte der Lübecker Altstadt">

Das Bild ist eine Skizze. Die Karte hat freemapper-print v0.2.0 gerendert, Kreise, Linien und Nummern hat
`make-preview.py` aus der Projektdatei darübergezeichnet. Den echten Bahnaufdruck liefert freemapper-print, sobald
es Bahnen aus einer `.ppen` drucken kann; dann ersetzt das Druck-PDF diese Skizze.

## Dateien

- `luebeck-sprint.courseplan.toml`: die Projektdatei
- `transcript.de.txt`, `transcript.en.txt`: die Befehle und ihre Ausgaben
- `check.de.txt`, `check.en.txt`: die Prüfung der Bahn, mit einem akzeptierten Hinweis
- `course-data.xml`: die Bahn als IOF-XML 3.0, erzeugt mit `export` (siehe unten)
- `luebeck-sprint.ppen`: die Bahn als Purple-Pen-Event, erzeugt mit `export --ppen` (siehe unten)
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
Regelprofil: wkb-2026
Posten: 7, Querungsstellen: 0
Bahnen: 1 (einzeln 1, Mannschaft 0, Staffel 0, Score 0)
Kategorien: 1
Bahn A: 7 Posten, 1,3 km, Anstieg 5 m
```

Die Koordinaten sind Kartenmillimeter, y zeigt nach unten. Die Postennummern hat `course numbers` platziert.

## Prüfen

`check` findet auf der Bahn genau einen Hinweis. Am Posten 37 bilden die Strecken einen Winkel von 85°, der
Richtwert liegt bei 90°. Weil die Bahn ein Training ist, gilt der Winkel hier als gewollt, und `finding accept`
trägt ihn mit Begründung in die Projektdatei ein.

```text
$ freemapper-courseplan check luebeck-sprint.courseplan.toml
Bahn A:
  Hinweis Bahn A, Strecke 36–37: Die Strecken am Posten 37 bilden einen Winkel von 85°, gefordert sind mindestens 90° (wkb2026.leg.acute_angle_hint, BL12 5. Bahnkonzeption)
0 Fehler, 0 Warnungen, 1 Hinweis; 0 akzeptiert, 5 übersprungen

$ freemapper-courseplan finding accept luebeck-sprint.courseplan.toml wkb2026.leg.acute_angle_hint --leg A:6 --note "Training: Der Winkel von 85° am letzten Posten ist gewollt."
Befund wkb2026.leg.acute_angle_hint bei Bahn A, Strecke 36–37 akzeptiert
```

`--leg A:6` nennt die Strecke, die von der sechsten Position der Bahn abgeht, also von Posten 36 zu Posten 37. In
der Projektdatei steht die Ausnahme mit beiden Enden der Strecke. Danach meldet `check` nichts mehr. Die fünf
übersprungenen Prüfregeln sind Längen- und Schwierigkeitsregeln des Wettkampfs, die beim Training nicht gelten,
und die Startkapazität, weil die Veranstaltung kein Startfenster hat. Alle Befehle und die Ausgabe von
`check --all` stehen in [`check.de.txt`](check.de.txt) und [`check.en.txt`](check.en.txt). Die englische Datei
zeigt eine englische Begründung; in der Projektdatei steht die deutsche.

## Export nach IOF-XML

```text
$ freemapper-courseplan export luebeck-sprint.courseplan.toml --iof-xml course-data.xml
```

Die Datei besteht die Prüfung gegen das IOF-Schema (`xmllint --noout --schema ../../tests/fixtures/iof-xml/IOF.xsd
course-data.xml`). Ihr `createTime` ist fest auf `2026-11-09T12:00:00Z` gesetzt, damit der Test die Datei vergleichen kann; ein eigener Lauf schreibt die aktuelle Zeit in UTC. Außerdem liest ein Test sie mit dem IOF-Leser von freemapper-course wieder ein. Ein Import in OE, Purple Pen oder Condes ist nicht getestet, weil im Projekt niemand OE hat. Wer es tut, soll melden, was ankommt.

## Export nach Purple Pen

```text
$ freemapper-courseplan export luebeck-sprint.courseplan.toml --ppen luebeck-sprint.ppen
```

Die Datei gibt es noch nicht, deshalb schreibt `export` sie neu und trägt sie im Projekt als `sources.ppen` ein. Ein Test vergleicht die Datei mit einem frischen Export, ein zweiter liest sie wieder ein und exportiert sie ein zweites Mal auf sich selbst, mit denselben Bytes. **In Purple Pen 4.0 ist die Datei noch nicht geöffnet worden.** Wer Purple Pen 4.0 hat, soll sie öffnen und melden, ob sie sich lädt und ob Posten, Bahn und Nummern an der richtigen Stelle stehen. Die Texte der Beschreibung sind englisch, weil das Beispiel mit englischer Oberfläche erzeugt ist.

## Skizze neu erzeugen

```sh
freemapper-print render ../../tests/fixtures/maps/luebeck.omap --out map.pdf --paper a4 --scale 4000
pdftoppm -r 110 -png -singlefile map.pdf map
python3 make-preview.py .
```

Das Skript schreibt `sketch-crop.png`; die Bildunterschrift setzt `magick` dazu.
