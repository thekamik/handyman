# Zadanie 4: Dodatkowy nagłówek

**Kategoria:** filtrowanie

## Co jest analizowane

Różnice między wiadomościami, które leżą wyłącznie w nagłówkach, i filtrowanie po nagłówkach żądań lub odpowiedzi.

## Teoria

Dwie odpowiedzi mogą mieć ten sam kod statusu, ten sam typ i prawie tę samą treść, a mimo to różnić się czymś istotnym. Często tą różnicą są nagłówki, bo to w nich serwery przekazują informacje „obok” właściwych danych: o pamięci podręcznej, sesji, wersji, diagnostyce.

Nagłówki występują w obu kierunkach i każdy kierunek ma swoje typowe zestawy:

- **nagłówki żądania** opisują klienta i jego oczekiwania: `Host`, `User-Agent`, `Accept`, `Cookie`, `Authorization`;
- **nagłówki odpowiedzi** opisują wynik i polecenia dla klienta: `Content-Type`, `Set-Cookie`, `Cache-Control`, `Location`.

Nazwy nagłówków nie rozróżniają wielkości liter (`content-type` i `Content-Type` to ten sam nagłówek). Kolejność zwykle nie ma znaczenia, a ten sam nagłówek może wystąpić kilka razy.

Przy filtrowaniu warto więc umieć powiedzieć nie tylko *czego* szukamy, ale też *gdzie*: w nagłówkach żądania, odpowiedzi czy w obu naraz. Zawężenie miejsca daje mniej przypadkowych trafień.

## Rola proxy

Proxy przechowuje każde żądanie i odpowiedź jako osobne części tej samej pary (w mitmproxy para nazywa się *flow*). Filtry mogą więc pytać o konkretną część, na przykład tylko o nagłówki odpowiedzi. Wyrażenia w filtrach są wyrażeniami regularnymi, więc pozwalają szukać wzorców, a nie tylko dokładnych napisów.

## Gdzie to spotkasz w praktyce

- Serwery za systemami pośredniczącymi (CDN, load balancer) dopisują własne nagłówki. Po nich widać architekturę za usługą.
- Nagłówki diagnostyczne bywają włączane tylko dla niektórych żądań, na przykład przy błędzie albo dla określonych kont.

## Do poczytania

- MDN: [Nagłówki HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers)
- Dokumentacja mitmproxy: [Filter expressions](https://docs.mitmproxy.org/stable/concepts-filters/)
