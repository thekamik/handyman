# analyzer

Narzędzia do podglądania i automatyzacji ruchu do [`test_server`](../test_server/):

| Plik                         | Co robi                                                                 |
|------------------------------|-------------------------------------------------------------------------|
| `addons/traffic_logger.py`   | addon mitmproxy: wypisuje żądania, pola formularzy i cookies            |
| `clients/login.py`           | skrypt w `requests`: wypełnia formularz logowania, ruch idzie przez proxy |

## Instalacja (jednorazowo)

```bash
cd analyzer
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt
```

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
