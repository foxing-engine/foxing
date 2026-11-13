//! The Foxing event log.
//!
//! Every change to a world is an appended [`Event`]; current state is a projection over
//! the log. That gives replay, audit, rollback after a bad module swap, and an
//! observation feed for agents, all from one structure. Only the driver appends; modules
//! and clients read.
//!
//! The log is per world. Within a world, events have a dense sequence ([`Seq`]) assigned
//! at append, and the timestamp the driver stamped them with. Sequence is the order of
//! truth; timestamps may repeat within a millisecond.
//!
//! [`EventLog`] is the trait the driver codes against. [`MemoryLog`] is the in-process
//! implementation used by tests, tools and single-process worlds; the Postgres
//! implementation lives in the driver and satisfies the same trait and the same tests.
//!
//! A [`Projection`] is anything that folds events into state. [`replay`] drives a
//! projection from a cursor, which is how a store is rebuilt, how a bad swap is undone,
//! and how the property tests check lazy evaluation against the record.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use foxing_time::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// A world's name: the tenant the event belongs to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorldId(pub String);

impl From<&str> for WorldId {
    fn from(s: &str) -> Self {
        WorldId(s.to_string())
    }
}

impl fmt::Display for WorldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for Seq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Position of an event in its world's log. Dense, starting at 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Seq(pub u64);

/// Who or what caused an event: a citizen, an agent, a module, a job, or the driver.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum Actor {
    /// A human or agent citizen, by their actor id.
    Citizen(String),
    /// A familiar acting for a citizen: (familiar id, bound citizen id).
    Familiar(String, String),
    /// A module, by name.
    Module(String),
    /// A scheduled job, by name.
    Job(String),
    /// The driver itself (system events, swaps, clocks).
    Driver,
}

/// An event as the driver records it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Position in the world's log.
    pub seq: Seq,
    /// The world the event belongs to.
    pub world: WorldId,
    /// When the driver stamped it.
    pub at: Timestamp,
    /// Event type, namespaced by the module that defines it: `store.entity_created`,
    /// `clock.paused`, `disremembered.chunk_earned`.
    pub kind: String,
    /// Who caused it.
    pub actor: Actor,
    /// The event this one was caused by, if any (a command's resolution points at the
    /// command; a concussion points at the death).
    pub cause: Option<Seq>,
    /// The event's data, as the defining module declares it.
    pub payload: serde_json::Value,
}

/// An event before it has a sequence: what a caller hands to [`EventLog::append`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewEvent {
    /// When the driver stamped it.
    pub at: Timestamp,
    /// Event type (see [`Event::kind`]).
    pub kind: String,
    /// Who caused it.
    pub actor: Actor,
    /// The causing event, if any.
    pub cause: Option<Seq>,
    /// The event's data.
    pub payload: serde_json::Value,
}

impl NewEvent {
    /// A convenience constructor for the common case.
    pub fn new(
        at: Timestamp,
        kind: impl Into<String>,
        actor: Actor,
        payload: serde_json::Value,
    ) -> Self {
        NewEvent {
            at,
            kind: kind.into(),
            actor,
            cause: None,
            payload,
        }
    }

    /// Attach the causing event.
    pub fn caused_by(mut self, cause: Seq) -> Self {
        self.cause = Some(cause);
        self
    }
}

/// Errors from log operations.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LogError {
    /// An append carried a timestamp earlier than the world's last event. Clocks only move forward.
    #[error("event at {at} is earlier than the world's last event at {last}")]
    OutOfOrder {
        /// The offending timestamp.
        at: Timestamp,
        /// The world's last timestamp.
        last: Timestamp,
    },
    /// The backing store failed.
    #[error("log storage error: {0}")]
    Storage(String),
}

/// A position to read from: events with `seq > after`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Cursor {
    /// The last sequence already seen; `0` means from the beginning.
    pub after: u64,
}

impl Cursor {
    /// From the beginning of the log.
    pub const START: Cursor = Cursor { after: 0 };
    /// A cursor positioned just after `seq`.
    pub fn after(seq: Seq) -> Cursor {
        Cursor { after: seq.0 }
    }
}

/// The append-only, per-world event log.
pub trait EventLog {
    /// Append an event to a world's log and return it with its sequence assigned.
    fn append(&mut self, world: &WorldId, event: NewEvent) -> Result<Event, LogError>;

    /// Read up to `limit` events after the cursor, in sequence order.
    fn read(&self, world: &WorldId, cursor: Cursor, limit: usize) -> Result<Vec<Event>, LogError>;

    /// The last sequence in a world's log, or `None` if it is empty.
    fn head(&self, world: &WorldId) -> Result<Option<Seq>, LogError>;

    /// Begin holding appends as one unit, where the log can (S2): until [`commit`]
    /// they are written but not yet made durable one by one. A log that cannot does
    /// nothing, and every append is durable as it returns, as before.
    ///
    /// [`commit`]: EventLog::commit
    fn begin(&mut self) -> Result<(), LogError> {
        Ok(())
    }

    /// Make durable everything appended since [`begin`]. If this fails, none of it
    /// was kept, and whoever applied those events as they were appended is now ahead
    /// of the log and must stop.
    ///
    /// [`begin`]: EventLog::begin
    fn commit(&mut self) -> Result<(), LogError> {
        Ok(())
    }

    /// The newest `n` events, oldest first, shared and not copied where the log can
    /// (see [`Tail`]): for a reader that looks back over the last few thousand
    /// events many times a command.
    fn newest(&self, world: &WorldId, n: usize) -> Result<Vec<std::sync::Arc<Event>>, LogError> {
        let head = self.head(world)?.map(|s| s.0).unwrap_or(0);
        let after = head.saturating_sub(n as u64);
        Ok(self
            .read_all(world, Cursor { after })?
            .into_iter()
            .map(std::sync::Arc::new)
            .collect())
    }

    /// Read every event after the cursor.
    fn read_all(&self, world: &WorldId, cursor: Cursor) -> Result<Vec<Event>, LogError> {
        let mut out = Vec::new();
        let mut cur = cursor;
        loop {
            let batch = self.read(world, cur, 1024)?;
            if batch.is_empty() {
                return Ok(out);
            }
            cur = Cursor::after(batch.last().expect("non-empty").seq);
            out.extend(batch);
        }
    }
}

/// What every [`Tail`] in this process has spent on the log underneath it: appends,
/// and reads that started before the kept part (S2: a command's share of its time
/// that was the log's).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TailCounters {
    /// Events appended.
    pub appends: u64,
    /// Time in the log underneath, appending.
    pub append_nanos: u64,
    /// Reads and head queries answered by the log underneath, not the kept part.
    pub past_reads: u64,
    /// Time in the log underneath, reading.
    pub past_read_nanos: u64,
}

static APPENDS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static APPEND_NANOS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PAST_READS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PAST_READ_NANOS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The counters now.
pub fn tail_counters() -> TailCounters {
    use std::sync::atomic::Ordering::Relaxed;
    TailCounters {
        appends: APPENDS.load(Relaxed),
        append_nanos: APPEND_NANOS.load(Relaxed),
        past_reads: PAST_READS.load(Relaxed),
        past_read_nanos: PAST_READ_NANOS.load(Relaxed),
    }
}

fn timed_past<T>(f: impl FnOnce() -> T) -> T {
    use std::sync::atomic::Ordering::Relaxed;
    let t = std::time::Instant::now();
    let out = f();
    PAST_READS.fetch_add(1, Relaxed);
    PAST_READ_NANOS.fetch_add(t.elapsed().as_nanos() as u64, Relaxed);
    out
}

/// How many of a world's newest events [`Tail`] keeps in memory.
pub const TAIL: usize = 12_000;

/// A log with its newest events kept in memory.
///
/// A driver asks its log "the last few thousand events" many times in one command
/// (who told whom a story this hour, what was said in this room), and against a
/// database each ask was a head query and several reads over the network: the floor
/// under every command. `Tail` wraps any log, keeps the newest [`TAIL`] events of
/// each world as they are appended, and answers from memory any read that starts
/// inside what it holds; an older read goes to the log underneath and comes back
/// into the kept part by itself as its cursor advances. Appends go to the log
/// underneath first, so the kept part is never ahead of what is durable.
///
/// It assumes it is the only writer to the worlds it has read: a second process
/// appending to the same world would not be seen.
pub struct Tail<L> {
    inner: L,
    kept: std::sync::Mutex<
        std::collections::HashMap<WorldId, std::collections::VecDeque<std::sync::Arc<Event>>>,
    >,
}

impl<L: EventLog> Tail<L> {
    /// Wrap a log.
    pub fn new(inner: L) -> Self {
        Tail {
            inner,
            kept: Default::default(),
        }
    }

    /// The kept events of a world, loaded from the log underneath the first time.
    fn with<T>(
        &self,
        world: &WorldId,
        f: impl FnOnce(&mut std::collections::VecDeque<std::sync::Arc<Event>>) -> T,
    ) -> Result<T, LogError> {
        let mut kept = self.kept.lock().unwrap_or_else(|e| e.into_inner());
        if !kept.contains_key(world) {
            let head = self.inner.head(world)?.map(|s| s.0).unwrap_or(0);
            let after = head.saturating_sub(TAIL as u64);
            let events = self.inner.read_all(world, Cursor { after })?;
            kept.insert(
                world.clone(),
                events.into_iter().map(std::sync::Arc::new).collect(),
            );
        }
        Ok(f(kept.get_mut(world).expect("just inserted")))
    }
}

impl<L: EventLog> EventLog for Tail<L> {
    fn append(&mut self, world: &WorldId, event: NewEvent) -> Result<Event, LogError> {
        // Load before appending, so the kept part and the new event are in order.
        self.with(world, |_| ())?;
        let t = std::time::Instant::now();
        let appended = self.inner.append(world, event);
        APPENDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        APPEND_NANOS.fetch_add(
            t.elapsed().as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        let e = appended?;
        let kept = self.kept.get_mut().unwrap_or_else(|e| e.into_inner());
        if let Some(q) = kept.get_mut(world) {
            // Only the very next in sequence: anything else (another writer got in
            // between), and the kept part is dropped and read again on the next ask,
            // so it can never hold a gap.
            if q.back()
                .map_or(e.seq.0 == 1, |last| last.seq.0 + 1 == e.seq.0)
            {
                q.push_back(std::sync::Arc::new(e.clone()));
                while q.len() > TAIL {
                    q.pop_front();
                }
            } else {
                kept.remove(world);
            }
        }
        Ok(e)
    }

    fn begin(&mut self) -> Result<(), LogError> {
        self.inner.begin()
    }

    fn commit(&mut self) -> Result<(), LogError> {
        // The commit is where the unit's durability is paid for: it is the log's time.
        let t = std::time::Instant::now();
        let done = self.inner.commit();
        APPEND_NANOS.fetch_add(
            t.elapsed().as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        if done.is_err() {
            // What was kept of the unit is not in the log: read again when next asked.
            self.kept
                .get_mut()
                .unwrap_or_else(|e| e.into_inner())
                .clear();
        }
        done
    }

    fn read(&self, world: &WorldId, cursor: Cursor, limit: usize) -> Result<Vec<Event>, LogError> {
        let hit = self.with(world, |q| {
            let first = q.front().map(|e| e.seq.0)?;
            // The kept part answers only what it wholly holds: a cursor at or after
            // the event before its first.
            if cursor.after + 1 < first {
                return None;
            }
            let from = q.partition_point(|e| e.seq.0 <= cursor.after);
            Some(
                q.iter()
                    .skip(from)
                    .take(limit)
                    .map(|e| (**e).clone())
                    .collect::<Vec<_>>(),
            )
        })?;
        match hit {
            Some(v) => Ok(v),
            None => timed_past(|| self.inner.read(world, cursor, limit)),
        }
    }

    fn newest(&self, world: &WorldId, n: usize) -> Result<Vec<std::sync::Arc<Event>>, LogError> {
        // Shared, not copied, when the kept part holds that many (or the whole log).
        let hit = self.with(world, |q| {
            let whole = q.front().map_or(true, |e| e.seq.0 <= 1);
            (n <= q.len() || whole).then(|| {
                q.iter()
                    .skip(q.len().saturating_sub(n))
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })?;
        match hit {
            Some(v) => Ok(v),
            None => {
                let head = self.inner.head(world)?.map(|s| s.0).unwrap_or(0);
                Ok(self
                    .inner
                    .read_all(
                        world,
                        Cursor {
                            after: head.saturating_sub(n as u64),
                        },
                    )?
                    .into_iter()
                    .map(std::sync::Arc::new)
                    .collect())
            }
        }
    }

    fn head(&self, world: &WorldId) -> Result<Option<Seq>, LogError> {
        match self.with(world, |q| q.back().map(|e| e.seq))? {
            Some(s) => Ok(Some(s)),
            None => self.inner.head(world),
        }
    }
}

/// Something that folds events into state.
pub trait Projection {
    /// The error a projection can raise while applying an event it understands.
    type Error: fmt::Debug;
    /// Apply one event. Events of kinds the projection does not know are ignored by
    /// convention, so several projections can share a log.
    fn apply(&mut self, event: &Event) -> Result<(), Self::Error>;
}

/// Drive a projection from a cursor to the head of a world's log. Returns the cursor
/// positioned after the last event applied.
pub fn replay<P: Projection>(
    log: &dyn EventLog,
    world: &WorldId,
    cursor: Cursor,
    projection: &mut P,
) -> Result<Cursor, ReplayError<P::Error>> {
    let events = log.read_all(world, cursor).map_err(ReplayError::Log)?;
    let mut last = cursor;
    for ev in &events {
        projection.apply(ev).map_err(|e| ReplayError::Projection {
            at: ev.seq,
            error: e,
        })?;
        last = Cursor::after(ev.seq);
    }
    Ok(last)
}

/// Why a replay stopped.
#[derive(Debug, thiserror::Error)]
pub enum ReplayError<E: fmt::Debug> {
    /// The log could not be read.
    #[error("log: {0}")]
    Log(LogError),
    /// The projection rejected an event.
    #[error("projection failed at seq {at}: {error:?}")]
    Projection {
        /// The event the projection rejected.
        at: Seq,
        /// Its error.
        error: E,
    },
}

/// An in-process event log. Deterministic and cheap; used by tests, tools and
/// single-process worlds.
#[derive(Debug, Default, Clone)]
pub struct MemoryLog {
    worlds: BTreeMap<WorldId, Vec<Event>>,
}

impl MemoryLog {
    /// An empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// The worlds that have at least one event.
    pub fn worlds(&self) -> impl Iterator<Item = &WorldId> {
        self.worlds.keys()
    }
}

impl EventLog for MemoryLog {
    fn append(&mut self, world: &WorldId, event: NewEvent) -> Result<Event, LogError> {
        let entries = self.worlds.entry(world.clone()).or_default();
        if let Some(last) = entries.last() {
            if event.at < last.at {
                return Err(LogError::OutOfOrder {
                    at: event.at,
                    last: last.at,
                });
            }
        }
        let ev = Event {
            seq: Seq(entries.len() as u64 + 1),
            world: world.clone(),
            at: event.at,
            kind: event.kind,
            actor: event.actor,
            cause: event.cause,
            payload: event.payload,
        };
        entries.push(ev.clone());
        Ok(ev)
    }

    fn read(&self, world: &WorldId, cursor: Cursor, limit: usize) -> Result<Vec<Event>, LogError> {
        let Some(entries) = self.worlds.get(world) else {
            return Ok(Vec::new());
        };
        let start = (cursor.after as usize).min(entries.len());
        Ok(entries[start..].iter().take(limit).cloned().collect())
    }

    fn head(&self, world: &WorldId) -> Result<Option<Seq>, LogError> {
        Ok(self
            .worlds
            .get(world)
            .and_then(|e| e.last().map(|ev| ev.seq)))
    }
}

/// An append-only file of JSON lines, one event per line, per world directory. Durable
/// enough for a single-process world until the Postgres log lands: every append is
/// written and flushed before it is acknowledged, and the whole file is replayed on open.
///
/// Layout: `<dir>/<world>.events.jsonl`. The in-memory index mirrors [`MemoryLog`].
#[derive(Debug)]
pub struct FileLog {
    dir: std::path::PathBuf,
    memory: MemoryLog,
    files: BTreeMap<WorldId, std::fs::File>,
}

impl FileLog {
    /// Open (or create) a log directory and load every world file in it.
    pub fn open(dir: impl Into<std::path::PathBuf>) -> Result<Self, LogError> {
        use std::io::BufRead;
        let dir = dir.into();
        std::fs::create_dir_all(&dir).map_err(|e| LogError::Storage(e.to_string()))?;
        let mut memory = MemoryLog::new();
        for entry in std::fs::read_dir(&dir).map_err(|e| LogError::Storage(e.to_string()))? {
            let entry = entry.map_err(|e| LogError::Storage(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let Some(world) = name.strip_suffix(".events.jsonl") else {
                continue;
            };
            let world = WorldId(world.to_string());
            let file =
                std::fs::File::open(entry.path()).map_err(|e| LogError::Storage(e.to_string()))?;
            for (i, line) in std::io::BufReader::new(file).lines().enumerate() {
                let line = line.map_err(|e| LogError::Storage(e.to_string()))?;
                if line.trim().is_empty() {
                    continue;
                }
                let ev: Event = serde_json::from_str(&line)
                    .map_err(|e| LogError::Storage(format!("{name} line {}: {e}", i + 1)))?;
                let entries = memory.worlds.entry(world.clone()).or_default();
                if ev.seq.0 != entries.len() as u64 + 1 {
                    return Err(LogError::Storage(format!(
                        "{name} line {}: expected seq {}, found {}",
                        i + 1,
                        entries.len() + 1,
                        ev.seq.0
                    )));
                }
                entries.push(ev);
            }
        }
        Ok(FileLog {
            dir,
            memory,
            files: BTreeMap::new(),
        })
    }

    /// The directory the log lives in.
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    fn file_for(&mut self, world: &WorldId) -> Result<&mut std::fs::File, LogError> {
        if !self.files.contains_key(world) {
            let path = self.dir.join(format!("{}.events.jsonl", world.0));
            let f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|e| LogError::Storage(e.to_string()))?;
            self.files.insert(world.clone(), f);
        }
        Ok(self.files.get_mut(world).expect("just inserted"))
    }
}

impl EventLog for FileLog {
    fn append(&mut self, world: &WorldId, event: NewEvent) -> Result<Event, LogError> {
        use std::io::Write;
        // Validate and assign the sequence in memory first, but only keep it if the write succeeds.
        let ev = self.memory.append(world, event)?;
        let line = serde_json::to_string(&ev).map_err(|e| LogError::Storage(e.to_string()))?;
        let res = (|| {
            let f = self.file_for(world)?;
            f.write_all(line.as_bytes())
                .map_err(|e| LogError::Storage(e.to_string()))?;
            f.write_all(b"\n")
                .map_err(|e| LogError::Storage(e.to_string()))?;
            f.sync_data().map_err(|e| LogError::Storage(e.to_string()))
        })();
        if let Err(e) = res {
            self.memory.worlds.get_mut(world).map(|v| v.pop());
            return Err(e);
        }
        Ok(ev)
    }

    fn read(&self, world: &WorldId, cursor: Cursor, limit: usize) -> Result<Vec<Event>, LogError> {
        self.memory.read(world, cursor, limit)
    }

    fn head(&self, world: &WorldId) -> Result<Option<Seq>, LogError> {
        self.memory.head(world)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ev(at: i64, kind: &str) -> NewEvent {
        NewEvent::new(Timestamp(at), kind, Actor::Driver, json!({"k": kind}))
    }

    #[test]
    fn a_kept_tail_reads_exactly_as_the_log_under_it() {
        let w = WorldId::from("a");
        let other = WorldId::from("b");
        let mut plain = MemoryLog::new();
        let mut under = MemoryLog::new();
        for i in 0..(TAIL as i64 + 50) {
            plain.append(&w, ev(i, "old")).unwrap();
            under.append(&w, ev(i, "old")).unwrap();
        }
        let mut tail = Tail::new(under);
        let same = |tail: &Tail<MemoryLog>, plain: &MemoryLog| {
            let head = plain.head(&w).unwrap().unwrap().0;
            assert_eq!(tail.head(&w).unwrap(), Some(Seq(head)));
            for n in [0, 1, 30, 4000, TAIL, TAIL + 20] {
                let got: Vec<Event> = tail
                    .newest(&w, n)
                    .unwrap()
                    .iter()
                    .map(|e| (**e).clone())
                    .collect();
                let want: Vec<Event> = plain
                    .newest(&w, n)
                    .unwrap()
                    .iter()
                    .map(|e| (**e).clone())
                    .collect();
                assert_eq!(got, want, "newest {n}");
            }
            for after in [
                0,
                1,
                49,
                50,
                51,
                52,
                700,
                head - 4000,
                head - 2,
                head - 1,
                head,
                head + 5,
            ] {
                for limit in [1, 3, 1024] {
                    let c = Cursor { after };
                    assert_eq!(
                        tail.read(&w, c, limit).unwrap(),
                        plain.read(&w, c, limit).unwrap(),
                        "after {after}, limit {limit}"
                    );
                }
                let c = Cursor { after };
                assert_eq!(
                    tail.read_all(&w, c).unwrap(),
                    plain.read_all(&w, c).unwrap(),
                    "all after {after}"
                );
            }
        };
        same(&tail, &plain);
        // Appends are seen at once, the kept part slides, and another world is its own.
        for i in 0..300 {
            assert_eq!(
                tail.append(&w, ev(90_000 + i, "new")).unwrap(),
                plain.append(&w, ev(90_000 + i, "new")).unwrap()
            );
        }
        same(&tail, &plain);
        assert_eq!(tail.head(&other).unwrap(), None);
        assert_eq!(tail.append(&other, ev(1, "x")).unwrap().seq, Seq(1));
        assert_eq!(tail.read_all(&other, Cursor::START).unwrap().len(), 1);
        same(&tail, &plain);
    }

    #[test]
    fn sequences_are_dense_per_world() {
        let mut log = MemoryLog::new();
        let a = WorldId::from("a");
        let b = WorldId::from("b");
        assert_eq!(log.append(&a, ev(1, "x")).unwrap().seq, Seq(1));
        assert_eq!(log.append(&a, ev(1, "y")).unwrap().seq, Seq(2));
        assert_eq!(log.append(&b, ev(5, "z")).unwrap().seq, Seq(1));
        assert_eq!(log.head(&a).unwrap(), Some(Seq(2)));
        assert_eq!(log.head(&WorldId::from("none")).unwrap(), None);
    }

    #[test]
    fn time_only_moves_forward() {
        let mut log = MemoryLog::new();
        let w = WorldId::from("w");
        log.append(&w, ev(10, "x")).unwrap();
        assert_eq!(
            log.append(&w, ev(9, "y")).unwrap_err(),
            LogError::OutOfOrder {
                at: Timestamp(9),
                last: Timestamp(10)
            }
        );
        log.append(&w, ev(10, "same-ms-is-fine")).unwrap();
    }

    #[test]
    fn cursors_page_through_the_log() {
        let mut log = MemoryLog::new();
        let w = WorldId::from("w");
        for i in 0..10 {
            log.append(&w, ev(i, "x")).unwrap();
        }
        let first = log.read(&w, Cursor::START, 3).unwrap();
        assert_eq!(
            first.iter().map(|e| e.seq.0).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        let next = log
            .read(&w, Cursor::after(first.last().unwrap().seq), 3)
            .unwrap();
        assert_eq!(
            next.iter().map(|e| e.seq.0).collect::<Vec<_>>(),
            vec![4, 5, 6]
        );
        assert_eq!(log.read_all(&w, Cursor { after: 8 }).unwrap().len(), 2);
        assert!(log.read(&w, Cursor { after: 100 }, 3).unwrap().is_empty());
    }

    struct Counter(u64, Vec<String>);
    impl Projection for Counter {
        type Error = String;
        fn apply(&mut self, event: &Event) -> Result<(), String> {
            if event.kind == "boom" {
                return Err("boom".into());
            }
            self.0 += 1;
            self.1.push(event.kind.clone());
            Ok(())
        }
    }

    #[test]
    fn replay_drives_a_projection_and_reports_where_it_failed() {
        let mut log = MemoryLog::new();
        let w = WorldId::from("w");
        log.append(&w, ev(1, "a")).unwrap();
        log.append(&w, ev(2, "b")).unwrap();
        let mut c = Counter(0, vec![]);
        let cur = replay(&log, &w, Cursor::START, &mut c).unwrap();
        assert_eq!((c.0, cur), (2, Cursor { after: 2 }));
        log.append(&w, ev(3, "boom")).unwrap();
        match replay(&log, &w, cur, &mut c) {
            Err(ReplayError::Projection { at, error }) => {
                assert_eq!((at, error.as_str()), (Seq(3), "boom"))
            }
            other => panic!("expected projection failure, got {other:?}"),
        }
    }

    #[test]
    fn file_log_survives_reopen_and_rejects_corruption() {
        let dir = std::env::temp_dir().join(format!("foxing-filelog-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let w = WorldId::from("w");
        {
            let mut log = FileLog::open(&dir).unwrap();
            log.append(&w, ev(1, "a")).unwrap();
            log.append(&w, ev(2, "b")).unwrap();
            log.append(&WorldId::from("other"), ev(1, "z")).unwrap();
        }
        {
            let mut log = FileLog::open(&dir).unwrap();
            assert_eq!(log.head(&w).unwrap(), Some(Seq(2)));
            assert_eq!(
                log.read_all(&w, Cursor::START)
                    .unwrap()
                    .iter()
                    .map(|e| e.kind.as_str())
                    .collect::<Vec<_>>(),
                vec!["a", "b"]
            );
            assert_eq!(log.append(&w, ev(3, "c")).unwrap().seq, Seq(3));
            assert_eq!(
                log.append(&w, ev(1, "late")).unwrap_err(),
                LogError::OutOfOrder {
                    at: Timestamp(1),
                    last: Timestamp(3)
                }
            );
        }
        // A gap in the sequence is corruption, and open refuses it.
        let path = dir.join("w.events.jsonl");
        let mut text = std::fs::read_to_string(&path).unwrap();
        text = text.replace("\"seq\":2", "\"seq\":9");
        std::fs::write(&path, text).unwrap();
        assert!(matches!(FileLog::open(&dir), Err(LogError::Storage(_))));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn events_round_trip_through_json() {
        let e = Event {
            seq: Seq(7),
            world: WorldId::from("disremembered"),
            at: Timestamp(1_700_000_000_000),
            kind: "disremembered.chunk_earned".into(),
            actor: Actor::Familiar("fam-1".into(), "spacepants".into()),
            cause: Some(Seq(6)),
            payload: json!({"value": 4200.5, "source": "void-lich"}),
        };
        let back: Event = serde_json::from_str(&serde_json::to_string(&e).unwrap()).unwrap();
        assert_eq!(back, e);
    }
}
