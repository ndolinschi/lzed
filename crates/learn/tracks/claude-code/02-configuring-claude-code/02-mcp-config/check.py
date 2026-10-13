import json
try:
    data = json.loads(user_source)
except Exception:
    data = None
check(isinstance(data, dict), "valid JSON", ".mcp.json is not valid JSON.")
servers = (data or {}).get("mcpServers", {})
fs = servers.get("filesystem") if isinstance(servers, dict) else None
check(isinstance(fs, dict), "filesystem server", "Add a server named `filesystem` under `mcpServers`.")
fs = fs or {}
check(fs.get("command") == "npx", "command", "Set `command` to `npx`.")
check(fs.get("args") == ["-y", "@modelcontextprotocol/server-filesystem", "./docs"], "args", "Set `args` to the three given values, in order.")
