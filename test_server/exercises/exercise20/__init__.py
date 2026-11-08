"""Exercise 20 (final): find a key in telemetry, reach the device-only service with it, answer a timed challenge."""

import hashlib
import secrets
import time

from flask import Blueprint, jsonify, render_template, request

from core.flags import flag_for

NUMBER = 20
TITLE = "Finał: termostat"
CATEGORY = "wszystko"

PACKETS = 40
TIME_LIMIT = 1.0
DEVICE_PREFIX = "HandymanDevice/"

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")

# nonce -> time issued; each nonce can be answered once
nonces: dict[str, float] = {}


def maintenance_key() -> str:
    return hashlib.sha256(f"maintenance:{flag_for(NUMBER)}".encode()).hexdigest()[:12]


def key_packet() -> int:
    return int(hashlib.sha256(flag_for(NUMBER).encode()).hexdigest(), 16) % PACKETS + 1


def device_check():
    """Error response if the request does not look like an authorised device, else None."""
    if not request.headers.get("User-Agent", "").startswith(DEVICE_PREFIX):
        return jsonify(error=f"Tryb serwisowy tylko dla urządzeń {DEVICE_PREFIX}*."), 403
    if request.headers.get("X-Maintenance-Key") != maintenance_key():
        return jsonify(error="Brak albo zły nagłówek X-Maintenance-Key."), 401
    return None


@bp.get("/")
def index():
    return render_template(f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY, packets=PACKETS)


@bp.get("/api/telemetry")
def telemetry():
    seq = request.args.get("seq", 0, type=int)
    data = {"seq": seq, "temperature": 21.5, "target": 22.0, "heating": seq % 3 == 0}
    if seq == key_packet():
        # Left over from the factory: only one packet out of many carries it
        data["maintenance_key"] = maintenance_key()
    return jsonify(data)


@bp.get("/api/service")
def service():
    if (error := device_check()) is not None:
        return error
    nonce = secrets.token_hex(8)
    nonces[nonce] = time.monotonic()
    return jsonify(
        nonce=nonce, expires_in=TIME_LIMIT,
        how="POST /exercise20/api/service/unlock, JSON {nonce, response}, response = sha256(key + nonce) hex",
    )


@bp.post("/api/service/unlock")
def unlock():
    if (error := device_check()) is not None:
        return error
    data = request.get_json(silent=True) or {}
    issued = nonces.pop(data.get("nonce", ""), None)
    if issued is None:
        return jsonify(error="Nieznany albo już użyty nonce."), 400
    if time.monotonic() - issued > TIME_LIMIT:
        return jsonify(error=f"Za późno, limit {TIME_LIMIT} s."), 408
    expected = hashlib.sha256(f"{maintenance_key()}{data['nonce']}".encode()).hexdigest()
    if data.get("response") != expected:
        return jsonify(error="Zła odpowiedź na wyzwanie."), 400
    return jsonify(ok=True, flag=flag_for(NUMBER))
