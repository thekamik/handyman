"""Exercise 17: a challenge must be answered within one second, too fast by hand; a requests script does it."""

import random
import secrets
import time

from flask import Blueprint, jsonify, render_template, request

from core.flags import flag_for

NUMBER = 17
TITLE = "Za szybko dla człowieka"
CATEGORY = "requests"

TIME_LIMIT = 1.0  # seconds
COUNT = 50

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")

# token -> (time issued, expected answer); each token can be answered once
challenges: dict[str, tuple[float, int]] = {}


@bp.get("/")
def index():
    return render_template(
        f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY,
        count=COUNT, time_limit=TIME_LIMIT,
    )


@bp.get("/api/challenge")
def challenge():
    numbers = [random.randint(100, 999) for _ in range(COUNT)]
    token = secrets.token_hex(8)
    challenges[token] = (time.monotonic(), sum(numbers))
    return jsonify(token=token, numbers=numbers, task="sum", expires_in=TIME_LIMIT)


@bp.post("/api/answer")
def answer():
    data = request.get_json(silent=True) or {}
    issued = challenges.pop(data.get("token", ""), None)
    if issued is None:
        return jsonify(error="Nieznany albo już użyty token."), 400
    started, expected = issued
    elapsed = time.monotonic() - started
    if elapsed > TIME_LIMIT:
        return jsonify(error=f"Za późno: {elapsed:.1f} s, limit {TIME_LIMIT} s."), 408
    if data.get("sum") != expected:
        return jsonify(error="Zła suma."), 400
    return jsonify(ok=True, flag=flag_for(NUMBER))
