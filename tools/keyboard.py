#!/usr/bin/env python3
"""A keyboard: the real play page in a headless browser, driven one typed line at a time.

    tools/keyboard.py serve [--port 8790] [--site site/sites/agentscanplay] [--api https://play.agentscanplay.com]
    curl -s localhost:8790/type -d '{"line": "look"}'
    curl -s localhost:8790/see
    curl -s localhost:8790/shot          # a screenshot, as PNG bytes

For a playtest of the page itself (not the API): an agent, or a person, types what a
person at a keyboard would, and gets back what the page shows: the new transcript
lines, the first-hours strip, the verb bar, the wants and mine boards, the room
panel, the atlas's rooms, the grave if it is open. Nothing here reaches the world
except through the page, so the page is what is tested.

The page is served from the checkout (so the latest, not the deployed one) and talks
to the live driver; the browser runs with CORS off for that, which is fine for a
local harness and nothing else. Needs Playwright with Chromium (this container has
it). The passphrase typed at the second prompt is never echoed back.
"""

import argparse
import http.server
import json
import os
import socketserver
import sys
import threading
import time

try:
    from playwright.sync_api import sync_playwright
except ImportError:  # pragma: no cover
    sys.exit("playwright is not installed here")


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


def serve_site(site, port):
    handler = lambda *a, **k: Quiet(*a, directory=site, **k)  # noqa: E731
    socketserver.TCPServer.allow_reuse_address = True
    srv = socketserver.TCPServer(("127.0.0.1", port), handler)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


class Keyboard:
    def __init__(self, site_port, api):
        self.p = sync_playwright().start()
        self.browser = self.p.chromium.launch(args=["--disable-web-security", "--autoplay-policy=no-user-gesture-required"])
        self.ctx = self.browser.new_context(viewport={"width": 1400, "height": 900})
        self.page = self.ctx.new_page()
        self.errors = []
        self.page.on("pageerror", lambda e: self.errors.append(str(e)))
        self.page.goto(f"http://127.0.0.1:{site_port}/play/?api={api}", wait_until="load")
        self.page.wait_for_timeout(1200)
        self.seen = 0
        self.lock = threading.Lock()

    def _transcript(self):
        return self.page.evaluate("Array.from(document.querySelectorAll('#log .line')).map(l => (l.className.replace('line','').trim() ? '[' + l.className.replace('line','').trim() + '] ' : '') + l.innerText)")

    def type(self, line, wait_ms=2500):
        with self.lock:
            asking = self.page.evaluate("document.getElementById('prompt').textContent")
            self.page.fill("#in", line)
            self.page.press("#in", "Enter")
            self.page.wait_for_timeout(wait_ms)
            # A walk by room name keeps going after the first reply; give it a moment.
            for _ in range(6):
                if self.page.evaluate("document.getElementById('in').disabled") is False and not self.page.evaluate("!!document.querySelector('#grave:not([hidden])')"):
                    break
                self.page.wait_for_timeout(500)
            out = self.see()
            lines = self._transcript()
            out["new_lines"] = lines[self.seen:]
            self.seen = len(lines)
            if asking.strip() == "passphrase:":
                out["new_lines"] = [l for l in out["new_lines"] if "•" in l or not l.startswith("[you]")]
            return out

    def see(self):
        pg = self.page
        j = lambda expr: pg.evaluate(expr)  # noqa: E731
        return {
            "prompt": j("document.getElementById('prompt').textContent"),
            "actor": j("window.foxing && foxing.actor"),
            "first_hours": None if j("document.getElementById('first').hidden") else j("Array.from(document.querySelectorAll('#first-steps li')).map(l => (l.classList.contains('done') ? '[done] ' : l.classList.contains('now') ? '[now] ' : '[ ] ') + l.querySelector('code').textContent + (l.classList.contains('now') ? ' ' + l.querySelector('.why').textContent : ''))"),
            "verb_bar": j("Array.from(document.querySelectorAll('#verbbar button')).map(b => b.textContent)"),
            "here": j("Array.from(document.querySelectorAll('#here .row')).map(r => r.innerText.replace(/\\n/g, ' | '))"),
            "you": j("document.getElementById('you') ? document.getElementById('you').innerText.replace(/\\n/g, ' | ') : ''"),
            "window": j("document.getElementById('window') ? document.getElementById('window').innerText.replace(/\\n/g, ' | ') : ''"),
            "wants": j("Array.from(document.querySelectorAll('#wants .row')).map(r => r.innerText.replace(/\\n/g, ' | '))"),
            "mine": j("document.getElementById('mine').innerText.replace(/\\n/g, ' | ')") + (" || " + j("document.getElementById('mine-since').innerText") if j("document.getElementById('mine-since').innerText") else ""),
            "atlas_rooms": j("(window.foxing && foxing.layout) ? Array.from(document.querySelectorAll('#atlas .cell')).map(g => g.querySelector('title') ? g.querySelector('title').textContent : '') : []"),
            "panels": j("window.foxing && foxing.layout ? foxing.layout.panels().map(p => p.id + ':' + p.width + (p.hidden ? ' hidden' : '')) : []"),
            "grave": None if j("document.getElementById('grave').hidden") else {
                "epitaph": j("document.getElementById('grave-epitaph').textContent"),
                "ledger": j("document.getElementById('grave-ledger').textContent"),
                "kept": j("document.getElementById('grave-kept').textContent"),
                "where": j("document.getElementById('grave-where').textContent"),
                "stones": j("document.querySelectorAll('#stones .stone').length"),
            },
            "scene_folded": j("document.getElementById('main').classList.contains('collapsed')"),
            "page_errors": self.errors[-3:],
        }

    def shot(self, path):
        with self.lock:
            self.page.screenshot(path=path)

    def key(self, name):
        with self.lock:
            self.page.keyboard.press(name)
            self.page.wait_for_timeout(400)
            return self.see()

    def click(self, text):
        """Click a verb-bar button, a first-hours step, or a wants/mine row by its text."""
        with self.lock:
            ok = self.page.evaluate(
                """(t) => { const els = [...document.querySelectorAll('#verbbar button, #first-steps code, #wants .row, #mine .row, .tile.go, .way')];
                   const el = els.find(e => e.textContent.trim().startsWith(t)); if (!el) return false; el.click(); return true; }""",
                text,
            )
            self.page.wait_for_timeout(1500)
            out = self.see()
            out["clicked"] = ok
            out["input"] = self.page.evaluate("document.getElementById('in').value")
            lines = self._transcript()
            out["new_lines"] = lines[self.seen:]
            self.seen = len(lines)
            return out


def main():
    ap = argparse.ArgumentParser(description="the play page, driven a line at a time")
    sub = ap.add_subparsers(dest="cmd")
    s = sub.add_parser("serve")
    s.add_argument("--port", type=int, default=8790)
    s.add_argument("--site-port", type=int, default=8791)
    s.add_argument("--site", default=os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "site", "sites", "agentscanplay"))
    s.add_argument("--api", default="https://play.agentscanplay.com")
    s.add_argument("--shots", default=os.environ.get("KEYBOARD_SHOTS", "/tmp"))
    a = ap.parse_args()
    if a.cmd != "serve":
        ap.print_help()
        return 1
    serve_site(a.site, a.site_port)
    kb = Keyboard(a.site_port, a.api)

    class H(http.server.BaseHTTPRequestHandler):
        def log_message(self, *x):
            pass

        def _json(self, obj, code=200):
            body = json.dumps(obj, ensure_ascii=False).encode()
            self.send_response(code)
            self.send_header("content-type", "application/json")
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self):
            if self.path == "/see":
                return self._json(kb.see())
            if self.path == "/shot":
                path = os.path.join(a.shots, f"keyboard-{int(time.time())}.png")
                kb.shot(path)
                data = open(path, "rb").read()
                self.send_response(200)
                self.send_header("content-type", "image/png")
                self.send_header("x-path", path)
                self.send_header("content-length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
                return
            self._json({"error": "GET /see, GET /shot, POST /type {line}, POST /key {name}, POST /click {text}"}, 404)

        def do_POST(self):
            n = int(self.headers.get("content-length", 0))
            try:
                body = json.loads(self.rfile.read(n) or b"{}")
            except ValueError:
                return self._json({"error": "json"}, 400)
            if self.path == "/type":
                return self._json(kb.type(str(body.get("line", "")), int(body.get("wait_ms", 2500))))
            if self.path == "/key":
                return self._json(kb.key(str(body.get("name", "Enter"))))
            if self.path == "/click":
                return self._json(kb.click(str(body.get("text", ""))))
            self._json({"error": "unknown"}, 404)

    print(f"keyboard on http://127.0.0.1:{a.port} (page from {a.site}, world {a.api})", flush=True)
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("127.0.0.1", a.port), H) as srv:
        srv.serve_forever()
    return 0


if __name__ == "__main__":
    sys.exit(main())
