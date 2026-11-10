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
error-edit-outside-map = point ({ $x }, { $y }) is outside the map, which spans ({ $min_x }, { $min_y }) to ({ $max_x }, { $max_y })
error-edit-in-use = { $id } is still used by { $courses }; change those first
error-edit-not-supported = course { $course } is not an individual course; only individual courses can be edited yet
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
error-edit-blank-rule-category = the rule category of class { $name } is empty; give a category or clear it

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
check-skip-print-scale = no rule of this kind for print scale 1:{ $scale }
check-skip-course-kind = { $kind ->
        [team] team courses
        [relay] relay courses
        [score] score courses
       *[other] { $kind } courses
    } are not checked yet
check-skip-course-unresolved = the course refers to { $id }, which the event does not define; it is not checked
check-skip-class-not-mapped = class { $name } has no rule category; class set --rule-category assigns one
check-skip-climb-missing = the course has no climb
check-skip-value-missing = the rule has no value { $name }
check-skip-no-map-extent = the map has no objects, so it has no edge
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
check-leg-out-of-bounds = the straight leg crosses out-of-bounds area { $id }, so the best route may lead through it
check-leg-out-of-bounds-symbol = the straight leg crosses map symbol { $code } (out of bounds), so the best route may lead through it
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
check-source-claude = proposal of freemapper-courseplan

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
cli-edit-start-leg-cleared = course { $name }: cleared the bends and taped flag of the leg from the start, which now runs to another stop

cli-help-course = Add, change, number and remove individual courses
cli-help-course-add = Add an individual course
cli-help-course-controls = Replace the controls of a course
cli-help-course-remove = Remove a course and its name from the special objects
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
cli-help-event-start-window = Set the start window, the minutes from the first to the last start. The start capacity check divides the expected starters by it
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
cli-arg-export-iof-xml = Write the individual courses as IOF XML 3.0 CourseData for result software such as OE; an existing file is replaced
export-iof-xml-written = { $count ->
    [one] wrote 1 course to { $path }
   *[other] wrote { $count } courses to { $path }
}
export-iof-xml-skipped = left out { $course }: { $kind } courses are not exported to IOF XML
export-iof-xml-no-geo = the map has no usable georeferencing, so the controls carry no latitude and longitude, only map positions
export-nothing-to-do = nothing to export; name a format, for example --iof-xml FILE or --ppen FILE
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

## MCP server (tool names, parameter names and JSON fields stay English)

cli-help-mcp = Serve course setting to AI assistants over MCP on stdin and stdout

mcp-instructions =
    freemapper-courseplan sets orienteering courses on an OOM map: controls, start and finish, classes, courses and
    the control numbers. Every tool takes project, the project file of the event, as an absolute path or one
    starting with ~. init_event makes a project file from a map. Positions are in map millimetres at map scale,
    with y pointing down, written as objects with the numbers x and y. Workflow: event_info, then map_extent and
    map_features_near to find positions, then add_special for the start and the finish, add_control for each
    control, add_course, place_numbers and event_info again to read the length. Each call saves its change. A
    refused change leaves the file as it was. Only individual courses can be edited, measured and checked yet.
    check tests the courses against the rule profile of the event, which set_rule_profile changes. Some checks
    need the start window (set_start_window) or the rule category and expected starters of a class (set_class);
    without them check lists the rule as skipped. A finding the course setter wants as it is gets accept_finding
    with a note; remove_accepted_finding takes that back.
mcp-error-relative-path = { $path } is a relative path; use an absolute path or one starting with ~
mcp-error-not-toml = { $path } is not a project file; project files end in .toml
error-import-not-map = { $path } is not an OOM map; maps end in .omap or .xmap
mcp-error-not-map = { $path } is not an OOM map; maps end in .omap or .xmap
mcp-error-not-xml = { $path } is not an XML file; the export file ends in .xml
mcp-error-project-parse = { $path } is not a valid project file; the first problem is at line { $line }, column { $column }
mcp-error-project-parse-nopos = { $path } is not a valid project file
mcp-error-map-invalid = { $path } is not a valid OOM map
mcp-error-no-home = this system has no home folder; use an absolute path
mcp-error-kind-needs-at = a { $kind } is a single point; give at and leave out points
mcp-error-kind-needs-points = a { $kind } is drawn through points; give points and leave out at
mcp-error-no-courses-for-kind = a { $kind } applies to every course; leave out courses
mcp-error-leg-target = give either from_start or after, not both and not neither
mcp-error-no-change = nothing to change; give at least one of classes, climb_m, no_climb, print_scale and no_print_scale
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
mcp-tool-set-course-controls = Replace the controls of a course. The n-th visit of a code keeps the number placement of the n-th visit before. A leg keeps its bends and marked-route flag only while it runs to the same next stop; the result lists the cleared legs in cleared_legs as course and after (0 is the leg from the start).
mcp-tool-set-leg = Set the bends and the marked-route flag of one leg of an individual course: the leg from the start (from_start) or the leg after a position (after, counted from 1 as on the map, crossing points included). Without bends the leg loses its bends.
mcp-tool-set-course-details = Change the classes, the climb or the planned print scale of a course. Parameters you leave out stay as they are.
mcp-tool-place-numbers = Place the control numbers of a course behind the circle as seen from the incoming leg, clear of legs, circles, other numbers and point and line symbols of the map. Returns how many numbers it placed. It runs only when called, so call it again after changing legs. A control with a central number (set_control_number) keeps it, and course numbers (set_course_number) stay unless overwrite is true. The number moves outward up to the maximum gap of the rule profile before it turns away from the preferred direction.
mcp-tool-set-course-number = Set or clear the number placement of one control in an individual course, at a position counted from 1 as on the map (crossing points included). It wins over the central placement of set_control_number and over place_numbers. Direction in degrees (0 = east, counter-clockwise), gap between circle line and number in print mm, more than 0 and at most 10. clear removes it, so the central or automatic placement applies again.
mcp-tool-map-features-near = Map objects within radius_mm of a position, nearest first: symbol code, geometry kind (point, line, area), distance in mm and the part of the map.
mcp-tool-map-extent = Smallest and largest corner of the map objects in map mm, the map scale and the map kind.
mcp-tool-init-event = Start an event from an OOM map (sprint or schoolyard) and write its project file. Returns the path of the project file.
mcp-tool-check = Check the individual courses against the rule profile and overlays of the event. Returns counts in summary; findings with rule, profile, severity (error, warning or hint), location as an object with kind, message_id, args, source with doc, section and title, rule_note, and text; accepted findings with the note of the course setter; unmatched, the accepted findings that match no finding any more, for example because a control or course is gone (remove them with remove_accepted_finding if they are no longer needed; they never count as errors); and skipped rules with the reason, including rules an overlay switched off. A leg location carries course, after and the ids of its ends in from and to. Team, relay and score courses are not checked yet.
mcp-tool-accept-finding = Accept a finding as an intended exception, with a note. Name it as check reports it: the rule and its place, given as controls, legs or the course. A start or finish id may stand in controls, as check reports it. A leg is given by course and after; the server stores the ids of its ends from the course as it is now, so the finding comes back when the leg later runs between other stops. Leave out all three for a finding of the whole event. Accepting the same finding again replaces the note. check then lists it under accepted.
mcp-tool-remove-accepted-finding = Remove an accepted finding, so check reports it again. Name it with the rule and the same place as when it was accepted. A leg is given by course and after. The entry for the leg as it runs now is removed first; if there is none, every entry at that position goes, so an entry from unmatched can be removed after its leg or course changed. Returns the removed entries.
mcp-tool-set-rule-profile = Set the base rule profile of the event and its overlays. Both are loaded first; an unknown profile, or an overlay given as base, is refused with the list of available profiles. Without overlays the event has none.
mcp-tool-set-start-window = Set the start window of the event, the minutes from the first to the last start, at least 1. The start capacity check divides the expected starters by it.
mcp-tool-export-iof-xml = Write the individual courses of the event as IOF XML 3.0 CourseData for result software such as OE. Takes project and path, the file to write, which ends in .xml; an existing file is replaced. Returns path, courses, the number of courses written, and skipped, the team, relay and score courses left out, each with name and kind.
mcp-tool-export-ppen = Write the individual courses of the event into a Purple Pen event (.ppen). Takes project, path (the .ppen file) and new. The export changes an existing file in place and keeps everything the project did not change. It creates a missing file, and with new it writes the file from scratch; either way a project without sources.ppen is linked to the file. The export never deletes: objects in the file that the project lacks stay and are reported. The project's Purple Pen ids count only for the file that sources.ppen names, and only for that file does the export store the ids it assigns in the project. Returns path, courses (the number of courses written), skipped (the team, relay and score courses left out, each with name and kind), notes (the texts on what was left out, changed or kept), patched (true when an existing file was changed in place) and ids_stored (how many objects of the project got a Purple Pen id for the first time).

mcp-param-project = Project file of the event, an absolute path or one starting with ~, for example ~/maps/hall.courseplan.toml
mcp-param-export-iof-xml-path = The file to write, an absolute path or one starting with ~, ending in .xml
mcp-param-export-ppen-path = The Purple Pen event to write, an absolute path or one starting with ~, ending in .ppen; it is created when missing
mcp-param-export-ppen-new = Write the file from scratch and replace an existing one [default: false]
mcp-param-init-event-project = Where to write the project file, an absolute path or one starting with ~ [default: next to the map, named after it with .courseplan.toml]
mcp-param-map = The OOM map (sprint or schoolyard), an absolute path or one starting with ~
mcp-param-title = Event title [default: the title in freemapper.toml next to the map, else the map name]
mcp-param-code = Code of the control, a string or a whole number
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
