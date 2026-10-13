from flask import Flask

app = Flask(__name__)

# GET /tasks -> {"tasks": tasks}


# GET /tasks/<int:task_id> -> the task, or {"error": "not found"} with 404

