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
  selbst auf der Karte an. `check` prüft Abstände, Winkel und Codes gegen die Regeln (Kapitel 8), aber nicht, ob
  der Posten am richtigen Objekt steht.
- Postennummern setzt das Programm nur, wenn der Assistent darum bittet. Ändert er danach Strecken oder die
  Reihenfolge, muss er sie neu setzen lassen. Eine Nummernlage, die für einen Posten oder in einer Bahn
  festgelegt ist, bleibt dabei stehen.
- Die automatische Platzierung setzt die Nummer hinter den Kreis, in Verlängerung der Strecke, die zum Posten
  hinführt. Knickt die Bahn kaum ab, läuft die weiterführende Strecke dort entlang, und die Nummer steht außen an
  der Spitze des Winkels. Auf Punkt- und Liniensymbolen der Karte steht sie nur, wenn kein freier Platz bleibt;
  Flächen darf sie überdecken. Gefällt dir die Lage nicht, setzt du sie selbst.
- Die Nummer steht 0,5 bis 5 mm vom Kreis entfernt (Abstand der Kreislinie zum Nummernfeld, Regel
  `wkb2026.number.gap`). Die automatische Platzierung beginnt bei 0,5 mm und rückt in 0,5-mm-Schritten nach
  außen, bevor sie von der bevorzugten Richtung abweicht. `check` warnt, wenn ein gespeicherter Abstand außerhalb
  liegt, etwa bei einer von Hand gesetzten Nummer mit 6 mm.
- Ändert sich die Reihenfolge einer Bahn, verliert jede Strecke, die jetzt zu einem anderen Bahnpunkt führt, ihre
  Knickpunkte und ihre Abflaggung. Der Assistent bekommt eine Liste dieser Strecken.
- Bisher lassen sich nur Einzelbahnen bearbeiten und messen. Bei Mannschafts-, Staffel- und Score-Bahnen sagt das
  Programm das.
- Die Projektdatei ist einfaches TOML. `freemapper-courseplan info <Projekt>` zeigt, was der Assistent getan hat.

## 8. Prüfen: Hinweise lesen, Ausnahmen begründen

`check` legt jede Einzelbahn neben die Regeln, nach denen die Veranstaltung gelten soll, und sagt dir, was
auffällt. Es ändert nichts an der Projektdatei. Du entscheidest, ob du die Bahn änderst oder den Befund als gewollte
Ausnahme begründest.

```sh
freemapper-courseplan check halle.courseplan.toml
freemapper-courseplan check halle.courseplan.toml --all
```

Winkel prüft `check` in beide Richtungen. Spitze Winkel meldet es unter 90° als Hinweis und unter 45° als Warnung.
Läuft die Bahn an einem Posten fast geradeaus, also mit einem Winkel über 170° zwischen den Strecken, gibt es
einen Hinweis (`wkb2026.leg.straight_angle_hint`). Der Posten ändert die Laufrichtung dann kaum, und hinter dem
Kreis ist kein Platz für die Nummer.

Mannschafts-, Staffel- und Score-Bahnen prüft das Programm noch nicht. Für jede meldet es einmal, dass sie nicht
geprüft wurde.

### Eine Zeile lesen

```text
Bahn A:
  Hinweis Bahn A, Strecke 36–37: Die Strecken am Posten 37 bilden einen Winkel von 85°, gefordert sind mindestens 90° (wkb2026.leg.acute_angle_hint, BL12 5. Bahnkonzeption)
```

Von links nach rechts steht da:

- die Schwere, hier „Hinweis“,
- der Ort, hier die Strecke von Posten 36 zu Posten 37 auf Bahn A. Eine Strecke heißt nach ihren Enden, etwa
  „Strecke Start–31“ oder „Strecke 37–Ziel“; Start und Ziel als Ort heißen „Start S1“ und „Ziel F1“,
- der Text mit gemessenem Wert und Grenze,
- in Klammern die Kennung der Prüfregel und die Quelle, aus der die Grenze stammt.

Die Kennung brauchst du, um eine Ausnahme zu begründen. Die Quelle sagt dir, wo du die Regel selbst nachlesen
kannst. Innerhalb einer Bahn stehen zuerst die Befunde der ganzen Bahn, dann die der Strecken in Laufrichtung. Am
Ende der Ausgabe steht einmal eine Zusammenfassung, im Beispiel „0 Fehler, 0 Warnungen, 1 Hinweis; 0 akzeptiert,
5 übersprungen“.

### Schwere und Exit-Code

Das Regelprofil legt für jede Prüfregel fest, wie schwer ein Verstoß wiegt. `check` selbst unterscheidet die
drei Stufen nur beim Exit-Code:

- Fehler: Das Profil führt die Regel als verbindlich. Ein offener Fehler setzt den Exit-Code 1.
- Warnung: Das Profil erwartet, dass du die Bahn änderst oder die Abweichung begründest.
- Hinweis: Etwas fällt auf, das du ansehen solltest, etwa ein Winkel knapp unter dem Richtwert.

Der Exit-Code sagt einem Skript, wie es ausging: 0, wenn kein Befund der Schwere Fehler offen ist, 1 bei
mindestens einem Fehler oder wenn die Prüfung abgelehnt wurde (Projektdatei oder Karte nicht lesbar), und 2 bei
falschem Aufruf, etwa einer unbekannten Option. Warnungen und Hinweise ändern den Exit-Code nicht, und
akzeptierte Befunde zählen nicht.

Einige Grenzen stehen so nicht in den Regeln, das Programm schlägt sie vor, zum Beispiel dass die halbierten
Postenabstände auch auf Schulhofkarten gelten. Als Quelle steht dann „Vorschlag von freemapper-courseplan“, und
`check --all` zeigt unter dem Befund die Anmerkung der Prüfregel, die den Vorschlag begründet. Führt die gerade
Strecke durch ein Sperrgebiet, meldet das Programm nur eine Warnung, denn die Optimalroute könnte hindurchführen.
Ob sie es tut, kann erst die spätere Routenanalyse sagen.

### Übersprungene Prüfregeln

`check` zählt die übersprungenen Prüfregeln in der Zusammenfassung, `check --all` nennt jede mit dem Grund.
Häufige Gründe:

- Beim Training wendet das Programm die Längen- und Schwierigkeitsregeln des Wettkampfs nicht an.
- Die Prüfregel gilt nur für eine andere Disziplin, Kartenart oder einen anderen Druckmaßstab.
- Ein Aufsatz hat die Prüfregel abgeschaltet; der Grund nennt ihn, etwa „Das Regelprofil nor-bb-2021 schaltet die
  Prüfregel ab“.
- Ein Wert fehlt. Das Startfenster setzt `event start-window halle.courseplan.toml 90`, die erwartete Starterzahl
  einer Kategorie `class set halle.courseplan.toml H21 --starters 40`, den Anstieg einer Bahn `course set
  halle.courseplan.toml A --climb 120`.
- Eine Kategorie hat keine Regelkategorie, zum Beispiel ein selbst gewählter Name wie „Offen“. `class set
  halle.courseplan.toml Offen --rule-category H21` ordnet ihr eine zu. Eine Regelkategorie, die in keiner Zeile
  der Tabelle steht, überspringt die Bahn ebenfalls.

Eine übersprungene Prüfregel hat nicht geprüft. Ein leeres Ergebnis heißt deshalb nicht, dass alles besteht. Lies
die Liste einmal durch, bevor du dich darauf verlässt.

### Eine Ausnahme begründen

Manche Befunde sind gewollt. Du akzeptierst sie mit der Kennung der Prüfregel, dem Ort und einer Begründung:

```sh
# zwei Posten, die bewusst nah beieinander stehen
freemapper-courseplan finding accept halle.courseplan.toml wkb2026.distance.min_half --controls 31,32 --note "Die Posten liegen an zwei Seiten desselben Gebäudes und gehören zu verschiedenen Bahnen."

# eine Strecke
freemapper-courseplan finding accept halle.courseplan.toml wkb2026.leg.acute_angle --leg A:3 --note "Der spitze Winkel ist gewollt, die Bahn wendet an diesem Posten."

# ein Posten und der Start
freemapper-courseplan finding accept halle.courseplan.toml wkb2026.distance.min_half --controls 31,S1 --note "Der Start liegt bewusst nah am ersten Posten, das Training beginnt mit einer kurzen Einlaufstrecke."
```

`--controls` nimmt Postencodes und die Kennung von Start oder Ziel, `--course BAHN` einen Befund der ganzen Bahn.
`--leg BAHN:N` nennt die Strecke, die von Position N der Bahn abgeht. Die Positionen zählen die Posten der Bahn ab
1, Querungsstellen mitgezählt, und 0 ist der Start. N ist also nicht die Postennummer auf der Karte: Auf Bahn A
mit den Posten 31 bis 37 ist `--leg A:6` die Strecke 36–37. Ein Befund an zwei Strecken bekommt `--leg` zweimal,
etwa `--leg A:2 --leg A:5`. Ein Befund der ganzen Veranstaltung braucht keine Ortsangabe. Gibst du einen Ort an, an
dem die Prüfregel nie etwas meldet, lehnt das Programm ab und nennt die Orte, an denen sie meldet. Akzeptierst du
denselben Befund noch einmal, ersetzt die neue Begründung die alte.

Die Begründung steht in der Projektdatei, bei einer Strecke zusammen mit ihren Enden. `check` zeigt akzeptierte
Befunde nicht mehr in der Liste, zählt sie aber in der Zusammenfassung. Mit `--all` stehen sie unter „akzeptiert“,
mit deiner Begründung. So sieht auch die Bahnlegerin im nächsten Jahr, warum die Bahn so ist.

Änderst du die Bahn, kann eine Ausnahme ihren Befund verlieren: Der Posten ist entfernt, die Bahn gelöscht, oder die
Strecke läuft jetzt zwischen anderen Posten. Im letzten Fall meldet `check` den Befund an der neuen Strecke wieder,
denn die alte Begründung galt für die alte Strecke. `check --all` listet solche Ausnahmen unter „nicht
zugeordnet“, die Zusammenfassung zählt sie, und am Exit-Code ändern sie nichts.

Willst du einen Befund wieder sehen oder eine nicht zugeordnete Ausnahme loswerden, nimmst du sie mit Prüfregel
und Ort zurück:

```sh
freemapper-courseplan finding remove halle.courseplan.toml wkb2026.distance.min_half --controls 31,32
freemapper-courseplan finding remove halle.courseplan.toml wkb2026.leg.acute_angle --leg A:3
```

`finding remove --leg` entfernt zuerst die Ausnahme für die Strecke, wie sie jetzt verläuft. Gibt es keine, entfernt
es die Ausnahmen an dieser Position, deren Strecke sich seitdem geändert hat.
Ändert sich die Kennung einer Prüfregel, etwa in einer neuen Programmversion, meldet `check` die Befunde dazu
wieder.

### Regelprofil und Aufsätze wählen

Eine Veranstaltung hat ein Basisprofil und beliebig viele Aufsätze darauf. Das Programm bringt zwei mit:
`wkb-2026` (Wettkampfbestimmungen Orientierungslauf 2026) und `nor-bb-2021`, einen Aufsatz mit den
Bahnempfehlungen von Berlin und Brandenburg. Beide setzt du mit einem Befehl:

```sh
freemapper-courseplan event profile halle.courseplan.toml wkb-2026 --overlay nor-bb-2021
```

Mehrere Aufsätze wiederholen `--overlay` und gelten in der angegebenen Reihenfolge. Das Programm lädt die Profile
vorher. Einen unbekannten Namen lehnt es mit der Liste der vorhandenen Profile ab. `info` zeigt das Profil an. In
jedem Befund von `check` beginnt die Kennung der Prüfregel mit ihrem Profil (`wkb2026`, `norbb2021`). Wie eine
Profildatei aufgebaut ist und wie man eine neue anlegt, steht in `crates/freemapper-courseplan/rules/README.md`.

### Mit dem KI-Assistenten

Der Assistent hat sechs Werkzeuge dafür: `check` liefert Befunde, akzeptierte und nicht zugeordnete Ausnahmen und
übersprungene Prüfregeln. `accept_finding` und `remove_accepted_finding` entsprechen `finding accept` und `finding
remove`, `set_rule_profile` entspricht `event profile`, `set_start_window` und `set_class` entsprechen `event
start-window` und `class set`. Bitte ihn zum Beispiel, die Bahnen zu prüfen, jeden Befund in einem Satz zu
erklären und dich zu fragen, bevor er einen akzeptiert. Die Begründung schreibst du oder diktierst sie ihm. Er soll
sie nicht erfinden.

## 9. Mit Purple Pen abgleichen

Der Weg hinein ist `import`; der Weg zurück ist `export --ppen` (weiter unten). Ein Abgleich, der Änderungen aus Purple Pen zurückholt, fehlt noch. `import` übernimmt ein Purple-Pen-Event (`.ppen`), das für dieselbe Karte gezeichnet
ist. Die Karte muss in OOM vorliegen und denselben Maßstab haben wie das Event.

```sh
freemapper-courseplan init karte.omap --ppen event.ppen
freemapper-courseplan import projekt.courseplan.toml --ppen event.ppen
```

`init --ppen` legt ein neues Projekt aus dem Event an. `import` füllt ein bestehendes. Hat das Projekt schon Posten,
Bahnen oder Sonderobjekte, bricht `import` ab. Mit `--replace` ersetzt es diese durch den Inhalt des Events;
Kategorien und akzeptierte Befunde bleiben. Der Pfad der `.ppen` steht danach relativ zur Projektdatei im Projekt.

Was ankommt: Posten mit Code, Lage und Beschreibung (nach dem Standard 2024 umnummeriert), Einzelbahnen mit
Knickpunkten und abgeflaggten Strecken, die Lage der Postennummern, Sperrgebiete, Gefahrenbereiche, verbotene
Routen, Erste-Hilfe- und Verpflegungsstellen. Purple Pen zählt die y-Achse nach oben; der Import dreht sie um.
Liegen die Posten dann nicht auf der Karte, bricht er ab. Ein Sonderobjekt mit einem Punkt außerhalb der Karte
entfällt mit einem Hinweis.

Was nicht ankommt, steht nach dem Import als Hinweis auf dem Terminal: Varianten und alternative Posten,
Score-Bahnen, Text in Spalte F der Beschreibung, Beschreibungen an Start und Ziel, Lücken in Kreisen und Strecken.
Auch Begrenzungen, Linien, Texte, Weißflächen und der Untertitel einer Bahn kommen nicht an. Ein Kartenwechsel wird zum Knickpunkt. Gleiche Bahnnamen bekommen eine Nummer. Der Abstand einer Nummer zum Kreis
ist nur ungefähr richtig, die Richtung stimmt. Ein Symbol ohne Gegenstück im Standard 2024, etwa die Funkstelle,
fehlt in der Beschreibung und steht im Hinweis.

Der KI-Assistent hat dafür das Werkzeug `import_ppen` mit `project`, `path` und `replace`. Die Entscheidungen
dahinter stehen in ADR 0034.

### Export nach Purple Pen

```sh
freemapper-courseplan export projekt.courseplan.toml --ppen event.ppen
freemapper-courseplan export projekt.courseplan.toml --ppen event.ppen --new
```

Der Export schreibt die Einzelbahnen als Purple-Pen-Event. Gibt es die Datei schon, ändert er sie an Ort und
Stelle: Was das Projekt nicht kennt, bleibt stehen, nämlich Lücken in Kreisen und Strecken, Kartenwechsel, Text
in Spalte F der Beschreibung und die Länge einer geänderten Bahn. Nur Zeilen, die sich unterscheiden, werden neu
geschrieben. Mit `--new` entsteht die Datei neu, und die alte geht verloren. Ohne vorhandene Datei legt der Export
sie an; die Beschreibungen folgen dann dem Standard 2018. Ein vorhandenes Event behält seinen Standard.

Der Export löscht nie. Steht ein Posten, eine Bahn oder ein Sonderobjekt in der Datei, aber nicht im Projekt, bleibt
es dort, und das Programm meldet es als behalten. Das Projekt kann nicht wissen, ob es dort gelöscht oder in Purple
Pen neu angelegt wurde. Das soll der spätere Abgleich entscheiden.

Die Purple-Pen-Kennungen bestimmen, welches Objekt welches ist. Gilt die Datei als Partnerdatei des Projekts
(`sources.ppen`), schickt der Export die gespeicherten Kennungen mit und speichert neue im Projekt. Das gilt auch,
wenn der Export die Datei neu anlegt oder mit `--new` ersetzt und das Projekt noch keine Partnerdatei hat; dann trägt
er sie ein. Hat das Projekt schon eine andere Partnerdatei, bleibt es dabei. Bei jedem anderen Ziel schreibt der Export
nichts ins Projekt zurück. Ohne Verknüpfung ordnet er Starts, Ziele und Querungsstellen nach ihrem Code zu, sonst nach
der Reihenfolge, wenn die Zahlen gleich sind (je Art ganz oder gar nicht).

Das Programm nennt auf dem Terminal, was es nicht oder anders schreibt:

- Mannschafts-, Staffel- und Score-Bahnen entfallen, jede mit ihrer Art.
- Ein Symbol ohne Gegenstück in Purple Pen fehlt in der Beschreibung.
- Ändern sich die Knickpunkte einer Strecke, verliert ihre teilweise abgeflaggte Strecke die Markierung.
- Ein geänderter Titel der Veranstaltung gelangt nicht in eine vorhandene Datei.
- Hat Purple Pen eine Strecke selbst abgeflaggt, die das Projekt nicht kennzeichnet, steht das als Hinweis da.

Als Ziel sind die Projektdatei, die Karte und die Zieldatei von `--iof-xml` im selben Aufruf nicht erlaubt. Eine Basisdatei
liest das Programm nur bis zur festen Größengrenze; eine kaputte oder fremde Datei gibt eine Fehlermeldung. Passt die
Datei nicht zur Karte, schreibt der Export nichts.

**Noch nicht in Purple Pen 4.0 geöffnet.** Die Datei wird mit dem Leser von freemapper-course wieder eingelesen, und der
Export einer eingelesenen Datei ergibt dieselben Bytes. Das Beispiel
[`luebeck-sprint.ppen`](../../examples/luebeck-sprint/luebeck-sprint.ppen) wartet auf einen Test mit Purple Pen 4.0.

Der KI-Assistent hat dafür das Werkzeug `export_ppen` mit `project`, `path` und `new`; der Pfad muss auf `.ppen`
enden. Der Schreiber liegt in `freemapper-course` (freemapper-core), die Entscheidungen stehen in ADR 0035.

## 10. Weitergeben: IOF-XML für OE

Mit `export` gibst du die Bahnen an Auswertungsprogramme weiter, die IOF-XML 3.0 lesen:

```sh
freemapper-courseplan export luebeck-sprint.courseplan.toml --iof-xml course-data.xml
```

Das Programm schreibt alle Einzelbahnen der Veranstaltung in die Datei: Posten mit Kartenposition, die
Postenfolge jeder Bahn, Streckenlängen, Anstieg, die Lage der Postennummern und die Zuordnung der Bahnen zu
Kategorien. Eine vorhandene Datei ersetzt es erst, wenn die neue vollständig geschrieben ist. Ohne `--iof-xml` tut
`export` nichts und endet mit Fehlercode 2.

Was du wissen solltest:

- Mannschafts-, Staffel- und Score-Bahnen schreibt das Programm nicht. Es nennt jede übersprungene Bahn mit
  ihrer Art.
- Nennst du als Zieldatei die Projektdatei oder die Karte, bricht `export` mit einem Fehler ab und lässt beide
  unberührt.
- Ist die Karte georeferenziert, tragen Start, Posten und Ziel Breite und Länge mit sechs Nachkommastellen. Fehlt die
  Georeferenzierung oder nutzt die Karte ein Koordinatensystem, das das Programm nicht kennt, enthält die Datei nur
  Kartenpositionen, und `export` sagt das. OE, OCAD und Condes sind mit diesen Positionen nicht getestet.
- Abgeflaggte Strecken fehlen noch. Die Kartenausdehnung steht als 0/0 mm in der Datei.
- Die Uhrzeit `createTime` ist UTC und trägt das Zeichen `Z`. Für die Bahnen spielt sie keine Rolle.
- Das Beispiel [`course-data.xml`](../../examples/luebeck-sprint/course-data.xml) ist gegen das IOF-Schema
  geprüft, und ein Test liest den Export mit dem IOF-Leser von freemapper-course (dem von freemapper-print) wieder
  ein. Ein Import in OE, Purple Pen oder Condes ist nicht getestet, weil im Projekt niemand OE hat. Wer die Datei
  dort importiert, soll melden, was ankommt.
- Purple Pen, OCAD und Condes schreiben die y-Achse nach oben, entgegen dem Standard. Dieser Export folgt dem
  Schema (y nach unten). Zeigt ein Programm die Posten gespiegelt, sag bitte, welches.

Der KI-Assistent hat dafür das Werkzeug `export_iof_xml` mit `project` und `path`; der Pfad muss auf `.xml` enden.
Die Entscheidungen dahinter stehen in ADR 0033.
