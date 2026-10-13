client = user["app"].test_client() if "app" in user else None
if check(client is not None, "app exists", "Keep `app = Flask(__name__)` in your file."):
    r = client.get("/tasks")
    check(r.status_code == 200 and r.get_json() == {"tasks": solution["tasks"]}, "GET /tasks", "`GET /tasks` should return `{\"tasks\": tasks}` as JSON.")
    r = client.get("/tasks/2")
    check(r.status_code == 200 and r.get_json() == solution["tasks"][1], "GET /tasks/2", "`GET /tasks/2` should return the task whose id is 2.")
    r = client.get("/tasks/99")
    check(r.status_code == 404 and r.get_json() == {"error": "not found"}, "GET /tasks/99", "An unknown id should return `{\"error\": \"not found\"}` with status 404.")
