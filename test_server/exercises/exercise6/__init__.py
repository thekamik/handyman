"""Exercise 6: a chain of redirects; one 302 response carries the flag in its body, which browsers never show."""

from flask import Blueprint, redirect, render_template, url_for

from core.flags import flag_for

NUMBER = 6
TITLE = "Łańcuch przekierowań"
CATEGORY = "obserwacja"

STEPS = 5
FLAG_STEP = 3

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


@bp.get("/")
def index():
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, done=False)


@bp.get("/step/<int:step>")
def step(step: int):
    if step >= STEPS:
        return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, done=True)
    response = redirect(url_for(".step", step=step + 1))
    if step == FLAG_STEP:
        # The browser follows Location at once and throws the body of a redirect away unseen
        response.set_data(f"<p>Zgubiona notatka: {flag_for(NUMBER)}</p>")
    return response
