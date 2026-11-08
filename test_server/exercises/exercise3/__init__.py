"""Exercise 3: fifty background requests; exactly one response has a different status code."""

import hashlib
import random

from flask import Blueprint, jsonify, render_template

from core.flags import flag_for

NUMBER = 3
TITLE = "Jeden z pięćdziesięciu"
CATEGORY = "filtrowanie"

SENSORS = 50

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


def special_sensor() -> int:
    # Picked from the flag, so it stays the same between restarts but cannot be read from the code
    return int(hashlib.sha256(flag_for(NUMBER).encode()).hexdigest(), 16) % SENSORS + 1


@bp.get("/")
def index():
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, sensors=SENSORS)


@bp.get("/sensor/<int:sensor_id>")
def sensor(sensor_id: int):
    reading = {"sensor": sensor_id, "temperature": round(random.uniform(18, 24), 1)}
    if sensor_id == special_sensor():
        # 202 Accepted instead of 200 OK: the only response that stands out
        return jsonify({**reading, "note": flag_for(NUMBER)}), 202
    return jsonify(reading)
