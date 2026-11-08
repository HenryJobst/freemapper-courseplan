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
| Sonderobjekt | special object | Bahnobjekt außer Posten: Start, Ziel, Pflichtstrecke, Sperrgebiet, verbotene Route, Gefahrenstelle, Erste Hilfe, Verpflegung. | `SpecialObject` |
| Querungsstelle | crossing point | Pflichtübergang, etwa über eine Straße oder durch einen Zaun. Ein Posten, den die Bahn in ihrer Folge führt, der aber keine Nummer bekommt. | `ControlKind::CrossingPoint` |
| markierte Strecke | taped leg | Strecke, die als Pflichtstrecke markiert ist; am Bahnposten oder, ab dem Start, an der Bahn. | `taped`, `start_taped` |
| Druckbereich | print area | Rechteck der Karte, das das Blatt einer Bahn zeigt, in Millimetern auf der Karte. | `PrintArea` |
| Staffelteil | relay part | Gemeinsamer Teil oder Gabelung einer Staffelbahn. | `RelayPart` |
| Score-Posten | score control | Posten einer Score-Bahn mit seinem Wert. | `ScoreControl` |
| Eingabedateien | sources | Karte und optional Purple-Pen-Projekt der Veranstaltung, Pfade relativ zur Projektdatei. | `Sources` |
| Gefahrenstelle | dangerous area | Fläche, vor der die Bahn warnt. | `SpecialObject::DangerousArea` |
| Erste Hilfe | first aid | Erste-Hilfe-Punkt auf der Karte. | `SpecialObject::FirstAid` |
| Verpflegung | water | Verpflegungsstelle auf der Karte. | `SpecialObject::Water` |
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

## Prüfen / checking

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Regelprofil | rule profile | Satz von Prüfregeln aus einem Regelwerk, zuerst WKB 2026. | `RuleProfile` |
| Prüfregel | check rule | Eine Regel mit Kennung, Werten und Quelle (Dokument und Abschnitt), als Daten gespeichert. | `CheckRule` |
| Befund | finding | Ergebnis einer Prüfregel an einem Posten, einer Strecke oder einer Bahn. | `Finding` |
| akzeptierter Befund | accepted finding | Befund, den der Bahnleger mit Begründung als gewollte Ausnahme markiert hat; wird nicht mehr gemeldet. | `AcceptedFinding` |
| ähnliche Objekte | similar features | Postenobjekte, die im Gelände oder auf der Karte verwechselbar sind (WKB B 7.2.1). | `similar_features` |

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
