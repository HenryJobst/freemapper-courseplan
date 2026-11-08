# Handbuch zu freemapper-courseplan

[English](en.md) · [README](../../README.md)

Dieses Handbuch entsteht mit der ersten nutzbaren Version. Es wird Schritt für Schritt von der fertigen OOM-Karte zu
fertigen Bahnen führen, die OE auswertet und freemapper-print druckt, auch ohne Erfahrung mit der Kommandozeile.

Geplante Kapitel:

1. Was freemapper-courseplan macht
2. Installation
3. Mit einem KI-Assistenten
4. Ohne Assistent: Editor und Kommandozeile
5. Eine Veranstaltung anlegen: Karte, Disziplin, Kategorien
6. Posten setzen und beschreiben
7. Bahnen legen: Reihenfolge, Knickpunkte, Postennummern
8. Prüfen: Hinweise lesen, Ausnahmen begründen
9. Mit Purple Pen abgleichen
10. Weitergeben: IOF-XML für OE, Druck mit freemapper-print
11. Meldungen und was zu tun ist

## 3. Mit einem KI-Assistenten

Der Assistent legt die Bahnen für dich. Dafür ist freemapper-courseplan ein „MCP-Server“, ein Werkzeug, das
KI-Assistenten nutzen können. Das Programm hat selbst keine KI. Der Assistent ruft dieselben Bearbeitungsfunktionen
auf wie die Kommandozeile, und jeder Aufruf speichert in deine Projektdatei.

### Einmal einrichten

- **Claude Desktop:**
  1. *Einstellungen > Entwickler > Konfiguration bearbeiten* öffnen.
  2. In die Datei `claude_desktop_config.json` eintragen, mit deinem eigenen Pfad (unter Windows mit doppelten
     Backslashes `\\`):

     ```json
     {
       "mcpServers": {
         "freemapper-courseplan": {
           "command": "/Users/anna/Documents/freemapper-courseplan/freemapper-courseplan",
           "args": ["mcp"]
         }
       }
     }
     ```

  3. Claude Desktop neu starten.
- **Claude Code:** im Terminal `claude mcp add freemapper-courseplan -- /Pfad/zu/freemapper-courseplan mcp`
  eingeben.
- **Andere Assistenten:** Jeder Assistent mit MCP-Unterstützung nimmt denselben Eintrag: das Programm und das
  Argument `mcp`.

Der Assistent liest die Werkzeugtexte in der Sprache deines Systems. Willst du die Sprache festlegen, schreibe
`"--lang", "de"` vor `"mcp"` in die Argumente oder setze `FREEMAPPER_LANG=de` in der Umgebung des Programms.
Werkzeugnamen und Feldnamen bleiben in jeder Sprache Englisch.

### Eine Bahn legen

Sag dem Assistenten, was du brauchst, zum Beispiel:

> Lege eine Veranstaltung aus ~/karten/halle.omap an. Setze den Start an die Südtür und das Ziel neben die
> Sporthalle. Setze drei Posten auf den Hof, lege daraus die Bahn „Kurz“ für die Kategorie D10, setze die Nummern
> und sag mir die Länge.

Der Assistent geht dann so vor:

1. Er legt die Veranstaltung an. Die Projektdatei liegt neben der Karte und heißt `halle.courseplan.toml`.
2. Er fragt die Ausdehnung der Karte und die Kartenobjekte rund um eine Stelle ab, um die Tür, die Sporthalle und
   gute Plätze für Posten zu finden.
3. Er setzt Start, Ziel und Posten.
4. Er legt die Bahn an, setzt die Postennummern und liest Länge und Anstieg.

Du kannst in einem späteren Gespräch weitermachen. Gib dem Assistenten den Pfad der Projektdatei, dann setzt er
dort an.

### Was du wissen solltest

- Pfade sind absolut oder beginnen mit `~`. Ein relativer Pfad wird abgelehnt, weil der Assistent den
  Arbeitsordner des Programms nicht kennt.
- Jede Änderung wird sofort gespeichert. Lehnt das Programm eine Änderung ab, etwa einen Punkt außerhalb der Karte
  oder einen vergebenen Code, bleibt die Datei, wie sie war, und der Assistent bekommt den Grund.
- Positionen stehen in Kartenmillimetern im Kartenmaßstab, y zeigt nach unten. Der Assistent holt sie aus den
  Kartenobjekten. Sieh dir jede Postenposition selbst auf der Karte an, denn noch prüft nichts sie gegen die
  Regeln.
- Bisher lassen sich nur Einzelbahnen bearbeiten und messen. Bei Mannschafts-, Staffel- und Score-Bahnen sagt das
  Programm das.
- Die Projektdatei ist einfaches TOML. `freemapper-courseplan info <Projekt>` zeigt, was der Assistent getan hat.
