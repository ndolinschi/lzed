import re
m = re.match(r"\s*---\n(.*?)\n---\n(.*)", user_source, re.S)
check(m, "frontmatter", "Start with YAML frontmatter between `---` lines.")
fm = dict(re.findall(r"^(\w+):\s*(.*)$", m.group(1), re.M)) if m else {}
body = m.group(2) if m else ""
check(fm.get("name", "").strip() == "code-reviewer", "name", "Set `name: code-reviewer`.")
check("review" in fm.get("description", "").lower(), "description", "Describe when to use it and mention review.")
tools = {t.strip() for t in fm.get("tools", "").split(",") if t.strip()}
check(tools and tools <= {"Read", "Grep", "Glob"} and "Read" in tools, "read-only tools", "Limit `tools` to Read, Grep, Glob (no Edit/Write/Bash).")
check(len([l for l in body.splitlines() if l.strip()]) >= 3, "system prompt", "Write at least 3 lines of system prompt after the frontmatter.")
