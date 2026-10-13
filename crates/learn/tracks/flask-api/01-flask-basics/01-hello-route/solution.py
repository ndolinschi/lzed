from flask import Flask

# Create the Flask app: app
app = Flask(__name__)


# Add a route for "/" that returns "Hello, LearnZed!"
@app.route("/")
def index():
    return "Hello, LearnZed!"
