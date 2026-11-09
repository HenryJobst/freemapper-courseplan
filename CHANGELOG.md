# Änderungen

[English version](CHANGELOG.en.md) · [Handbuch](docs/manual/de.md)

Hier stehen alle wichtigen Änderungen an freemapper-courseplan, die neueste zuerst. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/).

## [Unveröffentlicht]

### Hinzugefügt

- `check <PROJEKT>` prüft die Einzelbahnen gegen das Regelprofil der Veranstaltung und seine Aufsätze. Die
  Befunde stehen nach Bahnen geordnet, jeder mit Schwere, Ort, Erklärung, Kennung der Prüfregel und Quelle,
  danach eine Zusammenfassung. `--all` zeigt auch die akzeptierten Befunde mit Begründung und die übersprungenen
  Prüfregeln mit dem Grund. Eine Prüfregel, deren Filter (Disziplin, Kartenart, Druckmaßstab) nicht passt, gilt
  als übersprungen und wird so gemeldet. Mannschafts-, Staffel- und Score-Bahnen werden noch nicht geprüft. Der
  Exit-Code ist 0 ohne Befund der Schwere Fehler, 1 mit einem solchen Befund oder bei abgelehnter Prüfung und 2
  bei falschem Aufruf. Die einzelnen Prüfungen kommen mit den nächsten Schritten; bis dahin meldet `check` sie
  als noch nicht umgesetzt.
- `finding accept <PROJEKT> <REGEL> --note TEXT` akzeptiert einen Befund als gewollte Ausnahme. Den Ort gibst du
  mit `--controls 31,32`, `--leg A:3` oder `--course A` an, für einen Befund der ganzen Veranstaltung mit nichts
  davon. Wer denselben Befund noch einmal akzeptiert, ersetzt die Begründung. `finding remove` nimmt das zurück;
  einen nicht vorhandenen akzeptierten Befund lehnt es ab.
- `event profile <PROJEKT> <BASIS> [--overlay ID …]` setzt Regelprofil und Aufsätze, nachdem es sie geladen hat.
  `info` zeigt beides an.
- MCP: die Werkzeuge `check`, `accept_finding`, `remove_accepted_finding` und `set_rule_profile`. `check` gibt
  Befunde, akzeptierte Befunde und übersprungene Prüfregeln als JSON zurück, mit Maschinenfeldern und dem Text in
  der Sprache des Servers. `event_info` nennt Regelprofil und Aufsätze.
- `check` prüft jetzt Abstände auf der Karte: Posten näher beieinander als der Mindestabstand (Start und
  Querungsstellen zählen mit), Posten an ähnlichen Objekten zu nah beieinander, Posten, Start und Ziel nah am
  Kartenrand, Strecken, die durch einen anderen Posten derselben Bahn führen, und gerade Strecken durch ein
  Sperrgebiet als Warnung, dass die Optimalroute hindurchführen könnte. Abgeflaggte Strecken dürfen durch
  Sperrgebiete führen. Gebäude, Mauern, Zäune und Wasser prüft erst die spätere Routenanalyse. Jeder Befund nennt
  den gemessenen Wert und die Grenze, etwa „Posten 31 und Posten 32 liegen 12 m auseinander, mindestens 15 m“.
  Ein Paar oder ein Posten wird einmal gemeldet, auch wenn mehrere Bahnen ihn nutzen. Start und Ziel können Ort
  eines Befunds sein: `finding accept … --controls 31,S1`.
- `finding accept` lehnt einen Ort ab, an dem die Prüfregel nie einen Befund meldet, etwa `--course A` für den
  Mindestabstand, der immer zwei Posten betrifft.
- `check` prüft jetzt die Form der Strecken. Treffen sich ankommende und abgehende Strecke an einem Posten oder
  einer Querungsstelle in einem spitzen Winkel, meldet es eine Warnung unter 45° und einen Hinweis unter 90°,
  aber nur die strengere der beiden. Zwei Strecken einer Bahn, die auf mindestens 30 m höchstens 10 m
  voneinander entfernt gegenläufig verlaufen, ergeben eine Warnung, die beide Strecken nennt. Um einen Posten,
  an dem sich beide treffen, zählen die ersten 10 m nicht mit, auch in der Mitte einer Schmetterlingsschleife.
  Einen solchen Befund akzeptierst du mit beiden Strecken: `finding accept … --leg A:2 --leg A:5`.
- `check` prüft jetzt die Codes der Posten: Ein numerischer Code außerhalb von 31 bis 599 ist ein Fehler, ein
  Code, der keine Zahl ist, eine Warnung. Beide Codes eines verbotenen Paars (66 und 99, 68 und 89, 86 und 98)
  in der Veranstaltung sind ein Fehler, auch wenn sie in verschiedenen Bahnen stehen. Zwei aufeinanderfolgende
  Posten, deren Codes auf dem Kopf gleich aussehen oder sich nur durch 6 und 9 unterscheiden, ergeben eine
  Warnung. Querungsstellen, Start und Ziel haben keinen Code und zählen nicht; eine Querungsstelle zwischen zwei
  Posten trennt deren Paar nicht. Ein Posten an einem Punktsymbol, das im Postenkreis noch einmal vorkommt, ergibt
  eine Warnung, wenn Spalte C der Postenbeschreibung leer ist; Linien und Flächen werden nicht verglichen. Jeder
  Code und jedes Paar wird einmal gemeldet, auch wenn mehrere Bahnen sie nutzen.
- `check` prüft jetzt den Start. Bahnen teilen sich eine Startminute nur am selben Start. Für jeden Start ergeben
  die erwarteten Starter der Kategorien auf seinen Bahnen, geteilt durch das Startfenster, die Zahl der
  verschiedenen ersten Posten, die pro Minute nötig sind. Haben die Bahnen weniger, meldet eine Warnung am Start
  die ersten Posten, die mehrere Bahnen teilen. Ohne Startfenster wird die Prüfregel übersprungen, ein Start mit
  einer Kategorie ohne erwartete Starter mit den Namen der Kategorien. Als Hinweis wird gemeldet, wenn zwei
  Bahnen am selben Start mit verschiedenen ersten Posten in der Richtung der ersten Strecke um weniger als 30°
  abweichen, einmal je Paar erster Posten. Die Richtung führt zum ersten Knickpunkt am Start, sonst zum ersten
  Bahnpunkt.

## [0.1.0] - 2026-11-09

### Hinzugefügt

- Erstes Beispiel: eine Sprintbahn in der Lübecker Altstadt unter `examples/luebeck-sprint`, mit Projektdatei,
  Befehlen und einer Skizze, die auch die README zeigt.
- MCP-Server: `freemapper-courseplan mcp` bietet KI-Assistenten das Bahnlegen über MCP an (stdin und stdout).
  25 Werkzeuge lesen und ändern ein Projekt: `event_info`, `list_controls|courses|specials|classes`,
  `add_control`, `move_control`, `remove_control`, `describe_control`, `set_control_number`, `add_special`,
  `remove_special`, `set_special_courses`, `add_class`, `remove_class`, `add_course`, `remove_course`,
  `set_course_controls`, `set_leg`, `set_course_details`, `place_numbers`, `set_course_number`,
  `map_features_near`, `map_extent` und `init_event`. Positionen sind Objekte mit `x` und `y` in
  Kartenmillimetern. Ein Postencode darf Text oder eine ganze Zahl sein. `set_course_controls` und
  `remove_control` nennen in `cleared_legs` die Strecken, die ihre Knickpunkte verloren haben. Jeder Aufruf lädt die Projektdatei, wendet eine Änderung an und
  speichert nur bei Erfolg. Relative Pfade lehnt der Server ab. Beschreibungen, Hinweise für den Assistenten und
  Fehlertexte kommen aus dem Katalog in der Sprache von `--lang`, `FREEMAPPER_LANG` oder dem System; Werkzeug- und
  Feldnamen bleiben Englisch. Das Handbuch hat dazu das Kapitel 3.
- Befehle für Bahnen: `course add`, `course controls`, `course leg`, `course set`, `course number`,
  `course numbers` und `course remove`. Anlegen und ändern geht nur für Einzelbahnen. `course leg --after N` zählt
  ab 1 wie auf der Karte, Querungsstellen eingeschlossen. Knickpunkte gibst du mit wiederholtem `--bend X,Y` an;
  ohne `--bend` verliert die Strecke ihre Knickpunkte, ohne `--taped` gilt sie als nicht abgeflaggt. Führt eine
  Strecke nach `course controls` oder `control remove` zu einem anderen Bahnpunkt, verliert sie Knickpunkte und
  Abflaggung, und der Befehl meldet sie in einer eigenen Zeile. So ändern alte Knickpunkte die Länge nicht mehr
  unbemerkt. `course number --after N` setzt oder löscht die Nummernlage eines Postens in einer Bahn. `course
  numbers` setzt die übrigen Postennummern so, dass sie Strecken und Kreise frei lassen; es läuft nur auf Aufruf
  und behält zentrale Nummernlagen und die der Bahn. Richtungen rechnet das Programm auf 0 bis 360 Grad um, ein
  Abstand muss größer als 0 und höchstens 10 Druckmillimeter sein. Die Ausgabe nutzt im Deutschen das
  Dezimalkomma, Eingaben behalten den Punkt.
- `info` zeigt je Bahn eine Zeile mit Postenzahl, Länge in Kilometern und Anstieg. Querungsstellen zählen nicht als
  Posten, auch nicht in der Zeile mit den Summen, die sie getrennt zählt. Die Länge folgt WKB B 7.2.5: gerade
  Linien, Umwege als Knickpunkte, abgeflaggte Strecken entlang ihrer Knickpunkte. Für Mannschafts-, Staffel- und
  Score-Bahnen steht noch keine Länge da.
- Befehle für Posten, Sonderobjekte und Kategorien: `control add|move|remove|describe|number`,
  `special add|courses|remove` und `class add|remove`. Jede Änderung wird erst geprüft und dann gespeichert;
  schlägt die Prüfung fehl, bleibt die Datei unverändert. Punkte stehen in Kartenmillimetern, mit Punkt als
  Dezimaltrenner. Das Programm lehnt Punkte außerhalb der Karte ab. `special courses <PROJEKT> <ID> (A,B | --all)`
  ordnet ein Sonderobjekt Bahnen zu. Die einzige Bahn eines Sonderobjekts lässt sich nicht entfernen; die Meldung
  sagt, was zu tun ist. Codes, IDs, Namen und die Einträge von Listen mit Kommas verlieren Leerzeichen am Rand, so
  geht `--controls "31, 32"`. `help control add` zeigt die Hilfe eines Unterbefehls. Die deutsche Hilfe nennt die
  möglichen Werte auf Deutsch, und ein ungültiger Wert führt die gültigen auf.
- Bibliothek: Bahnlegen, Bahnlänge und Postennummern. Das Programm berechnet Strecken mit Knickpunkten, die
  Länge einer Bahn bei einem Kartenmaßstab und die Nummern der Posten (Querungsstellen bekommen keine). Eine Nummer
  platziert es neben dem Kreis, ohne Strecken, Kreise oder andere Nummern zu berühren. Überdruckgrößen folgen der
  Kartenart und bleiben bei vergrößertem Druck auf der Karte gleich groß.
- Kartenindex: Das Programm sucht jedes Kartensymbol nur einmal je Symbol nach, nicht einmal je Objekt. Das
  beschleunigt das Prüfen von Punkten gegen die Karte, die Ergebnisse bleiben gleich.
- Datenmodell: Querungsstellen, IDs für Sonderobjekte, Knickpunkte und Abflaggung der Strecke vom Start sowie ein
  Druckbereich je Bahn.
- Die Projektdatei lehnt leere oder von Leerzeichen umgebene Postencodes, Sonderobjekt-IDs, Bahn- und
  Kategorienamen ab, ebenso einen Postencode, der zugleich ID eines Sonderobjekts ist, eine doppelt genannte Bahn
  in einem Sonderobjekt, Linien mit weniger als zwei Punkten und Flächen mit weniger als drei.
- Die deutschen Programmtexte nutzen die Begriffe des Glossars: Pflichtstrecke, abgeflaggte Strecke,
  Querungsstelle, Gefahrenbereich, Erste-Hilfe-Stelle, Verpflegungsstelle, verbotene Route, Nummernlage, Strecke
  und Knickpunkt.
- Programmtexte kommen aus Fluent-Dateien, eine je Sprache (`crates/freemapper-courseplan/i18n/`). Deutsch und
  Englisch sind dabei. `--lang`, sonst `FREEMAPPER_LANG`, sonst die Systemsprache bestimmt die Sprache; Hilfe und
  Meldungen richten sich danach. Eine weitere Sprache braucht nur einen neuen Ordner. Platzhalter in der
  Aufrufzeile und seltene Fehlermeldungen beim Einlesen der Kommandozeile bleiben Englisch.
- Projektdatei `*.courseplan.toml` mit Individual-, Team-, Staffel- und Score-Bahnen. Das Programm prüft sie beim
  Laden und lehnt unbekannte Schlüssel ab, damit ein Tippfehler auffällt. Wenn es die Datei speichert, bleiben
  deine Kommentare und das Layout erhalten.
- `freemapper-courseplan init <Karte>` legt aus einer Sprint- oder Schulhofkarte ein Projekt an. Titel und
  Projektdatei lassen sich mit `--title` und `--project` setzen. Waldkarten lehnt der Befehl vorerst ab.
- `freemapper-courseplan info <Projekt>` zeigt Titel, Karte, Kartenart, Posten, Bahnen je Art und Klassen.
- OOM-Karten (`.omap`) lesen. Eine Karte im alten Binärformat lehnt das Programm ab und sagt, dass man sie in
  OOM neu speichern soll.
- Projektgerüst nach dem Vorbild von freemapper-print: Rust-Workspace mit Datenmodell, Bibliothek und Programm
  `freemapper-courseplan`, CI für Linux, macOS und Windows, Release-Workflow.
- Entwurf des Designs, Glossar, die ersten Architekturentscheidungen und eine Sammlung der Regeln zur
  Bahnlegung aus deutschen und internationalen Regelwerken.
