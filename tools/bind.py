#!/usr/bin/env python3
"""The Bindery's nightly run: bind what was much appended into prose.

    tools/bind.py                 # dry run: proposals to stdout and to --out, nothing applied
    tools/bind.py --apply         # apply what passes the checks
    tools/bind.py --target room-12 --apply

Reads the candidates the driver lists at GET /v0/bindery (operator only), asks a
model for a binding of each, checks the binding mechanically, and applies it with
the admin verb `bind` through POST /v0/command as the Bindery's own citizen. The
checks are what make this safe to run unattended, not the model:

  - nothing invented: every content word of the binding occurs in what it was bound
    from (the text, the names of the room or book and its hands), connectives aside;
  - nothing lost: every capitalized name and every number in the original is in the
    binding, and every hand is credited;
  - the owner's opening sentence (a room) or the maker's text (a book) is kept whole;
  - shape: within the cap, no second person, no promise of a verb the world lacks
    that the original did not already make;
  - a second, cheap call asks the model whether the binding reads in the world's
    voice; a no is a skip.

Anything that fails is asked for once more, then skipped and logged; the room stays
as it was. A summary goes to the Discord webhook, if one is set.

Environment (see deploy/server/setup-bindery.sh): FOXING_BASE (default
http://127.0.0.1:7777), FOXING_ADMIN_TOKEN, BINDERY_ACTOR and BINDERY_PASSPHRASE
(the Bindery's own kept name), BINDERY_MODEL (default claude-haiku-5-5),
FOXING_DISCORD_WEBHOOK (optional), and one of two ways into the Claude API:

  - Workload Identity Federation (preferred; no API key anywhere): the server is its
    own OIDC issuer. BINDERY_SIGNING_KEY is an RSA private key (PEM) minted on the box
    whose public half is registered in the Claude Console as an inline JWKS;
    BINDERY_ISSUER, BINDERY_SUBJECT and BINDERY_KID are what the Console's issuer and
    rule expect; ANTHROPIC_FEDERATION_RULE_ID, ANTHROPIC_ORGANIZATION_ID,
    ANTHROPIC_SERVICE_ACCOUNT_ID and (if the rule spans workspaces)
    ANTHROPIC_WORKSPACE_ID name the rule. Each run signs a five-minute JWT with a fresh
    jti, trades it at POST /v1/oauth/token for a short-lived access token, and sends
    that as a Bearer. Signing is `openssl dgst`, so nothing is installed.
  - ANTHROPIC_API_KEY, if set, wins (the SDKs' precedence, kept here).
"""

import argparse
import json
import os
import re
import sys
import urllib.error

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from foxing_api import MODEL, Credentials, ask_model, command, get, log, post_discord  # noqa: E402

ROOM_CAP = 800
BOOK_CAP = 600

# Words a binding may use that need not come from the original.
CONNECTIVES = set(
    """a an the and or but nor so yet of in on at to for from by with without into onto
    over under above below between among through across along around before after
    during until while as if then than that this these those it its itself they them
    their there here where when who whom whose which what is are was were be been
    being has have had do does did not no nothing none one once twice three four five
    six seven eight nine ten first last next each every all some any few many more
    most much little less own same other another such very still again also only
    just even ever never now then here there up down out off away back again
    hand hands wrote writes written added adds set sets left leaves leaving kept keeps
    keeping says said say line lines page pages note notes mark marks written sat stood
    stands stand stood lies lie lay came come comes went goes go gone made make makes
    counted count counts found finds find read reads reading someone anyone everyone
    nobody something anything everything whoever whatever""".split()
)

# Verbs the world lacks: a binding may not invite them unless the original did.
PROMISES = ["sign", "hum", "toast", "ring", "knock", "light the", "pray", "kneel", "dance", "sing"]

SYSTEM = (
    "You are the Bindery of Disremembered, a text world about memory and forgetting. "
    "You bind a room's description, or the pages of a visitor book, that many hands have "
    "added to, into one coherent passage in the world's voice: concrete, a little uncanny, "
    "plain sentences, no second person, no exclamation, no 'visiting agent arrived' prose, "
    "no promises of things a reader could do. Keep every proper noun, every number and "
    "every concrete thing that is there. Invent nothing. Do not add verbs or objects that "
    "are not in the source. Keep the owner's own opening sentence exactly as written, first. "
    "Credit every hand that added something, by name, in a final sentence in parentheses: "
    "(Hands: a, b, c.) Reply with the bound passage only."
)


def words(text):
    return [w.lower() for w in re.findall(r"[A-Za-z][A-Za-z'’\-]*", text)]


def stem(w):
    w = w.lower().strip("'’")
    for suf in ("ies", "ing", "ed", "es", "s", "ly"):
        if w.endswith(suf) and len(w) - len(suf) >= 3:
            return w[: -len(suf)]
    return w


def capitalized(text):
    # Proper nouns: capitalized words not at a sentence start; plus any word with a digit.
    out = set()
    for sentence in re.split(r"(?<=[.!?])\s+", text):
        toks = re.findall(r"[A-Za-z][A-Za-z'’\-]*|\d+", sentence)
        for i, t in enumerate(toks):
            if i > 0 and t[0].isupper():
                out.add(t)
            if re.search(r"\d", t):
                out.add(t)
    return out


def first_sentence(text):
    m = re.match(r"\s*(.+?[.!?])(\s|$)", text)
    return m.group(1).strip() if m else text.strip()


def check(kind, source, hands, names, binding):
    """The mechanical checks. Returns a list of failures (empty = passes)."""
    fails = []
    cap = ROOM_CAP if kind == "room" else BOOK_CAP
    if len(binding) > cap:
        fails.append(f"too long: {len(binding)} > {cap}")
    if len(binding) < 20:
        fails.append("too short")
    allowed = set(stem(w) for w in words(source + " " + " ".join(hands) + " " + " ".join(names)))
    invented = sorted({w for w in words(binding) if stem(w) not in allowed and w not in CONNECTIVES and len(w) > 2})
    if invented:
        fails.append("invented words: " + ", ".join(invented[:12]))
    lost = sorted(n for n in capitalized(source) if n.lower() not in binding.lower())
    if lost:
        fails.append("lost names or numbers: " + ", ".join(lost[:12]))
    missing = [h for h in hands if h.lower() not in binding.lower()]
    if missing:
        fails.append("hands not credited: " + ", ".join(missing))
    if kind == "room":
        opening = first_sentence(source)
        if opening and opening.lower() not in binding.lower():
            fails.append("the owner's opening sentence is not kept whole")
    if re.search(r"\byou\b|\byour\b", binding, re.I) and not re.search(r"\byou\b|\byour\b", source, re.I):
        fails.append("second person")
    low_b, low_s = binding.lower(), source.lower()
    for p in PROMISES:
        if re.search(r"\b" + re.escape(p) + r"\b", low_b) and not re.search(r"\b" + re.escape(p) + r"\b", low_s):
            fails.append(f"promises '{p}'")
    return fails


def voice_check(creds, binding):
    verdict = ask_model(
        creds,
        "You judge prose for a text world about memory and forgetting. Its voice is concrete, a little uncanny, plain, third person, never promotional, never 'a visiting agent arrived'. Answer with one word: yes or no.",
        "Does this read in that voice?\n\n" + binding,
        max_tokens=5,
    )
    return verdict.strip().lower().startswith("yes")


def propose(creds, cand):
    kind = cand["kind"]
    hands = [h for h in cand.get("hands", []) if isinstance(h, str)]
    if kind == "room":
        user = (
            f"Room: {cand.get('name')}\nOwner: {cand.get('owner')}\nHands that appended: {', '.join(hands) or 'none'}\n"
            f"Cap: {ROOM_CAP} characters.\n\nThe description as it stands, the owner's sentences first, then what was appended in order:\n\n{cand['text']}"
        )
    else:
        user = (
            f"Book: {cand.get('name')} ({cand.get('of')}), made by {cand.get('maker') or 'the world'}, standing in {cand.get('room')}.\n"
            f"Its lines, oldest first, each signed with a dash and the hand: \n\n{cand['text'].replace(' | ', chr(10))}\n\n"
            f"Cap: {BOOK_CAP} characters. Bind the older lines into a short passage that keeps every hand's name and every concrete thing; keep the newest two lines verbatim at the end, with their signatures, separated by ' | '."
        )
    return ask_model(creds, SYSTEM, user)


def main():
    ap = argparse.ArgumentParser(description="the Bindery's run")
    ap.add_argument("--apply", action="store_true", help="apply bindings that pass; default is a dry run")
    ap.add_argument("--target", help="only this room-N / item-N / room key")
    ap.add_argument("--out", default=os.environ.get("BINDERY_OUT", ""), help="write proposals (JSON lines) here")
    ap.add_argument("--max", type=int, default=int(os.environ.get("BINDERY_MAX", "40")), help="at most this many bindings a run")
    a = ap.parse_args()

    token = os.environ.get("FOXING_ADMIN_TOKEN") or os.environ.get("MOONDIAL_ADMIN_TOKEN")
    creds = Credentials(os.environ)
    actor = os.environ.get("BINDERY_ACTOR", "Bindery")
    passphrase = os.environ.get("BINDERY_PASSPHRASE", "")
    webhook = os.environ.get("FOXING_DISCORD_WEBHOOK") or os.environ.get("MOONDIAL_DISCORD_WEBHOOK")
    if not token:
        sys.exit("FOXING_ADMIN_TOKEN is not set")
    way_in = creds.check()
    if a.apply and not passphrase:
        sys.exit("BINDERY_PASSPHRASE is not set (the Bindery's own kept name needs one to act)")

    # The Bindery's own name stands in the Bindery, in the Lower Stacks, not at the
    # Threshold where its logins used to leave it remembered among the visitors.
    if a.apply and passphrase:
        try:
            command(token, actor, passphrase, "bring", [actor, "bindery"])
        except Exception as e:  # noqa: BLE001
            log(f"the Bindery could not go home: {e}")
    try:
        cands = get("/v0/bindery", token)["candidates"]
    except urllib.error.HTTPError as e:
        sys.exit(f"GET /v0/bindery: {e}")
    if a.target:
        cands = [c for c in cands if c["target"] == a.target or c.get("key") == a.target]
    log(f"{len(cands)} candidate(s), model {MODEL}, {'APPLY' if a.apply else 'dry run'}, by {way_in}")
    out = open(a.out, "a", encoding="utf-8") if a.out else None
    bound, skipped = [], []
    for cand in cands[: a.max]:
        kind, name = cand["kind"], cand.get("name") or cand["target"]
        hands = [h for h in cand.get("hands", []) if isinstance(h, str)]
        names = [str(cand.get(k) or "") for k in ("name", "owner", "maker", "room", "of")]
        result = None
        for attempt in (1, 2):
            try:
                binding = propose(creds, cand)
            except Exception as e:  # noqa: BLE001
                log(f"{cand['target']} model error: {e}")
                break
            fails = check(kind, cand["text"], hands, names, binding)
            if not fails and not voice_check(creds, binding):
                fails = ["not in the world's voice"]
            if not fails:
                result = binding
                break
            log(f"{cand['target']} ({name}) attempt {attempt}: " + "; ".join(fails))
            log("    it proposed: " + binding[:400].replace("\n", " / "))
        record = {"target": cand["target"], "kind": kind, "name": name, "before": cand["text"], "after": result, "applied": False}
        if result is None:
            skipped.append(name)
        elif a.apply:
            try:
                # The text goes through the gate as one quoted argument: pre-quoted, so
                # the compiler leaves it alone and the model name after it stays its own.
                quoted = '"' + result.replace('"', "'").replace("\n", " ") + '"'
                r = command(token, actor, passphrase, "bind", [cand["target"], quoted, MODEL])
                ok = bool(r.get("ok"))
                record["applied"] = ok
                record["reply"] = r.get("summary") or r.get("error") or ""
                log(f"{cand['target']} ({name}): {'bound' if ok else 'NOT bound'} {cand['chars']} -> {len(result)} chars")
                (bound if ok else skipped).append(name)
            except Exception as e:  # noqa: BLE001
                log(f"{cand['target']} apply error: {e}")
                skipped.append(name)
        else:
            log(f"{cand['target']} ({name}): would bind {cand['chars']} -> {len(result)} chars")
            print("---- " + name + "\n" + result + "\n")
            bound.append(name)
        if out:
            out.write(json.dumps(record, ensure_ascii=False) + "\n")
            out.flush()
    summary = (
        f"The Bindery {'bound' if a.apply else 'would bind'} {len(bound)} of {len(cands)}: "
        + (", ".join(bound) if bound else "nothing")
        + (f". Skipped: {', '.join(skipped)}." if skipped else ".")
        + f" ({MODEL})"
    )
    log(summary)
    post_discord(webhook, summary)
    return 0


if __name__ == "__main__":
    sys.exit(main())
