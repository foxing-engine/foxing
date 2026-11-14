//! Runs against a real PostgreSQL when `FOXING_TEST_PG` names it; otherwise skips.
//! CI provides one as a service; locally: `FOXING_TEST_PG=postgres://user@host/db`.

use foxing_log::{Actor, Cursor, EventLog, FileLog, LogError, NewEvent, Seq, WorldId};
use foxing_log_postgres::{copy_world, PostgresLog};
use foxing_time::Timestamp;
use serde_json::json;

fn url() -> Option<String> {
    std::env::var("FOXING_TEST_PG")
        .or_else(|_| std::env::var("MOONDIAL_TEST_PG"))
        .ok()
        .filter(|s| !s.is_empty())
}

fn fresh_world() -> WorldId {
    WorldId(format!(
        "test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn ev(at: i64, kind: &str, n: u64) -> NewEvent {
    NewEvent {
        at: Timestamp(at),
        kind: kind.into(),
        actor: Actor::Citizen("dave".into()),
        cause: None,
        payload: json!({"n": n, "text": "héllo \"quoted\""}),
    }
}

#[test]
fn appends_read_back_in_order_with_dense_sequences() {
    let Some(url) = url() else {
        eprintln!("FOXING_TEST_PG not set; skipping");
        return;
    };
    let mut log = PostgresLog::open(&url)
        .unwrap_or_else(|e| panic!("cannot open the test database at {url}: {e}"));
    let w = fresh_world();
    assert_eq!(log.head(&w).unwrap(), None);
    for i in 1..=5 {
        let e = log.append(&w, ev(1000 * i as i64, "t.k", i)).unwrap();
        assert_eq!(e.seq, Seq(i));
        assert_eq!(e.world, w);
    }
    assert_eq!(log.head(&w).unwrap(), Some(Seq(5)));
    let all = log.read_all(&w, Cursor::START).unwrap();
    assert_eq!(all.len(), 5);
    assert_eq!(all[2].payload["n"], 3);
    assert_eq!(all[2].payload["text"], "héllo \"quoted\"");
    assert_eq!(all[2].actor, Actor::Citizen("dave".into()));
    let tail = log.read(&w, Cursor { after: 3 }, 10).unwrap();
    assert_eq!(tail.iter().map(|e| e.seq.0).collect::<Vec<_>>(), vec![4, 5]);
    let page = log.read(&w, Cursor::START, 2).unwrap();
    assert_eq!(page.len(), 2);

    // Clocks only move forward.
    let err = log.append(&w, ev(10, "t.k", 9)).unwrap_err();
    assert!(matches!(err, LogError::OutOfOrder { .. }), "{err}");

    // Another world is another sequence.
    let w2 = fresh_world();
    assert_eq!(log.append(&w2, ev(1, "t.k", 1)).unwrap().seq, Seq(1));
    assert_eq!(log.head(&w).unwrap(), Some(Seq(5)));

    // A second connection sees the same log: it is the database, not the process.
    let log2 = PostgresLog::open(&url).unwrap();
    assert_eq!(log2.head(&w).unwrap(), Some(Seq(5)));
}

#[test]
fn a_file_log_copies_in_once() {
    let Some(url) = url() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let mut file = FileLog::open(dir.path()).unwrap();
    let w = fresh_world();
    for i in 1..=3 {
        file.append(&w, ev(i as i64 * 7, "store.x", i)).unwrap();
    }
    let mut pg = PostgresLog::open(&url)
        .unwrap_or_else(|e| panic!("cannot open the test database at {url}: {e}"));
    assert_eq!(copy_world(&file, &mut pg, &w).unwrap(), 3);
    assert_eq!(
        pg.read_all(&w, Cursor::START).unwrap(),
        file.read_all(&w, Cursor::START).unwrap()
    );
    let err = copy_world(&file, &mut pg, &w).unwrap_err();
    assert!(err.to_string().contains("already has events"), "{err}");
}

#[test]
fn a_second_writer_is_seen_and_sequences_stay_dense() {
    // S2: a connection appends by the head it remembers. Another writer in between
    // takes that sequence; the first must notice, not fail and not skip.
    let Some(url) = url() else {
        eprintln!("FOXING_TEST_PG not set; skipping");
        return;
    };
    let world = fresh_world();
    let mut a = PostgresLog::open(&url).unwrap();
    let mut b = PostgresLog::open(&url).unwrap();
    for n in 0..5 {
        assert_eq!(
            a.append(&world, ev(100 + n as i64, "from.a", n))
                .unwrap()
                .seq,
            Seq(n + 1)
        );
    }
    assert_eq!(b.append(&world, ev(200, "from.b", 0)).unwrap().seq, Seq(6));
    assert_eq!(b.append(&world, ev(201, "from.b", 1)).unwrap().seq, Seq(7));
    // `a` still remembers 5 as the head.
    assert_eq!(a.append(&world, ev(300, "from.a", 5)).unwrap().seq, Seq(8));
    assert_eq!(a.append(&world, ev(301, "from.a", 6)).unwrap().seq, Seq(9));
    // And an event older than the head is refused on the quick path as on the careful one.
    assert!(matches!(
        a.append(&world, ev(1, "late", 0)),
        Err(LogError::OutOfOrder { .. })
    ));
    let all = a.read_all(&world, Cursor::START).unwrap();
    assert_eq!(
        all.iter().map(|e| e.seq.0).collect::<Vec<_>>(),
        (1..=9).collect::<Vec<_>>()
    );
    assert_eq!(a.head(&world).unwrap(), Some(Seq(9)));
}

#[test]
fn a_thousand_appends_time() {
    // Not an assertion: prints what an append costs here.
    let Some(url) = url() else { return };
    let world = fresh_world();
    let mut log = PostgresLog::open(&url).unwrap();
    let t = std::time::Instant::now();
    for n in 0..1000 {
        log.append(&world, ev(n as i64, "timed", n)).unwrap();
    }
    eprintln!("1000 appends: {:.2} ms each", t.elapsed().as_secs_f64());
}

#[test]
fn a_unit_is_durable_at_its_commit_and_not_before() {
    // S2: appends between begin and commit are one transaction. The writer reads
    // its own at once; another connection sees none of them until the commit.
    let Some(url) = url() else {
        eprintln!("FOXING_TEST_PG not set; skipping");
        return;
    };
    let world = fresh_world();
    let mut a = PostgresLog::open(&url).unwrap();
    let b = PostgresLog::open(&url).unwrap();
    a.append(&world, ev(1, "before", 0)).unwrap();
    a.begin().unwrap();
    for n in 0..20 {
        assert_eq!(
            a.append(&world, ev(10 + n as i64, "in.unit", n))
                .unwrap()
                .seq,
            Seq(n + 2)
        );
    }
    assert_eq!(a.head(&world).unwrap(), Some(Seq(21)));
    assert_eq!(a.read_all(&world, Cursor::START).unwrap().len(), 21);
    assert_eq!(
        b.head(&world).unwrap(),
        Some(Seq(1)),
        "not visible before the commit"
    );
    a.commit().unwrap();
    assert_eq!(b.head(&world).unwrap(), Some(Seq(21)));
    // After the unit, appends are single again and carry on in sequence; a commit
    // with nothing open is nothing; a begin over an open unit keeps what it held.
    assert_eq!(a.append(&world, ev(100, "after", 0)).unwrap().seq, Seq(22));
    a.commit().unwrap();
    a.begin().unwrap();
    a.append(&world, ev(101, "left.open", 0)).unwrap();
    a.begin().unwrap();
    a.append(&world, ev(102, "second.unit", 0)).unwrap();
    a.commit().unwrap();
    let all = b.read_all(&world, Cursor::START).unwrap();
    assert_eq!(
        all.iter().map(|e| e.seq.0).collect::<Vec<_>>(),
        (1..=24).collect::<Vec<_>>()
    );
    // A unit on a world this connection has never appended to finds the head itself.
    let fresh = fresh_world();
    let mut c = PostgresLog::open(&url).unwrap();
    c.begin().unwrap();
    assert_eq!(c.append(&fresh, ev(1, "first", 0)).unwrap().seq, Seq(1));
    assert_eq!(c.append(&fresh, ev(2, "second", 0)).unwrap().seq, Seq(2));
    c.commit().unwrap();
    assert_eq!(b.head(&fresh).unwrap(), Some(Seq(2)));
}

#[test]
fn a_thousand_appends_in_units_of_ten_time() {
    let Some(url) = url() else { return };
    let world = fresh_world();
    let mut log = PostgresLog::open(&url).unwrap();
    let t = std::time::Instant::now();
    for n in 0..1000u64 {
        if n % 10 == 0 {
            log.begin().unwrap();
        }
        log.append(&world, ev(n as i64, "timed", n)).unwrap();
        if n % 10 == 9 {
            log.commit().unwrap();
        }
    }
    eprintln!(
        "1000 appends in units of ten: {:.2} ms each",
        t.elapsed().as_secs_f64()
    );
}

/// The database restarts under a running log (every backend is terminated): the
/// request in flight may fail, and the next one finds the connection closed,
/// reconnects, and carries on at the next sequence. Before this the log never
/// reconnected, and every command failed until the driver was restarted (11-14).
#[test]
fn the_log_reconnects_when_the_database_closes_its_connection() {
    let Some(url) = url() else {
        eprintln!("FOXING_TEST_PG not set; skipping");
        return;
    };
    // This log's connection carries a name of its own, so only it is ended: the
    // other tests share the database and run beside this one (ending every backend
    // failed one of them in CI the first time).
    let w = fresh_world();
    let name = format!("reconnect-{}", w.0);
    let mine = format!(
        "{url}{}application_name={name}",
        if url.contains('?') { "&" } else { "?" }
    );
    let mut log = PostgresLog::open(&mine).unwrap();
    for i in 1..=3 {
        log.append(&w, ev(1000 * i as i64, "t.k", i)).unwrap();
    }
    // Another connection ends this log's backend, as a restart ends them all.
    let mut other = postgres::Client::connect(&url, postgres::NoTls).unwrap();
    let ended = other
        .query(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE application_name = $1",
            &[&name],
        )
        .unwrap();
    assert_eq!(ended.len(), 1, "the log's one backend was found and ended");
    std::thread::sleep(std::time::Duration::from_millis(300));
    // At most one request is lost; then the log is itself again.
    let first = log.head(&w);
    let head = match first {
        Ok(h) => h,
        Err(_) => log.head(&w).expect("the second request reconnects"),
    };
    assert_eq!(head, Some(Seq(3)));
    let e = log
        .append(&w, ev(4000, "t.k", 4))
        .expect("appends after the reconnect");
    assert_eq!(e.seq, Seq(4));
    assert_eq!(log.read_all(&w, Cursor::START).unwrap().len(), 4);
}
