//! Pilot names near the door (D14, R42).
//!
//! Every Night's Watch arrived to a nameless world: the names made in one session
//! faded before the next, and a visitor who does not fight had no one to tell a
//! story to. The Docent keeps three named creatures within two rooms of the
//! Threshold. Every ten minutes it counts them; when fewer stand, it feeds a mild
//! unnamed creature nearby to its name with the world's own memories (things people
//! here have forgotten), and when one of its three is failing it feeds it back up.
//! The Docent is on the record as the keeper, so `look` says "kept by Hollis Quire",
//! and a creature it keeps asks for stories like any other.
//!
//! Nothing a player names is touched: the three are counted among every name near
//! the door, so a well-kept hub needs no pilot at all.

use super::mind;
use crate::world::World;
use foxing_log::Actor;
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const JOB_PILOTS: &str = "disremembered.pilots";
/// The Docent named or fed a pilot: `{creature, kind, name, room, why}`. Public.
pub const PILOTED: &str = "disremembered.piloted";
const EVERY_MS: i64 = 600_000;
/// How many named creatures the door keeps within reach.
pub const KEEP: usize = 3;
/// How far from the Threshold "near" reaches, in rooms.
pub const REACH: usize = 2;
/// What a pilot is fed to: the naming bar and a margin.
const FED_TO: f64 = 75.0;
/// The mild kinds the Docent will name, in order of preference.
const MILD: &[&str] = &["wisp", "mothwing", "ragpicker", "inkling"];
/// What the world's own memories are of.
const FORGOTTEN: &[&str] = &[
    "the way home",
    "a second verse",
    "a door that was bricked up",
    "someone's name",
    "the smell of rain",
    "a key to nothing",
    "the last line of a story",
    "a face in a window",
    "the night reader's lamp",
    "the water coming in",
];

/// The rooms within `REACH` of the Threshold, by the map's exits.
pub fn near_the_door(world: &World) -> Vec<EntityId> {
    let Some(start) = super::rooms::arrival(world) else {
        return Vec::new();
    };
    let mut seen: BTreeSet<EntityId> = BTreeSet::new();
    let mut queue: VecDeque<(EntityId, usize)> = VecDeque::new();
    seen.insert(start);
    queue.push_back((start, 0));
    while let Some((room, d)) = queue.pop_front() {
        if d >= REACH {
            continue;
        }
        for (_, to) in super::rooms::exits_of(world, room) {
            if world.store.entity(to).is_some() && seen.insert(to) {
                queue.push_back((to, d + 1));
            }
        }
    }
    seen.into_iter().collect()
}

/// The named creatures standing near the door, failing ones marked.
fn named_near(world: &World, rooms: &[EntityId]) -> Vec<(EntityId, f64)> {
    rooms
        .iter()
        .flat_map(|&r| super::combat::creatures_in(world, r))
        .filter(|c| mind::name(world, c.0).is_some())
        .map(|c| (c.0, mind::level(world, c.0)))
        .collect()
}

/// A mild, unnamed, unborn creature near the door, by preference of kind. A room is
/// left its stock: the Docent never names the last unnamed creature of a room that
/// spawns its kind, so a newcomer sent east to hit a wisp finds one to hit.
fn candidate(world: &World, rooms: &[EntityId]) -> Option<EntityId> {
    let all: Vec<(EntityId, String, EntityId)> = rooms
        .iter()
        .flat_map(|&r| {
            super::combat::creatures_in(world, r)
                .into_iter()
                .map(move |c| (c.0, c.1.clone(), r))
        })
        .filter(|c| mind::name(world, c.0).is_none())
        .filter(|c| super::kindle::born_at(world, c.0).is_none())
        .filter(|c| !mind::shaken(world, c.0))
        .filter(|(_, kind, room)| {
            let spawns_kind =
                super::plain_str(world, *room, super::SPAWN, "creature") == Some(kind);
            let unnamed_of_kind = super::combat::creatures_in(world, *room)
                .iter()
                .filter(|o| o.1 == *kind && mind::name(world, o.0).is_none())
                .count();
            !spawns_kind || unnamed_of_kind >= 2
        })
        .collect();
    MILD.iter()
        .find_map(|kind| all.iter().find(|(_, k, _)| k == kind).map(|(id, _, _)| *id))
}

/// Feed a creature with the world's own memories until it holds `to`, in the
/// Docent's name.
fn feed(world: &mut World, creature: EntityId, to: f64) -> Result<f64, String> {
    let curves = super::Curves::from_env();
    let window = mind::ensure_window_pub(world, creature).map_err(|e| e.to_string())?;
    let mut fed = 0.0;
    let mut guard = 0;
    while mind::level(world, creature) < to && guard < 6 {
        guard += 1;
        let what = FORGOTTEN[world.roll(FORGOTTEN.len() as u32) as usize];
        let size = (to - mind::level(world, creature)).clamp(10.0, 40.0);
        let chunk =
            super::memory::earn_chunk(world, super::docent::NAME, window, what, size, 0.9, &curves)
                .map_err(|e| e.to_string())?;
        mind::disown(world, chunk).map_err(|e| e.to_string())?;
        fed += size;
        let mut kept = mind::kept(world, creature);
        kept.push(json!({"by": super::docent::NAME, "with": what}));
        let mut changes = BTreeMap::new();
        changes.insert("kept".into(), Field::Plain(Value::Array(kept)));
        changes.insert("pilot".into(), Field::Plain(json!(true)));
        mind::set_mind_pub(world, Actor::Driver, creature, changes).map_err(|e| e.to_string())?;
    }
    mind::check_tier(world, creature).map_err(|e| e.to_string())?;
    Ok(fed)
}

/// One pass: count the names near the door; feed the failing; name a mild stranger
/// when fewer than `KEEP` stand.
pub fn tend(world: &mut World) -> Result<usize, String> {
    let rooms = near_the_door(world);
    let mut did = 0;
    // The failing among the Docent's own: fed back up (a player's name is theirs to tend).
    for (c, lvl) in named_near(world, &rooms) {
        let pilot = mind::is_pilot(world, c);
        if pilot && lvl < mind::FAILING_BELOW {
            let fed = feed(world, c, FED_TO)?;
            record(world, c, "fed", fed)?;
            did += 1;
        }
    }
    let mut standing = named_near(world, &rooms).len();
    let mut guard = 0;
    while standing < KEEP && guard < KEEP {
        guard += 1;
        let Some(c) = candidate(world, &rooms) else {
            break;
        };
        let fed = feed(world, c, FED_TO)?;
        record(world, c, "named", fed)?;
        did += 1;
        standing = named_near(world, &rooms).len();
    }
    Ok(did)
}

fn record(world: &mut World, creature: EntityId, why: &str, fed: f64) -> Result<(), String> {
    let kind = super::plain_str(world, creature, super::CREATURE, "name")
        .unwrap_or("?")
        .to_string();
    let name = mind::name(world, creature);
    let room = world.store.entity(creature).and_then(|e| e.parent);
    let room_name = room
        .and_then(|r| super::plain_str(world, r, super::ROOM, "name"))
        .unwrap_or("?")
        .to_string();
    let _ = world.note(
        Actor::Driver,
        PILOTED,
        json!({"creature": creature.0, "kind": kind, "name": name, "room": room.map(|r| r.0), "why": why, "fed": fed}),
    );
    world.notice(format!(
        "The Docent {} **{} the {kind}** in {room_name} ({fed:.0} of the world's own memory).",
        if why == "named" { "named" } else { "fed" },
        name.as_deref().unwrap_or("?")
    ));
    if why == "fed" {
        super::combat::narrate_room(
            world,
            creature,
            &format!(
                "The Docent stops by {} the {kind} and says something low; it straightens.",
                name.as_deref().unwrap_or("?")
            ),
        );
    }
    Ok(())
}

fn wind(world: &mut World) -> Result<(), String> {
    let due = world.now().plus(foxing_time::Duration(EVERY_MS));
    world
        .schedule_once(JOB_PILOTS, due, json!({}), |_| true)
        .map(|_| ())
}

/// Whether the Docent tends the door at all: on in the world, off in the tests
/// that want it nameless (they call `tend` themselves).
pub static ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub fn ensure(world: &mut World) -> Result<(), String> {
    if !ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }
    if !world.jobs.has(JOB_PILOTS, |_| true) {
        wind(world)?;
    }
    Ok(())
}

pub fn register(world: &mut World) {
    world.on_job(
        JOB_PILOTS,
        Box::new(|w: &mut World, _| {
            tend(w)?;
            wind(w)
        }),
    );
}
