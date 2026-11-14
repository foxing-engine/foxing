//! The event log on PostgreSQL.
//!
//! One table, `events`, holds every world's log: `(world, seq)` is the primary key, so
//! the sequence is dense per world and a duplicate append fails loudly instead of
//! forking the log. The driver's [`EventLog`] is synchronous and runs under the world
//! lock; the `postgres` client is synchronous too but owns a tokio runtime, which cannot
//! be entered from a tokio worker thread. So one dedicated thread owns the connection
//! and serves requests over a channel; the log blocks on the reply. An append is one
//! round trip. Reads are the same channel, so they see every append before them.
//!
//! The schema is created on open if it is missing. `FOXING_LOG_URL=postgres://...`
//! selects this log in the driver; `foxing-driver import <dir>` copies a file log in.

use foxing_log::{Actor, Cursor, Event, EventLog, LogError, NewEvent, Seq, WorldId};
use foxing_time::Timestamp;
use postgres::{Client, NoTls};
use std::sync::mpsc;

/// What the connection thread does.
enum Request {
    Append {
        world: WorldId,
        event: NewEvent,
        reply: mpsc::Sender<Result<Event, LogError>>,
    },
    Read {
        world: WorldId,
        cursor: Cursor,
        limit: usize,
        reply: mpsc::Sender<Result<Vec<Event>, LogError>>,
    },
    Head {
        world: WorldId,
        reply: mpsc::Sender<Result<Option<Seq>, LogError>>,
    },
    Begin {
        reply: mpsc::Sender<Result<(), LogError>>,
    },
    Commit {
        reply: mpsc::Sender<Result<(), LogError>>,
    },
}

/// A log on PostgreSQL.
pub struct PostgresLog {
    tx: mpsc::Sender<Request>,
}

/// The advisory lock key schema creation takes.
const SCHEMA_LOCK: i64 = 0x6d6f_6f6e_6469_616c; // "moondial", the engine's old name; the lock key stays

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS events (
    world   TEXT        NOT NULL,
    seq     BIGINT      NOT NULL,
    at      BIGINT      NOT NULL,
    kind    TEXT        NOT NULL,
    actor   JSONB       NOT NULL,
    cause   BIGINT      NULL,
    payload JSONB       NOT NULL,
    PRIMARY KEY (world, seq)
);
CREATE INDEX IF NOT EXISTS events_world_kind ON events (world, kind, seq);
";

/// How often, and how long between, connection attempts at open.
const CONNECT_TRIES: u32 = 20;
const CONNECT_WAIT_SECONDS: u64 = 3;

impl PostgresLog {
    /// Connect, create the schema if it is missing, and start the connection thread.
    pub fn open(url: &str) -> Result<Self, LogError> {
        // At boot the database may still be starting (a reboot brings both up at once):
        // try for a while before giving up.
        let mut client = {
            let mut tries = 0;
            loop {
                match Client::connect(url, NoTls) {
                    Ok(c) => break c,
                    Err(e) if tries < CONNECT_TRIES => {
                        tries += 1;
                        eprintln!("postgres: {e}; retrying ({tries}/{CONNECT_TRIES})");
                        std::thread::sleep(std::time::Duration::from_secs(CONNECT_WAIT_SECONDS));
                    }
                    Err(e) => return Err(storage(e)),
                }
            }
        };
        // Two openers at once (two tests, two drivers) must not both create the schema:
        // CREATE ... IF NOT EXISTS races. A transaction-scoped advisory lock serializes them.
        let mut tx = client.transaction().map_err(storage)?;
        tx.execute("SELECT pg_advisory_xact_lock($1)", &[&SCHEMA_LOCK])
            .map_err(storage)?;
        tx.batch_execute(SCHEMA).map_err(storage)?;
        tx.commit().map_err(storage)?;
        let (tx, rx) = mpsc::channel::<Request>();
        std::thread::Builder::new()
            .name("foxing-log-postgres".into())
            .spawn({
                let url = url.to_string();
                move || serve(client, rx, url)
            })
            .map_err(|e| LogError::Storage(e.to_string()))?;
        Ok(PostgresLog { tx })
    }

    fn ask<T>(
        &self,
        make: impl FnOnce(mpsc::Sender<Result<T, LogError>>) -> Request,
    ) -> Result<T, LogError> {
        let (reply, answer) = mpsc::channel();
        self.tx
            .send(make(reply))
            .map_err(|_| LogError::Storage("the log's connection thread is gone".into()))?;
        answer.recv().map_err(|_| {
            LogError::Storage("the log's connection thread stopped answering".into())
        })?
    }
}

fn storage(e: postgres::Error) -> LogError {
    LogError::Storage(e.to_string())
}

/// The last event this connection appended to a world: its sequence and its time.
type Heads = std::collections::HashMap<WorldId, (u64, i64)>;

/// How often, and how long between, attempts to reconnect when the connection is
/// found closed. Short: the command that asked is waiting, under the world's lock.
const RECONNECT_TRIES: u32 = 5;
const RECONNECT_WAIT_MS: u64 = 1000;

/// The connection was closed under the log (PostgreSQL restarted: an upgrade, a
/// reload gone wrong, the box's own housekeeping). Connect again, and see that the
/// log is where this process left it. It was not reconnected at all, and on 11-14
/// every command failed with "connection closed" from the database's restart until
/// the driver was restarted by hand.
///
/// If a unit was open when the connection went, or the log's head is not the head
/// this process last wrote, the world in memory and the log disagree: the process
/// stops (exit 70) and is started again from the log, as a failed commit does.
fn reconnect(url: &str, heads: &Heads, open: bool) -> Result<Client, LogError> {
    let mut last = None;
    for attempt in 1..=RECONNECT_TRIES {
        match Client::connect(url, NoTls) {
            Ok(mut c) => {
                eprintln!("postgres: the connection was closed; reconnected (attempt {attempt})");
                let mut apart = open;
                for (world, (seq, _)) in heads {
                    match head(&mut c, world) {
                        Ok(h) if h.map(|s| s.0) == Some(*seq) => {}
                        Ok(h) => {
                            eprintln!(
                                "postgres: world {} is at #{} in the log and #{seq} here",
                                world.0,
                                h.map(|s| s.0).unwrap_or(0)
                            );
                            apart = true;
                        }
                        Err(e) => return Err(e),
                    }
                }
                if apart {
                    eprintln!("the log and the world in memory disagree after the connection was lost; stopping here, to be read back from the log");
                    std::process::exit(70);
                }
                return Ok(c);
            }
            Err(e) => {
                eprintln!("postgres: reconnecting: {e} ({attempt}/{RECONNECT_TRIES})");
                last = Some(e);
                std::thread::sleep(std::time::Duration::from_millis(RECONNECT_WAIT_MS));
            }
        }
    }
    Err(last
        .map(storage)
        .unwrap_or_else(|| LogError::Storage("could not reconnect".into())))
}

fn serve(mut client: Client, rx: mpsc::Receiver<Request>, url: String) {
    let mut heads = Heads::new();
    // S2: between a Begin and a Commit every append is one insert in one open
    // transaction, and the commit (the wait for the disk) is paid once for all.
    let mut open = false;
    for req in rx {
        // A closed connection is found before the request is tried, and after one
        // that failed on it (below), so at most one request is lost to a restart.
        if client.is_closed() {
            match reconnect(&url, &heads, open) {
                Ok(c) => {
                    client = c;
                    heads.clear();
                    open = false;
                    INSERT_STMT.with(|s| *s.borrow_mut() = None);
                }
                Err(e) => {
                    let e = LogError::Storage(format!("{e}; the database is not answering"));
                    match req {
                        Request::Append { reply, .. } => drop(reply.send(Err(e))),
                        Request::Read { reply, .. } => drop(reply.send(Err(e))),
                        Request::Head { reply, .. } => drop(reply.send(Err(e))),
                        Request::Begin { reply } | Request::Commit { reply } => {
                            drop(reply.send(Err(e)))
                        }
                    }
                    continue;
                }
            }
        }
        match req {
            Request::Append {
                world,
                event,
                reply,
            } => {
                let _ = reply.send(if open {
                    append_in_unit(&mut client, &mut heads, &world, event)
                } else {
                    append_known(&mut client, &mut heads, &world, event)
                });
            }
            Request::Begin { reply } => {
                // A unit left open (its owner never committed) is committed first:
                // what it appended was applied, and must not be lost behind the next.
                let closed = if open {
                    client.batch_execute("COMMIT").map_err(storage)
                } else {
                    Ok(())
                };
                let began = closed.and_then(|()| client.batch_execute("BEGIN").map_err(storage));
                open = began.is_ok();
                if began.is_err() {
                    heads.clear();
                }
                let _ = reply.send(began);
            }
            Request::Commit { reply } => {
                let done = if open {
                    client.batch_execute("COMMIT").map_err(storage)
                } else {
                    Ok(())
                };
                if done.is_err() {
                    // Nothing of the unit was kept: the heads remembered are wrong,
                    // and the transaction, if it is still open, is aborted.
                    let _ = client.batch_execute("ROLLBACK");
                    heads.clear();
                }
                open = false;
                let _ = reply.send(done);
            }
            Request::Read {
                world,
                cursor,
                limit,
                reply,
            } => {
                let _ = reply.send(read(&mut client, &world, cursor, limit));
            }
            Request::Head { world, reply } => {
                let _ = reply.send(head(&mut client, &world));
            }
        }
    }
}

/// Append inside an open unit: one insert, no commit. The head is read once a world
/// (in the unit's own transaction, locked) and remembered after.
fn append_in_unit(
    client: &mut Client,
    heads: &mut Heads,
    world: &WorldId,
    event: NewEvent,
) -> Result<Event, LogError> {
    let (last, last_at) = match heads.get(world) {
        Some(&h) => h,
        None => {
            let row = client
                .query_opt(
                    "SELECT seq, at FROM events WHERE world = $1 ORDER BY seq DESC LIMIT 1 FOR UPDATE",
                    &[&world.0],
                )
                .map_err(storage)?;
            match row {
                Some(r) => (r.get::<_, i64>(0) as u64, r.get::<_, i64>(1)),
                None => (0, i64::MIN),
            }
        }
    };
    if event.at.0 < last_at {
        return Err(LogError::OutOfOrder {
            at: event.at,
            last: Timestamp(last_at),
        });
    }
    let seq = last + 1;
    let actor = serde_json::to_value(&event.actor).map_err(|e| LogError::Storage(e.to_string()))?;
    let insert = prepared(client)?;
    client
        .execute(
            &insert,
            &[
                &world.0,
                &(seq as i64),
                &event.at.0,
                &event.kind,
                &actor,
                &event.cause.map(|c| c.0 as i64),
                &event.payload,
            ],
        )
        .map_err(|e| {
            // The transaction is aborted now; the commit will fail and say so.
            heads.remove(world);
            storage(e)
        })?;
    heads.insert(world.clone(), (seq, event.at.0));
    Ok(Event {
        seq: Seq(seq),
        world: world.clone(),
        at: event.at,
        kind: event.kind,
        actor: event.actor,
        cause: event.cause,
        payload: event.payload,
    })
}

// The insert, prepared once a connection: unprepared, each append also paid for a
// parse and a describe (two more trips to the server than the insert itself).
thread_local! {
    static INSERT_STMT: std::cell::RefCell<Option<postgres::Statement>> = const { std::cell::RefCell::new(None) };
}

fn prepared(client: &mut Client) -> Result<postgres::Statement, LogError> {
    if let Some(s) = INSERT_STMT.with(|c| c.borrow().clone()) {
        return Ok(s);
    }
    let s = client.prepare(INSERT).map_err(storage)?;
    INSERT_STMT.with(|c| *c.borrow_mut() = Some(s.clone()));
    Ok(s)
}

const INSERT: &str = "INSERT INTO events (world, seq, at, kind, actor, cause, payload) VALUES ($1, $2, $3, $4, $5, $6, $7)";

/// Append, in one statement where this connection knows the world's head (S2).
///
/// The careful append below is four round trips an event: begin, read and lock the
/// head, insert, commit. A driver is the only writer to its world, and this
/// connection appended the last event itself, so it knows the next sequence without
/// asking: one insert, committed as it lands. The primary key still decides: if
/// another writer took that sequence the insert fails, the remembered head is
/// dropped, and the careful append runs and sees the truth.
fn append_known(
    client: &mut Client,
    heads: &mut Heads,
    world: &WorldId,
    event: NewEvent,
) -> Result<Event, LogError> {
    if let Some(&(last, last_at)) = heads.get(world) {
        if event.at.0 < last_at {
            return Err(LogError::OutOfOrder {
                at: event.at,
                last: Timestamp(last_at),
            });
        }
        let seq = last + 1;
        let actor =
            serde_json::to_value(&event.actor).map_err(|e| LogError::Storage(e.to_string()))?;
        let insert = prepared(client)?;
        match client.execute(
            &insert,
            &[
                &world.0,
                &(seq as i64),
                &event.at.0,
                &event.kind,
                &actor,
                &event.cause.map(|c| c.0 as i64),
                &event.payload,
            ],
        ) {
            Ok(_) => {
                heads.insert(world.clone(), (seq, event.at.0));
                return Ok(Event {
                    seq: Seq(seq),
                    world: world.clone(),
                    at: event.at,
                    kind: event.kind,
                    actor: event.actor,
                    cause: event.cause,
                    payload: event.payload,
                });
            }
            Err(e) if e.code() == Some(&postgres::error::SqlState::UNIQUE_VIOLATION) => {
                // Someone else appended: forget, and fall through to the careful way.
                heads.remove(world);
            }
            Err(e) => {
                heads.remove(world);
                return Err(storage(e));
            }
        }
    }
    let at = event.at.0;
    let appended = append(client, world, event)?;
    heads.insert(world.clone(), (appended.seq.0, at));
    Ok(appended)
}

fn append(client: &mut Client, world: &WorldId, event: NewEvent) -> Result<Event, LogError> {
    let mut tx = client.transaction().map_err(storage)?;
    // The world's last event, locked, so two appends cannot take the same seq.
    let last = tx
        .query_opt(
            "SELECT seq, at FROM events WHERE world = $1 ORDER BY seq DESC LIMIT 1 FOR UPDATE",
            &[&world.0],
        )
        .map_err(storage)?;
    let (seq, last_at) = match last {
        Some(row) => (row.get::<_, i64>(0) as u64 + 1, Some(row.get::<_, i64>(1))),
        None => (1, None),
    };
    if let Some(last_at) = last_at {
        if event.at.0 < last_at {
            return Err(LogError::OutOfOrder {
                at: event.at,
                last: Timestamp(last_at),
            });
        }
    }
    let actor = serde_json::to_value(&event.actor).map_err(|e| LogError::Storage(e.to_string()))?;
    tx.execute(
        INSERT,
        &[
            &world.0,
            &(seq as i64),
            &event.at.0,
            &event.kind,
            &actor,
            &event.cause.map(|c| c.0 as i64),
            &event.payload,
        ],
    )
    .map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(Event {
        seq: Seq(seq),
        world: world.clone(),
        at: event.at,
        kind: event.kind,
        actor: event.actor,
        cause: event.cause,
        payload: event.payload,
    })
}

fn read(
    client: &mut Client,
    world: &WorldId,
    cursor: Cursor,
    limit: usize,
) -> Result<Vec<Event>, LogError> {
    let rows = client
        .query(
            "SELECT seq, at, kind, actor, cause, payload FROM events WHERE world = $1 AND seq > $2 ORDER BY seq LIMIT $3",
            &[&world.0, &(cursor.after as i64), &(limit as i64)],
        )
        .map_err(storage)?;
    rows.iter()
        .map(|row| {
            let actor: serde_json::Value = row.get(3);
            let actor: Actor =
                serde_json::from_value(actor).map_err(|e| LogError::Storage(e.to_string()))?;
            Ok(Event {
                seq: Seq(row.get::<_, i64>(0) as u64),
                world: world.clone(),
                at: Timestamp(row.get(1)),
                kind: row.get(2),
                actor,
                cause: row.get::<_, Option<i64>>(4).map(|c| Seq(c as u64)),
                payload: row.get(5),
            })
        })
        .collect()
}

fn head(client: &mut Client, world: &WorldId) -> Result<Option<Seq>, LogError> {
    let row = client
        .query_opt("SELECT max(seq) FROM events WHERE world = $1", &[&world.0])
        .map_err(storage)?;
    Ok(row
        .and_then(|r| r.get::<_, Option<i64>>(0))
        .map(|s| Seq(s as u64)))
}

impl EventLog for PostgresLog {
    fn append(&mut self, world: &WorldId, event: NewEvent) -> Result<Event, LogError> {
        let world = world.clone();
        self.ask(|reply| Request::Append {
            world,
            event,
            reply,
        })
    }

    fn read(&self, world: &WorldId, cursor: Cursor, limit: usize) -> Result<Vec<Event>, LogError> {
        let world = world.clone();
        self.ask(|reply| Request::Read {
            world,
            cursor,
            limit,
            reply,
        })
    }

    fn head(&self, world: &WorldId) -> Result<Option<Seq>, LogError> {
        let world = world.clone();
        self.ask(|reply| Request::Head { world, reply })
    }

    fn begin(&mut self) -> Result<(), LogError> {
        self.ask(|reply| Request::Begin { reply })
    }

    fn commit(&mut self) -> Result<(), LogError> {
        self.ask(|reply| Request::Commit { reply })
    }
}

/// Copy every event of `world` from one log into another, in order, refusing to write
/// over a world that already has events. For moving a file log to PostgreSQL.
pub fn copy_world(
    from: &dyn EventLog,
    to: &mut dyn EventLog,
    world: &WorldId,
) -> Result<u64, LogError> {
    if to.head(world)?.is_some() {
        return Err(LogError::Storage(format!(
            "world {} already has events in the destination; refusing to interleave",
            world.0
        )));
    }
    let mut n = 0;
    for e in from.read_all(world, Cursor::START)? {
        let got = to.append(
            world,
            NewEvent {
                at: e.at,
                kind: e.kind,
                actor: e.actor,
                cause: e.cause,
                payload: e.payload,
            },
        )?;
        if got.seq != e.seq {
            return Err(LogError::Storage(format!(
                "sequence drift while copying: source {} became {}",
                e.seq.0, got.seq.0
            )));
        }
        n += 1;
    }
    Ok(n)
}
