# Zadanie 10: Jeszcze raz

**Kategoria:** powtarzanie

## Co jest analizowane

Ponowne wysyłanie żądań (ang. *replay*) i to, czy serwer potrafi odróżnić nowe żądanie od powtórzonego.

## Teoria

Żądanie HTTP to zamknięta paczka danych: metoda, adres, nagłówki, treść. Wszystko, czego serwer potrzebuje, żeby je obsłużyć, jest w środku. Wynika z tego prosta rzecz: tę samą paczkę można wysłać drugi, dziesiąty czy setny raz, a serwer zobaczy kolejne, poprawne żądania.

W HTTP istnieje pojęcie **idempotentności**: metoda jest idempotentna, gdy wielokrotne wykonanie daje ten sam efekt co jednokrotne. `GET`, `PUT` i `DELETE` mają być idempotentne, `POST` nie musi. Głosowanie, przelew czy dodanie produktu do koszyka to typowe operacje, których powtórzenie zmienia stan.

Ograniczenia w interfejsie (przycisk, który blokuje się po kliknięciu, licznik w JavaScripcie) nie chronią przed powtórzeniem, bo działają przed wysłaniem żądania, a nie po stronie serwera. Serwer może się bronić na przykład:

- jednorazowym tokenem w każdym żądaniu (*nonce*), który po użyciu traci ważność;
- znacznikiem czasu i odrzucaniem starych żądań;
- zapamiętaniem po swojej stronie, że dana operacja już się odbyła (dla danego konta, nie dla danego przycisku).

## Rola proxy

Proxy zapamiętuje każde żądanie w całości, więc może je wysłać ponownie, dokładnie w tej samej postaci, z tymi samymi nagłówkami i ciasteczkami. Dla serwera takie żądanie niczym się nie różni od wysłanego przez przeglądarkę. Każde powtórzenie trafia na listę jako nowa para żądanie–odpowiedź, więc widać, jak serwer reaguje na kolejne próby.

## Gdzie to spotkasz w praktyce

- Ataki powtórzeniowe (*replay attacks*) na urządzenia IoT: nagrane polecenie „otwórz” wysłane ponownie otwiera drzwi, jeśli urządzenie nie sprawdza jednorazowości.
- Testowanie limitów: czy kupon rabatowy, głos albo bonus można wykorzystać wielokrotnie.
- Debugowanie: powtórzenie jednego żądania jest szybsze niż odtwarzanie całej ścieżki klikania w interfejsie.

## Do poczytania

- MDN: [Idempotentność](https://developer.mozilla.org/en-US/docs/Glossary/Idempotent)
- Wikipedia: [Replay attack](https://en.wikipedia.org/wiki/Replay_attack)
