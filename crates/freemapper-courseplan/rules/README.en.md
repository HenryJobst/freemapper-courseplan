# Rule profiles

[Deutsch](README.md) | English

The `check` command tests courses against a rule profile. A profile is a TOML file in this folder.
The program embeds the files at build time, so a new or changed profile needs a new build. ADR 0025
fixes the format; `src/rules.rs` reads it.

There are two kinds of profile:

- A **base profile** holds a full rule set, for example `wkb-2026.toml` (German competition rules,
  WKB 2026).
- An **overlay** extends one base profile, for example `nor-bb-2021.toml` (course recommendations
  for Berlin and Brandenburg). It adds rules, replaces base rules or switches them off.

An event names one base profile in `rule_profile` and any number of overlays in `rule_overlays`.
The overlays apply in the order listed.

## Adding a profile

1. Copy the smaller of the two files and rename it. The file name without `.toml` is the profile
   id, such as `iof-2026` or `by-2025`. Use lower case, digits and dashes.
2. Fill in `[profile]`, `[sources]` and the rules as described below.
3. Add the file to `EMBEDDED` in `src/rules.rs`.
4. Run `cargo test -p freemapper-courseplan rules`. The tests parse every embedded file and refuse
   unknown keys, values without a unit, rules without a source and typos in field names.

## File layout

```toml
format = 1

[profile]
id = "wkb-2026"            # the file name without .toml
kind = "base"              # "base" or "overlay"
# base = "wkb-2026"        # overlays only: the base profile they extend
name = { de = "Wettkampfbestimmungen OL 2026", en = "German competition rules 2026" }
valid_from = 2026-03-01    # optional

[sources.WKB26]
title = "Wettkampfbestimmungen Orientierungslauf, 37. Änderung"
url = "https://o-sport.de/assets/dokumente/allgemein/wettkampfwesen/wkb2026.pdf"   # optional

[[rules]]
id = "wkb2026.distance.min_half"
key = "min_distance"
severity = "error"
enabled = true                       # optional, default true
map_kinds = []                       # optional filter
disciplines = []                     # optional filter
print_scales = [1000, 2000, 3000, 4000]   # optional filter
values = { metres = 15.0 }
source = { doc = "WKB26", section = "B 7.2.1" }
note = { de = "...", en = "..." }    # optional
```

A file without `format = 1`, or with a higher format, is refused. So is any field name the format
does not have. A misspelt `map_kind` is an error and does not silently mean "all map kinds".

## Rule fields

`id` is stable, dotted and lower case: `<profile>.<family>.<name>`, with the profile id written
without dashes (`wkb2026.code.pairs`). Accepted findings point to the id. If you rename a rule,
the findings that were accepted for it are reported again.

`key` names the check that runs the rule. The program knows these keys:

| Key | What it checks |
|---|---|
| `min_distance` | Two controls of the event closer than `metres` |
| `similar_features` | Two controls on similar features closer than `metres` |
| `map_edge` | A control, start or finish closer than `metres` to the map edge |
| `leg_through_control` | A leg passing through another control of the same course |
| `leg_through_forbidden` | A straight leg crossing an out-of-bounds area, so the best route may lead through it |
| `acute_angle` | The angle between the legs at a control below `degrees` |
| `opposing_legs` | Two legs of a course running close to each other in opposite directions |
| `code_range` | A control code outside `range` |
| `code_pairs` | Both codes of a forbidden pair used in one event |
| `code_confusable_next` | Consecutive codes that read alike upside down |
| `same_objects_in_circle` | Identical features in the control circle without column C |
| `start_capacity` | Too few different first controls for the starters per minute |
| `first_leg_angle` | First legs from one start in nearly the same direction |
| `length_table` | Course length in percent of the HE course |
| `difficulty_mix` | Easy and hard categories on one course |
| `nor_lkm` | Performance kilometres against the NOR recommendation |

A rule with any other key is refused. A new key needs code in `src/check/` and an entry in
`KNOWN_KEYS` in `src/rules.rs`.

`severity` is `error`, `warning` or `hint`.

`note` is shown with every finding of the rule. Write it in German and English; English is
required.

## Filters

`map_kinds` (`sprint`, `schoolyard`, `forest`), `disciplines` (`sprint`, `middle`, `long`,
`ultralong`, `training`) and `print_scales` (scale denominators such as `4000`) limit where a rule
applies. An empty or missing filter means all. A course's print scale is its
`planned_print_scale`, or the map scale if the course has none.

When a rule has different values for different maps, write one rule per case with its own id and
filter. `wkb-2026.toml` does this for the minimum distance: 30 m at 1:5000 to 1:15000, and 15 m at
1:1000 to 1:4000, because WKB B 7.2.1 halves the distances at 1:3000 and 1:4000. Counting 1:1000
and 1:2000 (schoolyard maps) as halved too is a proposal from plan M2b. A course printed
at a scale that no rule lists is not checked by these rules. The similar-feature rules also filter
by map kind, since forest maps use ISOM symbol codes and sprint maps ISSprOM codes.

## Values and units

`values` holds what the check reads. A number carries its unit in its name: the name is a unit or
ends in `_<unit>`. The units are `metres`, `mm`, `degrees`, `percent` and `factor`. Examples:
`metres = 30.0`, `min_length_metres = 30.0`, `tolerance_percent = 6`, `feature_search_mm = 1.0`.
A name like `distance = 30` is refused, because nobody can tell whether it means metres on the
ground or millimetres on the map.

Texts, true/false and lists of texts need no unit. Symbol codes are texts: `impassable = ["520",
"709"]`. A map symbol matches a code when it has that code or a sub-code of it: `505.1` matches
`505`, `5051` does not.

Tables and lists of rows have no single unit. Each one is listed with its shape in
`TABLE_VALUES` in `src/rules.rs`, and the profile file describes its tables in a comment at the
top. The current tables are `range` and `pairs` (codes), `courses` of `length_table` and of
`nor_lkm`, and `levels`, `easy_levels` and `hard_levels` of `difficulty_mix`.

Rule categories in tables are written without spaces. A trailing `-` of an age class is dropped,
and a `D/H` cell becomes one D and one H category: `H 35-` becomes `H35`, `D 19- A` becomes
`D19A`, `D/H -12b` becomes `D-12b` and `H-12b`. Rows of `length_table` also keep the cell as the
source prints it, in `printed`, so you can compare the row with the PDF.

## Sources

Every rule cites a document: `source = { doc, section }`. `doc` is a key of the file's own
`[sources]` table; the tests refuse a rule whose document is not listed there. Use the document
keys of `docs/research/2026-11-08-bahnlegung-kriterien.md` (WKB26, IOF-R, IOF-SG, BOF, SOA, BL02,
BL12, TD11, SOFT, BB-Lkm21, BB-Kombi14) and give the section, and the PDF page for tables.

Some values have no number in any source. They cite `doc = "claude"` with the plan as section,
and their `note` says what the sources say and that the number is a proposal.

## Overlays

An overlay names its base in `[profile] base`. An event can only use overlays for its own base
profile. Each rule of an overlay does one of three things:

- A rule with a new `id` is added after the base rules.
- A rule with the `id` of a base rule replaces it. The finding then names the overlay and the
  overlay's source.
- A rule with the `id` of a base rule and `enabled = false` switches the base rule off. Copy the
  whole base rule, set `enabled = false` and cite the document that allows it. `check --all`
  lists rules that are switched off. Switching off an id the base does not have is an error.

Overlay rules cite the overlay's own `[sources]`.

## Values to verify against the PDF

The tables were copied from `pdftotext -layout` output and compared with the rendered PDF pages.
These cells need a second look by someone with the printed rules:

- WKB 2026 Anlage 5 Nr. 1 (PDF p. 49), difficulty per category. The text output puts the numbers
  of merged cells on the wrong rows. Read from the rendered page: D/H -16, -18 and -20 have
  difficulty 4, DE/HE 5, D/H 35- to 45- 4, D/H 50- to 75- "4*". The research notes
  (section 3.4) say D/H -20 is 5; the page says 4.
- WKB 2026 Anlage 5 Nr. 3 (PDF p. 50), Lang row 10, prints "H 55" without the dash of the other
  age classes. The profile reads it as H55.
- WKB 2026 Anlage 5 Nr. 4 and 5 (PDF p. 51) print "H 45-," (Mittel row 4) and "D -12, H -12,"
  (Ultralang row 17) with a trailing comma. The profile reads no further category in these cells.
- WKB 2026 Anlage 5 Nr. 1 footnote **: senior categories continue in five-year steps beyond 75.
  The profile lists categories up to D/H 75 only.
- BB-Lkm21: course D (D/H-14, 2.6 Lkm) is shorter than course C (D/H-12, 3.9 Lkm) in all three
  tables, and Mittel A to D equal Lang A to D. Probably a transcription error in the source; ask
  the regional committee (LFA) before relying on these rows.

## Symbol codes to verify

The similar-feature groups and out-of-bounds symbols use ISSprOM 2019-2 codes on sprint and
schoolyard maps and ISOM 2017-2 codes on forest maps. The ISSprOM codes were checked against the
symbol set of `tests/fixtures/maps/luebeck.omap` and the ISSprOM text. The ISOM codes come from
the ISOM 2017-2 text and have no test map yet:

- 304 crossable watercourse, 305 small crossable watercourse, 306 minor/seasonal water channel,
  309 narrow marsh
- 111 small depression, 112 pit
- 109 small knoll, 110 small elongated knoll
- 516 fence, 517 ruined fence
- 505 footpath, 506 small footpath, 507 less distinct small footpath, 508 narrow ride or linear
  trace
- 520 area that shall not be entered, 708 out-of-bounds boundary, 709 out-of-bounds area;
  on sprint maps also 714 temporary construction or closed area

Two controls at the same point symbol are always similar. Lines and areas with the same symbol are
similar only when a group lists them, since contours, vegetation edges and building outlines are
everywhere. Spur and re-entrant (BOF B6.1.4) are drawn with contours and have no symbol of their
own, so no group contains them.

`leg_through_forbidden` looks only at these out-of-bounds symbols and at out-of-bounds areas of
the course. Uncrossable lines and objects (cliffs, water, walls, fences, buildings, bridges) and
forbidden routes are left to the later route analysis.
