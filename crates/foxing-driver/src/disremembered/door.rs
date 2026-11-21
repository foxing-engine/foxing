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
pub(crate) fn signed_way(world: &World, here: EntityId, to: EntityId) -> Option<Vec<String>> {
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

pub(super) fn room_name(world: &World, room: EntityId) -> String {
    plain_str(world, room, ROOM, "name")
        .unwrap_or("a room")
        .to_string()
}

/// The newest room anyone dug that still stands: the room and its digger.
pub(super) fn newest_dug(world: &World) -> Option<(EntityId, String)> {
    world
        .store
        .of_kind("room")
        .filter(|e| e.owner.is_some())
        .filter(|e| !plain_bool(world, e.id, ROOM, "hidden"))
        .max_by_key(|e| e.id.0)
        .map(|e| (e.id, e.owner.clone().unwrap_or_default()))
}

/// C10: the creature at the end of the signpost's first arm, and what it wants a
/// story about (none when it wants nothing just now): the Threshold's slip asks.
pub(super) fn first_arm(world: &World, here: EntityId) -> Option<(String, Option<String>)> {
    let mut named: Vec<(usize, String, String, Option<String>)> = world
        .store
        .of_kind("creature")
        .filter_map(|e| {
            let name = mind::name(world, e.id)?;
            let room = e.parent?;
            let way = signed_way(world, here, room)?;
            let story = super::errand::of(world, e.id).and_then(|x| x.story);
            // (The arms sort by how far and then by their line, which begins with
            // the way: the same order here.)
            Some((way.len(), way.join(", then "), name, story))
        })
        .collect();
    named.sort();
    named.into_iter().next().map(|(_, _, n, s)| (n, s))
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
                Some(s) => format!(
                    "wants a story about {s}{}",
                    super::errand::stands_for(world, e.id)
                ),
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
    // R216: the Dead Letter Office has an arm when a letter waits in it; nothing
    // told a newcomer it was there.
    let waiting = super::letters::waiting(world);
    if waiting > 0 {
        arms.push(format!(
            "the Dead Letter Office, off the Namery (`where \"Dead Letter Office\"`): {waiting} letter{} for names the world forgot, and a small memory for answering or carrying one (`letters`)",
            if waiting == 1 { " waits" } else { "s wait" }
        ));
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
    // R249: away from the front door, how far it is (the Postern's post named the
    // lane and the wall and not the Threshold, three steps round).
    let front = super::atlas::room_named(world, "The Threshold")
        .filter(|t| *t != here)
        .and_then(|t| signed_way(world, here, t))
        .filter(|way| way.len() > 1)
        .map(|way| {
            format!(
                " The Threshold, the world's front door, is {} steps by the signs: {}.",
                way.len(),
                way.join(", ")
            )
        })
        .unwrap_or_default();
    format!(
        "The signpost here has an arm for each of the world's own ways off (a dug door has none). {}.{front} `where <a room or a district>` is the way to anywhere; `travel <its name>` walks it.",
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

/// How many doors `read doors` says before it counts the rest (H18: sixty doors
/// were 3,800 characters, more than a reply carries whole).
pub const WALL_SHOWN: usize = 20;

/// H14: the wall, best remembered first. H18: the first twenty and a count of the
/// rest; `read doors all` is every door, and `read doors <a digger>` is theirs.
fn wall(world: &World, here: EntityId, arg: &str) -> String {
    let all = arg.split_whitespace().any(|w| w == "all" || w == "every");
    let mut doors: Vec<(f64, String, String)> = rooms::exits_of(world, here)
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
                // R207: three doors of nine had an age and the rest said nothing.
                .unwrap_or_else(|| ", dug before the wall kept dates".to_string());
            Some((
                m,
                {
                    // L3 (the Watch of 11-21: the wall gave dates and percentages
                    // and "made me want to visit nobody"): its standing, and the
                    // first of what its keeper wrote of it.
                    let stands = super::tending::STANDINGS[super::tending::standing(world, to)];
                    let first = super::plain_str(world, to, ROOM, "description")
                        .filter(|d| !d.starts_with("Freshly dug."))
                        .and_then(|d| d.split_inclusive(['.', '!', '?']).next())
                        .map(|s| {
                            let s: String = s.trim().chars().take(90).collect();
                            format!(" (\u{201c}{s}\u{201d})")
                        })
                        .unwrap_or_default();
                    let title = super::tending::title(world, &owner)
                        .map(|(t, _)| format!(" the {t}"))
                        .unwrap_or_default();
                    format!(
                        "{e}: {}, {owner}{title}'s{age}, {stands}, {:.0}% left{first}",
                        room_name(world, to),
                        m * 100.0
                    )
                },
                owner,
            ))
        })
        .collect();
    if doors.is_empty() {
        return "The wall is bare: nobody has dug a door here yet. `dig <a word for the door> <chunk>` makes the first.".to_string();
    }
    doors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let n = doors.len();
    let s = if n == 1 { "" } else { "s" };
    // One digger's doors, asked for by name.
    let digger = arg.split_whitespace().find_map(|w| {
        let w = w.trim_end_matches("'s");
        doors
            .iter()
            .find(|d| d.2.eq_ignore_ascii_case(w))
            .map(|d| d.2.clone())
    });
    if let Some(who) = digger {
        let theirs: Vec<String> = doors
            .iter()
            .filter(|d| d.2 == who)
            .map(|d| d.1.clone())
            .collect();
        return format!(
            "The Mudroom's wall, {n} door{s} dug; {who}'s: {}. A door opens by its word (`travel <word>`).",
            theirs.join("; ")
        );
    }
    if all || n <= WALL_SHOWN {
        return format!(
            "The Mudroom's wall, {n} door{s} dug, the best remembered first. {}. A door opens by its word (`travel <word>`), and a room nobody visits fades off the wall.",
            doors.into_iter().map(|d| d.1).collect::<Vec<_>>().join("; ")
        );
    }
    let rest = &doors[WALL_SHOWN..];
    let faint = rest.iter().filter(|d| d.0 < 0.1).count();
    format!(
        "The Mudroom's wall, {n} doors dug; the {WALL_SHOWN} best remembered: {}. And {} more, {} with under a tenth left of them: `read doors all` is every door, and `read doors <a digger's name>` is that digger's. A door opens by its word (`travel <word>`), and a room nobody visits fades off the wall.",
        doors[..WALL_SHOWN].iter().map(|d| d.1.clone()).collect::<Vec<_>>().join("; "),
        rest.len(),
        if faint == rest.len() { "all of them".to_string() } else { format!("{faint} of them") },
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
        MUDROOM if named(&["door", "wall", "row"]) => wall(world, room, &a),
        k if rooms::HUBS.contains(&k) && named(&["signpost", "sign", "post", "arm"]) => hub_post(world, room),
        MUDROOM if named(&["spade", "rack"]) => "The rack holds spades worn to the handle, more of them than there are diggers. `take spade` takes one. Carried, it knows a dug door when it passes one: the exits of any room say whose each dug door is and how much of the room behind it is left. A spade fades like anything else.".to_string(),
        _ => return None,
    };
    Some(Outcome::new(text, json!({"room": key, "read": a})))
}
