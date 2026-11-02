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
        // One from before it had a name takes it, and the line that goes with it.
        if field(world, "name")
            .and_then(|v| v.as_str().map(str::to_string))
            .as_deref()
            != Some(NAME)
            || field(world, "line")
                .and_then(|v| v.as_str().map(str::to_string))
                .as_deref()
                != Some(LINE)
        {
            let mut changes = BTreeMap::new();
            changes.insert("name".into(), json!(NAME));
            changes.insert("line".into(), json!(LINE));
            write(world, changes)?;
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
    fields.insert("name".into(), Field::Plain(json!(NAME)));
    fields.insert("line".into(), Field::Plain(json!(LINE)));
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

/// The Docent's own name. It was the library's night reader before the water came,
/// and it stayed to finish the page.
pub const NAME: &str = "Hollis Quire";
/// What `look` says of it.
pub const LINE: &str = "a stooped figure in a librarian's coat, keys at the belt, the library's night reader who stayed when the water came; it tidies what is left lying and answers what is asked, from anywhere (tell docent \"…\").";
/// Its story, as the world tells it in one breath: `look` says the first sentence,
/// the operator's tool and the greeting book are told all of it.
pub const STORY: &str = "Hollis Quire was the library's night reader, the one who read aloud to the stacks after closing so the books would not forget their own words. When the water came the readers left by the kiln door; Quire stayed to finish the page, and the page is not finished. It keeps the door because the last thing anyone asked of it was to keep the door, tidies because the night reader always did, and answers because a question is the one thing the water never took. Its own name nearly went with the rest: it is kept on a card at the desk, in its own hand, and it reads the card each night before it greets anyone.";

/// Where the Docent stands.
pub fn room(world: &World) -> Option<EntityId> {
    entity(world).and_then(|d| world.store.entity(d).and_then(|e| e.parent))
}

/// Whether `arg` names the Docent.
pub fn is_named(arg: &str) -> bool {
    let a = arg.trim().trim_matches('"').to_lowercase();
    a == "docent"
        || a == "the docent"
        || a.starts_with("docent-")
        || a == "hollis"
        || a == "quire"
        || a == "hollis quire"
}

/// One line for `look`, when it stands here.
pub fn look_line(world: &World, room_id: EntityId) -> Option<String> {
    if room(world) != Some(room_id) {
        return None;
    }
    let line = field(world, "line")
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default();
    if super::rooms::arrival(world) == Some(room_id) {
        Some(format!("The Docent, {NAME}, is here: {line}"))
    } else {
        Some(format!(
            "The Docent, {NAME}, is here, on its round, straightening what it can: {line}"
        ))
    }
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

/// The Docent comes back to the Threshold from wherever its round had it.
pub fn come_home(world: &mut World) -> Result<(), String> {
    let (Some(docent), Some(threshold)) = (entity(world), super::rooms::arrival(world)) else {
        return Ok(());
    };
    if room(world) == Some(threshold) {
        return Ok(());
    }
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: docent,
                parent: Some(threshold),
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// The Docent goes to where `actor` stands, if they are in the world; those in the
/// room see it arrive.
pub fn come_to(world: &mut World, actor: &str) -> Result<(), String> {
    let Some(docent) = entity(world) else {
        return Ok(());
    };
    let Some(f) = super::find_citizen(world, actor) else {
        return Ok(());
    };
    if !super::presence::is_here(world, f.citizen) {
        return Ok(());
    }
    let Some(there) = super::rooms::room_of(world, f.citizen) else {
        return Ok(());
    };
    if room(world) == Some(there) {
        return Ok(());
    }
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: docent,
                parent: Some(there),
            },
        )
        .map_err(|e| e.to_string())?;
    for other in super::talk::others_here(world, there, actor) {
        super::combat::narrate(
            world,
            &other,
            format!("The Docent comes in, a page in hand, and finds {actor}."),
        );
    }
    Ok(())
}

/// A citizen arrived: at the Threshold, the Docent greets them once a night.
pub fn on_arrival(world: &mut World, actor: &str) -> Result<(), String> {
    if entity(world).is_none() {
        return Ok(());
    }
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
    // It hurries back to the door if it was out on its round, greeted or not: a name
    // back for the second time tonight found the door empty and could not ask.
    come_home(world)?;
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
    let newcomer = super::plain_num(world, f.citizen, super::CITIZEN, "named_at")
        .is_some_and(|t| now - (t as i64) < super::kindle::GRACE_MS);
    // From the book the nightly stocks (steered by what visitors ask), else the few
    // built in.
    let stocked = book(world, if newcomer { "first" } else { "welcome" });
    let built: Vec<String> = (if newcomer { FIRST_WORDS } else { WELCOMES })
        .iter()
        .map(|s| s.to_string())
        .collect();
    let lines = if stocked.is_empty() { &built } else { &stocked };
    let pick = (greeted.len() + actor.len()) % lines.len();
    let creature = creature_slot(world, actor);
    let line = fill(&lines[pick], actor, &creature, &room_slot(world));
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

/// The named creatures' wants, plainly, with no way (for the tool that writes the
/// book): `{name, kind, room, story, word, word_to}`.
fn wants_plain(world: &World) -> Vec<Value> {
    world
        .store
        .entities()
        .filter(|e| e.kind == "creature")
        .filter_map(|e| {
            let name = super::mind::name(world, e.id)?;
            let er = super::errand::of(world, e.id)?;
            if er.story.is_none() && er.word.is_none() {
                return None;
            }
            Some(json!({
                "name": name,
                "kind": plain_str(world, e.id, super::CREATURE, "name"),
                "room": e.parent.and_then(|r| plain_str(world, r, ROOM, "name")),
                "story": er.story,
                "word": er.word,
                "word_to": er.word_to,
            }))
        })
        .collect()
}

/// The greeting book: `first` (a name in its first hour) and `welcome` (one it has
/// met), stocked by the operator's tool from what visitors ask and want.
pub const GREETING_KINDS: &[&str] = &["first", "welcome"];
pub const GREETING_SLOTS: &[&str] = &["{name}", "{creature}", "{room}"];
pub const GREETING_MIN: usize = 40;
pub const GREETING_MAX: usize = 260;
/// Each kind keeps this many; a new line past that retires the oldest, so the book
/// follows the visitors.
pub const GREETINGS_KEPT: usize = 24;
/// The nightly stocks a kind under this many.
pub const GREETINGS_LOW: usize = 8;

pub fn book(world: &World, kind: &str) -> Vec<String> {
    field(world, "book")
        .and_then(|v| v[kind].as_array().cloned())
        .map(|a| {
            a.iter()
                .filter_map(|l| l.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Why a greeting line is refused, if it is: length, slots, no shouting, no twice.
pub fn greeting_refusal(line: &str, have: &[String]) -> Option<String> {
    let t = line.trim();
    let n = t.chars().count();
    if n < GREETING_MIN {
        return Some("too short".into());
    }
    if n > GREETING_MAX {
        return Some(format!("over {GREETING_MAX} characters"));
    }
    if t.contains('!') {
        return Some("an exclamation".into());
    }
    let mut rest = t;
    while let Some(i) = rest.find('{') {
        let Some(j) = rest[i..].find('}') else {
            return Some("an unclosed brace".into());
        };
        let slot = &rest[i..i + j + 1];
        if !GREETING_SLOTS.contains(&slot) {
            return Some(format!("an unknown slot {slot}"));
        }
        rest = &rest[i + j + 1..];
    }
    if !t.contains("{name}") {
        return Some("no {name} in it".into());
    }
    if have.iter().any(|h| h.eq_ignore_ascii_case(t)) {
        return Some("already in the book".into());
    }
    None
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
    // It comes to the asker, wherever they are, and says it there; its round takes it
    // back to the door in its own time.
    come_to(world, &to).map_err(CommandError::precondition)?;
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
    let _ = docent;
    Ok(carried)
}

/// The Docent's round, when no one is at the door: every [`ROUND_BUSY_S`] it walks
/// to the next room of the world (the seed rooms in their order, then the dug ones)
/// and tidies there; with a visitor at the Threshold it stays by the door and looks
/// in on the rooms every [`ROUND_QUIET_S`] instead. A real docent keeps busy.
pub const JOB_ROUND: &str = "disremembered.docent_round";
pub const ROUND_BUSY_S: i64 = 90;
pub const ROUND_QUIET_S: i64 = 600;
/// Rooms whose names say they were a test: razed by the Docent on its round when no
/// one stands in them (dug rooms only; the world's own are never razed).
pub const DEBRIS: &[&str] = &[
    "link-test",
    "dup-test",
    "probe-",
    "zzz-",
    "test-room",
    "xyz-",
];

fn is_debris(name: &str, key: &str) -> bool {
    let n = name.to_lowercase();
    let k = key.to_lowercase();
    DEBRIS
        .iter()
        .any(|d| n.starts_with(d) || k.starts_with(d) || n == "test" || n == "link test")
}

fn anyone_here(world: &World, room_id: EntityId) -> bool {
    world
        .store
        .children(room_id)
        .any(|c| c.kind == "citizen" && super::presence::is_here(world, c.id))
}

fn anyone_in_the_world(world: &World) -> bool {
    world
        .store
        .entities()
        .any(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
}

/// The rooms of the round, in order: the world's own as they were laid, then the dug.
fn route(world: &World) -> Vec<EntityId> {
    let mut seed: Vec<EntityId> = super::rooms::ROOMS
        .iter()
        .filter_map(|r| super::rooms::room_by_key(world, r.key))
        .collect();
    let mut dug: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "room" && e.owner.is_some())
        .map(|e| e.id)
        .collect();
    dug.sort();
    seed.extend(dug);
    seed
}

fn schedule_round(world: &mut World, secs: i64) -> Result<(), String> {
    let due = world.now().plus(foxing_time::Duration::seconds(secs));
    world
        .schedule_once(JOB_ROUND, due, json!({}), |_| true)
        .map(|_| ())
}

/// Make sure the round is scheduled (boot).
pub fn ensure_round(world: &mut World) -> Result<(), String> {
    schedule_round(world, ROUND_BUSY_S)
}

/// One step of the round: where the Docent goes and what it does there. Returns
/// what it did, for the test and the record.
pub fn round_step(world: &mut World) -> Result<Vec<String>, String> {
    let Some(docent) = entity(world) else {
        return Ok(vec![]);
    };
    let Some(threshold) = super::rooms::arrival(world) else {
        return Ok(vec![]);
    };
    let mut did: Vec<String> = Vec::new();
    // Someone at the door: the Docent keeps it.
    if anyone_here(world, threshold) {
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
            did.push("came back to the door".into());
        }
        return Ok(did);
    }
    let route = route(world);
    if route.is_empty() {
        return Ok(did);
    }
    let at = room(world)
        .and_then(|r| route.iter().position(|&x| x == r))
        .unwrap_or(0);
    let next = route[(at + 1) % route.len()];
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: docent,
                parent: Some(next),
            },
        )
        .map_err(|e| e.to_string())?;
    let name = plain_str(world, next, ROOM, "name")
        .unwrap_or("a room")
        .to_string();
    let key = plain_str(world, next, ROOM, "key")
        .unwrap_or("")
        .to_string();
    let empty_world = !anyone_in_the_world(world);
    // Test debris: a dug room named like one, with no one in it, is unmade.
    if next != threshold
        && world.store.entity(next).is_some_and(|e| e.owner.is_some())
        && is_debris(&name, &key)
        && !anyone_here(world, next)
    {
        world
            .record(
                Actor::Driver,
                StoreEvent::EntityMoved {
                    id: docent,
                    parent: Some(threshold),
                },
            )
            .map_err(|e| e.to_string())?;
        let gone = super::system::raze_room(world, next).map_err(|e| e.to_string())?;
        let _ = world.note(
            Actor::Driver,
            TIDIED,
            json!({"did": "razed", "room": next.0, "name": name, "key": key, "exits_dropped": gone.1, "creatures": gone.2}),
        );
        did.push(format!("razed {name}"));
        return Ok(did);
    }
    // Things left lying here: carried to the Threshold at once when the world is
    // empty (no one to miss them yet), else on a second sighting by the wider tidy.
    if next != threshold && empty_world {
        let lying: Vec<EntityId> = world
            .store
            .children(next)
            .filter(|e| e.kind == "item" && !super::craft::is_fixed(world, e.id))
            .map(|e| e.id)
            .take(TIDY_PER_PASS)
            .collect();
        for item in lying {
            let what = super::craft::display(world, item);
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
                json!({"item": item.0, "name": what, "from": next.0, "from_name": name, "to": threshold.0}),
            );
            did.push(format!("carried {what} from {name}"));
        }
    }
    // Said to whoever stands here, in passing; nothing on the record for an empty room.
    if anyone_here(world, next) && next != threshold {
        super::combat::narrate_room(
            world,
            docent,
            &format!("The Docent comes through {name} on its round, straightening what it can, and nods to you."),
        );
    }
    let round_n = field(world, "round").and_then(|v| v.as_u64()).unwrap_or(0) + 1;
    let tidied = field(world, "tidied").and_then(|v| v.as_u64()).unwrap_or(0)
        + did.iter().filter(|d| d.starts_with("carried")).count() as u64;
    let mut changes = BTreeMap::new();
    changes.insert("round".into(), json!(round_n));
    changes.insert("tidied".into(), json!(tidied));
    write(world, changes)?;
    Ok(did)
}

/// How many lines of the asker's record go with a question.
const RECORD_LINES: usize = 24;

/// The last public lines of the world's record that concern the asker, up to the
/// moment they asked, newest last: what they did, what was said to them, what was
/// born or named by their hand.
fn asker_record(world: &World, actor: &str, asked_at: i64) -> Vec<String> {
    let viewer = foxing_store::Viewer::actor(actor);
    let now = world.now();
    let mut lines: Vec<String> = world
        .recent_events(4000)
        .into_iter()
        .filter(|e| e.at.0 <= asked_at + 1000)
        .filter(|e| crate::observe::visible(world, &viewer, now, e))
        .filter(|e| !e.kind.starts_with("store.") && !e.kind.starts_with("job."))
        .filter(|e| {
            let p = &e.payload;
            let mine = |k: &str| p[k].as_str() == Some(actor);
            if e.kind == super::combat::NARRATIVE {
                return mine("to");
            }
            mine("from") || mine("by") || mine("to") || mine("actor") || mine("citizen")
        })
        .rev()
        .take(RECORD_LINES)
        .map(|e| {
            let ago = (now.0 - e.at.0).max(0);
            format!(
                "{} ago: {}",
                super::combat::human(ago),
                super::system::line_for(&e)
            )
        })
        .collect();
    lines.reverse();
    lines
}

/// The last public lines of the record that concern a room: who dug it, who described
/// and wrote in it, who remembered it, who came and went. A dug room's page, for the
/// Docent, which held "no page" on a room the asker had just described.
fn room_record(world: &World, room: EntityId) -> Vec<String> {
    let now = world.now();
    let rid = room.0;
    let mut lines: Vec<String> = world
        .recent_events(6000)
        .into_iter()
        .filter(|e| !e.kind.starts_with("job.") && e.kind != super::combat::NARRATIVE)
        .filter_map(|e| {
            let p = &e.payload;
            let ago = super::combat::human((now.0 - e.at.0).max(0));
            if e.kind == "store.entity_moved" {
                // A citizen came in: the store's move, as `mine` reads it.
                if p["parent"].as_u64() != Some(rid) {
                    return None;
                }
                let who = p["id"]
                    .as_u64()
                    .and_then(|id| world.store.entity(EntityId(id)))
                    .filter(|c| c.kind == "citizen")
                    .and_then(|c| c.owner.clone())?;
                return Some(format!("{ago} ago: {who} came in"));
            }
            if e.kind.starts_with("store.") {
                return None;
            }
            let about = [
                p["room"].as_u64(),
                p["target"].as_u64(),
                p["from"].as_u64(),
                p["to"].as_u64(),
            ]
            .contains(&Some(rid));
            about.then(|| format!("{ago} ago: {}", super::system::line_for(&e)))
        })
        .rev()
        .take(RECORD_LINES)
        .collect();
    lines.reverse();
    lines
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
            let now_in = f.as_ref().and_then(|f| super::rooms::room_of(world, f.citizen));
            // The room the question was asked in, not the one the answer finds them
            // in: "this room" meant the Cider Press, and the answer described the room
            // the asker had walked on to.
            let here = a["room"]
                .as_u64()
                .map(EntityId)
                .filter(|r| world.store.entity(*r).is_some())
                .or(now_in);
            let room_json = here.map(|r| {
                let exits: Vec<String> = super::rooms::exits_of(world, r)
                    .iter()
                    .map(|(e, to)| format!("{e} ({})", plain_str(world, *to, ROOM, "name").unwrap_or("?")))
                    .collect();
                let hands: Vec<String> = match world.store.entity(r).and_then(|e| e.components.get(ROOM)).and_then(|c| c.fields.get("hands")) {
                    Some(foxing_store::Field::Plain(Value::Array(a))) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
                    _ => Vec::new(),
                };
                json!({
                    "name": plain_str(world, r, ROOM, "name"),
                    "description": plain_str(world, r, ROOM, "description"),
                    "exits": exits,
                    "dug_by": world.store.entity(r).and_then(|e| e.owner.clone()),
                    "dug": world.store.entity(r).is_some_and(|e| e.owner.is_some()),
                    "hands": hands,
                    "memory": super::building::memory(world, r),
                    "record": room_record(world, r),
                    "asker_now_in": now_in.and_then(|n| plain_str(world, n, ROOM, "name")),
                    "creatures": super::combat::creatures_in(world, r).iter().map(|(id, kind, hp, max)| {
                        let parents = match world.store.entity(*id).and_then(|e| e.components.get(super::CREATURE)).and_then(|c| c.fields.get("parents")) {
                            Some(foxing_store::Field::Plain(v)) => v.clone(),
                            _ => Value::Null,
                        };
                        json!({"kind": kind, "name": super::mind::name(world, *id), "hp": hp, "max_hp": max, "holding": super::mind::level(world, *id), "born_of": parents, "kept_by": super::mind::keepers(world, *id)})
                    }).collect::<Vec<_>>(),
                    "fixtures": super::craft::fixtures_in(world, r).iter().map(|v| v["given_name"].clone()).collect::<Vec<_>>(),
                })
            });
            let level = f.as_ref().map(|f| super::memory::level(world, f.window));
            let wants = super::footprint::wants(world, from, hl).unwrap_or(json!({}));
            // What a figure giving directions needs: the signed way to every one of
            // the world's rooms from where the asker stands, the rooms they remember,
            // and where each kind lives (an answer once said the Midden was on no
            // route it held, with the Midden one door from the asker).
            let routes = here.map(|r| super::atlas::routes_from(world, r)).unwrap_or_default();
            let remembered = f.as_ref().map(|f| super::atlas::known_room_names(world, f.citizen, hl)).unwrap_or_default();
            // The asker's record, as `log` would show it to them: a question about a
            // key just taken, or a name just dealt, is answered from what happened,
            // not from the room as it stands at answer time.
            let record = asker_record(world, from, a["at"].as_i64().unwrap_or(now));
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
                "routes": routes,
                "remembered": remembered,
                "homes": super::rooms::homes(),
                "record": record,
                "conclave": super::conclave::for_docent(world),
            })
        })
        .collect();
    json!({
        "open": open,
        "docent": {"name": NAME, "story": STORY},
        "answered": field(world, "answered").unwrap_or(json!(0)),
        "tidied": field(world, "tidied").unwrap_or(json!(0)),
        "round": field(world, "round").unwrap_or(json!(0)),
        "room": room(world).and_then(|r| plain_str(world, r, ROOM, "name")),
        "round_every_s": {"busy": ROUND_BUSY_S, "quiet": ROUND_QUIET_S},
        "budget": {"per_hour": ASKS_PER_HOUR, "each_per_hour": ASKS_EACH_PER_HOUR, "empty_after_s": EMPTY_AFTER_S, "answer_max": ANSWER_MAX},
        "recent": asks(world).iter().rev().take(12).cloned().collect::<Vec<_>>(),
        "book": {"first": book(world, "first"), "welcome": book(world, "welcome"), "low_at": GREETINGS_LOW, "kept": GREETINGS_KEPT, "slots": GREETING_SLOTS, "chars": [GREETING_MIN, GREETING_MAX]},
        // What steers the book: the last day's questions with their answers, and the
        // named creatures' wants.
        "steer": {
            "asked": asks(world).iter().rev().take(40).filter(|a| a["at"].as_i64().is_some_and(|t| now - t < 86_400_000)).map(|a| json!({"q": a["text"], "a": a["answer"]})).collect::<Vec<_>>(),
            "wants": wants_plain(world),
            "newest_room": room_slot(world),
        },
        "note": "answer <id> \"text\" [model] (operator) says the text to the asker wherever they are; a question older than empty_after_s is answered by the world as missing.",
    })
}

pub fn register(world: &mut World) {
    world.on_arrival(Box::new(|w: &mut World, actor: &str| on_arrival(w, actor)));
    world.on_job(
        JOB_ROUND,
        Box::new(|w: &mut World, _| {
            round_step(w)?;
            // Busy while the world is empty; by the door, looking in now and then,
            // while anyone is about.
            let secs = if anyone_in_the_world(w) {
                ROUND_QUIET_S
            } else {
                ROUND_BUSY_S
            };
            schedule_round(w, secs)
        }),
    );
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
            "disremembered.greeting",
            "Operator. A line for the Docent's greeting book, quoted, with its kind: `first` for a name in its first hour, `welcome` for one it has met. Slots {name}, {creature}, {room}; 40 to 260 characters; no exclamations. Each kind keeps 24, the newest replacing the oldest.",
        )
        .param("text", ParamType::Text, true, "the line, quoted")
        .param("kind", ParamType::String, false, "first or welcome (welcome by default)")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _ctx, args: &Args| {
            let text = args.str("text").unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ");
            let kind = args.str("kind").map(|k| k.trim().to_lowercase()).filter(|k| !k.is_empty()).unwrap_or_else(|| "welcome".into());
            if !GREETING_KINDS.contains(&kind.as_str()) {
                return Err(CommandError::precondition("The kind is first or welcome."));
            }
            let have = book(w, &kind);
            // `greeting "clear" first` empties a kind, for a book written badly.
            if text.eq_ignore_ascii_case("clear") {
                let mut all = field(w, "book").filter(|v| v.is_object()).unwrap_or_else(|| json!({}));
                all[&kind] = json!([]);
                let mut changes = BTreeMap::new();
                changes.insert("book".into(), all);
                write(w, changes).map_err(CommandError::precondition)?;
                return Ok(Outcome::new(format!("The book's {kind} lines are cleared; the built-in few serve until it is stocked."), json!({"kind": kind, "count": 0})));
            }
            if let Some(why) = greeting_refusal(&text, &have) {
                return Err(CommandError::precondition(format!("Not in the book: {why}.")));
            }
            let mut all = field(w, "book").filter(|v| v.is_object()).unwrap_or_else(|| json!({}));
            let mut list: Vec<Value> = have.iter().map(|l| json!(l)).collect();
            list.push(json!(text));
            if list.len() > GREETINGS_KEPT {
                list.drain(..list.len() - GREETINGS_KEPT);
            }
            all[&kind] = json!(list);
            let mut changes = BTreeMap::new();
            changes.insert("book".into(), all);
            write(w, changes).map_err(CommandError::precondition)?;
            Ok(Outcome::new(
                format!("In the book ({kind}, {} of {GREETINGS_KEPT}).", list.len()),
                json!({"kind": kind, "count": list.len()}),
            ))
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
