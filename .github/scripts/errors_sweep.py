#!/usr/bin/env python3
"""A sweep of the HTTP errors in the analytics store, for security risks and
misconfigurations (David, 11-28). Read-only. Run on a server by the Server check
workflow, whose log is PUBLIC: this prints counts and paths only. No address, no
name, no full user-agent string ever leaves the box; addresses are ranked ("#1")
and said to be the lab's or not.

Standard library only.
"""
import glob
import ipaddress
import os
import re
import sqlite3
import sys
from datetime import datetime, timedelta, timezone

DAYS = int(sys.argv[1]) if len(sys.argv) > 1 and sys.argv[1].isdigit() else 14
CANDIDATES = [
    os.environ.get("ANALYTICS_DB", ""),
    "/srv/foxing/shared/analytics/analytics.db",
    "/srv/moondial/shared/analytics/analytics.db",
    "/srv/agentscanplay/shared/analytics/analytics.db",
]
PROBE = re.compile(
    r"(?i)(\.(php\d?|aspx?|jsp|cgi|env|sql|bak|old|ini|zip|tar|gz|rar|7z|pem|key|log|swp|DS_Store|ya?ml|toml|conf|cfg|db|sqlite3?)(\b|$)"
    r"|/\.(git|svn|hg|aws|ssh|vscode|idea|docker|kube)\b|/\.env|/wp-|/wordpress|/xmlrpc|/phpmyadmin|/pma\b|/cgi-bin|/vendor/"
    r"|/actuator|/boaform|/hnap1|/owa\b|/autodiscover|/telescope|/_ignition|/solr|/jenkins|/manager/html"
    r"|/admin\b|/administrator|/login\b|/console\b|/config\.|/backup|/shell|/eval-stdin|/setup\.|/sdk\b|/druid"
    r"|/server-status|/debug|/metrics\b|/swagger|/api-docs|/graphql|\.\./|%2e%2e|/etc/passwd|/proc/self)"
)
# Paths of ours that are private by design: an answer other than a refusal is a finding.
PRIVATE = re.compile(r"^/(v0/ops|v0/admin|analytics|stats|report)(/|$)")


def lab_nets():
    text = os.environ.get("FOXING_LAB_NETS", "")
    if not text:
        for f in ("/etc/foxing/driver.env", "/etc/moondial/driver.env"):
            try:
                for line in open(f):
                    m = re.match(r"^(?:export )?(?:FOXING|MOONDIAL)_LAB_NETS=['\"]?([^'\"\n]*)", line)
                    if m:
                        text = m.group(1)
            except OSError:
                pass
    nets = []
    for part in text.split(","):
        try:
            nets.append(ipaddress.ip_network(part.strip(), strict=False))
        except ValueError:
            pass
    return nets


NETS = lab_nets()


def is_lab(ip):
    try:
        a = ipaddress.ip_address(ip)
    except ValueError:
        return False
    return any(a in n for n in NETS)


def clip(path, n=70):
    path = re.sub(r"[0-9a-fA-F]{24,}", "<hex>", path or "")
    path = re.sub(r"(token|key|pass(word|phrase)?|secret)=[^&]*", r"\1=<cut>", path, flags=re.I)
    return path if len(path) <= n else path[: n - 1] + "…"


def table(title, head, rows, limit=25):
    print(f"\n### {title}")
    if not rows:
        print("  (none)")
        return
    rows = [[str(c) for c in r] for r in rows[:limit]]
    widths = [max(len(h), *(len(r[i]) for r in rows)) for i, h in enumerate(head)]
    print("  " + "  ".join(h.ljust(w) for h, w in zip(head, widths)))
    for r in rows:
        print("  " + "  ".join(c.ljust(w) for c, w in zip(r, widths)))


def main():
    path = next((p for p in CANDIDATES if p and os.path.exists(p)), None)
    if not path:
        print("no analytics store found on this host:", [p for p in CANDIDATES if p])
        return
    db = sqlite3.connect(f"file:{path}?mode=ro", uri=True, timeout=20)
    since = (datetime.now(timezone.utc) - timedelta(days=DAYS)).strftime("%Y-%m-%d")
    q = lambda sql, *a: db.execute(sql, a).fetchall()
    total, first, last = q("SELECT COUNT(*), MIN(ts), MAX(ts) FROM requests WHERE day >= ?", since)[0]
    print(f"## HTTP error sweep: {os.uname().nodename}, last {DAYS} days ({first} to {last}), {total} requests; lab networks known: {len(NETS)}")
    cols = {r[1] for r in q("PRAGMA table_info(requests)")}
    method = "method" if "method" in cols else "''"

    table("Status by host", ["host", "status", "requests"],
          q("SELECT host, status, COUNT(*) FROM requests WHERE day >= ? GROUP BY host, status ORDER BY host, status", since), 80)

    rows = q(f"SELECT host, {method}, path, status, ip, ua_class, ua_name, referer_host, COUNT(*), MAX(ts), MAX(rt) FROM requests WHERE day >= ? AND (status >= 400 OR status < 200) GROUP BY host, {method}, path, status, ip, ua_class, ua_name, referer_host", since)
    ok_rows = q(f"SELECT host, {method}, path, status, COUNT(*), COUNT(DISTINCT ip), MAX(ts), MAX(bytes) FROM requests WHERE day >= ? AND status < 400 GROUP BY host, {method}, path, status", since)

    # 1. What a probe was ANSWERED with something other than a refusal.
    answered = [(h, m, clip(p), s, n, ips, t, b) for h, m, p, s, n, ips, t, b in ok_rows if PROBE.search(p or "") or PRIVATE.search(p or "")]
    answered.sort(key=lambda r: -r[4])
    table("FINDING CANDIDATES: probe-shaped or private paths answered below 400 (a 200 with a body here is the thing to look at; a 301/302 to https or to / is nginx being polite)",
          ["host", "method", "path", "status", "requests", "addresses", "last", "largest body"], answered, 40)

    # 2. Server errors.
    agg = {}
    for h, m, p, s, ip, cls, name, ref, n, t, rt in rows:
        if s >= 500:
            a = agg.setdefault((h, m, clip(p), s), [0, set(), "", 0.0, 0])
            a[0] += n; a[1].add(ip); a[2] = max(a[2], t); a[3] = max(a[3], rt or 0)
            a[4] += n if is_lab(ip) else 0
    table("Server errors (5xx): each is a misconfiguration or a fault until shown otherwise",
          ["host", "method", "path", "status", "requests", "addresses", "of them lab", "last", "slowest s"],
          [(h, m, p, s, a[0], len(a[1]), a[4], a[2], f"{a[3]:.1f}") for (h, m, p, s), a in sorted(agg.items(), key=lambda kv: -kv[1][0])], 40)

    # 3. Refusals at the private doors, and auth failures: who is trying.
    agg = {}
    for h, m, p, s, ip, cls, name, ref, n, t, rt in rows:
        if s in (401, 403, 429) or PRIVATE.search(p or ""):
            a = agg.setdefault((h, m, clip(p), s), [0, {}, ""])
            a[0] += n; a[1][ip] = a[1].get(ip, 0) + n; a[2] = max(a[2], t)
    table("Refusals: 401, 403, 429, and anything at a private path (many requests from one address is someone trying a door)",
          ["host", "method", "path", "status", "requests", "addresses", "most from one", "that one is lab", "last"],
          [(h, m, p, s, a[0], len(a[1]), max(a[1].values()), "yes" if is_lab(max(a[1], key=a[1].get)) else "no", a[2]) for (h, m, p, s), a in sorted(agg.items(), key=lambda kv: -kv[1][0])], 40)

    # 4. Errors that are ours: not probes, sent by browsers and agents, or linked from our own pages.
    agg = {}
    for h, m, p, s, ip, cls, name, ref, n, t, rt in rows:
        if 400 <= s < 500 and s not in (401, 403, 429) and not PROBE.search(p or "") and not PRIVATE.search(p or ""):
            a = agg.setdefault((h, m, clip(p), s), [0, set(), 0, 0, "", 0])
            a[0] += n; a[1].add(ip); a[4] = max(a[4], t)
            a[2] += n if cls in ("browser",) else 0
            a[3] += n if ref and ("agentscanplay" in ref or ref == h) else 0
            a[5] += n if is_lab(ip) else 0
    ours = sorted(agg.items(), key=lambda kv: (-kv[1][3], -kv[1][2], -kv[1][0]))
    table("Client errors that are not probes (linked from our own pages first: those are broken links or a page asking for what is not there)",
          ["host", "method", "path", "status", "requests", "addresses", "from browsers", "from our own pages", "of them lab", "last"],
          [(h, m, p, s, a[0], len(a[1]), a[2], a[3], a[5], a[4]) for (h, m, p, s), a in ours], 45)

    # 5. The probes themselves: volume and shape.
    agg = {}; by_ip = {}
    for h, m, p, s, ip, cls, name, ref, n, t, rt in rows:
        if PROBE.search(p or ""):
            a = agg.setdefault((h, clip(p, 50), s), [0, set()])
            a[0] += n; a[1].add(ip)
        by_ip[ip] = by_ip.get(ip, 0) + n
    table("Probes refused (top paths)", ["host", "path", "status", "requests", "addresses"],
          [(h, p, s, a[0], len(a[1])) for (h, p, s), a in sorted(agg.items(), key=lambda kv: -kv[1][0])], 30)
    print(f"\n  probe requests refused: {sum(a[0] for a in agg.values())} over {len(set().union(*[a[1] for a in agg.values()]) if agg else set())} addresses")

    # 6. Who sends the errors: ranked, never named.
    top = sorted(by_ip.items(), key=lambda kv: -kv[1])[:15]
    out = []
    for rank, (ip, n) in enumerate(top, 1):
        days, okn = q("SELECT COUNT(DISTINCT day), SUM(status < 400) FROM requests WHERE day >= ? AND ip = ?", since, ip)[0]
        peak = q("SELECT MAX(c) FROM (SELECT COUNT(*) c FROM requests WHERE day >= ? AND ip = ? AND status >= 400 GROUP BY substr(ts, 1, 16))", since, ip)[0][0]
        cls = q("SELECT ua_class, COUNT(*) FROM requests WHERE day >= ? AND ip = ? GROUP BY ua_class ORDER BY 2 DESC LIMIT 1", since, ip)[0][0]
        out.append((f"#{rank}", "lab" if is_lab(ip) else "not lab", n, okn or 0, days, peak or 0, cls))
    table("Addresses sending the most errors (ranked, not named; 'peak a minute' over about 60 with no ban is a rate limit not biting)",
          ["address", "whose", "errors", "good requests", "days seen", "peak errors a minute", "mostly"], out)

    # 7. Methods nobody here uses.
    if method == "method":
        table("Unusual methods", ["host", "method", "status", "requests", "addresses"],
              q("SELECT host, method, status, COUNT(*), COUNT(DISTINCT ip) FROM requests WHERE day >= ? AND method NOT IN ('GET','POST','HEAD','OPTIONS') GROUP BY host, method, status ORDER BY 4 DESC", since))
        table("OPTIONS and HEAD answered with an error", ["host", "method", "path", "status", "requests"],
              [(h, m, clip(p), s, n) for h, m, p, s, n in q("SELECT host, method, path, status, COUNT(*) FROM requests WHERE day >= ? AND method IN ('OPTIONS','HEAD') AND status >= 400 GROUP BY host, method, path, status ORDER BY 5 DESC LIMIT 15", since)])

    # 8. Hosts we do not serve, asked for by name.
    table("Hosts asked for (a name here that is not ours means the default server answers for anything pointed at the box)",
          ["host", "requests", "errors", "addresses"],
          q("SELECT host, COUNT(*), SUM(status >= 400), COUNT(DISTINCT ip) FROM requests WHERE day >= ? GROUP BY host ORDER BY 2 DESC LIMIT 20", since))

    # 9. Slow answers and the day's shape.
    table("Slowest paths (95th-ish: the largest time, where there were 20 or more requests)", ["host", "path", "requests", "largest s", "mean s"],
          [(h, clip(p), n, f"{mx:.1f}", f"{av:.2f}") for h, p, n, mx, av in q("SELECT host, path, COUNT(*), MAX(rt), AVG(rt) FROM requests WHERE day >= ? GROUP BY host, path HAVING COUNT(*) >= 20 AND MAX(rt) >= 5 ORDER BY 4 DESC LIMIT 12", since)])
    table("Errors by day", ["day", "requests", "4xx", "5xx", "addresses with an error"],
          q("SELECT day, COUNT(*), SUM(status BETWEEN 400 AND 499), SUM(status >= 500), COUNT(DISTINCT CASE WHEN status >= 400 THEN ip END) FROM requests WHERE day >= ? GROUP BY day ORDER BY day", since), 40)

    # 10. nginx's own error log, by kind of line, if this user may read it.
    kinds = {}
    for f in sorted(glob.glob("/var/log/nginx/error.log*"))[:3]:
        if f.endswith(".gz"):
            continue
        try:
            for line in open(f, errors="replace"):
                m = re.search(r"\[(\w+)\] \d+#\d+: (?:\*\d+ )?(.*)", line)
                if not m:
                    continue
                msg = re.sub(r"\b\d{1,3}(\.\d{1,3}){3}\b|\b[0-9a-f:]{6,}\b", "<addr>", m.group(2))
                msg = re.sub(r'"[^"]*"', '"…"', msg)
                msg = re.sub(r"\d+", "N", msg)[:110]
                kinds[(m.group(1), msg)] = kinds.get((m.group(1), msg), 0) + 1
        except OSError as e:
            print(f"\n  nginx error log {f}: not readable by this user ({e.strerror})")
    table("nginx error log, by kind", ["level", "lines", "what"], [(lv, n, msg) for (lv, msg), n in sorted(kinds.items(), key=lambda kv: -kv[1])], 25)


main()
