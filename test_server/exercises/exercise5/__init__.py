"""Exercise 5: the form only lets you pick some plans; the server gives the flag for one that is disabled."""

from flask import Blueprint, render_template, request

from core.flags import flag_for

NUMBER = 5
TITLE = "Niedostępna opcja"
CATEGORY = "edycja"

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


@bp.route("/", methods=["GET", "POST"])
def index():
    result = None
    if request.method == "POST":
        plan = request.form.get("plan", "")
        # The server trusts whatever value arrives; the browser's "disabled" option is only a suggestion
        if plan == "premium":
            result = f"Plan premium aktywny. Flaga: {flag_for(NUMBER)}"
        else:
            result = f"Wybrany plan: {plan or '(brak)'}. Flaga jest tylko w planie premium."
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, result=result)
