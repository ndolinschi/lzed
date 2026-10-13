import json
try:
    data = json.loads(user_source)
    ok = True
except Exception as e:
    data, ok = {}, False
check(ok, "valid JSON", "settings.json is not valid JSON.")
perms = data.get("permissions", {}) if isinstance(data, dict) else {}
allow, deny = set(perms.get("allow", [])), set(perms.get("deny", []))
check({"Bash(npm run lint)", "Bash(npm run test:*)"} <= allow, "allow rules", "`allow` needs Bash(npm run lint) and Bash(npm run test:*).")
check({"Read(./.env)", "Read(./secrets/**)"} <= deny, "deny rules", "`deny` needs Read(./.env) and Read(./secrets/**).")
