"""handyman CTF: exercises for learning mitmproxy, served by Flask.

Run from this folder:
    .venv/bin/flask --app app run --debug                      # local, reloads on code changes
    .venv/bin/flask --app app run --host 0.0.0.0 --port 8000   # Raspberry Pi, reachable from the LAN
"""

from pathlib import Path

from flask import Flask, render_template, request

from core import db
from core.routes import bp as core_bp
from exercises import load_exercises

# True: exercise N opens only after exercise N-1 is solved. False: everything is open.
LOCK_EXERCISES = True


def create_app() -> Flask:
    # instance/ holds files created at runtime (flag secret, progress database); it is not in git
    app = Flask(__name__, instance_path=str(Path(__file__).parent / "instance"))

    exercises = load_exercises()
    app.config["EXERCISES"] = exercises

    app.register_blueprint(core_bp)
    for exercise in exercises:
        # Each exercise is a blueprint: its own routes under /exerciseN/, its own templates
        app.register_blueprint(exercise.bp)

    @app.before_request
    def check_lock():
        # request.blueprint is the name of the blueprint handling this URL, e.g. "exercise3"
        if not LOCK_EXERCISES or not (request.blueprint or "").startswith("exercise"):
            return None
        number = int(request.blueprint.removeprefix("exercise"))
        if number > 1 and number - 1 not in db.solved():
            # Returning a response from before_request skips the exercise's own view
            return render_template("locked.html", number=number), 403
        return None

    return app


# `flask --app app run` looks for this variable
app = create_app()

if __name__ == "__main__":
    app.run(debug=True)
