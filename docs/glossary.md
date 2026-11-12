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
| Score-Bahn | score course | Menge von Posten mit Werten, Zeitlimit und Strafe; Reihenfolge frei. Kann Pflichtverbindungen haben; Einstellungen in `[score]`. | `CourseKind::Score` |
| Postenwert | control value | Punkte eines Postens der Score-Bahn. Der Code trägt ihn: bei zweistelligem Code die zweite Ziffer (31 gibt 1), bei längerem die letzte (ADR 0060). | `ScoreControl::value` |
| Pflichtverbindung | mandatory link | Paar von Posten einer Score-Bahn, das direkt nacheinander angelaufen werden muss, in beliebiger Richtung. Wer es nicht tut, wird disqualifiziert. Verbindungen bilden Ketten: an einem Posten treffen höchstens zwei. | `ScoreLink` |
| Läuferprofil | runner profile | Pace eines guten (Standard 4:10 min/km) und eines sehr guten Läufers (3:45), mit der Punktequote bzw. Zeitquote, die jeder auf der Bahn erreichen soll. Einstellungen in `[score]`. | `ScoreSettings` |
| Wert-1-Abstand | value-1 distance | Kleinster Abstand eines Postens einer Score-Bahn von Start und Ziel in Metern im Gelände (Standard 150). Wert 1 geht an die nächsten Plätze, beginnt also dort (ADR 0072). | `ScoreSettings::value1_min_m` |
| Postenabstand (Score) | score control gap | Kleinster Abstand zweier Posten einer Score-Bahn in Metern (Standard 100, nicht kleiner). `check` warnt darunter (ADR 0072). | `ScoreSettings::min_gap_m` |
| Reichweite | reach | Größter Abstand eines Postens vom Start in Metern. Ohne Angabe ein Viertel der Strecke, die der gute Läufer im Zeitlimit schafft (ADR 0072). | `ScoreSettings::max_reach_m` |
| Abstand hochwertiger Posten | high-value gap | Kleinster Abstand zweier hochwertiger Posten in Metern (Standard 200, mindestens der Postenabstand). Hochwertig ist ein Wert über 0,6 des höchsten Werts, bei 5 also 4 und 5 (ADR 0072). | `ScoreSettings::high_value_gap_m` |
| Offensichtliche Reihenfolge | obvious order | Eine Score-Bahn, deren Posten im Ring um Start und Ziel stehen oder auf der äußeren Hülle liegen und sich fast ebenso kurz vom jeweils nächsten Posten aus ablaufen lassen. `check` meldet sie, weil sie dem Läufer die Wahl nimmt (ADR 0066). | `check::score` |
| Kandidat | candidate | Stelle, an der ein Posten einer Score-Bahn stehen könnte: Lage, Symbolcode des Kartenobjekts, Routenwahl, Schwierigkeit und Laufzeit vom Start. `score suggest` ordnet sie (ADR 0062). | `Candidate` |
| Klarheit | clarity | Zahl von 0 bis 1 je Art von Kartenobjekt in `candidates.toml`: wie sicher ein Läufer die Stelle erkennt. Feste, scharfe Dinge (Gebäudeecke, Mauerende) liegen hoch, bewegliche oder weiche (Busch, Pfadgabelung, Pflasterkante) niedrig; niedrige Klarheit macht die Stelle schwerer, schließt sie aber nicht aus (ADR 0062, 0073). | `Classes`, `Site::clarity` |
| Kandidatenart | site kind | Wo auf einem Kartenobjekt ein Kandidat liegt: Punkt, Ecke einer Kontur, freies Ende einer offenen Linie oder Gabelung, an der eine Linie auf eine andere trifft (ADR 0073). | `Kind` |
| Routenwahl | route choice | Zahl von 0 bis 1: wie viele der Strecken von einem Kandidaten zu anderen und zum Start mehrere ähnlich schnelle Routen bieten. | `Candidate::route_choice` |
| Entwurf | draft | Vorschlag einer Score-Bahn aus Kandidaten mit Werten und Codes, der die Längenbänder beider Läuferprofile trifft. `score draft` schreibt ihn nur mit `--apply` (ADR 0065). | `Draft` |
| Längenband | length band | Bereich in Prozent, in den die Punktequote des guten Läufers (Standard 87 bis 93) und die Zeitquote des sehr guten Läufers (95 bis 100) fallen sollen; Einstellung in `[score]` (ADR 0064, 0065). | `ScoreSettings::good_band_percent` |
| Auswertungsbogen | evaluation sheet | Markdown-Text zu einer Score-Bahn mit Zeitlimit, Abzug, Gleichstandsregel, Pflichtverbindungen und Postenwerten. `export --sheet` schreibt ihn (ADR 0070). | `score::sheet::evaluation_sheet` |
| Bahnposten | course control | Ein Posten in einer Bahn, mit eigener Nummernlage und eigenen Knickpunkten der folgenden Strecke. | `CourseControl` |
| geplanter Druckmaßstab | planned print scale | Maßstab, für den die Bahn gelegt wird. Grundlage für maßstabsabhängige Prüfungen; für freemapper-print nur ein Vorgabewert. | `planned_print_scale` |
| Sonderobjekt | special object | Bahnobjekt außer Posten: Start, Ziel, Pflichtstrecke, Sperrgebiet, verbotene Route, Gefahrenbereich, Erste-Hilfe-Stelle, Verpflegungsstelle. | `SpecialObject` |
| Querungsstelle | crossing point | Pflichtübergang, etwa über eine Straße oder durch einen Zaun. Ein Posten, den die Bahn in ihrer Folge führt, der aber keine Nummer bekommt. | `ControlKind::CrossingPoint` |
| abgeflaggte Strecke | taped leg | Strecke, die als Pflichtstrecke abgeflaggt ist; am Bahnposten oder, ab dem Start, an der Bahn. | `taped`, `start_taped` |
| Streckengeometrie | leg geometry | Linienzug einer Strecke vom einen Bahnpunkt zum nächsten (Start, Posten, Querungsstelle, Ziel) über die Knickpunkte, mit Länge in Kartenmillimetern und Kennzeichen abgeflaggte Strecke. | `LegGeometry` |
| Bahnlänge | course length | Summe aller Streckenlinienzüge in Kartenmillimetern mal Kartenmaßstab: Luftlinie plus gezeichnete Umwege (Knickpunkte), abgeflaggte Strecken entlang ihrer Knickpunkte gemessen (WKB B 7.2.5). In Metern. | `course_length_m` |
| Aufdruckgrößen | overprint sizes | Kreisdurchmesser, Linienbreite und Nummernhöhe des Bahnaufdrucks in Druckmillimetern beim Grundmaßstab der Kartenart: ISSprOM 2019-2 für Sprint und Schulhof, ISOM 2017-2 für Wald. | `OverprintSizes` |
| Grundmaßstab | base scale | Maßstab, für den die Aufdruckgrößen gelten: Sprint 1:4000, Schulhof 1:1000, Wald 1:15000. Auf der Karte gilt `map_mm = print_mm × Kursfaktor` (`numbers::course_factor`, ADR 0058): `base_scale / map_scale` bei den Maßstäben der Norm, 1,0 bei allen anderen. | `base_scale` |
| Nummernabstand | number gap | Teil der Nummernlage: kürzester Abstand zwischen der Außenkante der Kreislinie und dem Feld der Postennummer, in Druckmillimetern. Das Feld sitzt mittig auf dem Strahl in Richtung der Nummernlage. Mit freemapper-print abgestimmt. | `gap_mm` |
| bevorzugte Nummernrichtung | preferred number direction | Richtung, in der die automatische Platzierung die Postennummer zuerst versucht: hinter dem Kreis in Verlängerung der ankommenden Strecke, bei fast gestrecktem Winkel genau außen. | `preferred` |
| Außenseite des Winkels | outer side of the angle | Der größere der beiden Bereiche, in die ankommende und abgehende Strecke den Raum um den Kreis teilen; dorthin zeigt die Spitze des Winkels. Ihre Mitte ist die Winkelhalbierende außen. | `outer_bisector` |
| Druckbereich | print area | Rechteck der Karte, das das Blatt einer Bahn zeigt, in Millimetern auf der Karte. | `PrintArea` |
| Staffelteil | relay part | Gemeinsamer Teil oder Gabelung einer Staffelbahn. | `RelayPart` |
| Score-Posten | score control | Posten einer Score-Bahn mit seinem Wert. | `ScoreControl` |
| Eingabedateien | sources | Karte und optional Purple-Pen-Projekt der Veranstaltung, Pfade relativ zur Projektdatei. | `Sources` |
| Gefahrenbereich | dangerous area | Fläche, vor der der Bahnaufdruck mit violetter Markierung warnt (WKB B 7.2.1). | `SpecialObject::DangerousArea` |
| Erste-Hilfe-Stelle | first aid | Ort der Ersten Hilfe, auf der Karte eingezeichnet. | `SpecialObject::FirstAid` |
| Verpflegungsstelle | water | Ort mit Getränken für die Läufer, auf der Karte eingezeichnet. | `SpecialObject::Water` |
| Sperrgebiet | out-of-bounds area | Fläche, die nicht betreten werden darf (ISOM/ISSprOM 709). | `OutOfBoundsArea` |
| verbotene Route | forbidden route | Linie, die nicht begangen werden darf (ISOM/ISSprOM 711). Ein einzelner Punkt ist ein einzelnes Kreuz, wie Purple Pen es setzt. | `ForbiddenRoute` |

## Veranstaltung / event

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Disziplin | discipline | Sprint, Mittel, Lang, Ultralang oder Training. Training schaltet die Tabellenprüfungen ab. | `Discipline` |
| Kategorie | class | Startklasse mit freiem Namen, passend zu OE; optional einer Regelkategorie zugeordnet. | `Class` |
| Regelkategorie | rule category | Kategorie nach dem Regelwerk eines Profils, etwa nach WKB (H21, D-12, 130T …), an der Längentabelle und Schwierigkeitsstufe hängen. | `RuleCategory` |
| Kartenpunkt | map point | Punkt in Millimetern auf der Karte im Kartenmaßstab, `f64`, y nach unten. `omap::MapCoord` ist die Dateiform der OOM in 1/1000 mm. | `MapPoint` |
| Kartenindex | map index | Alle Geländeobjekte einer Karte in Millimetern, abfragbar nach Abstand zu einem Punkt. Blatt-Objekte (`freemapper:source=sheet`) fehlen. | `MapIndex` |
| Kartenobjekt für Abfragen | map feature | Ein Geländeobjekt im Kartenindex: Symbolnummer, Kartenteil und Geometrie (Punkt, Linie, Fläche). | `MapFeature` |
| Startfenster | start window | Zeit vom ersten bis zum letzten Start, in ganzen Minuten, mindestens 1; `event start-window` setzt es. Die Startkapazität teilt die erwarteten Starter dadurch. | `start_window_min` |
| erster Posten | first control | Der erste Posten einer Bahn nach dem Start. | `first_control` |
| Sprache | language | Sprache der Programmtexte: `de` oder `en`, gewählt mit `--lang`, `FREEMAPPER_LANG` oder der Systemsprache (ADR 0022). | `Lang` |

## Prüfen / checking

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Bundesland | state | Deutsches Bundesland, etwa Berlin oder Bayern, mit eigenen Bestimmungen und Empfehlungen. In deutschen Texten nie „Land“ *(Nutzer, 2026-11-09)*. | – |
| Land | country | Ein Staat, etwa Deutschland, Tschechien oder Polen. Nur in dieser Bedeutung *(Nutzer, 2026-11-09)*. | – |
| Regelprofil | rule profile | Satz von Prüfregeln aus einem Regelwerk, zuerst WKB 2026. | `RuleProfile` |
| Prüfregel | check rule | Eine Regel mit Kennung, Werten und Quelle (Dokument und Abschnitt), als Daten gespeichert. | `Rule` |
| Prüfregel-Schlüssel | rule key | Name der Prüfung, die eine Prüfregel ausführt, etwa `min_distance` oder `length_table`. Das Programm kennt eine feste Liste; ein Regelprofil mit einem anderen Schlüssel wird abgelehnt. | `key` |
| Regelprofil-Aufsatz | rule overlay | Regelprofil, das ein Basisprofil ergänzt oder einzelne Regeln ersetzt, zum Beispiel für das Regelwerk eines Bundeslandes; der Veranstaltung zugeordnet über `rule_overlays`. | `rule_overlays` |
| Befund | finding | Ergebnis einer Prüfregel an einem Posten, einer Strecke oder einer Bahn. | `Finding` |
| akzeptierter Befund | accepted finding | Befund, den der Bahnleger mit Begründung als gewollte Ausnahme markiert hat; wird nicht mehr gemeldet. | `AcceptedFinding` |
| nicht zugeordnete Ausnahme | unmatched acceptance | Akzeptierter Befund, zu dem `check` keinen Befund mehr findet, etwa weil sein Posten oder seine Bahn fehlt oder seine Strecke jetzt zwischen anderen Posten läuft. `check --all` listet ihn, er zählt nie als Fehler. | `CheckReport::unmatched` |
| übersprungene Prüfregel | skipped rule | Prüfregel, die `check` für eine Veranstaltung oder Bahn nicht ausführt, mit dem Grund: Filter (Disziplin, Kartenart, Druckmaßstab) passt nicht, Angabe fehlt, ein Profil hat sie abgeschaltet. Auch eine Bahn, die nicht Einzelbahn ist. Wird gemeldet, nie stillschweigend als bestanden gezählt. | `Skipped` |
| abgeschaltete Prüfregel | switched-off rule | Prüfregel, die ein Profil mit `enabled = false` abschaltet, meist ein Aufsatz. `check --all` führt sie als übersprungen, mit der Kennung dieses Profils. | `SwitchedOff` |
| Vorschlag des Programms | proposal of the program | Grenzwert, den keine Quelle als Zahl nennt und den freemapper-courseplan vorschlägt; im Profil mit `doc = "claude"` und einer Anmerkung. Die Ausgabe nennt als Quelle „Vorschlag von freemapper-courseplan“. | `PROPOSAL_DOC` |
| Ort eines Befunds | finding location | Wo ein Befund sitzt: Veranstaltung, Bahn, ein Posten, mehrere Posten, eine Strecke oder zwei oder mehr Strecken einer Bahn. Start und Ziel zählen dabei als Posten. Bestimmt mit der Kennung der Prüfregel, welcher akzeptierte Befund passt. | `Location` |
| Strecken-Bezug | leg reference | Verweis auf eine Strecke einer Bahn in einem akzeptierten Befund: Bahnname, Position, von der die Strecke abgeht (ab 1, Querungsstellen mitgezählt; 0 ist die Strecke vom Start), und die Kennungen ihrer beiden Enden. Die Position ist nicht die Postennummer auf der Karte. | `LegRef` |
| Streckenende | leg end | Start, Posten oder Querungsstelle oder Ziel am Anfang oder Ende einer Strecke. Die Ausgabe nennt eine Strecke nach ihren Enden: „Strecke 36–37“, „Strecke Start–31“, „Strecke 37–Ziel“. | `LegEnd` |
| ähnliche Objekte | similar features | Postenobjekte, die im Gelände oder auf der Karte verwechselbar sind (WKB B 7.2.1). | `similar_features` |
| Ortsart | place kind | Art des Orts eines Befunds: Veranstaltung, Bahn, ein Posten, zwei oder mehr Posten, Strecke, zwei oder mehr Strecken einer Bahn. Jede Prüfung nennt die Ortsarten ihrer Befunde; `finding accept` lehnt andere ab, weil ein solcher akzeptierter Befund nie passen würde. | `Place` |
| Postenobjekt | control feature | Das Kartenobjekt, an dem ein Posten steht: das nächste Kartenobjekt mit Symbolnummer innerhalb von `feature_search_mm` um die Postenmitte. Eine Fläche zählt mit ihrem Rand; die Fläche, in der der Posten nur liegt, ist also nicht sein Objekt. | `control_feature` |
| Kartenrand | map edge | Rand des Rechtecks um alle Geländeobjekte des Kartenindex, ohne Blatt-Objekte. | `MapIndex::extent` |
| Sperrgebiet-Symbol | out-of-bounds symbol | Kartensymbol eines Sperrgebiets (ISOM/ISSprOM 520, 708, 709, auf Sprintkarten auch 714), als Symbolnummer in `impassable` von `leg_through_forbidden`. Eine Unternummer gehört zu ihrer Nummer: 709.1 zu 709, 7091 nicht. | `impassable`, `symbol_matches` |
| gezeichneter Teil einer Strecke | drawn part of a leg | Streckenlinienzug ohne je einen Kreisradius an beiden Enden, so wie der Bahnaufdruck ihn zeichnet. Die Prüfungen auf Strecken durch Posten und Sperrgebiete messen nur diesen Teil. | `drawn_part` |
| spitzer Winkel | acute angle | Winkel zwischen ankommender und abgehender Strecke an einem Posten oder einer Querungsstelle, vom letzten Knickpunkt davor zum ersten danach gemessen: 180° geradeaus, 0° zurück auf demselben Weg. Start und Ziel haben keinen. | `acute_angle` |
| fast gestreckter Winkel | almost straight angle | Winkel zwischen ankommender und abgehender Strecke an einem Posten über dem Wert der Prüfregel (im Profil 170°): Die Bahn läuft fast geradeaus weiter. Gemessen wie der spitze Winkel. | `straight_angle` |
| Abstandsbereich der Nummer | number gap range | Kleinster und größter erwünschter Nummernabstand laut Prüfregel (im Profil 0,5 bis 5 mm) | `number_gap`, `placement::GapRange` |
| Nummernbeschriftung | number label | Text, der an der Postennummer steht: nur die Nummer ("1"), Nummer und Code mit Strich ("1-31") oder mit Klammer ("1 (31)"). Der Text bestimmt die Breite des Nummernfelds. Auf Staffel- und Score-Karten gewollt; kommt mit deren Meilenstein. | `number_label` |
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
| Konflikt | conflict | Feld eines Objekts, das seit dem letzten Abgleich auf beiden Seiten verschieden geändert wurde (auch Löschen gegen Ändern). | `SyncConflict` |
| Seitenwahl | pick | Ergebnis der Dreiwegeregel für ein Feld: unverändert, nur hier geändert, nur in der `.ppen` geändert, auf beiden Seiten gleich geändert oder Konflikt. | `sync::merge3::Pick` |
| Purple-Pen-Sicht | Purple Pen view | Was der Import aus einer `.ppen` macht. Basis, Projekt (als Export auf die Basis) und Partnerdatei werden alle in dieser Sicht verglichen, damit Purple Pens Standardwerte und die verlustbehaftete Abbildung nicht als Änderung erscheinen. | `sync::view::Views` |
| Zuordnung | matching | Ordnet die Objekte der drei Sichten einander zu: Purple-Pen-Kennung, dann Code oder Name, neue Objekte nach Code, Name oder Form. Jedes Objekt gehört zu höchstens einem Tripel. | `sync::matching::Matching`, `Triple` |
| Abgleichsfeld | sync field | Kleinste Einheit des Zusammenführens: ein Wert eines Objekts in der Purple-Pen-Sicht (Code, Position, eine Beschreibungsspalte, Postenfolge, Nummernlage je Besuch, Strecke). Jedes Feld bekommt eine Seitenwahl; Löschen gegen Ändern eines Objekts ist ein Konflikt auf dem Objekt. | `sync::fields::FieldKey`, `FieldOutcome` |
| Abgleichsentscheidung | sync decision | Gespeicherte Antwort auf einen Konflikt: Schlüssel des Konflikts, gewählte Seite (Projekt oder `.ppen`) und Fingerabdruck seiner beiden Werte. Sie gilt nur, solange die Werte unverändert sind, und der nächste vollständige Abgleich löscht sie. Im Abschnitt `[sync]` der Projektdatei. | `SyncDecision`, `Side` |
| Abgleichsplan | sync plan | Ergebnis von `sync_ppen`: entweder die offenen Konflikte (nichts wird geschrieben) oder das fertige Projekt, der neue Text der `.ppen` (leer, wenn er der Partnerdatei gleicht) und der neue Stand der Abgleichsbasis. | `SyncPlan` |
| Abgleichshinweis | sync note | Meldung des Abgleichs über etwas, das er anders gemacht hat als erwartet: eine behaltene Löschung, verworfene Knicke, ein vergebener Name, ein nicht verwendeter Basisstand. | `SyncNote` |
| Basiszustand | base state | Urteil über die gespeicherte Abgleichsbasis vor einem Abgleich: verwenden, reparieren (der Abgleich wurde nach dem Schreiben der Projektdatei unterbrochen, die Basis wird die Partnerdatei) oder ignorieren (fehlt, gehört zu einer anderen Verknüpfung oder passt nicht zum gespeicherten Hash). Ohne Basis löscht der Abgleich nichts. | `sync::base::BaseState` |
| Postenart (Wort) | kind word | Lesbares Wort für die Art eines Postens in Abgleichsmeldungen (`control`, `crossing point`, `start`, `finish`); der Schlüssel, den die Oberfläche übersetzt. | `sync::decide::KindWord` |
| Import | import | Einmaliges Übernehmen einer `.ppen` in die Veranstaltung: Posten, Beschreibungen, Bahnen, Knickpunkte, abgeflaggte Strecken, Nummernlagen und Sonderobjekte. Was das Modell nicht fassen kann, wird nie stillschweigend weggelassen, sondern als Importhinweis genannt. | `import_ppen`, `PpenImport` |
| Importhinweis | import note | Meldung des Imports über etwas, das er nicht oder nur verändert übernimmt: übersprungene Bahn, Lücken, unbekanntes Beschreibungssymbol, Nummer im Kreis, Sonderobjekt ohne Gegenstück. | `ImportNote` |
| `.ppen`-Export | `.ppen` export | Schreibt die Bahnen einer Veranstaltung als Purple-Pen-Datei, neu oder in eine vorhandene Basisdatei. Wo das Projekt noch dem entspricht, was der Import aus der Basisdatei gemacht hat, schreibt er deren eigene Werte zurück; ein unverändertes Projekt ergibt die Basisdatei Byte für Byte. Gelöscht wird nichts. | `ppen_export::export_ppen`, `PpenExport` |
| Basisdatei | base file | Die vorhandene `.ppen`, in die ein Export schreibt (Begriff aus freemapper-core). Was das Projekt nicht erwähnt, bleibt als behaltenes Objekt stehen. | `Target::base`, `KeptObject` |
| Exporthinweis | export note | Meldung des `.ppen`-Exports über etwas, das er nicht oder anders schreibt: übersprungene Bahn oder übersprungenes Sonderobjekt, Symbol ohne Purple-Pen-Kennung, nicht gespeicherter Code, von Purple Pen selbst abgeflaggte Strecke, weggefallene Teilabflaggung, behaltenes Objekt. | `ExportNote` |
| Purple-Pen-Kennung | Purple Pen id | Die interne Nummer eines Postens, einer Bahn oder eines Sonderobjekts in der `.ppen` (`control/@id`, `course/@id`, `special-object/@id`), beim Import mitgespeichert; bei Start und Ziel ist es die Kennung des Postens. Zweiter Schlüssel beim Abgleich, damit ein geänderter Code oder Bahnname als Änderung erkannt wird. Hat ein Sonderobjekt mehrere Ringe, trägt nur das erste Objekt die Kennung. | `ppen_id` |

## Editor / editor

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Editor | editor | Die Bahnlegung im Browser: `freemapper-courseplan edit` startet einen lokalen Server, der eine eingebettete Seite und eine JSON-Schnittstelle anbietet. Links der Inhalt der Veranstaltung, in der Mitte die Karte, rechts Eigenschaften und Befunde. Jede Änderung geht durch dieselben Operationen wie CLI und MCP und wird sofort in der Projektdatei gespeichert (ADR 0042). | `editor`, `editor::run` |
| Stand | revision | Fingerabdruck der Projektdatei (FNV-1a über ihre Bytes, 16 Hexziffern). Jede ändernde Anfrage nennt den Stand, den die Seite zuletzt gesehen hat; weicht er von der Datei ab, wird nichts geschrieben. | `Revision` |
| Rückgängig-Verlauf | undo history | Die letzten Änderungen des Editors als Schritte aus Dateitext vorher und nachher. Rückgängig schreibt den alten Text nur, solange die Datei noch dem neuen entspricht. | `UndoHistory` |
| Kartenansicht | map view | Das, was der Editor in der Mitte zeigt: ein Kartenausschnitt als SVG in Seitenmillimetern samt Abbildung von Kartenmillimetern auf die Seite. Hinter einer kleinen Schnittstelle, damit der gemeinsame Renderer die Platzhalterkarte ablösen konnte. Er tut es mit `RenderMap` (ADR 0056). | `MapView` |
| Druckvorschau | print preview | Die Ansicht des Editors, in der der gemeinsame Renderer die Bahn als Purpurebenen in das Kartenbild malt, schreibgeschützt und ohne Overlay (ADR 0057). | `svg_with_course` |
| Bahnüberlagerung | course overlay | Die Bahnsymbole (Kreise, Start, Ziel, Querungsstellen, Strecken, Nummernfelder, Sonderobjekte) in Kartenmillimetern, vom Server berechnet und von der Seite nur gezeichnet. | `CourseOverlay` |
| Auswahl | selection | Was im Editor markiert ist: eine Bahn, eine Kategorie, ein Posten, eine Querungsstelle oder ein Sonderobjekt. Die Eigenschaften rechts gelten für sie. | `Selection` |
| Platzhalterkarte | placeholder map | Kartenansicht ohne Karteninhalt: Umriss der Kartenausdehnung, ein Raster von 10 mm und ein Hinweis in der Mitte. Sie ist nur noch für Modultests da; der Editor zeigt die echte Karte. | `PlaceholderMap` |

## Später / later

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Routenanalyse | route analysis | Berechnung der schnellsten und alternativer Routen je Strecke aus Kostenfläche und Höhenmodell. | `route_analysis` |
| Kostenfläche | cost surface | Laufzeit je Fläche, abgeleitet aus den Kartensymbolen und der Steigung. Die Score-Bahnen rechnen mit einem Raster der Kartensymbole ohne Höhenmodell. | `CostSurface` |
| Laufgeschwindigkeit | symbol speed | Geschwindigkeitsfaktor eines Kartensymbols in der Kostenfläche. | `symbol_speed` |
| Postensetzer | control placer | Helfer, der Posten ausbringt oder einsammelt. | `ControlPlacer` |
| Tour | control-placer tour | Geordnete Postenliste eines Postensetzers mit Start- und Endpunkt; gedruckt von freemapper-print. | `Tour` |

## Assistenten / assistants

| Deutsch | English | Definition | Code |
|---|---|---|---|
| MCP-Server | MCP server | Teil des Programms, der KI-Assistenten über das Model Context Protocol auf stdin und stdout Werkzeuge zum Lesen und Ändern eines Projekts anbietet. Das Programm selbst enthält keine KI (ADR 0016). | `mcp::CoursePlanServer` |
| Werkzeug | tool | Eine Funktion des MCP-Servers mit englischem Namen, zum Beispiel `add_control`. Beschreibung und Parametertexte stehen im Katalog. | `#[tool]` |
| IOF-XML-Export | IOF XML export | Schreibt die Einzel- und Score-Bahnen einer Veranstaltung als IOF-XML-3.0-`CourseData` für Auswertungssoftware wie OE: Posten mit Kartenposition, Bahnen mit Postenfolge, Streckenlängen und Nummernlage, Zuordnung der Bahnen zu Kategorien; eine Score-Bahn trägt den Wert je Posten und in einem Erweiterungselement Zeitlimit, Abzug und Verbindungen. Mannschafts- und Staffelbahnen werden übersprungen und genannt. | `iof_xml::export_iof_xml` |
| Vertrag der Suite | suite contract | Was freemapper in Karte und `freemapper.toml` schreibt und worauf sich freemapper-courseplan verlässt: die Tags `freemapper:source=sheet` und `base`, die Kartenteile „peripheral information“ und `base`, die Schlüssel `map_kind` und `title`. Festgehalten in `freemapper-contract` (freemapper-core, dort ADR 0018) und hier in jedem Testlauf geprüft (ADR 0041). | `contract_*` (Tests) |
| Selbstaktualisierung | self-update | Das Programm ersetzt sich auf Wunsch durch die neueste signierte Version von den GitHub-Releases, am selben Pfad, mit Weg zurück (`update --rollback`). Nur Release-Builds; ein aus dem Quellcode gebautes Programm tut es nie (ADR 0076, freemapper ADR 0094). | `update::command` |
| Aktualisierungsmanifest | update manifest | Die signierte Datei `freemapper-courseplan-update.json` jeder Version: Versionsnummer, Datum, Link auf die Versionshinweise, die älteste Version, die sie installieren darf (`min_self_update_from`), und je Plattform URL, Größe und SHA-256 des `.gz` und der Programmdatei. Das Feld `app` hält ein Manifest von freemapper fern. | `freemapper_update::Manifest` |
| Aktualisierungsmodus | update mode | Was die tägliche Prüfung tut, wenn sie eine neuere Version findet: `notify` (Hinweis, Standard), `install` (installiert nach einem erfolgreichen Befehl) oder `off`. Der MCP-Server installiert in keinem Modus selbst. | `freemapper_update::Mode` |
