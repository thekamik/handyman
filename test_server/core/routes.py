"""Landing page and flags page."""

from flask import Blueprint, current_app, redirect, render_template, request, url_for

from core import db
from core.flags import find_exercise

bp = Blueprint("core", __name__)


@bp.get("/")
def index():
    return render_template("index.html", exercises=current_app.config["EXERCISES"], solved=db.solved())


@bp.route("/flags", methods=["GET", "POST"])
def flags():
    exercises = current_app.config["EXERCISES"]
    message = None

    if request.method == "POST":
        number = find_exercise(request.form.get("flag", ""), [e.NUMBER for e in exercises])
        if number is None:
            message = ("error", "To nie jest poprawna flaga.")
        else:
            db.mark_solved(number)
            message = ("ok", f"Poprawna flaga! Zadanie {number} rozwiązane.")

    return render_template("flags.html", exercises=exercises, solved=db.solved(), message=message)


@bp.post("/flags/reset")
def reset():
    db.reset()
    return redirect(url_for("core.flags"))
