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
- Hat sich die Projektdatei geändert, nachdem der Assistent sie gelesen hat, etwa weil du im Editor einen Posten
  verschoben hast, schreibt das Programm nichts. Der Assistent bekommt „<Datei> hat sich auf der Platte geändert,
  nachdem es gelesen wurde. Es wurde nichts geschrieben. Lies das Projekt neu und wiederhole die Änderung“ und
  wiederholt die Änderung mit dem neuen Stand.
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

## 4. Der Editor (Vorschau)

Der Editor zeigt die Veranstaltung im Browser. Du kannst Posten setzen, verschieben und löschen, Bahnen und
Kategorien anlegen, Strecken knicken und abflaggen, Postennummern setzen, Sonderobjekte zeichnen und jeden Schritt
zurücknehmen. Postenbeschreibungen wählst du in den Eigenschaften eines Postens. Der Editor führt die Prüfung beim Arbeiten aus und zeigt ihre Befunde. Exportieren und Importieren findest du im
Menü „Veranstaltung“, ebenso das Abgleichen mit Purple Pen.

### Starten

```sh
freemapper-courseplan edit luebeck-sprint.courseplan.toml
```

Der Befehl lädt das Projekt, öffnet den Browser und gibt die Adresse in einer einzigen Zeile aus, etwa
`http://127.0.0.1:47123/3f9c…/`. Ist die Projektdatei kaputt, nennt er die Datei und endet mit Exit-Code 1, ohne
etwas auszugeben. Er läuft, bis du Strg+C drückst oder im Menü „Veranstaltung“ den Punkt „Editor schließen“ wählst.
Beide Wege beenden ihn mit Exit-Code 0.

- `--port N` wählt den Port. Ohne die Angabe nimmt das Programm einen freien. Ist der Port belegt, meldet es das und
  endet mit Exit-Code 1.
- `--no-browser` öffnet den Browser nicht. Du öffnest die ausgegebene Adresse selbst. Lässt sich der Browser nicht
  starten, schreibt das Programm einen Hinweis und läuft weiter.
- Die Sprache der Seite folgt der Sprache des Programms (`--lang de`, `FREEMAPPER_LANG` oder die Spracheinstellung
  des Systems). Die Seite hat keinen Sprachschalter; zum Wechseln startest du den Editor neu.

Ein Editor gehört zu einem Projekt. Willst du ein zweites öffnen, startest du einen zweiten Editor.

### Was die Seite zeigt

- Links steht der Baum der Veranstaltung mit den Gruppen Bahnen, Kategorien, Posten und Sonderobjekte. „Alle Posten“
  zeigt jeden Posten mit seinem Code.
- In der Mitte steht die Karte. Wählst du eine Bahn, zeichnet der Editor ihre Kreise, Startdreieck, Ziel, Strecken und
  Postennummern darüber. Nummernfelder mit gestrichelter Linie hat das Programm automatisch platziert; für sie ist
  noch keine Lage im Projekt gespeichert.
- Rechts stehen die Eigenschaften der Auswahl, etwa Name, Länge und Anstieg einer Bahn oder Code, Art, Position und
  Beschreibung eines Postens. Bei einem Posten nennt die Liste auch die Kartenobjekte im Umkreis von 3 mm. Darunter
  stehen die Befunde der Auswahl.
- Die Statusleiste unten nennt die Karte, den Maßstab, die Kartenart und die Mauszeigerposition in Kartenmillimetern.
  Meldungen erscheinen dort ebenfalls.

Die Karte ist in dieser Vorschau ein Platzhalter: ein Rahmen um die Kartenausdehnung, ein Gitter im Abstand von 10 mm
und ein Hinweistext. Kreise, Strecken und Nummern liegen an der richtigen Stelle der Karte, aber darunter ist noch
kein Karteninhalt zu sehen. Den liefert später freemapper-render.

### Posten

Die Werkzeugleiste hat die Werkzeuge „Auswählen“, „Posten setzen“, „Querungsstelle setzen“ und „Sonderobjekt“. Jede
Änderung wird sofort gespeichert, über dieselben Operationen wie in der Kommandozeile und beim KI-Assistenten; es
gelten also dieselben Prüfungen. Lehnt das Programm eine Änderung ab, etwa eine Position außerhalb der Karte, bleibt
die Datei, wie sie war, und eine Meldung nennt den Grund.

- **Auswählen.** Mit dem Werkzeug „Auswählen“ klickst du einen Posten in der Karte oder im Baum an. Ein Klick wählt
  den Posten, dessen Mittelpunkt dem Mauszeiger am nächsten liegt, solange der Zeiger höchstens den Kreisradius plus
  die Linienbreite entfernt ist. Rechts zeigen die Eigenschaften ihn an.
- **Setzen.** Wähle „Posten setzen“ und klicke auf die Karte. Der Editor legt an dieser Stelle einen Posten an.
- **Querungsstelle setzen.** „Querungsstelle setzen“ arbeitet genauso, legt aber einen Posten der Art
  „Querungsstelle“ an. Er bekommt den nächsten freien Code, teilt die Nummerierung mit den übrigen Posten und steht im
  Baum mit „(Querungsstelle)“. Auf der Karte erscheint er als zwei Bögen ohne Kreis. Die Eigenschaften zeigen Code, Art
  und Position, aber keine Postenbeschreibung, denn eine Querungsstelle hat keine. Zu einer Bahn fügst du sie im
  Bahnmodus hinzu wie jeden Posten.
- **Verschieben.** Ziehe einen Kreis, oder trage in den Eigenschaften neue Werte in die Felder X (mm) und Y (mm) ein.
  Du kannst Dezimalkomma oder Dezimalpunkt schreiben, `12,5` oder `12.5`. Alles andere markiert das Feld und zeigt
  einen Hinweis. Beim Ziehen bewegt sich nur der Kreis. Strecken und Nummern folgen, wenn du loslässt. Esc bricht das
  Ziehen ab, und der Posten bleibt, wo er war.
- **Schieben.** Bei gewähltem Posten verschieben ihn die Pfeiltasten um 0,1 mm, mit Umschalt um 1 mm. Der Editor
  speichert einmal, wenn du die Taste loslässt; eine gehaltene Taste ergibt also nur einen Schritt zum
  Rückgängigmachen. Ohne Auswahl verschieben die Pfeiltasten die Ansicht.
- **Code ändern.** Ändere das Feld „Kennzahl“ in den Eigenschaften. Die Bahnen und akzeptierten Befunde, die den
  Posten nennen, ziehen mit. Der Editor lehnt einen Code ab, der leer ist, Leerzeichen am Anfang oder Ende hat, länger
  als 64 Zeichen ist oder einem anderen Posten gehört. In der Kommandozeile geht dasselbe mit
  `freemapper-courseplan control code halle.courseplan.toml 31 K7` (siehe „Code oder Namen in der Kommandozeile
  ändern“ weiter unten).
- **Löschen.** Drücke Entf oder Rücktaste, solange der Fokus auf der Karte oder im Baum liegt, oder nimm „Posten
  löschen“ in den Eigenschaften. In einem Feld oder auf einer Schaltfläche tun die Tasten nichts. Einen Posten, den
  keine Bahn besucht, löscht der Editor sofort; mit Rückgängig holst du ihn zurück. Besuchen Bahnen den Posten, listet
  ein Dialog sie auf und fragt nach. Der Fokus liegt auf „Abbrechen“. „Löschen“ entfernt den Posten und nimmt ihn aus
  diesen Bahnen. Eine versehentliche Rücktaste löscht deshalb einen gewählten, unbenutzten Posten ohne Rückfrage;
  Rückgängig bringt ihn zurück.

### Bahnen und Kategorien

- **Neue Bahn.** Wähle unter „Bahnen“ im Baum „Neue Bahn...“ und tippe einen Namen. Die Bahn bekommt Start und Ziel der
  Veranstaltung. Hat die Veranstaltung nicht genau einen Start und ein Ziel, sagt der Editor das und legt nichts an.
  Einen Namen, den es schon gibt, der leer oder länger als 200 Zeichen ist, lehnt der Dialog ab.
- **Bahn umbenennen.** Ändere das Feld „Name“ in den Eigenschaften der Bahn. Sonderobjekte und akzeptierte Befunde,
  die die Bahn nennen, ziehen mit. In der Kommandozeile gilt `freemapper-courseplan course rename
  halle.courseplan.toml A Lang`.
- **Bahnmodus.** Wähle eine Bahn und dann „Bahnmodus“. Die Posten, die die Bahn nicht besucht, erscheinen grau. Ein
  Klick auf einen Posten hängt ihn vor dem Ziel an. Um einen Posten mitten in die Bahn einzufügen, klickst du zuerst
  die Strecke, an die er gehört, und dann den Posten. Die gewählte Strecke wird dick und blau, und die Statusleiste
  nennt sie. Esc verwirft die gewählte Strecke; ein zweites Esc verlässt den Bahnmodus. Der Bahnmodus hat keine Taste.
- **Posten der Bahn.** Die Eigenschaften einer Bahn listen ihre Posten. Mit „Nach oben“, „Nach unten“ und „Entfernen“
  änderst du die Reihenfolge. Liegt der Fokus auf einer Zeile, verschieben Alt+Pfeil hoch und Alt+Pfeil runter den
  Posten. Jede Änderung ist ein Schritt für Rückgängig und speichert die ganze Liste auf einmal. Läuft durch eine
  Änderung eine Strecke zu einem anderen Posten, verliert sie ihre Knicke und ihr Absperrband. Die Statusleiste nennt
  diese Strecken.
- **Bahndaten.** Steigung und geplanter Druckmaßstab sind Felder in den Eigenschaften; ein leeres Feld entfernt den
  Wert. Mit den Kontrollkästchen ordnest du die Bahn Kategorien zu.
- **Kategorien.** „Neue Kategorie...“ legt eine an. Ihre Eigenschaften enthalten die Regelkategorie und die
  erwarteten Starter; ein leeres Feld entfernt den Wert. „Kategorie löschen“ entfernt sie. Solange eine Bahn sie noch
  nutzt, lehnt der Editor das ab und sagt es über der Karte.
- **Veranstaltung.** „Einstellungen“ unter „Veranstaltung“ im Baum zeigen die Disziplin, die du hier nicht ändern
  kannst, das Regelprofil mit seinen Overlays, beides als Liste der Profile, die das Programm kennt, und das Startfenster
  in Minuten. Ein leeres Startfenster entfernt es.

### Strecken und Knicke

Eine Strecke ist die Linie zwischen zwei Posten. Klicke sie mit dem Auswahlwerkzeug an, um sie zu wählen.

- **Knick einfügen.** Ziehe an der Linie der gewählten Strecke. Dort, wo du loslässt, entsteht ein Knick.
- **Knick verschieben oder löschen.** Ziehe einen Knick, um ihn zu verschieben. Ein Klick wählt ihn, Entf löscht ihn.
- **Abgeflaggte Strecke.** Das Feld „Abgeflaggt“ in den Eigenschaften der Strecke markiert sie als abgeflaggte
  Strecke.

Jede Geste ist ein Schritt für Rückgängig, und die Bahnlänge in den Eigenschaften ändert sich mit. Läuft eine Strecke
nach einer Änderung der Posten zu einem anderen Posten, verliert sie ihre Knicke und ihr Absperrband.

### Nummern

Das Nummernfeld eines Postens steht neben seinem Kreis. Eine gestrichelte Linie heißt: Das Programm hat es
automatisch gesetzt, und das Projekt speichert noch keine Lage dafür.

- **Ziehen.** Ziehe im Bahnmodus ein Nummernfeld an eine andere Stelle. Beim Loslassen rechnet der Editor den Ort in
  Richtung und Abstand vom Kreis um und speichert beides für diese Bahn.
- **Für alle Bahnen.** Ist „für alle Bahnen“ angekreuzt, gilt die Lage am Posten für jede Bahn, die ihn nutzt. Hat die
  Bahn für die Nummer schon eine eigene Lage, geht sie der Lage am Posten vor. Das Feld springt dann zurück, und die
  Statusleiste sagt es.
- **Nummern platzieren.** „Nummern platzieren“ setzt die Nummern der Bahn automatisch und lässt gesetzte Nummern
  stehen. Mit „auch gesetzte Nummern ersetzen“ wird alles neu berechnet, nur die am Posten für alle Bahnen gesetzten
  bleiben.

Ein Ziehen ist ein Schritt für Rückgängig. Liegt der Abstand der Nummer außerhalb des Regelbereichs, steht am Nummernfeld ein
Warnzeichen (siehe „Befunde“).

### Postenbeschreibungen

Wähle einen Posten. Seine Eigenschaften führen die Spalten C bis H auf, jede zugeklappt und mit dem aktuellen Symbol.
Klappe eine Spalte auf, dann siehst du ihre Symbole als Raster aus Zeichen, jedes mit Nummer und Text in der Sprache
des Programms. Die Kartenobjekte im Umkreis von 3 mm stehen unter Spalte D.

- **Wählen.** Klicke ein Symbol an oder gehe mit den Pfeiltasten hin (Pos1 und Ende springen zum ersten und letzten)
  und drücke Enter. Tippst du eine Nummer wie `1.3`, springt die Auswahl dorthin. Das Symbol in Gebrauch ist
  markiert, und der Fokus bleibt auf der Wahl.
- **Was eine Spalte anbietet.** Spalte C hat die Symbole 0.1 bis 0.5 dafür, welches von mehreren gleichen Objekten
  gemeint ist, mit den Himmelsrichtungen 0.1N bis 0.2NW; die Anweisungszeilen bietet sie nicht an. Spalte D hat 1.1 bis 6.2, Spalte E die Symbole der Spalte D und 8.1 bis 8.11, Spalte F 10.1,
  10.2 und 11.1 (die Zahlensymbole 9.1 bis 9.4 bietet sie nicht an, weil die Projektdatei keinen Wert dafür hält), Spalte G die Symbole 12.x und Spalte H 13.1 bis 13.3.
- **Richtungen.** Ein Symbol mit Richtung gibt es nur mit einer: wähle `12.1N`, `12.1NE` und so weiter, ein
  einfaches `12.1` gibt es nicht.
- **Ein Symbol, das die Spalte nicht anbietet.** Hält die Kontrolle eines (ein altes oder von Hand geschriebenes), steht
  es oben als „aktueller Wert“ und bleibt, bis du etwas anderes wählst.
- **Kein Symbol.** Der Eintrag „Kein Symbol“ leert die Spalte.
- **Rückgängig.** Jede Wahl und jede geleerte Spalte ist ein Schritt für Rückgängig.

Die Liste kommt aus freemapper-core. Jedes Symbol, das du wählen kannst, lässt sich also auch in eine `.ppen`-Datei
schreiben (ADR 0051).

### Sonderobjekte

- **Zeichnen.** Wähle „Sonderobjekt“, dann eine Art (Start, Ziel, Erste-Hilfe-Posten, Wasserstelle, markierte
  Strecke, verbotene Route, Sperrgebiet, Gefahrenfläche) und prüfe die Kennung in der Werkzeugleiste. Sie wird als
  `<Art>-<n>` vorgeschlagen und lässt sich ändern. Start, Ziel, Erste-Hilfe-Posten und Wasserstelle setzt ein Klick.
  Eine Linie oder Fläche zeichnest du durch Klicken ihrer Punkte und schließt sie mit Enter oder einem Doppelklick ab.
  Esc bricht ab, Entf nimmt den letzten Punkt zurück, und eine gestrichelte Vorschau mit Punkten zeigt die Zeichnung.
  Eine Fläche braucht drei Punkte, eine markierte Strecke zwei. Eine verbotene Route darf einen einzigen Punkt haben;
  sie wird als Kreuz gezeichnet. Jedes Objekt ist ein Schritt für Rückgängig.
- **Zeichen in der Karte.** M markierte Strecke, Kreuz verbotene Route, OB Sperrgebiet, ! Gefahrenfläche, Plus Erste
  Hilfe, W Wasser.
- **Bahnen zuordnen.** Hake in den Eigenschaften eines gewählten Objekts die Bahnen an, zu denen es gehört. Ohne Haken
  gehört es zu allen Bahnen. Start und Ziel haben keine Bahnliste.
- **Löschen.** „Sonderobjekt löschen“ oder die Entf-Taste entfernt das Objekt, Rückgängig holt es zurück.
- **Kein Verschieben.** Der Editor kann ein Sonderobjekt nicht verschieben. Lösche es und zeichne es neu (ADR 0049).

### Befunde

Eine halbe Sekunde nach jeder Änderung führt der Editor dieselbe Prüfung aus wie `check` und zeigt das Ergebnis.

- Die Statusleiste zählt Fehler, Warnungen und Hinweise der ganzen Veranstaltung. Jede Zahl hat ein Zeichen (Kreuz,
  Dreieck, „i“) und eine Farbe. Ein Klick auf die Zahlen öffnet die Schublade: alle offenen Befunde, die akzeptierten
  mit ihren Begründungen und unter „Nicht geprüft“ die Regeln, die nicht liefen, mit dem Grund.
- Das Feld rechts listet die Befunde des gewählten Postens, der Strecke oder der Bahn. Ein Klick auf einen Befund wählt
  seine Stelle und zeigt sie in der Karte.
- In der Karte markiert ein Ring jeden Posten mit einem Befund und eine dicke gestrichelte Linie jede Strecke. Ein
  Warnzeichen am Nummernfeld heißt: Der Abstand liegt außerhalb des Regelbereichs. Die Zeichen unterscheiden sich auch
  in der Form nach Schwere.
- „Akzeptieren…“ fragt nach einer Begründung von höchstens 200 Zeichen und speichert die Ausnahme in der
  Projektdatei. Der Befund steht dann unter „Hier akzeptiert“ mit seiner Begründung, und „Akzeptanz zurücknehmen“
  holt ihn zurück. Beides ist ein Schritt für Rückgängig.

### Exportieren und Importieren

Das Menü „Veranstaltung“ hat vier Einträge. Der Browser nennt nie eine Datei; der Editor schreibt nur die
verknüpfte Purple-Pen-Datei des Projekts.

- **IOF-XML exportieren (Download).** Der Browser speichert das IOF-XML der Einzelbahnen, benannt nach der
  Projektdatei. Ein Fenster nennt ausgelassene Bahnen (Team-, Staffel- und Score-Bahnen) und sagt es, wenn die Karte
  keine Georeferenz hat.
- **Nach Purple Pen exportieren…** „In die verknüpfte Datei schreiben“ aktualisiert die Datei, die `sources.ppen`
  nennt, nach den Regeln von `export --ppen`: Sie ändert nur, was das Projekt geändert hat, und speichert neue
  Purple-Pen-IDs im Projekt. Ohne Verknüpfung ist die Schaltfläche aus. Der Befehl `export --ppen DATEI` legt eine
  Datei an und verknüpft sie. „Neue Datei herunterladen“ speichert eine neue `.ppen`; im Projekt wird nichts
  gespeichert. Ein Fenster listet die Hinweise.
- **Purple-Pen-Datei importieren…** Du wählst eine `.ppen` auf deinem Rechner. Ein Projekt, das schon Posten, Bahnen
  oder Sonderobjekte hat, wird abgelehnt, solange du „Ersetzen“ nicht ankreuzt; das ersetzt alle drei. Der Import
  ändert die Verknüpfung `sources.ppen` nicht: Der Editor erfährt nie, wo die Datei liegt. Ein Fenster listet die
  Hinweise. Dateien über 50 MiB werden abgelehnt (ADR 0053).
- **Mit Purple Pen abgleichen…** Nimmt die Datei aus `sources.ppen`, wie `sync`. Schließe die Veranstaltung vorher
  in Purple Pen; ein Hinweis sagt das und lässt sich für die Sitzung abschalten. Das Fenster listet Änderungen und
  Konflikte. Jeder Konflikt hat „Courseplan behalten“ und „Purple Pen übernehmen“; „Alle Courseplan“ und „Alle
  Purple Pen“ gleichen sofort ab und nehmen überall diese Seite. „Abgleichen“ ist möglich, wenn kein Konflikt offen
  ist (ADR 0054).

Das Schreiben in die verknüpfte Datei und der Import leeren den Rückgängig-Verlauf, ebenso ein Abgleich, der Dateien
geschrieben hat, weil sie mehr als einen Schritt ändern. Ein Abgleich, der nichts schrieb, lässt ihn bestehen.

### Rückgängig und Wiederholen

Die Schaltflächen „Rückgängig“ und „Wiederholen“ und die Tasten Strg/Cmd+Z und Strg/Cmd+Umschalt+Z (oder Strg+Y) gehen
die Schritte dieser Editor-Sitzung durch. Ein Setzen, ein Loslassen nach dem Ziehen, eine Feldänderung, ein Löschen
und eine Reihe von Pfeiltasten-Schritten sind je ein Schritt. Die Statusleiste meldet „Rückgängig gemacht“ oder
„Wiederholt“.

- Der Editor behält die letzten 100 Schritte, solange deren Dateikopien zusammen unter 64 MiB bleiben; die ältesten
  gehen zuerst. Schließt du den Editor, sind sie weg.
- Rückgängig und Wiederholen beziehen sich auf den Stand, den du siehst. Zeigt ein zweiter Browser-Tab einen älteren
  Stand als die Datei, lehnen sie ab, und der Tab lädt den aktuellen Stand; sie nehmen nie zurück, was der andere Tab
  getan hat, ohne dass du es gesehen hast.
- Rückgängig stellt die Datei so wieder her, wie sie vor dem Schritt war, samt Kommentaren und Aufbau. Das nimmt auch
  alles andere zurück, was der Schritt in der Datei geändert hat, etwa das Entfernen eines Postens aus einer Bahn.
- Hat ein anderes Programm die Datei nach deinem letzten Schritt geändert, lehnen Rückgängig und Wiederholen ab. Die
  Statusleiste meldet dann, dass die Projektdatei außerhalb des Editors geändert wurde und beides nicht mehr möglich
  ist; es wurde nichts geändert. Der Verlauf ist danach leer. Der Editor merkt das erst bei deinem nächsten Rückgängig
  oder Wiederholen, nicht im Moment der Änderung von außen.

### Tastenkürzel

Mit `?` oder der Schaltfläche „Tastenkürzel“ siehst du diese Liste in der Seite. Eine Taste, die du in ein Feld
tippst, gilt nie als Kürzel.

| Tasten | Wirkung |
| --- | --- |
| Strg/Cmd+Z | Rückgängig |
| Strg/Cmd+Umschalt+Z, Strg+Y | Wiederholen |
| Entf, Rücktaste | Gewählten Posten, Knick oder gewähltes Sonderobjekt löschen; beim Zeichnen den letzten Punkt |
| Esc | Ziehen oder Zeichnen abbrechen, gewählte Strecke verwerfen, Bahnmodus verlassen oder zum Auswahlwerkzeug zurückgehen |
| Alt+Pfeil hoch, Alt+Pfeil runter | Fokussierten Posten einer Bahn verschieben |
| Enter | Gerade gezeichnete Linie oder Fläche abschließen |
| V | Auswahlwerkzeug |
| C | Werkzeug „Posten setzen“ |
| S | Werkzeug „Sonderobjekt“ |
| Pfeiltasten | Gewählten Posten um 0,1 mm verschieben (ohne Auswahl: Ansicht verschieben) |
| Umschalt+Pfeiltasten | Gewählten Posten um 1 mm verschieben |
| `+`, `-` | Vergrößern, verkleinern (Strg oder Cmd mit diesen Tasten zoomt weiter den Browser) |
| `0` | Karte in das Fenster einpassen |
| `?` | Liste anzeigen |

### Maus und Baum

- Im Baum wählst du mit den Pfeiltasten, Pos1 und Ende einen Eintrag und bestätigst mit Eingabe.
- Mit der Maus verschiebst du die Ansicht, indem du auf leerer Fläche ziehst, und zoomst mit dem Mausrad.
- Mit der Tabulatortaste springst du vom Baum zur Karte, zu den Eigenschaften und zum Menü.

### Code oder Namen in der Kommandozeile ändern

```sh
freemapper-courseplan control code halle.courseplan.toml 31 K7
freemapper-courseplan course rename halle.courseplan.toml A Lang
```

`control code` gibt Posten 31 den Code K7. `course rename` benennt Bahn A in Lang um. Bahnen, Sonderobjekte und
akzeptierte Befunde, die den Posten oder die Bahn nennen, ziehen mit. Einen Code oder Namen, den es schon gibt, der
leer ist, Leerzeichen am Rand hat oder zu lang ist, lehnt das Programm mit einer Meldung ab, und die Datei bleibt,
wie sie war. Der KI-Assistent hat dafür die Werkzeuge `set_control_code` und `rename_course`.

### Änderungen von außerhalb des Editors

Ändert ein anderes Programm die Projektdatei, zum Beispiel `freemapper-courseplan control move` im Terminal oder der
KI-Assistent, lädt die Seite sie nach etwa zwei Sekunden neu und meldet das in der Statusleiste. Ein Tab im
Hintergrund fragt nicht nach, bis du ihn wieder öffnest. Ändert sich die Karte, lädt die Seite sie neu. Ändert sich
die verknüpfte `.ppen`, steht in der Statusleiste ein Hinweis.

Machst du in der Seite eine Änderung, während die Datei veraltet ist, etwa zwischen zwei Abfragen, lehnt der Editor sie
ab. Er lädt den aktuellen Stand, zeigt unten in der Karte die Meldung „Die Projektdatei wurde außerhalb des Editors
geändert. Der Editor hat den aktuellen Stand geladen; es wurde nichts geändert.“ und schreibt denselben Text in die
Statusleiste. Sieh nach, was das andere Programm getan hat, und mache deine Änderung noch einmal, wenn du sie weiter
willst. „Schließen“ blendet die Meldung aus.

Lässt sich die Projektdatei nicht laden, etwa weil jemand sie halb bearbeitet hat, lehnt der Editor jede Änderung mit
dem Ladefehler ab, bis die Datei wieder lesbar ist.

### Befehle, die die Datei verändert vorfinden

Der Editor ist nicht das einzige Programm, das die Projektdatei speichert. Jeder Befehl, der sie ändert, prüft
unmittelbar vor dem Speichern, ob die Datei noch die ist, die er gelesen hat. Ist sie es nicht, etwa weil der Editor
eine Verschiebung gespeichert hat, während der Befehl lief, schreibt der Befehl nichts, gibt

```
luebeck-sprint.courseplan.toml hat sich geändert, seit es gelesen wurde. Es wurde nichts geschrieben
```

aus und endet mit Exit-Code 1. Führe den Befehl noch einmal aus; er liest dann den neuen Stand. Die Prüfung zählt jede
Änderung an der Datei, auch ein Speichern im Texteditor, das am Ereignis nichts geändert hat. Der KI-Assistent bekommt
dieselbe Ablehnung in eigenen Worten (Kapitel 3). `sync` schreibt die `.ppen` vor der Projektdatei. Ändert sich die
Projektdatei dazwischen, meldet `sync` einen teilweisen Schreibvorgang, und ein zweiter Lauf macht ihn fertig
(Kapitel 9).

### Datenschutz und Sicherheit

- Der Server hört nur auf `127.0.0.1`. Andere Rechner im Netz erreichen ihn nicht.
- Die Adresse enthält ein zufälliges Zugriffszeichen (32 Hexziffern). Wer die vollständige Adresse kennt, kann auf
  deinem Rechner die Veranstaltung ansehen. Gib die Adresse deshalb nicht weiter.
- Das Zugriffszeichen steht auch in der Prozessliste, solange das Programm es dem Browser übergibt. Auf einem Rechner,
  den sich mehrere Leute teilen, kann ein anderer Benutzer es dort lesen. Starte den Editor dort mit `--no-browser`
  und öffne die Adresse selbst, oder arbeite an einem eigenen Rechner.
- Die Seite lädt nichts aus dem Netz. Der Server liest nur die Projektdatei, die Karte und die verknüpfte `.ppen`.
  Dateinamen nimmt er vom Browser nicht an.
- Schreibende Anfragen prüft der Server zusätzlich auf Herkunft und Inhaltstyp, damit eine fremde Webseite im selben
  Browser nichts auslösen kann. Der Editor ändert nur die Projektdatei, und nur über die Operationen der MCP-Werkzeuge.
Die Adresse einer Operation nennt keine Datei.

Die Entscheidungen dazu stehen in ADR 0042, 0045 und 0046.

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

Der Weg hinein ist `import`; der Weg zurück ist `export --ppen` (weiter unten). `sync` holt Änderungen aus Purple Pen zurück (weiter unten). `import` übernimmt ein Purple-Pen-Event (`.ppen`), das für dieselbe Karte gezeichnet
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
Pen neu angelegt wurde. Das entscheidet `sync` (weiter unten) mit Hilfe der Basiskopie.

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

**In Purple Pen 4.0 geöffnet.** Das Beispiel [`luebeck-sprint.ppen`](../../examples/luebeck-sprint/luebeck-sprint.ppen)
lässt sich dort ohne Fehler öffnen, und der Inhalt wirkt richtig (geprüft vom Autor, 10.11.2026). Außerdem wird die
Datei mit dem Leser von freemapper-course wieder eingelesen, und der Export einer eingelesenen Datei ergibt dieselben
Bytes.

Der KI-Assistent hat dafür das Werkzeug `export_ppen` mit `project`, `path` und `new`; der Pfad muss auf `.ppen`
enden. Der Schreiber liegt in `freemapper-course` (freemapper-core), die Entscheidungen stehen in ADR 0035.

### Mit Purple Pen abgleichen (`sync`)

```sh
freemapper-courseplan sync projekt.courseplan.toml --dry-run
freemapper-courseplan sync projekt.courseplan.toml
```

`sync` führt das Projekt und die verknüpfte `.ppen` in beide Richtungen zusammen. Das Projekt muss vorher verknüpft
sein, mit `export --ppen DATEI` oder `import --ppen DATEI`; ohne Verknüpfung bricht `sync` ab und sagt das. Starte
es jedes Mal, wenn du zwischen den beiden Programmen wechselst.

> **Warnung: Schließe das Event in Purple Pen, bevor du `sync` startest, und öffne es danach neu.** Purple Pen hält
> das Event im Speicher. Ist es noch offen, wenn `sync` die `.ppen` schreibt, verwirft sein nächstes Speichern alles,
> was `sync` hinzugefügt hat. Der folgende `sync` sieht diese Objekte dann als in Purple Pen gelöscht und löscht sie
> ohne Konflikt auch aus dem Projekt. Bist du unsicher, starte zuerst `--dry-run`; der schreibt nichts.

**Die Basiskopie.** `sync` vergleicht jedes Feld in drei Fassungen: die Datei, wie der letzte Abgleich sie
hinterlassen hat, die Datei jetzt und das Projekt jetzt. Die erste Fassung liegt als `X.courseplan.ppen-base` neben
der Projektdatei (zu `event.courseplan.toml` also `event.courseplan.ppen-base`). Ihre Endung ist nicht `.ppen`,
damit sie niemand aus Versehen in Purple Pen öffnet. Lege sie zusammen mit der Projektdatei in Git ab. `sync`
entfernt aus der Kopie das Attribut `absolute-path` der Karte, weil dort der Pfad deines Home-Ordners steht und der
Abgleich es nicht vergleicht. In der Projektdatei steht eine Tabelle `[sync]` mit der Verknüpfung und einem Hashwert
der Basiskopie.

**Erster Abgleich.** Es gibt noch keine Basiskopie. `sync` sagt das, löscht nichts und behandelt jeden Unterschied
als Konflikt. Am Ende schreibt es die Basiskopie. Dasselbe passiert, wenn die Basiskopie fehlt, zu einer anderen
Datei gehört oder nicht zu ihrem Hashwert passt; der Hinweis nennt den Grund.

**Probelauf.** `--dry-run` listet auf, was der Abgleich in jeder Richtung ändern würde und welche Konflikte offen
sind. Er schreibt nichts, auch keine Entscheidungen.

**Was zusammengeführt wird.** Jedes Feld für sich: bei einem Posten Code, Art, Position und jede Spalte der
Beschreibung; bei einer Bahn Name, Posten, Anstieg, Druckmaßstab und die Lage jeder Nummer; bei einer Strecke
Knickpunkte und Abflaggung; bei einem Sonderobjekt seine Form. Ein Feld, das nur eine Seite geändert hat, geht auf
die andere. Haben beide Seiten dieselbe Änderung gemacht, zählt sie einmal. Hast du im Projekt Posten 34 verschoben
und jemand in Purple Pen Posten 31, kommen beide Verschiebungen an. Auch ein Löschen ist eine Änderung: auf der einen
Seite gelöscht und auf der anderen unberührt heißt auf beiden gelöscht. Fügen zwei Leute an verschiedenen Stellen
einer Bahn verschiedene Posten ein, bleiben beide Einfügungen.

**Konflikte.** Ein Feld, das beide Seiten verschieden geändert haben, ist ein Konflikt. Ebenso, wenn eine Seite ein
Objekt löscht, das die andere bearbeitet hat. Löscht Purple Pen eine Bahn, auf der das Projekt Kategorien hat, ist
das auch ein Konflikt, weil die Kategorien mit verschwänden. `sync` gibt jeden Konflikt mit seinem Schlüssel und drei
Werten aus. Hier wurde Posten 34 im Projekt nach rechts und in Purple Pen nach links verschoben:

```text
Abgleich mit out.ppen.
Änderungen nach Purple Pen: 0
Änderungen ins Projekt: 0
Auf beiden Seiten gleich geändert: 0
control/34/position weicht ab.
Basis: -1.200, -12.400 mm
Projekt: -1.000, -12.400 mm
Purple Pen: -1.400, -12.400 mm
1 Konflikt ist noch offen. Es wurde nichts abgeglichen. Mit --take ours oder --take theirs wählst du für alle eine Seite, mit --take SCHLÜSSEL=ours oder --take SCHLÜSSEL=theirs für einen einzelnen Konflikt.
```

Solange ein Konflikt offen ist, schreibt `sync` nichts, auch nicht die Änderungen ohne Konflikt, und endet mit
Exitcode 1. Mit `--take` entscheidest du:

- `--take ours` behält für jeden offenen Konflikt den Wert des Projekts, `--take theirs` nimmt den der Purple-Pen-Datei.
  Es wird nichts gespeichert; es gilt nur für diesen Aufruf.
- `--take control/34/position=theirs` entscheidet einen Konflikt und darf mehrfach vorkommen. Bleiben andere Konflikte
  offen, steht die Entscheidung mit einem Fingerabdruck beider Werte in der Projektdatei unter
  `[[sync.decisions]]`. Sie gilt, solange die Werte gleich bleiben; ändert sich einer, ignoriert `sync` die
  Entscheidung und sagt das. Ein abgeschlossener Abgleich löscht alle gespeicherten Entscheidungen.

Die Schlüssel haben die Form `control/34/position`, `control/32/column/g`, `control/35` (der ganze Posten),
`course/A/stops`, `course/A/number/31`, `leg/31-32` oder `special/WATER1`. Mit `sync --dry-run` siehst du, welche
offen sind.

**Was geschrieben wird.** Ist kein Konflikt offen, schreibt `sync` in dieser Reihenfolge: die `.ppen`, die
Projektdatei, die Basiskopie. Kurz vor dem ersten Schreiben liest es die `.ppen` noch einmal. Hat Purple Pen
inzwischen gespeichert, wird nichts geschrieben, und du startest `sync` erneut. Bricht ein Absturz oder eine volle
Platte den Abgleich dazwischen ab, beendet der nächste `sync` die Arbeit: Änderungen, die beide Seiten schon haben,
sind keine Konflikte, und eine Basiskopie, die laut Projektdatei veraltet ist, wird repariert. Ein Posten bleibt in
der `.ppen`, wenn eine Bahn dort ihn noch anläuft, auch wenn das Projekt ihn gelöscht hat; ein Hinweis sagt das.

**Was nur in courseplan bleibt.** Kategorien, erwartete Starter, akzeptierte Befunde, Druckbereiche, Mannschafts-,
Staffel- und Score-Bahnen, abgeflaggte Strecken als Sonderobjekte und verbotene Routen mit mehreren Punkten kommen nie
nach Purple Pen, und `sync` rührt sie nicht an. Ein Hinweis nennt jede Mannschafts-, Staffel- und Score-Bahn, zum
Beispiel `Hinweis: nicht nach Purple Pen geschrieben: T ausgelassen: Bahnart Mannschaftsbahn wird nicht nach Purple
Pen exportiert`. Umgekehrt bleibt in der `.ppen` unverändert, was der Import auslässt, etwa Varianten und Lücken in
Kreisen. Ein in Purple Pen umbenannter Posten oder eine umbenannte Bahn wird in akzeptierten Befunden, Strecken und
den Bahnlisten der Sonderobjekte nachgezogen. Lässt Purple Pen eine Nummernlage automatisch oder setzt sie auf
automatisch zurück, bleibt sie auch im Projekt automatisch. Entfernt eine Seite eine Strecke, deren Verlauf die andere
ändert, gewinnt das Entfernen, und ein Hinweis sagt das.

**Ablehnungen.** `sync` lehnt ab und schreibt nichts, wenn die `.ppen` aus einer unbekannten Purple-Pen-Version
stammt, wenn die verknüpfte Datei fehlt oder wenn der Schreiber ein Objekt der Datei dem zusammengeführten Projekt
nicht zuordnen konnte. Der letzte Fall kann eintreten, wenn du in einem Abgleich zwei Postencodes vertauschst und
eine Bahn umbenennst; mach das in zwei Schritten.

Der KI-Assistent hat zwei Werkzeuge. `sync_ppen` nimmt `project`, `dry_run` und `take` (`ours` oder `theirs` für den
ganzen Aufruf) und gibt `written`, `conflicts`, `changes` und `notes` zurück. `resolve_sync_conflict` nimmt
`project`, `key` und `take`, speichert eine Entscheidung und gibt zurück, was noch offen ist; ist nichts mehr
offen, schließt `sync_ppen` den Abgleich ab. `sync --json` gibt dieselben Felder aus. Die Entscheidungen dahinter
stehen in ADR 0040.

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
