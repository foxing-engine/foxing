//! D34 (David, 11-16): the starfish of the Drowned Wing is its arms.
//!
//! Three names went at it in turn and all three fell: the first took it from 153 to
//! 100, and by keeping what that fall cost it stood at 256 when the second arrived.
//! A boss that grows on every fall cannot be worn down in turns. So its strength is
//! in its arms. It has five, and each is a fifth of its hit points and a point of its
//! blow. What it takes from a name it fells it holds in its arms, a share in each:
//! cut an arm off and that arm's share goes with it, the memory and the hit points
//! and the blow that memory gave. An arm grows back in twenty minutes, plain.
//!
//! Nothing is kept for the arms but a clock: how many it has is where its hit points
//! stand against its whole (four fifths and a little is five arms, one of them cut
//! into). So a starfish that stood before this was written needs nothing done to it.

use super::{plain_num, plain_str, CHUNK, CREATURE};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::CommandError;
use foxing_store::{EntityId, StoreEvent};
use serde_json::json;

pub const KIND: &str = "starfish";
pub const ARMS: usize = 5;
/// One arm back this long after the last was cut or grew, left alone.
pub const ARM_GROWS_MS: i64 = 20 * 60 * 1000;
/// Cut arms lying in a room at most; past that an arm cut is taken by the water.
pub const ARMS_LYING: usize = 3;
/// An arm came off: `{creature, by, arms, shed, room}`. Public.
pub const ARM_CUT: &str = "disremembered.arm_cut";
/// An arm grew back: `{creature, arms}`. Public.
pub const ARM_GREW: &str = "disremembered.arm_grew";

fn internal(e: impl std::fmt::Display) -> CommandError {
    CommandError::internal(e.to_string())
}

pub fn is(world: &World, creature: EntityId) -> bool {
    plain_str(world, creature, CREATURE, "name") == Some(KIND)
}

/// The arms a starfish at `hp` of `max` has: one for every fifth it has any of.
pub fn arms_of(hp: f64, max: f64) -> usize {
    if hp <= 0.0 || max <= 0.0 {
        return 0;
    }
    ((hp / max * ARMS as f64 - 1e-6).ceil() as usize).clamp(1, ARMS)
}

pub fn arms(world: &World, creature: EntityId) -> usize {
    let max = plain_num(world, creature, CREATURE, "max_hp").unwrap_or(1.0);
    arms_of(
        plain_num(world, creature, CREATURE, "hp").unwrap_or(max),
        max,
    )
}

/// What its blow is the less for the arms it lacks: a point an arm (`rescale`).
pub fn blow_lost(world: &World, creature: EntityId) -> f64 {
    if is(world, creature) {
        (ARMS - arms(world, creature).max(1)) as f64
    } else {
        0.0
    }
}

fn words(n: usize) -> &'static str {
    ["no", "one", "two", "three", "four", "five"][n.min(5)]
}

/// For `look`'s row: how many arms it has, when it is a starfish.
pub fn arms_said(world: &World, creature: EntityId) -> String {
    if !is(world, creature) {
        return String::new();
    }
    match arms(world, creature) {
        ARMS => ", all five arms".to_string(),
        1 => ", one arm of five".to_string(),
        n => format!(", {} arms of five", words(n)),
    }
}

fn set(world: &mut World, creature: EntityId, field: &str, value: serde_json::Value) {
    let _ = world.record(
        Actor::Driver,
        StoreEvent::FieldSet {
            id: creature,
            component: CREATURE.into(),
            field: field.into(),
            value,
        },
    );
}

/// A blow has landed on `creature` and its hit points are written. If it is a
/// starfish and the blow took it across a fifth, an arm comes off for each fifth
/// crossed: its share of what the starfish holds is shed, the body is resized to
/// what is left (the wound keeping its proportion, so the arms stand as counted),
/// and the arm lies on the floor. The line to add to the striker's reply.
pub fn after_blow(
    world: &mut World,
    actor: &str,
    creature: EntityId,
    hp_before: f64,
    hp_after: f64,
) -> Result<Option<String>, CommandError> {
    if !is(world, creature) {
        return Ok(None);
    }
    let max = plain_num(world, creature, CREATURE, "max_hp").unwrap_or(1.0);
    let now = world.now().0;
    // The clock an arm grows back by starts at the first wound.
    if hp_before >= max - 0.01 && hp_after < hp_before {
        set(world, creature, "arm_at", json!(now));
    }
    let before = arms_of(hp_before, max);
    let after = arms_of(hp_after, max);
    if after >= before {
        return Ok(None);
    }
    // What the cut arms held: a share each of what it holds.
    let held = super::mind::level(world, creature);
    let keep = after as f64 / before as f64;
    if let Some(window) = super::mind::window_of(world, creature) {
        let chunks: Vec<EntityId> = world
            .store
            .children(window)
            .filter(|e| e.kind == "chunk")
            .map(|e| e.id)
            .collect();
        let at = world.now();
        for id in chunks {
            let v = world
                .store
                .read_field(id, CHUNK, "value", at)
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            if after == 0 || v * keep < super::memory::FORGET_BELOW {
                world
                    .record(Actor::Driver, StoreEvent::EntityRemoved { id })
                    .map_err(internal)?;
            } else {
                world
                    .record(
                        Actor::Driver,
                        StoreEvent::FieldCommitted {
                            id,
                            component: CHUNK.into(),
                            field: "value".into(),
                            value: v * keep,
                        },
                    )
                    .map_err(internal)?;
            }
        }
    }
    let shed = held * (1.0 - keep);
    let room = world.store.entity(creature).and_then(|e| e.parent);
    // The arm, where it fell: a thing to make something of.
    if let Some(room) = room {
        let lying = world
            .store
            .children(room)
            .filter(|e| {
                e.kind == "item"
                    && plain_str(world, e.id, super::craft::ITEM, "name") == Some("a starfish arm")
            })
            .count();
        let curves = super::Curves::from_env();
        for _ in 0..(before - after).min(ARMS_LYING.saturating_sub(lying)) {
            let id = super::craft::make(
                world,
                &curves,
                Actor::Driver,
                None,
                room,
                "a starfish arm",
                "material",
                KIND,
                None,
                &[],
                0,
                false,
            )?;
            super::past::push(
                world,
                "nobody",
                id,
                json!({"what": "left", "kind": KIND, "room": plain_str(world, room, super::ROOM, "name").unwrap_or("nowhere")}),
            )?;
        }
    }
    set(world, creature, "arm_at", json!(now));
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        ARM_CUT,
        json!({"creature": creature.0, "by": actor, "arms": after, "shed": shed, "room": room.map(|r| r.0)}),
    );
    if after == 0 {
        // The last: what follows is the kill, which says the rest.
        let line = "The last arm comes off.".to_string();
        return Ok(Some(line));
    }
    // What it is now: smaller by what the arm held, and a point lighter in the blow.
    super::mind::check_tier(world, creature)?;
    let hp = plain_num(world, creature, CREATURE, "hp").unwrap_or(hp_after);
    let max = plain_num(world, creature, CREATURE, "max_hp").unwrap_or(max);
    let blow = plain_num(world, creature, CREATURE, "attack").unwrap_or(0.0);
    let cut = before - after;
    let mut line = format!(
        "{} off the starfish and {} where {} fell: {} of five left, and its blow is the lighter for it ({blow:.0}\u{2013}{:.0}).",
        if cut == 1 { "An arm comes".to_string() } else { format!("{} arms come", words(cut)) },
        if cut == 1 { "lies" } else { "lie" },
        if cut == 1 { "it" } else { "they" },
        words(after),
        blow + 2.0
    );
    if shed >= 1.0 {
        line.push_str(&format!(
            " What {} held of the fallen goes with {}: {shed:.0} of what it had taken, and the hit points that gave it ({hp:.0} of {max:.0} now).",
            if cut == 1 { "that arm" } else { "they" },
            if cut == 1 { "it" } else { "them" },
        ));
    }
    line.push_str(" An arm is twenty minutes growing back, once it is left alone.");
    super::combat::narrate_others(
        world,
        creature,
        &format!(
            "{actor} takes {} off the starfish: {} of five left.",
            if cut == 1 {
                "an arm".to_string()
            } else {
                format!("{} arms", words(cut))
            },
            words(after)
        ),
        actor,
    );
    Ok(Some(line))
}

/// The creatures' mending pass, for a starfish nobody is fighting: no slow mending
/// of the whole, but an arm back for each twenty minutes since the last was cut or
/// grew. Returns whether one grew.
pub fn regrow(world: &mut World, creature: EntityId) -> Result<bool, String> {
    let max = plain_num(world, creature, CREATURE, "max_hp").unwrap_or(1.0);
    let hp = plain_num(world, creature, CREATURE, "hp").unwrap_or(max);
    if hp >= max - 0.01 {
        return Ok(false);
    }
    let now = world.now().0;
    let Some(at) = plain_num(world, creature, CREATURE, "arm_at").map(|t| t as i64) else {
        set(world, creature, "arm_at", json!(now));
        return Ok(false);
    };
    let n = (now - at) / ARM_GROWS_MS;
    if n < 1 {
        return Ok(false);
    }
    let after = (hp + n as f64 * max / ARMS as f64).min(max);
    set(
        world,
        creature,
        "hp",
        json!(((after * 10.0).round() / 10.0).min(max)),
    );
    set(world, creature, "arm_at", json!(at + n * ARM_GROWS_MS));
    super::combat::rescale(world, creature).map_err(|e| e.to_string())?;
    let arms = arms(world, creature);
    let _ = world.note(
        Actor::Driver,
        ARM_GREW,
        json!({"creature": creature.0, "arms": arms}),
    );
    let who = super::mind::display(world, creature);
    let who = if super::mind::name(world, creature).is_some() {
        who
    } else {
        format!("The {who}")
    };
    super::combat::narrate_room(
        world,
        creature,
        &format!(
            "{who} has an arm back, pale and a little short: {} of five.",
            words(arms)
        ),
    );
    Ok(true)
}

/// How long until its next arm, if it lacks any of itself: for `retreat`'s line.
pub fn next_arm_in(world: &World, creature: EntityId) -> Option<i64> {
    if !is(world, creature) {
        return None;
    }
    let max = plain_num(world, creature, CREATURE, "max_hp")?;
    let hp = plain_num(world, creature, CREATURE, "hp")?;
    if hp >= max - 0.01 {
        return None;
    }
    let at = plain_num(world, creature, CREATURE, "arm_at")? as i64;
    Some((at + ARM_GROWS_MS - world.now().0).max(0))
}
