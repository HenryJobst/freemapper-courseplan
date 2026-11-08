# freemapper-courseplan

[English version](README.en.md) · [Handbuch](docs/manual/de.md) · [Glossar](docs/glossary.md) · [Änderungen](CHANGELOG.md)

freemapper-courseplan ist das Bahnlegungsprogramm der freemapper-Suite. Man legt die Bahnen von Hand in einem
Editor im Browser, auf der fertigen Karte aus OpenOrienteering Mapper (OOM). Das Programm prüft sie gegen die
Wettkampfbestimmungen und gibt sie an die Auswertung und an den Druck weiter:

- [freemapper](https://github.com/HenryJobst/freemapper) liefert den Ausgangsstand der Karte,
- die Karte zeichnet man in OOM,
- freemapper-courseplan legt die Bahnen,
- [freemapper-print](https://github.com/HenryJobst/freemapper-print) druckt die Kartenblätter.

Geplant ist für die erste Version:

- Posten mit Code und Postenbeschreibung, Einzelbahnen mit Knickpunkten, Pflichtstrecken und Sperrgebieten;
- Lage der Postennummern zentral je Posten und je Bahn änderbar, mit automatischer Platzierung;
- Prüfungen nach den Wettkampfbestimmungen 2026, etwa Postenabstände, spitze Winkel und genug verschiedene erste
  Posten für eine kompakte Startliste, jeweils mit Quelle;
- Abgleich mit Purple Pen (`.ppen`) in beide Richtungen, damit beide Programme parallel nutzbar sind;
- Export nach IOF-XML 3.0 für die Auswertung, zum Beispiel in OE;
- Sprint- und Schulhofkarten; Waldkarten kommen später;
- Bedienung über den Editor im Browser, über die Kommandozeile oder über einen KI-Assistenten (MCP).

Danach kommen eine Routenanalyse, Mannschafts-OL mit Pflicht- und Wahlposten, Staffeln, Score-OL, Touren für die
Postensetzer und eine KI-Unterstützung, die Bahnen beurteilt und Vorschläge macht.

Das Programm braucht kein OCAD und läuft auf macOS, Linux und Windows.

## Status

freemapper-courseplan ist früh in der Entwicklung. Auf einer Sprint- oder Schulhofkarte (`.omap`) legst du schon
ganze Einzelbahnen an. `freemapper-courseplan init <Karte>` erzeugt die Projektdatei. Mit `control`, `special`,
`class` und `course` legst du Posten, Start, Ziel, Kategorien und Bahnen an, setzt Knicke und lässt die
Postennummern platzieren. `freemapper-courseplan info <Projekt>` zeigt, was im Projekt steht, auch die Länge jeder
Bahn. Die Projektdatei ist TOML, du kannst sie auch von Hand bearbeiten. Mannschafts-, Staffel- und Score-Bahnen
kann das Programm noch nicht bearbeiten oder messen. Prüfen und drucken kann es auch noch nicht, und Waldkarten
liest es nicht.

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

## Lizenz

freemapper-courseplan steht unter der [GPL-3.0-or-later](LICENSE), wie freemapper und freemapper-print (ADR 0002).
