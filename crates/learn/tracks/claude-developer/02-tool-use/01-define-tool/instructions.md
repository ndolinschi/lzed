Define `weather_tool` with:
- `name`: `"get_weather"`;
- `description`: at least 20 characters, explaining what it does;
- `input_schema`: a JSON Schema `object` with a string property `city` and an optional string property `unit` whose `enum` is `["celsius", "fahrenheit"]`. Only `city` is required.

Then add it to `request["tools"]`.
