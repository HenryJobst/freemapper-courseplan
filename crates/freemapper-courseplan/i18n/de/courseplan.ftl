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
