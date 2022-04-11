# sw

`sw` is a simple terminal stopwatch.

## the Use Case

allegedly, this improves on the experience of using an interactive python shell
as a stopwatch.

```python
>>> from time import time
>>> t = 0
>>> start = time()
>>> # work on stuff
>>> t += time() - start
```

## note on installation

`sw` requires the nightly build of rustc.

## contributions

tickets and improvements are welcome and appreciated!

## license

`GPL-3.0-or-later`, see [LICENSE](./LICENSE).
