Review `assistant.py` and find:
- a hardcoded API key;
- a system prompt sent as a message with role `system`;
- a tool loop that never sends the tool result back with the matching `tool_use_id`.
