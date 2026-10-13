A response's `content` is a **list of content blocks**. Text arrives in blocks with `"type": "text"`. `stop_reason` says why generation stopped: for example `"end_turn"`, `"max_tokens"` or `"tool_use"`. `usage` reports `input_tokens` and `output_tokens`.

Because the API is stateless, you continue a conversation by sending the assistant's reply back as an `assistant` message, followed by the next `user` message.
