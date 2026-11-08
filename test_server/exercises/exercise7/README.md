# Zadanie 7: Ciasteczko roli

**Kategoria:** edycja

## Co jest analizowane

Ciasteczka (cookies): jak serwer je ustawia, jak przeglądarka je odsyła i dlaczego nie można ufać ich zawartości.

## Teoria

HTTP jest **bezstanowy**: każde żądanie jest osobne i serwer sam z siebie nie wie, że dwa żądania przyszły od tej samej osoby. Ciasteczka to sposób na dodanie pamięci:

1. Serwer w odpowiedzi wysyła nagłówek `Set-Cookie: nazwa=wartość; Path=/...`.
2. Przeglądarka zapisuje ciasteczko.
3. Przy każdym kolejnym żądaniu pod pasujący adres przeglądarka sama dołącza nagłówek `Cookie: nazwa=wartość`.

Ciasteczko może mieć atrybuty: `Path` i `Domain` (gdzie je wysyłać), `Expires`/`Max-Age` (jak długo żyje), `HttpOnly` (niedostępne dla JavaScriptu), `Secure` (tylko po HTTPS), `SameSite` (czy wysyłać przy żądaniach z innych stron).

Najważniejsze: wartość ciasteczka **leży po stronie klienta**, a nagłówek `Cookie` to zwykły tekst w żądaniu. Serwer dostaje to, co klient wyśle, a nie to, co sam wcześniej ustawił. Dlatego bezpieczne aplikacje trzymają w ciasteczku tylko losowy identyfikator sesji, a dane o użytkowniku (np. jego uprawnienia) przechowują u siebie. Inna metoda to cyfrowy podpis wartości, po którym serwer rozpozna każdą zmianę.

## Rola proxy

Proxy widzi oba kierunki tej wymiany: nagłówek `Set-Cookie` w odpowiedzi i nagłówek `Cookie` w każdym kolejnym żądaniu. Przechwycone żądanie można zmienić przed wysłaniem, także jego nagłówki. Zmiana w przechwyconym żądaniu dotyczy tylko tego jednego żądania. Ciasteczko zapisane w przeglądarce zostaje takie, jakie było.

## Gdzie to spotkasz w praktyce

- Proste panele urządzeń sieciowych (routery, kamery, drukarki) bywają napisane z ciasteczkami w rodzaju `admin=0`, `lang=pl`, `level=1`.
- Analiza ciasteczek to pierwszy krok przy badaniu, jak urządzenie rozpoznaje zalogowanego użytkownika.

## Do poczytania

- MDN: [Ciasteczka HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Cookies)
- OWASP: [Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
