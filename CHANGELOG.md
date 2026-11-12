# Änderungen

[English version](CHANGELOG.en.md) · [Handbuch](docs/manual/de.md)

Hier stehen alle wichtigen Änderungen an freemapper-courseplan, die neueste zuerst. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/).

## [Unveröffentlicht]

### Hinzugefügt

- `check` meldet einen Posten, der in einem Sperrgebiet steht, als Fehler. Die neuen Regeln `wkb2026.control.in_forbidden` (Waldkarten) und `wkb2026.control.in_forbidden_sprint` (Sprint- und Schulhofkarten) nennen als Quelle WKB26 A 2.2. Als Sperrgebiet gilt ein Sonderobjekt der Art Sperrgebiet, das zur Bahn des Postens gehört oder keine Bahn nennt, und jede Kartenfläche mit dem Symbol 520 oder 709, auf Sprint- und Schulhofkarten auch 714. Ein Posten auf dem Rand steht im Gebiet. Start, Ziel, Querungsstellen und Posten ohne Bahn prüft die Regel nicht. Siehe ADR 0081.

### Geändert

- `check` prüft die gerade Linie zwischen zwei Posten nicht mehr gegen Sperrgebiete. Die Warnung „Die gerade Strecke quert das Sperrgebiet …“ entfällt, denn WKB26 B 7.2.6 betrifft die Laufroute, und eine Luftlinie über ein Sperrgebiet ist normal. Die Regeln `wkb2026.leg.through_forbidden` und `wkb2026.leg.through_forbidden_sprint` bleiben im Regelprofil und stehen jetzt in `check --all` unter den nicht geprüften Regeln, bis es die Routenanalyse gibt. Hast du einen Befund dieser Regeln akzeptiert, bleibt die Ausnahme gültig und erscheint höchstens unter den Ausnahmen ohne Befund. Siehe ADR 0080.
- Das Beispiel „Rummelsburger Bucht“ hat jetzt fünf Posten und nur gerade Strecken, ohne Knickpunkte und ohne markierte Strecke. Eine Strecke ist die Luftlinie von Posten zu Posten, Knickpunkte sind die Ausnahme. IOF-XML, Purple-Pen-Datei, Ausgaben und beide READMEs folgen der neuen Bahn.
- Die Bildschirmfotos im Handbuch zeigen die neue Bahn. Das Bild zu den Knicken hat Knicke nur zur Anschauung, die Bildunterschrift sagt das. Die Befundbilder zeigen drei Winkelhinweise, die durch verschobene Posten in einer Arbeitskopie entstanden. Der Abschnitt „Strecken und Knicke“ sagt jetzt, dass eine Strecke gerade ist, bis du sie knickst.

## [0.14.1] - 2026-11-12

### Hinzugefügt

- Bildschirmfotos im Handbuch, Kapitel 4. Je Sprache vierzehn Bilder zeigen den Editor auf den Beispielkarten von Remo Madella (Rummelsburger Bucht für die Sprintbahn, Gleisdreieck für die Score-Bahn): die ganze Seite, einen Posten, eine Bahn mit ihren Kategorien, eine abgeflaggte Strecke mit Knicken, Postennummern, Postenbeschreibungen, ein Sonderobjekt, Befunde, das Menü „Veranstaltung“, den Dialog zum Export nach Purple Pen und das Score-Fenster mit Kandidaten, einem Entwurf und der übernommenen Bahn. Das deutsche Handbuch zeigt die deutsche Oberfläche, das englische die englische. Jede Bildunterschrift nennt die Karte und verweist auf die `ATTRIBUTION.txt` des Beispiels.

### Behoben

- Editor: Wählst du eine Strecke, zeigt „Länge“ jetzt die Länge dieser Strecke. Bisher stand dort die Länge der ganzen Bahn.
- Editor, Score-Fenster: Die Kandidatentabelle passt in das Fenster, die Spalte „Übernehmen“ ist nicht mehr abgeschnitten. Der Hinweis „Kein freier Code“ erscheint nur noch, wenn gar kein Kandidat einen Code hat. Hat nur der Rest der Liste keinen, steht das an den Zeilen selbst.
- Editor: Der Dialog für eine neue Score-Bahn heißt „Neue Score-Bahn“, nicht mehr „Neue Bahn“.
- Editor: Die Statusleiste bleibt einzeilig. Lange Meldungen enden mit „…“ und stehen ganz im Tooltip, die Fläche darüber springt nicht mehr.
- Editor: Die Befundeliste deckt die Fläche von Eigenschaften und Karte ganz ab. Oben blieb ein Streifen des Eigenschaftenfensters sichtbar.
- `edit`: Der Server liest bei Rückgängig, Wiederholen und Beenden auch den Anfrage-Rumpf, bevor er antwortet. Sonst konnte der Client unter macOS ab und zu „Connection reset by peer“ statt der Antwort sehen (ADR 0079).

## [0.14.0] - 2026-11-12

### Hinzugefügt

- Selbstaktualisierung (ADR 0076). `freemapper-courseplan update` ersetzt das Programm nach Rückfrage an Ort und Stelle durch die neueste signierte Version (`--check` endet mit 10, wenn es eine neuere Version gibt, `--yes`, `--version X.Y.Z`, `--rollback`, `--mode notify|install|off`). Die Version wird mit minisign gegen einen im Programm eingebauten Schlüssel geprüft, dann per SHA-256 des Downloads und der entpackten Datei, und die neue Datei muss auf `--version` antworten, bevor etwas ausgetauscht wird. Die alte Datei bleibt als `freemapper-courseplan.previous`. Ein Release-Build sucht höchstens einmal am Tag nach einer neueren Version, nach `info` und beim Start des Editors, und gibt eine Zeile auf stderr aus; der Editor zeigt sie in der Statusleiste. Die MCP-Werkzeuge `check_update` und `install_update` (nur mit `confirm: true`) sagen es dem Assistenten, der dich vorher fragt; das erste Werkzeugergebnis trägt einmal einen Hinweis. Aus dem Quellcode gebaute Programme aktualisieren sich nie. Das Programm sendet nur einen festen User-Agent an die GitHub-Server der Releases. Der Release-Workflow signiert jetzt ein Manifest mit den Secrets `MINISIGN_SECRET_KEY` und `MINISIGN_PASSWORD` der Umgebung `release`; die Umgebung hat einen Pflicht-Reviewer, ein Release fragt also einmal nach Freigabe, bevor er signiert und veröffentlicht wird. Von Hand entpacken bleibt der Weg zur ersten Version mit der Aktualisierung. Handbuch, Kapitel 2.
- Beispiele auf zwei Karten von Remo Madella (MOO Challenge Berlin 2025). Er hat sie am 12.11.2026 für die freemapper-Suite freigegeben; das ist keine offene Lizenz (ADR 0077). `examples/gleisdreieck` enthält eine Score-Bahn von 33 Minuten (27 Posten, 83 Punkte; der gute Läufer holt 92,8 Prozent, der sehr gute ist nach 96,1 Prozent des Limits im Ziel), `examples/rummelsburger-bucht` eine Sprintbahn mit vier Posten, Knickpunkten, einer markierten Strecke, einem Erste-Hilfe-Posten und drei akzeptierten Hinweisen, mit IOF-XML und Purple-Pen-Export. Beide haben Abläufe und Prüfausgabe auf Deutsch und Englisch, und jeder Ordner hat eine `ATTRIBUTION.txt`. 45 Minuten gelingen auf keiner der beiden Karten; die READMEs nennen die Versuche und die tatsächlichen Grenzen. Die Purple-Pen-Datei des neuen Beispiels ist in Purple Pen nicht geöffnet worden.
- Breiterer Vorrat an Postenstellen für Score-Bahnen (ADR 0073). `score suggest` nutzt jetzt auch die freien Enden von Mauern, Zäunen, Hecken, Stufenlinien, Treppen und Brücken, Wegenden und Wegegabelungen, Knicke und Ränder befestigter Flächen, kleine Gebäude, Überdachungen, Ufer und weitere Punktsymbole, jeweils mit einer Klarheit, die die Schwierigkeit erhöht und nie eine Stelle entfernt. Eine geschlossene Linie liefert alle ihre Ecken. Eine Stelle, die sich nicht anlaufen lässt, entfernt nicht mehr alle Stellen im Umkreis von 100 m, sondern nur noch die innerhalb von 30 m. Auf der Lübecker Karte findet `suggest` 31 statt 19 Stellen, auf einer Karte von Berlin-Kreuzberg (1,8 x 2,0 km) 183 statt 137. Beide Karten bestehen überwiegend aus Gebäuden, der Gewinn ist also mäßig; auf der Kreuzberger Karte endet der Entwurf bei 13 Posten für 30 und 26 bis 28 für 45 Minuten, begrenzt nicht mehr vom Vorrat, sondern von der Zeit des sehr guten Läufers.
- Vier Abstandseinstellungen für Score-Bahnen, in Metern im Gelände: `value1_min_m` (150) hält jeden Posten von Start und Ziel fern, Wert 1 beginnt also dort; `min_gap_m` (100, nicht kleiner) ist der kleinste Abstand zweier Posten und ersetzt den vom Maßstab abgeleiteten Standard von `score suggest`; `max_reach_m` begrenzt den Abstand vom Start (automatisch ein Viertel der Strecke, die der gute Läufer im Zeitlimit schafft); `high_value_gap_m` (200) ist der kleinste Abstand zweier hochwertiger Posten, also der Werte über 0,6 des höchsten. `score settings` (`--value1-min`, `--min-gap`, `--max-reach`, `--auto-reach`, `--high-value-gap`), das Werkzeug `set_score_settings` und das Score-Fenster des Editors setzen sie; ältere Projektdateien laden mit den Standardwerten. Der Entwurf hält den Abstand hochwertiger Posten ein, und `check` warnt, wenn eine Score-Bahn einen der Abstände unterschreitet (`score_min_gap`, `score_high_gap`; ADR 0072).

### Behoben

- `score suggest` und `score draft` nehmen die Ecken eines Kartenrahmens nicht mehr als Postenstellen (ADR 0078). Die Karte Gleisdreieck hat ein geschlossenes Rechteck aus Symbol 501.1 um den Karteninhalt, und ein Entwurf hatte dessen vier Ecken als Posten.
- `score draft` und `score length` (und `check`) stimmen jetzt über eine Postenmenge überein. Dieselbe Streckenzeit kam je nach Suche, die sie berechnet hat, in den letzten Stellen leicht verschieden heraus, und die Tourensuche machte daraus eine andere Tour: Beim Start 40,-40 des Berliner Beispiels versprach der Entwurf 97 Punkte und 99,8 Prozent, die übernommene Bahn ergab 96 Punkte und 100,7 Prozent des Limits. Streckenzeiten werden jetzt dort, wo sie entstehen, auf eine Millisekunde gerundet (ADR 0075).
- `score draft` findet eine Menge, die beide Bänder trifft, wo es sie gibt, und läuft schneller (ADR 0074). Nach der Präfix-Suche sucht der Entwurf jetzt lokal: Er tauscht, ergänzt oder streicht einen Posten oder verschiebt einen Wert um bis zu zwei Stufen gegen seinen Quantilwert, mit höchstens 150 Tourbewertungen und dem Seed der Einstellungen. Auf einer Karte von Berlin-Kreuzberg (1,8 x 2,0 km) trifft jetzt jeder Start und jedes Limit, die wir versucht haben (30 und 45 Minuten, drei Starts), beide Bänder; vorher lag der gute Läufer jedes Mal 0,3 bis 3 Punkte über seinem Band. Die Werte eines Entwurfs sind deshalb nicht immer genaue Quantile. Ein zu kleiner Vorrat scheitert weiterhin: Auf der Lübecker Karte erreicht der gute Läufer bei 30 und 45 Minuten alle Punkte.
- Schnellere Routensuche. Die Kürzeste-Wege-Suche nutzt A*, eine Suche je Posten liest seine Wege zu allen anderen, die Routenwahl-Suchen von `score suggest` und `score draft` laufen auf mehreren Threads, und der Or-opt-Schritt der Tour-Heuristik kopiert die Tour nicht mehr. Auf der Kreuzberger Karte sank die CPU-Zeit von `score draft` von 44 bis 63 s auf 24 bis 35 s; der Rest ist überwiegend die Routensuche von `score suggest` (ADR 0074).
- Score-Vorschläge liegen seltener am Kartenrand und näher am Start: Die Auswahl setzt Stellen nahe dem Start dichter und wertet Stellen am Rand ab. Im Lübecker Beispiel sanken die Kandidaten bis 48 m vom Rand von 21 auf 3 von 60, und die Posten mit Wert 1 rückten von 360 m auf 230 m an den Start (ADR 0071).

### Geändert

- Die Dokumentationsbeispiele `examples/luebeck-sprint` und `examples/berlin-kreuzberg` entfallen, ersetzt durch die beiden Beispiele oben. Das Lübecker Projekt, seine Exporte nach Purple Pen und IOF-XML und sein Score-Projekt bleiben als Testdaten in `tests/fixtures/luebeck-sprint/`. Der Test `berlin_score_example_meets_its_bands_and_check_is_quiet` heißt jetzt `gleisdreieck_score_example_meets_its_bands_and_check_is_quiet`.

## [0.13.0] - 2026-11-11

### Hinzugefügt

- Das Lübecker Beispiel hat in einer zweiten Projektdatei, `luebeck-score.courseplan.toml`, eine Score-Bahn: 9 Posten, 25 Punkte, 15 Minuten, aus `score draft`, mit Transkript, Ausgabe von `check`, IOF-XML, CSV-Liste und Auswertungsbogen auf Deutsch und Englisch. Ein Entwurf mit 45 Minuten scheitert, weil die freien Codes ausgehen (30 Posten beim höchsten Wert 5, 54 bei 9), nicht weil die Karte klein ist; die README sagt das.
- `score suggest` nimmt `--course NAME` (Werkzeugparameter `course`): Start und Ziel dieser Score-Bahn. Der Vorschlag im Editor schickt die gewählte Bahn; vorher benutzte er den ersten Start des Events, und der kann in einem Gebäude liegen.
- Ein gescheiterter Entwurf, dessen Postenzahl durch die freien Codes begrenzt ist, sagt das (Hinweis „höchsten Wert erhöhen oder Codes freigeben“), statt die Karte verantwortlich zu machen.
- `info` nennt Posten, Punkte und Zeitlimit einer Score-Bahn. Bisher stand dort, die Länge werde nicht unterstützt.
- Score-Bahnen in den Exporten. Das IOF-XML enthält jetzt Score-Bahnen mit dem Wert jedes Postens in `CourseControl/Score`; Zeitlimit, Abzug und Pflichtverbindungen stehen in einem `Extensions`-Element, weil das Schema dafür keinen Platz hat. Neu: `export --csv DATEI --course NAME` schreibt Code, Wert und Position der Posten, `export --sheet DATEI --course NAME` einen Auswertungsbogen mit Zeitlimit, Abzug, der Regel, dass bei gleicher Punktzahl gewinnt, wer schneller ist, und der Regel, dass disqualifiziert wird, wer eine Verbindung auslässt (Werkzeuge `export_score_csv`, `export_score_sheet`). Der Purple-Pen-Export lässt Score-Bahnen weiter aus und sagt es. Das IOF-XML ist in OE nicht erprobt (ADR 0070).
- Der Editor zeichnet Score-Bahnen und hat ein Score-Fenster: ein Vorschlag mit Kandidatenmarken und ihren Werten, „Übernehmen“ für einen einzelnen Kandidaten, ein Entwurf mit Längenbericht, „Entwurf übernehmen“ und eine Fortschrittszeile, solange das Programm rechnet. Pflichtverbindungen erscheinen als gestrichelte Linien zwischen ihren Posten. Kandidaten und Entwürfe sind nicht gespeichert, bis du sie übernimmst. Ein langer Lauf hindert den Editor nicht mehr daran, Änderungen an der Datei zu bemerken (ADR 0069).
- `score suggest`, `score draft` und `score length` (Werkzeuge `score_suggest`, `score_draft`, `score_length`): nach Routenwahl geordnete Postenplätze aus der Karte, ein Entwurf einer Score-Bahn, dessen Länge die Bänder beider Läuferprofile trifft, und die Längenanalyse einer vorhandenen Bahn. Nur `draft --apply` schreibt; es legt die Bahn an oder ersetzt die Posten einer Score-Bahn, und der Editor kann es rückgängig machen. Ein Entwurf, der ein Band verfehlt, endet mit 1 und nennt jede Abweichung und einen Hinweis. Postenplätze halten 4 m Abstand zu gesperrten Wegen (ADR 0068).
- Score-Bahnen bearbeiten, in der Kommandozeile, über MCP und in den Operationen des Editors: `course add --kind score --time-limit N`, `score controls`, `score value`, `score link add|remove`, `score settings` und `score course set`, dazu die Werkzeuge `add_score_course`, `set_score_controls`, `set_control_value`, `add_score_link`, `remove_score_link`, `set_score_settings` und `set_score_course_details`. Der Code bestimmt den Wert eines Postens, ein abweichender Wert wird abgelehnt. Ohne Abzug speichert eine Score-Bahn keinen und nimmt den Standard (höchster Postenwert, mindestens `value_max`); `--no-penalty` kehrt dorthin zurück. Wer den Code eines Postens ändert, gibt ihm den Wert des neuen Codes. `rename`, `remove`, Kategorien und Druckmaßstab gelten auch für Score-Bahnen; Strecken, Knicke und Nummern bleiben bei Einzelbahnen (ADR 0067).
- Score-Bahnen: Das Projekt kennt jetzt den Abschnitt `[score]` mit Wertebereich, Läuferprofilen und Seed, und eine Score-Bahn kann Pflichtverbindungen zwischen zwei Posten nennen. Die Prüfung der Projektdatei lehnt unbekannte Posten, doppelte Verbindungen und verzweigte Ketten ab (ADR 0060).
- `check` prüft jetzt Score-Bahnen: Wert im Code, Codes, Ring und Reihenfolge der Posten, Länge, Zeitlimit, Abzug, Pflichtverbindungen und Mischung. Mannschafts- und Staffelbahnen bleiben übersprungen (ADR 0066).

### Geändert

- Korrekturen an den Score-Bahnen vor der ersten Veröffentlichung: Ein vorgeschlagener Posten hat in den Touren immer eine Route, ein Längenbericht sagt nie „inf Prozent“, die Geschwindigkeitstabelle behandelt Wald 405 als gut laufbar (1,1) und den Bahnhelfer 515.1 nicht als Mauer, und `score suggest` zeigt Routenwahl und Schwierigkeit mit zwei Nachkommastellen und Sekunden mit einer. `score settings --value-max` lehnt einen Wert unter dem eines vorhandenen Postens ab.
- Die Werkzeugbeschreibungen von `check` und `export_iof_xml` sagen nicht mehr, Score-Bahnen würden nicht geprüft oder geschrieben. README, Handbuch, Glossar, Entwurf und AGENTS.md beschreiben Score-Bahnen als gebaut.
- Intern: Kartenart, Sidecar-Leser und Objektklasse kommen aus `freemapper-core` v0.19.0 (`freemapper-contract`) statt aus eigenen Kopien. Namen und Ausgaben bleiben gleich.

## [0.12.0] - 2026-11-11

### Hinzugefügt

- `list controls|courses|specials|classes` führen die Einzelheiten eines Projekts als Tabelle auf, `map extent` und `map near --at X,Y --radius MM` nennen den Umfang der Karte und die Kartenobjekte nahe einem Punkt. Mit `--json` geben sie die Antwort des gleichnamigen MCP-Werkzeugs unverändert aus (ADR 0059). `event start-window --none` entfernt das Startfenster, wie es `set_start_window` mit `no_start_window` tut.

### Geändert

- `AGENTS.md` nennt die Befehlszeile zuerst und MCP danach. Ein Assistent, der Befehle ausführen kann, kommt mit weniger Zügen aus; MCP bleibt für Assistenten ohne Shell wie Claude Desktop. Der Abschnitt zur Befehlszeile verweist auf `--help`, nennt Ausgabe und Exit-Codes, sagt, dass `edit` blockiert, und führt auf, was die Befehlszeile gegenüber den MCP-Werkzeugen nicht kann.

## [0.11.1] - 2026-11-11

### Geändert

- freemapper-core 0.16.1. Die gemeinsamen Platzierungsfälle nutzen das Nummernfeld des Renderers (0,8083747 × Höhe × Ziffern) und laufen ohne Ausnahmen durch.
- Bei Maßstäben, die die Norm nicht vorsieht (etwa Sprint 1:5000), behalten die Bahnsymbole ihre Grundgröße, wie in freemapper-print (ADR 0058). Overlay, Druckvorschau, Platzierung und Prüfungen messen mit demselben Faktor; vorher schrumpften die Kreise dort auf 4,8 mm. Die Prüfung meldet für solche Karten, dass die Bahn nur fürs Training taugt.

## [0.11.0] - 2026-11-11

### Hinzugefügt

- Der Editor hat eine Druckvorschau (ADR 0057). Die Schaltfläche „Druckvorschau“ oder die Taste P zeigt die gewählte Bahn so, wie freemapper-print sie zeichnet: Der Server gibt die Bahn an den Renderer (freemapper-core 0.16.0), der Start, Posten, Ziel, Strecken, Sonderobjekte und Nummern als Purpurebenen ins Kartenbild malt. Das Overlay ist dabei ausgeblendet, und es lässt sich nichts bearbeiten: Baum, Eigenschaften, Rückgängig und Wiederholen und die Tasten sind abgeschaltet. Ein Test vergleicht Kreise, Strecken, Bandstrecken, Querungsstellen und Nummernfelder des Overlays mit denen des Renderers, für Sprint-, Schulhof- und Waldgrößen.

### Behoben

- Das Nummernfeld ist so breit, wie freemapper-print es zeichnet (3,23 mm statt 2,4 mm für eine Ziffer, ADR 0030). Gespeicherte Nummernpositionen (Richtung und Abstand) werden um etwa 0,4 mm anders gezeichnet, und die automatische Platzierung, die Prüfungen und die Positionen in den Exporten für Purple Pen und IOF XML nutzen die neue Breite. Die Purple-Pen-Datei und das IOF XML des Beispiels sind neu erzeugt.
- Die Querungsstelle im Overlay ist das 3 mm große Zeichen, entlang der Bahn gedreht, und eine Bandstrecke hat die Striche des Drucks. Eine Karte ohne Maßstab ergibt keine unendlichen Zeichengrößen mehr.
- Das Overlay zeichnet Start und Ziel in den IOF-Größen und lässt Strecken dort enden, wo der Renderer sie enden lässt: am Eck des Starts, am äußeren Zielkreis, am Postenkreis, an der Querungsstelle selbst. Vorher schnitt es jede Strecke an beiden Enden mit dem Radius des Postenkreises ab.

## [0.10.0] - 2026-11-11

### Hinzugefügt

- `AGENTS.md`, eine Anleitung für KI-Agenten: was das Programm kann und was nicht, Installation aus den Release-Archiven, Eintragen über MCP (jedes Werkzeug mit seinen Parametern), Kommandozeile, Fehlerbehebung. Die Datei liegt in jedem Release-Archiv. Die READMEs und Kapitel 3 des Handbuchs erklären, wie ein Assistent, der Befehle ausführen kann, das Programm installiert und einträgt.
- Der Editor: `freemapper-courseplan edit <Projekt>` zeigt die Veranstaltung im Browser (ADR 0042, 0045, 0046). Links steht der Baum aus Bahnen, Kategorien, Posten und Sonderobjekten. In der Mitte zeigt die Ansicht die gewählte Bahn mit Kreisen, Strecken und Nummern über der Karte, rechts stehen die Eigenschaften. Die Statusleiste steht unten.

- Der Editor kann Posten setzen, verschieben und löschen und jeden Schritt rückgängig machen und wiederholen (ADR 0043, 0047). Mit dem Werkzeug „Posten setzen“ klickst du einen Posten auf die Karte, ziehst einen Kreis oder änderst X und Y in den Eigenschaften (Komma oder Punkt geht), um ihn zu verschieben, und löschst ihn mit Entf oder Rücktaste. Die Pfeiltasten verschieben den gewählten Posten um 0,1 mm, mit Umschalt um 1 mm. Jede Änderung wird sofort gespeichert, über dieselben Operationen wie in der Kommandozeile und bei den MCP-Werkzeugen; Prüfungen und Meldungen sind also dieselben. Hat sich die Projektdatei außerhalb des Editors geändert, lehnt der Editor die Änderung ab, zeigt eine Meldung und lädt den aktuellen Stand. Besucht eine Bahn den Posten, fragt das Löschen vorher nach. Strg/Cmd+Z macht rückgängig, Strg/Cmd+Umschalt+Z oder Strg+Y wiederholt, und `?` listet alle Tastenkürzel. Der Verlauf hält bis zu 100 Schritte und höchstens 64 MiB an Dateikopien. Hat ein anderes Programm die Datei geschrieben, lehnt das nächste Rückgängig oder Wiederholen ab und leert den Verlauf. Beide lehnen auch ab, wenn die Seite einen älteren Stand zeigt als die Datei, etwa in einem zweiten Tab.
- Der Editor setzt Postenbeschreibungen (ADR 0051). Die Eigenschaften eines Postens führen die Spalten C bis H auf, jede zugeklappt und mit dem aktuellen Symbol. Klappe eine Spalte auf und wähle ein Symbol aus einem Raster aus Zeichen mit Nummer und Text, mit der Maus, den Pfeiltasten oder durch Tippen einer Nummer wie `1.3`. Richtungssymbole gibt es mit ihrer Richtung (`12.1N`), Spalte E bietet auch die Symbole der Spalte D an, und „Kein Symbol“ leert eine Spalte. Jede Wahl ist ein Schritt für Rückgängig. Die Zahlensymbole 9.1 bis 9.4 bietet er nicht an, weil die Projektdatei keinen Wert dafür hält. Eine Beschreibung setzt der Befehl jetzt nur noch mit Symbolen, die in die Spalte passen, auf der Kommandozeile, in den MCP-Werkzeugen und im Editor gleich; Symbole, die schon in einer Datei stehen, bleiben unverändert.
- Der Editor legt Bahnen, Kategorien und Veranstaltungseinstellungen an (ADR 0048, 0049). „Neue Bahn...“ erzeugt eine Bahn mit Start und Ziel der Veranstaltung. Im Bahnmodus hängt ein Klick auf einen Posten ihn an, ein Klick auf eine Strecke und dann auf einen Posten fügt ihn dort ein. Die Postenliste ordnest du mit Alt+Pfeil hoch und Alt+Pfeil runter um. Anstieg, geplanter Druckmaßstab, Kategorien, Regelkategorie, erwartete Starter, Regelprofil mit Overlays und Startfenster sind Felder in den Eigenschaften.
- Strecken bekommen im Editor Knicke und Absperrband: Ziehe an der Linie einer Strecke, um einen Knick einzufügen, ziehe einen Knick, um ihn zu verschieben, drücke Entf, um ihn zu löschen, und hake „Abgeflaggt“ für eine abgeflaggte Strecke an. Die Bahnlänge zieht bei jeder Änderung mit.
- Postennummern lassen sich an eine neue Stelle ziehen. Der Editor speichert Richtung und Abstand für die Bahn, mit „für alle Bahnen“ am Posten. „Nummern platzieren“ setzt die automatischen Nummern und lässt gesetzte stehen, außer „auch gesetzte Nummern ersetzen“ ist angekreuzt. Die neue Route `api/number-at` rechnet einen Ablegepunkt in Richtung und Abstand um.
- Sonderobjekte lassen sich im Editor zeichnen (Start, Ziel, Erste Hilfe, Wasser, markierte Strecke, verbotene Route, Sperrgebiet, Gefahrenfläche), Bahnen zuordnen und löschen. Linien und Flächen enden mit Enter oder Doppelklick. Verschieben gibt es für Sonderobjekte nicht; lösche und zeichne neu.
- Ein Posten bekommt einen neuen Code und eine Bahn einen neuen Namen: `control code <Projekt> <von> <nach>`, `course rename <Projekt> <von> <nach>`, die MCP-Werkzeuge `set_control_code` und `rename_course` und die Felder „Kennzahl“ und „Name“ in den Eigenschaften des Editors (ADR 0024). Bahnen, Sonderobjekte und akzeptierte Befunde, die den Posten oder die Bahn nennen, ziehen mit. Ein Code oder Name, den es schon gibt, der leer ist, Leerzeichen am Rand hat oder zu lang ist, wird abgelehnt, und die Datei bleibt, wie sie war.
- Posten-Codes dürfen höchstens 64 Zeichen haben, Bahn- und Kategorienamen und die Begründungen akzeptierter Befunde höchstens 200. Längere Texte lehnt das Programm mit einer Meldung ab, im Editor, auf der Kommandozeile und bei den MCP-Werkzeugen gleich; bestehende Projektdateien bleiben gültig.
- Der Editor zeigt die Befunde der Prüfung (ADR 0052). Eine halbe Sekunde nach jeder Änderung führt er dieselbe Prüfung aus wie `check`. Die Statusleiste zählt Fehler, Warnungen und Hinweise; eine Schublade listet offene und akzeptierte Befunde und die Regeln, die nicht liefen. Die Eigenschaften nennen die Befunde der Auswahl, und die Karte markiert sie mit einem Ring oder einer gestrichelten Linie und einem Zeichen je Schwere. Ein Abstand außerhalb des Regelbereichs bekommt ein Warnzeichen am Nummernfeld. „Akzeptieren…“ speichert eine Ausnahme mit einer Begründung von höchstens 200 Zeichen; „Akzeptanz zurücknehmen“ holt den Befund zurück. Beides ist ein Schritt für Rückgängig.
- Das Menü „Veranstaltung“ exportiert und importiert (ADR 0053). „IOF-XML exportieren“ lädt die Datei herunter, „Nach Purple Pen exportieren“ schreibt die verknüpfte `.ppen` oder lädt eine neue herunter, und „Purple-Pen-Datei importieren“ liest eine hochgeladene Datei. Der Browser nennt keine Datei; geschrieben wird nur die Datei, die `sources.ppen` nennt. Ein Import in ein Projekt, das schon Inhalt hat, braucht „Ersetzen“. Das Schreiben in die verknüpfte Datei und der Import leeren den Rückgängig-Verlauf. Eine leere Veranstaltung erscheint jetzt richtig in der Seite.
- Das Menü „Veranstaltung“ gleicht mit Purple Pen ab (ADR 0054). Ein Hinweis bittet dich, die Veranstaltung vorher in Purple Pen zu schließen; er lässt sich für die Sitzung abschalten. Das Fenster listet die Änderungen und die Konflikte mit den Werten aus Basiskopie, Courseplan und Purple Pen. Du wählst für jeden Konflikt eine Seite oder mit „Alle Courseplan“ und „Alle Purple Pen“ sofort für alle. Ein Abgleich, der Dateien geschrieben hat, leert den Rückgängig-Verlauf; speichert Purple Pen oder ein anderes Programm während des Abgleichs eine Datei, wird nichts geschrieben, und der Editor lädt den aktuellen Stand.
- Der Editor zeigt die echte Karte (ADR 0056, 0046). Der gemeinsame Kartenzeichner freemapper-render zeichnet sie als SVG, die Bahnüberlagerung liegt darüber. Die Seite fragt den sichtbaren Ausschnitt an, 150 ms nachdem sich die Ansicht nicht mehr bewegt; der Server hält die Seite je Dateistand und den letzten Ausschnitt je Ansicht. Eine Karte, die im Kartenmaßstab größer ist als ein A3-Blatt, wird auf den mittleren Teil beschnitten gezeichnet, und die Statusleiste sagt das. Lässt der Kartenzeichner Objekte weg oder kürzt sie, sagt die Statusleiste, dass die Kartenfläche unvollständig ist. Ein zu großer Ausschnitt wird mit einer Meldung abgelehnt, das Overlay bleibt. Noch nicht gebaut: die Druckvorschau und der Test, der das Overlay mit dem Bahndruck von freemapper-print vergleicht; beides wartet auf dessen Bahnebene.

### Geändert

- Das Speichern einer Projektdatei lehnt jetzt ab, wenn sich die Datei geändert hat, seit der Befehl sie gelesen hat (Kommandozeile, MCP-Werkzeuge und Editor gleichermaßen). Es wird nichts geschrieben; die Kommandozeile endet mit Exit-Code 1 und meldet `<Datei> hat sich geändert, seit es gelesen wurde. Es wurde nichts geschrieben`. Führe den Befehl noch einmal aus. Das schließt die Lücke des verlorenen Updates beim Abgleich (ADR 0044, 0040).
- Die gemeinsamen Bausteine kommen aus freemapper-core v0.14.1. Der Editor zeichnet die Karte mit `freemapper-render` und nutzt die Liste der Beschreibungssymbole (`description_symbols::ids`) und deren Zeichnungen und Texte (`render`); am übrigen Programm ändert sich nichts. Die Lock-Datei hält `crc32fast` 1.5.3 und `flate2` 1.1.10 fest, weil neuere Versionen jünger als zwei Wochen waren (ADR 0006, 0056).
- Tests: Jeder Testlauf liest jetzt vier Karten von freemapper (Schulhof 1:1000, Sprint 1:4000, eine Karte von freemapper 0.9.0 und eine mit unbekannter Kartenart) und vergleicht Kartenart, Titel, Randobjekte, Grundfläche und Kartenausdehnung mit den Erwartungswerten aus `freemapper-contract` (ADR 0041). Ändert freemapper ein Tag oder einen Schlüssel der `freemapper.toml`, schlagen diese Tests fehl.

### Behoben

- Die Prüfung des Quelltexts (clippy) lief unter Windows nicht durch, weil ein Fehlertyp von `sync` dort zu groß war. Das Programm verhält sich wie zuvor.

## [0.9.0] - 2026-11-10

### Hinzugefügt

- `sync` gleicht das Projekt und die verknüpfte `.ppen` in beide Richtungen ab (ADR 0040). Es vergleicht Feld für Feld die Basiskopie `X.courseplan.ppen-base` (gehört in Git; `sync` entfernt daraus den Kartenpfad `absolute-path`), die Datei jetzt und das Projekt jetzt. Haben beide Seiten dasselbe Feld verschieden geändert, oder löscht die eine Seite, was die andere bearbeitet hat, nennt `sync` beide Werte und schreibt nichts. Mit `--take ours`, `--take theirs` oder `--take SCHLÜSSEL=SEITE` wählst du eine Seite, `--dry-run` schreibt nichts, `--json` gibt das Ergebnis maschinenlesbar aus. Ein unterbrochener Abgleich wird beim nächsten Lauf beendet. Der Abgleich lehnt Dateien aus einer unbekannten Purple-Pen-Version ab. Kategorien, akzeptierte Befunde und Mannschafts-, Staffel- und Score-Bahnen bleiben im Projekt.
- Neue MCP-Werkzeuge `sync_ppen` und `resolve_sync_conflict`.
- Die Projektdatei kennt eine optionale Tabelle `[sync]` mit Verknüpfung, Hashwert der Basiskopie und gespeicherten Entscheidungen. Das Format bleibt 1.

## [0.8.0] - 2026-11-10

### Hinzugefügt

- Der Export nach IOF-XML schreibt für Start, Posten und Ziel Breite und Länge (`Position`, sechs Nachkommastellen), wenn die Karte georeferenziert ist (ADR 0039). Ohne nutzbare Georeferenzierung bleiben es Kartenpositionen, und `export` meldet das. Das MCP-Werkzeug `export_iof_xml` gibt zusätzlich `geo_positions` zurück.

## [0.7.5] - 2026-11-10

### Geändert

- freemapper-core 0.6.1: Automatisch gesetzte Nummern speichern ihre Richtung wieder in sauberen Zehntelgrad
  (ADR 0036). Die Rundung als Zwischenlösung in courseplan entfällt.

## [0.7.4] - 2026-11-10

### Behoben

- Der Export nach Purple Pen schreibt die Variante `single` in das richtige Feld. Dasselbe Merkmal in den
  Spalten D und E über einer Kreuzung oder Einmündung in Spalte F steht jetzt als `single` in Spalte F.
- Der Export lässt Beschreibungssymbole aus, die der Standard der Datei nicht kennt, etwa 2.10 in einer Datei von
  2004, und nennt jedes ausgelassene Symbol (ADR 0037).
- Der Export vereinheitlicht die Richtungen `0.1` und `0.2`. Das einfache 9.1 und 9.3 schreibt er als `9.1high`
  und `9.3high` und weist in einem Hinweis darauf hin.

### Geändert

- Die Lage der Postennummern und die Tabellen der Beschreibungssymbole kommen jetzt aus freemapper-core 0.6.0,
  demselben Code, den freemapper-print nutzt. Die Lagen ändern sich nur in zwei Randfällen: bei einem Knick, der
  näher als eine Linienbreite am Kreis liegt, und bei einem Regelminimum mit drei Nachkommastellen. Im Beispiel
  `luebeck-sprint` verschiebt sich keine Nummer (ADR 0036, ADR 0037).

## [0.7.3] - 2026-11-10

### Behoben

- Der Maßstabsbalken und die Grundfläche, die freemapper 0.9.0 in die Karte schreibt (`freemapper:source=scale-bar`
  und `base`), zählen nicht mehr als Kartenobjekte und verschieben den Kartenrand nicht mehr. Die Prüfung
  `map_edge` findet Posten am südlichen Rand wieder.

### Geändert

- Die Kartenart kommt zuerst aus `map_kind` in der `freemapper.toml` neben der Karte, danach aus dem Namen des
  Symbolsatzes (Groß- und Kleinschreibung egal) und dem Maßstab, wie freemapper-print sie bestimmt. Ein
  unbekannter Wert oder eine fehlerhafte Datei wird übergangen, nicht gemeldet (ADR 0038).

## [0.7.2] - 2026-11-10

### Behoben

- Purple Pen 4.0 stürzte beim Öffnen einer mit `export --ppen` geschriebenen `.ppen`-Datei ab: Ein Ziel ohne
  Beschreibungssymbol brach die Beschreibungsansicht. Der Export gibt jedem Ziel, jeder Querungsstelle und jedem
  Kartenausgabepunkt jetzt Purple Pens eigenes Symbol (14.3, 13.3, 13.6), und ein Export auf eine ältere Datei
  ergänzt fehlende. freemapper-core 0.4.2 prüft die Ausgabe tiefer gegen die Anforderungen des Laders. Das
  Beispiel `luebeck-sprint.ppen` ist neu erzeugt.

## [0.7.1] - 2026-11-10

### Behoben

- `export --ppen` schrieb Bahnen, die Purple Pen 4.0 nicht öffnete: In `<options>` fehlte das Pflichtattribut
  `description-kind`. Dateien aus 0.7.0 lassen sich nach dem nächsten Export darauf wieder laden. freemapper-core
  0.4.1 prüft alles, was der Writer schreibt, gegen die Attribute, die Purple Pens Lader verlangt. Das Beispiel
  `luebeck-sprint.ppen` ist neu erzeugt.

## [0.7.0] - 2026-11-10

### Hinzugefügt

- `freemapper-courseplan export <Projekt> --ppen <Datei> [--new]` schreibt die Einzelbahnen als Purple-Pen-Event.
  Eine vorhandene Datei ändert der Export an Ort und Stelle und lässt alles stehen, was das Projekt nicht kennt
  (Lücken in Kreisen und Strecken, Kartenwechsel, Text in Spalte F, Bahnlänge); `--new` schreibt die Datei neu.
  Der Export löscht nie: Was in der Datei steht und im Projekt fehlt, bleibt und wird als behalten gemeldet. Was
  er nicht oder anders schreibt (Mannschafts-, Staffel- und Score-Bahnen, Symbole ohne Purple-Pen-Gegenstück,
  abgeflaggte Teilstrecken nach geänderten Knickpunkten), nennt er als Hinweis. Eine neu angelegte Datei trägt er
  im Projekt als `sources.ppen` ein, wenn dort noch keine steht; für diese Datei speichert er die
  Purple-Pen-Kennungen im Projekt. Der KI-Assistent hat das Werkzeug `export_ppen`. Den Schreiber liefert
  freemapper-core v0.4.0 (`freemapper-course`). **In Purple Pen 4.0 noch nicht geöffnet** (ADR 0035).
- Das Beispiel [`luebeck-sprint.ppen`](examples/luebeck-sprint/luebeck-sprint.ppen); ein Test vergleicht es mit
  einem frischen Export, ein zweiter liest es wieder ein und exportiert es auf sich selbst.
- Ein Test liest den IOF-XML-Export (Lübeck-Beispiel und ein Ereignis mit Wechselpunkt) mit dem IOF-Leser von
  freemapper-course wieder ein und vergleicht Posten, Positionen, Bahnen, Reihenfolge und Nummernlagen.

### Geändert

- Die Dokumentation sagt jetzt, was geprüft ist: Der Export besteht das IOF-Schema und lässt sich wieder einlesen.
  Ein Import in OE, Purple Pen oder Condes ist nicht getestet, weil im Projekt niemand OE hat (ADR 0033, Nachtrag).

## [0.6.0] - 2026-11-09

### Hinzugefügt

- `freemapper-courseplan init <Karte> --ppen <Datei>` und `import <Projekt> --ppen <Datei> [--replace]` übernehmen
  ein Purple-Pen-Event: Posten, Beschreibungen (auf den Standard 2024 umnummeriert), Einzelbahnen, Knickpunkte,
  abgeflaggte Strecken, Nummernlagen und Sonderobjekte. Der KI-Assistent hat das Werkzeug `import_ppen`. Was das
  Modell nicht fassen kann, nennt das Programm als Hinweis. Ohne `--replace` weigert sich `import`, ein Projekt mit
  Inhalt zu überschreiben (ADR 0034).
- Posten und Bahnen merken sich ihre Purple-Pen-Kennung (`ppen_id`) für den späteren Abgleich.

### Geändert

- Die Karte liest jetzt `freemapper-omap` und die `.ppen` `freemapper-course` aus freemapper-core 0.3.0, statt
  einer eigenen Kopie. `freemapper-text` und `freemapper-rules` stehen auf demselben Stand.
- Eine verbotene Route darf aus einem Punkt bestehen, wie Purple Pen sie speichert.

## [0.5.0] - 2026-11-09

### Hinzugefügt

- `freemapper-courseplan export <Projekt> --iof-xml <Datei>` schreibt die Einzelbahnen als IOF-XML 3.0
  (`CourseData`) für die Auswertung. Der KI-Assistent hat dafür das Werkzeug `export_iof_xml`. Mannschafts-,
  Staffel- und Score-Bahnen werden übersprungen und genannt. Geopositionen und abgeflaggte Strecken fehlen noch (ADR 0033). Die
  Ausgabe ist gegen das IOF-Schema geprüft, ein Import in OE steht noch aus. Ein Beispiel liegt unter
  `examples/luebeck-sprint/course-data.xml`.

### Geändert

- Die Programmtexte kommen jetzt aus `freemapper-text` (freemapper-core 0.1.0) statt aus einer eigenen Kopie des
  Codes. Für Nutzer ändert sich nichts.
- `freemapper-rules` (freemapper-core 0.2.0) liest jetzt die Regelprofile. Der Lader prüft zusätzlich die
  Wertebereiche nach Einheit; die eingebetteten Profile bleiben unverändert.

## [0.4.0] - 2026-11-09

### Hinzugefügt

- `check` warnt, wenn der Abstand einer Postennummer zum Kreis außerhalb von 0,5 bis 5 mm liegt
  (`wkb2026.number.gap`, Werte `min_gap_mm` und `max_gap_mm`). Keine Norm nennt einen Wert, die Grenzen hat der
  Nutzer festgelegt (ADR 0032). Gespeicherte Nummern bleiben, wo sie sind; alte Abstände über 5 mm werden jetzt
  gemeldet.

### Geändert

- Die automatische Platzierung probiert Abstände vom Mindestwert des Regelprofils an in Schritten von 0,5 mm bis
  zum Höchstwert. Ein größerer Abstand in der bevorzugten Richtung geht vor einer anderen Richtung mit kleinerem
  Abstand. Ohne passende Regel bleibt es bei 0,5 mm. Neu gesetzte Nummern können daher weiter außen stehen.

## [0.3.0] - 2026-11-09

### Hinzugefügt

- `check` weist auf fast gestreckte Winkel an Posten hin: Bilden die Strecken an einem Posten einen Winkel über 170°,
  gibt es einen Hinweis (`wkb2026.leg.straight_angle_hint`). Querungsstellen bleiben außen vor.

### Geändert

- Die automatische Platzierung setzt eine Postennummer hinter den Kreis, in Verlängerung der Strecke, die zum
  Posten führt. Läuft die weiterführende Strecke dort entlang (fast gestreckter Winkel), steht die Nummer außen, an
  der Spitze des Winkels. Die Nummer deckt keine Punkt- und Liniensymbole der Karte ab, solange ein freier Platz
  bleibt; Flächen und Nordlinien (Symbol 601) darf sie überdecken. Strecken, Kreise und andere Nummern zählen
  stärker als Kartensymbole. Ist der bevorzugte Platz belegt, nimmt die Platzierung den freien Platz mit der
  geringsten Abweichung, in Schritten von 15°; die Richtung ist auf 0,1° gerundet. Neu gesetzte Nummern stehen
  daher oft anders als bisher (ADR 0031). freemapper-print baut dieselben Regeln in sein gemeinsames
  Platzierungsmodul ein.
- Der Abstand einer Postennummer (`--gap`, `gap_mm`) zählt jetzt ab der Außenkante der Kreislinie und meint
  den kürzesten Abstand zum Nummernfeld, in jeder Richtung. Bisher zählte er ab der Mitte der Kreislinie, und
  schräg zum Kreis stand die Nummer weiter weg als angegeben. freemapper-print rechnet ab seinem M3 genauso.
  Gespeicherte Nummernlagen bleiben gültig. Gerade über, unter oder neben dem Kreis rückt eine Nummer um
  0,175 mm nach außen, schräg zum Kreis bis zu 0,6 mm näher heran.

## [0.2.0] - 2026-11-09

### Hinzugefügt

- `check <PROJEKT>` prüft die Einzelbahnen gegen das Regelprofil der Veranstaltung und seine Aufsätze. Die
  Befunde stehen nach Bahnen geordnet, jeder mit Schwere, Ort, Erklärung, Kennung der Prüfregel und Quelle,
  danach eine Zusammenfassung. `--all` zeigt auch die akzeptierten Befunde mit Begründung und die übersprungenen
  Prüfregeln mit dem Grund. Eine Prüfregel, deren Filter (Disziplin, Kartenart, Druckmaßstab) nicht passt, gilt
  als übersprungen und wird so gemeldet. Mannschafts-, Staffel- und Score-Bahnen werden noch nicht geprüft. Der
  Exit-Code ist 0 ohne Befund der Schwere Fehler, 1 mit einem solchen Befund oder bei abgelehnter Prüfung und 2
  bei falschem Aufruf.
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
  den gemessenen Wert und die Grenze, etwa „Posten 31 und Posten 32 liegen 12 m auseinander, gefordert sind
  mindestens 15 m“.
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
- `check` prüft jetzt die Bahntabellen. Bei Lang-, Mittel- und Ultralang-Distanz ergibt eine Bahn, deren Länge in
  Prozent der HE-Bahn außerhalb des Bereichs ihrer Kategoriezeile liegt, eine Warnung (WKB Anlage 5); ohne
  HE-Bahn oder bei einer HE-Bahn der Länge 0 wird die Prüfregel übersprungen, bei zwei HE-Bahnen dient die erste
  als Bezug, und ein Hinweis sagt das. Eine Bahn mit leichten und schweren Kategorien (Stufen 1 und 2 gegen 3
  bis 5) ist ein Fehler, der beide Seiten nennt. Mit dem NOR-Aufsatz ergeben Leistungskilometer (Länge plus
  Anstieg × 10 / 1000) außerhalb des Ziel-Lkm der Kategoriezeile ±6 % eine Warnung, mit bis zu zwei
  Nachkommastellen; eine Bahn ohne Anstieg wird übersprungen. Eine Kategorie, die in keiner Zeile der Tabelle
  steht, überspringt die Bahn für diese Kategorie, bei der Längentabelle wie bei der NOR-Tabelle. Trifft eine
  Bahn mit ihren Kategorien mehrere Zeilen, gibt es einen Befund, der jede verletzte Zeile nennt. Eine Kategorie
  ohne Regelkategorie überspringt die Bahn und wird genannt.
- Handbuch, Kapitel 8 „Prüfen: Hinweise lesen, Ausnahmen begründen“: wie man eine Zeile von `check` liest, was
  Schwere, Exit-Code und übersprungene Prüfregeln bedeuten, wie man Ausnahmen akzeptiert und zurücknimmt und wie
  man Profil und Aufsätze wählt. Das Lübecker Beispiel zeigt in `check.de.txt` und `check.en.txt` die Ausgabe von
  `check` und einen akzeptierten Hinweis; die Projektdatei enthält ihn.
- `event start-window <PROJEKT> <MINUTEN>` setzt das Startfenster, das die Prüfung der Startkapazität braucht;
  0 lehnt es ab. `class set <PROJEKT> <KATEGORIE> [--rule-category C] [--starters N]` ändert Regelkategorie und
  erwartete Starterzahl einer Kategorie, `--no-rule-category` und `--no-starters` entfernen sie. Über MCP heißen
  die Werkzeuge `set_start_window` und `set_class`. Fehlt eine dieser Angaben, nennt `check --all` bei der
  übersprungenen Prüfregel den Befehl, der sie setzt.
- `check --all` listet unter „nicht zugeordnet“ akzeptierte Befunde, zu denen kein Befund mehr passt, etwa weil
  ein Posten oder eine Bahn entfernt wurde, mit Begründung und dem Hinweis auf `finding remove`. Die
  Zusammenfassung zählt sie, der Exit-Code bleibt davon unberührt. MCP `check` gibt sie in `unmatched` zurück.
- Eine Prüfregel, die ein Aufsatz abschaltet, steht bei `check --all` unter den übersprungenen, mit einem Grund,
  der den Aufsatz nennt: „Das Regelprofil nor-bb-2021 schaltet die Prüfregel ab“.
- `check --all` zeigt unter jedem Befund die Anmerkung seiner Prüfregel in der gewählten Sprache. Ein Wert, den
  das Programm vorschlägt, weil keine Quelle eine Zahl nennt, trägt als Quelle „Vorschlag von
  freemapper-courseplan“.

### Geändert

- Eine Strecke heißt in der Ausgabe nach ihren Enden: „Strecke 36–37“, „Strecke Start–31“, „Strecke 37–Ziel“,
  zwei Strecken „Strecken 31–32 und 35–36“. Start und Ziel heißen als Ort „Start S1“ und „Ziel F1“. Innerhalb einer
  Bahn steht die Bahn vorn, dann die Strecken in Laufrichtung. Eingegeben wird eine Strecke weiter als
  `--leg A:6`, die Strecke ab Position 6 der Bahn, wobei Querungsstellen mitzählen; die Hilfe sagt das.
- Ein akzeptierter Befund an einer Strecke speichert deren Enden (`from`, `to`). Läuft die Strecke später zwischen
  anderen Posten, meldet `check` den Befund wieder. `finding remove --leg A:6` entfernt die Ausnahme für die
  Strecke, wie sie jetzt verläuft; gibt es keine, entfernt es die älteren an dieser Position.
- Texte: Grenzen lauten „gefordert sind mindestens 15 m“, eine Strecke über einen Posten „Die Strecke führt über
  den Posten 31“ statt „0 m am Posten 31 vorbei“. Start- und Tabellenbefunde wiederholen ihren Ort nicht mehr im
  Text. Eine leere Begründung wird mit einer eigenen Meldung abgelehnt.

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
