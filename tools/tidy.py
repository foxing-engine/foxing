#!/usr/bin/env python3
"""Clear test debris from the live world, by name, as the operator.

    tools/tidy.py --raze link-test-2791 --banish jkhgkkjhkjhkh Bindery_peek
    tools/tidy.py --dry-run --raze ...

`raze` removes a dug room by its key (never a seed room, never one with a citizen in
it); `banish` makes the world forget a citizen now, as it would after thirty days of
silence (what they dug stays). Both are the operator verbs, run through /v0/command
with the admin token, so the record says the operator did it.

Environment: FOXING_ADMIN_TOKEN, BINDERY_ACTOR (the operator's citizen, default
Bindery) and BINDERY_PASSPHRASE, as in /etc/foxing/bindery.env; on moondial-1:

    sudo -u starsailor env $(grep -v '^#' /etc/foxing/bindery.env | xargs) \\
        python3 /srv/foxing/current/tools/tidy.py --raze <key> --banish <name> ...
"""

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from foxing_api import command, log  # noqa: E402


def main():
    ap = argparse.ArgumentParser(description="clear test debris, as the operator")
    ap.add_argument("--raze", nargs="*", default=[], help="dug room keys to remove")
    ap.add_argument("--banish", nargs="*", default=[], help="citizens the world should forget now")
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args()
    token = os.environ.get("FOXING_ADMIN_TOKEN") or os.environ.get("MOONDIAL_ADMIN_TOKEN")
    actor = os.environ.get("BINDERY_ACTOR", "Bindery")
    passphrase = os.environ.get("BINDERY_PASSPHRASE", "")
    if not token or not passphrase:
        sys.exit("FOXING_ADMIN_TOKEN and BINDERY_PASSPHRASE are needed (source /etc/foxing/bindery.env)")
    if not a.raze and not a.banish:
        sys.exit("nothing named: --raze <key> and/or --banish <name>")
    for key in a.raze:
        if a.dry_run:
            print(f"would raze {key}")
            continue
        r = command(token, actor, passphrase, "raze", [key])
        log(f"raze {key}: " + (r.get("summary") if r.get("ok") else "refused: " + str(r.get("error"))[:200]))
    for name in a.banish:
        if a.dry_run:
            print(f"would banish {name}")
            continue
        r = command(token, actor, passphrase, "banish", [name])
        log(f"banish {name}: " + (r.get("summary") if r.get("ok") else "refused: " + str(r.get("error"))[:200]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
