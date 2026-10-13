When Claude wants a tool, the response has `stop_reason == "tool_use"` and contains a `tool_use` block with an `id`, a `name` and an `input`. Your code runs the tool. Then you continue the conversation:

1. Append the assistant's full `content` as an `assistant` message.
2. Append a `user` message whose content is a list containing a **`tool_result`** block. Its `tool_use_id` must match the `id` of the `tool_use` block.
