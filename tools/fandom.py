#!/usr/bin/env python3
"""Small helper for reading and writing pages on desaster.fandom.com.

Credentials: ~/.config/rustedpunk-agent/fandom.password with two lines,
`User: <name@botname>` and `Password: <bot password>` (Fandom bot password,
created via Spezial:BotPasswords). The file is never committed.

Usage:
    tools/fandom.py get <PageName>            # prints wikitext to stdout
    tools/fandom.py put <PageName> <file> "<edit summary>"
"""
import http.cookiejar
import json
import pathlib
import sys
import urllib.parse
import urllib.request

API = "https://desaster.fandom.com/api.php"
UA = "rustedPunk-agent/0.1 (Ben's campaign tool; contact ben@sommerfuchs.info)"

_opener = urllib.request.build_opener(
    urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar())
)


def _call(params, post=False):
    params = dict(params, format="json")
    data = urllib.parse.urlencode(params).encode()
    if post:
        req = urllib.request.Request(API, data=data, headers={"User-Agent": UA})
    else:
        req = urllib.request.Request(API + "?" + data.decode(), headers={"User-Agent": UA})
    with _opener.open(req, timeout=30) as response:
        return json.load(response)


def _credentials():
    creds = {}
    path = pathlib.Path.home() / ".config/rustedpunk-agent/fandom.password"
    for line in path.read_text().splitlines():
        if ":" in line:
            key, value = line.split(":", 1)
            creds[key.strip().lower()] = value.strip()
    return creds


def login():
    creds = _credentials()
    token = _call({"action": "query", "meta": "tokens", "type": "login"})["query"]["tokens"]["logintoken"]
    result = _call(
        {"action": "login", "lgname": creds["user"], "lgpassword": creds["password"], "lgtoken": token},
        post=True,
    )
    if result["login"]["result"] != "Success":
        raise SystemExit(f"Fandom login failed: {result['login']['result']}")
    return _call({"action": "query", "meta": "tokens"})["query"]["tokens"]["csrftoken"]


def get_page(page):
    return _call({"action": "parse", "page": page, "prop": "wikitext"})["parse"]["wikitext"]["*"]


def put_page(csrf, page, text, summary):
    result = _call(
        {"action": "edit", "title": page, "text": text, "summary": summary, "token": csrf, "nocreate": 1},
        post=True,
    )
    status = result.get("edit", {}).get("result")
    if status != "Success":
        raise SystemExit(f"Edit of '{page}' failed: {result}")
    print(f"{page}: {status}")


if __name__ == "__main__":
    if len(sys.argv) >= 3 and sys.argv[1] == "get":
        print(get_page(sys.argv[2]))
    elif len(sys.argv) >= 5 and sys.argv[1] == "put":
        csrf = login()
        put_page(csrf, sys.argv[2], pathlib.Path(sys.argv[3]).read_text(), sys.argv[4])
    else:
        raise SystemExit(__doc__)
