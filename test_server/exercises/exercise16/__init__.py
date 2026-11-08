"""Exercise 16: login with a CSRF token, then paginated logs the page never shows; a requests script reads them all."""

import hashlib
import secrets

from flask import Blueprint, jsonify, make_response, redirect, render_template, request, url_for

from core.flags import flag_for

NUMBER = 16
TITLE = "Z proxy do Pythona"
CATEGORY = "requests"

USERNAME = "demo"
PASSWORD = "demo123"
PAGES = 30

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")

# session id (cookie "sid") -> {"csrf": token from the login form, "user": logged-in name or None}
sessions: dict[str, dict] = {}


def flag_page() -> int:
    return int(hashlib.sha256(flag_for(NUMBER).encode()).hexdigest(), 16) % (PAGES - 1) + 2  # never page 1


def current_session() -> dict | None:
    return sessions.get(request.cookies.get("sid", ""))


@bp.get("/")
def index():
    sid = request.cookies.get("sid")
    if sid not in sessions:
        sid = secrets.token_hex(8)
        sessions[sid] = {"csrf": secrets.token_hex(8), "user": None}
    session = sessions[sid]
    page = render_template(
        f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY,
        user=session["user"], csrf=session["csrf"], username=USERNAME, password=PASSWORD,
        error=request.args.get("error"), pages=PAGES,
    )
    response = make_response(page)
    response.set_cookie("sid", sid, path=f"/exercise{NUMBER}")
    return response


@bp.post("/login")
def login():
    session = current_session()
    # The CSRF token must match the one sent in this session's form, so a script has to read the form first
    if session is None or request.form.get("csrf") != session["csrf"]:
        return redirect(url_for(".index", error="csrf"))
    if request.form.get("username") != USERNAME or request.form.get("password") != PASSWORD:
        return redirect(url_for(".index", error="login"))
    session["user"] = USERNAME
    return redirect(url_for(".index"))


@bp.get("/api/logs")
def logs():
    session = current_session()
    if session is None or session["user"] is None:
        return jsonify(error="Zaloguj się."), 401
    page = request.args.get("page", 1, type=int)
    if not 1 <= page <= PAGES:
        return jsonify(error="Nie ma takiej strony."), 404
    entries = [f"[{page:02}-{i}] odczyt czujnika: OK" for i in range(1, 6)]
    if page == flag_page():
        entries[2] = f"[{page:02}-3] serwis: {flag_for(NUMBER)}"
    return jsonify(page=page, pages=PAGES, entries=entries)
