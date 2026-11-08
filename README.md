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

freemapper-courseplan ist früh in der Entwicklung. Zwei Befehle laufen bisher.
`freemapper-courseplan init <Karte>` legt aus einer Sprint- oder Schulhofkarte (`.omap`) eine Projektdatei an,
und `freemapper-courseplan info <Projekt>` zeigt, was darin steht. Die Projektdatei ist TOML, du kannst sie von
Hand bearbeiten. Bahnen anlegen, prüfen und drucken kann das Programm noch nicht, Waldkarten auch nicht.

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
