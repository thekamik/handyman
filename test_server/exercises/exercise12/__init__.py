"""Exercise 12: the response is gzip-compressed JSON wrapped in base64, so the proxy shows only gibberish."""

import base64
import gzip
import json

from flask import Blueprint, make_response, render_template

from core.flags import flag_for

NUMBER = 12
TITLE = "Zakodowana treść"
CATEGORY = "obserwacja"

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


@bp.get("/")
def index():
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY)


@bp.get("/api/backup")
def backup():
    data = {"device": "router-03", "firmware": "2.7.1", "wifi_password": flag_for(NUMBER)}
    # Packed by the application itself, not via Content-Encoding, so the proxy cannot unpack it automatically
    body = base64.b64encode(gzip.compress(json.dumps(data).encode()))
    response = make_response(body)
    response.headers["Content-Type"] = "text/plain"
    response.headers["X-Payload-Format"] = "base64(gzip(json))"
    return response
