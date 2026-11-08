# handyman

Projekt do nauki automatyzacji i analizy ruchu sieciowego w Pythonie, z użyciem `requests` i `mitmproxy`.

## Projekty

| Katalog                         | Opis                                                                  |
|---------------------------------|-----------------------------------------------------------------------|
| [`test_server/`](test_server/)  | strona testowa w FastAPI, którą obserwujemy przez proxy (np. na Raspberry Pi) |
| [`proxy/`](proxy/)              | manual mitmproxy: podgląd, przechwytywanie i modyfikowanie ruchu do serwera testowego |
| [`analyzer/`](analyzer/)        | addony mitmproxy i skrypty w `requests` do analizy ruchu              |

Każdy projekt ma własne środowisko wirtualne (`.venv`) i własny plik `requirements.txt`. Instrukcje uruchomienia znajdziesz w README każdego katalogu.
