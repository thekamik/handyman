# Zadanie 2: Dane w tle

**Kategoria:** obserwacja

## Co jest analizowane

Żądania wysyłane przez JavaScript strony już po jej załadowaniu (tzw. AJAX, `fetch`) i format JSON, w którym API zwykle odpowiadają.

## Teoria

Otwarcie strony to rzadko jedno żądanie. Przeglądarka najpierw pobiera dokument HTML, potem style, skrypty i obrazki, a uruchomiony skrypt może wysyłać kolejne żądania w dowolnym momencie, bez przeładowania strony. Służy do tego funkcja `fetch()` (dawniej obiekt `XMLHttpRequest`).

Takie żądania zwykle trafiają do **API**, czyli adresów, które nie zwracają strony dla człowieka, tylko dane dla programu. Najczęstszy format to **JSON**:

```json
{"device": "thermostat-01", "online": true, "firmware": "1.4.2"}
```

Ważne rozróżnienie: **to, co serwer wysłał, a to, co strona pokazała, to dwie różne rzeczy.** Skrypt dostaje całą odpowiedź, a potem sam decyduje, które pola wstawić na stronę. Pozostałe pola nie znikają, tylko nikt ich nie wyświetla. Serwery często odsyłają cały obiekt z bazy danych, a o tym, co zobaczy użytkownik, decyduje dopiero kod po stronie przeglądarki.

## Rola proxy

Dla proxy nie ma różnicy, czy żądanie wysłała przeglądarka przy wejściu na stronę, czy skrypt w tle. Każde przechodzi tą samą drogą i każde trafia na listę. Dzięki temu proxy pokazuje pełny obraz komunikacji: ile żądań wysyła strona, do jakich adresów i co dokładnie dostaje w odpowiedzi, zanim kod strony cokolwiek z tego odrzuci.

## Gdzie to spotkasz w praktyce

- Aplikacje mobilne i panele urządzeń IoT to zwykle cienka warstwa wyświetlania nad API. Analiza ruchu do API mówi o urządzeniu więcej niż jego interfejs.
- Nadmierne ujawnianie danych (ang. *excessive data exposure*) to jeden z najczęstszych błędów w API: serwer zwraca pola, które nie powinny opuszczać serwera, licząc na to, że aplikacja ich nie pokaże.

## Do poczytania

- MDN: [Używanie Fetch API](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch)
- OWASP API Security Top 10: [Broken Object Property Level Authorization](https://owasp.org/API-Security/editions/2023/en/0xa3-broken-object-property-level-authorization/)
