"""Exercise 18: telemetry arrives over a WebSocket; one message among many carries the flag."""

import hashlib
import json
import random
import time

from flask import Blueprint, render_template
from flask_sock import Sock

from core.flags import flag_for

NUMBER = 18
TITLE = "WebSocket"
CATEGORY = "protokoły"

MESSAGES = 40

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")
sock = Sock()


def flag_message() -> int:
    return int(hashlib.sha256(flag_for(NUMBER).encode()).hexdigest(), 16) % MESSAGES + 1


@bp.get("/")
def index():
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, messages=MESSAGES)


@sock.route("/ws", bp=bp)
def telemetry(ws):
    # One HTTP request (101 Switching Protocols) opens the connection; after that, messages flow inside it
    special = flag_message()
    for seq in range(1, MESSAGES + 1):
        message = {"seq": seq, "type": "telemetry", "temperature": round(random.uniform(19, 23), 1)}
        if seq == special:
            message = {"seq": seq, "type": "debug", "msg": flag_for(NUMBER)}
        ws.send(json.dumps(message))
        time.sleep(0.1)
    ws.close()
