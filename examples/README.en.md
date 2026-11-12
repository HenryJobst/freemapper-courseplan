# Example events

[Deutsch](README.md)

Finished events on maps from [freemapper's examples](https://github.com/HenryJobst/freemapper/tree/main/examples).
Each one comes with its project file, the commands that made it and a preview image. The Lübeck example also has a short score course (15 minutes; the map is too small for more) in a second project file. The Berlin example has a 45-minute score course. Both come with the output of `check`. The IOF XML 3.0
export is included as `course-data.xml`; it is checked against the IOF schema and read back by the IOF reader of freemapper-course, and it carries latitude and longitude. An import into OE has not been tested. The Purple Pen export is included as `luebeck-sprint.ppen`; a test reads it back with the reader of freemapper-course. Purple Pen 4.0 opens it without an error. The print PDF from freemapper-print follows later.

| Example | Map | Preview |
|---|---|---|
| [Sprint training, Lübeck old town](luebeck-sprint/) | Lübeck, 1:4000 | <a href="luebeck-sprint/"><img src="luebeck-sprint/preview.jpg" width="160" alt=""></a> |
| [Score course 45 minutes, Berlin-Kreuzberg](berlin-kreuzberg/) | Berlin, 1:4000, made with freemapper from open data | no preview |
