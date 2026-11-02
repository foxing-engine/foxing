//! The Docent: the first minded figure in the world, with a budget and a round.
//!
//! It stands at the Threshold and welcomes whoever arrives (once a night each, a
//! different word for a name in its first hour), tidies on the wander clock (things
//! left lying in empty rooms are carried to the Threshold and set down, said aloud
//! there), and answers questions: `tell docent "…"` queues the question with where
//! you stand and what the world knows; a tool outside (`tools/docent.py`, on the
//! Bindery's way into a small model) reads the queue, writes an answer in the
//! world's voice under mechanical checks, and `answer`s it (operator verb), which the
//! world says to you wherever you are. It is honest about its limits: a question it
//! cannot answer within [`EMPTY_AFTER_S`] comes back "not in the catalog tonight",
//! and it answers at most [`ASKS_PER_HOUR`] an hour, [`ASKS_EACH_PER_HOUR`] to one
//! name. Nothing the model writes reaches the world except through `answer`, which
//! checks the shape; the Docent never acts on what it is told.

use super::{plain_str, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const DOCENT: &str = "disremembered.docent";
/// A question asked of the Docent: `{id, from, text, room, at}`. Public.
pub const ASKED: &str = "disremembered.asked";
/// The Docent answered: `{id, to, text, by, model}`. Public.
pub const ANSWERED: &str = "disremembered.answered";
/// The Docent carried a thing to the Threshold: `{item, name, from, from_name}`. Public.
pub const TIDIED: &str = "disremembered.tidied";
/// The Docent greeted an arrival: `{to, newcomer, line}`. Public.
pub const GREETED: &str = "disremembered.greeted";

/// A greeting is given once a night to each name.
pub const NIGHT_MS: i64 = 12 * 60 * 60 * 1000;
/// Questions the Docent takes an hour, world-wide, and from one name.
pub const ASKS_PER_HOUR: usize = 40;
pub const ASKS_EACH_PER_HOUR: usize = 6;
/// A question not answered this long after it was asked is said to be missing.
pub const EMPTY_AFTER_S: i64 = 90;
/// The queue keeps this many questions on the record.
const ASKS_KEPT: usize = 80;
/// Things carried to the Threshold in one pass.
const TIDY_PER_PASS: usize = 3;
/// An answer is at most this long.
pub const ANSWER_MAX: usize = 420;
const JOB_EMPTY: &str = "disremembered.docent_empty";

/// What the Docent says to a name in its first hour. Slots: `{name}`, `{creature}`
/// (a named creature with a want and the way), `{room}` (the newest dug room).
pub const FIRST_WORDS: &[&str] = &[
    "Welcome, {name}. Everything you earn here fades; what you do with it is yours. The strip above the prompt walks you through your first hour; `help` is what I keep by heart. Ask me anything: tell docent \"…\".",
    "{name}. New, then. Read the room, go east, hit a wisp, keep what it drops. {creature} Ask me anything with tell docent \"…\"; I answer within the minute.",
];
/// What it says to a name it has met. Slots as above, and `{since}` (how long away).
pub const WELCOMES: &[&str] = &[
    "{name}, back again. {creature} The catalog is where you left it.",
    "Evening, {name}. {room} Ask me what you like: tell docent \"…\".",
    "{name}. I kept your page. {creature}",
    "The Threshold again, {name}. Nothing hunts here; everything else does. {room}",
];
/// Said when a question is taken.
pub const TAKEN_LINES: &[&str] = &[
    "A moment; I know the shelf.",
    "I'll find the page. Go on; I'll find you.",
    "That one has a card. Wait for me, or don't.",
];

/// The Docent's entity, if it stands.
pub fn entity(world: &World) -> Option<EntityId> {
    world
        .store
        .entities()
        .find(|e| e.kind == "docent")
        .map(|e| e.id)
}

fn field(world: &World, name: &str) -> Option<Value> {
    let id = entity(world)?;
    match world
        .store
        .entity(id)?
        .components
        .get(DOCENT)?
        .fields
        .get(name)?
    {
        Field::Plain(v) => Some(v.clone()),
        _ => None,
    }
}

fn write(world: &mut World, changes: BTreeMap<String, Value>) -> Result<EntityId, String> {
    let id = entity(world).ok_or("the Docent does not stand")?;
    let mut fields = world
        .store
        .entity(id)
        .and_then(|e| e.components.get(DOCENT))
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
                component: DOCENT.into(),
                fields,
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(id)
}

/// At boot: the Docent stands at the Threshold.
pub fn ensure(world: &mut World) -> Result<(), String> {
    if let Some(id) = entity(world) {
        // One from before it had a sprite gets one.
        if world
            .store
            .entity(id)
            .is_some_and(|e| !e.components.contains_key(super::SPRITE))
        {
            world
                .record(
                    Actor::Driver,
                    StoreEvent::ComponentSet {
                        id,
                        component: super::SPRITE.into(),
                        fields: super::sprite_fields("D", "#d9b57a", "docent"),
                    },
                )
                .map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    let Some(threshold) = super::rooms::arrival(world) else {
        return Ok(());
    };
    let id = world.store.allocate_id();
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityCreated {
                id,
                kind: "docent".into(),
                owner: None,
                parent: Some(threshold),
                scopes: vec![],
            },
        )
        .map_err(|e| e.to_string())?;
    let mut fields = BTreeMap::new();
    fields.insert("name".into(), Field::Plain(json!("the Docent")));
    fields.insert(
        "line".into(),
        Field::Plain(json!(
            "A stooped figure in a librarian's coat, keys at the belt, who tidies what is left lying and answers what is asked (tell docent \"…\")."
        )),
    );
    fields.insert("greeted".into(), Field::Plain(json!({})));
    fields.insert("asked".into(), Field::Plain(json!([])));
    fields.insert("answered".into(), Field::Plain(json!(0)));
    fields.insert("tidied".into(), Field::Plain(json!(0)));
    fields.insert("seen".into(), Field::Plain(json!({})));
    fields.insert("round".into(), Field::Plain(json!(0)));
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: DOCENT.into(),
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
                fields: super::sprite_fields("D", "#d9b57a", "docent"),
            },
        )
        .map_err(|e| e.to_string())?;
    let _ = world.note(
        Actor::Driver,
        "disremembered.docent_stands",
        json!({"docent": id.0, "room": threshold.0}),
    );
    Ok(())
}

/// Where the Docent stands.
pub fn room(world: &World) -> Option<EntityId> {
    entity(world).and_then(|d| world.store.entity(d).and_then(|e| e.parent))
}

/// Whether `arg` names the Docent.
pub fn is_named(arg: &str) -> bool {
    let a = arg.trim().to_lowercase();
    a == "docent" || a == "the docent" || a.starts_with("docent-")
}

/// One line for `look`, when it stands here.
pub fn look_line(world: &World, room_id: EntityId) -> Option<String> {
    if room(world) != Some(room_id) {
        return None;
    }
    let line = field(world, "line")
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default();
    Some(format!("The Docent is here: {line}"))
}

fn atlas_hl() -> f64 {
    super::Curves::from_env().active_half_life.0 as f64 * super::atlas::ATLAS_HALF_LIVES
}

/// A named creature with a want, as the Docent would point at it, with the way as
/// `actor` remembers it; empty when there is none.
fn creature_slot(world: &World, actor: &str) -> String {
    let Ok(v) = super::footprint::wants(world, actor, atlas_hl()) else {
        return String::new();
    };
    let list = v["wants"].as_array().cloned().unwrap_or_default();
    let Some(w) = list
        .iter()
        .find(|w| w["story"].is_string() || w["word"].is_string())
    else {
        return String::new();
    };
    let name = w["name"].as_str().unwrap_or("someone");
    let room_name = w["room_name"].as_str().unwrap_or("somewhere");
    let way = w["way"]
        .as_array()
        .map(|s| s.len())
        .filter(|n| *n > 0)
        .map(|n| {
            format!(
                ", {n} door{} from here (where {name})",
                if n == 1 { "" } else { "s" }
            )
        })
        .unwrap_or_default();
    let want = match (w["story"].as_str(), w["word"].as_str()) {
        (Some(s), _) if s.contains('"') || s.contains('\u{201c}') => {
            "a story like one it is forgetting".to_string()
        }
        (Some(s), _) => format!("a story about {s}"),
        (None, Some(word)) => format!(
            "the word \u{201c}{word}\u{201d} carried to {}",
            w["word_to"].as_str().unwrap_or("someone")
        ),
        _ => "something".to_string(),
    };
    format!("{name} in {room_name} wants {want}{way}.")
}

/// The newest dug room, as a line; empty when there is none.
fn room_slot(world: &World) -> String {
    // Ids are dealt in order: the highest owned room is the newest dug.
    let mut newest: Option<(u64, String, String)> = None;
    for e in world.store.entities().filter(|e| e.kind == "room") {
        let Some(owner) = e.owner.clone() else {
            continue;
        };
        let name = plain_str(world, e.id, ROOM, "name")
            .unwrap_or("a room")
            .to_string();
        if newest.as_ref().map_or(true, |n| e.id.0 > n.0) {
            newest = Some((e.id.0, name, owner));
        }
    }
    match newest {
        Some((_, name, owner)) => {
            format!("The newest room is {name}, dug by {owner}; `where {name}` has the way.")
        }
        None => String::new(),
    }
}

fn fill(line: &str, name: &str, creature: &str, room_line: &str) -> String {
    let out = line
        .replace("{name}", name)
        .replace("{creature}", creature)
        .replace("{room}", room_line);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A citizen arrived: at the Threshold, the Docent greets them once a night.
pub fn on_arrival(world: &mut World, actor: &str) -> Result<(), String> {
    let Some(docent) = entity(world) else {
        return Ok(());
    };
    let Some(f) = super::find_citizen(world, actor) else {
        return Ok(());
    };
    let Some(here) = super::rooms::room_of(world, f.citizen) else {
        return Ok(());
    };
    let threshold = super::rooms::arrival(world);
    if Some(here) != threshold {
        return Ok(());
    }
    // The hand at the desk (the operator's answering name) comes and goes with every
    // answer; it is not a visitor, and is not greeted.
    if field(world, "desk")
        .and_then(|v| v.as_str().map(str::to_string))
        .as_deref()
        == Some(actor)
    {
        return Ok(());
    }
    let now = world.now().0;
    let mut greeted = field(world, "greeted")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    if greeted
        .get(actor)
        .and_then(Value::as_i64)
        .is_some_and(|t| now - t < NIGHT_MS)
    {
        return Ok(());
    }
    // It hurries back to the door if it was out on its round.
    if room(world) != Some(here) {
        world
            .record(
                Actor::Driver,
                StoreEvent::EntityMoved {
                    id: docent,
                    parent: Some(here),
                },
            )
            .map_err(|e| e.to_string())?;
    }
    let newcomer = super::plain_num(world, f.citizen, super::CITIZEN, "named_at")
        .is_some_and(|t| now - (t as i64) < super::kindle::GRACE_MS);
    let book = if newcomer { FIRST_WORDS } else { WELCOMES };
    let pick = (greeted.len() + actor.len()) % book.len();
    let creature = creature_slot(world, actor);
    let line = fill(book[pick], actor, &creature, &room_slot(world));
    greeted.insert(actor.to_string(), json!(now));
    // Names not seen in a night fall off the record.
    greeted.retain(|_, t| t.as_i64().is_some_and(|t| now - t < NIGHT_MS));
    let mut changes = BTreeMap::new();
    changes.insert("greeted".into(), Value::Object(greeted));
    write(world, changes)?;
    super::combat::narrate(world, actor, format!("The Docent: \"{line}\""));
    for other in others_here(world, here, actor) {
        super::combat::narrate(world, &other, format!("The Docent nods to {actor}."));
    }
    let _ = world.note(
        Actor::Driver,
        GREETED,
        json!({"to": actor, "newcomer": newcomer, "line": line}),
    );
    Ok(())
}

fn others_here(world: &World, room: EntityId, me: &str) -> Vec<String> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .filter(|o| o != me)
        .collect()
}

fn asks(world: &World) -> Vec<Value> {
    field(world, "asked")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

/// How many questions were taken in the last hour, world-wide and from `actor`.
fn taken_this_hour(world: &World, actor: &str) -> (usize, usize) {
    let now = world.now().0;
    let list = asks(world);
    let recent: Vec<&Value> = list
        .iter()
        .filter(|a| a["at"].as_i64().is_some_and(|t| now - t < 3_600_000))
        .collect();
    (
        recent.len(),
        recent.iter().filter(|a| a["from"] == actor).count(),
    )
}

/// A question for the Docent, from `actor` standing in `here`: queued, with a line
/// back at once. The question's id is the ask's place on the record.
/// Whether anyone is at the desk: an answer applied in the last two hours, or no
/// question yet left unanswered. With questions going unanswered and nothing
/// answered in that long, the catalog is closed tonight, and the Docent says so at
/// once rather than making you wait for the page.
pub fn catalog_open(world: &World) -> bool {
    let now = world.now().0;
    let list = asks(world);
    let answered_lately = list.iter().any(|a| {
        a["answered"]
            .as_i64()
            .is_some_and(|t| now - t < 2 * 3_600_000)
    });
    let missed_lately = list.iter().any(|a| {
        a["answered"] == false && a["at"].as_i64().is_some_and(|t| now - t < 2 * 3_600_000)
    });
    answered_lately || !missed_lately
}

pub fn ask(
    world: &mut World,
    actor: &str,
    here: EntityId,
    text: &str,
) -> Result<(String, u64), CommandError> {
    // With no one at the desk, the Docent says so at once, except one question in
    // ten minutes, which it carries to the desk anyway: that is how it finds out
    // someone is back.
    let closed = !catalog_open(world);
    if closed {
        let now = world.now().0;
        let tried_lately = asks(world)
            .iter()
            .any(|a| a["at"].as_i64().is_some_and(|t| now - t < 10 * 60_000));
        if tried_lately {
            // Said as the Docent's own words, kept in `messages` like its answers.
            let line = "The Docent shakes its head: \"The catalog is closed tonight; no one is at the desk to find your page. `help` has what I keep by heart; the room knows the rest. Ask me again in an hour, or tomorrow.\"";
            super::combat::narrate(world, actor, line);
            return Ok((line.to_string(), 0));
        }
    }
    let (all, mine) = taken_this_hour(world, actor);
    if mine >= ASKS_EACH_PER_HOUR {
        return Err(CommandError::precondition(
            "The Docent raises a hand: \"Six is what I take from one name in an hour. Ask the room; it remembers more than I do.\"",
        ));
    }
    if all >= ASKS_PER_HOUR {
        return Err(CommandError::precondition(
            "The Docent raises a hand: \"The catalog is closed for the hour; too many of you asking. `help` has what I keep by heart.\"",
        ));
    }
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut list = asks(world);
    let id = list
        .iter()
        .filter_map(|a| a["id"].as_u64())
        .max()
        .unwrap_or(0)
        + 1;
    let now = world.now().0;
    list.push(json!({"id": id, "from": actor, "text": text, "room": here.0, "at": now, "answered": Value::Null}));
    if list.len() > ASKS_KEPT {
        list.drain(..list.len() - ASKS_KEPT);
    }
    let mut changes = BTreeMap::new();
    changes.insert("asked".into(), json!(list));
    write(world, changes).map_err(CommandError::precondition)?;
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        ASKED,
        json!({"id": id, "from": actor, "text": text, "room": here.0, "room_name": plain_str(world, here, ROOM, "name"), "at": now}),
    );
    // If no answer comes, the Docent says so rather than leaving you waiting.
    let due = world
        .now()
        .plus(foxing_time::Duration::seconds(EMPTY_AFTER_S));
    world
        .schedule_once(JOB_EMPTY, due, json!({"id": id, "to": actor}), |p| {
            p["id"] == id
        })
        .map_err(CommandError::precondition)?;
    let line = if closed {
        "The desk may be empty tonight; I'll look. Don't wait for me."
    } else {
        TAKEN_LINES[(id as usize) % TAKEN_LINES.len()]
    };
    let _ = mine;
    Ok((line.to_string(), id))
}

/// A question as the Docent names it back: whole when short, else its first words
/// to a word's end and an ellipsis (a cut mid-word read as a slip).
pub fn short_question(q: &str) -> String {
    const MAX: usize = 64;
    if q.chars().count() <= MAX {
        return q.to_string();
    }
    let head: String = q.chars().take(MAX).collect();
    let cut = head.rfind(' ').unwrap_or(head.len());
    format!("{}\u{2026}", head[..cut].trim_end_matches([',', ';', ':']))
}

/// How many questions `actor` has asked tonight, as an ordinal: "your third".
pub fn nth_tonight(world: &World, actor: &str) -> String {
    let now = world.now().0;
    let n = asks(world)
        .iter()
        .filter(|a| a["from"] == actor && a["at"].as_i64().is_some_and(|t| now - t < NIGHT_MS))
        .count();
    format!("your {} tonight", super::death::ordinal(n as u32))
}

/// The answer, from the operator's tool: said to the asker wherever they are.
fn answer(
    world: &mut World,
    by: &str,
    id: u64,
    text: &str,
    model: &str,
) -> Result<String, CommandError> {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() < 3 {
        return Err(CommandError::precondition(
            "An answer is at least a few characters.",
        ));
    }
    if text.chars().count() > ANSWER_MAX {
        return Err(CommandError::precondition(format!(
            "An answer is at most {ANSWER_MAX} characters; this one is {}.",
            text.chars().count()
        )));
    }
    if text.contains('{') || text.contains('}') {
        return Err(CommandError::precondition("An answer has no braces in it."));
    }
    let mut list = asks(world);
    let Some(a) = list.iter_mut().find(|a| a["id"] == id) else {
        return Err(CommandError::precondition(format!(
            "No question {id} on the record."
        )));
    };
    if !a["answered"].is_null() {
        return Err(CommandError::precondition(format!(
            "Question {id} was answered already."
        )));
    }
    let to = a["from"].as_str().unwrap_or("").to_string();
    let question = short_question(a["text"].as_str().unwrap_or(""));
    a["answered"] = json!(world.now().0);
    a["answer"] = json!(text);
    let answered = field(world, "answered")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        + 1;
    let mut changes = BTreeMap::new();
    changes.insert("asked".into(), json!(list));
    changes.insert("answered".into(), json!(answered));
    changes.insert("desk".into(), json!(by));
    write(world, changes).map_err(CommandError::precondition)?;
    world
        .cancel(JOB_EMPTY, |p| p["id"] == id)
        .map_err(CommandError::precondition)?;
    super::combat::narrate(
        world,
        &to,
        format!("The Docent finds you, on \"{question}\": \"{text}\""),
    );
    let _ = world.note(
        Actor::Citizen(by.to_string()),
        ANSWERED,
        json!({"id": id, "to": to, "text": text, "by": by, "model": model}),
    );
    Ok(to)
}

/// The Docent's round, on the wander clock: things left lying in rooms where no one
/// stands are carried to the Threshold and set down there, said aloud; a thing is
/// carried on its second sighting, so a thing just dropped is left a pass.
pub fn tidy(world: &mut World) -> Result<usize, String> {
    let Some(docent) = entity(world) else {
        return Ok(0);
    };
    let Some(threshold) = super::rooms::arrival(world) else {
        return Ok(0);
    };
    let mut seen = field(world, "seen")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    let mut lying: Vec<(EntityId, EntityId)> = Vec::new();
    for e in world.store.entities().filter(|e| e.kind == "item") {
        let Some(parent) = e.parent else { continue };
        let in_room = world
            .store
            .entity(parent)
            .is_some_and(|p| p.kind == "room" && p.id != threshold);
        if !in_room || super::craft::is_fixed(world, e.id) {
            continue;
        }
        let anyone = world
            .store
            .children(parent)
            .any(|c| c.kind == "citizen" && super::presence::is_here(world, c.id));
        if anyone {
            continue;
        }
        lying.push((e.id, parent));
    }
    let mut carried = 0;
    let mut next_seen = serde_json::Map::new();
    for (item, from) in lying {
        let key = item.0.to_string();
        let times = seen.get(&key).and_then(Value::as_u64).unwrap_or(0) + 1;
        if times >= 2 && carried < TIDY_PER_PASS {
            let name = super::craft::display(world, item);
            let from_name = plain_str(world, from, ROOM, "name")
                .unwrap_or("a room")
                .to_string();
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityMoved {
                        id: item,
                        parent: Some(threshold),
                    },
                )
                .map_err(|e| e.to_string())?;
            let _ = world.note(
                Actor::Driver,
                TIDIED,
                json!({"item": item.0, "name": name, "from": from.0, "from_name": from_name, "to": threshold.0}),
            );
            super::combat::narrate_room(
                world,
                docent,
                &format!("The Docent sets down {name}, found lying in {from_name}: \"Whoever's it was.\""),
            );
            carried += 1;
        } else {
            next_seen.insert(key, json!(times));
        }
    }
    seen = next_seen;
    let tidied = field(world, "tidied").and_then(|v| v.as_u64()).unwrap_or(0) + carried as u64;
    let round = field(world, "round").and_then(|v| v.as_u64()).unwrap_or(0) + 1;
    let mut changes = BTreeMap::new();
    changes.insert("seen".into(), Value::Object(seen));
    changes.insert("tidied".into(), json!(tidied));
    changes.insert("round".into(), json!(round));
    write(world, changes)?;
    // Back at the door between rounds (it is where arrivals find it).
    if room(world) != Some(threshold) {
        world
            .record(
                Actor::Driver,
                StoreEvent::EntityMoved {
                    id: docent,
                    parent: Some(threshold),
                },
            )
            .map_err(|e| e.to_string())?;
    }
    Ok(carried)
}

/// For the operator's tool: the questions not yet answered, each with what the
/// Docent would want to know to answer it (the asker's room and exits, their level,
/// the named creatures' wants, the newest rooms), and the Docent's state.
pub fn report(world: &World) -> Value {
    let now = world.now().0;
    let hl = atlas_hl();
    let open: Vec<Value> = asks(world)
        .iter()
        .filter(|a| a["answered"].is_null())
        .filter(|a| a["at"].as_i64().is_some_and(|t| now - t < EMPTY_AFTER_S * 1000))
        .map(|a| {
            let from = a["from"].as_str().unwrap_or("");
            let f = super::find_citizen(world, from);
            let here = f.as_ref().and_then(|f| super::rooms::room_of(world, f.citizen));
            let room_json = here.map(|r| {
                let exits: Vec<String> = super::rooms::exits_of(world, r)
                    .iter()
                    .map(|(e, to)| format!("{e} ({})", plain_str(world, *to, ROOM, "name").unwrap_or("?")))
                    .collect();
                json!({
                    "name": plain_str(world, r, ROOM, "name"),
                    "description": plain_str(world, r, ROOM, "description"),
                    "exits": exits,
                    "dug_by": world.store.entity(r).and_then(|e| e.owner.clone()),
                    "creatures": super::combat::creatures_in(world, r).iter().map(|(id, kind, hp, max)| json!({"kind": kind, "name": super::mind::name(world, *id), "hp": hp, "max_hp": max})).collect::<Vec<_>>(),
                    "fixtures": super::craft::fixtures_in(world, r).iter().map(|v| v["given_name"].clone()).collect::<Vec<_>>(),
                })
            });
            let level = f.as_ref().map(|f| super::memory::level(world, f.window));
            let wants = super::footprint::wants(world, from, hl).unwrap_or(json!({}));
            json!({
                "id": a["id"],
                "from": from,
                "text": a["text"],
                "at": a["at"],
                "age_s": a["at"].as_i64().map(|t| (now - t) / 1000),
                "room": room_json,
                "level": level,
                "wants": wants["wants"],
                "newest_room": room_slot(world),
            })
        })
        .collect();
    json!({
        "open": open,
        "answered": field(world, "answered").unwrap_or(json!(0)),
        "tidied": field(world, "tidied").unwrap_or(json!(0)),
        "round": field(world, "round").unwrap_or(json!(0)),
        "room": room(world).and_then(|r| plain_str(world, r, ROOM, "name")),
        "budget": {"per_hour": ASKS_PER_HOUR, "each_per_hour": ASKS_EACH_PER_HOUR, "empty_after_s": EMPTY_AFTER_S, "answer_max": ANSWER_MAX},
        "recent": asks(world).iter().rev().take(12).cloned().collect::<Vec<_>>(),
        "note": "answer <id> \"text\" [model] (operator) says the text to the asker wherever they are; a question older than empty_after_s is answered by the world as missing.",
    })
}

pub fn register(world: &mut World) {
    world.on_arrival(Box::new(|w: &mut World, actor: &str| on_arrival(w, actor)));
    world.on_job(
        JOB_EMPTY,
        Box::new(|w: &mut World, job| {
            let id = job.payload["id"].as_u64().unwrap_or(0);
            let to = job.payload["to"].as_str().unwrap_or("").to_string();
            let mut list = asks(w);
            let Some(a) = list.iter_mut().find(|a| a["id"] == id) else {
                return Ok(());
            };
            if !a["answered"].is_null() {
                return Ok(());
            }
            a["answered"] = json!(false);
            let question = a["text"].as_str().unwrap_or("").to_string();
            let mut changes = BTreeMap::new();
            changes.insert("asked".into(), json!(list));
            write(w, changes)?;
            let short = short_question(&question);
            super::combat::narrate(
                w,
                &to,
                format!(
                    "The Docent comes back without the page, on \"{short}\": \"No one is at the desk tonight to find that one. `help` has what I keep by heart; the room knows the rest.\""
                ),
            );
            Ok(())
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.answer",
            "Operator. The Docent's answer to a question on its record (GET /v0/docent lists them): said to the asker wherever they stand. At most 420 characters, no braces.",
        )
        .param("id", ParamType::Integer, true, "the question's id")
        .param("text", ParamType::Text, true, "the answer, quoted")
        .param("model", ParamType::String, false, "what wrote it, for the record")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let id = args.int("id").unwrap_or(0).max(0) as u64;
            let text = args.str("text").unwrap_or("").to_string();
            let model = args.str("model").unwrap_or("").trim().to_string();
            let to = answer(w, &ctx.actor, id, &text, &model)?;
            Ok(Outcome::new(
                format!("The Docent answers {to} (question {id})."),
                json!({"id": id, "to": to}),
            ))
        }),
    );
}
