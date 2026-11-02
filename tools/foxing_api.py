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


def ask_model(creds, system, user, max_tokens=700, model=None):
    body = json.dumps(
        {
            "model": model or MODEL,
            "max_tokens": max_tokens,
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
