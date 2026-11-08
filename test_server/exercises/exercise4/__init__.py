"""Exercise 4: forty background requests; one response carries an extra header with the flag."""

import hashlib

from flask import Blueprint, jsonify, render_template

from core.flags import flag_for

NUMBER = 4
TITLE = "Dodatkowy nagłówek"
CATEGORY = "filtrowanie"

ITEMS = 40

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


def special_item() -> int:
    return int(hashlib.sha256(flag_for(NUMBER).encode()).hexdigest(), 16) % ITEMS + 1


@bp.get("/")
def index():
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, items=ITEMS)


@bp.get("/item/<int:item_id>")
def item(item_id: int):
    response = jsonify(item=item_id, name=f"Produkt {item_id}")
    # Same status, same body shape: only the headers differ, so a status filter will not help here
    response.headers["Cache-Control"] = "no-store"
    if item_id == special_item():
        response.headers["X-Handyman-Flag"] = flag_for(NUMBER)
    return response
