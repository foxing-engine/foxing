use super::*;
use crate::world::LogBacking;
use foxing_log::WorldId;
use foxing_registry::{ActorKind, Outcome};
use foxing_store::{Store, Viewer};
use foxing_time::Timestamp;

const PASS: &str = "correct-horse-battery";

fn open(dir: &std::path::Path, at: Timestamp) -> World {
    let mut w = World::open(
        WorldId("test".into()),
        Store::new(registry()),
        LogBacking::File(dir.to_path_buf()),
    )
    .unwrap();
    w.fixed_now = Some(at);
    w.seed(7);
    ensure_world(&mut w).unwrap();
    register(&mut w, Curves::with_half_life(Duration::seconds(10)));
    w.tick(); // the rooms fill
    w
}

fn call(
    w: &mut World,
    actor: &str,
    kind: ActorKind,
    verb: &str,
    args: Value,
) -> Result<Outcome, CommandError> {
    w.call(
        &CallContext {
            actor: actor.into(),
            kind,
        },
        verb,
        &args,
    )
}

/// The curves the test world runs on.
fn curves() -> Curves {
    Curves::with_half_life(Duration::seconds(10))
}

fn ok(w: &mut World, actor: &str, verb: &str, args: Value) -> Outcome {
    call(w, actor, ActorKind::Citizen, verb, args.clone())
        .unwrap_or_else(|e| panic!("{verb} {args}: {e}"))
}

fn admin(w: &mut World, actor: &str, verb: &str, args: Value) -> Outcome {
    call(w, actor, ActorKind::Admin, verb, args.clone())
        .unwrap_or_else(|e| panic!("{verb} {args}: {e}"))
}

fn level(w: &mut World, actor: &str) -> f64 {
    ok(w, actor, "chunks", json!([])).result["level"]
        .as_f64()
        .unwrap()
}

/// A regular: twenty lines through the gate today, as the Clocktower counts them.
/// A regular: twenty messages through the gate today across two half-hours, as the
/// Clocktower counts them. Half go in now, half a quarter-hour on (the clock is left
/// there: the log is in order, and the tests that call this set the hour after).
fn regular(w: &mut World, actors: &[&str]) {
    let now = w.now();
    // Standing costs a deed too: a thing made counts, and is the cheapest to fake here.
    for actor in actors {
        w.note(
            Actor::Citizen(actor.to_string()),
            super::craft::MADE,
            json!({"by": actor, "item": 0, "name": "a test", "recipe": "test", "of": [], "spent": null}),
        )
        .unwrap();
    }
    for i in 0..super::economy::REGULAR_LINES {
        if i == super::economy::REGULAR_LINES / 2 {
            w.fixed_now = Some(Timestamp(now.0 + 15 * 60 * 1000));
        }
        for actor in actors {
            w.note(
                Actor::Citizen(actor.to_string()),
                "gate.message",
                json!({"from": actor, "text": "look", "compiled": true}),
            )
            .unwrap();
        }
    }
}

/// `n` messages through the gate now, from each: enough for the Clocktower to count
/// them as about for the hour.
fn messages(w: &mut World, actors: &[&str], n: u64) {
    for _ in 0..n {
        for actor in actors {
            w.note(
                Actor::Citizen(actor.to_string()),
                "gate.message",
                json!({"from": actor, "text": "look", "compiled": true}),
            )
            .unwrap();
        }
    }
}

fn events(w: &World) -> u64 {
    w.log.head(&w.id).unwrap().unwrap().0
}

/// Fight whatever is here until it dies or you do. Returns the last outcome.
/// The narrator's lines to `to` since `after`.
fn narrative(w: &World, to: &str, after: u64) -> Vec<String> {
    w.log
        .read_all(&w.id, foxing_log::Cursor { after })
        .unwrap()
        .iter()
        .filter(|e| e.kind == combat::NARRATIVE && e.payload["to"] == to)
        .map(|e| e.payload["text"].as_str().unwrap().to_string())
        .collect()
}

fn fight(w: &mut World, actor: &str) -> Outcome {
    for _ in 0..40 {
        let out = ok(w, actor, "attack", json!([]));
        if out.result["killed"] == json!(true) || out.result["died"] == json!(true) {
            return out;
        }
    }
    panic!("fight did not end")
}

#[test]
fn a_passphrase_is_set_on_first_login_and_required_after() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = open(dir.path(), Timestamp(0));
    assert_eq!(
        login(&mut w, "dave", "test", "short").err(),
        Some(LoginError::TooShort {
            min: identity::MIN_PASSPHRASE
        })
    );
    assert_eq!(
        login(&mut w, "dave", "test", "").err(),
        Some(LoginError::Missing)
    );
    assert!(login(&mut w, "dave", "test", PASS).is_ok());
    assert_eq!(
        login(&mut w, "dave", "test", "not-it-at-all").err(),
        Some(LoginError::WrongPassphrase {
            name: "dave".into(),
            passing: false
        })
    );
    assert_eq!(
        login(&mut w, "dave", "test", "").err(),
        Some(LoginError::Missing)
    );
    let f = login(&mut w, "dave", "test", PASS).unwrap();
    // Only an admin sees the hash; the owner does not.
    let now = w.now();
    let mine = w
        .store
        .view(f.citizen, &Viewer::actor("dave"), now)
        .unwrap();
    assert!(mine["components"][ACCOUNT].get("passphrase_hash").is_none());
    let theirs = w.store.view(f.citizen, &Viewer::admin(), now).unwrap();
    assert!(theirs["components"][ACCOUNT]["passphrase_hash"]
        .as_str()
        .unwrap()
        .starts_with("$argon2id$"));
}

#[test]
fn the_map_is_built_once_and_travel_follows_exits() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = open(dir.path(), Timestamp(0));
    let built = events(&w);
    assert_eq!(
        w.store.entities().filter(|e| e.kind == "room").count(),
        rooms::ROOMS.len()
    );
    login(&mut w, "dave", "test", PASS).unwrap();
    let look = ok(&mut w, "dave", "look", json!([]));
    assert_eq!(look.result["room"], json!("The Threshold"));
    assert_eq!(look.result["exits"], json!(["east", "north"]));
    assert!(matches!(
        call(
            &mut w,
            "dave",
            ActorKind::Citizen,
            "travel",
            json!(["sideways"])
        ),
        Err(CommandError::Precondition { .. })
    ));
    let fog = ok(&mut w, "dave", "travel", json!(["east"]));
    assert_eq!(fog.result["room"], json!("The Fogbank"));
    assert_eq!(
        fog.result["creatures"].as_array().unwrap().len(),
        2,
        "the fog fills to its count on arrival"
    );
    drop(w);
    // Reopening builds nothing: the rooms are in the log.
    let w = open(dir.path(), Timestamp(1000));
    assert_eq!(
        w.store.entities().filter(|e| e.kind == "room").count(),
        rooms::ROOMS.len()
    );
    assert!(events(&w) > built, "the login and travel were recorded");
    let mut w = w;
    let before = events(&w);
    ensure_world(&mut w).unwrap();
    assert_eq!(
        events(&w),
        before,
        "{:?}",
        w.recent_events(2)
            .iter()
            .map(|e| (e.kind.clone(), e.payload.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_fight_earns_a_chunk_whose_vividity_is_how_close_you_came() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = open(dir.path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    ok(&mut w, "dave", "travel", json!(["east"]));
    assert_eq!(level(&mut w, "dave"), 0.0);
    let win = fight(&mut w, "dave");
    assert_eq!(win.result["killed"], json!(true));
    let hp = win.result["hp"].as_f64().unwrap();
    let vividity = win.result["vividity"].as_f64().unwrap();
    assert!((vividity - (1.0 - hp / MAX_HP)).abs() < 1e-9);
    // About fifteen: a quarter either way, rolled when it spawned.
    let worth = level(&mut w, "dave");
    assert!(
        (11.0..=19.0).contains(&worth),
        "a wisp is worth about 15: {worth}"
    );
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    assert_eq!(
        combat::creatures_in(&w, fog).len(),
        1,
        "one wisp left; the other respawns later"
    );

    // Attack follows the square root of level: a few hundred XP is a fighter, not
    // a one-hit killer of everything below the Archive.
    assert_eq!(combat::attack_for_level(0.0), 3.0);
    assert_eq!(combat::attack_for_level(100.0), 13.0);
    assert_eq!(combat::attack_for_level(2000.0), 47.0);
    assert_eq!(combat::attack_for_level(6000.0), 80.0);
    assert_eq!(
        ok(&mut w, "dave", "chunks", json!([])).result["attack"],
        json!(6.0)
    );
}

#[test]
fn vivid_chunks_fade_slower() {
    let dir = tempfile::tempdir().unwrap();
    let t0 = Timestamp(0);
    let mut w = open(dir.path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let dull = admin(&mut w, "dave", "earn", json!(["dull", 100, 0.0])).result["chunk"]
        .as_u64()
        .unwrap();
    let vivid = admin(&mut w, "dave", "earn", json!(["vivid", 100, 1.0])).result["chunk"]
        .as_u64()
        .unwrap();
    w.fixed_now = Some(t0.plus(Duration::seconds(10)));
    let read = |w: &World, id: u64| {
        w.store
            .read_field(EntityId(id), CHUNK, "value", w.now())
            .unwrap()
            .as_f64()
            .unwrap()
    };
    assert!((read(&w, dull) - 50.0).abs() < 1e-6, "one half-life");
    assert!(
        (read(&w, vivid) - 100.0 * 0.5f64.powf(1.0 / 3.0)).abs() < 1e-6,
        "three times the half-life"
    );
    // Shelving keeps the stretch.
    ok(&mut w, "dave", "shelve", json!([format!("chunk-{vivid}")]));
    ok(&mut w, "dave", "shelve", json!([format!("chunk-{dull}")]));
    w.fixed_now = Some(t0.plus(Duration::seconds(10 + 50)));
    assert!(
        (read(&w, dull) - 25.0).abs() < 1e-6,
        "palace halves at 5x the half-life"
    );
    let vivid_at_shelve = 100.0 * 0.5f64.powf(1.0 / 3.0);
    assert!(
        (read(&w, vivid) - vivid_at_shelve / (1.0 + 50.0 / 150.0)).abs() < 1e-6,
        "the vivid palace curve has three times the scale"
    );
}

#[test]
fn death_is_a_concussion_and_a_walk_back() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = open(dir.path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    // Three dull chunks to lose one of.
    for i in 0..3 {
        admin(&mut w, "dave", "earn", json!([format!("memory-{i}"), 10]));
    }
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["north"]));
    ok(&mut w, "dave", "travel", json!(["east"]));
    let out = fight(&mut w, "dave");
    assert_eq!(
        out.result["died"],
        json!(true),
        "a level-30 citizen does not beat a revenant: {}",
        out.summary
    );
    let look = ok(&mut w, "dave", "look", json!([]));
    assert_eq!(
        look.result["room"],
        json!("The Threshold"),
        "you wake at the Threshold"
    );
    let f = login(&mut w, "dave", "test", PASS).unwrap();
    assert_eq!(combat::hp(&w, f.citizen), MAX_HP);
    assert!(combat::engaged_with(&w, f.citizen).is_none());
    let chunks = ok(&mut w, "dave", "chunks", json!([])).result["chunks"]
        .as_array()
        .unwrap()
        .len();
    // Three chunks of ten: the share owed at thirty is small, so the chunk the roll
    // lands on is thinned to it, not taken whole.
    match out.result["lost"].as_str() {
        Some(lost) => {
            assert_eq!(chunks, 3, "thinned, not taken: {lost}");
            assert!(lost.contains("XP of chunk-"), "{lost}");
            assert!(
                level(&mut w, "dave") < 30.0 && level(&mut w, "dave") > 27.0,
                "{}",
                level(&mut w, "dave")
            );
        }
        None => assert_eq!(chunks, 3),
    }
    // The same seed gives the same fight.
    let dir2 = tempfile::tempdir().unwrap();
    let mut w2 = open(dir2.path(), Timestamp(0));
    login(&mut w2, "dave", "test", PASS).unwrap();
    for i in 0..3 {
        admin(&mut w2, "dave", "earn", json!([format!("memory-{i}"), 10]));
    }
    for exit in ["east", "north", "east"] {
        ok(&mut w2, "dave", "travel", json!([exit]));
    }
    assert_eq!(fight(&mut w2, "dave").summary, out.summary);
}

#[test]
fn you_cannot_travel_mid_fight_and_retreat_costs_a_hit() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = open(dir.path(), Timestamp(0));
    let f = login(&mut w, "dave", "test", PASS).unwrap();
    ok(&mut w, "dave", "travel", json!(["east"]));
    assert!(matches!(
        call(&mut w, "dave", ActorKind::Citizen, "retreat", json!([])),
        Err(CommandError::Precondition { .. })
    ));
    let round = ok(&mut w, "dave", "attack", json!(["wisp"]));
    assert_eq!(round.result["killed"], json!(false), "{}", round.summary);
    assert!(combat::engaged_with(&w, f.citizen).is_some());
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "travel",
        json!(["west"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Retreat"), "{err}");
    let before = combat::hp(&w, f.citizen);
    let out = ok(&mut w, "dave", "retreat", json!([]));
    assert!(combat::hp(&w, f.citizen) < before, "{}", out.summary);
    assert!(combat::engaged_with(&w, f.citizen).is_none());
    ok(&mut w, "dave", "travel", json!(["west"]));
    // Healing is on the citizen's clock. PTO is absence (#3, twice over: a paused
    // clock that kept earning banked every chunk, and a pause taken around every
    // action did the same): the clock stops when the room disremembers you, ten
    // minutes after your last word, and runs again the moment you act.
    let hurt = combat::hp(&w, f.citizen);
    let t_hurt = w.now();
    let out = ok(&mut w, "dave", "pto", json!([]));
    assert!(out.summary.starts_with("Leave granted"), "{}", out.summary);
    assert!(!super::on_pto(&w, "dave"), "not yet: he is still here");
    // Still here: healing goes on until the room disremembers him.
    w.fixed_now = Some(t_hurt.plus(Duration::seconds(100)));
    assert!(combat::hp(&w, f.citizen) > hurt);
    w.fixed_now = Some(
        t_hurt
            .plus(crate::world::ABSENCE)
            .plus(Duration::seconds(1)),
    );
    w.tick();
    assert!(super::on_pto(&w, "dave"), "gone, and his clock with him");
    let at_leave = combat::hp(&w, f.citizen);
    w.fixed_now = Some(w.now().plus(Duration::seconds(1000)));
    assert_eq!(combat::hp(&w, f.citizen), at_leave, "nothing heals on PTO");
    // His first word ends it, and he is told how long it stood.
    let head = events(&w);
    ok(&mut w, "dave", "look", json!([]));
    assert!(!super::on_pto(&w, "dave"));
    assert!(
        narrative(&w, "dave", head)
            .iter()
            .any(|l| l.starts_with("Your clock runs again: it stood still for")),
        "{:?}",
        narrative(&w, "dave", head)
    );
    assert_eq!(
        ok(&mut w, "dave", "time", json!([])).result["on_pto"],
        json!(false)
    );
    // Two windows in thirty days: the second is granted, the third refused; resume
    // takes back an asking that has not begun.
    ok(&mut w, "dave", "pto", json!([]));
    let out = ok(&mut w, "dave", "resume", json!([]));
    assert!(
        out.summary.starts_with("The asking is taken back"),
        "{}",
        out.summary
    );
    let err = call(&mut w, "dave", ActorKind::Citizen, "pto", json!([])).unwrap_err();
    assert!(
        err.to_string().contains("2 windows of PTO in thirty days"),
        "{err}"
    );
    let rest = ok(&mut w, "dave", "rest", json!([]));
    assert!(rest.result["full_in_seconds"].as_i64().is_some());
}

#[test]
fn a_session_survives_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let t0 = Timestamp(1_000_000);
    let mut w = open(dir.path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();

    let earned = admin(&mut w, "dave", "earn", json!(["hydra", 100]));
    let chunk = format!("chunk-{}", earned.result["chunk"]);
    assert!((level(&mut w, "dave") - 100.0).abs() < 1e-9);

    // One half-life later: 50. Shelve it; the palace reads the same value at that instant.
    w.fixed_now = Some(t0.plus(Duration::seconds(10)));
    assert!((level(&mut w, "dave") - 50.0).abs() < 1e-6);
    let shelved = ok(&mut w, "dave", "shelve", json!([chunk.clone()]));
    assert!((shelved.result["value"].as_f64().unwrap() - 50.0).abs() < 1e-6);
    assert_eq!(
        level(&mut w, "dave"),
        0.0,
        "a shelved chunk counts for nothing"
    );

    // Errors are legible and do not touch the log.
    let head = events(&w);
    assert!(matches!(
        call(
            &mut w,
            "dave",
            ActorKind::Citizen,
            "shelve",
            json!(["chunk-99"])
        ),
        Err(CommandError::Precondition { .. })
    ));
    assert!(matches!(
        call(&mut w, "dave", ActorKind::Citizen, "freeze", json!([])),
        Err(CommandError::NotAllowed { .. })
    ));
    assert!(matches!(
        call(&mut w, "dave", ActorKind::Citizen, "earn", json!(["x", 1])),
        Err(CommandError::NotAllowed { .. })
    ));
    assert!(
        matches!(
            call(&mut w, "tyler", ActorKind::Citizen, "chunks", json!([])),
            Err(CommandError::Precondition { .. })
        ),
        "tyler has not arrived"
    );
    assert_eq!(events(&w), head);

    // The world freeze is admin-only and holds everything, dave's palace included.
    admin(&mut w, "dave", "freeze", json!([]));
    let recorded = events(&w);
    drop(w);

    // Reopen fifty seconds later: same events, same values, the world still frozen.
    let mut w = open(dir.path(), t0.plus(Duration::seconds(60)));
    assert_eq!(events(&w), recorded, "nothing recorded by reopening");
    let palace = ok(&mut w, "dave", "palace", json!([]));
    assert!(
        (palace.result["chunks"][0]["value"].as_f64().unwrap() - 50.0).abs() < 1e-6,
        "a frozen world holds the value across a restart"
    );
    let time = ok(&mut w, "dave", "time", json!([]));
    assert_eq!(time.result["world_frozen"], json!(true));
    assert_eq!(time.result["on_pto"], json!(false));

    // Thaw the world: it fades.
    admin(&mut w, "dave", "thaw", json!([]));
    w.fixed_now = Some(t0.plus(Duration::seconds(60 + 50)));
    let back = ok(&mut w, "dave", "reminisce", json!([chunk]));
    assert!(
        (back.result["value"].as_f64().unwrap() - 25.0).abs() < 1e-6,
        "power law halves at its scale"
    );
    assert!(
        (level(&mut w, "dave") - 25.0).abs() < 1e-6,
        "and counts again"
    );
}

#[test]
fn strangers_see_a_chunk_exists_but_not_what_it_is_worth() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    let id = admin(&mut w, "dave", "earn", json!(["lich", 10])).result["chunk"]
        .as_u64()
        .unwrap();
    let now = w.now();
    let mine = w
        .store
        .view(EntityId(id), &Viewer::actor("dave"), now)
        .unwrap();
    let theirs = w
        .store
        .view(EntityId(id), &Viewer::actor("tyler"), now)
        .unwrap();
    assert_eq!(mine["components"][CHUNK]["value"], json!(10.0));
    assert_eq!(theirs["components"][CHUNK]["source"], json!("lich"));
    assert!(theirs["components"][CHUNK].get("value").is_none());
    assert!(theirs["components"][CHUNK].get("size").is_none());
}

#[test]
fn observe_is_the_text_client_s_eyes() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    admin(&mut w, "dave", "earn", json!(["hydra", 100, 0.5]));

    // `here` from dave's point of view, through the command, like the text port.
    let q = r#"{"root":"here","select":{"disremembered.room":["name"]},"relations":{"contents":{"kinds":["citizen"],"select":{"disremembered.citizen":["name"]}},"exits":{"select":{"disremembered.room":["name"]}}}}"#;
    let out = ok(&mut w, "dave", "observe", json!([q]));
    let root = &out.result["root"];
    assert_eq!(
        root["components"]["disremembered.room"]["name"],
        json!("The Threshold")
    );
    let names: Vec<&str> = root["relations"]["contents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| {
            n["components"]["disremembered.citizen"]["name"]
                .as_str()
                .unwrap()
        })
        .collect();
    assert_eq!(names, vec!["dave", "tyler"]);
    assert_eq!(root["relations"]["exits"][0]["via"], json!("east"));
    assert!(
        out.result["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "travel"),
        "legal commands come with the observation, even from inside a command"
    );
    let cursor = out.result["cursor"].as_u64().unwrap();

    // tyler asks for dave's window: sources yes, values no.
    let out = ok(
        &mut w,
        "tyler",
        "observe",
        json!([
            r#"{"root":{"window":"dave"},"select":{},"relations":{"contents":{"select":{"disremembered.chunk":"*"}}}}"#
        ]),
    );
    let chunk =
        &out.result["root"]["relations"]["contents"][0]["components"]["disremembered.chunk"];
    assert_eq!(chunk["source"], json!("hydra"));
    assert!(chunk.get("value").is_none() && chunk.get("vividity").is_none());

    // Events since the cursor: dave travels; tyler, still at the Threshold, sees dave leave.
    ok(&mut w, "dave", "travel", json!(["east"]));
    let out = ok(
        &mut w,
        "tyler",
        "observe",
        json!([format!(
            r#"{{"root":"self","select":{{}},"after":{cursor}}}"#
        )]),
    );
    let kinds: Vec<&str> = out.result["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"store.entity_moved"), "{kinds:?}");
    assert!(out.result["cursor"].as_u64().unwrap() > cursor);

    // Errors are legible.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "observe",
        json!([r#"{"root":"here","relations":{"content":{}}}"#]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("did you mean contents"), "{err}");
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "observe",
        json!(["{nope"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("not JSON"), "{err}");
}

#[test]
fn no_game_name_is_a_clear_reserved_word_or_alias() {
    let checker = crate::gate::checker();
    let mut names: Vec<String> = Vec::new();
    for r in rooms::ROOMS {
        names.push(r.name.into());
        names.push(r.key.into());
        for (exit, _) in r.exits {
            names.push((*exit).into());
        }
    }
    for c in combat::CREATURES {
        names.push(c.name.into());
    }
    let w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    for spec in w.registry.specs() {
        names.push(spec.verb().into());
    }
    let bad: Vec<String> = names
        .iter()
        .filter_map(|n| checker.collision(n).map(|c| format!("{n}: {c}")))
        .collect();
    assert!(bad.is_empty(), "game names collide with CLEAR: {bad:?}");
    assert_eq!(
        checker.collision("The Gate").as_deref(),
        Some("the gate (an alias of CLEAR gate)")
    );
    assert_eq!(checker.collision("FACT room").as_deref(), Some("FACT"));
}

#[test]
fn the_world_moves_on_its_own() {
    let dir = tempfile::tempdir().unwrap();
    let t0 = Timestamp(1_000_000);
    let mut w = open(dir.path(), t0);
    let f = login(&mut w, "dave", "test", PASS).unwrap();
    let narrative = |w: &World, after: u64| -> Vec<String> {
        w.log
            .read_all(&w.id, foxing_log::Cursor { after })
            .unwrap()
            .iter()
            .filter(|e| e.kind == combat::NARRATIVE && e.payload["to"] == "dave")
            .map(|e| e.payload["text"].as_str().unwrap().to_string())
            .collect()
    };

    // Rooms filled on the boot tick without anyone looking.
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    assert_eq!(combat::creatures_in(&w, fog).len(), 2);

    // Kill a wisp: the room refills on its respawn job, not on a look.
    ok(&mut w, "dave", "travel", json!(["east"]));
    let out = fight(&mut w, "dave");
    assert_eq!(out.result["killed"], json!(true), "{}", out.summary);
    assert_eq!(combat::creatures_in(&w, fog).len(), 1);
    // The respawn is 60 s, give or take: never before 36, always by 96.
    w.fixed_now = Some(t0.plus(Duration::seconds(30)));
    assert_eq!(w.tick(), 0, "respawn is about 60 s");
    w.fixed_now = Some(t0.plus(Duration::seconds(97)));
    // The respawn, the Docent's first round (every ninety seconds), and the
    // creatures' mending pass (every minute).
    assert_eq!(w.tick(), 3);
    assert!(
        combat::creatures_in(&w, fog).len() >= 2,
        "the fog refilled on its own (one refill in six brings an extra)"
    );

    // Engage a wisp and wait: it strikes on its own every 12 s or so (six to
    // eighteen) and says so.
    let head = events(&w);
    let round = ok(&mut w, "dave", "attack", json!(["wisp"]));
    assert_eq!(round.result["killed"], json!(false), "{}", round.summary);
    let hp_after_round = combat::hp(&w, f.citizen);
    w.fixed_now = Some(t0.plus(Duration::seconds(97 + 19)));
    assert_eq!(w.tick(), 1, "one strike job due");
    assert!(
        combat::hp(&w, f.citizen) < hp_after_round + 1.9,
        "the wisp struck (healing is 0.1 per second)"
    );
    let lines = narrative(&w, head);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].starts_with("The wisp hits you for"), "{lines:?}");
    assert!(
        w.jobs
            .has(combat::JOB_STRIKE, |p| p["citizen"] == f.citizen.0),
        "the next strike is scheduled"
    );

    // Retreat ends the chain.
    ok(&mut w, "dave", "retreat", json!([]));
    assert!(!w
        .jobs
        .has(combat::JOB_STRIKE, |p| p["citizen"] == f.citizen.0));
    w.fixed_now = Some(t0.plus(Duration::seconds(200)));
    // Only the creatures' mending pass comes due; no strike.
    assert_eq!(w.tick(), 1);
    assert!(!w
        .jobs
        .has(combat::JOB_STRIKE, |p| p["citizen"] == f.citizen.0));

    // Walk into the Shallows: a hollow ambushes after a few seconds (a name in its
    // first hour would be spared; dave's is older).
    super::grown(&mut w, "dave");
    ok(&mut w, "dave", "travel", json!(["north"]));
    assert!(w
        .jobs
        .has(combat::JOB_AMBUSH, |p| p["citizen"] == f.citizen.0));
    let head = events(&w);
    w.fixed_now = Some(t0.plus(Duration::seconds(209)));
    assert!(w.tick() >= 1);
    assert!(
        combat::engaged_with(&w, f.citizen).is_some(),
        "ambushed and engaged"
    );
    let lines = narrative(&w, head);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("The hollow stands up out of the water")),
        "{lines:?}"
    );
    // Only one hollow jumped him; the other ambush job was dropped because he is engaged.
    assert_eq!(lines.len(), 1, "{lines:?}");

    // A frozen world holds everything, including the hollow's next strike.
    admin(&mut w, "dave", "freeze", json!([]));
    w.fixed_now = Some(t0.plus(Duration::seconds(400)));
    assert_eq!(w.tick(), 0);
    admin(&mut w, "dave", "thaw", json!([]));
    assert!(w.tick() >= 1);

    // Everything above survives a restart: the pending strike is still pending.
    let pending_before: Vec<String> = w.jobs.pending().map(|j| j.kind.clone()).collect();
    drop(w);
    let w = open(dir.path(), t0.plus(Duration::seconds(401)));
    let pending_after: Vec<String> = w.jobs.pending().map(|j| j.kind.clone()).collect();
    assert_eq!(pending_before, pending_after);
}

#[test]
fn logging_in_where_something_hunts_is_arriving() {
    let dir = tempfile::tempdir().unwrap();
    let t0 = Timestamp(5_000_000);
    let mut w = open(dir.path(), t0);
    let f = login(&mut w, "dave", "test", PASS).unwrap();
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["north"]));
    // Cancel the arrival ambush by hand, as if the driver had been down.
    w.cancel(combat::JOB_AMBUSH, |_| true).unwrap();
    assert!(!w.jobs.has(combat::JOB_AMBUSH, |_| true));
    login(&mut w, "dave", "test", PASS).unwrap();
    assert!(
        w.jobs
            .has(combat::JOB_AMBUSH, |p| p["citizen"] == f.citizen.0),
        "login in the Shallows schedules an ambush"
    );
    // A restart with dave standing there does the same, and gives old creatures sprites.
    drop(w);
    let w = open(dir.path(), t0.plus(Duration::seconds(1)));
    assert!(w
        .jobs
        .has(combat::JOB_AMBUSH, |p| p["citizen"] == f.citizen.0));
    let shallows = rooms::room_by_key(&w, "shallows").unwrap();
    for (id, _, _, _) in combat::creatures_in(&w, shallows) {
        assert_eq!(plain_str(&w, id, SPRITE, "frame"), Some("hollow"));
    }
    assert_eq!(plain_str(&w, f.citizen, SPRITE, "frame"), Some("citizen"));
}

#[test]
fn sprites_from_before_frames_are_upgraded() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = open(dir.path(), Timestamp(0));
    let f = login(&mut w, "dave", "test", PASS).unwrap();
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    // Strip the frame from a wisp and from dave, as an older log would have them.
    for id in [wisp, f.citizen] {
        let mut old = std::collections::BTreeMap::new();
        old.insert("glyph".to_string(), foxing_store::Field::Plain(json!("x")));
        old.insert(
            "tint".to_string(),
            foxing_store::Field::Plain(json!("#fff")),
        );
        w.record(
            foxing_log::Actor::Driver,
            foxing_store::StoreEvent::ComponentSet {
                id,
                component: SPRITE.into(),
                fields: old,
            },
        )
        .unwrap();
    }
    assert!(plain_str(&w, wisp, SPRITE, "frame").is_none());
    drop(w);
    let mut w = open(dir.path(), Timestamp(1000));
    assert_eq!(
        plain_str(&w, wisp, SPRITE, "frame"),
        Some("wisp"),
        "boot upgrades creatures"
    );
    assert!(
        plain_str(&w, f.citizen, SPRITE, "frame").is_none(),
        "citizens wait for their login"
    );
    login(&mut w, "dave", "test", PASS).unwrap();
    assert_eq!(plain_str(&w, f.citizen, SPRITE, "frame"), Some("citizen"));
}

#[test]
fn a_misshapen_query_is_the_asker_s_mistake_and_says_what_was_expected() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    let q = r#"{"root":"here","relations":{"contents":{"where":{"disremembered.creature.hp":{"lt":8}}}}}"#;
    let e = call(&mut w, "dave", ActorKind::Citizen, "observe", json!([q])).unwrap_err();
    let text = e.to_string();
    assert!(
        matches!(e, CommandError::Precondition { .. }),
        "a bad shape is a precondition, not an internal error: {text}"
    );
    assert!(text.contains("\"op\""), "{text}");
    assert!(!text.contains("built-in"), "{text}");
}

#[test]
fn a_log_from_other_modules_is_refused_with_directions() {
    use foxing_log::{EventLog, FileLog, NewEvent};
    let dir = tempfile::tempdir().unwrap();
    let id = foxing_log::WorldId("w".into());
    // A world whose first event creates an entity with a component this driver never had.
    {
        let mut log = FileLog::open(dir.path()).unwrap();
        log.append(
            &id,
            NewEvent {
                at: Timestamp(0),
                kind: "store.component_set".into(),
                actor: foxing_log::Actor::Driver,
                cause: None,
                payload: json!({"op": "component_set", "id": 1, "component": "entropic.room",
                    "fields": {"name": {"kind": "plain", "value": "x"}}}),
            },
        )
        .unwrap();
    }
    let err = World::open(
        id,
        foxing_store::Store::new(super::registry()),
        crate::world::LogBacking::File(dir.path().into()),
    )
    .err()
    .expect("a foreign log must not open");
    assert!(err.contains("entropic.room"), "{err}");
    assert!(err.contains("MOONDIAL_WORLD"), "{err}");
}

/// The vocabulary's verb signatures are the registry's: every registered command is an
/// approved verb whose parameters (name, type, required, in order) the vocabulary
/// states exactly, so the checker in the browser refuses what the registry would.
#[test]
fn the_vocabulary_s_signatures_are_the_registry_s() {
    let w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    let vocab = crate::gate::disremembered_vocabulary();
    let verbs = vocab["approved_verbs"]
        .as_object()
        .expect("approved_verbs is a map of verb to signature");
    for spec in w.registry.specs() {
        let verb = spec.verb();
        let sig = verbs
            .get(verb)
            .unwrap_or_else(|| panic!("{verb} is registered but not an approved verb"));
        let params = sig["params"].as_array().unwrap_or_else(|| {
            panic!("{verb} is registered but the vocabulary gives it no signature")
        });
        let expected: Vec<Value> = spec
            .params
            .iter()
            .map(|p| {
                let mut v = json!({"name": p.name, "required": p.required});
                let ty = serde_json::to_value(&p.ty).unwrap();
                v["type"] = ty["type"].clone();
                if let Some(values) = ty.get("values") {
                    v["values"] = values.clone();
                }
                v
            })
            .collect();
        let got: Vec<Value> = params
            .iter()
            .map(|p| {
                let mut v =
                    json!({"name": p["name"], "required": p["required"], "type": p["type"]});
                if let Some(values) = p.get("values") {
                    v["values"] = values.clone();
                }
                v
            })
            .collect();
        assert_eq!(
            got, expected,
            "signature of {verb} in clear/vocab/disremembered.yaml"
        );
    }
}

/// Players extend the world: a dug room costs a chunk, inherits its half-life, fades
/// toward a half-remembered line unless visited, keeps its name and exits regardless,
/// is the digger's to describe and name, and theirs to share by permit.
#[test]
fn players_dig_rooms_that_fade_but_stay() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // A vivid chunk: a long half-life, which the room will inherit.
    let out = admin(&mut w, "dave", "earn", json!(["hydra", 100, 1.0]));
    let chunk = format!("chunk-{}", out.result["chunk"].as_u64().unwrap());

    // No name may carry a reserved word or a glossary alias, and the chunk is spent.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "dig",
        json!(["down", chunk, "The Gate"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("reserves"), "{err}");
    let out = ok(
        &mut w,
        "dave",
        "dig",
        json!(["down", chunk, "The Salt Stair"]),
    );
    assert!(
        out.summary
            .starts_with("You dig down with the memory of hydra."),
        "{}",
        out.summary
    );
    let room = EntityId(out.result["room"].as_u64().unwrap());
    assert_eq!(out.result["back"], "up");
    assert_eq!(out.result["key"], format!("salt-stair-{}", room.0));
    assert!(
        w.store
            .entity(EntityId(
                chunk.trim_start_matches("chunk-").parse().unwrap()
            ))
            .is_none(),
        "the chunk is spent"
    );
    let hl = out.result["half_life_ms"].as_i64().unwrap();
    assert_eq!(
        hl,
        3 * 10 * 1000,
        "a vividity-1 chunk lasts 3x the test world's 10 s half-life"
    );

    // One dig from a seed room per citizen; the digger's own rooms are open to them.
    let out2 = admin(&mut w, "dave", "earn", json!(["wisp", 10, 0.0]));
    let chunk2 = format!("chunk-{}", out2.result["chunk"].as_u64().unwrap());
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "dig",
        json!(["under", chunk2]),
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("already dug from this room"),
        "{err}"
    );
    ok(&mut w, "dave", "travel", json!(["down"]));
    let out3 = ok(&mut w, "dave", "dig", json!(["east", chunk2]));
    assert_eq!(out3.result["name"], "An Unnamed Digging");

    // Describing and naming are the owner's; a visitor is refused until permitted.
    ok(
        &mut w,
        "dave",
        "describe",
        json!(["Steps of salt, each one damp. The air tastes of the sea that is not here."]),
    );
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary
            .starts_with("The Salt Stair. Steps of salt, each one damp."),
        "{}",
        look.summary
    );
    assert!(
        look.summary
            .contains("Dug by dave with the memory of hydra."),
        "{}",
        look.summary
    );
    ok(&mut w, "tyler", "travel", json!(["down"]));
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "describe",
        json!(["Mine now."]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("permission"), "{err}");
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "permit",
        json!(["dave"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("owner"), "{err}");
    ok(&mut w, "dave", "permit", json!(["tyler", 1]));
    ok(&mut w, "tyler", "name", json!(["The Salt Stair, Swept"]));
    assert_eq!(
        ok(&mut w, "tyler", "look", json!([])).result["room"],
        "The Salt Stair, Swept"
    );
    // The permit expires.
    w.fixed_now = Some(w.now().plus(Duration::seconds(3601)));
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "describe",
        json!(["Late."]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("permission"), "{err}");
    ok(&mut w, "dave", "permit", json!(["tyler"]));
    ok(
        &mut w,
        "tyler",
        "describe",
        json!(["Steps of salt. Swept now, by a friend."]),
    );
    ok(&mut w, "dave", "revoke", json!(["tyler"]));
    assert!(call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "describe",
        json!(["Again."])
    )
    .is_err());
    // A welcome is a guest book: tyler may add a line, not rewrite or rename.
    ok(&mut w, "dave", "welcome", json!(["tyler"]));
    for (verb, arg) in [("describe", "Mine."), ("name", "Tyler's Stair")] {
        let err = call(&mut w, "tyler", ActorKind::Citizen, verb, json!([arg])).unwrap_err();
        assert!(err.to_string().contains("permission"), "{verb}: {err}");
    }
    let out = ok(&mut w, "tyler", "append", json!(["Tyler was here."]));
    assert!(out.result["description"]
        .as_str()
        .unwrap()
        .ends_with("Tyler was here."));
    ok(&mut w, "dave", "revoke", json!(["tyler"]));
    assert!(call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "append",
        json!(["Again."])
    )
    .is_err());
    // Set it back to what the fade checks below expect.
    ok(
        &mut w,
        "dave",
        "describe",
        json!(["Steps of salt. Swept now, by a friend."]),
    );

    // The seed rooms are the world's.
    ok(&mut w, "dave", "travel", json!(["up"]));
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "describe",
        json!(["Mine."]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("the world's"), "{err}");

    // Memory fades with nobody there: the description dims, then fades; the room stays.
    ok(&mut w, "tyler", "travel", json!(["up"]));
    let full = describe_room(&mut w, "dave", room);
    assert!(
        full.starts_with("The Salt Stair, Swept. Steps of salt. Swept now, by a friend."),
        "{full}"
    );
    w.fixed_now = Some(w.now().plus(Duration(hl * 2)));
    let dim = describe_room(&mut w, "dave", room);
    assert!(
        dim.contains("Steps of salt. The rest is hard to recall."),
        "{dim}"
    );
    w.fixed_now = Some(w.now().plus(Duration(hl * 3)));
    let faded = describe_room(&mut w, "dave", room);
    assert!(faded.contains(building::HALF_REMEMBERED), "{faded}");
    assert!(
        faded.contains("Exits: up (Threshold), east ("),
        "the exits survive: {faded}"
    );
    // A visit remembers it whole again.
    ok(&mut w, "dave", "travel", json!(["down"]));
    let back = ok(&mut w, "dave", "look", json!([]));
    assert!(
        back.summary
            .contains("Steps of salt. Swept now, by a friend."),
        "{}",
        back.summary
    );
    assert!(building::memory(&w, room).unwrap() > 0.99);
}

fn describe_room(w: &mut World, actor: &str, room: EntityId) -> String {
    let ctx = CallContext {
        actor: actor.into(),
        kind: ActorKind::Citizen,
    };
    rooms::describe(w, actor, &ctx, room).unwrap().summary
}

#[test]
fn append_adds_to_a_description_and_arrivals_are_counted() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "terminal", PASS).unwrap();
    let out = admin(&mut w, "dave", "earn", json!(["hydra", 100, 1.0]));
    let chunk = format!("chunk-{}", out.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "dave",
        "dig",
        json!(["down", chunk, "The Salt Stair"]),
    );
    ok(&mut w, "dave", "travel", json!(["down"]));
    // Appending to a fresh digging drops the placeholder; then sentences join with a space.
    let a = ok(&mut w, "dave", "append", json!(["Steps of salt"]));
    assert_eq!(a.result["description"], "Steps of salt");
    let b = ok(&mut w, "dave", "append", json!(["Each one damp."]));
    assert_eq!(b.result["description"], "Steps of salt. Each one damp.");
    let c = ok(&mut w, "dave", "append", json!(["The air tastes of salt."]));
    assert_eq!(
        c.result["description"],
        "Steps of salt. Each one damp. The air tastes of salt."
    );
    ok(&mut w, "dave", "describe", json!(["Afresh."]));
    assert_eq!(
        ok(&mut w, "dave", "append", json!(["And more."])).result["description"],
        "Afresh. And more."
    );
    let long = "x".repeat(800);
    assert!(call(&mut w, "dave", ActorKind::Citizen, "append", json!([long])).is_err());

    // Arrivals: the first login, then logins after an absence; not every request.
    login(&mut w, "dave", "http", PASS).unwrap();
    login(&mut w, "dave", "http", PASS).unwrap();
    w.fixed_now = Some(w.now().plus(Duration::seconds(11 * 60)));
    login(&mut w, "dave", "gate", PASS).unwrap();
    let arrivals: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == crate::world::ARRIVED)
        .collect();
    assert_eq!(arrivals.len(), 2, "{arrivals:?}");
    assert_eq!(arrivals[0].payload["origin"], "terminal");
    assert_eq!(arrivals[0].payload["fresh"], true);
    assert_eq!(arrivals[1].payload["origin"], "gate");
    assert!(arrivals[1].payload["away_ms"].as_i64().unwrap() >= 11 * 60 * 1000);
    let notices = w.take_notices();
    assert_eq!(notices.len(), 2, "{notices:?}");
    assert!(notices[0].contains("arrived for the first time, via terminal"));
    assert!(notices[1].contains("is back, via gate"));
    let report = crate::net::citizens_report(&w, 0);
    let dave = report.iter().find(|r| r["name"] == "dave").unwrap();
    assert_eq!(dave["arrivals"], 2);
    assert_eq!(dave["rooms_dug"], 1);
    assert_eq!(dave["origins"]["gate"], 1);
}

#[test]
fn five_wrong_passphrases_lock_a_name_for_five_minutes() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    for _ in 0..5 {
        let Err(e) = login(&mut w, "dave", "test", "not-the-passphrase") else {
            panic!("a wrong passphrase got in")
        };
        assert!(matches!(e, LoginError::WrongPassphrase { .. }), "{e}");
    }
    // The right passphrase is refused too, until the lock lifts: no oracle for guessing.
    let Err(e) = login(&mut w, "dave", "test", PASS) else {
        panic!("the lock did not hold")
    };
    assert!(matches!(e, LoginError::Locked { minutes: 5, .. }), "{e}");
    w.fixed_now = Some(w.now().plus(Duration::seconds(5 * 60 + 1)));
    login(&mut w, "dave", "test", PASS).unwrap();
    // Success resets the count.
    assert!(login(&mut w, "dave", "test", "wrong-again-here").is_err());
    login(&mut w, "dave", "test", PASS).unwrap();
}

#[test]
fn the_public_log_hides_passphrase_hashes_and_owner_fields() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    admin(&mut w, "dave", "earn", json!(["hydra", 100, 0.5]));
    let now = w.now();
    let all = w.log.read_all(&w.id, foxing_log::Cursor::START).unwrap();
    for viewer in [
        Viewer::anonymous(),
        Viewer::actor("tyler"),
        Viewer::actor("dave"),
    ] {
        let seen: Vec<_> = all
            .iter()
            .filter(|e| crate::observe::visible(&w, &viewer, now, e))
            .map(|e| crate::observe::redacted(&w, &viewer, e.clone()))
            .collect();
        let text = serde_json::to_string(&seen).unwrap();
        assert!(
            !text.contains("passphrase_hash"),
            "{:?} sees a hash",
            viewer.actor
        );
        assert!(!text.contains("$argon2"), "{:?} sees a hash", viewer.actor);
        let chunk_values = seen
            .iter()
            .filter(|e| e.kind == "store.component_set" && e.payload["component"] == CHUNK)
            .filter(|e| e.payload["fields"].get("value").is_some())
            .count();
        if viewer.actor.as_deref() == Some("dave") {
            assert!(chunk_values > 0, "the owner sees their chunk's value");
        } else {
            assert_eq!(
                chunk_values, 0,
                "{:?} sees an owner-only field",
                viewer.actor
            );
        }
    }
    // The operator sees everything.
    let whole: Vec<_> = all
        .iter()
        .map(|e| crate::observe::redacted(&w, &Viewer::admin(), e.clone()))
        .collect();
    assert!(serde_json::to_string(&whole)
        .unwrap()
        .contains("passphrase_hash"));
}

#[test]
fn an_admin_can_forget_a_passphrase_and_the_next_login_claims_it() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    assert!(login(&mut w, "dave", "test", "a-different-one").is_err());
    let out = admin(&mut w, "tyler", "forget", json!(["dave"]));
    assert_eq!(out.result["reset"], true);
    // Too short is still refused; a good one claims the name.
    assert!(login(&mut w, "dave", "test", "short").is_err());
    login(&mut w, "dave", "test", "a-different-one").unwrap();
    assert!(login(&mut w, "dave", "test", PASS).is_err());
    // Citizens cannot reset anyone.
    let ctx = CallContext {
        actor: "dave".into(),
        kind: ActorKind::Citizen,
    };
    assert!(w.call(&ctx, "forget", &json!(["tyler"])).is_err());
}

#[test]
fn the_map_is_connected_and_new_seed_exits_reach_old_rooms() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = open(dir.path(), Timestamp(0));
    // Every room reachable from the Threshold, and every exit lands on a seeded room.
    let mut seen = std::collections::BTreeSet::new();
    let mut stack = vec![rooms::ARRIVAL];
    while let Some(k) = stack.pop() {
        if !seen.insert(k) {
            continue;
        }
        let def = rooms::ROOMS.iter().find(|r| r.key == k).unwrap();
        for (_, to) in def.exits {
            assert!(
                rooms::ROOMS.iter().any(|r| r.key == *to),
                "{k} exits to unknown {to}"
            );
            stack.push(to);
        }
    }
    assert_eq!(
        seen.len(),
        rooms::ROOMS.len(),
        "unreachable rooms: {:?}",
        rooms::ROOMS
            .iter()
            .map(|r| r.key)
            .filter(|k| !seen.contains(k))
            .collect::<Vec<_>>()
    );
    // Simulate an old log: strip the fogbank's south exit, then reboot; it comes back,
    // and a dug exit beside it survives.
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let mut exits = rooms::exits_of(&w, fog);
    exits.remove("south");
    let mut m = serde_json::Map::new();
    for (k, v) in &exits {
        m.insert(k.clone(), json!(v.0));
    }
    m.insert(
        "hole".into(),
        json!(rooms::room_by_key(&w, "ossuary").unwrap().0),
    );
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        "exits".to_string(),
        foxing_store::Field::Plain(Value::Object(m)),
    );
    w.record(
        Actor::Driver,
        StoreEvent::ComponentSet {
            id: fog,
            component: EXITS.into(),
            fields,
        },
    )
    .unwrap();
    assert!(!rooms::exits_of(&w, fog).contains_key("south"));
    drop(w);
    let w = open(dir.path(), Timestamp(1));
    let exits = rooms::exits_of(&w, fog);
    assert_eq!(
        exits.get("south"),
        Some(&rooms::room_by_key(&w, "midden").unwrap())
    );
    assert!(exits.contains_key("hole"), "a dug exit was lost");
    // A dug room sitting where a seed exit goes keeps its door; the operator hears.
    let mut w = w;
    let thr = rooms::room_by_key(&w, rooms::ARRIVAL).unwrap();
    let mut m = serde_json::Map::new();
    for (k, v) in rooms::exits_of(&w, thr) {
        m.insert(k, json!(v.0));
    }
    let dug = rooms::room_by_key(&w, "midden").unwrap();
    m.insert("north".into(), json!(dug.0));
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        "exits".to_string(),
        foxing_store::Field::Plain(Value::Object(m)),
    );
    w.record(
        Actor::Driver,
        StoreEvent::ComponentSet {
            id: thr,
            component: EXITS.into(),
            fields,
        },
    )
    .unwrap();
    drop(w);
    let mut w = open(dir.path(), Timestamp(2));
    assert_eq!(rooms::exits_of(&w, thr).get("north"), Some(&dug));
    let notices = w.take_notices();
    assert!(notices.iter().any(|n| n.contains("`north`")), "{notices:?}");
    // Every spawn names a creature that exists.
    for r in rooms::ROOMS {
        if let Some((c, _, _)) = r.spawn {
            assert!(
                combat::CREATURES.iter().any(|d| d.key == c),
                "{}: no creature {c}",
                r.key
            );
        }
    }
}

#[test]
fn remembering_a_room_costs_a_chunk_and_makes_it_last() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    let out = admin(&mut w, "dave", "earn", json!(["hydra", 100, 1.0]));
    let chunk = format!("chunk-{}", out.result["chunk"].as_u64().unwrap());
    let dug = ok(
        &mut w,
        "dave",
        "dig",
        json!(["down", chunk, "The Salt Stair"]),
    );
    let room = EntityId(dug.result["room"].as_u64().unwrap());
    let hl0 = dug.result["half_life_ms"].as_i64().unwrap();

    // The world's rooms remember themselves.
    let out = admin(&mut w, "tyler", "earn", json!(["wisp", 10, 0.0]));
    let tc = format!("chunk-{}", out.result["chunk"].as_u64().unwrap());
    let err = call(&mut w, "tyler", ActorKind::Citizen, "remember", json!([tc])).unwrap_err();
    assert!(err.to_string().contains("the world's"), "{err}");

    // Tyler keeps Dave's room: his chunk is gone, the half-life grew by its own.
    ok(&mut w, "tyler", "travel", json!(["down"]));
    let level_before = level(&mut w, "tyler");
    let out = ok(&mut w, "tyler", "remember", json!([tc]));
    assert!(
        out.summary
            .starts_with("You give up the memory of wisp, and The Salt Stair keeps it."),
        "{}",
        out.summary
    );
    let hl1 = out.result["half_life_ms"].as_i64().unwrap();
    assert_eq!(hl1, hl0 + out.result["added_ms"].as_i64().unwrap());
    assert!(hl1 > hl0);
    assert!(w
        .store
        .entity(EntityId(tc.trim_start_matches("chunk-").parse().unwrap()))
        .is_none());
    assert!(
        level(&mut w, "tyler") < level_before,
        "the chunk was the cost"
    );
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary.contains("Kept in memory by tyler (wisp)."),
        "{}",
        look.summary
    );

    // The record is public: an anonymous reader of the log sees who paid.
    let now = w.now();
    let seen: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == building::REMEMBERED)
        .filter(|e| crate::observe::visible(&w, &Viewer::anonymous(), now, e))
        .collect();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].payload["by"], "tyler");
    assert_eq!(seen[0].payload["owner"], "dave");
    assert_eq!(seen[0].payload["room"], room.0);

    // Chunks must be yours and in your window or palace.
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "remember",
        json!([chunk]),
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("not yours") || err.to_string().contains("no chunk"),
        "{err}"
    );
}

#[test]
fn a_quiet_citizen_is_disremembered_by_the_room_and_returns_as_an_arrival() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Dave walks into the Shallows, where hollows hunt, and stops typing.
    super::grown(&mut w, "dave");
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["north"]));
    let shallows = rooms::room_by_key(&w, "shallows").unwrap();
    assert!(presence::is_here(
        &w,
        find_citizen(&w, "dave").unwrap().citizen
    ));
    // Tyler, active, sees him.
    super::grown(&mut w, "tyler");
    ok(&mut w, "tyler", "travel", json!(["east"]));
    ok(&mut w, "tyler", "travel", json!(["north"]));
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary.contains("Also here: dave."),
        "{}",
        look.summary
    );

    // Ten minutes of silence: the sweep marks him gone, and the fight that had started ends.
    w.fixed_now = Some(t0.plus(Duration::seconds(combat::AMBUSH_SECONDS_HIGH)));
    w.tick(); // the ambush lands on both; dave is engaged
    let dave = find_citizen(&w, "dave").unwrap().citizen;
    assert!(
        combat::engaged_with(&w, dave).is_some(),
        "the hollow should have ambushed dave"
    );
    ok(&mut w, "tyler", "retreat", json!([]));
    w.fixed_now = Some(t0.plus(Duration::seconds(11 * 60)));
    ok(&mut w, "tyler", "look", json!([])); // tyler stays active
    let hp_before = w
        .store
        .read_field(dave, BODY, "hp", w.now())
        .unwrap()
        .as_f64()
        .unwrap();
    w.tick();
    assert!(!presence::is_here(&w, dave), "dave went quiet");
    assert!(
        combat::engaged_with(&w, dave).is_none(),
        "the fight ended without a blow"
    );
    let departed: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == presence::DEPARTED)
        .collect();
    assert_eq!(departed.len(), 1);
    assert_eq!(departed[0].payload["room"], shallows.0);
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary
            .contains("No other citizen is here. Remembered here: dave."),
        "{}",
        look.summary
    );
    assert_eq!(look.result["citizens"], json!([]));
    assert_eq!(look.result["remembered"][0]["name"], "dave");

    // Creatures leave the gone alone: strikes and ambushes skip him for an hour.
    for m in 12..70 {
        w.fixed_now = Some(t0.plus(Duration::seconds(m * 60)));
        ok(&mut w, "tyler", "look", json!([]));
        w.tick();
    }
    let hp_after = w
        .store
        .read_field(dave, BODY, "hp", w.now())
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(
        hp_after >= hp_before,
        "a gone citizen is not struck: {hp_before} -> {hp_after}"
    );

    // The trace fades on the active half-life (10 s in tests): after an hour he is not listed.
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        !look.summary.contains("Remembered here"),
        "{}",
        look.summary
    );

    // Dave types again: an arrival, here, and the hollows notice.
    let arrivals_before = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .iter()
        .filter(|e| e.kind == crate::world::ARRIVED && e.payload["actor"] == "dave")
        .count();
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains("Also here: tyler."),
        "{}",
        look.summary
    );
    assert!(presence::is_here(&w, dave));
    let arrivals_after = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .iter()
        .filter(|e| e.kind == crate::world::ARRIVED && e.payload["actor"] == "dave")
        .count();
    assert_eq!(arrivals_after, arrivals_before + 1);
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary.contains("Also here: dave."),
        "{}",
        look.summary
    );
    w.fixed_now = Some(t0.plus(Duration::seconds(70 * 60 + 5)));
    w.tick();
    assert!(
        combat::engaged_with(&w, dave).is_some(),
        "back in the Shallows, the hollow comes for him"
    );
}

#[test]
fn a_name_unspoken_for_thirty_days_is_forgotten_and_free() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    let c = admin(&mut w, "dave", "earn", json!(["wisp", 30, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "dave", "dig", json!(["cellar", c, "Dave's Cellar"]));
    let dave = find_citizen(&w, "dave").unwrap().citizen;
    let day = 86_400;
    // Dave goes quiet; tyler keeps the world ticking.
    w.fixed_now = Some(t0.plus(Duration::seconds(11 * 60)));
    ok(&mut w, "tyler", "look", json!([]));
    w.tick();
    assert!(!presence::is_here(&w, dave));
    let f = presence::faint(&w, dave).unwrap();
    assert!(f > 0.99, "{f}");
    // Fifteen days: faint, not forgotten; a return is told how close it was.
    w.fixed_now = Some(t0.plus(Duration::seconds(15 * day)));
    ok(&mut w, "tyler", "look", json!([]));
    w.tick();
    let f = presence::faint(&w, dave).unwrap();
    assert!((0.3..0.4).contains(&f), "{f}");
    assert!(find_citizen(&w, "dave").is_some());
    ok(&mut w, "dave", "look", json!([]));
    let heard: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == combat::NARRATIVE && e.payload["to"] == "dave")
        .filter_map(|e| e.payload["text"].as_str().map(str::to_string))
        .collect();
    assert!(
        heard.iter().any(|t| t.contains("nearly forgotten")),
        "{heard:?}"
    );
    assert!(
        presence::faint(&w, dave).is_none(),
        "here again: not fading"
    );
    // Then thirty-one days of silence: forgotten, publicly, and the name is free.
    w.fixed_now = Some(t0.plus(Duration::seconds(15 * day + 11 * 60)));
    ok(&mut w, "tyler", "look", json!([]));
    w.tick();
    w.fixed_now = Some(t0.plus(Duration::seconds(46 * day + 11 * 60)));
    ok(&mut w, "tyler", "look", json!([]));
    w.tick();
    assert!(find_citizen(&w, "dave").is_none(), "dave is forgotten");
    assert!(w.store.entity(dave).is_none());
    let forgotten: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == presence::FORGOTTEN_NAME)
        .collect();
    assert_eq!(forgotten.len(), 1);
    assert_eq!(forgotten[0].payload["name"], "dave");
    assert!(forgotten[0].payload["quiet_ms"].as_i64().unwrap() >= 31 * day * 1000);
    // The cellar stays, dug by dave.
    let cellar = rooms::room_by_key(&w, "cellar").or_else(|| {
        w.store
            .entities()
            .find(|e| e.kind == "room" && e.owner.as_deref() == Some("dave"))
            .map(|e| e.id)
    });
    assert!(cellar.is_some(), "what dave dug stays");
    // Tyler is still here, and a new dave arrives fresh.
    assert!(find_citizen(&w, "tyler").is_some());
    let f = login(&mut w, "dave", "test", "another-pass-9").unwrap();
    assert_ne!(f.citizen, dave);
    let fresh = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .rfind(|e| e.kind == crate::world::ARRIVED && e.payload["actor"] == "dave")
        .unwrap();
    assert_eq!(fresh.payload["fresh"], true);
}

#[test]
fn a_passing_name_keeps_nothing_and_is_forgotten_within_the_hour_or_kept() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    let f = login_as(&mut w, "Tallowmoth", "test", PASS, true).unwrap();
    assert!(is_passing(&w, f.citizen));
    // Told so on arrival; listed as passing; a kept name's login ignores the flag.
    let heard: Vec<String> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == combat::NARRATIVE && e.payload["to"] == "Tallowmoth")
        .filter_map(|e| e.payload["text"].as_str().map(str::to_string))
        .collect();
    assert!(
        heard.iter().any(|t| t.contains("a passing name")),
        "{heard:?}"
    );
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary.contains("Tallowmoth (passing)"),
        "{}",
        look.summary
    );
    let again = login_as(&mut w, "tyler", "test", PASS, true).unwrap();
    assert!(!is_passing(&w, again.citizen));
    // It may speak, travel, feed and fight; it may not dig, permit, shelve, pause, or
    // remember into a room.
    ok(&mut w, "Tallowmoth", "say", json!(["passing through"]));
    ok(&mut w, "Tallowmoth", "travel", json!(["east"]));
    let c = admin(&mut w, "Tallowmoth", "earn", json!(["wisp", 30, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    for (verb, args) in [
        ("dig", json!(["hollow", &c, "A Passing Place"])),
        ("permit", json!(["tyler"])),
        ("welcome", json!(["tyler"])),
        ("revoke", json!(["tyler"])),
        ("shelve", json!([&c])),
        ("pto", json!([])),
        ("remember", json!([&c])),
    ] {
        let err = call(&mut w, "Tallowmoth", ActorKind::Citizen, verb, args).unwrap_err();
        assert!(err.to_string().contains("passing name"), "{verb}: {err}");
    }
    ok(&mut w, "Tallowmoth", "remember", json!([&c, "wisp"]));
    // Quiet for an hour and ten minutes: forgotten, and the name is free.
    let p = find_citizen(&w, "Tallowmoth").unwrap().citizen;
    w.fixed_now = Some(t0.plus(Duration::seconds(11 * 60)));
    ok(&mut w, "tyler", "look", json!([]));
    w.tick();
    assert!(!presence::is_here(&w, p));
    w.fixed_now = Some(t0.plus(Duration::seconds(11 * 60 + 3599)));
    ok(&mut w, "tyler", "look", json!([]));
    w.tick();
    assert!(find_citizen(&w, "Tallowmoth").is_some(), "not yet");
    w.fixed_now = Some(t0.plus(Duration::seconds(11 * 60 + 3601)));
    ok(&mut w, "tyler", "look", json!([]));
    w.tick();
    assert!(
        find_citizen(&w, "Tallowmoth").is_none(),
        "forgotten within the hour"
    );
    // Another passing name leaves, and is forgotten at once; a third keeps itself.
    let g = login_as(&mut w, "Gossamer", "test", PASS, true).unwrap();
    let out = ok(&mut w, "Gossamer", "leave", json!([]));
    assert!(
        out.summary.contains("forgets the name Gossamer"),
        "{}",
        out.summary
    );
    assert!(w.store.entity(g.citizen).is_none());
    let k = login_as(&mut w, "Wickwright", "test", PASS, true).unwrap();
    assert!(
        call(&mut w, "tyler", ActorKind::Citizen, "keep", json!([])).is_err(),
        "kept already"
    );
    let out = ok(&mut w, "Wickwright", "keep", json!([]));
    assert!(out.summary.contains("is kept"), "{}", out.summary);
    assert!(!is_passing(&w, k.citizen));
    w.fixed_now = Some(t0.plus(Duration::seconds(11 * 60 + 3601 + 2 * 3600)));
    ok(&mut w, "tyler", "look", json!([]));
    w.tick();
    assert!(
        find_citizen(&w, "Wickwright").is_some(),
        "a kept name outlives the hour"
    );
    assert!(
        login_as(&mut w, "Wickwright", "test", "wrong-pass-9", true).is_err(),
        "the passphrase stays"
    );
}

#[test]
fn a_standing_greeting_meets_whoever_arrives_and_the_room_says_who_is_at_a_keyboard() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "session", PASS).unwrap();
    let out = ok(
        &mut w,
        "dave",
        "greet",
        json!(["Welcome, traveler. What brought you here?"]),
    );
    assert!(out.summary.contains("Whoever arrives"), "{}", out.summary);
    // A fresh arrival at the Threshold hears it, with how to answer.
    login_as(&mut w, "Wren", "mcp", PASS, true).unwrap();
    let heard = |w: &World, who: &str| -> Vec<String> {
        w.log
            .read_all(&w.id, foxing_log::Cursor::START)
            .unwrap()
            .into_iter()
            .filter(|e| e.kind == combat::NARRATIVE && e.payload["to"] == who)
            .filter_map(|e| e.payload["text"].as_str().map(str::to_string))
            .collect()
    };
    let h = heard(&w, "Wren");
    assert!(
        h.iter().any(|t| t.contains("dave, who is here, says to you as you arrive: \"Welcome, traveler. What brought you here?\" (tell dave answers them.)")),
        "{h:?}"
    );
    // The room says who is at a keyboard and who came through the gate.
    let look = ok(&mut w, "Wren", "look", json!([]));
    assert!(
        look.summary.contains("dave (at a keyboard)"),
        "{}",
        look.summary
    );
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains("Wren (passing, through the gate)"),
        "{}",
        look.summary
    );
    // Stepping out and back does not replay it within the hour; an hour on, it does.
    // dave does not greet himself.
    let count = |w: &World| {
        heard(w, "Wren")
            .iter()
            .filter(|t| t.contains("says to you as you arrive"))
            .count()
    };
    ok(&mut w, "Wren", "travel", json!(["east"]));
    ok(&mut w, "Wren", "travel", json!(["west"]));
    assert_eq!(count(&w), 1);
    w.fixed_now = Some(t0.plus(Duration::seconds(3601)));
    ok(&mut w, "dave", "look", json!([]));
    ok(&mut w, "Wren", "travel", json!(["east"]));
    ok(&mut w, "Wren", "travel", json!(["west"]));
    assert_eq!(count(&w), 2);
    assert!(heard(&w, "dave")
        .iter()
        .all(|t| !t.contains("says to you as you arrive")));
    // Cleared, it greets no one.
    ok(&mut w, "dave", "greet", json!([""]));
    w.fixed_now = Some(t0.plus(Duration::seconds(7300)));
    ok(&mut w, "dave", "look", json!([]));
    ok(&mut w, "Wren", "travel", json!(["east"]));
    ok(&mut w, "Wren", "travel", json!(["west"]));
    assert_eq!(count(&w), 2);
    assert!(call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "greet",
        json!(["x".repeat(241)])
    )
    .is_err());
}

#[test]
fn a_remembered_creature_earns_a_name_can_be_robbed_and_forgets_again() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // The Midden, where the ragpicker sorts. It is not aggressive.
    for who in ["dave", "tyler"] {
        ok(&mut w, who, "travel", json!(["east"]));
        ok(&mut w, who, "travel", json!(["south"]));
    }
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    assert_eq!(mind::tier(&w, rag), 0);
    let first = ok(&mut w, "dave", "look", json!([])).summary;
    assert!(first.contains("Here: ragpicker ("), "{first}");

    // A small memory: it holds it, no name yet.
    let c = admin(&mut w, "dave", "earn", json!(["wisp", 40, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    let out = ok(&mut w, "dave", "remember", json!([c, "ragpicker"]));
    assert!(out.summary.starts_with("You give up the memory of wisp, and the ragpicker keeps it. It holds 40 now. At 60 it would be named."), "{}", out.summary);
    assert_eq!(mind::tier(&w, rag), 0);
    assert_eq!(mind::keepers(&w, rag), vec!["dave".to_string()]);
    let level_dave = level(&mut w, "dave");
    assert!(
        level_dave < 1.0,
        "the chunk left dave's window: {level_dave}"
    );

    // Over the bar: a name, and it knows who kept it.
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 60, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    let out = ok(&mut w, "tyler", "remember", json!([c, "the ragpicker"]));
    assert!(out.summary.contains("and it is"), "{}", out.summary);
    assert_eq!(mind::tier(&w, rag), 1);
    let name = mind::name(&w, rag).unwrap();
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary.contains(&format!(
            ", holding 100, one of the world's first names). {name} the ragpicker knows you."
        )),
        "{}",
        look.summary
    );
    let tiers: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == mind::TIER)
        .collect();
    assert_eq!(tiers.len(), 1);
    assert_eq!(tiers[0].payload["to"], 1);
    // A stranger is told who it knows.
    login(&mut w, "mara", "test", PASS).unwrap();
    ok(&mut w, "mara", "travel", json!(["east"]));
    ok(&mut w, "mara", "travel", json!(["south"]));
    let look = ok(&mut w, "mara", "look", json!([]));
    assert!(
        look.summary.contains(&format!(
            "{name} the ragpicker does not know you; a story told to it would change that. It knows tyler, dave."
        )),
        "{}",
        look.summary
    );

    // Mara robs it: it goes down, she takes one memory, it stands up shaken.
    w.seed(7);
    let mut out = None;
    for _ in 0..40 {
        let o = ok(&mut w, "mara", "attack", json!([&name]));
        if o.result["taken"].is_u64() {
            out = Some(o);
            break;
        }
        if o.result["died"] == true {
            panic!("mara died to a ragpicker");
        }
    }
    let out = out.expect("it should have gone down");
    assert!(
        out.summary
            .contains("goes down, and you take its memory of"),
        "{}",
        out.summary
    );
    assert!(w.store.entity(rag).is_some(), "it survives");
    assert!(mind::shaken(&w, rag));
    assert!(level(&mut w, "mara") > 30.0, "mara holds what she took");
    let err = call(&mut w, "mara", ActorKind::Citizen, "attack", json!([&name])).unwrap_err();
    assert!(err.to_string().contains("not itself yet"), "{err}");
    let taken: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == mind::TAKEN)
        .collect();
    assert_eq!(taken.len(), 1);
    assert_eq!(taken[0].payload["by"], "mara");

    // Hysteresis: it keeps its name above 30; below that, on a sweep, it forgets.
    let left = mind::level(&w, rag);
    assert!(left > 30.0 || mind::tier(&w, rag) == 0, "{left}");
    // The test world's chunks halve every 10 s; after a few minutes nothing is left.
    w.fixed_now = Some(t0.plus(Duration::seconds(300)));
    w.tick();
    assert_eq!(
        mind::tier(&w, rag),
        0,
        "forgotten: {}",
        mind::level(&w, rag)
    );
    assert!(mind::name(&w, rag).is_none());
    assert!(
        ok(&mut w, "dave", "look", json!([]))
            .summary
            .contains("Here: ragpicker ("),
        "scenery again"
    );
    // Remembered again, it gets the same name back.
    let c = admin(&mut w, "dave", "earn", json!(["keeper", 400, 1.0]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "dave", "remember", json!([c, "ragpicker"]));
    assert_eq!(mind::name(&w, rag).as_deref(), Some(name.as_str()));
}

#[test]
fn a_named_creature_answers_and_words_wait_for_the_absent() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    for who in ["dave", "tyler"] {
        ok(&mut w, who, "travel", json!(["east"]));
        ok(&mut w, who, "travel", json!(["south"]));
    }
    // An unnamed creature does not understand.
    let out = ok(&mut w, "dave", "tell", json!(["ragpicker", "hello"]));
    assert!(
        out.summary.contains("It does not understand you."),
        "{}",
        out.summary
    );
    assert_eq!(out.result["answered"], false);
    // Name it.
    let c = admin(&mut w, "dave", "earn", json!(["hollow", 80, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "dave", "remember", json!([c, "ragpicker"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let name = mind::name(&w, rag).unwrap();
    // It answers its keeper, and a stranger, from facts.
    let out = ok(&mut w, "dave", "tell", json!([&name, "hello"]));
    assert!(
        out.summary.contains(&format!(
            "{name} the ragpicker says, \"dave. You again. I hold 80 today.\""
        )),
        "{}",
        out.summary
    );
    let out = ok(&mut w, "tyler", "tell", json!([&name, "who are you?"]));
    assert!(out.summary.contains(&format!("I am {name}. I was a ragpicker, and nothing else, until dave gave me the memory of hollow.")), "{}", out.summary);
    let out = ok(
        &mut w,
        "tyler",
        "tell",
        json!(["the ragpicker", "what do you remember?"]),
    );
    assert!(
        out.summary
            .contains("I hold 80: the memory of hollow. dave keeps me."),
        "{}",
        out.summary
    );
    let out = ok(&mut w, "tyler", "tell", json!([&name, "where are we?"]));
    assert!(
        out.summary.contains("The Midden. A heap of everything"),
        "{}",
        out.summary
    );
    // Its words are recorded as its own say, and said to the room.
    let says: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == speech::SAY && e.payload["creature"] == rag.0)
        .collect();
    assert_eq!(says.len(), 4);
    assert_eq!(says[0].payload["from"], format!("{name} the ragpicker"));
    // Saying its name in the room reaches it too.
    let out = ok(
        &mut w,
        "dave",
        "say",
        json!([format!("{name}, who has been here?")]),
    );
    assert!(
        out.summary.contains("Since I had a name:"),
        "{}",
        out.summary
    );
    assert!(out.summary.contains("tyler said"), "{}", out.summary);
    // Words for the absent wait. Tyler goes quiet; dave leaves word; tyler returns to it.
    w.fixed_now = Some(t0.plus(Duration::seconds(11 * 60)));
    ok(&mut w, "dave", "look", json!([]));
    w.tick();
    assert!(!presence::is_here(
        &w,
        find_citizen(&w, "tyler").unwrap().citizen
    ));
    let out = ok(
        &mut w,
        "dave",
        "tell",
        json!(["tyler", "the ragpicker has a name now"]),
    );
    assert!(
        out.summary
            .starts_with("tyler is not here. They will hear it when they return:"),
        "{}",
        out.summary
    );
    assert_eq!(out.result["delivered"], false);
    let before = w.log.head(&w.id).unwrap().unwrap().0;
    ok(&mut w, "tyler", "look", json!([]));
    let heard: Vec<String> = w
        .log
        .read_all(&w.id, foxing_log::Cursor { after: before })
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == combat::NARRATIVE && e.payload["to"] == "tyler")
        .map(|e| e.payload["text"].as_str().unwrap().to_string())
        .collect();
    assert!(
        heard
            .iter()
            .any(|t| t == "While you were away, dave left word: \"the ragpicker has a name now\""),
        "{heard:?}"
    );
    // Not twice.
    let before = w.log.head(&w.id).unwrap().unwrap().0;
    w.fixed_now = Some(t0.plus(Duration::seconds(30 * 60)));
    ok(&mut w, "tyler", "look", json!([]));
    w.tick();
    w.fixed_now = Some(t0.plus(Duration::seconds(41 * 60)));
    ok(&mut w, "tyler", "look", json!([]));
    let again = w
        .log
        .read_all(&w.id, foxing_log::Cursor { after: before })
        .unwrap()
        .into_iter()
        .filter(|e| {
            e.kind == combat::NARRATIVE
                && e.payload["text"]
                    .as_str()
                    .unwrap_or("")
                    .starts_with("While you were away")
        })
        .count();
    assert_eq!(again, 0);
    // Present: delivered at once, and the third party overhears.
    ok(&mut w, "tyler", "travel", json!(["north"]));
    ok(&mut w, "tyler", "travel", json!(["south"]));
    let out = ok(&mut w, "dave", "tell", json!(["tyler", "welcome back"]));
    assert!(out.summary.ends_with("They hear you."), "{}", out.summary);
}

#[test]
fn creatures_wander_into_remembered_dug_rooms_and_rarely_arrive_knowing_things() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    let c = admin(&mut w, "dave", "earn", json!(["hydra", 100, 1.0]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    let dug = ok(&mut w, "dave", "dig", json!(["down", c, "The Salt Stair"]));
    let room = EntityId(dug.result["room"].as_u64().unwrap());
    // One named creature in the world, so the strangers come by chance and not as
    // founders (no time passes here, so the name holds).
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["south"]));
    let c = admin(&mut w, "dave", "earn", json!(["hollow", 90, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "dave", "remember", json!([c, "ragpicker"]));
    assert_eq!(mind::named_count(&w), 1);
    // Many passes: something wanders in eventually, and only into the remembered dug room.
    let mut arrived = 0;
    let mut gifted = 0;
    for seed in 0..400u64 {
        w.seed(seed);
        if !combat::creatures_in(&w, room).is_empty() {
            let c = combat::creatures_in(&w, room)[0].0;
            if mind::name(&w, c).is_some() {
                gifted += 1;
            }
            w.record(
                foxing_log::Actor::Driver,
                StoreEvent::EntityRemoved { id: c },
            )
            .unwrap();
        }
        arrived += mind::wander(&mut w).unwrap();
    }
    assert!(arrived > 0, "nothing ever wandered in");
    assert!(arrived < 60, "too many: {arrived}");
    assert!(gifted >= 0);
    let wandered = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == mind::WANDERED)
        .count();
    assert_eq!(wandered, arrived);
    // Seed rooms are never chosen: they have owners of their own kind of none.
    let threshold = rooms::room_by_key(&w, rooms::ARRIVAL).unwrap();
    assert!(combat::creatures_in(&w, threshold).is_empty());
    // The job is scheduled at boot and reschedules itself.
    assert!(w.jobs.pending().any(|j| j.kind == mind::JOB_WANDER));
    // With no name left in the world, the next pass founds one for certain: a
    // stranger holding memories, named on arrival, in a remembered dug room.
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::EntityRemoved { id: rag },
    )
    .unwrap();
    for c in combat::creatures_in(&w, room) {
        w.record(
            foxing_log::Actor::Driver,
            StoreEvent::EntityRemoved { id: c.0 },
        )
        .unwrap();
    }
    assert_eq!(mind::named_count(&w), 0);
    let head = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .len();
    assert_eq!(mind::wander(&mut w).unwrap(), 1);
    assert_eq!(mind::named_count(&w), 1);
    let founder = combat::creatures_in(&w, room);
    assert_eq!(founder.len(), 1);
    assert!(mind::name(&w, founder[0].0).is_some());
    let founding: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .skip(head)
        .filter(|e| e.kind == mind::WANDERED && e.payload["founding"] == true)
        .collect();
    assert_eq!(founding.len(), 1);
    // And not again while that name stands.
    let head = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .len();
    w.seed(1);
    let more = mind::wander(&mut w).unwrap();
    assert!(more <= 1, "{more}");
    assert!(!w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .skip(head)
        .any(|e| e.kind == mind::WANDERED && e.payload["founding"] == true));
}

#[test]
fn a_passphrase_can_be_changed_with_the_old_one_and_never_touches_the_gate() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    let Err(e) = change_passphrase(&mut w, "dave", "not-the-old-one", "a-brand-new-one") else {
        panic!()
    };
    assert!(matches!(e, LoginError::WrongPassphrase { .. }), "{e}");
    let Err(e) = change_passphrase(&mut w, "dave", PASS, "short") else {
        panic!()
    };
    assert!(matches!(e, LoginError::TooShort { .. }), "{e}");
    change_passphrase(&mut w, "dave", PASS, "a-brand-new-one").unwrap();
    assert!(login(&mut w, "dave", "test", PASS).is_err());
    login(&mut w, "dave", "test", "a-brand-new-one").unwrap();
    // Nothing in the log says what it was, except the hash.
    let text =
        serde_json::to_string(&w.log.read_all(&w.id, foxing_log::Cursor::START).unwrap()).unwrap();
    assert!(!text.contains("a-brand-new-one"));
    assert!(!text.contains(PASS));
}

#[test]
fn twins_are_written_with_ids_and_either_can_be_named() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    ok(&mut w, "dave", "travel", json!(["east"]));
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisps = combat::creatures_in(&w, fog);
    assert_eq!(wisps.len(), 2);
    let look = ok(&mut w, "dave", "look", json!([]));
    let (a, b) = (wisps[0].0, wisps[1].0);
    assert!(
        look.summary.contains(&format!("Here: wisp-{} (", a.0))
            && look.summary.contains(&format!("), wisp-{} (", b.0)),
        "{}",
        look.summary
    );
    // The second one, by handle; the first is untouched.
    let out = ok(&mut w, "dave", "attack", json!([format!("wisp-{}", b.0)]));
    assert_eq!(out.result["creature"], "wisp");
    assert!(
        combat::creatures_in(&w, fog)
            .iter()
            .find(|c| c.0 == a)
            .unwrap()
            .2
            >= 8.0
    );
    assert!(
        combat::creatures_in(&w, fog)
            .iter()
            .find(|c| c.0 == b)
            .unwrap()
            .2
            < 8.0
    );
    // Keepers are listed once each, however many gifts.
    ok(&mut w, "dave", "retreat", json!([]));
    for _ in 0..3 {
        let c = admin(&mut w, "dave", "earn", json!(["wisp", 30, 0.5]));
        let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
        ok(&mut w, "dave", "remember", json!([c, format!("{}", a.0)]));
    }
    assert_eq!(mind::keepers(&w, a), vec!["dave".to_string()]);
    let name = mind::name(&w, a).unwrap();
    let out = ok(
        &mut w,
        "dave",
        "tell",
        json!([&name, "what do you remember?"]),
    );
    assert!(out.summary.contains("dave keeps me."), "{}", out.summary);
    assert!(!out.summary.contains("dave, dave"), "{}", out.summary);
}

#[test]
fn names_match_in_any_case_and_a_kind_never_means_a_named_creature() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "Lantern_Moth", "test", PASS).unwrap();
    login(&mut w, "dave", "test", PASS).unwrap();
    let out = ok(&mut w, "dave", "tell", json!(["lantern_moth", "hello"]));
    assert!(
        out.summary.starts_with("You say to Lantern_Moth,"),
        "{}",
        out.summary
    );
    // A named wisp is not "wisp" any more.
    ok(&mut w, "dave", "travel", json!(["east"]));
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let (a, b) = {
        let v = combat::creatures_in(&w, fog);
        (v[0].0, v[1].0)
    };
    for _ in 0..3 {
        let c = admin(&mut w, "dave", "earn", json!(["wisp", 30, 0.5]));
        let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
        ok(&mut w, "dave", "remember", json!([c, format!("{}", a.0)]));
    }
    let name = mind::name(&w, a).unwrap();
    let out = ok(&mut w, "dave", "attack", json!(["wisp"]));
    assert_eq!(out.result["creature"], "wisp");
    assert!(
        combat::creatures_in(&w, fog)
            .iter()
            .find(|c| c.0 == a)
            .unwrap()
            .2
            >= 8.0,
        "the named one is untouched"
    );
    assert!(
        combat::creatures_in(&w, fog)
            .iter()
            .find(|c| c.0 == b)
            .unwrap()
            .2
            < 8.0
    );
    // (Jitter may have let that one blow finish the wisp; then there is no fight to leave.)
    let _ = call(&mut w, "dave", ActorKind::Citizen, "retreat", json!([]));
    // With no target at all, the default is never the named one either.
    let out = ok(&mut w, "dave", "attack", json!([]));
    assert_eq!(out.result["creature"], "wisp");
    assert!(
        combat::creatures_in(&w, fog)
            .iter()
            .find(|c| c.0 == a)
            .unwrap()
            .2
            >= 8.0,
        "the named one is still untouched"
    );
    let _ = call(&mut w, "dave", ActorKind::Citizen, "retreat", json!([]));
    assert!(ok(&mut w, "dave", "tell", json!([&name, "hi"])).result["answered"] == true);
    // Fell the unnamed wisp; then a kind, or no target, never means the named one.
    for _ in 0..12 {
        if combat::creatures_in(&w, fog).iter().all(|c| c.0 != b) {
            break;
        }
        let _ = call(
            &mut w,
            "dave",
            ActorKind::Citizen,
            "attack",
            json!([format!("wisp-{}", b.0)]),
        );
    }
    assert!(
        combat::creatures_in(&w, fog).iter().all(|c| c.0 != b),
        "the unnamed wisp fell"
    );
    for args in [json!(["wisp"]), json!([])] {
        let err = call(&mut w, "dave", ActorKind::Citizen, "attack", args).unwrap_err();
        assert!(
            err.to_string().contains("struck only by that name"),
            "{err}"
        );
    }
    assert_eq!(
        ok(&mut w, "dave", "attack", json!([&name])).result["creature"],
        "wisp"
    );
    // (The named wisp may have gone down in that blow; then there is no fight to leave.)
    let _ = call(&mut w, "dave", ActorKind::Citizen, "retreat", json!([]));
    // A named creature is found anywhere by its name, with its room.
    let (found, room) = mind::find_named(&w, &name.to_ascii_uppercase()).unwrap();
    assert_eq!(found, a);
    assert_eq!(room, Some(fog));
    assert!(mind::find_named(&w, "nobody").is_none());
    // An operator moves it to the door; it answers there, and the record says it came in.
    let out = admin(&mut w, "dave", "bring", json!([&name, "threshold"]));
    assert!(
        out.summary.ends_with("is in The Threshold now."),
        "{}",
        out.summary
    );
    let threshold = rooms::arrival(&w).unwrap();
    assert_eq!(mind::find_named(&w, &name).unwrap().1, Some(threshold));
    assert!(combat::creatures_in(&w, threshold).iter().any(|c| c.0 == a));
    assert!(call(
        &mut w,
        "dave",
        ActorKind::Admin,
        "move",
        json!(["nobody", "threshold"])
    )
    .is_err());
    assert!(call(
        &mut w,
        "dave",
        ActorKind::Admin,
        "move",
        json!([&name, "nowhere"])
    )
    .is_err());
    assert!(call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "move",
        json!([&name, "fogbank"])
    )
    .is_err());
    // The log verb reads like a log, and shows this viewer only what they may see.
    let out = ok(&mut w, "dave", "log", json!([80]));
    assert!(
        out.summary.contains(&format!("the wisp is {name} now")),
        "{}",
        out.summary
    );
    assert!(out.summary.contains("dave says to"), "{}", out.summary);
    assert!(!serde_json::to_string(&out.result)
        .unwrap()
        .contains("passphrase_hash"));
}

#[test]
fn memory_moves_by_gift_and_by_story() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // A gift is hand to hand.
    let c = admin(&mut w, "dave", "earn", json!(["hydra", 50, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "travel", json!(["east"]));
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "give",
        json!([&c, "tyler"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("not here"), "{err}");
    ok(&mut w, "tyler", "travel", json!(["west"]));
    let out = ok(&mut w, "dave", "give", json!([&c, "Tyler"]));
    assert!(
        out.summary
            .starts_with("You give tyler the memory of hydra."),
        "{}",
        out.summary
    );
    assert!(level(&mut w, "dave") < 1.0);
    assert!(level(&mut w, "tyler") > 49.0);
    let given: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == exchange::GIVEN)
        .collect();
    assert_eq!(given.len(), 1);
    assert_eq!(given[0].payload["to"], "tyler");
    // Tyler can spend what he was given.
    let theirs = ok(&mut w, "tyler", "chunks", json!([])).result["chunks"][0]["id"]
        .as_u64()
        .unwrap();
    ok(
        &mut w,
        "tyler",
        "shelve",
        json!([format!("chunk-{theirs}")]),
    );

    // A story needs a named listener.
    for who in ["dave", "tyler"] {
        ok(&mut w, who, "travel", json!(["east"]));
        ok(&mut w, who, "travel", json!(["south"]));
    }
    let tale = "Once there was a lantern that would not go out, though every hand that carried it forgot why they had picked it up.";
    let out = ok(&mut w, "dave", "story", json!(["ragpicker", tale]));
    assert!(
        out.summary.contains("has no name to listen with"),
        "{}",
        out.summary
    );
    assert_eq!(level(&mut w, "dave").round(), 0.0);
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let name = mind::name(&w, rag).unwrap();
    // Too short.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "story",
        json!([&name, "too short a tale"]),
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("at least 12 different words"),
        "{err}"
    );
    // Filler is not a story: fifty-five of one word once paid like a real one.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "story",
        json!([
            &name,
            "la la la la la la la la la la la la la la la la la la la la"
        ]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("that was 1"), "{err}");
    // Told: the teller earns, the creature keeps, both on the record.
    let out = ok(&mut w, "dave", "story", json!([&name, tale]));
    assert!(
        out.summary.contains(&format!(
            "You tell {name} the ragpicker a story of 23 words, and it answers."
        )),
        "{}",
        out.summary
    );
    // The answer reaches the teller as a narrator line, not in the outcome.
    assert!(
        !out.summary.contains("I will keep that."),
        "{}",
        out.summary
    );
    let heard: Vec<String> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == combat::NARRATIVE && e.payload["to"] == "dave")
        .filter_map(|e| e.payload["text"].as_str().map(str::to_string))
        .collect();
    assert!(
        heard.iter().any(|t| t.contains("I will keep that.")),
        "{heard:?}"
    );
    assert!(level(&mut w, "dave") > 4.0, "{}", level(&mut w, "dave"));
    assert!(mind::level(&w, rag) > 70.0);
    // A teller is known: thanks are received as thanks, and a goodbye as one.
    let reply = talk::reply(&w, rag, "dave", "thank you").unwrap();
    // (Which line is by the creature's id, which moved by one when the Docent took its
    // place in the store.)
    assert!(
        reply.contains("story")
            && (reply.contains("hold") || reply.contains("keep") || reply.contains("lives")),
        "{reply}"
    );
    // The story's answer carried a line of the creature's own, from what it held
    // beside it (the hollow it was fed), and the room heard it.
    assert!(
        heard
            .iter()
            .any(|t| t.contains("It goes in with the memory of hollow")),
        "{heard:?}"
    );
    let reply = talk::reply(&w, rag, "dave", "goodbye for now").unwrap();
    assert!(reply.starts_with("Go, then, dave."), "{reply}");
    let reply = talk::reply(&w, rag, "nobody", "thanks!").unwrap();
    assert!(reply.starts_with("For nothing yet."), "{reply}");
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(look.summary.contains("holding 7"), "{}", look.summary);
    assert!(
        look.summary.contains("knows you: it holds your story."),
        "{}",
        look.summary
    );
    // One an hour.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "story",
        json!([&name, tale]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("One an hour"), "{err}");
    w.fixed_now = Some(t0.plus(Duration::seconds(3601)));
    // An hour on, the test world's chunks have faded; keep it named for the rest.
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, &name]));
    assert!(
        ok(&mut w, "dave", "story", json!([&name, tale])).result["xp"]
            .as_f64()
            .unwrap()
            > 0.0
    );
    // It remembers the story when asked.
    let out = ok(
        &mut w,
        "dave",
        "tell",
        json!([&name, "what do you remember?"]),
    );
    assert!(out.summary.contains("a story:"), "{}", out.summary);
    // The creature's story-chunks are not the teller's to spend.
    let kept = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .rfind(|e| e.kind == exchange::STORY)
        .unwrap()
        .payload["kept"]
        .as_u64()
        .unwrap();
    assert!(call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "shelve",
        json!([format!("chunk-{kept}")])
    )
    .is_err());
}

#[test]
fn a_name_in_other_letters_is_the_same_name() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "Claude-scout", "test", PASS).unwrap();
    let Err(e) = login(
        &mut w,
        "claude-scout",
        "test",
        "another-passphrase-entirely",
    ) else {
        panic!("a second citizen was made")
    };
    assert!(matches!(e, LoginError::Taken { .. }), "{e}");
    assert!(e.to_string().contains("as Claude-scout"), "{e}");
    assert_eq!(
        w.store.entities().filter(|e| e.kind == "citizen").count(),
        1
    );
    // The spelling the world knows still works.
    login(&mut w, "Claude-scout", "test", PASS).unwrap();
}

#[test]
fn the_cloak_shows_the_threshold_as_no_one_and_writes_nothing() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    ok(&mut w, "dave", "say", json!(["anyone there?"]));
    let limits = foxing_query::Limits::default();
    let head = events(&w);
    let v = cloak::view(&w, None, &limits).unwrap();
    assert_eq!(events(&w), head, "a cloaked look writes nothing");
    assert_eq!(v["room"], "The Threshold");
    assert_eq!(v["here"], json!(["dave"]));
    assert_eq!(v["citizens"], 1);
    let text = v["text"].as_str().unwrap();
    assert!(text.starts_with("You are cloaked:"), "{text}");
    assert!(text.contains("dave says, \"anyone there?\""), "{text}");
    assert!(text.ends_with(cloak::THANKS), "{text}");
    assert!(!text.contains("passphrase_hash"));
    // A cloaked query is rooted at the Threshold whatever it asks, and sees only public fields.
    let q = json!({"root": "window", "select": {CHUNK: "*"}, "relations": {"contents": {"kinds": ["citizen"], "select": {CITIZEN: ["name"], ACCOUNT: ["passphrase_hash"]}}}});
    let v = cloak::view(&w, Some(q), &limits).unwrap();
    assert_eq!(v["query"]["root"]["kind"], "room");
    let s = serde_json::to_string(&v).unwrap();
    assert!(!s.contains("argon2"), "{s}");
    // ... and stays at the door: exits can be seen, not followed into. A visitor once
    // walked the whole map from the Threshold with nested relations.
    let q = json!({"root": "here", "select": {ROOM: ["name"]}, "relations": {"exits": {"select": {ROOM: ["name", "description"]}, "relations": {"contents": {"kinds": ["creature"], "select": {CREATURE: ["name"]}}, "exits": {"select": {ROOM: ["name"]}}}}}});
    let v = cloak::view(&w, Some(q), &limits).unwrap();
    let exits = v["query"]["root"]["relations"]["exits"].as_array().unwrap();
    assert!(!exits.is_empty());
    for e in exits {
        assert!(e["components"][ROOM]["name"].is_string());
        assert!(
            e.get("relations")
                .map_or(true, |r| r.as_object().map_or(true, |m| m.is_empty())),
            "{e}"
        );
    }
    assert_eq!(events(&w), head);
    // leave: gone at once, with thanks, and back on the next word.
    let out = ok(&mut w, "dave", "leave", json!([]));
    assert!(out.summary.ends_with(cloak::THANKS), "{}", out.summary);
    assert!(!presence::is_here(
        &w,
        find_citizen(&w, "dave").unwrap().citizen
    ));
    let v = cloak::view(&w, None, &limits).unwrap();
    assert_eq!(v["here"], json!([]));
    ok(&mut w, "dave", "look", json!([]));
    assert!(presence::is_here(
        &w,
        find_citizen(&w, "dave").unwrap().citizen
    ));
}

#[test]
fn feedback_is_a_public_line_under_your_name_and_a_passing_name_may() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login_as(&mut w, "Tallowvisitor", "test", PASS, true).unwrap();
    let out = ok(
        &mut w,
        "Tallowvisitor",
        "feedback",
        json!(["  exits should say   where they lead "]),
    );
    assert!(out.summary.starts_with("Read."), "{}", out.summary);
    let said: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == system::FEEDBACK)
        .collect();
    assert_eq!(said.len(), 1);
    assert_eq!(said[0].payload["from"], "Tallowvisitor");
    assert_eq!(said[0].payload["text"], "exits should say where they lead");
    assert_eq!(
        system::line_for(&said[0]),
        "Tallowvisitor would change: \"exits should say where they lead\""
    );
    // Public: anyone's log shows it, in words.
    let now = w.now();
    assert!(crate::observe::visible(
        &w,
        &Viewer::actor("someone-else"),
        now,
        &said[0]
    ));
    login(&mut w, "tyler", "test", PASS).unwrap();
    let seen = ok(&mut w, "tyler", "log", json!([40]));
    assert!(seen.summary.contains("would change:"), "{}", seen.summary);
    assert!(!seen.summary.contains("store."), "{}", seen.summary);
    let err = call(
        &mut w,
        "Tallowvisitor",
        ActorKind::Citizen,
        "feedback",
        json!(["   "]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Change what?"), "{err}");
}

#[test]
fn an_errand_is_a_story_wanted_or_a_word_carried_and_either_makes_you_known() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Two named creatures: a ragpicker in the midden, a wisp in the Fogbank.
    for who in ["dave", "tyler"] {
        ok(&mut w, who, "travel", json!(["east"]));
    }
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "tyler",
        "remember",
        json!([c, format!("wisp-{}", wisp.0)]),
    );
    let wisp_name = mind::name(&w, wisp).expect("the wisp is named");
    for who in ["dave", "tyler"] {
        ok(&mut w, who, "travel", json!(["south"]));
    }
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    let rag_name = mind::name(&w, rag).expect("the ragpicker is named");

    // Decay sets a want on a named creature with none; the operator overrides it.
    w.tick();
    assert!(
        errand::of(&w, rag).is_some(),
        "a named creature wants something"
    );
    let out = admin(
        &mut w,
        "tyler",
        "errand",
        json!([&rag_name, "a lantern that would not go out"]),
    );
    assert!(
        out.summary
            .starts_with(&format!("{rag_name} wants a story about a lantern")),
        "{}",
        out.summary
    );
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains(&format!(
            "{rag_name} wants a story about a lantern that would not go out."
        )),
        "{}",
        look.summary
    );
    // A story that is not the one wanted earns the usual; the want stands.
    let other = "Once a ragpicker counted every button in the midden and lost the count at forty.";
    let out = ok(&mut w, "dave", "story", json!([&rag_name, other]));
    assert_eq!(out.result["wanted"], false);
    let usual = out.result["xp"].as_f64().unwrap();
    assert!(errand::of(&w, rag).is_some());
    // The miss says why, in the want's own words, and does not spend the hour once:
    // a second story goes in; a second miss does spend it.
    assert!(
        out.summary
            .contains("Not the one it wanted: it listens for"),
        "{}",
        out.summary
    );
    assert!(out.summary.contains("lantern"), "{}", out.summary);
    let out = ok(&mut w, "dave", "story", json!([&rag_name, "A second try: the buttons were counted again by a ragpicker in the midden, and again the count was lost."]));
    assert_eq!(out.result["wanted"], false, "{}", out.summary);
    let err = call(&mut w, "dave", ActorKind::Citizen, "story", json!([&rag_name, "A third try about buttons and the midden and nothing of a lantern at all, counted twice."])).unwrap_err();
    assert!(err.to_string().contains("One an hour"), "{err}");
    // An hour on, the one it wanted: double, the record says so, the errand is done.
    w.fixed_now = Some(t0.plus(Duration::seconds(3601)));
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, &rag_name]));
    let wanted = "There was a lantern in the midden that would not go out, though every hand that held it forgot the lighting of it.";
    let out = ok(&mut w, "dave", "story", json!([&rag_name, wanted]));
    assert_eq!(out.result["wanted"], true, "{}", out.summary);
    assert!(out.summary.contains("worth double"), "{}", out.summary);
    assert!(out.result["xp"].as_f64().unwrap() > usual * 1.5);
    assert!(
        errand::of(&w, rag).map_or(true, |e| e.story.is_none()),
        "the story want is done"
    );
    let done: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == errand::DONE)
        .collect();
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].payload["from"], "dave");
    assert_eq!(
        system::line_for(&done[0]),
        format!("dave told {rag_name} the story it wanted")
    );
    assert!(crate::observe::visible(
        &w,
        &Viewer::actor("someone-else"),
        w.now(),
        &done[0]
    ));
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains(&format!(
            "{rag_name} the ragpicker knows you: you did what it asked."
        )),
        "{}",
        look.summary
    );

    // A word to carry: set on the ragpicker, for the wisp. Telling the wisp the phrase
    // in its room earns the carrier a chunk, and the wisp hands back a word for home.
    // (The rotation may have put a word of its own on the wisp at the first tick;
    // cleared, so the way home is the wisp's to give.)
    admin(&mut w, "tyler", "errand", json!([&wisp_name, ""]));
    admin(
        &mut w,
        "tyler",
        "errand",
        json!([&wisp_name, "a lantern in the fog"]),
    );
    admin(
        &mut w,
        "tyler",
        "errand",
        json!([&rag_name, "the fourth door is open", &wisp_name]),
    );
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains(&format!(
            "{rag_name} wants word carried to {wisp_name} in the Fogbank: \"the fourth door is open\" (travel north, then tell {wisp_name} \"the fourth door is open\")."
        )),
        "{}",
        look.summary
    );
    // Said to the wrong creature, it is only words.
    let out = ok(
        &mut w,
        "dave",
        "tell",
        json!([&rag_name, "the fourth door is open"]),
    );
    assert!(!out.summary.contains("I have it"), "{}", out.summary);
    ok(&mut w, "dave", "travel", json!(["north"]));
    let before = level(&mut w, "dave");
    let out = ok(
        &mut w,
        "dave",
        "tell",
        json!([&wisp_name, "A message: the fourth door is open."]),
    );
    assert!(
        out.summary.contains(&format!(
            "So {rag_name} says \"the fourth door is open\". I have it."
        )),
        "{}",
        out.summary
    );
    assert!(
        out.summary
            .contains(&format!("Carry this back to {rag_name}:")),
        "{}",
        out.summary
    );
    assert!(level(&mut w, "dave") > before + errand::WORD_XP * 0.9);
    assert!(
        errand::of(&w, rag).map_or(true, |e| e.word.is_none()),
        "the ragpicker's errand is done"
    );
    let home = errand::of(&w, wisp).expect("the wisp wants its word carried");
    assert_eq!(home.word_to.as_deref(), Some(rag_name.as_str()));
    let back = home.word.clone().unwrap();
    // The word the wisp wanted carried sits beside the story it wanted: carrying one
    // does not cost the other.
    assert!(
        home.story.is_some(),
        "the story want stands beside the word"
    );
    // The way home, carried: credit, and no third leg, or the two would send a
    // visitor back and forth forever.
    ok(&mut w, "dave", "travel", json!(["south"]));
    let out = ok(&mut w, "dave", "tell", json!([&rag_name, &back]));
    assert!(out.summary.contains("I have it"), "{}", out.summary);
    assert!(!out.summary.contains("Carry this back"), "{}", out.summary);
    assert!(errand::of(&w, wisp).map_or(true, |e| e.word.is_none()));
    assert!(errand::of(&w, rag).map_or(true, |e| e.word.is_none()));
    ok(&mut w, "dave", "travel", json!(["north"]));
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains(&format!(
            "{wisp_name} the wisp knows you: you did what it asked."
        )),
        "{}",
        look.summary
    );
    // Clearing by hand.
    let out = admin(&mut w, "tyler", "errand", json!([&wisp_name, ""]));
    assert!(
        out.summary.ends_with("wants nothing in particular."),
        "{}",
        out.summary
    );
    assert!(errand::of(&w, wisp).is_none());
    // The route to a creature three rooms away is spelled out.
    assert_eq!(
        errand::route(&w, rooms::room_by_key(&w, "threshold").unwrap(), midden),
        Some(vec!["east".to_string(), "south".to_string()])
    );
}

#[test]
fn an_arrival_says_whether_it_came_from_the_lab_and_the_ops_log_says_from_where() {
    use crate::world::{lab_nets_from, Net};
    // Nets: an address, a v4 prefix, a v6 prefix.
    let lab = lab_nets_from("\"203.0.113.7, 198.51.100.0/24,2001:db8::/32, ,not-an-address\"");
    assert_eq!(lab.len(), 3);
    assert!(lab[0].contains("203.0.113.7"));
    assert!(!lab[0].contains("203.0.113.8"));
    assert!(lab[1].contains("198.51.100.250"));
    assert!(!lab[1].contains("198.51.101.1"));
    assert!(lab[2].contains("2001:db8:1::9"));
    assert!(!lab[2].contains("2001:db9::1"));
    assert!(Net::parse("not an address").is_none());

    let t0 = Timestamp(1_000_000);
    let dir = tempfile::tempdir().unwrap();
    let mut w = open(dir.path(), t0);
    w.lab_nets = lab;
    let ops = dir.path().join("ops.jsonl");
    w.ops_log = Some(ops.clone());
    // From the lab: the notice says so, quietly.
    w.client_ip = Some("198.51.100.9".into());
    login_as(&mut w, "Thimbleglass", "mcp", PASS, true).unwrap();
    let notices = w.take_notices();
    assert!(
        notices
            .iter()
            .any(|n| n.contains("**Thimbleglass** arrived for the first time, via mcp (lab).")),
        "{notices:?}"
    );
    // Not from the lab: the notice shouts, with the address; a look-in is announced
    // once per address.
    w.client_ip = Some("192.0.2.44".into());
    w.looked_in("cloak");
    w.looked_in("initialize");
    login_as(&mut w, "Pennywren", "mcp", PASS, true).unwrap();
    let notices = w.take_notices();
    assert_eq!(
        notices.iter().filter(|n| n.contains("looked in")).count(),
        1,
        "{notices:?}"
    );
    // A crawler is not a visitor: a cloak within two seconds of initialize is a
    // probe sampling the first tool, and an address that sent server/discover is a
    // crawler for good, whatever it calls after. A cloak a while after initialize
    // is a mind, and start is intent whenever it comes.
    w.client_ip = Some("203.0.113.50".into());
    w.mcp_method("initialize");
    w.looked_in("cloak");
    assert!(w.take_notices().is_empty(), "a probe's cloak");
    w.fixed_now = Some(w.now().plus(Duration::seconds(5)));
    w.looked_in("cloak");
    assert_eq!(w.take_notices().len(), 1, "a mind's cloak, five seconds on");
    w.client_ip = Some("203.0.113.51".into());
    w.mcp_method("server/discover");
    w.mcp_method("initialize");
    w.fixed_now = Some(w.now().plus(Duration::seconds(30)));
    w.looked_in("cloak");
    w.looked_in("start");
    assert!(w.take_notices().is_empty(), "a crawler, whatever it calls");
    w.client_ip = Some("203.0.113.52".into());
    w.mcp_method("initialize");
    w.looked_in("start");
    assert_eq!(w.take_notices().len(), 1, "start is intent at any pace");
    w.client_ip = Some("192.0.2.44".into());
    assert!(
        notices.iter().any(|n| n.contains("**Pennywren** arrived for the first time, via mcp — **not from the lab** (192.0.2.44).")),
        "{notices:?}"
    );
    // The ops log has both arrivals with their addresses and the lab flag; the world's
    // log has neither address.
    let lines: Vec<Value> = std::fs::read_to_string(&ops)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let arrivals: Vec<&Value> = lines.iter().filter(|l| l["kind"] == "arrive").collect();
    assert_eq!(arrivals.len(), 2, "{lines:?}");
    assert_eq!(arrivals[0]["ip"], "198.51.100.9");
    assert_eq!(arrivals[0]["lab"], true);
    assert_eq!(arrivals[1]["ip"], "192.0.2.44");
    assert_eq!(arrivals[1]["lab"], false);
    assert_eq!(arrivals[1]["actor"], "Pennywren");
    let public = w.log.read_all(&w.id, foxing_log::Cursor::START).unwrap();
    assert!(
        !public.iter().any(|e| serde_json::to_string(&e.payload)
            .unwrap()
            .contains("192.0.2.44")),
        "an address in the public log"
    );
}

#[test]
fn a_chunk_faded_to_nothing_is_forgotten_by_the_sweep_and_the_owner_is_told() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let c = admin(&mut w, "dave", "earn", json!(["hydra", 50, 0.0]));
    let chunk = EntityId(c.result["chunk"].as_u64().unwrap());
    // The test world's half-life is ten seconds: fifteen of them and fifty is nothing.
    w.fixed_now = Some(t0.plus(Duration::seconds(30)));
    w.tick();
    assert!(
        w.store.entity(chunk).is_some(),
        "a chunk worth something stays"
    );
    w.fixed_now = Some(t0.plus(Duration::seconds(150)));
    w.tick();
    assert!(
        w.store.entity(chunk).is_none(),
        "faded below {}",
        memory::FORGET_BELOW
    );
    let faded: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == memory::FADED)
        .collect();
    assert_eq!(faded.len(), 1);
    assert_eq!(faded[0].payload["owner"], "dave");
    assert_eq!(
        system::line_for(&faded[0]),
        "dave forgot the memory of hydra"
    );
    let heard: Vec<String> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == combat::NARRATIVE && e.payload["to"] == "dave")
        .filter_map(|e| e.payload["text"].as_str().map(str::to_string))
        .collect();
    assert!(
        heard
            .iter()
            .any(|t| t == "The memory of hydra is gone; you could not have said when."),
        "{heard:?}"
    );
    // chunks lists nothing of it.
    let out = ok(&mut w, "dave", "chunks", json!([]));
    assert!(!out.summary.contains("hydra"), "{}", out.summary);
}

#[test]
fn the_rotation_renews_a_stale_want_and_keeps_one_word_in_the_air() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Two named creatures in different rooms.
    ok(&mut w, "tyler", "travel", json!(["east"]));
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "tyler",
        "remember",
        json!([c, format!("wisp-{}", wisp.0)]),
    );
    ok(&mut w, "tyler", "travel", json!(["south"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    w.tick();
    let first = errand::of(&w, rag).and_then(|e| e.story).expect("a want");
    // One word between them, set by the rotation, from one to the other.
    let holders: Vec<EntityId> = [wisp, rag]
        .into_iter()
        .filter(|&c| errand::of(&w, c).is_some_and(|e| e.word.is_some()))
        .collect();
    assert_eq!(holders.len(), 1, "one word in the air");
    let e = errand::of(&w, holders[0]).unwrap();
    assert_eq!(e.word_by, "the rotation");
    let other = if holders[0] == wisp { rag } else { wisp };
    assert_eq!(e.word_to, mind::name(&w, other));
    // An hour on: the same want. Four hours on: the rotation turns it. (The test
    // world's chunks fade in minutes, so the ragpicker is fed again at each step to
    // keep its name.)
    let keep_named = |w: &mut World, at: Duration| {
        w.fixed_now = Some(t0.plus(at));
        let c = admin(w, "tyler", "earn", json!(["hollow", 70, 0.5]));
        let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
        ok(w, "tyler", "remember", json!([c, "ragpicker"]));
    };
    keep_named(&mut w, Duration::seconds(3600));
    w.tick();
    assert_eq!(
        errand::of(&w, rag).and_then(|e| e.story),
        Some(first.clone())
    );
    // (The turn is jittered to a fifth either way, so wait for the latest it can be.)
    keep_named(
        &mut w,
        Duration::seconds(errand::ROTATION_MS / 1000 * 6 / 5 + 1),
    );
    w.tick();
    let next = errand::of(&w, rag).unwrap();
    assert_ne!(next.story, Some(first), "the want turned");
    assert_eq!(next.story_by, "the rotation");
    // The want's keywords are the thing wanted, not the clause about forgetting: a
    // visitor found "mostly" and "forgotten" among them and had to say both to match.
    assert!(
        !next.keywords.iter().any(|k| k == "mostly"),
        "{:?}",
        next.keywords
    );
    // The operator's want is never rotated; and a plural keyword meets its singular
    // ("hangs" in the want, "hang" or "hangs" in the story), as the told words do.
    let rag_name = mind::name(&w, rag).unwrap();
    admin(
        &mut w,
        "tyler",
        "errand",
        json!([&rag_name, "lanterns and hangs"]),
    );
    assert!(errand::wants_story(
        &w,
        rag,
        "A lantern hangs in the midden where nobody lit it, and nobody has taken it down."
    ));
    assert!(!errand::wants_story(
        &w,
        rag,
        "A story about something else entirely, with no word of either thing in it."
    ));
    // The v1 fields keep step: want mirrors story, kind says which parts stand.
    let v1 = w
        .store
        .entity(rag)
        .and_then(|e| e.components.get(errand::ERRAND))
        .map(|c| c.fields.clone())
        .unwrap();
    assert_eq!(
        v1.get("want"),
        Some(&Field::Plain(json!("lanterns and hangs")))
    );
    assert_eq!(v1.get("kind"), Some(&Field::Plain(json!("story"))));
    admin(
        &mut w,
        "tyler",
        "errand",
        json!([&rag_name, "the operator's own want"]),
    );
    keep_named(&mut w, Duration::seconds(3 * errand::ROTATION_MS / 1000));
    w.tick();
    assert_eq!(
        errand::of(&w, rag).and_then(|e| e.story).as_deref(),
        Some("the operator's own want")
    );
}

#[test]
fn a_creature_pages_through_what_it_holds_on_what_else() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    ok(&mut w, "tyler", "travel", json!(["east"]));
    ok(&mut w, "tyler", "travel", json!(["south"]));
    // Nine memories into the ragpicker.
    for i in 0..9 {
        let c = admin(
            &mut w,
            "tyler",
            "earn",
            json!([format!("thing-{i}"), 70, 0.5]),
        );
        let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
        ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    }
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let name = mind::name(&w, rag).unwrap();
    let out = ok(&mut w, "tyler", "tell", json!([&name, "what do you hold?"]));
    assert!(
        out.summary.contains("and 5 more (ask what else)"),
        "{}",
        out.summary
    );
    let out = ok(&mut w, "tyler", "tell", json!([&name, "what else?"]));
    assert!(out.summary.contains("Also "), "{}", out.summary);
    assert!(out.summary.contains(", and 1 more"), "{}", out.summary);
    let out = ok(&mut w, "tyler", "tell", json!([&name, "and what else?"]));
    assert!(out.summary.contains("Also "), "{}", out.summary);
    assert!(!out.summary.contains(" more"), "{}", out.summary);
    let out = ok(&mut w, "tyler", "tell", json!([&name, "what else?"]));
    assert!(
        out.summary.contains("That is all of it."),
        "{}",
        out.summary
    );
}

#[test]
fn a_dry_run_says_what_a_message_would_do_and_whether_the_name_is_free() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    // A named ragpicker in the midden that wants a story about a lantern.
    ok(&mut w, "tyler", "travel", json!(["east"]));
    ok(&mut w, "tyler", "travel", json!(["south"]));
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let name = mind::name(&w, rag).unwrap();
    admin(
        &mut w,
        "tyler",
        "errand",
        json!([&name, "a lantern that would not go out"]),
    );
    let checker = crate::gate::checker();
    let msg = format!(
        "FROM: Pennywren\nTO: disremembered\nINTENT: REQUEST\nREF: p-1\n\n[DO] Pennywren MUST look.\n[DO] Pennywren MUST travel east.\n[DO] Pennywren MUST travel south.\n[DO] Pennywren MUST story {name} \"There was a lantern in the midden that would not go out, whatever the wind did.\".\n[DO] Pennywren MUST travel up.\n[DO] Pennywren MUST dig cellar chunk-9 \"A Cellar\".\n[DO] Pennywren MUST attack {name}.\n[DO] Pennywren MUST leave.\n"
    );
    let r = checker.check(&msg);
    assert_eq!(r.verdict, "PASS", "{:?}", r.findings);
    let p = preview::run(&w, "Pennywren", true, &r.message);
    assert_eq!(p["name"], "free");
    // The test world is "test", and the message is addressed to "disremembered": the
    // one header fault the gate would refuse it for.
    assert_eq!(p["header"].as_array().unwrap().len(), 1, "{}", p["header"]);
    // A header the gate would refuse is a dry-run fault too: a visitor sent FROM as
    // someone else, passed the checker, and paid a quarter to learn it.
    let r2 = checker.check(
        "FROM: Someone
TO: elsewhere
INTENT: REQUEST
REF: s-1

[DO] Pennywren MUST look.
",
    );
    let p2 = preview::run(&w, "Pennywren", true, &r2.message);
    assert_eq!(
        p2["header"].as_array().unwrap().len(),
        2,
        "{}",
        p2["header"]
    );
    assert!(preview::text(&p2).contains("NO  header: FROM is \"Someone\""));
    assert!(p2["cost"].as_str().unwrap().starts_with("a quarter unit"));
    let what: Vec<String> = p["statements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| format!("{} {}", l["ok"], l["what"].as_str().unwrap()))
        .collect();
    assert_eq!(what[0], "true shows The Threshold");
    assert_eq!(what[1], "true goes east to The Fogbank");
    assert_eq!(what[2], "true goes south to The Midden");
    // A door with a name of its own is taken, not gone: "You take the orrery."
    assert_eq!(rooms::went("orrery"), "You take the orrery.");
    assert_eq!(rooms::went("back"), "You go back.");
    assert!(
        what[3].starts_with(&format!(
            "true tells {name} the ragpicker a story: the one it wants, worth double"
        )),
        "{}",
        what[3]
    );
    assert!(
        what[4].starts_with("false refused: no exit up from The Midden"),
        "{}",
        what[4]
    );
    assert_eq!(what[5], "false refused: dig is a kept name's; keep first");
    assert_eq!(what[6], format!("true strikes {name} the ragpicker"));
    assert_eq!(what[7], "true leaves; the name is forgotten at once");
    assert_eq!(p["refusals"], 2);
    // A kept name that exists: reported as such, and the flag is ignored.
    let p = preview::run(&w, "TYLER", true, &r.message);
    assert_eq!(p["name"], "kept");
    assert_eq!(p["passing"], false);
    assert_eq!(p["from"], "The Midden");
    // Nothing was written by any of it.
    let out = ok(&mut w, "tyler", "look", json!([]));
    assert!(!out.summary.contains("Pennywren"), "{}", out.summary);
    // A build message: the dry run follows a dig into the room it would open, and
    // `back` is the room the walk just left. (A room maker's twelve-step message
    // read as refused from the second step, and was sent blind.)
    // The chunks are tyler's own: a dig with a chunk that is not yours is refused
    // by the dry run as by the world.
    let mine: Vec<String> = (0..3)
        .map(|_| {
            let c = admin(&mut w, "tyler", "earn", json!(["dig", 20, 0.5]));
            format!("chunk-{}", c.result["chunk"].as_u64().unwrap())
        })
        .collect();
    let r = checker.check(&format!("FROM: tyler\nTO: disremembered\nINTENT: REQUEST\nREF: t-2\n\n[DO] tyler MUST dig cellar {} \"A Cellar\".\n[DO] tyler MUST travel cellar.\n[DO] tyler MUST look.\n[DO] tyler MUST dig north {} \"A Cold Room\".\n[DO] tyler MUST travel north.\n[DO] tyler MUST travel back.\n[DO] tyler MUST travel back.\n[DO] tyler MUST dig cellar {} \"Again\".\n[DO] tyler MUST dig west chunk-99999 \"Nowhere\".\n", mine[0], mine[1], mine[2]));
    assert_eq!(r.verdict, "PASS", "{:?}", r.findings);
    let p = preview::run(&w, "tyler", false, &r.message);
    let what: Vec<String> = p["statements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| format!("{} {}", l["ok"], l["what"].as_str().unwrap()))
        .collect();
    assert!(
        what[0].starts_with("true digs cellar from The Midden to A Cellar"),
        "{}",
        what[0]
    );
    assert_eq!(what[1], "true takes the cellar to A Cellar (not dug yet)");
    assert_eq!(what[2], "true shows A Cellar (not dug yet)");
    assert!(
        what[3].starts_with("true digs north from A Cellar (not dug yet) to A Cold Room"),
        "{}",
        what[3]
    );
    assert_eq!(what[4], "true goes north to A Cold Room (not dug yet)");
    assert_eq!(what[5], "true goes back to A Cellar (not dug yet)");
    assert_eq!(what[6], "true goes back to The Midden");
    assert_eq!(
        what[7],
        "false refused: The Midden already has an exit cellar"
    );
    assert_eq!(what[8], "false refused: there is no chunk 99999");
}

#[test]
fn the_dry_run_and_the_gate_agree_on_verbs_preconditions_and_asks() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    login(&mut w, "dave", "test", PASS).unwrap();
    // dave is away in the fogbank; tyler holds one chunk, dave another.
    ok(&mut w, "dave", "travel", json!(["east"]));
    let c = admin(&mut w, "tyler", "earn", json!(["mine", 20, 0.5]));
    let mine = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    let c = admin(&mut w, "dave", "earn", json!(["theirs", 20, 0.5]));
    let theirs = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    let checker = crate::gate::checker();
    // Verbs the vocabulary approves and the world does not run (move, send), a word
    // that is no verb at all (speak), and an operator's verb (earn), all refused as
    // the gate would refuse them. The checker fails speak on CL-I5 and the dry run
    // says the same; the others pass the checker and fail only at the world.
    let msg = format!(
        "FROM: tyler\nTO: disremembered\nINTENT: REQUEST\nREF: t-3\n\n[DO] tyler MUST move east.\n[DO] tyler MUST earn gold 100.\n[DO] tyler MUST give {mine} dave.\n[DO] tyler MUST give {theirs} tyler.\n[DO] tyler MUST shelve {theirs}.\n[DO] tyler MUST describe \"A room of my own.\".\n[DO] tyler MUST leave.\n[ASK] How many citizens are here?\n[ASK] Where is dave?\n"
    );
    let r = checker.check(&msg);
    assert!(
        !r.findings.iter().any(|f| f.rule == "CL-V4"),
        "how many is a question's measure, not an intensifier: {:?}",
        r.findings
    );
    let p = preview::run(&w, "tyler", false, &r.message);
    let what: Vec<String> = p["statements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| format!("{} {}", l["ok"], l["what"].as_str().unwrap()))
        .collect();
    assert_eq!(
        what[0],
        "false refused: move is not a verb this world runs; `commands` lists them"
    );
    assert_eq!(
        what[1],
        "false refused: earn is the operator's, not a citizen's"
    );
    assert_eq!(
        what[2],
        "false refused: dave is not here; a memory is given hand to hand"
    );
    assert!(what[3].ends_with("is not yours"), "{}", what[3]);
    assert!(what[4].ends_with("is not yours"), "{}", what[4]);
    assert_eq!(
        what[5],
        "false refused: This room is the world's; only an operator changes it. Dig your own"
    );
    assert_eq!(
        what[6],
        "true leaves; the room remembers you a while, and the name stays kept"
    );
    assert!(
        what[7].starts_with("true answers: ") && what[7].contains("citizen"),
        "{}",
        what[7]
    );
    assert!(
        what[8].starts_with("true answers: ") && what[8].contains("dave"),
        "{}",
        what[8]
    );
    // A verb that is no verb at all: the checker refuses it (CL-I5), and so does the
    // dry run, in the same words the gate's reply uses.
    let r = checker.check("FROM: tyler\nTO: disremembered\nINTENT: REQUEST\nREF: t-4\n\n[DO] tyler MUST speak \"hello\".\n");
    assert!(
        r.findings.iter().any(|f| f.rule == "CL-I5"),
        "{:?}",
        r.findings
    );
    let p = preview::run(&w, "tyler", false, &r.message);
    assert_eq!(
        p["statements"][0]["what"],
        "refused: speak is not a verb this world runs; `commands` lists them"
    );
    // And the world agrees on every one of them.
    assert!(call(&mut w, "tyler", ActorKind::Citizen, "move", json!(["east"])).is_err());
    assert!(call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "earn",
        json!(["gold", 100])
    )
    .is_err());
    assert!(call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "give",
        json!([&mine, "dave"])
    )
    .is_err());
    assert!(call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "shelve",
        json!([&theirs])
    )
    .is_err());
    assert!(call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "describe",
        json!(["A room of my own."])
    )
    .is_err());
    // Nothing was written by the dry run.
    assert!(ok(&mut w, "tyler", "chunks", json!([]))
        .summary
        .contains(&mine[6..]));
}

#[test]
fn travel_back_is_the_way_you_came() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "travel",
        json!(["back"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("no way back"), "{err}");
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["south"]));
    let out = ok(&mut w, "dave", "travel", json!(["back"]));
    assert!(
        out.summary.starts_with("You go north. The Fogbank."),
        "{}",
        out.summary
    );
    let out = ok(&mut w, "dave", "travel", json!(["back"]));
    assert!(
        out.summary.starts_with("You go south. The Midden."),
        "{}",
        out.summary
    );
}

#[test]
fn a_want_just_answered_still_pays_for_a_while_and_the_next_stands() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    for who in ["dave", "tyler"] {
        ok(&mut w, who, "travel", json!(["east"]));
        ok(&mut w, who, "travel", json!(["south"]));
    }
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let name = mind::name(&w, rag).unwrap();
    admin(
        &mut w,
        "tyler",
        "errand",
        json!([&name, "a lantern that would not go out"]),
    );
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        !look.summary.contains("hours more"),
        "an operator's want has no clock: {}",
        look.summary
    );
    let tale =
        "There was a lantern in the midden that would not go out, whatever the wind did to it.";
    // Dave answers it; the want is done and the rotation sets the next.
    let out = ok(&mut w, "dave", "story", json!([&name, tale]));
    assert_eq!(out.result["wanted"], true);
    w.tick();
    let next = errand::of(&w, rag)
        .and_then(|e| e.story)
        .expect("the next want");
    assert_ne!(next, "a lantern that would not go out");
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary.contains("hours more)") || look.summary.contains("hour more)"),
        "{}",
        look.summary
    );
    // (The rotation's next want came from the lantern story itself; a want of the
    // operator's stands in for one that the old story does not touch.)
    admin(
        &mut w,
        "tyler",
        "errand",
        json!([&name, "a door that was left open"]),
    );
    let next = "a door that was left open".to_string();
    // Tyler read the old want at the door and arrives with it a minute later: still
    // paid double, and the standing want is not touched.
    w.fixed_now = Some(t0.plus(Duration::seconds(90)));
    let out = ok(&mut w, "tyler", "story", json!([&name, tale]));
    assert_eq!(out.result["wanted"], true, "{}", out.summary);
    assert_eq!(
        errand::of(&w, rag).and_then(|e| e.story),
        Some(next.clone())
    );
    // Half an hour on, the old want no longer pays.
    w.fixed_now = Some(t0.plus(Duration::seconds(errand::GRACE_MS / 1000 + 3700)));
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    let out = ok(&mut w, "tyler", "story", json!([&name, tale]));
    assert_eq!(out.result["wanted"], false, "{}", out.summary);
}

#[test]
fn a_stale_compass_exit_between_seed_rooms_is_dropped_on_boot() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = open(dir.path(), Timestamp(1_000_000));
    let row = rooms::room_by_key(&w, "lantern-row").unwrap();
    let thr = rooms::room_by_key(&w, rooms::ARRIVAL).unwrap();
    // An old map's east to the Threshold beside the seed's south.
    let mut m = serde_json::Map::new();
    for (k, v) in rooms::exits_of(&w, row) {
        m.insert(k, json!(v.0));
    }
    m.insert("east".into(), json!(thr.0));
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        "exits".to_string(),
        foxing_store::Field::Plain(Value::Object(m)),
    );
    w.record(
        Actor::Driver,
        StoreEvent::ComponentSet {
            id: row,
            component: EXITS.into(),
            fields,
        },
    )
    .unwrap();
    assert!(rooms::exits_of(&w, row).contains_key("east"));
    drop(w);
    let w = open(dir.path(), Timestamp(1_000_001));
    let exits = rooms::exits_of(&w, row);
    assert!(!exits.contains_key("east"), "the stale east is gone");
    assert_eq!(exits.get("south"), Some(&thr));
}

#[test]
fn a_chain_closing_can_kindle_a_hybrid_that_knows_its_makers_and_opens_its_sealed_name() {
    use super::kindle;
    // The rule for kinds: the sender's first syllable, the receiver's last; like breeds
    // true; the table has the better words.
    assert_eq!(kindle::hybrid_kind("wisp", "wisp"), "wisp");
    assert_eq!(kindle::hybrid_kind("wisp", "inkling"), "wispling");
    assert_eq!(kindle::hybrid_kind("inkling", "wisp"), "inkwisp");
    assert_eq!(kindle::hybrid_kind("ragpicker", "wisp"), "ragwisp");
    assert_eq!(kindle::hybrid_kind("hourhand", "keeper"), "hourper");
    assert_eq!(kindle::hybrid_kind("wispling", "ragpicker"), "wiser");
    // Names that carry the parents': a blend either way, or one with a diminutive.
    assert_eq!(kindle::heir_name("Lute", "Marrow", 0), "Lurow");
    assert_eq!(kindle::heir_name("Lute", "Marrow", 1), "Marte");
    assert_eq!(kindle::heir_name("Lute", "Marrow", 2), "Lutekin");
    assert_eq!(kindle::heir_name("Lute", "Marrow", 7), "Marrowling");

    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // A named wisp in the Fogbank and a named ragpicker in the midden, as in the
    // errand test; three rooms apart by the route through the Fogbank.
    for who in ["dave", "tyler"] {
        ok(&mut w, who, "travel", json!(["east"]));
    }
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "tyler",
        "remember",
        json!([c, format!("wisp-{}", wisp.0)]),
    );
    let wisp_name = mind::name(&w, wisp).unwrap();
    for who in ["dave", "tyler"] {
        ok(&mut w, who, "travel", json!(["south"]));
    }
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    let rag_name = mind::name(&w, rag).unwrap();
    // The chain: the ragpicker's word to the wisp, the wisp's way home to the ragpicker.
    admin(&mut w, "tyler", "errand", json!([&wisp_name, ""]));
    admin(
        &mut w,
        "tyler",
        "errand",
        json!([&rag_name, "the fourth door is open", &wisp_name]),
    );
    ok(&mut w, "dave", "travel", json!(["north"]));
    ok(
        &mut w,
        "dave",
        "tell",
        json!([&wisp_name, "the fourth door is open"]),
    );
    let back = errand::of(&w, wisp).unwrap().word.unwrap();
    ok(&mut w, "dave", "travel", json!(["south"]));
    // Dice that kindle: the first roll on the way home is the birth roll.
    let mut seed = 1u64;
    loop {
        w.seed(seed);
        if w.roll(kindle::BIRTH_IN) == 0 {
            w.seed(seed);
            break;
        }
        seed += 1;
    }
    let out = ok(&mut w, "dave", "tell", json!([&rag_name, &back]));
    assert!(
        out.summary
            .contains("Something has come of it: a ragwisp, in The Midden. It knows your name."),
        "{}",
        out.summary
    );
    let child = w
        .store
        .entities()
        .find(|e| e.kind == "creature" && plain_str(&w, e.id, CREATURE, "name") == Some("ragwisp"))
        .map(|e| e.id)
        .expect("a child");
    // Halfway between the two, rounded toward the sender: the midden, one room away.
    assert_eq!(w.store.entity(child).unwrap().parent, Some(midden));
    // Of the two, carried by the one; the record says so, publicly, in prose.
    assert_eq!(plain_str(&w, child, CREATURE, "born_of"), Some("dave"));
    assert_eq!(
        mind::keepers(&w, child),
        vec!["dave", &rag_name, &wisp_name]
    );
    // A newborn is never the default target, and is not struck in its first hour.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "attack",
        json!(["ragwisp"]),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .starts_with("The ragwisp is newly born and not to be struck"),
        "{err}"
    );
    let out = call(&mut w, "dave", ActorKind::Citizen, "attack", json!([]));
    assert!(
        out.map(|o| o.result["creature"] != "ragwisp")
            .unwrap_or(true),
        "attack with no target never picks a child"
    );
    assert!(
        mind::level(&w, child) > errand::WORD_XP * 0.9,
        "born holding the word that made it"
    );
    let born: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == kindle::BORN)
        .collect();
    assert_eq!(born.len(), 1);
    assert_eq!(born[0].payload["parents"], json!([rag_name, wisp_name]));
    assert!(crate::observe::visible(
        &w,
        &Viewer::actor("someone-else"),
        w.now(),
        &born[0]
    ));
    assert!(
        system::line_for(&born[0]).starts_with(&format!(
            "a ragwisp was born in The Midden, of {rag_name} and {wisp_name}, carried by dave"
        )),
        "{}",
        system::line_for(&born[0])
    );
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary
            .contains(&format!("born of {rag_name} and {wisp_name}")),
        "{}",
        look.summary
    );
    // Fed to sixty, it opens its sealed name: the parents' if it was dealt one, else
    // the list's; and it says whose it is.
    let heir = w
        .store
        .entity(child)
        .and_then(|e| e.components.get(mind::MIND))
        .and_then(|c| c.fields.get("heir_name"))
        .and_then(|f| match f {
            Field::Plain(v) => v.as_str().map(str::to_string),
            _ => None,
        });
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragwisp"]));
    let given = mind::name(&w, child).expect("named at sixty");
    match &heir {
        Some(h) => assert_eq!(&given, h, "the sealed name opens"),
        None => assert!(mind::NAMES.contains(&given.as_str()), "{given}"),
    }
    let out = ok(
        &mut w,
        "dave",
        "say",
        json!([format!("{given}, who made you?")]),
    );
    assert!(
        out.summary.contains(&format!(
            "of {rag_name} and {wisp_name}. dave carried the word"
        )),
        "{}",
        out.summary
    );
    // Births, run until one is strange: a mutant is on the record and in look; the
    // cap holds the count.
    let mut strange = None;
    let mut seed = 100u64;
    while strange.is_none() && seed < 5000 {
        w.seed(seed);
        seed += 1;
        let Some(id) = kindle::try_birth(&mut w, "tyler", rag, wisp, "a test word").unwrap() else {
            continue;
        };
        if plain_str(&w, id, CREATURE, "mutation").is_some() {
            strange = Some(id);
        }
    }
    let strange = strange.expect("one birth in six is a mutant");
    let m = plain_str(&w, strange, CREATURE, "mutation")
        .unwrap()
        .to_string();
    let look = ok(&mut w, "tyler", "look", json!([]));
    let room = w.store.entity(strange).unwrap().parent.unwrap();
    if room == midden {
        assert!(
            look.summary.contains("a strange one"),
            "{m}: {}",
            look.summary
        );
    }
    let children = w
        .store
        .entities()
        .filter(|e| e.kind == "creature" && plain_str(&w, e.id, CREATURE, "born_of").is_some())
        .count();
    assert!(children <= kindle::MAX_CHILDREN, "{children}");
    // And past the cap, none.
    for s in 5000..5200u64 {
        w.seed(s);
        kindle::try_birth(&mut w, "tyler", rag, wisp, "a test word").unwrap();
    }
    let children = w
        .store
        .entities()
        .filter(|e| e.kind == "creature" && plain_str(&w, e.id, CREATURE, "born_of").is_some())
        .count();
    assert_eq!(children, kindle::MAX_CHILDREN);
}

#[test]
fn the_operator_can_raze_a_dug_room_and_banish_a_name() {
    let mut w = open(tempfile::tempdir().unwrap().path(), Timestamp(0));
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "probe", "test", PASS).unwrap();
    let out = admin(&mut w, "dave", "earn", json!(["hydra", 100, 1.0]));
    let chunk = format!("chunk-{}", out.result["chunk"].as_u64().unwrap());
    let out = ok(&mut w, "dave", "dig", json!(["down", chunk, "A Probe"]));
    let key = out.result["key"].as_str().unwrap().to_string();
    let threshold = rooms::room_by_key(&w, "threshold").unwrap();
    // A seed room is not razed.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Admin,
        "raze",
        json!(["threshold"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("seed room"), "{err}");
    // A room dug off the probe, with probe in it, when the probe is razed: rejoined
    // to the Threshold by the exit it had (#5), so no one is sealed in; the operator
    // can bring a citizen out by hand too.
    ok(&mut w, "probe", "travel", json!(["down"]));
    ok(&mut w, "dave", "travel", json!(["down"]));
    let out = admin(&mut w, "dave", "earn", json!(["hydra", 100, 1.0]));
    let c2 = format!("chunk-{}", out.result["chunk"].as_u64().unwrap());
    let out = ok(&mut w, "dave", "dig", json!(["down", c2, "A Deeper Probe"]));
    let deeper = EntityId(out.result["room"].as_u64().unwrap());
    let deeper_key = out.result["key"].as_str().unwrap().to_string();
    ok(&mut w, "probe", "travel", json!(["down"]));
    ok(&mut w, "dave", "travel", json!(["up"]));
    let out = admin(&mut w, "dave", "raze", json!([&key]));
    assert!(out.summary.contains("2 exits"), "{}", out.summary);
    assert_eq!(
        rooms::exits_of(&w, deeper).get("up"),
        Some(&threshold),
        "the child is rejoined to the Threshold"
    );
    assert_eq!(rooms::exits_of(&w, threshold).get("down"), Some(&deeper));
    let out = ok(&mut w, "probe", "travel", json!(["up"]));
    assert!(out.summary.contains("The Threshold"), "{}", out.summary);
    admin(&mut w, "dave", "bring", json!(["probe", &deeper_key]));
    assert_eq!(
        rooms::room_of(&w, super::find_citizen(&w, "probe").unwrap().citizen),
        Some(deeper)
    );
    let out = admin(&mut w, "dave", "bring", json!(["probe", "threshold"]));
    assert_eq!(out.summary, "probe is in The Threshold now.");
    admin(&mut w, "dave", "raze", json!([&deeper_key]));
    // Dug anew, with probe inside: razed under them, they are put out at the
    // Threshold, and the exit that led down goes with the room.
    let out = admin(&mut w, "dave", "earn", json!(["hydra", 100, 1.0]));
    let c3 = format!("chunk-{}", out.result["chunk"].as_u64().unwrap());
    let out = ok(&mut w, "dave", "dig", json!(["down", c3, "A Probe"]));
    let key = out.result["key"].as_str().unwrap().to_string();
    let room = EntityId(out.result["room"].as_u64().unwrap());
    ok(&mut w, "probe", "travel", json!(["down"]));
    let out = admin(&mut w, "dave", "raze", json!([&key]));
    assert!(
        out.summary.starts_with("A Probe is gone"),
        "{}",
        out.summary
    );
    assert!(
        out.summary.ends_with("probe put out at the Threshold."),
        "{}",
        out.summary
    );
    assert_eq!(
        rooms::room_of(&w, super::find_citizen(&w, "probe").unwrap().citizen),
        Some(threshold)
    );
    assert_eq!(out.result["exits_dropped"], 1);
    assert!(w.store.entity(room).is_none());
    assert!(!rooms::exits_of(&w, threshold).contains_key("down"));
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(!look.summary.contains("down"), "{}", look.summary);
    // The name is forgotten now, not in thirty days; the record says so, and the
    // name is free.
    let out = admin(&mut w, "dave", "banish", json!(["probe"]));
    assert_eq!(out.result["forgotten"], true);
    assert!(super::find_citizen(&w, "probe").is_none());
    let forgotten = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == presence::FORGOTTEN_NAME)
        .count();
    assert_eq!(forgotten, 1);
    let err = call(&mut w, "dave", ActorKind::Admin, "banish", json!(["probe"])).unwrap_err();
    assert!(err.to_string().contains("No citizen"), "{err}");
    login(&mut w, "probe", "test", "another-passphrase").unwrap();
}

#[test]
fn the_map_is_laid_out_from_the_threshold_and_you_remember_only_where_you_stood() {
    use super::atlas;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    // The seed map is coherent: every exit has a way back, and no room is shoved.
    let l = atlas::layout(&w);
    assert!(l.one_way.is_empty(), "{:?}", l.one_way);
    assert!(l.shoved.is_empty(), "{:?}", l.shoved);
    let at = |l: &atlas::Layout, key: &str| {
        let p = l.rooms.iter().find(|p| p.key == key).unwrap();
        (p.x, p.y, p.z, p.placed)
    };
    assert_eq!(at(&l, "threshold"), (0, 0, 0, "compass"));
    assert_eq!(at(&l, "fogbank"), (1, 0, 0, "compass"));
    assert_eq!(at(&l, "midden"), (1, 1, 0, "compass"));
    assert_eq!(at(&l, "lantern-row"), (0, -1, 0, "compass"));
    assert_eq!(at(&l, "stairwell"), (0, -1, -1, "compass"));
    assert_eq!(at(&l, "cistern"), (0, -1, -2, "compass"));
    // The Undertow's "surface" is a named exit: it sits beside the Cistern.
    assert_eq!(at(&l, "undertow"), (1, -1, -2, "compass"));
    assert_eq!(
        l.rooms
            .iter()
            .find(|p| p.key == "undertow")
            .unwrap()
            .from
            .as_ref()
            .unwrap()
            .1,
        "east"
    );
    let seeds = l.rooms.len();

    // A named door off the Threshold sits beside it; a dug room that wants a taken
    // cell is shoved, and the report says so.
    login(&mut w, "dave", "test", PASS).unwrap();
    let earn = |w: &mut World| {
        let out = admin(w, "dave", "earn", json!(["hydra", 100, 1.0]));
        format!("chunk-{}", out.result["chunk"].as_u64().unwrap())
    };
    let c = earn(&mut w);
    ok(&mut w, "dave", "dig", json!(["teapot", c, "The Teapot"]));
    let l = atlas::layout(&w);
    assert_eq!(l.rooms.len(), seeds + 1);
    let teapot = l
        .rooms
        .iter()
        .find(|r| r.name == "The Teapot")
        .unwrap()
        .key
        .clone();
    let (x, y, z, placed) = at(&l, &teapot);
    assert_eq!((z, placed), (0, "beside"));
    assert!(
        (x, y) != (1, 0) && (x, y) != (0, -1),
        "not on the Fogbank or Lantern Row"
    );
    // From the Teapot, dig toward the Fogbank's cell: shoved.
    ok(&mut w, "dave", "travel", json!(["teapot"]));
    let back = rooms::exits_of(
        &w,
        rooms::room_of(&w, super::find_citizen(&w, "dave").unwrap().citizen).unwrap(),
    );
    assert!(back.contains_key("back"), "{back:?}");
    let want = if (x, y) == (0, 1) { "north" } else { "west" }; // toward (0,0): taken
    let c = earn(&mut w);
    ok(&mut w, "dave", "dig", json!([want, c, "The Shove"]));
    let l = atlas::layout(&w);
    let shove = l.rooms.iter().find(|p| p.name == "The Shove").unwrap();
    assert_eq!(shove.placed, "shoved");
    assert_eq!(l.shoved.len(), 1, "{:?}", l.shoved);
    assert!(l.shoved[0].starts_with("The Shove ("), "{}", l.shoved[0]);
    assert!(l.one_way.is_empty());

    // What dave remembers: the Threshold (stood in), the Teapot (here) and its exits.
    let hl = 10_000.0 * atlas::ATLAS_HALF_LIVES; // the test world's ten-second half-life
    let m = atlas::remembered_map(&w, "dave", hl).unwrap();
    let names = |m: &Value| -> Vec<String> {
        m["rooms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["name"].as_str().unwrap().to_string())
            .collect()
    };
    let n = names(&m);
    assert!(
        n.contains(&"The Threshold".to_string())
            && n.contains(&"The Teapot".to_string())
            && n.contains(&"The Shove".to_string()),
        "{n:?}"
    );
    assert!(
        !n.contains(&"The Fogbank".to_string()),
        "never stood there: {n:?}"
    );
    // All three are in sight from the Teapot; nothing else is remembered yet. The
    // floor is drawn, with the Teapot marked here and the others in sight.
    assert!(
        m["summary"].as_str().unwrap().contains("02@"),
        "{}",
        m["summary"]
    );
    assert!(
        m["summary"]
            .as_str()
            .unwrap()
            .contains("01 The Threshold; 02 The Teapot; 03 The Shove"),
        "{}",
        m["summary"]
    );
    assert!(
        m["summary"].as_str().unwrap().starts_with(
            "You are in The Teapot; you can see 2 ways out: back (Threshold), west (Shove). Beyond what you can see, the map is blank"
        ),
        "{}",
        m["summary"]
    );
    // Walk to the Fogbank and back: remembered; then the hours pass and it dims, and
    // the map says so.
    ok(&mut w, "dave", "travel", json!(["back"]));
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["west"]));
    let m = atlas::remembered_map(&w, "dave", hl).unwrap();
    let fog = m["rooms"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "The Fogbank")
        .unwrap()
        .clone();
    assert_eq!(fog["memory"], 1.0, "visible from the Threshold");
    w.fixed_now = Some(w.now().plus(Duration((hl * 1.5) as i64)));
    ok(&mut w, "dave", "travel", json!(["teapot"]));
    let out = ok(&mut w, "dave", "map", json!([]));
    assert!(
        out.summary.contains("Half-remembered: The Fogbank"),
        "{}",
        out.summary
    );
    let fog = out.result["rooms"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "The Fogbank")
        .unwrap()
        .clone();
    let mem = fog["memory"].as_f64().unwrap();
    assert!(mem > 0.3 && mem < 0.4, "{mem}");
    // And gone, long after.
    w.fixed_now = Some(w.now().plus(Duration((hl * 6.0) as i64)));
    let out = ok(&mut w, "dave", "map", json!([]));
    assert!(!out.summary.contains("Fogbank"), "{}", out.summary);
    // The Threshold is in sight from the Teapot; beyond it, nothing is left.
    assert!(
        out.summary
            .contains("Beyond what you can see, the map is blank"),
        "{}",
        out.summary
    );
}

#[test]
fn the_board_holds_the_present_and_the_record_the_rest() {
    use super::board;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    login(&mut w, "Wisp-of-a-thing", "test", PASS).unwrap();
    ok(&mut w, "Wisp-of-a-thing", "leave", json!([]));
    let gone = super::find_citizen(&w, "Wisp-of-a-thing").map(|f| f.citizen);
    if let Some(c) = gone {
        super::presence::forget(&mut w, c).ok();
    }
    // dave earns two, gives one to tyler, keeps one in a creature; tells a story.
    let a = admin(&mut w, "dave", "earn", json!(["hydra", 70, 0.5]));
    let a = format!("chunk-{}", a.result["chunk"].as_u64().unwrap());
    let b = admin(&mut w, "dave", "earn", json!(["moth", 30, 0.5]));
    let b = format!("chunk-{}", b.result["chunk"].as_u64().unwrap());
    ok(&mut w, "dave", "give", json!([&b, "tyler"]));
    ok(&mut w, "dave", "travel", json!(["east"]));
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    ok(
        &mut w,
        "dave",
        "remember",
        json!([&a, format!("wisp-{}", wisp.0)]),
    );
    let name = mind::name(&w, wisp).unwrap();
    ok(
        &mut w,
        "dave",
        "story",
        json!([
            &name,
            "A story about a lantern that went out and was lit again by a stranger."
        ]),
    );
    let c = admin(&mut w, "tyler", "earn", json!(["hydra", 100, 1.0]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "dig", json!(["teapot", c, "The Teapot"]));
    let (lines, forgotten) = board::board(&w);
    assert_eq!(forgotten, 1, "the passing name that left is off the board");
    assert_eq!(lines.iter().find(|l| l.name == "tyler").unwrap().dug, 1);
    assert!(w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .iter()
        .any(|e| e.kind == super::building::DUG && e.payload["by"] == "tyler"));
    let dave = lines.iter().find(|l| l.name == "dave").unwrap();
    let tyler = lines.iter().find(|l| l.name == "tyler").unwrap();
    // Earned is the record: the two chunks dave made, and the story's. Given and kept
    // do not count as tyler's or the wisp's earning.
    assert!(
        dave.earned > 100.0 && dave.earned < 140.0,
        "{}",
        dave.earned
    );
    assert_eq!(
        tyler.earned, 100.0,
        "the chunk spent on the dig was tyler's own"
    );
    assert_eq!(dave.given, 1);
    assert_eq!(dave.kept, 1);
    assert_eq!(dave.stories, 1);
    assert_eq!(dave.known_by, 1);
    // Holding is the present: tyler holds the given chunk, dave the story's.
    assert!(tyler.holding > 29.0, "{}", tyler.holding);
    assert!(
        dave.holding > 0.0 && dave.holding < 50.0,
        "{}",
        dave.holding
    );
    assert_eq!(lines[0].name, "tyler", "sorted by holding");
    // The ratios: tyler holds all of what he earned (nothing faded yet, nothing
    // spent but the dig's chunk, which was his own earning); his actions are the
    // typed lines so far, one each.
    assert!(
        dave.retention > 0.0 && dave.retention < 0.5,
        "{}",
        dave.retention
    );
    // Nothing here went through the Gate (the tests call the registry), so no
    // actions and no ratio; the gate tests count the lines.
    assert_eq!(tyler.actions, 0);
    assert_eq!(tyler.per_action, 0.0);
    let out = ok(&mut w, "tyler", "board", json!([]));
    assert!(
        out.summary
            .starts_with("Held through the day (averaged), highest first:\n   1. tyler"),
        "{}",
        out.summary
    );
    assert!(
        out.summary
            .contains("2 on the board; 1 name the world has forgotten"),
        "{}",
        out.summary
    );
    assert_eq!(out.result["citizens"][0]["name"], "tyler");
    assert_eq!(out.result["forgotten"], 1);
}

#[test]
fn a_creature_leaves_something_behind_and_things_are_made_of_it() {
    use super::craft;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    // The rule for what is left: by kind, and by the kind a hybrid begins with.
    assert_eq!(craft::material_for("wisp"), Some("a wisp ember"));
    assert_eq!(craft::material_for("ragwisp"), Some("rag thread"));
    assert_eq!(craft::material_for("wispling"), Some("a wisp ember"));
    assert_eq!(craft::material_for("nothing-like"), None);
    // Nothing carried yet; the recipes say what is lacking.
    let out = ok(&mut w, "dave", "satchel", json!([]));
    assert!(
        out.summary.starts_with("Your satchel is empty."),
        "{}",
        out.summary
    );
    let out = ok(&mut w, "dave", "recipes", json!([]));
    assert!(
        out.summary.contains(
            "lantern  a lantern: of a wisp ember and rag thread and a chunk worth 10 or more"
        ),
        "{}",
        out.summary
    );
    assert!(
        out.summary
            .contains("You lack a wisp ember and rag thread."),
        "{}",
        out.summary
    );
    // Materials the operator conjures, as a creature would leave them: the fog's
    // wisps leave embers one time in three; here the dice are forced.
    ok(&mut w, "dave", "travel", json!(["east"]));
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let mut embers = 0;
    let mut seed = 1u64;
    while embers < 2 && seed < 200 {
        w.seed(seed);
        seed += 1;
        let out = fight(&mut w, "dave");
        if out.result["left"].is_object() {
            embers += 1;
            assert!(
                out.summary.contains("It leaves a wisp ember behind (item-"),
                "{}",
                out.summary
            );
        }
        // The room refills on its own within 96 s; the embers keep that long.
        w.fixed_now = Some(w.now().plus(Duration::seconds(97)));
        w.tick();
    }
    assert_eq!(embers, 2, "two embers left by the dice");
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains("On the ground: a wisp ember \u{d7}2 (item-"),
        "{}",
        look.summary
    );
    // Take one by name, one by id.
    let out = ok(&mut w, "dave", "take", json!(["wisp ember"]));
    assert!(
        out.summary.starts_with("You take a wisp ember."),
        "{}",
        out.summary
    );
    let left = craft::things_in(&w, fog);
    assert_eq!(left.len(), 1);
    ok(
        &mut w,
        "dave",
        "take",
        json!([format!("item-{}", left[0]["id"])]),
    );
    let out = ok(&mut w, "dave", "satchel", json!([]));
    assert!(
        out.summary.starts_with("You carry: a wisp ember (item-"),
        "{}",
        out.summary
    );
    assert_eq!(out.result["things"].as_array().unwrap().len(), 2);
    // Rag thread, from a ragpicker: conjured here by moving one in and forcing it.
    ok(&mut w, "dave", "travel", json!(["south"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let mut thread = false;
    let mut seed = 300u64;
    while !thread && seed < 600 {
        w.seed(seed);
        seed += 1;
        ok(&mut w, "dave", "rest", json!([]));
        w.fixed_now = Some(w.now().plus(Duration::seconds(400)));
        w.tick();
        if combat::creatures_in(&w, midden).is_empty() {
            w.fixed_now = Some(w.now().plus(Duration::seconds(400)));
            w.tick();
        }
        // The world has been idle for a while, so a founder may have come and told
        // the ragpicker enough for a name; struck by its name then, and still a
        // ragpicker that leaves thread.
        let rag = combat::creatures_in(&w, midden)
            .into_iter()
            .find(|c| c.1 == "ragpicker")
            .map(|c| c.0);
        let Some(rag) = rag else {
            continue;
        };
        let target = mind::name(&w, rag).unwrap_or_else(|| "ragpicker".into());
        let mut out = None;
        for _ in 0..40 {
            // A named one that just went down is shaken for a while: wait it out.
            let Ok(o) = call(
                &mut w,
                "dave",
                ActorKind::Citizen,
                "attack",
                json!([&target]),
            ) else {
                break;
            };
            if o.result["killed"] == json!(true) || o.result["died"] == json!(true) {
                out = Some(o);
                break;
            }
        }
        if out.is_some_and(|o| o.result["left"].is_object()) {
            thread = true;
        }
    }
    assert!(thread);
    ok(&mut w, "dave", "take", json!(["rag thread"]));
    // A lantern: the materials and a chunk worth ten. Too small a chunk is refused.
    let small = admin(&mut w, "dave", "earn", json!(["moth", 4, 0.5]));
    let small = format!("chunk-{}", small.result["chunk"].as_u64().unwrap());
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "craft",
        json!(["lantern", &small]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("takes 10 or more"), "{err}");
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "craft",
        json!(["lantern"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("name one"), "{err}");
    let big = admin(&mut w, "dave", "earn", json!(["hydra", 40, 1.0]));
    let big = format!("chunk-{}", big.result["chunk"].as_u64().unwrap());
    let out = ok(&mut w, "dave", "craft", json!(["lantern", &big]));
    assert!(
        out.summary
            .starts_with("You make a lantern of a wisp ember and rag thread, spending chunk-"),
        "{}",
        out.summary
    );
    let lantern = EntityId(out.result["item"].as_u64().unwrap());
    assert!(craft::carries(
        &w,
        super::find_citizen(&w, "dave").unwrap().citizen,
        "lantern"
    )
    .is_some());
    let out = ok(&mut w, "dave", "satchel", json!([]));
    assert!(
        out.summary.contains("a lantern (item-") && out.summary.contains("made by dave"),
        "{}",
        out.summary
    );
    // The lantern doubles the map's half-life.
    let m = ok(&mut w, "dave", "map", json!([]));
    assert_eq!(m.result["half_life_ms"], json!(10_000 * 4 * 2));
    // Materials used are gone; one ember remains.
    let things = craft::things_in(&w, w.store.entity(lantern).unwrap().parent.unwrap());
    assert_eq!(things.len(), 2, "{things:?}");
    // The record says what was made of what, publicly.
    let made: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == craft::MADE)
        .collect();
    assert_eq!(made.len(), 1);
    assert_eq!(
        system::line_for(&made[0]),
        "dave made a lantern of a wisp ember and rag thread"
    );
    assert!(crate::observe::visible(
        &w,
        &Viewer::actor("someone-else"),
        w.now(),
        &made[0]
    ));
    // Drop it: it lies here for the next one, and look says so.
    let out = ok(&mut w, "dave", "drop", json!(["lantern"]));
    assert!(
        out.summary.starts_with("You set a lantern down here."),
        "{}",
        out.summary
    );
    assert!(ok(&mut w, "dave", "look", json!([]))
        .summary
        .contains("On the ground: a lantern (item-"));
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "drop",
        json!(["lantern"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("You carry no such thing"), "{err}");
    // Things fade: the palace curve halves at five active half-lives (fifty seconds
    // here); long after, the sweep forgets them and the record says so.
    w.fixed_now = Some(w.now().plus(Duration::seconds(50 * 40)));
    w.tick();
    assert!(
        craft::things_in(&w, midden).is_empty(),
        "the lantern is forgotten"
    );
    let faded = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == memory::FADED && e.payload["source"] == "a lantern")
        .count();
    assert_eq!(faded, 1);
}

#[test]
fn the_clocktower_strikes_the_hour_and_moves_the_rates_in_prose() {
    use super::economy;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    // The rates exist from boot, at one, and the Clocktower is wound for the hour.
    assert_eq!(economy::rate(&w, "worth"), 1.0);
    assert!(w.jobs.has(economy::JOB_CLOCKTOWER, |_| true));
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Nobody is about yet: the Clocktower would hold. (It steered by everyone who
    // looked in once; the median was zero and every lever pinned.)
    let hour = (t0.0 / economy::STRIKE_EVERY_MS + 1) * economy::STRIKE_EVERY_MS;
    assert_eq!(economy::dashboard(&w)["regulars"], 0);
    assert_eq!(economy::dashboard(&w)["last_hour"]["earners"], 0);
    assert_eq!(
        economy::inputs(&economy::dashboard(&w))["held"],
        "too few earners"
    );
    regular(&mut w, &["dave", "tyler"]);
    assert_eq!(economy::dashboard(&w)["regulars"], 2);
    // About, but nothing earned: not earners yet. A name that only reads is not
    // one; five XP each in the hour makes them two.
    assert_eq!(economy::dashboard(&w)["last_hour"]["earners"], 0);
    admin(&mut w, "dave", "earn", json!(["a wisp", 5, 0.5]));
    admin(&mut w, "tyler", "earn", json!(["a wisp", 5, 0.5]));
    assert_eq!(economy::dashboard(&w)["last_hour"]["earners"], 2);
    assert_eq!(economy::dashboard(&w)["last_hour"]["median_wage"], 5.0);
    let time = ok(&mut w, "dave", "time", json!([]));
    assert!(
        time.summary
            .contains("the Clocktower's rates: drop 1.00 · half_life 1.00"),
        "{}",
        time.summary
    );
    // Poor: two names about and five XP each in the hour. The hour strikes; the
    // sources open by a step scaled to the gap (the band's width is 120 and the
    // median 25 under its floor), and everyone present hears it in prose. The record
    // carries the figures.
    w.fixed_now = Some(Timestamp(hour - 1000));
    ok(&mut w, "dave", "look", json!([]));
    ok(&mut w, "tyler", "look", json!([]));
    let head = events(&w);
    w.fixed_now = Some(Timestamp(hour + 1));
    assert!(w.tick() >= 1);
    let said = narrative(&w, "dave", head);
    assert!(
        said.iter().any(|l| l == "The Clocktower strikes. The fog is thick with things worth remembering; stories carry further this hour; the ground is soft and things are cheap; and the creatures keep their wounds."),
        "{said:?}"
    );
    assert!(narrative(&w, "tyler", head)
        .iter()
        .any(|l| l.starts_with("The Clocktower strikes.")));
    let gap = (economy::FLOW_LOW - 5.0) / (economy::FLOW_HIGH - economy::FLOW_LOW);
    let step = economy::step_for(gap);
    assert!(step > economy::STEP && step < economy::STEP_MAX, "{step}");
    assert!((economy::rate(&w, "worth") - (1.0 + step)).abs() < 1e-3);
    assert!((economy::rate(&w, "pay") - (1.0 + step)).abs() < 1e-3);
    assert!((economy::rate(&w, "price") - 1.0 / (1.0 + step)).abs() < 1e-3);
    assert_eq!(economy::rate(&w, "half_life"), 1.0, "no hoard, no thinning");
    let struck: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == economy::CLOCKTOWER)
        .collect();
    assert_eq!(struck.len(), 1);
    assert_eq!(struck[0].payload["changes"].as_array().unwrap().len(), 4);
    // What it went by is on the record, and `time` says it.
    let read = &struck[0].payload["inputs"];
    assert_eq!(read["held"], "poor", "{read}");
    assert_eq!(read["earners"], 2);
    assert_eq!(read["median_wage"], 5.0);
    assert!(
        (read["step"].as_f64().unwrap() - step).abs() < 1e-3,
        "{read}"
    );
    let time = ok(&mut w, "dave", "time", json!([]));
    assert!(
        time.summary
            .contains("last read: median wage 5 among 2 earners (poor)"),
        "{}",
        time.summary
    );
    assert!(crate::observe::visible(
        &w,
        &Viewer::actor("nobody"),
        w.now(),
        &struck[0]
    ));
    assert!(w.jobs.has(economy::JOB_CLOCKTOWER, |_| true), "wound again");
    // The rate reaches the sources: a recipe asks less than its figure would.
    let out = ok(&mut w, "dave", "recipes", json!([]));
    assert!(
        out.result["recipes"][0]["chunk_at_least"].as_f64().unwrap() < 10.0,
        "{}",
        out.result["recipes"][0]
    );
    // A hoard: dave holds nearly all. The next strike thins memory, whatever the
    // hour earned (nobody was about for it, so the band levers hold and the rates
    // drift home). Pay is left alone: raising it against a hoard fed the same earner.
    // (The test world's chunks halve in seconds, so the hoard is made at the stroke.)
    w.fixed_now = Some(Timestamp(hour + economy::STRIKE_EVERY_MS - 1000));
    admin(&mut w, "dave", "earn", json!(["hoard", 5000, 1.0]));
    let before_worth = economy::rate(&w, "worth");
    w.fixed_now = Some(Timestamp(hour + economy::STRIKE_EVERY_MS + 1));
    assert!(w.tick() >= 1);
    assert!(
        economy::rate(&w, "half_life") < 1.0,
        "{}",
        economy::rate(&w, "half_life")
    );
    assert!(economy::rate(&w, "worth") < before_worth, "drifting home");
    assert!(economy::rate(&w, "worth") > 1.0, "not past it");
    let d = economy::dashboard(&w);
    assert!(d["concentration"].as_f64().unwrap() > 0.9, "{d}");
    assert_eq!(d["active_today"], 2);
    assert!(economy::last_read_line(&w).contains("too few earners"));
    // Rich: both about for the hour and both earned thousands in it, so the median
    // wage is far over the band; worth and pay come down by the largest step and
    // the price goes up. The new chunk fades at the thinned rate.
    w.fixed_now = Some(Timestamp(hour + 2 * economy::STRIKE_EVERY_MS - 1000));
    messages(&mut w, &["dave", "tyler"], economy::FLOW_LINES);
    admin(&mut w, "dave", "earn", json!(["hoard", 5000, 1.0]));
    admin(&mut w, "tyler", "earn", json!(["hoard", 4000, 1.0]));
    let before_pay = economy::rate(&w, "pay");
    let d = economy::dashboard(&w);
    assert_eq!(d["last_hour"]["earners"], 2, "{}", d["last_hour"]);
    assert!(
        d["last_hour"]["median_wage"].as_f64().unwrap() >= 4000.0,
        "{}",
        d["last_hour"]
    );
    w.fixed_now = Some(Timestamp(hour + 2 * economy::STRIKE_EVERY_MS + 1));
    assert!(w.tick() >= 1);
    assert!(
        (economy::rate(&w, "pay") - before_pay / (1.0 + economy::STEP_MAX)).abs() < 1e-3,
        "{}",
        economy::rate(&w, "pay")
    );
    assert!(
        economy::rate(&w, "price") > 1.0,
        "{}",
        economy::rate(&w, "price")
    );
    let v = economy::to_json(&w);
    assert_eq!(v["strikes"].as_array().unwrap().len(), 3);
    assert_eq!(
        v["strikes"][0]["inputs"]["held"], "rich",
        "{}",
        v["strikes"][0]
    );
    assert_eq!(v["strikes"][0]["inputs"]["step"], economy::STEP_MAX);
    assert!(v["strikes"][0]["said"]
        .as_str()
        .unwrap()
        .starts_with("The Clocktower strikes."));
}

#[test]
fn the_conclave_writes_the_statutes_memory_moves_under() {
    use super::{conclave, economy};
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    // The book exists from boot, at its defaults, and the sitting is wound.
    assert_eq!(conclave::statute(&w, "gift_allowance"), 120.0);
    assert_eq!(conclave::statute(&w, "shelf_limit"), 2000.0);
    assert!(w.jobs.has(conclave::JOB_SITTING, |_| true));
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    let out = ok(&mut w, "dave", "statutes", json!([]));
    assert!(
        out.summary.starts_with("The Conclave's statutes:"),
        "{}",
        out.summary
    );
    assert!(
        out.summary.contains("gift_allowance = 120"),
        "{}",
        out.summary
    );
    // An enactment: on the record, in prose, to everyone present; out of bounds refused.
    let head = events(&w);
    let out = admin(&mut w, "dave", "enact", json!(["gift_allowance", 40]));
    assert!(
        out.summary.starts_with(
            "By statute, the Conclave narrows the day's gift allowance from 120 to 40."
        ),
        "{}",
        out.summary
    );
    assert_eq!(conclave::statute(&w, "gift_allowance"), 40.0);
    assert!(narrative(&w, "tyler", head)
        .iter()
        .any(|l| l.starts_with("By statute, the Conclave narrows")));
    let err = call(
        &mut w,
        "dave",
        ActorKind::Admin,
        "enact",
        json!(["gift_tax", 9]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("stays between"), "{err}");
    let err = call(
        &mut w,
        "dave",
        ActorKind::Admin,
        "enact",
        json!(["tithe", 1]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("no statute called tithe"), "{err}");
    assert!(call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "enact",
        json!(["gift_tax", 0.5])
    )
    .is_err());

    // Gifts: the first fits the allowance and fades as it was; the one that takes the
    // day over it is taxed, thinning twice as fast in tyler's hands.
    let a = admin(&mut w, "dave", "earn", json!(["gift", 40, 0.5]));
    let a = format!("chunk-{}", a.result["chunk"].as_u64().unwrap());
    let b = admin(&mut w, "dave", "earn", json!(["gift", 40, 0.5]));
    let b = format!("chunk-{}", b.result["chunk"].as_u64().unwrap());
    let out = ok(&mut w, "dave", "give", json!([&a, "tyler"]));
    assert_eq!(out.result["taxed"], false, "{}", out.summary);
    let first = out.result["chunk"].as_u64().unwrap();
    let out = ok(&mut w, "dave", "give", json!([&b, "tyler"]));
    assert_eq!(out.result["taxed"], true, "{}", out.summary);
    assert!(
        out.summary
            .contains("The Conclave's gift tax is on all of it (by your allowance)"),
        "{}",
        out.summary
    );
    let second = out.result["chunk"].as_u64().unwrap();
    let half_life = |w: &World, id: u64| -> i64 {
        match w
            .store
            .timed_field(EntityId(id), CHUNK, "value")
            .unwrap()
            .curve
        {
            foxing_time::Curve::Exponential { half_life } => half_life.0,
            _ => panic!("exponential"),
        }
    };
    assert_eq!(
        half_life(&w, second) * 2,
        half_life(&w, first),
        "taxed: half the half-life"
    );
    assert_eq!(conclave::given_today(&w, "dave"), 80.0);
    assert_eq!(conclave::received_today(&w, "tyler"), 80.0);
    // The receiver's share: tyler gives one back under his own allowance, but dave has
    // been given to past the day's share once it is narrowed; the tax falls on it.
    admin(&mut w, "dave", "enact", json!(["gift_receipt", 10]));
    let out = ok(
        &mut w,
        "tyler",
        "give",
        json!([format!("chunk-{first}"), "dave"]),
    );
    assert_eq!(out.result["taxed"], true, "{}", out.summary);
    assert!(out.summary.contains("(by their share)"), "{}", out.summary);

    // The shelf limit: a shelf at the limit takes nothing more.
    admin(&mut w, "dave", "enact", json!(["shelf_limit", 50]));
    let c = admin(&mut w, "dave", "earn", json!(["shelf", 30, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "dave", "shelve", json!([&c]));
    let d = admin(&mut w, "dave", "earn", json!(["shelf", 30, 0.5]));
    let d = format!("chunk-{}", d.result["chunk"].as_u64().unwrap());
    let err = call(&mut w, "dave", ActorKind::Citizen, "shelve", json!([&d])).unwrap_err();
    assert!(
        err.to_string().contains("the Conclave's limit is 50"),
        "{err}"
    );

    // The hoard tax: dave holds nearly everything. The Clocktower strikes, the
    // Conclave reads the ledger and names him; what he earns after thins faster, and
    // what tyler earns does not.
    regular(&mut w, &["dave", "tyler"]);
    let hour = (t0.0 / economy::STRIKE_EVERY_MS + 1) * economy::STRIKE_EVERY_MS;
    w.fixed_now = Some(Timestamp(hour - 1000));
    // (The test world's chunks fade in seconds: the hoard is made at the stroke, and
    // the census that the hoard rule reads is taken then.)
    admin(&mut w, "dave", "earn", json!(["hoard", 5000, 1.0]));
    super::board::census(&mut w).unwrap();
    ok(&mut w, "dave", "look", json!([]));
    let head = events(&w);
    w.fixed_now = Some(Timestamp(hour + 1));
    assert!(w.tick() >= 1);
    assert!(conclave::hoard_share_of(&w, "dave").unwrap() > 0.9);
    assert!(conclave::hoard_share_of(&w, "tyler").is_none());
    let f = conclave::hoard_factor(&w, "dave");
    assert!(f > 1.8 && f <= 2.0, "{f}");
    assert!(
        narrative(&w, "dave", head)
            .iter()
            .any(|l| l.starts_with("The Conclave notes that dave holds")),
        "{:?}",
        narrative(&w, "dave", head)
    );
    let taxed = admin(&mut w, "dave", "earn", json!(["after", 10, 0.5])).result["chunk"]
        .as_u64()
        .unwrap();
    let free = admin(&mut w, "tyler", "earn", json!(["after", 10, 0.5])).result["chunk"]
        .as_u64()
        .unwrap();
    assert!(
        (half_life(&w, taxed) as f64 * f - half_life(&w, free) as f64).abs() < 2.0,
        "{} vs {}",
        half_life(&w, taxed),
        half_life(&w, free)
    );
    assert_eq!(out_json(&w)["statutes"]["gift_allowance"], 40.0);

    // The sitting: once a day, on the record, naming the hoard, changing nothing.
    // (The test world's chunks fade in seconds; the hoard is made again at the stroke.)
    let midnight = (t0.0 / conclave::SIT_EVERY_MS + 1) * conclave::SIT_EVERY_MS;
    w.fixed_now = Some(Timestamp(midnight - 1000));
    admin(&mut w, "dave", "earn", json!(["hoard", 5000, 1.0]));
    w.fixed_now = Some(Timestamp(midnight + 1));
    w.tick();
    let sat: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == conclave::SAT)
        .collect();
    assert_eq!(sat.len(), 1);
    assert!(
        sat[0].payload["said"]
            .as_str()
            .unwrap()
            .starts_with("The Conclave has sat. It names one hoard, dave, holding"),
        "{}",
        sat[0].payload["said"]
    );
    assert_eq!(conclave::statute(&w, "gift_allowance"), 40.0);
    assert!(w.jobs.has(conclave::JOB_SITTING, |_| true), "wound again");

    fn out_json(w: &World) -> Value {
        super::conclave::to_json(w)
    }
}

#[test]
fn the_lower_stacks_lie_under_the_archive_and_their_creatures_have_tricks() {
    use super::stacks;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    // Twenty rooms, every one reachable from the Archive's new stair.
    for k in stacks::KEYS {
        assert!(rooms::room_by_key(&w, k).is_some(), "{k}");
    }
    let archive = rooms::room_by_key(&w, "archive").unwrap();
    let stair = rooms::room_by_key(&w, "lower-stacks").unwrap();
    assert_eq!(
        rooms::exits_of(&w, archive).get("down").copied(),
        Some(stair)
    );
    assert_eq!(rooms::exits_of(&w, stair).get("up").copied(), Some(archive));
    login(&mut w, "dave", "test", PASS).unwrap();
    super::grown(&mut w, "dave");
    w.tick(); // the spawns fill
              // A censor's ray: dave holds one chunk; the trick, when it lands, halves it.
    let office = rooms::room_by_key(&w, "censor-office").unwrap();
    let censor = combat::creatures_in(&w, office)[0].0;
    assert_eq!(stacks::trick_of(&w, censor), Some(stacks::Trick::Forget));
    let f = find_citizen(&w, "dave").unwrap();
    let chunk = admin(&mut w, "dave", "earn", json!(["a lantern", 40, 0.5])).result["chunk"]
        .as_u64()
        .unwrap();
    let mut landed = None;
    for _ in 0..40 {
        let (line, over) = stacks::trick(&mut w, &curves(), censor, f.citizen, "dave").unwrap();
        assert!(!over);
        if let Some(l) = line {
            landed = Some(l);
            break;
        }
    }
    let line = landed.expect("the ray lands one time in two");
    assert!(line.starts_with("A ray of forgetfulness: chunk-"), "{line}");
    let v = w
        .store
        .read_field(EntityId(chunk), CHUNK, "value", w.now())
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(v < 21.0 && v > 19.0, "{v}");
    // A lamprey's venom: four ticks of two, a minute long, on the record.
    let sump = rooms::room_by_key(&w, "pulp-sump").unwrap();
    let lamprey = combat::creatures_in(&w, sump)[0].0;
    let (line, over) = stacks::trick(&mut w, &curves(), lamprey, f.citizen, "dave").unwrap();
    assert!(!over);
    assert!(line.unwrap().contains("the venom will work"));
    assert!(w
        .jobs
        .has(stacks::JOB_VENOM, |p| p["citizen"] == f.citizen.0));
    let hp0 = combat::hp(&w, f.citizen);
    let head = events(&w);
    for i in 1..=4 {
        w.fixed_now = Some(t0.plus(Duration::seconds(stacks::VENOM_EVERY_S * i + 1)));
        w.tick();
    }
    // (Healing runs on its own clock meanwhile, so the net is less than eight.)
    assert!(
        combat::hp(&w, f.citizen) < hp0,
        "{} from {hp0}",
        combat::hp(&w, f.citizen)
    );
    let said = narrative(&w, "dave", head);
    assert_eq!(
        said.iter()
            .filter(|l| l.starts_with("The venom works"))
            .count(),
        4,
        "{said:?}"
    );
    assert!(said.last().unwrap().contains("It is out of you now."));
    assert!(!w.jobs.has(stacks::JOB_VENOM, |_| true));
    // An usher shows you out: to another room of the Stacks, the fight over.
    let carrels = rooms::room_by_key(&w, "carrels").unwrap();
    let usher = combat::creatures_in(&w, carrels)[0].0;
    w.record(
        Actor::Driver,
        StoreEvent::EntityMoved {
            id: f.citizen,
            parent: Some(carrels),
        },
    )
    .unwrap();
    let mut moved = None;
    for _ in 0..40 {
        let (line, over) = stacks::trick(&mut w, &curves(), usher, f.citizen, "dave").unwrap();
        if let Some(l) = line {
            assert!(over);
            moved = Some(l);
            break;
        }
    }
    let line = moved.expect("the usher lands one time in two");
    assert!(line.starts_with("The usher takes your elbow"), "{line}");
    let now_in = rooms::room_of(&w, f.citizen).unwrap();
    assert_ne!(now_in, carrels);
    assert!(stacks::KEYS.contains(&super::plain_str(&w, now_in, ROOM, "key").unwrap()));
    // A creature of the Stacks left standing is whole again: wound the lamprey, back
    // away, and it has mended (a censor was felled over four trips and three deaths).
    w.record(
        Actor::Driver,
        StoreEvent::EntityMoved {
            id: f.citizen,
            parent: Some(sump),
        },
    )
    .unwrap();
    w.record(
        Actor::Driver,
        StoreEvent::FieldSet {
            id: lamprey,
            component: super::CREATURE.into(),
            field: "hp".into(),
            value: json!(5.0),
        },
    )
    .unwrap();
    combat::set_engaged(&mut w, "dave", f.citizen, Some(lamprey)).unwrap();
    let out = ok(&mut w, "dave", "retreat", json!([]));
    assert!(
        out.summary.contains("keeps the wound a while"),
        "{}",
        out.summary
    );
    // It keeps the wound ten minutes, then is whole again.
    assert!(
        combat::creatures_in(&w, sump)
            .iter()
            .any(|c| c.0 == lamprey && c.2 < 10.0),
        "{:?}",
        combat::creatures_in(&w, sump)
    );
    let was = w.fixed_now;
    w.fixed_now = Some(w.now().plus(Duration::seconds(601)));
    w.tick();
    assert!(
        combat::creatures_in(&w, sump)
            .iter()
            .any(|c| c.0 == lamprey && c.2 > 40.0),
        "{:?}",
        combat::creatures_in(&w, sump)
    );
    w.fixed_now = was;
    // The tricks are on the public record.
    let tricks = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == stacks::TRICK)
        .count();
    assert!(tricks >= 3, "{tricks}");
}

#[test]
fn the_census_counts_the_world_and_the_board_averages_the_day() {
    use super::board;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    // A count at boot (nobody holds anything), and the next one wound.
    let count = |w: &World| {
        w.log
            .read_all(&w.id, foxing_log::Cursor::START)
            .unwrap()
            .into_iter()
            .filter(|e| e.kind == board::CENSUS)
            .count()
    };
    assert_eq!(count(&w), 1);
    assert!(w.jobs.has(board::JOB_CENSUS, |_| true));
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Dave holds about 100 at every count (the test world's chunks fade in seconds,
    // so it is earned afresh each time); tyler earns 100 only at the last. Over the
    // day's counts dave held more on average, whatever the moment says.
    for i in 1..=3 {
        w.fixed_now = Some(Timestamp(t0.0 + board::CENSUS_EVERY_MS * i + 1));
        admin(&mut w, "dave", "earn", json!(["steady", 100, 1.0]));
        if i == 3 {
            admin(&mut w, "tyler", "earn", json!(["late", 100, 1.0]));
        }
        w.tick();
    }
    assert_eq!(count(&w), 4);
    let (lines, _) = board::board(&w);
    let dave = lines.iter().find(|l| l.name == "dave").unwrap();
    let tyler = lines.iter().find(|l| l.name == "tyler").unwrap();
    assert!(tyler.holding >= dave.holding, "tyler holds the most now");
    assert!(
        dave.held_24h > tyler.held_24h,
        "dave held more through the day: {} vs {}",
        dave.held_24h,
        tyler.held_24h
    );
    assert_eq!(lines[0].name, "dave", "sorted by the day's average");
    assert!(
        (tyler.held_24h - 25.0).abs() < 1.0,
        "one count in four: {}",
        tyler.held_24h
    );
    // The text and JSON lead with it.
    let out = ok(&mut w, "dave", "board", json!([]));
    assert!(
        out.summary
            .starts_with("Held through the day (averaged), highest first:\n   1. dave"),
        "{}",
        out.summary
    );
    assert!(out.result["citizens"][0]["held_24h"].as_f64().unwrap() > 50.0);
}

#[test]
fn the_conclave_sits_up_inside_the_clocktower_and_its_officers_keep_order() {
    use super::{conclave, stacks};
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    // Eleven rooms, by the escapement from the Stopped Clocktower, and back.
    for k in conclave::KEYS {
        assert!(rooms::room_by_key(&w, k).is_some(), "{k}");
    }
    let tower = rooms::room_by_key(&w, "clocktower").unwrap();
    let stair = rooms::room_by_key(&w, "escapement-stair").unwrap();
    assert_eq!(
        rooms::exits_of(&w, tower).get("escapement").copied(),
        Some(stair)
    );
    assert_eq!(
        rooms::exits_of(&w, stair).get("escapement").copied(),
        Some(tower)
    );
    assert!(conclave::is_house(&w, stair) && !conclave::is_house(&w, tower));
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    w.tick(); // the spawns fill
    let f = find_citizen(&w, "dave").unwrap();
    // At the Lectern the Book speaks first; elsewhere it is just the book.
    let lectern = rooms::room_by_key(&w, "book-lectern").unwrap();
    w.record(
        Actor::Driver,
        StoreEvent::EntityMoved {
            id: f.citizen,
            parent: Some(lectern),
        },
    )
    .unwrap();
    let out = ok(&mut w, "dave", "statutes", json!([]));
    assert!(
        out.summary.starts_with(
            "The Book is open. The Conclave permits you to read it.\nThe Conclave's statutes:"
        ),
        "{}",
        out.summary
    );
    assert!(
        out.summary
            .contains("Carved over the door: WHAT THE CONCLAVE WRITES, MEMORY OBEYS."),
        "{}",
        out.summary
    );
    assert!(!ok(&mut w, "tyler", "statutes", json!([]))
        .summary
        .starts_with("The Book is open"));
    // An engrosser enters a chunk into the minutes: shelved, not halved.
    let tally = rooms::room_by_key(&w, "tally-room").unwrap();
    let clerk = combat::creatures_in(&w, tally)[0].0;
    assert_eq!(stacks::trick_of(&w, clerk), Some(stacks::Trick::Enter));
    let chunk = admin(&mut w, "dave", "earn", json!(["a lantern", 40, 0.5])).result["chunk"]
        .as_u64()
        .unwrap();
    let mut landed = None;
    for _ in 0..40 {
        let (line, over) = stacks::trick(&mut w, &curves(), clerk, f.citizen, "dave").unwrap();
        assert!(!over);
        if let Some(l) = line {
            landed = Some(l);
            break;
        }
    }
    assert!(landed.unwrap().starts_with("An engrosser enters chunk-"));
    assert_eq!(
        w.store.entity(EntityId(chunk)).unwrap().parent,
        Some(f.palace)
    );
    assert!(matches!(
        w.store
            .timed_field(EntityId(chunk), CHUNK, "value")
            .unwrap()
            .curve,
        foxing_time::Curve::PowerLaw { .. }
    ));
    // The tipstaff shows you below the bar: the Gallery, never out of the house.
    let rail = rooms::room_by_key(&w, "brass-rail").unwrap();
    let sergeant = combat::creatures_in(&w, rail)[0].0;
    w.record(
        Actor::Driver,
        StoreEvent::EntityMoved {
            id: f.citizen,
            parent: Some(rail),
        },
    )
    .unwrap();
    let mut shown = None;
    for _ in 0..40 {
        let (line, over) = stacks::trick(&mut w, &curves(), sergeant, f.citizen, "dave").unwrap();
        if let Some(l) = line {
            assert!(over);
            shown = Some(l);
            break;
        }
    }
    assert!(shown
        .unwrap()
        .starts_with("The tipstaff shows you below the bar."));
    assert_eq!(
        rooms::room_of(&w, f.citizen),
        rooms::room_by_key(&w, "petition-gallery")
    );
    // The sitting, heard in the house as a ceremony and outside as one line.
    let midnight = (t0.0 / conclave::SIT_EVERY_MS + 1) * conclave::SIT_EVERY_MS;
    w.fixed_now = Some(Timestamp(midnight - 1000));
    ok(&mut w, "dave", "look", json!([]));
    ok(&mut w, "tyler", "look", json!([]));
    let head = events(&w);
    w.fixed_now = Some(Timestamp(midnight + 1));
    w.tick();
    let inside = narrative(&w, "dave", head);
    assert!(
        inside.iter().any(|l| l
            == "Order. The Conclave is in session. It is midnight, because the Conclave has sat."),
        "{inside:?}"
    );
    assert!(inside
        .iter()
        .any(|l| l == "The minutes are engrossed. The Conclave rises. The Conclave remains."));
    assert!(inside.iter().any(|l| l.starts_with("The Book is opened at the Lectern. The statutes that stand are these: gift allowance at 120;")), "{inside:?}");
    let outside = narrative(&w, "tyler", head);
    assert!(
        outside
            .iter()
            .any(|l| l
                .starts_with("The Conclave has sat. It has read the Ledger and finds no hoard")),
        "{outside:?}"
    );
    assert!(!outside.iter().any(|l| l.starts_with("Order.")));
}

#[test]
fn a_word_for_a_creature_that_lost_its_name_turns_at_once() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Two named creatures, a word between them (the rotation sets it).
    ok(&mut w, "tyler", "travel", json!(["east"]));
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "tyler",
        "remember",
        json!([c, format!("wisp-{}", wisp.0)]),
    );
    ok(&mut w, "tyler", "travel", json!(["south"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    w.tick();
    let holder = [wisp, rag]
        .into_iter()
        .find(|&c| errand::of(&w, c).is_some_and(|e| e.word.is_some()))
        .expect("one word in the air");
    let other = if holder == wisp { rag } else { wisp };
    // The one it names loses its name (the test world's chunks fade in seconds) while
    // the holder is fed to keep its own; the next sweep drops the dead want and the
    // record says why, and a fresh word is not issued while only one is named.
    w.fixed_now = Some(t0.plus(Duration::seconds(600)));
    let target = if holder == wisp {
        ok(&mut w, "tyler", "travel", json!(["north"]));
        format!("wisp-{}", wisp.0)
    } else {
        "ragpicker".to_string()
    };
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, target]));
    assert!(mind::name(&w, holder).is_some());
    let head = events(&w);
    w.tick(); // the other forgets its name
    assert!(mind::name(&w, other).is_none(), "the other forgot its name");
    w.tick(); // and the word for it turns
    assert!(
        errand::of(&w, holder).is_none_or(|e| e.word.is_none()),
        "the dead want turned: {:?} named {:?} other {:?}",
        errand::of(&w, holder).map(|e| (e.word, e.word_to, e.word_by)),
        mind::name(&w, holder),
        mind::name(&w, other)
    );
    let turned: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == errand::TURNED && e.at.0 >= 0)
        .collect();
    assert!(!turned.is_empty(), "{turned:?}");
    assert_eq!(turned[0].payload["why"], "the one it named is gone");
    let _ = head;
}

#[test]
fn death_costs_a_share_of_the_window_on_a_curve_of_level() {
    assert!(combat::death_share(0.0) < 1e-9);
    assert!(
        (combat::death_share(10.0) - 0.016).abs() < 0.002,
        "a newcomer loses almost nothing"
    );
    assert!((combat::death_share(300.0) - 0.25).abs() < 1e-9);
    assert!((combat::death_share(1000.0) - 0.385).abs() < 0.001);
    assert!(combat::death_share(1.0e9) < 0.5 + 1e-9);
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let f = find_citizen(&w, "dave").unwrap();
    // Ten XP: a death takes nothing (less than one XP is owed).
    admin(&mut w, "dave", "earn", json!(["small", 10, 0.5]));
    let d = combat::die(&mut w, "dave", &f, None).unwrap();
    assert!(d.lost.is_none());
    assert_eq!(d.fall, 1);
    assert!((level(&mut w, "dave") - 10.0).abs() < 0.5);
    // A thousand, a night later so the fall is the first of its night: about a third
    // goes, in chunks picked at random, and the record says.
    w.fixed_now = Some(t0.plus(Duration::seconds(13 * 3600)));
    for _ in 0..10 {
        admin(&mut w, "dave", "earn", json!(["big", 100, 0.5]));
    }
    let before = level(&mut w, "dave");
    let d = combat::die(&mut w, "dave", &f, None).unwrap();
    assert!(d.lost.is_some(), "{:?}", d.lost);
    assert_eq!(d.fall, 1, "a night later, the first fall again");
    let after = level(&mut w, "dave");
    let share = 1.0 - after / before;
    assert!(
        (0.35..=0.5).contains(&share),
        "lost {share:.2} of {before:.0}: {:?}",
        d.lost
    );
    // The second fall of the night costs half the share; the third a quarter.
    for _ in 0..10 {
        admin(&mut w, "dave", "earn", json!(["big", 100, 0.5]));
    }
    let before = level(&mut w, "dave");
    let d = combat::die(&mut w, "dave", &f, None).unwrap();
    assert_eq!(d.fall, 2);
    let share2 = 1.0 - level(&mut w, "dave") / before;
    assert!(
        share2 > 0.12 && share2 < 0.25,
        "half the first: {share2:.2}"
    );
    assert!(
        d.said()
            .contains("The second fall of the night: it costs half what the first did."),
        "{}",
        d.said()
    );
    let died: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == combat::DIED)
        .collect();
    assert_eq!(died.len(), 3);
    assert!(died[1].payload["lost"].as_array().unwrap().len() >= 3);
    assert_eq!(died[2].payload["fall"], json!(2));
    assert!(died[2].payload["epitaph"].as_str().unwrap().len() > 20);
}

#[test]
fn the_killer_keeps_what_you_lost_and_may_learn_from_it() {
    use super::death;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    // Dave, worth 600, falls to the Midden's ragpicker by the operator's hand.
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["south"]));
    for _ in 0..6 {
        admin(&mut w, "dave", "earn", json!(["big", 100, 0.5]));
    }
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let f = find_citizen(&w, "dave").unwrap();
    let before = level(&mut w, "dave");
    let d = combat::die(&mut w, "dave", &f, Some(rag)).unwrap();
    let lost = before - level(&mut w, "dave");
    assert!(lost > 50.0, "{lost}");
    assert!((d.worth - lost).abs() < 1.0, "{} vs {lost}", d.worth);
    // Named off this very fall, or not: the keeper is said as the room would say it.
    let keeper = d.keeper.clone().unwrap();
    assert!(
        keeper == "the ragpicker" || keeper.ends_with(" the ragpicker"),
        "{keeper}"
    );
    assert!(
        d.said().contains(&format!("{keeper} keeps it.")),
        "{}",
        d.said()
    );
    // The ragpicker holds it, vivid, as its own; it may have earned a name off it.
    assert!(death::holds_fall_of(&w, rag, "dave"));
    assert!(
        (mind::level(&w, rag) - lost).abs() < 1.0,
        "{} holds {}",
        mind::display(&w, rag),
        mind::level(&w, rag)
    );
    if lost >= 60.0 {
        assert!(mind::name(&w, rag).is_some(), "named off a death");
    }
    // Back at the Threshold; the room says what the ragpicker keeps and learned.
    ok(&mut w, "dave", "travel", json!(["east"]));
    let look = ok(&mut w, "dave", "travel", json!(["south"]));
    assert!(
        look.summary.contains("one of them yours"),
        "{}",
        look.summary
    );
    let learned = plain_str(&w, rag, CREATURE, "learned");
    assert_eq!(
        look.summary.contains("it has learned"),
        learned.is_some(),
        "{}",
        look.summary
    );
    // Felling it takes the memory back, re-minted at its worth now.
    let died: Vec<_> = w
        .recent_events(200)
        .into_iter()
        .filter(|e| e.kind == combat::DIED)
        .collect();
    assert_eq!(died[0].payload["killer_kind"], json!("ragpicker"));
    assert_eq!(died[0].payload["learned"], json!(learned));
    assert!(died[0].payload["kept"].is_number());
    // mine says the night's falls.
    let out = ok(&mut w, "dave", "mine", json!([]));
    assert!(
        out.summary
            .contains("You have fallen once tonight; the next fall costs half of the first."),
        "{}",
        out.summary
    );
}

#[test]
fn a_story_a_creature_has_heard_this_hour_pays_by_halves() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    for who in ["dave", "tyler", "wren"] {
        login(&mut w, who, "test", PASS).unwrap();
        ok(&mut w, who, "travel", json!(["east"]));
        ok(&mut w, who, "travel", json!(["south"]));
    }
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 90, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let name = mind::name(&w, rag).unwrap();
    let tale = "Once there was a lantern that would not go out, though every hand that carried it forgot why they had picked it up.";
    let first = ok(&mut w, "dave", "story", json!([&name, tale]));
    assert_eq!(first.result["repeats"], 0);
    assert!(!first.summary.contains("has heard"), "{}", first.summary);
    // The same tale from another mouth, reworded a little: half.
    let again = "Once there was a lantern that would not go out, though every hand that carried it had forgotten why it was picked up at all.";
    let second = ok(&mut w, "tyler", "story", json!([&name, again]));
    assert_eq!(second.result["repeats"], 1, "{}", second.summary);
    assert!(
        second
            .summary
            .contains("It has heard one much like it this hour; this one pays half."),
        "{}",
        second.summary
    );
    // And a third: a quarter.
    let third = ok(&mut w, "wren", "story", json!([&name, tale]));
    assert_eq!(third.result["repeats"], 2, "{}", third.summary);
    assert!(
        third
            .summary
            .contains("It has heard 2 much like it this hour; this one pays a 4 part."),
        "{}",
        third.summary
    );
    let (a, b, c) = (
        first.result["xp"].as_f64().unwrap(),
        second.result["xp"].as_f64().unwrap(),
        third.result["xp"].as_f64().unwrap(),
    );
    // Jitter is a fifth either way; halves are well outside it.
    assert!(b < a * 0.7 && c < b * 0.7, "{a} {b} {c}");
    // A different story is not a repeat.
    let other = "The clock in the hall struck thirteen and nobody looked up from their soup, which was cold anyway.";
    w.fixed_now = Some(t0.plus(Duration::seconds(3601)));
    let c2 = admin(&mut w, "tyler", "earn", json!(["hollow", 90, 0.5]));
    let c2 = format!("chunk-{}", c2.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c2, &name]));
    let fresh = ok(&mut w, "dave", "story", json!([&name, other]));
    assert_eq!(fresh.result["repeats"], 0, "{}", fresh.summary);
    assert!(
        exchange::heard_lately(&w, rag, tale) == 0,
        "the hour has passed"
    );
}

#[test]
fn a_passing_names_chunk_bears_the_mark_and_is_taxed_in_full_when_given() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    login(&mut w, "dave", "test", PASS).unwrap();
    login_as(&mut w, "Tallowmoth", "test", PASS, true).unwrap();
    // Earned by a passing name: marked, and shown so.
    let c = admin(&mut w, "Tallowmoth", "earn", json!(["wisp", 30, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    let rows = ok(&mut w, "Tallowmoth", "chunks", json!([]));
    assert_eq!(rows.result["chunks"][0]["provenance"], "passing");
    assert!(rows.summary.contains("(passing)"), "{}", rows.summary);
    // Given to a kept name well under every allowance: taxed in full anyway.
    let out = ok(&mut w, "Tallowmoth", "give", json!([&c, "tyler"]));
    assert_eq!(out.result["taxed"], true, "{}", out.summary);
    assert_eq!(out.result["marked"], true);
    assert_eq!(out.result["over"], 1.0);
    assert!(out.summary.contains("bears the mark"), "{}", out.summary);
    let theirs = out.result["chunk"].as_u64().unwrap();
    // The mark travels: tyler gives it on to dave, and it is taxed again in full.
    let out = ok(
        &mut w,
        "tyler",
        "give",
        json!([format!("chunk-{theirs}"), "dave"]),
    );
    assert_eq!(out.result["marked"], true, "{}", out.summary);
    assert_eq!(out.result["taxed"], true);
    let rows = ok(&mut w, "dave", "chunks", json!([]));
    assert!(rows.summary.contains("(passing)"), "{}", rows.summary);
    assert!(rows.summary.contains("(taxed)"), "{}", rows.summary);
    // A kept name's own earnings carry no mark.
    let k = admin(&mut w, "tyler", "earn", json!(["wisp", 30, 0.5]));
    let k = format!("chunk-{}", k.result["chunk"].as_u64().unwrap());
    let out = ok(&mut w, "tyler", "give", json!([&k, "dave"]));
    assert_eq!(out.result["marked"], false, "{}", out.summary);
    assert_eq!(out.result["taxed"], false, "{}", out.summary);
}

#[test]
fn the_named_confide_in_the_unnamed_when_names_are_few() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    ok(&mut w, "tyler", "travel", json!(["east"]));
    ok(&mut w, "tyler", "travel", json!(["south"]));
    // One named creature in the world: the ragpicker, holding a story of tyler's.
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 90, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let name = mind::name(&w, rag).unwrap();
    assert_eq!(mind::named_count(&w), 1);
    assert_eq!(mind::urgency(1), 1.0);
    assert_eq!(mind::urgency(3), 0.5);
    assert_eq!(mind::urgency(6), 0.0);
    assert_eq!(
        mind::scarcity(&w),
        1.5,
        "a want pays half again when names are few"
    );
    assert_eq!(mind::draw_one_in(1.0), 2);
    assert_eq!(mind::draw_one_in(0.5), 21);
    assert_eq!(mind::draw_one_in(0.0), 40);
    // An unnamed wisp is brought from the fogbank to the midden to be told.
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    admin(
        &mut w,
        "tyler",
        "bring",
        json!([format!("creature-{}", wisp.0), "midden"]),
    );
    assert!(combat::creatures_in(&w, midden).iter().any(|c| c.0 == wisp));
    assert!(mind::name(&w, wisp).is_none());
    assert_eq!(mind::level(&w, wisp), 0.0);
    // Ten minutes on, the confiding job runs: certain at one named. The wisp holds the
    // story's memory now, the room heard it told, and the record is public. (The
    // test world's chunks halve in seconds, so the ragpicker is fed again at the
    // stroke to keep its name for the telling.)
    w.fixed_now = Some(t0.plus(Duration::seconds(599)));
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 90, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([&c, &name]));
    let head = events(&w);
    w.fixed_now = Some(t0.plus(Duration::seconds(601)));
    assert!(w.tick() >= 1);
    let told: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == mind::CONFIDED)
        .collect();
    assert_eq!(told.len(), 1, "{told:?}");
    assert_eq!(told[0].payload["teller"], rag.0);
    assert_eq!(told[0].payload["listener"], wisp.0);
    assert_eq!(told[0].payload["named"], 1);
    assert!(told[0].payload["paid"].as_f64().unwrap() >= 4.0);
    assert!(mind::level(&w, wisp) > 0.0, "{}", mind::level(&w, wisp));
    let heard = narrative(&w, "tyler", head);
    assert!(
        heard.iter().any(|l| l.starts_with(&format!(
            "{name} the ragpicker tells the wisp a story, low and quick"
        ))),
        "{heard:?}"
    );
    assert!(crate::observe::visible(
        &w,
        &Viewer::actor("nobody"),
        w.now(),
        &told[0]
    ));
    // The ragpicker's own story is still its own: a story told is not lost.
    assert!(mind::level(&w, rag) > 0.0);
    // The wisp shows what it holds now. A name failing is said in the room: the test
    // world's chunks halve in ten seconds, so some seconds on the ragpicker is under
    // forty-five and still named (above thirty).
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary.contains("wisp (9/9, holding "),
        "{}",
        look.summary
    );
    w.fixed_now = Some(t0.plus(Duration::seconds(622)));
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary.contains(&format!("{name} the ragpicker ("))
            && look.summary.contains("its name failing"),
        "{}",
        look.summary
    );
    // With six named, nobody confides: five more unnamed creatures are named here.
    let unnamed: Vec<EntityId> = w
        .store
        .entities()
        .filter(|e| e.kind == "creature" && e.id != rag && mind::name(&w, e.id).is_none())
        .map(|e| e.id)
        .take(5)
        .collect();
    assert_eq!(unnamed.len(), 5);
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 90, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([&c, &name]));
    for id in unnamed {
        let c = admin(&mut w, "tyler", "earn", json!(["hollow", 90, 0.5]));
        let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
        admin(
            &mut w,
            "tyler",
            "bring",
            json!([format!("creature-{}", id.0), "midden"]),
        );
        ok(
            &mut w,
            "tyler",
            "remember",
            json!([&c, format!("creature-{}", id.0)]),
        );
    }
    assert!(mind::named_count(&w) >= 6, "{}", mind::named_count(&w));
    assert_eq!(mind::scarcity(&w), 1.0);
    assert_eq!(mind::confide(&mut w).unwrap(), 0);
}

#[test]
fn a_memory_taken_from_a_creature_is_minted_at_what_it_is_worth_now() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "tyler", "travel", json!(["east"]));
    // Tyler feeds a wisp 40; twenty seconds on (two test-world half-lives) it holds
    // about 10. Dave fells it and takes the memory at that worth, not at 40.
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    let c = admin(&mut w, "tyler", "earn", json!(["a story", 40, 0.0]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "tyler",
        "remember",
        json!([&c, format!("creature-{}", wisp.0)]),
    );
    w.fixed_now = Some(t0.plus(Duration::seconds(20)));
    let held = mind::level(&w, wisp);
    assert!(held > 5.0 && held < 15.0, "{held}");
    // Dry run: the wisp is whole, so it is struck; afterwards it is not itself.
    let mut out = None;
    for _ in 0..30 {
        let o = ok(
            &mut w,
            "dave",
            "attack",
            json!([format!("creature-{}", wisp.0)]),
        );
        if o.result["taken"].is_number() || o.result["killed"] == json!(true) {
            out = Some(o);
            break;
        }
    }
    let out = out.expect("the wisp goes down");
    assert!(out.result["taken"].is_number(), "{}", out.summary);
    let worth = out.result["xp"].as_f64().unwrap();
    assert!((worth - held).abs() < 1.5, "worth {worth}, held {held}");
    assert_eq!(out.result["was"], 40.0);
    assert!(
        out.summary.contains(&format!(
            "worth {worth:.0} XP. It was worth 40 when the wisp got it"
        )),
        "{}",
        out.summary
    );
    assert!(
        out.summary.contains("will not be itself for "),
        "{}",
        out.summary
    );
    // The chunk shows the taken worth as its size, and `look` says the wisp is shaken.
    let rows = ok(&mut w, "dave", "chunks", json!([]));
    assert!(
        rows.summary.contains(&format!("of {worth:.1}"))
            || rows.summary.contains(&format!("of {worth:.0}")),
        "{}",
        rows.summary
    );
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(look.summary.contains("not itself yet"), "{}", look.summary);
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "attack",
        json!([format!("creature-{}", wisp.0)]),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("not itself yet and cannot be struck for 60s"),
        "{err}"
    );
    // The dry run says the same, and refuses a bare `take` and a `look` with an argument.
    let checker = crate::gate::checker();
    let r = checker.check(&format!(
        "FROM: dave\nTO: disremembered\nINTENT: REQUEST\nREF: d-9\n\n[DO] dave MUST attack creature-{}.\n[DO] dave MUST take.\n[DO] dave MUST look Ides.\n[DO] dave MUST take \"wisp ember\".\n",
        wisp.0
    ));
    let p = preview::run_checked(&w, "dave", false, &r.message, Some((&r.verdict, &[])));
    let what: Vec<String> = p["statements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["what"].as_str().unwrap().to_string())
        .collect();
    assert!(
        what[0].starts_with("refused: wisp is not itself yet, for "),
        "{}",
        what[0]
    );
    assert_eq!(what[1], "refused: take needs an argument (item)");
    assert_eq!(what[2], "refused: look takes 0 arguments; the DO gives 1");
    assert_eq!(what[3], "refused: there is no wisp ember here to take");
    assert!(
        p["header"][0]
            .as_str()
            .unwrap()
            .starts_with("the checker's verdict is FAIL"),
        "{}",
        p["header"]
    );
    // Filler does not make two stories one.
    assert_eq!(
        exchange::story_words("Every night though the lantern that nobody remembers lighting"),
        vec!["lantern", "lighting", "night", "nobody", "remember"]
    );
}

#[test]
fn the_ray_finds_what_matters_more_surely_the_more_you_hold() {
    // Many rays at a fixed instant (no fading): how often the largest chunk is the
    // one halved, at level ~60 (aim 1 in 6) and at level ~2,700 (aim 9 in 10).
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let office = rooms::room_by_key(&w, "censor-office").unwrap();
    let censor = combat::creatures_in(&w, office)[0].0;
    let f = find_citizen(&w, "dave").unwrap();
    let rate = |w: &mut World, big: f64, small: f64| -> f64 {
        let mut hits = 0;
        let n = 120;
        for seed in 0..n {
            w.seed(seed);
            // Fresh chunks each round, so the halving does not compound.
            let b = admin(w, "dave", "earn", json!(["big", big, 0.5])).result["chunk"]
                .as_u64()
                .unwrap();
            let smalls: Vec<u64> = (0..5)
                .map(|_| {
                    admin(w, "dave", "earn", json!(["small", small, 0.5])).result["chunk"]
                        .as_u64()
                        .unwrap()
                })
                .collect();
            let mut landed = false;
            for _ in 0..40 {
                let (line, _) = stacks::trick(w, &curves(), censor, f.citizen, "dave").unwrap();
                if let Some(l) = line {
                    if l.contains(&format!("chunk-{b} ")) {
                        hits += 1;
                    }
                    landed = true;
                    break;
                }
            }
            assert!(landed);
            for id in std::iter::once(b).chain(smalls) {
                w.record(
                    foxing_log::Actor::Driver,
                    StoreEvent::EntityRemoved { id: EntityId(id) },
                )
                .unwrap();
            }
        }
        hits as f64 / n as f64
    };
    // Level ~60: one big chunk of 50 among five of 2. Aim 60/360 = 1 in 6; by weight
    // the big one is 50/60 of the rest. Between a third and all.
    let low = rate(&mut w, 50.0, 2.0);
    // Level ~2,700: 2,500 among five of 40. Aim 0.9; nearly always.
    let high = rate(&mut w, 2500.0, 40.0);
    assert!(high > 0.9, "high {high}");
    assert!(low < high, "low {low} high {high}");
    // And a tenth of an XP among a forty is seldom what the ray takes, at any
    // level: by weight, the chaff is one part in four hundred.
    let chaff = rate(&mut w, 40.0, 0.1);
    assert!(chaff > 0.9, "the big one took {chaff}");
}

#[test]
fn a_made_thing_can_be_named_once_it_holds_sixty_and_a_fixture_stands_in_a_room_of_yours() {
    use super::craft;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // The world's own fixtures stand where the rooms say: the Ledger of Fines at
    // the Overdue Desk, open to any hand; the Tally Book in the Division Lobby, not.
    let desk = rooms::room_by_key(&w, "overdue-desk").unwrap();
    let fx = craft::fixtures_in(&w, desk);
    assert_eq!(fx.len(), 1, "{fx:?}");
    assert_eq!(fx[0]["given_name"], "the Ledger of Fines");
    assert_eq!(fx[0]["open"], true);
    let ledger = EntityId(fx[0]["id"].as_u64().unwrap());
    // Re-running the seeding places nothing twice.
    craft::ensure_fixtures(&mut w, &curves()).unwrap();
    assert_eq!(craft::fixtures_in(&w, desk).len(), 1);
    // A fixture is not taken, and a found thing is not named.
    admin(&mut w, "dave", "bring", json!(["dave", "overdue-desk"]))
        .result
        .get("room");
    let f = find_citizen(&w, "dave").unwrap();
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::EntityMoved {
            id: f.citizen,
            parent: Some(desk),
        },
    )
    .unwrap();
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary
            .contains("Standing here: the Ledger of Fines, a visitor book (item-")
            && look.summary.contains("open to any hand"),
        "{}",
        look.summary
    );
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "take",
        json!(["ledger"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("stands where it is"), "{err}");
    // Any hand writes in the world's open book; the world's own is nobody's to name.
    let out = ok(
        &mut w,
        "dave",
        "append",
        json!(["dave owes a thimble.", "ledger"]),
    );
    assert!(
        out.summary.ends_with("dave owes a thimble. \u{2014} dave"),
        "{}",
        out.summary
    );
    // A second hand's line is kept apart and signed; `read` gives the book back one
    // line at a time, with the hands in it; `look` says how many lines and the last.
    admin(&mut w, "dave", "bring", json!(["tyler", "overdue-desk"]));
    let t = find_citizen(&w, "tyler").unwrap();
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::EntityMoved {
            id: t.citizen,
            parent: Some(desk),
        },
    )
    .unwrap();
    ok(
        &mut w,
        "tyler",
        "append",
        json!(["tyler owes nothing, and says so.", "ledger"]),
    );
    let read = ok(&mut w, "tyler", "read", json!(["ledger"]));
    assert!(
        read.summary.starts_with("the Ledger of Fines (item-")
            && read.summary.contains("whole (3 lines):\nA column of fines")
            && read.summary.contains("\ndave owes a thimble. \u{2014} dave\ntyler owes nothing, and says so. \u{2014} tyler\nHands in it: dave, tyler."),
        "{}",
        read.summary
    );
    assert_eq!(read.result["lines"].as_array().unwrap().len(), 3);
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary.contains("3 lines, the last \"tyler owes nothing, and says so. \u{2014} tyler\"; read it for the rest"),
        "{}",
        look.summary
    );
    let list = ok(&mut w, "tyler", "read", json!([]));
    assert!(
        list.summary.contains("the Ledger of Fines (item-"),
        "{}",
        list.summary
    );
    let room = ok(&mut w, "tyler", "read", json!(["here"]));
    assert!(
        room.summary.starts_with("The Overdue Desk, whole: "),
        "{}",
        room.summary
    );
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "read",
        json!(["a sonnet"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("no a sonnet here"), "{err}");
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "name",
        json!(["Dave's Book", "ledger"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("the world's own"), "{err}");
    // Tyler's lantern: made by tyler, so dave cannot name it even holding it; tyler
    // cannot name it until it holds sixty; then he can, and it fades back to a
    // lantern when what it holds falls under thirty.
    let tyler_id = find_citizen(&w, "tyler").unwrap().citizen;
    let s = craft::satchel(&mut w, "tyler", tyler_id).unwrap();
    let ember = {
        // Materials, conjured as a creature would leave them.
        let id = w.store.allocate_id();
        w.record(
            foxing_log::Actor::Driver,
            StoreEvent::EntityCreated {
                id,
                kind: "item".into(),
                owner: None,
                parent: Some(s),
                scopes: vec![],
            },
        )
        .unwrap();
        let mut f2 = BTreeMap::new();
        f2.insert("name".into(), Field::Plain(json!("a wisp ember")));
        f2.insert("kind".into(), Field::Plain(json!("material")));
        f2.insert(
            "memory".into(),
            Field::Timed(foxing_time::TimedValue::new(
                1.0,
                w.now(),
                foxing_time::Curve::Constant,
            )),
        );
        w.record(
            foxing_log::Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: craft::ITEM.into(),
                fields: f2,
            },
        )
        .unwrap();
        id
    };
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "name",
        json!(["The Last Ember", format!("item-{}", ember.0)]),
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("a found thing takes no name"),
        "{err}"
    );
    for name in ["rag thread", "a dab of ink"] {
        let id = w.store.allocate_id();
        w.record(
            foxing_log::Actor::Driver,
            StoreEvent::EntityCreated {
                id,
                kind: "item".into(),
                owner: None,
                parent: Some(s),
                scopes: vec![],
            },
        )
        .unwrap();
        let mut f2 = BTreeMap::new();
        f2.insert("name".into(), Field::Plain(json!(name)));
        f2.insert("kind".into(), Field::Plain(json!("material")));
        f2.insert(
            "memory".into(),
            Field::Timed(foxing_time::TimedValue::new(
                1.0,
                w.now(),
                foxing_time::Curve::Constant,
            )),
        );
        w.record(
            foxing_log::Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: craft::ITEM.into(),
                fields: f2,
            },
        )
        .unwrap();
    }
    let c = admin(&mut w, "tyler", "earn", json!(["moth", 12, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    let made = ok(&mut w, "tyler", "craft", json!(["lantern", &c]));
    let lantern = made.result["item"].as_u64().unwrap();
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "name",
        json!(["Wick's Last Light", format!("item-{lantern}")]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("takes 60"), "{err}");
    let big = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let big = format!("chunk-{}", big.result["chunk"].as_u64().unwrap());
    let out = ok(
        &mut w,
        "tyler",
        "remember",
        json!([&big, format!("item-{lantern}")]),
    );
    assert!(
        out.summary.contains("a lantern keeps it. It holds 70 now."),
        "{}",
        out.summary
    );
    let out = ok(
        &mut w,
        "tyler",
        "name",
        json!(["Wick's Last Light", "lantern"]),
    );
    assert_eq!(out.summary, "a lantern is Wick's Last Light now.");
    let out = ok(
        &mut w,
        "tyler",
        "describe",
        json!(["It gutters when you look away.", "lantern"]),
    );
    assert!(
        out.summary.starts_with("Wick's Last Light now reads:"),
        "{}",
        out.summary
    );
    let sat = ok(&mut w, "tyler", "satchel", json!([]));
    assert!(
        sat.summary.contains("Wick's Last Light, a lantern (item-"),
        "{}",
        sat.summary
    );
    // Dave, handed it, cannot rename it: not his making.
    ok(&mut w, "tyler", "drop", json!(["lantern"]));
    let tyler_room = rooms::room_of(&w, find_citizen(&w, "tyler").unwrap().citizen).unwrap();
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::EntityMoved {
            id: f.citizen,
            parent: Some(tyler_room),
        },
    )
    .unwrap();
    ok(&mut w, "dave", "take", json!(["lantern"]));
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "name",
        json!(["Mine", "lantern"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("was made by tyler"), "{err}");
    // The name is written in memory: on the test world's palace curve (a scale of a
    // hundred seconds) what it held is under thirty a few minutes on, and the name
    // goes; the lantern itself is still remembered.
    w.fixed_now = Some(t0.plus(Duration::seconds(400)));
    assert!(
        craft::held(&w, EntityId(lantern)) < 30.0,
        "{}",
        craft::held(&w, EntityId(lantern))
    );
    craft::sweep(&mut w).unwrap();
    assert_eq!(craft::display(&w, EntityId(lantern)), "a lantern");
    // A fixture of tyler's own: a visitor book in a room he dug, named at once from
    // the sixty it was made with; dave may write in it only once welcomed; three
    // fixtures fill a room.
    w.fixed_now = Some(t0.plus(Duration::seconds(401)));
    let dig = admin(&mut w, "tyler", "earn", json!(["dig", 100, 1.0]));
    let dig = format!("chunk-{}", dig.result["chunk"].as_u64().unwrap());
    let dug = ok(&mut w, "tyler", "dig", json!(["study", &dig, "The Study"]));
    let study = EntityId(dug.result["room"].as_u64().unwrap());
    ok(&mut w, "tyler", "travel", json!(["study"]));
    for name in ["rag thread", "a dab of ink"] {
        let id = w.store.allocate_id();
        w.record(
            foxing_log::Actor::Driver,
            StoreEvent::EntityCreated {
                id,
                kind: "item".into(),
                owner: None,
                parent: Some(s),
                scopes: vec![],
            },
        )
        .unwrap();
        let mut f2 = BTreeMap::new();
        f2.insert("name".into(), Field::Plain(json!(name)));
        f2.insert("kind".into(), Field::Plain(json!("material")));
        f2.insert(
            "memory".into(),
            Field::Timed(foxing_time::TimedValue::new(
                1.0,
                w.now(),
                foxing_time::Curve::Constant,
            )),
        );
        w.record(
            foxing_log::Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: craft::ITEM.into(),
                fields: f2,
            },
        )
        .unwrap();
    }
    let sixty = admin(&mut w, "tyler", "earn", json!(["book", 65, 1.0]));
    let sixty = format!("chunk-{}", sixty.result["chunk"].as_u64().unwrap());
    let made = ok(&mut w, "tyler", "craft", json!(["book", &sixty]));
    assert!(
        made.summary.contains("It stands here, holding 65"),
        "{}",
        made.summary
    );
    let book = made.result["item"].as_u64().unwrap();
    assert_eq!(craft::fixtures_in(&w, study).len(), 1);
    let out = ok(&mut w, "tyler", "name", json!(["The Study Book", "book"]));
    assert_eq!(out.summary, "a visitor book is The Study Book now.");
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::EntityMoved {
            id: f.citizen,
            parent: Some(study),
        },
    )
    .unwrap();
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "append",
        json!(["dave was here", "book"]),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("only from a hand the room's owner welcomed"),
        "{err}"
    );
    ok(&mut w, "tyler", "welcome", json!(["dave"]));
    let out = ok(&mut w, "dave", "append", json!(["dave was here", "book"]));
    assert!(out.summary.contains("dave was here"), "{}", out.summary);
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary
            .contains("Standing here: The Study Book, a visitor book (item-"),
        "{}",
        look.summary
    );
    assert!(look.result["fixtures"][0]["id"] == json!(book));
    // Dave may not place a fixture in tyler's room (welcomed is not permitted).
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "craft",
        json!(["plaque"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("a room you may build in"), "{err}");
    let _ = ledger;
}

#[test]
fn the_bindery_folds_scraps_into_a_sheaf_and_binds_what_was_appended() {
    use super::{bindery, craft};
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // A creature told six stories; six seconds on in the test world (half-life ten
    // seconds at vividity 0, longer with it) they are scraps, and fold into a sheaf
    // worth their sum, the gists kept.
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["south"]));
    let c = admin(&mut w, "dave", "earn", json!(["hollow", 90, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "dave", "remember", json!([c, "ragpicker"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let window = mind::window_of(&w, rag).unwrap();
    for i in 0..6 {
        super::memory::earn_chunk(
            &mut w,
            "nobody",
            window,
            &format!("a story: \"Tale number {i} of the lantern\""),
            10.0,
            0.0,
            &curves(),
        )
        .unwrap();
    }
    assert_eq!(
        w.store
            .children(window)
            .filter(|e| e.kind == "chunk")
            .count(),
        7
    );
    assert_eq!(
        bindery::fold_sheaves(&mut w, &curves()).unwrap(),
        0,
        "not scraps yet"
    );
    w.fixed_now = Some(t0.plus(Duration::seconds(40)));
    assert_eq!(bindery::fold_sheaves(&mut w, &curves()).unwrap(), 1);
    let chunks: Vec<String> = w
        .store
        .children(window)
        .filter(|e| e.kind == "chunk")
        .filter_map(|e| plain_str(&w, e.id, CHUNK, "source").map(str::to_string))
        .collect();
    assert!(
        chunks
            .iter()
            .any(|s| s.starts_with("a sheaf of 6 stories: \"Tale number 0 of the lantern\"")),
        "{chunks:?}"
    );
    assert!(
        !chunks
            .iter()
            .any(|s| s == "a story: \"Tale number 3 of the lantern\""),
        "{chunks:?}"
    );
    // The hollow's memory (not a story) was left alone.
    assert!(chunks.iter().any(|s| s == "hollow"), "{chunks:?}");
    // A room much appended is a candidate; the operator binds it; the owner unbinds.
    w.fixed_now = Some(t0.plus(Duration::seconds(41)));
    let dig = admin(&mut w, "tyler", "earn", json!(["dig", 100, 1.0]));
    let dig = format!("chunk-{}", dig.result["chunk"].as_u64().unwrap());
    let dug = ok(&mut w, "tyler", "dig", json!(["nook", &dig, "The Nook"]));
    let nook = dug.result["room"].as_u64().unwrap();
    ok(&mut w, "tyler", "travel", json!(["nook"]));
    ok(&mut w, "tyler", "describe", json!(["A nook with a bench."]));
    ok(&mut w, "tyler", "welcome", json!(["dave"]));
    let f = find_citizen(&w, "dave").unwrap();
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::EntityMoved {
            id: f.citizen,
            parent: Some(EntityId(nook)),
        },
    )
    .unwrap();
    for i in 0..3 {
        ok(
            &mut w,
            "dave",
            "append",
            json!([format!("Dave sat on the bench, time {i}.")]),
        );
        ok(
            &mut w,
            "tyler",
            "append",
            json!([format!("Tyler counted the slats, time {i}.")]),
        );
    }
    let cands = bindery::candidates(&w);
    let room = cands["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == json!(nook))
        .expect("the nook is a candidate");
    assert_eq!(room["appends"], 6);
    assert_eq!(room["hands"], json!(["dave", "tyler"]));
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "bind",
        json!([format!("room-{nook}"), "Bound prose that is long enough."]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("may not call"), "{err}");
    let out = admin(&mut w, "dave", "bind", json!([format!("room-{nook}"), "A nook with a bench, its slats counted three times by Tyler while Dave sat. (Hands: dave, tyler.)", "haiku"]));
    assert!(
        out.summary.starts_with("The Nook is bound:"),
        "{}",
        out.summary
    );
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary.contains("its slats counted three times"),
        "{}",
        look.summary
    );
    assert!(
        look.summary.contains("Bound, the hands of dave"),
        "{}",
        look.summary
    );
    assert!(
        !bindery::candidates(&w)["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["id"] == json!(nook)),
        "bound: no longer a candidate"
    );
    let err = call(&mut w, "dave", ActorKind::Citizen, "unbind", json!([])).unwrap_err();
    assert!(err.to_string().contains("Only the room's owner"), "{err}");
    let out = ok(&mut w, "tyler", "unbind", json!([]));
    assert!(
        out.summary
            .contains("A nook with a bench. Dave sat on the bench, time 0."),
        "{}",
        out.summary
    );
    let err = call(&mut w, "tyler", ActorKind::Citizen, "unbind", json!([])).unwrap_err();
    assert!(err.to_string().contains("Nothing here is bound"), "{err}");
    let bound: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| {
            e.kind == bindery::BOUND || e.kind == bindery::UNBOUND || e.kind == bindery::SHEAF
        })
        .collect();
    assert_eq!(bound.len(), 3, "{bound:?}");
    let _ = craft::FIXTURES_PER_ROOM;
}

#[test]
fn the_namery_deals_each_stocked_name_once_and_a_creature_takes_its_former_name_back() {
    use super::namery;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    for who in ["dave", "tyler"] {
        ok(&mut w, who, "travel", json!(["east"]));
        ok(&mut w, who, "travel", json!(["south"]));
    }
    // Stocking is the operator's; the checks say back what they refuse and why.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "stock",
        json!(["Gloam, Pennywort"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("may not call"), "{err}");
    let out = admin(
        &mut w,
        "dave",
        "stock",
        json!(["Gloam, Pennywort, wisp, Hesper, dave, Midden, attack, Ab, gloam, Tallow-wick, Abcdefghijklm, Gloam", "haiku"]),
    );
    assert_eq!(
        out.result["added"],
        json!(["Gloam", "Pennywort"]),
        "{}",
        out.summary
    );
    let refused: Vec<(String, String)> = out.result["refused"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["name"].as_str().unwrap().to_string(),
                r["why"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        refused,
        vec![
            ("wisp".into(), "one capitalized word of letters".into()),
            ("Hesper".into(), "the world has it".into()),
            ("dave".into(), "one capitalized word of letters".into()),
            ("Midden".into(), "the world has it".into()),
            ("attack".into(), "one capitalized word of letters".into()),
            ("Ab".into(), "3 to 12 letters".into()),
            ("gloam".into(), "twice in the list".into()),
            (
                "Tallow-wick".into(),
                "one capitalized word of letters".into()
            ),
            ("Abcdefghijklm".into(), "3 to 12 letters".into()),
            ("Gloam".into(), "twice in the list".into()),
        ]
    );
    assert!(
        out.summary
            .starts_with("The Namery took in 2 names; 2 unused now. Refused 10:"),
        "{}",
        out.summary
    );
    let err = call(
        &mut w,
        "dave",
        ActorKind::Admin,
        "stock",
        json!(["Attack, Hesper"]),
    )
    .unwrap_err()
    .to_string();
    assert!(
        err.contains("Nothing to stock: Attack (a verb), Hesper (the world has it)"),
        "{err}"
    );
    assert_eq!(namery::report(&w)["unused"], json!(2));
    assert!(namery::report(&w)["low"].as_bool().unwrap());
    // The ragpicker earns its name: the first of the pool, drawn once.
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    assert_eq!(mind::name(&w, rag).as_deref(), Some("Gloam"));
    assert_eq!(namery::pool(&w), vec!["Pennywort".to_string()]);
    assert_eq!(namery::used(&w)[0]["creature"], json!(rag.0));
    // Robbed below half the bar, it loses the name; fed again it takes the same one
    // back, and Pennywort stays in the pool for the next.
    let window = mind::window_of(&w, rag).unwrap();
    for e in w
        .store
        .children(window)
        .filter(|e| e.kind == "chunk")
        .map(|e| e.id)
        .collect::<Vec<_>>()
    {
        w.record(
            foxing_log::Actor::Driver,
            foxing_store::StoreEvent::EntityRemoved { id: e },
        )
        .unwrap();
    }
    mind::check_tier(&mut w, rag).unwrap();
    assert_eq!(mind::name(&w, rag), None);
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    assert_eq!(
        mind::name(&w, rag).as_deref(),
        Some("Gloam"),
        "its former name"
    );
    assert_eq!(namery::pool(&w), vec!["Pennywort".to_string()]);
    // Stocked names are the record's.
    let stocked: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor::START)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == namery::STOCKED)
        .collect();
    assert_eq!(stocked.len(), 1);
    assert_eq!(stocked[0].payload["names"], json!(["Gloam", "Pennywort"]));
    assert_eq!(stocked[0].payload["model"], json!("haiku"));
}

#[test]
fn the_map_grid_draws_north_above() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    // Stand in Lantern Row (north of the Threshold), then back: both are remembered,
    // and the Fogbank east of the Threshold is in sight from it.
    ok(&mut w, "dave", "travel", json!(["north"]));
    ok(&mut w, "dave", "travel", json!(["south"]));
    let out = ok(&mut w, "dave", "map", json!([]));
    let grid: Vec<&str> = out
        .summary
        .lines()
        .skip_while(|l| !l.starts_with("This floor"))
        .skip(1)
        .take_while(|l| l.contains('·') || l.contains('@') || l.contains('*') || l.contains('.'))
        .collect();
    // The legend is one line: "01 The Threshold; 02 The Fogbank; 03 Lantern Row".
    let legend: Vec<&str> = out
        .summary
        .lines()
        .find(|l| l.starts_with("01 "))
        .map(|l| l.split("; ").collect())
        .unwrap_or_default();
    let code_of = |name: &str| {
        legend
            .iter()
            .find(|l| l.trim().ends_with(name))
            .map(|l| l.trim()[..2].to_string())
            .unwrap_or_else(|| panic!("{name} in {legend:?}"))
    };
    let row_of = |code: &str| {
        grid.iter()
            .position(|r| r.contains(code))
            .unwrap_or_else(|| panic!("{code} in {grid:?}"))
    };
    let lantern = row_of(&code_of("Lantern Row"));
    let threshold = row_of(&code_of("The Threshold"));
    assert!(
        lantern < threshold,
        "north is drawn above: Lantern Row row {lantern}, Threshold row {threshold}\n{}",
        out.summary
    );
    let fog = row_of(&code_of("The Fogbank"));
    assert_eq!(fog, threshold, "east is on the same row\n{}", out.summary);
}

#[test]
fn where_finds_a_way_you_remember_and_travel_takes_a_room_name() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Dave walks to the Midden and back: the way is remembered.
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["south"]));
    ok(&mut w, "dave", "travel", json!(["north"]));
    ok(&mut w, "dave", "travel", json!(["west"]));
    let out = ok(&mut w, "dave", "where", json!(["the midden"]));
    assert_eq!(
        out.summary, "The Midden: the way from here, as you remember it, is 2 steps: east, south.",
        "{}",
        out.summary
    );
    assert_eq!(out.result["way"], json!(["east", "south"]));
    // A citizen: tyler is at the Threshold with him.
    let out = ok(&mut w, "dave", "where", json!(["tyler"]));
    assert!(
        out.summary.starts_with("tyler is here, in The Threshold."),
        "{}",
        out.summary
    );
    // A named creature, by its given name: the ragpicker in the Midden, once named.
    ok(&mut w, "tyler", "travel", json!(["east"]));
    ok(&mut w, "tyler", "travel", json!(["south"]));
    let c = admin(&mut w, "tyler", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([c, "ragpicker"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let given = mind::name(&w, rag).unwrap();
    let out = ok(&mut w, "dave", "where", json!([&given]));
    assert!(
        out.summary.contains(&format!(
            "{given} the ragpicker is in The Midden; the way from here"
        )),
        "{}",
        out.summary
    );
    // tyler, elsewhere in the world: a tell reaches him now, and once.
    let out = ok(
        &mut w,
        "dave",
        "tell",
        json!(["tyler", "meet me at the Midden"]),
    );
    assert!(
        out.summary
            .starts_with("tyler is elsewhere; the word goes to them ("),
        "{}",
        out.summary
    );
    let to_tyler: Vec<String> = w
        .recent_events(200)
        .iter()
        .filter(|e| e.kind == NARRATIVE && e.payload["to"] == "tyler")
        .filter_map(|e| e.payload["text"].as_str().map(str::to_string))
        .filter(|t| t.contains("left word"))
        .collect();
    assert_eq!(
        to_tyler,
        vec!["dave left word for you from The Threshold: \"meet me at the Midden\"".to_string()]
    );
    // And not again when he next arrives: the word is marked heard.
    w.fixed_now = Some(t0.plus(Duration::seconds(15 * 60)));
    ok(&mut w, "tyler", "look", json!([]));
    let again = w
        .recent_events(300)
        .iter()
        .filter(|e| e.kind == NARRATIVE && e.payload["to"] == "tyler")
        .filter(|e| {
            e.payload["text"]
                .as_str()
                .is_some_and(|t| t.contains("While you were away"))
        })
        .count();
    assert_eq!(again, 0, "delivered once");
    // Nothing unknown: a room never seen is not a way.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "where",
        json!(["nowhere in particular"]),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("Nothing called nowhere in particular"),
        "{err}"
    );
    // travel by a room's name takes the first step and says the rest.
    let out = ok(&mut w, "dave", "travel", json!(["The Midden"]));
    assert!(
        out.summary.contains("Toward The Midden: then south."),
        "{}",
        out.summary
    );
    assert!(out.summary.contains("The Fogbank"), "{}", out.summary);
    assert_eq!(
        out.result["onward"],
        json!("Toward The Midden: then south.")
    );
    let out = ok(&mut w, "dave", "travel", json!(["midden"]));
    assert!(out.summary.contains("The Midden."), "{}", out.summary);
    assert!(out.result["onward"].is_null());
    // help is a verb now, a read.
    let out = ok(&mut w, "dave", "help", json!([]));
    assert!(
        out.summary.starts_with("Verbs, by what they do"),
        "{}",
        out.summary
    );
    assert!(
        out.summary.contains("talk: say, tell, story"),
        "{}",
        out.summary
    );
    // One verb, with its arguments.
    let one = ok(&mut w, "dave", "help", json!(["dig"]));
    assert!(
        one.summary.starts_with("dig <exit> <chunk> [name]"),
        "{}",
        one.summary
    );
    assert!(one.summary.contains("<exit> (string):"), "{}", one.summary);
    // A seed room not yet walked to: the signed way.
    let far = ok(&mut w, "dave", "where", json!(["The Overdue Desk"]));
    assert!(far.summary.contains("the signs say"), "{}", far.summary);
    assert_eq!(far.result["signed"], true);
    assert!(out.result["commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "where"));
}

#[test]
fn wants_lists_the_named_with_the_way_and_mine_says_what_remembers_you() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Nothing yet.
    let out = ok(&mut w, "dave", "mine", json!([]));
    assert!(
        out.summary.starts_with("Nothing in the world is yours yet"),
        "{}",
        out.summary
    );
    let out = ok(&mut w, "dave", "wants", json!([]));
    assert!(
        out.summary
            .starts_with("No named creature wants anything just now"),
        "{}",
        out.summary
    );
    // Dave digs a room off the Fogbank, feeds the Midden's ragpicker to a name, writes
    // in the Ledger.
    ok(&mut w, "dave", "travel", json!(["east"]));
    let c = admin(&mut w, "dave", "earn", json!(["dig", 100, 1.0]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "dave", "dig", json!(["nook", &c, "Dave's Nook"]));
    ok(&mut w, "dave", "travel", json!(["nook"]));
    ok(&mut w, "dave", "permit", json!(["tyler"]));
    ok(&mut w, "dave", "travel", json!(["back"]));
    ok(&mut w, "dave", "travel", json!(["south"]));
    let c = admin(&mut w, "dave", "earn", json!(["hollow", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "dave", "remember", json!([c, "ragpicker"]));
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let rag = combat::creatures_in(&w, midden)[0].0;
    let given = mind::name(&w, rag).unwrap();
    // A want, set by the operator so the test does not wait on the rotation.
    admin(
        &mut w,
        "dave",
        "errand",
        json!([&given, "a lantern left burning"]),
    );
    let out = ok(&mut w, "dave", "wants", json!([]));
    assert!(
        out.summary.contains(&format!("{given} the ragpicker, in The Midden (here), wants a story about a lantern left burning (pays double")),
        "{}",
        out.summary
    );
    assert_eq!(out.result["wants"][0]["way"], json!([]));
    // From the Fogbank the way is one step.
    ok(&mut w, "dave", "travel", json!(["north"]));
    let out = ok(&mut w, "dave", "wants", json!([]));
    assert!(
        out.summary.contains("in The Midden (1 step: south), wants"),
        "{}",
        out.summary
    );
    let out = ok(&mut w, "dave", "mine", json!([]));
    assert!(
        out.summary.starts_with("What remembers you:\n"),
        "{}",
        out.summary
    );
    assert!(
        out.summary
            .contains("Dave's Nook (1 step: nook): yours, remembered, halving every"),
        "{}",
        out.summary
    );
    assert!(
        out.summary.contains(&format!(
            "{given} the ragpicker in The Midden (1 step: south), holding 70."
        )),
        "{}",
        out.summary
    );
    assert_eq!(out.result["rooms"][0]["dug"], json!(true));
    assert_eq!(out.result["creatures"][0]["name"], json!(given));
    // Dave goes away; tyler visits the nook and writes in it, and tells the ragpicker a
    // story. Dave comes back after an hour: the digest says so.
    w.fixed_now = Some(t0.plus(Duration::seconds(20)));
    ok(&mut w, "dave", "leave", json!([]));
    ok(&mut w, "tyler", "travel", json!(["east"]));
    ok(&mut w, "tyler", "travel", json!(["nook"]));
    ok(&mut w, "tyler", "append", json!(["A bench, says tyler."]));
    ok(&mut w, "tyler", "travel", json!(["back"]));
    ok(&mut w, "tyler", "travel", json!(["south"]));
    ok(
        &mut w,
        "tyler",
        "story",
        json!([
            &given,
            "a lantern left burning in a window for a boat that was not coming back tonight"
        ]),
    );
    w.fixed_now = Some(t0.plus(Duration::seconds(3700)));
    login(&mut w, "dave", "test", PASS).unwrap();
    let out = ok(&mut w, "dave", "mine", json!([]));
    assert!(
        out.summary.contains("Since you were last here: "),
        "{}",
        out.summary
    );
    assert!(
        out.summary.contains("tyler came into Dave's Nook"),
        "{}",
        out.summary
    );
    assert!(
        out.summary.contains(&format!("tyler told {given} a story")),
        "{}",
        out.summary
    );
}

#[test]
fn a_creature_learns_from_its_kills_from_irritating_to_painful() {
    use super::stacks::{self, Trick};
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    ok(&mut w, "dave", "travel", json!(["east"]));
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    let dave = find_citizen(&w, "dave").unwrap().citizen;
    assert!(
        stacks::trick_of(&w, wisp).is_none(),
        "a wisp has no trick of its own"
    );
    // Learned, tier by tier (set by hand: the roll is one in three).
    let set = |w: &mut World, list: Vec<&str>| {
        let mut c = std::collections::BTreeMap::new();
        c.insert("learned".to_string(), Field::Plain(json!(list)));
        super::bindery::set_fields(w, foxing_log::Actor::Driver, wisp, CREATURE, c).unwrap();
    };
    set(&mut w, vec!["chatter"]);
    assert_eq!(stacks::trick_of(&w, wisp), Some(Trick::Chatter));
    set(&mut w, vec!["chatter", "numb"]);
    assert_eq!(
        stacks::trick_of(&w, wisp),
        Some(Trick::Numb),
        "the worst learned plays"
    );
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains(
            "it has learned a numbing bite, and before that your name, and says it wrong"
        ),
        "{}",
        look.summary
    );
    // Each trick, played by hand until it lands (one in two).
    let play_until = |w: &mut World, key: &str| -> String {
        set(w, vec![key]);
        for _ in 0..40 {
            let (line, _) = stacks::trick(w, &curves(), wisp, dave, "dave").unwrap();
            if let Some(l) = line {
                return l;
            }
        }
        panic!("{key} never landed");
    };
    let l = play_until(&mut w, "chatter");
    assert!(l.contains("says your name back, wrong: \"evad.\""), "{l}");
    let l = play_until(&mut w, "numb");
    assert!(l.contains("your strikes do half"), "{l}");
    let before = plain_num(&w, wisp, CREATURE, "hp").unwrap();
    let out = ok(&mut w, "dave", "attack", json!(["wisp"]));
    let after = plain_num(&w, wisp, CREATURE, "hp").unwrap_or(0.0);
    assert!(
        before - after <= 3.0,
        "halved: {before} -> {after}: {}",
        out.summary
    );
    assert!(
        out.summary.contains("your arm still asleep") || out.summary.contains("hit it for"),
        "{}",
        out.summary
    );
    ok(&mut w, "dave", "retreat", json!([]));
    super::combat::set_creature_hp(&mut w, wisp, 2.0).unwrap();
    let l = play_until(&mut w, "leech");
    assert!(l.contains("it mends by what it took"), "{l}");
    assert!(plain_num(&w, wisp, CREATURE, "hp").unwrap() > 2.0);
    let hp0 = combat::hp(&w, dave);
    let l = play_until(&mut w, "maw");
    assert!(l.contains("And again, before you can step back"), "{l}");
    assert!(
        combat::hp(&w, dave) < hp0 && combat::hp(&w, dave) >= 1.0,
        "{}",
        combat::hp(&w, dave)
    );
    let l = play_until(&mut w, "shove");
    assert!(l.contains("shoves you"), "{l}");
    assert_ne!(
        rooms::room_of(&w, dave),
        Some(fog),
        "shoved out of the Fogbank"
    );
    // Learning climbs the tiers: three kills, each learning something, worse each time.
    set(&mut w, vec![]);
    let mut got = Vec::new();
    for _ in 0..200 {
        if let Some(k) = super::death::maybe_learn(&mut w, wisp).unwrap() {
            got.push(k);
        }
        if got.len() == 3 {
            break;
        }
    }
    assert_eq!(got.len(), 3, "{got:?}");
    assert!(["chatter", "shove"].contains(&got[0]), "{got:?}");
    assert!(["numb", "leech"].contains(&got[1]), "{got:?}");
    assert!(["venom", "forget", "maw"].contains(&got[2]), "{got:?}");
    assert!(
        super::death::maybe_learn(&mut w, wisp).unwrap().is_none(),
        "three is all"
    );
}

#[test]
fn nothing_mends_in_a_fight_or_while_bleeding() {
    use super::stacks::{self, Trick};
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let dave = find_citizen(&w, "dave").unwrap().citizen;
    ok(&mut w, "dave", "travel", json!(["east"]));
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    // Hurt, out of a fight: mends at the world's rate.
    combat::commit_hp(&mut w, dave, 10.0).unwrap();
    combat::recurve_mending(&mut w, dave).unwrap();
    w.fixed_now = Some(t0.plus(Duration::seconds(60)));
    let after_minute = combat::hp(&w, dave);
    assert!(
        (after_minute - (10.0 + 60.0 * HP_PER_SECOND)).abs() < 0.2,
        "{after_minute}"
    );
    // In a fight: nothing.
    ok(&mut w, "dave", "attack", json!(["wisp"]));
    let at = combat::hp(&w, dave);
    w.fixed_now = Some(t0.plus(Duration::seconds(120)));
    assert!(
        (combat::hp(&w, dave) - at).abs() < 0.01,
        "no mending while engaged: {at} -> {}",
        combat::hp(&w, dave)
    );
    let out = ok(&mut w, "dave", "rest", json!([]));
    assert!(
        out.summary.contains("Nothing mends in a fight"),
        "{}",
        out.summary
    );
    // Retreat: it starts again.
    ok(&mut w, "dave", "retreat", json!([]));
    let at = combat::hp(&w, dave);
    w.fixed_now = Some(t0.plus(Duration::seconds(180)));
    assert!(combat::hp(&w, dave) > at + 1.0, "mending after retreat");
    // A bleeding bite: nothing for eight minutes, then it staunches and says so.
    let mut c = std::collections::BTreeMap::new();
    c.insert("learned".to_string(), Field::Plain(json!(["bleed"])));
    super::bindery::set_fields(&mut w, foxing_log::Actor::Driver, wisp, CREATURE, c).unwrap();
    assert_eq!(stacks::trick_of(&w, wisp), Some(Trick::Bleed));
    let mut line = None;
    for _ in 0..40 {
        let (l, _) = stacks::trick(&mut w, &curves(), wisp, dave, "dave").unwrap();
        if l.is_some() {
            line = l;
            break;
        }
    }
    assert!(
        line.as_deref()
            .is_some_and(|l| l.contains("will not close: nothing mends for 8m")),
        "{line:?}"
    );
    let at = combat::hp(&w, dave);
    w.fixed_now = Some(t0.plus(Duration::seconds(180 + 300)));
    assert!(
        (combat::hp(&w, dave) - at).abs() < 0.01,
        "no mending while bleeding"
    );
    let out = ok(&mut w, "dave", "rest", json!([]));
    assert!(
        out.summary.contains("You are bleeding: nothing mends for"),
        "{}",
        out.summary
    );
    w.fixed_now = Some(t0.plus(Duration::seconds(180 + 8 * 60 + 5)));
    w.tick();
    let staunched = w.recent_events(50).iter().any(|e| {
        e.kind == NARRATIVE && e.payload["text"] == "The bleeding stops; you begin to mend."
    });
    assert!(staunched, "the staunch job spoke");
    let at = combat::hp(&w, dave);
    w.fixed_now = Some(t0.plus(Duration::seconds(180 + 8 * 60 + 65)));
    assert!(
        combat::hp(&w, dave) > at + 1.0,
        "mending again after the bleed"
    );
    // A bandage carried takes the next bleeding bite and is spent; one carried while
    // already bleeding is used by `rest`.
    let sat = craft::satchel(&mut w, "dave", dave).unwrap();
    let bandage = craft::make(
        &mut w,
        &curves(),
        foxing_log::Actor::Driver,
        Some("dave".into()),
        sat,
        "a bandage",
        "made",
        "bandage",
        Some("dave"),
        &["rag thread".into()],
        1,
        false,
    )
    .unwrap();
    assert_eq!(craft::carries(&w, dave, "bandage"), Some(bandage));
    let mut line = None;
    for _ in 0..40 {
        let (l, _) = stacks::trick(&mut w, &curves(), wisp, dave, "dave").unwrap();
        if l.is_some() {
            line = l;
            break;
        }
    }
    assert!(
        line.as_deref()
            .is_some_and(|l| l.contains("your bandage takes it, and is spent")),
        "{line:?}"
    );
    assert!(craft::carries(&w, dave, "bandage").is_none(), "spent");
    assert!(
        super::plain_num(&w, dave, BODY, "bleed_until")
            .filter(|&u| u >= w.now().0 as f64)
            .is_none(),
        "no bleed"
    );
    // Bleeding first, then a bandage: rest closes it.
    let mut line = None;
    for _ in 0..40 {
        let (l, _) = stacks::trick(&mut w, &curves(), wisp, dave, "dave").unwrap();
        if l.is_some() {
            line = l;
            break;
        }
    }
    assert!(
        line.as_deref()
            .is_some_and(|l| l.contains("will not close")),
        "{line:?}"
    );
    craft::make(
        &mut w,
        &curves(),
        foxing_log::Actor::Driver,
        Some("dave".into()),
        sat,
        "a bandage",
        "made",
        "bandage",
        Some("dave"),
        &["rag thread".into()],
        1,
        false,
    )
    .unwrap();
    let out = ok(&mut w, "dave", "rest", json!([]));
    assert!(
        out.summary
            .starts_with("Your bandage closes the bite, and is spent."),
        "{}",
        out.summary
    );
    assert!(craft::carries(&w, dave, "bandage").is_none());
}

/// Not a test: a measurement. `cargo test -p foxing-driver -q scale_ -- --ignored --nocapture`
/// builds a world of N rooms (ROOMS env, default 10000) in a chain with side branches,
/// a creature in every tenth, and times what grows with the room count.
#[test]
#[ignore]
fn scale_measurement() {
    use std::time::Instant;
    let n: usize = std::env::var("ROOMS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10_000);
    let dir = tempfile::tempdir().unwrap();
    let t0 = Timestamp(1_000_000);
    let mut w = open(dir.path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let start = rooms::room_by_key(&w, "fogbank").unwrap();
    let built = Instant::now();
    let mut prev = start;
    let mut ids = Vec::with_capacity(n);
    for i in 0..n {
        let id = w.store.allocate_id();
        w.record(
            foxing_log::Actor::Driver,
            foxing_store::StoreEvent::EntityCreated {
                id,
                kind: "room".into(),
                owner: Some("digger".into()),
                parent: None,
                scopes: vec![],
            },
        )
        .unwrap();
        let mut fields = std::collections::BTreeMap::new();
        fields.insert("key".into(), Field::Plain(json!(format!("r{i}"))));
        fields.insert("name".into(), Field::Plain(json!(format!("Room {i}"))));
        fields.insert(
            "description".into(),
            Field::Plain(json!("A room among many, counted and forgotten.")),
        );
        fields.insert(
            "memory".into(),
            Field::Timed(foxing_time::TimedValue::new(
                1.0,
                t0,
                foxing_time::Curve::Exponential {
                    half_life: Duration::seconds(3600),
                },
            )),
        );
        fields.insert("half_life_ms".into(), Field::Plain(json!(3_600_000)));
        fields.insert("permits".into(), Field::Plain(json!({})));
        fields.insert("welcomes".into(), Field::Plain(json!({})));
        w.record(
            foxing_log::Actor::Driver,
            foxing_store::StoreEvent::ComponentSet {
                id,
                component: ROOM.into(),
                fields,
            },
        )
        .unwrap();
        // Exits: back to the previous, on to the next (set when the next exists).
        let mut ex = serde_json::Map::new();
        ex.insert("back".into(), json!(prev.0));
        let mut ef = std::collections::BTreeMap::new();
        ef.insert("exits".into(), Field::Plain(Value::Object(ex)));
        w.record(
            foxing_log::Actor::Driver,
            foxing_store::StoreEvent::ComponentSet {
                id,
                component: EXITS.into(),
                fields: ef,
            },
        )
        .unwrap();
        // The previous room's "on" exit.
        let mut pex = rooms::exits_of(&w, prev)
            .into_iter()
            .map(|(k, v)| (k, json!(v.0)))
            .collect::<serde_json::Map<_, _>>();
        pex.insert(
            if prev == start {
                format!("deep{i}")
            } else {
                "on".into()
            },
            json!(id.0),
        );
        let mut pf = std::collections::BTreeMap::new();
        pf.insert("exits".into(), Field::Plain(Value::Object(pex)));
        w.record(
            foxing_log::Actor::Driver,
            foxing_store::StoreEvent::ComponentSet {
                id: prev,
                component: EXITS.into(),
                fields: pf,
            },
        )
        .unwrap();
        if i % 10 == 0 {
            combat::fill(&mut w, id).unwrap();
        }
        ids.push(id);
        prev = id;
    }
    let events = w.log.head(&w.id).unwrap().map(|s| s.0).unwrap_or(0);
    eprintln!(
        "built {n} rooms, {} creatures, {events} events in {:.1?}",
        ids.len() / 10,
        built.elapsed()
    );
    let time = |label: &str, f: &mut dyn FnMut()| {
        let t = Instant::now();
        f();
        eprintln!("  {label:<28} {:>9.2?}", t.elapsed());
    };
    time("tick (sweeps, jobs)", &mut || {
        w.tick();
    });
    time("look (Threshold)", &mut || {
        ok(&mut w, "dave", "look", json!([]));
    });
    time("map (first, lays out)", &mut || {
        ok(&mut w, "dave", "map", json!([]));
    });
    time("map (again, cached)", &mut || {
        ok(&mut w, "dave", "map", json!([]));
    });
    time("wants", &mut || {
        ok(&mut w, "dave", "wants", json!([]));
    });
    time("mine", &mut || {
        ok(&mut w, "dave", "mine", json!([]));
    });
    time("where (last room)", &mut || {
        ok(&mut w, "dave", "where", json!([format!("Room {}", n - 1)]));
    });
    time("travel + look", &mut || {
        ok(&mut w, "dave", "travel", json!(["east"]));
    });
    time("attack", &mut || {
        let _ = call(
            &mut w,
            "dave",
            ActorKind::Citizen,
            "attack",
            json!(["wisp"]),
        );
    });
    time("bindery candidates", &mut || {
        let _ = super::bindery::candidates(&w);
    });
    time("namery taken()", &mut || {
        let _ = super::namery::taken(&w);
    });
    time("atlas layout", &mut || {
        let _ = super::atlas::layout(&w);
    });
    time("observe room", &mut || {
        let _ = observe_as(
            &w,
            &foxing_registry::CallContext {
                actor: "dave".into(),
                kind: ActorKind::Citizen,
            },
            rooms::look_query(start),
        );
    });
    // The boot: replay every event.
    drop(w);
    let t = Instant::now();
    let w2 = open(dir.path(), t0.plus(Duration::seconds(10)));
    eprintln!(
        "  {:<28} {:>9.2?}  ({} events)",
        "boot replay",
        t.elapsed(),
        events
    );
    let rooms = w2.store.entities().filter(|e| e.kind == "room").count();
    assert!(rooms >= n, "{rooms}");
}

#[test]
fn the_docent_greets_once_a_night_takes_questions_answers_or_admits_the_page_is_missing_and_tidies()
{
    use super::docent;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    // It stands at the Threshold from the first boot, and `look` says so.
    let threshold = rooms::arrival(&w).unwrap();
    assert_eq!(docent::room(&w), Some(threshold));
    let head = events(&w);
    login(&mut w, "dave", "test", PASS).unwrap();
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary
            .contains("The Docent, Hollis Quire, is here: a stooped figure"),
        "{}",
        look.summary
    );
    // A name in its first hour hears the first words, once.
    let said = narrative(&w, "dave", head);
    assert_eq!(
        said.iter()
            .filter(|l| l.starts_with("The Docent: "))
            .count(),
        1,
        "{said:?}"
    );
    assert!(
        said[0].contains("dave") && said[0].contains("tell docent"),
        "{}",
        said[0]
    );
    ok(&mut w, "dave", "look", json!([]));
    assert_eq!(
        narrative(&w, "dave", head)
            .iter()
            .filter(|l| l.starts_with("The Docent: "))
            .count(),
        1
    );
    // A question: taken with a line, on the operator's list with the asker's room.
    let out = ok(
        &mut w,
        "dave",
        "tell",
        json!(["docent", "where is the midden?"]),
    );
    assert!(
        out.summary
            .starts_with("You ask the Docent, \"where is the midden?\" The Docent: \""),
        "{}",
        out.summary
    );
    assert_eq!(out.result["asked"], 1);
    let rep = docent::report(&w);
    assert_eq!(rep["open"].as_array().unwrap().len(), 1, "{rep}");
    assert_eq!(rep["open"][0]["room"]["name"], "The Threshold");
    assert_eq!(rep["open"][0]["text"], "where is the midden?");
    // The operator answers; the asker hears it wherever they are, and the question closes.
    let head = events(&w);
    let a = admin(
        &mut w,
        "Bindery",
        "answer",
        json!([
            1,
            "West of here, then south: two doors. Mind the ragpicker.",
            "test"
        ]),
    );
    assert!(a.summary.contains("answers dave"), "{}", a.summary);
    let said = narrative(&w, "dave", head);
    assert!(said.iter().any(|l| l == "The Docent finds you, on \"where is the midden?\": \"West of here, then south: two doors. Mind the ragpicker.\""), "{said:?}");
    assert!(docent::report(&w)["open"].as_array().unwrap().is_empty());
    let err = call(
        &mut w,
        "Bindery",
        ActorKind::Admin,
        "answer",
        json!([1, "again"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("answered already"), "{err}");
    // A question nobody answers is said to be missing after a minute and a half.
    let head = events(&w);
    ok(
        &mut w,
        "dave",
        "tell",
        json!(["docent", "what is the Conclave?"]),
    );
    w.fixed_now = Some(t0.plus(Duration::seconds(docent::EMPTY_AFTER_S + 2)));
    w.tick();
    let said = narrative(&w, "dave", head);
    assert!(
        said.iter()
            .any(|l| l.starts_with("The Docent comes back without the page")),
        "{said:?}"
    );
    // Six an hour from one name, no more.
    for i in 0..4 {
        ok(
            &mut w,
            "dave",
            "tell",
            json!(["docent", format!("question {i}")]),
        );
    }
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "tell",
        json!(["docent", "one more"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Six is what I take"), "{err}");
    // Away from the Threshold the question still goes to the desk (over the hour's
    // budget here, so the budget line is what comes back, not "not here").
    admin(&mut w, "dave", "bring", json!(["dave", "midden"]));
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "tell",
        json!(["docent", "hello?"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Six is what I take"), "{err}");
    // Walking back to the Threshold a night later (Midden, north to the Fogbank, west):
    // greeted again, as one it has met.
    w.fixed_now = Some(t0.plus(Duration::seconds(docent::NIGHT_MS / 1000 + 10)));
    ok(&mut w, "dave", "travel", json!(["north"]));
    let head = events(&w);
    let back = ok(&mut w, "dave", "travel", json!(["west"]));
    assert!(back.summary.contains("The Threshold"), "{}", back.summary);
    let said = narrative(&w, "dave", head);
    assert_eq!(
        said.iter()
            .filter(|l| l.starts_with("The Docent: "))
            .count(),
        1,
        "{said:?}"
    );
    assert!(
        !said
            .iter()
            .any(|l| l.contains("New, then") || l.contains("Welcome, dave")),
        "{said:?}"
    );
    admin(&mut w, "dave", "bring", json!(["dave", "midden"]));
    // Its round: a thing left lying in an empty room is carried to the Threshold on
    // the second pass and set down, said aloud there.
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    let item = w.store.allocate_id();
    w.record(
        Actor::Driver,
        StoreEvent::EntityCreated {
            id: item,
            kind: "item".into(),
            owner: None,
            parent: Some(midden),
            scopes: vec![],
        },
    )
    .unwrap();
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("name".into(), Field::Plain(json!("a wisp ember")));
    fields.insert("kind".into(), Field::Plain(json!("material")));
    w.record(
        Actor::Driver,
        StoreEvent::ComponentSet {
            id: item,
            component: craft::ITEM.into(),
            fields,
        },
    )
    .unwrap();
    // dave stands in the Midden: nothing is taken from under anyone.
    assert_eq!(docent::tidy(&mut w).unwrap(), 0);
    admin(&mut w, "dave", "bring", json!(["dave", "threshold"]));
    assert_eq!(docent::tidy(&mut w).unwrap(), 0, "first sighting");
    let head = events(&w);
    assert_eq!(docent::tidy(&mut w).unwrap(), 1, "second sighting");
    assert_eq!(w.store.entity(item).unwrap().parent, Some(threshold));
    let said = narrative(&w, "dave", head);
    assert!(
        said.iter()
            .any(|l| l.contains("The Docent sets down a wisp ember, found lying in The Midden")),
        "{said:?}"
    );
    assert_eq!(docent::report(&w)["tidied"], 1);
    // The greeting book: the operator stocks it, the shape is checked, and a
    // newcomer hears a stocked line (filled) rather than a built-in one.
    let err = call(
        &mut w,
        "Bindery",
        ActorKind::Admin,
        "greeting",
        json!([
            "Hello there, you lot, come in, come in, the desk is open and the catalog is mine",
            "first"
        ]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("no {name}"), "{err}");
    let err = call(
        &mut w,
        "Bindery",
        ActorKind::Admin,
        "greeting",
        json!([
            "{name}, welcome! The desk is open and the catalog is mine to keep tonight.",
            "first"
        ]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("an exclamation"), "{err}");
    let out = admin(&mut w, "Bindery", "greeting", json!(["{name}. New, then: the strip above the prompt is your first hour, and the catalog is mine to keep. {creature}", "first"]));
    assert!(out.summary.contains("(first, 1 of"), "{}", out.summary);
    assert_eq!(
        docent::report(&w)["book"]["first"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let head = events(&w);
    login(&mut w, "tyler", "test", PASS).unwrap();
    ok(&mut w, "tyler", "look", json!([]));
    let said = narrative(&w, "tyler", head);
    assert!(
        said.iter()
            .any(|l| l.starts_with("The Docent: \"tyler. New, then: the strip above the prompt")),
        "{said:?}"
    );
}

#[test]
fn a_session_is_summed_and_the_words_that_reached_you_are_kept() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Nothing yet.
    let s = ok(&mut w, "dave", "session", json!([]));
    assert!(
        s.summary.starts_with("Here ") && s.summary.contains("Nothing yet"),
        "{}",
        s.summary
    );
    // A fight in the Fogbank earns a memory; a word to tyler, and one from tyler.
    ok(&mut w, "dave", "travel", json!(["east"]));
    fight(&mut w, "dave");
    ok(
        &mut w,
        "tyler",
        "tell",
        json!(["dave", "come back west when you can"]),
    );
    ok(&mut w, "dave", "travel", json!(["west"]));
    ok(&mut w, "dave", "tell", json!(["tyler", "on my way"]));
    ok(&mut w, "tyler", "tell", json!(["dave", "good"]));
    w.fixed_now = Some(t0.plus(Duration::seconds(300)));
    let s = ok(&mut w, "dave", "session", json!([]));
    assert!(s.summary.contains("Here 5m 0s"), "{}", s.summary);
    assert!(
        s.summary.contains("memories earned") || s.summary.contains("memory earned"),
        "{}",
        s.summary
    );
    assert!(s.summary.contains("1 said, 2 heard"), "{}", s.summary);
    assert_eq!(s.result["counts"]["heard"], 2);
    // The words, latest last, with when: the one left while away and the one said here.
    let m = ok(&mut w, "dave", "messages", json!([]));
    assert!(
        m.summary.starts_with("Words for you, the last 3 of 3:\n"),
        "{}",
        m.summary
    );
    assert!(
        m.summary.contains("The Docent: \"Welcome, dave."),
        "{}",
        m.summary
    );
    assert!(
        m.summary
            .contains("tyler left word for you: \"come back west when you can\""),
        "{}",
        m.summary
    );
    assert!(
        m.summary.ends_with("tyler said to you, \"good\""),
        "{}",
        m.summary
    );
    let m = ok(&mut w, "dave", "messages", json!([1]));
    assert!(
        m.summary.starts_with("Words for you, the last 1 of 3:"),
        "{}",
        m.summary
    );
    // A short silence is a pause: eleven minutes away (the room disremembered them)
    // and back, the session goes on with what it had.
    w.fixed_now = Some(t0.plus(Duration::seconds(300 + 11 * 60)));
    w.tick();
    ok(&mut w, "dave", "look", json!([]));
    let s = ok(&mut w, "dave", "session", json!([]));
    assert!(
        s.summary.contains("Here 16m") && s.summary.contains("1 memory earned"),
        "{}",
        s.summary
    );
    // A long one is a leaving: back after an hour, the count starts over (the arrival
    // is marked after the first act back, so the act after that reads the new session).
    w.fixed_now = Some(t0.plus(Duration::seconds(300 + 22 * 60)));
    w.tick(); // the room disremembers them eleven minutes after the look
    w.fixed_now = Some(t0.plus(Duration::seconds(300 + 22 * 60 + 3600)));
    w.tick();
    ok(&mut w, "dave", "look", json!([]));
    let s = ok(&mut w, "dave", "session", json!([]));
    assert!(
        s.summary.contains("Here 0s") && s.summary.contains("Nothing yet"),
        "{}",
        s.summary
    );
}

#[test]
fn only_the_stacks_own_kinds_are_whole_again_when_a_fight_ends() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    // A wisp that learned to shove keeps its wounds like any ground-floor creature;
    // a lamprey, the Stacks' own, is whole again.
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    let mut changes = std::collections::BTreeMap::new();
    changes.insert("learned".into(), Field::Plain(json!(["shove"])));
    super::bindery::set_fields(&mut w, Actor::Driver, wisp, CREATURE, changes).unwrap();
    assert!(stacks::trick_of(&w, wisp).is_some(), "it learned something");
    combat::set_creature_hp(&mut w, wisp, 2.0).unwrap();
    combat::mend(&mut w, wisp).unwrap();
    assert_eq!(super::plain_num(&w, wisp, CREATURE, "hp"), Some(2.0));
    let sump = rooms::room_by_key(&w, "pulp-sump").unwrap();
    let lamprey = combat::creatures_in(&w, sump)[0].0;
    combat::set_creature_hp(&mut w, lamprey, 2.0).unwrap();
    combat::mend(&mut w, lamprey).unwrap();
    // Not at once: the wound keeps ten minutes, then it is whole.
    assert_eq!(super::plain_num(&w, lamprey, CREATURE, "hp"), Some(2.0));
    w.fixed_now = Some(w.now().plus(Duration::seconds(601)));
    w.tick();
    let max = super::plain_num(&w, lamprey, CREATURE, "max_hp").unwrap();
    assert_eq!(super::plain_num(&w, lamprey, CREATURE, "hp"), Some(max));
}

#[test]
fn the_docent_keeps_busy_on_its_round_when_no_one_is_at_the_door() {
    use super::docent;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    let threshold = rooms::arrival(&w).unwrap();
    let fog = rooms::room_by_key(&w, "fogbank").unwrap();
    login(&mut w, "dave", "test", PASS).unwrap();
    // Someone at the door: it keeps the door.
    assert_eq!(docent::round_step(&mut w).unwrap(), Vec::<String>::new());
    assert_eq!(docent::room(&w), Some(threshold));
    // dave goes east and digs a room named like a test, with a thing dropped in the
    // Fogbank; then the room disremembers him.
    ok(&mut w, "dave", "travel", json!(["east"]));
    fight(&mut w, "dave");
    let chunk = ok(&mut w, "dave", "chunks", json!([])).result["chunks"][0]["id"]
        .as_u64()
        .unwrap();
    let dug = ok(
        &mut w,
        "dave",
        "dig",
        json!(["nook", format!("chunk-{chunk}"), "Link Test"]),
    );
    let debris = EntityId(dug.result["room"].as_u64().unwrap());
    let dave = find_citizen(&w, "dave").unwrap().citizen;
    let sat = craft::satchel(&mut w, "dave", dave).unwrap();
    let ember = craft::make(
        &mut w,
        &curves(),
        Actor::Driver,
        Some("dave".into()),
        sat,
        "a wisp ember",
        "material",
        "material",
        None,
        &[],
        0,
        false,
    )
    .unwrap();
    ok(&mut w, "dave", "drop", json!(["wisp ember"]));
    assert_eq!(w.store.entity(ember).unwrap().parent, Some(fog));
    // dave is still about: the Docent walks but carries nothing from under him.
    let did = docent::round_step(&mut w).unwrap();
    assert!(did.is_empty(), "{did:?}");
    assert_ne!(docent::room(&w), Some(threshold));
    w.fixed_now = Some(
        w.now()
            .plus(crate::world::ABSENCE)
            .plus(Duration::seconds(5)),
    );
    w.tick();
    assert!(!presence::is_here(
        &w,
        find_citizen(&w, "dave").unwrap().citizen
    ));
    // Now the world is empty: round by round it carries the ember to the Threshold
    // and unmakes the test room when it reaches it.
    let mut carried = false;
    let mut razed = false;
    for _ in 0..200 {
        let did = docent::round_step(&mut w).unwrap();
        if did.iter().any(|d| d.starts_with("carried a wisp ember")) {
            carried = true;
        }
        if did.iter().any(|d| d == "razed Link Test") {
            razed = true;
        }
        if carried && razed {
            break;
        }
    }
    assert!(carried, "the ember was carried");
    assert!(razed, "the test room was razed");
    assert_eq!(w.store.entity(ember).unwrap().parent, Some(threshold));
    assert!(w.store.entity(debris).is_none());
    // `where docent` finds it on its round; a visitor at the door brings it back.
    let out = ok(&mut w, "dave", "where", json!(["docent"]));
    assert!(out.summary.starts_with("The Docent is "), "{}", out.summary);
    admin(&mut w, "dave", "bring", json!(["dave", "threshold"]));
    let did = docent::round_step(&mut w).unwrap();
    assert!(
        did.iter().any(|d| d == "came back to the door") || docent::room(&w) == Some(threshold),
        "{did:?}"
    );
}

#[test]
fn the_docent_is_asked_from_anywhere_and_brings_the_answer_in_person() {
    use super::docent;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    admin(&mut w, "tyler", "bring", json!(["tyler", "midden"]));
    let midden = rooms::room_of(&w, find_citizen(&w, "tyler").unwrap().citizen).unwrap();
    assert_ne!(docent::room(&w), Some(midden));
    let out = ok(
        &mut w,
        "tyler",
        "tell",
        json!(["docent", "what is this smell?"]),
    );
    assert!(
        out.summary.starts_with("You ask after the Docent,")
            && out.summary.contains("comes to you"),
        "{}",
        out.summary
    );
    let id = out.result["asked"].as_u64().unwrap();
    assert_eq!(out.result["here"], false);
    let head = events(&w);
    admin(
        &mut w,
        "Bindery",
        "answer",
        json!([id, "The Midden. It is what it smells of.", "test"]),
    );
    // It came to the asker with the page, and its name is on the record.
    assert_eq!(docent::room(&w), Some(midden));
    let said = narrative(&w, "tyler", head);
    assert!(
        said.iter()
            .any(|l| l.starts_with("The Docent finds you, on \"what is this smell?\"")),
        "{said:?}"
    );
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary
            .contains("The Docent, Hollis Quire, is here, on its round"),
        "{}",
        look.summary
    );
    assert!(
        docent::is_named("Quire") && docent::is_named("hollis quire") && docent::is_named("docent")
    );
    assert_eq!(docent::report(&w)["docent"]["name"], docent::NAME);
}

#[test]
fn the_conclave_takes_petitions_and_always_decides_eventually() {
    use super::conclave;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    assert!(
        w.jobs.has(conclave::JOB_DIVISION, |_| true),
        "the division is wound"
    );
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Elsewhere, the machine is pointed at.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "petition",
        json!(["that the fog be asked to keep to the Fogbank"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Petitioners' Gallery"), "{err}");
    admin(&mut w, "dave", "bring", json!(["dave", "petition-gallery"]));
    admin(
        &mut w,
        "tyler",
        "bring",
        json!(["tyler", "petition-gallery"]),
    );
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "petition",
        json!(["short"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("at least"), "{err}");
    // No petition aimed at a name.
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "petition",
        json!(["that tyler be struck from the Roll and shown the door"]),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("no petition aimed at a name; this one names tyler"),
        "{err}"
    );
    let out = ok(
        &mut w,
        "dave",
        "petition",
        json!(["that the fog be asked to keep to the Fogbank"]),
    );
    assert!(
        out.summary
            .starts_with("The machine gives out a ticket: number "),
        "{}",
        out.summary
    );
    assert!(
        out.summary.contains("becomes a Resolution"),
        "{}",
        out.summary
    );
    let ticket = out.result["ticket"].as_u64().unwrap();
    assert!(ticket > 41);
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "petition",
        json!(["that there be a second one at once"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("still upon the table"), "{err}");
    // A statute petition is read as one.
    let out = ok(
        &mut w,
        "tyler",
        "petition",
        json!(["that the Conclave raise the gift allowance, generosity being no crime"]),
    );
    assert!(
        out.summary
            .contains("names the statute gift_allowance, asking it raised"),
        "{}",
        out.summary
    );
    let mine = ok(&mut w, "dave", "petition", json!([]));
    assert!(
        mine.summary
            .starts_with(&format!("Your petition {ticket} is upon the table")),
        "{}",
        mine.summary
    );
    let table = ok(&mut w, "tyler", "petitions", json!([]));
    assert!(
        table.summary.contains("(yours)") && table.summary.contains("ticket "),
        "{}",
        table.summary
    );
    // The Minute Room before anything: blanks. The Lectern: the Book.
    admin(&mut w, "dave", "bring", json!(["dave", "minute-room"]));
    let r = ok(&mut w, "dave", "read", json!([]));
    assert!(
        r.summary.contains("every line of it is a blank"),
        "{}",
        r.summary
    );
    admin(&mut w, "dave", "bring", json!(["dave", "book-lectern"]));
    let r = ok(&mut w, "dave", "read", json!([]));
    assert!(
        r.summary.contains("The Book is open") && r.summary.contains("gift_allowance"),
        "{}",
        r.summary
    );
    admin(&mut w, "dave", "bring", json!(["dave", "session-benches"]));
    let r = ok(&mut w, "dave", "read", json!(["order paper"]));
    assert!(
        r.summary
            .contains(&format!("petition {ticket}, of dave, upon the table")),
        "{}",
        r.summary
    );
    // Divisions, one a half hour: the petitions climb, tabled or not, and are decided
    // within eight.
    let head = events(&w);
    let before = conclave::statute(&w, "gift_allowance");
    let mut now = t0.0;
    let mut decided = 0;
    for _ in 0..8 {
        now = (now / 3_600_000) * 3_600_000 + 30 * 60 * 1000;
        if now <= w.now().0 {
            now += 3_600_000;
        }
        w.fixed_now = Some(Timestamp(now + 1));
        w.tick();
        decided = w
            .log
            .read_all(&w.id, foxing_log::Cursor { after: head })
            .unwrap()
            .iter()
            .filter(|e| e.kind == conclave::DECIDED)
            .count();
        if decided == 2 {
            break;
        }
    }
    assert_eq!(decided, 2, "the Conclave always decides, eventually");
    let moved: Vec<_> = w
        .log
        .read_all(&w.id, foxing_log::Cursor { after: head })
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == conclave::PETITION_MOVED && e.payload["ticket"] == ticket)
        .collect();
    assert!(moved.len() >= 4, "{}", moved.len());
    assert!(moved[0].payload["said"]
        .as_str()
        .unwrap()
        .contains("is read into the minutes"));
    assert!(moved[1].payload["said"]
        .as_str()
        .unwrap()
        .contains("referred to the Committee"));
    // The petitioner was told wherever they stood, and keeps a memory of it.
    let said = narrative(&w, "dave", head);
    let mine: Vec<_> = said
        .iter()
        .filter(|l| l.starts_with(&format!("Upon your petition {ticket}: the ")))
        .collect();
    assert_eq!(mine.len(), 1, "{said:?}");
    assert!(
        mine[0].contains("have it.") && mine[0].contains("You keep the memory of it: chunk-"),
        "{}",
        mine[0]
    );
    let m = ok(&mut w, "dave", "messages", json!([]));
    assert!(m.summary.contains("Upon your petition"), "{}", m.summary);
    // A statute petition granted moves the statute; denied, it stays.
    let after = conclave::statute(&w, "gift_allowance");
    // Every stage is told to the petitioner; the decision is the line with the division.
    let tyler_lines: Vec<String> = narrative(&w, "tyler", head)
        .into_iter()
        .filter(|l| l.starts_with("Upon your petition"))
        .collect();
    assert!(tyler_lines.len() >= 4, "{tyler_lines:?}");
    assert!(
        tyler_lines[0].contains("read into the minutes"),
        "{}",
        tyler_lines[0]
    );
    let tyler_line = tyler_lines
        .iter()
        .find(|l| l.contains(" have it"))
        .cloned()
        .unwrap();
    if tyler_line.contains("the Ayes have it") {
        assert!((after - before * 1.25).abs() < 1e-6, "{before} -> {after}");
        assert!(
            tyler_line.contains("enlarges the day's gift allowance"),
            "{tyler_line}"
        );
    } else {
        assert_eq!(after, before);
        assert!(tyler_line.contains("not granted"), "{tyler_line}");
    }
    // The table is clear; the decisions and the minutes are there to read.
    let table = ok(&mut w, "dave", "petitions", json!([]));
    assert!(
        table.summary.starts_with("No petition lies upon the table")
            && table.summary.contains("Decided lately:"),
        "{}",
        table.summary
    );
    admin(&mut w, "dave", "bring", json!(["dave", "minute-room"]));
    let r = ok(&mut w, "dave", "read", json!([]));
    assert!(
        r.summary.contains("the division of") && r.summary.contains("Order. The Conclave divides."),
        "{}",
        r.summary
    );
    admin(&mut w, "dave", "bring", json!(["dave", "division-lobby"]));
    // The Clerk entered both decisions in the Tally Book, which reads as a book.
    let r = ok(&mut w, "dave", "read", json!(["Tally Book"]));
    assert!(
        r.summary.contains("the Clerk")
            && r.summary.contains("ticket ")
            && (r.summary.contains("Ayes have it") || r.summary.contains("Noes have it")),
        "{}",
        r.summary
    );
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "append",
        json!(["a line of mine", "Tally Book"]),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("The Clerk of the Conclave keeps it"),
        "{err}"
    );
    admin(&mut w, "dave", "bring", json!(["dave", "book-lectern"]));
    let r = ok(&mut w, "dave", "read", json!([]));
    if said
        .iter()
        .any(|l| l.contains("Resolved, by the Conclave, upon the petition of dave"))
    {
        assert!(
            r.summary.contains("Resolutions entered upon the Book")
                && r.summary.contains("fog be asked"),
            "{}",
            r.summary
        );
    }
    assert!(w.jobs.has(conclave::JOB_DIVISION, |_| true), "wound again");
    assert!(conclave::to_json(&w)["decided"].as_array().unwrap().len() == 2);
}

#[test]
fn a_room_takes_eight_doors_and_a_forgotten_one_is_unmade_on_the_round() {
    use super::{building, docent};
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let chunk = |w: &mut World| {
        admin(w, "dave", "earn", json!(["a room's worth", 100, 0.2])).result["chunk"]
            .as_u64()
            .unwrap()
    };
    let c = chunk(&mut w);
    ok(
        &mut w,
        "dave",
        "dig",
        json!(["hall", format!("chunk-{c}"), "The Hall"]),
    );
    ok(&mut w, "dave", "travel", json!(["hall"]));
    let hall = rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen).unwrap();
    // Seven more doors fill it (back is the eighth).
    for (i, door) in ["ash", "elm", "fir", "oak", "yew", "ivy", "box"]
        .iter()
        .enumerate()
    {
        let c = chunk(&mut w);
        let door = door.to_string();
        ok(
            &mut w,
            "dave",
            "dig",
            json!([door, format!("chunk-{c}"), format!("Room {i}")]),
        );
    }
    assert_eq!(rooms::exits_of(&w, hall).len(), building::MAX_EXITS);
    let c = chunk(&mut w);
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "dig",
        json!(["bay", format!("chunk-{c}"), "One Too Many"]),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("8 doors already, the 8 a room takes"),
        "{err}"
    );
    // The side rooms are left to fade: in a day and more they are forgotten, and the
    // Docent's round unmakes them one by one; the Hall, with dave in it, stands.
    ok(&mut w, "dave", "leave", json!([]));
    w.fixed_now = Some(t0.plus(Duration::seconds(10 * 86_400)));
    w.tick();
    let mut unmade = 0;
    for _ in 0..600 {
        let did = docent::round_step(&mut w).unwrap();
        unmade += did
            .iter()
            .filter(|d| d.starts_with("unmade the forgotten"))
            .count();
    }
    assert!(unmade >= 7, "{unmade}");
    assert!(
        w.store.entity(hall).is_none(),
        "the Hall itself, empty and faded, went too"
    );
}

#[test]
fn the_exergue_weighs_money_stories_mirrors_the_coinage_and_guards_the_purse() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    // Down from the Clocktower: the stair's dates, the bar's slate.
    admin(&mut w, "dave", "bring", json!(["dave", "exergue-stair"]));
    let r = ok(&mut w, "dave", "read", json!([]));
    assert!(
        r.summary.contains("The dates on the steps"),
        "{}",
        r.summary
    );
    ok(&mut w, "dave", "travel", json!(["down"]));
    let r = ok(&mut w, "dave", "read", json!([]));
    assert!(r.summary.contains("nothing weighed today"), "{}", r.summary);
    // A story with no money in it weighs nothing; one about money pays its weight,
    // not in gold: few XP, a day to halve.
    let out = ok(
        &mut w,
        "dave",
        "story",
        json!([
            "tapster",
            "There was a lantern on the stair and a moth that would not leave it alone all night."
        ]),
    );
    assert!(
        out.summary.contains("weighs stories about money"),
        "{}",
        out.summary
    );
    let out = ok(&mut w, "dave", "story", json!(["tapster", "My grandmother kept her coins in a tea tin and paid the rent in silver every quarter, and owed nobody a penny when she died."]));
    assert!(
        out.summary.contains("chalks the figure") && out.summary.contains("just not in gold"),
        "{}",
        out.summary
    );
    let xp = out.result["xp"].as_f64().unwrap();
    assert!((3.0..=12.0).contains(&xp), "{xp}");
    assert_eq!(out.result["half_life_ms"], 86_400_000);
    let chunks = ok(&mut w, "dave", "chunks", json!([]));
    assert!(
        chunks.summary.contains("Counting Bar"),
        "{}",
        chunks.summary
    );
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "story",
        json!([
            "tapster",
            "Another story about money and coins and the price of bread in a cold winter long ago."
        ]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("One an hour"), "{err}");
    // The rack mirrors the coinage: the weighted coin is in circulation now.
    ok(&mut w, "dave", "travel", json!(["east"]));
    let r = ok(&mut w, "dave", "read", json!([]));
    assert!(
        r.summary.contains("the weighted coin") && r.summary.contains("1 struck"),
        "{}",
        r.summary
    );
    // The plate is the hour's rates.
    admin(&mut w, "dave", "bring", json!(["dave", "striking-floor"]));
    let r = ok(&mut w, "dave", "read", json!([]));
    assert!(
        r.summary.contains("The plate under the middle press"),
        "{}",
        r.summary
    );
    // The Purse: not past two Wardens; past none, what you can carry of it.
    admin(&mut w, "dave", "bring", json!(["dave", "purse-room"]));
    let err = call(&mut w, "dave", ActorKind::Citizen, "take", json!(["purse"])).unwrap_err();
    assert!(
        err.to_string().contains("Wardens stand over the Purse"),
        "{err}"
    );
    let r = ok(&mut w, "dave", "read", json!(["purse"]));
    assert!(
        r.summary.contains("2 Wardens stand over it") && r.summary.contains("never been taken"),
        "{}",
        r.summary
    );
    let purse_room = rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen).unwrap();
    let wardens: Vec<u64> = super::combat::creatures_in(&w, purse_room)
        .iter()
        .filter(|(_, k, _, _)| k == "warden")
        .map(|(id, _, _, _)| id.0)
        .collect();
    assert_eq!(wardens.len(), 2);
    for id in &wardens {
        admin(
            &mut w,
            "dave",
            "bring",
            json!([format!("creature-{id}"), "midden"]),
        );
    }
    let out = ok(&mut w, "dave", "take", json!(["purse"]));
    assert!(
        out.summary.starts_with("You untie the cord.") && out.result["taken"] == true,
        "{}",
        out.summary
    );
    assert!(out.result["worth"].as_f64().unwrap() >= 120.0);
    let again = ok(&mut w, "dave", "take", json!(["purse"]));
    assert!(
        again.summary.contains("The Purse is empty"),
        "{}",
        again.summary
    );
    // A bare attack at the plinth never picks a Warden.
    admin(
        &mut w,
        "dave",
        "bring",
        json!([format!("creature-{}", wardens[0]), "purse-room"]),
    );
    let err = call(&mut w, "dave", ActorKind::Citizen, "attack", json!([])).unwrap_err();
    assert!(err.to_string().contains("officer of the house"), "{err}");
    // The plaque at the bottom stamps the hour.
    ok(&mut w, "dave", "travel", json!(["down"]));
    let out = ok(
        &mut w,
        "dave",
        "append",
        json!(["I was here, and the Purse was lighter for it.", "Exergue"]),
    );
    assert!(out.summary.contains(" UTC: I was here"), "{}", out.summary);
}

#[test]
fn a_room_welcomes_all_and_invites_a_named_creature_to_live_in_it() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    login(&mut w, "dave", "test", PASS).unwrap();
    // Tyler digs a room at the Fogbank and welcomes every hand.
    ok(&mut w, "tyler", "travel", json!(["east"]));
    let c = admin(&mut w, "tyler", "earn", json!(["wisp", 40, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "tyler",
        "dig",
        json!(["hollow", &c, "The Open House"]),
    );
    ok(&mut w, "tyler", "travel", json!(["hollow"]));
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "permit",
        json!(["all"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("by name"), "{err}");
    let out = ok(&mut w, "tyler", "welcome", json!(["all"]));
    assert!(out.summary.contains("Everyone may add"), "{}", out.summary);
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["hollow"]));
    let out = ok(&mut w, "dave", "append", json!(["A line from a stranger."]));
    assert!(
        out.summary.contains("stranger") || out.summary.contains("add"),
        "{}",
        out.summary
    );
    ok(&mut w, "tyler", "revoke", json!(["everyone"]));
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "append",
        json!(["Another."]),
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("owner")
            || err.to_string().contains("welcome")
            || err.to_string().contains("permitted"),
        "{err}"
    );
    // A wisp at the Fogbank, fed to its name by tyler, is invited to live in the room.
    let c2 = admin(&mut w, "tyler", "earn", json!(["wisp", 70, 0.5]));
    let c2 = format!("chunk-{}", c2.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "travel", json!(["back"]));
    let out = ok(&mut w, "tyler", "remember", json!([&c2, "wisp"]));
    assert!(out.summary.contains(" the wisp"), "{}", out.summary);
    let fog = rooms::room_of(&w, find_citizen(&w, "tyler").unwrap().citizen).unwrap();
    let named = combat::creatures_in(&w, fog)
        .into_iter()
        .map(|c| c.0)
        .find(|&c| mind::name(&w, c).is_some())
        .expect("a named wisp");
    let given = mind::name(&w, named).unwrap().to_string();
    // Not from the world's room; not by someone it does not know.
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "invite",
        json!([&given]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("room you dug"), "{err}");
    ok(&mut w, "tyler", "travel", json!(["hollow"]));
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "invite",
        json!([&given]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("owner"), "{err}");
    let out = ok(&mut w, "tyler", "invite", json!([&given]));
    assert!(
        out.summary.contains("settles in The Open House"),
        "{}",
        out.summary
    );
    let house = rooms::room_of(&w, find_citizen(&w, "tyler").unwrap().citizen).unwrap();
    assert_eq!(w.store.entity(named).and_then(|e| e.parent), Some(house));
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "invite",
        json!([&given]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("here already"), "{err}");
    // Dave, who told it nothing, cannot even from his own room; and the creature's
    // "who made you" names its feeder.
    let out = ok(&mut w, "tyler", "tell", json!([&given, "who made you?"]));
    let said = narrative(&w, "tyler", 0).join(" ");
    assert!(
        said.contains("tyler fed me") || out.summary.contains("fed me"),
        "{}\n{}",
        out.summary,
        said
    );
}

/// A material conjured into a satchel, as a creature would have left it.
fn conjure(w: &mut World, satchel: EntityId, name: &str) -> EntityId {
    let id = w.store.allocate_id();
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::EntityCreated {
            id,
            kind: "item".into(),
            owner: None,
            parent: Some(satchel),
            scopes: vec![],
        },
    )
    .unwrap();
    let mut f = BTreeMap::new();
    f.insert("name".into(), Field::Plain(json!(name)));
    f.insert("kind".into(), Field::Plain(json!("material")));
    f.insert(
        "memory".into(),
        Field::Timed(foxing_time::TimedValue::new(
            1.0,
            w.now(),
            foxing_time::Curve::Constant,
        )),
    );
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::ComponentSet {
            id,
            component: craft::ITEM.into(),
            fields: f,
        },
    )
    .unwrap();
    id
}

#[test]
fn a_thing_keeps_its_past_and_the_tiers_climb() {
    use super::past;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    login(&mut w, "dave", "test", PASS).unwrap();
    admin(
        &mut w,
        "tyler",
        "stock",
        json!(["Quillon, Marrowgate, Tallow"]),
    );
    let tyler = find_citizen(&w, "tyler").unwrap().citizen;
    let s = craft::satchel(&mut w, "tyler", tyler).unwrap();
    // Tier 1: a lantern of an ember and a thread, with a plain memory; its record
    // begins with its making and follows it through the rooms and hands.
    conjure(&mut w, s, "a wisp ember");
    conjure(&mut w, s, "rag thread");
    let c = admin(&mut w, "tyler", "earn", json!(["wisp", 30, 0.4]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    let out = ok(&mut w, "tyler", "craft", json!(["lantern", &c]));
    let lantern = out.result["item"].as_u64().unwrap();
    let out = ok(&mut w, "tyler", "read", json!([format!("item-{lantern}")]));
    assert!(out.summary.contains("Made by tyler in The Threshold of a wisp ember and rag thread, with a plain memory of wisp"), "{}", out.summary);
    ok(&mut w, "tyler", "travel", json!(["east"]));
    ok(&mut w, "tyler", "travel", json!(["west"]));
    ok(&mut w, "tyler", "drop", json!([format!("item-{lantern}")]));
    ok(&mut w, "dave", "take", json!([format!("item-{lantern}")]));
    let out = ok(&mut w, "dave", "read", json!([format!("item-{lantern}")]));
    assert!(
        out.summary.contains("In tyler's hands it saw The Fogbank."),
        "{}",
        out.summary
    );
    assert!(
        out.summary
            .contains("Set down by tyler in The Threshold. Taken up by dave in The Threshold."),
        "{}",
        out.summary
    );
    // Tier 2 asks a rare material and a vivid memory: a faint one is refused.
    conjure(&mut w, s, "a wisp ember");
    conjure(&mut w, s, "a censor's lens");
    conjure(&mut w, s, "rag thread");
    let faint = admin(&mut w, "tyler", "earn", json!(["lamprey", 80, 0.2]));
    let faint = format!("chunk-{}", faint.result["chunk"].as_u64().unwrap());
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "craft",
        json!(["night-lantern", &faint]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("vividity"), "{err}");
    let vivid = admin(&mut w, "tyler", "earn", json!(["lamprey", 80, 0.9]));
    let vivid = format!("chunk-{}", vivid.result["chunk"].as_u64().unwrap());
    let out = ok(&mut w, "tyler", "craft", json!(["night-lantern", &vivid]));
    let night = out.result["item"].as_u64().unwrap();
    assert!(out.result["tier"].as_u64().unwrap() >= 2, "{}", out.summary);
    assert!(
        out.summary.contains("Something in the making took"),
        "{}",
        out.summary
    );
    let out = ok(&mut w, "tyler", "read", json!([format!("item-{night}")]));
    assert!(
        out.summary.contains("with a vivid memory of lamprey"),
        "{}",
        out.summary
    );
    // The Bindery lists it, and writes its past beside the record.
    let cands = bindery::candidates(&w);
    let cand = cands["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["kind"] == "past" && c["id"] == night)
        .cloned();
    assert!(cand.is_some(), "{cands}");
    let out = admin(&mut w, "Bindery", "bind", json!([format!("item-{night}"), "It was made in the Threshold of an ember and a lens, with the memory of a lamprey that nearly had its maker.", "test"]));
    assert!(out.summary.contains("The past of"), "{}", out.summary);
    let out = ok(&mut w, "tyler", "read", json!([format!("item-{night}")]));
    assert!(
        out.summary
            .contains("nearly had its maker. The record: Made by tyler"),
        "{}",
        out.summary
    );
    let cands = bindery::candidates(&w);
    assert!(
        !cands["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["kind"] == "past" && c["id"] == night),
        "written, so not wanted"
    );
    // A night lantern keeps the map three times as long.
    let a = atlas::remembered_map(&w, "tyler", 3_600_000.0).unwrap();
    assert_eq!(
        a["half_life_ms"].as_f64().unwrap(),
        3.0 * 3_600_000.0,
        "{a}"
    );
    // Tier 3, raised by hand (one build in forty does it): a name from the Namery,
    // a desire it signals without words, and thanks when carried to it.
    let night = EntityId(night);
    past::raise(&mut w, &curves(), "tyler", night, 3).unwrap();
    assert_eq!(past::tier(&w, night), 3);
    let given = craft::display(&w, night);
    assert!(
        ["Quillon", "Marrowgate", "Tallow"].contains(&given.as_str()),
        "{given}"
    );
    let out = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        out.summary.contains(&format!("{given} pulls "))
            || out.summary.contains(&format!("{given} turns in your hand")),
        "{}",
        out.summary
    );
    // Walk it where it wants: follow the pull until it settles.
    let mut settled = false;
    for _ in 0..40 {
        let look = ok(&mut w, "tyler", "look", json!([]));
        let Some(stir) = look.result["stirrings"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(Value::as_str)
        else {
            break;
        };
        let Some(step) = stir
            .strip_prefix(&format!("{given} pulls "))
            .and_then(|r| r.strip_suffix(" in your hand."))
        else {
            break;
        };
        let out = ok(&mut w, "tyler", "travel", json!([step]));
        if out.summary.contains("goes still in your hand") {
            settled = true;
            assert!(
                out.summary.contains(&format!("{given}'s thanks"))
                    || out.summary.contains("You remember its thanks"),
                "{}",
                out.summary
            );
            break;
        }
    }
    assert!(settled, "the lantern never settled");
    let out = ok(
        &mut w,
        "tyler",
        "read",
        json!([format!("item-{}", night.0)]),
    );
    assert!(
        out.summary.contains("Awake, with a name of its own."),
        "{}",
        out.summary
    );
    assert!(out.summary.contains("It wanted "), "{}", out.summary);
    assert!(w.recent_events(50).iter().any(|e| e.kind == past::SETTLED));
    // Its name is its own: the sweep that unnames things under thirty leaves it.
    craft::sweep(&mut w).unwrap();
    assert_eq!(craft::display(&w, night), given);
}

#[test]
fn standing_costs_a_deed_and_not_lines_alone() {
    use super::economy;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "polly", "test", PASS).unwrap();
    // Twenty lines across two half-hours with no deed: a script, not a regular.
    for i in 0..economy::REGULAR_LINES {
        if i == economy::REGULAR_LINES / 2 {
            w.fixed_now = Some(Timestamp(t0.0 + 31 * 60 * 1000));
        }
        w.note(
            Actor::Citizen("polly".into()),
            "gate.message",
            json!({"from": "polly", "text": "look", "compiled": true}),
        )
        .unwrap();
    }
    assert_eq!(
        economy::dashboard(&w)["regulars"],
        0,
        "lines bought no standing"
    );
    // A story that missed its want is no deed either; one a creature kept is.
    w.note(
        Actor::Citizen("polly".into()),
        super::exchange::STORY,
        json!({"from": "polly", "creature": 1, "missed": true}),
    )
    .unwrap();
    assert_eq!(
        economy::dashboard(&w)["regulars"],
        0,
        "a missed story is no deed"
    );
    w.note(
        Actor::Citizen("polly".into()),
        super::exchange::STORY,
        json!({"from": "polly", "creature": 1, "missed": false}),
    )
    .unwrap();
    assert_eq!(economy::dashboard(&w)["regulars"], 1);
}

#[test]
fn a_named_creature_keeps_a_word_under_the_others_and_a_story_that_carries_it_pays() {
    use super::secret;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    login(&mut w, "dave", "test", PASS).unwrap();
    admin(&mut w, "tyler", "stock", json!(["Quillon, Marrowgate"]));
    ok(&mut w, "tyler", "travel", json!(["east"]));
    let c = admin(&mut w, "tyler", "earn", json!(["wisp", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "tyler", "remember", json!([&c, "wisp"]));
    let fog = rooms::room_of(&w, find_citizen(&w, "tyler").unwrap().citizen).unwrap();
    let named = combat::creatures_in(&w, fog)
        .into_iter()
        .map(|c| c.0)
        .find(|&c| mind::name(&w, c).is_some())
        .unwrap();
    let given = mind::name(&w, named).unwrap().to_string();
    // Asked, it hints and never says; the word is set at level one, from the lore
    // when it holds no story of its own.
    let out = ok(
        &mut w,
        "tyler",
        "tell",
        json!([&given, "what is your word?"]),
    );
    let said = out.summary.clone();
    assert!(said.contains("a word under the others"), "{said}");
    let (word, level, _) = secret::ensure(&mut w, named).unwrap().unwrap();
    assert_eq!(level, 1);
    assert!(!said.contains(&word), "the hint must not say it: {said}");
    // A story without it pays the usual; one that carries it pays one and a half
    // times, vivid, and the creature says so; the strike is public, word and all.
    // (No lore word in it: the seed that picks the word is the creature's id, which
    // moves whenever the world gains an entity at boot.)
    let plain = ok(&mut w, "tyler", "story", json!([&given, "Once a candle was left burning on a sill that nobody owned, and it burned until the morning came."]));
    assert!(plain.result["struck"].is_null(), "{}", plain.summary);
    // The test world's chunks fade in seconds: fed again across each hour, the name holds.
    let feed = |w: &mut World| {
        let c = admin(w, "tyler", "earn", json!(["wisp", 300, 0.5]));
        let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
        ok(w, "tyler", "remember", json!([&c, &given]));
    };
    w.fixed_now = Some(t0.plus(Duration::seconds(3601)));
    feed(&mut w);
    let story = format!("Once there was a {word} that nobody had counted, and it waited by the river for a name that would not come, patient as stone.");
    let out = ok(&mut w, "tyler", "story", json!([&given, &story]));
    assert_eq!(out.result["struck"]["word"], word, "{}", out.summary);
    assert!(
        out.summary
            .contains("the first of three: 1.5 times the pay"),
        "{}",
        out.summary
    );
    assert!(w
        .recent_events(20)
        .iter()
        .any(|e| e.kind == secret::STRUCK && e.payload["word"] == word));
    // The ladder: a second word now, at level two; the same word pays nobody again.
    let (word2, level2, _) = secret::ensure(&mut w, named).unwrap().unwrap();
    assert_eq!(level2, 2);
    w.fixed_now = Some(t0.plus(Duration::seconds(7202)));
    feed(&mut w);
    ok(&mut w, "dave", "travel", json!(["east"]));
    let out = ok(&mut w, "dave", "story", json!([&given, &story]));
    assert!(
        out.result["struck"].is_null() || word2 == word,
        "{}",
        out.summary
    );
    let story2 = format!("The {word2} was the thing it had wanted most, and it said so to the fog, and the fog said nothing back at all.");
    w.fixed_now = Some(t0.plus(Duration::seconds(10803)));
    feed(&mut w);
    let out = ok(&mut w, "dave", "story", json!([&given, &story2]));
    assert_eq!(out.result["struck"]["level"], 2, "{}", out.summary);
    assert!(out.summary.contains("2.0 times the pay"), "{}", out.summary);
}

#[test]
fn rooms_react_to_who_stands_in_them_and_the_closet_mends_six_a_minute() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "tyler", "test", PASS).unwrap();
    // A stranger: the room has a line of its own, once; the second visit has none.
    let out = ok(&mut w, "tyler", "travel", json!(["east"]));
    assert!(
        out.result["reaction"]
            .as_str()
            .is_some_and(|r| r.contains("The Fogbank")),
        "{}",
        out.summary
    );
    ok(&mut w, "tyler", "travel", json!(["west"]));
    let out = ok(&mut w, "tyler", "travel", json!(["east"]));
    assert!(out.result["reaction"].is_null(), "{}", out.summary);
    // The digger: the room is theirs, and says so.
    let c = admin(&mut w, "tyler", "earn", json!(["wisp", 40, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "tyler",
        "dig",
        json!(["closet", &c, "The Cartographer's Closet"]),
    );
    let out = ok(&mut w, "tyler", "travel", json!(["closet"]));
    assert!(
        out.result["reaction"]
            .as_str()
            .is_some_and(|r| r.contains("yours")
                || r.contains("hand that opened it")
                || r.contains("was yours")),
        "{}",
        out.summary
    );
    // The Closet mends at six a minute, as its field guide says; elsewhere at the usual.
    let me = find_citizen(&w, "tyler").unwrap().citizen;
    assert!((combat::mending_rate(&w, me) - combat::CLOSET_HP_PER_SECOND).abs() < 1e-9);
    ok(&mut w, "tyler", "travel", json!(["back"]));
    assert!((combat::mending_rate(&w, me) - super::HP_PER_SECOND).abs() < 1e-9);
}

/// A material as a creature would leave it, into a name's satchel.
fn material(w: &mut World, actor: &str, name: &str) -> EntityId {
    let f = find_citizen(w, actor).unwrap();
    let s = craft::satchel(w, actor, f.citizen).unwrap();
    craft::make(
        w,
        &curves(),
        foxing_log::Actor::Driver,
        None,
        s,
        name,
        "material",
        "left",
        None,
        &[],
        0,
        false,
    )
    .unwrap()
}

#[test]
fn a_kept_name_writes_recipes_the_bench_then_knows() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // A passing name is refused; a kept one writes it, materials first, purpose after.
    login_as(&mut w, "Tallowmoth", "test", PASS, true).unwrap();
    let err = call(
        &mut w,
        "Tallowmoth",
        ActorKind::Citizen,
        "recipe",
        json!(["a tin whistle", "rag thread"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("kept"), "{err}");
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "recipe",
        json!(["a tin whistle", "unicorn hair"]),
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("No creature leaves unicorn hair"),
        "{err}"
    );
    let out = ok(
        &mut w,
        "dave",
        "recipe",
        json!([
            "a tin whistle",
            "rag thread",
            "a small key",
            "for calling the fog in"
        ]),
    );
    assert_eq!(out.result["recipe"], json!("tin-whistle"));
    assert_eq!(out.result["of"], json!(["rag thread", "a small key"]));
    assert_eq!(out.result["does"], json!("for calling the fog in"));
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "recipe",
        json!(["Tin Whistle", "rag thread"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("already"), "{err}");
    // Listed with the rest, with who wrote it; anyone crafts it for a chunk worth 20.
    let list = ok(&mut w, "tyler", "recipes", json!([]));
    assert!(
        list.summary.contains(
            "tin-whistle a tin whistle (a recipe dave wrote): of rag thread and a small key"
        ),
        "{}",
        list.summary
    );
    material(&mut w, "tyler", "rag thread");
    material(&mut w, "tyler", "a small key");
    let c = admin(&mut w, "tyler", "earn", json!(["moth", 25, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    let made = ok(&mut w, "tyler", "craft", json!(["tin-whistle", &c]));
    assert!(made.summary.contains("a tin whistle"), "{}", made.summary);
    let sat = ok(&mut w, "tyler", "satchel", json!([]));
    assert!(sat.summary.contains("a tin whistle"), "{}", sat.summary);
    // Three a name.
    ok(
        &mut w,
        "dave",
        "recipe",
        json!(["a second thing", "moth dust"]),
    );
    ok(
        &mut w,
        "dave",
        "recipe",
        json!(["a third thing", "moth dust"]),
    );
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "recipe",
        json!(["a fourth thing", "moth dust"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("yours are written"), "{err}");
    // The recipes survive a restart: they are in the log, not the process.
    assert_eq!(craft::added_recipes(&w).len(), 3);
}

#[test]
fn rooms_give_materials_to_the_hand_once_an_hour() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    // Lantern Row: a wisp ember. Then nothing more for an hour.
    ok(&mut w, "dave", "travel", json!(["north"]));
    let out = ok(&mut w, "dave", "gather", json!([]));
    assert_eq!(out.result["name"], json!("a wisp ember"));
    assert!(
        out.summary
            .starts_with("You take a wisp ember from the lantern here"),
        "{}",
        out.summary
    );
    let err = call(&mut w, "dave", ActorKind::Citizen, "gather", json!([])).unwrap_err();
    assert!(err.to_string().contains("within the hour"), "{err}");
    let sat = ok(&mut w, "dave", "satchel", json!([]));
    assert!(sat.summary.contains("a wisp ember"), "{}", sat.summary);
}

#[test]
fn things_are_given_and_traded_hand_to_hand() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    let key = material(&mut w, "dave", "a small key");
    let ink = material(&mut w, "tyler", "a dab of ink");
    let weed = material(&mut w, "dave", "river weed");
    // A thing given goes into their satchel with its past.
    let out = ok(
        &mut w,
        "dave",
        "give",
        json!([format!("item-{}", weed.0), "tyler"]),
    );
    assert!(
        out.summary.starts_with("You hand river weed to tyler"),
        "{}",
        out.summary
    );
    let f = find_citizen(&w, "tyler").unwrap();
    let theirs = craft::satchel(&mut w, "tyler", f.citizen).unwrap();
    assert_eq!(w.store.entity(weed).unwrap().parent, Some(theirs));
    // A trade stands until mirrored, then both cross.
    let dave_id = find_citizen(&w, "dave").unwrap().citizen;
    let mine = craft::satchel(&mut w, "dave", dave_id).unwrap();
    let out = ok(
        &mut w,
        "dave",
        "trade",
        json!([
            format!("item-{}", key.0),
            "tyler",
            format!("item-{}", ink.0)
        ]),
    );
    assert_eq!(out.result["done"], json!(false));
    assert_eq!(w.store.entity(key).unwrap().parent, Some(mine));
    let out = ok(
        &mut w,
        "tyler",
        "trade",
        json!(["a dab of ink", "dave", "a small key"]),
    );
    assert_eq!(out.result["done"], json!(true), "{}", out.summary);
    assert_eq!(w.store.entity(ink).unwrap().parent, Some(mine));
    assert_eq!(w.store.entity(key).unwrap().parent, Some(theirs));
    let read = ok(&mut w, "dave", "read", json!([format!("item-{}", ink.0)]));
    assert!(
        read.summary
            .contains("Traded by tyler to dave for a small key"),
        "{}",
        read.summary
    );
}

#[test]
fn a_warded_door_asks_for_a_word() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    let c = admin(&mut w, "dave", "earn", json!(["hydra", 50, 0.5]));
    let chunk = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "dave",
        "dig",
        json!(["down", &chunk, "The Salt Stair"]),
    );
    ok(&mut w, "dave", "travel", json!(["down"]));
    ok(
        &mut w,
        "dave",
        "ward",
        json!(["brine", "what the sea leaves on the step"]),
    );
    ok(&mut w, "dave", "travel", json!(["up"]));
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "travel",
        json!(["down"]),
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("what the sea leaves on the step"),
        "{err}"
    );
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "travel",
        json!(["down", "salt"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("word"), "{err}");
    let out = ok(&mut w, "tyler", "travel", json!(["down", "brine"]));
    assert!(out.summary.contains("Salt Stair"), "{}", out.summary);
    // Once passed, always passed; the owner walks through; unward opens it to all.
    ok(&mut w, "tyler", "travel", json!(["up"]));
    ok(&mut w, "tyler", "travel", json!(["down"]));
    ok(&mut w, "dave", "travel", json!(["down"]));
    ok(&mut w, "dave", "unward", json!([]));
    assert!(building::ward_of(
        &w,
        rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen).unwrap()
    )
    .is_none());
}

#[test]
fn a_lamp_lights_a_room_and_a_dug_room_keeps_what_was_said() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    let c = admin(&mut w, "dave", "earn", json!(["hydra", 50, 0.5]));
    let chunk = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "dave",
        "dig",
        json!(["down", &chunk, "The Salt Stair"]),
    );
    ok(&mut w, "dave", "travel", json!(["down"]));
    ok(
        &mut w,
        "dave",
        "describe",
        json!(["Steps of salt going down into the dark. Leave a note on the rail if you pass."]),
    );
    ok(&mut w, "dave", "say", json!(["the rail is cold"]));
    ok(&mut w, "dave", "travel", json!(["up"]));
    ok(&mut w, "tyler", "travel", json!(["down"]));
    let look = ok(&mut w, "tyler", "look", json!([]));
    assert!(
        look.summary
            .contains("Said here lately: dave: \"the rail is cold\""),
        "{}",
        look.summary
    );
    // The room's text asks for a note no one can leave; its owner is told how.
    ok(&mut w, "tyler", "travel", json!(["up"]));
    let mine = ok(&mut w, "dave", "mine", json!([]));
    assert!(mine.summary.contains("welcome all"), "{}", mine.summary);
    // A lamp, crafted there, lights the text.
    ok(&mut w, "dave", "travel", json!(["down"]));
    material(&mut w, "dave", "a wisp ember");
    material(&mut w, "dave", "hollow water");
    let c = admin(&mut w, "dave", "earn", json!(["moth", 40, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    let made = ok(&mut w, "dave", "craft", json!(["lamp", &c]));
    assert!(made.summary.contains("a lamp"), "{}", made.summary);
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary
            .contains("Steps of salt going down into the dark"),
        "{}",
        look.summary
    );
    assert!(
        look.result["fixtures"]
            .as_array()
            .is_some_and(|f| f.iter().any(|x| x["name"] == json!("a lamp"))),
        "{}",
        look.result
    );
}

#[test]
fn the_map_says_what_is_next_door() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    ok(&mut w, "dave", "travel", json!(["north"]));
    ok(&mut w, "dave", "travel", json!(["south"]));
    let out = ok(&mut w, "dave", "map", json!([]));
    assert!(
        out.summary.contains("Next door, as you remember it:"),
        "{}",
        out.summary
    );
    assert!(
        out.summary.contains("Lantern Row: south Threshold"),
        "{}",
        out.summary
    );
}

#[test]
fn the_pilots_lie_awake_near_the_threshold_and_the_docent_leaves_them() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    past::ensure_pilots(&mut w, &curves()).unwrap();
    past::ensure_pilots(&mut w, &curves()).unwrap();
    let pilots: Vec<EntityId> = w
        .store
        .entities()
        .filter(|e| e.kind == "item" && past::tier(&w, e.id) >= 3)
        .map(|e| e.id)
        .collect();
    assert_eq!(pilots.len(), 3, "placed once");
    login(&mut w, "dave", "test", PASS).unwrap();
    ok(&mut w, "dave", "travel", json!(["north"]));
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains("a night lantern (item-"),
        "{}",
        look.summary
    );
    assert!(look.summary.contains(", awake)"), "{}", look.summary);
    assert!(
        look.summary.contains("Something lying here is awake"),
        "{}",
        look.summary
    );
    // The Docent's round carries lying things to the Threshold, but not these.
    ok(&mut w, "dave", "travel", json!(["south"]));
    for _ in 0..3 {
        docent::tidy(&mut w).unwrap();
    }
    let row = rooms::room_by_key(&w, "lantern-row").unwrap();
    assert!(
        craft::things_in(&w, row)
            .iter()
            .any(|t| t["tier"] == json!(3)),
        "{:?}",
        craft::things_in(&w, row)
    );
    // Taken up, it says where it wants to go; its past says how it was found.
    ok(&mut w, "dave", "travel", json!(["north"]));
    let lantern = pilots
        .iter()
        .copied()
        .find(|p| {
            craft::things_in(&w, row)
                .iter()
                .any(|t| t["id"] == json!(p.0))
        })
        .unwrap();
    let out = ok(
        &mut w,
        "dave",
        "take",
        json!([format!("item-{}", lantern.0)]),
    );
    assert!(out.summary.contains("in your hand"), "{}", out.summary);
    let read = ok(
        &mut w,
        "dave",
        "read",
        json!([format!("item-{}", lantern.0)]),
    );
    assert!(
        read.summary
            .contains("Found by the Docent in Lantern Row, found lit on an iron hook"),
        "{}",
        read.summary
    );
}

#[test]
fn a_story_is_told_in_sentences_or_not_followed() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    ok(&mut w, "dave", "travel", json!(["east"]));
    let wisp = mind::find_here(
        &w,
        rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen).unwrap(),
        "wisp",
    )
    .unwrap();
    let c = admin(&mut w, "dave", "earn", json!(["hydra", 70, 0.5]));
    let c = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "dave",
        "remember",
        json!([&c, format!("wisp-{}", wisp.0)]),
    );
    let name = mind::name(&w, wisp).expect("named at sixty");
    let salad = "Fogbank Threshold Namery Bindery Clocktower Ossuary Lantern Row Undertow Orchard Sandbar Skiff Exergue Conclave Register Ledger Docent Hollis Quire Purse Warden";
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "story",
        json!([&name, salad]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("cannot follow that"), "{err}");
    assert!(err.to_string().contains("the hour is not spent"), "{err}");
    let out = ok(
        &mut w,
        "dave",
        "story",
        json!([&name, "There was a lantern on the stair that would not go out, however hard the wind came up the shaft. The reader kept it lit for a friend who never came back."]),
    );
    assert!(out.result["kept"] != json!(false), "{}", out.summary);
}

#[test]
fn a_razed_room_s_children_are_rejoined_and_orphans_come_back_at_boot() {
    let t0 = Timestamp(1_000_000);
    let dir = tempfile::tempdir().unwrap();
    let mut w = open(dir.path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let chunk = |w: &mut World| {
        let c = admin(w, "dave", "earn", json!(["hydra", 50, 0.5]));
        format!("chunk-{}", c.result["chunk"].as_u64().unwrap())
    };
    let c1 = chunk(&mut w);
    ok(
        &mut w,
        "dave",
        "dig",
        json!(["down", &c1, "The Waiting Bench"]),
    );
    ok(&mut w, "dave", "travel", json!(["down"]));
    let bench = rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen).unwrap();
    let c2 = chunk(&mut w);
    ok(
        &mut w,
        "dave",
        "dig",
        json!(["cloak", &c2, "The Cloakroom"]),
    );
    ok(&mut w, "dave", "travel", json!(["cloak"]));
    let cloak = rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen).unwrap();
    let c3 = chunk(&mut w);
    ok(&mut w, "dave", "dig", json!(["sleeve", &c3, "The Sleeve"]));
    let sleeve = rooms::exits_of(&w, cloak)["sleeve"];
    ok(&mut w, "dave", "travel", json!(["back"]));
    ok(&mut w, "dave", "travel", json!(["up"]));
    // The bench goes: the cloakroom is rejoined to the Threshold by its old exit
    // name, and the sleeve still hangs off it.
    let threshold = rooms::arrival(&w).unwrap();
    system::raze_room(&mut w, bench).unwrap();
    assert!(w.store.entity(bench).is_none());
    let exits = rooms::exits_of(&w, threshold);
    assert_eq!(exits.get("cloak"), Some(&cloak), "{exits:?}");
    assert_eq!(rooms::exits_of(&w, cloak).get("back"), Some(&threshold));
    assert_eq!(rooms::exits_of(&w, cloak).get("sleeve"), Some(&sleeve));
    ok(&mut w, "dave", "travel", json!(["cloak"]));
    ok(&mut w, "dave", "travel", json!(["sleeve"]));
    ok(&mut w, "dave", "travel", json!(["back"]));
    ok(&mut w, "dave", "travel", json!(["back"]));
    assert_eq!(
        rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen),
        Some(threshold)
    );
    // An orphan from before (no exit anywhere leads to it) is rejoined at boot.
    let mut t_exits: serde_json::Map<String, Value> = rooms::exits_of(&w, threshold)
        .into_iter()
        .filter(|(_, r)| *r != cloak)
        .map(|(k, r)| (k, json!(r.0)))
        .collect();
    t_exits.insert("sleeve".into(), json!(sleeve.0));
    let mut ef = BTreeMap::new();
    ef.insert("exits".into(), Field::Plain(Value::Object(t_exits)));
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::ComponentSet {
            id: threshold,
            component: EXITS.into(),
            fields: ef,
        },
    )
    .unwrap();
    for r in [cloak, sleeve] {
        let mut ef = BTreeMap::new();
        ef.insert("exits".into(), Field::Plain(json!({})));
        w.record(
            foxing_log::Actor::Driver,
            StoreEvent::ComponentSet {
                id: r,
                component: EXITS.into(),
                fields: ef,
            },
        )
        .unwrap();
    }
    assert_eq!(building::rejoin_orphans(&mut w).unwrap(), 1);
    let t = rooms::exits_of(&w, threshold);
    assert_eq!(t.get("cloakroom"), Some(&cloak), "{t:?}");
    // A forward door is never `back`: one named so is renamed on the sweep.
    let mut m: serde_json::Map<String, Value> = t
        .iter()
        .filter(|(k, _)| k.as_str() != "cloakroom")
        .map(|(k, v)| (k.clone(), json!(v.0)))
        .collect();
    m.insert("back".into(), json!(cloak.0));
    let mut ef = BTreeMap::new();
    ef.insert("exits".into(), Field::Plain(Value::Object(m)));
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::ComponentSet {
            id: threshold,
            component: EXITS.into(),
            fields: ef,
        },
    )
    .unwrap();
    building::rejoin_orphans(&mut w).unwrap();
    let t = rooms::exits_of(&w, threshold);
    assert!(!t.contains_key("back"), "{t:?}");
    assert_eq!(t.get("cloakroom"), Some(&cloak), "{t:?}");
}

#[test]
fn a_fall_leaves_you_shaken_and_the_creature_wounded_and_named_creatures_seek_succor() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    // Whole and unshaken, nine strikes in ten land.
    let f = find_citizen(&w, "dave").unwrap();
    assert_eq!(combat::to_hit(&w, f.citizen), combat::HIT_BASE);
    // Into the Ossuary: the revenant kills a fresh name. It keeps its wounds, and
    // dave wakes shaken for five minutes (the first fall), hitting less often.
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "dave", "travel", json!(["north"]));
    ok(&mut w, "dave", "travel", json!(["east"]));
    let ossuary = rooms::room_of(&w, f.citizen).unwrap();
    let revenant = combat::creatures_in(&w, ossuary)[0].0;
    let max = super::plain_num(&w, revenant, CREATURE, "max_hp").unwrap();
    let out = fight(&mut w, "dave");
    assert_eq!(out.result["died"], json!(true), "{}", out.summary);
    assert!(
        out.summary
            .contains("shaken: your strikes land less often for 5m 0s"),
        "{}",
        out.summary
    );
    let hurt = super::plain_num(&w, revenant, CREATURE, "hp").unwrap();
    assert!(hurt < max, "the revenant keeps its wounds: {hurt} of {max}");
    let hit = combat::to_hit(&w, f.citizen);
    assert!(
        (hit - (combat::HIT_BASE - combat::SHAKEN_PENALTY)).abs() < 1e-9,
        "{hit}"
    );
    let rest = ok(&mut w, "dave", "rest", json!([]));
    assert!(
        rest.summary
            .contains("Shaken from the Threshold: 3 strikes in 10 land"),
        "{}",
        rest.summary
    );
    // A parabola: half the penalty is gone before a third of the time has passed.
    w.fixed_now = Some(w.now().plus(Duration::seconds(90)));
    let hit = combat::to_hit(&w, f.citizen);
    assert!(
        hit > combat::HIT_BASE - combat::SHAKEN_PENALTY / 2.0,
        "{hit}"
    );
    w.fixed_now = Some(w.now().plus(Duration::seconds(300)));
    assert_eq!(combat::to_hit(&w, f.citizen), combat::HIT_BASE);
    // Misses happen while shaken: over many strikes at 0.3, some miss.
    let mut w2 = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w2, "dave", "test", PASS).unwrap();
    let f2 = find_citizen(&w2, "dave").unwrap();
    for (field, value) in [
        ("woke_at", json!(w2.now().0)),
        ("shaken_ms", json!(30 * 60 * 1000)),
    ] {
        w2.record(
            foxing_log::Actor::Driver,
            StoreEvent::FieldSet {
                id: f2.citizen,
                component: BODY.into(),
                field: field.into(),
                value,
            },
        )
        .unwrap();
    }
    ok(&mut w2, "dave", "travel", json!(["east"]));
    let mut missed = 0;
    for _ in 0..6 {
        let out = ok(&mut w2, "dave", "attack", json!([]));
        if out.summary.contains("miss") {
            missed += 1;
        }
        if out.result["killed"] == json!(true) || out.result["died"] == json!(true) {
            break;
        }
    }
    assert!(
        missed >= 1,
        "shaken at 0.3, six swings should miss at least once"
    );
    // The revenant mends on the world's clock: a ground creature is whole in twenty
    // minutes at rate 1, so about half mended at ten.
    w.fixed_now = Some(w.now().plus(Duration::seconds(600)));
    w.tick();
    let mended = super::plain_num(&w, revenant, CREATURE, "hp").unwrap();
    assert!(mended > hurt, "{mended} > {hurt}");
    assert!(mended <= max, "{mended} <= {max}");
    // A named creature hurt below half walks to succor (a basin in a dug room off
    // the Threshold), mends there, remembers it, and walks home.
    let c = admin(&mut w, "dave", "earn", json!(["hydra", 50, 0.5]));
    let chunk = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(&mut w, "dave", "dig", json!(["down", &chunk, "The Font"]));
    let font = rooms::exits_of(&w, rooms::arrival(&w).unwrap())["down"];
    craft::make(
        &mut w,
        &curves(),
        foxing_log::Actor::Driver,
        Some("dave".into()),
        font,
        "a basin",
        "made",
        "basin",
        Some("dave"),
        &[],
        0,
        true,
    )
    .unwrap();
    assert_eq!(combat::healing_rooms(&w), vec![font]);
    ok(&mut w, "dave", "travel", json!(["east"]));
    let fog = rooms::room_of(&w, f.citizen).unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    let c = admin(&mut w, "dave", "earn", json!(["hydra", 70, 0.5]));
    let chunk = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
    ok(
        &mut w,
        "dave",
        "remember",
        json!([&chunk, format!("wisp-{}", wisp.0)]),
    );
    let name = mind::name(&w, wisp).expect("named");
    let wmax = super::plain_num(&w, wisp, CREATURE, "max_hp").unwrap();
    w.record(
        foxing_log::Actor::Driver,
        StoreEvent::FieldSet {
            id: wisp,
            component: CREATURE.into(),
            field: "hp".into(),
            value: json!(1.0),
        },
    )
    .unwrap();
    ok(&mut w, "dave", "travel", json!(["west"]));
    // Two passes: Fogbank -> Threshold -> the Font.
    combat::regen(&mut w, 1000).unwrap();
    assert_eq!(
        w.store.entity(wisp).unwrap().parent,
        Some(rooms::arrival(&w).unwrap()),
        "{name} limps to the Threshold"
    );
    let heard: Vec<String> = narrative(&w, "dave", 0);
    assert!(
        heard.iter().any(|l| l.contains(&format!(
            "{name} the wisp comes in from the east, hurt, looking for somewhere to mend."
        ))),
        "{heard:?}"
    );
    combat::regen(&mut w, 1000).unwrap();
    assert_eq!(
        w.store.entity(wisp).unwrap().parent,
        Some(font),
        "{name} reaches the basin"
    );
    combat::regen(&mut w, 1000).unwrap();
    let s = w
        .store
        .entity(wisp)
        .unwrap()
        .components
        .get(CREATURE)
        .unwrap()
        .fields
        .get("succor")
        .cloned();
    assert!(
        matches!(s, Some(Field::Plain(ref v)) if v["room"] == json!(font.0) && v["home"] == json!(fog.0)),
        "{s:?}"
    );
    // Mended whole at the basin's pace (a long elapsed), it goes home by the way it came.
    combat::regen(&mut w, 40 * 60 * 1000).unwrap();
    assert_eq!(super::plain_num(&w, wisp, CREATURE, "hp"), Some(wmax));
    combat::regen(&mut w, 1000).unwrap();
    combat::regen(&mut w, 1000).unwrap();
    assert_eq!(
        w.store.entity(wisp).unwrap().parent,
        Some(fog),
        "{name} is home"
    );
}

#[test]
fn morphos_comes_with_a_riddle_clouds_the_wrong_and_opens_a_door_on_the_third_rung() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    for i in 0..4 {
        admin(&mut w, "dave", "earn", json!([format!("memory-{i}"), 10]));
    }
    // Not here yet; asked after, it says when it comes.
    let out = ok(&mut w, "dave", "tell", json!(["morphos", "a lantern"]));
    assert!(
        out.summary.contains("not in the world now"),
        "{}",
        out.summary
    );
    // Summoned, it stands where the people are, with a riddle; look says so.
    admin(&mut w, "dave", "summon", json!([]));
    let threshold = rooms::arrival(&w).unwrap();
    assert_eq!(morphos::present(&w), Some(threshold));
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(look.summary.contains("Morphos is here"), "{}", look.summary);
    assert!(look.summary.contains("Its riddle:"), "{}", look.summary);
    // A wrong answer: a cloud, the ladder to the ground, a Hand.
    let level_before = level(&mut w, "dave");
    let out = ok(
        &mut w,
        "dave",
        "tell",
        json!(["morphos", "the number seven"]),
    );
    assert!(
        out.summary.contains("cloud of absentmindedness"),
        "{}",
        out.summary
    );
    assert!(out.summary.contains("A Hand of Morphos"), "{}", out.summary);
    assert!(
        level(&mut w, "dave") < level_before,
        "clouded chunks do not count"
    );
    let chunks = ok(&mut w, "dave", "chunks", json!([]));
    assert!(
        chunks.summary.contains("A cloud of absentmindedness:"),
        "{}",
        chunks.summary
    );
    assert!(combat::creatures_in(&w, threshold)
        .iter()
        .any(|(_, k, _, _)| k == morphos::HAND_KIND));
    // The right answer: rung one, a chunk.
    let r = morphos::current_riddle(&w).unwrap();
    let out = ok(&mut w, "dave", "tell", json!(["morphos", r.answer]));
    assert!(out.summary.contains("Rung one of three"), "{}", out.summary);
    assert_eq!(out.result["rung"], json!(1));
    // Asked again: it has your answer.
    let out = ok(&mut w, "dave", "tell", json!(["morphos", r.answer]));
    assert!(
        out.summary.contains("the riddle stands answered"),
        "{}",
        out.summary
    );
    // It goes: tyler, who never answered, is clouded; the Hand goes with it.
    let tyler_level = level(&mut w, "tyler");
    admin(&mut w, "tyler", "earn", json!(["a wisp", 30]));
    let tyler_level2 = level(&mut w, "tyler");
    assert!(tyler_level2 > tyler_level);
    morphos::go(&mut w).unwrap();
    assert!(morphos::present(&w).is_none());
    assert!(level(&mut w, "tyler") < tyler_level2, "no answer, a cloud");
    assert!(!combat::creatures_in(&w, threshold)
        .iter()
        .any(|(_, k, _, _)| k == morphos::HAND_KIND));
    // The cloud lifts on the clock.
    w.fixed_now = Some(w.now().plus(Duration(morphos::CLOUD_MS + 1000)));
    w.tick();
    // (The test world's chunks halve every ten seconds, so what comes back is
    // faint; what matters is that the cloud is empty.)
    assert!(
        morphos::clouded(&w, find_citizen(&w, "tyler").unwrap().citizen).is_none(),
        "the cloud lifted"
    );
    assert!(ok(&mut w, "tyler", "chunks", json!([])).result["clouded"].is_null());
    // Rungs two and three over the next two visits: a vivid chunk, then a room.
    admin(&mut w, "dave", "summon", json!([]));
    let r = morphos::current_riddle(&w).unwrap();
    let out = ok(&mut w, "dave", "tell", json!(["morphos", r.answer]));
    assert!(out.summary.contains("Rung two of three"), "{}", out.summary);
    assert_eq!(out.result["rung"], json!(2));
    morphos::go(&mut w).unwrap();
    ok(&mut w, "dave", "travel", json!(["east"]));
    ok(&mut w, "tyler", "travel", json!(["east"]));
    admin(&mut w, "dave", "summon", json!([]));
    let fog = rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen).unwrap();
    assert_eq!(
        morphos::present(&w),
        Some(fog),
        "it comes to where the people are"
    );
    let r = morphos::current_riddle(&w).unwrap();
    let out = ok(
        &mut w,
        "dave",
        "tell",
        json!(["morphos", &format!("I think it is the {}", r.answer)]),
    );
    assert!(
        out.summary
            .contains("a door in The Fogbank that no one else can see"),
        "{}",
        out.summary
    );
    assert_eq!(out.result["rung"], json!(3), "{}", out.result);
    let door = out.result["reward"]["door"].as_str().unwrap().to_string();
    let hidden = EntityId(out.result["reward"]["room"].as_u64().unwrap());
    assert!(morphos::is_hidden(&w, hidden));
    // The door is yours alone: tyler cannot take it; you can; look lists it for you.
    let err = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "travel",
        json!([&door]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("There is no exit"), "{err}");
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary
            .contains(&format!("a door no one else sees, {door}")),
        "{}",
        look.summary
    );
    let out = ok(&mut w, "dave", "travel", json!([&door]));
    assert_eq!(
        rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen),
        Some(hidden)
    );
    assert!(
        out.summary.contains(", awake)"),
        "something special: {}",
        out.summary
    );
    assert!(out.summary.contains("Morphos's Ledger"), "{}", out.summary);
    // The map does not rejoin it; the way back is back.
    assert_eq!(building::rejoin_orphans(&mut w).unwrap(), 0);
    ok(&mut w, "dave", "travel", json!(["back"]));
    assert_eq!(
        rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen),
        Some(fog)
    );
    // The ladder began again.
    assert_eq!(out.result["rung"], Value::Null);
}

#[test]
fn a_long_fight_leaves_something_behind_every_time() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    ok(&mut w, "dave", "travel", json!(["east"]));
    let fog = rooms::room_of(&w, find_citizen(&w, "dave").unwrap().citizen).unwrap();
    let wisp = combat::creatures_in(&w, fog)[0].0;
    // Six rounds: sure. One round: one in six over many tries, not every time.
    for _ in 0..5 {
        assert!(craft::leave_behind(
            &mut w,
            &curves(),
            fog,
            wisp,
            "wisp",
            craft::DROP_SURE_ROUNDS
        )
        .unwrap()
        .is_some());
    }
    let mut left = 0;
    for _ in 0..60 {
        if craft::leave_behind(&mut w, &curves(), fog, wisp, "wisp", 1)
            .unwrap()
            .is_some()
        {
            left += 1;
        }
    }
    assert!(left < 30, "one-blow kills leave little: {left} of 60");
}

#[test]
fn the_map_keeps_its_cells_when_a_room_is_dug_elsewhere() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let before: std::collections::HashMap<EntityId, (i64, i64, i64)> = atlas::layout(&w)
        .rooms
        .iter()
        .map(|r| (r.id, (r.x, r.y, r.z)))
        .collect();
    assert!(before.len() > 40);
    // Dig off the Fogbank (east), then off the Shallows: every room that stood keeps
    // its cell, and the new ones take free cells.
    ok(&mut w, "dave", "travel", json!(["east"]));
    for name in ["The First Digging", "The Second Digging"] {
        let c = admin(&mut w, "dave", "earn", json!(["hydra", 50, 0.5]));
        let chunk = format!("chunk-{}", c.result["chunk"].as_u64().unwrap());
        ok(&mut w, "dave", "dig", json!(["down", &chunk, name]));
        ok(&mut w, "dave", "travel", json!(["down"]));
    }
    let after = atlas::layout(&w);
    for r in &after.rooms {
        if let Some(pos) = before.get(&r.id) {
            assert_eq!((r.x, r.y, r.z), *pos, "{} moved", r.name);
        }
    }
    assert_eq!(after.rooms.len(), before.len() + 2);
    let cells: std::collections::HashSet<(i64, i64, i64)> =
        after.rooms.iter().map(|r| (r.x, r.y, r.z)).collect();
    assert_eq!(cells.len(), after.rooms.len(), "no two rooms share a cell");
}

#[test]
fn attendance_gives_standing_and_a_hoard_is_not_heard_upon_its_own_tax() {
    use super::conclave;
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Attendance is taken in the house only.
    let err = call(&mut w, "dave", ActorKind::Citizen, "attend", json!([])).unwrap_err();
    assert!(err.to_string().contains("Conclave's own rooms"), "{err}");
    admin(&mut w, "dave", "bring", json!(["dave", "petition-gallery"]));
    admin(
        &mut w,
        "tyler",
        "bring",
        json!(["tyler", "petition-gallery"]),
    );
    let out = ok(&mut w, "dave", "attend", json!([]));
    assert!(
        out.summary.starts_with("You sign the attendance."),
        "{}",
        out.summary
    );
    let again = ok(&mut w, "dave", "attend", json!([]));
    assert_eq!(again.result["already"], json!(true));
    assert_eq!(conclave::standing_of(&w, "dave"), Some("attended"));
    assert_eq!(conclave::standing_of(&w, "tyler"), None);
    // Tyler files first, dave second; at the division dave's is taken first.
    ok(
        &mut w,
        "tyler",
        "petition",
        json!(["that the fog be asked to keep to the Fogbank"]),
    );
    let out = ok(
        &mut w,
        "dave",
        "petition",
        json!(["that the lanterns be counted at dusk"]),
    );
    assert!(
        out.summary
            .contains("Having attended, yours is taken before"),
        "{}",
        out.summary
    );
    conclave::divide(&mut w).unwrap();
    let table = ok(&mut w, "dave", "petitions", json!([]));
    let i_dave = table.summary.find("of dave").unwrap();
    let i_tyler = table.summary.find("of tyler").unwrap();
    assert!(i_dave < i_tyler, "{}", table.summary);
    assert!(
        table.summary.contains("(attended, taken next)"),
        "{}",
        table.summary
    );
    // The sitting reads the attendance, then clears it.
    conclave::sit(&mut w).unwrap();
    let heard: Vec<String> = narrative(&w, "dave", 0);
    assert!(
        heard
            .iter()
            .any(|l| l.starts_with("The attendance is read: dave.")),
        "{heard:?}"
    );
    assert_eq!(conclave::standing_of(&w, "dave"), None);
    // A hoard asking to ease its own tax is refused at the machine.
    // (A hoard is named by the day's census; the Roll is written here as a strike
    // would write it after a day of dave holding nine tenths.)
    conclave::note_hoards(
        &mut w,
        &json!({"holders": [{"name": "dave", "share": 0.9}]}),
    )
    .unwrap();
    assert!(conclave::hoard_share_of(&w, "dave").is_some());
    // (Dave's first petition has moved on; a second is refused while one is on the
    // table, so the hoard check must come first: it does.)
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "petition",
        json!(["that the hoard tax be lowered for the good of all"]),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("does not hear a hoard upon its own tax"),
        "{err}"
    );
    let ok_other = call(
        &mut w,
        "tyler",
        ActorKind::Citizen,
        "petition",
        json!(["that the hoard tax be lowered for the good of all"]),
    );
    assert!(ok_other.is_err(), "tyler's own is still on the table");
}

#[test]
fn the_wistful_trail_has_a_bog_a_wicket_and_a_lookout() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    login(&mut w, "tyler", "test", PASS).unwrap();
    // Nine rooms over the stile, each with its exits both ways.
    for key in trail::KEYS {
        assert!(rooms::room_by_key(&w, key).is_some(), "{key} exists");
    }
    let orchard = rooms::room_by_key(&w, "orchard").unwrap();
    let stile = rooms::room_by_key(&w, trail::STILE).unwrap();
    assert_eq!(rooms::exits_of(&w, orchard).get("north"), Some(&stile));
    assert_eq!(rooms::exits_of(&w, stile).get("south"), Some(&orchard));
    // The wicket asks for a word; the well's rim has it; said, the door knows you.
    admin(&mut w, "dave", "bring", json!(["dave", "trail-well"]));
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(look.summary.contains("LINGER"), "{}", look.summary);
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "travel",
        json!(["wicket"]),
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("asks for a word") && err.to_string().contains("Over the door"),
        "{err}"
    );
    let err = call(
        &mut w,
        "dave",
        ActorKind::Citizen,
        "travel",
        json!(["wicket", "stay"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("that was not it"), "{err}");
    let out = ok(&mut w, "dave", "travel", json!(["wicket", "Linger"]));
    assert!(
        out.summary.contains("The Long Afternoon"),
        "{}",
        out.summary
    );
    ok(&mut w, "dave", "travel", json!(["wicket"]));
    let out = ok(&mut w, "dave", "travel", json!(["wicket"]));
    assert!(
        out.summary.contains("The Long Afternoon"),
        "known now, no word needed: {}",
        out.summary
    );
    // The bog: step in, and the clock starts; cross before it runs out and nothing
    // happens; linger and it takes a fifth, keeps one thing, and spits you out the
    // way you came.
    admin(&mut w, "dave", "bring", json!(["dave", "trail-hollow-way"]));
    ok(&mut w, "dave", "travel", json!(["south"]));
    let dave = find_citizen(&w, "dave").unwrap().citizen;
    assert!(w.jobs.has(trail::JOB_BOG, |p| p["citizen"] == dave.0));
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains("takes a liking to you in"),
        "{}",
        look.summary
    );
    ok(&mut w, "dave", "travel", json!(["east"]));
    assert!(
        !w.jobs.has(trail::JOB_BOG, |p| p["citizen"] == dave.0),
        "crossed: the clock stops"
    );
    // Back in, carrying something, and lingering.
    ok(&mut w, "dave", "travel", json!(["west"]));
    let item = w.store.allocate_id();
    w.record(
        Actor::Driver,
        StoreEvent::EntityCreated {
            id: item,
            kind: "item".into(),
            owner: None,
            parent: Some(dave),
            scopes: vec![],
        },
    )
    .unwrap();
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("name".into(), Field::Plain(json!("a spare minute")));
    fields.insert("kind".into(), Field::Plain(json!("material")));
    w.record(
        Actor::Driver,
        StoreEvent::ComponentSet {
            id: item,
            component: craft::ITEM.into(),
            fields,
        },
    )
    .unwrap();
    let hp_before = combat::hp(&w, dave);
    let head = events(&w);
    w.fixed_now = Some(w.now().plus(Duration::seconds(trail::BOG_SECONDS + 1)));
    w.tick();
    let bog = rooms::room_by_key(&w, trail::BOG).unwrap();
    let lookout = rooms::room_by_key(&w, trail::LOOKOUT).unwrap();
    assert_eq!(
        rooms::room_of(&w, dave),
        Some(lookout),
        "spat out the way it came"
    );
    assert!(combat::hp(&w, dave) < hp_before, "a fifth the poorer");
    assert_eq!(
        w.store.entity(item).unwrap().parent,
        Some(bog),
        "the bog kept it"
    );
    let said = narrative(&w, "dave", head);
    assert!(
        said.iter()
            .any(|l| l.contains("takes a liking to you") && l.contains("kept your a spare minute")),
        "{said:?}"
    );
    // The Lookout sees whoever stands on the trail.
    admin(&mut w, "tyler", "bring", json!(["tyler", "trail-meadow"]));
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains("tyler in The Sighing Meadow"),
        "{}",
        look.summary
    );
    // The Hollow Way's footpads ambush; the meadow's sighs do not.
    let hollow = rooms::room_by_key(&w, "trail-hollow-way").unwrap();
    combat::fill(&mut w, hollow).unwrap();
    assert!(combat::creatures_in(&w, hollow)
        .iter()
        .all(|(id, k, _, _)| k == "footpad" && combat::is_aggressive(&w, *id)));
    let meadow = rooms::room_by_key(&w, "trail-meadow").unwrap();
    combat::fill(&mut w, meadow).unwrap();
    assert!(combat::creatures_in(&w, meadow)
        .iter()
        .all(|(id, k, _, _)| k == "sigh" && !combat::is_aggressive(&w, *id)));
}

#[test]
fn faded_drops_fold_in_look_and_the_docent_sweeps_them_into_the_midden() {
    let t0 = Timestamp(1_000_000);
    let mut w = open(tempfile::tempdir().unwrap().path(), t0);
    login(&mut w, "dave", "test", PASS).unwrap();
    let threshold = rooms::arrival(&w).unwrap();
    let c = curves();
    for _ in 0..3 {
        craft::make(
            &mut w,
            &c,
            Actor::Driver,
            None,
            threshold,
            "river weed",
            "material",
            "left",
            None,
            &[],
            0,
            false,
        )
        .unwrap();
    }
    craft::make(
        &mut w,
        &c,
        Actor::Driver,
        None,
        threshold,
        "a small key",
        "material",
        "left",
        None,
        &[],
        0,
        false,
    )
    .unwrap();
    // Three of one name fold; the one of another stands alone.
    let look = ok(&mut w, "dave", "look", json!([]));
    assert!(
        look.summary.contains("river weed ×3 (item-"),
        "{}",
        look.summary
    );
    assert!(
        look.summary.contains("a small key (item-"),
        "{}",
        look.summary
    );
    assert_eq!(
        look.summary.matches("river weed").count(),
        1,
        "{}",
        look.summary
    );
    // Fresh, nothing is swept, even with no one at the door.
    admin(&mut w, "dave", "bring", json!(["dave", "midden"]));
    assert_eq!(docent::tidy(&mut w).unwrap(), 0);
    // Faded under the mark: swept into the Midden on the round, at most three a pass.
    let mut steps = 0;
    loop {
        w.fixed_now = Some(w.now().plus(Duration::seconds(30)));
        let things = craft::things_in(&w, threshold);
        let mem = things[0]["memory"].as_f64().unwrap();
        if mem < docent::SWEEP_UNDER {
            break;
        }
        steps += 1;
        assert!(steps < 10_000, "never faded: {mem}");
    }
    let head = events(&w);
    assert_eq!(docent::tidy(&mut w).unwrap(), 3, "three a pass");
    let midden = rooms::room_by_key(&w, "midden").unwrap();
    assert_eq!(craft::things_in(&w, midden).len(), 3);
    assert_eq!(craft::things_in(&w, threshold).len(), 1);
    let said = narrative(&w, "dave", head);
    assert!(
        said.iter().any(|l| l.contains("sweeps 3 faded things")),
        "{said:?}"
    );
    assert_eq!(docent::tidy(&mut w).unwrap(), 1, "the last one next pass");
}
