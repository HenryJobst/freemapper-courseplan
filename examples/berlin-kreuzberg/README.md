# Score-Bahn, Berlin Kreuzberg

[English](README.en.md) · [Alle Beispiele](../README.md)

Eine Score-Bahn mit 45 Minuten Zeitlimit, der üblichen Größe für Erwachsene (30 Minuten sind das sinnvolle Minimum).
Die Lübecker Karte ist dafür zu klein, deshalb nutzt dieses Beispiel eine eigene Karte: Berlin-Kreuzberg und
Schöneberg, 1,8 mal 2,0 km, 1:4000, mit freemapper 0.12.0 am 11.11.2026 aus offenen Daten gemacht. Die Bahn hat 27
Posten mit 80 Punkten. Sie wurde nur mit der Kommandozeile gelegt, und die Posten stammen aus `score draft`. Ein
Vorschaubild gibt es nicht: `make-preview.py` des Lübecker Beispiels braucht freemapper-print und ist für diese eine
Bahn geschrieben.

## Dateien

- `kreuzberg.omap`: die Karte (1,5 MB), dazu `freemapper.toml` und `ATTRIBUTION.txt` aus dem freemapper-Projekt, das sie gemacht hat
- `kreuzberg.courseplan.toml`: die Projektdatei
- `transcript.de.txt`, `transcript.en.txt`: die Befehle und ihre Ausgabe
- `check.de.txt`, `check.en.txt`: die Prüfung der Bahn, mit einer akzeptierten Warnung
- `course-data.xml`: die Bahn als IOF-XML 3.0
- `score-45.csv`, `score-45.sheet.de.md`, `score-45.sheet.md`: Postenliste und Auswertungsbogen

## So ist es entstanden

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
freemapper-courseplan export kreuzberg.courseplan.toml --sheet score-45.sheet.de.md --course "Score 45"
```

`map extent` ergab eine Ausdehnung von -226,-253 bis 225,247 mm, `0,0` ist also die Kartenmitte. `map near` zeigte um beide
Punkte Wege und offenes Land, das nächste Gebäude 4 bis 12 m entfernt. Der Entwurf hat den Start erreicht; in einem
Gebäude oder gesperrten Gebiet bräche er mit „nicht erreichbar“ ab. `map near` nennt Abstände, sagt aber nicht, ob ein
Punkt in einer Fläche liegt; sieh dir beide Positionen auf der Karte an. `transcript.de.txt` hat jeden Befehl mit seiner Ausgabe.

## Zahlen

| | |
|---|---|
| Posten, Punkte | 27, 80 |
| Zeitlimit | 45 Minuten |
| Guter Läufer | 74 von 80 Punkten, 92,5 Prozent (Band 87 bis 93) |
| Sehr guter Läufer | alle 80 Punkte, zurück nach 99,2 Prozent des Limits (Band 95 bis 100) |
| Laufzeit | etwa 30 s für `score draft`, Release-Build |

Beide Bänder sind getroffen. Ein Test (`berlin_score_example_meets_its_bands_and_check_is_quiet`) hält es so.

`check` meldet keinen Fehler und eine Warnung: Die Codes 63 und 93 sehen auf dem Kopf gleich aus. Der Entwurf hat beide
gewählt, und die Projektdatei akzeptiert den Befund mit dem Hinweis, dass die beiden Posten rund 1,3 km auseinander
stehen. Das Limit von 45 Minuten liegt im üblichen Bereich von 30 bis 90 Minuten, die Limit-Warnung des Lübecker
Beispiels entfällt also.

## Andere Starts und Limits

Bei 45 Minuten habe ich drei Starts versucht: `0,0`, `-30,10` und `40,-40`. Die ersten beiden ergeben 27 und 30 Posten
in beiden Bändern, und ein neuer Lauf mit `score length` stimmt zu. Bei `40,-40` (Ziel `60,-40`) meldet der Entwurf 33
Posten, 97 Punkte und 99,8 Prozent für den sehr guten Läufer. Nach `--apply` finden `score length` und `check` nur 96
Punkte erreichbar und 100,7 Prozent des Limits für alle, knapp außerhalb des Bands. Die beiden Bewertungen weichen dort
voneinander ab, deshalb nimmt das Beispiel diesen Start nicht.

## Grenzen dieses Beispiels

Die Routen kommen aus einem Raster der Kartensymbole ohne Höhe, und die Läuferpaces sind eine erste Fassung. Die Stellen
liegen dort, wo das Programm sie gefunden hat. Ich habe nicht jede auf der Karte angesehen; sieh dir vor dem Einsatz jede
Position an. Die Karte besteht überwiegend aus Straßen, Höfen und Gebäuden. Das IOF-XML trägt Zeitlimit, Abzug und
Verbindungen in einem Erweiterungselement, das in OE nicht ausprobiert ist. `createTime` in `course-data.xml` ist die
Zeit des Laufs, der die Datei erzeugt hat. Die Datei besteht die Prüfung gegen das IOF-Schema.

## Herkunft und Lizenzen

Die Karte entstand mit freemapper 0.12.0 am 11.11.2026 für das Polygon Länge 13,3640 bis 13,3860, Breite 52,4890 bis
52,5040 (Art Sprint, Maßstab 1:4000). `freemapper.toml` listet die Kacheln von Orthofoto, Geländemodell und
Oberflächenmodell mit ihrer Aktualität. In `ATTRIBUTION.txt` stehen die Quellenangaben:

| Daten | Lizenz |
|---|---|
| OpenStreetMap, Auszug vom 06.11.2026 | © OpenStreetMap contributors, ODbL |
| Orthofoto, Gelände- und Oberflächenmodell, GeoBasis-DE/LGB 2026 | dl-de/by-2-0 (Daten geändert) |
| Geoportal Berlin: ALKIS, Karte 1:5000 (K5), Baumbestand, Straßenbefahrung 2014 (geändert), Öffentliche Beleuchtung, Zierbrunnen, Trinkwasserbrunnen, Gebäudehöhen | dl-de/zero-2-0 |

Die Ordner `templates/` und `data/` des freemapper-Projekts gehören nicht zu diesem Repository. Das Programm und die
Bahndaten stehen unter GPL-3.0-or-later; der Karteninhalt behält die Lizenzen oben.
