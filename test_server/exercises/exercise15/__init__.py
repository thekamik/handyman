"""Exercise 15: commands are signed with HMAC over a one-time nonce; an addon must re-sign the changed command."""

import hashlib
import hmac
import secrets

from flask import Blueprint, render_template, request

from core.flags import flag_for

NUMBER = 15
TITLE = "Podpis żądania"
CATEGORY = "skrypty"

bp = Blueprint(f"exercise{NUMBER}", __name__, url_prefix=f"/exercise{NUMBER}", template_folder="templates")

# Nonces handed out and not used yet; each one is accepted only once
nonces: set[str] = set()


def device_key() -> str:
    # The "leaked" key shown on the page; derived from the flag so it differs per installation
    return hashlib.sha256(f"key:{flag_for(NUMBER)}".encode()).hexdigest()[:16]


def sign(nonce: str, cmd: str) -> str:
    return hmac.new(device_key().encode(), f"{nonce}:{cmd}".encode(), hashlib.sha256).hexdigest()


@bp.route("/", methods=["GET", "POST"])
def index():
    result = None
    if request.method == "POST":
        nonce = request.form.get("nonce", "")
        cmd = request.form.get("cmd", "")
        sig = request.form.get("sig", "")
        if nonce not in nonces:
            result = ("error", "Nieznany albo już użyty nonce. Odśwież stronę i spróbuj ponownie.")
        else:
            # Used up even when the signature is wrong, so every attempt needs a fresh signature
            nonces.discard(nonce)
            if not hmac.compare_digest(sig, sign(nonce, cmd)):
                result = ("error", "Zły podpis. Urządzenie odrzuciło polecenie.")
            elif cmd == "unlock":
                result = ("ok", f"Urządzenie odblokowane. Flaga: {flag_for(NUMBER)}")
            else:
                result = ("ok", f"Polecenie {cmd!r} wykonane. Stan: OK.")

    nonce = secrets.token_hex(8)
    nonces.add(nonce)
    return render_template(
        f"exercise{NUMBER}.html", number=NUMBER, title=TITLE, category=CATEGORY,
        result=result, nonce=nonce, sig=sign(nonce, "status"), key=device_key(),
    )
