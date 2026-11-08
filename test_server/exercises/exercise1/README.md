# Zadanie 1: Niewidoczny nagłówek

**Kategoria:** obserwacja

## Co jest analizowane

Budowa odpowiedzi HTTP, a w szczególności jej nagłówki: część, której przeglądarka nie pokazuje użytkownikowi.

## Teoria

Każda wymiana w HTTP składa się z dwóch wiadomości: **żądania** (od klienta do serwera) i **odpowiedzi** (od serwera do klienta). Odpowiedź ma trzy części:

```
HTTP/1.1 200 OK                       ← linia statusu: wersja protokołu, kod, opis
Content-Type: text/html; charset=utf-8 ← nagłówki: pary „Nazwa: wartość”
Content-Length: 1532
                                      ← pusta linia oddziela nagłówki od treści
<!doctype html> ...                   ← treść (body)
```

Przeglądarka rysuje na ekranie tylko treść. Nagłówki zużywa po cichu: z `Content-Type` dowiaduje się, jak potraktować treść, z `Set-Cookie`, co zapamiętać, z `Cache-Control`, czy można trzymać kopię. Użytkownik ich nie widzi.

Standard HTTP pozwala serwerowi wysłać dowolny nagłówek, także taki, którego żadna przeglądarka nie rozumie. Dawniej własne nagłówki oznaczano przedrostkiem `X-` (np. `X-Powered-By`). Nieznane nagłówki klient po prostu pomija, ale **dalej są one częścią przesłanej wiadomości**.

## Rola proxy

Proxy pośredniczące, takie jak mitmproxy, stoi między przeglądarką a serwerem. Każda wiadomość przechodzi przez nie w całości, zanim trafi do odbiorcy. Proxy nie interpretuje treści tak jak przeglądarka. Zapisuje i pokazuje wiadomość dokładnie w takiej postaci, w jakiej przyszła: linię statusu, wszystkie nagłówki i treść. Dlatego widać w nim to, co przeglądarka ukrywa albo pomija.

## Gdzie to spotkasz w praktyce

- Serwery i urządzenia często zdradzają w nagłówkach swoje oprogramowanie i wersję (`Server`, `X-Powered-By`). To pierwsza informacja przy analizie urządzenia sieciowego.
- Nagłówki niosą informacje o sesji, limitach zapytań (`X-RateLimit-Remaining`), identyfikatorach żądań do debugowania.
- Programiści czasem zostawiają w nagłówkach dane diagnostyczne, które miały być widoczne tylko w czasie testów.

## Do poczytania

- MDN: [Wiadomości HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages)
- MDN: [Nagłówki HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers)
