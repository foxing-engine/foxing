//! Where a command's time goes (S2).
//!
//! `took_ms` said how long the world took and nothing of what it spent it on. This
//! splits it: the wait for the world's lock, the command itself, and inside that the
//! log's share (appends, and any read that went past the kept tail to the log
//! underneath). It is said on each reply (`took`) and kept by verb since the driver
//! started, with what the sessions' pushes cost, for the operator (`GET /v0/ops`).

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

/// One verb's figures since the driver started.
#[derive(Default, Clone)]
struct Verb {
    n: u64,
    world_ms: f64,
    worst_ms: f64,
    log_ms: f64,
    appends: u64,
    past_reads: u64,
}

#[derive(Default)]
struct Stats {
    verbs: BTreeMap<String, Verb>,
    wait_ms: f64,
    worst_wait_ms: f64,
    pushes: u64,
    pushes_skipped: u64,
    push_wait_ms: f64,
    push_held_ms: f64,
    worst_push_ms: f64,
}

static STATS: Mutex<Option<Stats>> = Mutex::new(None);

fn with(f: impl FnOnce(&mut Stats)) {
    if let Ok(mut g) = STATS.lock() {
        f(g.get_or_insert_with(Stats::default));
    }
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn tenth(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// A command, from the moment it has the world's lock.
pub struct Mark {
    asked: Instant,
    locked: Instant,
    log0: foxing_log::TailCounters,
}

impl Mark {
    /// Call with the lock just taken; `asked` is when the request came in.
    pub fn locked(asked: Instant) -> Mark {
        Mark {
            asked,
            locked: Instant::now(),
            log0: foxing_log::tail_counters(),
        }
    }

    /// Call with the command run and the lock still held. Keeps the figures under
    /// the verb, and gives the split for the reply.
    pub fn done(self, verb: &str) -> Value {
        let wait = ms(self.locked - self.asked);
        let world = ms(self.locked.elapsed());
        let log1 = foxing_log::tail_counters();
        let appends = log1.appends - self.log0.appends;
        let past_reads = log1.past_reads - self.log0.past_reads;
        let log = (log1.append_nanos - self.log0.append_nanos + log1.past_read_nanos
            - self.log0.past_read_nanos) as f64
            / 1e6;
        // A verb's name only: a typed line's first word, never its arguments.
        let name: String = verb
            .split_whitespace()
            .next()
            .unwrap_or("?")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '.')
            .take(24)
            .collect::<String>()
            .to_ascii_lowercase();
        with(|s| {
            s.wait_ms += wait;
            s.worst_wait_ms = s.worst_wait_ms.max(wait);
            if s.verbs.len() < 200 || s.verbs.contains_key(&name) {
                let v = s.verbs.entry(name).or_default();
                v.n += 1;
                v.world_ms += world;
                v.worst_ms = v.worst_ms.max(world);
                v.log_ms += log;
                v.appends += appends;
                v.past_reads += past_reads;
            }
        });
        json!({
            "wait_ms": tenth(wait),
            "world_ms": tenth(world),
            "log_ms": tenth(log),
            "appends": appends,
            "past_reads": past_reads,
        })
    }
}

/// A session's push after a change: how long it waited for the lock, and held it.
pub fn pushed(wait: Duration, held: Duration) {
    with(|s| {
        s.pushes += 1;
        s.push_wait_ms += ms(wait);
        s.push_held_ms += ms(held);
        s.worst_push_ms = s.worst_push_ms.max(ms(held));
    });
}

/// S5: a change that concerned a session in nothing: no lock taken, nothing sent.
pub fn push_skipped() {
    with(|s| s.pushes_skipped += 1);
}

/// Everything kept, for the operator.
pub fn report() -> Value {
    let log = foxing_log::tail_counters();
    let mut out = json!({});
    with(|s| {
        let n: u64 = s.verbs.values().map(|v| v.n).sum();
        let mut verbs: Vec<(&String, &Verb)> = s.verbs.iter().collect();
        verbs.sort_by(|a, b| {
            b.1.world_ms
                .partial_cmp(&a.1.world_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out = json!({
            "commands": n,
            "lock_wait_ms": {"mean": tenth(if n > 0 { s.wait_ms / n as f64 } else { 0.0 }), "worst": tenth(s.worst_wait_ms)},
            "log": {
                "appends": log.appends,
                "append_mean_ms": tenth(if log.appends > 0 { log.append_nanos as f64 / 1e6 / log.appends as f64 } else { 0.0 }),
                "reads_past_the_tail": log.past_reads,
                "past_read_mean_ms": tenth(if log.past_reads > 0 { log.past_read_nanos as f64 / 1e6 / log.past_reads as f64 } else { 0.0 }),
            },
            "pushes": {
                "count": s.pushes,
                "skipped": s.pushes_skipped,
                "wait_mean_ms": tenth(if s.pushes > 0 { s.push_wait_ms / s.pushes as f64 } else { 0.0 }),
                "held_mean_ms": tenth(if s.pushes > 0 { s.push_held_ms / s.pushes as f64 } else { 0.0 }),
                "held_worst_ms": tenth(s.worst_push_ms),
                "held_total_ms": tenth(s.push_held_ms),
            },
            "verbs": verbs.iter().map(|(name, v)| json!({
                "verb": name,
                "count": v.n,
                "mean_ms": tenth(v.world_ms / v.n.max(1) as f64),
                "worst_ms": tenth(v.worst_ms),
                "log_share": tenth(if v.world_ms > 0.0 { 100.0 * v.log_ms / v.world_ms } else { 0.0 }),
                "appends_each": tenth(v.appends as f64 / v.n.max(1) as f64),
                "past_reads": v.past_reads,
                "total_ms": tenth(v.world_ms),
            })).collect::<Vec<_>>(),
        });
    });
    out
}
