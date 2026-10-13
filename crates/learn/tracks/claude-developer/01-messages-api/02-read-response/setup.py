request = {
    "model": "claude-sonnet-4-5",
    "max_tokens": 1024,
    "messages": [{"role": "user", "content": "Explain list comprehensions in one sentence."}],
}
response = {
    "id": "msg_mock_01",
    "type": "message",
    "role": "assistant",
    "content": [
        {"type": "text", "text": "A list comprehension builds a list "},
        {"type": "text", "text": "from an iterable in a single expression."},
    ],
    "stop_reason": "end_turn",
    "usage": {"input_tokens": 21, "output_tokens": 17},
}
