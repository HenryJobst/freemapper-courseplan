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

Der Assistent sieht die Karte nicht. Er kann ihre Ausdehnung abfragen und für einen Punkt die Kartenobjekte im
Umkreis, jeweils mit Symbolnummer, Art (Punkt, Linie, Fläche) und Abstand. Namen wie „Südtür“ oder
„Sporthalle“ kennt er nicht, und wo ein Objekt genau liegt, sagt ihm die Abfrage auch nicht. Gib ihm deshalb
Positionen vor, in Kartenmillimetern oder bezogen auf die Ausdehnung der Karte, zum Beispiel:

> Lege eine Veranstaltung aus ~/karten/halle.omap an. Setze den Start ein Viertel vom linken Rand auf halbe Höhe
> und das Ziel 10 mm rechts daneben. Setze drei Posten über die Karte verteilt und sag mir zu jedem, welche
> Kartenobjekte im Umkreis von 2 mm liegen. Lege daraus die Bahn „Kurz“ für die Kategorie D10 an, setze die
> Postennummern und sag mir die Länge.

Der Assistent geht dann so vor:

1. Er legt die Veranstaltung an. Die Projektdatei liegt neben der Karte und heißt `halle.courseplan.toml`.
2. Er fragt die Ausdehnung der Karte ab, also ihre kleinste und größte Ecke in Kartenmillimetern, den Maßstab
   und die Kartenart, und rechnet daraus die Positionen aus.
3. Er setzt Start, Ziel und Posten an diese Positionen.
4. Er fragt für jeden Posten die Kartenobjekte in der Nähe ab und nennt dir Symbolnummern und Abstände, etwa
   „Symbol 521, Fläche, 0,4 mm“. Was das Symbol bedeutet, schlägst du im Symbolsatz der Karte nach.
5. Er legt die Bahn an, setzt die Postennummern und liest Länge und Anstieg.

Du kannst in einem späteren Gespräch weitermachen. Gib dem Assistenten den Pfad der Projektdatei, dann setzt er
dort an.

### Was du wissen solltest

- Pfade sind absolut oder beginnen mit `~`. Ein relativer Pfad wird abgelehnt, weil der Assistent den
  Arbeitsordner des Programms nicht kennt.
- Jede Änderung wird sofort gespeichert. Lehnt das Programm eine Änderung ab, etwa einen Punkt außerhalb der Karte
  oder einen vergebenen Code, bleibt die Datei, wie sie war, und der Assistent bekommt den Grund.
- Positionen stehen in Kartenmillimetern im Kartenmaßstab, y zeigt nach unten. Sieh dir jede Postenposition
  selbst auf der Karte an, denn noch prüft nichts sie gegen die Regeln.
- Postennummern setzt das Programm nur, wenn der Assistent darum bittet. Ändert er danach Strecken oder die
  Reihenfolge, muss er sie neu setzen lassen. Eine Nummernlage, die für einen Posten oder in einer Bahn
  festgelegt ist, bleibt dabei stehen.
- Ändert sich die Reihenfolge einer Bahn, verliert jede Strecke, die jetzt zu einem anderen Bahnpunkt führt, ihre
  Knickpunkte und ihre Abflaggung. Der Assistent bekommt eine Liste dieser Strecken.
- Bisher lassen sich nur Einzelbahnen bearbeiten und messen. Bei Mannschafts-, Staffel- und Score-Bahnen sagt das
  Programm das.
- Die Projektdatei ist einfaches TOML. `freemapper-courseplan info <Projekt>` zeigt, was der Assistent getan hat.

## 8. Prüfen: Hinweise lesen, Ausnahmen begründen

`check` legt jede Einzelbahn neben die Regeln, nach denen die Veranstaltung gelten soll, und sagt dir, was auffällt.
Es ändert nichts an der Projektdatei. Du entscheidest, ob du die Bahn änderst oder den Befund als gewollte Ausnahme
begründest.

```sh
freemapper-courseplan check halle.courseplan.toml
freemapper-courseplan check halle.courseplan.toml --all
```

Mannschafts-, Staffel- und Score-Bahnen prüft das Programm noch nicht. Für jede meldet es einmal, dass sie nicht geprüft wurde.

### Eine Zeile lesen

```text
Bahn A:
  Hinweis Bahn A, Strecke nach 6: Die Strecken am Posten 37 bilden einen Winkel von 85°, mindestens 90° (wkb2026.leg.acute_angle_hint, BL12 5. Bahnkonzeption)
```

Von links nach rechts steht da:

- die Schwere, hier „Hinweis“,
- der Ort, hier die Strecke nach der sechsten Position von Bahn A (Position 0 wäre die Strecke vom Start),
- der Text mit gemessenem Wert und Grenze,
- in Klammern die Kennung der Prüfregel und die Quelle, aus der die Grenze stammt.

Die Kennung brauchst du, um eine Ausnahme zu begründen. Die Quelle sagt dir, wo du die Regel selbst nachlesen
kannst. Unter den Befunden einer Bahn steht am Ende eine Zusammenfassung, im Beispiel
„0 Fehler, 0 Warnungen, 1 Hinweis; 0 akzeptiert, 5 übersprungen“.

### Schwere und Exit-Code

Das Regelprofil legt für jede Prüfregel fest, wie schwer ein Verstoß wiegt:

- Fehler: Die Bahn verletzt eine Regel, die das Profil als verbindlich führt.
- Warnung: Die Bahn weicht ab, und du solltest sie ändern oder die Abweichung begründen.
- Hinweis: Etwas fällt auf, das du ansehen solltest, etwa ein Winkel knapp unter dem Richtwert.

Der Exit-Code sagt einem Skript, wie es ausging: 0, wenn kein Befund der Schwere Fehler offen ist, 1 bei
mindestens einem Fehler oder wenn die Prüfung abgelehnt wurde (Projektdatei oder Karte nicht lesbar), und 2 bei
falschem Aufruf, etwa einer unbekannten Option. Warnungen und Hinweise ändern den Exit-Code nicht, und
akzeptierte Befunde zählen nicht.

Einige Prüfungen sind Vorschläge des Programms und stehen nicht wörtlich in den Regeln, zum Beispiel dass die
halbierten Postenabstände auch auf Schulhofkarten gelten. Die Hinweise zum Profil (`note` in der Profildatei,
siehe `crates/freemapper-courseplan/rules/README.md`) kennzeichnen sie mit „Claude's call“. Führt die gerade
Strecke durch ein Sperrgebiet, meldet das Programm nur eine Warnung, denn die Optimalroute könnte hindurchführen.
Ob sie es tut, kann erst die spätere Routenanalyse sagen.

### Übersprungene Prüfregeln

`check` zählt die übersprungenen Prüfregeln in der Zusammenfassung, `check --all` nennt jede mit dem Grund.
Häufige Gründe:

- Beim Training wendet das Programm die Längen- und Schwierigkeitsregeln des Wettkampfs nicht an.
- Die Prüfregel gilt nur für eine andere Disziplin, Kartenart oder einen anderen Druckmaßstab.
- Ein Wert fehlt, etwa das Startfenster der Veranstaltung für die Startkapazität oder der Anstieg einer Bahn.
- Eine Kategorie ist keiner Kategorie der Regeltabelle zugeordnet, zum Beispiel ein selbst gewählter Name wie
  „Offen“.

Eine übersprungene Prüfregel hat nichts bestanden, sie hat nur nicht geprüft. Lies die Liste deshalb einmal durch,
bevor du dich auf ein leeres Ergebnis verlässt.

### Eine Ausnahme begründen

Manche Befunde sind gewollt. Du akzeptierst sie mit der Kennung der Prüfregel, dem Ort, wie `check` ihn zeigt, und
einer Begründung:

```sh
# zwei Posten, die bewusst nah beieinander stehen
freemapper-courseplan finding accept halle.courseplan.toml wkb2026.distance.min_half --controls 31,32 --note "Die Posten liegen an zwei Seiten desselben Gebäudes und gehören zu verschiedenen Bahnen."

# eine Strecke
freemapper-courseplan finding accept halle.courseplan.toml wkb2026.leg.acute_angle --leg A:3 --note "Der spitze Winkel ist gewollt, die Bahn wendet an diesem Posten."

# ein Posten und der Start
freemapper-courseplan finding accept halle.courseplan.toml wkb2026.distance.min_half --controls 31,S1 --note "Der Start liegt bewusst nah am ersten Posten, das Training beginnt mit einer kurzen Einlaufstrecke."
```

`--controls` nimmt Postencodes und die Kennung von Start oder Ziel, `--leg BAHN:N` die Strecke nach Position N
(`--leg A:2 --leg A:5` für einen Befund an zwei Strecken), `--course BAHN` einen Befund der ganzen Bahn. Ein Befund
der ganzen Veranstaltung braucht keine Ortsangabe. Gibst du einen Ort an, an dem die Prüfregel nie etwas meldet,
lehnt das Programm ab. Akzeptierst du denselben Befund noch einmal, ersetzt die neue Begründung die alte.

Die Begründung steht in der Projektdatei. `check` zeigt akzeptierte Befunde nicht mehr in der Liste, zählt sie aber
in der Zusammenfassung. Mit `--all` stehen sie unter „akzeptiert“, mit deiner Begründung. So sieht auch die
Bahnlegerin im nächsten Jahr, warum die Bahn so ist.

Willst du einen Befund wieder sehen, nimmst du die Ausnahme mit denselben Angaben zurück:

```sh
freemapper-courseplan finding remove halle.courseplan.toml wkb2026.distance.min_half --controls 31,32
```

Ändert sich die Kennung einer Prüfregel, etwa in einer neuen Programmversion, meldet `check` die Befunde dazu wieder.

### Regelprofil und Aufsätze wählen

Eine Veranstaltung hat ein Basisprofil und beliebig viele Aufsätze darauf. Das Programm bringt zwei mit:
`wkb-2026` (Wettkampfbestimmungen Orientierungslauf 2026) und `nor-bb-2021`, einen Aufsatz mit den
Bahnempfehlungen von Berlin und Brandenburg. Beide setzt du mit einem Befehl:

```sh
freemapper-courseplan event profile halle.courseplan.toml wkb-2026 --overlay nor-bb-2021
```

Mehrere Aufsätze wiederholen `--overlay` und gelten in der angegebenen Reihenfolge. Das Programm lädt die Profile
vorher. Einen unbekannten Namen lehnt es mit der Liste der vorhandenen Profile ab. `info` zeigt das Profil an. In jedem Befund von `check`
beginnt die Kennung der Prüfregel mit ihrem Profil (`wkb2026`, `norbb2021`). Wie eine Profildatei aufgebaut ist und wie man
eine neue anlegt, steht in `crates/freemapper-courseplan/rules/README.md`.

### Mit dem KI-Assistenten

Der Assistent hat vier Werkzeuge dafür: `check` liefert Befunde, akzeptierte Befunde und übersprungene
Prüfregeln, `accept_finding` und `remove_accepted_finding` entsprechen `finding accept` und `finding remove`, und
`set_rule_profile` entspricht `event profile`. Bitte ihn zum Beispiel, die Bahnen zu prüfen, jeden Befund in einem
Satz zu erklären und dich zu fragen, bevor er einen akzeptiert. Die Begründung schreibst du oder diktierst sie ihm.
Er soll sie nicht erfinden.
