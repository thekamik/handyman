"""Exercise 9: the page hides a feature when the server's config says "locked"; editing the response unlocks it."""

from flask import Blueprint, jsonify, render_template

from core.flags import flag_for

NUMBER = 9
TITLE = "Edycja odpowiedzi"
CATEGORY = "edycja"

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


@bp.get("/")
def index():
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY)


@bp.get("/api/config")
def config():
    return jsonify(device="camera-07", locked=True, features=["live", "snapshot"])


@bp.post("/api/report")
def report():
    # The lock exists only in the page's JavaScript; the server never checks it again
    return jsonify(report="Raport diagnostyczny", flag=flag_for(NUMBER))
