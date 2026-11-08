"""Flags are computed from a random secret, so reading the source code does not reveal them."""

import hashlib
import hmac
import secrets
from pathlib import Path

from flask import current_app


def _secret() -> bytes:
    # Created on first use and kept in instance/ (not in git); deleting it changes every flag
    path = Path(current_app.instance_path) / "secret.key"
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(secrets.token_hex(32))
    return path.read_text().strip().encode()


def flag_for(number: int) -> str:
    """The flag of exercise `number`, e.g. handyman{3f9a...}; always the same for the same secret."""
    digest = hmac.new(_secret(), f"exercise{number}".encode(), hashlib.sha256).hexdigest()
    return f"handyman{{{digest[:20]}}}"


def find_exercise(flag: str, numbers: list[int]) -> int | None:
    """Number of the exercise this flag belongs to, or None if it matches none."""
    flag = flag.strip()
    for number in numbers:
        # compare_digest takes the same time whether the strings differ early or late
        if hmac.compare_digest(flag, flag_for(number)):
            return number
    return None
