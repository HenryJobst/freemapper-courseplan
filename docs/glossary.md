# Glossar / Glossary

Verbindliche Begriffe für Code und Dokumentation. Neue Fachbegriffe werden **zuerst hier** ergänzt, dann
verwendet. Code-Bezeichner sind Englisch (Rust-Konventionen: `TypeName`, `fn_name`, `CONST_NAME`). Begriffe, die
das [Glossar von freemapper](https://github.com/HenryJobst/freemapper/blob/main/docs/glossary.md) oder das
[Glossar von freemapper-print](https://github.com/HenryJobst/freemapper-print/blob/main/docs/glossary.md) schon festlegen, gelten hier unverändert,
mit denselben Code-Bezeichnern. Das sind unter anderem Maßstab, Symbol, Kartenart, Sprintkarte (freemapper) sowie
Veranstaltung (`Event`), Projektdatei (`ProjectFile`), Kartenquelle (`MapSource`), Kartenkoordinaten (`MapCoord`),
Druckmaßstab (`print_scale`), Bahn (`Course`), Posten (`Control`), Postencode (`ControlCode`), Postennummer
(`control_number`), Strecke (`Leg`), Knickpunkt (`bends`), Pflichtstrecke (`MarkedRoute`), Nummernlage
(`NumberPlacement`), automatische Platzierung (`auto_place`), Bahnaufdruck (`CourseOverprint`),
Postenbeschreibung (`ControlDescriptions`), Gabelung (`Forking`) und Purple-Pen-Projekt (`PurplePenProject`)
(freemapper-print). Hier stehen nur Begriffe, die freemapper-courseplan neu braucht.

Binding terms for code and documentation. New domain terms are added **here first**, then used. Code
identifiers are English. Terms already fixed by freemapper's or freemapper-print's glossary keep their meaning
and code identifiers here; only new terms appear below.

Abkürzungen / abbreviations: **WKB** = Wettkampfbestimmungen OL (AGO, German competition rules), **AGO** =
Arbeitsgemeinschaft Orientierungssport, **IOF** = International Orienteering Federation, **OE** = OE12 von
SportSoftware (Auswertung), **ISSprOM**, **ISOM**, **IOF-XML 3.0**, **OOM** wie bei freemapper-print.

## Bahnen / courses

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Bahnart | course kind | Einzel, Mannschaft, Staffel oder Score. Bestimmt, wie Posten einer Bahn angelaufen werden. | `CourseKind` |
| Einzelbahn | individual course | Feste Folge vom Start über die Posten zum Ziel. | `CourseKind::Individual` |
| Mannschaftsbahn | team course | Bahn des aufgeteilten Mannschafts-OL: Pflichtposten für alle, Wahlposten für die Mannschaft (WKB B 3.3.2.3). | `CourseKind::Team` |
| Pflichtposten | mandatory control | Posten einer Mannschaftsbahn, den jedes Mitglied in vorgegebener Reihenfolge anläuft. Gekennzeichnet mit A, B, C … | `MandatoryControl` |
| Wahlposten | free control | Posten einer Mannschaftsbahn, den mindestens ein Mitglied anläuft, nur zwischen erstem und letztem Pflichtposten. In OE „Aufteilposten“. | `FreeControl` |
| Staffelbahn | relay course | Bahnfamilie mit gemeinsamen Teilen und Gabelungsvarianten, verteilt auf Strecken und Teams. | `CourseKind::Relay` |
| Score-Bahn | score course | Menge von Posten mit Werten, Zeitlimit und Strafe; Reihenfolge frei. | `CourseKind::Score` |
| Bahnposten | course control | Ein Posten in einer Bahn, mit eigener Nummernlage und eigenen Knickpunkten der folgenden Strecke. | `CourseControl` |
| geplanter Druckmaßstab | planned print scale | Maßstab, für den die Bahn gelegt wird. Grundlage für maßstabsabhängige Prüfungen; für freemapper-print nur ein Vorgabewert. | `planned_print_scale` |
| Sonderobjekt | special object | Bahnobjekt außer Posten: Start, Ziel, Pflichtstrecke, Sperrgebiet, verbotene Route, Gefahrenbereich, Erste-Hilfe-Stelle, Verpflegungsstelle. | `SpecialObject` |
| Querungsstelle | crossing point | Pflichtübergang, etwa über eine Straße oder durch einen Zaun. Ein Posten, den die Bahn in ihrer Folge führt, der aber keine Nummer bekommt. | `ControlKind::CrossingPoint` |
| abgeflaggte Strecke | taped leg | Strecke, die als Pflichtstrecke abgeflaggt ist; am Bahnposten oder, ab dem Start, an der Bahn. | `taped`, `start_taped` |
| Streckengeometrie | leg geometry | Linienzug einer Strecke vom einen Bahnpunkt zum nächsten (Start, Posten, Querungsstelle, Ziel) über die Knickpunkte, mit Länge in Kartenmillimetern und Kennzeichen abgeflaggte Strecke. | `LegGeometry` |
| Bahnlänge | course length | Summe aller Streckenlinienzüge in Kartenmillimetern mal Kartenmaßstab: Luftlinie plus gezeichnete Umwege (Knickpunkte), abgeflaggte Strecken entlang ihrer Knickpunkte gemessen (WKB B 7.2.5). In Metern. | `course_length_m` |
| Aufdruckgrößen | overprint sizes | Kreisdurchmesser, Linienbreite und Nummernhöhe des Bahnaufdrucks in Druckmillimetern beim Grundmaßstab der Kartenart: ISSprOM 2019-2 für Sprint und Schulhof, ISOM 2017-2 für Wald. | `OverprintSizes` |
| Grundmaßstab | base scale | Maßstab, für den die Aufdruckgrößen gelten: Sprint 1:4000, Schulhof 1:1000, Wald 1:15000. Auf der Karte gilt `map_mm = print_mm × base_scale / map_scale`, auch bei vergrößertem Druck. | `base_scale` |
| Druckbereich | print area | Rechteck der Karte, das das Blatt einer Bahn zeigt, in Millimetern auf der Karte. | `PrintArea` |
| Staffelteil | relay part | Gemeinsamer Teil oder Gabelung einer Staffelbahn. | `RelayPart` |
| Score-Posten | score control | Posten einer Score-Bahn mit seinem Wert. | `ScoreControl` |
| Eingabedateien | sources | Karte und optional Purple-Pen-Projekt der Veranstaltung, Pfade relativ zur Projektdatei. | `Sources` |
| Gefahrenbereich | dangerous area | Fläche, vor der der Bahnaufdruck mit violetter Markierung warnt (WKB B 7.2.1). | `SpecialObject::DangerousArea` |
| Erste-Hilfe-Stelle | first aid | Ort der Ersten Hilfe, auf der Karte eingezeichnet. | `SpecialObject::FirstAid` |
| Verpflegungsstelle | water | Ort mit Getränken für die Läufer, auf der Karte eingezeichnet. | `SpecialObject::Water` |
| Sperrgebiet | out-of-bounds area | Fläche, die nicht betreten werden darf (ISOM/ISSprOM 709). | `OutOfBoundsArea` |
| verbotene Route | forbidden route | Linie, die nicht begangen werden darf (ISOM/ISSprOM 711). | `ForbiddenRoute` |

## Veranstaltung / event

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Disziplin | discipline | Sprint, Mittel, Lang, Ultralang oder Training. Training schaltet die Tabellenprüfungen ab. | `Discipline` |
| Kategorie | class | Startklasse mit freiem Namen, passend zu OE; optional einer WKB-Kategorie zugeordnet. | `Class` |
| WKB-Kategorie | rule category | Kategorie nach WKB (H21, D-12, 130T …), an der Längentabelle und Schwierigkeitsstufe hängen. | `RuleCategory` |
| Kartenpunkt | map point | Punkt in Millimetern auf der Karte im Kartenmaßstab, `f64`, y nach unten. `omap::MapCoord` ist die Dateiform der OOM in 1/1000 mm. | `MapPoint` |
| Kartenindex | map index | Alle Geländeobjekte einer Karte in Millimetern, abfragbar nach Abstand zu einem Punkt. Blatt-Objekte (`freemapper:source=sheet`) fehlen. | `MapIndex` |
| Kartenobjekt für Abfragen | map feature | Ein Geländeobjekt im Kartenindex: Symbolnummer, Kartenteil und Geometrie (Punkt, Linie, Fläche). | `MapFeature` |
| Startfenster | start window | Zeit vom ersten bis zum letzten Start. | `start_window` |
| erster Posten | first control | Der erste Posten einer Bahn nach dem Start. | `first_control` |
| Sprache | language | Sprache der Programmtexte: `de` oder `en`, gewählt mit `--lang`, `FREEMAPPER_LANG` oder der Systemsprache (ADR 0022). | `Lang` |

## Prüfen / checking

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Regelprofil | rule profile | Satz von Prüfregeln aus einem Regelwerk, zuerst WKB 2026. | `RuleProfile` |
| Prüfregel | check rule | Eine Regel mit Kennung, Werten und Quelle (Dokument und Abschnitt), als Daten gespeichert. | `Rule` |
| Prüfregel-Schlüssel | rule key | Name der Prüfung, die eine Prüfregel ausführt, etwa `min_distance` oder `length_table`. Das Programm kennt eine feste Liste; ein Regelprofil mit einem anderen Schlüssel wird abgelehnt. | `key` |
| Regelprofil-Aufsatz | rule overlay | Regelprofil, das ein Basisprofil ergänzt oder einzelne Regeln ersetzt, zum Beispiel für ein Landesregelwerk; der Veranstaltung zugeordnet über `rule_overlays`. | `rule_overlays` |
| Befund | finding | Ergebnis einer Prüfregel an einem Posten, einer Strecke oder einer Bahn. | `Finding` |
| akzeptierter Befund | accepted finding | Befund, den der Bahnleger mit Begründung als gewollte Ausnahme markiert hat; wird nicht mehr gemeldet. | `AcceptedFinding` |
| übersprungene Prüfregel | skipped rule | Prüfregel, die `check` für eine Veranstaltung oder Bahn nicht ausführt, mit dem Grund: Filter (Disziplin, Kartenart, Druckmaßstab) passt nicht, Angabe fehlt. Auch eine Bahn, die nicht Einzelbahn ist. Wird gemeldet, nie stillschweigend als bestanden gezählt. | `Skipped` |
| Ort eines Befunds | finding location | Wo ein Befund sitzt: Veranstaltung, Bahn, ein Posten, mehrere Posten, eine Strecke oder zwei oder mehr Strecken einer Bahn. Start und Ziel zählen dabei als Posten. Bestimmt mit der Kennung der Prüfregel, welcher akzeptierte Befund passt. | `Location` |
| Strecken-Bezug | leg reference | Verweis auf eine Strecke einer Bahn: Bahnname und Nummer des Postens, von dem die Strecke abgeht (ab 1; 0 ist die Strecke vom Start). | `LegRef` |
| ähnliche Objekte | similar features | Postenobjekte, die im Gelände oder auf der Karte verwechselbar sind (WKB B 7.2.1). | `similar_features` |
| Ortsart | place kind | Art des Orts eines Befunds: Veranstaltung, Bahn, ein Posten, zwei oder mehr Posten, Strecke, zwei oder mehr Strecken einer Bahn. Jede Prüfung nennt die Ortsarten ihrer Befunde; `finding accept` lehnt andere ab, weil ein solcher akzeptierter Befund nie passen würde. | `Place` |
| Postenobjekt | control feature | Das Kartenobjekt, an dem ein Posten steht: das nächste Kartenobjekt mit Symbolnummer innerhalb von `feature_search_mm` um die Postenmitte. Eine Fläche zählt mit ihrem Rand; die Fläche, in der der Posten nur liegt, ist also nicht sein Objekt. | `control_feature` |
| Kartenrand | map edge | Rand des Rechtecks um alle Geländeobjekte des Kartenindex, ohne Blatt-Objekte. | `MapIndex::extent` |
| Sperrgebiet-Symbol | out-of-bounds symbol | Kartensymbol eines Sperrgebiets (ISOM/ISSprOM 520, 708, 709, auf Sprintkarten auch 714), als Symbolnummer in `impassable` von `leg_through_forbidden`. Eine Unternummer gehört zu ihrer Nummer: 709.1 zu 709, 7091 nicht. | `impassable`, `symbol_matches` |
| gezeichneter Teil einer Strecke | drawn part of a leg | Streckenlinienzug ohne je einen Kreisradius an beiden Enden, so wie der Bahnaufdruck ihn zeichnet. Die Prüfungen auf Strecken durch Posten und Sperrgebiete messen nur diesen Teil. | `drawn_part` |
| spitzer Winkel | acute angle | Winkel zwischen ankommender und abgehender Strecke an einem Posten oder einer Querungsstelle, vom letzten Knickpunkt davor zum ersten danach gemessen: 180° geradeaus, 0° zurück auf demselben Weg. Start und Ziel haben keinen. | `acute_angle` |
| gegenläufige Strecken | opposing legs | Zwei Strecken einer Bahn, die auf mindestens `min_length_metres` höchstens `metres` voneinander entfernt in Richtungen laufen, die um mehr als `min_angle_degrees` verschieden sind (WKB B 7.2.6). Treffen sich beide an einem Posten, zählen die ersten `metres` um ihn nicht mit. | `opposing_legs` |
| Startkapazität | start capacity | Zahl der verschiedenen ersten Posten, die die Bahnen eines Starts brauchen: erwartete Starter aller Kategorien dieser Bahnen geteilt durch `start_window_min`, aufgerundet. Belegt sind die verschiedenen ersten Posten der Bahnen mit Startern. Bahnen teilen sich eine Startminute nur am selben Start. | `start_capacity` |
| Richtung der ersten Strecke | first-leg direction | Richtung vom Start zum ersten Punkt der ersten Strecke: dem ersten Knickpunkt am Start, sonst dem ersten Bahnpunkt (auch eine Querungsstelle). Zwei Bahnen am selben Start mit verschiedenen ersten Posten sollen sich darin um mindestens `degrees` unterscheiden. | `first_leg_angle` |
| Schmetterlingsschleife | butterfly loop | Teil einer Bahn, der zwei oder mehr Schleifen über denselben Posten führt. Der Posten ist ein gewollter Treffpunkt; gegenläufige Teilstrecken weit von ihm zählen trotzdem. | `opposing_legs` |

## Austausch / exchange

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Abgleich | sync | Zusammenführen der Projektdatei mit der verknüpften `.ppen` in beide Richtungen. | `sync` |
| Partnerdatei | linked Purple Pen file | Die `.ppen`, mit der eine Veranstaltung abgeglichen wird. | `linked_ppen` |
| Abgleichsbasis | sync base | Kopie der `.ppen` vom letzten Abgleich; dritte Fassung beim Zusammenführen. | `SyncBase` |
| Konflikt | conflict | Objekt, das seit dem letzten Abgleich auf beiden Seiten verschieden geändert wurde. | `SyncConflict` |

## Später / later

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Routenanalyse | route analysis | Berechnung der schnellsten und alternativer Routen je Strecke aus Kostenfläche und Höhenmodell. | `route_analysis` |
| Kostenfläche | cost surface | Laufzeit je Fläche, abgeleitet aus den Kartensymbolen und der Steigung. | `CostSurface` |
| Laufgeschwindigkeit | symbol speed | Geschwindigkeitsfaktor eines Kartensymbols in der Kostenfläche. | `symbol_speed` |
| Postensetzer | control placer | Helfer, der Posten ausbringt oder einsammelt. | `ControlPlacer` |
| Tour | control-placer tour | Geordnete Postenliste eines Postensetzers mit Start- und Endpunkt; gedruckt von freemapper-print. | `Tour` |

## Assistenten / assistants

| Deutsch | English | Definition | Code |
|---|---|---|---|
| MCP-Server | MCP server | Teil des Programms, der KI-Assistenten über das Model Context Protocol auf stdin und stdout Werkzeuge zum Lesen und Ändern eines Projekts anbietet. Das Programm selbst enthält keine KI (ADR 0016). | `mcp::CoursePlanServer` |
| Werkzeug | tool | Eine Funktion des MCP-Servers mit englischem Namen, zum Beispiel `add_control`. Beschreibung und Parametertexte stehen im Katalog. | `#[tool]` |
