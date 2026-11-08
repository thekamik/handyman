"""Exercise 11: the API returns any device by id without checking the owner (IDOR); id 1000 is the admin's."""

from flask import Blueprint, jsonify, render_template, request

from core.flags import flag_for

NUMBER = 11
TITLE = "Cudze urządzenie"
CATEGORY = "powtarzanie"

FIRST_ID = 1000
LAST_ID = 1099
YOUR_ID = 1042

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


@bp.get("/")
def index():
    return render_template(
        f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY,
        your_id=YOUR_ID, first_id=FIRST_ID,
    )


@bp.get("/api/device")
def device():
    device_id = request.args.get("id", type=int)
    if device_id is None or not FIRST_ID <= device_id <= LAST_ID:
        return jsonify(error="Nie ma takiego urządzenia."), 404
    # The server never asks who is asking: knowing the id is enough to read any device
    if device_id == FIRST_ID:
        return jsonify(id=device_id, name="Bramka główna", owner="admin", notes=flag_for(NUMBER))
    if device_id == YOUR_ID:
        return jsonify(id=device_id, name="Twój czujnik ruchu", owner="you")
    return jsonify(id=device_id, name=f"Czujnik {device_id}", owner="inny użytkownik")
