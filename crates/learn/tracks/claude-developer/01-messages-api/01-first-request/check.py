r = user.get("request")
check(isinstance(r, dict), "request is a dict", "Define `request` as a dict.")
r = r if isinstance(r, dict) else {}
check(str(r.get("model", "")).startswith("claude-"), "model", "Set `model` to a Claude model id like \"claude-sonnet-4-5\".")
check(r.get("max_tokens") == 1024, "max_tokens", "`max_tokens` is required. Set it to 1024.")
check(r.get("system") == "You are a concise Python tutor.", "system prompt", "Put the instructions in the top-level `system` key.")
msgs = r.get("messages") or []
check(not any(isinstance(m, dict) and m.get("role") == "system" for m in msgs), "no system role", "The Messages API has no `system` role in `messages`. Use the `system` parameter.")
check(msgs == [{"role": "user", "content": "Explain list comprehensions in one sentence."}], "messages", "`messages` should hold exactly one user message with the given text.")
