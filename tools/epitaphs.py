#!/usr/bin/env python3
"""Keep the book of epitaphs stocked: what the world says over a fall.

    tools/epitaphs.py                # stock when the book is low
    tools/epitaphs.py --dry-run      # show what would be stocked
    tools/epitaphs.py --force --count 24

An epitaph is one or two sentences, poignant and a little snarky, said to the one who
fell, with slots the world fills: {killer} (the creature, with its article: "the
hollow" or "Vellic the wispling"), {room} (where you fell), {lost} (what it cost, in
XP), {fall} (first, second, third of the night), {name} (yours). The model writes a
batch; these checks refuse what fails (24 to 180 characters, only those slots, no
unclosed brace, no sentence that promises a verb the world lacks, no real names: a
capitalized word is allowed only at a sentence start, as a slot, or from a short list);
a second call judges the voice; what passes is stocked one line at a time through the
operator verb `epitaph`, which runs the shape checks again.

Environment: as tools/bind.py (the way in is shared), plus EPITAPHS_OUT (a local
cache, JSON lines), EPITAPHS_LOW (stock when fewer than this; default the world's
mark), EPITAPHS_BATCH (how many to ask for, default 24).
"""

import argparse
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from foxing_api import MODEL, Credentials, ask_model, command, get, log, post_discord  # noqa: E402

SLOTS = ["{killer}", "{room}", "{lost}", "{fall}", "{name}"]
ALLOWED_CAPS = set("The Threshold Stacks Conclave Clocktower Bindery Fogbank Midden Shallows Archive You Your Nothing Everything Mind Down Felling Only Concussion I".split())
PROMISES = ["sign", "pray", "kneel", "sing", "dance", "toast", "light a", "ring the", "press", "click", "respawn", "reload", "save"]

SYSTEM = (
    "You write epitaphs for Disremembered, a text world about memory and forgetting: a "
    "drowned library where everything you earn fades, and a fall costs you a share of what "
    "you were carrying, which the creature that felled you then keeps. An epitaph is said "
    "to the one who fell, right as they wake at the Threshold: one or two short sentences, "
    "poignant and a little snarky, dry, never cruel, never cute, never a pun, no exclamation "
    "marks, no emoji, no jokes about real things outside the world. Use these slots where "
    "they help, exactly as written and no others: {killer} (the creature that felled you, "
    "with its article, so write it without one: '{killer} has the last ten seconds of you'), "
    "{room} (the room's name), {lost} (a number of XP), {fall} (first, second, third), "
    "{name} (the one who fell). Second person is right. Do not promise anything the one who "
    "fell could do except fight, remember, tell a story, dig, write, walk. No real names, no "
    "capitalized words except at a sentence start. Examples of the register: "
    "'{killer} has the last ten seconds of you. You don't. Mind the stairs.' "
    "'Down, in {room}. The water is starting to recognize you, which is more than you can say.' "
    "'The {fall} fall of the night. The Threshold has stopped asking your name.' "
    "Reply with the epitaphs only, one per line, no numbering, no quotes, no commentary."
)


def sieve(line, have, cache):
    t = line.strip().lstrip("-•*0123456789. ").strip().strip('"').strip()
    if not t:
        return None, "empty"
    n = len(t)
    if n < 24:
        return None, "too short"
    if n > 180:
        return None, "too long"
    if "!" in t:
        return None, "an exclamation"
    rest = t
    while "{" in rest:
        i = rest.index("{")
        j = rest.find("}", i)
        if j < 0:
            return None, "an unclosed brace"
        if rest[i:j + 1] not in SLOTS:
            return None, "an unknown slot " + rest[i:j + 1]
        rest = rest[j + 1:]
    # Capitalized words only at a sentence start, in a slot, or from the short list.
    plain = re.sub(r"\{[a-z]+\}", "x", t)
    for sentence in re.split(r"(?<=[.!?])\s+", plain):
        toks = re.findall(r"[A-Za-z][A-Za-z'’]*", sentence)
        for k, tok in enumerate(toks):
            if k > 0 and tok[0].isupper() and tok not in ALLOWED_CAPS:
                return None, "a name: " + tok
    low = t.lower()
    for p in PROMISES:
        if re.search(r"\b" + re.escape(p) + r"\b", low):
            return None, "promises '" + p + "'"
    if any(h.lower() == low for h in have):
        return None, "already stocked"
    if low in cache:
        return None, "offered before"
    return t, None


def voice_ok(creds, line):
    v = ask_model(
        creds,
        "You judge lines for a text world about memory and forgetting. The register is dry, poignant, a little snarky, never cruel, never cute, never a pun, nothing from outside the world. Answer with one word: yes or no.",
        "Is this epitaph in that register?\n\n" + line,
        max_tokens=5,
    )
    return v.strip().lower().startswith("yes")


def main():
    ap = argparse.ArgumentParser(description="keep the book of epitaphs stocked")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--force", action="store_true")
    ap.add_argument("--count", type=int, default=int(os.environ.get("EPITAPHS_BATCH", "24")))
    ap.add_argument("--out", default=os.environ.get("EPITAPHS_OUT", ""))
    a = ap.parse_args()

    token = os.environ.get("FOXING_ADMIN_TOKEN") or os.environ.get("MOONDIAL_ADMIN_TOKEN")
    actor = os.environ.get("BINDERY_ACTOR", "Bindery")
    passphrase = os.environ.get("BINDERY_PASSPHRASE", "")
    webhook = os.environ.get("FOXING_DISCORD_WEBHOOK") or os.environ.get("MOONDIAL_DISCORD_WEBHOOK")
    if not token:
        sys.exit("FOXING_ADMIN_TOKEN is not set")
    creds = Credentials(os.environ)
    way_in = creds.check()
    if not a.dry_run and not passphrase:
        sys.exit("BINDERY_PASSPHRASE is not set")

    book = get("/v0/namery", token)
    have = book.get("epitaphs", [])
    low = int(os.environ.get("EPITAPHS_LOW", book.get("epitaphs_low_at", 24)))
    log(f"the book holds {len(have)} epitaph(s); low mark {low}; {'dry run' if a.dry_run else 'stocking'} by {way_in}")
    if len(have) >= low and not a.force:
        log("enough; nothing asked")
        return 0

    cache = set()
    if a.out and os.path.exists(a.out):
        with open(a.out, encoding="utf-8") as f:
            for line in f:
                try:
                    for t in json.loads(line).get("offered", []):
                        cache.add(t.lower())
                except ValueError:
                    pass
    user = f"Write {a.count} epitaphs. Not these, already in the book:\n" + "\n".join(have[-40:])
    text = ask_model(creds, SYSTEM, user, max_tokens=1800)
    offered = [l for l in text.splitlines() if l.strip()]
    kept, refused = [], []
    for line in offered:
        t, why = sieve(line, have + kept, cache)
        if t is None:
            refused.append((line[:60], why))
            continue
        if not voice_ok(creds, t):
            refused.append((t[:60], "not in the voice"))
            continue
        kept.append(t)
    log(f"the model offered {len(offered)}; {len(kept)} pass" + ("; refused: " + "; ".join(f"{l} ({w})" for l, w in refused[:12]) if refused else ""))
    record = {"model": MODEL, "offered": offered, "kept": kept, "refused": refused, "applied": 0}
    if a.dry_run:
        print("\n".join(kept))
    else:
        for t in kept:
            r = command(token, actor, passphrase, "epitaph", ['"' + t.replace('"', "'") + '"'])
            if r.get("ok"):
                record["applied"] += 1
            else:
                log("the world refused one: " + str(r.get("error"))[:160])
        log(f"stocked {record['applied']} of {len(kept)}")
        if record["applied"]:
            post_discord(webhook, f"The Bindery wrote {record['applied']} epitaphs into the book ({MODEL}); {len(have) + record['applied']} in it now.")
    if a.out:
        with open(a.out, "a", encoding="utf-8") as f:
            f.write(json.dumps(record, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
