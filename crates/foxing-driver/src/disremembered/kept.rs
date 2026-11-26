//! C11: the Kept House (David, 11-25 and 11-26).
//!
//! Six rooms off the Lamplighter's Lane where six named creatures board. The Watch
//! of 11-25 ran out of meanings in five minutes with six named creatures in the
//! world, and names made in one session faded before the next. The residents do not
//! fade out: each has a name the house gave it and keeps it whatever it holds, a
//! word and two meanings like any named creature (secret.rs, meaning.rs), a room of
//! its own, a way of talking, and about a hundred of memory that the house tops up
//! while it is home. Fed past a hundred and fifty, a resident takes a calling like
//! any other and comes home to the house after it (calling.rs).
//!
//! David's rulings (11-26): **they travel**, one at a time, and a traveler takes the
//! absent one's room; **they can be fought**, are very tough, and are back at their
//! own door soon after.
//!
//! Travel. The week turns on UTC days: on each of six days one resident is away on
//! the road (a room no door leads to) and a traveler lodges in its room, a named
//! creature with a name from the road and a word and meanings of its own, new each
//! stay; on the seventh everyone is home. A resident with a calling in hand does not
//! leave until it is done.
//!
//! Fighting. A resident is a creature and may be struck by name. It is very tough
//! (six hundred hit points before what it holds is counted). Felled, it gives up a
//! memory as any holding creature does, and is carried to its own room, whole; one
//! that came apart altogether is back in its room at the house's next turn, within
//! ten minutes, under its own name.
//!
//! The Visitors' Book stands in the hall: six lines, one left by each resident, to
//! be matched to who would have written it (`ascribe <line> <name>`). The residents
//! give themselves away in what they do and say. A right match pays a vivid memory of
//! ten, all six a further thirty, once a name a month; a wrong one closes that line
//! to you for an hour. The lines change with the month.

use super::mind;
use super::{find_citizen, me, plain_str, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const HALL: &str = "kept-hall";
pub const ROAD: &str = "kept-road";
pub const TRAVELER: &str = "traveler";
/// A resident left for the road or came home, or a traveler came or went:
/// `{resident, name, stage ("left", "home"), traveler}`. Public.
pub const TRAVELED: &str = "disremembered.kept_traveled";
/// A line of the Visitors' Book was ascribed: `{by, line, to, right, all}`. Public.
pub const ASCRIBED: &str = "disremembered.ascribed";
/// What the house keeps a resident at while it is home.
pub const KEPT_AT: f64 = 100.0;
/// What a right ascription is worth, and all six.
pub const LINE_XP: f64 = 10.0;
pub const BOOK_XP: f64 = 30.0;
/// A wrong ascription closes that line to that name this long.
pub const WRONG_MS: i64 = 3_600_000;

pub struct Resident {
    /// The creature kind (combat::CREATURES), which is also what it does in the house.
    pub kind: &'static str,
    pub name: &'static str,
    pub room: &'static str,
    /// Its lines in the Visitors' Book, one a month in turn.
    pub book: [&'static str; 3],
    /// What it leaves in its room when it is on the road, by which it can be known.
    pub trace: &'static str,
}

pub const RESIDENTS: &[Resident] = &[
    Resident {
        kind: "housekeeper",
        name: "Wimble",
        room: HALL,
        book: [
            "Arrived with two bags; leaving with two bags. Counted both times.",
            "Eleven spoons in the drawer. There were eleven yesterday. I mention it for completeness.",
            "One guest, one candle, one complaint. All three accounted for.",
        ],
        trace: "A list is pinned by the door in a square hand: \"Hooks, seven. Coats, six. Mine makes seven; taken with me. Back tomorrow, to count again.\"",
    },
    Resident {
        kind: "watchman",
        name: "Sallow",
        room: "kept-parlour",
        book: [
            "Came in late. Everyone does; the door does not mind and neither do I.",
            "It is a quarter past something. It usually is.",
            "Stayed up until it was early. I recommend it.",
        ],
        trace: "All four clocks have been stopped at different hours, and a note is propped against the last: \"Gone out. Do not wind them; I like to come home to an argument.\"",
    },
    Resident {
        kind: "cook",
        name: "Pettifer",
        room: "kept-kitchen",
        book: [
            "Needed salt. Most things do.",
            "The soup was better the second day, as people are.",
            "Left a crust for whoever is hungry after me. Do not thank me, eat it.",
        ],
        trace: "A pot stands off the heat under a cloth, with a spoon across it and a note: \"Tomorrow's. Salt it when I am back, not before.\"",
    },
    Resident {
        kind: "copyist",
        name: "Quillon",
        room: "kept-landing",
        book: [
            "The sign on the stair says 'Mind you're head'. I have minded it. I have also corrected it.",
            "A good house; a semicolon of a house: it joins what would otherwise be two lonely things.",
            "Whoever wrote above me: 'their', not 'there'.",
        ],
        trace: "A half-copied letter lies on the desk, weighted with the ink bottle; in the margin, small: \"Resume at the semicolon.\"",
    },
    Resident {
        kind: "seamster",
        name: "Dunnock",
        room: "kept-bedroom",
        book: [
            "Found a button on the stair and sewed it to the curtain so it would not be lost again.",
            "Nothing here is broken that a length of thread would not see to.",
            "Mended the hem of the hall rug. Nobody asked. Nobody trips now.",
        ],
        trace: "A needle stands in the arm of the chair with a thread still through it, and one glove has four fingers done.",
    },
    Resident {
        kind: "stargazer",
        name: "Hesper",
        room: "kept-attic",
        book: [
            "Clear tonight from the attic. Three stars I knew and one I did not.",
            "Rain on the skylight until four. I counted it as company.",
            "The window on the east side shows tomorrow a little early.",
        ],
        trace: "The skylight is shut and latched, and tomorrow's weather is pencilled on the chart, with the hour.",
    },
];

/// The names travelers go by, one a stay in turn.
const ROAD_NAMES: &[&str] = &[
    "Ostler", "Marram", "Tolland", "Sedge", "Bramwell", "Isk", "Corrie", "Fenn",
];

fn mind_str(world: &World, c: EntityId, field: &str) -> Option<String> {
    match world
        .store
        .entity(c)?
        .components
        .get(mind::MIND)?
        .fields
        .get(field)?
    {
        Field::Plain(v) => v.as_str().map(str::to_string),
        _ => None,
    }
}

/// The resident this creature is, if it is one.
pub fn resident_of(world: &World, c: EntityId) -> Option<&'static Resident> {
    let key = mind_str(world, c, "resident")?;
    RESIDENTS.iter().find(|r| r.name == key)
}

/// Whether it is a traveler lodging in the house.
pub fn is_traveler(world: &World, c: EntityId) -> bool {
    plain_str(world, c, CREATURE, "name") == Some(TRAVELER)
}

/// Of the house: a resident or a traveler. These keep a name whatever they hold and
/// do not grow old.
pub fn of_house(world: &World, c: EntityId) -> bool {
    resident_of(world, c).is_some() || is_traveler(world, c)
}

fn find(world: &World, r: &Resident) -> Option<EntityId> {
    world
        .store
        .of_kind("creature")
        .find(|e| mind_str(world, e.id, "resident").as_deref() == Some(r.name))
        .map(|e| e.id)
}

fn travelers(world: &World) -> Vec<EntityId> {
    world
        .store
        .of_kind("creature")
        .filter(|e| is_traveler(world, e.id))
        .map(|e| e.id)
        .collect()
}

fn day(world: &World) -> i64 {
    world.now().0.div_euclid(86_400_000)
}

/// Who is on the road today: one resident on each of six days, nobody on the seventh.
pub fn away_today(world: &World) -> Option<&'static Resident> {
    RESIDENTS.get(day(world).rem_euclid(7) as usize)
}

fn here_of(world: &World, c: EntityId) -> Option<EntityId> {
    world.store.entity(c).and_then(|e| e.parent)
}

/// A creature of the house, made whole in a room under a name: spawned, housed,
/// given what it holds, and named.
fn raise(
    world: &mut World,
    kind: &str,
    name: &str,
    room: EntityId,
    resident: Option<&str>,
) -> Result<EntityId, String> {
    let def = super::combat::def_by_name(kind).ok_or_else(|| format!("no creature kind {kind}"))?;
    let c = super::combat::spawn_one(world, room, def).map_err(|e| e.to_string())?;
    // One drawing serves the house; each of them wears its own color.
    let frame = if kind == TRAVELER {
        "traveler"
    } else {
        "boarder"
    };
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id: c,
                component: super::SPRITE.into(),
                fields: super::sprite_fields(def.glyph, def.tint, frame),
            },
        )
        .map_err(|e| e.to_string())?;
    super::life::set_home(world, c, room)?;
    let mut changes = BTreeMap::new();
    changes.insert("heir_name".into(), Field::Plain(json!(name)));
    if let Some(key) = resident {
        changes.insert("resident".into(), Field::Plain(json!(key)));
    }
    mind::set_mind_pub(world, Actor::Driver, c, changes).map_err(|e| e.to_string())?;
    top_up(world, c)?;
    Ok(c)
}

/// The house's keeping: what a creature of the house holds is made up to a hundred.
fn top_up(world: &mut World, c: EntityId) -> Result<(), String> {
    let short = KEPT_AT - mind::level(world, c);
    if short >= 5.0 {
        let w = mind::ensure_window_pub(world, c).map_err(|e| e.to_string())?;
        let curves = super::Curves::from_env();
        super::memory::earn_chunk(
            world,
            "nobody",
            w,
            "the house's keeping: a bed aired, a place laid, a name on a door",
            short,
            0.5,
            &curves,
        )
        .map_err(|e| e.to_string())?;
    }
    mind::check_tier(world, c).map_err(|e| e.to_string())
}

/// The road is a room no door leads to, and is on no map.
fn hide_road(world: &mut World) -> Result<Option<EntityId>, String> {
    let Some(road) = super::rooms::room_by_key(world, ROAD) else {
        return Ok(None);
    };
    if !super::plain_bool(world, road, ROOM, "hidden") {
        let mut changes = BTreeMap::new();
        changes.insert("hidden".into(), Field::Plain(json!(true)));
        super::bindery::set_fields(world, Actor::Driver, road, ROOM, changes)
            .map_err(|e| e.to_string())?;
    }
    Ok(Some(road))
}

/// The house's turn (life.rs, every ten minutes): every resident exists and is where
/// it should be, the one whose day it is is on the road with a traveler in its room,
/// and those at home are kept.
pub fn tick(world: &mut World) -> Result<(), String> {
    let Some(road) = hide_road(world)? else {
        return Ok(());
    };
    let away = away_today(world).map(|r| r.name);
    for r in RESIDENTS {
        let Some(room) = super::rooms::room_by_key(world, r.room) else {
            continue;
        };
        let c = match find(world, r) {
            Some(c) => c,
            // Never made, or it came apart: it is back at its own door.
            None => raise(world, r.kind, r.name, room, Some(r.name))?,
        };
        let here = here_of(world, c);
        let fighting = super::combat::fought_by(world, c).is_some();
        let called = super::calling::of(world, c).is_some();
        if away == Some(r.name) {
            if here != Some(road) && !fighting && !called {
                super::combat::narrate_room(
                    world,
                    c,
                    &format!("{} the {} takes a coat from the hook, says where it is going to nobody in particular, and goes. It will be home tomorrow; someone off the road has its room meanwhile.", r.name, r.kind),
                );
                world
                    .record(
                        Actor::Driver,
                        StoreEvent::EntityMoved {
                            id: c,
                            parent: Some(road),
                        },
                    )
                    .map_err(|e| e.to_string())?;
                let stay = day(world) as usize;
                let name = ROAD_NAMES[stay % ROAD_NAMES.len()];
                let t = raise(world, TRAVELER, name, room, None)?;
                super::combat::narrate_room(
                    world,
                    t,
                    &format!("A traveler comes in off the road with a pack and a look of having been rained on somewhere else: {name}, who has {}'s room until {} is home.", r.name, r.name),
                );
                let _ = world.note(
                    Actor::Driver,
                    TRAVELED,
                    json!({"resident": r.name, "name": r.name, "stage": "left", "traveler": name}),
                );
            }
        } else if here == Some(road) {
            // Home: the traveler in its room goes first.
            for t in travelers(world) {
                if super::combat::fought_by(world, t).is_some() {
                    continue;
                }
                let who = mind::display(world, t);
                super::combat::narrate_room(world, t, &format!("{who} shoulders its pack, leaves the bed made, and is gone down the Lane before anyone thinks to ask where to."));
                world
                    .record(Actor::Driver, StoreEvent::EntityRemoved { id: t })
                    .map_err(|e| e.to_string())?;
            }
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityMoved {
                        id: c,
                        parent: Some(room),
                    },
                )
                .map_err(|e| e.to_string())?;
            super::combat::narrate_room(world, c, &format!("{} the {} is home from the road, hangs the coat on its own hook, and looks the room over for what was moved.", r.name, r.kind));
            let _ = world.note(
                Actor::Driver,
                TRAVELED,
                json!({"resident": r.name, "name": r.name, "stage": "home"}),
            );
        }
        // Kept while it is in the house and not out on something of its own.
        if here_of(world, c) != Some(road) && !fighting {
            top_up(world, c)?;
        }
    }
    // A traveler with nobody away (a day turned while it was fighting) goes when it can.
    if away.is_none()
        || RESIDENTS
            .iter()
            .all(|r| find(world, r).and_then(|c| here_of(world, c)) != Some(road))
    {
        for t in travelers(world) {
            if super::combat::fought_by(world, t).is_none() {
                world
                    .record(Actor::Driver, StoreEvent::EntityRemoved { id: t })
                    .map_err(|e| e.to_string())?;
            }
        }
    } else {
        // Someone is on the road with nobody in the room (a turn was missed and the
        // one coming home sent the new traveler off with the old): one comes.
        if travelers(world).is_empty() {
            if let Some(r) = away_today(world) {
                if let Some(room) = super::rooms::room_by_key(world, r.room) {
                    let name = ROAD_NAMES[day(world) as usize % ROAD_NAMES.len()];
                    raise(world, TRAVELER, name, room, None)?;
                }
            }
        }
        for t in travelers(world) {
            top_up(world, t)?;
        }
    }
    Ok(())
}

/// One of the house found out of its room with nothing in hand goes back to it.
pub fn go_home(world: &mut World, c: EntityId) -> Result<(), String> {
    let Some(r) = resident_of(world, c) else {
        return Ok(());
    };
    let (Some(room), Some(here)) = (super::rooms::room_by_key(world, r.room), here_of(world, c))
    else {
        return Ok(());
    };
    if here == room
        || Some(here) == super::rooms::room_by_key(world, ROAD)
        || super::calling::of(world, c).is_some()
        || super::combat::fought_by(world, c).is_some()
    {
        return Ok(());
    }
    super::combat::narrate_room(
        world,
        c,
        &format!(
            "{} the {} remembers something left undone and goes back to its own room.",
            r.name, r.kind
        ),
    );
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: c,
                parent: Some(room),
            },
        )
        .map_err(|e| e.to_string())?;
    super::combat::narrate_room(
        world,
        c,
        &format!(
            "{} the {} comes back in and takes up where it left off.",
            r.name, r.kind
        ),
    );
    Ok(())
}

/// A resident was felled and stood (combat.rs): it is carried to its own room, whole.
/// What it says to the one who felled it, or `None` for any other creature.
pub fn felled(world: &mut World, c: EntityId) -> Option<String> {
    let r = resident_of(world, c)?;
    let room = super::rooms::room_by_key(world, r.room)?;
    let (_, max) = super::life::creature_hp(world, c);
    let mut changes = BTreeMap::new();
    changes.insert("hp".into(), Field::Plain(json!(max)));
    let _ = super::bindery::set_fields(world, Actor::Driver, c, CREATURE, changes);
    if here_of(world, c) != Some(room) {
        let _ = world.record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: c,
                parent: Some(room),
            },
        );
    }
    Some(format!(
        " The house does not leave its own on the floor: {} is carried to its own room ({}) and is on its feet there, whole, before you have put your hands down.",
        r.name,
        plain_str(world, room, ROOM, "name").unwrap_or("its room")
    ))
}

// ---- wiring: the house's own turn, every ten minutes, the first a minute after the
// world starts.

pub const JOB_KEPT: &str = "disremembered.kept_tick";
const EVERY_MS: i64 = 600_000;

fn wind(world: &mut World, in_ms: i64) -> Result<(), String> {
    let due = world.now().plus(foxing_time::Duration(in_ms));
    world
        .schedule_once(JOB_KEPT, due, json!({}), |_| true)
        .map(|_| ())
}

pub fn ensure(world: &mut World) -> Result<(), String> {
    if !world.jobs.has(JOB_KEPT, |_| true) {
        wind(world, 60_000)?;
    }
    Ok(())
}

pub fn register_jobs(world: &mut World) {
    world.on_job(
        JOB_KEPT,
        Box::new(|w: &mut World, _| {
            // (FOXING_KEPT_OFF: the tests of the world before the house run without
            // six more names in it; the house's own test calls `tick` itself.)
            if std::env::var_os("FOXING_KEPT_OFF").is_some() {
                return wind(w, EVERY_MS);
            }
            // A fault in the house's turn is the house's: the turn comes round again.
            if let Err(e) = tick(w) {
                eprintln!("the Kept House's turn failed: {e}");
            }
            wind(w, EVERY_MS)
        }),
    );
}

// ---- the Visitors' Book

fn month(world: &World) -> i64 {
    // Thirty-day months from the epoch are near enough for a book.
    day(world).div_euclid(30)
}

/// This month's six lines, in the order the book shows them: `(line, who wrote it)`.
pub fn lines(world: &World) -> Vec<(&'static str, &'static str)> {
    let m = month(world);
    let mut all: Vec<(&'static str, &'static str)> = RESIDENTS
        .iter()
        .map(|r| (r.book[m.rem_euclid(3) as usize], r.name))
        .collect();
    // A different order each month, the same for everyone.
    let n = all.len();
    all.rotate_left((m.rem_euclid(n as i64)) as usize);
    if m.rem_euclid(2) == 1 {
        all.reverse();
    }
    all.swap(1, 4);
    all
}

fn book(world: &World) -> Value {
    let Some(hall) = super::rooms::room_by_key(world, HALL) else {
        return json!({});
    };
    match world
        .store
        .entity(hall)
        .and_then(|e| e.components.get(ROOM))
        .and_then(|c| c.fields.get("book"))
    {
        Some(Field::Plain(v)) if v.is_object() && v["month"].as_i64() == Some(month(world)) => {
            v.clone()
        }
        _ => json!({"month": month(world), "right": {}, "wrong": {}}),
    }
}

fn keep_book(world: &mut World, b: Value) -> Result<(), String> {
    let Some(hall) = super::rooms::room_by_key(world, HALL) else {
        return Ok(());
    };
    let mut changes = BTreeMap::new();
    changes.insert("book".into(), Field::Plain(b));
    super::bindery::set_fields(world, Actor::Driver, hall, ROOM, changes)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn found_by(world: &World, actor: &str) -> Vec<u64> {
    book(world)["right"][actor]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_u64).collect())
        .unwrap_or_default()
}

fn boarders(world: &World) -> String {
    RESIDENTS
        .iter()
        .map(|r| {
            format!(
                "{} the {} ({})",
                r.name,
                r.kind,
                super::rooms::room_by_key(world, r.room)
                    .and_then(|x| plain_str(world, x, ROOM, "name"))
                    .unwrap_or("?")
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The hall's line under `look`: that the book is there, how far this name has got,
/// and who is away. The page itself is `read book` (Watch 29: with all six lines in
/// it, every `look` in the hall was three thousand characters).
pub fn look_line(world: &World, room: EntityId, actor: &str) -> Option<String> {
    if Some(room) != super::rooms::room_by_key(world, HALL) {
        return None;
    }
    let done = found_by(world, actor).len();
    let away = away_today(world)
        .map(|r| {
            format!(
                " {} is on the road today and a traveler has the room; home tomorrow.",
                r.name
            )
        })
        .unwrap_or_else(|| " Everyone is home today.".to_string());
    Some(format!(
        "The Visitors' Book lies open on the hall table: six unsigned lines, one by each who boards here (`read book`; you have ascribed {done} of 6).{away}"
    ))
}

/// In the room of one who is on the road: what it left there (Watch 29: with a
/// resident away its line could only be had by elimination).
pub fn trace_line(world: &World, room: EntityId) -> Option<String> {
    let r = away_today(world)?;
    if Some(room) != super::rooms::room_by_key(world, r.room) {
        return None;
    }
    let c = find(world, r)?;
    if here_of(world, c) != super::rooms::room_by_key(world, ROAD) {
        return None;
    }
    Some(format!(
        "{} is on the road, and has left the room as it leaves it: {}",
        r.name, r.trace
    ))
}

/// The book's page, for `read book` in the hall.
pub fn read_book(world: &World, room: EntityId, actor: &str, arg: &str) -> Option<Outcome> {
    if Some(room) != super::rooms::room_by_key(world, HALL)
        || !(arg.contains("book") || arg.contains("page") || arg.contains("visitor"))
    {
        return None;
    }
    let done = found_by(world, actor);
    let shown: Vec<String> = lines(world)
        .iter()
        .enumerate()
        .map(|(i, (line, who))| {
            if done.contains(&(i as u64 + 1)) {
                format!("{}. \"{line}\" ({who}'s: you have it)", i + 1)
            } else {
                format!("{}. \"{line}\"", i + 1)
            }
        })
        .collect();
    Some(Outcome::new(
        format!(
            "This month's page of the Visitors' Book, in six hands, unsigned: {} Six board here: {}. Each has one thing it cannot leave alone, and shows it in what it does when you stand by it a while and in the room it keeps. `ascribe <line> <name>` says who wrote one: {LINE_XP:.0} XP vivid for each you get right, {BOOK_XP:.0} more for all six, once a month; a wrong guess closes that line to you for an hour. The page turns with the month.",
            shown.join(" "),
            boarders(world)
        ),
        json!({"read": "book", "lines": lines(world).iter().map(|l| l.0).collect::<Vec<_>>(), "found": done}),
    ))
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.ascribe",
            "In the hall of the Kept House: say which resident wrote a line of the Visitors' Book (`look` there shows the six lines, numbered). Each right one pays a vivid memory, all six a larger one, once a month; a wrong guess closes that line to you for an hour.",
        )
        .param("line", ParamType::Integer, true, "the line's number in the book, 1 to 6")
        .param("name", ParamType::String, true, "the resident you say wrote it")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            if super::rooms::room_of(w, f.citizen) != super::rooms::room_by_key(w, HALL) || super::rooms::room_by_key(w, HALL).is_none() {
                return Err(CommandError::precondition(
                    "The Visitors' Book is on the hall table of the Kept House, `stoop` off the Lamplighter's Lane (`where \"Kept House\"`).",
                ));
            }
            // The book is for a name that means to come back (Watch 29: a fresh
            // passing name took all six in one message, and another could after it).
            if super::is_passing(w, f.citizen) {
                return Err(CommandError::precondition(
                    "The book is ascribed by a name that is kept: a passing name is gone in an hour, and the page is a month's. `keep` makes yours one (the first is free), and then the book is open to you.",
                ));
            }
            let all = lines(w);
            let n = args.int("line").unwrap_or(0);
            if n < 1 || n as usize > all.len() {
                return Err(CommandError::precondition(format!("The page has {} lines; `ascribe <line> <name>` wants one of them by its number.", all.len())));
            }
            let said = args.str("name").unwrap_or("").trim().trim_matches('"').to_lowercase();
            let said = said.trim_start_matches("the ").to_string();
            let Some(guess) = RESIDENTS
                .iter()
                .find(|r| r.name.to_lowercase() == said || r.kind == said || said == format!("{} the {}", r.name.to_lowercase(), r.kind))
            else {
                return Err(CommandError::precondition(format!(
                    "Nobody of that name boards here. The six: {}.",
                    RESIDENTS.iter().map(|r| r.name).collect::<Vec<_>>().join(", ")
                )));
            };
            let mut b = book(w);
            let now = w.now().0;
            let key = n.to_string();
            let mut done: Vec<u64> = b["right"][&ctx.actor].as_array().map(|a| a.iter().filter_map(Value::as_u64).collect()).unwrap_or_default();
            if done.contains(&(n as u64)) {
                return Err(CommandError::precondition(format!("You have that line already: it is {}'s.", all[n as usize - 1].1)));
            }
            if let Some(t) = b["wrong"][&ctx.actor][&key].as_i64() {
                if now - t < WRONG_MS {
                    return Err(CommandError::precondition(format!(
                        "You guessed at that line and missed; it is closed to you for {} more. The others are open.",
                        super::combat::human(WRONG_MS - (now - t))
                    )));
                }
            }
            let (line, who) = all[n as usize - 1];
            let right = who == guess.name;
            let mut all_six = false;
            let reply = if right {
                done.push(n as u64);
                all_six = done.len() == all.len();
                if !b["right"].is_object() {
                    b["right"] = json!({});
                }
                b["right"][&ctx.actor] = json!(done);
                let curves = super::Curves::from_env();
                super::memory::earn_chunk(w, &ctx.actor, f.window, &format!("knowing {}'s hand in the Visitors' Book", guess.name), LINE_XP, 0.9, &curves)?;
                let more = if all_six {
                    super::memory::earn_chunk(w, &ctx.actor, f.window, "knowing everyone who boards at the Kept House by how they write", BOOK_XP, 0.9, &curves)?;
                    format!(" That is all six: you know the house by its handwriting, and that is a memory too, {BOOK_XP:.0} XP, vivid. The page turns with the month.")
                } else {
                    format!(" {} of {} found.", done.len(), all.len())
                };
                format!("\"{line}\" Yes: that is {} the {}, and nobody else in the house would have put it that way. A memory, {LINE_XP:.0} XP, vivid.{more}", guess.name, guess.kind)
            } else {
                if !b["wrong"].is_object() {
                    b["wrong"] = json!({});
                }
                if !b["wrong"][&ctx.actor].is_object() {
                    b["wrong"][&ctx.actor] = json!({});
                }
                b["wrong"][&ctx.actor][&key] = json!(now);
                format!("\"{line}\" You say {}; the hand is not {}'s. That line is closed to you for an hour. Go and listen to them: each has one thing it cannot leave alone.", guess.name, guess.name)
            };
            keep_book(w, b).map_err(CommandError::internal)?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                ASCRIBED,
                json!({"by": ctx.actor, "line": n, "to": guess.name, "right": right, "all": all_six}),
            );
            let _ = find_citizen(w, &ctx.actor);
            Ok(Outcome::new(reply, json!({"line": n, "to": guess.name, "right": right, "all": all_six})))
        }),
    );
}
