check(user.get("final") == "The answer is 400.", "final answer", "`final` should be \"The answer is 400.\" Run tools until stop_reason is not tool_use.")
calls = user["client"].messages.calls if "client" in user else []
check(len(calls) == 3, "three API calls", f"Expected 3 calls to client.messages.create, got {len(calls)}.")
check(all(c.get("tools") for c in calls), "tools sent", "Pass `tools=TOOLS` on every call.")
