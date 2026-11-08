# proxy: manual mitmproxy

Jak używać mitmproxy do podglądania i modyfikowania ruchu do [`test_server`](../test_server/).

W przykładach serwer testowy działa na Raspberry Pi pod adresem `192.168.1.10:8000`. Jeśli uruchamiasz go lokalnie, zamień ten adres na `127.0.0.1:8000`.

## Spis treści

1. [Instalacja](#1-instalacja)
2. [Trzy narzędzia](#2-trzy-narzędzia)
3. [Dwa sposoby kierowania ruchu](#3-dwa-sposoby-kierowania-ruchu)
4. [Interfejs w terminalu (`mitmproxy`)](#4-interfejs-w-terminalu-mitmproxy)
5. [Filtry](#5-filtry)
6. [Przechwytywanie i modyfikowanie](#6-przechwytywanie-i-modyfikowanie)
7. [Zapisywanie i odtwarzanie ruchu](#7-zapisywanie-i-odtwarzanie-ruchu)
8. [Panel w przeglądarce (`mitmweb`)](#8-panel-w-przeglądarce-mitmweb)
9. [Plik konfiguracyjny](#9-plik-konfiguracyjny)
10. [Addony w Pythonie](#10-addony-w-pythonie)
11. [Ćwiczenia z aplikacją testową](#11-ćwiczenia-z-aplikacją-testową)
12. [HTTPS](#12-https)
13. [Rozwiązywanie problemów](#13-rozwiązywanie-problemów)

---

## 1. Instalacja

Jednorazowo:

```bash
cd proxy
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt
```

Wszystkie komendy poniżej uruchamiasz z katalogu `proxy`.

## 2. Trzy narzędzia

Instalacja mitmproxy daje trzy programy. Wszystkie mają ten sam silnik, opcje i addony, a różnią się tylko interfejsem.

| Program     | Interfejs                  | Kiedy używać                                          |
|-------------|----------------------------|-------------------------------------------------------|
| `mitmproxy` | interaktywny, w terminalu  | codzienna analiza: lista, podgląd, edycja wiadomości  |
| `mitmweb`   | interaktywny, w przeglądarce | to samo co wyżej, ale klikane myszką                |
| `mitmdump`  | sam tekst, bez interakcji  | logowanie, zapis do pliku, uruchamianie addonów       |

## 3. Dwa sposoby kierowania ruchu

### Tryb zwykły (domyślny): przeglądarka wie o proxy

```
przeglądarka (ustawione proxy 127.0.0.1:8080) ──▶ mitmproxy ──▶ dowolny serwer
```

```bash
.venv/bin/mitmproxy --listen-host 127.0.0.1 --listen-port 8080
```

W przeglądarce ustaw proxy HTTP `127.0.0.1`, port `8080`, i wejdź na `http://192.168.1.10:8000`.

- **Firefox:** Ustawienia → Sieć → Ustawienia połączenia → Ręczna konfiguracja proxy.
- **Wygodniej:** dodatek FoxyProxy z regułą „przez proxy tylko `192.168.1.10`”. Wtedy reszta internetu nie zapycha listy.

To jest tryb, w którym docelowo podsłuchuje się urządzenia i aplikacje.

### Tryb reverse: proxy udaje serwer

```
przeglądarka (bez ustawień) ──▶ mitmproxy :8080 ──▶ 192.168.1.10:8000
```

```bash
.venv/bin/mitmproxy --mode reverse:http://192.168.1.10:8000 --listen-host 127.0.0.1 --listen-port 8080
```

W przeglądarce wchodzisz na `http://127.0.0.1:8080`. Niczego nie trzeba konfigurować, ale proxy obsługuje tylko ten jeden serwer. Domyślnie podmienia też nagłówek `Host` na adres celu.

### Opcje adresu

| Opcja                          | Znaczenie                                                    |
|--------------------------------|--------------------------------------------------------------|
| `--listen-host 127.0.0.1`      | proxy dostępne tylko z tego komputera                        |
| `--listen-host 0.0.0.0`        | proxy dostępne z całej sieci (np. dla telefonu); tak jest domyślnie |
| `--listen-port 8080`           | port proxy (domyślnie 8080)                                  |
| `--mode reverse:<url>`         | tryb reverse z podanym celem                                 |

## 4. Interfejs w terminalu (`mitmproxy`)

Po uruchomieniu widać listę wiadomości, która uzupełnia się na bieżąco. Każdy wiersz to jedna para żądanie + odpowiedź (w mitmproxy nazywana **flow**).

### Lista

| Klawisz        | Działanie                                   |
|----------------|---------------------------------------------|
| `↑` `↓` / `j` `k` | poruszanie się po liście                 |
| `Enter`        | otwarcie wiadomości                         |
| `g` / `G`      | początek / koniec listy                     |
| `F`            | automatyczne przewijanie do najnowszej      |
| `f`            | filtr listy (rozdział 5)                    |
| `o`            | sortowanie (czas, adres, rozmiar...)        |
| `d`            | usunięcie wiadomości z listy                |
| `z`            | wyczyszczenie całej listy                   |
| `m`            | oznaczenie wiadomości (`M` = pokaż tylko oznaczone) |
| `;`            | dodanie komentarza                          |

### Podgląd wiadomości (po `Enter`)

| Klawisz        | Działanie                                                   |
|----------------|-------------------------------------------------------------|
| `Tab`          | zakładki: Request / Response / Detail (adresy, czasy, TLS)  |
| `m`            | sposób wyświetlania treści: auto, raw, hex, JSON, URL-encoded... |
| `z`            | dekodowanie / kodowanie treści (np. gzip)                   |
| `b`            | zapis treści odpowiedzi do pliku                            |
| `x`            | eksport, np. jako komenda `curl` albo kod Pythona           |
| `space` / `p`  | następna / poprzednia wiadomość                             |
| `q`            | powrót do listy                                             |

### Ogólne

| Klawisz | Działanie                                         |
|---------|---------------------------------------------------|
| `?`     | pomoc ze wszystkimi skrótami w danym widoku       |
| `:`     | wiersz poleceń mitmproxy                          |
| `O`     | wszystkie opcje (można je zmieniać w trakcie)     |
| `E`     | dziennik zdarzeń (błędy, ostrzeżenia, wypisy addonów) |
| `q`     | wyjście z bieżącego widoku; na liście: wyjście z programu |

## 5. Filtry

Filtry działają w wielu miejscach: `f` (filtr listy), `i` (przechwytywanie), w `mitmweb` i jako argument `mitmdump`.

| Wyrażenie        | Pasuje do                                         |
|------------------|---------------------------------------------------|
| `~m POST`        | metoda POST                                       |
| `~u /login`      | adres zawiera `/login` (wyrażenie regularne)      |
| `~d 192.168.1.10`| domena / host                                     |
| `~c 404`         | kod odpowiedzi 404                                |
| `~t html`        | Content-Type zawiera `html`                       |
| `~h Cookie`      | nagłówek (żądania lub odpowiedzi) zawiera `Cookie` |
| `~hq` / `~hs`    | to samo, tylko nagłówki żądania / odpowiedzi      |
| `~b password`    | treść (żądania lub odpowiedzi) zawiera `password` |
| `~bq` / `~bs`    | to samo, tylko treść żądania / odpowiedzi         |
| `~q`             | żądania bez odpowiedzi                            |
| `~e`             | błędy połączenia                                  |
| `!` `&` `\|` `( )` | negacja, i, lub, nawiasy                        |

Przykłady:

```
~m POST & ~u /login           wysłania formularza logowania
~d 192.168.1.10 & !~u /static  ruch do Pi bez plików statycznych
~c 400 | ~c 500               wybrane kody błędów
```

## 6. Przechwytywanie i modyfikowanie

Przechwycenie zatrzymuje wiadomość w proxy, zanim dotrze do celu. Możesz ją wtedy obejrzeć, zmienić i puścić dalej albo odrzucić.

1. Naciśnij `i` i wpisz filtr, np. `~m POST & ~u /login`.
2. Wyślij formularz w przeglądarce. Strona będzie się „ładować”, a wiadomość na liście zostanie podświetlona jako zatrzymana.
3. `Enter`, żeby ją otworzyć, potem `e`, żeby edytować. Wybierz, co zmienić, np. `urlencoded form` (pola formularza), `headers`, `path`.
4. W edytorze tabeli: `Enter` edytuje komórkę, `a` dodaje wiersz, `d` usuwa wiersz, `q` kończy edycję.
5. `a` puszcza wiadomość dalej (`A` = wszystkie zatrzymane). `X` na liście zabija połączenie.

Inne przydatne klawisze:

| Klawisz | Działanie                                                      |
|---------|----------------------------------------------------------------|
| `I`     | włączenie / wyłączenie przechwytywania bez kasowania filtra    |
| `r`     | ponowne wysłanie (replay) wybranego żądania                    |
| `D`     | duplikat wiadomości, np. żeby zmienić kopię i ją wysłać przez `r` |
| `V`     | cofnięcie zmian w wiadomości                                   |

Przechwycić można też odpowiedź: gdy filtr pasuje do odpowiedzi, zatrzyma się ona w drodze do przeglądarki.

Przechwytywanie można włączyć od razu przy starcie:

```bash
.venv/bin/mitmproxy --set intercept='~m POST & ~u /login'
```

## 7. Zapisywanie i odtwarzanie ruchu

| Co                                  | Jak                                                  |
|-------------------------------------|------------------------------------------------------|
| zapis całej sesji od startu         | `.venv/bin/mitmproxy -w sesja.mitm`                  |
| zapis listy w trakcie               | `w` na liście (zapisuje widoczne wiadomości)         |
| ciągły zapis od teraz               | `W`                                                  |
| otwarcie zapisu                     | `.venv/bin/mitmproxy -r sesja.mitm` albo `L` na liście |
| wypisanie zapisu w terminalu        | `.venv/bin/mitmdump -n -r sesja.mitm`                |
| ponowne wysłanie zapisanych żądań   | `.venv/bin/mitmdump -n -C sesja.mitm` (client replay) |

`-n` oznacza „nie uruchamiaj proxy”: mitmdump tylko czyta plik.

Pliki `.mitm` zawierają pełny ruch, łącznie z hasłami i cookies. Nie commituj ich do repozytorium.

## 8. Panel w przeglądarce (`mitmweb`)

```bash
.venv/bin/mitmweb --mode reverse:http://192.168.1.10:8000 --listen-port 8080 --set web_open_browser=false
```

- Proxy działa na porcie 8080, a panel pod `http://127.0.0.1:8081`.
- Przy starcie w terminalu pojawia się link z tokenem (`...?token=...`). Bez niego panel się nie otworzy.
- U góry jest pole filtra (składnia z rozdziału 5) i pole **Intercept** (przechwytywanie).
- Po kliknięciu wiadomości widać zakładki Request, Response, Connection i Timing. Zatrzymaną wiadomość edytujesz w zakładce i puszczasz przyciskiem **Resume**.
- Panel ma też menu do zapisu i otwierania plików `.mitm` oraz do ponownego wysłania żądania (Replay).

## 9. Plik konfiguracyjny

Żeby nie wpisywać opcji za każdym razem, zapisz je w `~/.mitmproxy/config.yaml`. Nazwy opcji są takie jak w `--set`, a myślniki zamienia się na podkreślenia:

```yaml
listen_host: 127.0.0.1
listen_port: 8080
mode:
  - reverse:http://192.168.1.10:8000
web_open_browser: false
```

Teraz wystarczy samo `.venv/bin/mitmproxy`. Opcje z linii poleceń nadpisują te z pliku.

Ten plik działa dla wszystkich uruchomień mitmproxy na koncie. Gdy chcesz wrócić do trybu zwykłego, zakomentuj `mode`.

Pełna lista opcji: `O` w `mitmproxy` albo `.venv/bin/mitmdump --options`.

## 10. Addony w Pythonie

Addon to skrypt w Pythonie z klasą, której metody mitmproxy wywołuje przy kolejnych zdarzeniach. Ładujesz go opcją `-s` w każdym z trzech programów:

```bash
.venv/bin/mitmproxy -s moj_addon.py
```

Minimalny addon:

```python
import logging

from mitmproxy import http


class Hello:
    def request(self, flow: http.HTTPFlow) -> None:
        # runs for every request, before it is sent to the server
        logging.info("request to %s", flow.request.pretty_url)


addons = [Hello()]
```

- Wypisy z `logging` widać w dzienniku zdarzeń (`E`) w `mitmproxy` albo w terminalu przy `mitmdump`.
- Addon przeładowuje się sam po zapisaniu pliku, bez restartu proxy.
- Najważniejsze zdarzenia to `request`, `response`, `error` i `client_connected`. Pola żądania to `flow.request.method`, `.url`, `.headers`, `.cookies`, `.urlencoded_form` i `.content`.

Dokumentacja: https://docs.mitmproxy.org/stable/addons/overview/

## 11. Ćwiczenia z aplikacją testową

Uruchom proxy w trybie reverse (rozdział 3) i otwórz `http://127.0.0.1:8080`.

1. **Ładowanie strony.** Otwórz stronę startową. Ile żądań wysłała przeglądarka? Dlaczego więcej niż jedno? Podpowiedź: CSS i logo.
2. **Formularz.** Zaloguj się. Otwórz żądanie `POST /login`, potem `Tab` na Request i `m` → `urlencoded form`. Hasło jest widoczne otwartym tekstem: to podstawowe ustalenie przy analizie urządzeń (brak szyfrowania).
3. **Porównanie.** Strona wyniku pokazuje nagłówki, które dotarły do serwera. Porównaj je z zakładką Request w proxy. Co się różni w trybie reverse? Podpowiedź: `Host`.
4. **Modyfikacja.** Ustaw przechwytywanie `~m POST & ~u /login`, wyślij formularz, zmień w proxy login na inny i puść dalej. Strona wyniku pokaże zmienioną wartość.
5. **Replay.** Wybierz żądanie `POST /login` i naciśnij `r`. Serwer dostaje je drugi raz, bez udziału przeglądarki.
6. **API.** Wejdź na `/docs`. Znajdź żądanie `GET /openapi.json` i obejrzyj odpowiedź w trybie JSON (`m`).
7. **Zapis.** Uruchom proxy z `-w sesja.mitm`, powtórz kroki 1–2, zamknij i otwórz zapis przez `-r sesja.mitm`.

## 12. HTTPS

Serwer testowy działa po HTTP, więc niczego nie trzeba instalować. Do podglądu ruchu HTTPS przeglądarka lub urządzenie musi ufać certyfikatowi mitmproxy:

1. Uruchom proxy w trybie zwykłym i ustaw je w przeglądarce.
2. Wejdź na http://mitm.it i pobierz certyfikat dla swojego systemu lub przeglądarki.
3. Zainstaluj go jako zaufany urząd certyfikacji. Firefox ma własny magazyn: Ustawienia → Prywatność → Certyfikaty → Importuj.

Certyfikat leży też w `~/.mitmproxy/mitmproxy-ca-cert.pem`. Instaluj go tylko na urządzeniach testowych: kto ma klucz z tego katalogu, może podsłuchiwać ich ruch HTTPS.

Aplikacje i urządzenia z **certificate pinning** odrzucą certyfikat mitmproxy mimo instalacji. To już osobny temat.

## 13. Rozwiązywanie problemów

| Objaw                                       | Przyczyna i rozwiązanie                                       |
|---------------------------------------------|---------------------------------------------------------------|
| `address already in use`                    | port zajęty (np. inne proxy działa w tle); użyj innego `--listen-port` |
| lista pusta, strona działa                  | przeglądarka idzie bezpośrednio: sprawdź ustawienia proxy albo użyj trybu reverse |
| ruch do `localhost` nie trafia do proxy     | Firefox omija proxy dla localhost: użyj adresu Pi albo ustaw `network.proxy.allow_hijacking_localhost` w `about:config` |
| `502 Bad Gateway`                           | proxy nie może połączyć się z serwerem: czy serwer na Pi działa i adres jest poprawny? |
| strona „wisi”                               | wiadomość jest przechwycona: naciśnij `a` albo wyłącz przechwytywanie (`I`) |
| ostrzeżenie o certyfikacie na stronach HTTPS | certyfikat mitmproxy nie jest zainstalowany (rozdział 12)    |
