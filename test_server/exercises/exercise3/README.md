# Zadanie 3: Jeden z pięćdziesięciu

**Kategoria:** filtrowanie

## Co jest analizowane

Kody statusu HTTP i wyszukiwanie jednej nietypowej wiadomości w dużym ruchu.

## Teoria

Każda odpowiedź HTTP zaczyna się od trzycyfrowego **kodu statusu**. Pierwsza cyfra mówi, do jakiej grupy należy:

| Grupa | Znaczenie | Przykłady |
|-------|-----------|-----------|
| 1xx | informacja, rozmowa trwa | `101 Switching Protocols` |
| 2xx | sukces | `200 OK`, `201 Created`, `202 Accepted`, `204 No Content` |
| 3xx | przekierowanie | `301 Moved Permanently`, `302 Found`, `304 Not Modified` |
| 4xx | błąd po stronie klienta | `400 Bad Request`, `401 Unauthorized`, `403 Forbidden`, `404 Not Found` |
| 5xx | błąd po stronie serwera | `500 Internal Server Error`, `503 Service Unavailable` |

Kody w obrębie jednej grupy znaczą co innego. `200 OK` oznacza „gotowe, oto wynik”, a `202 Accepted` „przyjąłem, ale jeszcze nie skończyłem”. Dla skryptu, który sprawdza tylko „czy się udało”, obie odpowiedzi wyglądają tak samo, bo obie są z grupy 2xx.

Gdy urządzenie albo strona wysyła dziesiątki podobnych żądań, ręczne przeglądanie przestaje mieć sens. Potrzebne jest **filtrowanie**: wyrażenie opisujące cechy szukanych wiadomości (adres, metodę, kod, nagłówki, treść), które ukrywa całą resztę.

## Rola proxy

Proxy widzi metadane każdej wiadomości, w tym kod statusu, i może je przeszukiwać. mitmproxy ma własny język filtrów: krótkie wyrażenia, które łączy się operatorami logicznymi „i”, „lub” i „nie”. Ten sam filtr działa w kilku miejscach programu, więc raz poznany przydaje się w każdym kolejnym zadaniu.

## Gdzie to spotkasz w praktyce

- Urządzenia IoT i aplikacje odpytują serwer bez przerwy (telemetria, sprawdzanie stanu). W godzinie ruchu są tysiące prawie identycznych wiadomości, a ciekawe są te, które od nich odbiegają.
- Nietypowy kod statusu często zdradza inną ścieżkę w kodzie serwera: błąd, wyjątek, tryb specjalny.

## Do poczytania

- MDN: [Kody statusu odpowiedzi HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
- Dokumentacja mitmproxy: [Filter expressions](https://docs.mitmproxy.org/stable/concepts-filters/)
