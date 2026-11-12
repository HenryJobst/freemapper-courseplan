# Score-Bahn, Gleisdreieck

[English](README.en.md) · [Alle Beispiele](../README.md)

Eine Score-Bahn mit 33 Minuten Zeitlimit auf der Karte „Gleisdreieck“ von Remo Madella (MOO Challenge Berlin 2025,
1:4000, 297 mal 441 Kartenmillimeter, also etwa 1,2 mal 1,8 km). Die Bahn hat 27 Posten mit 83 Punkten. Sie wurde nur
mit der Kommandozeile gelegt, und die Posten stammen aus `score draft`. Die Karte liegt hier als `gleisdreieck.omap`.
Remo Madella hat sie am 12.11.2026 für die freemapper-Suite freigegeben; das ist keine offene Lizenz, Einzelheiten
stehen in [`ATTRIBUTION.txt`](ATTRIBUTION.txt) und unten. Ein Vorschaubild gibt es nicht; die Bahn auf der Karte
siehst du im Editor (`freemapper-courseplan edit gleisdreieck-score.courseplan.toml`).

**45 Minuten, die übliche Größe für Erwachsene, schafft das Programm auf dieser Karte nicht.** Siehe „Warum 33 und nicht 45 Minuten“.

## Dateien

- `gleisdreieck.omap`: die Karte, dazu `ATTRIBUTION.txt` mit Urheber und Erlaubnis
- `gleisdreieck-score.courseplan.toml`: die Projektdatei
- `transcript.de.txt`, `transcript.en.txt`: die Befehle und ihre Ausgabe
- `check.de.txt`, `check.en.txt`: die Prüfung der Bahn, mit einer akzeptierten Warnung
- `course-data.xml`: die Bahn als IOF-XML 3.0
- `score-33.csv`, `score-33.sheet.de.md`, `score-33.sheet.md`: Postenliste und Auswertungsbogen

## So ist es entstanden

```sh
freemapper-courseplan init gleisdreieck.omap --project gleisdreieck-score.courseplan.toml --title "Score-Bahn Gleisdreieck"
freemapper-courseplan special add gleisdreieck-score.courseplan.toml --kind start --id S-score --at 100,-290
freemapper-courseplan special add gleisdreieck-score.courseplan.toml --kind finish --id F-score --at 100,-275
freemapper-courseplan score suggest gleisdreieck-score.courseplan.toml --start S-score --finish F-score --limit 8
freemapper-courseplan score draft gleisdreieck-score.courseplan.toml "Score 33" --time-limit 33 --start S-score --finish F-score --apply
freemapper-courseplan score length gleisdreieck-score.courseplan.toml "Score 33"
freemapper-courseplan check gleisdreieck-score.courseplan.toml
freemapper-courseplan finding accept gleisdreieck-score.courseplan.toml wkb2026.score.code_ambiguity --controls 65,95 --note "..."
freemapper-courseplan export gleisdreieck-score.courseplan.toml --iof-xml course-data.xml
freemapper-courseplan export gleisdreieck-score.courseplan.toml --csv score-33.csv --course "Score 33"
freemapper-courseplan export gleisdreieck-score.courseplan.toml --sheet score-33.sheet.de.md --course "Score 33"
```

`map extent` ergab eine Ausdehnung von -138,9 bis 157,8 mm in x und -330,4 bis 110,9 mm in y. Start und Ziel liegen im
Norden der Karte, 15 mm (60 m) auseinander. Der Entwurf hat beide erreicht; in einem Gebäude oder gesperrten Gebiet
bräche er mit „nicht erreichbar“ ab. `map near` nennt Abstände, sagt aber nicht, ob ein Punkt in einer Fläche liegt;
sieh dir beide Positionen auf der Karte an. `transcript.de.txt` hat jeden Befehl mit seiner Ausgabe.

## Zahlen

| | |
|---|---|
| Posten, Punkte | 27, 83 |
| Zeitlimit | 33 Minuten |
| Guter Läufer | 77 von 83 Punkten, 92,8 Prozent (Band 87 bis 93) |
| Sehr guter Läufer | alle 83 Punkte, zurück nach 96,1 Prozent des Limits (Band 95 bis 100) |
| Laufzeit, Release-Build, ruhiger Rechner | `score suggest` etwa 4,5 s, `score draft` etwa 23 s, `score length` etwa 2,7 s |

Beide Bänder sind getroffen. Ein Test (`gleisdreieck_score_example_meets_its_bands_and_check_is_quiet`) hält es so.

`check` meldet keinen Fehler und eine Warnung: Die Codes 65 und 95 sehen auf dem Kopf gleich aus. Der Entwurf hat beide
gewählt, und die Projektdatei akzeptiert den Befund mit dem Hinweis, dass die beiden Posten rund 260 m auseinander
stehen. Das Limit von 33 Minuten liegt im üblichen Bereich von 30 bis 90 Minuten, eine Limit-Warnung gibt es nicht.

## Warum 33 und nicht 45 Minuten

Ich habe Limits von 20 bis 45 Minuten, mehrere Starts und mehrere Startwerte (`score settings --seed`) durchprobiert.
Mit dem Start oben gelingen 33 Minuten. Bei 34 und 35 Minuten verfehlt der gute Läufer das Band um 0,3 und 0,8
Prozentpunkte, bei 36 und 38 um 2,8 und 3,8, bei 40 um 5,1 (der sehr gute Läufer ist dann nach 92,6 Prozent des Limits im
Ziel, 2,4 Punkte vor dem Band). Bei 45 Minuten holt der gute Läufer alle Punkte, und der sehr gute Läufer ist nach
80,8 Prozent des Limits im Ziel statt nach 95 bis 100. Beim höchsten Wert 5 gehen dabei die freien Codes aus, beim
höchsten Wert 9 fehlen brauchbare Stellen (83,3 Prozent). 33 Minuten sind also das größte Limit, das ich auf dieser
Karte erreicht habe; ein anderer Start oder Startwert kann ein oder zwei Minuten mehr bringen, nicht zwölf.

Mit Start und Ziel in der Kartenmitte (`60,-60` und `90,-60`) gelingen Entwürfe in beiden Bändern bei 28, 30 und 31
Minuten; bei 32 verfehlt der gute Läufer das Band um 4,6 Prozentpunkte, bei 45 ist der sehr gute Läufer nach 64,9
Prozent des Limits zurück.

Eine frühere Fassung dieses Beispiels hatte 40 Minuten, und das Programm hat sie geliefert. Vier der Posten
standen aber auf den Ecken eines Rechtecks, das der Kartenautor um den Karteninhalt gezeichnet hat; dort ist
nichts. Das war ein Fehler des Programms, der mit dieser Karte auffiel: Eine solche Umrandung liefert seitdem
keine Postenstellen mehr (ADR 0078). Seit der Korrektur gelingen 40 Minuten nicht mehr.

## Grenzen dieses Beispiels

Die Routen kommen aus einem Raster der Kartensymbole ohne Höhe, und die Läuferpaces (4:10 und 3:45 Minuten je
Kilometer) sind eine erste Fassung. Die Stellen liegen dort, wo das Programm sie gefunden hat. Ich habe nicht jede auf
der Karte angesehen; sieh dir vor dem Einsatz jede Position an. Viele liegen an Gebäudeecken. Posten 32 steht auf einer
versetzten Stelle (Koordinaten mit vielen Nachkommastellen); sieh ihn dir besonders an. Das IOF-XML trägt Zeitlimit, Abzug und Verbindungen in einem
Erweiterungselement, das in OE nicht ausprobiert ist. Die Datei besteht die Prüfung gegen das IOF-Schema (`xmllint
--noout --schema ../../tests/fixtures/iof-xml/IOF.xsd course-data.xml`); `createTime` ist die Zeit des Laufs, der die
Datei erzeugt hat.

## Herkunft und Lizenz

Karte und Urheber stehen in [`ATTRIBUTION.txt`](ATTRIBUTION.txt). **Remo Madella hat die Karte für die
freemapper-Suite freigegeben, nicht unter einer offenen Lizenz.** Wer sie außerhalb der Suite nutzen will, fragt ihn.
Das Programm und die Bahndaten (Projektdatei, Exporte, Ausgaben) stehen unter GPL-3.0-or-later; der Karteninhalt
behält seine eigene Lage. Ob die Erlaubnis auch das Verteilen der `.omap` im öffentlichen Repository meint, ist nicht
geklärt (ADR 0077).
