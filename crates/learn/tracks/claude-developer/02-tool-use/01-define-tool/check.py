t = user.get("weather_tool")
t = t if isinstance(t, dict) else {}
check(t.get("name") == "get_weather", "tool name", "Name the tool `get_weather`.")
check(len(str(t.get("description", ""))) >= 20, "description", "Give the tool a clear description (20+ characters).")
s = t.get("input_schema") or {}
props = s.get("properties") or {}
check(s.get("type") == "object", "schema type", "`input_schema` must be a JSON Schema with `\"type\": \"object\"`.")
check((props.get("city") or {}).get("type") == "string", "city property", "Add a string property `city`.")
check((props.get("unit") or {}).get("enum") == ["celsius", "fahrenheit"], "unit enum", "Add `unit` with `enum` [\"celsius\", \"fahrenheit\"].")
check(s.get("required") == ["city"], "required", "Only `city` should be in `required`.")
check(user.get("request", {}).get("tools") == [t] and t, "tools param", "Add the tool to `request[\"tools\"]` as a list.")
