from flask import Flask

if check(isinstance(user.get("app"), Flask), "app is a Flask app", "Create your app with `app = Flask(__name__)`."):
    response = user["app"].test_client().get("/")
    check(response.status_code == 200, "GET / returns 200", f"`GET /` returned status {response.status_code}; did you add a route for `/`?")
    check(response.get_data(as_text=True) == "Hello, LearnZed!", "GET / body", "`GET /` should return exactly `Hello, LearnZed!`.")
