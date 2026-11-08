# Program texts of freemapper-courseplan, English (reference language, ADR 0022).
# Variable names are the same in every language. Values from other programs (parser
# messages of the toml crate, operating system errors) arrive in $source and $message
# and stay in English.

## Words used inside other messages

word-kind-individual = individual
word-kind-team = team
word-kind-relay = relay
word-kind-score = score
word-what-control = control
word-what-course = course
word-what-special-object = special object
word-what-class = class
word-what-control-or-special-object = control or special object
word-map-sprint = sprint
word-map-schoolyard = schoolyard
word-map-forest = forest

## Inconsistencies inside an event (ModelError)

error-model-duplicate-control = control { $code } is defined more than once
error-model-duplicate-course = course { $name } is defined more than once
error-model-duplicate-class = class { $name } is defined more than once
error-model-duplicate-special-id = special object id { $id } is used more than once
error-model-unknown-course-in-special = special object { $id } belongs to course { $course }, which is not defined
error-model-unknown-control = course { $course } uses control { $code }, which is not defined
error-model-unknown-start = course { $course } has start { $id }, which is not defined
error-model-unknown-finish = course { $course } has finish { $id }, which is not defined
error-model-team-too-small = course { $course } has team size { $team_size }, at least 2 is needed
error-model-free-control-is-mandatory = course { $course } lists control { $code } as free and as mandatory
error-model-date-not-a-date = event date { $date } is not a date; write it as YYYY-MM-DD, without time or offset
error-model-unsupported-format = project file format { $format } is newer than this program supports
error-model-unknown-key-top = unknown key { $key } at the top level
error-model-unknown-key-in = unknown key { $key } in { $table }

## Project files (ProjectError)

error-project-read = cannot read { $path }: { $source }
error-project-write = cannot write { $path }: { $source }
error-project-already-exists = { $path } already exists; choose another project file or remove it
error-project-parse = cannot parse { $path }: { $message }
error-project-serialise = cannot serialise { $path }: { $message }
error-project-invalid = { $path } is not a consistent event:{ $list }

## Starting an event (InitError)

error-init-map = cannot read map { $path }: { $source }
error-init-forest = Forest maps (ISOM) are not supported yet; this version handles sprint and schoolyard maps (ADR 0010).
error-init-io = cannot create folder { $path }: { $source }

## Measuring courses (CourseError)

error-course-not-supported = course { $course } is a { $kind } course; only individual courses are supported yet
error-course-unknown-course = course { $course } does not exist
error-course-unknown-reference = course { $course } refers to { $id }, which the event does not define

## Editing (EditError)

error-edit-invalid = the change would make the event inconsistent: { $errors }
error-edit-not-found = there is no { $what } { $id }
error-edit-code-taken = control { $code } already exists
error-edit-id-taken = special object { $id } already exists
error-edit-outside-map = point ({ $x }, { $y }) is outside the map, which spans ({ $min_x }, { $min_y }) to ({ $max_x }, { $max_y })
error-edit-in-use = { $id } is still used by { $courses }; change those first
error-edit-not-supported = course { $course } is not an individual course; only individual courses can be edited yet
error-edit-bad-index = course { $course } has no control at index { $index }

## Reading OOM maps (OmapError)

error-omap-xml = invalid OOM XML: { $message }
error-omap-missing-element = element <{ $name }> not found
error-omap-io = { $message }
error-omap-binary-format = This file is in the old binary format of OpenOrienteering Mapper, which this program cannot read. Open it in Mapper and save it again as a current .omap or .xmap file.
error-omap-unsupported = { $message }

## Command line

cli-error-prefix = error
cli-error-unknown-lang = unknown language { $value }; available: { $available }

cli-info-title = title: { $value }
cli-info-map = map: { $value }
cli-info-map-kind = map kind: { $value }
cli-info-controls = controls: { $count }
cli-info-courses = courses: { $total } (individual { $individual }, team { $team }, relay { $relay }, score { $score })
cli-info-classes = classes: { $count }

cli-about = Set orienteering courses on an OOM map
cli-help-init = Start an event from a map and write its project file
cli-help-info = Print the title, map, controls, courses and classes of a project file
cli-arg-init-map = The OOM map (sprint or schoolyard)
cli-arg-init-project = Where to write the project file [default: <map folder>/<map name>.courseplan.toml]
cli-arg-init-title = Event title [default: `title` in freemapper.toml next to the map, else the map name]
cli-arg-info-project = The project file
cli-arg-lang = Language of messages and help, for example de or en [default: FREEMAPPER_LANG, else the system locale, else en]
cli-arg-help = Print help
cli-arg-version = Print version

cli-heading-usage = Usage:
cli-heading-commands = Commands:
cli-heading-options = Options:
cli-heading-arguments = Arguments:
cli-help-help = Show the help of a command
cli-arg-help-command = The command to show the help of
cli-parse-hint = For more information, try '--help'.
cli-parse-unknown-argument = unexpected argument { $arg }
cli-parse-missing-required = missing required argument: { $args }
cli-parse-invalid-value = invalid value { $value } for { $arg }
cli-parse-invalid-subcommand = unknown command { $value }
cli-parse-missing-subcommand = a command is required
cli-parse-too-many-values = unexpected value { $value } for { $arg }
cli-parse-argument-conflict = { $arg } cannot be used together with { $other }

## Edit commands

cli-edit-control-added = added control { $code } at { $x },{ $y }
cli-edit-control-moved = moved control { $code } to { $x },{ $y }
cli-edit-control-removed = removed control { $code }
cli-edit-control-removed-from = removed control { $code } and its visits in { $courses }
cli-edit-control-described = description of control { $code }: { $columns }
cli-edit-number-set = control { $code } number: direction { $direction } degrees, gap { $gap } mm
cli-edit-number-cleared = control { $code } number placement cleared
cli-edit-special-added-at = added { $kind } { $id } at { $x },{ $y }
cli-edit-special-added-points = added { $kind } { $id } with { $count } points
cli-edit-special-removed = removed special object { $id }
cli-edit-class-added = added class { $name }
cli-edit-class-removed = removed class { $name }
cli-edit-bad-point = { $value } is not a point; write X,Y in map millimetres with a dot as decimal separator, for example 12.5,-30
cli-edit-bad-number = { $value } is not a number; use a dot as decimal separator, for example 0.5
cli-edit-bad-count = { $value } is not a whole number
cli-edit-kind-needs-at = a { $kind } is a single point; give --at X,Y and no --point
cli-edit-kind-needs-points = a { $kind } is drawn through points; give --point X,Y (once per point) and no --at
cli-edit-no-courses-for-kind = a { $kind } applies to every course; leave out --courses

word-empty = none
word-special-start = start
word-special-finish = finish
word-special-marked-route = marked route
word-special-out-of-bounds = out-of-bounds area
word-special-forbidden-route = forbidden route
word-special-dangerous-area = dangerous area
word-special-first-aid = first aid post
word-special-water = water point

cli-help-control = Add, move, describe, number and remove controls
cli-help-special = Add and remove special objects (start, finish, marked routes, areas, first aid, water)
cli-help-class = Add and remove classes
cli-help-control-add = Add a control
cli-help-control-move = Move a control
cli-help-control-remove = Remove a control and its visits in individual courses
cli-help-control-describe = Set the description symbols of a control
cli-help-control-number = Set or clear where the number of a control is drawn
cli-help-special-add = Add a special object
cli-help-special-remove = Remove a special object
cli-help-class-add = Add a class
cli-help-class-remove = Remove a class and its name from every course

cli-arg-edit-code = Code of the control
cli-arg-control-add-at = Where the control goes: X,Y in map millimetres
cli-arg-control-add-code = Code of the new control [default: the lowest free number from 31]
cli-arg-control-add-crossing-point = Add a crossing point, which gets no number
cli-arg-control-move-at = The new place: X,Y in map millimetres
cli-arg-control-describe-c = Column C symbol (IOF symbol id)
cli-arg-control-describe-d = Column D symbol (IOF symbol id)
cli-arg-control-describe-e = Column E symbol (IOF symbol id)
cli-arg-control-describe-f = Column F symbol (IOF symbol id)
cli-arg-control-describe-g = Column G symbol (IOF symbol id)
cli-arg-control-describe-h = Column H symbol (IOF symbol id)
cli-arg-control-describe-clear = Empty all columns first; columns given in the same call are set afterwards
cli-arg-control-number-direction = Degrees, 0 = east, counter-clockwise
cli-arg-control-number-gap = Gap to the circle in print millimetres
cli-arg-control-number-clear = Go back to the default placement
cli-arg-special-add-kind = What the object is
cli-arg-special-add-id = Id of the object, unique among special objects
cli-arg-special-add-at = Place of a single-point kind (start, finish, first aid, water): X,Y in map millimetres
cli-arg-special-add-point = A point of a line or area kind: X,Y in map millimetres; repeat the option for each point, in order
cli-arg-special-add-courses = Limit the object to these courses, separated by commas [default: all courses]
cli-arg-special-remove-id = Id of the special object
cli-arg-class-add-name = Name of the class
cli-arg-class-add-rule-category = Rule category of the class
cli-arg-class-add-starters = Expected number of starters
cli-arg-class-remove-name = Name of the class
