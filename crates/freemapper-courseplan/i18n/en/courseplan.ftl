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
word-name-control-code = control code
word-name-special-object-id = special object id
word-name-course-name = course name
word-name-class-name = class name
word-name-accepted-finding-note = note of an accepted finding
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
error-model-blank-name = { $what } "{ $value }" is empty or starts or ends with a space
error-model-invalid-character = { $what } contains the control or invisible character { $point }
error-model-too-long = { $what } is { $len } characters long, at most { $limit } are allowed
error-model-blank-note = the note of accepted finding { $name } is empty
error-model-code-clashes-with-special = control code { $code } is also the id of a special object
error-model-duplicate-course-in-special = special object { $id } lists course { $course } more than once
error-model-too-few-points = { $kind } { $id } needs at least { $needed ->
    [one] { $needed } point
   *[other] { $needed } points
}
error-model-date-not-a-date = event date { $date } is not a date; write it as YYYY-MM-DD, without time or offset
error-model-unsupported-format = project file format { $format } is newer than this program supports
error-model-unknown-key-top = unknown key { $key } at the top level
error-model-unknown-key-in = unknown key { $key } in { $table }

## Project files (ProjectError)

error-project-read = cannot read { $path }: { $source }
error-project-write = cannot write { $path }: { $source }
error-project-changed = { $path } changed since it was read; nothing was written
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

## Rule profiles (RulesError)

error-rules-unknown-profile = unknown rule profile { $id }; available: { $list }
error-rules-not-a-base = rule profile { $id } is an overlay, not a base profile
error-rules-not-an-overlay = rule profile { $id } is a base profile, not an overlay
error-rules-overlay-for-another-base = overlay { $id } extends { $other }, not { $name }
error-rules-malformed = rule profile { $id } is malformed: { $message }
error-rules-unknown-key = rule { $name } in profile { $id } has the unknown key { $key }
error-rules-unknown-rule = overlay { $id } switches off rule { $name }, which the base profile does not have
error-rules-ambiguous = more than one active rule gives { $key } { $value }: { $list }

## Editing (EditError)

error-edit-invalid = the change would make the event inconsistent: { $errors }
error-edit-not-found = there is no { $what } { $id }
error-edit-code-taken = control { $code } already exists
error-edit-id-taken = special object { $id } already exists
error-edit-name-taken = course { $name } already exists
error-edit-outside-map = point ({ $x }, { $y }) is outside the map, which spans ({ $min_x }, { $min_y }) to ({ $max_x }, { $max_y })
error-edit-in-use = { $id } is still used by { $courses }; change those first
error-edit-not-supported = course { $course } is not an individual course; only individual and score courses can be edited yet
error-edit-bad-index = course { $course } has no control at position { $index }
error-edit-special-would-cover-all-courses = { $count ->
    [one] course { $course } is the only course of { $specials }; without it, { $specials } would apply to all courses. Assign { $specials } to another course first or remove it
   *[other] course { $course } is the only course of { $specials }; without it, they would apply to all courses. Assign them to another course first or remove them
}
error-edit-start-or-finish-has-no-courses = special object { $id } is a start or finish; each course names its start and finish itself
error-edit-unknown-rule = rule { $name } is not an active rule of rule profile { $id }
error-edit-finding-needs-place = findings of rule { $name } sit at { $places }; name that place
error-edit-finding-has-one-place = a finding of rule { $name } has one place: give a course, controls or legs of one course
error-edit-finding-wrong-place = findings of rule { $name } sit at { $places }, never at { $place }
error-edit-accepted-not-found = there is no accepted finding of rule { $name } at this place
error-edit-start-window-zero = the start window must be at least 1 minute
error-edit-too-many-bends = a leg of course { $course } may have at most { $limit } bends
error-edit-blank-rule-category = the rule category of class { $name } is empty; give a category or clear it
error-edit-bad-symbol = symbol { $id } cannot be used in column { $column } of a control description; pick a symbol that column offers

## Reading OOM maps (OmapError)

error-omap-xml = invalid OOM XML: { $message }
error-omap-missing-element = element <{ $name }> not found
error-omap-io = { $message }
error-omap-binary-format = This file is in the old binary format of OpenOrienteering Mapper, which this program cannot read. Open it in Mapper and save it again as a current .omap or .xmap file.
error-omap-unsupported = { $message }

## Checking (check)

check-severity-error = error
check-severity-warning = warning
check-severity-hint = hint
check-location-event = event
check-location-course = course { $course }
check-location-control = control { $code }
check-location-controls = controls { $codes }
check-location-start = start { $id }
check-location-finish = finish { $id }
check-location-leg = course { $course }, leg { $leg }
check-location-legs = course { $course }, legs { $legs }
check-leg-end-start = start
check-leg-end-finish = finish
check-skip-training = not checked at a training
check-skip-discipline = the rule is not for { $discipline ->
        [sprint] sprint
        [middle] middle distance
        [long] long distance
        [ultralong] ultralong distance
       *[other] { $discipline }
    } events
check-skip-map-kind = the rule is not for { $kind ->
        [sprint] sprint maps
        [schoolyard] schoolyard maps
        [forest] forest maps
       *[other] { $kind } maps
    }
check-note-non-standard-scale = 1:{ $scale } is not a standard scale for { $kind ->
        [sprint] sprint maps
        [schoolyard] schoolyard maps
        [forest] forest maps
       *[other] { $kind } maps
    }. The course symbols keep their 1:{ $base_scale } size. Use this course for training only.
check-skip-print-scale = no rule of this kind for print scale 1:{ $scale }
check-skip-course-kind = { $kind ->
        [team] team courses
        [relay] relay courses
       *[other] { $kind } courses
    } are not checked yet
check-skip-course-unresolved = the course refers to { $id }, which the event does not define; it is not checked
check-skip-class-not-mapped = class { $name } has no rule category; class set --rule-category assigns one
check-skip-climb-missing = the course has no climb
check-skip-needs-route-analysis = the rule is about the running route, and the route analysis is missing; a straight line between controls is not checked
check-skip-value-missing = the rule has no value { $name }
check-skip-no-map-extent = the map has no objects, so it has no edge
check-skip-score-no-map = the map has no objects, so no route times can be computed
check-skip-score-travel = route times cannot be computed. The error text is English: { $error }
check-skip-start-window-missing = the event has no start window; event start-window sets it
check-skip-starters-missing = classes without an expected number of starters: { $classes }; class set --starters sets it
check-skip-length-no-reference = no course carries category { $category }, so the length table has no reference
check-skip-length-reference-empty = the reference course { $course } has length 0
check-skip-category-not-in-table = categories in no row of the table: { $categories }
check-skip-switched-off = rule profile { $overlay } switches the rule off
check-distance-min = { $first_kind ->
        [start] start
        [finish] finish
        [crossing] crossing point
       *[control] control
    } { $first } and { $second_kind ->
        [start] start
        [finish] finish
        [crossing] crossing point
       *[control] control
    } { $second } are { $metres } m apart, at least { $limit } m required
check-distance-similar = controls { $first } and { $second } stand at similar features ({ $first_symbol }, { $second_symbol }) and are { $metres } m apart, at least { $limit } m required
check-map-edge = { $kind ->
        [start] start
        [finish] finish
        [crossing] crossing point
       *[control] control
    } { $id } is { $metres } m from the map edge, at least { $limit } m required
check-leg-through-control = { $metres ->
        [0] the leg runs over { $stop_kind ->
            [start] start
            [finish] finish
            [crossing] crossing point
           *[control] control
        } { $stop }
       *[other] the leg passes { $metres } m from { $stop_kind ->
            [start] start
            [finish] finish
            [crossing] crossing point
           *[control] control
        } { $stop }, inside its circle radius of { $limit } m
    }
check-acute-angle = the legs at { $stop_kind ->
        [crossing] crossing point
       *[control] control
    } { $stop } form an angle of { $degrees }°, at least { $limit }° required
check-straight-angle = the legs at control { $stop } form an angle of { $degrees }°, at most { $limit }° wanted
check-number-gap-small = the number of control { $stop } is { $gap } mm from the circle, at least { $limit } mm wanted
check-number-gap-large = the number of control { $stop } is { $gap } mm from the circle, at most { $limit } mm wanted
check-opposing-legs = the two legs run in opposite directions for { $metres } m at most { $limit } m apart, less than { $min_length } m allowed
check-code-range = code { $code } is outside the allowed range { $min } to { $max }
check-code-not-numeric = code { $code } is not a number
check-code-pair = codes { $first } and { $second } must not both be used in one event
check-code-confusable-rotated = codes { $first } and { $second } look alike when turned upside down and follow each other
check-code-confusable-six-nine = codes { $first } and { $second } differ only by 6 and 9 and follow each other
check-same-objects-in-circle = control { $code } stands at symbol { $symbol }, which occurs again within its circle, and column C of the description is empty
check-start-capacity = { $needed } different first controls are needed for the starters per minute, but only { $available } in use
check-start-capacity-shared = { $needed } different first controls are needed for the starters per minute, but only { $available } in use; first controls shared by several courses: { $shared }
check-first-leg-angle = the first legs from start { $start } to controls { $first } and { $second } differ in direction by only { $degrees }°, at least { $limit }° required
check-length-table = the course length is outside the table: { $rows }
check-length-table-row = { $categories }: { $percent } % of the { $reference } course, allowed { $min } to { $max } %
check-length-table-reference-twice = courses { $courses } all carry category { $category }; course { $first } serves as the reference for the length table
check-difficulty-mix = the course mixes easy categories ({ $easy }) with hard ones ({ $hard })
check-nor-lkm = the course has { $actual } Lkm ({ $length } km length, { $climb } m climb), outside the recommendation: { $rows }
check-nor-lkm-row = { $categories }: target { $target } Lkm, allowed { $min } to { $max } Lkm
check-score-value-mismatch = control { $code } carries the value { $expected } in its code, but score course { $course } gives it { $value } points
check-score-value-no-code = the code { $code } carries no value, because a score code has two or more digits and does not end in 0, but score course { $course } gives the control { $value } points
check-score-code-alike = codes { $first } and { $second } in score course { $course } look alike when turned upside down or differ only by 6 and 9
check-score-ring = the { $count } controls of score course { $course } stand in a ring round start and finish. The largest gap is { $gap } degrees and the distances vary by { $variation } percent, so the order is obvious
check-score-obvious-order = the controls of score course { $course } have an obvious order. { $hull } percent lie on the outer hull, and running to the nearest control each time is only { $excess } percent longer than the shortest order found
check-score-length-good-below = in score course { $course } the good runner scores { $percent } percent of all points, below the band of { $low } to { $high } percent. The course is too long
check-score-length-good-above = in score course { $course } the good runner scores { $percent } percent of all points, above the band of { $low } to { $high } percent. The course is too short
check-score-length-very-good-misses = in score course { $course } the very good runner cannot take every control within the limit. { $missing } points stay out of reach
check-score-length-very-good-over = in score course { $course } the very good runner needs { $percent } percent of the limit for every control
check-score-length-very-good-early = in score course { $course } the very good runner takes every control and finishes after { $percent } percent of the limit, before the band of { $low } to { $high } percent
check-score-limit-range = score course { $course } has a time limit of { $limit } minutes. { $min } to { $max } minutes is usual
check-score-penalty-pays = score course { $course } deducts { $penalty } points per minute over the limit, less than its highest control value of { $highest }. Running over can pay off
check-score-link-self = score course { $course } links control { $code } to itself
check-score-link-not-in-course = the link { $first }–{ $second } of score course { $course } names control { $missing }, which the course does not have
check-score-link-duplicate = the link { $first }–{ $second } appears more than once in score course { $course }
check-score-link-out-of-bounds = the straight line of the link { $first }–{ $second } in score course { $course } crosses the out-of-bounds area { $id }
check-score-link-forbidden-route = the straight line of the link { $first }–{ $second } in score course { $course } crosses the forbidden route { $id }
check-score-link-no-choice = the link { $first }–{ $second } in score course { $course } has no second route of similar length. Accept the finding with a note if that is intended
check-score-mix = in score course { $course } { $hard } of { $count } controls are hard, { $actual } percent. The setting asks for { $wanted } percent, with a tolerance of { $tolerance } points
check-score-min-gap = controls { $first } and { $second } of score course { $course } stand { $distance } m apart; the score settings ask for at least { $gap } m
check-score-high-gap = the high-valued controls { $first } and { $second } of score course { $course } stand { $distance } m apart; the score settings ask for at least { $gap } m
check-source-claude = proposal of freemapper-courseplan
check-source-user = decision of the project owner

## Command line

cli-error-prefix = error
cli-error-unknown-lang = unknown language { $value }; available: { $available }

cli-info-title = title: { $value }
cli-info-map = map: { $value }
cli-info-map-kind = map kind: { $value }
cli-info-controls = controls: { $count }, crossing points: { $crossings }
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
cli-arg-help-command = The command to show the help of, with its subcommands, for example control add
cli-parse-hint = For more information, try '--help'.
cli-parse-unknown-argument = unexpected argument { $arg }
cli-parse-missing-required = missing required argument: { $args }
cli-parse-invalid-value = invalid value { $value } for { $arg }
cli-parse-invalid-value-choices = invalid value { $value } for { $arg }; possible values: { $values }
cli-help-possible-values = possible values
cli-parse-invalid-subcommand = unknown command { $value }
cli-parse-missing-subcommand = a command is required
cli-parse-too-many-values = unexpected value { $value } for { $arg }
cli-parse-argument-conflict = { $arg } cannot be used together with { $other }

## Edit commands

cli-edit-control-added = added control { $code } at { $x },{ $y }
cli-edit-crossing-point-added = added crossing point { $code } at { $x },{ $y }
cli-edit-control-moved = moved control { $code } to { $x },{ $y }
cli-edit-control-code-changed = changed the code of control { $from } to { $to }
cli-edit-control-code-unchanged = the code of control { $from } stays { $to }; nothing changed
cli-edit-control-removed = removed control { $code }
cli-edit-control-removed-from = removed control { $code } and its visits in { $courses }
cli-edit-control-described = description of control { $code }: { $columns }
cli-edit-number-set = control { $code } number: direction { $direction } degrees, gap { $gap } mm
cli-edit-number-cleared = control { $code } number placement cleared
cli-edit-special-added-at = added { $kind } { $id } at { $x },{ $y }
cli-edit-special-added-points = { $count ->
    [one] added { $kind } { $id } with 1 point
   *[other] added { $kind } { $id } with { $count } points
}
cli-edit-special-removed = removed special object { $id }
cli-edit-special-courses = special object { $id } now belongs to { $courses }
cli-edit-special-all-courses = special object { $id } now applies to all courses
cli-edit-no-course-names = give at least one course name, or --all for all courses
cli-edit-class-added = added class { $name }
cli-edit-class-removed = removed class { $name }
cli-edit-class-set = changed class { $name }
cli-edit-bad-point = { $value } is not a point; write X,Y in map millimetres with a dot as decimal separator, for example 12.5,-30
cli-edit-bad-number = { $value } is not a number; use a dot as decimal separator, for example 0.5
cli-edit-bad-count = { $value } is not a whole number
cli-edit-bad-gap = { $value } is not a usable gap; give more than 0 and at most { $max } print millimetres
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
cli-help-special = Add, assign and remove special objects (start, finish, marked routes, areas, first aid, water)
cli-help-class = Add, change and remove classes
cli-help-control-add = Add a control
cli-help-control-move = Move a control
cli-help-control-code = Change the code of a control
cli-help-control-remove = Remove a control and its visits in individual courses
cli-help-control-describe = Set the description symbols of a control
cli-help-control-number = Set or clear where the number of a control is drawn
cli-help-special-add = Add a special object
cli-help-special-remove = Remove a special object
cli-help-special-courses = Set the courses a special object belongs to
cli-help-class-add = Add a class
cli-help-class-remove = Remove a class and its name from every course
cli-help-class-set = Change the rule category or the expected number of starters of a class. Give at least one option

cli-arg-edit-code = Code of the control
cli-arg-control-add-at = Where the control goes: X,Y in map millimetres
cli-arg-control-add-code = Code of the new control [default: the lowest free number from 31]
cli-arg-control-add-crossing-point = Add a crossing point, which gets no number
cli-arg-control-move-at = The new place: X,Y in map millimetres
cli-arg-control-code-from = The current code
cli-arg-control-code-to = The new code, which no other control has
cli-arg-course-rename-from = The current name
cli-arg-course-rename-to = The new name, which no other course has
cli-arg-control-describe-c = Column C symbol (IOF symbol id)
cli-arg-control-describe-d = Column D symbol (IOF symbol id)
cli-arg-control-describe-e = Column E symbol (IOF symbol id)
cli-arg-control-describe-f = Column F symbol (IOF symbol id)
cli-arg-control-describe-g = Column G symbol (IOF symbol id)
cli-arg-control-describe-h = Column H symbol (IOF symbol id)
cli-arg-control-describe-clear = Empty all columns first; columns given in the same call are set afterwards
cli-arg-control-number-direction = Degrees, 0 = east, counter-clockwise; any value is brought into 0 to 360
cli-arg-control-number-gap = Gap between circle line and number in print millimetres, more than 0 and at most 10
cli-arg-control-number-clear = Go back to the default placement
cli-arg-special-add-kind = What the object is
cli-arg-special-add-id = Id of the object, unique among special objects
cli-arg-special-add-at = Place of a single-point kind (start, finish, first aid, water): X,Y in map millimetres
cli-arg-special-add-point = A point of a line or area kind: X,Y in map millimetres; repeat the option for each point, in order
cli-arg-special-add-courses = Limit the object to these courses, separated by commas [default: all courses]
cli-arg-special-remove-id = Id of the special object
cli-arg-special-courses-id = Id of the special object; not a start or finish
cli-arg-special-courses-courses = The courses, separated by commas
cli-arg-special-courses-all = Let the object apply to all courses
cli-arg-class-add-name = Name of the class
cli-arg-class-add-rule-category = Rule category of the class
cli-arg-class-add-starters = Expected number of starters
cli-arg-class-remove-name = Name of the class
cli-arg-class-set-name = Name of the class
cli-arg-class-set-rule-category = The rule category, such as H21 or D-14; the table checks use it
cli-arg-class-set-no-rule-category = Remove the rule category
cli-arg-class-set-starters = Expected number of starters; the start capacity check uses it
cli-arg-class-set-no-starters = Remove the expected number of starters

## Course commands

cli-info-course = course { $name }: { $controls ->
    [one] 1 control
   *[other] { $controls } controls
}, { $km } km, climb { $climb } m
cli-info-course-score = score course { $name }: { $controls ->
    [one] 1 control
   *[other] { $controls } controls
}, { $points } points, time limit { $limit } minutes
cli-info-course-unmeasured = course { $name }: { $kind }, length not supported yet
word-decimal-point = .
word-taped = taped
word-not-taped = not taped
word-place-event = the event as a whole
word-place-course = a course
word-place-control = one control
word-place-controls = two or more controls
word-place-leg = a leg
word-place-legs = two or more legs of one course
word-or = or
word-and = and
cli-edit-course-added = added course { $name }
cli-edit-course-removed = removed course { $name }
cli-edit-course-renamed = renamed course { $from } to { $to }
cli-edit-course-unchanged = course { $from } keeps its name; nothing changed
cli-edit-course-controls = course { $name } now visits { $codes }
cli-edit-course-set = changed course { $name }
cli-edit-course-numbers = { $count ->
    [one] placed 1 control number on course { $name }
   *[other] placed { $count } control numbers on course { $name }
}
cli-edit-course-numbers-none = placed no control numbers on course { $name }, because every control has its number already; --overwrite places the course numbers again
cli-edit-course-leg-start = set the leg from the start of course { $name }: bends { $bends }, { $taped }
cli-edit-course-leg-after = set the leg after position { $index } of course { $name }: bends { $bends }, { $taped }
cli-edit-course-number-set = course { $name }, position { $index }: number at direction { $direction } degrees, gap { $gap } mm
cli-edit-course-number-cleared = course { $name }, position { $index }: number placement cleared
cli-edit-leg-cleared = course { $name }: cleared the bends and taped flag of the leg after position { $index }, which now runs to another stop
cli-edit-acceptance-dropped = course { $name }: dropped the accepted finding on the leg { $from } to { $to }, which no longer exists
cli-edit-start-leg-cleared = course { $name }: cleared the bends and taped flag of the leg from the start, which now runs to another stop

cli-help-course = Add, change, number and remove individual and score courses
cli-help-course-add = Add an individual or a score course
cli-help-course-controls = Replace the controls of a course
cli-help-course-remove = Remove a course and its name from the special objects
cli-help-course-rename = Rename a course
cli-help-course-leg = Set the bends and the marked-route flag of one leg
cli-help-course-set = Change classes, climb or planned print scale of a course
cli-help-course-number = Set or clear the number placement of one control in a course; it wins over the central and the automatic placement
cli-help-course-numbers = Place the control numbers of a course behind the circle as seen from the incoming leg, clear of legs, circles and point and line symbols of the map. Runs only when called, so call it again after changing legs; central and course numbers stay. The number moves outward up to the maximum gap of the rule profile before it turns away from the preferred direction

cli-arg-edit-course = Name of the course
cli-arg-course-add-start = Id of the start special object
cli-arg-course-add-finish = Id of the finish special object
cli-arg-course-add-controls = Codes of the controls in running order, separated by commas; a crossing point counts as a position [default: none]
cli-arg-course-add-classes = Classes that run the course, separated by commas
cli-arg-course-add-climb = Climb in metres
cli-arg-course-add-print-scale = Planned print scale, for example 3000 for 1:3000
cli-arg-course-controls-codes = Codes in running order, separated by commas. The n-th visit of a code keeps the number placement of the n-th visit before; its leg keeps bends and taped flag only if it still runs to the same next stop
cli-arg-course-leg-from-start = Edit the leg from the start to the first control
cli-arg-course-leg-after = Edit the leg after this position, counted from 1 as on the map; crossing points count as positions
cli-arg-course-leg-bend = A bend of the leg: X,Y in map millimetres; repeat the option for each bend, in order. Without any --bend the leg loses its bends
cli-arg-course-leg-taped = Mark the leg as a marked route. Without --taped the flag is cleared
cli-arg-course-set-classes = Classes that run the course, separated by commas
cli-arg-course-set-climb = Climb in metres
cli-arg-course-set-no-climb = Remove the climb
cli-arg-course-set-print-scale = Planned print scale, for example 3000 for 1:3000
cli-arg-course-set-no-print-scale = Remove the planned print scale
cli-arg-course-number-after = Position of the control in the course, counted from 1 as on the map; crossing points count as positions
cli-arg-course-number-direction = Degrees, 0 = east, counter-clockwise; any value is brought into 0 to 360
cli-arg-course-number-gap = Gap between circle line and number in print millimetres, more than 0 and at most 10
cli-arg-course-number-clear = Remove the placement of the course, so the central or the automatic placement applies again
cli-arg-course-numbers-overwrite = Also replace course numbers placed before; controls with a central number keep it [default: keep them]

## Check commands

cli-help-check = Check the individual courses against the rule profile and its overlays
cli-help-check-exit-codes =
    Exit code 0: no finding of severity error; accepted findings do not count.
    Exit code 1: at least one error finding, or the check was refused, for example because the project file or the map cannot be read.
    Exit code 2: wrong call, for example an unknown option.
cli-arg-check-all = Also list the accepted findings with their notes, the acceptances that match no finding, the skipped rules with the reason, and under each finding the note of its rule
cli-check-group-event = event:
cli-check-group-course = course { $course }:
cli-check-heading-accepted = accepted:
cli-check-heading-skipped = skipped:
cli-check-heading-unmatched = not matched:
cli-check-unmatched-hint = these acceptances match no finding; finding remove removes one that is no longer needed
cli-check-note = note: { $note }
cli-check-rule-note = about the rule: { $note }
cli-check-summary = { $errors ->
        [one] 1 error
       *[other] { $errors } errors
    }, { $warnings ->
        [one] 1 warning
       *[other] { $warnings } warnings
    }, { $hints ->
        [one] 1 hint
       *[other] { $hints } hints
    }; { $accepted } accepted, { $unmatched ->
        [0] {""}
       *[other] { $unmatched } not matched,{" "}
    }{ $skipped } skipped
cli-check-show-all = check --all also lists the accepted findings, the acceptances that match no finding and the skipped rules
cli-info-rule-profile = rule profile: { $value }
cli-info-rule-profile-overlays = rule profile: { $value }, { $count ->
        [one] overlay
       *[other] overlays
    }: { $list }

cli-help-finding = Accept findings as intended exceptions, or let check report them again
cli-help-finding-accept = Accept a finding as an intended exception, with a note. Name its place as check shows it: --controls, --leg or --course, or none of them for a finding of the whole event. Accepting the same finding again replaces the note
cli-help-finding-remove = Remove an accepted finding, so check reports it again. Name it as when it was accepted
cli-arg-finding-accept-rule = Id of the rule as check shows it, for example wkb2026.distance.min
cli-arg-finding-accept-controls = The controls of the finding, separated by commas
cli-arg-finding-accept-leg = The leg of the finding as COURSE:N, the leg that leaves position N of the course: positions count the controls of the course from 1, crossing points included, and 0 is the start. N is not the control number on the map; check names the leg by its ends, for example 36–37. Repeat it for a finding at two legs: --leg A:2 --leg A:5
cli-arg-finding-accept-course = The course of a finding about a whole course
cli-arg-finding-accept-note = Why the exception is intended
cli-arg-finding-remove-rule = Id of the rule as check shows it, for example wkb2026.distance.min
cli-arg-finding-remove-controls = The controls of the finding, separated by commas
cli-arg-finding-remove-leg = The leg of the finding as COURSE:N, the leg that leaves position N of the course: positions count the controls of the course from 1, crossing points included, and 0 is the start. N is not the control number on the map; check names the leg by its ends, for example 36–37. Repeat it for a finding at two legs: --leg A:2 --leg A:5. The exception for the leg as it runs now is removed first; without one, those at this position whose leg has changed since
cli-arg-finding-remove-course = The course of a finding about a whole course
cli-finding-bad-leg = { $value } is not a leg; write COURSE:N, for example A:3, where N is a position in the course and 0 is the start
cli-finding-at = at { $location }
cli-finding-at-event = at the event
cli-finding-accepted = accepted finding { $rule } { $location }
cli-finding-note-replaced = replaced the note of accepted finding { $rule } { $location }
cli-finding-removed = removed accepted finding { $rule } { $location }

cli-help-event = Settings of the whole event: rule profile and start window
cli-help-event-profile = Set the base rule profile and its overlays. The profiles are loaded first, and an unknown one is refused with the list of available profiles. Without --overlay the event has no overlays
cli-arg-event-profile-base = The base rule profile, for example wkb-2026
cli-arg-event-profile-overlay = An overlay on the base profile, for example nor-bb-2021; repeat it for more, they apply in this order
cli-help-event-start-window = Set the start window, the minutes from the first to the last start, or remove it with --none. The start capacity check divides the expected starters by it
cli-arg-event-start-window-minutes = Minutes from the first to the last start, at least 1
cli-event-profile-set = set rule profile { $value }
cli-event-start-window-set = set the start window to { $minutes ->
        [one] 1 minute
       *[other] { $minutes } minutes
    }
cli-event-profile-set-overlays = set rule profile { $value } with { $count ->
        [one] overlay
       *[other] overlays
    } { $list }

## Export commands

cli-help-export = Write the courses to a file for other programs
cli-arg-export-project = The project file
cli-arg-export-iof-xml = Write the individual and score courses as IOF XML 3.0 CourseData for result software such as OE; an existing file is replaced
export-iof-xml-written = { $count ->
    [one] wrote 1 course to { $path }
   *[other] wrote { $count } courses to { $path }
}
export-iof-xml-skipped = left out { $course }: { $kind } courses are not exported to IOF XML
export-iof-xml-no-geo = the map has no usable georeferencing, so the controls carry no latitude and longitude, only map positions
export-nothing-to-do = nothing to export; name a format, for example --iof-xml FILE, --ppen FILE or --csv FILE with --course NAME
error-export-iof-xml-character = cannot export: a text of the event holds the character { $point }, which XML does not allow
error-export-iof-xml-course = cannot export course { $course }: { $source }
error-export-overwrites-input = { $path } is the project file or its map, and the export would overwrite it; name another file
cli-arg-export-ppen = Write the individual courses into a Purple Pen event. The export changes an existing file in place and keeps everything the project did not change. It creates a missing file and, if the project has no sources.ppen yet, links the project to it. The project's Purple Pen ids count only for the linked file
cli-arg-export-new = With --ppen, write the file from scratch and replace the existing one; a project without sources.ppen is linked to it
export-ppen-written = { $count ->
    [one] wrote 1 course to the new Purple Pen event { $path }
   *[other] wrote { $count } courses to the new Purple Pen event { $path }
}
export-ppen-patched = { $count ->
    [one] wrote 1 course into the Purple Pen event { $path }
   *[other] wrote { $count } courses into the Purple Pen event { $path }
}
export-ppen-note-line = note: { $note }
export-ppen-ids-stored = { $count ->
    [one] stored a Purple Pen id for 1 object of the project that had none
   *[other] stored a Purple Pen id for { $count } objects of the project that had none
}
export-ppen-note-course-skipped = left out { $course }: { $kind } courses are not exported to Purple Pen
export-ppen-note-special-skipped = special object { $id } is left out: { $why }
export-ppen-note-symbol-without-id = control { $control }, column { $column }: symbol { $id } has no Purple Pen counterpart and is left out
export-ppen-note-symbol-not-in-standard = control { $control }, column { $column }: symbol { $id } is not in the file's { $standard } description standard and is left out
export-ppen-note-symbol-wrong-column = control { $control }, column { $column }: Purple Pen does not offer symbol { $id } in this column, so it is left out
export-ppen-note-symbol-variant-chosen = control { $control }, column { $column }: Purple Pen has several symbols for { $id }; the file gets { $written }
export-ppen-note-code-not-stored = the code "{ $code }" has blanks at an end, and reading the event again drops them, so the control comes back under another code
export-ppen-note-flagging-implied = course { $course }, leg { $from } to { $to }: Purple Pen flags this leg by itself, although the project does not mark it
export-ppen-note-partial-flagging-dropped = course { $course }, leg { $from } to { $to }: the bends changed and the flagging no longer starts or stops at a bend, so the partial flagging is dropped
export-ppen-kept-control = control { $code } (Purple Pen id { $id }) is in the file but not in the project; the export keeps it, because it never deletes
export-ppen-kept-course = course { $name } (Purple Pen id { $id }) is in the file but not in the project; the export keeps it, because it never deletes
export-ppen-kept-special = special object of kind { $kind } (Purple Pen id { $id }) is in the file but not in the project; the export keeps it, because it never deletes
error-export-ppen-base = the existing Purple Pen event { $path } cannot be patched: { $source }
error-export-ppen-unknown-version = the Purple Pen event comes from a version this program does not know ({ $version }); nothing was written
error-export-ppen-unsupported = the courses cannot be written to a Purple Pen event: { $what }
error-export-ppen-invalid = the courses give an invalid Purple Pen event: { $what }
error-export-ppen-shared-leg = courses { $first } and { $second } draw the leg { $from } to { $to } differently, and Purple Pen keeps one leg per pair of controls
error-export-ppen-shared-circle-gaps = courses { $first } and { $second } give control { $control } different circle gaps at 1:{ $scale }
error-export-ppen-limit = too large for a Purple Pen event: { $what }
error-export-ppen-circle-gap-limit = too large for a Purple Pen event: the courses would copy more than { $limit } circle gaps from the existing file
error-export-ppen-ids-not-stored = wrote { $path }, but the project could not be saved, so it lacks the Purple Pen ids of that file: { $source }
error-export-same-target = --iof-xml and --ppen name the same file { $path }; name two different files
export-ppen-linked = sources.ppen now names { $path }
word-export-why-marked-route = Purple Pen has no marked route object; the flagging belongs to the legs
word-export-why-forbidden-route-with-several-points = Purple Pen draws a forbidden route only as a single cross
word-export-why-no-course-left = it belongs only to courses that were left out

## Purple Pen import (init --ppen, import --ppen)

cli-help-import = Take over the controls and courses of a Purple Pen event (.ppen)
cli-arg-import-project = The project file
cli-arg-import-ppen = The Purple Pen event to import; it must be drawn for the project's map
cli-arg-import-replace = Replace the controls, courses and special objects the project already has; classes and accepted findings stay
cli-arg-init-ppen = Start from a Purple Pen event (.ppen) drawn for this map
import-ppen-done = { $controls ->
    [one] imported 1 control
   *[other] imported { $controls } controls
} and { $courses ->
    [one] 1 course
   *[other] { $courses } courses
} from { $path }
import-ppen-note-line = note: { $note }
import-ppen-needs-replace = { $path } already has controls, courses or special objects; run again with --replace to swap them for the content of the Purple Pen event
error-import-ppen-read = cannot read { $path }: { $source }
error-ppen-not-a-file = { $path } is not a file
error-ppen-too-large = { $path } is larger than { $limit } MB, the limit for a Purple Pen event
error-import-ppen-scale-mismatch = the Purple Pen event is drawn for 1:{ $file } and the map is 1:{ $map }; export the courses for the map's scale
error-import-ppen-map-scale-unknown = the Purple Pen event is drawn for 1:{ $file }, and the map states no scale; set the scale in the map file
error-import-ppen-outside-map = only { $inside } of { $total } controls lie on the map; the event belongs to another map
error-import-ppen-invalid = the Purple Pen event cannot be read: { $source }
error-import-ppen-fit = the Purple Pen event does not fit the map: { $source }
mcp-error-not-ppen = { $path } is not a Purple Pen event; these files end in .ppen
mcp-error-ppen-symlink = The link sources.ppen of the project names { $path }, which is a symbolic link. The server does not follow symbolic links.
mcp-error-sync-ppen-invalid = The Purple Pen file { $path } cannot be synced. It is not a readable Purple Pen event, or it does not fit the project's map. Open it in Purple Pen to check it.
mcp-error-sync-invalid = The merged project would not be valid, so nothing was written. Run the sync on the command line to see the details.
import-ppen-note-unknown-element = not known and left out: { $what }
import-ppen-note-skipped = course { $course } is left out: { $why }
import-ppen-note-gaps = course { $course } has circle or leg gaps; the model has no gaps, so they are drawn whole
import-ppen-note-unmapped-symbol = control { $control }, column { $column }: symbol { $id } does not exist in the 2024 standard and is left out
import-ppen-note-number-into-circle = control { $control }: the number touches the circle or reaches into it; the editors want a gap above 0, so the placement is kept as the file has it
import-ppen-note-fit-scale-unchecked = the event states no map scale, so it was not compared with the map's
import-ppen-note-fit-orientation-assumed = the positions fit the map with y up and with y down; y up was used, so check the positions
import-ppen-note-fit-axis-against-format = the y axis in the project goes against the file format; the sheet may print mirrored
import-ppen-note-fit-other = the fit to the map reported something that has no text in this program
import-ppen-note-map-exchange = course { $course }: map exchange { $control } became a bend of the leg
import-ppen-note-map-exchange-at-control = course { $course }: the map exchange at control { $control } is not kept
import-ppen-note-secondary-title = course { $course }: the secondary title "{ $title }" is not kept
import-ppen-note-column-f-text = control { $control }: the text "{ $text }" in column F is not kept
import-ppen-note-start-finish-description = { $id }: the description of a start or finish is not kept
import-ppen-note-map-issue-as-start = map issue point { $id } was imported as a start
import-ppen-note-special-skipped = a special object of kind { $kind } is not supported and left out
import-ppen-note-special-without-course = a special object of kind { $kind } applies only to courses that were left out and is left out too
import-ppen-note-course-renamed = course "{ $from }" is called "{ $to }" in the project; course names are unique and have no white space at the ends
import-ppen-note-control-without-position = control { $control } has no position and no course uses it; it is left out
import-ppen-note-special-off-map = a special object of kind { $kind } has a point outside the map and is left out
import-ppen-note-number-gap-clamped = control { $control }: the number lies far from the circle; the gap is cut to { $max } mm
import-ppen-note-special-course-ambiguous = a special object of kind { $kind } names the course { $name }, but several courses have that name; it applies to all of them
import-ppen-note-forked = courses { $variants } are the variants of the fork { $family }
import-ppen-note-notes-truncated = the event has more notes than the importer can hold; the rest is left out
word-import-why-variations = it has variations
word-import-why-alternative-controls = it has alternative controls
word-import-why-score-course = score courses are not imported
word-import-why-no-start = it does not begin at a start
word-import-why-no-finish = it does not end at a finish
word-import-why-start-or-finish-inside-the-course = a start or finish lies inside the course
word-import-why-no-name = it has no name
word-import-special-out-of-bounds = out-of-bounds area
word-import-special-dangerous-area = dangerous area
word-import-special-forbidden-route = forbidden route
word-import-special-boundary = boundary
word-import-special-marked-route = marked route
word-import-special-first-aid = first aid
word-import-special-water = water
word-import-special-text = text
word-import-special-line = line
word-import-special-white-out = white-out
mcp-tool-import-ppen = Take over the controls, courses and special objects of a Purple Pen event (.ppen) drawn for the project's map. Takes project, path, the .ppen file, and replace; without replace the project must hold no controls, courses or special objects yet, and with it they are swapped for the event's, while classes and accepted findings stay. Returns controls, courses and notes, the texts on what was left out or changed.
mcp-param-import-ppen-path = The Purple Pen event, an absolute path or one starting with ~, ending in .ppen
mcp-param-import-ppen-replace = Replace what the project already has [default: false]

## Sync with the linked Purple Pen file (sync)

cli-cmd-sync = Merge the project with its linked Purple Pen file
cli-cmd-sync-exit-codes =
    Exit code 0: synced, or nothing to do.
    Exit code 1: conflicts remain, or the sync was refused. Then nothing is synced; only decisions given as KEY=SIDE are stored in the project file.
    Exit code 2: wrong call, for example an unknown option.
cli-arg-sync-project = The project file
cli-arg-sync-dry-run = Show what the sync would do and write nothing, not even decisions
cli-arg-sync-take = Pick a side for conflicts. ours is the project, theirs is the Purple Pen file. A bare side decides every open conflict. KEY=SIDE decides one conflict and stays stored in the project file while other conflicts remain. Can be given more than once
cli-arg-sync-json = Print the result as JSON
sync-summary =
    Sync with { $path }.
    Changes to Purple Pen: { $ours }
    Changes into the project: { $theirs }
    Changed the same way on both sides: { $both }
sync-change-line = { $side ->
    [ours] { $key } goes to Purple Pen: { $value }
    [theirs] { $key } comes from Purple Pen: { $value }
   *[both] { $key } changed the same way on both sides: { $value }
}
sync-conflict-line =
    { $key } differs.
      Base: { $base }
      Project: { $ours }
      Purple Pen: { $theirs }
sync-conflicts-remain = { $count ->
    [one] 1 conflict is still open.
   *[other] { $count } conflicts are still open.
} Nothing was synced. Pick a side for all of them with --take ours or --take theirs, or for one conflict with --take KEY=ours or --take KEY=theirs.
sync-decisions-stored = { $count ->
    [one] Stored 1 decision in { $path }.
   *[other] Stored { $count } decisions in { $path }.
} They apply as long as the conflicts stay the same.
sync-stale-decision = The stored decision for { $key } no longer fits, because one of its values changed. It is ignored.
sync-nothing-to-do = The project and { $path } already agree. Nothing to do.
sync-dry-run = Dry run. Nothing was written.
sync-note-deletion-kept = control { $control } stays in the Purple Pen file, although the project deleted it, because course { $course } still visits it
sync-note-leg-change-dropped = the changed bends or marking of the leg { $from } to { $to } are dropped, because no course runs that leg any more
sync-note-new-name-taken = the new object { $from } from Purple Pen is called { $to } in the project, because the project already uses { $from }
sync-note-base-ignored = { $why }. Without a base copy, the sync deletes nothing and treats every difference as a conflict
error-sync-internal = internal error in sync ({ $what }). Please report this
error-sync-take-parse = Expected ours, theirs, KEY=ours or KEY=theirs after --take, got `{ $value }`.
error-sync-take-twice-all = --take names ours and theirs for all conflicts. Give one side.
error-sync-take-twice-key = --take names two different sides for { $key }. Give one side.
error-sync-base-unusable = The base copy { $path } cannot be read as a Purple Pen event. Delete that file and run sync again: the sync then works without a base, deletes nothing and asks about every difference.
sync-note-first-sync = first sync of this project with this file. There is no base copy yet, so the sync deletes nothing and treats every difference as a conflict; it writes the base copy when it has finished without open conflicts
sync-note-base-repaired = an earlier sync did not write the base copy. It matches the linked file again now
sync-note-not-written = not written to Purple Pen: { $note }
word-sync-why-missing = there is no usable base copy
word-sync-why-other-link = the base copy belongs to another Purple Pen file
word-sync-why-hash = the base copy does not match the hash in the project file
word-sync-value-deleted = deleted
word-sync-value-none = none
word-sync-value-empty = empty
word-sync-kind-control = control
word-sync-kind-crossing-point = crossing point
word-sync-kind-start = start
word-sync-kind-finish = finish
error-sync-no-link = { $path } has no link to a Purple Pen file. Link it first with export --ppen FILE or import --ppen FILE
error-sync-link-missing = the linked Purple Pen file { $path } does not exist
error-sync-changed-meanwhile = { $path } changed while the sync ran, probably because Purple Pen saved it. Nothing was written. Run sync again
error-sync-partial = the sync wrote { $written } and then failed: { $source }. Run sync again to finish the job
error-sync-take-unknown-key = no open conflict has the key { $key }
error-sync-base = the base copy for { $path } cannot be imported: { $source }
error-sync-theirs = the linked Purple Pen file { $path } cannot be imported: { $source }
error-sync-export = the project cannot be written as a Purple Pen event for { $path }: { $source }
error-sync-unknown-version = { $path } comes from a Purple Pen version this program does not know ({ $version }). Nothing was written
error-sync-unmatched = { $count ->
    [one] the writer could not match 1 object of { $path }: { $list }. Nothing was written
   *[other] the writer could not match { $count } objects of { $path }: { $list }. Nothing was written
}
error-sync-invalid = the merged event is not valid, so nothing was written: { $errors }
error-sync-verify = the Purple Pen file would not read back as the merged project ({ $list }). Nothing was written

## MCP server (tool names, parameter names and JSON fields stay English)

cli-help-mcp = Serve course setting to AI assistants over MCP on stdin and stdout

mcp-instructions =
    freemapper-courseplan sets orienteering courses on an OOM map: controls, start and finish, classes, courses and
    the control numbers. Every tool takes project, the project file of the event, as an absolute path or one
    starting with ~. init_event makes a project file from a map. Positions are in map millimetres at map scale,
    with y pointing down, written as objects with the numbers x and y. Workflow: event_info, then map_extent and
    map_features_near to find positions, then add_special for the start and the finish, add_control for each
    control, add_course, place_numbers and event_info again to read the length. Each call saves its change. A
    refused change leaves the file as it was. Individual and score courses can be edited and checked, but only
    individual courses have a measured length. A score course is made by add_score_course; set_score_controls and
    set_control_value set its controls, add_score_link and remove_score_link its mandatory links,
    set_score_course_details the time limit and the penalty, and set_score_settings the point range and the paces
    of the event. score_suggest lists control sites, score_draft drafts a score course and writes only with apply
    true, score_length analyses an existing one. All three take seconds. Team and relay courses cannot be edited yet.
    check tests the courses against the rule profile of the event, which set_rule_profile changes. Some checks
    need the start window (set_start_window) or the rule category and expected starters of a class (set_class);
    without them check lists the rule as skipped. A finding the course setter wants as it is gets accept_finding
    with a note; remove_accepted_finding takes that back. check_update tells whether a newer freemapper-courseplan
    exists; install_update installs it, only with confirm: true after the person agreed, and the assistant then
    restarts the freemapper-courseplan server.
mcp-error-relative-path = { $path } is a relative path; use an absolute path or one starting with ~
mcp-error-not-toml = { $path } is not a project file; project files end in .toml
error-import-not-map = { $path } is not an OOM map; maps end in .omap or .xmap
mcp-error-not-map = { $path } is not an OOM map; maps end in .omap or .xmap
mcp-error-not-xml = { $path } is not an XML file; the export file ends in .xml
mcp-error-project-parse = { $path } is not a valid project file; the first problem is at line { $line }, column { $column }
mcp-error-project-parse-nopos = { $path } is not a valid project file
mcp-error-project-changed = { $path } changed on disk after it was read, so nothing was written. Read the project again and repeat the change
mcp-error-map-invalid = { $path } is not a valid OOM map
mcp-error-no-home = this system has no home folder; use an absolute path
mcp-error-kind-needs-at = a { $kind } is a single point; give at and leave out points
mcp-error-kind-needs-points = a { $kind } is drawn through points; give points and leave out at
mcp-error-no-courses-for-kind = a { $kind } applies to every course; leave out courses
mcp-error-leg-target = give either from_start or after, not both and not neither
mcp-error-no-change = nothing to change; give at least one of classes, climb_m, no_climb, print_scale and no_print_scale
mcp-error-start-window-choice = give either minutes or no_start_window: minutes sets the start window, no_start_window removes it
mcp-error-no-class-change = nothing to change; give at least one of rule_category, no_rule_category, expected_starters and no_expected_starters
mcp-error-number-args = give direction and gap_mm together, or clear
mcp-error-bad-gap = gap_mm { $value } is out of range; give more than 0 and at most { $max } print millimetres
mcp-error-special-courses-args = give either courses with at least one course name, or all as true

mcp-tool-event-info = Summary of the event: title, map and map kind, map extent, the numbers of controls, special objects, classes and courses, and per course the controls, the length in km, the climb and how many control numbers are placed.
mcp-tool-list-controls = List the controls with code, kind, position in map mm, description symbols and number placement.
mcp-tool-list-courses = List the courses with name, kind, start, finish, controls in running order, classes, climb, planned print scale and length in km.
mcp-tool-list-specials = List the special objects (start, finish, marked routes, out-of-bounds areas, forbidden routes, dangerous areas, first aid posts, water points) with their geometry and courses.
mcp-tool-list-classes = List the classes with rule category and expected starters.
mcp-tool-add-control = Add a control at a position in map mm. Without code the lowest free code from 31 up is used. The position must lie on the map. Returns the code.
mcp-tool-move-control = Move a control to a new position in map mm. The position must lie on the map.
mcp-tool-set-control-code = Change the code of a control. The courses, the accepted findings and every other place that names the control follow. In a score course the control takes the value its new code carries; a code that carries none keeps the old value. The new code must be free, not empty and at most 64 characters long.
mcp-tool-remove-control = Remove a control and its visits in individual courses. The result lists the courses it left, and in cleared_legs the legs that lost their bends and marked-route flag because they now run to another stop.
mcp-tool-describe-control = Set the description symbols of a control, column C to H, as IOF symbol ids. Columns you leave out keep their symbol. clear empties all columns first.
mcp-tool-set-control-number = Set where the number of a control is drawn, as direction in degrees (0 = east, counter-clockwise) and gap between circle line and number in print mm. clear goes back to the default placement.
mcp-tool-add-special = Add a special object. A start, finish, first aid post or water point is a single point (at). A marked route, forbidden route, out-of-bounds area or dangerous area is drawn through points. Its id must be new.
mcp-tool-remove-special = Remove a special object by id. A start or finish that a course uses cannot be removed.
mcp-tool-set-special-courses = Set the courses a special object belongs to: a list of course names, or all as true for all courses. Not for start and finish. remove_course refuses to remove the only course of a special object, so widen or reassign it here first.
mcp-tool-add-class = Add a class.
mcp-tool-remove-class = Remove a class and its name in all courses.
mcp-tool-set-class = Change the rule category or the expected number of starters of a class. Parameters you leave out stay as they are; no_rule_category and no_expected_starters remove the value. The table checks read the rule category, the start capacity check the expected starters.
mcp-tool-add-course = Add an individual course from a start, a finish and the control codes in running order. The start and finish are ids of special objects.
mcp-tool-remove-course = Remove a course and its name from the special objects.
mcp-tool-rename-course = Rename a course. The course lists of special objects and the accepted findings follow. The new name must be free, not empty, without white space at its ends and at most 200 characters long.
mcp-tool-set-course-controls = Replace the controls of a course. The n-th visit of a code keeps the number placement of the n-th visit before. A leg keeps its bends and marked-route flag only while it runs to the same next stop; the result lists the cleared legs in cleared_legs as course and after (0 is the leg from the start).
mcp-tool-set-leg = Set the bends and the marked-route flag of one leg of an individual course: the leg from the start (from_start) or the leg after a position (after, counted from 1 as on the map, crossing points included). Without bends the leg loses its bends.
mcp-tool-set-course-details = Change the classes, the climb or the planned print scale of a course. Parameters you leave out stay as they are.
mcp-tool-place-numbers = Place the control numbers of a course behind the circle as seen from the incoming leg, clear of legs, circles, other numbers and point and line symbols of the map. Returns how many numbers it placed. It runs only when called, so call it again after changing legs. A control with a central number (set_control_number) keeps it, and course numbers (set_course_number) stay unless overwrite is true. The number moves outward up to the maximum gap of the rule profile before it turns away from the preferred direction.
mcp-tool-set-course-number = Set or clear the number placement of one control in an individual course, at a position counted from 1 as on the map (crossing points included). It wins over the central placement of set_control_number and over place_numbers. Direction in degrees (0 = east, counter-clockwise), gap between circle line and number in print mm, more than 0 and at most 10. clear removes it, so the central or automatic placement applies again.
mcp-tool-map-features-near = Map objects within radius_mm of a position, nearest first: symbol code, geometry kind (point, line, area), distance in mm and the part of the map.
mcp-tool-map-extent = Smallest and largest corner of the map objects in map mm, the map scale and the map kind.
mcp-tool-init-event = Start an event from an OOM map (sprint or schoolyard) and write its project file. Returns the path of the project file.
mcp-tool-check = Check the individual and score courses against the rule profile and overlays of the event. Returns counts in summary; findings with rule, profile, severity (error, warning or hint), location as an object with kind, message_id, args, source with doc, section and title, rule_note, and text; accepted findings with the note of the course setter; unmatched, the accepted findings that match no finding any more, for example because a control or course is gone (remove them with remove_accepted_finding if they are no longer needed; they never count as errors); and skipped rules with the reason, including rules an overlay switched off. A leg location carries course, after and the ids of its ends in from and to. Team and relay courses are not checked yet.
mcp-tool-accept-finding = Accept a finding as an intended exception, with a note. Name it as check reports it: the rule and its place, given as controls, legs or the course. A start or finish id may stand in controls, as check reports it. A leg is given by course and after; the server stores the ids of its ends from the course as it is now, so the finding comes back when the leg later runs between other stops. Leave out all three for a finding of the whole event. Accepting the same finding again replaces the note. check then lists it under accepted.
mcp-tool-remove-accepted-finding = Remove an accepted finding, so check reports it again. Name it with the rule and the same place as when it was accepted. A leg is given by course and after. The entry for the leg as it runs now is removed first; if there is none, every entry at that position goes, so an entry from unmatched can be removed after its leg or course changed. Returns the removed entries.
mcp-tool-set-rule-profile = Set the base rule profile of the event and its overlays. Both are loaded first; an unknown profile, or an overlay given as base, is refused with the list of available profiles. Without overlays the event has none.
mcp-tool-set-start-window = Set the start window of the event, the minutes from the first to the last start, at least 1, or remove it with no_start_window. The start capacity check divides the expected starters by it.
mcp-tool-export-iof-xml = Write the individual and score courses of the event as IOF XML 3.0 CourseData for result software such as OE. Takes project and path, the file to write, which ends in .xml; an existing file is replaced. Returns path, courses, the number of courses written, and skipped, the team and relay courses left out, each with name and kind. Controls, start and finish get latitude and longitude when the map is georeferenced; geo_positions says whether they did.
mcp-tool-export-ppen = Write the individual courses of the event into a Purple Pen event (.ppen). Takes project, path (the .ppen file) and new. The export changes an existing file in place and keeps everything the project did not change. It creates a missing file, and with new it writes the file from scratch; either way a project without sources.ppen is linked to the file. The export never deletes: objects in the file that the project lacks stay and are reported. The project's Purple Pen ids count only for the file that sources.ppen names, and only for that file does the export store the ids it assigns in the project. Returns path, courses (the number of courses written), skipped (the team, relay and score courses left out, each with name and kind), notes (the texts on what was left out, changed or kept), patched (true when an existing file was changed in place) and ids_stored (how many objects of the project got a Purple Pen id for the first time).

mcp-param-project = Project file of the event, an absolute path or one starting with ~, for example ~/maps/hall.courseplan.toml
mcp-param-export-iof-xml-path = The file to write, an absolute path or one starting with ~, ending in .xml
mcp-param-export-ppen-path = The Purple Pen event to write, an absolute path or one starting with ~, ending in .ppen; it is created when missing
mcp-param-export-ppen-new = Write the file from scratch and replace an existing one [default: false]
mcp-tool-sync-ppen = Merge the project with the Purple Pen file that sources.ppen names. The merge compares both with the base copy the last sync left, so a change on one side lands on the other and a change on both sides to the same field is a conflict. Takes project, dry_run and take. With dry_run nothing is written. take (ours or theirs) settles every open conflict for this call only and stores nothing. While conflicts are open, the sync writes nothing; use resolve_sync_conflict to settle them one by one. A sync with a clean outcome writes the Purple Pen file, the project and the base copy next to the project file. Returns written (true when the sync changed files), dry_run, ppen_changed (the Purple Pen file changed), base (use, repair, first or ignore), changes (key, side and value of each change), conflicts (key, base, ours and theirs of each open conflict), notes (texts on the base copy and on what was kept or not written) and stale (stored decisions that no longer fit).
mcp-tool-resolve-sync-conflict = Store the answer to one open sync conflict in the project file, without syncing. Takes project, key (as sync_ppen lists it under conflicts) and take (ours keeps the project's value, theirs the Purple Pen file's). The answer counts only while both values stay as they are, and the next completed sync clears it. A key that is no open conflict is refused. Returns the same fields as a dry run of sync_ppen, so conflicts holds what is still open; when it is empty, call sync_ppen to finish.
mcp-param-sync-ppen-dry-run = Report what the sync would do and write nothing [default: false]
mcp-param-sync-ppen-take = Settle every open conflict for this call with the project's value (ours) or the Purple Pen file's (theirs); nothing is stored [default: leave conflicts open]
mcp-param-resolve-sync-conflict-key = The key of the conflict, as sync_ppen lists it under conflicts
mcp-param-resolve-sync-conflict-take = Which value to keep, the project's (ours) or the Purple Pen file's (theirs)
mcp-param-init-event-project = Where to write the project file, an absolute path or one starting with ~ [default: next to the map, named after it with .courseplan.toml]
mcp-param-map = The OOM map (sprint or schoolyard), an absolute path or one starting with ~
mcp-param-title = Event title [default: the title in freemapper.toml next to the map, else the map name]
mcp-param-code = Code of the control, a string or a whole number
mcp-param-set-control-code-from = Current code of the control, a string or a whole number
mcp-param-set-control-code-to = New code, a string or a whole number, that no other control has
mcp-param-rename-course-from = Current name of the course
mcp-param-rename-course-to = New name of the course, which no other course has
mcp-param-add-control-code = Code of the new control, a string or a whole number [default: the lowest free number from 31 up]
mcp-param-at = Position in map millimetres, y pointing down
mcp-param-add-special-at = Position of a single-point kind (start, finish, first_aid, water)
mcp-param-x = X in map millimetres
mcp-param-y = Y in map millimetres, pointing down
mcp-param-crossing-point = Add a crossing point, which gets no number [default: false]
mcp-param-c = Symbol for column C (IOF symbol id)
mcp-param-d = Symbol for column D (IOF symbol id)
mcp-param-e = Symbol for column E (IOF symbol id)
mcp-param-f = Symbol for column F (IOF symbol id)
mcp-param-g = Symbol for column G (IOF symbol id)
mcp-param-h = Symbol for column H (IOF symbol id)
mcp-param-set-control-number-clear = Go back to the default placement; then leave out direction and gap_mm [default: false]
mcp-param-describe-control-clear = Empty all columns first; columns given in the same call are set afterwards [default: false]
mcp-param-direction = Degrees, 0 = east, counter-clockwise; any value is brought into 0 to 360
mcp-param-gap-mm = Gap between circle line and number in print millimetres, more than 0 and at most 10
mcp-param-kind = What the object is: start, finish, marked_route, out_of_bounds, forbidden_route, dangerous_area, first_aid or water
mcp-param-id = Id of the object, unique among special objects
mcp-param-points = The points of a marked_route, forbidden_route, out_of_bounds or dangerous_area, in order; each is an object with x and y in map millimetres
mcp-param-courses = Limit the object to these courses [default: all courses]; not for start and finish
mcp-param-set-special-courses-courses = The courses the object belongs to, at least one name; leave out when all is true
mcp-param-all = Let the object apply to all courses; leave out courses then [default: false]
mcp-param-set-special-courses-id = Id of the special object; not a start or finish
mcp-param-name = Name of the course
mcp-param-add-class-name = Name of the class
mcp-param-remove-class-name = Name of the class
mcp-param-rule-category = Rule category of the class
mcp-param-expected-starters = Expected number of starters
mcp-param-set-class-name = Name of the class
mcp-param-no-rule-category = Remove the rule category [default: false]
mcp-param-no-expected-starters = Remove the expected number of starters [default: false]
mcp-param-minutes = Minutes from the first to the last start, at least 1
mcp-param-start = Id of the start special object
mcp-param-finish = Id of the finish special object
mcp-param-controls = Codes of the controls in running order, strings or whole numbers; a crossing point counts as a position [default: none]
mcp-param-classes = Classes that run the course
mcp-param-climb-m = Climb in metres
mcp-param-print-scale = Planned print scale, for example 3000 for 1:3000
mcp-param-codes = Codes in running order, strings or whole numbers; an empty list removes all controls
mcp-param-from-start = Edit the leg from the start to the first control [default: false]
mcp-param-after = Edit the leg after this position, counted from 1 as on the map; crossing points count as positions
mcp-param-bends = The bends of the leg in order, each an object with x and y in map millimetres [default: none, the leg loses its bends]
mcp-param-taped = Mark the leg as a marked route [default: false, the flag is cleared]
mcp-param-no-climb = Remove the climb [default: false]
mcp-param-no-print-scale = Remove the planned print scale [default: false]
mcp-param-no-start-window = Remove the start window [default: false]
mcp-param-overwrite = Also replace course numbers placed before; central numbers stay [default: false]
mcp-param-set-course-number-after = Position of the control in the course, counted from 1 as on the map; crossing points count as positions
mcp-param-set-course-number-clear = Remove the placement of the course, so the central or the automatic placement applies again; then leave out direction and gap_mm [default: false]
mcp-param-radius-mm = Search radius in map millimetres
mcp-param-limit = Most features to return [default: 20, at most 200]
mcp-param-rule = Id of the rule as check reports it, for example wkb2026.distance.min
mcp-param-note = Why the exception is intended
mcp-param-accept-finding-controls = The controls of the finding, strings or whole numbers, as in the location of check; start and finish ids are allowed [default: none]
mcp-param-remove-accepted-finding-controls = The controls of the finding, strings or whole numbers, as in the location of check [default: none]
mcp-param-accept-finding-legs = The legs of the finding, each an object with course and after as in the location of check: one leg, or two for a finding at two legs [default: none]
mcp-param-remove-accepted-finding-legs = The legs of the finding, each an object with course and after as in the location of check: one leg, or two for a finding at two legs [default: none]
mcp-param-accept-finding-course = Name of the course; for a finding about a whole course, and inside legs for the course of the leg
mcp-param-remove-accepted-finding-course = Name of the course; for a finding about a whole course, and inside legs for the course of the leg
mcp-param-accept-finding-after = Position the leg leaves in the stop list of the course: the controls of the course count from 1, crossing points included, and 0 is the leg from the start. It is not the control number on the map; the after of a leg location in check is the value to give
mcp-param-remove-accepted-finding-after = Position the leg leaves in the stop list of the course: the controls of the course count from 1, crossing points included, and 0 is the leg from the start. It is not the control number on the map; the after of a leg location in check is the value to give
mcp-param-base = The base rule profile, for example wkb-2026
mcp-param-overlays = Overlays on the base profile in the order they apply, for example nor-bb-2021 [default: none]

## Editor (edit command and the editor page; editor-* messages take no arguments)

cli-help-edit = Open the course editor in the browser
cli-arg-edit-project = The project file
cli-arg-edit-port = Port on 127.0.0.1 [default: a free port]
cli-arg-edit-no-browser = Print the address and leave the browser closed
cli-edit-stop-hint = The editor is running. Stop it with Ctrl+C or with "Close editor" in the page.
cli-edit-browser-failed = The browser did not open ({ $reason }). Open the address above by hand.
cli-edit-bind-failed = Cannot listen on port { $port } of 127.0.0.1 ({ $reason }). Choose another port with --port, or leave it out to take a free one.
cli-edit-runtime-failed = Cannot start the editor: { $reason }
cli-edit-server-failed = The editor stopped with an error: { $reason }

editor-page-title = Course editor
editor-menu-event = Event
editor-menu-export-xml = Export IOF XML (download)
editor-menu-export-ppen = Export to Purple Pen…
editor-menu-import = Import Purple Pen file…
editor-menu-sync = Sync with Purple Pen…
editor-menu-close = Close editor
editor-pane-tree = Contents of the event
editor-pane-map = Map
editor-pane-props = Properties and findings
editor-tree-courses = Courses
editor-tree-classes = Categories
editor-tree-controls = Controls
editor-tree-specials = Special objects
editor-props-heading = Properties
editor-props-empty = Select a course, a control or a special object to see its properties.
editor-findings-heading = Findings
editor-findings-empty = No findings for this selection.
editor-toolbar-label = Tools
editor-tool-select = Select
editor-tool-control = Place control
editor-tool-crossing = Place crossing point
editor-tool-special = Special object
editor-tool-course = Course mode
editor-tool-preview = Print preview
editor-tool-preview-hint = Shows the selected course the way the print draws it. Editing is off while it is on. Select a course first.
editor-status-label = Status
editor-status-ready = Ready
editor-status-closed = The editor has stopped. You can close this tab.
editor-status-busy = Another change is still running. Nothing was sent; try again.
editor-status-lost = The editor does not answer. Start it again with the edit command.
editor-map-bad-request = The requested map section is not valid.
editor-map-too-large = The map section is too large to draw at this zoom. The courses stay visible; zoom out or move to a smaller section.
editor-map-clipped = The map is larger than one A3 sheet at map scale. The editor draws only the middle part; courses outside it stay on the overlay.
editor-map-incomplete = The renderer left out or cut short some map objects because the map is very detailed. The map area is incomplete; the courses are not affected.
editor-tree-label = Event tree
editor-tree-all-controls = All controls
editor-none = none
editor-kind-course = Course
editor-kind-control = Control
editor-kind-crossing-point = Crossing point
editor-kind-start = Start
editor-kind-finish = Finish
editor-kind-marked-route = Marked route
editor-kind-out-of-bounds-area = Out-of-bounds area
editor-kind-forbidden-route = Forbidden route
editor-kind-dangerous-area = Dangerous area
editor-kind-first-aid = First aid post
editor-kind-water = Water point
editor-prop-name = Name
editor-prop-code = Code
editor-prop-id = Id
editor-prop-kind = Kind
editor-prop-position = Position
editor-prop-description = Description
editor-prop-length = Length
editor-prop-climb = Climb
editor-prop-stops = Course stops
editor-prop-start = Start
editor-prop-finish = Finish
editor-prop-classes = Categories
editor-prop-used-in = Used in
editor-desc-column-c = Column C
editor-desc-column-d = Column D
editor-desc-column-e = Column E
editor-desc-column-f = Column F
editor-desc-column-g = Column G
editor-desc-column-h = Column H
editor-legend-auto-number = Dashed number boxes are placed automatically.
editor-zoom-in = Zoom in
editor-zoom-out = Zoom out
editor-zoom-fit = Fit the map to the window
editor-status-map = Map
editor-status-scale = Scale
editor-status-kind = Kind
editor-status-cursor = Cursor (mm)
editor-mapkind-sprint = sprint
editor-mapkind-schoolyard = schoolyard
editor-mapkind-forest = forest
editor-status-reloaded = The project file changed and was loaded again.
editor-status-ppen-changed = The Purple Pen file changed. Sync?
editor-status-state-failed = The project cannot be shown. The server console tells why.
editor-status-overlay-failed = The course symbols could not be loaded.
editor-status-course-unsupported = This kind of course (team or relay) cannot be shown in the editor yet.
editor-status-preview-on = Print preview on. The course comes from the print renderer, and nothing can be edited.
editor-status-preview-off = Print preview off.
editor-op-unknown = This operation does not exist.
editor-op-no-revision = The request does not say which project revision it is based on.
editor-op-bad-body = The request body is not a JSON object.
editor-op-stale = The project file changed outside the editor. The editor loaded the current state; nothing was changed.
editor-io-failed = The project file could not be read or written. Nothing was changed.
editor-undo-nothing = Nothing to undo.
editor-redo-nothing = Nothing to redo.
editor-undo-changed = The project file changed outside the editor, so undo and redo are no longer possible. Nothing was changed.
editor-undo-failed = The project file could not be written
editor-op-add-control = Place control
editor-op-move-control = Move control
editor-op-set-control-code = Change control code
editor-op-remove-control = Delete control
editor-op-describe-control = Describe control
editor-op-set-control-number = Set control number
editor-op-add-special = Add special object
editor-op-remove-special = Remove special object
editor-op-set-special-courses = Set courses of special object
editor-op-add-class = Add category
editor-op-set-class = Change category
editor-op-remove-class = Remove category
editor-op-add-course = Add course
editor-op-remove-course = Remove course
editor-op-rename-course = Rename course
editor-op-set-course-controls = Change course controls
editor-op-set-leg = Change leg
editor-op-set-course-details = Change course details
editor-op-place-numbers = Place numbers
editor-op-set-course-number = Set course number
editor-op-accept-finding = Accept finding
editor-op-remove-accepted-finding = Withdraw accepted finding
editor-op-set-rule-profile = Change rule profile
editor-op-set-start-window = Change start window
editor-op-resolve-sync-conflict = Resolve sync conflict
editor-tool-undo = Undo
editor-tool-redo = Redo
editor-tool-help = Keyboard shortcuts
editor-banner-dismiss = Dismiss
editor-help-title = Keyboard shortcuts
editor-help-close = Close
editor-key-undo = Undo
editor-key-redo = Redo
editor-key-delete = Delete the selected control, bend or special object, or take back the last point of a drawing (focus on the map or the list; undo brings it back)
editor-key-cancel = Cancel the drag, the drawing or the chosen leg, or go back to the select tool
editor-key-select-tool = Select tool
editor-key-control-tool = Place-control tool
editor-key-nudge = Move the selected control by 0.1 mm
editor-key-nudge-big = Move the selected control by 1 mm
editor-key-zoom-in = Zoom in (Ctrl or Cmd with + still zooms the browser)
editor-key-zoom-out = Zoom out
editor-key-fit = Fit the map to the window
editor-key-help = Show this list
editor-confirm-title = Delete control
editor-confirm-used = These courses visit the control. Deleting it removes it from them.
editor-confirm-delete = Delete
editor-cancel = Cancel
editor-prop-x = X (mm)
editor-prop-y = Y (mm)
editor-prop-delete = Delete control
editor-prop-near-heading = Map objects within 3 mm
editor-prop-near-none = No map objects nearby.
editor-geometry-point = point
editor-geometry-line = line
editor-geometry-area = area
editor-error-number = Enter a number in millimetres, for example 12.5 or 12,5.
editor-error-code = Enter a code of 1 to 64 characters, without spaces at its ends, that no other control has.
editor-error-course-name = Enter a name of 1 to 200 characters, without spaces at its ends, that no other course has.
editor-status-code-changed = Code changed.
editor-status-course-renamed = Course renamed.
editor-status-placed-crossing = Crossing point placed.
editor-status-placed = Control placed.
editor-status-moved = Control moved.
editor-status-deleted = Control deleted
editor-status-undone = Undone
editor-status-redone = Redone
editor-tree-event = Event
editor-tree-event-settings = Event settings
editor-tree-new-course = New course...
editor-tree-new-class = New category...
editor-new-course-title = New course
editor-new-score-course-title = New score course
editor-new-course-label = Course name
editor-new-class-title = New category
editor-new-class-label = Category name
editor-create = Create
editor-error-class-name = Enter a name of 1 to 200 characters, without spaces at its ends, that no other category has.
editor-error-count = Enter a whole number, for example 120.
editor-prop-print-scale = Planned print scale (1:n)
editor-prop-rule-category = Rule category
editor-prop-starters = Expected starters
editor-prop-discipline = Discipline
editor-prop-rule-profile = Rule profile
editor-prop-rule-overlays = Overlays (separated by commas)
editor-prop-start-window = Start window (minutes)
editor-prop-delete-course = Delete course
editor-prop-delete-class = Delete category
editor-stop-up = Move up
editor-stop-down = Move down
editor-stop-remove = Remove
editor-stops-empty = The course has no controls yet. Use course mode and click controls on the map.
editor-stops-hint = Alt with the up or down arrow moves the focused stop.
editor-leg-from-start = from the start
editor-leg-after = after stop
editor-status-course-created = Course created.
editor-status-course-changed = Course changed.
editor-status-cleared = These legs lost their bends or tape
editor-status-course-deleted = Course deleted.
editor-status-details-changed = Course details changed.
editor-status-class-added = Category added.
editor-status-class-changed = Category changed.
editor-status-class-deleted = Category deleted.
editor-status-profile-changed = Rule profile changed.
editor-status-window-changed = Start window changed.
editor-status-pick-course = Select a course in the tree first.
editor-status-need-start-finish = A new course needs exactly one start and one finish in the event. Place them first.
editor-status-course-mode = Course mode. Click a control to add it before the finish. Click a leg, then a control, to insert it there.
editor-status-leg-picked = Click a control to insert it there. Esc drops the choice.
editor-leg-word = Leg
editor-status-leg-cancelled = Choice of the leg dropped.
editor-key-move-stop = Move the focused stop of a course up or down
error-editor-map = cannot read the map { $file }: { $reason }
error-editor-project-too-large = { $file } is larger than { $limit } MiB. The editor does not open project files that big.
editor-prop-taped = Taped
editor-prop-bends = Bends
editor-prop-bend = Bend
editor-status-leg-selected = Drag the line to add a bend. Drag a bend to move it. Delete removes the selected bend.
editor-status-bend-added = Bend added.
editor-status-bend-moved = Bend moved.
editor-status-bend-removed = Bend removed.
editor-status-taped-changed = Taped flag of the leg changed.
editor-number-bad-request = The question about the number position is not valid.
editor-number-not-found = The course or the number does not exist.
editor-numbers-heading = Control numbers
editor-numbers-hint = In course mode, drag a number box to place it. Dashed boxes are placed automatically.
editor-numbers-for-all = for all courses
editor-numbers-place = Place numbers
editor-numbers-overwrite = also replace placed numbers
editor-status-number-moved = Number moved.
editor-status-numbers-placed = Numbers placed:
editor-status-number-selected = Drag the number to move it.
editor-prop-source = Placement
editor-number-source-course = set for this course
editor-number-source-central = set for the control in all courses
editor-number-source-auto = automatic
editor-status-number-shadowed = Saved on the control. This course has its own placement for the number, so the box stays.
editor-status-number-clamped = The gap was set to the nearest allowed value, between 0.01 and 10 mm.
editor-special-kind = Kind
editor-special-id = Id
editor-key-special-tool = Special-object tool
editor-key-finish = Finish the line or area being drawn
editor-key-preview = Turn the print preview on or off
editor-prop-points = Points
editor-prop-courses = Courses
editor-prop-courses-all = No course ticked: the object belongs to all courses.
editor-prop-courses-some = The object belongs to the ticked courses only.
editor-prop-delete-special = Delete special object
editor-prop-special-no-move = A special object cannot be moved. Delete it and draw it again.
editor-error-special-id = Enter an id of 1 to 200 characters, without spaces at its ends, that no other special object has.
editor-status-special-tool = Special object. Choose a kind, then click on the map.
editor-status-draft-points = points
editor-status-draft-hint = Click to add a point. Enter or a double click finishes, Esc cancels, Delete removes the last point.
editor-status-draft-cancelled = Drawing cancelled.
editor-status-draft-short = Too few points for this kind. Add more.
editor-status-special-added = Special object added.
editor-status-special-deleted = Special object deleted.
editor-status-special-courses = Courses of the special object changed.
editor-symbols-bad-request = The column must be one letter from C to H.
editor-desc-heading = Control description
editor-desc-hint = Arrow keys move, Enter picks a symbol, digits and dots jump to a number.
editor-desc-none = No symbol
editor-desc-loading = Loading symbols.
editor-desc-failed = The symbols could not be loaded.
editor-desc-selected = selected
editor-desc-current = current value:
editor-status-described = Description changed.
editor-severity-error = Error
editor-severity-warning = Warning
editor-severity-hint = Hint
editor-findings-errors = Errors
editor-findings-warnings = Warnings
editor-findings-hints = Hints
editor-findings-open = Show all findings of the event
editor-findings-pending = Checking.
editor-findings-failed = The check could not run.
editor-findings-pick = Select a control, a leg or a course to see its findings. The status bar counts those of the whole event.
editor-findings-accepted = Accepted here
editor-findings-accept = Accept…
editor-findings-remove = Remove acceptance
editor-findings-note = Note
editor-accept-title = Accept finding
editor-accept-label = Why is this fine? The note is saved in the project file, up to 200 characters.
editor-accept-ok = Accept
editor-error-accept-note = Write a note of 1 to 200 characters.
editor-drawer-title = Findings of the event
editor-drawer-close = Close
editor-drawer-findings = Open findings
editor-drawer-accepted = Accepted
editor-drawer-unmatched = Acceptances that match nothing now
editor-drawer-skipped = Not checked
editor-drawer-none = No open findings.
editor-drawer-all-checked = Every rule ran.
editor-status-accepted = Finding accepted.
editor-status-acceptance-removed = Acceptance removed.
editor-status-located = Shown on the map.

editor-export-xml-title = IOF XML
editor-export-ppen-title = Export to Purple Pen
editor-export-ppen-intro = Choose where the Purple Pen file goes.
editor-export-ppen-linked = Linked file
editor-export-ppen-to-linked = Write to the linked file
editor-export-ppen-linked-note = The linked file is updated in place. Undo and redo are cleared afterwards.
editor-export-ppen-download = Download a new file
editor-export-ppen-download-note = A new file from the current project. Nothing is stored in the project.
editor-export-no-link = This event is not linked to a Purple Pen file. The command export --ppen FILE creates one and links it. Until then, download a new file.
editor-export-link-refused = The linked Purple Pen file must end in .ppen and must not be a symbolic link. Nothing was written.
editor-export-ppen-failed = The Purple Pen file could not be written. Either the project does not fit its map, or the existing file is not a readable Purple Pen event. Run the export on the command line to see the details. Nothing was changed.
editor-export-bad-request = The export request is not valid.
editor-import-title = Import a Purple Pen file
editor-import-file = Purple Pen file (.ppen)
editor-import-replace = Replace what the event has now
editor-import-replace-warning = The controls, courses and special objects of this event are replaced by those of the file. Undo and redo are cleared afterwards.
editor-import-has-content = This event already has controls, courses or special objects. Tick Replace to overwrite them.
editor-import-ok = Import
editor-import-no-file = Choose a file first.
editor-import-needs-replace = This event already has controls, courses or special objects. Tick Replace to overwrite them. Nothing was changed.
editor-import-invalid = The file is not a Purple Pen event the editor can read. Nothing was changed.
editor-import-too-large = The file is too large for the editor. Nothing was changed.
editor-import-bad-request = The import request is not valid.
editor-report-title-ppen = Purple Pen export
editor-report-title-import = Purple Pen import
editor-report-close = Close
editor-report-courses = Courses
editor-report-controls = Controls
editor-report-skipped = Courses left out
editor-report-ids = New Purple Pen ids stored
editor-report-patched = The linked file was updated in place.
editor-report-created = A new file was made.
editor-report-linked-now = The file is now linked to the event.
editor-report-notes = Notes
editor-report-no-notes = No notes.
editor-report-xml-no-geo = The map has no usable georeferencing, so the file carries no geographic positions.
editor-report-xml-skipped = Courses left out of the IOF XML (team, relay and score courses)
editor-status-iof-downloaded = IOF XML downloaded.
editor-status-ppen-downloaded = Purple Pen file downloaded.
editor-status-ppen-exported = Purple Pen file written.
editor-status-imported = Purple Pen file imported.
editor-status-exchange-failed = The request failed. Nothing was changed.

editor-sync-warn-title = Before you sync
editor-sync-warn-text = Close this event in Purple Pen before you sync. A later save from Purple Pen drops what the sync added.
editor-sync-warn-hide = Don't show again this session
editor-sync-warn-continue = Continue
editor-sync-title = Sync with Purple Pen
editor-sync-linked = Linked file
editor-sync-checking = Comparing the project with the Purple Pen file…
editor-sync-nothing = Nothing to sync. Both files agree.
editor-sync-changes = Changes
editor-sync-side-ours = courseplan
editor-sync-side-theirs = Purple Pen
editor-sync-side-both = both
editor-sync-conflicts = Conflicts
editor-sync-conflicts-open = open
editor-sync-col-base = Before
editor-sync-col-ours = Courseplan
editor-sync-col-theirs = Purple Pen
editor-sync-keep-ours = Keep courseplan
editor-sync-take-theirs = Take Purple Pen
editor-sync-all-ours = Everything from courseplan
editor-sync-all-theirs = Everything from Purple Pen
editor-sync-all-note = The two "All" buttons sync at once and take that side for every open conflict.
editor-sync-resolve-first = Choose a side for each conflict first. Then Sync is available.
editor-sync-history-note = Undo and redo are cleared after a sync.
editor-sync-run = Sync
editor-sync-run-first = Create the base copy
editor-sync-first-note = The two files agree, but no base copy exists yet. Without it, the next change on either side counts as a conflict. This step writes the base copy and changes nothing else.
editor-sync-close = Close
editor-sync-stale = A stored choice no longer fits because the values changed:
editor-sync-done = Synced.
editor-sync-written-ppen = The Purple Pen file was updated.
editor-sync-written-project = The project was updated.
editor-sync-conflicts-left = Conflicts remain. Nothing was written.
editor-sync-notes = Notes
editor-sync-no-link = This event is not linked to a Purple Pen file. The command export --ppen FILE creates one and links it. Nothing was changed.
editor-sync-link-missing = The linked Purple Pen file does not exist. Nothing was changed.
editor-sync-bad-request = The sync request is not valid.
editor-sync-ppen-invalid = The linked Purple Pen file cannot be synced. It is not a readable Purple Pen event, or it does not fit the map. Open it in Purple Pen to check it. Nothing was changed.
editor-sync-invalid = The merged event would not be valid. Run the sync on the command line to see the details. Nothing was changed.
editor-status-synced = Synced with Purple Pen.
editor-status-conflict-chosen = Choice stored.
editor-status-sync-failed = The sync failed. Nothing was changed.

## List and map commands

cli-help-list = List the controls, courses, special objects or classes of a project, as a table or as JSON
cli-help-list-controls = List the controls with code, kind, position in map millimetres and number placement. With --json the answer is the one of the MCP tool list_controls
cli-help-list-courses = List the courses with kind, start, finish, number of controls, length in kilometres, climb and classes. With --json the answer is the one of the MCP tool list_courses
cli-help-list-specials = List the special objects with id, kind, position and courses. With --json the answer is the one of the MCP tool list_specials
cli-help-list-classes = List the classes with rule category and expected starters. With --json the answer is the one of the MCP tool list_classes
cli-arg-list-controls-json = Print the answer as JSON, the same as the MCP tool gives
cli-arg-list-courses-json = Print the answer as JSON, the same as the MCP tool gives
cli-arg-list-specials-json = Print the answer as JSON, the same as the MCP tool gives
cli-arg-list-classes-json = Print the answer as JSON, the same as the MCP tool gives
cli-help-map = Show the extent of the map and the map objects near a point
cli-help-map-extent = Show the extent of the map content in map millimetres, the scale and the map kind. With --json the answer is the one of the MCP tool map_extent
cli-help-map-near = List the map objects within a radius of a point, nearest first. Points and the radius are in map millimetres of the .omap frame. With --json the answer is the one of the MCP tool map_features_near
cli-arg-map-extent-json = Print the answer as JSON, the same as the MCP tool gives
cli-arg-map-near-at = The point: X,Y in map millimetres
cli-arg-map-near-radius = Search radius in map millimetres, more than 0
cli-arg-map-near-limit = Most objects to list, 1 to 200 [default: 20]
cli-arg-map-near-json = Print the answer as JSON, the same as the MCP tool gives
cli-col-code = code
cli-col-kind = kind
cli-col-position = position
cli-col-number = number
cli-col-name = name
cli-col-start = start
cli-col-finish = finish
cli-col-controls = controls
cli-col-length = km
cli-col-climb = climb m
cli-col-classes = classes
cli-col-id = id
cli-col-courses = courses
cli-col-category = category
cli-col-starters = starters
cli-col-distance = mm
cli-col-symbol = symbol
cli-col-geometry = geometry
cli-col-part = part
cli-list-empty = nothing to list
cli-list-points = { $count } points
cli-map-kind-point = point
cli-map-kind-line = line
cli-map-kind-area = area
cli-map-extent = extent: { $min } to { $max } mm
cli-map-extent-none = extent: the map has no content
cli-map-scale = scale: 1:{ $scale }
cli-map-near-total = { $count } of { $total } objects shown
cli-map-bad-radius = { $value } is not a radius; give a number of map millimetres above 0, with a dot as decimal separator
cli-query-io-failed = the project file could not be read
cli-arg-event-start-window-none = Remove the start window instead of setting it
cli-event-start-window-removed = removed the start window
error-model-link-to-unknown-control = course { $course } has a link between { $a } and { $b }, but { $code } is not one of its controls
error-model-link-to-itself = course { $course } links control { $code } to itself
error-model-duplicate-link = course { $course } links { $a } and { $b } more than once
error-model-link-branches = course { $course } links control { $code } to more than two controls; links must form chains
error-model-score-setting-out-of-range = score setting { $what } is out of range: { $reason }
word-score-problem-must-be-1-to-9 = must be 1 to 9
word-score-problem-must-be-more-than-0 = must be more than 0
word-score-problem-low-is-above-high = low is above high
word-score-problem-must-not-exceed-100 = must not exceed 100
error-model-link-cycle = course { $course } links { $a } and { $b }, which closes a loop; links must form chains
word-score-problem-must-be-below-the-good-pace = must be below the good pace

## Score courses (ADR 0067)

word-what-link = link
error-edit-score-course = course { $course } is a score course; this change works on individual courses only
error-edit-not-score-course = course { $course } is not a score course
error-edit-value-disagrees = control { $code } carries the value { $expected } in its code, not { $value }; change the code instead, and the value follows
error-edit-code-carries-no-value = the code of control { $code } carries no value; give one
error-edit-duplicate-score-control = control { $code } appears twice in the score course
error-edit-bad-time-limit = the time limit must be at least 1 minute
error-edit-value-max-below-control = The highest value cannot be { $value_max }: control { $code } is worth { $value }. Recode or remove that control first.
error-score-unreachable = no route from ({ $from_x }, { $from_y }) to ({ $to_x }, { $to_y }) on this map
error-score-start-unreachable = the start at ({ $x }, { $y }) cannot be reached: it lies in a blocked area
error-score-finish-unreachable = the finish at ({ $x }, { $y }) cannot be reached: it lies in a blocked area
error-score-map-empty = the map has no objects, so there is nothing to search routes on
error-score-bad-route-setting = route setting { $name } is out of range: { $value }
error-score-grid-too-large = the map would need { $cells } raster cells, more than the limit of { $limit }
error-score-code-space-exhausted = only { $limit } free codes are left for the controls asked for
error-score-value-space-exhausted = only { $limit } free codes carry the value { $value }; more controls of that value were asked for
error-score-bad-value = a point value must lie from 1 to { $value_max }, not { $value }
error-score-positions-mismatch = { $values } values but { $coords } positions were given
error-score-link-does-not-fit = the link between controls { $a } and { $b } does not fit the time limit
error-score-bad-link = the link between controls { $a } and { $b } names a control that does not exist or the same control twice
error-score-link-end-not-candidate = an end of the requested link { $link } lies on no candidate
error-score-links-not-a-chain = the links at control { $control } do not form a chain: a control may have at most two links and they may not close a ring
error-score-link-tour-not-found = no tour was found that holds every link within the time limit; the link between controls { $a } and { $b } fits worst. The search is not exhaustive for larger courses, so this is no proof
mcp-error-no-score-settings-change = nothing to change; give at least one of value_max, good_pace_s_per_km, very_good_pace_s_per_km, mix_hard_percent, seed, value1_min_m, min_gap_m, max_reach_m, auto_reach and high_value_gap_m
mcp-error-no-score-course-change = nothing to change; give time_limit_min, penalty_per_min or no_penalty
mcp-error-score-code-empty = a control of a score course needs a code
mcp-tool-add-score-course = Add a score course without controls, from a start, a finish and a time limit in minutes. Without a penalty the course has no stored penalty and uses the default: the highest control value of the course, at least value_max, so a minute over never pays off. Returns name, time_limit_min, penalty_per_min and penalty_is_default. Add the controls with set_score_controls.
mcp-tool-set-score-controls = Replace the controls of a score course, in any order. Each entry has a code and may have a value. Without a value the code gives it: the second digit of a two-digit code, the last digit of a longer one. A value that differs from the code is refused, and so is a value outside 1 to value_max. A control needs a code and may appear once, and a control that a link names cannot be dropped. A default penalty follows the controls on its own. Returns name, controls with code and value, penalty_per_min and penalty_is_default.
mcp-tool-set-control-value = Set the value of one control of a score course. The code carries the value, so only the value of the code is accepted; to give a control another value, change its code with set_control_code, and the value follows. The call repairs a stored value that no longer matches its code. Returns name, code and value.
mcp-tool-add-score-link = Require two controls of a score course to be run directly one after the other, in either direction. A runner who does not is disqualified. Both controls must be in the course, a control takes at most two links, and links may not close a ring. Returns name, a, b and links, all links of the course.
mcp-tool-remove-score-link = Remove the link between two controls of a score course; either order matches. Returns name, a, b and links, the links that remain.
mcp-tool-set-score-settings = Change the score settings that all score courses of the event share: value_max (highest point value, 1 to 9), good_pace_s_per_km and very_good_pace_s_per_km (seconds per kilometre), mix_hard_percent (share of hard controls), seed (fixes the draft search), value1_min_m (nearest distance of any control to start and finish, in metres), min_gap_m (least distance between two controls, at least 100 m), max_reach_m (farthest distance from the start; auto_reach returns it to a quarter of the ground the good runner covers in the time limit) and high_value_gap_m (least distance between two high-valued controls). Fields not given stay as they are. Returns all settings.
mcp-tool-set-score-course-details = Change the time limit in minutes or the penalty per started minute of a score course. penalty_per_min stores a penalty; no_penalty goes back to the default, the highest control value of the course, at least value_max. Returns name, time_limit_min, penalty_per_min and penalty_is_default.
mcp-param-time-limit-min = Time limit in minutes, at least 1
mcp-param-penalty-per-min = Points deducted per started minute over the limit [default: the highest control value of the course, at least value_max]
mcp-param-set-score-course-details-time-limit-min = Time limit in minutes, at least 1 [default: unchanged]
mcp-param-set-score-course-details-penalty-per-min = Points deducted per started minute over the limit [default: unchanged]
mcp-param-set-score-course-details-no-penalty = Go back to the default penalty, the highest control value of the course, at least value_max [default: false]
mcp-param-set-score-controls-controls = The controls of the course, each an object with code and an optional value
mcp-param-set-score-controls-value = Points of the control [default: the value the code carries]
mcp-param-value = Points of the control, from 1 to the highest value of the event
mcp-param-a = One control of the link, a code
mcp-param-b = The other control of the link, a code
mcp-param-value-max = Highest point value, 1 to 9
mcp-param-good-pace-s-per-km = Pace of a good runner in seconds per kilometre
mcp-param-very-good-pace-s-per-km = Pace of a very good runner in seconds per kilometre, below the pace of a good runner
mcp-param-mix-hard-percent = Share of hard controls in the mix, 0 to 100 percent
mcp-param-seed = Seed of the draft search; the same seed gives the same draft
cli-help-score = Edit score courses: controls with values, mandatory links, settings
cli-help-score-controls = Replace the controls of a score course. Each control is a code, optionally followed by = and its value; without a value the code gives it
cli-help-score-value = Set the value of one control of a score course. The code carries the value, so only that value is accepted; change the code to change the value
cli-help-score-link = Add or remove mandatory links of a score course
cli-help-score-link-add = Require two controls to be run directly one after the other, in either direction
cli-help-score-link-remove = Remove the link between two controls
cli-help-score-settings = Change the score settings of the event
cli-help-score-course = Change a score course
cli-help-score-course-set = Change the time limit or the penalty of a score course; --no-penalty goes back to the default penalty
cli-arg-score-controls-codes = Controls separated by commas, for example 31,42 or 31=1,42=2
cli-arg-score-value-value = Points of the control
cli-arg-score-json = Print the answer as JSON, the same as the MCP tool gives
cli-arg-score-link-a = One control of the link
cli-arg-score-link-b = The other control of the link
cli-arg-score-settings-value-max = Highest point value, 1 to 9
cli-arg-score-settings-good-pace = Pace of a good runner, in seconds per kilometre or as m:ss
cli-arg-score-settings-very-good-pace = Pace of a very good runner, in seconds per kilometre or as m:ss
cli-arg-score-settings-mix-hard = Share of hard controls in the mix, 0 to 100 percent
cli-arg-score-settings-seed = Seed of the draft search
cli-arg-score-settings-value1-min = Nearest distance in metres of any control to start and finish, so value 1 starts there
cli-arg-score-settings-min-gap = Least distance in metres between two controls, at least 100
cli-arg-score-settings-max-reach = Farthest distance in metres of a control from the start
cli-arg-score-settings-auto-reach = Reach from the time limit: a quarter of the ground the good runner covers
cli-arg-score-settings-high-value-gap = Least distance in metres between two high-valued controls
cli-score-reach-automatic = automatic
mcp-param-value1-min-m = Nearest distance in metres of any control to start and finish
mcp-param-min-gap-m = Least distance in metres between two controls, at least 100
mcp-param-max-reach-m = Farthest distance in metres of a control from the start
mcp-param-auto-reach = true returns the reach to automatic: a quarter of the ground the good runner covers in the time limit
mcp-param-high-value-gap-m = Least distance in metres between two high-valued controls (the upper 40 percent of the values)
editor-score-value1-min = Value 1 from (m)
editor-score-min-gap = Controls apart (m)
editor-score-max-reach = Reach from start (m, empty: automatic)
editor-score-high-value-gap = High values apart (m)
cli-arg-score-course-set-time-limit = Time limit in minutes
cli-arg-score-course-set-penalty = Points deducted per started minute over the limit
cli-arg-score-course-set-no-penalty = Go back to the default penalty, the highest control value of the course, at least the highest value of the event
cli-arg-course-add-kind = Kind of course, individual or score
cli-arg-course-add-time-limit = Time limit in minutes; required for a score course
cli-arg-course-add-json = Print the answer as JSON, the same as the MCP tool gives; only with --kind score
cli-arg-course-add-penalty = Points deducted per started minute over the limit [default: the highest control value of the course, at least the highest value of the event]
cli-score-course-added = added score course { $name }: { $limit } minutes, { $penalty } points per minute over the limit{ $default ->
    [yes] { " " }(the default)
   *[no] { "" }
}
cli-score-controls = { $name }: { $count } controls, { $total } points ({ $list })
cli-score-value = control { $code } of { $name } is worth { $value } points
cli-score-link-added = linked { $a } and { $b } in { $name }
cli-score-link-removed = removed the link between { $a } and { $b } in { $name }
cli-score-settings = score settings: highest value { $value_max }, good runner { $good } s/km, very good runner { $very_good } s/km, { $mix_hard } percent hard controls, seed { $seed }, value 1 from { $value1_min } m, controls { $min_gap } m apart, high values { $high_gap } m apart, reach { $reach }
cli-score-course-set = { $name }: { $limit } minutes, { $penalty } points per minute over the limit{ $default ->
    [yes] { " " }(the default)
   *[no] { "" }
}
cli-score-bad-pace = { $value } is not a pace; give seconds per kilometre, such as 250, or minutes and seconds, such as 4:10
cli-score-bad-control = { $value } is not a control with a value; write the code, or the code, = and the value, such as 31 or 31=1
cli-course-add-score-only = --time-limit, --penalty and --json belong to --kind score
cli-course-add-needs-time-limit = a score course needs --time-limit
cli-course-add-individual-only = --controls, --classes, --climb and --print-scale do not apply to --kind score; set the controls with score controls and the classes with course set
editor-op-add-score-course = Add score course
editor-op-set-score-controls = Change score controls
editor-op-set-control-value = Set control value
editor-op-add-score-link = Add link
editor-op-remove-score-link = Remove link
editor-op-set-score-settings = Change score settings
editor-op-set-score-course-details = Change score course details
score-miss-good-below = The good runner scores { $percent } percent of all points, { $miss } percentage points below the band of { $low } to { $high } percent. The set is too long for the limit.
score-miss-good-above = The good runner scores { $percent } percent of all points, { $miss } percentage points above the band of { $low } to { $high } percent. The set is too short for the limit.
score-miss-very-good-misses = The very good runner cannot take every control within the limit. { $missing } points, { $miss } percent of all points, stay out of reach.
score-miss-very-good-over = The very good runner needs { $percent } percent of the limit for every control, { $miss } percentage points too much.
score-miss-very-good-early = The very good runner takes every control and finishes after { $percent } percent of the limit, { $miss } percentage points before the band of { $low } to { $high } percent.
score-draft-best = The closest attempt has { $count } controls worth { $points } points.
score-draft-hint-limit-up = A longer time limit would help.
score-draft-hint-limit-down = A shorter time limit would help.
score-draft-hint-more-candidates = The map offers too few suitable sites for this limit. More map features, or a shorter limit, would help.
score-draft-hint-code-space = The map has enough sites, but the free codes run out first. Raise the highest value in the score settings (9 at most), or free codes on the map.
score-draft-hint-faster-profile = The good runner scores too few points. A faster pace for the good runner in the score settings would help.
score-draft-hint-impossible = No change of limit or pace helps with these inputs. Check the start, the finish and the links.
mcp-error-score-no-start = the event has no start; add one with add_special first
mcp-error-score-no-finish = the event has no finish; add one with add_special first
mcp-error-score-bad-distance = { $value } is not a distance; give map millimetres, zero or more
mcp-error-score-draft-ends = the score course { $name } has its own start and finish; leave start and finish out or name the ones it has
mcp-tool-score-suggest = Suggest control sites for a score course from the map, ranked by route choice. Each candidate has a position in map millimetres, the symbol code of its feature, route_choice and difficulty from 0 to 1, effort_s (seconds from the start for the good runner) and the suggested value. Writes nothing. Takes seconds on a large map. Start and finish default to those of the named course, else to the first of each in the event; ends that lie in a blocked area are refused.
mcp-tool-score-draft = Draft a score course: a set of controls with values and codes so that the good runner scores about 90 percent of all points within the time limit and the very good runner takes every control and finishes just under the limit. Writes nothing unless apply is true; then it creates the score course or replaces the controls of an existing score course, and a course of another kind is refused. links names pairs of positions, each taken from score_suggest, to be run one directly after the other. A draft that misses a band comes back as an error naming each miss with its size, and what would help. Takes seconds.
mcp-tool-score-length = Analyse the length of an existing score course: the best round trips of the good and the very good runner within the limit, the share of points and of the limit, and each miss of the bands with its size. Reports only; never changes the course.
mcp-param-score-suggest-limit = Most candidates to return [default: 60, at most 200]
mcp-param-score-suggest-min-distance-mm = Smallest distance in map millimetres between candidates, controls, start and finish [default: 15 m on the ground at 1:4000 and larger scales, else 30 m]
mcp-param-score-suggest-course = Name of a score course. Its start and finish are used; a start or finish id given as well wins
mcp-param-score-suggest-start = Id of the start special object [default: the first start]
mcp-param-score-suggest-finish = Id of the finish special object [default: the first finish]
mcp-param-score-draft-start = Id of the start special object of a new course [default: the first start]
mcp-param-score-draft-finish = Id of the finish special object of a new course [default: the first finish]
mcp-param-score-draft-links = Pairs of positions to run one directly after the other, each end the position of a candidate from score_suggest
mcp-param-score-draft-a = One end of the link, a position in map millimetres
mcp-param-score-draft-b = The other end of the link, a position in map millimetres
mcp-param-score-draft-apply = Write the draft as the score course [default: false, the draft is only returned]
editor-op-score-draft = Draft score course
editor-op-score-apply-draft = Apply score draft
editor-score-heading = Score course
editor-score-limit = Time limit (minutes)
editor-score-penalty = Penalty per minute over
editor-score-penalty-default = Default penalty
editor-score-controls = Controls and values
editor-score-col-code = Code
editor-score-col-value = Value
editor-score-col-choice = Route choice
editor-score-col-difficulty = Difficulty
editor-score-col-effort = From start (s)
editor-score-links = Mandatory links
editor-score-links-none = No links.
editor-score-link-hint = A link is drawn as a dashed line between its two controls.
editor-score-link-remove = Remove link
editor-score-settings = Runners and mix (all score courses)
editor-score-value-max = Highest value
editor-score-good-pace = Good runner pace (s per km)
editor-score-very-good-pace = Very good runner pace (s per km)
editor-score-mix-hard = Hard controls (percent)
editor-score-seed = Seed of the draft
editor-score-good-band = Good runner band (percent of all points)
editor-score-very-good-band = Very good runner band (finish, percent of the limit)
editor-score-suggest = Suggest controls
editor-score-draft = Draft course
editor-score-length-run = Check length
editor-score-apply = Apply draft
editor-score-discard = Discard
editor-score-accept = Accept
editor-score-working = Working. This can take several seconds.
editor-score-candidates = Candidates
editor-score-candidates-hint = Candidates and drafts are drawn as dashed circles with their value. They are not saved until you accept or apply them.
editor-score-no-code-hint = Some candidates have no free code, because the codes for their value are used up. Raise the highest value or remove controls.
editor-score-no-code = No free code
editor-score-draft-result = Draft
editor-score-draft-hard = Hard controls (actual / wanted)
editor-score-length-heading = Length report
editor-score-good = Good runner, points
editor-score-good-share = Good runner, share of all points
editor-score-very-good = Very good runner, points
editor-score-very-good-share = Very good runner, finish (percent of the limit)
editor-score-bands-ok = Both runners are inside the bands.
editor-score-bands-missed = A band is missed.
editor-score-discarded = The project changed outside the editor. Candidates and the draft are discarded.
editor-score-status-suggested = Candidates found.
editor-score-status-drafted = Draft ready. The course stays as it is until you apply it.
editor-score-status-applied = Draft applied.
editor-score-status-length = Length checked.
editor-score-status-accepted = Control added to the course.
editor-score-no-course-mode = Course mode edits individual courses. Use the Score panel for this course.
editor-tree-new-score-course = New score course...
editor-status-score-created = Score course created.
cli-help-score-suggest = Suggest control sites for a score course, ranked by route choice. Writes nothing. Takes seconds on a large map. With --json the answer is the one of the MCP tool score_suggest
cli-help-score-draft = Draft a score course so that both runner profiles meet the length bands. Writes only with --apply. A draft that misses a band exits with 1 and names each miss. With --json the answer is the one of the MCP tool score_draft
cli-help-score-length = Analyse the length of a score course and report each miss of the bands. Changes nothing. With --json the answer is the one of the MCP tool score_length
cli-arg-score-suggest-limit = Most candidates, at most 200 [default: 60]
cli-arg-score-suggest-min-distance = Smallest distance between candidates in map millimetres [default: from the map scale]
cli-arg-score-suggest-course = Score course whose start and finish are used; --start and --finish win [default: the first start and finish]
cli-arg-score-suggest-start = Id of the start special object [default: the first start]
cli-arg-score-suggest-finish = Id of the finish special object [default: the first finish]
cli-arg-score-draft-time-limit = Time limit in minutes
cli-arg-score-draft-start = Id of the start special object of a new course [default: the first start]
cli-arg-score-draft-finish = Id of the finish special object of a new course [default: the first finish]
cli-arg-score-draft-link = Two positions of candidates, X,Y;X,Y in map millimetres, to run one directly after the other; repeat for more links
cli-arg-score-draft-apply = Write the draft as the score course; a course of that name is replaced if it is a score course
cli-score-suggest = { $count ->
    [one] 1 candidate
   *[other] { $count } candidates
}, best route choice first
cli-score-suggest-line = { $at }  { $symbol }  value { $value }, route choice { $choice }, difficulty { $difficulty }, { $effort } s from the start
cli-score-draft = draft of score course { $name }: { $count } controls, { $points } points; nothing written, use --apply to write it
cli-score-draft-applied = wrote score course { $name }: { $count } controls, { $points } points
cli-score-draft-control = { $code }  value { $value }  at { $at }
cli-score-length = { $name }: { $count } controls, time limit { $limit } minutes
cli-score-length-good = good runner: { $points } of { $all } points, { $percent } percent (band { $low } to { $high } percent)
cli-score-length-very-good = very good runner: { $points } points, finishes after { $percent } percent of the limit (band { $low } to { $high } percent)
cli-score-length-ok = both runners are inside the bands
cli-score-bad-link-ends = { $value } is not a link; write two positions as X,Y;X,Y
cli-score-bad-distance = { $value } is not a distance; give map millimetres, zero or more
mcp-error-interrupted = the operation was interrupted before it finished
error-score-unknown-course = course { $course } does not exist
error-score-not-a-score-course = course { $course } is not a score course
error-score-unknown-control = the course names control { $code }, which the event does not have
cli-arg-export-csv = Write the controls of a score course as CSV with code, value and position in map millimetres; needs --course. An existing file is replaced
cli-arg-export-sheet = Write the evaluation sheet of a score course as Markdown; needs --course. An existing file is replaced
cli-arg-export-course = The score course for --csv and --sheet
export-needs-course = --csv and --sheet need --course NAME, the score course to write
export-csv-written = wrote the control list of { $course } to { $path }
export-sheet-written = wrote the evaluation sheet of { $course } to { $path }
mcp-tool-export-score-csv = Write the controls of a score course as CSV. Takes project, course and path, the file to write, which ends in .csv; an existing file is replaced. The first row is the header code,value,x_mm,y_mm. Positions are map millimetres at map scale with y pointing down, written with a dot. Rows are ordered by code. Returns path, course and controls, the number of rows. A course that is not a score course is refused.
mcp-tool-export-score-sheet = Write the evaluation sheet of a score course as a Markdown file that prints well. Takes project, course and path, the file to write, which ends in .md; an existing file is replaced. The sheet names the time limit, the penalty per started minute, the rule that the faster runner wins on equal points, the mandatory links with the rule that a runner who does not run a link directly is disqualified, and every control with its value. Returns path and course. A course that is not a score course is refused.
mcp-param-export-score-csv-path = The file to write, an absolute path or one starting with ~, ending in .csv
mcp-param-export-score-sheet-path = The file to write, an absolute path or one starting with ~, ending in .md
mcp-param-export-score-csv-course = Name of the score course
mcp-param-export-score-sheet-course = Name of the score course
mcp-error-not-csv = { $path } does not end in .csv; name a .csv file
mcp-error-not-md = { $path } does not end in .md; name a .md file
score-sheet-title = Evaluation sheet for { $course }
score-sheet-event = Event: { $title }
score-sheet-time-limit = Time limit: { $minutes ->
    [one] 1 minute
   *[other] { $minutes } minutes
}
score-sheet-penalty = Penalty: { $points ->
    [one] 1 point
   *[other] { $points } points
} for each started minute over the limit
score-sheet-penalty-default = Penalty: { $points ->
    [one] 1 point
   *[other] { $points } points
} for each started minute over the limit. The course sets no penalty, so this is the default.
score-sheet-tie = Equal points: the faster runner wins.
score-sheet-total = Points in all: { $points } on { $count ->
    [one] 1 control
   *[other] { $count } controls
}
score-sheet-links-heading = Mandatory links
score-sheet-no-links = This course has no mandatory links.
score-sheet-links-rule = A link is two controls that a runner visits directly one after the other, in either order. A runner who does not run a link directly is disqualified.
score-sheet-col-first = First control
score-sheet-col-second = Second control
score-sheet-controls-heading = Controls
score-sheet-col-code = Code
score-sheet-col-value = Value
export-course-unused = --course applies only with --csv or --sheet; --iof-xml and --ppen take no course

## update (ADR 0076)

cli-warning-line = warning: { $text }
cli-help-update = Install a newer freemapper-courseplan from the GitHub releases, after asking. A build from source says how to update it instead
cli-help-update-exit-codes =
    Exit code 0: done, or this version is current.
    Exit code 1: the update failed.
    Exit code 2: wrong call, for example an unknown option.
    Exit code 10: with --check, a newer version exists.
cli-arg-update-check = Only check. Exit code 10 when a newer version exists, 0 when this one is current
cli-arg-update-yes = Install without asking. Needed when the input is not a terminal
cli-arg-update-version = Install this version (x.y.z) instead of the newest one, also an older one
cli-arg-update-rollback = Go back to the version the last update replaced
cli-arg-update-mode = Store what the daily check does when it finds a newer version (default notify). Works in every build, because it only writes the setting
update-source-build = This freemapper-courseplan was built from source. Update it with git pull and cargo build --release --locked.
update-current = freemapper-courseplan { $version } is up to date.
update-available = freemapper-courseplan { $new } is available (you have { $old }), released { $released }.
update-details =
    Release notes: { $notes }
    Download: { $size } MB
update-decimal-separator = .
update-ask = Install it now? [y/N]
update-not-installed = Nothing was installed.
update-needs-yes = the input is not a terminal, so freemapper-courseplan cannot ask before installing; run freemapper-courseplan update --yes to install without asking
update-installing = downloading and checking freemapper-courseplan { $version }
update-done = freemapper-courseplan updated from { $old } to { $new }. Release notes: { $notes }
update-mcp-restart = A running MCP server keeps the old version until the AI assistant restarts it.
update-rolled-back = Went back to freemapper-courseplan { $version }. The version you left is kept as { $path }.
update-not-writable = cannot write to the program folder { $path }, and freemapper-courseplan does not ask for administrator rights. Unpack the new release there by hand, or move freemapper-courseplan to a folder you can write to.
update-too-old = freemapper-courseplan { $new } cannot be installed by the self-update of { $old }; install { $needed } or newer by hand, see { $notes }
update-busy = another update or rollback is running in the program folder (lock file { $path }); wait for it to finish and try again
update-bad-signature = the release could not be verified ({ $error }); nothing was installed
update-check-bad-signature = the newest freemapper-courseplan release could not be verified ({ $error }); nothing was installed
update-no-manifest = the newest release cannot install itself (it has no update manifest); download it from { $page }
update-no-manifest-version = release { $version } does not exist or cannot install itself (it has no update manifest); see { $page }
update-no-asset = the release has no program for this system ({ $platform }); nothing was installed
update-no-previous = there is no previous version to go back to ({ $path } is missing)
update-failed = the update failed: { $error }
update-mode-set = Update mode set to { $mode } in { $path }.
update-mode-env = FREEMAPPER_COURSEPLAN_UPDATE={ $value } overrides this setting while it is set.
update-mode-failed = could not write { $path }: { $error }
update-notice = freemapper-courseplan { $new } is available (you have { $old }). Update with: freemapper-courseplan update
update-applies-next-start = The new version applies from the next start of freemapper-courseplan.
update-install-failed = freemapper-courseplan { $new } was not installed: { $error }
update-no-config-dir = no configuration folder on this system; set FREEMAPPER_COURSEPLAN_CONFIG_DIR
update-mcp-note = Note for the person: freemapper-courseplan { $new } is available (this is { $old }). Ask them before calling install_update.
update-mcp-needs-confirm = nothing was installed; ask the person first and call install_update with confirm: true once they agree
update-mcp-note-manual = Note for the person: freemapper-courseplan { $new } is available (this is { $old }), but install_update cannot install it. Install { $needed } or newer by hand, see { $notes }
update-mcp-note-bad-signature = Note for the person: the newest freemapper-courseplan release could not be verified ({ $error }); nothing was installed. Tell them, and do not install it some other way.
update-mcp-restart-now = Restart the freemapper-courseplan MCP server now; it still runs the old version.
mcp-tool-check-update = Whether a newer freemapper-courseplan release exists, with its version, release date, release notes and download size, or that this one is up to date. Installs nothing.
mcp-tool-install-update = Install the newest freemapper-courseplan release over this program. Call it only after the person agreed, with confirm true. The new version applies once the assistant restarts the freemapper-courseplan server.
mcp-param-install-update-confirm = true once the person agreed to install the newer version; without it nothing is installed.
update-editor-notice = freemapper-courseplan { $version } is available (you have { $current }).
update-editor-howto = Update with: freemapper-courseplan update
update-editor-notes = Release notes
