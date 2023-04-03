# changelog

## [unreleased]
### added
* added `--no-colors` (`-c`) flag to disable color output to the terminal
* added `--version` (`-V`) flag to display version
* added suggestions for similar command names after unknown command error

### changed
* make reset message more clearly indicate that it also stops the stopwatch
* changed fatal error prefix from "fatal: " to "fatal error: "
* added to cli description
* redirect precision parsing overflow errors to clamping warnings
* optimized output to perform less syscalls
* **BREAKING:** changed unit format from float to a integer or decimal number
* **BREAKING:** changed `--no-visual-cues` short name from `-x` to `-v`
* MSRV lowered to `1.61.0`

### fixed
* fixed panic on fatal errors
* fixed inconsistent duration formatting
* fixed unreachable time of check bug
  * when toggling a running and overflowing stopwatch with a non-monotonic Instant, the new elapsed time could be slightly inaccurate compared to the time that overflow was checked. this is unreachable because we use `std::instant::Instant`, which is monotonic.
* only display elapsed time since last stop if it doesn't overflow

## [0.12.0] - 2023-03-20
### added
* added license information of direct dependencies to license command output

### changed
* improved readability of no visual cues duration format
* optimized output to emit less color resets

## [0.11.0] - 2023-03-03
### added
* rewrote README
* added `--no-visual-cues` (`-x`) flag to remove text-based graphics and visual cues
* added `v` command to toggle visual cues
* added status messages for all commands
  * commands should never output nothing
* added help message for parsing unit format when value is missing or invalid

### changed
* change help command format
  * no longer a markdown table, it's very plain now

## [0.10.0] - 2023-02-19
### added
* support for HH:MM:SS.ss duration format
* overhauled error messages
* added message when subtracting clamps to zero

### changed
* changed time display formatting
* changed prompt from "<" and ">" to ";" and "*"
* changed empty precision input to reset to default
* check for overflow when applying offset instead of saturating
* check for overflow when toggling the stopwatch
* check for overflow when displaying elapsed time
* MSRV bumped to `1.66.1`

## [0.9.2] - 2022-09-05
### fixed
* fixed portability of colors

## [0.9.1] - 2022-09-05
### changed
* removed dependency on `log` crate
* refactored internals

### fixed
* made "X seconds since start" message more accurate

## [0.9.0] - 2022-08-09
### added
* parse duration and unit together

## [0.8.3] - 2022-08-07
### changed
* moved stopwatch to `libsw` crate

## [0.8.2] - 2022-06-03
### changed
* improved help message formatting

### fixed
* improved exit code portability
  * now using `std` defined failure/success codes

### fixed
* fixed error message formatting
  * error messages weren't followed with two newlines, like normal output

## [0.8.1] - 2022-05-02
### changed
* refactored internals

## [0.8.0] - 2022-04-25
### added
* added precision command
  * allows user to change the display precision on the fly
* made display command prettier
* print time since last stop

## [0.7.0] - 2022-04-24
### added
* colored logs

## [0.6.3] - 2022-04-24
### changed
* changed `quit` help message

### fixed
* fixed escape control characters when taking input

## [0.6.2] - 2022-04-12
### fixed
* fixed incorrect stopwatch subtraction behavior

## [0.6.1] - 2022-04-12
### added
* added license command `l`
* name command: improved help and added status messages

### changed
* buffered input so memory allocation never depends on user input

## [0.6.0] - 2022-04-11
### added
* added stopwatch naming
* print license in splash text

## [0.5.0] - 2022-04-09
### added
* prompt now indicates stopwatch status

### fixed
* fixed panic on invalid float to duration conversion
* replaced calls to functions which may (but don't) panic

## [0.4.0] - 2022-04-02
### added
* changed set command to `c` (c for change)
* changed offset command to `o`

## [0.3.0] - 2022-04-02
### added
* added offset command: `+`
  * allows the user to add or subtract from the total elapsed time
* added message when elapsed time is updated

### changed
* improved help message

## [0.2.0] - 2022-04-02
### added
* added set command: `=`
  * allows the user to set the total elapsed time

## [0.1.0] - 2022-03-31
### added
* working stopwatch in interactive shell
  * added commands: `q`, `h`, `s`, `r`
* stopwatch backend (with documentation)
