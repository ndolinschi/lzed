check_equal("cost", "Use input_tokens and output_tokens with the per-million prices.")
check_equal("daily_cost", "Multiply the single-request cost by 5000.")
check_output_contains("240.0", "Print daily_cost rounded to 2 decimals.")
