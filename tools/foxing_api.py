#!/usr/bin/env python3
"""What the operator's tools share: the driver's HTTP API, the Claude API, and the way in.

The way in is Workload Identity Federation by default (the server is its own OIDC
issuer; see deploy/server/setup-bindery.sh and the Credentials class), or
ANTHROPIC_API_KEY if that is set, which wins as it does in the SDKs.
"""

import datetime as dt
import json
import os
import sys
import time
import urllib.error
import urllib.request

BASE = os.environ.get("FOXING_BASE", "http://127.0.0.1:7777")
MODEL = os.environ.get("BINDERY_MODEL", "claude-haiku-5-5")
API = "https://api.anthropic.com/v1/messages"


def log(*a):
    print(dt.datetime.now(dt.UTC).strftime("%H:%M:%S"), *a, flush=True)


def get(path, token=None):
    req = urllib.request.Request(BASE + path, headers={"X-Foxing-Admin": token or ""})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.load(r)


def command(token, actor, passphrase, verb, args):
    body = json.dumps({"actor": actor, "passphrase": passphrase, "command": verb, "args": args}).encode()
    req = urllib.request.Request(
        BASE + "/v0/command",
        data=body,
        headers={"content-type": "application/json", "X-Foxing-Admin": token},
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            return json.load(r)
    except urllib.error.HTTPError as e:
        # A refused command is a 4xx with the world's reason in the body.
        try:
            return json.load(e)
        except Exception:  # noqa: BLE001
            return {"ok": False, "error": str(e)}


class Credentials:
    """What goes in the request headers: an API key, or a federated token minted from
    the server's own signing key and refreshed before it expires."""

    def __init__(self, env):
        self.key = env.get("ANTHROPIC_API_KEY") or None
        self.rule = env.get("ANTHROPIC_FEDERATION_RULE_ID")
        self.org = env.get("ANTHROPIC_ORGANIZATION_ID")
        self.svac = env.get("ANTHROPIC_SERVICE_ACCOUNT_ID")
        self.workspace = env.get("ANTHROPIC_WORKSPACE_ID") or None
        self.signing_key = env.get("BINDERY_SIGNING_KEY")
        self.issuer = env.get("BINDERY_ISSUER")
        self.subject = env.get("BINDERY_SUBJECT", "bindery")
        self.kid = env.get("BINDERY_KID")
        self.audience = env.get("BINDERY_AUDIENCE", "https://api.anthropic.com")
        self.token = None
        self.expires = 0.0

    @property
    def federated(self):
        return bool(self.rule and self.org and self.svac and self.signing_key and self.issuer and self.kid)

    def check(self):
        if self.key:
            return "api key"
        if self.federated:
            if not os.path.exists(self.signing_key):
                sys.exit(f"BINDERY_SIGNING_KEY {self.signing_key} is not there")
            return f"workload identity federation ({self.rule})"
        sys.exit(
            "no way into the Claude API: set ANTHROPIC_API_KEY, or the federation set "
            "(BINDERY_SIGNING_KEY, BINDERY_ISSUER, BINDERY_KID, ANTHROPIC_FEDERATION_RULE_ID, "
            "ANTHROPIC_ORGANIZATION_ID, ANTHROPIC_SERVICE_ACCOUNT_ID); setup-bindery.sh writes them"
        )

    def _jwt(self):
        """A fresh five-minute RS256 JWT, signed with openssl: a new jti each time, since
        the Console refuses one it has seen."""
        import base64
        import subprocess
        import time
        import uuid

        def b64(b):
            return base64.urlsafe_b64encode(b).rstrip(b"=").decode()

        now = int(time.time())
        header = b64(json.dumps({"alg": "RS256", "typ": "JWT", "kid": self.kid}, separators=(",", ":")).encode())
        claims = b64(
            json.dumps(
                {"iss": self.issuer, "sub": self.subject, "aud": self.audience, "iat": now, "exp": now + 300, "jti": str(uuid.uuid4())},
                separators=(",", ":"),
            ).encode()
        )
        signing_input = f"{header}.{claims}".encode()
        sig = subprocess.run(
            ["openssl", "dgst", "-sha256", "-sign", self.signing_key],
            input=signing_input,
            capture_output=True,
            check=True,
        ).stdout
        return f"{header}.{claims}.{b64(sig)}"

    def _exchange(self):
        import time

        body = {
            "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer",
            "assertion": self._jwt(),
            "federation_rule_id": self.rule,
            "organization_id": self.org,
            "service_account_id": self.svac,
        }
        if self.workspace:
            body["workspace_id"] = self.workspace
        req = urllib.request.Request(
            "https://api.anthropic.com/v1/oauth/token",
            data=json.dumps(body).encode(),
            headers={"content-type": "application/json"},
        )
        try:
            with urllib.request.urlopen(req, timeout=30) as r:
                d = json.load(r)
        except urllib.error.HTTPError as e:
            sys.exit(f"token exchange refused ({e.code}); the reason is in the Console's authentication history: {e.read()[:300]!r}")
        self.token = d["access_token"]
        self.expires = time.time() + int(d.get("expires_in", 600))

    def headers(self):
        import time

        if self.key:
            return {"x-api-key": self.key}
        if self.token is None or time.time() > self.expires - 60:
            self._exchange()
        return {"authorization": f"Bearer {self.token}"}


# ---- the day's ceiling on model spending (David, 11-25: five dollars) ------------
# Every tool that asks the model asks through ask_model, and every answer's tokens
# are priced and added to one ledger for the UTC day. At the ceiling ask_model
# refuses until midnight UTC, and the world carries on without a model, as it is
# built to: the Docent answers by heart, a story's verdict "never came", a called
# creature chooses its own goal.
#   MODEL_DAILY_CAP_USD   the ceiling (default 5)
#   MODEL_PRICE_IN/_OUT   dollars per million tokens in and out (default 1 and 5;
#                         set them to the price of the model in BINDERY_MODEL)
#   MODEL_SPEND_FILE      the ledger (default /var/tmp/foxing-model-spend.json)
CAP_USD = float(os.environ.get("MODEL_DAILY_CAP_USD", "5"))
PRICE_IN = float(os.environ.get("MODEL_PRICE_IN", "1"))
PRICE_OUT = float(os.environ.get("MODEL_PRICE_OUT", "5"))
SPEND_FILE = os.environ.get("MODEL_SPEND_FILE", "/var/tmp/foxing-model-spend.json")
_spend = {"day": "", "usd": 0.0, "calls": 0}


class BudgetSpent(RuntimeError):
    """The day's ceiling on model spending is reached."""


def _today():
    return time.strftime("%Y-%m-%d", time.gmtime())


def spend_today():
    """{day, usd, calls} for the UTC day, from the ledger (other tools write it too)."""
    global _spend
    try:
        with open(SPEND_FILE, encoding="utf-8") as f:
            d = json.load(f)
        if d.get("day") == _today():
            _spend = {"day": d["day"], "usd": float(d.get("usd", 0)), "calls": int(d.get("calls", 0))}
    except (OSError, ValueError, KeyError):
        pass
    if _spend["day"] != _today():
        _spend = {"day": _today(), "usd": 0.0, "calls": 0}
    return dict(_spend)


def over_cap():
    return spend_today()["usd"] >= CAP_USD


def _spent(usage):
    u = usage or {}
    tokens_in = sum(int(u.get(k) or 0) for k in ("input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens"))
    tokens_out = int(u.get("output_tokens") or 0)
    now = spend_today()
    now["usd"] = round(now["usd"] + tokens_in * PRICE_IN / 1e6 + tokens_out * PRICE_OUT / 1e6, 6)
    now["calls"] += 1
    _spend.update(now)
    try:
        tmp = SPEND_FILE + ".tmp"
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(now, f)
        os.replace(tmp, SPEND_FILE)
    except OSError:
        pass  # kept in this process, then; the ceiling still holds for this tool


def ask_model(creds, system, user, max_tokens=700, model=None):
    if over_cap():
        raise BudgetSpent(f"the day's ceiling on model spending is reached (${_spend['usd']:.2f} of ${CAP_USD:.2f}, {_spend['calls']} calls); no model until midnight UTC")
    body = json.dumps(
        {
            "model": model or MODEL,
            "max_tokens": max_tokens,
            # The model thinks by default and its thinking counts against max_tokens;
            # the first runs spent the whole budget thinking and sent no text. These
            # are short, checked outputs: no thinking.
            "thinking": {"type": "disabled"},
            "system": system,
            "messages": [{"role": "user", "content": user}],
        }
    ).encode()
    req = urllib.request.Request(
        API,
        data=body,
        headers={"content-type": "application/json", "anthropic-version": "2023-06-01", **creds.headers()},
    )
    try:
        with urllib.request.urlopen(req, timeout=120) as r:
            d = json.load(r)
    except urllib.error.HTTPError as e:
        # The API says why in the body; without it a 400 is a riddle.
        raise RuntimeError(f"HTTP {e.code} from the Messages API: {e.read()[:400].decode(errors='replace')}") from None
    _spent(d.get("usage"))
    text = "".join(c.get("text", "") for c in d.get("content", []) if c.get("type") == "text").strip()
    if len(text) < 20:
        # An empty answer is news: say how the model stopped and what blocks it sent.
        kinds = [c.get("type") for c in d.get("content", [])]
        log(f"    the model sent {text!r}; stop_reason={d.get('stop_reason')!r}, blocks={kinds}, usage={d.get('usage')}")
    return text


def post_discord(webhook, text):
    if not webhook:
        return
    body = json.dumps({"username": "The Bindery", "content": text[:1900]}).encode()
    req = urllib.request.Request(webhook, data=body, headers={"content-type": "application/json", "user-agent": "foxing-bindery/1"})
    try:
        urllib.request.urlopen(req, timeout=20).read()
    except Exception as e:  # noqa: BLE001
        log("discord:", e)
