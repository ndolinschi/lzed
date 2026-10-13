At its core, an agent is a loop. Call Claude with your tools. While the response has `stop_reason == "tool_use"`, run each requested tool, send the results back as `tool_result` blocks and call again. Stop when Claude ends its turn. Always cap the number of iterations.

Here `client` is a **mock** with the same shape as the Anthropic SDK: `client.messages.create(**kwargs)` returns a response dict. No network is used.
