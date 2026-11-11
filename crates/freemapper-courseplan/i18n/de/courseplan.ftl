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
word-name-control-code = Postencode
word-name-special-object-id = Sonderobjekt-ID
word-name-course-name = Bahnname
word-name-class-name = Kategoriename
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
error-model-blank-name = { $what } "{ $value }" ist leer oder beginnt oder endet mit einem Leerzeichen
error-model-blank-note = Die Begründung des akzeptierten Befunds { $name } ist leer
error-model-code-clashes-with-special = Postencode { $code } ist auch die ID eines Sonderobjekts
error-model-duplicate-course-in-special = Sonderobjekt { $id } nennt Bahn { $course } mehrfach
error-model-too-few-points = { $kind } { $id } braucht mindestens { $needed ->
    [one] { $needed } Punkt
   *[other] { $needed } Punkte
}
error-model-date-not-a-date = Das Veranstaltungsdatum { $date } ist kein Datum. Schreibe es als JJJJ-MM-TT, ohne Uhrzeit und Zeitzone
error-model-unsupported-format = Das Format { $format } der Projektdatei ist neuer, als dieses Programm versteht
error-model-unknown-key-top = Unbekannter Schlüssel { $key } auf oberster Ebene
error-model-unknown-key-in = Unbekannter Schlüssel { $key } in { $table }

## Projektdateien (ProjectError)

error-project-read = { $path } lässt sich nicht lesen: { $source }
error-project-write = { $path } lässt sich nicht schreiben: { $source }
error-project-changed = { $path } hat sich geändert, seit es gelesen wurde. Es wurde nichts geschrieben
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

## Regelprofile (RulesError)

error-rules-unknown-profile = Regelprofil { $id } gibt es nicht; verfügbar: { $list }
error-rules-not-a-base = Regelprofil { $id } ist ein Aufsatz, kein Basisprofil
error-rules-not-an-overlay = Regelprofil { $id } ist ein Basisprofil, kein Aufsatz
error-rules-overlay-for-another-base = Aufsatz { $id } ergänzt { $other }, nicht { $name }
error-rules-malformed = Regelprofil { $id } ist fehlerhaft: { $message }
error-rules-unknown-key = Prüfregel { $name } in Regelprofil { $id } hat den unbekannten Schlüssel { $key }
error-rules-unknown-rule = Aufsatz { $id } schaltet Prüfregel { $name } ab, die das Basisprofil nicht hat
error-rules-ambiguous = Mehr als eine aktive Prüfregel liefert { $key } { $value }: { $list }

## Bearbeiten (EditError)

error-edit-invalid = Mit der Änderung wäre die Veranstaltung widersprüchlich: { $errors }
error-edit-not-found = { $what } { $id } gibt es nicht
error-edit-code-taken = Posten { $code } gibt es schon
error-edit-id-taken = Sonderobjekt { $id } gibt es schon
error-edit-outside-map = Der Punkt ({ $x }, { $y }) liegt außerhalb der Karte, die von ({ $min_x }, { $min_y }) bis ({ $max_x }, { $max_y }) reicht
error-edit-in-use = { $id } wird noch von { $courses } verwendet. Ändere zuerst diese Bahnen
error-edit-not-supported = Bahn { $course } ist keine Einzelbahn. Bisher lassen sich nur Einzelbahnen bearbeiten
error-edit-bad-index = Bahn { $course } hat keinen Posten an Position { $index }
error-edit-special-would-cover-all-courses = { $count ->
    [one] Bahn { $course } ist die einzige Bahn von { $specials }; ohne sie gälte { $specials } für alle Bahnen. Ordne { $specials } zuerst einer anderen Bahn zu oder lösche es
   *[other] Bahn { $course } ist die einzige Bahn von { $specials }; ohne sie gälten diese Sonderobjekte für alle Bahnen. Ordne sie zuerst einer anderen Bahn zu oder lösche sie
}
error-edit-start-or-finish-has-no-courses = Sonderobjekt { $id } ist ein Start oder Ziel; jede Bahn nennt ihren Start und ihr Ziel selbst
error-edit-unknown-rule = Prüfregel { $name } ist im Regelprofil { $id } nicht aktiv
error-edit-finding-needs-place = Befunde der Prüfregel { $name } sitzen an { $places }; nenne diesen Ort
error-edit-finding-has-one-place = Ein Befund der Prüfregel { $name } hat einen Ort: gib eine Bahn, Posten oder Strecken einer Bahn an
error-edit-finding-wrong-place = Befunde der Prüfregel { $name } betreffen { $places }, nie { $place }
error-edit-accepted-not-found = An diesem Ort gibt es keinen akzeptierten Befund der Prüfregel { $name }
error-edit-start-window-zero = Das Startfenster muss mindestens 1 Minute lang sein
error-edit-blank-rule-category = Die Regelkategorie der Kategorie { $name } ist leer; gib eine an oder entferne sie

## OOM-Karten lesen (OmapError)

error-omap-xml = Ungültiges OOM-XML: { $message }
error-omap-missing-element = Das Element <{ $name }> fehlt
error-omap-io = Lesefehler: { $message }
error-omap-binary-format = Diese Datei hat das alte Binärformat von OpenOrienteering Mapper, das dieses Programm nicht lesen kann. Öffne sie in Mapper und speichere sie als aktuelle .omap- oder .xmap-Datei neu.
error-omap-unsupported = { $message }

## Prüfen (check)

check-severity-error = Fehler
check-severity-warning = Warnung
check-severity-hint = Hinweis
check-location-event = Veranstaltung
check-location-course = Bahn { $course }
check-location-control = Posten { $code }
check-location-controls = Posten { $codes }
check-location-start = Start { $id }
check-location-finish = Ziel { $id }
check-location-leg = Bahn { $course }, Strecke { $leg }
check-location-legs = Bahn { $course }, Strecken { $legs }
check-leg-end-start = Start
check-leg-end-finish = Ziel
check-skip-training = Beim Training wird die Prüfregel nicht angewendet
check-skip-discipline = Die Prüfregel gilt nicht für { $discipline ->
        [sprint] den Sprint
        [middle] die Mitteldistanz
        [long] die Langdistanz
        [ultralong] die Ultralangdistanz
       *[other] { $discipline }
    }
check-skip-map-kind = Die Prüfregel gilt nicht für { $kind ->
        [sprint] Sprintkarten
        [schoolyard] Schulhofkarten
        [forest] Waldkarten
       *[other] Karten der Art { $kind }
    }
check-skip-print-scale = Keine Prüfregel dieser Art für den Druckmaßstab 1:{ $scale }
check-skip-course-kind = { $kind ->
        [team] Mannschaftsbahnen
        [relay] Staffelbahnen
        [score] Score-Bahnen
       *[other] Bahnen der Art { $kind }
    } werden noch nicht geprüft
check-skip-course-unresolved = Die Bahn nennt { $id }, das die Veranstaltung nicht definiert; sie wird nicht geprüft
check-skip-class-not-mapped = Kategorie { $name } ist keiner Regelkategorie zugeordnet; class set --rule-category ordnet ihr eine zu
check-skip-climb-missing = Für die Bahn ist kein Anstieg angegeben
check-skip-value-missing = Der Prüfregel fehlt der Wert { $name }
check-skip-no-map-extent = Die Karte enthält keine Objekte und hat daher keinen Rand
check-skip-start-window-missing = Die Veranstaltung hat kein Startfenster; event start-window setzt es
check-skip-starters-missing = Kategorien ohne erwartete Starterzahl: { $classes }; class set --starters setzt sie
check-skip-length-no-reference = Keine Bahn gehört zur Kategorie { $category }, daher fehlt der Längentabelle die Bezugsbahn
check-skip-length-reference-empty = Die Bezugsbahn { $course } hat die Länge 0
check-skip-category-not-in-table = Kategorien, die in keiner Zeile der Tabelle stehen: { $categories }
check-skip-switched-off = Das Regelprofil { $overlay } schaltet die Prüfregel ab
check-distance-min = { $first_kind ->
        [start] Start
        [finish] Ziel
        [crossing] Querungsstelle
       *[control] Posten
    } { $first } und { $second_kind ->
        [start] Start
        [finish] Ziel
        [crossing] Querungsstelle
       *[control] Posten
    } { $second } liegen { $metres } m auseinander, gefordert sind mindestens { $limit } m
check-distance-similar = Posten { $first } und { $second } stehen an ähnlichen Objekten ({ $first_symbol }, { $second_symbol }) und liegen { $metres } m auseinander, gefordert sind mindestens { $limit } m
check-map-edge = { $kind ->
        [start] Start
        [finish] Ziel
        [crossing] Querungsstelle
       *[control] Posten
    } { $id } liegt { $metres } m vom Kartenrand entfernt, gefordert sind mindestens { $limit } m
check-leg-through-control = { $metres ->
        [0] Die Strecke führt über { $stop_kind ->
            [start] den Start
            [finish] das Ziel
            [crossing] die Querungsstelle
           *[control] den Posten
        } { $stop }
       *[other] Die Strecke führt { $metres } m { $stop_kind ->
            [start] am Start
            [finish] am Ziel
            [crossing] an der Querungsstelle
           *[control] am Posten
        } { $stop } vorbei, innerhalb des Kreisradius von { $limit } m
    }
check-leg-out-of-bounds = Die gerade Strecke quert das Sperrgebiet { $id }; die Optimalroute könnte hindurchführen
check-leg-out-of-bounds-symbol = Die gerade Strecke quert das Kartensymbol { $code } (Sperrgebiet); die Optimalroute könnte hindurchführen
check-acute-angle = Die Strecken { $stop_kind ->
        [crossing] an der Querungsstelle
       *[control] am Posten
    } { $stop } bilden einen Winkel von { $degrees }°, gefordert sind mindestens { $limit }°
check-straight-angle = Die Strecken am Posten { $stop } bilden einen Winkel von { $degrees }°, erwünscht sind höchstens { $limit }°
check-number-gap-small = Die Nummer von Posten { $stop } steht { $gap } mm vom Kreis entfernt, erwünscht sind mindestens { $limit } mm
check-number-gap-large = Die Nummer von Posten { $stop } steht { $gap } mm vom Kreis entfernt, erwünscht sind höchstens { $limit } mm
check-opposing-legs = Die beiden Strecken verlaufen auf { $metres } m gegenläufig in höchstens { $limit } m Abstand, erlaubt sind weniger als { $min_length } m
check-code-range = Der Code { $code } liegt außerhalb des erlaubten Bereichs { $min } bis { $max }
check-code-not-numeric = Der Code { $code } ist keine Zahl
check-code-pair = Die Codes { $first } und { $second } dürfen nicht beide in einer Veranstaltung vorkommen
check-code-confusable-rotated = Die Codes { $first } und { $second } sehen auf dem Kopf gleich aus und folgen aufeinander
check-code-confusable-six-nine = Die Codes { $first } und { $second } unterscheiden sich nur durch 6 und 9 und folgen aufeinander
check-same-objects-in-circle = Der Posten { $code } steht an Symbol { $symbol }, das im Postenkreis noch einmal vorkommt, und Spalte C der Postenbeschreibung ist leer
check-start-capacity = Für die Starter pro Minute braucht es { $needed } verschiedene erste Posten, belegt sind aber nur { $available }
check-start-capacity-shared = Für die Starter pro Minute braucht es { $needed } verschiedene erste Posten, belegt sind aber nur { $available }; erste Posten mehrerer Bahnen: { $shared }
check-first-leg-angle = Die ersten Strecken vom Start { $start } zu den Posten { $first } und { $second } unterscheiden sich in der Richtung nur um { $degrees }°, gefordert sind mindestens { $limit }°
check-length-table = Die Bahnlänge liegt außerhalb der Tabelle: { $rows }
check-length-table-row = { $categories }: { $percent } % der { $reference }-Bahn, erlaubt sind { $min } bis { $max } %
check-length-table-reference-twice = Die Bahnen { $courses } gehören alle zur Kategorie { $category }; Bahn { $first } dient als Bezug für die Längentabelle
check-difficulty-mix = Die Bahn mischt leichte Kategorien ({ $easy }) mit schweren ({ $hard })
check-nor-lkm = Die Bahn hat { $actual } Lkm ({ $length } km Länge, { $climb } m Anstieg) und liegt außerhalb der Empfehlung: { $rows }
check-nor-lkm-row = { $categories }: Ziel-Lkm { $target }, erlaubt sind { $min } bis { $max } Lkm
check-source-claude = Vorschlag von freemapper-courseplan

## Kommandozeile

cli-error-prefix = Fehler
cli-error-unknown-lang = Unbekannte Sprache { $value }. Verfügbar: { $available }

cli-info-title = Titel: { $value }
cli-info-map = Karte: { $value }
cli-info-map-kind = Kartenart: { $value }
cli-info-controls = Posten: { $count }, Querungsstellen: { $crossings }
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
cli-arg-help-command = Der Befehl, dessen Hilfe angezeigt wird, mit seinen Unterbefehlen, zum Beispiel control add
cli-parse-hint = Mehr dazu mit '--help'.
cli-parse-unknown-argument = Unbekanntes Argument { $arg }
cli-parse-missing-required = Pflichtargument fehlt: { $args }
cli-parse-invalid-value = Ungültiger Wert { $value } für { $arg }
cli-parse-invalid-value-choices = Ungültiger Wert { $value } für { $arg }. Mögliche Werte: { $values }
cli-help-possible-values = mögliche Werte
cli-parse-invalid-subcommand = Unbekannter Befehl { $value }
cli-parse-missing-subcommand = Ein Befehl ist nötig
cli-parse-too-many-values = Unerwarteter Wert { $value } für { $arg }
cli-parse-argument-conflict = { $arg } lässt sich nicht zusammen mit { $other } verwenden

## Bearbeiten

cli-edit-control-added = Posten { $code } bei { $x },{ $y } angelegt
cli-edit-crossing-point-added = Querungsstelle { $code } bei { $x },{ $y } angelegt
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
cli-edit-special-courses = Sonderobjekt { $id } gehört jetzt zu { $courses }
cli-edit-special-all-courses = Sonderobjekt { $id } gilt jetzt für alle Bahnen
cli-edit-no-course-names = Gib mindestens einen Bahnnamen an, oder --all für alle Bahnen
cli-edit-class-added = Kategorie { $name } angelegt
cli-edit-class-removed = Kategorie { $name } entfernt
cli-edit-class-set = Kategorie { $name } geändert
cli-edit-bad-point = { $value } ist kein Punkt. Schreibe X,Y in Kartenmillimetern, mit Punkt als Dezimaltrenner, zum Beispiel 12.5,-30
cli-edit-bad-number = { $value } ist keine Zahl. Nimm einen Punkt als Dezimaltrenner, zum Beispiel 0.5
cli-edit-bad-count = { $value } ist keine ganze Zahl
cli-edit-bad-gap = { $value } ist kein brauchbarer Abstand. Gib mehr als 0 und höchstens { $max } Druckmillimeter an
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
cli-help-special = Sonderobjekte anlegen, Bahnen zuordnen und entfernen (Start, Ziel, Pflichtstrecken, Flächen, Erste-Hilfe- und Verpflegungsstellen)
cli-help-class = Kategorien anlegen, ändern und entfernen
cli-help-control-add = Einen Posten anlegen
cli-help-control-move = Einen Posten verschieben
cli-help-control-remove = Einen Posten und seine Besuche in Einzelbahnen entfernen
cli-help-control-describe = Die Beschreibungssymbole eines Postens setzen
cli-help-control-number = Den Platz der Postennummer setzen oder zurücksetzen
cli-help-special-add = Ein Sonderobjekt anlegen
cli-help-special-remove = Ein Sonderobjekt entfernen
cli-help-special-courses = Festlegen, zu welchen Bahnen ein Sonderobjekt gehört
cli-help-class-add = Eine Kategorie anlegen
cli-help-class-remove = Eine Kategorie samt ihrem Namen in allen Bahnen entfernen
cli-help-class-set = Regelkategorie oder erwartete Starterzahl einer Kategorie ändern. Gib mindestens eine Option an

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
cli-arg-control-number-direction = Grad, 0 = Osten, gegen den Uhrzeigersinn; jeder Wert wird auf 0 bis 360 umgerechnet
cli-arg-control-number-gap = Abstand zwischen Kreislinie und Nummer in Druckmillimetern, mehr als 0 und höchstens 10
cli-arg-control-number-clear = Zum Standardplatz zurückkehren
cli-arg-special-add-kind = Was das Objekt ist
cli-arg-special-add-id = ID des Objekts, eindeutig unter den Sonderobjekten
cli-arg-special-add-at = Ort einer Einzelpunkt-Art (Start, Ziel, Erste-Hilfe-Stelle, Verpflegungsstelle): X,Y in Kartenmillimetern
cli-arg-special-add-point = Ein Punkt einer Linien- oder Flächenart: X,Y in Kartenmillimetern; die Option je Punkt wiederholen, in der Reihenfolge
cli-arg-special-add-courses = Das Objekt auf diese Bahnen beschränken, durch Kommas getrennt [Standard: alle Bahnen]
cli-arg-special-remove-id = ID des Sonderobjekts
cli-arg-special-courses-id = ID des Sonderobjekts; kein Start und kein Ziel
cli-arg-special-courses-courses = Die Bahnen, durch Kommas getrennt
cli-arg-special-courses-all = Das Objekt für alle Bahnen gelten lassen
cli-arg-class-add-name = Name der Kategorie
cli-arg-class-add-rule-category = Regelkategorie der Kategorie
cli-arg-class-add-starters = Erwartete Zahl der Starter
cli-arg-class-remove-name = Name der Kategorie
cli-arg-class-set-name = Name der Kategorie
cli-arg-class-set-rule-category = Die Regelkategorie, etwa H21 oder D-14; die Tabellenprüfungen lesen sie
cli-arg-class-set-no-rule-category = Die Regelkategorie entfernen
cli-arg-class-set-starters = Erwartete Zahl der Starter; die Prüfung der Startkapazität liest sie
cli-arg-class-set-no-starters = Die erwartete Starterzahl entfernen

## Bahnbefehle

cli-info-course = Bahn { $name }: { $controls } Posten, { $km } km, Anstieg { $climb } m
cli-info-course-unmeasured = Bahn { $name }: { $kind }, Länge noch nicht unterstützt
word-decimal-point = ,
word-taped = abgeflaggt
word-not-taped = nicht abgeflaggt
word-place-event = die ganze Veranstaltung
word-place-course = eine Bahn
word-place-control = einen Posten
word-place-controls = zwei oder mehr Posten
word-place-leg = eine Strecke
word-place-legs = zwei oder mehr Strecken einer Bahn
word-or = oder
word-and = und
cli-edit-course-added = Bahn { $name } angelegt
cli-edit-course-removed = Bahn { $name } entfernt
cli-edit-course-controls = Bahn { $name } führt jetzt über { $codes }
cli-edit-course-set = Bahn { $name } geändert
cli-edit-course-numbers = { $count ->
    [one] 1 Postennummer auf Bahn { $name } gesetzt
   *[other] { $count } Postennummern auf Bahn { $name } gesetzt
}
cli-edit-course-numbers-none = Keine Postennummer auf Bahn { $name } gesetzt, weil jeder Posten schon seine Nummer hat; --overwrite setzt die Nummernlagen der Bahn neu
cli-edit-course-leg-start = Strecke vom Start der Bahn { $name } gesetzt: Knickpunkte { $bends }, { $taped }
cli-edit-course-leg-after = Strecke nach Position { $index } der Bahn { $name } gesetzt: Knickpunkte { $bends }, { $taped }
cli-edit-course-number-set = Bahn { $name }, Position { $index }: Nummer in Richtung { $direction } Grad, Abstand { $gap } mm
cli-edit-course-number-cleared = Bahn { $name }, Position { $index }: Nummernlage zurückgesetzt
cli-edit-leg-cleared = Bahn { $name }: Knickpunkte und Abflaggung der Strecke nach Position { $index } entfernt, weil sie jetzt zu einem anderen Bahnpunkt führt
cli-edit-start-leg-cleared = Bahn { $name }: Knickpunkte und Abflaggung der Strecke vom Start entfernt, weil sie jetzt zu einem anderen Bahnpunkt führt

cli-help-course = Einzelbahnen anlegen, ändern, nummerieren und entfernen
cli-help-course-add = Eine Einzelbahn anlegen
cli-help-course-controls = Die Posten einer Bahn ersetzen
cli-help-course-remove = Eine Bahn samt ihrem Namen in den Sonderobjekten entfernen
cli-help-course-leg = Die Knickpunkte einer Strecke setzen und festlegen, ob sie abgeflaggt ist
cli-help-course-set = Kategorien, Anstieg oder geplanten Druckmaßstab einer Bahn ändern
cli-help-course-number = Die Nummernlage eines Postens in einer Bahn setzen oder zurücksetzen; sie geht der zentralen und der automatischen Lage vor
cli-help-course-numbers = Die Postennummern einer Bahn hinter dem Kreis platzieren, von der ankommenden Strecke aus gesehen, frei von Strecken, Kreisen sowie Punkt- und Liniensymbolen der Karte. Läuft nur auf Aufruf, also nach Änderungen an Strecken erneut aufrufen; zentrale Nummernlagen und die der Bahn bleiben. Die Nummer rückt bis zum größten Abstand des Regelprofils nach außen, bevor sie von der bevorzugten Richtung abweicht

cli-arg-edit-course = Name der Bahn
cli-arg-course-add-start = ID des Start-Sonderobjekts
cli-arg-course-add-finish = ID des Ziel-Sonderobjekts
cli-arg-course-add-controls = Codes der Posten in Laufreihenfolge, durch Kommas getrennt; eine Querungsstelle zählt als Position [Standard: keine]
cli-arg-course-add-classes = Kategorien, die die Bahn laufen, durch Kommas getrennt
cli-arg-course-add-climb = Anstieg in Metern
cli-arg-course-add-print-scale = Geplanter Druckmaßstab, zum Beispiel 3000 für 1:3000
cli-arg-course-controls-codes = Codes in Laufreihenfolge, durch Kommas getrennt. Der n-te Besuch eines Codes behält die Nummernlage des bisherigen n-ten Besuchs; seine Strecke behält Knickpunkte und Abflaggung nur, wenn sie weiter zum selben Bahnpunkt führt
cli-arg-course-leg-from-start = Die Strecke vom Start zum ersten Posten bearbeiten
cli-arg-course-leg-after = Die Strecke nach dieser Position bearbeiten, ab 1 gezählt wie auf der Karte; Querungsstellen zählen als Positionen
cli-arg-course-leg-bend = Ein Knickpunkt der Strecke: X,Y in Kartenmillimetern; die Option je Knickpunkt wiederholen, in der Reihenfolge. Ohne --bend verliert die Strecke ihre Knickpunkte
cli-arg-course-leg-taped = Die Strecke ist abgeflaggt. Ohne --taped gilt sie als nicht abgeflaggt
cli-arg-course-set-classes = Kategorien, die die Bahn laufen, durch Kommas getrennt
cli-arg-course-set-climb = Anstieg in Metern
cli-arg-course-set-no-climb = Den Anstieg entfernen
cli-arg-course-set-print-scale = Geplanter Druckmaßstab, zum Beispiel 3000 für 1:3000
cli-arg-course-set-no-print-scale = Den geplanten Druckmaßstab entfernen
cli-arg-course-number-after = Position des Postens in der Bahn, ab 1 gezählt wie auf der Karte; Querungsstellen zählen als Positionen
cli-arg-course-number-direction = Grad, 0 = Osten, gegen den Uhrzeigersinn; jeder Wert wird auf 0 bis 360 umgerechnet
cli-arg-course-number-gap = Abstand zwischen Kreislinie und Nummer in Druckmillimetern, mehr als 0 und höchstens 10
cli-arg-course-number-clear = Die Nummernlage der Bahn entfernen, sodass wieder die zentrale oder die automatische Lage gilt
cli-arg-course-numbers-overwrite = Auch früher gesetzte Nummernlagen der Bahn ersetzen; Posten mit zentraler Nummernlage behalten sie [Standard: behalten]

## Prüfbefehle

cli-help-check = Die Einzelbahnen gegen das Regelprofil und seine Aufsätze prüfen
cli-help-check-exit-codes =
    Exit-Code 0: kein Befund der Schwere Fehler; akzeptierte Befunde zählen nicht.
    Exit-Code 1: mindestens ein Befund der Schwere Fehler, oder die Prüfung wurde abgelehnt, etwa weil Projektdatei oder Karte nicht lesbar sind.
    Exit-Code 2: falscher Aufruf, etwa eine unbekannte Option.
cli-arg-check-all = Auch die akzeptierten Befunde mit Begründung, die nicht zugeordneten Ausnahmen, die übersprungenen Prüfregeln mit dem Grund und unter jedem Befund die Anmerkung seiner Prüfregel zeigen
cli-check-group-event = Veranstaltung:
cli-check-group-course = Bahn { $course }:
cli-check-heading-accepted = akzeptiert:
cli-check-heading-skipped = übersprungen:
cli-check-heading-unmatched = nicht zugeordnet:
cli-check-unmatched-hint = Diese Ausnahmen passen zu keinem Befund; finding remove entfernt eine, die nicht mehr gebraucht wird
cli-check-note = Begründung: { $note }
cli-check-rule-note = Zur Prüfregel: { $note }
cli-check-summary = { $errors } Fehler, { $warnings ->
        [one] 1 Warnung
       *[other] { $warnings } Warnungen
    }, { $hints ->
        [one] 1 Hinweis
       *[other] { $hints } Hinweise
    }; { $accepted } akzeptiert, { $unmatched ->
        [0] {""}
       *[other] { $unmatched } nicht zugeordnet,{" "}
    }{ $skipped } übersprungen
cli-check-show-all = check --all zeigt auch die akzeptierten Befunde, die nicht zugeordneten Ausnahmen und die übersprungenen Prüfregeln
cli-info-rule-profile = Regelprofil: { $value }
cli-info-rule-profile-overlays = Regelprofil: { $value }, { $count ->
        [one] Aufsatz
       *[other] Aufsätze
    }: { $list }

cli-help-finding = Befunde als gewollte Ausnahme akzeptieren oder wieder melden lassen
cli-help-finding-accept = Einen Befund mit Begründung als gewollte Ausnahme akzeptieren. Gib seinen Ort so an, wie check ihn zeigt: --controls, --leg oder --course, oder nichts davon für einen Befund der ganzen Veranstaltung. Wer denselben Befund noch einmal akzeptiert, ersetzt die Begründung
cli-help-finding-remove = Einen akzeptierten Befund entfernen, damit check ihn wieder meldet. Gib ihn so an wie beim Akzeptieren
cli-arg-finding-accept-rule = Kennung der Prüfregel, wie check sie zeigt, zum Beispiel wkb2026.distance.min
cli-arg-finding-accept-controls = Die Posten des Befunds, durch Kommas getrennt
cli-arg-finding-accept-leg = Die Strecke des Befunds als Bahn:N, die Strecke, die von Position N der Bahn abgeht: Die Positionen zählen die Posten der Bahn ab 1, Querungsstellen mitgezählt, und 0 ist der Start. N ist nicht die Postennummer auf der Karte; check nennt die Strecke nach ihren Enden, zum Beispiel 36–37. Für einen Befund an zwei Strecken wiederholen: --leg A:2 --leg A:5
cli-arg-finding-accept-course = Die Bahn eines Befunds, der die ganze Bahn betrifft
cli-arg-finding-accept-note = Warum die Ausnahme gewollt ist
cli-arg-finding-remove-rule = Kennung der Prüfregel, wie check sie zeigt, zum Beispiel wkb2026.distance.min
cli-arg-finding-remove-controls = Die Posten des Befunds, durch Kommas getrennt
cli-arg-finding-remove-leg = Die Strecke des Befunds als Bahn:N, die Strecke, die von Position N der Bahn abgeht: Die Positionen zählen die Posten der Bahn ab 1, Querungsstellen mitgezählt, und 0 ist der Start. N ist nicht die Postennummer auf der Karte; check nennt die Strecke nach ihren Enden, zum Beispiel 36–37. Für einen Befund an zwei Strecken wiederholen: --leg A:2 --leg A:5. Entfernt wird zuerst die Ausnahme für die Strecke, wie sie jetzt verläuft; gibt es keine, die an dieser Position, deren Strecke sich seitdem geändert hat
cli-arg-finding-remove-course = Die Bahn eines Befunds, der die ganze Bahn betrifft
cli-finding-bad-leg = { $value } ist keine Strecke; schreibe Bahn:N, zum Beispiel A:3, wobei N eine Position in der Bahn und 0 der Start ist
cli-finding-at = bei { $location }
cli-finding-at-event = für die Veranstaltung
cli-finding-accepted = Befund { $rule } { $location } akzeptiert
cli-finding-note-replaced = Begründung des akzeptierten Befunds { $rule } { $location } ersetzt
cli-finding-removed = Akzeptierten Befund { $rule } { $location } entfernt

cli-help-event = Einstellungen der ganzen Veranstaltung: Regelprofil und Startfenster
cli-help-event-profile = Basis-Regelprofil und Aufsätze setzen. Die Profile werden vorher geladen; ein unbekanntes wird mit der Liste der verfügbaren Profile abgelehnt. Ohne --overlay hat die Veranstaltung keine Aufsätze
cli-arg-event-profile-base = Das Basis-Regelprofil, zum Beispiel wkb-2026
cli-arg-event-profile-overlay = Ein Aufsatz auf das Basisprofil, zum Beispiel nor-bb-2021; für mehrere wiederholen, sie gelten in dieser Reihenfolge
cli-help-event-start-window = Das Startfenster setzen, die Minuten vom ersten bis zum letzten Start. Die Prüfung der Startkapazität teilt die erwarteten Starter dadurch
cli-arg-event-start-window-minutes = Minuten vom ersten bis zum letzten Start, mindestens 1
cli-event-profile-set = Regelprofil { $value } gesetzt
cli-event-start-window-set = Startfenster auf { $minutes ->
        [one] 1 Minute
       *[other] { $minutes } Minuten
    } gesetzt
cli-event-profile-set-overlays = Regelprofil { $value } mit { $count ->
        [one] dem Aufsatz
       *[other] den Aufsätzen
    } { $list } gesetzt

## Exportbefehle

cli-help-export = Die Bahnen für andere Programme in eine Datei schreiben
cli-arg-export-project = Die Projektdatei
cli-arg-export-iof-xml = Die Einzelbahnen als IOF-XML-3.0-CourseData für Auswertungsprogramme wie OE schreiben; eine vorhandene Datei wird ersetzt
export-iof-xml-written = { $count ->
    [one] 1 Bahn nach { $path } geschrieben
   *[other] { $count } Bahnen nach { $path } geschrieben
}
export-iof-xml-skipped = { $course } ausgelassen: Bahnart { $kind } wird nicht nach IOF XML exportiert
export-iof-xml-no-geo = die Karte hat keine nutzbare Georeferenzierung, die Posten tragen daher keine Breite und Länge, nur Kartenpositionen
export-nothing-to-do = nichts zu exportieren; nenne ein Format, zum Beispiel --iof-xml DATEI oder --ppen DATEI
error-export-iof-xml-course = Bahn { $course } kann nicht exportiert werden: { $source }
error-export-overwrites-input = { $path } ist die Projektdatei oder ihre Karte, der Export würde sie überschreiben; nenne eine andere Datei
cli-arg-export-ppen = Die Einzelbahnen in ein Purple-Pen-Event schreiben. Der Export ändert eine vorhandene Datei an Ort und Stelle und behält alles, was das Projekt nicht geändert hat. Eine fehlende Datei legt er an und verknüpft das Projekt mit ihr, wenn es noch kein sources.ppen hat. Die Purple-Pen-Kennungen des Projekts gelten nur für die verknüpfte Datei
cli-arg-export-new = Mit --ppen die Datei von Grund auf schreiben und eine vorhandene ersetzen; ein Projekt ohne sources.ppen wird mit ihr verknüpft
export-ppen-written = { $count ->
    [one] 1 Bahn in das neue Purple-Pen-Event { $path } geschrieben
   *[other] { $count } Bahnen in das neue Purple-Pen-Event { $path } geschrieben
}
export-ppen-patched = { $count ->
    [one] 1 Bahn in das Purple-Pen-Event { $path } geschrieben
   *[other] { $count } Bahnen in das Purple-Pen-Event { $path } geschrieben
}
export-ppen-note-line = Hinweis: { $note }
export-ppen-ids-stored = { $count ->
    [one] 1 Objekt des Projekts, das keine hatte, hat eine Purple-Pen-Kennung bekommen
   *[other] { $count } Objekte des Projekts, die keine hatten, haben eine Purple-Pen-Kennung bekommen
}
export-ppen-note-course-skipped = { $course } ausgelassen: Bahnart { $kind } wird nicht nach Purple Pen exportiert
export-ppen-note-special-skipped = Sonderobjekt { $id } ausgelassen: { $why }
export-ppen-note-symbol-without-id = Posten { $control }, Spalte { $column }: Symbol { $id } hat in Purple Pen keine Entsprechung und wird ausgelassen
export-ppen-note-symbol-not-in-standard = Posten { $control }, Spalte { $column }: Symbol { $id } gehört nicht zum Beschreibungsstandard { $standard } der Datei, es wird ausgelassen
export-ppen-note-symbol-wrong-column = Posten { $control }, Spalte { $column }: Purple Pen bietet das Symbol { $id } in dieser Spalte nicht an, es wird ausgelassen
export-ppen-note-symbol-variant-chosen = Posten { $control }, Spalte { $column }: Purple Pen kennt für { $id } mehrere Symbole; in die Datei kommt { $written }
export-ppen-note-code-not-stored = der Code „{ $code }“ hat Leerzeichen am Rand; beim erneuten Einlesen fallen sie weg, und der Posten kommt mit einem anderen Code zurück
export-ppen-note-flagging-implied = Bahn { $course }, Strecke { $from } nach { $to }: Purple Pen flaggt diese Strecke von selbst, obwohl das Projekt sie nicht abgeflaggt hat
export-ppen-note-partial-flagging-dropped = Bahn { $course }, Strecke { $from } nach { $to }: die Knickpunkte haben sich geändert und die Flaggung beginnt oder endet nicht mehr an einem Knickpunkt, daher entfällt die Teilflaggung
export-ppen-kept-control = Posten { $code } (Purple-Pen-Kennung { $id }) steht in der Datei, aber nicht im Projekt; der Export behält ihn, weil er nie löscht
export-ppen-kept-course = Bahn { $name } (Purple-Pen-Kennung { $id }) steht in der Datei, aber nicht im Projekt; der Export behält sie, weil er nie löscht
export-ppen-kept-special = Sonderobjekt der Art { $kind } (Purple-Pen-Kennung { $id }) steht in der Datei, aber nicht im Projekt; der Export behält es, weil er nie löscht
error-export-ppen-base = das vorhandene Purple-Pen-Event { $path } lässt sich nicht ändern: { $source }
error-export-ppen-unknown-version = das Purple-Pen-Event stammt aus einer Version, die dieses Programm nicht kennt ({ $version }); es wurde nichts geschrieben
error-export-ppen-unsupported = die Bahnen lassen sich nicht in ein Purple-Pen-Event schreiben: { $what }
error-export-ppen-invalid = die Bahnen ergeben kein gültiges Purple-Pen-Event: { $what }
error-export-ppen-shared-leg = die Bahnen { $first } und { $second } zeichnen die Strecke { $from } nach { $to } verschieden, und Purple Pen kennt je Postenpaar nur eine Strecke
error-export-ppen-shared-circle-gaps = die Bahnen { $first } und { $second } geben Posten { $control } bei 1:{ $scale } verschiedene Kreislücken
error-export-ppen-limit = zu groß für ein Purple-Pen-Event: { $what }
error-export-ppen-circle-gap-limit = zu groß für ein Purple-Pen-Event: die Bahnen würden mehr als { $limit } Kreislücken aus der vorhandenen Datei übernehmen
error-export-ppen-ids-not-stored = { $path } geschrieben, aber das Projekt ließ sich nicht speichern und hat daher die Purple-Pen-Kennungen dieser Datei nicht: { $source }
error-export-same-target = --iof-xml und --ppen nennen dieselbe Datei { $path }; nenne zwei verschiedene Dateien
export-ppen-linked = sources.ppen nennt jetzt { $path }
word-export-why-marked-route = Purple Pen hat kein Objekt für die abgeflaggte Strecke, die Flaggung gehört dort zu den Strecken
word-export-why-forbidden-route-with-several-points = Purple Pen zeichnet eine verbotene Route nur als einzelnes Kreuz
word-export-why-no-course-left = es gehört nur zu Bahnen, die ausgelassen wurden

## Purple-Pen-Import (init --ppen, import --ppen)

cli-help-import = Posten und Bahnen eines Purple-Pen-Events (.ppen) übernehmen
cli-arg-import-project = Die Projektdatei
cli-arg-import-ppen = Das zu importierende Purple-Pen-Event; es muss für die Karte des Projekts gezeichnet sein
cli-arg-import-replace = Die Posten, Bahnen und Sonderobjekte ersetzen, die das Projekt schon hat; Kategorien und akzeptierte Befunde bleiben
cli-arg-init-ppen = Von einem Purple-Pen-Event (.ppen) ausgehen, das für diese Karte gezeichnet ist
import-ppen-done = { $controls ->
    [one] 1 Posten
   *[other] { $controls } Posten
} und { $courses ->
    [one] 1 Bahn
   *[other] { $courses } Bahnen
} aus { $path } übernommen
import-ppen-note-line = Hinweis: { $note }
import-ppen-needs-replace = { $path } hat schon Posten, Bahnen oder Sonderobjekte; mit --replace erneut aufrufen, um sie durch den Inhalt des Purple-Pen-Events zu ersetzen
error-import-ppen-read = { $path } lässt sich nicht lesen: { $source }
error-ppen-not-a-file = { $path } ist keine Datei
error-ppen-too-large = { $path } ist größer als { $limit } MB, die Grenze für ein Purple-Pen-Event
error-import-ppen-scale-mismatch = das Purple-Pen-Event ist für 1:{ $file } gezeichnet, die Karte hat 1:{ $map }; die Bahnen für den Maßstab der Karte exportieren
error-import-ppen-map-scale-unknown = das Purple-Pen-Event ist für 1:{ $file } gezeichnet, und die Karte nennt keinen Maßstab; den Maßstab in der Kartendatei setzen
error-import-ppen-outside-map = nur { $inside } von { $total } Posten liegen auf der Karte; das Event gehört zu einer anderen Karte
error-import-ppen-invalid = das Purple-Pen-Event lässt sich nicht lesen: { $source }
error-import-ppen-fit = das Purple-Pen-Event passt nicht zur Karte: { $source }
mcp-error-not-ppen = { $path } ist kein Purple-Pen-Event; diese Dateien enden auf .ppen
mcp-error-ppen-symlink = Der Verweis sources.ppen des Projekts nennt { $path }, einen symbolischen Link. Der Server folgt keinen symbolischen Links.
mcp-error-sync-ppen-invalid = Die Purple-Pen-Datei { $path } lässt sich nicht abgleichen. Sie ist kein lesbares Purple-Pen-Event oder passt nicht zur Karte des Projekts. Öffne sie in Purple Pen, um sie zu prüfen.
mcp-error-sync-invalid = Das zusammengeführte Projekt wäre ungültig, deshalb wurde nichts geschrieben. Führe den Abgleich auf der Kommandozeile aus, um die Einzelheiten zu sehen.
import-ppen-note-unknown-element = unbekannt und ausgelassen: { $what }
import-ppen-note-skipped = Bahn { $course } ausgelassen: { $why }
import-ppen-note-gaps = Bahn { $course } hat Lücken in Kreisen oder Strecken; das Modell kennt keine Lücken, Kreise und Strecken bleiben ganz
import-ppen-note-unmapped-symbol = Posten { $control }, Spalte { $column }: Symbol { $id } gibt es im Standard 2024 nicht und wird ausgelassen
import-ppen-note-number-into-circle = Posten { $control }: die Nummer berührt den Kreis oder reicht hinein; die Editoren verlangen einen Abstand über 0, die Platzierung bleibt wie in der Datei
import-ppen-note-fit-scale-unchecked = das Event nennt keinen Kartenmaßstab und wurde deshalb nicht mit dem der Karte verglichen
import-ppen-note-fit-orientation-assumed = die Positionen passen mit y nach oben und mit y nach unten auf die Karte; es gilt y nach oben, prüfe die Positionen daher
import-ppen-note-fit-axis-against-format = die y-Achse im Projekt widerspricht dem Dateiformat; das Blatt kann gespiegelt gedruckt werden
import-ppen-note-fit-other = die Anpassung an die Karte meldet etwas, wofür dieses Programm keinen Text hat
import-ppen-note-map-exchange = Bahn { $course }: der Kartenwechsel { $control } wurde zu einem Knickpunkt der Strecke
import-ppen-note-map-exchange-at-control = Bahn { $course }: der Kartenwechsel am Posten { $control } wird nicht übernommen
import-ppen-note-secondary-title = Bahn { $course }: der zweite Titel „{ $title }“ wird nicht übernommen
import-ppen-note-column-f-text = Posten { $control }: der Text „{ $text }“ in Spalte F wird nicht übernommen
import-ppen-note-start-finish-description = { $id }: die Beschreibung eines Starts oder Ziels wird nicht übernommen
import-ppen-note-map-issue-as-start = Kartenausgabe { $id } wurde als Start übernommen
import-ppen-note-special-skipped = ein Sonderobjekt der Art { $kind } wird nicht unterstützt und ausgelassen
import-ppen-note-special-without-course = ein Sonderobjekt der Art { $kind } gilt nur für ausgelassene Bahnen und entfällt auch
import-ppen-note-course-renamed = Bahn „{ $from }“ heißt im Projekt „{ $to }“; Bahnnamen sind eindeutig und haben keine Leerzeichen an den Enden
import-ppen-note-control-without-position = Posten { $control } hat keine Position und wird von keiner Bahn benutzt und entfällt
import-ppen-note-special-off-map = ein Sonderobjekt der Art { $kind } hat einen Punkt außerhalb der Karte und entfällt
import-ppen-note-number-gap-clamped = Posten { $control }: die Nummer liegt weit vom Kreis; der Abstand wird auf { $max } mm gekürzt
import-ppen-note-special-course-ambiguous = ein Sonderobjekt der Art { $kind } nennt die Bahn { $name }, aber mehrere Bahnen heißen so; es gilt für alle
import-ppen-note-forked = die Bahnen { $variants } sind die Varianten der Gabelung { $family }
import-ppen-note-notes-truncated = das Event hat mehr Hinweise, als der Import aufnimmt; der Rest entfällt
word-import-why-variations = sie hat Varianten
word-import-why-alternative-controls = sie hat alternative Posten
word-import-why-score-course = Score-Bahnen werden nicht übernommen
word-import-why-no-start = sie beginnt nicht an einem Start
word-import-why-no-finish = sie endet nicht an einem Ziel
word-import-why-start-or-finish-inside-the-course = ein Start oder Ziel liegt mitten in der Bahn
word-import-why-no-name = sie hat keinen Namen
word-import-special-out-of-bounds = Sperrgebiet
word-import-special-dangerous-area = Gefahrenbereich
word-import-special-forbidden-route = verbotene Route
word-import-special-boundary = Grenze
word-import-special-marked-route = abgeflaggte Strecke
word-import-special-first-aid = Erste-Hilfe-Stelle
word-import-special-water = Verpflegungsstelle
word-import-special-text = Text
word-import-special-line = Linie
word-import-special-white-out = Abdeckung
mcp-tool-import-ppen = Die Posten, Bahnen und Sonderobjekte eines Purple-Pen-Events (.ppen) übernehmen, das für die Karte des Projekts gezeichnet ist. Erwartet project, path (die .ppen-Datei) und replace; ohne replace darf das Projekt noch keine Posten, Bahnen oder Sonderobjekte haben, mit replace werden sie durch die des Events ersetzt, Kategorien und akzeptierte Befunde bleiben. Gibt controls, courses und notes zurück, die Texte zu dem, was ausgelassen oder geändert wurde.
mcp-param-import-ppen-path = Das Purple-Pen-Event, ein absoluter Pfad oder einer mit ~ am Anfang, mit der Endung .ppen
mcp-param-import-ppen-replace = Vorhandenes ersetzen [Standard: false]

## Abgleich mit der verknüpften Purple-Pen-Datei (sync)

cli-cmd-sync = Das Projekt mit seiner verknüpften Purple-Pen-Datei abgleichen
cli-cmd-sync-exit-codes =
    Exitcode 0: abgeglichen, oder es gab nichts zu tun.
    Exitcode 1: Konflikte bleiben offen, oder der Abgleich wurde abgelehnt. Dann wird nichts abgeglichen; nur Entscheidungen der Form SCHLÜSSEL=SEITE landen in der Projektdatei.
    Exitcode 2: falscher Aufruf, zum Beispiel eine unbekannte Option.
cli-arg-sync-project = Die Projektdatei
cli-arg-sync-dry-run = Zeigen, was der Abgleich täte, und nichts schreiben, auch keine Entscheidungen
cli-arg-sync-take = Für Konflikte eine Seite wählen. ours ist das Projekt, theirs die Purple-Pen-Datei. Eine Seite allein entscheidet jeden offenen Konflikt. SCHLÜSSEL=SEITE entscheidet einen Konflikt und bleibt in der Projektdatei gespeichert, solange andere Konflikte offen sind. Darf mehrfach vorkommen
cli-arg-sync-json = Das Ergebnis als JSON ausgeben
sync-summary =
    Abgleich mit { $path }.
    Änderungen nach Purple Pen: { $ours }
    Änderungen ins Projekt: { $theirs }
    Auf beiden Seiten gleich geändert: { $both }
sync-change-line = { $side ->
    [ours] { $key } geht nach Purple Pen: { $value }
    [theirs] { $key } kommt aus Purple Pen: { $value }
   *[both] { $key } ist auf beiden Seiten gleich geändert: { $value }
}
sync-conflict-line =
    { $key } weicht ab.
      Basis: { $base }
      Projekt: { $ours }
      Purple Pen: { $theirs }
sync-conflicts-remain = { $count ->
    [one] 1 Konflikt ist noch offen.
   *[other] { $count } Konflikte sind noch offen.
} Es wurde nichts abgeglichen. Mit --take ours oder --take theirs wählst du für alle eine Seite, mit --take SCHLÜSSEL=ours oder --take SCHLÜSSEL=theirs für einen einzelnen Konflikt.
sync-decisions-stored = { $count ->
    [one] 1 Entscheidung in { $path } gespeichert.
   *[other] { $count } Entscheidungen in { $path } gespeichert.
} Sie gelten, solange die Konflikte gleich bleiben.
sync-stale-decision = Die gespeicherte Entscheidung für { $key } passt nicht mehr, weil sich einer der Werte geändert hat. Sie wird ignoriert.
sync-nothing-to-do = Das Projekt und { $path } stimmen schon überein. Nichts zu tun.
sync-dry-run = Probelauf. Es wurde nichts geschrieben.
sync-note-deletion-kept = Posten { $control } bleibt in der Purple-Pen-Datei, obwohl das Projekt ihn gelöscht hat, weil Bahn { $course } ihn noch anläuft
sync-note-leg-change-dropped = die geänderten Knickpunkte oder die Flaggung der Strecke { $from } nach { $to } entfallen, weil keine Bahn diese Strecke mehr läuft
sync-note-new-name-taken = das neue Objekt { $from } aus Purple Pen heißt im Projekt { $to }, weil das Projekt { $from } schon vergeben hat
sync-note-base-ignored = { $why }. Ohne Basiskopie löscht der Abgleich nichts und behandelt jeden Unterschied als Konflikt
error-sync-internal = interner Fehler im Abgleich ({ $what }). Bitte melden
error-sync-take-parse = Nach --take erwartet: ours, theirs, KEY=ours oder KEY=theirs, bekommen: `{ $value }`.
error-sync-take-twice-all = --take nennt ours und theirs für alle Konflikte. Gib eine Seite an.
error-sync-take-twice-key = --take nennt zwei verschiedene Seiten für { $key }. Gib eine Seite an.
error-sync-base-unusable = Die Basiskopie { $path } lässt sich nicht als Purple-Pen-Event lesen. Lösche die Datei und führe sync erneut aus: Der Abgleich arbeitet dann ohne Basis, löscht nichts und fragt bei jedem Unterschied nach.
sync-note-first-sync = erster Abgleich dieses Projekts mit dieser Datei. Es gibt noch keine Basiskopie, deshalb löscht der Abgleich nichts und behandelt jeden Unterschied als Konflikt; er schreibt die Basiskopie, sobald er ohne offene Konflikte durchgelaufen ist
sync-note-base-repaired = ein früherer Abgleich hat die Basiskopie nicht geschrieben. Sie stimmt jetzt wieder mit der verknüpften Datei überein
sync-note-not-written = nicht nach Purple Pen geschrieben: { $note }
word-sync-why-missing = es gibt keine brauchbare Basiskopie
word-sync-why-other-link = die Basiskopie gehört zu einer anderen Purple-Pen-Datei
word-sync-why-hash = die Basiskopie passt nicht zum Hashwert in der Projektdatei
word-sync-value-deleted = gelöscht
word-sync-value-none = keine
word-sync-value-empty = leer
word-sync-kind-control = Posten
word-sync-kind-crossing-point = Querungsstelle
word-sync-kind-start = Start
word-sync-kind-finish = Ziel
error-sync-no-link = { $path } hat keine Verknüpfung mit einer Purple-Pen-Datei. Verknüpfe sie zuerst mit export --ppen DATEI oder import --ppen DATEI
error-sync-link-missing = die verknüpfte Purple-Pen-Datei { $path } existiert nicht
error-sync-changed-meanwhile = { $path } hat sich während des Abgleichs geändert, wahrscheinlich weil Purple Pen gespeichert hat. Es wurde nichts geschrieben. Starte sync erneut
error-sync-partial = der Abgleich hat { $written } geschrieben und ist dann gescheitert: { $source }. Starte sync erneut, um ihn zu beenden
error-sync-take-unknown-key = kein offener Konflikt hat den Schlüssel { $key }
error-sync-base = die Basiskopie zu { $path } lässt sich nicht einlesen: { $source }
error-sync-theirs = die verknüpfte Purple-Pen-Datei { $path } lässt sich nicht einlesen: { $source }
error-sync-export = das Projekt lässt sich für { $path } nicht als Purple-Pen-Event schreiben: { $source }
error-sync-unknown-version = { $path } stammt aus einer Purple-Pen-Version, die dieses Programm nicht kennt ({ $version }). Es wurde nichts geschrieben
error-sync-unmatched = { $count ->
    [one] der Schreiber konnte 1 Objekt von { $path } nicht zuordnen: { $list }. Es wurde nichts geschrieben
   *[other] der Schreiber konnte { $count } Objekte von { $path } nicht zuordnen: { $list }. Es wurde nichts geschrieben
}
error-sync-invalid = das zusammengeführte Event ist nicht gültig, deshalb wurde nichts geschrieben: { $errors }
error-sync-verify = die Purple-Pen-Datei ließe sich nicht als das zusammengeführte Projekt wieder einlesen ({ $list }). Es wurde nichts geschrieben

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
    Datei, wie sie war. Bisher lassen sich nur Einzelbahnen bearbeiten, messen und prüfen. check prüft die Bahnen
    gegen das Regelprofil der Veranstaltung, das set_rule_profile ändert. Manche Prüfungen brauchen das Startfenster
    (set_start_window) oder Regelkategorie und erwartete Starterzahl einer Kategorie (set_class); fehlen sie, führt
    check die Prüfregel als übersprungen. Einen Befund, den der Bahnleger so haben will, bekommt accept_finding mit
    Begründung; remove_accepted_finding nimmt das zurück.
mcp-error-relative-path = { $path } ist ein relativer Pfad. Gib einen absoluten Pfad an oder einen mit ~ am Anfang
mcp-error-not-toml = { $path } ist keine Projektdatei. Projektdateien enden auf .toml
error-import-not-map = { $path } ist keine OOM-Karte. Karten enden auf .omap oder .xmap
mcp-error-not-map = { $path } ist keine OOM-Karte. Karten enden auf .omap oder .xmap
mcp-error-not-xml = { $path } ist keine XML-Datei. Die Exportdatei endet auf .xml
mcp-error-project-parse = { $path } ist keine gültige Projektdatei. Das erste Problem liegt in Zeile { $line }, Spalte { $column }
mcp-error-project-parse-nopos = { $path } ist keine gültige Projektdatei
mcp-error-project-changed = { $path } hat sich auf der Platte geändert, nachdem es gelesen wurde. Es wurde nichts geschrieben. Lies das Projekt neu und wiederhole die Änderung
mcp-error-map-invalid = { $path } ist keine gültige OOM-Karte
mcp-error-no-home = Dieses System hat keinen Home-Ordner. Gib einen absoluten Pfad an
mcp-error-kind-needs-at = { $kind } ist ein einzelner Punkt. Gib at an und lass points weg
mcp-error-kind-needs-points = { $kind } wird durch Punkte gezeichnet. Gib points an und lass at weg
mcp-error-no-courses-for-kind = { $kind } gilt für jede Bahn. Lass courses weg
mcp-error-leg-target = Gib entweder from_start oder after an, nicht beides und nicht keins
mcp-error-no-change = Nichts zu ändern. Gib mindestens eines von classes, climb_m, no_climb, print_scale und no_print_scale an
mcp-error-no-class-change = Nichts zu ändern. Gib mindestens eines von rule_category, no_rule_category, expected_starters und no_expected_starters an
mcp-error-number-args = Gib direction und gap_mm zusammen an, oder clear
mcp-error-bad-gap = gap_mm { $value } liegt außerhalb des Bereichs. Gib mehr als 0 und höchstens { $max } Druckmillimeter an
mcp-error-special-courses-args = Gib entweder courses mit mindestens einem Bahnnamen an oder all mit true

mcp-tool-event-info = Überblick über die Veranstaltung: Titel, Karte und Kartenart, Kartenausdehnung, die Zahl der Posten, Sonderobjekte, Kategorien und Bahnen, und je Bahn die Posten, die Länge in km, den Anstieg und wie viele Postennummern gesetzt sind.
mcp-tool-list-controls = Die Posten auflisten, mit Code, Art, Position in Kartenmillimetern, Beschreibungssymbolen und Nummernlage.
mcp-tool-list-courses = Die Bahnen auflisten, mit Name, Art, Start, Ziel, Posten in Laufreihenfolge, Kategorien, Anstieg, geplantem Druckmaßstab und Länge in km.
mcp-tool-list-specials = Die Sonderobjekte auflisten (Start, Ziel, Pflichtstrecken, Sperrgebiete, verbotene Routen, Gefahrenbereiche, Erste-Hilfe-Stellen, Verpflegungsstellen), mit Geometrie und Bahnen.
mcp-tool-list-classes = Die Kategorien auflisten, mit Regelkategorie und erwarteten Startern.
mcp-tool-add-control = Einen Posten an einer Position in Kartenmillimetern anlegen. Ohne code gilt der niedrigste freie Code ab 31. Die Position muss auf der Karte liegen. Gibt den Code zurück.
mcp-tool-move-control = Einen Posten an eine neue Position in Kartenmillimetern setzen. Die Position muss auf der Karte liegen.
mcp-tool-remove-control = Einen Posten samt seinen Besuchen in Einzelbahnen entfernen. Das Ergebnis nennt die Bahnen, die er verlassen hat, und in cleared_legs die Strecken, die Knickpunkte und Abflaggung verloren haben, weil sie jetzt zu einem anderen Bahnpunkt führen.
mcp-tool-describe-control = Die Beschreibungssymbole eines Postens setzen, Spalte C bis H, als IOF-Symbol-IDs. Spalten, die du weglässt, behalten ihr Symbol. clear leert zuerst alle Spalten.
mcp-tool-set-control-number = Festlegen, wo die Nummer eines Postens steht, als Richtung in Grad (0 = Osten, gegen den Uhrzeigersinn) und Abstand zwischen Kreislinie und Nummer in Druckmillimetern. clear stellt den Standardplatz wieder her.
mcp-tool-add-special = Ein Sonderobjekt anlegen. Start, Ziel, Erste-Hilfe-Stelle und Verpflegungsstelle sind ein einzelner Punkt (at). Pflichtstrecke, verbotene Route, Sperrgebiet und Gefahrenbereich werden durch Punkte gezeichnet (points). Die id muss neu sein.
mcp-tool-remove-special = Ein Sonderobjekt anhand seiner id entfernen. Ein Start oder Ziel, den eine Bahn nutzt, lässt sich nicht entfernen.
mcp-tool-set-special-courses = Festlegen, zu welchen Bahnen ein Sonderobjekt gehört: eine Liste von Bahnnamen oder all mit true für alle Bahnen. Nicht für Start und Ziel. remove_course entfernt die einzige Bahn eines Sonderobjekts nicht, also hier zuerst erweitern oder umhängen.
mcp-tool-add-class = Eine Kategorie anlegen.
mcp-tool-remove-class = Eine Kategorie samt ihrem Namen in allen Bahnen entfernen.
mcp-tool-set-class = Regelkategorie oder erwartete Starterzahl einer Kategorie ändern. Weggelassene Parameter bleiben, wie sie sind; no_rule_category und no_expected_starters entfernen den Wert. Die Tabellenprüfungen lesen die Regelkategorie, die Prüfung der Startkapazität die erwartete Starterzahl.
mcp-tool-add-course = Eine Einzelbahn aus Start, Ziel und den Postencodes in Laufreihenfolge anlegen. Start und Ziel sind IDs von Sonderobjekten.
mcp-tool-remove-course = Eine Bahn samt ihrem Namen bei den Sonderobjekten entfernen.
mcp-tool-set-course-controls = Die Posten einer Bahn ersetzen. Der n-te Besuch eines Codes behält die Nummernlage des n-ten Besuchs davor. Eine Strecke behält Knickpunkte und Abflaggung nur, solange sie zum selben nächsten Bahnpunkt führt; das Ergebnis nennt die geleerten Strecken in cleared_legs mit course und after (0 ist die Strecke vom Start).
mcp-tool-set-leg = Die Knickpunkte einer Strecke einer Einzelbahn setzen und festlegen, ob sie abgeflaggt ist: die Strecke ab dem Start (from_start) oder die nach einer Position (after, ab 1 gezählt wie auf der Karte, Querungsstellen eingeschlossen). Ohne bends verliert die Strecke ihre Knickpunkte.
mcp-tool-set-course-details = Kategorien, Anstieg oder geplanten Druckmaßstab einer Bahn ändern. Parameter, die du weglässt, bleiben, wie sie sind.
mcp-tool-place-numbers = Die Postennummern einer Bahn hinter dem Kreis setzen, von der ankommenden Strecke aus gesehen, frei von Strecken, Kreisen, anderen Nummern sowie Punkt- und Liniensymbolen der Karte. Gibt zurück, wie viele Nummern es gesetzt hat. Läuft nur auf Aufruf, also nach Änderungen an Strecken erneut aufrufen. Ein Posten mit zentraler Nummernlage (set_control_number) behält sie, und Nummernlagen der Bahn (set_course_number) bleiben, außer overwrite ist true. Die Nummer rückt bis zum größten Abstand des Regelprofils nach außen, bevor sie von der bevorzugten Richtung abweicht.
mcp-tool-set-course-number = Die Nummernlage eines Postens in einer Einzelbahn setzen oder zurücksetzen, an einer Position ab 1 gezählt wie auf der Karte (Querungsstellen eingeschlossen). Sie geht der zentralen Lage aus set_control_number und place_numbers vor. Richtung in Grad (0 = Osten, gegen den Uhrzeigersinn), Abstand zwischen Kreislinie und Nummer in Druckmillimetern, mehr als 0 und höchstens 10. clear entfernt sie, dann gilt wieder die zentrale oder die automatische Lage.
mcp-tool-map-features-near = Kartenobjekte im Umkreis radius_mm um eine Position, das nächste zuerst: Symbolnummer, Geometrieart (point, line, area), Abstand in mm und der Teil der Karte.
mcp-tool-map-extent = Kleinste und größte Ecke der Kartenobjekte in Kartenmillimetern, der Kartenmaßstab und die Kartenart.
mcp-tool-init-event = Eine Veranstaltung aus einer OOM-Karte (Sprint oder Schulhof) anlegen und ihre Projektdatei schreiben. Gibt den Pfad der Projektdatei zurück.
mcp-tool-check = Die Einzelbahnen gegen das Regelprofil und die Aufsätze der Veranstaltung prüfen. Gibt in summary die Anzahlen zurück; die Befunde mit rule, profile, severity (error, warning oder hint), location als Objekt mit kind, message_id, args, source mit doc, section und title, rule_note und text; die akzeptierten Befunde mit der Begründung des Bahnlegers; in unmatched die akzeptierten Befunde, zu denen kein Befund mehr passt, etwa weil ein Posten oder eine Bahn fehlt (remove_accepted_finding entfernt sie, wenn sie nicht mehr gebraucht werden; sie zählen nie als Fehler); und die übersprungenen Prüfregeln mit dem Grund, auch die von einem Aufsatz abgeschalteten. Eine Strecke in location hat course, after und die Kennungen ihrer Enden in from und to. Mannschafts-, Staffel- und Score-Bahnen werden noch nicht geprüft.
mcp-tool-accept-finding = Einen Befund mit Begründung als gewollte Ausnahme akzeptieren. Nenne ihn so, wie check ihn meldet: die Prüfregel und ihren Ort, als controls, Strecken in legs oder course. Auch die id eines Starts oder Ziels kann in controls stehen, so wie check sie meldet. Eine Strecke wird mit course und after angegeben; der Server speichert die Kennungen ihrer Enden aus der Bahn, wie sie jetzt ist, sodass der Befund wiederkommt, wenn die Strecke später zwischen anderen Punkten verläuft. Lass alle drei weg für einen Befund der ganzen Veranstaltung. Wer denselben Befund noch einmal akzeptiert, ersetzt die Begründung. check führt ihn dann unter accepted.
mcp-tool-remove-accepted-finding = Einen akzeptierten Befund entfernen, damit check ihn wieder meldet. Nenne ihn mit der Prüfregel und demselben Ort wie beim Akzeptieren. Eine Strecke wird mit course und after angegeben. Entfernt wird zuerst der Eintrag für die Strecke, wie sie jetzt verläuft; gibt es keinen, alle Einträge an dieser Position, sodass sich ein Eintrag aus unmatched auch dann entfernen lässt, wenn sich seine Strecke oder Bahn geändert hat. Gibt die entfernten Einträge zurück.
mcp-tool-set-rule-profile = Das Basis-Regelprofil der Veranstaltung und seine Aufsätze setzen. Beide werden vorher geladen; ein unbekanntes Profil oder ein Aufsatz als Basis wird mit der Liste der verfügbaren Profile abgelehnt. Ohne overlays hat die Veranstaltung keine Aufsätze.
mcp-tool-set-start-window = Das Startfenster der Veranstaltung setzen, die Minuten vom ersten bis zum letzten Start, mindestens 1. Die Prüfung der Startkapazität teilt die erwarteten Starter dadurch.
mcp-tool-export-iof-xml = Die Einzelbahnen der Veranstaltung als IOF-XML-3.0-CourseData für Auswertungsprogramme wie OE schreiben. Erwartet project und path, die zu schreibende Datei mit der Endung .xml; eine vorhandene Datei wird ersetzt. Gibt path, courses (die Zahl der geschriebenen Bahnen) und skipped zurück, die ausgelassenen Mannschafts-, Staffel- und Score-Bahnen mit name und kind. Hat die Karte eine Georeferenzierung, tragen Posten, Start und Ziel Breite und Länge; geo_positions sagt, ob das der Fall war.
mcp-tool-export-ppen = Die Einzelbahnen der Veranstaltung in ein Purple-Pen-Event (.ppen) schreiben. Erwartet project, path (die .ppen-Datei) und new. Der Export ändert eine vorhandene Datei an Ort und Stelle und behält alles, was das Projekt nicht geändert hat. Eine fehlende Datei legt er an, und mit new schreibt er die Datei von Grund auf; in beiden Fällen wird ein Projekt ohne sources.ppen mit der Datei verknüpft. Der Export löscht nie; Objekte, die in der Datei stehen und im Projekt fehlen, bleiben und werden gemeldet. Die Purple-Pen-Kennungen des Projekts gelten nur für die Datei, die sources.ppen nennt, und nur für diese Datei speichert der Export die Kennungen, die er vergibt, im Projekt. Gibt path, courses (die Zahl der geschriebenen Bahnen), skipped (die ausgelassenen Mannschafts-, Staffel- und Score-Bahnen mit name und kind), notes (die Texte zu dem, was ausgelassen, geändert oder behalten wurde), patched (true, wenn eine vorhandene Datei geändert wurde) und ids_stored (wie viele Objekte des Projekts zum ersten Mal eine Purple-Pen-Kennung bekamen) zurück.

mcp-param-project = Projektdatei der Veranstaltung, ein absoluter Pfad oder einer mit ~ am Anfang, zum Beispiel ~/karten/halle.courseplan.toml
mcp-param-export-iof-xml-path = Die zu schreibende Datei, ein absoluter Pfad oder einer mit ~ am Anfang, mit der Endung .xml
mcp-param-export-ppen-path = Das zu schreibende Purple-Pen-Event, ein absoluter Pfad oder einer mit ~ am Anfang, mit der Endung .ppen; eine fehlende Datei wird angelegt
mcp-param-export-ppen-new = Die Datei von Grund auf schreiben und eine vorhandene ersetzen [Standard: false]
mcp-tool-sync-ppen = Das Projekt mit der Purple-Pen-Datei abgleichen, die sources.ppen nennt. Der Abgleich vergleicht beide mit der Basiskopie, die der letzte Abgleich hinterlassen hat. Eine Änderung auf einer Seite kommt so auf der anderen an, und eine Änderung am selben Feld auf beiden Seiten ist ein Konflikt. Erwartet project, dry_run und take. Mit dry_run wird nichts geschrieben. take (ours oder theirs) entscheidet alle offenen Konflikte nur für diesen Aufruf und speichert nichts. Solange Konflikte offen sind, schreibt der Abgleich nichts; mit resolve_sync_conflict entscheidest du sie einzeln. Ein Abgleich ohne offene Konflikte schreibt die Purple-Pen-Datei, das Projekt und die Basiskopie neben der Projektdatei. Gibt written (true, wenn der Abgleich Dateien geändert hat), dry_run, ppen_changed (die Purple-Pen-Datei hat sich geändert), base (use, repair, first oder ignore), changes (key, side und value jeder Änderung), conflicts (key, base, ours und theirs jedes offenen Konflikts), notes (Texte zur Basiskopie und zu dem, was behalten oder nicht geschrieben wurde) und stale (gespeicherte Entscheidungen, die nicht mehr passen) zurück.
mcp-tool-resolve-sync-conflict = Die Antwort auf einen offenen Abgleichskonflikt in der Projektdatei speichern, ohne abzugleichen. Erwartet project, key (so, wie sync_ppen ihn unter conflicts aufführt) und take (ours behält den Wert des Projekts, theirs den der Purple-Pen-Datei). Die Antwort gilt nur, solange beide Werte unverändert bleiben, und der nächste abgeschlossene Abgleich löscht sie. Ein Schlüssel, der kein offener Konflikt ist, wird abgelehnt. Gibt dieselben Felder zurück wie ein Probelauf von sync_ppen, conflicts enthält also, was noch offen ist; ist die Liste leer, schließt ein Aufruf von sync_ppen den Abgleich ab.
mcp-param-sync-ppen-dry-run = Melden, was der Abgleich täte, und nichts schreiben [Standard: false]
mcp-param-sync-ppen-take = Alle offenen Konflikte für diesen Aufruf mit dem Wert des Projekts (ours) oder der Purple-Pen-Datei (theirs) entscheiden; es wird nichts gespeichert [Standard: Konflikte bleiben offen]
mcp-param-resolve-sync-conflict-key = Der Schlüssel des Konflikts, so, wie sync_ppen ihn unter conflicts aufführt
mcp-param-resolve-sync-conflict-take = Welcher Wert bleibt, der des Projekts (ours) oder der der Purple-Pen-Datei (theirs)
mcp-param-init-event-project = Wohin die Projektdatei kommt, ein absoluter Pfad oder einer mit ~ am Anfang [Standard: neben der Karte, nach ihr benannt mit .courseplan.toml]
mcp-param-map = Die OOM-Karte (Sprint oder Schulhof), ein absoluter Pfad oder einer mit ~ am Anfang
mcp-param-title = Titel der Veranstaltung [Standard: der Titel in freemapper.toml neben der Karte, sonst der Kartenname]
mcp-param-code = Code des Postens, als Text oder ganze Zahl
mcp-param-add-control-code = Code des neuen Postens, als Text oder ganze Zahl [Standard: die niedrigste freie Zahl ab 31]
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
mcp-param-direction = Grad, 0 = Osten, gegen den Uhrzeigersinn; jeder Wert wird auf 0 bis 360 umgerechnet
mcp-param-gap-mm = Abstand zwischen Kreislinie und Nummer in Druckmillimetern, mehr als 0 und höchstens 10
mcp-param-kind = Was das Objekt ist: start, finish, marked_route, out_of_bounds, forbidden_route, dangerous_area, first_aid oder water
mcp-param-id = ID des Objekts, eindeutig unter den Sonderobjekten
mcp-param-points = Die Punkte von marked_route, forbidden_route, out_of_bounds oder dangerous_area in Reihenfolge; jeder ist ein Objekt mit x und y in Kartenmillimetern
mcp-param-courses = Das Objekt auf diese Bahnen beschränken [Standard: alle Bahnen]; nicht für start und finish
mcp-param-set-special-courses-courses = Die Bahnen, zu denen das Objekt gehört, mindestens ein Name; weglassen, wenn all true ist
mcp-param-all = Das Objekt für alle Bahnen gelten lassen; dann courses weglassen [Standard: false]
mcp-param-set-special-courses-id = ID des Sonderobjekts; kein Start und kein Ziel
mcp-param-name = Name der Bahn
mcp-param-add-class-name = Name der Kategorie
mcp-param-remove-class-name = Name der Kategorie
mcp-param-rule-category = Regelkategorie der Kategorie
mcp-param-expected-starters = Erwartete Zahl der Starter
mcp-param-set-class-name = Name der Kategorie
mcp-param-no-rule-category = Die Regelkategorie entfernen [Standard: false]
mcp-param-no-expected-starters = Die erwartete Starterzahl entfernen [Standard: false]
mcp-param-minutes = Minuten vom ersten bis zum letzten Start, mindestens 1
mcp-param-start = ID des Sonderobjekts, das den Start bildet
mcp-param-finish = ID des Sonderobjekts, das das Ziel bildet
mcp-param-controls = Codes der Posten in Laufreihenfolge, als Text oder ganze Zahl; eine Querungsstelle zählt als Position [Standard: keine]
mcp-param-classes = Kategorien, die die Bahn laufen
mcp-param-climb-m = Anstieg in Metern
mcp-param-print-scale = Geplanter Druckmaßstab, zum Beispiel 3000 für 1:3000
mcp-param-codes = Codes in Laufreihenfolge, als Text oder ganze Zahl; eine leere Liste entfernt alle Posten
mcp-param-from-start = Die Strecke vom Start zum ersten Posten bearbeiten [Standard: false]
mcp-param-after = Die Strecke nach dieser Position bearbeiten, ab 1 gezählt wie auf der Karte; Querungsstellen zählen als Positionen
mcp-param-bends = Die Knickpunkte der Strecke in Reihenfolge, jeder ein Objekt mit x und y in Kartenmillimetern [Standard: keine, die Strecke verliert ihre Knickpunkte]
mcp-param-taped = Die Strecke ist abgeflaggt [Standard: false, die Abflaggung entfällt]
mcp-param-no-climb = Den Anstieg entfernen [Standard: false]
mcp-param-no-print-scale = Den geplanten Druckmaßstab entfernen [Standard: false]
mcp-param-overwrite = Auch früher gesetzte Nummernlagen der Bahn ersetzen; zentrale Nummernlagen bleiben [Standard: false]
mcp-param-set-course-number-after = Position des Postens in der Bahn, ab 1 gezählt wie auf der Karte; Querungsstellen zählen als Positionen
mcp-param-set-course-number-clear = Die Nummernlage der Bahn entfernen, sodass wieder die zentrale oder die automatische Lage gilt; dann direction und gap_mm weglassen [Standard: false]
mcp-param-radius-mm = Suchradius in Kartenmillimetern
mcp-param-limit = Höchstzahl der Objekte [Standard: 20, höchstens 200]
mcp-param-rule = Kennung der Prüfregel, wie check sie meldet, zum Beispiel wkb2026.distance.min
mcp-param-note = Warum die Ausnahme gewollt ist
mcp-param-accept-finding-controls = Die Posten des Befunds, als Texte oder ganze Zahlen, wie in location von check; ids von Start und Ziel sind erlaubt [Standard: keine]
mcp-param-remove-accepted-finding-controls = Die Posten des Befunds, als Texte oder ganze Zahlen, wie in location von check [Standard: keine]
mcp-param-accept-finding-legs = Die Strecken des Befunds, jede ein Objekt mit course und after wie in location von check: eine Strecke oder zwei für einen Befund an zwei Strecken [Standard: keine]
mcp-param-remove-accepted-finding-legs = Die Strecken des Befunds, jede ein Objekt mit course und after wie in location von check: eine Strecke oder zwei für einen Befund an zwei Strecken [Standard: keine]
mcp-param-accept-finding-course = Name der Bahn; für einen Befund, der die ganze Bahn betrifft, und in legs für die Bahn der Strecke
mcp-param-remove-accepted-finding-course = Name der Bahn; für einen Befund, der die ganze Bahn betrifft, und in legs für die Bahn der Strecke
mcp-param-accept-finding-after = Position, von der die Strecke in der Punktliste der Bahn abgeht: Die Posten der Bahn zählen ab 1, Querungsstellen mitgezählt, und 0 ist die Strecke vom Start. Das ist nicht die Postennummer auf der Karte; anzugeben ist after aus dem Ort einer Strecke in check
mcp-param-remove-accepted-finding-after = Position, von der die Strecke in der Punktliste der Bahn abgeht: Die Posten der Bahn zählen ab 1, Querungsstellen mitgezählt, und 0 ist die Strecke vom Start. Das ist nicht die Postennummer auf der Karte; anzugeben ist after aus dem Ort einer Strecke in check
mcp-param-base = Das Basis-Regelprofil, zum Beispiel wkb-2026
mcp-param-overlays = Aufsätze auf das Basisprofil in der Reihenfolge, in der sie gelten, zum Beispiel nor-bb-2021 [Standard: keine]

## Editor (Befehl edit und Editorseite; editor-* Meldungen haben keine Argumente)

cli-help-edit = Den Bahnlegungs-Editor im Browser öffnen
cli-arg-edit-project = Die Projektdatei
cli-arg-edit-port = Port auf 127.0.0.1 [Vorgabe: ein freier Port]
cli-arg-edit-no-browser = Nur die Adresse ausgeben, den Browser nicht öffnen
cli-edit-stop-hint = Der Editor läuft. Beende ihn mit Strg+C oder mit „Editor schließen“ in der Seite.
cli-edit-browser-failed = Der Browser hat sich nicht geöffnet ({ $reason }). Öffne die Adresse oben von Hand.
cli-edit-bind-failed = Port { $port } auf 127.0.0.1 lässt sich nicht belegen ({ $reason }). Wähle mit --port einen anderen Port oder lass die Angabe weg, dann nimmt das Programm einen freien.
cli-edit-runtime-failed = Der Editor lässt sich nicht starten: { $reason }
cli-edit-server-failed = Der Editor ist mit einem Fehler beendet worden: { $reason }

editor-page-title = Bahnlegung
editor-menu-event = Veranstaltung
editor-menu-export = Exportieren
editor-menu-import = Importieren
editor-menu-sync = Mit Purple Pen abgleichen
editor-menu-close = Editor schließen
editor-pane-tree = Inhalt der Veranstaltung
editor-pane-map = Karte
editor-pane-props = Eigenschaften und Befunde
editor-tree-courses = Bahnen
editor-tree-classes = Kategorien
editor-tree-controls = Posten
editor-tree-specials = Sonderobjekte
editor-props-heading = Eigenschaften
editor-props-empty = Wähle eine Bahn, einen Posten oder ein Sonderobjekt, um die Eigenschaften zu sehen.
editor-findings-heading = Befunde
editor-findings-empty = Noch nichts anzuzeigen.
editor-toolbar-label = Werkzeuge
editor-tool-select = Auswählen
editor-tool-control = Posten setzen
editor-tool-crossing = Querungsstelle setzen
editor-tool-special = Sonderobjekt
editor-tool-course = Bahnmodus
editor-status-label = Status
editor-status-ready = Bereit
editor-status-closed = Der Editor ist beendet. Du kannst diesen Tab schließen.
editor-status-lost = Der Editor antwortet nicht. Starte ihn mit dem Befehl edit neu.
editor-map-placeholder = Die Kartendarstellung folgt mit freemapper-render
editor-map-bad-request = Der angeforderte Kartenausschnitt ist ungültig.
editor-tree-label = Baum der Veranstaltung
editor-tree-all-controls = Alle Posten
editor-none = keine
editor-kind-course = Bahn
editor-kind-control = Posten
editor-kind-crossing-point = Querungsstelle
editor-kind-start = Start
editor-kind-finish = Ziel
editor-kind-marked-route = Pflichtstrecke
editor-kind-out-of-bounds-area = Sperrgebiet
editor-kind-forbidden-route = Verbotene Route
editor-kind-dangerous-area = Gefahrenbereich
editor-kind-first-aid = Erste-Hilfe-Stelle
editor-kind-water = Verpflegungsstelle
editor-prop-name = Name
editor-prop-code = Kennzahl
editor-prop-id = Kennung
editor-prop-kind = Art
editor-prop-position = Position
editor-prop-description = Beschreibung
editor-prop-length = Länge
editor-prop-climb = Steigung
editor-prop-stops = Posten der Bahn
editor-prop-start = Start
editor-prop-finish = Ziel
editor-prop-classes = Kategorien
editor-prop-used-in = Verwendet in
editor-desc-column-c = Spalte C
editor-desc-column-d = Spalte D
editor-desc-column-e = Spalte E
editor-desc-column-f = Spalte F
editor-desc-column-g = Spalte G
editor-desc-column-h = Spalte H
editor-legend-auto-number = Gestrichelte Nummernfelder sind automatisch platziert.
editor-zoom-in = Vergrößern
editor-zoom-out = Verkleinern
editor-zoom-fit = Karte ins Fenster einpassen
editor-status-map = Karte
editor-status-scale = Maßstab
editor-status-kind = Art
editor-status-cursor = Zeiger (mm)
editor-mapkind-sprint = Sprint
editor-mapkind-schoolyard = Schulhof
editor-mapkind-forest = Wald
editor-status-reloaded = Die Projektdatei hat sich geändert und wurde neu geladen.
editor-status-ppen-changed = Die Purple-Pen-Datei hat sich geändert. Abgleichen?
editor-status-state-failed = Das Projekt lässt sich nicht anzeigen. Die Serverkonsole nennt den Grund.
editor-status-overlay-failed = Die Bahnsymbole ließen sich nicht laden.
editor-status-course-unsupported = Diese Art von Bahn (Score-, Team- oder Staffelbahn) lässt sich im Editor noch nicht anzeigen.
editor-op-unknown = Diese Operation gibt es nicht.
editor-op-no-revision = Die Anfrage nennt nicht, auf welcher Revision der Projektdatei sie beruht.
editor-op-bad-body = Der Inhalt der Anfrage ist kein JSON-Objekt.
editor-op-stale = Die Projektdatei wurde außerhalb des Editors geändert. Der Editor hat den aktuellen Stand geladen; es wurde nichts geändert.
editor-undo-nothing = Nichts zum Rückgängigmachen.
editor-redo-nothing = Nichts zum Wiederherstellen.
editor-undo-changed = Die Projektdatei wurde außerhalb des Editors geändert, daher sind Rückgängig und Wiederherstellen nicht mehr möglich. Es wurde nichts geändert.
editor-undo-failed = Die Projektdatei konnte nicht geschrieben werden
editor-op-add-control = Posten setzen
editor-op-move-control = Posten verschieben
editor-op-remove-control = Posten löschen
editor-op-describe-control = Posten beschreiben
editor-op-set-control-number = Nummer des Postens setzen
editor-op-add-special = Sonderobjekt hinzufügen
editor-op-remove-special = Sonderobjekt entfernen
editor-op-set-special-courses = Bahnen des Sonderobjekts setzen
editor-op-add-class = Kategorie hinzufügen
editor-op-set-class = Kategorie ändern
editor-op-remove-class = Kategorie entfernen
editor-op-add-course = Bahn hinzufügen
editor-op-remove-course = Bahn entfernen
editor-op-set-course-controls = Posten der Bahn ändern
editor-op-set-leg = Strecke ändern
editor-op-set-course-details = Bahndaten ändern
editor-op-place-numbers = Nummern platzieren
editor-op-set-course-number = Nummer der Bahn setzen
editor-op-accept-finding = Befund akzeptieren
editor-op-remove-accepted-finding = Akzeptierten Befund zurücknehmen
editor-op-set-rule-profile = Regelprofil ändern
editor-op-set-start-window = Startfenster ändern
editor-op-resolve-sync-conflict = Synchronisationskonflikt lösen
editor-tool-undo = Rückgängig
editor-tool-redo = Wiederholen
editor-tool-help = Tastenkürzel
editor-banner-dismiss = Schließen
editor-help-title = Tastenkürzel
editor-help-close = Schließen
editor-key-undo = Rückgängig
editor-key-redo = Wiederholen
editor-key-delete = Gewählten Posten löschen
editor-key-cancel = Ziehen abbrechen oder zurück zum Auswahlwerkzeug
editor-key-select-tool = Auswahlwerkzeug
editor-key-control-tool = Werkzeug Posten setzen
editor-key-nudge = Gewählten Posten um 0,1 mm verschieben
editor-key-nudge-big = Gewählten Posten um 1 mm verschieben
editor-key-zoom-in = Vergrößern (Strg oder Cmd mit + zoomt weiterhin den Browser)
editor-key-zoom-out = Verkleinern
editor-key-fit = Karte ins Fenster einpassen
editor-key-help = Diese Liste anzeigen
editor-confirm-title = Posten löschen
editor-confirm-used = Diese Bahnen führen über den Posten. Beim Löschen verschwindet er aus ihnen.
editor-confirm-delete = Löschen
editor-cancel = Abbrechen
editor-prop-x = X (mm)
editor-prop-y = Y (mm)
editor-prop-delete = Posten löschen
editor-prop-near-heading = Kartenobjekte im Umkreis von 3 mm
editor-prop-near-none = Keine Kartenobjekte in der Nähe.
editor-geometry-point = Punkt
editor-geometry-line = Linie
editor-geometry-area = Fläche
editor-error-number = Gib eine Zahl in Millimetern ein, zum Beispiel 12,5 oder 12.5.
editor-status-placed = Posten gesetzt.
editor-status-moved = Posten verschoben.
editor-status-deleted = Posten gelöscht
editor-status-undone = Rückgängig gemacht
editor-status-redone = Wiederholt
error-editor-map = Die Karte { $file } lässt sich nicht lesen: { $reason }
error-editor-project-too-large = { $file } ist größer als { $limit } MiB. Der Editor öffnet keine so großen Projektdateien.
