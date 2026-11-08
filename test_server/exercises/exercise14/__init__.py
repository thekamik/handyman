"""Exercise 14: the flag is split across many responses, one character each; an addon collects and joins them."""

import hashlib
import random

from flask import Blueprint, jsonify, render_template

from core.flags import flag_for

NUMBER = 14
TITLE = "Pierwszy addon"
CATEGORY = "skrypty"

REQUESTS = 60

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


def parts() -> dict[int, tuple[int, str]]:
    """request number -> (position in the flag, character); positions start at 1."""
    flag = flag_for(NUMBER)
    # Seeded from the flag: the same layout every time, but not readable from the code
    rng = random.Random(hashlib.sha256(flag.encode()).digest())
    carriers = rng.sample(range(1, REQUESTS + 1), len(flag))
    return {request: (position + 1, char) for position, (request, char) in enumerate(zip(carriers, flag))}


@bp.get("/")
def index():
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, requests=REQUESTS)


@bp.get("/api/telemetry/<int:request_number>")
def telemetry(request_number: int):
    response = jsonify(packet=request_number, ok=True)
    part = parts().get(request_number)
    if part:
        position, char = part
        response.headers["X-Part"] = f"{position}/{len(flag_for(NUMBER))}={char}"
    return response
