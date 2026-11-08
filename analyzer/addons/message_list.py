"""mitmproxy addon: prints every HTTP message as one row of a list, with its direction.

Run from the analyzer folder:  .venv/bin/mitmdump -q -s addons/message_list.py
(-q hides mitmdump's own log lines, so only the list is printed)
"""

from mitmproxy import http

# Column layout shared by the header and the rows
ROW = "{num:>4}  {direction:<16}  {what:<7}  {detail:<50}  {size:>9}"


def size(message: http.Message) -> str:
    # raw_content = body exactly as it travelled on the wire (may be compressed)
    return f"{len(message.raw_content or b'')} B"


class MessageList:
    def __init__(self) -> None:
        self.count = 0

    def running(self) -> None:
        # Called once when the proxy has started
        print(ROW.format(num="#", direction="direction", what="type", detail="address / content", size="body"), flush=True)

    def request(self, flow: http.HTTPFlow) -> None:
        # A request and its response share one number, so pairs are easy to spot in the list
        self.count += 1
        flow.metadata["num"] = self.count
        req = flow.request
        print(ROW.format(num=self.count, direction="client → server", what=req.method, detail=req.pretty_url, size=size(req)), flush=True)

    def response(self, flow: http.HTTPFlow) -> None:
        resp = flow.response
        print(
            ROW.format(
                num=flow.metadata.get("num", "?"),
                direction="server → client",
                what=resp.status_code,
                detail=resp.headers.get("content-type", "-"),
                size=size(resp),
            ),
            flush=True,
        )

    def error(self, flow: http.HTTPFlow) -> None:
        # The server could not be reached or the connection broke before a response came back
        print(ROW.format(num=flow.metadata.get("num", "?"), direction="error", what="-", detail=str(flow.error), size=""), flush=True)


addons = [MessageList()]
