sw(1)

# NAME

sw - terminal stopwatch

# SYNOPSIS

*sw* [_OPTIONS_]

# DESCRIPTION

*sw* is a terminal stopwatch that runs as a shell.

# OPTIONS

*-v*, *--no-visual-cues*
	Disable text-based graphics and visual cues.

	To toggle this value at runtime, use the _Visuals_ command.

	For more details, see the section on visual cues.

*-c*, *--no-colors*
	Disable the use of colors in output.

	For more details, see the section on colors.

*--no-tty-check*
	Disable the TTY check.

	Typically, *sw* will check if standard output and input are terminals,
	and exit abnormally if either aren't. This flag disables the check.

*-V*, *--version*
	Display the version of *sw*.

*--help*
	Display usage information.

# COMMANDS

The shell interface allows you to manipulate a stopwatch by issuing
commands and responding to prompts that may follow.

The following table is a quick overview of the available commands. The "input"
field shows which key combination maps to the command. The command's name is
also a valid input.

[[ Command name
:- Input
:< Description
|[ Help
:  h
:  Show help
|  Display
:  <Enter>
:  Display elapsed time
|  Toggle
:  s
:  Toggle stopwatch
|  Reset
:  r
:  Reset stopwatch
|  Change
:  c
:  Change elapsed time
|  Offset
:  o
:  Offset elapsed time
|  Name
:  n
:  Set stopwatch name
|  Precision
:  p
:  Set display precision
|  Visuals
:  v
:  Toggle visual cues
|  License
:  l
:  Print license info
|  Quit
:  q
:  Abandon all Data

## HELP

Lists the available commands and their description.

Each line of the output corresponds to a command, and contains the required key
combination and a description of the command.

## DISPLAY

Displays the state of the stopwatch.

The first line of output contains the elapsed time. The subsecond precision of
this output is affected by the _Precision_ command.

The second line indicates whether the stopwatch is running or stopped. The
colors used to indicate this are an exception to their typical meaning. For more
information, see the the section on colors.

## TOGGLE

Toggles whether the stopwatch is running or stopped.

If the stopwatch is running, it is stopped. If the new elapsed time overflows,
it prints a warning.

If stopped, it starts the stopwatch. Then it prints the duration that has
elapsed since the last time it was stopped. This may be useful if you forget to
start the stopwatch, since you can start it and it'll tell you how long the
stopwatch was idling. The precision of this output is affected by the
_Precision_ command.

## RESET

Resets the stopwatch.

This command stops the stopwatch and sets its elapsed time to zero.

## CHANGE

Stops and sets the elapsed time.

This command prompts you for a new elapsed time.

The duration will be parsed according to the duration formats. It must be
positive. If the duration is invalid, it will be printed back with the errors
colored red.

## OFFSET

Offsets the elapsed time by a positive or negative duration.

This command prompts you for a positive or negative duration. If it parses
according to the duration formats, the elapsed time will be offset by it without
changing whether the stopwatch is running. Otherwise, the invalid duration will
be printed with errors indicated in red.

## NAME

Sets the name of the stopwatch.

This command will prompt you to name the stopwatch.

The name is purely for display: it's literal text that prefixes the prompt. It
may be useful for distinguishing the meaning of the stopwatch if you have
multiple instances open.

## PRECISION

Sets the decimal precision used to display the subsecond component of durations.

Several commands output formatted elapsed times and durations. This command is
used to configure the decimal precision of this output.

The default precision is 2, meaning two subsecond digits will be displayed.

## VISUALS

Toggles whether text-based graphics and visual cues are enabled (they are by
default).

This parameter can be controlled with the *--no-visual-cues* flag.

For more details, see the section on visual cues.

## LICENSE

Prints license and copyright information for *sw* and its direct dependencies.

## QUIT

Terminates the program with a successful exit code, abandoning all stopwatch
state.

# PROMPT

*sw* displays a prompt before commands are entered.

The prompt contains the name of the stopwatch (see the _Name_ command). If
visual cues are enabled, the name is followed by a character indicating the
stopwatch state. This indicator is ';' if the stopwatch is stopped and '\*' if
it's running.

For example, if the stopwatch is named "work" and stopped, it will prompt with
the following with visual cues enabled:

```
work ; 
```

And if it's running:

```
work * 
```

With the same stopwatch but visual cues disabled, it will prompt with this:

```
work. 
```

# DURATION INPUT FORMAT

The _Offset_ and _Change_ commands accept a duration as input. The following
formats are supported.

## STOPWATCH FORMAT

```
number unit
```

_number_ is a whole or decimal number, and _unit_ is one of "s", "m", or "h",
meaning seconds, minutes, and hours respectively.

Whitespace is ignored, so "1s" is just as valid as "1 s".

## UNIT FORMAT

```
sign hours : minutes : seconds . subseconds
```

The details shouldn't be surprising, it's a superset of how durations are
displayed.

_hours_, _minutes_, _seconds_ and _subseconds_ are all integers. _minutes_ and
_seconds_ must be less than 60.

_sign_ is optional, and one of "+" or "-". It indicates whether the duration is
positive or negative, defaulting to positive.

It's okay to omit separators and values. Rightmost values are the most
important, so the meaning of the input will be inferred from right to left.

Some examples of terse inputs:

- ":5" and "::5" represent 5 seconds
- ":5:" represents 5 minutes
- ":.6" represents 0.6 seconds
- "1::1.1" represents 1 hour and 1.1 seconds

It's also okay to add whitespace between separators.

# DURATION OUTPUT FORMAT

With visual cues enabled, durations are formatted as
_hours:minutes:seconds.subseconds_. Otherwise they're formatted as they would be
spoken. For example, "00:05:32.98" would be formatted as "5 minutes, 32.98
seconds". If the precision is zero, this will simply be "5 minutes, 32 seconds".

# COLORS

*sw* will use colors if they are supported by the terminal, or as directed by
the *--no-colors* flag.

The following table documents the meaning of each color as it appears:

[[ Color
:< Meaning
|  Magenta
:  Information about program state change
|  Cyan
:  Information, but state is unchanged
|  Yellow
:  Warning
|  Red
:  Error

The indication for whether the stopwatch is running or stopped in the _Display_
command is an exception to this, with "running" colored green and "stopped"
colored yellow.

# VISUAL CUES

Visual cues and text-based graphics are enabled by default, but can be disabled
using the _Visuals_ command and the *--no-visual-cues* flag.

The following aspects of the interface are affected by this parameter:
- Prompt
- Duration parsing error messages
- Duration output format

# AUTHORS

Maintained by Ula Shipman <ula.hello@mailbox.org>.

Source code and issue tracker can be found at
<https://github.com/ulahello/sw>.
