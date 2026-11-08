"""Starts mitmproxy from Python, with every address set explicitly.

Reverse proxy mode: clients connect to the proxy as if it were the server, and the proxy
forwards everything to --target. The browser needs no proxy settings: just open the proxy's address.

    browser ──▶ proxy (--listen-host:--listen-port) ──▶ test server (--target)

Examples (from the analyzer folder):
    .venv/bin/python run_proxy.py --target http://192.168.1.10:8000
    .venv/bin/python run_proxy.py --target http://192.168.1.10:8000 --web
"""

import argparse
import asyncio
import secrets

import argon2
from mitmproxy import options
from mitmproxy.tools.dump import DumpMaster
from mitmproxy.tools.web.master import WebMaster

from addons.message_list import MessageList


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Reverse proxy in front of the test server.")
    parser.add_argument("--listen-host", default="127.0.0.1", help="address the proxy listens on (0.0.0.0 = whole network)")
    parser.add_argument("--listen-port", type=int, default=8080, help="port the proxy listens on")
    parser.add_argument("--target", required=True, help="server to forward to, e.g. http://192.168.1.10:8000")
    parser.add_argument("--web", action="store_true", help="also open the mitmweb interface to inspect messages")
    parser.add_argument("--web-port", type=int, default=8081, help="port of the mitmweb interface")
    return parser.parse_args()


async def run(args: argparse.Namespace) -> None:
    # Core options: where to listen and in which mode. "reverse:<url>" forwards every connection to that one server
    opts = options.Options(
        listen_host=args.listen_host,
        listen_port=args.listen_port,
        mode=[f"reverse:{args.target}"],
    )

    # A "master" is the running mitmproxy: the proxy server plus the addons that react to its events.
    # DumpMaster is what mitmdump uses (terminal only), WebMaster is what mitmweb uses (browser UI)
    if args.web:
        master = WebMaster(opts)
        # Our own random token, so we can print a working link. mitmweb is given only its argon2 hash
        # (a one-way fingerprint), which it uses to check the token in the link
        token = secrets.token_hex(16)
        master.options.update(
            web_host="127.0.0.1",  # the UI is reachable only from this computer
            web_port=args.web_port,
            web_open_browser=False,
            web_password=argon2.PasswordHasher().hash(token),
        )
    else:
        # Without mitmdump's own one-line-per-flow output; our MessageList prints the list instead
        master = DumpMaster(opts, with_dumper=False)

    # Show warnings and errors (e.g. "port already in use"), hide routine connect/disconnect lines
    master.options.update(termlog_verbosity="warn")
    master.addons.add(MessageList())

    print(f"proxy:  http://{args.listen_host}:{args.listen_port}  →  {args.target}")
    if args.web:
        print(f"web UI: http://127.0.0.1:{args.web_port}/?token={token}")
    print("Ctrl+C to stop\n")

    await master.run()


def main() -> None:
    try:
        asyncio.run(run(parse_args()))
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
