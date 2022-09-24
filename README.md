# sw

`sw` is a simple terminal stopwatch which runs as a shell.

you interact with it by typing single-character commands, and responding to any prompts which follow.

reads are blocking, so there's no fancy live display of the current elapsed time, but that also means that when it's idling, it does literally nothing.

## usage

after running `sw`, type `h` (for "help") to list the commands.
this is all you really need to know.

the core commands are `<enter>` to display the elapsed time, `s` to start or
stop timing, and `r` to reset the stopwatch state.

`sw` also allows you to modify the elapsed time.
you can change it with `c`, or offset it by a positive or negative duration with `o`.

there are some extra features too, like giving the stopwatch a name which prefixes the prompt.
this is useful when you have several instances open.

### duration format

the offset and change commands accept a duration input with the following formatting:

```
float unit
```

`float` is a floating point number, and `unit` is one of "s", "m", or "h", meaning seconds, minutes, and hours respectively.

whitespace is ignored, so `1s` is just as valid as `1 s`.

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

anyhow, `sw` has progressed since then, and is useful in a more general context.
if you're looking for a lightweight stopwatch that runs as a shell, this is for you.

## note on installation

`sw` requires the nightly build of rustc.

## contributions

tickets and improvements are welcome and appreciated!

## versioning

`sw` follows [semver](https://semver.org).

unfortunately, 0.4.0 made breaking changes.
everything else should be correct though.

## license

`GPL-3.0-or-later`, see [LICENSE](./LICENSE).
