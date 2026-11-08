# test_server: handyman CTF

Aplikacja w Flasku z zadaniami do nauki mitmproxy. Każde zadanie ukrywa flagę `handyman{...}`, którą znajdziesz tylko w ruchu przechodzącym przez proxy. Zadania są ułożone od najłatwiejszego.

| Adres            | Co to jest                                  |
|------------------|---------------------------------------------|
| `/`              | lista zadań                                 |
| `/flags`         | wklejanie flag i postęp                     |
| `/exerciseN/`    | zadanie numer N                             |

Zadanie N otwiera się po rozwiązaniu zadania N-1. Blokadę wyłącza `LOCK_EXERCISES = False` w `app.py`.

## Instalacja (jednorazowo)

```bash
cd test_server
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt
```

## Uruchomienie lokalnie (podczas pracy nad kodem)

```bash
cd test_server
.venv/bin/flask --app app run --debug
```

Strona będzie pod adresem http://127.0.0.1:5000. Z `--debug` serwer sam się przeładowuje po każdej zmianie w kodzie i pokazuje szczegóły błędów.

## Uruchomienie na Raspberry Pi (dostęp z sieci LAN)

```bash
cd test_server
.venv/bin/flask --app app run --host 0.0.0.0 --port 8000
```

- `--host 0.0.0.0` oznacza nasłuch na wszystkich interfejsach Pi (Wi-Fi, kabel, localhost). Bez tej opcji Flask nasłuchuje tylko na `127.0.0.1` i strona nie jest widoczna z sieci.
- Z innego komputera wejdź na `http://<IP-Pi>:8000`. Adres Pi sprawdzisz poleceniem `hostname -I`. Zwykle działa też `http://raspberrypi.local:8000`.
- Na Pi nie używaj `--debug`: tryb debug pozwala uruchomić kod Pythona z poziomu strony błędu.

### Jeśli strona nie otwiera się z innego komputera

1. Sprawdź, czy serwer uruchomiłeś z `--host 0.0.0.0`.
2. Jeśli na Pi działa firewall, otwórz port: `sudo ufw allow 8000/tcp`.
3. Sprawdź, czy oba urządzenia są w tej samej sieci.

### Warto ustawić

Zarezerwuj stały adres IP dla Pi w ustawieniach DHCP routera. Wtedy adres się nie zmieni po restarcie.

## Postęp i flagi

Przy pierwszym uruchomieniu Flask tworzy katalog `instance/` (nie trafia do gita):

- `secret.key`: losowy klucz, z którego liczone są flagi. Dzięki temu flag nie widać w kodzie źródłowym. Usunięcie pliku zmienia wszystkie flagi.
- `progress.db`: baza SQLite z rozwiązanymi zadaniami. Postęp zerujesz przyciskiem na stronie `/flags` albo usuwając ten plik.

## Struktura kodu

```
test_server/
├── app.py                 # creates the Flask app, registers exercises, exercise lock
├── core/                  # platform: flags, progress database, landing and flags pages
├── exercises/
│   ├── __init__.py        # finds every exerciseN folder automatically
│   └── exercise1/
│       ├── __init__.py    # NUMBER, TITLE, CATEGORY, routes (Blueprint)
│       └── templates/exercise1.html
├── templates/             # shared layout, landing page, flags page, exercise_base.html
└── static/                # style.css, logo.png
```

Nowe zadanie: skopiuj folder istniejącego zadania, zmień nazwę folderu i pliku szablonu, a w `__init__.py` zmień `NUMBER`, `TITLE` i `CATEGORY`. Aplikacja znajdzie je sama.

## Podgląd ruchu przez proxy

Układ do testów: przeglądarka → mitmproxy na laptopie → serwer na Pi. Instrukcja: [`proxy/README.md`](../proxy/README.md).
