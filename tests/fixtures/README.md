# Test fixtures

Every fixture file is listed here with its source, date and licence. Files made by hand for a test say so.

| File | Source | Date | Licence |
|---|---|---|---|
| `maps/luebeck.omap` | freemapper `examples/schleswig-holstein-luebeck/schleswig-holstein-luebeck.omap`, commit `743ce4b`; copy chain freemapper `743ce4b` → freemapper-print `8b0a0a6` → here. Data sources are named in the map's `<notes>` element. | 2026-11-08 | GPL-3.0-or-later (freemapper) |
| `maps/forest-sample.omap` | OpenOrienteering Mapper, https://github.com/OpenOrienteering/mapper, tag `v0.9.6`, `examples/forest sample.omap`; copied from freemapper-print `8b0a0a6`. | 2026-11-08 | GPL-3 (OpenOrienteering Mapper) |
| `iof-xml/IOF.xsd` | International Orienteering Federation, IOF XML 3.0 schema, https://github.com/international-orienteering-federation/datastandard-v3 (`IOF.xsd`, commit `24eb108e4c6b5e2904e5f8f0e49142e45e2c5230` of `master`, fetched 2026-11-09). | 2026-11-09 | The repository states no licence (GitHub API reports none), so the file is not redistributed beyond this fixture. Used here only to validate our output in tests. |
| `ppen/lincoln-park.ppen` | Purple Pen, https://github.com/petergolde/PurplePen, commit `201417f97bb69302e3e0db123f428870e0914515` (2026-11-07), `src/TestFiles/controller/Lincoln Park.ppen`; copied unchanged from freemapper-core tag `v0.3.0` (commit `d1e826a`), `crates/freemapper-course/tests/fixtures/courses/ppen/`. SHA-256 `686e07e774a94a650d5169a268661201ec3dff3186d1dc2c79c2247743c324e8`. | 2026-11-09 | BSD, Purple Pen (below) |
| `ppen/overlapper.ppen` | As above, `src/TestFiles/courserenderer/Overlapper.ppen`. SHA-256 `9f8cd5455c3957f492e2169eef5085933e00bb2de60ff1c9cff77740c4162f78`. | 2026-11-09 | BSD, Purple Pen (below) |
| `ppen/sampleevent8.ppen` | As above, `src/TestFiles/descriptions/sampleevent8.ppen`. SHA-256 `d8d4872f040f56dc9ec2c8728e70e530d63331ee5cb19cb399e2abbec288f1fb`. | 2026-11-09 | BSD, Purple Pen (below) |
| `crates/freemapper-courseplan/tests/fixtures/placement/cases.toml` | freemapper-core worktree `placement-symbols`, commit `bca5cc4`, `crates/freemapper-course/tests/fixtures/placement/cases.toml`; copied byte for byte. Shared number placement cases (format in its header). | 2026-11-10 | GPL-3.0-or-later (freemapper-core) |
| `crates/freemapper-courseplan/tests/fixtures/placement/luebeck-sprint-auto.toml` | Made here: `auto_place(keep_existing = false)` of every course of `examples/luebeck-sprint`, written by `PLACEMENT_CASES_DUMP=1 cargo test -p freemapper-courseplan --test placement_cases`. | 2026-11-10 | GPL-3.0-or-later |

The `.ppen` files keep their CRLF line ends and byte order mark (`.gitattributes`: `*.ppen -text`), so their
hashes stay those of the originals. Their maps are not in this repository: Lincoln Park is a 1:5000 bitmap,
Overlapper and sampleevent8 have no map. The import test (`crates/freemapper-courseplan/tests/ppen_import.rs`)
fits each against a frame map at the file's scale; none of them belongs to `maps/luebeck.omap`.

The files carry no licence header of their own. Purple Pen as a whole is published under the BSD licence below:
every source file in its repository carries it (for example `src/PurplePenCore/EventDB.cs`, "Copyright (c)
2006-2008, Peter Golde"), and the user documentation states it as the Purple Pen licence
(`doc/userdocs/WebSite/license.htm`, "Copyright © 2006-2007, Peter Golde").

```
Copyright (c) 2006-2008, Peter Golde
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

1. Redistributions of source code must retain the above copyright
notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright
notice, this list of conditions and the following disclaimer in the
documentation and/or other materials provided with the distribution.

3. Neither the name of Peter Golde, nor "Purple Pen", nor the names
of its contributors may be used to endorse or promote products
derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND
CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING,
BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY
OF SUCH DAMAGE.
```
