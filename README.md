# freemapper-courseplan

[English version](README.en.md) · [Handbuch](docs/manual/de.md) · [Glossar](docs/glossary.md) · [Änderungen](CHANGELOG.md)

<!-- claude-cost:start -->
[![KI-Aufwand (Claude API): 449 $](https://img.shields.io/badge/KI--Aufwand%20%28Claude%20API%29-449%20%24-8A2BE2)](docs/ai-cost/history.csv)
<!-- claude-cost:end -->

freemapper-courseplan ist das Bahnlegungsprogramm der freemapper-Suite. Man legt die Bahnen von Hand in einem
Editor im Browser, auf der fertigen Karte aus OpenOrienteering Mapper (OOM). Das Programm prüft sie gegen die
Wettkampfbestimmungen und gibt sie an die Auswertung und an den Druck weiter:

- [freemapper](https://github.com/HenryJobst/freemapper) liefert den Ausgangsstand der Karte,
- die Karte zeichnet man in OOM,
- freemapper-courseplan legt die Bahnen,
- [freemapper-print](https://github.com/HenryJobst/freemapper-print) druckt die Kartenblätter.

Code, den mehrere dieser Programme brauchen, liegt in einer gemeinsamen Bibliothek:
[freemapper-core](https://github.com/HenryJobst/freemapper-core). Das sind kleine Rust-Crates, zum Beispiel
`freemapper-text` für die Programmtexte, `freemapper-omap` für das Lesen der Karte, `freemapper-course` für das
Lesen von Purple-Pen-Dateien und `freemapper-web` für den lokalen Server des Editors. Die Programme binden sie in
einer festen Version ein (courseplan: 0.14.1), statt den Code zu kopieren. Die Lage der Postennummern und die
Tabellen der Beschreibungssymbole kommen von dort, daher platzieren courseplan und freemapper-print die Nummern
gleich.

Geplant ist für die erste Version:

- Posten mit Code und Postenbeschreibung, Einzelbahnen mit Knickpunkten, Pflichtstrecken und Sperrgebieten;
- Lage der Postennummern zentral je Posten und je Bahn änderbar, mit automatischer Platzierung hinter dem Kreis,
  abseits von Punkt- und Liniensymbolen der Karte, mit einem Abstand von 0,5 bis 5 mm zum Kreis (von `check` geprüft);
- Prüfungen nach den Wettkampfbestimmungen 2026 mit Aufsätzen für regionale Empfehlungen, etwa Postenabstände,
  spitze und fast gestreckte Winkel und genug verschiedene erste Posten für eine kompakte Startliste, jeweils
  mit Quelle und mit begründeten Ausnahmen (`check`, schon nutzbar);
- Abgleich mit Purple Pen (`.ppen`) in beide Richtungen, damit beide Programme parallel nutzbar sind (Import und
  Export und Abgleich sind schon nutzbar: `import`, `export --ppen` und `sync`; Purple Pen 4.0 öffnet den Export);
- Export nach IOF-XML 3.0 für die Auswertung, zum Beispiel in OE (`export`, schon nutzbar, der OE-Import ist noch nicht getestet);
- Sprint- und Schulhofkarten; Waldkarten kommen später;
- Bedienung über den Editor im Browser (`edit`, schon nutzbar: Posten, Bahnen, Kategorien, Knicke, Nummern und
  Sonderobjekte und Postenbeschreibungen, Befunde der Prüfung, Export, Import und Abgleich mit Purple Pen, mit
  Rückgängig und Wiederholen; auf der echten Karte), über die Kommandozeile oder über einen KI-Assistenten (MCP).

Danach kommen eine Routenanalyse, Mannschafts-OL mit Pflicht- und Wahlposten, Staffeln, Score-OL, Touren für die
Postensetzer und eine KI-Unterstützung, die Bahnen beurteilt und Vorschläge macht.

Das Programm braucht kein OCAD und läuft auf macOS, Linux und Windows.

## Beispiel

<a href="examples/luebeck-sprint/"><img src="examples/luebeck-sprint/preview.jpg" width="260" align="right" alt="Skizze einer Sprintbahn in der Lübecker Altstadt"></a>

Eine Sprintbahn mit sieben Posten in der Lübecker Altstadt, gelegt nur mit der Kommandozeile. Das Bild ist eine
Skizze: Die Karte stammt aus freemapper-print, die Bahn ist darübergezeichnet. Befehle, Projektdatei und Ausgaben
stehen unter [`examples/luebeck-sprint`](examples/luebeck-sprint/); alle Beispiele zeigt
[`examples/README.md`](examples/README.md).

```text
$ freemapper-courseplan info luebeck-sprint.courseplan.toml
Titel: Sprinttraining Lübecker Altstadt
Kartenart: Sprintkarte
Posten: 7, Querungsstellen: 0
Bahn A: 7 Posten, 1,3 km, Anstieg 5 m
```

<br clear="right">

## Installieren

Fertige Programme für Windows, macOS und Linux gibt es auf der
[Release-Seite](https://github.com/HenryJobst/freemapper-courseplan/releases). Archiv herunterladen, entpacken und
`freemapper-courseplan` im Terminal starten oder beim KI-Assistenten eintragen (Handbuch, Kapitel 3). Ein Assistent,
der Befehle ausführen kann (Claude Code, Codex), erledigt auch Download und Eintragen selbst; sag ihm „Lies AGENTS.md
von github.com/HenryJobst/freemapper-courseplan und richte freemapper-courseplan ein.“ Die Datei
[`AGENTS.md`](AGENTS.md) liegt in jedem Archiv.

## Status

freemapper-courseplan ist früh in der Entwicklung. Auf einer Sprint- oder Schulhofkarte (`.omap`) legst du schon
ganze Einzelbahnen an. `freemapper-courseplan init <Karte>` erzeugt die Projektdatei. Mit `control`, `special`,
`class` und `course` legst du Posten, Start, Ziel, Kategorien und Bahnen an, setzt Knickpunkte und lässt die
Postennummern platzieren. `freemapper-courseplan info <Projekt>` zeigt, was im Projekt steht, auch die Länge jeder
Bahn. Dasselbe kann ein KI-Assistent über den MCP-Server `freemapper-courseplan mcp` für dich tun; wie du ihn
einrichtest, steht in [Kapitel 3 des Handbuchs](docs/manual/de.md#3-mit-einem-ki-assistenten), und [`AGENTS.md`](AGENTS.md) sagt dem
Assistenten, wie er das Programm selbst installiert und benutzt. Die Projektdatei ist
TOML, du kannst sie auch von Hand bearbeiten. `import`, `export --ppen` und `sync` tauschen Veranstaltungen mit Purple Pen aus. Mannschafts-, Staffel- und Score-Bahnen kann das Programm noch nicht
bearbeiten oder messen. Drucken kann es auch noch nicht, und Waldkarten liest es nicht.

`freemapper-courseplan check <Projekt>` prüft die Einzelbahnen gegen ein Regelprofil, jeder Befund mit Schwere, Ort,
Prüfregel und Quelle. Eingebaut sind das Profil `wkb-2026` (Wettkampfbestimmungen 2026) und der Aufsatz
`nor-bb-2021` (Bahnempfehlungen Berlin/Brandenburg), `freemapper-courseplan event profile <Projekt> wkb-2026
--overlay nor-bb-2021` wählt sie. Eine gewollte Ausnahme trägt `finding accept <Projekt> <Prüfregel> --note TEXT` mit
Begründung ein, `finding remove` nimmt sie zurück, und `check --all` zeigt auch die akzeptierten Befunde, die
Ausnahmen, zu denen kein Befund mehr passt, und die übersprungenen Prüfregeln. `event start-window` und `class set`
tragen Startfenster, Regelkategorien und erwartete Starter nach, die manche Prüfungen brauchen. Das Handbuch erklärt
es in [Kapitel 8](docs/manual/de.md#8-prüfen-hinweise-lesen-ausnahmen-begründen), und der KI-Assistent hat dafür die
Werkzeuge `check`, `accept_finding`, `remove_accepted_finding`, `set_rule_profile`, `set_start_window` und
`set_class`. Mannschafts-, Staffel- und Score-Bahnen prüft das Programm noch nicht, und die Routenanalyse
fehlt, deshalb meldet es eine gerade Strecke durch ein Sperrgebiet nur als Warnung.

`freemapper-courseplan export <Projekt> --iof-xml <Datei>` schreibt die Einzelbahnen als IOF-XML 3.0 für die
Auswertung; der KI-Assistent hat dafür das Werkzeug `export_iof_xml`. Die Ausgabe ist gegen das IOF-Schema
geprüft und lässt sich mit dem IOF-Leser von freemapper-course (dem von freemapper-print) wieder einlesen; ein
fertiges Beispiel ist [`course-data.xml`](examples/luebeck-sprint/course-data.xml). Ein Import in OE, Purple Pen
oder Condes ist nicht getestet, weil im Projekt niemand OE hat. Rückmeldungen von Leuten mit OE sind willkommen. Ist die Karte georeferenziert, tragen Start, Posten und Ziel
auch Breite und Länge (ADR 0039). Abgeflaggte Strecken schreibt das Programm noch nicht, Mannschafts-, Staffel- und
Score-Bahnen überspringt es. Kapitel 10 des
[Handbuchs](docs/manual/de.md#10-weitergeben-iof-xml-für-oe) nennt die Einzelheiten.

`freemapper-courseplan init <Karte> --ppen <Datei>` und `freemapper-courseplan import <Projekt> --ppen <Datei>`
übernehmen Posten, Beschreibungen, Bahnen, Knickpunkte, abgeflaggte Strecken, Nummernlagen und Sonderobjekte aus einem
Purple-Pen-Event, das für dieselbe Karte gezeichnet ist; der KI-Assistent hat dafür das Werkzeug `import_ppen`. Was
das Modell nicht fassen kann, etwa Varianten, Score-Bahnen oder Lücken, nennt das Programm als Hinweis. Ein Projekt
mit Inhalt ersetzt `import` nur mit `--replace`; Kategorien und akzeptierte Befunde bleiben dabei. Kapitel 9 des [Handbuchs](docs/manual/de.md#9-mit-purple-pen-abgleichen) erklärt es.

`freemapper-courseplan export <Projekt> --ppen <Datei>` schreibt die Einzelbahnen als Purple-Pen-Event; der
KI-Assistent hat dafür das Werkzeug `export_ppen`. Eine vorhandene Datei ändert der Export an Ort und Stelle und
lässt stehen, was das Projekt nicht kennt, etwa Lücken in Kreisen oder Kartenwechsel; mit `--new` schreibt er sie neu.
Er löscht nie etwas und meldet, was in der Datei steht und im Projekt fehlt. Den Schreiber liefert `freemapper-course`
in freemapper-core. Das Beispiel [`luebeck-sprint.ppen`](examples/luebeck-sprint/luebeck-sprint.ppen) ist ein
Export. **Purple Pen 4.0 öffnet die Datei
ohne Fehler, und der Inhalt wirkt richtig** (geprüft vom Autor, 10.11.2026).

`freemapper-courseplan sync <Projekt>` führt das Projekt und die verknüpfte `.ppen` in beide Richtungen zusammen. Es
vergleicht Feld für Feld die Datei, wie der letzte Abgleich sie hinterlassen hat (die Basiskopie
`X.courseplan.ppen-base`, die in Git gehört), die Datei jetzt und das Projekt jetzt. Eine Änderung auf einer Seite
geht auf die andere. Haben beide Seiten dasselbe Feld verschieden geändert, nennt `sync` beide Werte und schreibt
nichts, bis du mit `--take ours`, `--take theirs` oder `--take SCHLÜSSEL=SEITE` eine Seite wählst; `--dry-run` meldet
nur. Kategorien, akzeptierte Befunde und Mannschafts-, Staffel- und Score-Bahnen bleiben im Projekt. Der
KI-Assistent hat die Werkzeuge `sync_ppen` und `resolve_sync_conflict`. Der Editor hat dafür eine Konfliktansicht (Menü
„Veranstaltung“). Siehe Kapitel 9 des [Handbuchs](docs/manual/de.md#9-mit-purple-pen-abgleichen) und ADR 0040.

`freemapper-courseplan edit <Projekt>` startet den Editor und zeigt die Veranstaltung im Browser: links den Baum aus
Bahnen, Kategorien, Posten und Sonderobjekten, in der Mitte die gewählte Bahn mit Kreisen, Strecken und Nummern, rechts
die Eigenschaften. Mit Maus oder Tastatur setzt, verschiebst und löschst du Posten, legst Bahnen und Kategorien an,
knickst und flaggst Strecken ab, ziehst oder setzt Postennummern, wählst die Beschreibungssymbole der Spalten C bis H
und zeichnest Sonderobjekte; Strg+Z und Strg+Umschalt+Z machen Schritte rückgängig und wiederholen sie. Jede Änderung
wird sofort gespeichert und läuft über dieselben Operationen wie Kommandozeile und KI-Assistent. Änderungen von außen
siehst du nach etwa zwei Sekunden im Browser; hat sich die Datei unter dem Editor geändert, lehnt er die Änderung ab
und lädt den neuen Stand. Die Befunde der Prüfung erscheinen im Feld rechts, in der Karte und in der Statusleiste, und du kannst einen Befund
mit einer Begründung akzeptieren. Das Menü „Veranstaltung“ exportiert IOF-XML und Purple-Pen-Dateien, importiert eine
Purple-Pen-Datei und gleicht mit der verknüpften ab; schließe die Veranstaltung vorher in Purple Pen. Die Karte ist
die echte, als SVG gezeichnet vom Kartenzeichner, den auch freemapper-print benutzt (freemapper-render). Eine Karte, die
im Kartenmaßstab größer ist als ein A3-Blatt, wird auf ihren mittleren Teil beschnitten, und die Statusleiste sagt das.
Die Druckvorschau ist noch nicht gebaut; sie wartet darauf, dass freemapper-print Bahnen zeichnet. Der Server hört nur auf `127.0.0.1` und verlangt ein Zugriffszeichen in der
Adresse. Siehe Kapitel 4 des [Handbuchs](docs/manual/de.md#4-der-editor) sowie ADR 0042, 0043, 0047, 0048,
0049, 0051, 0052, 0053, 0054 und 0056.

Im Repository liegen außerdem das [Design](docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md),
das [Glossar](docs/glossary.md), die [Architekturentscheidungen](docs/adr/README.md) und eine
[Sammlung der Regeln zur Bahnlegung](docs/research/2026-11-08-bahnlegung-kriterien.md).

## Selbst bauen (für Entwickler)

```sh
git clone https://github.com/HenryJobst/freemapper-courseplan
cd freemapper-courseplan
cargo build --release
```

`rust-toolchain.toml` legt die Rust-Version fest; `rustup` installiert sie beim ersten Bauen. Vor jedem Commit
prüft [lefthook](https://github.com/evilmartians/lefthook) Formatierung und clippy (`lefthook install`).

Wer freemapper-core und dieses Programm zusammen ändert, legt freemapper-core neben freemapper-courseplan ab und
bindet es während der Entwicklung über einen lokalen `[patch]` ein. Die Einzelheiten stehen in der README von
[freemapper-core](https://github.com/HenryJobst/freemapper-core).

## Lizenz

freemapper-courseplan steht unter der [GPL-3.0-or-later](LICENSE), wie freemapper und freemapper-print (ADR 0002).
