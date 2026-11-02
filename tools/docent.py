#!/usr/bin/env python3
"""The Docent's answers: a small model behind the figure at the Threshold.

    tools/docent.py                 # run as a daemon: poll the record, answer what waits
    tools/docent.py --once          # one pass
    tools/docent.py --dry-run       # write answers, apply none
    tools/docent.py --greetings [--dry-run]   # stock the greeting book once and exit

The daemon also keeps the Docent's greeting book: at start and once a day, when a
kind of greeting (first words for a newcomer, a welcome for a name it has met) runs
under the world's low mark, it asks the model for a handful in the Docent's voice,
steered by what visitors asked it in the last day and what the named creatures want
(the owner's ask: player conversations steer the model), sieves them (40 to 260
characters, only the slots {name} {creature} {room}, {name} present, no exclamations,
no real names), and stocks them through the operator verb `greeting`, which checks
the shape again; each kind keeps the newest two dozen, so the book follows the
visitors.

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
    "lacks. The world gives you the signed routes from where the asker stands to every one "
    "of its own rooms, and where each kind of creature lives: when asked where something is, "
    "give that route as the exits are named (\"east, then south\"), never say no route is "
    "held, and when asked where a kind lives name its home room, not a named creature of "
    "that kind (the named are story partners, not prey). `rest` says how you mend; waking "
    "whole is what a fall does. `observe` is a tool outside the world, not a verb: `look` "
    "says what a creature has learned. If you do not know, say what the room or `help` "
    "knows instead, in one sentence. Never mention being a model, a program, or anything "
    "outside the world; never give a URL. Reply with the answer only."
)

# The rules the Docent answers from, not from guesswork: each line is how the world
# behaves, checked against the verbs. A visitor was told a dug room's memory is not
# restored by a visit (it is), and that a carried word "was a story" (it was not).
RULES = (
    "The rules, as the world keeps them. Rooms: `dig <exit> <chunk>` opens a room off "
    "the one the digger stands in; the digger stays where they were, and `travel <exit>` "
    "goes in. A dug room's memory halves at its own rate unless someone visits; any "
    "visit remembers it whole, and `remember <chunk>` inside it lengthens that half-life "
    "for good. Stories: `story <creature> \"…\"` earns a chunk; the story a named "
    "creature wanted is worth double; a story it has heard lately pays by halves each "
    "repeat, and one told this hour comes back later in `wants`. Words: when a named "
    "creature asks for a word carried, `travel` to the other and `tell` it the phrase; "
    "the first carrier earns a chunk on delivery, once; anyone bringing the same word "
    "after that hears it was brought already and earns nothing; the receiver hands the "
    "first carrier a word for the way home, and a word carried home may give birth to a "
    "creature between the two. Names: a creature fed to sixty by `remember <chunk> "
    "<creature>` takes a name and knows who keeps it; below seventy its name is failing. "
    "Falls: a visitor who goes down wakes at the Threshold with what they had; a censor's "
    "ray of forgetfulness halves one carried chunk where it stands. The Docent: asked only at the "
    "Threshold, it finds the asker anywhere within minutes; between visitors it walks "
    "its round through the rooms, tidying what lies about."
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
    routes = q.get("routes") or []
    if routes:
        lines.append("Signed routes from where the asker stands (the world's own rooms, nearest first): " + "; ".join(
            f"{r.get('room')}: {', '.join(r.get('way') or [])}" for r in routes[:40]))
    if q.get("remembered"):
        lines.append("Rooms the asker remembers walking: " + ", ".join(q["remembered"][:30]) + ".")
    homes = q.get("homes") or []
    if homes:
        lines.append("Where each kind lives (and comes back when felled): " + "; ".join(
            f"{h.get('kind')}: {', '.join(h.get('rooms') or [])}" for h in homes))
    record = q.get("record") or []
    if record:
        lines.append("The asker's record up to the moment they asked, oldest first (answer from this, not from the room as it stands now):\n  " + "\n  ".join(record))
    return "\n".join(lines)


GREETING_SYSTEM = (
    "You write the Docent's greetings for Disremembered: a stooped figure in a librarian's "
    "coat who keeps the Threshold of a drowned library where everything a visitor earns "
    "fades. Two kinds. FIRST: said to a name in its first hour, newly arrived; it should "
    "orient them in one or two sentences with ONE concrete first move in backticks (`look`, "
    "`travel east`, `attack wisp`, `chunks`; the strip above the prompt walks the first "
    "hour; `help` is what the Docent keeps by heart; questions go `tell docent \"...\"`), "
    "and point at what newcomers ask about most; at least half the FIRST lines carry "
    "{creature} so the world can name a creature with a want. Never generic comfort "
    "('take it slowly'): a page number, not a pat. WELCOME: said to a name the Docent has met, "
    "coming back to the door; shorter, dry, warm under the dryness, never gushing. Use the "
    "slots exactly as written: {name} (always, once), {creature} (optional: the world fills "
    "it with a named creature's want and the way, a whole sentence, so put it where a "
    "sentence can stand alone), {room} (optional: the world fills it with the newest dug "
    "room, a whole sentence). The voice: dry, exact, a little wry, never cute, never cruel, no "
    "exclamation marks, no emoji, no puns, no real-world references, no capitalized words "
    "except at a sentence start, the slots, and these: The Threshold, Ask, Everything, "
    "Nothing. 40 to 240 characters each. Say only what is true of this world: everything "
    "earned fades; rooms give their exits freely and want nothing; it is the NAMED "
    "CREATURES that want stories (and pay for them); wisps drift in the Fogbank east of the "
    "Threshold and are the first fight; `chunks` is what you hold; a chunk fed to a "
    "creature at sixty gives it a name; `dig` opens a room of your own; the Docent answers "
    "`tell docent \"...\"`. Nothing else is promised. Reply with the lines only, one per "
    "line, each line beginning FIRST: or WELCOME: and nothing else."
)
ALLOWED_CAPS = set("The Threshold Ask Everything Nothing Stacks Fogbank Midden Shallows Archive Clocktower Conclave Bindery Docent I".split())


def greeting_sieve(t, have, extra_caps=()):
    t = " ".join(t.strip().strip('"').split())
    n = len(t)
    if n < 40:
        return None, "too short"
    if n > 240:
        return None, "too long"
    if "!" in t:
        return None, "an exclamation"
    if "{name}" not in t:
        return None, "no {name}"
    rest = t
    while "{" in rest:
        i = rest.index("{")
        j = rest.find("}", i)
        if j < 0:
            return None, "an unclosed brace"
        if rest[i:j + 1] not in ("{name}", "{creature}", "{room}"):
            return None, "an unknown slot " + rest[i:j + 1]
        rest = rest[j + 1:]
    plain = re.sub(r"\{[a-z]+\}", "x", t)
    for sentence in re.split(r"(?<=[.?])\s+", plain):
        toks = re.findall(r"[A-Za-z][A-Za-z'’]*", sentence)
        for k, tok in enumerate(toks):
            if k > 0 and tok[0].isupper() and tok not in ALLOWED_CAPS and tok not in extra_caps:
                return None, "a name: " + tok
    low = t.lower()
    for f in FORBIDDEN:
        if re.search(f, low):
            return None, "forbidden: " + f
    if any(h.lower() == low for h in have):
        return None, "already in the book"
    return t, None


def stock_greetings(creds, token, actor, passphrase, dry, out=None, fill=False):
    """Stock the kinds that run low (or, with fill, any kind not yet full), steered by
    the day's questions and wants."""
    rep = get("/v0/docent", token)
    learn_identity(rep)
    book = rep.get("book", {})
    low = int(book.get("low_at", 8))
    full = int(book.get("kept", 24))
    need = [k for k in ("first", "welcome") if len(book.get(k, [])) < (full if fill else low)]
    if not need:
        log(f"the greeting book is stocked (first {len(book.get('first', []))}, welcome {len(book.get('welcome', []))}); nothing asked")
        return 0
    steer = rep.get("steer", {})
    asked = [a for a in steer.get("asked", []) if a.get("q")]
    wants = steer.get("wants", [])
    lines = []
    if asked:
        lines.append("What visitors asked the Docent in the last day (and what it answered):")
        for a in asked[:24]:
            lines.append(f"- Q: {a.get('q')}" + (f" / A: {a.get('a')}" if a.get("a") else " / (no answer found)"))
    if wants:
        lines.append("What the named creatures want right now: " + "; ".join(
            f"{w.get('name')} the {w.get('kind')} in {w.get('room')} wants " + (f"a story about {w.get('story')}" if w.get("story") else f"the word {w.get('word')!r} carried to {w.get('word_to')}")
            for w in wants[:8]))
    if steer.get("newest_room"):
        lines.append(steer["newest_room"])
    have = {k: book.get(k, []) for k in ("first", "welcome")}
    # The world's own proper nouns the lines may use: rooms and named creatures it told us about.
    extra = set()
    for w in wants:
        for v in (w.get("room"), w.get("name"), w.get("word_to")):
            extra.update(re.findall(r"[A-Z][A-Za-z'’]*", str(v or "")))
    extra.update(re.findall(r"[A-Z][A-Za-z'’]*", str(steer.get("newest_room") or "")))
    user = (
        f"Write {6 if 'first' in need else 0} FIRST lines and {6 if 'welcome' in need else 0} WELCOME lines. "
        "Let what visitors asked steer what the first words point at (if many asked where something is, say how to find things; if "
        "they asked about fading, say what keeps). Not these, already in the book:\n"
        + "\n".join(f"FIRST: {l}" for l in have["first"]) + "\n" + "\n".join(f"WELCOME: {l}" for l in have["welcome"])
        + "\n\nWhat the world knows tonight:\n" + ("\n".join(lines) if lines else "(a quiet day: nobody asked anything)")
    )
    text = ask_model(creds, GREETING_SYSTEM + IDENTITY, user, max_tokens=1400)
    offered = [l.strip() for l in text.splitlines() if l.strip()]
    kept, refused = [], []
    for raw in offered:
        m = re.match(r"^(FIRST|WELCOME):\s*(.+)$", raw, re.I)
        if not m:
            refused.append((raw[:50], "no kind"))
            continue
        kind, line = m.group(1).lower(), m.group(2)
        if kind not in need:
            continue
        t, why = greeting_sieve(line, have[kind] + [k for kk, k in kept if kk == kind], extra)
        if t is None:
            refused.append((line[:50], why))
            continue
        kept.append((kind, t))
    log(f"the model offered {len(offered)} greeting(s); {len(kept)} pass" + ("; refused: " + "; ".join(f"{l} ({w})" for l, w in refused[:8]) if refused else ""))
    applied = 0
    if dry:
        for kind, t in kept:
            print(f"{kind}: {t}")
    else:
        for kind, t in kept:
            r = command(token, actor, passphrase, "greeting", ['"' + t.replace('"', "'") + '"', kind])
            if r.get("ok"):
                applied += 1
            else:
                log("the world refused one: " + str(r.get("error"))[:160])
        log(f"stocked {applied} greeting(s)")
    if out:
        with open(out, "a", encoding="utf-8") as f:
            f.write(json.dumps({"greetings": True, "model": MODEL, "offered": offered, "kept": kept, "refused": refused, "applied": applied}, ensure_ascii=False) + "\n")
    return applied


# Who the Docent is, as the world says it (GET /v0/docent carries name and story):
# filled at start, so the figure answers "who are you" from the record, not from guesswork.
IDENTITY = ""


def learn_identity(rep):
    global IDENTITY
    d = rep.get("docent") or {}
    if d.get("name") and d.get("story"):
        IDENTITY = f" Your name is {d['name']}; the visitors call you the Docent, and either will do. {d['story']} Asked who you are, say so in a sentence or two, in your own voice."


def answer_one(creds, token, actor, passphrase, q, cat, verbs, dry):
    user = f"The catalog (the quickstart):\n\n{cat}\n\n---\nWhat the world knows:\n{context(q)}\n\nThe question, from {q.get('from')}: {q.get('text')}\n\nThe Docent answers:"
    text = ask_model(creds, SYSTEM + IDENTITY + " " + RULES, user, max_tokens=300)
    t, why = sieve(text, verbs)
    if t is None:
        log(f"question {q['id']} ({q.get('from')}): the model's answer refused ({why}): {text[:120]!r}")
        # One more try, told why.
        text = ask_model(creds, SYSTEM + IDENTITY + " " + RULES + " The last answer was refused: " + why + ". Answer again within the rules.", user, max_tokens=300)
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
    ap.add_argument("--greetings", action="store_true", help="stock the greeting book once and exit")
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
    if a.greetings:
        stock_greetings(creds, token, actor, passphrase, a.dry_run, a.out, fill=True)
        return 0
    cat = catalog()
    verbs = verbs_known(token)
    log(f"the Docent listens by {way_in}; catalog {len(cat)} characters; {len(verbs)} verbs known; every {a.every:g} s")
    seen = set()
    last_catalog = time.time()
    # The greeting book: at start, then once a day.
    last_book = 0.0
    while True:
        if time.time() - last_book > 86_400 and not a.dry_run:
            first = last_book == 0.0
            last_book = time.time()
            try:
                # At start the book is filled toward full (a release brings a better
                # pen); after that, topped up when a kind runs low.
                stock_greetings(creds, token, actor, passphrase, False, a.out, fill=first)
            except Exception as e:  # noqa: BLE001
                log(f"the greeting book could not be stocked: {e}")
        try:
            rep = get("/v0/docent", token)
            learn_identity(rep)
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
