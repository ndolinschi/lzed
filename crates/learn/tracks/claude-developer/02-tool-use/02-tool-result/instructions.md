`setup.py` gives you `messages` (the conversation so far), a mock `response` that requests `get_weather`, and a local function `get_weather(city)`.

1. Find the `tool_use` block in `response["content"]` and store it in `block`.
2. Call `get_weather` with its input and store the string in `result`.
3. Build `messages_next = messages + [assistant message, user message with the tool_result]`.
