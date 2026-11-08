# Zadanie 8: Udawane urządzenie

**Kategoria:** edycja

## Co jest analizowane

Nagłówek `User-Agent` i rozpoznawanie klientów po tym, jak się przedstawiają.

## Teoria

Każdy klient HTTP przedstawia się serwerowi w nagłówku `User-Agent`. Przykłady:

```
User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0
User-Agent: python-requests/2.32.0
User-Agent: curl/8.5.0
User-Agent: ESP8266HTTPClient
```

Serwery używają go do statystyk, do dopasowania strony (wersja mobilna, desktopowa) i czasem do rozróżniania „swoich” urządzeń od reszty świata. Producent może na przykład udostępniać adres aktualizacji oprogramowania tylko klientom, które przedstawiają się jako jego urządzenia.

Problem polega na tym, że `User-Agent` to **deklaracja, a nie dowód**. Wpisuje go klient i może wpisać cokolwiek. Ten sam mechanizm dotyczy wszystkich nagłówków opisujących klienta: `Referer`, `Origin` (poza przeglądarką), `Accept-Language` czy własnych nagłówków w rodzaju `X-Device-Model`. Prawdziwe rozpoznanie urządzenia wymaga sekretu, którego inni nie znają: klucza, certyfikatu, podpisu.

## Rola proxy

Proxy może zmienić dowolny nagłówek żądania, zanim dotrze do serwera. Serwer nie ma sposobu, żeby odróżnić nagłówek wpisany przez przeglądarkę od nagłówka zmienionego po drodze. Widzi tylko wynik.

## Gdzie to spotkasz w praktyce

- Przy badaniu urządzenia IoT najpierw podsłuchuje się, jak przedstawia się samo urządzenie. Potem można rozmawiać z jego serwerem z komputera, udając to urządzenie.
- Część API zwraca inne dane w zależności od klienta: aplikacja mobilna dostaje więcej pól niż przeglądarka albo starsza wersja aplikacji dostaje mniej zabezpieczeń.

## Do poczytania

- MDN: [User-Agent](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent)
