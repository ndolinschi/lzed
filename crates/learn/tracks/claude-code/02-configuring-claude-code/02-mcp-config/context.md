The **Model Context Protocol (MCP)** is an open standard for connecting AI applications to tools and data sources. Claude Code can share project-scoped MCP servers through a `.mcp.json` file at the project root:

```json
{ "mcpServers": { "<name>": { "command": "...", "args": ["..."], "env": {} } } }
```

A local server like this one uses the stdio transport: Claude Code starts the command and talks to it over stdin and stdout.
