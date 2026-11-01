//! The Foxing driver, pre-alpha. Loads one world (the Disremembered text slice), replays its
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

mod disremembered;
mod gate;
mod jobs;
mod mcp;
mod net;
mod notify;
mod observe;
mod openapi;
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

    let mut world = match World::open(
        WorldId(world_name.clone()),
        Store::new(disremembered::registry()),
        backing,
    ) {
        Ok(w) => w,
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
        "foxing-driver {} · world {} · {} events replayed from {} · {} commands",
        env!("CARGO_PKG_VERSION"),
        world_name,
        events,
        where_,
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
    let shared = net::Shared {
        world: Arc::new(Mutex::new(world)),
        admin_token: env::var("FOXING_ADMIN_TOKEN"),
        budget: Arc::new(Mutex::new(observe::Budget::new(30, 10_000))),
        checker: Arc::new(gate::checker()),
        hour_budget: Arc::new(Mutex::new(gate::HourBudget::new(60.0, 0.25))),
        changed: tokio::sync::broadcast::channel(64).0,
        limits: foxing_query::Limits::default(),
        log_where: where_.clone(),
        cors_origins: env_or(
            "FOXING_CORS_ORIGINS",
            "https://agentscanplay.com,https://www.agentscanplay.com",
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

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    rt.block_on(async move {
        let a = tokio::spawn(net::serve_tcp(shared.clone(), tcp));
        let b = tokio::spawn(net::serve_http(shared.clone(), http));
        // The scheduler: every due job, once a second.
        tokio::spawn(async move {
            let mut every = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                every.tick().await;
                let (ran, notices) = shared
                    .world
                    .lock()
                    .map(|mut w| (w.tick(), w.take_notices()))
                    .unwrap_or((0, Vec::new()));
                for n in notices {
                    let _ = notify.send(n);
                }
                if ran > 0 {
                    let _ = shared.changed.send(());
                }
            }
        });
        tokio::select! {
            _ = tokio::signal::ctrl_c() => eprintln!("shutting down"),
            _ = a => eprintln!("tcp listener stopped"),
            _ = b => eprintln!("http listener stopped"),
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
