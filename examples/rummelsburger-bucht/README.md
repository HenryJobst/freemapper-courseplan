# Sprinttraining Rummelsburger Bucht

[English](README.en.md) · [Alle Beispiele](../README.md)

Eine Sprintbahn mit vier Posten auf der Karte „Rummelsburger Bucht“ von Remo Madella (MOO Challenge Berlin 2025,
1:4000, 308 mal 257 Kartenmillimeter, also etwa 1,2 mal 1,0 km). Gelegt wurde sie nur mit der Kommandozeile.
Die Karte liegt hier als `rummelsburg.omap`. Remo Madella hat sie am 12.11.2026 für die freemapper-Suite
freigegeben; das ist keine offene Lizenz, Einzelheiten stehen in [`ATTRIBUTION.txt`](ATTRIBUTION.txt) und unten.

Ein Vorschaubild gibt es noch nicht. Den Bahnaufdruck liefert freemapper-print, sobald es Bahnen aus einer `.ppen`
drucken kann; die Karte selbst siehst du im Editor (`freemapper-courseplan edit rummelsburg-sprint.courseplan.toml`).

## Dateien

- `rummelsburg.omap`: die Karte, dazu `ATTRIBUTION.txt` mit Urheber und Erlaubnis
- `rummelsburg-sprint.courseplan.toml`: die Projektdatei
- `transcript.de.txt`, `transcript.en.txt`: die Befehle und ihre Ausgaben
- `check.de.txt`, `check.en.txt`: die Prüfung der Bahn, mit drei akzeptierten Hinweisen
- `course-data.xml`: die Bahn als IOF-XML 3.0, erzeugt mit `export`
- `rummelsburg-sprint.ppen`: die Bahn als Purple-Pen-Event, erzeugt mit `export --ppen`

## So entstand die Bahn

```text
$ freemapper-courseplan init rummelsburg.omap --project rummelsburg-sprint.courseplan.toml --title "Sprinttraining Rummelsburger Bucht"
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind start --id S1 --at -120,-75
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind finish --id F1 --at 48,69
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind first-aid --id H1 --at -112,-68
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at -70.05,-40.78
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at 9,-6.85
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at -13.67,7.12
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at 6.42,37.92
$ freemapper-courseplan class add rummelsburg-sprint.courseplan.toml Offen
$ freemapper-courseplan course add rummelsburg-sprint.courseplan.toml A --start S1 --finish F1 --controls 31,32,33,34 --classes Offen
$ freemapper-courseplan course leg rummelsburg-sprint.courseplan.toml A --after 1 --bend=-57,-43 --bend=-50,-50
$ freemapper-courseplan course leg rummelsburg-sprint.courseplan.toml A --after 3 --bend 6,27
$ freemapper-courseplan course leg rummelsburg-sprint.courseplan.toml A --after 4 --bend 6,41 --bend 13,34 --bend 20,50 --taped
$ freemapper-courseplan course numbers rummelsburg-sprint.courseplan.toml A
$ freemapper-courseplan info rummelsburg-sprint.courseplan.toml
Bahn A: 4 Posten, 1,1 km, Anstieg – m
```

Die Koordinaten sind Kartenmillimeter, y zeigt nach unten. `map extent` nennt die Ausdehnung der Karte, -154,9 bis
153,0 mm in x und -126,6 bis 130,2 mm in y. Jede Ausgabe steht in [`transcript.de.txt`](transcript.de.txt). Das
Projekt setzt keinen Anstieg, weil ich keinen gemessen habe; `info` zeigt dann einen Strich.

## Die Karte setzt die Grenzen

Die Karte besteht zu großen Teilen aus Flächen, die man nicht betreten darf (Symbol 520), aus Gebäuden und aus
Wasser. Zwischen ihnen liegen schmale Gassen. Eine gerade Strecke von Posten zu Posten kreuzt fast immer eine
gesperrte Fläche: Mein erster Entwurf mit acht Posten gab sieben Warnungen, bei fast jeder Strecke eine. Die
Strecken dieser Bahn laufen deshalb mit Knickpunkten durch die Gassen. Die Posten stehen an hervorgehobenen
Kartenobjekten (Symbole 531 und 530, Kreuz und Ring), jeweils mindestens 0,6 mm von Wasser und gesperrten Flächen entfernt. Die
Knickpunkte hat ein kleines Skript ausgewählt, das die Umrisse von Gebäuden, Wasser und gesperrten Flächen prüft,
nicht das Programm, und ich habe sie nicht auf der Karte angesehen. Sieh dir jede Strecke im Editor an, bevor du
die Bahn benutzt. `check` meldet für Strecken mit Knickpunkten keine Sperrflächen, weil die Routenanalyse noch
fehlt; es prüft sie also nicht.

Die Bahn hat vier Posten und 1,1 km. Ich habe Bahnen mit fünf und sechs Posten durchgerechnet; keine, die ich fand, blieb ohne
Warnung, meist weil zwei Strecken auf demselben Weg gegeneinander laufen.

## Prüfen

`check` findet drei Hinweise, alle Winkel. Die Strecken am Posten 32 bilden 67° (gefordert mindestens 90°), am
Posten 33 76°, und am Posten 34 läuft die Bahn mit 171° fast geradeaus weiter (erwünscht höchstens 170°). Die Gassen
geben die Richtung vor. Weil die Bahn ein Training ist, trägt `finding accept` die drei als gewollt in die
Projektdatei ein, mit der Begründung, dass sich die Winkel aus den Gassen ergeben. Danach meldet `check` nichts mehr:
`0 Fehler, 0 Warnungen, 0 Hinweise; 3 akzeptiert, 5 übersprungen`. Die fünf übersprungenen Prüfregeln sind
Längen- und Schwierigkeitsregeln des Wettkampfs, die beim Training nicht gelten, und die Startkapazität, weil die
Veranstaltung kein Startfenster hat. Alle Befehle und die Ausgabe von `check --all` stehen in
[`check.de.txt`](check.de.txt) und [`check.en.txt`](check.en.txt); die englische Datei zeigt englische
Begründungen, in der Projektdatei stehen die deutschen.

## Sonderobjekte und markierte Strecke

Außer Start und Ziel hat das Projekt einen Erste-Hilfe-Posten `H1` im Startbereich. Die letzte Strecke (von Posten 34
ins Ziel) ist als markierte Strecke gekennzeichnet (`--taped`).

## Export nach IOF-XML

```text
$ freemapper-courseplan export rummelsburg-sprint.courseplan.toml --iof-xml course-data.xml
```

Die Datei besteht die Prüfung gegen das IOF-Schema (`xmllint --noout --schema ../../tests/fixtures/iof-xml/IOF.xsd
course-data.xml`). Die Karte ist georeferenziert, deshalb tragen Start, Posten und Ziel Breite und Länge. `createTime`
ist die Zeit des Laufs, der die Datei erzeugt hat; ein neuer Lauf ändert nur diese Zeile. Ein Import in OE, Purple Pen oder Condes ist nicht getestet, weil im Projekt niemand OE hat. Wer es tut, soll melden, was
ankommt.

## Export nach Purple Pen

```text
$ freemapper-courseplan export rummelsburg-sprint.courseplan.toml --ppen rummelsburg-sprint.ppen
```

Die Datei gibt es noch nicht, deshalb schreibt `export` sie neu und trägt sie im Projekt als `sources.ppen` ein. **Diese
Datei ist in Purple Pen noch nicht geöffnet worden.** Ein Export eines früheren Beispiels (Lübecker Altstadt) hat
Purple Pen 4.0 ohne Fehler geöffnet; der Schreiber ist derselbe. Wer Purple Pen 4.0 hat, soll die Datei öffnen und
melden, ob sich alles laden lässt und ob Posten, Bahn, Knickpunkte und Nummern an der richtigen Stelle stehen.

## Score-Bahn auf dieser Karte

Die Karte ist für eine Score-Bahn klein. Mit Start `S1` und Ziel `F1` gelingt ein Entwurf mit 15 Minuten
(9 Posten, 27 Punkte; der gute Läufer holt 24 davon, 88,9 Prozent, der sehr gute alle und ist nach 99,7 Prozent
des Limits im Ziel) und mit 20 Minuten (13 Posten, 28 Punkte, 92,9 und 95,1 Prozent). Bei 25, 30 und 45 Minuten
reichen die Stellen nicht: Der sehr gute Läufer ist nach 80, 67 und 44,5 Prozent des Limits im Ziel statt nach 95 bis 100.
Für 45 Minuten nimm [`../gleisdreieck/`](../gleisdreieck/), die größere Karte. Das sind Läufe mit `score draft` ohne
`--apply`; das Projekt hier enthält keine Score-Bahn.

## Herkunft und Lizenz

Karte und Urheber stehen in [`ATTRIBUTION.txt`](ATTRIBUTION.txt). **Remo Madella hat die Karte für die
freemapper-Suite freigegeben, nicht unter einer offenen Lizenz.** Wer sie außerhalb der Suite nutzen will, fragt ihn.
Das Programm und die Bahndaten (Projektdatei, Exporte, Ausgaben) stehen unter GPL-3.0-or-later; der Karteninhalt
behält seine eigene Lage. Ob die Erlaubnis auch das Verteilen der `.omap` im öffentlichen Repository meint, ist nicht
geklärt (ADR 0077).
