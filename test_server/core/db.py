"""Progress storage: one SQLite file with the list of solved exercises."""

import sqlite3
from contextlib import closing
from pathlib import Path

from flask import current_app


def _connect() -> sqlite3.Connection:
    path = Path(current_app.instance_path) / "progress.db"
    path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(path)
    conn.execute("CREATE TABLE IF NOT EXISTS solved (number INTEGER PRIMARY KEY, solved_at TEXT NOT NULL)")
    return conn


def solved() -> dict[int, str]:
    """Solved exercises: number -> date and time of solving."""
    with closing(_connect()) as conn:
        return dict(conn.execute("SELECT number, solved_at FROM solved"))


def mark_solved(number: int) -> None:
    with closing(_connect()) as conn:
        # OR IGNORE: submitting the same flag again keeps the first solving time
        conn.execute("INSERT OR IGNORE INTO solved VALUES (?, datetime('now', 'localtime'))", (number,))
        conn.commit()


def reset() -> None:
    with closing(_connect()) as conn:
        conn.execute("DELETE FROM solved")
        conn.commit()
