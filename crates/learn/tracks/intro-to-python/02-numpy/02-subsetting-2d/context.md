If your 2D NumPy array has a regular structure, i.e. each row and column has a fixed number of values, complicated ways of subsetting become very easy. Have a look at the code below where the elements `"a"` and `"c"` are extracted from a list of lists.

```python
# numpy
import numpy as np
np_x = np.array(x)
np_x[:, 0]
```

For regular Python lists, this is a real pain. For 2D NumPy arrays, however, it's pretty intuitive! The indexes before the comma refer to the rows, while those after the comma refer to the columns. The `:` is for slicing; in this example, it tells Python to include all rows.

The code that converts the pre-loaded `baseball` list to a 2D NumPy array is already in the script. The first column contains the players' height in inches and the second column holds player weight, in pounds.
