# Review

L4: off-by-one: range(len(values) - k) misses the last window, should be + 1
L10: mutable default argument groups={} is shared between calls; use None
L17: crashes with ZeroDivisionError on empty input
