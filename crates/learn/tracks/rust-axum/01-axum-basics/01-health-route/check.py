import re
src = strip_comments(user_source)
check(re.search(r"async\s+fn\s+health\s*\(\s*\)\s*->\s*&\s*'static\s+str", src), "health handler", "Define `async fn health() -> &'static str`.")
check(re.search(r"fn\s+health[^{]*\{\s*\"ok\"\s*\}", src), "returns ok", "`health` should return `\"ok\"`.")
check(re.search(r"\.route\(\s*\"/health\"\s*,\s*get\(\s*health\s*\)\s*\)", src), "route registered", "Register the handler with `.route(\"/health\", get(health))`.")
check(balanced(src), "braces balanced", "Your braces or parentheses are unbalanced.")
