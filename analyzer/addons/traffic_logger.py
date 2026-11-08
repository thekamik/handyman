"""mitmproxy addon: prints what passes through the proxy (requests, form fields, cookies).

Run from the analyzer folder:  .venv/bin/mitmdump -s addons/traffic_logger.py
"""

import logging

from mitmproxy import http


class TrafficLogger:
    # mitmproxy calls methods named after events ("hooks"); `request` runs when a request
    # has been fully received from the client, before it is sent on to the server
    def request(self, flow: http.HTTPFlow) -> None:
        req = flow.request
        logging.info("→ %s %s", req.method, req.pretty_url)

        if req.cookies:
            logging.info("  cookies sent: %s", dict(req.cookies))

        # Parses bodies of HTML forms ("username=ala&password=kot"); empty for any other content type
        for name, value in req.urlencoded_form.items(multi=True):
            logging.info("  form field %s = %r", name, value)

    # `response` runs when the server's answer has arrived, before it is passed back to the client
    def response(self, flow: http.HTTPFlow) -> None:
        resp = flow.response
        logging.info(
            "← %s %s, %d bytes (for %s)",
            resp.status_code,
            resp.headers.get("content-type", "-"),
            len(resp.raw_content or b""),
            flow.request.path,
        )

        # Each Set-Cookie header: name -> (value, attributes such as Path, HttpOnly, Secure, Max-Age)
        for name, (value, attrs) in resp.cookies.items(multi=True):
            logging.info("  cookie set: %s=%s %s", name, value, dict(attrs))


# mitmdump loads every object listed here as an addon
addons = [TrafficLogger()]
