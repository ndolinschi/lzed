check_equal("block", "Pick the content block with type `tool_use`.")
check_equal("result", "Call get_weather with the block input.")
check_equal("messages_next", "Append the assistant content, then a user message with a tool_result whose tool_use_id matches.")
