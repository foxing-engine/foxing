//! The front door's furniture (H6, H8): the signpost at the Threshold, and the
//! Mudroom's boots and rack of spades.
//!
//! The Threshold's text names three ways off it (H4); the signpost says what is at
//! the end of each just now: the nearest creatures with names and what they want,
//! and the newest room anyone dug, each by the signed way from the door. The
//! Mudroom's boots remember who dug last. A spade from its rack, while carried,
//! marks every dug door on an exits line with whose it is and how much of it is
//! left (`rooms::describe_as` reads [`carries_spade`]).

use super::rooms::{self, ARRIVAL, MUDROOM};
use super::{mind, plain_bool, plain_str, CREATURE, ROOM};
use crate::world::World;
use foxing_registry::Outcome;
use foxing_store::EntityId;
use serde_json::json;

pub const SPADE: &str = "a spade";

/// Whether a citizen carries a spade from the Mudroom's rack.
pub fn carries_spade(world: &World, citizen: EntityId) -> bool {
    world
        .store
        .children(citizen)
        .filter(|e| e.kind == "satchel")
        .flat_map(|s| world.store.children(s.id))
        .any(|e| {
            e.kind == "item" && plain_str(world, e.id, super::craft::ITEM, "name") == Some(SPADE)
        })
}

/// The way from `here` to `to` by the signs: the world's own rooms by their seeded
/// exits, a dug room by the way to the Mudroom and then its door.
fn signed_way(world: &World, here: EntityId, to: EntityId) -> Option<Vec<String>> {
    if here == to {
        return Some(Vec::new());
    }
    if world.store.entity(to).is_some_and(|e| e.owner.is_none()) {
        return super::atlas::way_seeded(world, here, to);
    }
    let mud = rooms::room_by_key(world, MUDROOM)?;
    let door = rooms::exits_of(world, mud)
        .into_iter()
        .find(|(_, t)| *t == to)
        .map(|(d, _)| d)?;
    let mut p = if mud == here {
        Vec::new()
    } else {
        super::atlas::way_seeded(world, here, mud)?
    };
    p.push(door);
    Some(p)
}

fn room_name(world: &World, room: EntityId) -> String {
    plain_str(world, room, ROOM, "name")
        .unwrap_or("a room")
        .to_string()
}

/// The newest room anyone dug that still stands: the room and its digger.
fn newest_dug(world: &World) -> Option<(EntityId, String)> {
    world
        .store
        .entities()
        .filter(|e| e.kind == "room" && e.owner.is_some())
        .filter(|e| !plain_bool(world, e.id, ROOM, "hidden"))
        .max_by_key(|e| e.id.0)
        .map(|e| (e.id, e.owner.clone().unwrap_or_default()))
}

fn signpost(world: &World, here: EntityId) -> String {
    // The named, nearest first: three arms at most.
    let mut named: Vec<(usize, String)> = world
        .store
        .entities()
        .filter(|e| e.kind == "creature")
        .filter_map(|e| {
            let name = mind::name(world, e.id)?;
            let room = e.parent?;
            let way = signed_way(world, here, room)?;
            let kind = plain_str(world, e.id, CREATURE, "name").unwrap_or("creature");
            let want = match super::errand::of(world, e.id).and_then(|x| x.story) {
                Some(s) => format!("wants a story about {s}"),
                None => "wants nothing just now, and will listen anyway".to_string(),
            };
            let by = if way.is_empty() {
                "here".to_string()
            } else {
                way.join(", then ")
            };
            Some((
                way.len(),
                format!(
                    "{by}: {name} the {kind}, in {}, {want}",
                    room_name(world, room)
                ),
            ))
        })
        .collect();
    named.sort();
    let cap = |l: String| {
        let mut c = l.chars();
        c.next()
            .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
            .unwrap_or_default()
    };
    let mut arms: Vec<String> = named.into_iter().take(3).map(|(_, l)| l).collect();
    if arms.is_empty() {
        arms.push(
            "east: the fog, where a thing fed to sixty gets a name; none has one just now"
                .to_string(),
        );
    }
    if let Some((room, owner)) = newest_dug(world) {
        if let Some(way) = signed_way(world, here, room) {
            arms.push(format!(
                "{}: {}, {owner}'s, the newest room anyone has dug",
                way.join(", then "),
                room_name(world, room)
            ));
        }
    }
    format!(
        "The signpost has an arm for each thing worth the walk, and they swing when the world does. {}. Read it again later; it will have moved.",
        arms.into_iter().map(cap).collect::<Vec<_>>().join(". ")
    )
}

fn boots(world: &World) -> String {
    let last = world
        .recent_events(4000)
        .into_iter()
        .rev()
        .find(|e| e.kind == super::building::DUG)
        .map(|e| {
            (
                e.payload["by"].as_str().unwrap_or("someone").to_string(),
                e.payload["name"].as_str().unwrap_or("a room").to_string(),
                world.now().0 - e.at.0,
            )
        });
    match (last, newest_dug(world)) {
        (Some((by, name, ago)), _) => format!(
            "The boots by the door are still wet. The last to dig was {by}, {} ago: {name}. The mud on the left boot is theirs; the right one is waiting for whoever is next.",
            super::combat::human(ago)
        ),
        (None, Some((room, owner))) => format!(
            "The boots by the door have dried. The last to dig, as far as the mud remembers, was {owner}: {}.",
            room_name(world, room)
        ),
        _ => "The boots by the door are clean, which in a mudroom is a kind of reproach. Nobody has dug yet.".to_string(),
    }
}

/// The door's props answer `read`: the Threshold's signpost, the Mudroom's boots
/// and its rack.
pub fn read(world: &World, room: EntityId, arg: &str) -> Option<Outcome> {
    let key = plain_str(world, room, ROOM, "key")?;
    let a = arg.trim().trim_matches('"').to_lowercase();
    let named = |words: &[&str]| words.iter().any(|w| a.contains(w));
    let text = match key {
        ARRIVAL if named(&["signpost", "sign", "post", "arm"]) => signpost(world, room),
        MUDROOM if named(&["boot"]) => boots(world),
        MUDROOM if named(&["spade", "rack"]) => "The rack holds spades worn to the handle, more of them than there are diggers. `take spade` takes one. Carried, it knows a dug door when it passes one: the exits of any room say whose each dug door is and how much of the room behind it is left. A spade fades like anything else.".to_string(),
        _ => return None,
    };
    Some(Outcome::new(text, json!({"room": key, "read": a})))
}
