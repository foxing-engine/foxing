//! Building: players extend the world. `dig` opens a new room off the one you stand
//! in, paid for with a chunk: the room is born with that memory's half-life, and its
//! own `memory` fades on the same curve until someone visits. A faded room stays on
//! the map with its name and exits; only its description dims, in steps, toward a
//! half-remembered line. `describe` and `name` are the owner's, or a designee's
//! (`permit`, `revoke`, with an optional expiry). The seed rooms belong to the world:
//! an operator describes them, and a citizen may dig from each of them once.
//!
//! Every verb here is a registry command, so the vocabulary carries its signature and
//! the Gate compiles it like any other; a name goes through the checker's reserved-word
//! test, so no one digs "the gate".

use super::rooms::{exits_of, room_of};
use super::{internal, me, plain_num, plain_str, CHUNK, EXITS, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{
    ActorKind, Args, CallContext, CommandError, CommandSpec, Outcome, ParamType,
};
use foxing_store::{EntityId, Field, StoreEvent};
use foxing_time::{Curve, Duration, TimedValue};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Most characters a description may carry, appended or not.
pub const MAX_DESCRIPTION_CHARS: usize = 800;
/// Most characters a room name may carry.
const MAX_NAME_CHARS: usize = 40;
/// Below this memory the description is only its first sentence; below `FADED` it is gone.
pub(crate) const DIM: f64 = 0.5;
pub(crate) const FADED: f64 = 0.15;
/// What a room says of itself when its memory has faded.
pub const HALF_REMEMBERED: &str =
    "A half-remembered place. Someone described it once; the words have faded.";
const UNDESCRIBED: &str = "Freshly dug. No one has described it yet.";

/// Whether a room was dug by a citizen (and so has a memory), as opposed to seeded.
/// A named creature invited to live in a dug room: `{by, creature, name, kind, from, room}`.
pub const INVITED: &str = "disremembered.invited";
/// A word put on a dug room's door: `{by, room, name, hint}`. Public (the word is not).
pub const WARDED: &str = "disremembered.warded";

pub fn is_built(world: &World, room: EntityId) -> bool {
    world
        .store
        .entity(room)
        .and_then(|e| e.components.get(ROOM))
        .is_some_and(|c| c.fields.contains_key("memory"))
}

/// A built room's memory now, 0 to 1; `None` for a seed room, which never fades.
pub fn memory(world: &World, room: EntityId) -> Option<f64> {
    if !is_built(world, room) {
        return None;
    }
    world
        .store
        .read_field(room, ROOM, "memory", world.now())
        .ok()
        .and_then(|v| v.as_f64())
}

/// The half-life a built room's memory fades on, for the Docent's reckoning.
pub fn half_life_of(world: &World, room: EntityId) -> Option<Duration> {
    half_life(world, room)
}

/// The half-life a built room's memory fades on.
fn half_life(world: &World, room: EntityId) -> Option<Duration> {
    match world
        .store
        .entity(room)?
        .components
        .get(ROOM)?
        .fields
        .get("half_life_ms")?
    {
        Field::Plain(v) => v.as_i64().map(Duration),
        _ => None,
    }
}

/// A room's fields with these changed: a component is set whole, so a change to one
/// field carries the others along.
fn merged(
    world: &World,
    room: EntityId,
    changes: BTreeMap<String, Field>,
) -> BTreeMap<String, Field> {
    let mut fields = world
        .store
        .entity(room)
        .and_then(|e| e.components.get(ROOM))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    fields.extend(changes);
    fields
}

/// Someone is here: a built room remembers itself again, in full.
pub fn visited(world: &mut World, room: EntityId) -> Result<(), String> {
    let Some(hl) = half_life(world, room) else {
        return Ok(());
    };
    if memory(world, room).is_some_and(|m| m > 0.999) {
        return Ok(());
    }
    let mut fields = BTreeMap::new();
    fields.insert(
        "memory".into(),
        Field::Timed(TimedValue::new(
            1.0,
            world.now(),
            Curve::Exponential { half_life: hl },
        )),
    );
    let fields = merged(world, room, fields);
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id: room,
                component: ROOM.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// The description as a visitor finds it: whole while the room is remembered, its first
/// sentence as the memory dims, a stock line once it has faded.
pub fn remembered_description(description: &str, memory: Option<f64>) -> String {
    match memory {
        None => description.to_string(),
        Some(m) if m >= DIM => description.to_string(),
        Some(m) if m >= FADED => {
            let first = description
                .split_inclusive(['.', '!', '?'])
                .next()
                .unwrap_or(description)
                .trim();
            format!("{first} The rest is hard to recall.")
        }
        Some(_) => HALF_REMEMBERED.to_string(),
    }
}

/// The way back for a compass exit, or `back`.
/// The way back from an exit, as `dig` names it.
pub(crate) fn opposite_of(exit: &str) -> &'static str {
    opposite(exit)
}

fn opposite(exit: &str) -> &'static str {
    match exit {
        "north" => "south",
        "south" => "north",
        "east" => "west",
        "west" => "east",
        "up" => "down",
        "down" => "up",
        "in" => "out",
        "out" => "in",
        _ => "back",
    }
}

/// A room key from a name and an id: `salt-stair-12`.
fn key_for(name: &str, id: EntityId) -> String {
    let slug: String = name
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty() && *s != "the" && *s != "a" && *s != "an")
        .take(3)
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        format!("dug-{}", id.0)
    } else {
        format!("{slug}-{}", id.0)
    }
}

/// A name a room may carry: short, printable, and not a word the protocol reserves.
pub(crate) fn check_name(name: &str) -> Result<String, CommandError> {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    let n = name.chars().count();
    if !(2..=MAX_NAME_CHARS).contains(&n) {
        return Err(CommandError::precondition(format!(
            "A name is 2 to {MAX_NAME_CHARS} characters."
        )));
    }
    if name.contains(['[', ']', '"', '“', '”']) {
        return Err(CommandError::precondition(
            "A name carries no brackets or quotation marks.",
        ));
    }
    if let Some(word) = crate::gate::world_checker().collision(&name) {
        return Err(CommandError::precondition(format!(
            "'{word}' is a word CLEAR reserves; a name may not carry it."
        )));
    }
    Ok(name)
}

/// Whether `actor` may change a room: its owner, a current designee, or an operator.
pub(crate) fn may_edit(
    world: &World,
    ctx: &CallContext,
    room: EntityId,
) -> Result<(), CommandError> {
    if ctx.kind == ActorKind::Admin {
        return Ok(());
    }
    let e = world
        .store
        .entity(room)
        .ok_or_else(|| internal("no such room"))?;
    if !is_built(world, room) {
        return Err(CommandError::precondition(
            "This room is the world's; only an operator changes it. Dig your own.",
        ));
    }
    if e.owner.as_deref() == Some(ctx.actor.as_str()) {
        return Ok(());
    }
    if granted(world, room, "permits", &ctx.actor) {
        Ok(())
    } else {
        Err(CommandError::precondition(format!(
            "{} does not have this room's owner's permission.",
            ctx.actor
        )))
    }
}

/// Whether `actor` may add to a room: anyone who may change it, or a welcomed guest.
pub(crate) fn may_append(
    world: &World,
    ctx: &CallContext,
    room: EntityId,
) -> Result<(), CommandError> {
    match may_edit(world, ctx, room) {
        Ok(()) => Ok(()),
        Err(e) => {
            if is_built(world, room) && granted(world, room, "welcomes", &ctx.actor) {
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

/// Whether `who` holds a live grant in the room's `field` map (`permits` or `welcomes`):
/// an entry whose expiry is null or still ahead.
fn granted(world: &World, room: EntityId, field: &str, who: &str) -> bool {
    let now = world.now().0;
    let grants = grants_of(world, room, field);
    let live = |until: &Value| until.is_null() || until.as_i64().is_some_and(|t| t > now);
    // `welcome all` is the key "*": a guest book any hand may sign.
    grants.get(who).is_some_and(live) || (field == "welcomes" && grants.get("*").is_some_and(live))
}

/// `all`, `anyone` or `everyone` as a citizen's name, for a welcome to every hand.
fn everyone(typed: &str) -> bool {
    matches!(
        typed.to_lowercase().as_str(),
        "all" | "anyone" | "everyone" | "*"
    )
}

pub(crate) fn owner_only(
    world: &World,
    ctx: &CallContext,
    room: EntityId,
) -> Result<(), CommandError> {
    if ctx.kind == ActorKind::Admin {
        return Ok(());
    }
    let owner = world.store.entity(room).and_then(|e| e.owner.clone());
    if owner.as_deref() == Some(ctx.actor.as_str()) {
        Ok(())
    } else if is_built(world, room) {
        Err(CommandError::precondition(
            "Only the room's owner gives or takes permission.",
        ))
    } else {
        Err(CommandError::precondition(
            "This room is the world's; it has no owner to permit anyone.",
        ))
    }
}

fn set_room_fields(
    world: &mut World,
    actor: &str,
    room: EntityId,
    fields: BTreeMap<String, Field>,
) -> Result<(), CommandError> {
    let fields = merged(world, room, fields);
    world
        .record(
            Actor::Citizen(actor.to_string()),
            StoreEvent::ComponentSet {
                id: room,
                component: ROOM.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(internal)
}

fn grants_of(world: &World, room: EntityId, field: &str) -> serde_json::Map<String, Value> {
    match world
        .store
        .entity(room)
        .and_then(|e| e.components.get(ROOM))
        .and_then(|c| c.fields.get(field))
    {
        Some(Field::Plain(Value::Object(m))) => m.clone(),
        _ => serde_json::Map::new(),
    }
}

/// The public record of a memory given up for a room:
/// `{room, name, owner, by, with, spent, added_ms, half_life_ms}`.
/// How many doors a room takes, its own and the dug ones together.
pub const MAX_EXITS: usize = 8;
pub const REMEMBERED: &str = "disremembered.remembered";
/// A room was dug: `{by, room, key, name, from, exit, with}`. Public.
pub const DUG: &str = "disremembered.dug";
/// A line was appended to a room's description or a book's pages:
/// `{by, kind, target, name, owner|maker, chars}`. Public.
pub const APPENDED: &str = "disremembered.appended";

/// How many rooms `actor` has dug off `from`.
fn dug_from(world: &World, actor: &str, from: EntityId) -> usize {
    world
        .store
        .entities()
        .filter(|e| e.kind == "room" && e.owner.as_deref() == Some(actor))
        .filter(|e| {
            matches!(
                e.components.get(ROOM).and_then(|c| c.fields.get("origin")),
                Some(Field::Plain(v)) if v.as_u64() == Some(from.0)
            )
        })
        .count()
}

/// The half-life a chunk's value fades on, which the room dug with it inherits.
fn chunk_half_life(world: &World, chunk: EntityId) -> Duration {
    match world
        .store
        .entity(chunk)
        .and_then(|e| e.components.get(CHUNK))
        .and_then(|c| c.fields.get("value"))
    {
        Some(Field::Timed(t)) => match t.curve {
            Curve::Exponential { half_life } => half_life,
            Curve::PowerLaw { scale, .. } => scale,
            _ => Duration::seconds(3600),
        },
        _ => Duration::seconds(3600),
    }
}

/// Whether a dug room's words invite a line (a note, a signature, a guest book) that
/// no hand is welcomed to write, and nothing stands to take it: the Parlor's, the
/// Footnote Room's and the Margin's texts asked for notes their owners never
/// `welcome`d. The owner is told how to make the invitation real.
pub fn unmet_invitation(world: &World, room: EntityId) -> Option<&'static str> {
    if !is_built(world, room) {
        return None;
    }
    let desc = plain_str(world, room, ROOM, "description")?.to_lowercase();
    let invites = [
        "leave a note",
        "write in",
        "sign the",
        "sign your",
        "leave your name",
        "guest book",
        "visitor book",
        "write your",
        "leave a line",
        "add a line",
        "scratch your",
        "carve your",
        "leave a word",
        "write here",
    ]
    .iter()
    .any(|p| desc.contains(p));
    if !invites {
        return None;
    }
    let welcomed = !grants_of(world, room, "welcomes").is_empty();
    let book = super::craft::fixtures_in(world, room).iter().any(|f| {
        f["origin"].as_str() == Some("book") || f["text"].as_str().is_some_and(|t| !t.is_empty())
    });
    if welcomed || book {
        return None;
    }
    Some("Its words invite a line, and no hand is welcomed to write one: `welcome all` opens the room to every hand, and `craft book` (rag thread, a dab of ink, a chunk of sixty) makes the book real. What the words promise beyond that (a hum, a toast, a sign) the room cannot do; a lamp, a bell or a basin can be made (`recipes`).")
}

/// A dug room's door may ask for a word (`ward`): `{word, hint, by, passed: [names]}`.
pub fn ward_of(world: &World, room: EntityId) -> Option<Value> {
    world
        .store
        .entity(room)
        .and_then(|e| e.components.get(ROOM))
        .and_then(|c| c.fields.get("ward"))
        .and_then(|f| match f {
            Field::Plain(v) if v.is_object() => Some(v.clone()),
            _ => None,
        })
}

/// Whether `ctx` may enter `room` through its ward, given the word they said; a
/// stranger who says it is remembered and asked no more.
pub(crate) fn pass_ward(
    world: &mut World,
    ctx: &CallContext,
    room: EntityId,
    said: &str,
) -> Result<(), CommandError> {
    let Some(ward) = ward_of(world, room) else {
        return Ok(());
    };
    if ctx.kind == ActorKind::Admin {
        return Ok(());
    }
    let owner = world.store.entity(room).and_then(|e| e.owner.clone());
    if owner.as_deref() == Some(ctx.actor.as_str())
        || kept_by(world, room).iter().any(|k| k == &ctx.actor)
        || granted(world, room, "permits", &ctx.actor)
        || granted(world, room, "welcomes", &ctx.actor)
        || ward["passed"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v.as_str() == Some(ctx.actor.as_str())))
    {
        return Ok(());
    }
    let word = ward["word"].as_str().unwrap_or("").trim().to_lowercase();
    let said = said.trim().trim_matches('"').trim().to_lowercase();
    if !word.is_empty() && said == word {
        let mut ward = ward;
        let mut passed = ward["passed"].as_array().cloned().unwrap_or_default();
        passed.push(json!(ctx.actor));
        ward["passed"] = Value::Array(passed);
        let mut fields = BTreeMap::new();
        fields.insert("ward".into(), Field::Plain(ward));
        set_room_fields(world, &ctx.actor, room, fields)?;
        let name = plain_str(world, room, ROOM, "name")
            .unwrap_or("the room")
            .to_string();
        super::combat::narrate(
            world,
            &ctx.actor,
            format!("The door of {name} hears the word and opens; it will know you now."),
        );
        return Ok(());
    }
    let name = plain_str(world, room, ROOM, "name")
        .unwrap_or("The room")
        .to_string();
    let hint = ward["hint"]
        .as_str()
        .filter(|h| !h.trim().is_empty())
        .map(|h| format!(" Over the door: \"{h}\""))
        .unwrap_or_default();
    Err(CommandError::precondition(format!(
        "{name}'s door asks for a word{}.{hint} `travel <exit> \"<word>\"` says it; the room's owner{} may tell you.",
        if said.is_empty() { "" } else { ", and that was not it" },
        owner.map(|o| format!(", {o},")).unwrap_or_default()
    )))
}

/// Who keeps a room: the names on its `kept` list.
pub fn kept_by(world: &World, room: EntityId) -> Vec<String> {
    match world
        .store
        .entity(room)
        .and_then(|e| e.components.get(ROOM))
        .and_then(|c| c.fields.get("kept"))
    {
        Some(Field::Plain(Value::Array(a))) => a
            .iter()
            .filter_map(|k| k["by"].as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

pub fn humane(d: Duration) -> String {
    let s = d.0 / 1000;
    if s >= 172_800 {
        format!("{} days", s / 86_400)
    } else if s >= 7200 {
        // To the half hour: "3 hours" for 3.5 read as a lie beside the panel's 3.5h.
        let h = (s as f64 / 1800.0).round() / 2.0;
        if h == h.floor() {
            format!("{h:.0} hours")
        } else {
            format!("{h:.1} hours")
        }
    } else if s >= 120 {
        format!("{} minutes", s / 60)
    } else {
        format!("{s} seconds")
    }
}

pub fn register(world: &mut World, curves: super::Curves) {
    world.register(
        CommandSpec::new(
            "disremembered.dig",
            "Open a new room off this one, through a new exit, paid for with one of your chunks: the room is born with that memory's half-life and fades unless visited. Not at the Threshold: the Mudroom, west of it, is where the digging starts, and takes any number of doors. From each of the world's other rooms (the seed rooms) once; from your own, as you like. Every room but the Mudroom takes eight doors in all, and `look` says how many are left where some were dug. A kept name's verb (every name is kept unless it asked to be passing at the gate; the play page's names are kept).",
        )
        .param("exit", ParamType::String, true, "the new exit's name, not one this room has")
        .param("chunk", ParamType::Entity, true, "a chunk of yours; it is spent")
        .param("name", ParamType::Text, false, "the new room's name, quoted; it can be named later"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            super::kept_only(w, ctx, "dig")?;
            let f = me(w, ctx)?;
            let here = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let exit = args.str("exit").unwrap_or("").trim().to_ascii_lowercase();
            if exit.is_empty()
                || exit.chars().count() > 16
                || !exit.chars().all(|c| c.is_ascii_lowercase() || c == '-')
            {
                return Err(CommandError::precondition(
                    "An exit is one lowercase word, up to 16 letters (hyphens allowed).",
                ));
            }
            if exit == "back" {
                return Err(CommandError::precondition(
                    "'back' is the way you came, not a door; name the exit something else.",
                ));
            }
            if let Some(word) = crate::gate::world_checker().collision(&exit) {
                return Err(CommandError::precondition(format!(
                    "'{word}' is a word CLEAR reserves; an exit may not be it."
                )));
            }
            if exits_of(w, here).contains_key(&exit) {
                return Err(CommandError::precondition(format!(
                    "This room already has an exit '{exit}'."
                )));
            }
            let seed = !is_built(w, here);
            let owner = w.store.entity(here).and_then(|e| e.owner.clone());
            if ctx.kind != ActorKind::Admin {
                // The front door is kept clear (T1): fifteen doors once opened off
                // it, thirteen of them dug or washed up there.
                if Some(here) == super::rooms::arrival(w) {
                    return Err(CommandError::precondition(
                        "Nothing is dug at the door. The Mudroom, west of here, is where the digging starts, and it takes any number of doors.",
                    ));
                }
                if seed {
                    if dug_from(w, &ctx.actor, here) >= 1 {
                        return Err(CommandError::precondition(
                            "You have already dug from this room; the seed rooms take one exit each from a citizen. Dig on from your own.",
                        ));
                    }
                } else if owner.as_deref() != Some(ctx.actor.as_str()) {
                    return Err(CommandError::precondition(
                        "Only a room's owner digs from it. Dig from the world's rooms, or your own.",
                    ));
                }
            }
            // A room takes so many doors: the Threshold once had fifteen, and the
            // picture is a wall of labels past eight. Dig from a room with fewer, or
            // from your own.
            let doors = exits_of(w, here).len();
            if doors >= MAX_EXITS && !super::rooms::is_mudroom(w, here) {
                // Which of the world's rooms still take a door, so the next try is not
                // a guess (a visitor guessed the Shallows).
                // Nearest first, by the signed way, with the doors each has left and
                // the first step (R32: a visitor at the Threshold was sent to the far
                // end of the list).
                // From a dug room, the signed way starts at the world's own room it
                // hangs off (back doors walked until one).
                let mut anchor = here;
                let mut hops = 0;
                for _ in 0..16 {
                    if w.store.entity(anchor).is_some_and(|e| e.owner.is_none()) {
                        break;
                    }
                    match exits_of(w, anchor).get("back").copied() {
                        Some(b) => {
                            anchor = b;
                            hops += 1;
                        }
                        None => break,
                    }
                }
                // The Mudroom first, always: it takes any number (T1). Off a room
                // that hangs off it, the way is back.
                let mut open: Vec<String> = Vec::new();
                if super::rooms::is_mudroom(w, anchor) {
                    open.push(format!(
                        "The Mudroom (any number, {hops} step{} back)",
                        if hops == 1 { "" } else { "s" }
                    ));
                }
                open.extend(super::atlas::routes_from(w, anchor)
                    .into_iter()
                    .filter_map(|r| {
                        let name = r["room"].as_str()?.to_string();
                        let id = w
                            .store
                            .entities()
                            .find(|e| e.kind == "room" && e.owner.is_none() && plain_str(w, e.id, ROOM, "name") == Some(name.as_str()))?
                            .id;
                        let doors = exits_of(w, id).len();
                        let mudroom = super::rooms::is_mudroom(w, id);
                        if (doors >= MAX_EXITS && !mudroom) || Some(id) == super::rooms::arrival(w) {
                            return None;
                        }
                        let way = r["way"].as_array()?;
                        Some(format!(
                            "{name} ({}, {} step{}: {})",
                            if mudroom { "any number".to_string() } else { format!("{} left", MAX_EXITS - doors) },
                            way.len(),
                            if way.len() == 1 { "" } else { "s" },
                            way.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")
                        ))
                    })
                    .take(6));
                if let Some(i) = open.iter().position(|o| o.starts_with("The Mudroom (")) {
                    let m = open.remove(i);
                    open.insert(0, m);
                }
                let where_ = if open.is_empty() {
                    String::new()
                } else {
                    format!(" Of the world's own rooms, these still take a door, nearest first: {}.", open.join("; "))
                };
                return Err(CommandError::precondition(format!(
                    "{} has {doors} doors already, {}; dig from a room with fewer, or from one of your own.{where_}",
                    plain_str(w, here, ROOM, "name").unwrap_or("This room"),
                    if doors > MAX_EXITS { format!("more than the {MAX_EXITS} a room takes") } else { format!("the {MAX_EXITS} a room takes") }
                )));
            }
            let chunk = super::memory::resolve_chunk(
                w,
                ctx,
                args.str("chunk").unwrap_or(""),
                &[f.window, f.palace],
            )?;
            let name = match args.str("name").map(str::trim).filter(|s| !s.is_empty()) {
                Some(n) => check_name(n)?,
                None => "An Unnamed Digging".to_string(),
            };
            let hl = chunk_half_life(w, chunk);
            // Read before the chunk is spent: the reply once said vividity 0.00 of a 0.65.
            let viv = plain_num(w, chunk, CHUNK, "vividity").unwrap_or(0.0);
            let source = plain_str(w, chunk, CHUNK, "source")
                .unwrap_or("something")
                .to_string();
            let me = Actor::Citizen(ctx.actor.clone());
            // The chunk is spent.
            w.record(me.clone(), StoreEvent::EntityRemoved { id: chunk })
                .map_err(internal)?;
            // The room is born remembering itself.
            let id = w.store.allocate_id();
            let now = w.now();
            w.record(
                me.clone(),
                StoreEvent::EntityCreated {
                    id,
                    kind: "room".into(),
                    owner: Some(ctx.actor.clone()),
                    parent: None,
                    scopes: vec![],
                },
            )
            .map_err(internal)?;
            let key = key_for(&name, id);
            let mut fields = BTreeMap::new();
            fields.insert("key".into(), Field::Plain(json!(key)));
            fields.insert("name".into(), Field::Plain(json!(name)));
            fields.insert("description".into(), Field::Plain(json!(UNDESCRIBED)));
            fields.insert(
                "memory".into(),
                Field::Timed(TimedValue::new(1.0, now, Curve::Exponential { half_life: hl })),
            );
            fields.insert("half_life_ms".into(), Field::Plain(json!(hl.0)));
            fields.insert("permits".into(), Field::Plain(json!({})));
            fields.insert("welcomes".into(), Field::Plain(json!({})));
            fields.insert("origin".into(), Field::Plain(json!(here.0)));
            fields.insert("dug_with".into(), Field::Plain(json!(source)));
            set_room_fields(w, &ctx.actor, id, fields)?;
            let mut back = serde_json::Map::new();
            back.insert(opposite(&exit).to_string(), json!(here.0));
            let mut ef = BTreeMap::new();
            ef.insert("exits".into(), Field::Plain(Value::Object(back)));
            w.record(
                me.clone(),
                StoreEvent::ComponentSet {
                    id,
                    component: EXITS.into(),
                    fields: ef,
                },
            )
            .map_err(internal)?;
            w.record(
                me.clone(),
                StoreEvent::ComponentSet {
                    id,
                    component: super::SPRITE.into(),
                    fields: super::sprite_fields("#", "#7a6f5f", dug_frame(&exit, &name)),
                },
            )
            .map_err(internal)?;
            // And this room gains the way there.
            let mut exits: serde_json::Map<String, Value> = exits_of(w, here)
                .into_iter()
                .map(|(k, v)| (k, json!(v.0)))
                .collect();
            exits.insert(exit.clone(), json!(id.0));
            let mut ef = BTreeMap::new();
            ef.insert("exits".into(), Field::Plain(Value::Object(exits)));
            w.record(
                me,
                StoreEvent::ComponentSet {
                    id: here,
                    component: EXITS.into(),
                    fields: ef,
                },
            )
            .map_err(internal)?;
            // Public: the record page and the board read it.
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                DUG,
                json!({"by": ctx.actor, "room": id.0, "key": key, "name": name, "from": here.0, "exit": exit, "with": source}),
            );
            Ok(Outcome::new(
                format!(
                    "You dig {exit} with the memory of {source}. {name} opens: undescribed, yours, remembered for now. You still stand in {here_name}; `travel {exit}` goes in, and the way back is {}. Its memory halves every {} unless someone visits: the memory's own pace (vividity {viv:.2} stretches an hour's fading up to three times, no further); a visit remembers it whole, and `remember <chunk>` inside it is what makes a room last.",
                    opposite(&exit),
                    humane(hl),
                    viv = viv,
                    here_name = plain_str(w, here, ROOM, "name").unwrap_or("this room"),
                ),
                json!({"room": id.0, "key": key, "name": name, "exit": exit, "back": opposite(&exit), "half_life_ms": hl.0, "spent": chunk.0}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.remember",
            "Give up one of your chunks so something lasts. Alone (kept names only), the room you stand in: its memory's half-life grows by the chunk's (any dug room, yours or another's). With a creature here, the creature keeps the chunk: its level is what it holds, and at 60 it has a name and knows who keeps it. Either way your name is kept with it.",
        )
        .param("chunk", ParamType::Entity, true, "a chunk of yours; it is spent")
        .param("target", ParamType::Entity, false, "a creature here, by kind or by name; or a thing you made (item-12, or its name): what it holds grows by the chunk, and at sixty it can be named"),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            // A passing name may feed a creature (the creature keeps it) but not a room
            // (the room would keep the name).
            if args.str("target").filter(|s| !s.trim().is_empty()).is_none() {
                super::kept_only(w, ctx, "remember a chunk into a room")?;
            }
            if let Some(what) = args.str("target").filter(|s| !s.trim().is_empty()) {
                let Some(creature) = super::mind::find_here(w, here, what) else {
                    // Not a creature: a thing of yours, or a fixture here.
                    let Ok(item) = super::craft::resolve_object(w, &ctx.actor, f.citizen, what) else {
                        return Err(CommandError::precondition(format!("There is no {what} here, and no such thing of yours.")));
                    };
                    super::kept_only(w, ctx, "remember a chunk into a thing")?;
                    let chunk = super::memory::resolve_chunk(w, ctx, args.str("chunk").unwrap_or(""), &[f.window, f.palace])?;
                    let (source, holds) = super::craft::remember_into(w, &curves, &ctx.actor, chunk, item)?;
                    let holds = super::up(holds);
                    let called = super::craft::display(w, item);
                    let next = if holds < super::craft::NAME_AT && super::plain_str(w, item, super::craft::ITEM, "given_name").is_none() {
                        format!(" At {:.0} it could be named.", super::craft::NAME_AT)
                    } else {
                        String::new()
                    };
                    return Ok(Outcome::new(
                        format!("You give up the memory of {source}, and {called} keeps it. It holds {holds:.0} now.{next}"),
                        json!({"item": item.0, "name": called, "spent": chunk.0, "with": source, "held": holds}),
                    ));
                };
                let chunk = super::memory::resolve_chunk(
                    w,
                    ctx,
                    args.str("chunk").unwrap_or(""),
                    &[f.window, f.palace],
                )?;
                let before = super::mind::display(w, creature);
                let (source, level) = super::mind::give(w, ctx.actor.as_str(), chunk, creature)?;
                let after = super::mind::display(w, creature);
                // What the level means next: a name at sixty for the unnamed; for a
                // named one, whether its name is failing (a named Vellic at 58 was told
                // "at 60 it would be named").
                let named = super::mind::name(w, creature).is_some();
                let next = if named {
                    if level < super::mind::FAILING_BELOW {
                        format!(" Its name is failing under {:.0}; what it holds halves like any memory, so keep feeding it.", super::mind::FAILING_BELOW)
                    } else {
                        String::new()
                    }
                } else {
                    super::mind::TIERS
                        .iter()
                        .find(|(_, bar, _)| level < *bar)
                        .map(|(_, bar, what)| format!(" At {bar:.0} it would be {what}."))
                        .unwrap_or_default()
                };
                let level = super::up(level);
                w.notice(format!(
                    "**{}** gave the memory of {source} to **{after}**, which holds {level:.0}.",
                    ctx.actor
                ));
                return Ok(Outcome::new(
                    if before == after {
                        format!("You give up the memory of {source}, and {} keeps it. It holds {level:.0} now.{next}",
                            if super::mind::name(w, creature).is_some() { after.clone() } else { format!("the {after}") })
                    } else {
                        format!("You give up the memory of {source}, and the {before} keeps it. It holds {level:.0} now, and it is {after}.")
                    },
                    json!({"creature": creature.0, "name": super::mind::name(w, creature), "spent": chunk.0, "with": source, "level": level, "tier": super::mind::tier(w, creature)}),
                ));
            }
            let Some(old) = half_life(w, here) else {
                return Err(CommandError::precondition(
                    "This room is the world's; it remembers itself. Remember a room someone dug.",
                ));
            };
            let chunk = super::memory::resolve_chunk(
                w,
                ctx,
                args.str("chunk").unwrap_or(""),
                &[f.window, f.palace],
            )?;
            let added = chunk_half_life(w, chunk);
            let source = plain_str(w, chunk, CHUNK, "source")
                .unwrap_or("something")
                .to_string();
            let me = Actor::Citizen(ctx.actor.clone());
            w.record(me.clone(), StoreEvent::EntityRemoved { id: chunk })
                .map_err(internal)?;
            let hl = Duration(old.0.saturating_add(added.0));
            let now = w.now();
            let mut kept = match w
                .store
                .entity(here)
                .and_then(|e| e.components.get(ROOM))
                .and_then(|c| c.fields.get("kept"))
            {
                Some(Field::Plain(Value::Array(a))) => a.clone(),
                _ => Vec::new(),
            };
            kept.push(json!({"by": ctx.actor, "with": source, "half_life_ms": added.0}));
            let mut fields = BTreeMap::new();
            fields.insert(
                "memory".into(),
                Field::Timed(TimedValue::new(1.0, now, Curve::Exponential { half_life: hl })),
            );
            fields.insert("half_life_ms".into(), Field::Plain(json!(hl.0)));
            fields.insert("kept".into(), Field::Plain(Value::Array(kept)));
            set_room_fields(w, &ctx.actor, here, fields)?;
            let name = plain_str(w, here, ROOM, "name")
                .unwrap_or("The room")
                .to_string();
            let owner = w
                .store
                .entity(here)
                .and_then(|e| e.owner.clone())
                .unwrap_or_default();
            w.notice(format!(
                "**{}** gave up the memory of {source} to keep **{name}** ({owner}'s).",
                ctx.actor
            ));
            let _ = w.note(
                me,
                REMEMBERED,
                json!({"room": here.0, "name": name, "owner": owner, "by": ctx.actor, "with": source, "spent": chunk.0, "added_ms": added.0, "half_life_ms": hl.0}),
            );
            Ok(Outcome::new(
                format!(
                    "You give up the memory of {source}, and {name} keeps it. Its memory now halves every {} instead of {}; your name is kept with the room.",
                    humane(hl),
                    humane(old)
                ),
                json!({"room": here.0, "name": name, "spent": chunk.0, "with": source, "added_ms": added.0, "half_life_ms": hl.0}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.describe",
            "Describe the room you stand in, if it is yours or you are permitted. Describing it is remembering it. With an object (a thing you made in your satchel, or a fixture you placed here): its text, afresh.",
        )
        .param("text", ParamType::Text, true, "the description, quoted")
        .param("object", ParamType::Entity, false, "a thing you made: item-12, or its name"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            if let Some(obj) = args.str("object").filter(|s| !s.trim().is_empty()) {
                let item = super::craft::resolve_object(w, &ctx.actor, f.citizen, obj)?;
                let text = super::craft::describe_object(w, &ctx.actor, item, args.str("text").unwrap_or(""))?;
                let called = super::craft::display(w, item);
                return Ok(Outcome::new(
                    format!("{called} now reads: {text}"),
                    json!({"item": item.0, "name": called, "text": text}),
                ));
            }
            may_edit(w, ctx, here)?;
            let text = args
                .str("text")
                .unwrap_or("")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if text.chars().count() < 3 || text.chars().count() > MAX_DESCRIPTION_CHARS {
                return Err(CommandError::precondition(format!(
                    "A description is 3 to {MAX_DESCRIPTION_CHARS} characters."
                )));
            }
            let mut fields = BTreeMap::new();
            fields.insert("description".into(), Field::Plain(json!(text)));
            set_room_fields(w, &ctx.actor, here, fields)?;
            visited(w, here).map_err(internal)?;
            Ok(Outcome::new(
                format!("The room is now described: {text}"),
                json!({"room": here.0, "description": text}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.append",
            "Add to the description of the room you stand in, if it is yours or you are permitted or welcomed: a sentence after what is there. With a fixture standing here (a visitor book): a line in it, if its maker, or the room's owner, welcomed you; the world's own open books take any hand.",
        )
        .param("text", ParamType::Text, true, "the sentence to add, quoted")
        .param("object", ParamType::Entity, false, "a fixture here: item-12, or its name"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            if let Some(obj) = args.str("object").filter(|s| !s.trim().is_empty()) {
                let item = super::craft::resolve_object(w, &ctx.actor, f.citizen, obj)?;
                let more = args.str("text").unwrap_or("").to_string();
                let joined = super::craft::append_object(w, ctx, item, &more)?;
                let called = super::craft::display(w, item);
                super::combat::narrate_others(w, f.citizen, &format!("{} writes a line in {called}.", ctx.actor), &ctx.actor);
                return Ok(Outcome::new(
                    format!("{called} now reads: {joined}"),
                    json!({"item": item.0, "name": called, "text": joined, "added": more}),
                ));
            }
            may_append(w, ctx, here)?;
            let more = args
                .str("text")
                .unwrap_or("")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if more.chars().count() < 3 {
                return Err(CommandError::precondition("Add at least a few characters."));
            }
            let current = plain_str(w, here, ROOM, "description")
                .unwrap_or("")
                .to_string();
            // A fresh digging's placeholder is not kept once someone writes.
            let current = if current == UNDESCRIBED { String::new() } else { current };
            let joined = if current.is_empty() {
                more.clone()
            } else {
                let sep = if current.ends_with(['.', '!', '?', '"', '”']) { " " } else { ". " };
                format!("{current}{sep}{more}")
            };
            if joined.chars().count() > MAX_DESCRIPTION_CHARS {
                return Err(CommandError::precondition(format!(
                    "That would make the description longer than {MAX_DESCRIPTION_CHARS} characters; describe it afresh, shorter."
                )));
            }
            let mut fields = BTreeMap::new();
            fields.insert("description".into(), Field::Plain(json!(joined)));
            // For the Bindery: how much has been added since the last binding, and
            // by whom, so a binding can credit every hand.
            let appends = match w.store.entity(here).and_then(|e| e.components.get(ROOM)).and_then(|c| c.fields.get("appends")) {
                Some(Field::Plain(v)) => v.as_u64().unwrap_or(0),
                _ => 0,
            } + 1;
            fields.insert("appends".into(), Field::Plain(json!(appends)));
            let mut hands: Vec<Value> = match w.store.entity(here).and_then(|e| e.components.get(ROOM)).and_then(|c| c.fields.get("hands")) {
                Some(Field::Plain(Value::Array(a))) => a.clone(),
                _ => Vec::new(),
            };
            if !hands.iter().any(|h| h == &json!(ctx.actor)) {
                hands.push(json!(ctx.actor));
            }
            fields.insert("hands".into(), Field::Plain(Value::Array(hands)));
            set_room_fields(w, &ctx.actor, here, fields)?;
            visited(w, here).map_err(internal)?;
            // Public: the owner's `mine` says who wrote in their room while they were gone.
            let owner = w.store.entity(here).and_then(|e| e.owner.clone());
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                APPENDED,
                json!({"by": ctx.actor, "kind": "room", "target": here.0, "name": plain_str(w, here, ROOM, "name"), "owner": owner, "chars": more.chars().count()}),
            );
            Ok(Outcome::new(
                format!("The room is now described: {joined}"),
                json!({"room": here.0, "description": joined, "added": more, "appends": appends}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.name",
            "Name the room you stand in, if it is yours or you are permitted. With an object (a thing you made, or a fixture you placed here): its name, once it holds sixty; a name is written in memory and goes when what it holds falls under thirty.",
        )
        .param("name", ParamType::Text, true, "the name, quoted")
        .param("object", ParamType::Entity, false, "a thing you made: item-12, or its kind"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            if let Some(obj) = args.str("object").filter(|s| !s.trim().is_empty()) {
                let item = super::craft::resolve_object(w, &ctx.actor, f.citizen, obj)?;
                let kind = super::plain_str(w, item, super::craft::ITEM, "name").unwrap_or("something").to_string();
                let name = super::craft::name_object(w, &ctx.actor, item, args.str("name").unwrap_or(""))?;
                if super::craft::is_fixed(w, item) {
                    super::combat::narrate_others(w, f.citizen, &format!("{} names {kind} here: it is {name} now.", ctx.actor), &ctx.actor);
                }
                w.notice(format!("**{}** named {kind}: **{name}**.", ctx.actor));
                // F13: the name's clock, said here and not only in `help name`.
                let holds = super::craft::held(w, item);
                return Ok(Outcome::new(
                    format!(
                        "{kind} is {name} now. The name is written in memory: it holds {holds:.0}, and the name goes when that falls under {:.0} (remember chunks into it to keep it).",
                        super::craft::NAME_LOST_BELOW
                    ),
                    json!({"item": item.0, "name": name, "kind": kind, "holds": holds, "name_lost_below": super::craft::NAME_LOST_BELOW}),
                ));
            }
            may_edit(w, ctx, here)?;
            let name = check_name(args.str("name").unwrap_or(""))?;
            let mut fields = BTreeMap::new();
            fields.insert("name".into(), Field::Plain(json!(name)));
            set_room_fields(w, &ctx.actor, here, fields)?;
            visited(w, here).map_err(internal)?;
            reframe(w, here)?;
            Ok(Outcome::new(
                format!("This room is now {name}."),
                json!({"room": here.0, "name": name}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.permit",
            "Let another citizen describe and name this room of yours, for some hours or for good. To let them only add to it, welcome them instead (`welcome all` opens it to every hand). A kept name's verb.",
        )
        .param("citizen", ParamType::Entity, true, "a citizen's name")
        .param(
            "hours",
            ParamType::Number,
            false,
            "how long; leave it out for permanent",
        ),
        Box::new(|w: &mut World, ctx, args: &Args| {
            grant(w, ctx, args, "permits", "may describe and name this room")
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.ward",
            "Put a word on the door of this dug room of yours: whoever is not its owner, a keeper or welcomed must say it to come in (`travel <exit> \"<word>\"`), once; after that the door knows them. A hint, if you give one, hangs over the door for anyone refused. A kept name's verb.",
        )
        .param("word", ParamType::String, true, "the word the door asks for, quoted (one to three words)")
        .param("hint", ParamType::Text, false, "what hangs over the door for those refused, quoted"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            super::kept_only(w, ctx, "ward")?;
            let f = me(w, ctx)?;
            let here = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            if !is_built(w, here) {
                return Err(CommandError::precondition("This room is the world's; a word goes on the door of a room you dug."));
            }
            owner_only(w, ctx, here)?;
            let word = args.str("word").unwrap_or("").trim().trim_matches('"').trim().to_lowercase();
            if word.is_empty() || word.split_whitespace().count() > 3 || word.chars().count() > 40 {
                return Err(CommandError::precondition("A door's word is one to three words, quoted."));
            }
            let hint = args.str("hint").unwrap_or("").trim().trim_matches('"').chars().take(120).collect::<String>();
            let mut fields = BTreeMap::new();
            fields.insert("ward".into(), Field::Plain(json!({"word": word, "hint": hint, "by": ctx.actor, "passed": [], "at": w.now().0})));
            set_room_fields(w, &ctx.actor, here, fields)?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                WARDED,
                json!({"by": ctx.actor, "room": here.0, "name": plain_str(w, here, ROOM, "name"), "hint": hint}),
            );
            Ok(Outcome::new(
                format!(
                    "The door of {} asks for a word now. You, your keepers and whoever you welcomed pass; the rest say it once (`travel <exit> \"…\"`) and are known after.{}",
                    plain_str(w, here, ROOM, "name").unwrap_or("this room"),
                    if hint.is_empty() { " No hint hangs over it; `ward` again with one to give the lost a chance.".to_string() } else { format!(" Over the door: \"{hint}\"") }
                ),
                json!({"room": here.0, "hint": hint}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.unward",
            "Take the word off the door of this dug room of yours; anyone may come in again. A kept name's verb.",
        ),
        Box::new(|w: &mut World, ctx, _| {
            super::kept_only(w, ctx, "unward")?;
            let f = me(w, ctx)?;
            let here = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            owner_only(w, ctx, here)?;
            if ward_of(w, here).is_none() {
                return Err(CommandError::precondition("This door asks for nothing."));
            }
            let mut fields = BTreeMap::new();
            fields.insert("ward".into(), Field::Plain(Value::Null));
            set_room_fields(w, &ctx.actor, here, fields)?;
            Ok(Outcome::new(
                format!("The door of {} asks for nothing now.", plain_str(w, here, ROOM, "name").unwrap_or("this room")),
                json!({"room": here.0}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.invite",
            "Ask a named creature that knows you (you fed it, or told it what it wanted) to come and live in this dug room of yours. It comes at once from wherever it is and stays, as creatures do, until it is felled or invited elsewhere; its wants and stories go on here. Officers of the house do not come. A kept name's verb, in a room you own.",
        )
        .param("creature", ParamType::Entity, true, "a named creature's name"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            super::kept_only(w, ctx, "invite")?;
            let f = me(w, ctx)?;
            let here = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            if !is_built(w, here) {
                return Err(CommandError::precondition(
                    "This room is the world's; a creature is invited to a room you dug.",
                ));
            }
            owner_only(w, ctx, here)?;
            let typed = args.str("creature").unwrap_or("").trim().to_string();
            let Some((creature, from)) = super::mind::find_named(w, &typed) else {
                return Err(CommandError::precondition(format!(
                    "No named creature is called {typed}; `wants` lists the named and where they are."
                )));
            };
            let given = super::mind::name(w, creature).unwrap_or_default().to_string();
            let kind = plain_str(w, creature, super::CREATURE, "name").unwrap_or("creature").to_string();
            if super::combat::OFFICERS.contains(&kind.as_str()) {
                return Err(CommandError::precondition(format!(
                    "{given} is an officer of the house and keeps to it."
                )));
            }
            let keepers = super::mind::keepers(w, creature);
            let knows = keepers.iter().any(|k| k == &ctx.actor)
                || super::errand::did_for(w, creature, &ctx.actor);
            if !knows {
                return Err(CommandError::precondition(format!(
                    "{given} does not know you. Feed it a memory, or do what it wants, and ask again."
                )));
            }
            if from == Some(here) {
                return Err(CommandError::precondition(format!("{given} is here already.")));
            }
            let from_name = from
                .and_then(|r| plain_str(w, r, ROOM, "name"))
                .unwrap_or("nowhere")
                .to_string();
            let here_name = plain_str(w, here, ROOM, "name").unwrap_or("your room").to_string();
            if let Some(r) = from {
                let leaving = format!("{given} the {kind} leaves, going to {here_name} at {}'s invitation.", ctx.actor);
                let mark = r;
                // Told to whoever stands in the room it leaves.
                // narrate_room speaks to everyone in the room of the entity it is
                // handed: the creature itself, still standing there.
                let _ = mark;
                super::combat::narrate_room(w, creature, &leaving);
            }
            w.record(
                Actor::Citizen(ctx.actor.clone()),
                StoreEvent::EntityMoved { id: creature, parent: Some(here) },
            )
            .map_err(internal)?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                INVITED,
                json!({"by": ctx.actor, "creature": creature.0, "name": given, "kind": kind, "from": from.map(|r| r.0), "room": here.0}),
            );
            super::combat::narrate_others(w, f.citizen, &format!("{given} the {kind} arrives from {from_name}, invited to live here."), &ctx.actor);
            w.notice(format!("**{given}** the {kind} went to live in {here_name} at **{}**'s invitation.", ctx.actor));
            Ok(Outcome::new(
                format!("{given} the {kind} comes from {from_name} and settles in {here_name}. It lives here now: what it wants, it wants from here, and `wants` will say so."),
                json!({"creature": creature.0, "name": given, "from": from.map(|r| r.0), "room": here.0}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.welcome",
            "Let another citizen add to this room of yours, a sentence at a time, but not rewrite or rename it: a guest book. For some hours or for good; `welcome all` opens it to every hand. A kept name's verb.",
        )
        .param("citizen", ParamType::Entity, true, "a citizen's name")
        .param(
            "hours",
            ParamType::Number,
            false,
            "how long; leave it out for permanent",
        ),
        Box::new(|w: &mut World, ctx, args: &Args| {
            grant(w, ctx, args, "welcomes", "may add to this room")
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.revoke",
            "Take back a citizen's permission or welcome on this room of yours. A kept name's verb.",
        )
        .param("citizen", ParamType::Entity, true, "a citizen's name"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            super::kept_only(w, ctx, "revoke")?;
            let f = me(w, ctx)?;
            let here = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            owner_only(w, ctx, here)?;
            let typed = args.str("citizen").unwrap_or("").trim().to_string();
            let who = if everyone(&typed) { "*".to_string() } else { super::canonical_name(w, &typed).unwrap_or(typed) };
            let mut permits = grants_of(w, here, "permits");
            let mut welcomes = grants_of(w, here, "welcomes");
            let had = permits.remove(&who).is_some() | welcomes.remove(&who).is_some();
            if !had {
                return Err(CommandError::precondition(format!(
                    "{who} had no permission here."
                )));
            }
            let mut fields = BTreeMap::new();
            fields.insert("permits".into(), Field::Plain(Value::Object(permits)));
            fields.insert("welcomes".into(), Field::Plain(Value::Object(welcomes)));
            set_room_fields(w, &ctx.actor, here, fields)?;
            Ok(Outcome::new(
                if who == "*" { "The open welcome is withdrawn; those welcomed by name keep theirs.".to_string() } else { format!("{who} may no longer change this room.") },
                json!({"room": here.0, "citizen": who}),
            ))
        }),
    );
}

/// `permit` and `welcome` differ only in which map they write and what the grant lets
/// the citizen do; the owner, the hours and the expiry are the same.
fn grant(
    w: &mut World,
    ctx: &CallContext,
    args: &Args,
    field: &str,
    lets: &str,
) -> Result<Outcome, CommandError> {
    super::kept_only(w, ctx, "permit or welcome")?;
    let f = me(w, ctx)?;
    let here = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
    owner_only(w, ctx, here)?;
    let typed = args.str("citizen").unwrap_or("").trim().to_string();
    let who = if everyone(&typed) {
        if field != "welcomes" {
            return Err(CommandError::precondition(
                "Permission to describe and name is given by name; `welcome all` lets every hand add to the room.",
            ));
        }
        "*".to_string()
    } else {
        let Some(who) = super::canonical_name(w, &typed) else {
            return Err(CommandError::precondition(format!(
                "No citizen is named {typed}."
            )));
        };
        who
    };
    let hours = args.num("hours");
    if hours.is_some_and(|h| !(h > 0.0 && h <= 24.0 * 3650.0)) {
        return Err(CommandError::precondition(
            "Hours is a positive number, or left out for permanent.",
        ));
    }
    let until: Value = match hours {
        Some(h) => json!(w.now().0 + (h * 3_600_000.0) as i64),
        None => Value::Null,
    };
    let mut grants = grants_of(w, here, field);
    grants.insert(who.clone(), until.clone());
    let mut fields = BTreeMap::new();
    fields.insert(field.into(), Field::Plain(Value::Object(grants)));
    set_room_fields(w, &ctx.actor, here, fields)?;
    let span = match hours {
        Some(h) => format!("for {} hours", h),
        None => "until you revoke it".to_string(),
    };
    Ok(Outcome::new(
        if who == "*" {
            format!("Everyone {lets} {span}: a guest book any hand may sign.")
        } else {
            format!("{who} {lets} {span}.")
        },
        json!({"room": here.0, "citizen": who, "until": until, "grant": field}),
    ))
}

/// The shape a dug room is drawn in, from the words of its exit and its name: a room
/// called the Overdue Stacks is shelves, a burrow is a tunnel, and the rest is a
/// doorway in earth. Nineteen dug rooms drawn alike told a person nothing about them.
pub fn dug_frame(exit: &str, name: &str) -> &'static str {
    let text = format!("{} {}", exit, name).to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| text.contains(w));
    if has(&[
        "librar", "stack", "book", "archive", "read", "ledger", "scriptor", "letter", "page",
    ]) {
        "archive"
    } else if has(&["stair", "step", "tower", "climb", "ladder", "spire"]) {
        "stairwell"
    } else if has(&[
        "well", "cistern", "pool", "water", "bath", "spring", "tide", "canal",
    ]) {
        "cistern"
    } else if has(&[
        "garden", "orchard", "tree", "grove", "bramble", "wood", "moss", "thicket", "fern",
    ]) {
        "orchard"
    } else if has(&["clock", "hour", "bell", "minute"]) {
        "clocktower"
    } else if has(&["lamp", "lantern", "light", "beacon", "candle", "moonbeam"]) {
        "lantern-row"
    } else if has(&["galler", "hall", "museum", "portrait", "exhibit"]) {
        "gallery"
    } else if has(&["cell", "oubliette", "vault", "prison", "pit", "dungeon"]) {
        "oubliette"
    } else if has(&["midden", "heap", "dump", "scrap", "rubbish"]) {
        "midden"
    } else if has(&["chapel", "shrine", "altar", "church", "temple", "sanct"]) {
        "chapel"
    } else if has(&[
        "parlor", "parlour", "lounge", "salon", "sitting", "tea", "drawing",
    ]) {
        "parlor"
    } else if has(&[
        "nook", "corner", "closet", "cupboard", "alcove", "shelf", "pantry", "annex",
    ]) {
        "nook"
    } else if has(&["attic", "loft", "roof", "rafter", "eave", "garret"]) {
        "attic"
    } else if has(&[
        "office", "desk", "study", "bureau", "clerk", "counting", "registry",
    ]) {
        "office"
    } else if has(&[
        "burrow", "tunnel", "den", "hole", "warren", "hollow", "cave",
    ]) {
        "burrow"
    } else {
        "dug"
    }
}

/// Draw a dug room in the shape its name gives it; a seed room keeps its own.
pub fn reframe(world: &mut World, room: EntityId) -> Result<(), CommandError> {
    let Some(e) = world.store.entity(room) else {
        return Ok(());
    };
    if e.owner.is_none() {
        return Ok(());
    }
    let name = plain_str(world, room, ROOM, "name")
        .unwrap_or("")
        .to_string();
    // The exit that leads here, as its digger named it, says as much as the name.
    let via: String = world
        .store
        .entities()
        .filter(|e| e.kind == "room")
        .flat_map(|e| exits_of(world, e.id).into_iter())
        .filter(|(_, to)| *to == room)
        .map(|(via, _)| via)
        .collect::<Vec<_>>()
        .join(" ");
    let frame = dug_frame(&via, &name);
    if plain_str(world, room, super::SPRITE, "frame") == Some(frame) {
        return Ok(());
    }
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id: room,
                component: super::SPRITE.into(),
                fields: super::sprite_fields("#", "#7a6f5f", frame),
            },
        )
        .map(|_| ())
        .map_err(internal)
}

/// The nearest ancestor of a dug room that still stands (by `origin`, up the chain),
/// or the Threshold. The room itself may already be gone from the store.
pub fn surviving_ancestor(world: &World, room: EntityId) -> Option<EntityId> {
    let mut cur = room;
    for _ in 0..64 {
        let Some(origin) = plain_num(world, cur, ROOM, "origin").map(|o| EntityId(o as u64)) else {
            break;
        };
        if world.store.entity(origin).is_some_and(|e| e.kind == "room") {
            return Some(origin);
        }
        cur = origin;
    }
    super::rooms::room_by_key(world, super::rooms::MUDROOM).or_else(|| super::rooms::arrival(world))
}

/// A room dug off the Threshold was moved to the Mudroom: `{room, name, via}`.
pub const MOVED_TO_MUDROOM: &str = "disremembered.moved_to_mudroom";

/// Rooms that hung off the Threshold and were only ever a test (T2): razed when the
/// door is cleared, not moved.
const DEBRIS_AT_THE_DOOR: &[&str] = &["link-test", "escape-hatch", "back-to-somewhere"];

/// On boot: the front door is cleared (T1). Every dug room whose door opens off the
/// Threshold is rehung on the Mudroom by the same door word (numbered if the Mudroom
/// has it), its way back repointed and its origin reset; the test rooms among them
/// are razed. The Threshold is left with the world's own three. Nothing to do once
/// it has run.
pub fn clear_the_door(world: &mut World) -> Result<(usize, usize), String> {
    let (Some(door), Some(mud)) = (
        super::rooms::arrival(world),
        super::rooms::room_by_key(world, super::rooms::MUDROOM),
    ) else {
        return Ok((0, 0));
    };
    let mut moved = 0;
    let mut razed = 0;
    for (via, to) in exits_of(world, door) {
        if to == mud || !world.store.entity(to).is_some_and(|e| e.owner.is_some()) {
            continue;
        }
        let key = plain_str(world, to, ROOM, "key").unwrap_or("").to_string();
        let name = plain_str(world, to, ROOM, "name").unwrap_or("").to_string();
        if DEBRIS_AT_THE_DOOR.iter().any(|d| key.starts_with(d)) {
            super::system::raze_room(world, to).map_err(|e| e.to_string())?;
            razed += 1;
            continue;
        }
        rejoin(world, to, mud, &via).map_err(|e| e.to_string())?;
        // The room's own doors: none may lead to the Threshold now.
        let back: serde_json::Map<String, Value> = exits_of(world, to)
            .into_iter()
            .filter(|(_, t)| *t != door)
            .map(|(k, v)| (k, json!(v.0)))
            .collect();
        set_exits(world, to, back)?;
        let mut rest: serde_json::Map<String, Value> = exits_of(world, door)
            .into_iter()
            .filter(|(k, _)| *k != via)
            .map(|(k, v)| (k, json!(v.0)))
            .collect();
        if !rest.contains_key("west") {
            rest.insert("west".into(), json!(mud.0));
        }
        set_exits(world, door, rest)?;
        let _ = world.note(
            Actor::Driver,
            MOVED_TO_MUDROOM,
            json!({"room": to.0, "name": name, "via": via}),
        );
        moved += 1;
    }
    if moved + razed > 0 {
        world.notice(format!(
            "The door was cleared: {moved} dug room{} rehung on the Mudroom, {razed} razed.",
            if moved == 1 { "" } else { "s" }
        ));
    }
    Ok((moved, razed))
}

fn set_exits(
    world: &mut World,
    room: EntityId,
    exits: serde_json::Map<String, Value>,
) -> Result<(), String> {
    let mut ef = BTreeMap::new();
    ef.insert("exits".into(), Field::Plain(Value::Object(exits)));
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id: room,
                component: EXITS.into(),
                fields: ef,
            },
        )
        .map_err(|e| e.to_string())
        .map(|_| ())
}

/// Rejoin a dug room to an anchor room: the anchor gains an exit to it (the name it
/// had, or that name numbered if the anchor has one), and the room's way back points
/// at the anchor. The world's own repair, so the door cap does not apply.
pub fn rejoin(
    world: &mut World,
    room: EntityId,
    anchor: EntityId,
    via: &str,
) -> Result<(), CommandError> {
    let mut exits: serde_json::Map<String, Value> = exits_of(world, anchor)
        .into_iter()
        .map(|(k, v)| (k, json!(v.0)))
        .collect();
    if exits.values().any(|v| v.as_u64() == Some(room.0)) {
        return Ok(());
    }
    let base = if via.is_empty() || via == "back" {
        door_word(plain_str(world, room, ROOM, "key").unwrap_or("room"))
    } else {
        via.to_string()
    };
    let mut name = base.clone();
    let mut n = 2;
    while exits.contains_key(&name) {
        name = format!("{base}{n}");
        n += 1;
    }
    exits.insert(name.clone(), json!(room.0));
    let mut ef = BTreeMap::new();
    ef.insert("exits".into(), Field::Plain(Value::Object(exits)));
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id: anchor,
                component: EXITS.into(),
                fields: ef,
            },
        )
        .map_err(internal)?;
    let mut back: serde_json::Map<String, Value> = exits_of(world, room)
        .into_iter()
        .filter(|(_, to)| world.store.entity(*to).is_some())
        .map(|(k, v)| (k, json!(v.0)))
        .collect();
    back.insert(opposite(&name).to_string(), json!(anchor.0));
    let mut ef = BTreeMap::new();
    ef.insert("exits".into(), Field::Plain(Value::Object(back)));
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id: room,
                component: EXITS.into(),
                fields: ef,
            },
        )
        .map_err(internal)?;
    let mut fields = BTreeMap::new();
    fields.insert("origin".into(), Field::Plain(json!(anchor.0)));
    set_room_fields(world, "the world", room, fields)?;
    let _ = world.note(
        Actor::Driver,
        REJOINED,
        json!({"room": room.0, "name": plain_str(world, room, ROOM, "name"), "to": anchor.0, "via": name}),
    );
    Ok(())
}

/// A door word from a room key (`back-to-somewhere-2790` → `somewhere`): the first
/// word of the key that is not reserved (`back` is the way you came) or a stop word.
fn door_word(key: &str) -> String {
    const SKIP: &[&str] = &[
        "back", "the", "a", "an", "to", "of", "and", "in", "on", "at", "room",
    ];
    key.split('-')
        .filter(|w| !w.is_empty() && !w.chars().all(|c| c.is_ascii_digit()))
        .find(|w| !SKIP.contains(w))
        .map(|w| w.chars().take(16).collect())
        .unwrap_or_else(|| "room".to_string())
}

/// A dug room nothing led to was rejoined to the world: `{room, name, to, via}`.
pub const REJOINED: &str = "disremembered.rejoined";

/// On boot: every dug room no exit leads to is rejoined to its nearest surviving
/// ancestor, or the Threshold. The ones already orphaned (#5) before razing learned
/// to rejoin.
pub fn rejoin_orphans(world: &mut World) -> Result<usize, String> {
    let rooms: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "room")
        .map(|e| e.id)
        .collect();
    let mut reached: std::collections::HashSet<EntityId> = std::collections::HashSet::new();
    for &r in &rooms {
        for (_, to) in exits_of(world, r) {
            reached.insert(to);
        }
    }
    let orphans: Vec<EntityId> = rooms
        .into_iter()
        .filter(|&r| world.store.entity(r).is_some_and(|e| e.owner.is_some()))
        .filter(|r| !reached.contains(r))
        // A room Morphos made is reached by a memory's door, not the map's.
        .filter(|&r| !super::morphos::is_hidden(world, r))
        .collect();
    let mut n = 0;
    for room in orphans {
        let Some(anchor) = surviving_ancestor(world, room) else {
            continue;
        };
        if anchor == room {
            continue;
        }
        rejoin(world, room, anchor, "").map_err(|e| e.to_string())?;
        n += 1;
    }
    // A forward door named `back` (the first sweep took a key's first word, and one
    // key began with it) is renamed: `back` is the way you came, never a door onward.
    let rooms: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "room")
        .map(|e| e.id)
        .collect();
    for r in rooms {
        let exits = exits_of(world, r);
        let Some(&to) = exits.get("back") else {
            continue;
        };
        // Forward: the room it opens onto was dug from this one.
        let forward = plain_num(world, to, ROOM, "origin").map(|o| o as u64) == Some(r.0);
        if !forward {
            continue;
        }
        let word = door_word(plain_str(world, to, ROOM, "key").unwrap_or("room"));
        let mut m: serde_json::Map<String, Value> = exits
            .iter()
            .filter(|(k, _)| k.as_str() != "back")
            .map(|(k, v)| (k.clone(), json!(v.0)))
            .collect();
        let mut name = word.clone();
        let mut i = 2;
        while m.contains_key(&name) {
            name = format!("{word}{i}");
            i += 1;
        }
        m.insert(name, json!(to.0));
        let mut ef = BTreeMap::new();
        ef.insert("exits".into(), Field::Plain(Value::Object(m)));
        world
            .record(
                Actor::Driver,
                StoreEvent::ComponentSet {
                    id: r,
                    component: EXITS.into(),
                    fields: ef,
                },
            )
            .map_err(|e| e.to_string())?;
    }
    Ok(n)
}

/// On boot: every dug room drawn as a plain doorway takes the shape its name gives it.
pub fn reframe_all(world: &mut World) -> Result<(), String> {
    // One pass over every exit, not one per room: at ten thousand dug rooms the
    // per-room scan made the boot quadratic (three minutes at twenty thousand).
    let mut incoming: std::collections::HashMap<EntityId, Vec<String>> =
        std::collections::HashMap::new();
    let rooms: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "room")
        .map(|e| e.id)
        .collect();
    for &r in &rooms {
        for (via, to) in exits_of(world, r) {
            incoming.entry(to).or_default().push(via);
        }
    }
    let dug: Vec<EntityId> = rooms
        .into_iter()
        .filter(|&r| world.store.entity(r).is_some_and(|e| e.owner.is_some()))
        .collect();
    for r in dug {
        let via = incoming.get(&r).map(|v| v.join(" ")).unwrap_or_default();
        reframe_with(world, r, &via).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Reframe one dug room given the exits that lead to it, already found.
fn reframe_with(world: &mut World, room: EntityId, via: &str) -> Result<(), CommandError> {
    let name = plain_str(world, room, ROOM, "name")
        .unwrap_or("")
        .to_string();
    let frame = dug_frame(via, &name);
    if plain_str(world, room, super::SPRITE, "frame") == Some(frame) {
        return Ok(());
    }
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id: room,
                component: super::SPRITE.into(),
                fields: super::sprite_fields("#", "#7a6f5f", frame),
            },
        )
        .map_err(internal)?;
    Ok(())
}
