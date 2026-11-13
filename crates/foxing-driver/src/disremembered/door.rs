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
        .of_kind("room")
        .filter(|e| e.owner.is_some())
        .filter(|e| !plain_bool(world, e.id, ROOM, "hidden"))
        .max_by_key(|e| e.id.0)
        .map(|e| (e.id, e.owner.clone().unwrap_or_default()))
}

fn signpost(world: &World, here: EntityId) -> String {
    // The named, nearest first: three arms at most.
    let mut named: Vec<(usize, String)> = world
        .store
        .of_kind("creature")
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

/// H11: a hub's own post: each way off it, where it goes, what district that is,
/// what lives there and whether it hunts, and what lies one room beyond.
fn hub_post(world: &World, here: EntityId) -> String {
    let exits = rooms::exits_of(world, here);
    let mut order: Vec<(&String, &EntityId)> = exits.iter().collect();
    order.sort_by_key(|(e, _)| {
        let c = rooms::COMPASS.iter().position(|x| x == *e);
        (c.is_none(), c.unwrap_or(0), (*e).clone())
    });
    let mine = rooms::district_of(world, here).map(|d| d.name);
    let arms: Vec<String> = order
        .into_iter()
        .filter(|(_, to)| world.store.entity(**to).is_some_and(|e| e.owner.is_none()))
        .map(|(via, &to)| {
            let name = room_name(world, to);
            let key = plain_str(world, to, ROOM, "key").unwrap_or("");
            let district = rooms::district_of(world, to)
                .map(|d| d.name)
                .filter(|d| Some(*d) != mine)
                .map(|d| format!(", in {d}"))
                .unwrap_or_default();
            let def = rooms::ROOMS.iter().find(|r| r.key == key);
            let lives = match def {
                Some(r) if r.description.contains("Nothing hunts here") => match r.spawn {
                    Some((k, _, _)) => {
                        format!("{} live there and nothing hunts", super::combat::plural(k))
                    }
                    None => "nothing hunts there".to_string(),
                },
                Some(r) => match r.spawn {
                    Some((k, _, _)) => format!("{} live there", super::combat::plural(k)),
                    None => "nothing lives there".to_string(),
                },
                None => String::new(),
            };
            let beyond: Vec<String> = rooms::exits_of(world, to)
                .into_values()
                .filter(|&b| b != here && world.store.entity(b).is_some_and(|e| e.owner.is_none()))
                .map(|b| room_name(world, b).trim_start_matches("The ").to_string())
                .collect();
            let mut cap = via.clone();
            if let Some(c) = cap.get_mut(0..1) {
                c.make_ascii_uppercase();
            }
            format!(
                "{cap}: {name}{district} ({lives}){}",
                if beyond.is_empty() {
                    "; it ends there".to_string()
                } else {
                    format!("; beyond it, {}", beyond.join(" and "))
                }
            )
        })
        .collect();
    format!(
        "The signpost here has an arm for each way off. {}. `where <a room or a district>` is the way to anywhere; `travel <its name>` walks it.",
        arms.join(". ")
    )
}

/// H14: the Mudroom's exits as rows. The world's own doors, then yours, the three
/// newest dug and the three best remembered; the rest are counted. Nine dug doors
/// or fewer are all said.
pub fn rows(
    world: &World,
    actor: &str,
    exits: &[String],
    by_name: &std::collections::BTreeMap<String, EntityId>,
) -> (Vec<String>, usize) {
    let dug: Vec<(&String, EntityId, String)> = exits
        .iter()
        .filter_map(|e| {
            let to = *by_name.get(e)?;
            let owner = world.store.entity(to)?.owner.clone()?;
            Some((e, to, owner))
        })
        .collect();
    if dug.len() <= 9 {
        return (exits.to_vec(), 0);
    }
    let mut chosen: Vec<String> = exits
        .iter()
        .filter(|e| !dug.iter().any(|(d, _, _)| d == e))
        .cloned()
        .collect();
    let add = |e: &String, chosen: &mut Vec<String>| {
        if !chosen.contains(e) {
            chosen.push(e.clone());
        }
    };
    for (e, _, o) in &dug {
        if o == actor {
            add(e, &mut chosen);
        }
    }
    let mut newest: Vec<&(&String, EntityId, String)> = dug.iter().collect();
    newest.sort_by_key(|(_, to, _)| std::cmp::Reverse(to.0));
    for (e, _, _) in newest.into_iter().take(3) {
        add(e, &mut chosen);
    }
    let mut kept: Vec<(f64, &String)> = dug
        .iter()
        .map(|(e, to, _)| (super::building::memory(world, *to).unwrap_or(0.0), *e))
        .collect();
    kept.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    for (_, e) in kept.into_iter().take(3) {
        add(e, &mut chosen);
    }
    let shown_dug = chosen
        .iter()
        .filter(|e| dug.iter().any(|(d, _, _)| d == e))
        .count();
    (chosen, dug.len() - shown_dug)
}

/// H14: the whole wall, best remembered first.
fn wall(world: &World, here: EntityId) -> String {
    let mut doors: Vec<(f64, String)> = rooms::exits_of(world, here)
        .into_iter()
        .filter_map(|(e, to)| {
            let owner = world.store.entity(to)?.owner.clone()?;
            let m = super::building::memory(world, to).unwrap_or(0.0);
            // H19: and how long ago it was dug, where the room knows.
            let age = super::plain_num(world, to, ROOM, "dug_at")
                .map(|at| world.now().0 - at as i64)
                .filter(|ms| *ms >= 0)
                .map(|ms| {
                    if ms < 60_000 {
                        ", dug just now".to_string()
                    } else {
                        format!(", dug {} ago", super::combat::human(ms))
                    }
                })
                .unwrap_or_default();
            Some((
                m,
                format!(
                    "{e}: {}, {owner}'s{age}, {:.0}% left",
                    room_name(world, to),
                    m * 100.0
                ),
            ))
        })
        .collect();
    if doors.is_empty() {
        return "The wall is bare: nobody has dug a door here yet. `dig <a word for the door> <chunk>` makes the first.".to_string();
    }
    doors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let n = doors.len();
    format!(
        "The Mudroom's wall, {n} door{} dug, the best remembered first. {}. A door opens by its word (`travel <word>`), and a room nobody visits fades off the wall.",
        if n == 1 { "" } else { "s" },
        doors.into_iter().map(|(_, s)| s).collect::<Vec<_>>().join("; ")
    )
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
        MUDROOM if named(&["door", "wall", "row"]) => wall(world, room),
        k if rooms::HUBS.contains(&k) && named(&["signpost", "sign", "post", "arm"]) => hub_post(world, room),
        MUDROOM if named(&["spade", "rack"]) => "The rack holds spades worn to the handle, more of them than there are diggers. `take spade` takes one. Carried, it knows a dug door when it passes one: the exits of any room say whose each dug door is and how much of the room behind it is left. A spade fades like anything else.".to_string(),
        _ => return None,
    };
    Some(Outcome::new(text, json!({"room": key, "read": a})))
}
