import re

prompt = template.replace("{{TICKET}}", ticket)
summary = re.search(r"<summary>(.*?)</summary>", reply, re.S).group(1).strip()
