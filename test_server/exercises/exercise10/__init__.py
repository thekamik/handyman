"""Exercise 10: the page lets you vote once, the server wants ten votes; replaying the request does it."""

import secrets

from flask import Blueprint, jsonify, make_response, render_template, request

from core.flags import flag_for

NUMBER = 10
TITLE = "Jeszcze raz"
CATEGORY = "powtarzanie"

NEEDED = 10

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")

# voter id (from the cookie) -> number of votes; kept in memory, so a server restart clears it
votes: dict[str, int] = {}


@bp.get("/")
def index():
    voter = request.cookies.get("voter") or secrets.token_hex(8)
    votes.setdefault(voter, 0)
    page = render_template(
        f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY,
        count=votes[voter], needed=NEEDED,
    )
    response = make_response(page)
    response.set_cookie("voter", voter, path=f"/exercise{NUMBER}")
    return response


@bp.post("/vote")
def vote():
    voter = request.cookies.get("voter")
    if voter not in votes:
        return jsonify(error="Nieznany głosujący. Otwórz najpierw stronę zadania."), 400
    # The one-vote limit lives only in the page; every request that arrives is counted
    votes[voter] += 1
    data = {"votes": votes[voter], "needed": NEEDED}
    if votes[voter] >= NEEDED:
        data["flag"] = flag_for(NUMBER)
    return jsonify(data)
