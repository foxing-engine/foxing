//! S16, step (a): a checkpoint of everything boot builds from the log, so that a
//! start replays what came after it and not every event the world ever wrote.
//!
//! Boot replayed the whole log: 1.39 million events on 11-15, a minute in which
//! nobody was answered at every deploy, growing for ever. A checkpoint is the store,
//! the pending jobs and the three folds kept from the whole log (the board, the
//! citizens' report, the words left for names), as they stood after one event.
//!
//! **A checkpoint is never the truth; the log is.** One that is missing, written by
//! another build's shapes, for another world, or that does not sit on this log is set
//! aside with a line in the journal, and the world boots from the whole log as it
//! always has. So a build that changes a shape needs no migration: it ignores the
//! old checkpoint and writes its own.
//!
//! It is a file in the data directory (`checkpoint-<world>.json`, and the one before
//! it as `.prev`), not a table: the same for every kind of log, and already inside
//! what the nightly backup copies.

use crate::jobs::Job;
use foxing_store::Store;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The shape of this file **and the meaning of what is in it**. Raise it whenever
/// (1) what a checkpoint must hold changes (a new fold kept from the whole log, a new
/// thing boot learns from it), or (2) the same events would now fold to something
/// else: a fold's `take` counts differently, or the store applies an event
/// differently. An older file is then set aside, never half-read, and the world
/// boots from the whole log once and writes a new one.
///
/// It is not tied to the build: every deploy is a boot, and a checkpoint no deploy
/// could use would save nothing. So this number is the only thing that says "the old
/// folds are no longer right"; the full replay compared with the checkpoint (S16 b)
/// is what catches a change that forgot to raise it.
pub const FORMAT: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct Checkpoint {
    pub format: u32,
    /// The build that wrote it, for the journal; see [`FORMAT`] for what decides.
    pub driver: String,
    pub world: String,
    /// The last event folded into everything below, and when that event was written:
    /// the pair is checked against the log, so a checkpoint from another log (a
    /// restore, the load world's) is not laid over this one.
    pub seq: u64,
    pub at: i64,
    /// The schemas boot started from when this was written: another set, and the
    /// file is set aside.
    pub schemas: foxing_store::Registry,
    pub store: Store,
    pub jobs: Vec<Job>,
    pub board: Value,
    pub citizens: Value,
    pub tells: Value,
}

/// What names this build: its version and, where the deploy sets it, its commit.
pub fn driver() -> String {
    format!(
        "{}+{}",
        env!("CARGO_PKG_VERSION"),
        option_env!("FOXING_BUILD").unwrap_or("dev")
    )
}

pub fn path(dir: &Path, world: &str) -> PathBuf {
    dir.join(format!("checkpoint-{world}.json"))
}

fn prev(dir: &Path, world: &str) -> PathBuf {
    dir.join(format!("checkpoint-{world}.json.prev"))
}

/// The newest checkpoint this build can use for `world` under `registry`, or `None`
/// with the reason in the journal. The one before is tried when the newest will not do.
pub fn load(dir: &Path, world: &str, registry: &foxing_store::Registry) -> Option<Checkpoint> {
    for file in [path(dir, world), prev(dir, world)] {
        if !file.exists() {
            continue;
        }
        match read(&file, world, registry) {
            Ok(c) => {
                eprintln!(
                    "checkpoint: {} holds the world as of event {}",
                    file.display(),
                    c.seq
                );
                return Some(c);
            }
            Err(why) => eprintln!("checkpoint: {} set aside: {why}", file.display()),
        }
    }
    None
}

fn read(file: &Path, world: &str, registry: &foxing_store::Registry) -> Result<Checkpoint, String> {
    let f = std::fs::File::open(file).map_err(|e| e.to_string())?;
    let c: Checkpoint = serde_json::from_reader(std::io::BufReader::new(f))
        .map_err(|e| format!("it does not parse: {e}"))?;
    if c.format != FORMAT {
        return Err(format!("format {} and this build reads {FORMAT}", c.format));
    }
    if c.world != world {
        return Err(format!("it is of world {}", c.world));
    }
    if &c.schemas != registry {
        return Err("its schemas are not this build's".into());
    }
    Ok(c)
}

/// Write `c` beside the one before it: to a temporary file, then the newest becomes
/// `.prev` and the new one takes its place, so a crash mid-write leaves a whole file.
pub fn write(dir: &Path, c: &Checkpoint) -> Result<u64, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let tmp = dir.join(format!("checkpoint-{}.json.tmp", c.world));
    {
        let f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
        let mut w = std::io::BufWriter::new(f);
        serde_json::to_writer(&mut w, c).map_err(|e| e.to_string())?;
        use std::io::Write;
        w.flush().map_err(|e| e.to_string())?;
        w.get_ref().sync_all().map_err(|e| e.to_string())?;
    }
    let bytes = std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0);
    let now = path(dir, &c.world);
    if now.exists() {
        let _ = std::fs::rename(&now, prev(dir, &c.world));
    }
    std::fs::rename(&tmp, &now).map_err(|e| e.to_string())?;
    Ok(bytes)
}

/// Take a checkpoint of `world` as it stands: under the lock the caller holds, the
/// state is copied (and the folds brought to the head and copied); writing it out is
/// the caller's, outside the lock. `None` for a world with nothing in its log.
pub fn take(world: &crate::world::World, schemas: &foxing_store::Registry) -> Option<Checkpoint> {
    let head = world.log.head(&world.id).ok().flatten()?.0;
    let last = world
        .log
        .read(&world.id, foxing_log::Cursor { after: head - 1 }, 1)
        .ok()?
        .into_iter()
        .next()?;
    let board = world.folded("board", |f: &crate::disremembered::board::Fold| {
        serde_json::to_value(f)
    });
    let citizens = world.folded("citizens", |f: &crate::net::CitizensFold| {
        serde_json::to_value(f)
    });
    let tells = world.folded("tells", |f: &crate::disremembered::talk::Tells| {
        serde_json::to_value(f)
    });
    Some(Checkpoint {
        format: FORMAT,
        driver: driver(),
        world: world.id.0.clone(),
        seq: last.seq.0,
        at: last.at.0,
        schemas: schemas.clone(),
        store: world.store.clone(),
        jobs: world.jobs.pending().cloned().collect(),
        board: board.ok()?,
        citizens: citizens.ok()?,
        tells: tells.ok()?,
    })
}

/// The writer: a minute after boot, and then once a day, a checkpoint is taken and
/// written. The world's lock is held for the copy and no longer.
pub fn start(
    world: std::sync::Arc<std::sync::Mutex<crate::world::World>>,
    dir: PathBuf,
    schemas: foxing_store::Registry,
) {
    let _ = std::thread::Builder::new().name("checkpoint".into()).spawn(move || {
        let mut wait = std::time::Duration::from_secs(60);
        loop {
            std::thread::sleep(wait);
            wait = std::time::Duration::from_secs(24 * 3600);
            let began = std::time::Instant::now();
            let taken = match world.lock() {
                Ok(w) => take(&w, &schemas),
                Err(_) => None,
            };
            let held = began.elapsed();
            let Some(c) = taken else {
                eprintln!("checkpoint: nothing taken");
                continue;
            };
            match write(&dir, &c) {
                Ok(bytes) => eprintln!(
                    "checkpoint: the world as of event {} written, {:.1} MB; the lock was held {} ms, the writing took {} ms",
                    c.seq,
                    bytes as f64 / 1e6,
                    held.as_millis(),
                    (began.elapsed() - held).as_millis()
                ),
                Err(e) => eprintln!("checkpoint: not written: {e}"),
            }
        }
    });
}
