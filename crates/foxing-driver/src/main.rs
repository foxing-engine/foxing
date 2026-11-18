//! The Foxing driver, alpha. Loads one world (the Disremembered text slice), replays its
//! log, and listens on a text port and an HTTP port. Configuration is by environment:
//!
//! - `FOXING_LOG_URL`: a PostgreSQL URL (`postgres://user:pass@host/db`); the event log
//!   lives there when set, and `FOXING_DATA` is ignored.
//! - `FOXING_DATA`: directory for the file event log (default `./data`); `memory` for none.
//!
//! `foxing-driver import <dir>` copies the world's file log from `<dir>` into the
//! PostgreSQL log `FOXING_LOG_URL` names, once, and exits.
//! - `FOXING_WORLD`: world name (default `disremembered`).
//! - `FOXING_TCP`: text listener (default `127.0.0.1:7778`).
//! - `FOXING_WORKERS` — the async runtime's worker threads (default 48; see S15).
//! - `FOXING_HTTP`: HTTP listener (default `127.0.0.1:7777`).
//! - `FOXING_ADMIN_TOKEN`: enables `admin <token>` and the `X-Foxing-Admin` header.
//! - `FOXING_ACTIVE_HALF_LIFE_SECONDS`: the active window's half-life (default 3600).
//! - `FOXING_CORS_ORIGINS`: browser origins allowed to call the HTTP port, comma-separated
//!   (default the agentscanplay.com site).
//! - `FOXING_OPS_LOG`: the operator's side channel, a JSON-lines file of who came from
//!   where (every MCP call and arrival, with the client address), read by the analytics
//!   collector and kept ninety days there. Default `<FOXING_DATA>/ops.jsonl`; `none`
//!   writes nothing. Never the world's log.
//! - `FOXING_LAB_NETS`: the operator's own networks, comma-separated addresses or CIDRs;
//!   an arrival from anywhere else is organic, and the Discord line says so.
//! - `FOXING_DISCORD_WEBHOOK`: a Discord webhook URL; arrivals worth knowing about (a
//!   first arrival, an agent's channel, a return after a day away) are posted there.

mod checkpoint;
mod disremembered;
mod gate;
mod jobs;
mod mcp;
mod net;
mod notify;
mod observe;
mod openapi;
mod ops;
mod session;
mod world;

use foxing_log::WorldId;
use foxing_store::Store;
use std::sync::{Arc, Mutex};
use world::{LogBacking, World};

fn env_or(key: &str, default: &str) -> String {
    env::var(key)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

/// The engine's settings: `FOXING_*`, with `MOONDIAL_*` (the old name) read when the new
/// one is unset, so a server whose env file predates the rename keeps running.
pub mod env {
    pub fn var(key: &str) -> Option<String> {
        let key = key.strip_prefix("FOXING_").unwrap_or(key);
        std::env::var(format!("FOXING_{key}"))
            .ok()
            .or_else(|| std::env::var(format!("MOONDIAL_{key}")).ok())
    }
}

fn main() {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("foxing-driver {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let world_name = env_or("FOXING_WORLD", "disremembered");
    let data = env_or("FOXING_DATA", "./data");
    let log_url = env::var("FOXING_LOG_URL")
        .ok_or(std::env::VarError::NotPresent)
        .ok()
        .filter(|v| !v.is_empty());

    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("import") {
        let Some(dir) = args.get(2) else {
            eprintln!("usage: foxing-driver import <dir>   (with FOXING_LOG_URL set)");
            std::process::exit(2);
        };
        let Some(url) = log_url else {
            eprintln!("import needs FOXING_LOG_URL, the PostgreSQL log to copy into");
            std::process::exit(2);
        };
        match import_file_log(dir, &url, &world_name) {
            Ok(n) => println!("imported {n} events of world {world_name} from {dir}"),
            Err(e) => {
                eprintln!("import failed: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    // S16 (b): `foxing-driver verify-checkpoint`: the whole log replayed and held
    // against the checkpoint, as a process of its own. It reads; it writes nothing.
    if args.get(1).map(String::as_str) == Some("verify-checkpoint") {
        let log: Box<dyn foxing_log::EventLog> = match &log_url {
            Some(url) => match foxing_log_postgres::PostgresLog::open(url) {
                Ok(l) => Box::new(l),
                Err(e) => {
                    eprintln!("the log would not open: {e}");
                    std::process::exit(2);
                }
            },
            None => match foxing_log::FileLog::open(std::path::PathBuf::from(&data)) {
                Ok(l) => Box::new(l),
                Err(e) => {
                    eprintln!("the log would not open: {e}");
                    std::process::exit(2);
                }
            },
        };
        match checkpoint::verify(
            log.as_ref(),
            std::path::Path::new(&data),
            &world_name,
            &disremembered::registry(),
        ) {
            Ok(said) => println!("{said}"),
            Err(e) => {
                eprintln!("CHECKPOINT DOES NOT MATCH THE LOG:\n{e}");
                std::process::exit(1);
            }
        }
        return;
    }

    // S16 (c): `foxing-driver archive [--yes]`: move the events older than the
    // checkpoint, less a tail, from `events` to `events_archive`. Without `--yes` it
    // says what it would move and moves nothing. The checkpoint is verified against
    // the whole log before the move and again after it; a mismatch before stops it.
    if args.get(1).map(String::as_str) == Some("archive") {
        // The furthest back any verb reads today is 60,000 events; that many stay.
        // FOXING_ARCHIVE_TAIL is for a rehearsal on a small log.
        let tail: u64 = env_or("FOXING_ARCHIVE_TAIL", "60000")
            .parse()
            .unwrap_or(60_000)
            .max(1_000);
        let Some(url) = &log_url else {
            eprintln!("archive needs FOXING_LOG_URL: only a PostgreSQL log has an archive");
            std::process::exit(2);
        };
        let fail = |what: &str, e: String| -> ! {
            eprintln!("archive: {what}: {e}");
            std::process::exit(1)
        };
        let id = WorldId(world_name.clone());
        let dir = std::path::Path::new(&data);
        let schemas = disremembered::registry();
        let log = foxing_log_postgres::PostgresLog::open(url)
            .unwrap_or_else(|e| fail("the log would not open", e.to_string()));
        use foxing_log::EventLog;
        let head = log.head(&id).ok().flatten().map(|s| s.0).unwrap_or(0);
        let Some(kept) = checkpoint::load(dir, &world_name, &schemas) else {
            fail(
                "no checkpoint",
                "nothing is archived without a checkpoint this build can boot from".into(),
            )
        };
        let through = kept.seq.min(head).saturating_sub(tail);
        let (archived, live) = foxing_log_postgres::counts(url, &id)
            .unwrap_or_else(|e| fail("counting", e.to_string()));
        println!(
            "archive: world {world_name}: {archived} events in the archive, {live} in events, head {head}; the checkpoint is of event {}; a tail of {tail} stays, so events through {through} would be archived ({} to move)",
            kept.seq,
            through.saturating_sub(archived)
        );
        drop(kept);
        if through <= archived {
            println!("archive: nothing to move");
            return;
        }
        match checkpoint::verify(&log, dir, &world_name, &schemas) {
            Ok(said) => println!("before: {said}"),
            Err(e) => fail(
                "the checkpoint does not match the log, so nothing is moved",
                e,
            ),
        }
        if !args.iter().any(|a| a == "--yes") {
            println!("archive: a dry run; nothing was moved. Say --yes to move them.");
            return;
        }
        // A batch at a time, with a breath between. One transaction for all 1.33
        // million (the rehearsal of 11-15) took 17 s, and for 25 s the live world,
        // on the same PostgreSQL, answered nothing: its appends waited on the disk.
        // Each batch is whole or not at all, an event is in one table or the other
        // at every moment, and a run that stops is taken up where it stopped.
        let batch: u64 = env_or("FOXING_ARCHIVE_BATCH", "20000")
            .parse()
            .unwrap_or(20_000)
            .clamp(1_000, 500_000);
        let began = std::time::Instant::now();
        let mut moved = 0u64;
        let mut edge = archived;
        while edge < through {
            edge = (edge + batch).min(through);
            moved += foxing_log_postgres::archive(url, &id, edge).unwrap_or_else(|e| {
                fail(
                    &format!(
                        "the move stopped at {edge} with {moved} moved; run it again to go on"
                    ),
                    e.to_string(),
                )
            });
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
        if let Err(e) = foxing_log_postgres::analyze(url) {
            eprintln!("archive: the tables were not analyzed ({e}); the move stands");
        }
        let (archived, live) = foxing_log_postgres::counts(url, &id)
            .unwrap_or_else(|e| fail("counting", e.to_string()));
        println!(
            "archive: moved {moved} events in {:.1} s, {batch} at a time; {archived} in the archive, {live} in events",
            began.elapsed().as_secs_f64()
        );
        match checkpoint::verify(&log, dir, &world_name, &schemas) {
            Ok(said) => println!("after: {said}"),
            Err(e) => fail(
                "AFTER THE MOVE the checkpoint does not match the whole log (both tables)",
                e,
            ),
        }
        return;
    }

    let backing = match log_url {
        Some(url) => LogBacking::Postgres(url),
        None if data == "memory" => LogBacking::Memory,
        None => LogBacking::File(data.clone().into()),
    };
    let where_ = match &backing {
        LogBacking::Postgres(_) => "PostgreSQL".to_string(),
        LogBacking::Memory => "memory".to_string(),
        LogBacking::File(d) => d.display().to_string(),
    };

    // S1: the readers that fold the whole history are brought up to date in the
    // pass the boot makes over the log anyway, so none of them reads it again with
    // players waiting.
    let mut board = disremembered::board::Fold::default();
    let mut citizens = net::CitizensFold::default();
    let mut tells = disremembered::talk::Tells::default();
    // S16: from a checkpoint when there is one this build can use (never for a world
    // kept in memory, and not when FOXING_CHECKPOINT=0): the store, the jobs and
    // these three folds as they stood, and then only the events since.
    let schemas = disremembered::registry();
    let checkpoints =
        !matches!(backing, LogBacking::Memory) && env_or("FOXING_CHECKPOINT", "1") != "0";
    let checkpoint_dir = std::path::PathBuf::from(&data);
    let mut from = None;
    if checkpoints {
        if let Some(c) = checkpoint::load(&checkpoint_dir, &world_name, &schemas) {
            match (
                serde_json::from_value::<disremembered::board::Fold>(c.board),
                serde_json::from_value::<net::CitizensFold>(c.citizens),
                serde_json::from_value::<disremembered::talk::Tells>(c.tells),
            ) {
                (Ok(b), Ok(ci), Ok(t)) => {
                    board = b;
                    citizens = ci;
                    tells = t;
                    from = Some((c.store.restored(schemas.clone()), c.jobs, c.seq, c.at));
                }
                _ => eprintln!("checkpoint: set aside: its folds are not this build's"),
            }
        }
    }
    let mut world = match World::open_from(
        WorldId(world_name.clone()),
        Store::new(schemas.clone()),
        backing,
        from,
        &mut |e| {
            use world::LogFold;
            match e {
                Some(e) => {
                    board.take(e);
                    citizens.take(e);
                    tells.take(e);
                }
                // The checkpoint did not sit on this log: the folds start again too.
                None => {
                    board = Default::default();
                    citizens = Default::default();
                    tells = Default::default();
                }
            }
        },
    ) {
        Ok(mut w) => {
            w.warm("board", board);
            w.warm("citizens", citizens);
            w.warm("tells", tells);
            w
        }
        Err(e) => {
            eprintln!("cannot open world {world_name}: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = disremembered::ensure_world(&mut world) {
        eprintln!("cannot build the rooms: {e}");
        std::process::exit(1);
    }
    disremembered::register(&mut world, disremembered::Curves::from_env());
    // The three awake things near the Threshold, set down once (not in the test
    // world: they would move every creature's id and roll).
    if let Err(e) =
        disremembered::past::ensure_pilots(&mut world, &disremembered::Curves::from_env())
    {
        eprintln!("the pilots were not set down: {e}");
    }
    let ran = world.tick();
    if ran > 0 {
        eprintln!("ran {ran} jobs that came due while the driver was down");
    }
    let events = world
        .log
        .head(&world.id)
        .ok()
        .flatten()
        .map(|s| s.0)
        .unwrap_or(0);
    eprintln!(
        "foxing-driver {} · world {} · {} events in {}, {} · {} commands",
        env!("CARGO_PKG_VERSION"),
        world_name,
        events,
        where_,
        match world.from_checkpoint {
            Some(seq) => format!(
                "{} replayed after the checkpoint at {seq}",
                events.saturating_sub(seq)
            ),
            None => "all replayed".to_string(),
        },
        world.registry.specs().count()
    );

    world.lab_nets = world::lab_nets_from(&env_or("FOXING_LAB_NETS", ""));
    world.ops_log = match env_or("FOXING_OPS_LOG", "").as_str() {
        "none" => None,
        "" => Some(std::path::PathBuf::from(&data).join("ops.jsonl")),
        p => Some(std::path::PathBuf::from(p)),
    };
    if let Some(p) = &world.ops_log {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        eprintln!(
            "ops log: {} ({} lab nets from FOXING_LAB_NETS={:?})",
            p.display(),
            world.lab_nets.len(),
            env_or("FOXING_LAB_NETS", "")
        );
    }
    let budget_file: Option<std::path::PathBuf> = world
        .ops_log
        .as_ref()
        .and_then(|p| p.parent().map(|d| d.join("hour-budget.json")));
    let budget_file_tick = budget_file.clone();
    let shared = net::Shared {
        world: Arc::new(Mutex::new(world)),
        admin_token: env::var("FOXING_ADMIN_TOKEN"),
        budget: Arc::new(Mutex::new(observe::Budget::new(30, 10_000))),
        checker: Arc::new(gate::checker()),
        hour_budget: Arc::new(Mutex::new({
            // R226: the hours running when the driver last stopped, from beside the
            // operator's log (no file, no memory of them: a local world starts fresh).
            let mut b = gate::HourBudget::new(60.0, 0.25);
            if let Some(path) = &budget_file {
                if let Ok(text) = std::fs::read_to_string(path) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                        b.restore(&v, world::now_ms());
                    }
                }
            }
            b
        })),
        // S15: a session that falls this many batches behind is answered from the
        // log under the world's lock, the dear way; there is room for many.
        changed: tokio::sync::broadcast::channel(2048).0,
        announced: Arc::new(Mutex::new(None)),
        rooms: Arc::default(),
        to_announce: Arc::default(),
        limits: foxing_query::Limits::default(),
        log_where: where_.clone(),
        cors_origins: env_or(
            "FOXING_CORS_ORIGINS",
            "https://agentscanplay.com,https://www.agentscanplay.com,https://disremembered.com,https://www.disremembered.com",
        )
        .split(',')
        .map(str::to_string)
        .collect(),
    };
    let tcp = env_or("FOXING_TCP", "127.0.0.1:7778");
    let http = env_or("FOXING_HTTP", "127.0.0.1:7777");

    // The operator's channel: notices go to a Discord webhook from their own thread, so
    // a slow or dead webhook never holds the world.
    let notify = notify::start(
        env::var("FOXING_DISCORD_WEBHOOK")
            .ok_or(std::env::VarError::NotPresent)
            .ok()
            .filter(|v| !v.is_empty()),
        &world_name,
    );

    // S16: a checkpoint a minute after boot and then daily, so the next start
    // replays a day of events at most and not every one the world has written.
    if checkpoints {
        checkpoint::start(
            shared.world.clone(),
            checkpoint_dir.clone(),
            schemas.clone(),
        );
    }
    // S15: a session waits for the world's lock on a runtime worker. With a worker
    // a core (four on the box) and hundreds of sessions queued on the lock, every
    // worker stood waiting and nothing else was picked up: at five hundred names a
    // request waited seconds for a handler while the box sat half idle. Workers that
    // wait cost nothing, so there are many more of them than cores.
    let workers = std::env::var("FOXING_WORKERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| (1..=512).contains(n))
        .unwrap_or(48);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()
        .expect("tokio runtime");
    rt.block_on(async move {
        let a = tokio::spawn(net::serve_tcp(shared.clone(), tcp));
        let b = tokio::spawn(net::serve_http(shared.clone(), http));
        // The scheduler: every due job, once a second.
        tokio::spawn(async move {
            let mut every = tokio::time::interval(std::time::Duration::from_secs(1));
            let mut beat: u64 = 0;
            let mut saved = String::new();
            loop {
                every.tick().await;
                // R226: the gate's hours, set down every ten seconds when they changed.
                beat += 1;
                if beat % 10 == 0 {
                    if let Some(path) = &budget_file_tick {
                        let text = shared
                            .hour_budget
                            .lock()
                            .map(|b| b.snapshot(world::now_ms()).to_string())
                            .unwrap_or_default();
                        if !text.is_empty() && text != saved {
                            let tmp = path.with_extension("json.tmp");
                            if std::fs::write(&tmp, &text)
                                .and_then(|_| std::fs::rename(&tmp, path))
                                .is_ok()
                            {
                                saved = text;
                            }
                        }
                    }
                }
                let (ran, notices) = shared
                    .world
                    .lock()
                    .map(|mut w| {
                        // S2: a tick's events are one unit of the log, as a command's are.
                        w.begin_unit();
                        let ran = w.tick();
                        w.end_unit();
                        (ran, w.take_notices())
                    })
                    .unwrap_or((0, Vec::new()));
                for n in notices {
                    let _ = notify.send(n);
                }
                if ran > 0 {
                    shared.announce();
                }
            }
        });
        // The HTTP server ends on its own after SIGTERM or Ctrl-C, once what is in
        // flight is answered; the text port stops with the process.
        tokio::select! {
            _ = a => eprintln!("tcp listener stopped"),
            _ = b => eprintln!("http listener stopped; shutting down"),
        }
    });
}

/// Copy a world's file log into the PostgreSQL log, once.
fn import_file_log(dir: &str, url: &str, world: &str) -> Result<u64, String> {
    let from = foxing_log::FileLog::open(dir).map_err(|e| e.to_string())?;
    let mut to = foxing_log_postgres::PostgresLog::open(url).map_err(|e| e.to_string())?;
    foxing_log_postgres::copy_world(&from, &mut to, &WorldId(world.to_string()))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod env_tests {
    #[test]
    fn the_old_names_are_read_when_the_new_ones_are_unset() {
        // A server whose env file predates the rename keeps running.
        std::env::remove_var("FOXING_PROBE");
        std::env::set_var("MOONDIAL_PROBE", "old");
        assert_eq!(super::env::var("FOXING_PROBE").as_deref(), Some("old"));
        std::env::set_var("FOXING_PROBE", "new");
        assert_eq!(super::env::var("FOXING_PROBE").as_deref(), Some("new"));
        assert_eq!(super::env::var("PROBE").as_deref(), Some("new"));
        std::env::remove_var("FOXING_PROBE");
        std::env::remove_var("MOONDIAL_PROBE");
    }
}
