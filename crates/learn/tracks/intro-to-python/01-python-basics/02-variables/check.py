check_equal("height")
check_equal("weight")
check_equal("bmi", "Did you compute `bmi` as `weight / height ** 2`?")
check_output_matches_solution()
