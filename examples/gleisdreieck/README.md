# Score-Bahn, Gleisdreieck

[English](README.en.md) · [Alle Beispiele](../README.md)

<p align="center"><a href="../../docs/manual/images/gleisdreieck-course.jpg"><img src="../../docs/manual/images/gleisdreieck-course.jpg" alt="Die Score-Bahn auf der Karte Gleisdreieck im Editor; Start und Ziel liegen oben rechts" width="520"></a><br><sub>Die Bahn im Editor; Start und Ziel oben rechts, 60 m auseinander. Karte: Remo Madella, MOO Challenge Berlin 2025.</sub></p>

Eine Score-Bahn mit 34 Minuten Zeitlimit auf der Karte „Gleisdreieck“ von Remo Madella (MOO Challenge Berlin 2025,
1:4000, 297 mal 441 Kartenmillimeter, also etwa 1,2 mal 1,8 km). Die Bahn hat 33 Posten mit 95 Punkten. Sie wurde nur
mit der Kommandozeile gelegt, und die Posten stammen aus `score draft`. Die Karte liegt hier als `gleisdreieck.omap`.
Remo Madella hat sie am 12.11.2026 für die freemapper-Suite freigegeben; das ist keine offene Lizenz, Einzelheiten
stehen in [`ATTRIBUTION.txt`](ATTRIBUTION.txt) und unten. Das Bild ist eine Aufnahme des Editors; ein gedrucktes PDF von
freemapper-print folgt. Die Bahn auf der Karte siehst du im Editor (`freemapper-courseplan edit gleisdreieck-score.courseplan.toml`).

**45 Minuten, die übliche Größe für Erwachsene, schafft das Programm auf dieser Karte nicht.** Siehe „Warum 34 und nicht 45 Minuten“. Bei 34 Minuten liegt der gute Läufer nahe am Ziel, nicht im Band; siehe „Zahlen“.

## Dateien

- `gleisdreieck.omap`: die Karte, dazu `ATTRIBUTION.txt` mit Urheber und Erlaubnis
- `gleisdreieck-score.courseplan.toml`: die Projektdatei
- `transcript.de.txt`, `transcript.en.txt`: die Befehle und ihre Ausgabe
- `check.de.txt`, `check.en.txt`: die Prüfung der Bahn, mit vier akzeptierten Warnungen
- `course-data.xml`: die Bahn als IOF-XML 3.0
- `score-34.csv`, `score-34.sheet.de.md`, `score-34.sheet.md`: Postenliste und Auswertungsbogen

## So ist es entstanden

```sh
freemapper-courseplan init gleisdreieck.omap --project gleisdreieck-score.courseplan.toml --title "Score-Bahn Gleisdreieck"
freemapper-courseplan special add gleisdreieck-score.courseplan.toml --kind start --id S-score --at 100,-290
freemapper-courseplan special add gleisdreieck-score.courseplan.toml --kind finish --id F-score --at 100,-275
freemapper-courseplan score suggest gleisdreieck-score.courseplan.toml --start S-score --finish F-score --limit 8
freemapper-courseplan score draft gleisdreieck-score.courseplan.toml "Score 34" --time-limit 34 --start S-score --finish F-score --apply
freemapper-courseplan score length gleisdreieck-score.courseplan.toml "Score 34"
freemapper-courseplan check gleisdreieck-score.courseplan.toml
freemapper-courseplan finding accept gleisdreieck-score.courseplan.toml wkb2026.score.code_ambiguity --controls 63,93 --note "..."
freemapper-courseplan export gleisdreieck-score.courseplan.toml --iof-xml course-data.xml
freemapper-courseplan export gleisdreieck-score.courseplan.toml --csv score-34.csv --course "Score 34"
freemapper-courseplan export gleisdreieck-score.courseplan.toml --sheet score-34.sheet.de.md --course "Score 34"
```

`map extent` ergab eine Ausdehnung von -138,9 bis 157,8 mm in x und -330,4 bis 110,9 mm in y. Start und Ziel liegen im
Norden der Karte, 15 mm (60 m) auseinander. Der Entwurf hat beide erreicht; in einem Gebäude oder gesperrten Gebiet
bräche er mit „nicht erreichbar“ ab. Beide liegen auf der kartierten Fläche, wie alle Posten; das prüft `check` seit
ADR 0084 (Regel `wkb2026.map.inside`). `map near` nennt Abstände, sagt aber nicht, ob ein Punkt in einer Fläche liegt;
sieh dir beide Positionen auf der Karte an. `transcript.de.txt` hat jeden Befehl mit seiner Ausgabe.

## Zahlen

| | |
|---|---|
| Posten, Punkte | 33, 95 |
| Zeitlimit | 34 Minuten |
| Guter Läufer | 90 von 95 Punkten, 94,7 Prozent (Band 87 bis 93): nahe am Ziel, 1,7 Prozentpunkte über dem Band |
| Sehr guter Läufer | alle 95 Punkte, zurück nach 96,3 Prozent des Limits (Band 95 bis 100): im Band |
| Laufzeit, Release-Build, ausgelasteter Rechner | `score suggest` etwa 2 s, `score draft` etwa 15 s, `score length` etwa 2 s |

Die Bänder sind Zielwerte mit einer Toleranz von 3 Prozentpunkten (ADR 0083). Der sehr gute Läufer liegt im Band, der
gute 1,7 Punkte darüber, also innerhalb der Toleranz: nahe am Ziel, kein Fehlschlag. `score draft` und `score length`
sagen das und enden mit 0. Ein Test (`gleisdreieck_score_example_reaches_its_targets_and_check_is_quiet`) hält es so.
Die Laufzeiten habe ich auf einem Rechner gemessen, den andere Prozesse stark auslasteten (Last etwa 30); auf einem
ruhigen sind sie eher kürzer.

`check` meldet keinen Fehler und vier Warnungen: Die Codes 63 und 93, 61 und 91, 64 und 94 sowie 62 und 92 sehen auf dem
Kopf gleich aus. Mit 33 Posten sind fast alle freien Codes vergeben, deshalb kommen solche Paare vor. Die Projektdatei
akzeptiert die vier Befunde mit dem Hinweis, wie weit die beiden Posten eines Paares auseinander stehen (rund 470, 320,
520 und 530 m; der Test rechnet die Abstände nach). Das Limit von 34 Minuten liegt im üblichen Bereich von 30 bis 90
Minuten, eine Limit-Warnung gibt es nicht.

## Warum 34 und nicht 45 Minuten

Ich habe 45, 40, 35 und 30 Minuten und alle Limits dazwischen von 31 bis 34 mit den Standardeinstellungen und dem
Startwert 1 probiert (der Start oben, wie im Beispiel). Das Ergebnis des guten und des sehr guten Läufers:

| Limit | Guter Läufer | Sehr guter Läufer | Zustand |
|---|---|---|---|
| 45 min | 100 Prozent der Punkte | zurück nach 73,3 Prozent des Limits | verfehlt |
| 40 min | 100 Prozent der Punkte | zurück nach 82,5 Prozent | verfehlt |
| 35 min | 98,9 Prozent der Punkte | keine Abweichung genannt | verfehlt, der gute Läufer 5,9 Punkte über dem Band |
| 34 min | 94,7 Prozent | zurück nach 96,3 Prozent | nahe am Ziel (gut), im Band (sehr gut) |
| 33 min | 100 Prozent | keine Abweichung genannt | verfehlt, der Entwurf meldet, dass die freien Codes ausgehen (bester Versuch 30 Posten, 80 Punkte) |
| 32 min | 95,3 Prozent | zurück nach 96,4 Prozent | nahe am Ziel |
| 31 min | 93,0 Prozent | zurück nach 97,9 Prozent | im Band |
| 30 min | 93,0 Prozent | zurück nach 99,2 Prozent | im Band |

34 Minuten sind also das größte Limit, bei dem beide Läufer mindestens nahe am Ziel liegen. Das größte Limit mit beiden
Läufern im Band sind 31 Minuten. Ab 35 Minuten holt der gute Läufer fast oder ganz alle Punkte, und bei 40 und 45 Minuten hat der sehr gute Läufer viel
Zeit übrig; der beste Versuch hat dann 33 Posten mit 95 Punkten, so viele wie der Entwurf hier überhaupt baut. Warum 33
Minuten scheitern, 32 und 34 aber nicht, habe ich nicht untersucht; die Suche ist nicht erschöpfend, das Ergebnis kann
zwischen benachbarten Limits springen. Mit anderem Start, anderem Startwert oder höchstem Wert 9 habe
ich diesmal nichts probiert.

So kam das Beispiel zu seinem Stand. Eine erste Fassung hatte 40 Minuten, und das Programm hat sie geliefert; vier
ihrer Posten standen aber auf den Ecken eines Rechtecks, das der Kartenautor um den Karteninhalt gezeichnet hat. Das
war ein Fehler des Programms, der mit dieser Karte auffiel: Eine solche Umrandung liefert seitdem keine Postenstellen
mehr (ADR 0078). Die zweite Fassung hatte 33 Minuten mit beiden Läufern im Band. Sechs ihrer 27 Posten standen aber in
Kartenflächen mit dem Symbol 520, „Fläche, die nicht betreten werden darf“, die meisten an Gebäudeumrissen. `check`
meldet das seit ADR 0081 als Fehler, und seit ADR 0082 schlagen `score suggest` und `score draft` solche Stellen nicht
mehr vor. Dazu kam, dass die Bänder seit ADR 0083 Zielwerte mit Toleranz sind. Beides zusammen hat das Beispiel auf
34 Minuten und 33 Posten gebracht; ich habe keinen Posten von Hand verschoben.

## Grenzen dieses Beispiels

Die Routen kommen aus einem Raster der Kartensymbole ohne Höhe, und die Läuferpaces (4:10 und 3:45 Minuten je
Kilometer) sind eine erste Fassung. Die Stellen liegen dort, wo das Programm sie gefunden hat. Ich habe nicht jede auf
der Karte angesehen; sieh dir vor dem Einsatz jede Position an. Viele liegen an Gebäudeecken. Posten 73 steht auf einer
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
