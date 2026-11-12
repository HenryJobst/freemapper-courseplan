# Sprinttraining Rummelsburger Bucht

[English](README.en.md) · [Alle Beispiele](../README.md)

Eine Sprintbahn mit fünf Posten auf der Karte „Rummelsburger Bucht“ von Remo Madella (MOO Challenge Berlin 2025,
1:4000, 308 mal 257 Kartenmillimeter, also etwa 1,2 mal 1,0 km). Gelegt wurde sie nur mit der Kommandozeile.
Die Karte liegt hier als `rummelsburg.omap`. Remo Madella hat sie am 12.11.2026 für die freemapper-Suite
freigegeben; das ist keine offene Lizenz, Einzelheiten stehen in [`ATTRIBUTION.txt`](ATTRIBUTION.txt) und unten.

Ein Vorschaubild gibt es noch nicht. Den Bahnaufdruck liefert freemapper-print, sobald es Bahnen aus einer `.ppen`
drucken kann; die Karte selbst siehst du im Editor (`freemapper-courseplan edit rummelsburg-sprint.courseplan.toml`).

## Dateien

- `rummelsburg.omap`: die Karte, dazu `ATTRIBUTION.txt` mit Urheber und Erlaubnis
- `rummelsburg-sprint.courseplan.toml`: die Projektdatei
- `transcript.de.txt`, `transcript.en.txt`: die Befehle und ihre Ausgaben
- `check.de.txt`, `check.en.txt`: die Prüfung der Bahn, ohne Befunde
- `course-data.xml`: die Bahn als IOF-XML 3.0, erzeugt mit `export`
- `rummelsburg-sprint.ppen`: die Bahn als Purple-Pen-Event, erzeugt mit `export --ppen`

## So entstand die Bahn

```text
$ freemapper-courseplan init rummelsburg.omap --project rummelsburg-sprint.courseplan.toml --title "Sprinttraining Rummelsburger Bucht"
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind start --id S1 --at -85,-68
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind finish --id F1 --at 95,0
$ freemapper-courseplan special add rummelsburg-sprint.courseplan.toml --kind first-aid --id H1 --at -93,-70
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at -70.05,-40.78
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at -13.67,7.12
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at 23.39,76.2
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at 60.73,73.78
$ freemapper-courseplan control add rummelsburg-sprint.courseplan.toml --at 67.69,27.2
$ freemapper-courseplan class add rummelsburg-sprint.courseplan.toml Offen
$ freemapper-courseplan course add rummelsburg-sprint.courseplan.toml A --start S1 --finish F1 --controls 31,32,33,34,35 --classes Offen
$ freemapper-courseplan course numbers rummelsburg-sprint.courseplan.toml A
$ freemapper-courseplan info rummelsburg-sprint.courseplan.toml
Bahn A: 5 Posten, 1,2 km, Anstieg – m
```

Die Koordinaten sind Kartenmillimeter, y zeigt nach unten. `map extent` nennt die Ausdehnung der Karte, -154,9 bis
153,0 mm in x und -126,6 bis 130,2 mm in y. Jede Ausgabe steht in [`transcript.de.txt`](transcript.de.txt). Das
Projekt setzt keinen Anstieg, weil ich keinen gemessen habe; `info` zeigt dann einen Strich.

## Gerade Strecken

Jede Strecke ist die gerade Linie von Posten zu Posten, ohne Knickpunkte. Knickpunkte sind die Ausnahme: Du setzt
sie nur, wenn du eine Strecke bewusst führen willst (siehe das Handbuch, Abschnitt „Strecken und Knicke“), und
eine markierte Strecke ins Ziel darf einen Umweg haben. Diese Bahn braucht beides nicht.

Die Karte besteht zu großen Teilen aus Gebäuden, Wasser und Flächen, die man nicht betreten darf (Symbol 520), mit
schmalen Gassen dazwischen. Die Luftlinie zwischen zwei Posten läuft deshalb meist über solche Flächen. Das ist
gewollt: Der Läufer geht drumherum, und diese Wahl zwischen Gassen und Plätzen ist die Routenwahl der Bahn. `check`
meldet dazu nichts, denn die Linie zwischen zwei Posten ist nicht die Laufroute (ADR 0080).

Die Posten stehen an hervorgehobenen Kartenobjekten außerhalb gesperrter Flächen, jeweils mindestens 0,8 mm von
Gebäuden, Wasser und gesperrten Flächen entfernt: 31 und 32 an Kreuzen (Symbol 531), 33 und 35 an Denkmälern (526),
34 an einem Ring (530). Keiner der Posten hat ein gleiches Objekt in seinem Kreis. Die Strecken sind 124 m (Start
bis 31), 296 m, 314 m, 150 m, 188 m und 154 m (35 bis Ziel) lang, zusammen 1,2 km Luftlinie. Ich habe die Stellen mit
einem kleinen Skript gesucht, das auf den Umrissen der Karte rechnet, und sie nicht jede auf der Karte angesehen.
Sieh dir jeden Posten im Editor an, bevor du die Bahn benutzt.

## Prüfen

`check` findet bei dieser Bahn nichts: `0 Fehler, 0 Warnungen, 0 Hinweise; 0 akzeptiert, 6 übersprungen`. Alle
Winkel an den Posten liegen zwischen 102° und 159°. Von den sechs übersprungenen Prüfregeln sind vier Längen- und
Schwierigkeitsregeln des Wettkampfs, die beim Training nicht gelten. Die fünfte ist die Startkapazität, weil die
Veranstaltung kein Startfenster hat, die sechste die Regel, dass die beste Laufroute nicht durch ein Sperrgebiet führen
darf: Sie braucht eine Routenanalyse, die das Programm noch nicht hat. Alle Befehle und die Ausgabe von `check --all`
stehen in [`check.de.txt`](check.de.txt) und [`check.en.txt`](check.en.txt).

## Sonderobjekte

Außer Start und Ziel hat das Projekt einen Erste-Hilfe-Posten `H1` im Startbereich. Die Bahn hat keine markierte
Strecke.

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
melden, ob sich alles laden lässt und ob Posten, Bahn und Nummern an der richtigen Stelle stehen.

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
