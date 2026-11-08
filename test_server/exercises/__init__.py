"""Exercises: every subfolder named exerciseN is one exercise, found and loaded automatically.

Each exercise package must define:
    NUMBER    its position in the course (1 = easiest)
    TITLE     short name shown in the list
    CATEGORY  skill it trains (obserwacja, filtrowanie, edycja, replay...)
    bp        a Flask Blueprint with url_prefix="/exerciseN"

To add an exercise: copy the folder of an existing one, rename it and change the four values above.
"""

import importlib
import pkgutil
from types import ModuleType


def load_exercises() -> list[ModuleType]:
    modules = []
    # __path__ is this package's folder; iter_modules lists the packages inside it
    for info in pkgutil.iter_modules(__path__):
        if info.ispkg and info.name.startswith("exercise"):
            modules.append(importlib.import_module(f"{__name__}.{info.name}"))
    # Sort by number, not by name: "exercise10" would otherwise come before "exercise2"
    return sorted(modules, key=lambda module: module.NUMBER)
