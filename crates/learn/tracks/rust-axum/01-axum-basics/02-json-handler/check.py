import re
src = strip_comments(user_source)
check(re.search(r"#\[derive\([^)]*Deserialize[^)]*\)\]\s*struct\s+CreateTask", src), "CreateTask: Deserialize", "Add `#[derive(Deserialize)]` to `CreateTask`.")
check(re.search(r"#\[derive\([^)]*Serialize[^)]*\)\]\s*struct\s+Task\b", src), "Task: Serialize", "Add `#[derive(Serialize)]` to `Task`.")
check(re.search(r"async\s+fn\s+create_task\s*\(\s*Json\(\s*\w+\s*\)\s*:\s*Json<\s*CreateTask\s*>\s*\)\s*->\s*Json<\s*Task\s*>", src), "handler signature", "Use `async fn create_task(Json(input): Json<CreateTask>) -> Json<Task>`.")
check(re.search(r"Task\s*\{\s*id\s*:\s*1\s*,\s*title\s*:\s*\w+\.title\s*,?\s*\}", src), "returns task", "Return `Json(Task { id: 1, title: input.title })`.")
check(re.search(r"\.route\(\s*\"/tasks\"\s*,\s*post\(\s*create_task\s*\)\s*\)", src), "route registered", "Register with `.route(\"/tasks\", post(create_task))`.")
check(balanced(src), "braces balanced", "Your braces or parentheses are unbalanced.")
