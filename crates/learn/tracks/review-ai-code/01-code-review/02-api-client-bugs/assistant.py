import anthropic

client = anthropic.Anthropic(api_key="sk-ant-REDACTED-EXAMPLE")


def ask(question, tools, run_tool):
    messages = [
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": question},
    ]
    while True:
        response = client.messages.create(
            model="claude-sonnet-4-5", max_tokens=1024, tools=tools, messages=messages
        )
        if response.stop_reason != "tool_use":
            return response.content[0].text
        messages.append({"role": "assistant", "content": response.content})
        for block in response.content:
            if block.type == "tool_use":
                result = run_tool(block.name, block.input)
                messages.append({"role": "user", "content": [{"type": "tool_result", "content": result}]})
