"""Test target for the analyzer: a small site whose traffic we inspect through the proxy.

Run from this folder:  .venv/bin/fastapi dev main.py   (http://127.0.0.1:8000)
"""

from pathlib import Path
from typing import Annotated

from fastapi import FastAPI, Form, Request
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates

BASE_DIR = Path(__file__).parent

app = FastAPI(title="handyman test server")

# Files in static/ are sent as they are, under /static/... (CSS, images)
app.mount("/static", StaticFiles(directory=BASE_DIR / "static"), name="static")

# Pages in templates/ are HTML with {{ placeholders }} filled in on every request (Jinja2)
templates = Jinja2Templates(directory=BASE_DIR / "templates")


@app.get("/", response_class=HTMLResponse)
def index(request: Request):
    return templates.TemplateResponse(request, "index.html")


@app.get("/login", response_class=HTMLResponse)
def login_form(request: Request):
    # GET only shows the empty form; submitting it sends a POST to the same address
    return templates.TemplateResponse(request, "login.html")


@app.post("/login", response_class=HTMLResponse)
def login_submit(
    request: Request,
    # Form() reads fields from the POST body (username=...&password=...), matched by the inputs' `name`
    username: Annotated[str, Form()],
    password: Annotated[str, Form()],
):
    # No real authentication yet: the page echoes what the server received,
    # so it can be compared with what the proxy saw on the way
    return templates.TemplateResponse(
        request,
        "output.html",
        {"username": username, "password": password, "headers": request.headers},
    )
