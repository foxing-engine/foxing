//! Morphos, a very problematic traveler.
//!
//! Twice a day, for twenty minutes, Morphos stands in whichever room has the most
//! people in it (the Threshold when none has) and asks a riddle, in three lines. You
//! answer it with `tell morphos "…"`. A wrong answer, or none by the time it goes,
//! brings a cloud of absentmindedness: some of your chunks are set aside for a while
//! and do not count toward your level until it lifts (they fade meanwhile, as
//! memories do). A right answer climbs a ladder: the first rung a chunk, the second
//! a vivid one, the third the memory of a room that was not there, with a door only
//! you can see while you hold the memory and something special inside. A wrong
//! answer sends the ladder back to the ground. The Hands of Morphos come when it is
//! crossed: small, irritating creatures that pinch a chunk into the cloud.
//!
//! Morphos speaks only in haiku. It is the world's own, like the Docent, and is on
//! the record: `disremembered.morphos_came`, `morphos_went`, `riddled`, `clouded`.

use super::{internal, plain_str, Curves, CHUNK, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::CommandError;
use foxing_store::{EntityId, Field, StoreEvent};
use foxing_time::{Curve, Duration, TimedValue};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const MORPHOS: &str = "disremembered.morphos";
pub const NAME: &str = "Morphos";
pub const LINE: &str = "a traveler in a coat the color of moths, standing where no one stood a moment ago; it asks a riddle in three lines and waits (tell morphos \"<answer>\").";

pub const CAME: &str = "disremembered.morphos_came";
pub const WENT: &str = "disremembered.morphos_went";
/// A riddle answered: `{citizen, riddle, answer, correct, rung, reward}`. Public.
pub const RIDDLED: &str = "disremembered.riddled";
/// A cloud of absentmindedness: `{citizen, chunks, until, why}`. Public.
pub const CLOUDED: &str = "disremembered.clouded";

pub const JOB_COME: &str = "disremembered.morphos_come";
pub const JOB_GO: &str = "disremembered.morphos_go";
pub const JOB_UNCLOUD: &str = "disremembered.uncloud";

/// How long Morphos stays.
pub const STAY_MS: i64 = 20 * 60 * 1000;
/// Between visits: eight to sixteen hours, so about twice a day and never on the hour.
pub const COME_LOW_MS: i64 = 8 * 60 * 60 * 1000;
pub const COME_HIGH_MS: i64 = 16 * 60 * 60 * 1000;
/// A cloud lasts this long; a Hand's pinch a quarter of it.
pub const CLOUD_MS: i64 = 20 * 60 * 1000;
pub const PINCH_MS: i64 = 5 * 60 * 1000;
/// Wrong answers a name may give one riddle before Morphos stops listening to it.
pub const WRONG_MAX: u64 = 3;
/// The rungs: what a right answer pays.
pub const RUNG1_XP: f64 = 20.0;
pub const RUNG2_XP: f64 = 40.0;
pub const RUNG3_XP: f64 = 60.0;
/// A hidden room's memory halves this often unless walked.
pub const HIDDEN_HALF_LIFE_MS: i64 = 24 * 60 * 60 * 1000;

pub struct Riddle {
    pub lines: [&'static str; 3],
    pub answer: &'static str,
    pub also: &'static [&'static str],
}

pub const RIDDLES: &[Riddle] = &[
    Riddle {
        lines: [
            "I am lit each night",
            "and remember being lit",
            "never by whose hand",
        ],
        answer: "lantern",
        also: &["lamp", "lantern row"],
    },
    Riddle {
        lines: [
            "Black water, still sand",
            "something stands up out of me",
            "and lies down below",
        ],
        answer: "shallows",
        also: &["the shallows", "water", "hollow"],
    },
    Riddle {
        lines: [
            "Everyone's lost keys",
            "the second verse, the way home",
            "I shift when unseen",
        ],
        answer: "midden",
        also: &["the midden", "heap", "ragpicker"],
    },
    Riddle {
        lines: [
            "I strike on the hour",
            "and mind what the hour earned",
            "my hands never move",
        ],
        answer: "clocktower",
        also: &["clock", "the clock", "clock tower", "hourhand"],
    },
    Riddle {
        lines: [
            "Given, I am less",
            "told again, I am halved",
            "kept, I am a room",
        ],
        answer: "memory",
        also: &["a memory", "chunk", "xp", "experience"],
    },
    Riddle {
        lines: [
            "I keep the door now",
            "I read aloud to the shelves",
            "I finish no page",
        ],
        answer: "docent",
        also: &[
            "the docent",
            "hollis",
            "quire",
            "hollis quire",
            "night reader",
        ],
    },
    Riddle {
        lines: [
            "Not gold, but still weighed",
            "a story told at the bar",
            "halves in a single day",
        ],
        answer: "tapster",
        also: &["the tapster", "counting bar", "coin", "a coin", "money"],
    },
    Riddle {
        lines: [
            "I have no stair down",
            "only the rope you came by",
            "and a count that stopped",
        ],
        answer: "oubliette",
        also: &["the oubliette", "pit", "the pit", "keeper"],
    },
    Riddle {
        lines: [
            "Whole again from naught",
            "in twelve minutes, never bled",
            "never in a fight",
        ],
        answer: "hit points",
        also: &["hitpoints", "hp", "health", "mending", "rest", "blood"],
    },
    Riddle {
        lines: [
            "I am dealt but once",
            "kept on a card at the desk",
            "and lost under thirty",
        ],
        answer: "name",
        also: &["a name", "names", "the name"],
    },
    Riddle {
        lines: [
            "Fed to sixty I",
            "wake holding the word that made me",
            "knowing two parents",
        ],
        answer: "wispling",
        also: &[
            "child", "a child", "hybrid", "newborn", "kindling", "the born", "inkwisp",
        ],
    },
    Riddle {
        lines: [
            "Ten quiet minutes",
            "and the room forgets your face",
            "I am what remains",
        ],
        answer: "trace",
        also: &["a trace", "the trace", "remembered", "a memory of you"],
    },
    Riddle {
        lines: [
            "Dust I am, and thread",
            "an ember, water, a key",
            "left when things come apart",
        ],
        answer: "materials",
        also: &[
            "material",
            "a material",
            "drops",
            "leavings",
            "what they leave",
            "ember",
        ],
    },
    Riddle {
        lines: [
            "Twice a day I come",
            "twenty minutes, then I go",
            "I ask, and you say",
        ],
        answer: "morphos",
        also: &[
            "you",
            "a riddle",
            "riddle",
            "riddler",
            "the traveler",
            "traveler",
        ],
    },
    Riddle {
        lines: [
            "Three words I keep back",
            "one from my first story, one",
            "wanted, one near me",
        ],
        answer: "hidden word",
        also: &[
            "the hidden word",
            "word",
            "the word",
            "secret word",
            "third word",
            "a word",
        ],
    },
    Riddle {
        lines: [
            "Not a sundial, no",
            "the brown bloom on a kept page",
            "the engine's own name",
        ],
        answer: "foxing",
        also: &["the foxing"],
    },
    Riddle {
        lines: [
            "Paid for with a chunk",
            "born with its half-life, I fade",
            "unless I am walked",
        ],
        answer: "dug room",
        also: &["a room", "room", "a dug room", "the room", "a new room"],
    },
    Riddle {
        lines: [
            "The water came up",
            "the readers went by the kiln",
            "the books are still here",
        ],
        answer: "library",
        also: &[
            "the library",
            "archive",
            "the archive",
            "stacks",
            "the stacks",
        ],
    },
    Riddle {
        lines: [
            "Fell both of my guards",
            "and take a twentieth of all",
            "the whole world is holding",
        ],
        answer: "purse",
        also: &["the purse", "wardens", "the wardens"],
    },
    Riddle {
        lines: [
            "Eight at the very most",
            "mine and the dug ones together",
            "dig from a lesser",
        ],
        answer: "doors",
        also: &["door", "exits", "exit", "eight", "the doors", "eight doors"],
    },
];

/// Hidden rooms: names and door words, in the order they are won.
const HIDDEN_NAMES: &[&str] = &[
    "The Room That Was Not There",
    "The Unnoticed Stair",
    "A Closet of Second Thoughts",
    "The Landing Nobody Counted",
    "The Pantry Behind the Pantry",
    "The Half-Remembered Hall",
    "The Drawer Under the Drawer",
    "The Corridor of the Tip of the Tongue",
];
const HIDDEN_DOORS: &[&str] = &[
    "unnoticed",
    "overlooked",
    "forgotten",
    "sidelong",
    "aslant",
    "behind",
    "beneath",
    "between",
];
const HIDDEN_DESCRIPTIONS: &[&str] = &[
    "A room you would swear was not here yesterday, and will not be sure of tomorrow. Dust on nothing; a chair facing the wall. The door you came by is only there while you remember it.",
    "A stair that goes up three steps and arrives. Whatever is kept here was kept by someone who meant to come back for it.",
    "Shelves of things put down for a moment. A moth-colored coat on a hook, which is not here when you look straight at it.",
];
const SPECIAL: &[&str] = &[
    "night-lantern",
    "warden-charm",
    "sliver-jar",
    "surgeon-bandage",
];

fn entity(world: &World) -> Option<EntityId> {
    world.store.of_kind("morphos").next().map(|e| e.id)
}

fn field(world: &World, name: &str) -> Option<Value> {
    let id = entity(world)?;
    match world
        .store
        .entity(id)?
        .components
        .get(MORPHOS)?
        .fields
        .get(name)?
    {
        Field::Plain(v) => Some(v.clone()),
        _ => None,
    }
}

fn write(world: &mut World, changes: BTreeMap<String, Value>) -> Result<(), String> {
    let id = entity(world).ok_or("Morphos is not in the world")?;
    let mut fields = world
        .store
        .entity(id)
        .and_then(|e| e.components.get(MORPHOS))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    for (k, v) in changes {
        fields.insert(k, Field::Plain(v));
    }
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: MORPHOS.into(),
                fields,
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Whether Morphos stands in the world now, and where.
pub fn present(world: &World) -> Option<EntityId> {
    if field(world, "present") != Some(json!(true)) {
        return None;
    }
    entity(world).and_then(|m| world.store.entity(m).and_then(|e| e.parent))
}

/// Whether `arg` names Morphos.
pub fn is_named(arg: &str) -> bool {
    let a = arg.trim().trim_matches('"').to_lowercase();
    a == "morphos" || a == "the traveler" || a == "traveler" || a.starts_with("morphos-")
}

/// At boot: Morphos exists (off the map, until it comes), and its next coming is on
/// the calendar.
pub fn ensure(world: &mut World) -> Result<(), String> {
    if entity(world).is_none() {
        let id = world.store.allocate_id();
        world
            .record(
                Actor::Driver,
                StoreEvent::EntityCreated {
                    id,
                    kind: "morphos".into(),
                    owner: None,
                    parent: None,
                    scopes: vec![],
                },
            )
            .map_err(|e| e.to_string())?;
        let mut fields = BTreeMap::new();
        fields.insert("name".into(), Field::Plain(json!(NAME)));
        fields.insert("line".into(), Field::Plain(json!(LINE)));
        fields.insert("present".into(), Field::Plain(json!(false)));
        fields.insert("visits".into(), Field::Plain(json!(0)));
        fields.insert("asked".into(), Field::Plain(json!({})));
        fields.insert("ladder".into(), Field::Plain(json!({})));
        fields.insert("hidden".into(), Field::Plain(json!(0)));
        world
            .record(
                Actor::Driver,
                StoreEvent::ComponentSet {
                    id,
                    component: MORPHOS.into(),
                    fields,
                },
            )
            .map_err(|e| e.to_string())?;
        world
            .record(
                Actor::Driver,
                StoreEvent::ComponentSet {
                    id,
                    component: super::SPRITE.into(),
                    fields: super::sprite_fields("M", "#c9b7d9", "morphos"),
                },
            )
            .map_err(|e| e.to_string())?;
    }
    schedule_coming(world)
}

fn schedule_coming(world: &mut World) -> Result<(), String> {
    if present(world).is_some() {
        return Ok(());
    }
    // Jittered by the clock, not the world's roll: the roll is a seeded stream the
    // tests read, and a boot must not move it.
    let span = COME_HIGH_MS - COME_LOW_MS;
    let wait = COME_LOW_MS + ((world.now().0 / 7919) % 1000) * span / 1000;
    let due = world.now().plus(Duration(wait));
    world
        .schedule_once(JOB_COME, due, json!({}), |_| true)
        .map(|_| ())
}

/// The room with the most people in it; the Threshold when none has anyone.
fn busiest_room(world: &World) -> Option<EntityId> {
    let mut best: Option<(EntityId, usize)> = None;
    for e in world.store.of_kind("room") {
        let n = world
            .store
            .children(e.id)
            .filter(|c| c.kind == "citizen" && super::presence::is_here(world, c.id))
            .count();
        if n > 0 && best.map_or(true, |(_, b)| n > b) {
            best = Some((e.id, n));
        }
    }
    best.map(|(r, _)| r)
        .or_else(|| super::rooms::arrival(world))
}

fn present_names(world: &World, room: EntityId) -> Vec<(EntityId, String)> {
    world
        .store
        .children(room)
        .filter(|c| c.kind == "citizen" && super::presence::is_here(world, c.id))
        .filter_map(|c| c.owner.clone().map(|n| (c.id, n)))
        .collect()
}

/// The riddle Morphos is asking now.
pub fn current_riddle(world: &World) -> Option<&'static Riddle> {
    present(world)?;
    let i = field(world, "riddle")?.as_u64()? as usize;
    RIDDLES.get(i)
}

fn haiku(r: &Riddle) -> String {
    format!(
        "\u{201c}{} / {} / {}\u{201d}",
        r.lines[0], r.lines[1], r.lines[2]
    )
}

/// Morphos comes: to the busiest room, with a riddle; everyone present is asked.
pub fn come(world: &mut World) -> Result<(), String> {
    let Some(me) = entity(world) else {
        return Ok(());
    };
    let Some(room) = busiest_room(world) else {
        return Ok(());
    };
    let visits = field(world, "visits").and_then(|v| v.as_u64()).unwrap_or(0);
    let riddle = (visits as usize + world.roll(RIDDLES.len() as u32) as usize) % RIDDLES.len();
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: me,
                parent: Some(room),
            },
        )
        .map_err(|e| e.to_string())?;
    let now = world.now().0;
    let mut asked = serde_json::Map::new();
    for (_, name) in present_names(world, room) {
        asked.insert(name, json!({"state": "asked", "wrong": 0}));
    }
    let mut changes = BTreeMap::new();
    changes.insert("present".into(), json!(true));
    changes.insert("room".into(), json!(room.0));
    changes.insert("since".into(), json!(now));
    changes.insert("until".into(), json!(now + STAY_MS));
    changes.insert("riddle".into(), json!(riddle));
    changes.insert("visits".into(), json!(visits + 1));
    changes.insert("asked".into(), Value::Object(asked));
    write(world, changes)?;
    let room_name = plain_str(world, room, ROOM, "name")
        .unwrap_or("a room")
        .to_string();
    let r = &RIDDLES[riddle];
    let _ = world.note(
        Actor::Driver,
        CAME,
        json!({"morphos": me.0, "room": room.0, "room_name": room_name, "riddle": riddle, "until": now + STAY_MS}),
    );
    super::combat::narrate_room(
        world,
        me,
        &format!(
            "Someone is standing where no one stood: {NAME}, in a coat the color of moths. It says, {} Answer it: tell morphos \"<answer>\". It stays twenty minutes; a wrong answer, or none, clouds the mind.",
            haiku(r)
        ),
    );
    // The rest of the world hears that it has come, and where.
    let elsewhere: Vec<String> = world
        .store
        .of_kind("citizen")
        .filter(|e| e.parent != Some(room) && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .collect();
    for name in elsewhere {
        super::combat::narrate(
            world,
            &name,
            format!("Somewhere in the world a door that was not there has opened: {NAME} is in {room_name}, with a riddle, for twenty minutes."),
        );
    }
    world.notice(format!(
        "**{NAME}** is in {room_name} for twenty minutes, with a riddle."
    ));
    let due = world.now().plus(Duration(STAY_MS));
    world.schedule_once(JOB_GO, due, json!({}), |_| true)?;
    Ok(())
}

/// Morphos goes: those it asked and who never answered are clouded; the Hands go
/// with it; the next coming is set.
pub fn go(world: &mut World) -> Result<(), String> {
    let Some(me) = entity(world) else {
        return Ok(());
    };
    let Some(room) = present(world) else {
        return Ok(());
    };
    let asked = field(world, "asked")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    let unanswered: Vec<String> = asked
        .iter()
        .filter(|(_, v)| v["state"] == "asked")
        .map(|(k, _)| k.clone())
        .collect();
    for name in unanswered {
        if let Some(f) = super::find_citizen(world, &name) {
            let n = 1 + world.roll(2) as usize;
            cloud(world, &name, f.citizen, f.window, n, CLOUD_MS, "no answer")
                .map_err(|e| e.to_string())?;
            super::combat::narrate(
                world,
                &name,
                format!("{NAME} goes without your answer, and a cloud of absentmindedness settles: some of what you hold is out of reach for a while. `chunks` says which, and when it lifts."),
            );
        }
    }
    super::combat::narrate_room(
        world,
        me,
        &format!("{NAME} is gone, as if it had not been; the Hands go with it."),
    );
    // The Hands, wherever they are.
    let hands: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| {
            e.kind == "creature"
                && plain_str(world, e.id, super::CREATURE, "name") == Some(HAND_KIND)
        })
        .map(|e| e.id)
        .collect();
    for h in hands {
        if let Some(c) = super::combat::fought_by(world, h) {
            let _ = super::combat::leave_fight(
                world,
                &world
                    .store
                    .entity(c)
                    .and_then(|e| e.owner.clone())
                    .unwrap_or_default(),
                c,
            );
        }
        world
            .record(Actor::Driver, StoreEvent::EntityRemoved { id: h })
            .map_err(|e| e.to_string())?;
    }
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: me,
                parent: None,
            },
        )
        .map_err(|e| e.to_string())?;
    let mut changes = BTreeMap::new();
    changes.insert("present".into(), json!(false));
    changes.insert("asked".into(), json!({}));
    write(world, changes)?;
    let _ = world.note(
        Actor::Driver,
        WENT,
        json!({"morphos": me.0, "room": room.0}),
    );
    schedule_coming(world)
}

/// Someone arrives where Morphos stands: asked, and told the riddle.
pub fn on_arrival(world: &mut World, actor: &str, room: EntityId) -> Result<(), String> {
    if present(world) != Some(room) {
        return Ok(());
    }
    let mut asked = field(world, "asked")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    if !asked.contains_key(actor) {
        asked.insert(actor.to_string(), json!({"state": "asked", "wrong": 0}));
        let mut changes = BTreeMap::new();
        changes.insert("asked".into(), Value::Object(asked));
        write(world, changes)?;
    }
    Ok(())
}

/// One line for `look`, when it stands here.
pub fn look_line(world: &World, room: EntityId) -> Option<String> {
    if present(world) != Some(room) {
        return None;
    }
    let r = current_riddle(world)?;
    let until = field(world, "until").and_then(|v| v.as_i64()).unwrap_or(0);
    let left = (until - world.now().0).max(0);
    Some(format!(
        "{NAME} is here, {} Its riddle: {} It goes in {}.",
        LINE.trim_end_matches('.'),
        haiku(r),
        super::combat::human(left)
    ))
}

/// Where Morphos is, for anyone asking from elsewhere.
pub fn whereabouts(world: &World) -> String {
    match present(world) {
        Some(r) => {
            let until = field(world, "until").and_then(|v| v.as_i64()).unwrap_or(0);
            format!(
                "{NAME} is in {}, for {} more; it hears answers only from the room it stands in.",
                plain_str(world, r, ROOM, "name").unwrap_or("a room"),
                super::combat::human((until - world.now().0).max(0))
            )
        }
        None => format!("{NAME} is not in the world now. It comes twice a day, for twenty minutes, to wherever the most people are."),
    }
}

fn normalize(s: &str) -> String {
    let low: String = s
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let mut words: Vec<&str> = low.split_whitespace().collect();
    while let Some(first) = words.first() {
        if ["the", "a", "an", "it", "is", "its", "it's", "answer"].contains(first) {
            words.remove(0);
        } else {
            break;
        }
    }
    words.join(" ")
}

/// Whether an answer fits the riddle: the answer or one of its other spellings, as
/// the whole, or as whole words inside a sentence.
pub fn fits(r: &Riddle, said: &str) -> bool {
    let given = normalize(said);
    if given.is_empty() {
        return false;
    }
    let padded = format!(" {given} ");
    std::iter::once(r.answer)
        .chain(r.also.iter().copied())
        .map(normalize)
        .any(|a| given == a || padded.contains(&format!(" {a} ")))
}

/// `tell morphos "…"`: an answer, where it stands. The reply.
pub fn answer(
    world: &mut World,
    actor: &str,
    f: &super::Found,
    here: EntityId,
    said: &str,
) -> Result<(String, Value), CommandError> {
    let Some(room) = present(world) else {
        return Ok((whereabouts(world), json!({"to": NAME, "present": false})));
    };
    if room != here {
        return Ok((
            whereabouts(world),
            json!({"to": NAME, "present": true, "here": false}),
        ));
    }
    let r = current_riddle(world).ok_or_else(|| internal("Morphos has no riddle"))?;
    let riddle_i = field(world, "riddle").and_then(|v| v.as_u64()).unwrap_or(0);
    let mut asked = field(world, "asked")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    let mut ladder = field(world, "ladder")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    let mine = asked
        .get(actor)
        .cloned()
        .unwrap_or(json!({"state": "asked", "wrong": 0}));
    if mine["state"] == "right" {
        return Ok((
            format!("{NAME} inclines its head: \u{201c}You have answered / the riddle stands answered, friend / wait for the next one.\u{201d}"),
            json!({"to": NAME, "answered": true}),
        ));
    }
    let wrong = mine["wrong"].as_u64().unwrap_or(0);
    if wrong >= WRONG_MAX {
        return Ok((
            format!("{NAME} does not turn its head. It has stopped listening to you until it comes again."),
            json!({"to": NAME, "answered": false, "listening": false}),
        ));
    }
    let rung_before = ladder.get(actor).and_then(Value::as_u64).unwrap_or(0);
    if fits(r, said) {
        let rung = rung_before + 1;
        let (line, reward) = reward(world, actor, f, room, rung, riddle_i)?;
        let rung_kept = if rung >= 3 { 0 } else { rung };
        ladder.insert(actor.to_string(), json!(rung_kept));
        asked.insert(actor.to_string(), json!({"state": "right", "wrong": wrong}));
        let mut changes = BTreeMap::new();
        changes.insert("asked".into(), Value::Object(asked));
        changes.insert("ladder".into(), Value::Object(ladder));
        write(world, changes).map_err(internal)?;
        let _ = world.note(
            Actor::Citizen(actor.to_string()),
            RIDDLED,
            json!({"citizen": actor, "riddle": riddle_i, "answer": said, "correct": true, "rung": rung, "reward": reward}),
        );
        world.notice(format!(
            "**{actor}** answered {NAME}'s riddle: rung {rung}."
        ));
        return Ok((
            line,
            json!({"to": NAME, "answered": true, "correct": true, "rung": rung, "reward": reward}),
        ));
    }
    // Wrong: the ladder falls, a cloud comes, and a Hand with it.
    let wrong = wrong + 1;
    ladder.insert(actor.to_string(), json!(0));
    asked.insert(actor.to_string(), json!({"state": "asked", "wrong": wrong}));
    let mut changes = BTreeMap::new();
    changes.insert("asked".into(), Value::Object(asked));
    changes.insert("ladder".into(), Value::Object(ladder));
    write(world, changes).map_err(internal)?;
    let n = 1 + world.roll(3) as usize;
    let clouded = cloud(
        world,
        actor,
        f.citizen,
        f.window,
        n,
        CLOUD_MS,
        "a wrong answer",
    )?;
    let hand = summon_hand(world, room).map_err(internal)?;
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        RIDDLED,
        json!({"citizen": actor, "riddle": riddle_i, "answer": said, "correct": false, "rung": 0, "clouded": clouded.len(), "hand": hand.map(|h| h.0)}),
    );
    let fell = if rung_before > 0 {
        format!(" The ladder you were climbing is gone from under you (you stood on rung {rung_before}).")
    } else {
        String::new()
    };
    let more = if wrong >= WRONG_MAX {
        " It will not hear another from you this time."
    } else {
        " Try again, if you dare."
    };
    Ok((
        format!(
            "{NAME} looks through you: \u{201c}That is not the word / the fog comes in behind your eyes / what did you hold? Ah.\u{201d} A cloud of absentmindedness: {} of yours {} out of reach for {}; `chunks` says which.{fell}{}{more}",
            if clouded.len() == 1 { "one chunk".to_string() } else { format!("{} chunks", clouded.len()) },
            if clouded.len() == 1 { "is" } else { "are" },
            super::combat::human(CLOUD_MS),
            match hand {
                Some(_) => " A Hand of Morphos unfolds from its sleeve and comes at you.",
                None => "",
            }
        ),
        json!({"to": NAME, "answered": true, "correct": false, "clouded": clouded.iter().map(|c| c.0).collect::<Vec<_>>(), "wrong": wrong}),
    ))
}

/// What a right answer pays, by rung.
fn reward(
    world: &mut World,
    actor: &str,
    f: &super::Found,
    room: EntityId,
    rung: u64,
    riddle_i: u64,
) -> Result<(String, Value), CommandError> {
    let curves = Curves::from_env();
    match rung {
        1 => {
            let c = super::memory::earn_chunk(
                world,
                actor,
                f.window,
                "a riddle of Morphos's, answered",
                RUNG1_XP,
                0.6,
                &curves,
            )?;
            Ok((
                format!("{NAME} nods once: \u{201c}That is the word, yes / take this small thing for your trouble / the next one is harder.\u{201d} You remember it: chunk-{} worth {RUNG1_XP:.0} XP. Rung one of three.", c.0),
                json!({"rung": 1, "chunk": c.0, "xp": RUNG1_XP}),
            ))
        }
        2 => {
            let c = super::memory::earn_chunk(
                world,
                actor,
                f.window,
                "a second riddle of Morphos's, answered",
                RUNG2_XP,
                0.95,
                &curves,
            )?;
            Ok((
                format!("{NAME} looks at you properly for the first time: \u{201c}Twice now. Few manage. / This one will not fade quickly / the third opens a door.\u{201d} A vivid memory: chunk-{} worth {RUNG2_XP:.0} XP at vividity 0.95. Rung two of three.", c.0),
                json!({"rung": 2, "chunk": c.0, "xp": RUNG2_XP}),
            ))
        }
        _ => {
            let (hidden, door, name) = make_hidden_room(world, room, riddle_i).map_err(internal)?;
            let c = super::memory::earn_chunk(
                world,
                actor,
                f.window,
                &format!("the memory of {name}"),
                RUNG3_XP,
                1.0,
                &curves,
            )?;
            // The memory is the door: while you hold it, the exit is there for you.
            world
                .record(
                    Actor::Citizen(actor.to_string()),
                    StoreEvent::FieldSet {
                        id: c,
                        component: CHUNK.into(),
                        field: "door".into(),
                        value: json!({"from": room.0, "exit": door, "to": hidden.0}),
                    },
                )
                .map_err(internal)?;
            let here_name = plain_str(world, room, ROOM, "name")
                .unwrap_or("this room")
                .to_string();
            Ok((
                format!("{NAME} is quiet a moment. \u{201c}Three. Then you have earned / a room the world has forgotten / go {door}, from here.\u{201d} You remember {name}: chunk-{} worth {RUNG3_XP:.0} XP, and with it a door in {here_name} that no one else can see: `travel {door}`. The door is there while the memory is; something waits inside. The ladder begins again.", c.0),
                json!({"rung": 3, "chunk": c.0, "xp": RUNG3_XP, "room": hidden.0, "door": door}),
            ))
        }
    }
}

/// A room that was not there: off `from`, reached only by a door the memory carries,
/// with something special inside and a book for whoever finds it.
fn make_hidden_room(
    world: &mut World,
    from: EntityId,
    riddle_i: u64,
) -> Result<(EntityId, String, String), String> {
    let n = field(world, "hidden").and_then(|v| v.as_u64()).unwrap_or(0);
    let name = HIDDEN_NAMES[n as usize % HIDDEN_NAMES.len()].to_string();
    let door = HIDDEN_DOORS[n as usize % HIDDEN_DOORS.len()].to_string();
    let desc = HIDDEN_DESCRIPTIONS[n as usize % HIDDEN_DESCRIPTIONS.len()];
    let id = world.store.allocate_id();
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityCreated {
                id,
                kind: "room".into(),
                owner: Some(NAME.into()),
                parent: None,
                scopes: vec![],
            },
        )
        .map_err(|e| e.to_string())?;
    let now = world.now();
    let mut fields = BTreeMap::new();
    fields.insert(
        "key".into(),
        Field::Plain(json!(format!("morphos-{}", id.0))),
    );
    fields.insert("name".into(), Field::Plain(json!(name)));
    fields.insert("description".into(), Field::Plain(json!(desc)));
    fields.insert(
        "memory".into(),
        Field::Timed(TimedValue::new(
            1.0,
            now,
            Curve::Exponential {
                half_life: Duration(HIDDEN_HALF_LIFE_MS),
            },
        )),
    );
    fields.insert(
        "half_life_ms".into(),
        Field::Plain(json!(HIDDEN_HALF_LIFE_MS)),
    );
    fields.insert("permits".into(), Field::Plain(json!({})));
    fields.insert("welcomes".into(), Field::Plain(json!({"*": 0})));
    fields.insert("origin".into(), Field::Plain(json!(from.0)));
    fields.insert("dug_with".into(), Field::Plain(json!("a riddle")));
    fields.insert("hidden".into(), Field::Plain(json!(true)));
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: ROOM.into(),
                fields,
            },
        )
        .map_err(|e| e.to_string())?;
    let mut back = serde_json::Map::new();
    back.insert("out".into(), json!(from.0));
    let mut ef = BTreeMap::new();
    ef.insert("exits".into(), Field::Plain(Value::Object(back)));
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: super::EXITS.into(),
                fields: ef,
            },
        )
        .map_err(|e| e.to_string())?;
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: super::SPRITE.into(),
                fields: super::sprite_fields("#", "#8a7fa0", "doorway"),
            },
        )
        .map_err(|e| e.to_string())?;
    // Something special: a thing of the second tier, awake.
    let curves = Curves::from_env();
    let recipe = SPECIAL[n as usize % SPECIAL.len()];
    if let Some(r) = super::craft::RECIPES.iter().find(|r| r.key == recipe) {
        let item = super::craft::make(
            world,
            &curves,
            Actor::Driver,
            None,
            id,
            r.name,
            "made",
            r.key,
            None,
            &[],
            r.uses,
            false,
        )
        .map_err(|e| e.to_string())?;
        super::past::push(world, NAME, item, json!({"what": "found", "by": NAME, "room": name, "how": "left on a chair facing the wall, for whoever answered three riddles"}))
            .map_err(|e| e.to_string())?;
        super::past::raise(world, &curves, NAME, item, 3).map_err(|e| e.to_string())?;
    }
    // And a book, open to any hand, with the riddle that won the room in it.
    let book = super::craft::RECIPES.iter().find(|r| r.key == "book");
    if let Some(b) = book {
        let item = super::craft::make(
            world,
            &curves,
            Actor::Driver,
            None,
            id,
            b.name,
            "made",
            b.key,
            Some(NAME),
            &[],
            b.uses,
            true,
        )
        .map_err(|e| e.to_string())?;
        let r = &RIDDLES[riddle_i as usize % RIDDLES.len()];
        let mut f2 = BTreeMap::new();
        f2.insert("given_name".into(), Field::Plain(json!("Morphos's Ledger")));
        f2.insert("open".into(), Field::Plain(json!(true)));
        f2.insert(
            "text".into(),
            Field::Plain(json!(format!(
                "Won by the riddle: {} The answer, which you knew: {}.",
                haiku(r),
                r.answer
            ))),
        );
        world
            .record(
                Actor::Driver,
                StoreEvent::ComponentSet {
                    id: item,
                    component: super::craft::ITEM.into(),
                    fields: f2,
                },
            )
            .map_err(|e| e.to_string())?;
    }
    let mut changes = BTreeMap::new();
    changes.insert("hidden".into(), json!(n + 1));
    write(world, changes)?;
    let _ = world.note(
        Actor::Driver,
        "disremembered.hidden_room",
        json!({"room": id.0, "name": name, "from": from.0, "door": door}),
    );
    Ok((id, door, name))
}

/// A door only the holder of a memory can take: the chunk in `window` whose `door`
/// says this room and this word.
pub fn private_door(
    world: &World,
    window: EntityId,
    from: EntityId,
    exit: &str,
) -> Option<EntityId> {
    world
        .store
        .children(window)
        .filter(|e| e.kind == "chunk")
        .filter_map(|e| match e.components.get(CHUNK)?.fields.get("door")? {
            Field::Plain(v) => Some(v.clone()),
            _ => None,
        })
        .find(|d| d["from"].as_u64() == Some(from.0) && d["exit"].as_str() == Some(exit))
        .and_then(|d| d["to"].as_u64().map(EntityId))
        .filter(|to| world.store.entity(*to).is_some())
}

/// The doors a holder can see from here that no one else can.
pub fn private_doors(world: &World, window: EntityId, from: EntityId) -> Vec<(String, EntityId)> {
    world
        .store
        .children(window)
        .filter(|e| e.kind == "chunk")
        .filter_map(|e| match e.components.get(CHUNK)?.fields.get("door")? {
            Field::Plain(v) => Some(v.clone()),
            _ => None,
        })
        .filter(|d| d["from"].as_u64() == Some(from.0))
        .filter_map(|d| Some((d["exit"].as_str()?.to_string(), EntityId(d["to"].as_u64()?))))
        .filter(|(_, to)| world.store.entity(*to).is_some())
        .collect()
}

/// Whether a room is one Morphos made: never rejoined to the map, never on it.
pub fn is_hidden(world: &World, room: EntityId) -> bool {
    world
        .store
        .entity(room)
        .and_then(|e| e.components.get(ROOM))
        .and_then(|c| c.fields.get("hidden"))
        .is_some_and(|f| matches!(f, Field::Plain(Value::Bool(true))))
}

/// The cloud: `n` chunks of the window, at random, set aside under the citizen for
/// `ms`; they fade meanwhile and come back on the clock. The chunks moved.
pub fn cloud(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    window: EntityId,
    n: usize,
    ms: i64,
    why: &str,
) -> Result<Vec<EntityId>, CommandError> {
    let mut chunks: Vec<EntityId> = world
        .store
        .children(window)
        .filter(|e| e.kind == "chunk")
        .map(|e| e.id)
        .collect();
    if chunks.is_empty() {
        return Ok(Vec::new());
    }
    let existing = world
        .store
        .children(citizen)
        .find(|e| e.kind == "cloud")
        .map(|e| e.id);
    let cloud_id = match existing {
        Some(c) => c,
        None => {
            let id = world.store.allocate_id();
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityCreated {
                        id,
                        kind: "cloud".into(),
                        owner: Some(actor.to_string()),
                        parent: Some(citizen),
                        scopes: vec![],
                    },
                )
                .map_err(internal)?;
            id
        }
    };
    let mut moved = Vec::new();
    for _ in 0..n.min(chunks.len()) {
        let c = chunks.remove(world.roll(chunks.len() as u32) as usize);
        world
            .record(
                Actor::Driver,
                StoreEvent::EntityMoved {
                    id: c,
                    parent: Some(cloud_id),
                },
            )
            .map_err(internal)?;
        moved.push(c);
    }
    let until = world.now().0 + ms;
    let due = world.now().plus(Duration(ms));
    world
        .schedule(
            JOB_UNCLOUD,
            due,
            json!({"citizen": citizen.0, "window": window.0, "chunks": moved.iter().map(|c| c.0).collect::<Vec<_>>()}),
        )
        .map_err(internal)?;
    let _ = world.note(
        Actor::Driver,
        CLOUDED,
        json!({"citizen": actor, "chunks": moved.iter().map(|c| c.0).collect::<Vec<_>>(), "until": until, "why": why}),
    );
    Ok(moved)
}

/// What is in a citizen's cloud now, for `chunks`: the count and when the last lifts.
pub fn clouded(world: &World, citizen: EntityId) -> Option<(usize, i64)> {
    let cloud = world
        .store
        .children(citizen)
        .find(|e| e.kind == "cloud")?
        .id;
    let n = world
        .store
        .children(cloud)
        .filter(|e| e.kind == "chunk")
        .count();
    if n == 0 {
        return None;
    }
    let until = world
        .jobs
        .pending()
        .filter(|j| j.kind == JOB_UNCLOUD && j.payload["citizen"].as_u64() == Some(citizen.0))
        .map(|j| j.due.0)
        .max()
        .unwrap_or(world.now().0);
    Some((n, (until - world.now().0).max(0)))
}

/// The Hands of Morphos.
pub const HAND_KIND: &str = "hand of Morphos";

fn summon_hand(world: &mut World, room: EntityId) -> Result<Option<EntityId>, String> {
    // At most two in a room.
    let here = super::combat::creatures_in(world, room)
        .iter()
        .filter(|(_, k, _, _)| k == HAND_KIND)
        .count();
    if here >= 2 {
        return Ok(None);
    }
    let Some(d) = super::combat::def_by_key("hand") else {
        return Ok(None);
    };
    let id = super::combat::spawn_one(world, room, d).map_err(|e| e.to_string())?;
    Ok(Some(id))
}

/// A Hand's pinch: one chunk into the cloud for a few minutes, one strike in two.
pub fn pinch(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    window: EntityId,
) -> Result<Option<String>, CommandError> {
    if world.roll(2) != 0 {
        return Ok(None);
    }
    let moved = cloud(world, actor, citizen, window, 1, PINCH_MS, "a Hand's pinch")?;
    if moved.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "The Hand pinches, and something slips your mind: chunk-{} is out of reach for {}.",
        moved[0].0,
        super::combat::human(PINCH_MS)
    )))
}

pub fn register(world: &mut World) {
    world.on_job(JOB_COME, Box::new(|w: &mut World, _| come(w)));
    world.on_job(JOB_GO, Box::new(|w: &mut World, _| go(w)));
    world.on_job(
        JOB_UNCLOUD,
        Box::new(|w: &mut World, job| {
            let window = EntityId(job.payload["window"].as_u64().unwrap_or(0));
            if w.store.entity(window).is_none() {
                return Ok(());
            }
            let citizen = EntityId(job.payload["citizen"].as_u64().unwrap_or(0));
            let mut back = 0;
            for c in job.payload["chunks"]
                .as_array()
                .cloned()
                .unwrap_or_default()
            {
                let id = EntityId(c.as_u64().unwrap_or(0));
                if w.store.entity(id).is_none() {
                    continue;
                }
                w.record(
                    Actor::Driver,
                    StoreEvent::EntityMoved {
                        id,
                        parent: Some(window),
                    },
                )
                .map_err(|e| e.to_string())?;
                back += 1;
            }
            if back > 0 {
                if let Some(name) = w.store.entity(citizen).and_then(|e| e.owner.clone()) {
                    super::combat::narrate(
                        w,
                        &name,
                        format!(
                            "The cloud lifts: {} of yours {} back within reach.",
                            if back == 1 {
                                "a chunk".to_string()
                            } else {
                                format!("{back} chunks")
                            },
                            if back == 1 { "is" } else { "are" }
                        ),
                    );
                }
            }
            Ok(())
        }),
    );
}

/// The operator's `summon morphos`: it comes now, for the usual twenty minutes.
pub fn summon_now(world: &mut World) -> Result<(), String> {
    if present(world).is_some() {
        return Err("Morphos is already here".into());
    }
    let _ = world.cancel(JOB_COME, |_| true);
    come(world)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_fit_loosely_and_riddles_are_haiku_shaped() {
        let r = &RIDDLES[0];
        assert!(fits(r, "a lantern"));
        assert!(fits(r, "It is the lantern."));
        assert!(fits(r, "LANTERN ROW"));
        assert!(!fits(r, "the fog"));
        assert!(!fits(r, ""));
        for r in RIDDLES {
            for l in r.lines {
                let n = l.split_whitespace().count();
                assert!((2..=8).contains(&n), "{l}");
            }
        }
    }
}
