Claude Code supports **subagents**: specialized assistants with their own context window, system prompt and allowed tools. Project subagents live in `.claude/agents/<name>.md`. Each is a Markdown file with YAML frontmatter:

```markdown
---
name: my-agent
description: When Claude should use this agent
tools: Read, Grep
---
System prompt for the agent...
```

If you leave out `tools`, the subagent inherits all of the main thread's tools.
