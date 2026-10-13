answer = "".join(b["text"] for b in response["content"] if b["type"] == "text")
total_tokens = response["usage"]["input_tokens"] + response["usage"]["output_tokens"]
next_messages = request["messages"] + [
    {"role": "assistant", "content": answer},
    {"role": "user", "content": "Show an example."},
]
