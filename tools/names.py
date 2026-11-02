#!/usr/bin/env python3
"""Keep the Namery stocked: names for creatures, a batch at a time, from a small model.

    tools/names.py                # stock the pool if it is low
    tools/names.py --dry-run      # show what would be stocked
    tools/names.py --force --count 80

Reads GET /v0/namery (operator only): the pool, what was ever drawn, and everything
the world already uses for something (creature names and kinds, citizens, rooms, the
fixed list, verbs). If the pool is below its low mark, asks the model for a batch in
the world's register, keeps what passes the checks here (one capitalized word of 3 to
12 ASCII letters, not in use, not twice in the batch, not an everyday English word),
and `stock`s them through POST /v0/command as the Bindery's own citizen. The world
runs the same checks again and says back what it refused.

Every batch, taken or refused, is appended to the local cache (NAMERY_OUT, JSON
lines) so a name the model offered twice, or one the world refused, is never asked
for again. The pool in the world is the cache that matters: each name is dealt once.

Environment: as tools/bind.py (the way in is shared), plus NAMERY_OUT (the cache
file), NAMERY_LOW (stock when fewer unused than this; default the world's own mark),
NAMERY_BATCH (how many to ask for, default 60).
"""

import argparse
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from foxing_api import MODEL, Credentials, ask_model, command, get, log, post_discord  # noqa: E402

SHAPE = re.compile(r"^[A-Z][a-z]{2,11}$")

# Everyday words the model reaches for that read as nouns, not names, in a room.
EVERYDAY = set(
    """Candle Lantern Mirror Window Shadow Silver Golden River Stone Cloud Rain Snow Frost
    Ember Ash Smoke Dust Thread Needle Button Ribbon Paper Letter Story Dream Night Dawn
    Dusk Star Moon Sun Light Dark Hollow Whisper Echo Memory Ghost Spirit Wander Drift
    Harbor Meadow Forest Garden Orchard Willow Maple Cedar Pine Birch Rowan Ivy Fern Moss
    Clover Thistle Bramble Nettle Sorrel Tansy Yarrow Sage Mint Pepper Honey Sugar Salt
    Copper Iron Brass Pewter Marble Velvet Linen Wool Flax Hemp Reed Rush Sedge Heather
    Gorse Broom Furze Bracken Lichen Fungus Toad Newt Hare Fox Owl Crow Rook Wren Finch
    Lark Thrush Robin Swift Martin Heron Crane Stork Otter Badger Weasel Stoat Mole Vole
    Shrew Bat Moth Wasp Bee Ant Beetle Cricket Spider Snail Slug Worm Eel Pike Perch
    Trout Minnow Pebble Gravel Shingle Slate Flint Chalk Clay Loam Peat Turf Sod Furrow""".split()
)

SYSTEM = (
    "You name the creatures of Disremembered, a text world about memory and forgetting, "
    "where small things live in a drowned library: wisps, inklings, hollows, ragpickers, "
    "lampreys, ushers, censors. A name is one word, capitalized, three to twelve letters, "
    "ASCII only. The register is the world's existing names: Ossian, Wren, Tallow, Mote, "
    "Hesper, Fennick, Lute, Sorrel, Quill, Brindle, Nim, Vesper, Ashby, Loam, Pell, "
    "Thimble, Marrow, Ides, Cinder, Hollis, Tansy, Grist, Orrin, Moth, Sable, Dunmore, Isk, "
    "Pyre, Lantern, Rook, Wick, Fathom. Old words, odd words, half-words, names from "
    "bookbinding, candle-making, map-making, weather and small hours; soft and plain; "
    "nothing cute, nothing grand, nothing from a famous book or game, no real people, no "
    "everyday nouns a reader would mistake for the thing itself. Reply with the names only, "
    "one per line, no numbering, no commentary."
)


def ask_names(creds, count, avoid, cache):
    avoid_text = ", ".join(sorted(avoid)[:400])
    user = (
        f"Give {count} new names. None of these, which the world already has or has refused, "
        f"and nothing that is one of them with a letter changed:\n{avoid_text}"
        + (f"\nAlso none of these, offered before: {', '.join(sorted(cache)[:200])}" if cache else "")
    )
    text = ask_model(creds, SYSTEM, user, max_tokens=1200)
    out = []
    for line in text.splitlines():
        n = line.strip().strip("-•*0123456789. ").strip()
        if n:
            out.append(n)
    return out


def sieve(offered, taken, cache):
    """What passes here, and what was refused with why."""
    keep, refused = [], []
    seen = set()
    for n in offered:
        low = n.lower()
        if not SHAPE.match(n):
            refused.append((n, "shape"))
        elif low in taken:
            refused.append((n, "the world has it"))
        elif low in cache:
            refused.append((n, "offered before"))
        elif n in EVERYDAY:
            refused.append((n, "an everyday word"))
        elif low in seen:
            refused.append((n, "twice"))
        else:
            seen.add(low)
            keep.append(n)
    return keep, refused


def main():
    ap = argparse.ArgumentParser(description="keep the Namery stocked")
    ap.add_argument("--dry-run", action="store_true", help="show the batch, stock nothing")
    ap.add_argument("--force", action="store_true", help="stock even if the pool is not low")
    ap.add_argument("--count", type=int, default=int(os.environ.get("NAMERY_BATCH", "60")), help="how many to ask for")
    ap.add_argument("--out", default=os.environ.get("NAMERY_OUT", ""), help="the local cache (JSON lines)")
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
    low = int(os.environ.get("NAMERY_LOW", book.get("low_at", 30)))
    unused = int(book.get("unused", 0))
    log(f"the Namery holds {unused} unused name(s); low mark {low}; {'dry run' if a.dry_run else 'stocking'} by {way_in}")
    if unused >= low and not a.force:
        log("enough; nothing asked")
        return 0

    taken = set(s.lower() for s in book.get("taken", []))
    cache = set()
    if a.out and os.path.exists(a.out):
        with open(a.out, encoding="utf-8") as f:
            for line in f:
                try:
                    rec = json.loads(line)
                except ValueError:
                    continue
                for n in rec.get("offered", []):
                    cache.add(n.lower())
    offered = ask_names(creds, a.count, taken, cache)
    keep, refused = sieve(offered, taken, cache)
    log(f"the model offered {len(offered)}; {len(keep)} pass here" + (f"; refused: " + ", ".join(f"{n} ({why})" for n, why in refused[:20]) if refused else ""))
    record = {"model": MODEL, "offered": offered, "kept": keep, "refused": refused, "applied": False}
    if a.dry_run:
        print("\n".join(keep))
    elif keep:
        r = command(token, actor, passphrase, "stock", ['"' + ", ".join(keep) + '"', MODEL])
        if r.get("ok"):
            res = r.get("result") or {}
            record["applied"] = True
            record["world_refused"] = res.get("refused", [])
            log(r.get("summary", ""))
            post_discord(webhook, f"The Namery took in {len(res.get('added', []))} names from {MODEL}; {res.get('unused', '?')} unused now.")
        else:
            log("the world refused the stocking: " + str(r.get("error")))
    if a.out:
        with open(a.out, "a", encoding="utf-8") as f:
            f.write(json.dumps(record, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
