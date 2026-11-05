"use strict";

const $ = (selector) => document.querySelector(selector);

const now = new Date();

// Informacje o przeglądarce

$("#jsStatus").textContent = "aktywny ✓";
$("#currentUrl").textContent = location.href;
$("#protocol").textContent = location.protocol;
$("#userAgent").textContent = navigator.userAgent;
$("#language").textContent = navigator.language;
$("#online").textContent = navigator.onLine ? "tak" : "nie";

$("#screenSize").textContent =
  `${screen.width} × ${screen.height}`;

$("#timezone").textContent =
  Intl.DateTimeFormat().resolvedOptions().timeZone;

$("#pageTime").textContent = now.toLocaleString();


// Obsługa żądań HTTP

function getEndpoint() {
  const path = $("#endpoint").value.trim() || "/";
  return new URL(path, location.origin).toString();
}


async function sendRequest(method) {

  const output = $("#httpOutput");

  output.textContent = `Wysyłanie ${method}…`;

  const headers = {
    "Accept": "application/json, text/plain, */*",
    "Cache-Control": "no-cache",
    "X-Requested-With": "fetch"
  };

  const options = {
    method,
    headers,
    cache: "no-store"
  };

  if (["POST", "PUT", "PATCH"].includes(method)) {

    headers["Content-Type"] = "application/json";

    options.body = JSON.stringify({
      message: "Hello, world!",
      timestamp: new Date().toISOString(),
      client: "HTTP Hello World Test Page"
    });
  }

  try {

    const response = await fetch(getEndpoint(), options);

    const body = method === "HEAD"
      ? "(HEAD: odpowiedź bez treści)"
      : await response.text();

    const responseHeaders =
      [...response.headers.entries()]
        .map(([key, value]) => `${key}: ${value}`)
        .join("\n");

    output.textContent =

`REQUEST
${method} ${new URL(getEndpoint()).pathname}
${Object.entries(headers).map(([k,v]) => `${k}: ${v}`).join("\n")}

RESPONSE
HTTP ${response.status} ${response.statusText}
URL: ${response.url}
Redirected: ${response.redirected}
Type: ${response.type}

RESPONSE HEADERS
${responseHeaders || "(brak dostępnych nagłówków)"}

BODY (fragment)
${body.slice(0, 5000)}`;

  } catch (error) {

    output.textContent =
      `Błąd Fetch: ${error.name}: ${error.message}\n\n` +
      `Sprawdź konsolę przeglądarki, adres endpointu, HTTPS/CORS oraz dostępność serwera.`;
  }
}


// Przyciski metod HTTP

document.querySelectorAll("[data-method]").forEach((button) => {

  button.addEventListener("click", () => {
    sendRequest(button.dataset.method);
  });

});


// Odczytywanie nagłówków odpowiedzi

$("#headersButton").addEventListener("click", async () => {

  const out = $("#headersOutput");

  out.textContent = "Pobieranie…";

  try {

    const response = await fetch("/", {
      method: "GET",
      cache: "no-store"
    });

    out.textContent =
      `Status: ${response.status} ${response.statusText}\n` +
      [...response.headers.entries()]
        .map(([k,v]) => `${k}: ${v}`)
        .join("\n");

  } catch (e) {

    out.textContent = `${e.name}: ${e.message}`;

  }

});


// Performance Timing API

$("#timingButton").addEventListener("click", () => {

  const nav = performance.getEntriesByType("navigation")[0];

  $("#timingOutput").textContent = nav

    ? JSON.stringify({

        type: nav.type,
        protocol: nav.nextHopProtocol,

        redirect_ms:
          nav.redirectEnd - nav.redirectStart,

        dns_ms:
          nav.domainLookupEnd - nav.domainLookupStart,

        connect_ms:
          nav.connectEnd - nav.connectStart,

        response_ms:
          nav.responseEnd - nav.responseStart,

        dom_interactive_ms:
          nav.domInteractive,

        load_event_ms:
          nav.loadEventEnd

      }, null, 2)

    : "Navigation Timing API niedostępne.";

});


// Cookies

$("#cookieButton").addEventListener("click", () => {

  document.cookie =
    "hello_world_test=1; Path=/; SameSite=Lax; Max-Age=3600";

  $("#storageOutput").textContent =
    `document.cookie = ${
      document.cookie || "(brak widocznych cookies)"
    }`;

});


// LocalStorage

$("#storageButton").addEventListener("click", () => {

  try {

    localStorage.setItem(
      "hello_world_test",
      new Date().toISOString()
    );

    $("#storageOutput").textContent =
      `localStorage.hello_world_test = ${
        localStorage.getItem("hello_world_test")
      }`;

  } catch (e) {

    $("#storageOutput").textContent =
      `${e.name}: ${e.message}`;

  }

});


// Generowanie pliku Blob

$("#downloadButton").addEventListener("click", () => {

  const content =
    `Hello, world!\nGenerated: ${new Date().toISOString()}\n`;

  const blob = new Blob(
    [content],
    { type: "text/plain;charset=utf-8" }
  );

  const url = URL.createObjectURL(blob);

  const link = document.createElement("a");

  link.href = url;
  link.download = "hello-world.txt";

  link.click();

  URL.revokeObjectURL(url);

});


// Monitorowanie połączenia

window.addEventListener("online", () => {

  $("#online").textContent = "tak";

});


window.addEventListener("offline", () => {

  $("#online").textContent = "nie";

});
