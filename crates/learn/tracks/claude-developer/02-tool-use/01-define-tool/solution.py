weather_tool = {
    "name": "get_weather",
    "description": "Get the current weather for a city. Returns temperature and conditions.",
    "input_schema": {
        "type": "object",
        "properties": {
            "city": {"type": "string", "description": "City name, e.g. Paris"},
            "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]},
        },
        "required": ["city"],
    },
}

request["tools"] = [weather_tool]
