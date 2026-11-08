# test_server

Strona testowa w FastAPI. Generuje ruch HTTP, który podglądamy przez proxy (mitmproxy).

| Adres          | Metoda | Co robi                                                  |
|----------------|--------|----------------------------------------------------------|
| `/`            | GET    | strona startowa                                          |
| `/login`       | GET    | formularz logowania                                      |
| `/login`       | POST   | pokazuje dane z formularza i nagłówki, które dotarły do serwera |
| `/static/...`  | GET    | pliki statyczne (CSS, logo)                              |
| `/docs`        | GET    | automatyczna dokumentacja API z FastAPI                  |

## Instalacja (jednorazowo)

```bash
cd test_server
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt
```

## Uruchomienie lokalnie (podczas pracy nad kodem)

```bash
cd test_server
.venv/bin/fastapi dev main.py
```

Strona będzie pod adresem http://127.0.0.1:8000. Serwer sam się przeładowuje po każdej zmianie w plikach.

`fastapi dev` nasłuchuje tylko na `127.0.0.1`, więc strona jest dostępna wyłącznie z tego samego komputera.

## Uruchomienie na Raspberry Pi (dostęp z sieci LAN)

```bash
cd test_server
.venv/bin/fastapi run main.py --host 0.0.0.0 --port 8000
```

- `fastapi run` to tryb bez automatycznego przeładowania, przeznaczony do „normalnej” pracy serwera.
- `--host 0.0.0.0` oznacza nasłuch na wszystkich interfejsach Pi (Wi-Fi, kabel, localhost).
- Z innego komputera wejdź na `http://<IP-Pi>:8000`. Adres Pi sprawdzisz poleceniem `hostname -I`. Zwykle działa też `http://raspberrypi.local:8000`.
- Zamiast `0.0.0.0` możesz podać konkretny adres Pi, np. `--host 192.168.1.50`. Serwer będzie wtedy dostępny tylko przez ten interfejs. Musi to być adres, który Pi faktycznie ma.

### Jeśli strona nie otwiera się z innego komputera

1. Sprawdź, czy serwer uruchomiłeś przez `fastapi run` albo z `--host 0.0.0.0`, a nie przez `fastapi dev`.
2. Jeśli na Pi działa firewall, otwórz port: `sudo ufw allow 8000/tcp`.
3. Sprawdź, czy oba urządzenia są w tej samej sieci.

### Warto ustawić

Zarezerwuj stały adres IP dla Pi w ustawieniach DHCP routera. Wtedy adres się nie zmieni po restarcie.

## Podgląd ruchu przez proxy

Układ do testów: przeglądarka lub skrypt → mitmproxy na laptopie → serwer na Pi. Instrukcja pojawi się razem z projektem analizatora.
