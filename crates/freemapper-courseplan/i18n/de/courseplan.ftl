# Programmtexte von freemapper-courseplan, Deutsch (ADR 0022).
# Die Variablennamen sind in jeder Sprache gleich. Texte aus fremden Programmen
# (Parser-Meldungen der toml-Bibliothek, Fehler des Betriebssystems) kommen in $source
# und $message an und bleiben Englisch.

## Wörter, die in anderen Meldungen vorkommen

word-kind-individual = Einzelbahn
word-kind-team = Mannschaftsbahn
word-kind-relay = Staffelbahn
word-kind-score = Score-Bahn
word-what-control = Posten
word-what-course = Bahn
word-what-special-object = Sonderobjekt
word-what-class = Kategorie
word-what-control-or-special-object = Posten oder Sonderobjekt
word-map-sprint = Sprintkarte
word-map-schoolyard = Schulhofkarte
word-map-forest = Waldkarte

## Widersprüche innerhalb einer Veranstaltung (ModelError)

error-model-duplicate-control = Posten { $code } ist mehrfach definiert
error-model-duplicate-course = Bahn { $name } ist mehrfach definiert
error-model-duplicate-class = Kategorie { $name } ist mehrfach definiert
error-model-duplicate-special-id = Die Sonderobjekt-Kennung { $id } wird mehrfach verwendet
error-model-unknown-course-in-special = Sonderobjekt { $id } gehört zur Bahn { $course }, die nicht definiert ist
error-model-unknown-control = Bahn { $course } benutzt Posten { $code }, der nicht definiert ist
error-model-unknown-start = Bahn { $course } hat den Start { $id }, der nicht definiert ist
error-model-unknown-finish = Bahn { $course } hat das Ziel { $id }, das nicht definiert ist
error-model-team-too-small = Bahn { $course } hat die Mannschaftsgröße { $team_size }, nötig sind mindestens 2
error-model-free-control-is-mandatory = Bahn { $course } führt Posten { $code } als Wahlposten und als Pflichtposten
error-model-date-not-a-date = Das Veranstaltungsdatum { $date } ist kein Datum. Schreibe es als JJJJ-MM-TT, ohne Uhrzeit und Zeitzone
error-model-unsupported-format = Das Format { $format } der Projektdatei ist neuer, als dieses Programm versteht
error-model-unknown-key-top = Unbekannter Schlüssel { $key } auf oberster Ebene
error-model-unknown-key-in = Unbekannter Schlüssel { $key } in { $table }

## Projektdateien (ProjectError)

error-project-read = { $path } lässt sich nicht lesen: { $source }
error-project-write = { $path } lässt sich nicht schreiben: { $source }
error-project-already-exists = { $path } gibt es schon. Wähle eine andere Projektdatei oder lösche die vorhandene
error-project-parse = { $path } lässt sich nicht einlesen: { $message }
error-project-serialise = { $path } lässt sich nicht erzeugen: { $message }
error-project-invalid = { $path } ist keine stimmige Veranstaltung:{ $list }

## Veranstaltung anlegen (InitError)

error-init-map = Die Karte { $path } lässt sich nicht lesen: { $source }
error-init-forest = Waldkarten (ISOM) werden noch nicht unterstützt. Diese Version arbeitet mit Sprint- und Schulhofkarten (ADR 0010).
error-init-io = Der Ordner { $path } lässt sich nicht anlegen: { $source }

## Bahnen messen (CourseError)

error-course-not-supported = Bahn { $course } ist eine { $kind }. Bisher werden nur Einzelbahnen unterstützt
error-course-unknown-course = Bahn { $course } gibt es nicht
error-course-unknown-reference = Bahn { $course } nennt { $id }, das die Veranstaltung nicht definiert

## Bearbeiten (EditError)

error-edit-invalid = Mit der Änderung wäre die Veranstaltung widersprüchlich: { $errors }
error-edit-not-found = { $what } { $id } gibt es nicht
error-edit-code-taken = Posten { $code } gibt es schon
error-edit-id-taken = Sonderobjekt { $id } gibt es schon
error-edit-outside-map = Der Punkt ({ $x }, { $y }) liegt außerhalb der Karte, die von ({ $min_x }, { $min_y }) bis ({ $max_x }, { $max_y }) reicht
error-edit-in-use = { $id } wird noch von { $courses } verwendet. Ändere zuerst diese Bahnen
error-edit-not-supported = Bahn { $course } ist keine Einzelbahn. Bisher lassen sich nur Einzelbahnen bearbeiten
error-edit-bad-index = Bahn { $course } hat keinen Posten an Index { $index }

## OOM-Karten lesen (OmapError)

error-omap-xml = Ungültiges OOM-XML: { $message }
error-omap-missing-element = Das Element <{ $name }> fehlt
error-omap-io = Lesefehler: { $message }
error-omap-binary-format = Diese Datei hat das alte Binärformat von OpenOrienteering Mapper, das dieses Programm nicht lesen kann. Öffne sie in Mapper und speichere sie als aktuelle .omap- oder .xmap-Datei neu.
error-omap-unsupported = { $message }

## Kommandozeile

cli-error-prefix = Fehler
cli-error-unknown-lang = Unbekannte Sprache { $value }. Verfügbar: { $available }

cli-info-title = Titel: { $value }
cli-info-map = Karte: { $value }
cli-info-map-kind = Kartenart: { $value }
cli-info-controls = Posten: { $count }
cli-info-courses = Bahnen: { $total } (einzeln { $individual }, Mannschaft { $team }, Staffel { $relay }, Score { $score })
cli-info-classes = Kategorien: { $count }

cli-about = Orientierungslauf-Bahnen auf einer OOM-Karte legen
cli-help-init = Eine Veranstaltung aus einer Karte anlegen und ihre Projektdatei schreiben
cli-help-info = Titel, Karte, Posten, Bahnen und Kategorien einer Projektdatei ausgeben
cli-arg-init-map = Die OOM-Karte (Sprint- oder Schulhofkarte)
cli-arg-init-project = Wohin die Projektdatei kommt [Standard: <Kartenordner>/<Kartenname>.courseplan.toml]
cli-arg-init-title = Titel der Veranstaltung [Standard: `title` in freemapper.toml neben der Karte, sonst der Kartenname]
cli-arg-info-project = Die Projektdatei
cli-arg-lang = Sprache der Meldungen und der Hilfe, zum Beispiel de oder en [Standard: FREEMAPPER_LANG, sonst die Systemsprache, sonst en]
cli-arg-help = Hilfe anzeigen
cli-arg-version = Version anzeigen

cli-heading-usage = Aufruf:
cli-heading-commands = Befehle:
cli-heading-options = Optionen:
cli-heading-arguments = Argumente:
cli-help-help = Die Hilfe eines Befehls anzeigen
cli-arg-help-command = Der Befehl, dessen Hilfe angezeigt wird
cli-parse-hint = Mehr dazu mit '--help'.
cli-parse-unknown-argument = Unbekanntes Argument { $arg }
cli-parse-missing-required = Pflichtargument fehlt: { $args }
cli-parse-invalid-value = Ungültiger Wert { $value } für { $arg }
cli-parse-invalid-subcommand = Unbekannter Befehl { $value }
cli-parse-missing-subcommand = Ein Befehl ist nötig
cli-parse-too-many-values = Unerwarteter Wert { $value } für { $arg }
cli-parse-argument-conflict = { $arg } lässt sich nicht zusammen mit { $other } verwenden

## Bearbeiten

cli-edit-control-added = Posten { $code } bei { $x },{ $y } angelegt
cli-edit-control-moved = Posten { $code } nach { $x },{ $y } verschoben
cli-edit-control-removed = Posten { $code } entfernt
cli-edit-control-removed-from = Posten { $code } und seine Besuche in { $courses } entfernt
cli-edit-control-described = Beschreibung von Posten { $code }: { $columns }
cli-edit-number-set = Nummer von Posten { $code }: Richtung { $direction } Grad, Abstand { $gap } mm
cli-edit-number-cleared = Nummernlage von Posten { $code } zurückgesetzt
cli-edit-special-added-at = { $kind } { $id } bei { $x },{ $y } angelegt
cli-edit-special-added-points = { $count ->
    [one] { $kind } { $id } mit 1 Punkt angelegt
   *[other] { $kind } { $id } mit { $count } Punkten angelegt
}
cli-edit-special-removed = Sonderobjekt { $id } entfernt
cli-edit-class-added = Kategorie { $name } angelegt
cli-edit-class-removed = Kategorie { $name } entfernt
cli-edit-bad-point = { $value } ist kein Punkt. Schreibe X,Y in Kartenmillimetern, mit Punkt als Dezimaltrenner, zum Beispiel 12.5,-30
cli-edit-bad-number = { $value } ist keine Zahl. Nimm einen Punkt als Dezimaltrenner, zum Beispiel 0.5
cli-edit-bad-count = { $value } ist keine ganze Zahl
cli-edit-kind-needs-at = { $kind } ist ein einzelner Punkt. Gib --at X,Y an und kein --point
cli-edit-kind-needs-points = { $kind } wird durch Punkte gezeichnet. Gib --point X,Y (einmal je Punkt) an und kein --at
cli-edit-no-courses-for-kind = { $kind } gilt für alle Bahnen. Lass --courses weg

word-empty = leer
word-special-start = Start
word-special-finish = Ziel
word-special-marked-route = Pflichtstrecke
word-special-out-of-bounds = Sperrgebiet
word-special-forbidden-route = Verbotene Route
word-special-dangerous-area = Gefahrenbereich
word-special-first-aid = Erste-Hilfe-Stelle
word-special-water = Verpflegungsstelle

cli-help-control = Posten anlegen, verschieben, beschreiben, nummerieren und entfernen
cli-help-special = Sonderobjekte anlegen und entfernen (Start, Ziel, Pflichtstrecken, Flächen, Erste-Hilfe- und Verpflegungsstellen)
cli-help-class = Kategorien anlegen und entfernen
cli-help-control-add = Einen Posten anlegen
cli-help-control-move = Einen Posten verschieben
cli-help-control-remove = Einen Posten und seine Besuche in Einzelbahnen entfernen
cli-help-control-describe = Die Beschreibungssymbole eines Postens setzen
cli-help-control-number = Den Platz der Postennummer setzen oder zurücksetzen
cli-help-special-add = Ein Sonderobjekt anlegen
cli-help-special-remove = Ein Sonderobjekt entfernen
cli-help-class-add = Eine Kategorie anlegen
cli-help-class-remove = Eine Kategorie samt ihrem Namen in allen Bahnen entfernen

cli-arg-edit-code = Code des Postens
cli-arg-control-add-at = Wohin der Posten kommt: X,Y in Kartenmillimetern
cli-arg-control-add-code = Code des neuen Postens [Standard: die niedrigste freie Zahl ab 31]
cli-arg-control-add-crossing-point = Eine Querungsstelle anlegen, die keine Nummer bekommt
cli-arg-control-move-at = Der neue Ort: X,Y in Kartenmillimetern
cli-arg-control-describe-c = Symbol für Spalte C (IOF-Symbol-ID)
cli-arg-control-describe-d = Symbol für Spalte D (IOF-Symbol-ID)
cli-arg-control-describe-e = Symbol für Spalte E (IOF-Symbol-ID)
cli-arg-control-describe-f = Symbol für Spalte F (IOF-Symbol-ID)
cli-arg-control-describe-g = Symbol für Spalte G (IOF-Symbol-ID)
cli-arg-control-describe-h = Symbol für Spalte H (IOF-Symbol-ID)
cli-arg-control-describe-clear = Zuerst alle Spalten leeren; Spalten im selben Aufruf werden danach gesetzt
cli-arg-control-number-direction = Grad, 0 = Osten, gegen den Uhrzeigersinn
cli-arg-control-number-gap = Abstand zum Kreis in Druckmillimetern
cli-arg-control-number-clear = Zum Standardplatz zurückkehren
cli-arg-special-add-kind = Was das Objekt ist
cli-arg-special-add-id = ID des Objekts, eindeutig unter den Sonderobjekten
cli-arg-special-add-at = Ort einer Einzelpunkt-Art (Start, Ziel, Erste-Hilfe-Stelle, Verpflegungsstelle): X,Y in Kartenmillimetern
cli-arg-special-add-point = Ein Punkt einer Linien- oder Flächenart: X,Y in Kartenmillimetern; die Option je Punkt wiederholen, in der Reihenfolge
cli-arg-special-add-courses = Das Objekt auf diese Bahnen beschränken, durch Kommas getrennt [Standard: alle Bahnen]
cli-arg-special-remove-id = ID des Sonderobjekts
cli-arg-class-add-name = Name der Kategorie
cli-arg-class-add-rule-category = Regelkategorie der Kategorie
cli-arg-class-add-starters = Erwartete Zahl der Starter
cli-arg-class-remove-name = Name der Kategorie

## Bahnbefehle

cli-info-course = Bahn { $name }: { $controls } Posten, { $km } km, Anstieg { $climb } m
cli-info-course-unmeasured = Bahn { $name }: { $kind }, Länge noch nicht unterstützt
word-decimal-point = ,
word-taped = abgeflaggt
word-not-taped = nicht abgeflaggt
cli-edit-course-added = Bahn { $name } angelegt
cli-edit-course-removed = Bahn { $name } entfernt
cli-edit-course-controls = Bahn { $name } führt jetzt über { $codes }
cli-edit-course-set = Bahn { $name } geändert
cli-edit-course-numbers = { $count ->
    [one] 1 Postennummer auf Bahn { $name } gesetzt
   *[other] { $count } Postennummern auf Bahn { $name } gesetzt
}
cli-edit-course-leg-start = Strecke vom Start der Bahn { $name } gesetzt: Knickpunkte { $bends }, { $taped }
cli-edit-course-leg-after = Strecke nach Position { $index } der Bahn { $name } gesetzt: Knickpunkte { $bends }, { $taped }

cli-help-course = Einzelbahnen anlegen, ändern, nummerieren und entfernen
cli-help-course-add = Eine Einzelbahn anlegen
cli-help-course-controls = Die Posten einer Bahn ersetzen
cli-help-course-remove = Eine Bahn samt ihrem Namen in den Sonderobjekten entfernen
cli-help-course-leg = Die Knickpunkte einer Strecke setzen und festlegen, ob sie abgeflaggt ist
cli-help-course-set = Kategorien, Anstieg oder geplanten Druckmaßstab einer Bahn ändern
cli-help-course-numbers = Die Postennummern einer Bahn frei von Strecken und Kreisen platzieren

cli-arg-edit-course = Name der Bahn
cli-arg-course-add-start = ID des Start-Sonderobjekts
cli-arg-course-add-finish = ID des Ziel-Sonderobjekts
cli-arg-course-add-controls = Codes der Posten in Laufreihenfolge, durch Kommas getrennt; eine Querungsstelle zählt als Position [Standard: keine]
cli-arg-course-add-classes = Kategorien, die die Bahn laufen, durch Kommas getrennt
cli-arg-course-add-climb = Anstieg in Metern
cli-arg-course-add-print-scale = Geplanter Druckmaßstab, zum Beispiel 10000 für 1:10000
cli-arg-course-controls-codes = Codes in Laufreihenfolge, durch Kommas getrennt. Der n-te Besuch eines Codes behält Knickpunkte, Abflaggung und Nummernlage vom bisherigen n-ten Besuch
cli-arg-course-leg-from-start = Die Strecke vom Start zum ersten Posten bearbeiten
cli-arg-course-leg-after = Die Strecke nach dieser Position bearbeiten, ab 1 gezählt wie auf der Karte; Querungsstellen zählen als Positionen
cli-arg-course-leg-bend = Ein Knickpunkt der Strecke: X,Y in Kartenmillimetern; die Option je Knickpunkt wiederholen, in der Reihenfolge. Ohne --bend verliert die Strecke ihre Knickpunkte
cli-arg-course-leg-taped = Die Strecke ist abgeflaggt. Ohne --taped gilt sie als nicht abgeflaggt
cli-arg-course-set-classes = Kategorien, die die Bahn laufen, durch Kommas getrennt
cli-arg-course-set-climb = Anstieg in Metern
cli-arg-course-set-no-climb = Den Anstieg entfernen
cli-arg-course-set-print-scale = Geplanter Druckmaßstab, zum Beispiel 10000 für 1:10000
cli-arg-course-set-no-print-scale = Den geplanten Druckmaßstab entfernen
cli-arg-course-numbers-overwrite = Schon gesetzte Nummern ersetzen [Standard: behalten]

## MCP-Server (Werkzeugnamen, Parameternamen und JSON-Felder bleiben Englisch)

cli-help-mcp = Das Bahnlegen über MCP auf stdin und stdout für KI-Assistenten anbieten

mcp-instructions =
    freemapper-courseplan legt Orientierungslauf-Bahnen auf einer OOM-Karte: Posten, Start und Ziel, Kategorien,
    Bahnen und die Postennummern. Jedes Werkzeug nimmt project, die Projektdatei der Veranstaltung, als absoluten
    Pfad oder mit ~ am Anfang. init_event macht aus einer Karte eine Projektdatei. Positionen stehen in
    Kartenmillimetern im Kartenmaßstab, y zeigt nach unten, und werden als Objekte mit den Zahlen x und y
    geschrieben. Ablauf: event_info, dann map_extent und map_features_near, um Positionen zu finden, dann
    add_special für Start und Ziel, add_control für jeden Posten, add_course, place_numbers und noch einmal
    event_info, um die Länge zu lesen. Jeder Aufruf speichert seine Änderung. Eine abgelehnte Änderung lässt die
    Datei, wie sie war. Bisher lassen sich nur Einzelbahnen bearbeiten und messen.
mcp-error-relative-path = { $path } ist ein relativer Pfad. Gib einen absoluten Pfad an oder einen mit ~ am Anfang
mcp-error-not-toml = { $path } ist keine Projektdatei. Projektdateien enden auf .toml
mcp-error-not-map = { $path } ist keine OOM-Karte. Karten enden auf .omap oder .xmap
mcp-error-project-parse = { $path } ist keine gültige Projektdatei. Das erste Problem liegt in Zeile { $line }, Spalte { $column }
mcp-error-project-parse-nopos = { $path } ist keine gültige Projektdatei
mcp-error-map-invalid = { $path } ist keine gültige OOM-Karte
mcp-error-no-home = Dieses System hat keinen Home-Ordner. Gib einen absoluten Pfad an
mcp-error-kind-needs-at = { $kind } ist ein einzelner Punkt. Gib at an und lass points weg
mcp-error-kind-needs-points = { $kind } wird durch Punkte gezeichnet. Gib points an und lass at weg
mcp-error-no-courses-for-kind = { $kind } gilt für jede Bahn. Lass courses weg
mcp-error-leg-target = Gib entweder from_start oder after an, nicht beides und nicht keins
mcp-error-no-change = Nichts zu ändern. Gib mindestens eines von classes, climb_m, no_climb, print_scale und no_print_scale an
mcp-error-number-args = Gib direction und gap_mm zusammen an, oder clear

mcp-tool-event-info = Überblick über die Veranstaltung: Titel, Karte und Kartenart, Kartenausdehnung, die Zahl der Posten, Sonderobjekte, Kategorien und Bahnen, und je Bahn die Posten, die Länge in km, den Anstieg und wie viele Postennummern gesetzt sind.
mcp-tool-list-controls = Die Posten auflisten, mit Code, Art, Position in Kartenmillimetern, Beschreibungssymbolen und Nummernlage.
mcp-tool-list-courses = Die Bahnen auflisten, mit Name, Art, Start, Ziel, Posten in Laufreihenfolge, Kategorien, Anstieg, geplantem Druckmaßstab und Länge in km.
mcp-tool-list-specials = Die Sonderobjekte auflisten (Start, Ziel, Pflichtstrecken, Sperrgebiete, verbotene Routen, Gefahrenbereiche, Erste-Hilfe-Stellen, Verpflegungsstellen), mit Geometrie und Bahnen.
mcp-tool-list-classes = Die Kategorien auflisten, mit Regelkategorie und erwarteten Startern.
mcp-tool-add-control = Einen Posten an einer Position in Kartenmillimetern anlegen. Ohne code gilt der niedrigste freie Code ab 31. Die Position muss auf der Karte liegen. Gibt den Code zurück.
mcp-tool-move-control = Einen Posten an eine neue Position in Kartenmillimetern setzen. Die Position muss auf der Karte liegen.
mcp-tool-remove-control = Einen Posten samt seinen Besuchen in Einzelbahnen entfernen. Das Ergebnis nennt die Bahnen, die er verlassen hat.
mcp-tool-describe-control = Die Beschreibungssymbole eines Postens setzen, Spalte C bis H, als IOF-Symbol-IDs. Spalten, die du weglässt, behalten ihr Symbol. clear leert zuerst alle Spalten.
mcp-tool-set-control-number = Festlegen, wo die Nummer eines Postens steht, als Richtung in Grad (0 = Osten, gegen den Uhrzeigersinn) und Abstand zum Kreis in Druckmillimetern. clear stellt den Standardplatz wieder her.
mcp-tool-add-special = Ein Sonderobjekt anlegen. Start, Ziel, Erste-Hilfe-Stelle und Verpflegungsstelle sind ein einzelner Punkt (at). Pflichtstrecke, verbotene Route, Sperrgebiet und Gefahrenbereich werden durch Punkte gezeichnet (points). Die id muss neu sein.
mcp-tool-remove-special = Ein Sonderobjekt anhand seiner id entfernen. Ein Start oder Ziel, den eine Bahn nutzt, lässt sich nicht entfernen.
mcp-tool-add-class = Eine Kategorie anlegen.
mcp-tool-remove-class = Eine Kategorie samt ihrem Namen in allen Bahnen entfernen.
mcp-tool-add-course = Eine Einzelbahn aus Start, Ziel und den Postencodes in Laufreihenfolge anlegen. Start und Ziel sind IDs von Sonderobjekten.
mcp-tool-remove-course = Eine Bahn samt ihrem Namen bei den Sonderobjekten entfernen.
mcp-tool-set-course-controls = Die Posten einer Bahn ersetzen. Der n-te Besuch eines Codes behält Knickpunkte, Abflaggung und Nummernlage des n-ten Besuchs davor.
mcp-tool-set-leg = Die Knickpunkte einer Strecke einer Einzelbahn setzen und festlegen, ob sie abgeflaggt ist: die Strecke ab dem Start (from_start) oder die nach einer Position (after, ab 1 gezählt wie auf der Karte, Querungsstellen eingeschlossen). Ohne bends verliert die Strecke ihre Knickpunkte.
mcp-tool-set-course-details = Kategorien, Anstieg oder geplanten Druckmaßstab einer Bahn ändern. Parameter, die du weglässt, bleiben, wie sie sind.
mcp-tool-place-numbers = Die Postennummern einer Bahn so setzen, dass sie Strecken, Kreise und andere Nummern frei lassen. Gibt zurück, wie viele Nummern es gesetzt hat. Schon gesetzte Nummern bleiben, außer overwrite ist true.
mcp-tool-map-features-near = Kartenobjekte im Umkreis radius_mm um eine Position, das nächste zuerst: Symbolnummer, Geometrieart (point, line, area), Abstand in mm und der Teil der Karte.
mcp-tool-map-extent = Kleinste und größte Ecke der Kartenobjekte in Kartenmillimetern, der Kartenmaßstab und die Kartenart.
mcp-tool-init-event = Eine Veranstaltung aus einer OOM-Karte (Sprint oder Schulhof) anlegen und ihre Projektdatei schreiben. Gibt den Pfad der Projektdatei zurück.

mcp-param-project = Projektdatei der Veranstaltung, ein absoluter Pfad oder einer mit ~ am Anfang, zum Beispiel ~/karten/halle.courseplan.toml
mcp-param-init-event-project = Wohin die Projektdatei kommt, ein absoluter Pfad oder einer mit ~ am Anfang [Standard: neben der Karte, nach ihr benannt mit .courseplan.toml]
mcp-param-map = Die OOM-Karte (Sprint oder Schulhof), ein absoluter Pfad oder einer mit ~ am Anfang
mcp-param-title = Titel der Veranstaltung [Standard: der Titel in freemapper.toml neben der Karte, sonst der Kartenname]
mcp-param-code = Code des Postens
mcp-param-add-control-code = Code des neuen Postens [Standard: die niedrigste freie Zahl ab 31]
mcp-param-at = Position in Kartenmillimetern, y zeigt nach unten
mcp-param-add-special-at = Position einer Einzelpunkt-Art (start, finish, first_aid, water)
mcp-param-x = X in Kartenmillimetern
mcp-param-y = Y in Kartenmillimetern, zeigt nach unten
mcp-param-crossing-point = Eine Querungsstelle anlegen, die keine Nummer bekommt [Standard: false]
mcp-param-c = Symbol für Spalte C (IOF-Symbol-ID)
mcp-param-d = Symbol für Spalte D (IOF-Symbol-ID)
mcp-param-e = Symbol für Spalte E (IOF-Symbol-ID)
mcp-param-f = Symbol für Spalte F (IOF-Symbol-ID)
mcp-param-g = Symbol für Spalte G (IOF-Symbol-ID)
mcp-param-h = Symbol für Spalte H (IOF-Symbol-ID)
mcp-param-set-control-number-clear = Zum Standardplatz zurückkehren; dann direction und gap_mm weglassen [Standard: false]
mcp-param-describe-control-clear = Zuerst alle Spalten leeren; Spalten im selben Aufruf werden danach gesetzt [Standard: false]
mcp-param-direction = Grad, 0 = Osten, gegen den Uhrzeigersinn
mcp-param-gap-mm = Abstand zum Kreis in Druckmillimetern
mcp-param-kind = Was das Objekt ist: start, finish, marked_route, out_of_bounds, forbidden_route, dangerous_area, first_aid oder water
mcp-param-id = ID des Objekts, eindeutig unter den Sonderobjekten
mcp-param-points = Die Punkte von marked_route, forbidden_route, out_of_bounds oder dangerous_area in Reihenfolge; jeder ist ein Objekt mit x und y in Kartenmillimetern
mcp-param-courses = Das Objekt auf diese Bahnen beschränken [Standard: alle Bahnen]; nicht für start und finish
mcp-param-name = Name der Bahn
mcp-param-add-class-name = Name der Kategorie
mcp-param-remove-class-name = Name der Kategorie
mcp-param-rule-category = Regelkategorie der Kategorie
mcp-param-expected-starters = Erwartete Zahl der Starter
mcp-param-start = ID des Sonderobjekts, das den Start bildet
mcp-param-finish = ID des Sonderobjekts, das das Ziel bildet
mcp-param-controls = Codes der Posten in Laufreihenfolge; eine Querungsstelle zählt als Position [Standard: keine]
mcp-param-classes = Kategorien, die die Bahn laufen
mcp-param-climb-m = Anstieg in Metern
mcp-param-print-scale = Geplanter Druckmaßstab, zum Beispiel 10000 für 1:10000
mcp-param-codes = Codes in Laufreihenfolge; eine leere Liste entfernt alle Posten
mcp-param-from-start = Die Strecke vom Start zum ersten Posten bearbeiten [Standard: false]
mcp-param-after = Die Strecke nach dieser Position bearbeiten, ab 1 gezählt wie auf der Karte; Querungsstellen zählen als Positionen
mcp-param-bends = Die Knickpunkte der Strecke in Reihenfolge, jeder ein Objekt mit x und y in Kartenmillimetern [Standard: keine, die Strecke verliert ihre Knickpunkte]
mcp-param-taped = Die Strecke ist abgeflaggt [Standard: false, die Abflaggung entfällt]
mcp-param-no-climb = Den Anstieg entfernen [Standard: false]
mcp-param-no-print-scale = Den geplanten Druckmaßstab entfernen [Standard: false]
mcp-param-overwrite = Schon gesetzte Nummern ersetzen [Standard: false]
mcp-param-radius-mm = Suchradius in Kartenmillimetern
mcp-param-limit = Höchstzahl der Objekte [Standard: 20, höchstens 200]
