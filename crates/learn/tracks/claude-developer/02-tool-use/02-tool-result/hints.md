`block = next(b for b in response["content"] if b["type"] == "tool_use")`

---
`{"type": "tool_result", "tool_use_id": block["id"], "content": result}`
