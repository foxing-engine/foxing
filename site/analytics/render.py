#!/usr/bin/env python3
"""Render the analytics store into a report: a text summary on stdout, and (with --html)
the interim admin page at /srv/agentscanplay/shared/analytics/html/index.html.

The questions the report answers are the ones this site exists to answer:
  - How many requests, and from what kind of visitor: browsers, AI crawlers, tools, agents?
  - Of the hits on "/", how many were served CLEAR text versus the HTML page?
  - Who is reading llms.txt, the invitation, the world card, the checker?
  - Where do the humans come from?

Standard library only. Usage: render.py [--days N] [--html]
"""

from __future__ import annotations

import argparse
import html
import json
import os
import re
import sqlite3
import urllib.request
from datetime import datetime, timezone, timedelta

DB = os.environ.get("ANALYTICS_DB", "/srv/agentscanplay/shared/analytics/analytics.db")
# The lab: the operator's own addresses or CIDRs, comma-separated, the same list the
# driver reads as FOXING_LAB_NETS. A request from anywhere else is organic.
LAB_NETS = os.environ.get("ANALYTICS_LAB_NETS", os.environ.get("FOXING_LAB_NETS", os.environ.get("MOONDIAL_LAB_NETS", "")))


def driver_env_paths() -> list[str]:
    """Where the driver's env file is: the new name first, the old second."""
    named = os.environ.get("FOXING_DRIVER_ENV") or os.environ.get("MOONDIAL_DRIVER_ENV")
    paths = [named] if named else []
    return paths + ["/etc/foxing/driver.env", "/etc/moondial/driver.env"]


def env_from_driver(*keys: str) -> str:
    """A value from the driver's env file, when the timer's own environment lacks it:
    the first of `keys` found, in the first file that exists."""
    for path in driver_env_paths():
        try:
            with open(path, encoding="utf-8") as fh:
                for line in fh:
                    for key in keys:
                        if line.startswith(key + "="):
                            return line.split("=", 1)[1].strip().strip('"')
        except OSError:
            continue
    return ""


if not LAB_NETS:
    LAB_NETS = env_from_driver("FOXING_LAB_NETS", "MOONDIAL_LAB_NETS")
HTML_DIR = os.environ.get("ANALYTICS_HTML", "/srv/agentscanplay/shared/analytics/html")

# The game driver, for the citizens report: who came, by what door, and what they did.
DRIVER = os.environ.get("FOXING_HTTP_URL", os.environ.get("MOONDIAL_HTTP_URL", "http://127.0.0.1:7777"))

KEY_PATHS = ["/", "/invitation.txt", "/llms.txt", "/.well-known/world-card.json", "/clear/clear-spec-v0.3.md",
             "/clear/disremembered.yaml", "/clear/clear_check.py", "/robots.txt",
             # the game server
             "/health", "/v0/commands", "/v0/command", "/v0/log"]


def q(db, sql, args=()):
    return db.execute(sql, args).fetchall()


def lab_nets() -> list:
    import ipaddress
    nets = []
    for part in LAB_NETS.split(","):
        part = part.strip()
        if not part:
            continue
        try:
            nets.append(ipaddress.ip_network(part, strict=False))
        except ValueError:
            pass
    return nets


_LAB = None


def is_lab(ip: str) -> bool:
    global _LAB
    import ipaddress
    if _LAB is None:
        _LAB = lab_nets()
    try:
        a = ipaddress.ip_address(ip)
    except ValueError:
        return False
    return any(a in n for n in _LAB)


def has_table(db, name: str) -> bool:
    return bool(q(db, "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?", (name,)))


def has_column(db, table: str, col: str) -> bool:
    return any(r[1] == col for r in q(db, f"PRAGMA table_info({table})"))


def networks(db, since: str) -> list[list]:
    """Every address seen in the window: lab or not, first and last seen (over the whole
    store, so a returning visitor shows as one), requests, classes, the game's doors it
    went through and the names it took. The question the store exists to answer."""
    rows = {}
    if not has_column(db, "requests", "ip"):
        # A store the collector has not migrated yet: the networks wait for it.
        return []
    for ip, first, last, n, classes, hosts in q(db, """
        SELECT ip, MIN(ts), MAX(ts), COUNT(*), GROUP_CONCAT(DISTINCT ua_class), GROUP_CONCAT(DISTINCT host)
        FROM requests WHERE ip != '' AND status < 300 AND ip IN (SELECT DISTINCT ip FROM requests WHERE day >= ?)
        GROUP BY ip""", (since,)):
        rows[ip] = {"ip": ip, "first": first, "last": last, "requests": n, "classes": classes or "", "hosts": hosts or "",
                    "mcp": 0, "cloaks": 0, "gates": 0, "names": ""}
    if has_table(db, "doors"):
        for ip, first, last, mcp, cloaks, gates, names in q(db, """
            SELECT ip, MIN(ts), MAX(ts),
                   SUM(CASE WHEN kind='mcp' THEN 1 ELSE 0 END),
                   SUM(CASE WHEN tool='cloak' THEN 1 ELSE 0 END),
                   SUM(CASE WHEN tool='gate' THEN 1 ELSE 0 END),
                   GROUP_CONCAT(DISTINCT CASE WHEN kind='arrive' THEN actor END)
            FROM doors WHERE ip != '' AND ip IN (SELECT DISTINCT ip FROM doors WHERE day >= ?) GROUP BY ip""", (since,)):
            r = rows.setdefault(ip, {"ip": ip, "first": first, "last": last, "requests": 0, "classes": "", "hosts": "",
                                     "mcp": 0, "cloaks": 0, "gates": 0, "names": ""})
            r["first"] = min(r["first"], first); r["last"] = max(r["last"], last)
            r["mcp"], r["cloaks"], r["gates"], r["names"] = mcp or 0, cloaks or 0, gates or 0, names or ""
    out = []
    for r in sorted(rows.values(), key=lambda r: r["last"], reverse=True):
        out.append([r["ip"], "lab" if is_lab(r["ip"]) else "", r["first"][:16].replace("T", " "), r["last"][:16].replace("T", " "),
                    r["requests"], r["mcp"], r["cloaks"], r["gates"], r["names"], r["classes"], r["hosts"]])
    return out


def funnel(db, since: str) -> list[list]:
    """Per day, lab and organic apart: addresses that initialized, that cloaked, that
    checked, that sent through the gate, that arrived under a name. The door, counted."""
    if not has_table(db, "doors"):
        return []
    rows = []
    for day, lab, init, cloak, check, gate, arrived in q(db, """
        SELECT day, lab,
               COUNT(DISTINCT CASE WHEN method='initialize' THEN ip END),
               COUNT(DISTINCT CASE WHEN tool='cloak' THEN ip END),
               COUNT(DISTINCT CASE WHEN tool='check' THEN ip END),
               COUNT(DISTINCT CASE WHEN tool='gate' THEN ip END),
               COUNT(DISTINCT CASE WHEN kind='arrive' AND fresh=1 THEN actor END)
        FROM doors WHERE day >= ? GROUP BY day, lab ORDER BY day DESC, lab""", (since,)):
        rows.append([day, "lab" if lab else "organic", init, cloak, check, gate, arrived])
    return rows


def admin_token() -> str | None:
    tok = os.environ.get("FOXING_ADMIN_TOKEN") or os.environ.get("MOONDIAL_ADMIN_TOKEN")
    if tok:
        return tok
    # The driver's env file: where the engine's name put it, then where it was.
    return env_from_driver("FOXING_ADMIN_TOKEN", "MOONDIAL_ADMIN_TOKEN") or None


def citizens(days: int) -> tuple[list[dict], str]:
    """The driver's citizens report, or an empty list and why."""
    tok = admin_token()
    if not tok:
        return [], "no admin token (set FOXING_ADMIN_TOKEN or make the driver env readable)"
    since_ms = int((datetime.now(timezone.utc) - timedelta(days=days)).timestamp() * 1000)
    req = urllib.request.Request(f"{DRIVER}/v0/citizens?since={since_ms}", headers={"X-Foxing-Admin": tok})
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            return json.load(resp).get("citizens", []), ""
    except Exception as exc:  # noqa: BLE001 - the report must render whatever the driver does
        return [], f"driver not reachable at {DRIVER}: {exc}"


def refusals() -> tuple[list[list], str]:
    """Why the driver turned requests away since it last started: path, reason, count."""
    tok = admin_token()
    if not tok:
        return [], "no admin token"
    req = urllib.request.Request(f"{DRIVER}/v0/refusals", headers={"X-Foxing-Admin": tok})
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            body = json.load(resp)
    except Exception as exc:  # noqa: BLE001 - the report must render whatever the driver does
        return [], f"driver not reachable at {DRIVER}: {exc}"
    rows = sorted(([r["path"], r["reason"], r["count"]] for r in body.get("refusals", [])), key=lambda r: -r[2])
    hours = body.get("since_boot_seconds", 0) / 3600
    return rows, f"since the driver started, {hours:.1f} hours ago"


def ops() -> tuple[list[list], list[list], str]:
    """A5/S2: where the driver's time has gone since it started: a summary, and a
    line a verb."""
    tok = admin_token()
    if not tok:
        return [], [], "no admin token"
    req = urllib.request.Request(f"{DRIVER}/v0/ops", headers={"X-Foxing-Admin": tok})
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            b = json.load(resp)
    except Exception as exc:  # noqa: BLE001 - the report must render whatever the driver does
        return [], [], f"driver not reachable at {DRIVER}: {exc}"
    log, push, wait = b.get("log", {}), b.get("pushes", {}), b.get("lock_wait_ms", {})
    summary = [
        ["commands", b.get("commands", 0), ""],
        ["events in the log", b.get("events", 0), ""],
        ["entities in the store", b.get("entities", 0), ""],
        ["wait for the world's lock, mean", f"{wait.get('mean', 0)} ms", f"worst {wait.get('worst', 0)} ms"],
        ["appends to the log", log.get("appends", 0), f"{log.get('append_mean_ms', 0)} ms each"],
        ["reads past the kept tail", log.get("reads_past_the_tail", 0), f"{log.get('past_read_mean_ms', 0)} ms each"],
        ["pushes to open sessions", push.get("count", 0), f"held the world {push.get('held_mean_ms', 0)} ms each, worst {push.get('held_worst_ms', 0)} ms, {round(push.get('held_total_ms', 0) / 1000, 1)} s in all"],
    ]
    verbs = [[v["verb"], v["count"], f"{v['mean_ms']} ms", f"{v['worst_ms']} ms", f"{v['log_share']}%", v["appends_each"], v["past_reads"], f"{round(v['total_ms'] / 1000, 1)} s"]
             for v in b.get("verbs", [])[:40]]
    hours = b.get("since_boot_seconds", 0) / 3600
    return summary, verbs, f"since the driver started, {hours:.1f} hours ago"


def vitals(db) -> tuple[list[list], list[list], str]:
    """S6: the box, from the collector's five-minute samples: how it stands now, and
    the last day by the hour. CPU is a process's ticks between two samples over the
    time between them, as a share of one core."""
    if not has_table(db, "vitals"):
        return [], [], "no samples yet"
    since = (datetime.now(timezone.utc) - timedelta(hours=25)).strftime("%Y-%m-%dT%H:%M:%S")
    rows = q(db, "SELECT ts, load1, cpus, mem_used, mem_total, swap_used, disk_pct, driver_rss, driver_ticks, pg_rss, pg_ticks, db_bytes, events, conns, commits, inserted FROM vitals WHERE ts >= ? ORDER BY ts", (since,))
    if not rows:
        return [], [], "no samples in the last day"
    hz = os.sysconf("SC_CLK_TCK") if hasattr(os, "sysconf") else 100
    def secs(a, b):
        return (datetime.fromisoformat(b) - datetime.fromisoformat(a)).total_seconds()
    def rate(a, b, i, per=1.0):
        # A counter that fell is a process that restarted: no figure for that step.
        if a[i] is None or b[i] is None or b[i] < a[i] or secs(a[0], b[0]) <= 0:
            return None
        return (b[i] - a[i]) / per / secs(a[0], b[0])
    steps = []
    for a, b in zip(rows, rows[1:]):
        steps.append({"hour": b[0][:13], "load": b[1], "mem": b[3], "drv": rate(a, b, 8, hz), "pg": rate(a, b, 10, hz),
                      "ev": rate(a, b, 12), "commits": rate(a, b, 14), "conns": b[13], "drss": b[7]})
    last = rows[-1]
    def f(v, unit="", places=1):
        return "" if v is None else f"{v:.{places}f}{unit}"
    cur = steps[-1] if steps else {}
    now = [
        ["load, one minute", f(last[1], "", 2), f"of {last[2]} cores"],
        ["memory in use", f(last[3], " MB", 0), f"of {f(last[4], ' MB', 0)}; swap {f(last[5], ' MB', 0)}"],
        ["disk in use", f(last[6], "%", 0), ""],
        ["the driver", f(last[7], " MB", 0), f"{f((cur.get('drv') or 0) * 100, '%', 1)} of a core, these five minutes"],
        ["PostgreSQL", f(last[9], " MB", 0), f"{f((cur.get('pg') or 0) * 100, '%', 1)} of a core; {last[13] if last[13] is not None else '?'} connections"],
        ["the log's database", f((last[11] or 0) / 1e6, " MB", 0), f"{last[12] or '?'} events; {f(cur.get('ev'), ' written a second', 2)}"],
    ]
    hours: dict[str, list[dict]] = {}
    for st in steps:
        hours.setdefault(st["hour"], []).append(st)
    def peak(xs, k):
        v = [x[k] for x in xs if x[k] is not None]
        return max(v) if v else None
    def mean(xs, k):
        v = [x[k] for x in xs if x[k] is not None]
        return sum(v) / len(v) if v else None
    by_hour = [[h.replace("T", " ") + ":00", f(peak(xs, "load"), "", 2), f((peak(xs, "drv") or 0) * 100, "%", 1), f((peak(xs, "pg") or 0) * 100, "%", 1),
                f(peak(xs, "mem"), " MB", 0), f(peak(xs, "drss"), " MB", 0), f(mean(xs, "ev"), "", 2), f(peak(xs, "ev"), "", 2), peak(xs, "conns") or ""]
               for h, xs in sorted(hours.items(), reverse=True)]
    return now, by_hour, f"sampled every five minutes; last at {last[0][11:16]} UTC"


def when(ms: int) -> str:
    return datetime.fromtimestamp(ms / 1000, timezone.utc).strftime("%Y-%m-%d %H:%M") if ms else ""


def citizen_rows(cz: list[dict], days: int) -> list[list]:
    cutoff = int((datetime.now(timezone.utc) - timedelta(days=days)).timestamp() * 1000)
    rows = []
    for c in cz:
        new = "new" if c.get("first_seen", 0) >= cutoff else ""
        doors = ", ".join(f"{k} {v}" for k, v in sorted((c.get("origins") or {}).items(), key=lambda kv: -kv[1]))
        via = ", ".join(f"{k} {v}" for k, v in sorted((c.get("via") or {}).items(), key=lambda kv: -kv[1]))
        msgs = c.get("messages", 0)
        passed = c.get("passed", 0)
        rate = f"{100 * passed // msgs}%" if msgs else ""
        rows.append([c.get("name", ""), new, c.get("origin", ""), when(c.get("first_seen", 0)), when(c.get("last_seen", 0)),
                     c.get("arrivals", 0), doors, msgs, c.get("written", 0), rate, via, c.get("rooms_dug", 0)])
    return rows


def perf_rows(db, since: str) -> list[list]:
    """P7: the play page's own figures, a row a day, lab apart from the rest.

    Each report is one player's last twenty commands; the day's figure is the median
    of the reports' medians (and of their 95th percentiles), so one slow visitor does
    not set it.
    """
    if not has_table(db, "perf"):
        return []
    by: dict = {}
    for day, ip, n, rt50, rt95, rtmax, srv, fps, frame, ws in q(
        db, "SELECT day, ip, n, rt50, rt95, rtmax, srv, fps, frame, ws FROM perf WHERE day >= ? ORDER BY day DESC", (since,)
    ):
        by.setdefault((day, "lab" if is_lab(ip) else "organic"), []).append((ip, n, rt50, rt95, rtmax, srv, fps, frame, ws))

    def med(vals):
        vals = sorted(v for v in vals if v is not None)
        return vals[len(vals) // 2] if vals else None

    def show(v, unit=""):
        return "" if v is None else f"{v:.0f}{unit}" if v >= 10 else f"{v:.1f}{unit}"

    rows = []
    for (day, who), rs in sorted(by.items(), key=lambda kv: (kv[0][0], kv[0][1]), reverse=True):
        rows.append([
            day, who, len({r[0] for r in rs}), sum(r[1] for r in rs),
            show(med(r[2] for r in rs), " ms"), show(med(r[3] for r in rs), " ms"), show(max(r[4] for r in rs), " ms"),
            show(med(r[5] for r in rs), " ms"), show(med(r[6] for r in rs)), show(med(r[7] for r in rs), " ms"),
            f"{100 * sum(1 for r in rs if r[8]) // len(rs)}%",
        ])
    return rows


# A4: requests for things this site has never had: scanners trying doors. Counted in
# one row and left out of the error rate, so a night of probing is not a bad night.
PROBE = re.compile(
    r"(?i)(\.(php\d?|aspx?|jsp|cgi|env|sql|bak|old|ini|zip|tar|gz|rar|7z|pem|key|log|swp|DS_Store)(\b|$)"
    r"|/\.(git|svn|hg|aws|ssh|vscode|idea)\b|/\.env|/wp-|/wordpress|/xmlrpc|/phpmyadmin|/pma\b|/cgi-bin|/vendor/"
    r"|/actuator|/boaform|/hnap1|/owa\b|/autodiscover|/telescope|/_ignition|/solr|/jenkins|/manager/html"
    r"|/admin\b|/administrator|/login\b|/console\b|/config\.|/backup|/shell|/eval-stdin|/setup\.|/sdk\b|/druid)"
)


def is_probe(path: str, status: int) -> bool:
    return status in (400, 403, 404, 405) and bool(PROBE.search(path or ""))


def error_rows(db, since: str, reasons: list[list], since_ts: str | None = None) -> tuple[list[list], dict]:
    """A4: errors with the lab apart from everyone else, scanners' probes folded into
    one row, and beside a 422 the reason the driver most often gave on that path."""
    top_reason: dict = {}
    for path, reason, n in reasons:
        if path not in top_reason or n > top_reason[path][1]:
            top_reason[path] = (reason, n)
    by: dict = {}
    probes = {"organic": 0, "lab": 0, "paths": set()}
    # `since_ts` narrows a day's rows to the hours wanted (the last 24).
    for host, path, status, ip, n, last in q(
        db, "SELECT host, path, status, ip, COUNT(*), MAX(ts) FROM requests WHERE day >= ? AND ts >= ? AND status >= 400 GROUP BY host, path, status, ip",
        (since, since_ts or ""),
    ):
        who = "lab" if is_lab(ip) else "organic"
        if is_probe(path, status):
            probes[who] += n
            probes["paths"].add(path)
            continue
        row = by.setdefault((host, path, status), {"organic": 0, "lab": 0, "last": ""})
        row[who] += n
        row["last"] = max(row["last"], last or "")
    rows = []
    for (host, path, status), c in sorted(by.items(), key=lambda kv: (-kv[1]["organic"], -kv[1]["lab"]))[:40]:
        why = top_reason.get(path, ("", 0))[0] if status == 422 else ""
        rows.append([host, path, status, c["organic"], c["lab"], c["last"], why])
    return rows, {"organic": probes["organic"], "lab": probes["lab"], "paths": len(probes["paths"])}


def headline(db) -> list[dict]:
    """A1: the four figures the operator opens the page for, each for the last full
    day (UTC), with today so far, the day before and the week's daily mean beside it.
    The lab is left out of all four."""
    today = datetime.now(timezone.utc).date()
    days = [(today - timedelta(days=i)).strftime("%Y-%m-%d") for i in range(0, 9)]
    since = days[-1]
    came = {d: set() for d in days}
    total = {d: 0 for d in days}
    bad = {d: 0 for d in days}
    for day, ip, cls, path, status, n in q(
        db, "SELECT day, ip, ua_class, path, status, COUNT(*) FROM requests WHERE day >= ? GROUP BY day, ip, ua_class, path, status", (since,)
    ):
        if day not in total or is_lab(ip):
            continue
        if status < 300 and cls in ("browser", "agent", "tool"):
            came[day].add(ip)
        if not is_probe(path, status):
            total[day] += n
            if status >= 400:
                bad[day] += n
    names = {d: 0 for d in days}
    if has_table(db, "doors"):
        for day, n in q(db, "SELECT day, COUNT(DISTINCT actor) FROM doors WHERE day >= ? AND lab = 0 AND kind = 'arrive' AND fresh = 1 GROUP BY day", (since,)):
            if day in names:
                names[day] = n
    trips: dict = {d: [] for d in days}
    if has_table(db, "perf"):
        for day, ip, rt50 in q(db, "SELECT day, ip, rt50 FROM perf WHERE day >= ?", (since,)):
            if day in trips and not is_lab(ip):
                trips[day].append(rt50)

    def med(vals):
        vals = sorted(vals)
        return vals[len(vals) // 2] if vals else None

    def series(f):
        vals = [f(d) for d in days]
        week = [v for v in vals[1:8] if v is not None]
        return {"today": vals[0], "yesterday": vals[1], "before": vals[2], "week": (sum(week) / len(week)) if week else None}

    return [
        {"title": "People and agents who came", "note": "addresses, crawlers and the lab left out", "to": "#who-came", "unit": "", **series(lambda d: len(came[d]))},
        {"title": "Names through the gate", "note": "new names that arrived", "to": "#the-door", "unit": "", **series(lambda d: names[d])},
        {"title": "A command's round trip", "note": "median, Enter to reply, at the keyboard", "to": "#how-it-felt", "unit": " ms", "low_is_good": True, **series(lambda d: med(trips[d]))},
        {"title": "Errors per thousand requests", "note": "scanners' probes left out", "to": "#what-broke", "unit": "", "low_is_good": True,
         **series(lambda d: (1000 * bad[d] / total[d]) if total[d] else None)},
    ]


def collected(db) -> tuple[str, bool]:
    """A6: when the collector last took a request in, and whether that is stale: a
    page with a stopped collector looks like a quiet night."""
    row = q(db, "SELECT MAX(ts) FROM requests")
    last = row[0][0] if row and row[0][0] else None
    if not last:
        return "nothing collected yet", True
    try:
        at = datetime.fromisoformat(str(last).replace("Z", "+00:00"))
        if at.tzinfo is None:
            at = at.replace(tzinfo=timezone.utc)
    except ValueError:
        return f"last request collected {last}", False
    mins = int((datetime.now(timezone.utc) - at).total_seconds() // 60)
    ago = f"{mins} min ago" if mins < 120 else f"{mins // 60} hours ago"
    return f"last request collected {at.strftime('%a %m-%d %H:%M')} UTC ({ago})", mins > 45


def report(db: sqlite3.Connection, days: int) -> dict:
    since = (datetime.now(timezone.utc) - timedelta(days=days)).strftime("%Y-%m-%d")
    w = "WHERE day >= ? AND status < 300"  # redirects (http->https) are not visits
    out = {"since": since, "days": days}
    out["totals"] = q(db, f"SELECT host, COUNT(*), COUNT(DISTINCT visitor) FROM requests {w} GROUP BY host ORDER BY 2 DESC", (since,))
    out["by_class"] = q(db, f"SELECT host, ua_class, COUNT(*), COUNT(DISTINCT visitor) FROM requests {w} GROUP BY host, ua_class ORDER BY host, 3 DESC", (since,))
    out["root_split"] = q(db, f"SELECT host, accept_kind, ua_class, COUNT(*) FROM requests {w} AND path = '/' GROUP BY host, accept_kind, ua_class ORDER BY host, 4 DESC", (since,))
    out["key_paths"] = q(db, f"SELECT host, path, COUNT(*), COUNT(DISTINCT visitor) FROM requests {w} AND path IN ({','.join('?' * len(KEY_PATHS))}) GROUP BY host, path ORDER BY host, 3 DESC", (since, *KEY_PATHS))
    out["agents"] = q(db, f"SELECT ua_class, ua_name, COUNT(*), COUNT(DISTINCT visitor), MAX(ts) FROM requests {w} AND ua_class IN ('ai-crawler','tool','agent') GROUP BY ua_class, ua_name ORDER BY 3 DESC LIMIT 40", (since,))
    # A2: every kind of visitor in one table, by class and name; the page filters it.
    out["visitors"] = q(db, f"SELECT ua_class, ua_name, COUNT(*), COUNT(DISTINCT visitor), MAX(ts) FROM requests {w} GROUP BY ua_class, ua_name ORDER BY 3 DESC LIMIT 80", (since,))
    out["referers"] = q(db, f"SELECT referer_host, COUNT(*) FROM requests {w} AND referer_host != '' AND referer_host NOT IN ('agentscanplay.com','www.agentscanplay.com','foxing.dev','www.foxing.dev','moondial.ai','www.moondial.ai') GROUP BY referer_host ORDER BY 2 DESC LIMIT 25", (since,))
    out["daily"] = q(db, f"SELECT day, host, COUNT(*), COUNT(DISTINCT visitor) FROM requests {w} GROUP BY day, host ORDER BY day DESC, host", (since,))
    out["error_senders"] = q(db, "SELECT host, path, status, ua_class, ua_name, COUNT(*), COUNT(DISTINCT visitor), MAX(ts) FROM requests WHERE day >= ? AND status >= 400 AND status != 404 GROUP BY host, path, status, ua_class, ua_name ORDER BY 6 DESC LIMIT 20", (since,))
    out["refusals"], out["refusals_note"] = refusals()
    out["ops"], out["ops_verbs"], out["ops_note"] = ops()
    out["vitals"], out["vitals_hours"], out["vitals_note"] = vitals(db)
    out["errors"], out["probes"] = error_rows(db, since, out["refusals"])
    # The same table for the last day and the last week, for the page's filter: a
    # month's errors bury last night's.
    now = datetime.now(timezone.utc)
    out["errors_by"] = []
    for label, key, delta in (("the last 24 hours", "24h", timedelta(hours=24)), ("the last 7 days", "7d", timedelta(days=7))):
        if delta >= timedelta(days=days):
            continue  # not narrower than the page's own window
        start = now - delta
        rows, probes = error_rows(db, start.strftime("%Y-%m-%d"), out["refusals"], start.strftime("%Y-%m-%dT%H:%M:%S") if key == "24h" else None)
        senders = q(db, "SELECT host, path, status, ua_class, ua_name, COUNT(*), COUNT(DISTINCT visitor), MAX(ts) FROM requests WHERE day >= ? AND ts >= ? AND status >= 400 AND status != 404 GROUP BY host, path, status, ua_class, ua_name ORDER BY 6 DESC LIMIT 20",
                    (start.strftime("%Y-%m-%d"), start.strftime("%Y-%m-%dT%H:%M:%S") if key == "24h" else ""))
        out["errors_by"].append((label, key, rows, probes, senders))
    out["errors_by"].append((f"the whole window ({window_label(days)})", "all", out["errors"], out["probes"], out["error_senders"]))
    out["headline"] = headline(db)
    out["collected"], out["stale"] = collected(db)
    out["perf"] = perf_rows(db, since)
    cz, why = citizens(days)
    out["citizens"] = citizen_rows(cz, days)
    out["citizens_note"] = why
    out["networks"] = networks(db, since)
    out["funnel"] = funnel(db, since)
    # Organic: not from the lab, and in the game, cloaked at least (a cloak read the
    # Threshold; a gate acted; a name arrived). An address that only initialized is a
    # directory crawler and is counted, not listed.
    out["organic"] = [r for r in out["networks"] if r[1] != "lab" and (r[6] or r[7] or r[8])]
    out["crawlers"] = sum(1 for r in out["networks"] if r[1] != "lab" and r[5] and not (r[6] or r[7] or r[8]))
    return out


def text(r: dict) -> str:
    lines = [f"Analytics since {r['since']} ({r['days']} days), 2xx requests only unless noted (redirects excluded)",
             f"{'STALE: ' if r['stale'] else ''}{r['collected']}", "",
             "The last full day (UTC), with today so far, the day before, and the week's daily mean; the lab left out"]
    for h in r["headline"]:
        lines.append(f"  {h['title']:<32} {fig(h['yesterday'], h['unit']):>9}   today {fig(h['today'], h['unit'])}, before {fig(h['before'], h['unit'])}, week {fig(h['week'], h['unit'])}")
    lines.append("")
    lines.append("Totals (host, requests, unique visitors)")
    for h, n, v in r["totals"]:
        lines.append(f"  {h:<22} {n:>7} {v:>7}")
    lines.append("\nBy visitor class (host, class, requests, uniques)")
    for h, c, n, v in r["by_class"]:
        lines.append(f"  {h:<22} {c:<11} {n:>7} {v:>7}")
    lines.append("\nRoot '/' served as (host, accept, class, requests): text = the CLEAR invitation, html = the page")
    for h, a, c, n in r["root_split"]:
        lines.append(f"  {h:<22} {a:<6} {c:<11} {n:>7}")
    lines.append("\nKey paths (host, path, requests, uniques)")
    for h, p, n, v in r["key_paths"]:
        lines.append(f"  {h:<22} {p:<34} {n:>7} {v:>7}")
    lines.append("\nAgents, AI crawlers and tools (class, name, requests, uniques, last seen)")
    for c, name, n, v, last in r["agents"]:
        lines.append(f"  {c:<11} {name:<24} {n:>7} {v:>7}  {last}")
    lines.append("\nReferrers")
    for ref, n in r["referers"]:
        lines.append(f"  {ref:<40} {n:>7}")
    for label, _key, rows, _probes, _senders in r["errors_by"]:
        lines.append(f"\nErrors, {label} (host, path, status, organic, lab, last, the driver's most given reason)")
        for h, p, s, n, lab, last, why in rows:
            lines.append(f"  {h:<22} {p:<34} {s:>4} {n:>6} {lab:>6}  {last}  {why}")
    pr = r["probes"]
    lines.append(f"  probes: {pr['organic']:,} requests for {pr['paths']:,} paths this site has never had (scanners), {pr['lab']:,} from the lab; not in the rate")
    lines.append("\nWho sent the errors (host, path, status, class, name, count, addresses, last)")
    for row in r["error_senders"]:
        lines.append("  " + "  ".join(str(c) for c in row))
    lines.append(f"\nWhy the driver refused ({r['refusals_note']}): path, reason, count")
    for row in r["refusals"]:
        lines.append("  " + "  ".join(str(c) for c in row))
    lines.append(f"\nThe driver's side ({r['ops_note']})")
    for row in r["ops"]:
        lines.append("  " + "  ".join(str(c) for c in row))
    lines.append("  verb, count, mean, worst, the log's share, appends each, reads past the tail, in all")
    for row in r["ops_verbs"][:15]:
        lines.append("  " + "  ".join(str(c) for c in row))
    lines.append("\nOrganic agent visits (address, first seen, last seen, requests, mcp calls, cloaks, gates, names): not from the lab, and in the game, cloaked at least")
    for row in r["organic"]:
        lines.append("  " + "  ".join(str(c) for c in row[:9] if c != ""))
    lines.append(f"  ({r['crawlers']} other addresses only initialized: directory crawlers, not listed)")
    lines.append("\nThe door, per day (day, lab/organic, addresses that initialized, cloaked, checked, sent through the gate; names that arrived)")
    for row in r["funnel"]:
        lines.append("  " + "  ".join(str(c) for c in row))
    lines.append("\nNetworks (address, lab, first seen, last seen, requests, mcp, cloaks, gates, names, classes, hosts)")
    for row in r["networks"]:
        lines.append("  " + "  ".join(str(c) for c in row))
    lines.append("\nPlay page performance (day, lab/organic, addresses, commands, round trip median, 95th, worst, world's share, frames a second, frame cost, on the socket)")
    for row in r.get("perf", []):
        lines.append("  " + "  ".join(str(c) for c in row))
    lines.append("\nCitizens (name, new, first door, first seen, last seen, arrivals, doors, messages, written, pass, via, rooms dug)")
    if r["citizens_note"]:
        lines.append(f"  {r['citizens_note']}")
    for row in r["citizens"]:
        lines.append("  " + "  ".join(str(c) for c in row))
    return "\n".join(lines)


def fig(v, unit: str = "") -> str:
    """A number as the page says it: thousands separated, one decimal under ten."""
    if v is None:
        return "no data"
    if isinstance(v, float) and v != int(v) and abs(v) < 10:
        return f"{v:.1f}{unit}"
    return f"{round(v):,}{unit}"


DAY = re.compile(r"^(\d{4}-\d{2}-\d{2})(?:[T ](\d{2}:\d{2})(?::\d{2})?(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?)?$")
FOLD = 20


def cell(c) -> tuple[str, bool]:
    """A6: a cell as shown, and whether it is a number (right-aligned). A day reads
    "Tue 11-10"; a count has its thousands apart."""
    if isinstance(c, bool):
        return str(c), False
    if isinstance(c, int):
        return f"{c:,}", True
    if isinstance(c, float):
        return fig(c), True
    text = str(c)
    m = DAY.match(text)
    if m:
        try:
            d = datetime.strptime(m.group(1), "%Y-%m-%d")
            return d.strftime("%a %m-%d") + (f" {m.group(2)}" if m.group(2) else ""), False
        except ValueError:
            pass
    return text, bool(re.fullmatch(r"-?[\d,.]+ ?(ms|%|s)?", text)) and text != ""


def table(headers, rows, row_class=None) -> str:
    """A table whose numbers sit right, whose lab rows are dimmed, and which folds
    after twenty rows (A6)."""
    th = "".join(f"<th>{html.escape(str(h))}</th>" for h in headers)

    def tr(row) -> str:
        cells = [cell(c) for c in row]
        names = (["lab"] if len(row) > 1 and row[1] == "lab" else []) + ([row_class(row)] if row_class else [])
        lab = f' class="{" ".join(names)}"' if names else ""
        return f"<tr{lab}>" + "".join(f"<td{' class=n' if num else ''}>{html.escape(t)}</td>" for t, num in cells) + "</tr>"

    def whole(rs) -> str:
        return f"<div class=scroll><table><thead><tr>{th}</tr></thead><tbody>{''.join(tr(r) for r in rs)}</tbody></table></div>"

    rows = list(rows)
    if not rows:
        return f"<table><thead><tr>{th}</tr></thead><tbody><tr><td colspan={len(headers)} class=none>nothing yet</td></tr></tbody></table>"
    if len(rows) <= FOLD + 3:
        return whole(rows)
    return whole(rows[:FOLD]) + f"<details><summary>{len(rows) - FOLD:,} more rows</summary>{whole(rows[FOLD:])}</details>"


def errors_panes(windows) -> str:
    """The errors and who sent them, once for each window, with chips to choose: the
    last 24 hours (shown first), the last 7 days, the whole window."""
    chips, panes = [], []
    for i, (label, key, rows, pr, senders) in enumerate(windows):
        chips.append(f"<button type=button data-pane=\"err-{key}\"{' class=on' if i == 0 else ''}>{html.escape(label)}</button>")
        panes.append(
            f"<div id=\"err-{key}\"{'' if i == 0 else ' hidden'}>"
            + table(["host", "path", "status", "organic", "lab", "last", "the driver's most given reason"], rows)
            + f"<p class=\"sub\">Not in the table or the rate: {pr['organic']:,} requests for {pr['paths']:,} paths this site has never had (scanners trying doors: <code>.php</code>, <code>.env</code>, <code>/wp-</code>), and {pr['lab']:,} of the same from the lab.</p>"
            + "<h3>Who sent them</h3>"
            + table(["host", "path", "status", "class", "name", "count", "addresses", "last"], senders)
            + "</div>"
        )
    return f"<p class=\"chips\" data-panes>{''.join(chips)}</p>{''.join(panes)}"


def class_filter(rows) -> str:
    """A2: chips over the visitors table, one a class; a click shows that class alone."""
    classes = sorted({str(r[0]) for r in rows})
    chips = "".join(f"<button type=button data-c=\"c-{slug(c)}\">{html.escape(c)}</button>" for c in classes)
    return f"<p class=\"chips\" data-filter><button type=button data-c=\"\" class=on>all</button>{chips}</p>"


FILTER_JS = """<script>
for (const bar of document.querySelectorAll("[data-panes]")) {
  bar.addEventListener("click", (e) => {
    const b = e.target.closest("button"); if (!b) return;
    for (const o of bar.querySelectorAll("button")) { o.classList.toggle("on", o === b); document.getElementById(o.dataset.pane).hidden = o !== b; }
  });
}
for (const bar of document.querySelectorAll("[data-filter]")) {
  const scope = [];
  for (let n = bar.nextElementSibling; n && n.tagName !== "H3" && n.tagName !== "H2"; n = n.nextElementSibling) scope.push(n);
  bar.addEventListener("click", (e) => {
    const b = e.target.closest("button"); if (!b) return;
    for (const o of bar.querySelectorAll("button")) o.classList.toggle("on", o === b);
    for (const el of scope) for (const tr of el.querySelectorAll("tbody tr")) tr.hidden = !!b.dataset.c && !tr.classList.contains(b.dataset.c);
    for (const el of scope) if (el.tagName === "DETAILS" && b.dataset.c) el.open = true;
  });
}
</script>"""


def tiles(heads: list[dict]) -> str:
    """A1: the top line."""
    out = []
    for h in heads:
        y, b = h["yesterday"], h["before"]
        trend = ""
        if y is not None and b is not None and y != b:
            up = y > b
            good = (not up) if h.get("low_is_good") else up
            trend = f"<span class=\"{'good' if good else 'bad'}\">{'&#9650;' if up else '&#9660;'}</span>"
        out.append(
            f"<a class=tile href=\"{h['to']}\"><span class=k>{html.escape(h['title'])}</span>"
            f"<span class=v>{html.escape(fig(y, h['unit']))} {trend}</span>"
            f"<span class=s>yesterday &middot; today so far {html.escape(fig(h['today'], h['unit']))} &middot; the day before {html.escape(fig(b, h['unit']))} &middot; the week's mean {html.escape(fig(h['week'], h['unit']))}</span>"
            f"<span class=s>{html.escape(h['note'])}</span></a>"
        )
    return f"<div class=tiles>{''.join(out)}</div>"


def slug(title: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", title.lower().split(":")[0].split("(")[0]).strip("-")


def page(r: dict) -> str:
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    cz_note = f"<p class=\"sub\">{html.escape(r['citizens_note'])}</p>" if r["citizens_note"] else ""
    pr = r["probes"]
    # A2: four sections that answer the operator's four questions, and a fold for the
    # tables that answer one a month.
    groups = [
        ("Who came", "who-came", [
            ("Hosts", table(["host", "requests", "unique visitors"], r["totals"])),
            ("Visitors by class and name", class_filter(r["visitors"])
                + table(["class", "name", "requests", "uniques", "last seen"], r["visitors"], row_class=lambda row: f"c-{slug(str(row[0]))}")),
            ("Organic agent visits: not from the lab, and in the game, cloaked at least", table(
                ["address", "", "first seen", "last seen", "requests", "mcp calls", "cloaks", "gates", "names taken", "classes", "hosts"], r["organic"])
                + f"<p class=\"sub\">{r['crawlers']:,} other addresses only initialized: directory crawlers, not listed. Lab nets come from FOXING_LAB_NETS at render time, so a net added later relabels the past.</p>"),
            ("Where the humans came from", table(["referrer", "requests"], r["referers"])),
        ]),
        ("The door", "the-door", [
            ("Per day: how far each address got", table(["day", "", "initialized", "cloaked", "checked", "gate", "names arrived"], r["funnel"])),
            ("Citizens: who came, by what door, and what they did", cz_note + table(
                ["name", "", "first door", "first seen", "last seen", "arrivals", "doors", "messages", "written", "pass", "via", "rooms dug"], r["citizens"])),
        ]),
        ("How it felt", "how-it-felt", [
            ("What a player at the keyboard waited", table(
                ["day", "", "addresses", "commands", "round trip (median)", "95th", "worst", "world's share", "frames/s", "frame cost", "on the socket"], r.get("perf", []))
                + "<p class=\"sub\">Each report is a player's last twenty commands, sent by the play page itself; the round trip is from Enter to the reply, and the world's share is how long the driver held it. <code>?perf=1</code> on the play page shows the same live.</p>"),
            (f"The driver's side ({r['ops_note']})", table(["", "", ""], r["ops"])),
            ("By verb: the world's own time, heaviest in all first", table(
                ["verb", "count", "mean", "worst", "the log's share", "appends each", "reads past the tail", "in all"], r["ops_verbs"])),
            (f"The box ({r.get('vitals_note', '')})", table(["", "", ""], r.get("vitals", []))),
            ("The box, the last day by the hour", table(
                ["hour (UTC)", "load, peak", "driver CPU, peak", "PostgreSQL CPU, peak", "memory in use, peak", "driver memory", "events a second, mean", "peak", "connections"], r.get("vitals_hours", []))
                + "<p class=\"sub\">CPU is a share of one core over five minutes, so a burst shorter than that is smoothed; a load test reads the box every five seconds with the server check's <code>vitals</code>.</p>"),
        ]),
        ("What broke", "what-broke", [
            ("Errors, the lab apart", errors_panes(r["errors_by"])),
            (f"Why the driver refused ({r['refusals_note']})", table(["path", "reason", "count"], r["refusals"])),
        ]),
    ]
    more = [
        ("Daily", table(["day", "host", "requests", "uniques"], r["daily"])),
        ("Key paths", table(["host", "path", "requests", "uniques"], r["key_paths"])),
        ("Root / served as: text is the CLEAR invitation, html the page", table(["host", "accept", "class", "requests"], r["root_split"])),
        ("By host and class", table(["host", "class", "requests", "uniques"], r["by_class"])),
        ("Networks: every address in the window", table(
            ["address", "", "first seen", "last seen", "requests", "mcp calls", "cloaks", "gates", "names taken", "classes", "hosts"], r["networks"])),
    ]
    nav = " &middot; ".join(f"<a href=\"#{gid}\">{html.escape(t)}</a>" for t, gid, _ in groups) + " &middot; <a href=\"#more\">More</a>"
    body = f"<p class=\"nav\">{nav}</p>" + "".join(
        f"<section><h2 id=\"{gid}\">{html.escape(t)}</h2>" + "".join(f"<h3>{html.escape(st)}</h3>{tb}" for st, tb in subs) + "</section>"
        for t, gid, subs in groups
    ) + "<section><h2 id=\"more\">More</h2><details><summary>Five tables for a question once a month: daily, key paths, how the root was served, hosts by class, every address</summary>" + "".join(
        f"<h3>{html.escape(st)}</h3>{tb}" for st, tb in more) + "</details></section>" + FILTER_JS
    stale = " class=\"stale\"" if r["stale"] else ""
    windows = " &middot; ".join(
        f"<b>{html.escape(label)}</b>" if here else f"<a href=\"{html.escape(name)}\">{html.escape(label)}</a>"
        for label, name, here in r.get("windows", [(window_label(r["days"]), "", True)])
    )
    return f"""<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex, nofollow"><title>Foxing analytics</title>
<style>
:root {{ --night:#0b0d12; --ink:#e8e4da; --dim:#9a948a; --dust:#d9b57a; --line:#262a33; --panel:#11141b; }}
html {{ background: var(--night); color: var(--ink); font: 15px/1.5 "Iowan Old Style", Palatino, Georgia, serif; }}
body {{ margin: 0; }} main {{ max-width: 960px; margin: 0 auto; padding: 32px 20px 80px; }}
h1 {{ font-weight: 400; font-size: 32px; margin: 0 0 4px; }} .sub {{ color: var(--dim); margin: 0 0 24px; }}
h2 {{ font-weight: 400; font-size: 20px; color: var(--dust); margin: 32px 0 8px; }}
table {{ border-collapse: collapse; width: 100%; font: 13px/1.4 ui-monospace, Menlo, Consolas, monospace; }}
th, td {{ text-align: left; padding: 6px 10px; border-bottom: 1px solid var(--line); }} th {{ color: var(--dim); font-weight: 400; }}
td.n {{ text-align: right; font-variant-numeric: tabular-nums; }} td.none {{ color: var(--dim); }} tr:hover td {{ background: var(--panel); }}
td:nth-child(2):not(.n) {{ color: var(--dust); }} tr.lab td {{ color: var(--dim); opacity: .6; }}
.scroll {{ overflow-x: auto; }} td, th {{ white-space: nowrap; }} td:last-child {{ white-space: normal; }}
details {{ margin: 0; }} summary {{ cursor: pointer; color: var(--dim); font: 13px/2.2 ui-monospace, Menlo, Consolas, monospace; padding-left: 10px; }}
code {{ font: 12.5px ui-monospace, Menlo, Consolas, monospace; color: var(--dust); }}
.tiles {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(210px, 1fr)); gap: 12px; margin: 8px 0 8px; }}
.tile {{ display: flex; flex-direction: column; gap: 4px; padding: 14px 16px; background: var(--panel); border: 1px solid var(--line); border-radius: 8px; color: inherit; text-decoration: none; }}
.tile:hover {{ border-color: var(--dust); }}
.tile .k {{ color: var(--dust); font-size: 14px; }} .tile .v {{ font-size: 34px; line-height: 1.15; font-variant-numeric: tabular-nums; }}
.tile .s {{ color: var(--dim); font-size: 12px; line-height: 1.45; }}
.good {{ color: #8fc79a; font-size: 16px; }} .bad {{ color: #d98f8f; font-size: 16px; }}
.stale {{ color: #d98f8f; }}
h3 {{ font-weight: 400; font-size: 15px; color: var(--ink); margin: 22px 0 6px; }}
section {{ border-top: 1px solid var(--line); margin-top: 36px; }} section h2 {{ font-size: 24px; margin-top: 22px; }}
.nav {{ margin: 14px 0 0; }} .window {{ margin: 18px 0 2px; color: var(--ink); }} .window b {{ color: var(--dust); font-weight: 400; border-bottom: 1px solid var(--dust); }} a {{ color: var(--dust); }}
.chips {{ margin: 0 0 8px; display: flex; flex-wrap: wrap; gap: 6px; }}
.chips button {{ font: 12px ui-monospace, Menlo, Consolas, monospace; color: var(--dim); background: none; border: 1px solid var(--line); border-radius: 999px; padding: 2px 10px; cursor: pointer; }}
.chips button.on {{ color: var(--night); background: var(--dust); border-color: var(--dust); }}
tr[hidden] {{ display: none; }}
html {{ scroll-behavior: smooth; }} h2 {{ scroll-margin-top: 12px; }}
</style></head><body><main>
<h1>Foxing analytics</h1>
<p class="sub"><span{stale}>{'Stale: ' if r['stale'] else ''}{html.escape(r['collected'])}</span> · rendered {now}</p>
{tiles(r['headline'])}
<p class="sub window">Everything below is for: {windows}</p>
<p class="sub">Since {html.escape(r['since'])} (UTC) · 2xx requests unless noted · addresses kept {html.escape(os.environ.get("ANALYTICS_RETENTION_DAYS", "90"))} days, operator-only · lab = {html.escape(LAB_NETS or "no lab nets set")}</p>
{body}
</main></body></html>"""


def window_label(days: int) -> str:
    return "yesterday and today" if days == 1 else f"{days} days"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--days", type=int, default=30)
    ap.add_argument("--html", action="store_true", help="also write the admin page")
    args = ap.parse_args()
    if not os.path.exists(DB):
        print(f"no analytics store at {DB}; set ANALYTICS_DB (on the game server: /srv/agentscanplay/shared/analytics/analytics.db)")
        return 1
    db = sqlite3.connect(DB)
    r = report(db, args.days)
    print(text(r))
    if args.html:
        os.makedirs(HTML_DIR, exist_ok=True)
        # A3: the same page for three windows, each a file beside the others with
        # links between them: the one asked for is the index.
        windows = [(args.days, "index.html")] + [(d, f"{d}.html") for d in (7, 1) if d != args.days]
        windows.sort(key=lambda w: w[0])
        for days, name in windows:
            rr = r if days == args.days else report(db, days)
            rr["windows"] = [(window_label(d), n, d == days) for d, n in windows]
            tmp = os.path.join(HTML_DIR, name + ".tmp")
            with open(tmp, "w", encoding="utf-8") as fh:
                fh.write(page(rr))
            os.replace(tmp, os.path.join(HTML_DIR, name))
            print(f"\nwrote {HTML_DIR}/{name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
