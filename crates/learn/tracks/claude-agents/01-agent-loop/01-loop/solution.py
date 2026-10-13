def run_agent(user_text, max_turns=5):
    messages = [{"role": "user", "content": user_text}]
    for _ in range(max_turns):
        response = client.messages.create(model=MODEL, max_tokens=1024, tools=TOOLS, messages=messages)
        if response["stop_reason"] != "tool_use":
            return "".join(b["text"] for b in response["content"] if b["type"] == "text")
        messages.append({"role": "assistant", "content": response["content"]})
        results = []
        for b in response["content"]:
            if b["type"] == "tool_use":
                out = TOOL_FUNCS[b["name"]](**b["input"])
                results.append({"type": "tool_result", "tool_use_id": b["id"], "content": str(out)})
        messages.append({"role": "user", "content": results})
    return None


final = run_agent("What's 17 * 23, then add 9?")
print(final)
