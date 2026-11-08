# Änderungen

[English version](CHANGELOG.en.md) · [Handbuch](docs/manual/de.md)

Hier stehen alle wichtigen Änderungen an freemapper-courseplan, die neueste zuerst. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/).

## [Unveröffentlicht]

### Hinzugefügt

- Projektdatei `*.courseplan.toml` mit Individual-, Team-, Staffel- und Score-Bahnen. Das Programm prüft sie beim
  Laden und behält beim Speichern deine Kommentare und das Layout.
- `freemapper-courseplan init <Karte>` legt aus einer Sprint- oder Schulhofkarte ein Projekt an. Titel und
  Projektdatei lassen sich mit `--title` und `--project` setzen. Waldkarten lehnt der Befehl vorerst ab.
- `freemapper-courseplan info <Projekt>` zeigt Titel, Karte, Kartenart, Posten, Bahnen je Art und Klassen.
- OOM-Karten (`.omap`) lesen, auch die Objekte in der Nähe eines Punkts. Das alte Binärformat lehnt das Programm
  mit einer klaren Meldung ab.
- Projektgerüst nach dem Vorbild von freemapper-print: Rust-Workspace mit Datenmodell, Bibliothek und Programm
  `freemapper-courseplan` (bisher nur `--version` und `--help`), CI für Linux, macOS und Windows,
  Release-Workflow.
- Entwurf des Designs, Glossar, die ersten Architekturentscheidungen und eine Sammlung der Regeln zur
  Bahnlegung aus deutschen und internationalen Regelwerken.
