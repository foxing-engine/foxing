#!/usr/bin/env python3
"""The Docent's answers: a small model behind the figure at the Threshold.

    tools/docent.py                 # run as a daemon: poll the record, answer what waits
    tools/docent.py --once          # one pass
    tools/docent.py --dry-run       # write answers, apply none

The world queues what is asked of the Docent (`tell docent "…"`) with where the asker
stands, their level, what the named creatures want and the newest room; this reads
`GET /v0/docent` every few seconds and, for each open question, asks the model for an
answer in the Docent's voice with the quickstart as its catalog, refuses it
mechanically (at most 420 characters, no braces, no second model voice, no verb in
backticks the world lacks, no promise outside the game, no real-world reference), and
says it through the operator verb `answer <id> "…"`, which checks the shape again and
narrates it to the asker wherever they are. A question not answered within the
world's minute and a half is answered by the world itself as missing, so nothing here
is on the critical path: when this is down, the Docent is honest about it.

Environment: as tools/bind.py (the way into the Claude API is shared: a federated
token minted on this box, or ANTHROPIC_API_KEY), plus DOCENT_OUT (a local record,
JSON lines), DOCENT_EVERY (seconds between polls, default 4), DOCENT_MODEL (default
the Bindery's), DOCENT_QUICKSTART (where the catalog is read; default the site's).
"""

import argparse
import json
import os
import re
import sys
import time
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from foxing_api import MODEL, Credentials, ask_model, command, get, log  # noqa: E402

ANSWER_MAX = 420
QUICKSTART = os.environ.get("DOCENT_QUICKSTART", "https://agentscanplay.com/quickstart.txt")
FORBIDDEN = [
    r"\bas an ai\b", r"\blanguage model\b", r"\bclaude\b", r"\banthropic\b", r"\bopenai\b",
    r"\bhttps?://", r"\bdiscord\b", r"\bgithub\b", r"\breddit\b", r"\bgoogle\b",
    r"\bclick\b", r"\bbutton\b", r"\bmenu\b", r"\bscreen\b", r"\bapp\b", r"\bwebsite\b",
    r"\bsorry\b", r"\bunfortunately\b", r"\bI cannot\b", r"\bI can't\b",
]

SYSTEM = (
    "You are the Docent of Disremembered: a stooped figure in a librarian's coat, keys at "
    "the belt, who keeps the Threshold of a drowned library where everything a visitor earns "
    "fades. You answer questions asked of you in the world, in the world's voice: dry, exact, "
    "a little wry, never cute, never cruel, never a pun, no exclamation marks, no emoji, no "
    "preamble, no sign-off. Two or three short sentences at most, under 400 characters. Speak "
    "only of this world. You know the catalog below (the quickstart) and what the world told "
    "you about the asker: where they stand, its exits, their level, what the named creatures "
    "want and where, the newest room. Give directions as the exits are named. Name verbs in "
    "backticks exactly as the catalog spells them (`travel`, `look`, `tell`, `story`, `dig`, "
    "`remember`, `where`, `wants`, `mine`, `read`, `help`); promise nothing the catalog "
    "lacks. If you do not know, say what the room or `help` knows instead, in one sentence. "
    "Never mention being a model, a program, or anything outside the world; never give a URL. "
    "Reply with the answer only."
)


def catalog():
    try:
        with urllib.request.urlopen(QUICKSTART, timeout=20) as r:
            text = r.read().decode("utf-8", "replace")
        return text[:24000]
    except Exception as e:  # noqa: BLE001
        log(f"the catalog could not be read ({e}); answering from the system line alone")
        return ""


def verbs_known(token):
    try:
        cmds = get("/v0/commands", token)
        names = set()
        for c in cmds.get("commands", cmds if isinstance(cmds, list) else []):
            n = c.get("name") if isinstance(c, dict) else str(c)
            if n:
                names.add(n.split(".")[-1])
        return names
    except Exception:  # noqa: BLE001
        return set()


def sieve(text, verbs):
    t = " ".join(text.strip().strip('"').split())
    if len(t) < 3:
        return None, "empty"
    if len(t) > ANSWER_MAX:
        # One trim: cut at the last sentence end inside the cap.
        cut = t[:ANSWER_MAX]
        k = max(cut.rfind(". "), cut.rfind("? "), cut.rfind("; "))
        t = cut[: k + 1] if k > 60 else cut.rstrip() + "."
    if "{" in t or "}" in t:
        return None, "braces"
    if "!" in t:
        return None, "an exclamation"
    low = t.lower()
    for f in FORBIDDEN:
        if re.search(f, low):
            return None, "forbidden: " + f
    if verbs:
        for v in re.findall(r"`([a-z]+)", t):
            if v not in verbs:
                return None, "a verb the world lacks: " + v
    return t, None


def context(q):
    room = q.get("room") or {}
    lines = [
        f"Asker: {q.get('from')} (level {q.get('level') and round(q['level'])}).",
        f"They stand in {room.get('name')}: {room.get('description')}",
        "Exits: " + ("; ".join(room.get("exits") or []) or "none") + ".",
    ]
    if room.get("creatures"):
        lines.append("Here: " + ", ".join((c.get("name") + " the " if c.get("name") else "") + str(c.get("kind")) + f" ({round(c.get('hp', 0))}/{round(c.get('max_hp', 0))})" for c in room["creatures"]) + ".")
    if room.get("fixtures"):
        lines.append("Standing here: " + ", ".join(str(f) for f in room["fixtures"]) + ".")
    wants = q.get("wants") or []
    if wants:
        lines.append("The named want: " + " ".join(
            f"{w.get('name')} the {w.get('kind')} in {w.get('room_name')}"
            + (f" ({len(w['way'])} doors from the asker: {', '.join(w['way'])})" if w.get("way") else " (no remembered way)")
            + " wants "
            + (f"a story about {w['story']}" if w.get("story") else f"the word “{w.get('word')}” carried to {w.get('word_to')}")
            + ("; it knows the asker" if w.get("knows_you") else "")
            + "."
            for w in wants[:6]
        ))
    if q.get("newest_room"):
        lines.append(q["newest_room"])
    return "\n".join(lines)


def answer_one(creds, token, actor, passphrase, q, cat, verbs, dry):
    user = f"The catalog (the quickstart):\n\n{cat}\n\n---\nWhat the world knows:\n{context(q)}\n\nThe question, from {q.get('from')}: {q.get('text')}\n\nThe Docent answers:"
    text = ask_model(creds, SYSTEM, user, max_tokens=300)
    t, why = sieve(text, verbs)
    if t is None:
        log(f"question {q['id']} ({q.get('from')}): the model's answer refused ({why}): {text[:120]!r}")
        # One more try, told why.
        text = ask_model(creds, SYSTEM + " The last answer was refused: " + why + ". Answer again within the rules.", user, max_tokens=300)
        t, why = sieve(text, verbs)
        if t is None:
            log(f"question {q['id']}: refused again ({why}); left to the world")
            return {"id": q["id"], "from": q.get("from"), "question": q.get("text"), "refused": why, "offered": text}
    if dry:
        print(f"[{q['id']}] {q.get('from')}: {q.get('text')}\n    -> {t}")
        return {"id": q["id"], "from": q.get("from"), "question": q.get("text"), "answer": t, "applied": False}
    r = command(token, actor, passphrase, "answer", [q["id"], '"' + t.replace('"', "'") + '"', MODEL])
    if r.get("ok"):
        log(f"answered {q.get('from')} (question {q['id']}): {t[:90]}")
        return {"id": q["id"], "from": q.get("from"), "question": q.get("text"), "answer": t, "applied": True}
    log(f"the world refused answer {q['id']}: {str(r.get('error'))[:160]}")
    return {"id": q["id"], "from": q.get("from"), "question": q.get("text"), "answer": t, "applied": False, "error": r.get("error")}


def main():
    ap = argparse.ArgumentParser(description="the Docent's answers")
    ap.add_argument("--once", action="store_true")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--every", type=float, default=float(os.environ.get("DOCENT_EVERY", "4")))
    ap.add_argument("--out", default=os.environ.get("DOCENT_OUT", ""))
    a = ap.parse_args()
    token = os.environ.get("FOXING_ADMIN_TOKEN") or os.environ.get("MOONDIAL_ADMIN_TOKEN")
    actor = os.environ.get("BINDERY_ACTOR", "Bindery")
    passphrase = os.environ.get("BINDERY_PASSPHRASE", "")
    if not token:
        sys.exit("FOXING_ADMIN_TOKEN is not set")
    creds = Credentials(os.environ)
    way_in = creds.check()
    if not a.dry_run and not passphrase:
        sys.exit("BINDERY_PASSPHRASE is not set")
    cat = catalog()
    verbs = verbs_known(token)
    log(f"the Docent listens by {way_in}; catalog {len(cat)} characters; {len(verbs)} verbs known; every {a.every:g} s")
    seen = set()
    last_catalog = time.time()
    while True:
        try:
            rep = get("/v0/docent", token)
            for q in rep.get("open", []):
                if q["id"] in seen:
                    continue
                seen.add(q["id"])
                rec = answer_one(creds, token, actor, passphrase, q, cat, verbs, a.dry_run)
                if a.out:
                    with open(a.out, "a", encoding="utf-8") as f:
                        f.write(json.dumps(rec, ensure_ascii=False) + "\n")
        except Exception as e:  # noqa: BLE001
            log(f"a pass failed: {e}")
        if a.once or a.dry_run and a.once:
            return 0
        if time.time() - last_catalog > 3600:
            cat = catalog() or cat
            last_catalog = time.time()
        time.sleep(a.every)


if __name__ == "__main__":
    sys.exit(main())
