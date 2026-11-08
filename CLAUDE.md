# handyman

Projekt edukacyjny: nauka pracy z mitmproxy (i docelowo Pythonem `requests`) do badania urządzeń sieciowych/IoT.
`test_server/` to platforma CTF we Flasku, `proxy/` to instrukcja mitmproxy.

## Sposób pracy

- Małe kroki: zadania dodajemy partiami po 2–3, użytkownik sprawdza każdą partię w mitmproxy przed następną.
- Commituj na bieżąco bezpośrednio do `main` po każdej ukończonej i przetestowanej partii.
- Treść dla uczniów (szablony, README) po polsku; komentarze i docstringi w kodzie po angielsku.

## Zadania (`test_server/exercises/exerciseN/`)

- Każde zadanie: `__init__.py` (`NUMBER`, `TITLE`, `CATEGORY`, Blueprint `bp` z `url_prefix=/exerciseN`) i `templates/exerciseN.html` rozszerzający `exercise_base.html` (bloki `task`, `body`, `hints`).
- Flaga zawsze z `core.flags.flag_for(NUMBER)`, nigdy wpisana na sztywno. Wybory „który element jest specjalny” wyliczaj z flagi (patrz exercise3), żeby nie dało się ich odczytać z kodu.
- Weryfikacja flag (`core/flags.py` → `find_exercise`) dostaje numery wszystkich wczytanych zadań z `/flags`, więc nowe zadanie nie wymaga zmian w `flags.py`. Po dodaniu zadania sprawdź to testem: POST flagi na `/flags`.
- Podpowiedzi: dwie, od ogólnej do dokładnej (konkretne klawisze i filtry mitmproxy).
- Testuj klientem testowym Flaska z `LOCK_EXERCISES = False`; do testów `/flags` ustaw tymczasowy `app.instance_path`, żeby nie ruszać postępu użytkownika w `test_server/instance/`.

## Plan zadań 6–20

6 łańcuch przekierowań · 7 ciasteczko roli · 8 User-Agent · 9 edycja odpowiedzi · 10 replay · 11 edycja + replay `?id=` (IDOR) · 12 zakodowana treść (base64+gzip) · 13 stała podmiana (`modify_headers`/`modify_body`) · 14 pierwszy addon (sklejanie kawałków flagi) · 15 addon podpisujący żądania · 16 eksport do `requests` (sesja, CSRF) · 17 token ważny 2 s (skrypt `requests`) · 18 WebSocket · 19 martwa chmura (`http.Response.make`) · 20 finał: termostat.

Otwarte decyzje: zależność dla WebSocket (`flask-sock` vs SSE), czy dodać zadanie z HTTPS.
