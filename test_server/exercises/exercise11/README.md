# Zadanie 11: Cudze urządzenie

**Kategoria:** powtarzanie

## Co jest analizowane

Identyfikatory obiektów w adresach API i sprawdzanie, czy serwer kontroluje, kto o dany obiekt pyta (podatność IDOR).

## Teoria

API zwykle wskazują obiekty identyfikatorem w adresie:

```
GET /api/device?id=1042          ← parametr zapytania (query string)
GET /api/devices/1042            ← fragment ścieżki
```

Część po znaku `?` to **parametry zapytania**: pary `nazwa=wartość` połączone znakiem `&`. Są częścią adresu, więc klient może je zmienić tak samo łatwo jak każdy inny fragment żądania.

W kontroli dostępu rozróżnia się dwa pojęcia:

- **uwierzytelnienie** (*authentication*): kim jesteś? (login, hasło, token);
- **autoryzacja** (*authorization*): czy wolno ci zrobić *to* z *tym* obiektem?

**IDOR** (*Insecure Direct Object Reference*) to sytuacja, w której serwer wykonuje tylko pierwszy krok albo żaden. Zwraca obiekt każdemu, kto poda jego identyfikator, bez sprawdzania, czy obiekt należy do pytającego. Problem nasila się, gdy identyfikatory są **kolejnymi liczbami**: znając jeden, łatwo zgadnąć wszystkie pozostałe. Losowe identyfikatory (np. UUID) utrudniają zgadywanie, ale nie zastępują sprawdzania uprawnień.

## Rola proxy

Proxy pozwala wziąć istniejące, poprawne żądanie i zrobić z niego kopię. Kopię można zmienić, np. w adresie albo parametrach, i wysłać, a oryginał zostaje do porównania. Odpowiedź na kopię pokazuje, jak serwer traktuje zmienione dane.

## Gdzie to spotkasz w praktyce

- IDOR to jedna z najczęściej zgłaszanych podatności w API i stoi na pierwszym miejscu listy OWASP API Security Top 10.
- W chmurach urządzeń IoT znane są przypadki, w których zmiana numeru seryjnego w żądaniu dawała podgląd z cudzych kamer albo sterowanie cudzymi urządzeniami.

## Do poczytania

- OWASP API Security Top 10: [Broken Object Level Authorization](https://owasp.org/API-Security/editions/2023/en/0xa1-broken-object-level-authorization/)
- MDN: [Budowa adresu URL](https://developer.mozilla.org/en-US/docs/Learn/Common_questions/Web_mechanics/What_is_a_URL)
