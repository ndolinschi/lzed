MODEL = "claude-sonnet-4-5"

def multiply(a, b):
    return a * b

def add(a, b):
    return a + b

TOOL_FUNCS = {"multiply": multiply, "add": add}
TOOLS = [
    {"name": n, "description": f"{n.capitalize()} two integers.",
     "input_schema": {"type": "object", "properties": {"a": {"type": "integer"}, "b": {"type": "integer"}}, "required": ["a", "b"]}}
    for n in TOOL_FUNCS
]

class _Messages:
    def __init__(self):
        self.calls = []
    def create(self, **kw):
        import copy
        self.calls.append(copy.deepcopy(kw))
        msgs = kw["messages"]
        results = [b for m in msgs if m["role"] == "user" and isinstance(m["content"], list) for b in m["content"] if b.get("type") == "tool_result"]
        if len(results) == 0:
            return {"stop_reason": "tool_use", "content": [{"type": "tool_use", "id": "toolu_1", "name": "multiply", "input": {"a": 17, "b": 23}}]}
        if len(results) == 1:
            if results[0].get("tool_use_id") != "toolu_1":
                return {"stop_reason": "end_turn", "content": [{"type": "text", "text": "ERROR: tool_use_id mismatch"}]}
            return {"stop_reason": "tool_use", "content": [{"type": "text", "text": "Now adding."}, {"type": "tool_use", "id": "toolu_2", "name": "add", "input": {"a": int(results[0]["content"]), "b": 9}}]}
        return {"stop_reason": "end_turn", "content": [{"type": "text", "text": f"The answer is {results[-1]['content']}."}]}

class _Client:
    def __init__(self):
        self.messages = _Messages()

client = _Client()
