# Zadanie 9: Edycja odpowiedzi

**Kategoria:** edycja

## Co jest analizowane

Logika po stronie klienta: decyzje, które podejmuje kod strony lub aplikacji na podstawie odpowiedzi serwera.

## Teoria

W nowoczesnych aplikacjach serwer często nie wysyła gotowego widoku, tylko dane i ustawienia, a o wyglądzie decyduje kod u klienta (JavaScript w przeglądarce, aplikacja mobilna, oprogramowanie urządzenia). Typowe dane konfiguracyjne to na przykład:

```json
{"locked": true, "features": ["live", "snapshot"], "premium": false}
```

Kod klienta czyta te pola i na ich podstawie pokazuje albo ukrywa przyciski, włącza funkcje, wyświetla ostrzeżenia. Takie przełączniki funkcji (ang. *feature flags*) są wygodne, ale mają ważną właściwość: **decyzja zapada u klienta**, a klient ufa temu, co dostał.

Jeśli serwer nie sprawdza tego samego warunku przy kolejnym żądaniu, ukryta funkcja jest dostępna dla każdego, kto sprawi, że klient zobaczy inne dane. Zabezpieczenie istnieje wtedy tylko w interfejsie.

Odpowiedź serwera nie jest dla klienta czymś pewnym. Przychodzi przez sieć i może zostać zmieniona w drodze, jeśli połączenie nie jest chronione (np. przez HTTPS ze sprawdzaniem certyfikatu).

## Rola proxy

Proxy pośredniczy w obu kierunkach, więc przechwytywać można nie tylko żądania, ale też **odpowiedzi**: zatrzymać je w drodze od serwera do klienta, zmienić i oddać klientowi. Klient dostaje zmienioną wersję i działa na niej, nie wiedząc o zmianie. Serwer o niczym się nie dowiaduje, bo jego odpowiedź opuściła go w oryginalnej postaci.

## Gdzie to spotkasz w praktyce

- Aplikacje mobilne sterujące urządzeniami często pobierają przy starcie konfigurację z flagami funkcji. Zmiana odpowiedzi pozwala zbadać ukryte ekrany i funkcje serwisowe.
- Testowanie zachowania klienta przy nietypowych danych, np. błędach, pustych listach, ogromnych liczbach, bez zmieniania serwera.

## Do poczytania

- OWASP: [Client-side security](https://owasp.org/www-project-top-10-client-side-security-risks/)
- Dokumentacja mitmproxy: [Modifying traffic](https://docs.mitmproxy.org/stable/mitmproxytutorial-modifyrequests/)
