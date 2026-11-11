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
- `luebeck-score.courseplan.toml`: eine Kopie des Projekts mit einer Score-Bahn (siehe unten)
- `score-transcript.de.txt`, `score-transcript.en.txt`: die Befehle, die sie ergänzten, mit ihrer Ausgabe
- `score-check.de.txt`, `score-check.en.txt`: die Prüfung der Score-Bahn, mit vier akzeptierten Warnungen
- `score-course-data.xml`, `score-15.csv`, `score-15.sheet.md`, `score-15.sheet.de.md`: die Score-Bahn als IOF-XML, Postenliste und Auswertungsbogen

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
course-data.xml`). Ihr `createTime` ist fest auf `2026-11-09T12:00:00Z` gesetzt, damit der Test die Datei vergleichen kann; ein eigener Lauf schreibt die aktuelle Zeit in UTC. Außerdem liest ein Test sie mit dem IOF-Leser von freemapper-course wieder ein. Die Lübecker Karte ist georeferenziert, deshalb tragen Start, Posten und Ziel Breite und Länge. Ein Import in OE, Purple Pen oder Condes ist nicht getestet, weil im Projekt niemand OE hat. Wer es tut, soll melden, was ankommt.

## Export nach Purple Pen

```text
$ freemapper-courseplan export luebeck-sprint.courseplan.toml --ppen luebeck-sprint.ppen
```

Die Datei gibt es noch nicht, deshalb schreibt `export` sie neu und trägt sie im Projekt als `sources.ppen` ein. Ein Test vergleicht die Datei mit einem frischen Export, ein zweiter liest sie wieder ein und exportiert sie ein zweites Mal auf sich selbst, mit denselben Bytes. **In Purple Pen 4.0 ist die Datei noch nicht geöffnet worden.** Wer Purple Pen 4.0 hat, soll sie öffnen und melden, ob sie sich lädt und ob Posten, Bahn und Nummern an der richtigen Stelle stehen. Die Texte der Beschreibung sind englisch, weil das Beispiel mit englischer Oberfläche erzeugt ist.

## Score-Bahn

`luebeck-score.courseplan.toml` ist eine Kopie des Projekts oben. Sie ergänzt einen Start `S-score`, ein Ziel
`F-score` und eine Score-Bahn „Score 15“ mit 9 Posten, 25 Punkten und einem Zeitlimit von 15 Minuten. Das
Sprint-Projekt, sein Purple-Pen-Event und sein IOF-XML bleiben, wie sie waren. Die Kopie gibt es, weil der Start `S1`
der Sprintbahn in einem Gebäudeumriss liegt. Dort brechen `score suggest` und `score draft` mit der Meldung ab, dass der
Start nicht erreichbar ist, und eine Score-Bahn braucht Start und Ziel außerhalb von Gebäuden. `S1` habe ich nicht
verschoben; die beiden neuen Objekte stehen auf freier Karte bei `-60,-3.6` und `-30,10.2`, gefunden mit `map near`.

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

`score-transcript.de.txt` hat jeden Befehl mit seiner Ausgabe, auch `score draft`.

**Der Entwurf scheitert bei 45 Minuten, und die Karte ist nicht der Grund.** Ich habe zuerst 45 versucht, das übliche
Limit. Der gute Läufer holt alle Posten, also 100 Prozent der Punkte bei einem Band von 87 bis 93, und der sehr gute
Läufer ist nach 55 Prozent des Limits zurück statt nach 95 bis 100. Der Entwurf hört bei 30 Posten auf, weil die freien
Codes ausgehen. Ein zweistelliger Code trägt seinen Wert in der zweiten Ziffer, ein höchster Wert von 5 lässt also 35
Codes, und die Paare, die auf dem Kopf gleich aussehen (6 und 9), nehmen einen Teil davon weg. Die letzte Zeile der
Meldung sagt das. Mit einem auf 9 erhöhten höchsten Wert kommt der Entwurf auf 54 Posten und scheitert trotzdem: Der
sehr gute Läufer ist nach 63 Prozent des Limits zurück. Auch andere Limits verfehlen die Bänder, meist knapp (probiert
bei 14, 16 bis 26, 30 und 45; der gute Läufer liegt 0,5 bis 7 Prozentpunkte über dem Band, oder der sehr gute Läufer lässt einen Posten aus). Bei 15 Minuten gelingt der Entwurf in
wenigen Sekunden: Der gute Läufer holt 23 von 25 Punkten (92,0 Prozent), der sehr gute Läufer läuft alle Posten an und
ist nach 97,6 Prozent des Limits im Ziel. Das Beispiel nimmt 15 Minuten, und ein Test
(`luebeck_score_example_meets_its_bands_and_check_is_quiet`) hält es dort. Das ist eine kurze Bahn mit 9 Posten; eine
längere braucht mehr Codes, als der Standardbereich hergibt.

**Seit ADR 0072 verfehlt ein neuer Entwurf die Bänder.** Die Einstellungen halten Posten jetzt 100 m auseinander
(150 m von Start und Ziel, 200 m zwischen hochwertigen). Auf dieser Karte, etwa 1 km mal 0,8 km, bleiben 25 Stellen.
Ein neuer Lauf mit 15 Minuten gibt dem guten Läufer 94,7 Prozent der Punkte (Band 87 bis 93). Bei 30 und bei 45
Minuten reicht die Menge nicht: Der gute Läufer holt alle Punkte, und der sehr gute Läufer ist nach 69 und nach 46
Prozent des Limits im Ziel. Das nächste Limit, das beide Bänder trifft, sind 13 Minuten mit 4 Posten. Die Score-Bahn
in der Projektdatei ist der frühere Entwurf. Sie hält alle neuen Abstände ein, und `check` bleibt still.

`check` meldet für dieses Projekt keinen Fehler und eine Warnung, die die Projektdatei mit Begründung akzeptiert:

- Das Limit von 15 Minuten liegt unter den 30 bis 90 Minuten, die `wkb2026.score.limit_penalty` für üblich hält. Die
  Begründung sagt, dass der Entwurf die Bänder bei 15 Minuten trifft und bei 45 die freien Codes vorher ausgehen.

`score-check.de.txt` zeigt den Lauf. `info` nennt für die Score-Bahn Posten, Punkte und Limit; eine Länge in
Kilometern misst es nicht.

Grenzen dieses Beispiels. Die Routen hinter dem Entwurf kommen aus einem Raster der Kartensymbole ohne Höhe, und die
Läuferpaces (4:10 und 3:45 Minuten je Kilometer) sind eine erste Fassung. Die Stellen liegen dort, wo das Programm sie
gefunden hat, und ich habe nicht jede auf der Karte angesehen; sieh dir vor dem Einsatz jede Position an. Das IOF-XML
trägt Zeitlimit, Abzug und Verbindungen in einem Erweiterungselement, das in OE nicht ausprobiert ist. `createTime`
in `score-course-data.xml` ist die Zeit des Laufs, der die Datei erzeugt hat; ein neuer Lauf ändert nur diese Zeile.
Die Datei besteht die Prüfung gegen das IOF-Schema. Den Bogen gibt es auf Englisch und Deutsch (`--lang de`).

## Skizze neu erzeugen

```sh
freemapper-print render ../../tests/fixtures/maps/luebeck.omap --out map.pdf --paper a4 --scale 4000
pdftoppm -r 110 -png -singlefile map.pdf map
python3 make-preview.py .
```

Das Skript schreibt `sketch-crop.png`; die Bildunterschrift setzt `magick` dazu.
