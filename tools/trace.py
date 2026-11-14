#!/usr/bin/env python3
"""Trace one name through the public record.

    tools/trace.py blackrock
    tools/trace.py blackrock --since 6h --verbose
    tools/trace.py possibly --json > possibly.json

Pages the world's public log (`GET /v0/log`, cursor-paged) and prints a timeline for
one citizen: arrivals and departures, every message through the gate (the verbs, how
it came in, the checker's verdict when it was not PASS), refusals and what the world
said back, moves room to room, kills and what was taken, deaths and what was lost,
chunks earned, gifts given and received, stories told, words carried, things said,
feedback, digs. A summary at the end: first and last seen, messages by route, kills,
deaths, XP earned, rooms walked.

The log is cached in ~/.cache/foxing/<world>.jsonl so the second run only fetches
what is new. Nothing here needs a passphrase or the admin header: it is the record
anyone may read.
"""

import argparse
import datetime as dt
import json
import os
import re
import sys
import urllib.error
import urllib.request

BASE = "https://play.agentscanplay.com"
PAGE = 1000


def fetch(url):
    req = urllib.request.Request(url, headers={"User-Agent": "foxing-trace/1"})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.load(r)


def cache_path(world):
    d = os.path.join(os.path.expanduser("~"), ".cache", "foxing")
    os.makedirs(d, exist_ok=True)
    return os.path.join(d, f"{world}.jsonl")


def load_log(base, fresh=False):
    """The whole log, from the cache plus whatever is new."""
    health = fetch(f"{base}/health")
    world = health.get("world", "world")
    path = cache_path(world)
    events = []
    if not fresh and os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line:
                    events.append(json.loads(line))
    after = events[-1]["seq"] if events else 0
    new = []
    while True:
        try:
            d = fetch(f"{base}/v0/log?after={after}&limit={PAGE}")
        except urllib.error.HTTPError as e:
            if e.code == 502:
                sys.stderr.write("the driver is restarting; try again in a minute\n")
                sys.exit(2)
            raise
        page = d.get("events", [])
        if not page:
            break
        new.extend(page)
        after = page[-1]["seq"]
        if sys.stderr.isatty():
            sys.stderr.write(f"\r  fetched to ev-{after}")
    if new and sys.stderr.isatty():
        sys.stderr.write("\n")
        with open(path, "a", encoding="utf-8") as f:
            for e in new:
                f.write(json.dumps(e, separators=(",", ":")) + "\n")
        events.extend(new)
    return world, events


def when(ms):
    return dt.datetime.fromtimestamp(ms / 1000, dt.UTC).strftime("%Y-%m-%d %H:%M:%S")


def human(ms):
    s = int(ms // 1000)
    if s < 90:
        return f"{s}s"
    m = s // 60
    if m < 90:
        return f"{m}m"
    h = m // 60
    if h < 48:
        return f"{h}h{m % 60:02d}"
    return f"{h // 24}d{h % 24}h"


def parse_since(s):
    if not s:
        return None
    m = re.fullmatch(r"(\d+)([hdm])", s)
    if not m:
        raise SystemExit("--since takes 30m, 6h or 2d")
    n, unit = int(m.group(1)), m.group(2)
    secs = {"m": 60, "h": 3600, "d": 86400}[unit] * n
    return (dt.datetime.now(dt.UTC).timestamp() - secs) * 1000


def plain(fields, key):
    f = fields.get(key)
    if isinstance(f, dict):
        return f.get("value")
    return f


def build_index(events):
    """Rooms and creatures by id, citizen ids by name, chunk owners by id."""
    rooms, creatures, citizens, chunks = {}, {}, {}, {}
    for e in events:
        k, p = e["kind"], e.get("payload", {})
        if k == "store.entity_created":
            kind = p.get("kind")
            if kind == "citizen" and p.get("owner"):
                citizens[p["owner"]] = p["id"]
            elif kind == "creature":
                creatures[p["id"]] = {"kind": "?", "name": None}
            elif kind == "chunk":
                chunks[p["id"]] = {"owner": p.get("owner"), "parent": p.get("parent")}
        elif k == "store.component_set":
            c, f = p.get("component"), p.get("fields", {})
            if c == "disremembered.room" and plain(f, "name"):
                rooms[p["id"]] = plain(f, "name")
            elif c == "disremembered.creature" and p["id"] in creatures:
                creatures[p["id"]]["kind"] = plain(f, "name") or "?"
            elif c == "disremembered.mind" and p["id"] in creatures:
                creatures[p["id"]]["name"] = plain(f, "name")
            elif c == "disremembered.chunk" and p["id"] in chunks:
                chunks[p["id"]].update(
                    {"source": plain(f, "source"), "size": plain(f, "size")}
                )
        elif k == "store.field_set" and p.get("component") == "disremembered.room":
            if p.get("field") == "name" and p.get("id") in rooms:
                rooms[p["id"]] = p.get("value")
    return rooms, creatures, citizens, chunks


def creature_name(creatures, cid, at_kind=None, at_name=None):
    c = creatures.get(cid, {})
    kind = at_kind or c.get("kind", "?")
    name = at_name if at_name is not None else c.get("name")
    return f"{name} the {kind}" if name else f"a {kind}"


def verbs_of(text):
    """The DO lines of a CLEAR message, as `verb args`, and the ASKs."""
    out = []
    for line in text.split("\n"):
        line = line.strip()
        m = re.match(r"\[DO\]\s+\S+\s+(?:MUST|SHOULD|MAY)\s+(.*?)\.?$", line)
        if m:
            out.append(m.group(1))
            continue
        m = re.match(r"\[ASK\]\s+(.*)$", line)
        if m:
            out.append("? " + m.group(1))
    return out


def quotes_of(text):
    return [
        l.strip()[7:].strip()
        for l in text.split("\n")
        if l.strip().startswith("[QUOTE]")
    ]


def trace(events, name, since=None, verbose=False):
    rooms, creatures, citizens, chunks = build_index(events)
    me = citizens.get(name)
    lines = []
    stats = {
        "first": None,
        "last": None,
        "messages": {},
        "statements": 0,
        "refused": 0,
        "kills": 0,
        "deaths": 0,
        "lost": 0.0,
        "earned": 0.0,
        "rooms": [],
        "said": 0,
        "stories": 0,
        "gifts_out": 0,
        "gifts_in": 0,
        "feedback": 0,
    }

    def add(e, text, kind="·"):
        if since and e["at"] < since:
            return
        stats["first"] = stats["first"] or e["at"]
        stats["last"] = e["at"]
        lines.append((e["at"], e["seq"], kind, text))

    for e in events:
        k, p, at = e["kind"], e.get("payload", {}), e["at"]
        actor = e.get("actor", {})
        mine = actor.get("id") == name
        if k == "citizen.arrived" and p.get("actor") == name:
            away = p.get("away_ms")
            add(
                e,
                "arrives{} via {}{}".format(
                    " for the first time" if p.get("fresh") else "",
                    p.get("origin", "?"),
                    f" after {human(away)} away" if away else "",
                ),
                ">",
            )
        elif k == "gate.message" and p.get("from") == name:
            via = p.get("via", "?")
            stats["messages"][via] = stats["messages"].get(via, 0) + 1
            vs = verbs_of(p.get("text", ""))
            stats["statements"] += len(vs)
            verdict = p.get("verdict", "PASS")
            tail = "" if verdict == "PASS" else f"  [{verdict} {p.get('score')}]"
            add(e, f"[{via}] " + " | ".join(vs) + tail, "»")
        elif k == "gate.reply" and p.get("to") == name:
            text = p.get("text", "")
            refusals = [
                l
                for l in text.split("\n")
                if "was refused" in l or "did not run" in l or "failed rule" in l
            ]
            qs = quotes_of(text)
            for q in qs:
                for m in re.finditer(r"chunk-(\d+) worth (\d+(?:\.\d+)?) XP", q):
                    stats["earned"] += float(m.group(2))
                    add(e, f"  chunk-{m.group(1)} is worth {float(m.group(2)):.0f} XP", "+")
                if "comes apart" in q:
                    # An unnamed creature's end has no public event of its own; the
                    # reply is the record. (A named one's is `disremembered.taken`.)
                    stats["kills"] += 1
                    add(e, "  " + q[:200], "⚔")
            if refusals:
                stats["refused"] += len(refusals)
                why = qs[0] if qs else ""
                add(e, "refused: " + (why[:160] if why else refusals[0][:160]), "«")
            elif verbose and qs:
                for q in qs[:3]:
                    add(e, "  " + q[:200], "«")
        elif k == "store.entity_moved" and me and p.get("id") == me:
            room = rooms.get(p.get("parent"), f"room-{p.get('parent')}")
            if not stats["rooms"] or stats["rooms"][-1] != room:
                stats["rooms"].append(room)
            add(e, room, "→")
        elif k == "disremembered.taken" and p.get("by") == name:
            stats["kills"] += 1
            add(
                e,
                "fells {} and takes {} (chunk-{})".format(
                    creature_name(creatures, p.get("creature"), p.get("kind"), p.get("name")),
                    p.get("with"),
                    p.get("chunk"),
                ),
                "⚔",
            )
        elif k == "disremembered.died" and p.get("citizen") == name:
            stats["deaths"] += 1
            lost = p.get("lost", [])
            total = sum(c.get("value", 0) for c in lost)
            stats["lost"] += total
            add(
                e,
                "goes down at level {:.0f}; loses {} ({:.0f}% share)".format(
                    p.get("level", 0),
                    f"{total:.0f} XP in {len(lost)} chunks" if lost else "nothing",
                    p.get("share", 0) * 100,
                ),
                "✝",
            )
        elif k == "store.component_set" and p.get("component") == "disremembered.chunk":
            # The public record shows a chunk's source, not its size (that is the
            # owner's); the worth is read from the world's reply when it says it.
            c = chunks.get(p.get("id"))
            if c and c.get("owner") == name and mine and "source" in p.get("fields", {}):
                src = plain(p.get("fields", {}), "source")
                add(e, f"earns chunk-{p['id']}: {src}", "+")
        elif k == "disremembered.given":
            if p.get("from") == name:
                stats["gifts_out"] += 1
                add(
                    e,
                    "gives {} {}{}".format(
                        p.get("to"), p.get("with"), " (taxed)" if p.get("taxed") else ""
                    ),
                    "⇢",
                )
            elif p.get("to") == name:
                stats["gifts_in"] += 1
                add(e, f"is given {p.get('with')} by {p.get('from')}", "⇠")
        elif k == "disremembered.story" and p.get("from") == name:
            stats["stories"] += 1
            add(
                e,
                "tells {} a story of {} words, earns {:.0f}{}".format(
                    creature_name(creatures, p.get("creature"), p.get("kind"), p.get("name")),
                    p.get("words"),
                    p.get("earned_xp", 0) or 0,
                    f", heard {p['repeats']} like it" if p.get("repeats") else "",
                ),
                "✎",
            )
        elif k == "disremembered.say" and p.get("from") == name:
            stats["said"] += 1
            to = f" to {p['to']}" if p.get("to") else ""
            add(e, f"says{to}: \"{p.get('text', '')[:140]}\"", "“")
        elif k == "disremembered.feedback" and (p.get("from") == name or p.get("by") == name):
            stats["feedback"] += 1
            add(e, f"feedback: \"{p.get('text', '')[:200]}\"", "✉")
        elif k == "disremembered.errand_done" and p.get("from") == name:
            add(e, f"{p.get('errand')} errand done for {p.get('name')}: {p.get('want')}", "✓")
        elif k == "disremembered.dug" and (p.get("by") == name or mine):
            add(e, f"digs {p.get('exit')} to {p.get('name')}", "⛏")
        elif k == "disremembered.remembered" and p.get("by") == name:
            add(e, f"remembers {p.get('with')} into {creature_name(creatures, p.get('creature'), p.get('kind'), p.get('name'))}", "♥")
        elif k == "disremembered.departed" and p.get("name") == name:
            add(e, f"the room forgets them ({rooms.get(p.get('room'), '?')})", "<")
        elif k == "disremembered.forgotten_name" and p.get("name") == name:
            add(e, "the world forgets the name", "<")
        elif k == "disremembered.narrative" and (p.get("to") == name or (isinstance(p.get("to"), list) and name in p["to"])) and verbose:
            add(e, "  hears: " + p.get("text", "")[:200], "·")
    return lines, stats, me


def main():
    ap = argparse.ArgumentParser(description="trace one name through the public record")
    ap.add_argument("name", nargs="?", help="the citizen to trace; omit with --names")
    ap.add_argument("--names", action="store_true", help="list every name in the record with first and last seen")
    ap.add_argument("--base", default=BASE)
    ap.add_argument("--since", help="only the last 30m, 6h, 2d")
    ap.add_argument("--verbose", "-v", action="store_true", help="what the world said back, and every narrator line")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--fresh", action="store_true", help="ignore the cache and fetch the whole log")
    a = ap.parse_args()
    world, events = load_log(a.base, fresh=a.fresh)
    if a.names or not a.name:
        seen = {}
        for e in events:
            p = e.get("payload", {})
            if e["kind"] == "gate.message" and p.get("from"):
                s_ = seen.setdefault(p["from"], {"first": e["at"], "last": e["at"], "messages": 0, "via": set()})
                s_["last"] = e["at"]
                s_["messages"] += 1
                s_["via"].add(p.get("via", "?"))
            elif e["kind"] == "citizen.arrived" and p.get("actor"):
                s_ = seen.setdefault(p["actor"], {"first": e["at"], "last": e["at"], "messages": 0, "via": set()})
                s_["last"] = e["at"]
        since = parse_since(a.since)
        rows = sorted(seen.items(), key=lambda kv: -kv[1]["last"])
        for n, s_ in rows:
            if since and s_["last"] < since:
                continue
            print(f"{n:<20} first {when(s_['first'])}  last {when(s_['last'])}  {s_['messages']:>5} messages  {','.join(sorted(s_['via']))}")
        return 0
    lines, stats, me = trace(events, a.name, parse_since(a.since), a.verbose)
    if me is None:
        print(f"{a.name}: no such name in the record of {world}")
        return 1
    if a.json:
        print(json.dumps({"name": a.name, "citizen": me, "stats": stats, "lines": [
            {"at": at, "seq": seq, "kind": kind, "text": text} for at, seq, kind, text in lines
        ]}, indent=1))
        return 0
    print(f"{a.name} (citizen-{me}) in {world}, {len(lines)} lines{f' since {a.since}' if a.since else ''}\n")
    day = None
    for at, seq, kind, text in lines:
        d = when(at)[:10]
        if d != day:
            print(f"--- {d}")
            day = d
        print(f"{when(at)[11:]} {kind} {text}")
    print()
    if stats["first"]:
        span = human(stats["last"] - stats["first"])
        msgs = ", ".join(f"{n} via {v}" for v, n in sorted(stats["messages"].items()))
        print(f"first seen {when(stats['first'])}, last {when(stats['last'])} ({span})")
        print(f"messages: {msgs or 'none'}; {stats['statements']} statements, {stats['refused']} refused")
        print(f"kills {stats['kills']}, deaths {stats['deaths']} (lost {stats['lost']:.0f} XP), earned {stats['earned']:.0f} XP")
        print(f"said {stats['said']}, stories {stats['stories']}, gifts out {stats['gifts_out']} in {stats['gifts_in']}, feedback {stats['feedback']}")
        if stats["rooms"]:
            print("rooms: " + " → ".join(stats["rooms"][:40]) + (" …" if len(stats["rooms"]) > 40 else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
