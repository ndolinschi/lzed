import numpy as np
import random as _random

_rng = _random.Random(1015)
# baseball: list of [height (inches), weight (pounds)] for 1015 MLB players
baseball = [[_rng.randint(67, 83), _rng.randint(150, 290)] for _ in range(1015)]
del _rng, _random
