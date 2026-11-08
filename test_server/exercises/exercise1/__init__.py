"""Exercise 1: the flag travels in a response header, which the browser never displays."""

from flask import Blueprint, make_response, render_template

from core.flags import flag_for

NUMBER = 1
TITLE = "Niewidoczny nagłówek"
CATEGORY = "obserwacja"

# template_folder: Flask also looks for templates in this exercise's own templates/ folder.
# Template names are global across blueprints, so each file is named after its exercise.
bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


@bp.get("/")
def index():
    page = render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY)
    response = make_response(page)
    response.headers["X-Flag"] = flag_for(NUMBER)
    return response
