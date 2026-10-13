import re
def block(tag):
    m = re.search(rf"<{tag}>(.*?)</{tag}>", user_source, re.S)
    return m.group(1) if m else None
ins = block("instructions")
check(ins is not None and len([l for l in ins.splitlines() if l.strip()]) >= 2, "instructions", "Add an `<instructions>...</instructions>` block with at least 2 lines.")
t = block("ticket")
check(t is not None and "{{TICKET}}" in t, "ticket placeholder", "Add `<ticket>{{TICKET}}</ticket>` so code can fill in the ticket.")
check(block("example") not in (None, ""), "example", "Add an `<example>...</example>` block.")
check(re.search(r"inside\s+<summary>\s*</summary>|<summary></summary>", user_source), "output tag", "Ask for the answer inside `<summary></summary>` tags.")
