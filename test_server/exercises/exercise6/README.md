# Zadanie 6: Łańcuch przekierowań

**Kategoria:** obserwacja

## Co jest analizowane

Przekierowania HTTP i to, jak przeglądarka przechodzi przez nie automatycznie, bez udziału użytkownika.

## Teoria

Przekierowanie to odpowiedź z kodem z grupy 3xx i nagłówkiem `Location`, który wskazuje nowy adres:

```
HTTP/1.1 302 Found
Location: /exercise6/step/2
Content-Type: text/html; charset=utf-8
```

Przeglądarka, która dostanie taką odpowiedź, od razu wysyła kolejne żądanie pod nowy adres. Pasek adresu pokazuje tylko adres końcowy, a w historii nie zostaje ślad po krokach pośrednich. Jedno kliknięcie może więc oznaczać kilka par żądanie–odpowiedź.

Najczęstsze kody przekierowań:

| Kod | Znaczenie |
|-----|-----------|
| `301` | zasób przeniesiono na stałe (przeglądarka może to zapamiętać) |
| `302` | zasób jest chwilowo gdzie indziej |
| `303` | „zobacz inny adres”, typowo po wysłaniu formularza |
| `307` / `308` | jak 302 / 301, ale z zachowaniem metody i treści żądania |

Odpowiedź z przekierowaniem jest pełnoprawną odpowiedzią HTTP: ma linię statusu, nagłówki i może mieć treść. Standard dopuszcza treść w przekierowaniu, np. krótki tekst z linkiem dla bardzo starych klientów, które nie umieją przejść dalej same. Współczesne przeglądarki nie wyświetlają tej treści, bo przekierowanie wykonują natychmiast.

## Rola proxy

Przeglądarka „połyka” kroki pośrednie, ale każde z tych żądań i każda z tych odpowiedzi przechodzi przez proxy jako osobna wiadomość. Proxy pokazuje cały łańcuch w kolejności: skąd, dokąd, z jakim kodem i z jaką zawartością.

## Gdzie to spotkasz w praktyce

- Logowanie przez zewnętrznego dostawcę (OAuth, „Zaloguj przez Google”) to seria przekierowań, w których przekazywane są kody i tokeny.
- Urządzenia przy pierwszej konfiguracji często przechodzą przez kilka adresów (portal, rejestracja, aktywacja), zanim trafią do panelu.
- Ciasteczka ustawione w kroku pośrednim działają dalej, choć użytkownik nigdy nie widział strony, która je ustawiła.

## Do poczytania

- MDN: [Przekierowania w HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Redirections)
