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
cli-edit-number-cleared = Nummernplatz von Posten { $code } zurückgesetzt
cli-edit-special-added-at = { $kind } { $id } bei { $x },{ $y } angelegt
cli-edit-special-added-points = { $kind } { $id } mit { $count } Punkten angelegt
cli-edit-special-removed = Sonderobjekt { $id } entfernt
cli-edit-class-added = Kategorie { $name } angelegt
cli-edit-class-removed = Kategorie { $name } entfernt
cli-edit-bad-point = { $value } ist kein Punkt. Schreibe X,Y in Kartenmillimetern, mit Punkt als Dezimaltrenner, zum Beispiel 12.5,-30
cli-edit-bad-number = { $value } ist keine Zahl. Nimm einen Punkt als Dezimaltrenner, zum Beispiel 0.5
cli-edit-bad-count = { $value } ist keine ganze Zahl
cli-edit-kind-needs-at = { $kind } ist ein einzelner Punkt. Gib --at X,Y an und kein --points
cli-edit-kind-needs-points = { $kind } wird durch Punkte gezeichnet. Gib --points X,Y X,Y ... an und kein --at
cli-edit-no-courses-for-kind = { $kind } gilt für alle Bahnen. Lass --courses weg

word-empty = leer
word-special-start = Start
word-special-finish = Ziel
word-special-marked-route = Markierte Strecke
word-special-out-of-bounds = Sperrgebiet
word-special-forbidden-route = Verbotene Strecke
word-special-dangerous-area = Gefahrenbereich
word-special-first-aid = Erste-Hilfe-Posten
word-special-water = Verpflegungsstelle

cli-help-control = Posten anlegen, verschieben, beschreiben, nummerieren und entfernen
cli-help-special = Sonderobjekte anlegen und entfernen (Start, Ziel, markierte Strecken, Flächen, Erste Hilfe, Wasser)
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
cli-arg-control-add-crossing-point = Einen Querungspunkt anlegen, der keine Nummer bekommt
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
cli-arg-special-add-at = Ort einer Einzelpunkt-Art (Start, Ziel, Erste Hilfe, Wasser): X,Y in Kartenmillimetern
cli-arg-special-add-points = Punkte einer Linien- oder Flächenart, als einzelne Argumente: X,Y X,Y ... (diese Option zuletzt angeben, damit negative Werte nicht als Optionen gelten)
cli-arg-special-add-courses = Das Objekt auf diese Bahnen beschränken, durch Kommas getrennt [Standard: alle Bahnen]
cli-arg-special-remove-id = ID des Sonderobjekts
cli-arg-class-add-name = Name der Kategorie
cli-arg-class-add-rule-category = Regelkategorie der Kategorie
cli-arg-class-add-starters = Erwartete Zahl der Starter
cli-arg-class-remove-name = Name der Kategorie
