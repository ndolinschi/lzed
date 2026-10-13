Write `code-reviewer.md`, a subagent definition with:
- frontmatter `name: code-reviewer`;
- a `description` that says to use it after code changes, mentioning "review";
- `tools` limited to read-only tools: `Read`, `Grep` and `Glob`, with no `Edit`, `Write` or `Bash`;
- a system prompt body of at least 3 lines below the frontmatter.
