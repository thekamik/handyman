"""Exercise 19: the device's cloud service is shut down; an addon answers in its place, so the request never reaches it."""

import json

from flask import Blueprint, jsonify, render_template, request

from core.flags import flag_for

NUMBER = 19
TITLE = "Martwa chmura"
CATEGORY = "skrypty"

DEVICE_ID = "plug-5521"

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")


@bp.get("/")
def index():
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, device_id=DEVICE_ID)


@bp.get("/cloud/config")
def cloud_config():
    # The "cloud" is gone for good: it never sends a config, only this error
    return jsonify(error="Usługa chmurowa została wyłączona 2026-01-01."), 503


@bp.post("/device/setup")
def device_setup():
    # The device passes on whatever config it got; a valid one switches it to local mode
    config = request.get_json(silent=True) or {}
    if config.get("device_id") != DEVICE_ID:
        return jsonify(error=f"Konfiguracja dla innego urządzenia: {config.get('device_id')!r}."), 400
    if config.get("mode") != "local":
        return jsonify(error=f"Nieobsługiwany tryb: {config.get('mode')!r}. Bez chmury działa tylko tryb local."), 400
    if not config.get("owner"):
        return jsonify(error="Brak pola owner."), 400
    return jsonify(ok=True, message=f"Gniazdko działa lokalnie. Właściciel: {config['owner']}.", flag=flag_for(NUMBER))
