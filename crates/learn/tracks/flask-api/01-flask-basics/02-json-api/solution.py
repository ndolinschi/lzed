from flask import Flask

app = Flask(__name__)

# GET /tasks -> {"tasks": tasks}
@app.route("/tasks")
def list_tasks():
    return {"tasks": tasks}


# GET /tasks/<int:task_id> -> the task, or {"error": "not found"} with 404
@app.route("/tasks/<int:task_id>")
def get_task(task_id):
    for task in tasks:
        if task["id"] == task_id:
            return task
    return {"error": "not found"}, 404
