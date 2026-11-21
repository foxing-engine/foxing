//! L3: Keep. A dug room has a standing that rises with care and falls without it.
//!
//! David, 11-21: "we should add graphical upgrades, improved beneficial creature
//! spawning etc. for well tended rooms. high level keepers are renowned and that has
//! effects"; then "L3a:1 / L3b:1 / L3c:3 / L3d:2 / L3E:yes, everything decays". The
//! Watch of that morning called keeping "bailing a boat": nothing kept grew, nobody
//! came, and coming home reported only decay.
//!
//! **Standing** is counted from what a room already has, investment and use together
//! (L3a:1): it is described; its half-life has been lengthened (two hours, eight);
//! something stands in it; a named creature lives in it; other names have stood in
//! it in the last day (one, three). Two points is Kept, four Tended, six Cherished.
//! A room under half remembered counts one step down, and under a tenth is Bare:
//! everything decays (L3e), and a visit, which remembers a room whole, is what holds
//! it up.
//!
//! **What a standing does.** The picture gains a lamp, then a rug and a hearth, then
//! a window (the page reads the room's `tend`). A resident comes at each step
//! (L3c:3): a glowworm, by whose light strikes land nine in ten for ten minutes
//! after you leave; a mender, in whose room a body mends as by a basin; a porter,
//! that leaves a making on the floor each day. Strangers wander in more often too.
//! A visitor's first visit of the day to a Tended room is a small memory for the
//! visitor and a smaller one for the keeper (L3b:1), three rooms a day at most.
//!
//! **Renown** (L3d:2) is the sum of a keeper's rooms' standings: a title others see,
//! rooms that fade more slowly, and at Steward a second door off the Mudroom.

use super::{plain_str, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const STANDINGS: [&str; 4] = ["Bare", "Kept", "Tended", "Cherished"];
/// The residents, by the standing that draws each.
pub const RESIDENTS: [(usize, &str); 3] = [(1, "glowworm"), (2, "mender"), (3, "porter")];
/// One chance in this many, each pass of the wander clock, that a stranger comes in.
pub const WANDER_ONE_IN: [u32; 4] = [40, 30, 20, 12];
/// A visitor's first visit of the day to a Tended room, and the keeper's share.
pub const VISIT_XP: f64 = 6.0;
pub const KEEPER_XP: f64 = 3.0;
/// Paid visits a name may make in a day.
pub const VISITS_A_DAY: u64 = 3;
/// How long a glowworm's light stays on a name.
pub const LIT_MS: i64 = 10 * 60_000;
/// What a porter leaves.
const PORTER_BRINGS: &[&str] = &[
    "rag thread",
    "a dab of ink",
    "hollow water",
    "moth dust",
    "a wisp ember",
];
/// A visitor counts toward a room's standing for a day.
const VISITOR_MS: i64 = 24 * 3_600_000;

/// A keeper's renown: `(sum of standings at least, title, how many times longer its
/// rooms hold)`.
pub const TITLES: [(u32, &str, f64); 4] = [
    (2, "Householder", 1.0),
    (4, "Keeper", 1.25),
    (6, "Steward", 1.5),
    (9, "Castellan", 2.0),
];

pub struct Tend {
    pub points: u32,
    pub standing: usize,
    /// What it has, and what would raise it.
    pub has: Vec<String>,
    pub wants: Vec<String>,
    pub visitors: Vec<String>,
}

/// A dug room's standing, as it stands now. `None` for a room of the world's own.
pub fn assess(world: &World, room: EntityId) -> Option<Tend> {
    let e = world.store.entity(room)?;
    let owner = e.owner.clone()?;
    if e.kind != "room" {
        return None;
    }
    let memory = super::building::memory(world, room)?;
    let mut points = 0;
    let mut has = Vec::new();
    let mut wants = Vec::new();
    let described = plain_str(world, room, ROOM, "description")
        .is_some_and(|d| !d.trim().is_empty() && !d.starts_with("Freshly dug."));
    if described {
        points += 1;
        has.push("it is described".to_string());
    } else {
        wants.push("a description (`describe`)".to_string());
    }
    let hl_h = super::building::half_life_of(world, room)
        .map(|d| d.0 as f64 / 3_600_000.0)
        .unwrap_or(0.0);
    if hl_h >= 8.0 {
        points += 2;
        has.push("it holds a long while".to_string());
    } else if hl_h >= 2.0 {
        points += 1;
        has.push("it holds two hours and more".to_string());
        wants.push(
            "more memory put into it, to hold eight hours (`remember <chunk>` here)".to_string(),
        );
    } else {
        wants.push("memory put into it, to hold two hours (`remember <chunk>` here)".to_string());
    }
    if !super::craft::fixtures_in(world, room).is_empty() {
        points += 1;
        has.push("something stands in it".to_string());
    } else {
        wants.push("a thing standing in it (a visitor book, a basin: `recipes`)".to_string());
    }
    let lives_here = world.store.children(room).any(|c| {
        c.kind == "creature"
            && super::mind::name(world, c.id).is_some()
            && super::life::home_of(world, c.id) == Some(room)
    });
    if lives_here {
        points += 1;
        has.push("a named creature lives in it".to_string());
    } else {
        wants.push("a named creature living in it (`invite`)".to_string());
    }
    let visitors: Vec<String> = super::building::came_through_within(world, room, VISITOR_MS)
        .into_iter()
        .map(|(n, _)| n)
        .filter(|n| *n != owner)
        .collect();
    match visitors.len() {
        0 => wants.push("another name standing in it (a visitor counts for a day)".to_string()),
        1 | 2 => {
            points += 1;
            has.push(format!("{} came by today", visitors.join(" and ")));
            wants.push("three visitors in a day".to_string());
        }
        n => {
            points += 2;
            has.push(format!("{n} names came by today"));
        }
    }
    let by_points: usize = match points {
        0 | 1 => 0,
        2 | 3 => 1,
        4 | 5 => 2,
        _ => 3,
    };
    // Everything decays: a room half forgotten stands a step lower, and one nearly
    // gone is bare whatever is in it.
    let standing = if memory < 0.1 {
        0
    } else if memory < 0.5 {
        by_points.saturating_sub(1)
    } else {
        by_points
    };
    Some(Tend {
        points,
        standing,
        has,
        wants,
        visitors,
    })
}

pub fn standing(world: &World, room: EntityId) -> usize {
    assess(world, room).map(|t| t.standing).unwrap_or(0)
}

/// One sentence for `look` in a dug room.
pub fn line(world: &World, room: EntityId, actor: &str) -> Option<String> {
    let t = assess(world, room)?;
    let owner = world.store.entity(room)?.owner.clone()?;
    let mine = owner == actor;
    let whose = if mine {
        "Your room".to_string()
    } else {
        format!("{owner}'s room")
    };
    let residents: Vec<&str> = RESIDENTS
        .iter()
        .filter(|(_, k)| resident(world, room, k).is_some())
        .map(|(_, k)| *k)
        .collect();
    let gives = match residents.as_slice() {
        [] => String::new(),
        r => format!(
            " ({})",
            r.iter()
                .map(|k| match *k {
                    "glowworm" => "by the glowworm's light your strikes land nine in ten for ten minutes after you leave",
                    "mender" => "the mender mends a body here three times as fast",
                    _ => "the porter leaves a making here each day",
                })
                .collect::<Vec<_>>()
                .join("; ")
        ),
    };
    Some(if mine && t.standing < 3 && !t.wants.is_empty() {
        format!(
            "{whose} is {}{gives}. It would rise with {}.",
            STANDINGS[t.standing],
            t.wants[..t.wants.len().min(2)].join(", or ")
        )
    } else {
        format!("{whose} is {}{gives}.", STANDINGS[t.standing])
    })
}

/// A keeper's renown: the sum of the standings of the rooms it dug.
pub fn renown(world: &World, name: &str) -> u32 {
    world
        .store
        .of_kind("room")
        .filter(|e| e.owner.as_deref() == Some(name))
        .map(|e| standing(world, e.id) as u32)
        .sum()
}

/// `(title, how many times longer its rooms hold)`, or nothing under Householder.
pub fn title(world: &World, name: &str) -> Option<(&'static str, f64)> {
    let r = renown(world, name);
    TITLES
        .iter()
        .rev()
        .find(|(need, _, _)| r >= *need)
        .map(|(_, t, m)| (*t, *m))
}

/// How many doors a name may dig off the Mudroom (a seed room): two for a Steward.
pub fn seed_doors(world: &World, name: &str) -> usize {
    if renown(world, name) >= TITLES[2].0 {
        2
    } else {
        1
    }
}

fn resident(world: &World, room: EntityId, kind: &str) -> Option<EntityId> {
    world
        .store
        .children(room)
        .find(|c| c.kind == "creature" && plain_str(world, c.id, CREATURE, "name") == Some(kind))
        .map(|c| c.id)
}

/// Whether a room has a mender in it (a body mends there as by a basin).
pub fn has_mender(world: &World, room: EntityId) -> bool {
    resident(world, room, "mender").is_some()
}

pub fn is_resident_kind(kind: &str) -> bool {
    RESIDENTS.iter().any(|(_, k)| *k == kind)
}

fn set_room(
    world: &mut World,
    room: EntityId,
    fields: BTreeMap<String, Field>,
) -> Result<(), String> {
    let fields = super::building::merged(world, room, fields);
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

/// On the wander clock: each dug room's standing is written where the page reads it,
/// its residents come and go with it, and a porter leaves the day's making.
pub fn round(world: &mut World) -> Result<(), String> {
    let rooms: Vec<EntityId> = world
        .store
        .of_kind("room")
        .filter(|e| e.owner.is_some())
        .map(|e| e.id)
        .collect();
    for room in rooms {
        let Some(t) = assess(world, room) else {
            continue;
        };
        let was = super::plain_num(world, room, ROOM, "tend").unwrap_or(0.0) as usize;
        if was != t.standing {
            let mut f = BTreeMap::new();
            f.insert("tend".to_string(), Field::Plain(json!(t.standing)));
            set_room(world, room, f)?;
            let owner = world
                .store
                .entity(room)
                .and_then(|e| e.owner.clone())
                .unwrap_or_default();
            let rname = plain_str(world, room, ROOM, "name")
                .unwrap_or("a room")
                .to_string();
            let _ = world.note(
                Actor::Driver,
                STOOD,
                json!({"room": room.0, "name": rname, "owner": owner, "was": was, "standing": t.standing, "said": STANDINGS[t.standing]}),
            );
            super::combat::narrate(
                world,
                &owner,
                if t.standing > was {
                    format!(
                        "{rname} is {} now: {}.",
                        STANDINGS[t.standing],
                        t.has.join(", ")
                    )
                } else {
                    format!(
                        "{rname} has slipped to {}: {}.",
                        STANDINGS[t.standing],
                        if t.wants.is_empty() {
                            "it is fading".to_string()
                        } else {
                            format!("it wants {}", t.wants[0])
                        }
                    )
                },
            );
            if t.standing > was && t.standing >= 2 {
                world.notice(format!(
                    "**{rname}**, {owner}'s room, is **{}**.",
                    STANDINGS[t.standing]
                ));
            }
        }
        for (need, kind) in RESIDENTS {
            let here = resident(world, room, kind);
            if t.standing >= need && here.is_none() {
                if let Some(d) = super::combat::def_by_key(kind) {
                    let c = super::combat::spawn_one(world, room, d).map_err(|e| e.to_string())?;
                    super::combat::narrate_room(world, c, &arrives(kind));
                }
            } else if t.standing < need {
                if let Some(c) = here {
                    super::combat::narrate_room(
                        world,
                        c,
                        &format!("The {kind} goes: the room is not what it was."),
                    );
                    world
                        .record(Actor::Driver, StoreEvent::EntityRemoved { id: c })
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        // The porter's day: one making on the floor, where there are fewer than three.
        if resident(world, room, "porter").is_some() {
            let day = world.now().0.div_euclid(86_400_000);
            if super::plain_num(world, room, ROOM, "portered").map(|d| d as i64) != Some(day) {
                let loose = world
                    .store
                    .children(room)
                    .filter(|c| c.kind == "item" && !super::craft::is_fixed(world, c.id))
                    .count();
                if loose < 3 {
                    let what = PORTER_BRINGS[world.roll(PORTER_BRINGS.len() as u32) as usize];
                    let curves = super::Curves::from_env();
                    super::craft::make(
                        world,
                        &curves,
                        Actor::Driver,
                        None,
                        room,
                        what,
                        "material",
                        "brought by the porter",
                        None,
                        &[],
                        0,
                        false,
                    )
                    .map_err(|e| e.to_string())?;
                    if let Some(p) = resident(world, room, "porter") {
                        super::combat::narrate_room(world, p, &format!("The porter sets down {what} by the door, and goes back to not being looked at."));
                    }
                }
                let mut f = BTreeMap::new();
                f.insert("portered".to_string(), Field::Plain(json!(day)));
                set_room(world, room, f)?;
            }
        }
    }
    Ok(())
}

fn arrives(kind: &str) -> String {
    match kind {
        "glowworm" => "A glowworm has come up through the floor and hung itself in the corner, a small steady light: the room is kept, and it knows a kept room. By its light your strikes land nine in ten, for ten minutes after you leave.".to_string(),
        "mender" => "A mender has come in, a thing of patient hands with a needle through its lapel, and taken the chair by the wall: a body mends here three times as fast while it stays.".to_string(),
        _ => "A porter has taken up a place by the door, bent under a pack that is always nearly empty: each day it leaves one making on the floor here.".to_string(),
    }
}

/// A room rose or slipped: `{room, name, owner, was, standing, said}`. Public.
pub const STOOD: &str = "disremembered.room_stood";
/// A visit was paid: `{room, name, owner, visitor, xp, keeper_xp}`. Public.
pub const VISITED: &str = "disremembered.room_visited";

/// A name has arrived in a room: a glowworm's light goes on it, and a first visit
/// of the day to a Tended room is paid. Returns what to say to the arriver.
pub fn on_arrival(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    window: EntityId,
    room: EntityId,
) -> Result<Option<String>, String> {
    let Some(owner) = world.store.entity(room).and_then(|e| e.owner.clone()) else {
        return Ok(None);
    };
    let mut said = Vec::new();
    if resident(world, room, "glowworm").is_some() {
        let until = world.now().0 + LIT_MS;
        world
            .record(
                Actor::Driver,
                StoreEvent::FieldSet {
                    id: citizen,
                    component: super::BODY.into(),
                    field: "lit_until".into(),
                    value: json!(until),
                },
            )
            .map_err(|e| e.to_string())?;
    }
    if owner != actor && standing(world, room) >= 2 {
        let day = world.now().0.div_euclid(86_400_000);
        let mut paid = match world
            .store
            .entity(room)
            .and_then(|e| e.components.get(ROOM))
            .and_then(|c| c.fields.get("paid"))
        {
            Some(Field::Plain(Value::Object(m))) => m.clone(),
            _ => serde_json::Map::new(),
        };
        paid.retain(|_, v| v.as_i64() == Some(day));
        let (vday, vn) = match world
            .store
            .entity(citizen)
            .and_then(|e| e.components.get(super::CITIZEN))
            .and_then(|c| c.fields.get("visits"))
        {
            Some(Field::Plain(v)) => (
                v["day"].as_i64().unwrap_or(-1),
                v["n"].as_u64().unwrap_or(0),
            ),
            _ => (-1, 0),
        };
        let n = if vday == day { vn } else { 0 };
        if !paid.contains_key(actor) && n < VISITS_A_DAY {
            paid.insert(actor.to_string(), json!(day));
            let mut f = BTreeMap::new();
            f.insert("paid".to_string(), Field::Plain(Value::Object(paid)));
            set_room(world, room, f)?;
            let mut cf = BTreeMap::new();
            cf.insert(
                "visits".to_string(),
                Field::Plain(json!({"day": day, "n": n + 1})),
            );
            super::set_citizen_fields(world, actor, citizen, cf).map_err(|e| e.to_string())?;
            let rname = plain_str(world, room, ROOM, "name")
                .unwrap_or("a room")
                .to_string();
            let curves = super::Curves::with_half_life(foxing_time::Duration::seconds(3 * 3600));
            let c = super::memory::earn_chunk(
                world,
                actor,
                window,
                &format!("a visit to {rname}, {owner}'s"),
                VISIT_XP,
                0.6,
                &curves,
            )
            .map_err(|e| e.to_string())?;
            said.push(format!(
                "A {} room is worth the walk: chunk-{} worth {VISIT_XP:.0} XP, your first visit here today ({} of {VISITS_A_DAY} such visits a day).",
                STANDINGS[standing(world, room)].to_lowercase(),
                c.0,
                n + 1
            ));
            if let Some(k) = super::find_citizen(world, &owner) {
                let kc = super::memory::earn_chunk(
                    world,
                    &owner,
                    k.window,
                    &format!("{actor} visiting {rname}"),
                    KEEPER_XP,
                    0.6,
                    &curves,
                )
                .map_err(|e| e.to_string())?;
                super::combat::narrate(world, &owner, format!("{actor} came to see {rname}: chunk-{} worth {KEEPER_XP:.0} XP, the keeper's share of a visit.", kc.0));
            }
            let _ = world.note(
                Actor::Citizen(actor.to_string()),
                VISITED,
                json!({"room": room.0, "name": rname, "owner": owner, "visitor": actor, "xp": VISIT_XP, "keeper_xp": KEEPER_XP}),
            );
        }
    }
    Ok(if said.is_empty() {
        None
    } else {
        Some(said.join(" "))
    })
}

/// Whether a glowworm's light is still on a name.
pub fn lit(world: &World, citizen: EntityId) -> bool {
    super::plain_num(world, citizen, super::BODY, "lit_until")
        .is_some_and(|u| u as i64 > world.now().0)
}
