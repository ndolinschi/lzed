def get_weather(city, unit="celsius"):
    return f"{city}: 18 degrees {unit}, cloudy"

messages = [{"role": "user", "content": "What's the weather in Paris?"}]
response = {
    "role": "assistant",
    "stop_reason": "tool_use",
    "content": [
        {"type": "text", "text": "I'll check the weather."},
        {"type": "tool_use", "id": "toolu_01A", "name": "get_weather", "input": {"city": "Paris"}},
    ],
}
