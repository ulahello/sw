# Changelog

## [Unreleased]

## [0.14.1] - 2024-09-12
### Changed
* more lax duration parsing
  * truncate excess subsecond digits instead of raising error
  * don't error when minutes or seconds exceed maximum
* replaced `libsw` dependency with `libsw_core`

### Removed
* removed `deny.toml`

## [0.14.0] - 2024-08-28
### Added
* added `name` positional argument to set stopwatch name
* added icon

### Changed
* reuse strings to reduce frequent allocations
* removed nix files

## [0.13.3] - 2023-07-18
### Added
* added parse error for unexpected negative duration
* added `default.nix` and `shell.nix` for building with nix

### Changed
* replaced `is-terminal` dependency with `std::io::IsTerminal`
  * MSRV bumped to `1.70.0`

### Fixed
* replaced int overflow errors in duration parsing with duration overflow errors
* fixed hyperlink formatting in README

## [0.13.2] - 2023-06-03
### Fixed
* fixed `sw -V` requiring tty
* fixed `sw -V` writing to `stderr` instead of `stdout`

## [0.13.1] - 2023-04-05
### Added
* added tty check
* added `--no-tty-check` flag to disable tty check

### Fixed
* fixed unexpected negative offset behavior while elapsed time is overflowing
* fixed incorrect MSRV (now compiles as expected)

## [0.13.0] - 2023-04-03
### Added
* added `--no-colors` (`-c`) flag to disable color output to the terminal
* added `--version` (`-V`) flag to display version
* added suggestions for similar command names after unknown command error

### Changed
* make reset message more clearly indicate that it also stops the stopwatch
* changed fatal error prefix from "fatal: " to "fatal error: "
* added to cli description
* redirect precision parsing overflow errors to clamping warnings
* optimized output to perform less syscalls
* **BREAKING:** changed unit format from float to a integer or decimal number
* **BREAKING:** changed `--no-visual-cues` short name from `-x` to `-v`
* MSRV lowered to `1.61.0`

### Fixed
* fixed panic on fatal errors
* fixed inconsistent duration formatting
* fixed unreachable time of check bug
  * when toggling a running and overflowing stopwatch with a non-monotonic Instant, the new elapsed time could be slightly inaccurate compared to the time that overflow was checked. this is unreachable because we use `std::instant::Instant`, which is monotonic.
* only display elapsed time since last stop if it doesn't overflow

## [0.12.0] - 2023-03-20
### Added
* added license information of direct dependencies to license command output

### Changed
* improved readability of no visual cues duration format
* optimized output to emit less color resets

## [0.11.0] - 2023-03-03
### Added
* rewrote README
* added `--no-visual-cues` (`-x`) flag to remove text-based graphics and visual cues
* added `v` command to toggle visual cues
* added status messages for all commands
  * commands should never output nothing
* added help message for parsing unit format when value is missing or invalid

### Changed
* change help command format
  * no longer a markdown table, it's very plain now

## [0.10.0] - 2023-02-19
### Added
* support for HH:MM:SS.ss duration format
* overhauled error messages
* added message when subtracting clamps to zero

### Changed
* changed time display formatting
* changed prompt from "<" and ">" to ";" and "*"
* changed empty precision input to reset to default
* check for overflow when applying offset instead of saturating
* check for overflow when toggling the stopwatch
* check for overflow when displaying elapsed time
* MSRV bumped to `1.66.1`

## [0.9.2] - 2022-09-05
### Fixed
* fixed portability of colors

## [0.9.1] - 2022-09-05
### Changed
* removed dependency on `log` crate
* refactored internals

### Fixed
* made "X seconds since start" message more accurate

## [0.9.0] - 2022-08-09
### Added
* parse duration and unit together

## [0.8.3] - 2022-08-07
### Changed
* moved stopwatch to `libsw` crate

## [0.8.2] - 2022-06-03
### Changed
* improved help message formatting

### Fixed
* improved exit code portability
  * now using `std` defined failure/success codes

### Fixed
* fixed error message formatting
  * error messages weren't followed with two newlines, like normal output

## [0.8.1] - 2022-05-02
### Changed
* refactored internals

## [0.8.0] - 2022-04-25
### Added
* added precision command
  * allows user to change the display precision on the fly
* made display command prettier
* print time since last stop

## [0.7.0] - 2022-04-24
### Added
* colored logs

## [0.6.3] - 2022-04-24
### Changed
* changed `quit` help message

### Fixed
* fixed escape control characters when taking input

## [0.6.2] - 2022-04-12
### Fixed
* fixed incorrect stopwatch subtraction behavior

## [0.6.1] - 2022-04-12
### Added
* added license command `l`
* name command: improved help and added status messages

### Changed
* buffered input so memory allocation never depends on user input

## [0.6.0] - 2022-04-11
### Added
* added stopwatch naming
* print license in splash text

## [0.5.0] - 2022-04-09
### Added
* prompt now indicates stopwatch status

### Fixed
* fixed panic on invalid float to duration conversion
* replaced calls to functions which may (but don't) panic

## [0.4.0] - 2022-04-02
### Added
* changed set command to `c` (c for change)
* changed offset command to `o`

## [0.3.0] - 2022-04-02
### Added
* added offset command: `+`
  * allows the user to add or subtract from the total elapsed time
* added message when elapsed time is updated

### Changed
* improved help message

## [0.2.0] - 2022-04-02
### Added
* added set command: `=`
  * allows the user to set the total elapsed time

## [0.1.0] - 2022-03-31
### Added
* working stopwatch in interactive shell
  * added commands: `q`, `h`, `s`, `r`
* stopwatch backend (with documentation)
