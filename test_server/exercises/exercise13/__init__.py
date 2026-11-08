"""Exercise 13: a device polls every second; ten polls in a row must carry a changed header, too fast to edit by hand."""

import secrets
import time

from flask import Blueprint, jsonify, make_response, render_template, request

from core.flags import flag_for

NUMBER = 13
TITLE = "Stała podmiana"
CATEGORY = "automatyzacja"

NEEDED = 10
MAX_GAP = 3.0  # seconds; editing each request by hand takes longer, which breaks the streak

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")

# device id (from the cookie) -> (time of last poll, service polls in a row)
streaks: dict[str, tuple[float, int]] = {}


@bp.get("/")
def index():
    page = render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, needed=NEEDED)
    response = make_response(page)
    if "device" not in request.cookies:
        response.set_cookie("device", secrets.token_hex(8), path=f"/exercise{NUMBER}")
    return response


@bp.get("/api/poll")
def poll():
    device = request.cookies.get("device", "")
    mode = request.headers.get("X-Mode", "")
    now = time.monotonic()
    last, streak = streaks.get(device, (0.0, 0))
    if mode != "service":
        streak = 0
    elif now - last > MAX_GAP:
        streak = 1
    else:
        streak += 1
    streaks[device] = (now, streak)

    data = {"mode": mode, "streak": streak, "needed": NEEDED}
    if streak >= NEEDED:
        data["flag"] = flag_for(NUMBER)
    return jsonify(data)
