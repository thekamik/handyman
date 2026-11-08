# Zadanie 5: Niedostępna opcja

**Kategoria:** edycja

## Co jest analizowane

Wysyłanie formularzy HTML i różnica między walidacją po stronie przeglądarki a sprawdzaniem po stronie serwera.

## Teoria

Formularz HTML to przepis dla przeglądarki: jakie pola pokazać i jak zbudować z nich żądanie. Po kliknięciu „wyślij” przeglądarka składa żądanie `POST`, w którym pola są zakodowane jako `application/x-www-form-urlencoded`:

```
POST /exercise5/ HTTP/1.1
Content-Type: application/x-www-form-urlencoded

plan=basic
```

Atrybuty HTML takie jak `disabled`, `required`, `maxlength` czy lista opcji w `<select>` ograniczają tylko to, **co użytkownik może kliknąć w przeglądarce**. Serwer nigdy ich nie widzi. Do serwera dociera samo żądanie, czyli ciąg bajtów, który mógł zbudować ktokolwiek i czymkolwiek.

Stąd podstawowa zasada bezpieczeństwa aplikacji: **klient jest niezaufany**. Każde ograniczenie, które ma znaczenie, serwer musi sprawdzić u siebie. Walidacja w przeglądarce służy wygodzie (szybki komunikat o błędzie), a nie ochronie.

## Rola proxy

Proxy może nie tylko podglądać ruch, ale też go **przechwytywać** (ang. *intercept*): zatrzymać wiadomość w połowie drogi, pozwolić ją zmienić i dopiero wtedy puścić dalej. Przeglądarka nie wie, że wiadomość została zmieniona, a serwer nie wie, że w ogóle przez coś przeszła. Dla serwera zmienione żądanie niczym nie różni się od oryginalnego.

## Gdzie to spotkasz w praktyce

- Panele urządzeń IoT często ukrywają opcje „dla serwisu” albo „dla wersji premium” tylko w interfejsie, podczas gdy oprogramowanie urządzenia przyjmuje je od każdego.
- Testy bezpieczeństwa aplikacji zaczynają się zwykle od sprawdzenia, czy ograniczenia z interfejsu mają odpowiednik na serwerze.

## Do poczytania

- MDN: [Wysyłanie danych formularza](https://developer.mozilla.org/en-US/docs/Learn/Forms/Sending_and_retrieving_form_data)
- OWASP: [Input Validation Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html)
