input_tokens = 12_000
output_tokens = 800

cost = input_tokens / 1_000_000 * price_per_mtok["input"] + output_tokens / 1_000_000 * price_per_mtok["output"]
daily_cost = cost * 5000
print(round(daily_cost, 2))
