`setup.py` provides `request` (the earlier request) and a mock `response` dict.

1. Set `answer` to the concatenated text of all `text` blocks in `response["content"]`.
2. Set `total_tokens` to input plus output tokens from `response["usage"]`.
3. Build `next_messages`: the old `request["messages"]`, then `{"role": "assistant", "content": answer}`, then a user message `"Show an example."`.
