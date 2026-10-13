import re
src = user_source
sections = markdown_sections(src)
check(re.search(r"^# \S", src, re.M), "title", "Start the file with a `# ` title.")
check(sections.get("project", "").strip(), "Project section", "Add a `## Project` section describing the project.")
cmds = sections.get("commands", "")
check("```" in cmds and "cargo test" in cmds, "Commands section", "Add `## Commands` with a fenced code block that includes `cargo test`.")
check(len(re.findall(r"^\s*[-*] \S", sections.get("conventions", ""), re.M)) >= 2, "Conventions section", "Add `## Conventions` with at least two bullet points.")
