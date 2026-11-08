"""Exercise 2: the page loads data in the background (fetch); the flag is in the JSON, but never displayed."""

from flask import Blueprint, jsonify, render_template

from core.flags import flag_for

NUMBER = 2
TITLE = "Dane w tle"
CATEGORY = "obserwacja"

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


@bp.get("/")
def index():
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY)


@bp.get("/api/status")
def status():
    # The page's JavaScript shows only "device" and "online"; the other fields stay in the traffic
    return jsonify(device="thermostat-01", online=True, firmware="1.4.2", debug_token=flag_for(NUMBER))
