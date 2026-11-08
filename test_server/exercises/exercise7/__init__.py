"""Exercise 7: the server reads the user's role from a plain cookie; changing it in the request gives the flag."""

from flask import Blueprint, make_response, render_template, request

from core.flags import flag_for

NUMBER = 7
TITLE = "Ciasteczko roli"
CATEGORY = "edycja"

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


@bp.get("/")
def index():
    # Not signed and not checked against anything: the server believes whatever the request says
    role = request.cookies.get("role", "user")
    flag = flag_for(NUMBER) if role == "admin" else None
    page = render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, role=role, flag=flag)
    response = make_response(page)
    if "role" not in request.cookies:
        # path limits the cookie to this exercise, so it does not travel with other requests
        response.set_cookie("role", "user", path=f"/exercise{NUMBER}")
    return response
