# Regelprofile

Deutsch | [English](README.en.md)

Der Befehl `check` prüft Bahnen gegen ein Regelprofil. Ein Regelprofil ist eine TOML-Datei in
diesem Ordner. Das Programm bettet die Dateien beim Bauen ein; ein neues oder geändertes Profil
braucht also einen neuen Build. ADR 0025 legt das Format fest, `src/rules.rs` liest es.

Es gibt zwei Arten von Regelprofilen:

- Ein **Basisprofil** enthält einen vollständigen Satz Prüfregeln, zum Beispiel `wkb-2026.toml`
  (Wettkampfbestimmungen OL 2026).
- Ein **Aufsatz** ergänzt genau ein Basisprofil, zum Beispiel `nor-bb-2021.toml`
  (Bahnempfehlungen Berlin/Brandenburg). Er fügt Prüfregeln hinzu, ersetzt Regeln des
  Basisprofils oder schaltet sie ab.

Eine Veranstaltung nennt ein Basisprofil in `rule_profile` und beliebig viele Aufsätze in
`rule_overlays`. Die Aufsätze gelten in der angegebenen Reihenfolge.

## Ein Regelprofil hinzufügen

1. Die kleinere der beiden Dateien kopieren und umbenennen. Der Dateiname ohne `.toml` ist die
   Kennung des Profils, etwa `iof-2026` oder `by-2025`. Erlaubt sind Kleinbuchstaben, Ziffern und
   Bindestriche.
2. `[profile]`, `[sources]` und die Prüfregeln ausfüllen, wie unten beschrieben.
3. Die Datei in `EMBEDDED` in `src/rules.rs` eintragen.
4. `cargo test -p freemapper-courseplan rules` ausführen. Die Tests lesen jede eingebettete Datei
   und lehnen unbekannte Schlüssel, Werte ohne Einheit, Regeln ohne Quelle und Tippfehler in
   Feldnamen ab.

## Aufbau der Datei

```toml
format = 1

[profile]
id = "wkb-2026"            # der Dateiname ohne .toml
kind = "base"              # "base" (Basisprofil) oder "overlay" (Aufsatz)
# base = "wkb-2026"        # nur bei Aufsätzen: das Basisprofil, das sie ergänzen
name = { de = "Wettkampfbestimmungen OL 2026", en = "German competition rules 2026" }
valid_from = 2026-03-01    # optional

[sources.WKB26]
title = "Wettkampfbestimmungen Orientierungslauf, 37. Änderung"
url = "https://o-sport.de/assets/dokumente/allgemein/wettkampfwesen/wkb2026.pdf"   # optional

[[rules]]
id = "wkb2026.distance.min_half"
key = "min_distance"
severity = "error"
enabled = true                       # optional, Vorgabe true
map_kinds = []                       # optionaler Filter
disciplines = []                     # optionaler Filter
print_scales = [1000, 2000, 3000, 4000]   # optionaler Filter
values = { metres = 15.0 }
source = { doc = "WKB26", section = "B 7.2.1" }
note = { de = "...", en = "..." }    # optional
```

Eine Datei ohne `format = 1` oder mit einem höheren Format wird abgelehnt, ebenso jeder Feldname,
den das Format nicht kennt. Ein vertipptes `map_kind` ist also ein Fehler und bedeutet nicht still
"alle Kartenarten".

## Felder einer Prüfregel

`id` ist stabil, mit Punkten gegliedert und kleingeschrieben: `<profil>.<familie>.<name>`, die
Profilkennung ohne Bindestriche (`wkb2026.code.pairs`). Akzeptierte Befunde verweisen auf die
`id`. Wer eine Regel umbenennt, bekommt die für sie akzeptierten Befunde wieder gemeldet.

`key` (Prüfregel-Schlüssel) nennt die Prüfung, die die Regel ausführt. Das Programm kennt diese
Schlüssel:

| Schlüssel | Was geprüft wird |
|---|---|
| `min_distance` | Zwei Posten der Veranstaltung näher als `metres` |
| `similar_features` | Zwei Posten auf ähnlichen Objekten näher als `metres` |
| `map_edge` | Posten, Start oder Ziel näher als `metres` am Kartenrand |
| `leg_through_control` | Eine Strecke führt durch einen anderen Posten derselben Bahn |
| `leg_through_forbidden` | Die optimale Laufroute führt durch ein Sperrgebiet; ungeprüft, bis es die Routenanalyse gibt, die Regel steht deshalb unter den nicht geprüften Regeln |
| `control_in_forbidden` | Ein Posten einer Bahn steht in einem Sperrgebiet-Sonderobjekt dieser Bahn oder in einer Kartenfläche mit einem Symbol aus `area_symbols`; auf dem Rand gilt als drin |
| `acute_angle` | Winkel zwischen den Strecken an einem Posten unter `degrees` |
| `straight_angle` | Winkel zwischen den Strecken an einem Posten über `degrees`, fast geradeaus |
| `number_gap` | Abstand der Nummer zum Kreis unter `min_gap_mm` oder über `max_gap_mm`; jeder Wert darf fehlen |
| `opposing_legs` | Zwei Strecken einer Bahn laufen dicht nebeneinander gegeneinander |
| `code_range` | Ein Postencode außerhalb von `range` |
| `code_pairs` | Beide Codes eines verbotenen Paars in einer Veranstaltung |
| `code_confusable_next` | Aufeinanderfolgende Codes, die auf dem Kopf gleich aussehen |
| `same_objects_in_circle` | Gleiche Objekte im Postenkreis ohne Angabe in Spalte C |
| `start_capacity` | Zu wenige verschiedene erste Posten für die Starter pro Minute |
| `first_leg_angle` | Erste Strecken eines Starts in fast gleicher Richtung |
| `length_table` | Bahnlänge in Prozent der HE-Bahn |
| `difficulty_mix` | Leichte und schwere Kategorien auf einer Bahn |
| `nor_lkm` | Leistungskilometer gegen die NOR-Empfehlung |
| `score_value_matches_code` | Wert und Code der Score-Posten stimmen überein |
| `score_code_ambiguity` | Verwechselbare Codes in einer Score-Bahn |
| `score_ring` | Posten im Ring um Start und Ziel |
| `score_obvious_order` | Offensichtliche Reihenfolge der Score-Posten |
| `score_length_bands` | Länge der Score-Bahn für gute und sehr gute Läufer |
| `score_limit_and_penalty` | Zeitlimit und Strafe der Score-Bahn |
| `score_link_valid` | Pflichtverbindungen der Score-Bahn |
| `score_mix` | Anteil schwerer Score-Posten |
| `score_min_gap` | Zwei Posten einer Score-Bahn näher als `min_gap_m` der Score-Einstellungen |
| `score_high_gap` | Zwei hochwertige Posten einer Score-Bahn näher als `high_value_gap_m` |

Die Schwellen der Score-Regeln (Ring, Reihenfolge, Mischung) sind Vorschläge dieses Programms, keine Werte einer Quelle.

Eine Regel mit einem anderen Schlüssel wird abgelehnt. Ein neuer Schlüssel braucht Code in
`src/check/` und einen Eintrag in `SCHEMA` in `src/rules.rs`.

`severity` ist `error`, `warning` oder `hint`.

`note` erscheint bei jedem Befund der Regel. Bitte deutsch und englisch schreiben; der englische
Text ist Pflicht.

## Filter

`map_kinds` (`sprint`, `schoolyard`, `forest`), `disciplines` (`sprint`, `middle`, `long`,
`ultralong`, `training`) und `print_scales` (Maßstabszahlen wie `4000`) schränken ein, wo eine
Regel gilt. Ein leerer oder fehlender Filter bedeutet: überall. Der Druckmaßstab einer Bahn ist ihr
`planned_print_scale`, sonst der Kartenmaßstab.

Hat eine Regel für verschiedene Karten verschiedene Werte, schreibt man eine Regel je Fall, jede
mit eigener `id` und eigenem Filter. `wkb-2026.toml` macht das beim Mindestabstand: 30 m bei
1:5000 bis 1:15000, 15 m bei 1:1000 bis 1:4000, weil WKB B 7.2.1 die Abstände bei 1:3000 und
1:4000 halbiert. Dass auch 1:1000 und 1:2000 (Schulhofkarten) halbiert zählen, ist ein Vorschlag
aus Plan M2b. Eine Bahn in einem Maßstab, den keine Regel nennt, prüfen diese Regeln nicht. Die
Regeln für ähnliche Objekte filtern zusätzlich nach Kartenart, weil Waldkarten ISOM-Symbole und
Sprintkarten ISSprOM-Symbole haben.

## Werte und Einheiten

`values` enthält, was die Prüfung liest. Eine Zahl trägt ihre Einheit im Namen: Der Name ist eine
Einheit oder endet auf `_<einheit>`. Die Einheiten sind `metres`, `mm`, `degrees`, `percent`, `factor` und
`minutes`. Beispiele: `metres = 30.0`, `min_length_metres = 30.0`, `tolerance_percent = 6`,
`feature_search_mm = 1.0`. Ein Name wie `distance = 30` wird abgelehnt, weil niemand sieht, ob
Meter im Gelände oder Millimeter auf der Karte gemeint sind. Eine Zahl unter einem Einheitennamen
muss im Bereich ihrer Einheit liegen: `metres` und `mm` über 0, `factor` mindestens 0, `percent` von 0
bis 100. Ein Text oder Wahrheitswert unter einem Einheitennamen wird abgelehnt.

Texte, true/false und Listen von Texten brauchen keine Einheit. Symbolnummern sind Texte:
`impassable = ["520", "709"]`. Ein Kartensymbol passt zu einer Nummer, wenn es dieselbe Nummer hat
oder eine Unternummer davon: `505.1` passt zu `505`, `5051` nicht.

Tabellen und Zeilenlisten haben keine einzelne Einheit. Jede steht mit ihrer Form in
`SCHEMA` in `src/rules.rs`, und die Profildatei beschreibt ihre Tabellen in einem Kommentar
am Anfang. Zurzeit sind das `range` und `pairs` (Codes), `courses` von `length_table` und von
`nor_lkm` sowie `levels`, `easy_levels` und `hard_levels` von `difficulty_mix`.

WKB-Kategorien in Tabellen stehen ohne Leerzeichen. Der nachgestellte `-` einer Altersklasse
entfällt, und eine `D/H`-Zelle wird zu einer D- und einer H-Kategorie: aus `H 35-` wird `H35`,
aus `D 19- A` wird `D19A`, aus `D/H -12b` werden `D-12b` und `H-12b`. Die Zeilen von
`length_table` behalten die Zelle außerdem so, wie die Quelle sie druckt, in `printed`. So lässt
sich jede Zeile mit dem PDF vergleichen.

## Quellen

Jede Prüfregel nennt ein Dokument: `source = { doc, section }`. `doc` ist ein Schlüssel aus der
eigenen `[sources]`-Tabelle der Datei; die Tests lehnen eine Regel ab, deren Dokument dort fehlt.
Als Schlüssel dienen die Kürzel aus `docs/research/2026-11-08-bahnlegung-kriterien.md` (WKB26,
IOF-R, IOF-SG, BOF, SOA, BL02, BL12, TD11, SOFT, BB-Lkm21, BB-Kombi14). Dazu kommt der Abschnitt,
bei Tabellen auch die PDF-Seite.

Für manche Werte nennt keine Quelle eine Zahl. Sie verweisen auf `doc = "claude"` mit dem Plan als
Abschnitt, und ihre `note` sagt, was die Quellen sagen und dass die Zahl ein Vorschlag ist. Der Lader lehnt eine solche Regel ohne `note` ab. `check`
nennt eine solche Quelle „Vorschlag von freemapper-courseplan“, ohne den Abschnitt aus dem Plan, und
`check --all` zeigt die Anmerkung unter dem Befund.

## Aufsätze

Ein Aufsatz nennt sein Basisprofil in `[profile] base`. Eine Veranstaltung kann nur Aufsätze für
ihr eigenes Basisprofil nutzen. Jede Regel eines Aufsatzes bewirkt eines von drei Dingen:

- Eine Regel mit neuer `id` kommt nach den Regeln des Basisprofils dazu.
- Eine Regel mit der `id` einer Basisregel ersetzt diese. Der Befund nennt dann den Aufsatz und
  dessen Quelle.
- Eine Regel mit der `id` einer Basisregel und `enabled = false` schaltet die Basisregel ab. Dazu
  die ganze Basisregel kopieren, `enabled = false` setzen und das Dokument nennen, das die
  Ausnahme erlaubt. `check --all` listet abgeschaltete Regeln. Eine `id` abzuschalten, die das
  Basisprofil nicht hat, ist ein Fehler.

Regeln eines Aufsatzes verweisen auf die `[sources]` des Aufsatzes.

## Werte, die am PDF zu prüfen sind

Die Tabellen stammen aus der Ausgabe von `pdftotext -layout` und wurden mit den gerenderten
PDF-Seiten verglichen. Diese Zellen sollte jemand mit den gedruckten Regeln noch einmal ansehen:

- WKB 2026 Anlage 5 Nr. 1 (PDF S. 49), Schwierigkeit je Kategorie. Die Textausgabe setzt die
  Zahlen verbundener Zellen in falsche Zeilen. Von der gerenderten Seite gelesen: D/H -16, -18 und
  -20 haben Schwierigkeit 4, DE/HE 5, D/H 35- bis 45- 4, D/H 50- bis 75- "4*". Die Recherche
  (Abschnitt 3.4) gibt D/H -20 mit 5 an; die Seite zeigt 4.
- WKB 2026 Anlage 5 Nr. 3 (PDF S. 50), Lang Zeile 10, druckt "H 55" ohne den Bindestrich der
  anderen Altersklassen. Das Profil liest H55.
- WKB 2026 Anlage 5 Nr. 4 und 5 (PDF S. 51) drucken "H 45-," (Mittel Zeile 4) und "D -12, H -12,"
  (Ultralang Zeile 17) mit einem Komma am Ende. Das Profil liest in diesen Zellen keine weitere
  Kategorie.
- WKB 2026 Anlage 5 Nr. 1, Fußnote **: Seniorenkategorien gehen über 75 hinaus in
  Fünfjahresschritten weiter. Das Profil listet Kategorien nur bis D/H 75.
- BB-Lkm21: Bahn D (D/H-14, 2,6 Lkm) ist in allen drei Tabellen kürzer als Bahn C (D/H-12,
  3,9 Lkm), und Mittel A bis D gleichen Lang A bis D. Vermutlich ein Übertragungsfehler in der
  Quelle; vor der Nutzung dieser Zeilen beim LFA nachfragen.

## Symbolnummern, die zu prüfen sind

Die Gruppen ähnlicher Objekte und die Sperrgebiet-Symbole nutzen auf Sprint- und Schulhofkarten
ISSprOM-2019-2-Nummern und auf Waldkarten ISOM-2017-2-Nummern. Die ISSprOM-Nummern sind mit dem
Symbolsatz von `tests/fixtures/maps/luebeck.omap` und dem ISSprOM-Text abgeglichen. Die
ISOM-Nummern stammen aus dem ISOM-2017-2-Text; eine Testkarte dafür gibt es noch nicht:

- 304 crossable watercourse, 305 small crossable watercourse, 306 minor/seasonal water channel,
  309 narrow marsh
- 111 small depression, 112 pit
- 109 small knoll, 110 small elongated knoll
- 516 fence, 517 ruined fence
- 505 footpath, 506 small footpath, 507 less distinct small footpath, 508 narrow ride or linear
  trace
- 520 area that shall not be entered, 708 out-of-bounds boundary, 709 out-of-bounds area;
  auf Sprintkarten auch 714 temporary construction or closed area

Zwei Posten am gleichen Punktsymbol sind immer ähnlich. Linien und Flächen mit gleichem Symbol
sind nur ähnlich, wenn eine Gruppe sie nennt, denn Höhenlinien, Vegetationsgrenzen und
Gebäudeumrisse gibt es überall. Sporn und Rinne (BOF B6.1.4) entstehen aus Höhenlinien und haben
kein eigenes Symbol; keine Gruppe enthält sie.

`leg_through_forbidden` hält diese Sperrgebiet-Symbole für die Routenanalyse bereit. Seit dem 12.11.2026
prüft das Programm die gerade Linie zwischen zwei Posten nicht mehr dagegen (ADR 0080): WKB26 B 7.2.6 betrifft die
Laufroute, und eine Luftlinie über ein Sperrgebiet ist normal. Bis es die Routenanalyse gibt, führt `check --all`
die Regel unter den nicht geprüften Regeln auf. Unpassierbare Linien und Objekte (Felsen, Wasser, Mauern, Zäune,
Gebäude, Brücken) und verbotene Routen warten ebenfalls auf die Routenanalyse.

`control_in_forbidden` liest seine Kartensymbole aus `area_symbols`: 520 und 709 auf Waldkarten, 520, 709 und 714 auf
Sprint- und Schulhofkarten. Die Grenzlinie 708 ist eine Linie und steht nicht in der Liste (ADR 0081).
