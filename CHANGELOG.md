# Änderungen

[English version](CHANGELOG.en.md) · [Handbuch](docs/manual/de.md)

Hier stehen alle wichtigen Änderungen an freemapper-courseplan, die neueste zuerst. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/).

## [Unveröffentlicht]

### Hinzugefügt

- MCP-Server: `freemapper-courseplan mcp` bietet KI-Assistenten das Bahnlegen über MCP an (stdin und stdout).
  23 Werkzeuge lesen und ändern ein Projekt: `event_info`, `list_controls|courses|specials|classes`,
  `add_control`, `move_control`, `remove_control`, `describe_control`, `set_control_number`, `add_special`,
  `remove_special`, `add_class`, `remove_class`, `add_course`, `remove_course`, `set_course_controls`, `set_leg`,
  `set_course_details`, `place_numbers`, `map_features_near`, `map_extent` und `init_event`. Positionen sind
  Objekte mit `x` und `y` in Kartenmillimetern. Jeder Aufruf lädt die Projektdatei, wendet eine Änderung an und
  speichert nur bei Erfolg. Relative Pfade lehnt der Server ab. Beschreibungen, Hinweise für den Assistenten und
  Fehlertexte kommen aus dem Katalog in der Sprache von `--lang`, `FREEMAPPER_LANG` oder dem System; Werkzeug- und
  Feldnamen bleiben Englisch. Das Handbuch hat dazu das Kapitel 3.
- Befehle für Bahnen: `course add`, `course controls`, `course leg`, `course set`, `course numbers` und
  `course remove`. Anlegen und ändern geht nur für Einzelbahnen. `course leg --after INDEX` zählt ab 1 wie auf der Karte, Querungspunkte
  eingeschlossen. Knicke gibst du mit wiederholtem `--bend X,Y` an; ohne `--bend` verliert die Teilstrecke ihre
  Knicke, ohne `--taped` verliert sie die Markierung. `course numbers` setzt die Postennummern so, dass sie
  Strecken und Kreise frei lassen. Die Ausgabe nutzt im Deutschen das Dezimalkomma, Eingaben behalten den Punkt.
- `info` zeigt je Bahn eine Zeile mit Postenzahl, Länge in Kilometern und Anstieg. Querungspunkte zählen nicht als
  Posten. Die Länge folgt WKB B 7.2.5: gerade Linien, Umwege als Knicke, markierte Strecken entlang ihrer Knicke.
  Für Mannschafts-, Staffel- und Score-Bahnen steht noch keine Länge da.
- Befehle für Posten, Sonderobjekte und Kategorien: `control add|move|remove|describe|number`,
  `special add|remove` und `class add|remove`. Jede Änderung wird erst geprüft und dann gespeichert; schlägt die
  Prüfung fehl, bleibt die Datei unverändert. Punkte stehen in Kartenmillimetern, mit Punkt als Dezimaltrenner.
  Das Programm lehnt Punkte außerhalb der Karte ab.
- Bibliothek: Bahnlegen, Bahnlänge und Postennummern. Das Programm berechnet Teilstrecken mit Knicken, die Länge
  einer Bahn bei einem Kartenmaßstab und die Nummern der Posten (Querungspunkte bekommen keine). Eine Nummer
  platziert es neben dem Kreis, ohne Strecken, Kreise oder andere Nummern zu berühren. Überdruckgrößen folgen der
  Kartenart und bleiben bei vergrößertem Druck auf der Karte gleich groß.
- Kartenindex: Das Programm sucht jedes Kartensymbol nur einmal je Symbol nach, nicht einmal je Objekt. Das
  beschleunigt das Prüfen von Punkten gegen die Karte, die Ergebnisse bleiben gleich.
- Datenmodell: Querungspunkte, IDs für Sonderobjekte, Knicke und Markierung der Teilstrecke vom Start sowie ein
  Druckbereich je Bahn.
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
