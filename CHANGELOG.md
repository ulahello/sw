# changelog

## [unreleased]

## [0.9.1] - 2022-09-05
* remove dependency on `log` crate
* make "x seconds since start" msg more accurate
* refactored internals

## [0.9.0] - 2022-08-09
* feat: parse duration and unit together

## [0.8.3] - 2022-08-07
* moved stopwatch to `libsw`

## [0.8.2] - 2022-06-03
* fixed error message formatting
  * error messages weren't followed with two newlines, like normal output
* improved help message formatting
* improved exit code portability
  * now using std defined failure/success codes

## [0.8.1] - 2022-05-02
* refactored internals

## [0.8.0] - 2022-04-25
* added precision command
  * allows user to change the display precision on the fly
* made display command prettier
* print time since last stop

## [0.7.0] - 2022-04-24
* colored logs

## [0.6.3] - 2022-04-24
* fixed: escape control characters when taking input
* changed `quit` help message

## [0.6.2] - 2022-04-12
* fixed incorrect stopwatch subtraction behavior

## [0.6.1] - 2022-04-12
* added license command `l`
* name command: improved help and added status messages
* buffered input so memory allocation never depends on user input

## [0.6.0] - 2022-04-11
* added stopwatch naming
* print license in splash text

## [0.5.0] - 2022-04-09
* prompt now indicates stopwatch status
* fixed panic on invalid float to duration conversion
* replaced calls to functions which may (but don't) panic

## [0.4.0] - 2022-04-02
* changed set command to `c` (c for change)
* changed offset command to `o`

## [0.3.0] - 2022-04-02
* added offset command: `+`
  * allows the user to add or subtract from the total elapsed time
* added message when elapsed time is updated
* improved help message

## [0.2.0] - 2022-04-02
* added set command: `=`
  * allows the user to set the total elapsed time

## [0.1.0] - 2022-03-31
* working stopwatch in interactive shell
  * added commands: `q`, `h`, `s`, `r`
* stopwatch backend (with documentation)
