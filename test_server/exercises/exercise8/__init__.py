"""Exercise 8: the server only answers "devices", recognised by nothing more than the User-Agent header."""

from flask import Blueprint, render_template, request

from core.flags import flag_for

NUMBER = 8
TITLE = "Udawane urządzenie"
CATEGORY = "edycja"

DEVICE_PREFIX = "HandymanDevice/"

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


@bp.get("/")
def index():
    # User-Agent is written by the client, so any client can claim to be anything
    user_agent = request.headers.get("User-Agent", "")
    flag = flag_for(NUMBER) if user_agent.startswith(DEVICE_PREFIX) else None
    return render_template(
        f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY,
        user_agent=user_agent, device_prefix=DEVICE_PREFIX, flag=flag,
    )
