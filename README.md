# sw

`sw` is a simple terminal stopwatch which runs as a shell.

you interact with it by typing commands, and responding to prompts.

## usage

after running `sw`, type `h` (for "help") to list the commands.
this is all you really need to know.

the core commands are `<enter>` to display the elapsed time, `s` to start or stop timing, and `r` to reset the stopwatch state.

`sw` also allows you to modify the elapsed time.
you can change it with `c`, or offset it by a positive or negative duration with `o`.

there are some extra features too, like giving the stopwatch a name which prefixes the prompt.
this is useful when you have several instances open.

### duration format

the offset and change commands accept a duration input.
the following (non-standardised) formats are supported.

#### "unit" format

```
float unit
```

`float` is a floating point number, and `unit` is one of "s", "m", or "h", meaning seconds, minutes, and hours respectively.

leading and trailing whitespace is ignored, so `1s` is just as valid as `1 s` and ` 1s`.

#### HH:MM:SS.ss ("stopwatch") format

```
hours : minutes : seconds . subseconds
```

the details shouldn't be surprising, it's a superset of how durations are displayed.

`hours`, `minutes`, `seconds` and `subseconds` are all integers.
`minutes` and `seconds` must be less than 60.

it's okay to omit separators and values.
rightmost values are the most important, so the meaning of the input will be inferred from right to left.

some examples of terse inputs:
- `:5` and `::5` represent 5 seconds
- `:5:` represents 5 minutes
- `:.6` represents 0.6 seconds
- `1::1.1` represents 1 hour and 1.1 seconds

it's also okay to add whitespace between separators.

## the Use Case

the original use case was to improve on the experience of using an interactive python shell as a stopwatch.

if you're tired of this specifically, you are probably me:

```python
from time import time
t = 0
start = time()
# do whatever is being timed
t += time() - start
```

anyways, `sw` has progressed since then, and is useful in a more general context.

## contributions

tickets and improvements are welcome and appreciated!

## license

`GPL-3.0-or-later`, see [LICENSE](./LICENSE).
