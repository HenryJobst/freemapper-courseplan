# Änderungen

[English version](CHANGELOG.en.md) · [Handbuch](docs/manual/de.md)

Hier stehen alle wichtigen Änderungen an freemapper-courseplan, die neueste zuerst. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/).

## [Unveröffentlicht]

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
