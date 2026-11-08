# analyzer

Narzędzia do podglądania i automatyzacji ruchu do [`test_server`](../test_server/):

| Plik                         | Co robi                                                                 |
|------------------------------|-------------------------------------------------------------------------|
| `run_proxy.py`               | uruchamia mitmproxy ze skryptu: własny adres nasłuchu i cel przekierowania |
| `addons/message_list.py`     | addon mitmproxy: lista wiadomości HTTP z kierunkiem, jedna na wiersz    |
| `addons/traffic_logger.py`   | addon mitmproxy: wypisuje żądania, pola formularzy i cookies            |
| `clients/login.py`           | skrypt w `requests`: wypełnia formularz logowania, ruch idzie przez proxy |

## Instalacja (jednorazowo)

```bash
cd analyzer
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt
```

## Proxy ze skryptu (`run_proxy.py`)

Proxy działa jako **reverse proxy**: przeglądarka łączy się z proxy tak, jakby to był serwer, a proxy przekazuje cały ruch do `--target`. W przeglądarce nie trzeba ustawiać żadnego proxy.

```
przeglądarka ──▶ proxy (--listen-host:--listen-port) ──▶ serwer testowy (--target)
```

```bash
cd analyzer
.venv/bin/python run_proxy.py --target http://192.168.1.10:8000
```

Potem otwórz w przeglądarce http://127.0.0.1:8080. W terminalu pojawi się lista wiadomości z kierunkiem.

| Opcja             | Domyślnie   | Znaczenie                                                              |
|-------------------|-------------|------------------------------------------------------------------------|
| `--target`        | (wymagana)  | serwer, do którego proxy przekazuje ruch                               |
| `--listen-host`   | `127.0.0.1` | adres nasłuchu proxy; `0.0.0.0` = dostępne z całej sieci               |
| `--listen-port`   | `8080`      | port nasłuchu proxy                                                    |
| `--web`           | wyłączone   | dodatkowo panel mitmweb do otwierania i analizy wiadomości             |
| `--web-port`      | `8081`      | port panelu mitmweb (zawsze tylko na `127.0.0.1`)                      |

Z opcją `--web` skrypt wypisze link z tokenem, np. `http://127.0.0.1:8081/?token=...`. Token zmienia się przy każdym uruchomieniu, a bez niego panel się nie otworzy.

Przykład: proxy dostępne dla innych urządzeń w sieci (np. telefonu), z panelem do analizy:

```bash
.venv/bin/python run_proxy.py --listen-host 0.0.0.0 --listen-port 8080 --target http://192.168.1.10:8000 --web
```

Jeśli port jest zajęty, w terminalu pojawi się błąd `address already in use`. Zatrzymaj skrypt (Ctrl+C) i wybierz inny `--listen-port`.

## Uruchomienie: trzy terminale

**1. Serwer testowy** (lokalnie albo na Raspberry Pi, szczegóły w [`test_server/README.md`](../test_server/README.md)):

```bash
cd test_server
.venv/bin/fastapi dev main.py
```

**2. Proxy z addonem:**

```bash
cd analyzer
.venv/bin/mitmdump -s addons/traffic_logger.py
```

Proxy nasłuchuje na porcie 8080. Inny port ustawisz przez `--listen-port 9090`.

**3. Klient:**

```bash
cd analyzer
.venv/bin/python clients/login.py --user ala --password kot
```

Gdy serwer działa na Pi, dodaj jego adres: `--target http://<IP-Pi>:8000`.

### Co zobaczysz w terminalu z proxy

```
→ GET http://127.0.0.1:8000/login
← 200 text/html; charset=utf-8, 758 bytes (for /login)
→ POST http://127.0.0.1:8000/login
  form field username = 'ala'
  form field password = 'kot'
← 200 text/html; charset=utf-8, 1211 bytes (for /login)
```

Hasło widać otwartym tekstem, bo strona działa po HTTP, bez szyfrowania. To jest właśnie to, co proxy pozwala wykryć.

## Lista wiadomości z kierunkiem

```bash
cd analyzer
.venv/bin/mitmdump -q -s addons/message_list.py
```

`-q` ukrywa własne komunikaty mitmdump, więc w terminalu zostaje sama lista:

```
   #  direction         type     address / content                                        body
   1  client → server   GET      http://192.168.1.10:8000/login                            0 B
   1  server → client   200      text/html; charset=utf-8                                758 B
   2  client → server   POST     http://192.168.1.10:8000/login                           25 B
   2  server → client   200      text/html; charset=utf-8                               1209 B
```

Żądanie i odpowiedź mają ten sam numer. Jeśli serwer nie odpowie, w miejscu odpowiedzi pojawi się wiersz `error` z przyczyną.

Oba addony można uruchomić naraz: `-s addons/message_list.py -s addons/traffic_logger.py` (bez `-q`, żeby widzieć wypisy loggera).

## Przeglądarka zamiast skryptu

Ustaw w przeglądarce proxy HTTP `127.0.0.1`, port `8080`, i otwórz stronę testową.

- Firefox domyślnie **nie** wysyła przez proxy ruchu do `localhost` i `127.0.0.1`. Najprościej użyć serwera na Pi (adres z sieci LAN). Inna opcja: w `about:config` ustaw `network.proxy.allow_hijacking_localhost` na `true`.
- Dla stron HTTPS trzeba zainstalować certyfikat mitmproxy: z włączonym proxy wejdź na http://mitm.it.

## Przydatne opcje mitmdump

| Opcja                         | Działanie                                                      |
|-------------------------------|----------------------------------------------------------------|
| `--listen-host 127.0.0.1`     | proxy dostępne tylko z tego komputera (domyślnie z całej sieci) |
| `--listen-port 9090`          | inny port proxy                                                |
| `-w ruch.mitm`                | zapis całego ruchu do pliku                                    |
| `-r ruch.mitm`                | ponowne odczytanie zapisanego ruchu                            |
| `--flow-detail 0`             | bez domyślnych linii mitmdump, tylko wypisy addonu             |

## Pisanie własnych addonów

Addon to klasa z metodami nazwanymi tak jak zdarzenia mitmproxy (`request`, `response`, `client_connected`...). Każda metoda dostaje obiekt `flow` z polami `flow.request` i `flow.response`. Jako wzór możesz skopiować `addons/traffic_logger.py`.

Dokumentacja: https://docs.mitmproxy.org/stable/addons/overview/
