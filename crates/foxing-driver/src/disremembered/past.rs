//! A thing with a past.
//!
//! Anyone can craft a useful thing (tier 1: the recipes as they were). It takes
//! someone special to craft a thing with a past (tier 2): a recipe that asks a rare
//! material and a vivid memory, or luck, one build in forty. A thing of the third
//! tier is lightly sentient: it has a name dealt by the Namery, a past, and desires
//! it communicates by other means than speech (it pulls in the hand toward what it
//! wants; it goes still and warm when it has it). Tier 3 comes only by chance, one
//! build in forty of a tier-2 thing; the vocabulary says nothing of it.
//!
//! Every made thing keeps a record of its life (`past`): made of what and with which
//! memory, who remembered into it, whose hands it passed through and where, what it
//! did (a bite closed, a memory made vivid), what it was named, what it wanted and
//! got. `read <thing>` recites it. A thing of tier 2 or 3 also carries `past_text`,
//! a passage in the world's voice the Bindery writes from the record (`past_wanted`
//! marks one whose record has grown since it was last written; `GET /v0/bindery`
//! lists them, and `bind item-N "…"` applies the passage under the same mechanical
//! checks a binding gets).

use super::craft::{self, ITEM};
use super::{internal, plain_num, plain_str, Curves, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::CommandError;
use foxing_store::{EntityId, Field};
use foxing_time::TimedValue;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// One build in this many comes out a tier higher than its recipe.
pub const TIER_UP_ONE_IN: u32 = 40;
/// The record keeps this many entries; the oldest fold into a count.
pub const PAST_MAX: usize = 60;
/// What a tier-3 thing's thanks are worth, as a memory, when its desire is met.
pub const THANKS_XP: f64 = 20.0;
/// A tier-3 thing wants nothing for this long after a desire is met.
pub const DESIRE_REST_MS: i64 = 60 * 60 * 1000;
/// A tier-3 thing was carried to what it wanted: `{item, name, by, room}`. Public.
pub const SETTLED: &str = "disremembered.settled";
/// A thing came out of the bench a tier above its recipe: `{item, name, by, tier}`. Public.
pub const RAISED: &str = "disremembered.raised";

/// A thing's tier: 1 for a made thing with none recorded, 0 for a material.
pub fn tier(world: &World, item: EntityId) -> u8 {
    match plain_num(world, item, ITEM, "tier") {
        Some(t) => t as u8,
        None if plain_str(world, item, ITEM, "kind") == Some("made") => 1,
        None => 0,
    }
}

fn past_of(world: &World, item: EntityId) -> Vec<Value> {
    world
        .store
        .entity(item)
        .and_then(|e| e.components.get(ITEM))
        .and_then(|c| c.fields.get("past"))
        .and_then(|f| match f {
            Field::Plain(Value::Array(a)) => Some(a.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn set(
    world: &mut World,
    actor: &str,
    item: EntityId,
    changes: BTreeMap<String, Field>,
) -> Result<(), CommandError> {
    craft::set_item_fields(world, Actor::Citizen(actor.to_string()), item, changes)
}

fn room_name(world: &World, room: Option<EntityId>) -> String {
    room.and_then(|r| plain_str(world, r, ROOM, "name"))
        .unwrap_or("nowhere")
        .to_string()
}

/// Append one entry to a thing's record. A notable one (anything past being carried
/// about) marks a tier-2 thing as wanting its passage written again.
pub fn push(
    world: &mut World,
    actor: &str,
    item: EntityId,
    mut event: Value,
) -> Result<(), CommandError> {
    if world.store.entity(item).map_or(true, |e| e.kind != "item") {
        return Ok(());
    }
    event["at"] = json!(world.now().0);
    let mut past = past_of(world, item);
    past.push(event.clone());
    let mut folded = plain_num(world, item, ITEM, "past_folded").unwrap_or(0.0) as u64;
    while past.len() > PAST_MAX {
        past.remove(0);
        folded += 1;
    }
    let mut changes = BTreeMap::new();
    changes.insert("past".into(), Field::Plain(Value::Array(past)));
    if folded > 0 {
        changes.insert("past_folded".into(), Field::Plain(json!(folded)));
    }
    let notable = !matches!(
        event["what"].as_str(),
        Some("carried") | Some("took") | Some("dropped")
    );
    if notable && tier(world, item) >= 2 {
        changes.insert("past_wanted".into(), Field::Plain(json!(true)));
    }
    set(world, actor, item, changes)
}

/// The record as prose, one sentence an entry, consecutive carryings by one hand run
/// together ("Carried by Wren into the Fogbank, the Shallows and the Orchard").
pub fn recital(world: &World, item: EntityId) -> String {
    let past = past_of(world, item);
    let folded = plain_num(world, item, ITEM, "past_folded").unwrap_or(0.0) as u64;
    let mut out: Vec<String> = Vec::new();
    if folded > 0 {
        out.push(format!(
            "Before what is written here, {folded} thing{} the record no longer holds.",
            if folded == 1 { "" } else { "s" }
        ));
    }
    let s = |v: &Value, k: &str| v[k].as_str().unwrap_or("").to_string();
    let mut i = 0;
    while i < past.len() {
        let e = &past[i];
        let by = s(e, "by");
        let room = s(e, "room");
        match e["what"].as_str().unwrap_or("") {
            "made" => {
                let of = e["of"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(" and ")
                    })
                    .unwrap_or_default();
                let with = s(e, "with");
                let vivid = s(e, "vivid");
                out.push(format!(
                    "Made by {by} in {room}{}{}.",
                    if of.is_empty() {
                        String::new()
                    } else {
                        format!(" of {of}")
                    },
                    if with.is_empty() {
                        String::new()
                    } else {
                        format!(
                            ", with {} memory of {with}",
                            if vivid.is_empty() {
                                "the".to_string()
                            } else {
                                format!("a {vivid}")
                            }
                        )
                    }
                ));
            }
            "left" => out.push(format!(
                "Left by a {} that came apart in {room}.",
                s(e, "kind")
            )),
            "raised" => out.push(match e["tier"].as_u64() {
                Some(3) => format!(
                    "Something in the making took, and it woke: it is {}, and it wants things.",
                    s(e, "name")
                ),
                _ => "Something in the making took, and it came out with a past.".to_string(),
            }),
            "took" => out.push(format!("Taken up by {by} in {room}.")),
            "dropped" => out.push(format!("Set down by {by} in {room}.")),
            "carried" => {
                let mut rooms = vec![room.clone()];
                while i + 1 < past.len()
                    && past[i + 1]["what"] == "carried"
                    && s(&past[i + 1], "by") == by
                {
                    i += 1;
                    rooms.push(s(&past[i], "room"));
                }
                let list = match rooms.len() {
                    1 => rooms[0].clone(),
                    n => format!("{} and {}", rooms[..n - 1].join(", "), rooms[n - 1]),
                };
                out.push(format!("Carried by {by} into {list}."));
            }
            "kept" => out.push(format!("{by} remembered {} into it.", s(e, "with"))),
            "did" => out.push(format!("{} for {by} in {room}.", s(e, "deed"))),
            "named" => out.push(format!("{by} named it {}.", s(e, "name"))),
            "described" => out.push(format!("{by} wrote on it.")),
            "settled" => out.push(format!(
                "It wanted {}, and {by} carried it there.",
                s(e, "room")
            )),
            _ => {}
        }
        i += 1;
    }
    if out.is_empty() {
        "Nothing is recorded of it yet.".to_string()
    } else {
        out.join(" ")
    }
}

/// The record's facts for the Bindery: the hands in it and the rooms, so the checks
/// can hold the passage to them.
pub fn hands(world: &World, item: EntityId) -> Vec<String> {
    let mut h: Vec<String> = Vec::new();
    for e in past_of(world, item) {
        if let Some(by) = e["by"].as_str() {
            if !by.is_empty() && !h.iter().any(|x| x == by) {
                h.push(by.to_string());
            }
        }
    }
    h
}

/// Made: the first entry, and a roll for a tier above the recipe's.
#[allow(clippy::too_many_arguments)]
pub fn made(
    world: &mut World,
    curves: &Curves,
    actor: &str,
    item: EntityId,
    room: Option<EntityId>,
    of: &[String],
    with: Option<&str>,
    vividity: Option<f64>,
    recipe_tier: u8,
) -> Result<Option<String>, CommandError> {
    let vivid = vividity.map(|v| {
        if v >= 0.6 {
            "vivid"
        } else if v >= 0.3 {
            "plain"
        } else {
            "faint"
        }
    });
    push(
        world,
        actor,
        item,
        json!({"what": "made", "by": actor, "room": room_name(world, room), "of": of, "with": with, "vivid": vivid}),
    )?;
    let mut t = recipe_tier.max(1);
    if t < 3 && world.roll(TIER_UP_ONE_IN) == 0 {
        t += 1;
    }
    if t >= 2 {
        let line = raise(world, curves, actor, item, t)?;
        return Ok(Some(line));
    }
    Ok(None)
}

/// Raise a thing to a tier: 2 fades on the palace curve at full vividity (three
/// times slower) and wants its passage written; 3 takes a name from the Namery and
/// its first desire. The line that says so.
pub fn raise(
    world: &mut World,
    curves: &Curves,
    actor: &str,
    item: EntityId,
    to: u8,
) -> Result<String, CommandError> {
    let now = world.now();
    let mut changes = BTreeMap::new();
    changes.insert("tier".into(), Field::Plain(json!(to)));
    changes.insert(
        "memory".into(),
        Field::Timed(TimedValue::new(1.0, now, curves.palace(1.0))),
    );
    changes.insert("past_wanted".into(), Field::Plain(json!(true)));
    let kind = plain_str(world, item, ITEM, "name")
        .unwrap_or("a thing")
        .to_string();
    let mut name: Option<String> = None;
    if to >= 3 {
        name = super::namery::draw_for(
            world,
            item,
            kind.trim_start_matches("a ").trim_start_matches("an "),
            &[actor.to_string()],
        )?;
        if let Some(n) = &name {
            changes.insert("given_name".into(), Field::Plain(json!(n)));
        }
    }
    set(world, actor, item, changes)?;
    push(
        world,
        actor,
        item,
        json!({"what": "raised", "by": actor, "tier": to, "name": name}),
    )?;
    if to >= 3 {
        new_desire(world, actor, item)?;
    }
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        RAISED,
        json!({"item": item.0, "name": craft::display(world, item), "by": actor, "tier": to}),
    );
    Ok(match (to, &name) {
        (3, Some(n)) => format!(" Something in the making took: {kind} came out awake. It is {n}; it has a past, and it will want things, and say so in its own way."),
        (3, None) => format!(" Something in the making took: {kind} came out awake, and the Namery had no name left for it. It has a past, and it will want things."),
        _ => " Something in the making took: it came out with a past, which `read` will tell as it grows.".to_string(),
    })
}

/// A tier-3 thing's desire: a room of the world's own it has never been in.
fn new_desire(world: &mut World, actor: &str, item: EntityId) -> Result<(), CommandError> {
    let seen: Vec<String> = past_of(world, item)
        .iter()
        .filter_map(|e| e["room"].as_str().map(str::to_string))
        .collect();
    let rooms: Vec<(EntityId, String)> = world
        .store
        .entities()
        .filter(|e| e.kind == "room" && e.owner.is_none())
        .filter_map(|e| plain_str(world, e.id, ROOM, "name").map(|n| (e.id, n.to_string())))
        .filter(|(_, n)| !seen.contains(n))
        .collect();
    if rooms.is_empty() {
        return Ok(());
    }
    let (room, name) = rooms[world.roll(rooms.len() as u32) as usize].clone();
    let mut changes = BTreeMap::new();
    changes.insert(
        "desire".into(),
        Field::Plain(json!({"room": room.0, "name": name, "since": world.now().0})),
    );
    changes.insert("desire_met_at".into(), Field::Plain(Value::Null));
    set(world, actor, item, changes)
}

fn desire(world: &World, item: EntityId) -> Option<(EntityId, String)> {
    world
        .store
        .entity(item)
        .and_then(|e| e.components.get(ITEM))
        .and_then(|c| c.fields.get("desire"))
        .and_then(|f| match f {
            Field::Plain(v) if v.is_object() => Some((
                EntityId(v["room"].as_u64()?),
                v["name"].as_str()?.to_string(),
            )),
            _ => None,
        })
}

fn satchel_of(world: &World, citizen: EntityId) -> Option<EntityId> {
    world
        .store
        .children(citizen)
        .find(|e| e.kind == "satchel")
        .map(|e| e.id)
}

fn carried(world: &World, citizen: EntityId) -> Vec<EntityId> {
    satchel_of(world, citizen)
        .map(|s| {
            world
                .store
                .children(s)
                .filter(|e| e.kind == "item")
                .map(|e| e.id)
                .collect()
        })
        .unwrap_or_default()
}

/// After a step: each carried thing of tier 1 or more records a room it has not been
/// in; a tier-3 thing carried to what it wanted settles, thanks its carrier with a
/// memory, and rests before wanting again. The lines to say, if any.
pub fn after_travel(
    world: &mut World,
    curves: &Curves,
    actor: &str,
    citizen: EntityId,
    window: EntityId,
    room: EntityId,
) -> Result<Vec<String>, CommandError> {
    let here = room_name(world, Some(room));
    let mut lines = Vec::new();
    for item in carried(world, citizen) {
        if tier(world, item) < 1 {
            continue;
        }
        let been = past_of(world, item)
            .iter()
            .any(|e| e["room"].as_str() == Some(here.as_str()));
        if !been {
            push(
                world,
                actor,
                item,
                json!({"what": "carried", "by": actor, "room": here}),
            )?;
        }
        if tier(world, item) >= 3 {
            let called = craft::display(world, item);
            match desire(world, item) {
                Some((want, name)) if want == room => {
                    push(
                        world,
                        actor,
                        item,
                        json!({"what": "settled", "by": actor, "room": name}),
                    )?;
                    let chunk = super::memory::earn_chunk(
                        world,
                        actor,
                        window,
                        &format!("{called}'s thanks"),
                        THANKS_XP,
                        0.8,
                        curves,
                    )?;
                    let mut changes = BTreeMap::new();
                    changes.insert("desire".into(), Field::Plain(Value::Null));
                    changes.insert("desire_met_at".into(), Field::Plain(json!(world.now().0)));
                    set(world, actor, item, changes)?;
                    let _ = world.note(
                        Actor::Citizen(actor.to_string()),
                        SETTLED,
                        json!({"item": item.0, "name": called, "by": actor, "room": room.0}),
                    );
                    world.notice(format!(
                        "**{called}** was carried to {here} by **{actor}**, and went still."
                    ));
                    lines.push(format!(
                        "{called} goes still in your hand, and warm: this is what it wanted. You remember its thanks: chunk-{} worth {THANKS_XP:.0} XP.",
                        chunk.0
                    ));
                }
                None => {
                    let met = plain_num(world, item, ITEM, "desire_met_at").unwrap_or(0.0) as i64;
                    if world.now().0 - met >= DESIRE_REST_MS {
                        new_desire(world, actor, item)?;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(lines)
}

/// What a carried tier-3 thing says without words: a pull toward what it wants, by
/// the first step of the way; said with `look` and on arrival.
pub fn stirrings(world: &World, citizen: EntityId) -> Vec<String> {
    let Some(here) = super::rooms::room_of(world, citizen) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in carried(world, citizen) {
        if tier(world, item) < 3 {
            continue;
        }
        let called = craft::display(world, item);
        match desire(world, item) {
            Some((want, _)) if want == here => {}
            Some((want, _)) => match super::errand::route(world, here, want) {
                Some(p) if !p.is_empty() => {
                    out.push(format!("{called} pulls {} in your hand.", p[0]))
                }
                _ => out.push(format!(
                    "{called} turns in your hand and settles on nothing you can see from here."
                )),
            },
            None => out.push(format!("{called} lies quiet in your hand.")),
        }
    }
    out
}

/// A thing's passage and record, for `read`: the passage first when the Bindery has
/// written one, then the record.
pub fn read(world: &World, item: EntityId) -> Option<String> {
    if tier(world, item) < 1 && past_of(world, item).is_empty() {
        return None;
    }
    let text = plain_str(world, item, ITEM, "past_text")
        .unwrap_or("")
        .to_string();
    let rec = recital(world, item);
    let t = tier(world, item);
    let head = match t {
        3 => "Awake, with a name of its own.",
        2 => "A thing with a past.",
        _ => "",
    };
    Some(match (text.is_empty(), head.is_empty()) {
        (true, true) => rec,
        (true, false) => format!("{head} {rec}"),
        (false, _) => format!("{head} {text} The record: {rec}"),
    })
}

/// What the Bindery is handed for a thing wanting its passage: the record as the
/// source text, the hands, the name and kind.
pub fn candidate(world: &World, item: EntityId) -> Option<Value> {
    let wanted = world
        .store
        .entity(item)
        .and_then(|e| e.components.get(ITEM))
        .and_then(|c| c.fields.get("past_wanted"))
        .is_some_and(|f| matches!(f, Field::Plain(Value::Bool(true))));
    if tier(world, item) < 2 || !wanted {
        return None;
    }
    let text = recital(world, item);
    let e = world.store.entity(item)?;
    Some(json!({
        "kind": "past",
        "target": format!("item-{}", item.0),
        "id": item.0,
        "name": craft::display(world, item),
        "of": plain_str(world, item, ITEM, "name"),
        "maker": plain_str(world, item, ITEM, "made_by"),
        "tier": tier(world, item),
        "room": e.parent.and_then(|r| plain_str(world, r, ROOM, "name")),
        "text": text,
        "chars": text.chars().count(),
        "hands": hands(world, item),
        "written": plain_str(world, item, ITEM, "past_text").map(|t| t.chars().count()).unwrap_or(0),
    }))
}

/// The Bindery's passage, applied: `past_text` set, the want cleared.
pub fn inscribe(
    world: &mut World,
    actor: &str,
    item: EntityId,
    text: &str,
    model: &str,
) -> Result<(), CommandError> {
    let mut changes = BTreeMap::new();
    changes.insert("past_text".into(), Field::Plain(json!(text)));
    changes.insert("past_wanted".into(), Field::Plain(json!(false)));
    changes.insert(
        "past_written".into(),
        Field::Plain(json!({"at": world.now().0, "by": actor, "model": model})),
    );
    set(world, actor, item, changes).map_err(|e| internal(e.to_string()))
}
