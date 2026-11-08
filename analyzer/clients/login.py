"""Submits the test server's login form through mitmproxy, the way a scripted device or app would.

Run from the analyzer folder:  .venv/bin/python clients/login.py --user ala --password kot
"""

import argparse

import requests


def main() -> None:
    parser = argparse.ArgumentParser(description="Log in to the test server through a proxy.")
    parser.add_argument("--target", default="http://127.0.0.1:8000", help="test server address")
    parser.add_argument("--proxy", default="http://127.0.0.1:8080", help='mitmproxy address; "" = connect directly')
    parser.add_argument("--user", default="ala")
    parser.add_argument("--password", default="kot")
    args = parser.parse_args()

    # Passed to every call explicitly: proxy settings from environment variables (or NO_PROXY
    # for localhost) could otherwise silently send traffic past mitmproxy
    proxies = {"http": args.proxy, "https": args.proxy} if args.proxy else None

    # A Session keeps cookies between requests, like a browser does
    session = requests.Session()

    # Step 1: open the form page, as a user would
    page = session.get(f"{args.target}/login", proxies=proxies, timeout=10)
    print(f"GET  /login -> {page.status_code}")

    # Step 2: submit it; `data=` sends the fields the same way an HTML form does (urlencoded body)
    result = session.post(
        f"{args.target}/login",
        data={"username": args.user, "password": args.password},
        proxies=proxies,
        timeout=10,
    )
    print(f"POST /login -> {result.status_code}")
    print(f"cookies in session: {session.cookies.get_dict()}")


if __name__ == "__main__":
    main()
