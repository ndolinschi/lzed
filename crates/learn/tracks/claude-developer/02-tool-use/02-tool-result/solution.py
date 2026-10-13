block = next(b for b in response["content"] if b["type"] == "tool_use")
result = get_weather(**block["input"])
messages_next = messages + [
    {"role": "assistant", "content": response["content"]},
    {"role": "user", "content": [
        {"type": "tool_result", "tool_use_id": block["id"], "content": result},
    ]},
]
