Complete `run_agent(user_text, max_turns=5)`:
1. Start `messages` with the user message.
2. Loop up to `max_turns`: call `client.messages.create(model=MODEL, max_tokens=1024, tools=TOOLS, messages=messages)`.
3. If `stop_reason` is `"tool_use"`: append the assistant content, run every `tool_use` block with `TOOL_FUNCS[name](**input)` and append one user message holding all the `tool_result` blocks.
4. Otherwise, return the joined text of the response.

At the end, set `final = run_agent("What's 17 * 23, then add 9?")`.
