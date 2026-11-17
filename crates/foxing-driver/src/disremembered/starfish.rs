//! D34 (David, 11-16): the starfish of the Drowned Wing is its arms.
//!
//! Three names went at it in turn and all three fell: the first took it from 153 to
//! 100, and by keeping what that fall cost it stood at 256 when the second arrived.
//! A boss that grows on every fall cannot be worn down in turns. So its strength is
//! in its arms. It has five, and each is a fifth of its plain hit points and a point
//! of its blow.
//!
//! D44 (David, 11-17: "recalculate xp per arm after a fall and distribute the new
//! memory from a fall to an arm"): each arm keeps its own tally. What it takes from
//! a name it fells goes into ONE arm, the arm that will come off next, and that arm
//! is the thicker for it: the hit points the memory gives are that arm's. Cut the
//! arm and all of it goes, the memory and the hit points. (The party of 11-17 cut
//! three arms that shed 14 each and then fell five times: it went from holding 70 to
//! 165 with two arms, every arm fatter, and came apart holding 339.) What it holds
//! that was not taken in a fall (a story told to it, a word carried) is in all its
//! arms alike, a share to each, as before.
//!
//! What is kept: how many arms it has, how much of the leading arm is left, a clock
//! for the growing back, and on each memory taken in a fall the arm it went into.
//! Its hit points are worked out from those and written beside them, so `look` and
//! the fight read one figure. The arms are numbered from the bottom: a cut takes
//! the top one, and an arm that grows back grows at the bottom, plain and holding
//! nothing, so the leading arm stays the leading arm with its wound and its tally.
//! A starfish that stood before the tallies were kept has none, and is read by
//! fifths of its hit points, as it was.

use super::{plain_num, plain_str, CHUNK, CREATURE};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::CommandError;
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::json;
use std::collections::BTreeMap;

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
/// What a creature holds against its body: a level of this doubles it (`rescale`).
const LEVELS_A_BODY: f64 = 150.0;
/// What a creature holds against its blow: a point for each of these (`rescale`).
const LEVELS_A_POINT: f64 = 75.0;

fn internal(e: impl std::fmt::Display) -> CommandError {
    CommandError::internal(e.to_string())
}

pub fn is(world: &World, creature: EntityId) -> bool {
    plain_str(world, creature, CREATURE, "name") == Some(KIND)
}

/// The arms a starfish at `hp` of `max` has when no tally is kept for it: one for
/// every fifth it has any of.
pub fn arms_of(hp: f64, max: f64) -> usize {
    if hp <= 0.0 || max <= 0.0 {
        return 0;
    }
    ((hp / max * ARMS as f64 - 1e-6).ceil() as usize).clamp(1, ARMS)
}

/// Where its arms stand: the number of the bottom one, how many it has, and how
/// much of the leading (top) one is left, of one.
#[derive(Clone, Copy, Debug)]
struct State {
    lo: i64,
    n: usize,
    fill: f64,
}

impl State {
    fn top(&self) -> i64 {
        self.lo + self.n as i64 - 1
    }
    fn whole(&self) -> bool {
        self.n == ARMS && self.fill >= 1.0 - 1e-9
    }
}

fn state(world: &World, creature: EntityId) -> State {
    state_at(world, creature, None)
}

/// [`state`], reading a starfish with no tally by the hit points it had at `hp`
/// (a blow's are written before its arms are asked after).
fn state_at(world: &World, creature: EntityId, hp: Option<f64>) -> State {
    if let Some(n) = plain_num(world, creature, CREATURE, "arms") {
        return State {
            lo: plain_num(world, creature, CREATURE, "arm_lo").unwrap_or(0.0) as i64,
            n: (n as usize).min(ARMS),
            fill: plain_num(world, creature, CREATURE, "arm_fill")
                .unwrap_or(1.0)
                .clamp(0.0, 1.0),
        };
    }
    // No tally kept yet: by fifths of what it has.
    let max = plain_num(world, creature, CREATURE, "max_hp").unwrap_or(1.0);
    let hp = hp
        .or_else(|| plain_num(world, creature, CREATURE, "hp"))
        .unwrap_or(max);
    let n = arms_of(hp, max);
    let fifth = max / ARMS as f64;
    let fill = if n == 0 || fifth <= 0.0 {
        1.0
    } else {
        ((hp - (n as f64 - 1.0) * fifth) / fifth).clamp(0.0, 1.0)
    };
    State { lo: 0, n, fill }
}

/// What it holds, a memory at a time: the chunk, its worth now, and the arm it is
/// in, when it was taken in a fall.
fn held(world: &World, creature: EntityId) -> Vec<(EntityId, f64, Option<i64>)> {
    let Some(window) = super::mind::window_of(world, creature) else {
        return Vec::new();
    };
    let at = world.now();
    world
        .store
        .children(window)
        .filter(|e| e.kind == "chunk")
        .map(|e| {
            (
                e.id,
                world
                    .store
                    .read_field(e.id, CHUNK, "value", at)
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                plain_num(world, e.id, CHUNK, "arm").map(|a| a as i64),
            )
        })
        .collect()
}

fn base(world: &World, creature: EntityId) -> f64 {
    plain_num(world, creature, CREATURE, "base_hp")
        .or_else(|| plain_num(world, creature, CREATURE, "max_hp"))
        .or_else(|| super::combat::def_by_name(KIND).map(|d| d.hp))
        .unwrap_or(160.0)
}

/// How big its arms are: a plain arm (a fifth of its base, and of what it holds in
/// all its arms alike), and for each arm with a tally what the tally adds to it.
fn sizes(world: &World, creature: EntityId) -> (f64, BTreeMap<i64, f64>) {
    let base = base(world, creature);
    let mut shared = 0.0;
    let mut extra: BTreeMap<i64, f64> = BTreeMap::new();
    for (_, v, arm) in held(world, creature) {
        match arm {
            Some(t) => *extra.entry(t).or_default() += base * v / LEVELS_A_BODY,
            None => shared += v,
        }
    }
    (base / ARMS as f64 * (1.0 + shared / LEVELS_A_BODY), extra)
}

fn size(plain: f64, extra: &BTreeMap<i64, f64>, arm: i64) -> f64 {
    plain + extra.get(&arm).copied().unwrap_or(0.0)
}

pub fn arms(world: &World, creature: EntityId) -> usize {
    state(world, creature).n
}

/// What the leading arm holds of the fallen: its own tally.
pub fn leading_holds(world: &World, creature: EntityId) -> f64 {
    let st = state(world, creature);
    if st.n == 0 {
        return 0.0;
    }
    let top = st.top();
    held(world, creature)
        .iter()
        .filter(|(_, _, arm)| *arm == Some(top))
        .map(|(_, v, _)| v)
        .sum()
}

fn words(n: usize) -> &'static str {
    ["no", "one", "two", "three", "four", "five"][n.min(5)]
}

/// For `look`'s row: how many arms it has, and what the leading one holds of the
/// fallen, when it is a starfish.
pub fn arms_said(world: &World, creature: EntityId) -> String {
    if !is(world, creature) {
        return String::new();
    }
    let n = match arms(world, creature) {
        ARMS => ", all five arms".to_string(),
        1 => ", one arm of five".to_string(),
        n => format!(", {} arms of five", words(n)),
    };
    let leading = leading_holds(world, creature);
    if leading >= 1.0 {
        format!("{n}, the leading one thick with {leading:.0} of what it took")
    } else {
        n
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

/// Its body, worked out from its arms and what they hold, and written: the whole it
/// would be with all five (`max_hp`), what it has (`hp`), its blow, what it holds.
/// `combat::rescale` for a starfish. `to` is where its arms now stand, when that
/// has just changed; else as kept.
fn write(world: &mut World, creature: EntityId, to: Option<State>) -> Result<(), CommandError> {
    let kept = plain_num(world, creature, CREATURE, "arms").is_some();
    let st = to.unwrap_or_else(|| state(world, creature));
    let (plain, extra) = sizes(world, creature);
    let level = super::mind::level(world, creature);
    let max = (ARMS as f64 * plain + extra.values().sum::<f64>()).round();
    let hp = if st.n == 0 {
        0.0
    } else if st.whole() {
        max
    } else {
        let below: f64 = (0..st.n as i64 - 1)
            .map(|j| size(plain, &extra, st.lo + j))
            .sum();
        let hp = below + st.fill * size(plain, &extra, st.top());
        // Short of whole reads short of whole, and standing reads standing.
        ((hp * 10.0).round() / 10.0).clamp(0.1, (max - 0.1).max(0.1))
    };
    let d_attack = super::combat::def_by_name(KIND).map_or(9.0, |d| d.attack);
    let attack = (d_attack + (level / LEVELS_A_POINT).floor()
        - (ARMS - st.n.clamp(1, ARMS)) as f64)
        .max(1.0);
    let holding = level.round();
    let same = |field: &str, v: f64| {
        plain_num(world, creature, CREATURE, field).is_some_and(|x| (x - v).abs() < 0.05)
    };
    if kept
        && to.is_none()
        && plain_num(world, creature, CREATURE, "base_hp").is_some()
        && same("max_hp", max)
        && same("hp", hp)
        && same("attack", attack)
        && same("holding", holding)
    {
        return Ok(());
    }
    let mut changes = BTreeMap::new();
    changes.insert("base_hp".into(), Field::Plain(json!(base(world, creature))));
    changes.insert("max_hp".into(), Field::Plain(json!(max)));
    changes.insert("hp".into(), Field::Plain(json!(hp)));
    changes.insert("attack".into(), Field::Plain(json!(attack)));
    changes.insert("holding".into(), Field::Plain(json!(holding)));
    changes.insert("arms".into(), Field::Plain(json!(st.n)));
    changes.insert("arm_fill".into(), Field::Plain(json!(st.fill)));
    changes.insert("arm_lo".into(), Field::Plain(json!(st.lo)));
    super::bindery::set_fields(world, Actor::Driver, creature, CREATURE, changes).map(|_| ())
}

/// `combat::rescale` for a starfish: what it holds has changed (a fall, a story, the
/// fading), and its body follows, each arm by its own tally.
pub fn resize(world: &mut World, creature: EntityId) -> Result<(), CommandError> {
    write(world, creature, None)
}

/// It has felled a name, and `chunk` is what it took: into the leading arm with it.
/// The hit points that adds to that arm, for the lines that say so.
pub fn took(world: &mut World, creature: EntityId, chunk: EntityId) -> Result<f64, CommandError> {
    if !is(world, creature) {
        return Ok(0.0);
    }
    let st = state(world, creature);
    if st.n == 0 {
        return Ok(0.0);
    }
    world
        .record(
            Actor::Driver,
            StoreEvent::FieldSet {
                id: chunk,
                component: CHUNK.into(),
                field: "arm".into(),
                value: json!(st.top()),
            },
        )
        .map_err(internal)?;
    let worth = world
        .store
        .read_field(chunk, CHUNK, "value", world.now())
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    Ok(base(world, creature) * worth / LEVELS_A_BODY * st.fill)
}

/// What a fall to it says, to the one who fell and to the room: where the memory
/// went. `thicker` is from [`took`].
pub fn took_said(thicker: f64) -> String {
    format!(
        "What it took went into the arm it struck with, the one that comes off next: that arm is the thicker by {thicker:.0} hit points, and all of it goes when the arm is cut."
    )
}

/// An arm comes off: what it holds by its own tally goes whole, and of what the
/// starfish holds in all its arms alike, this arm's share. What was shed.
fn shed(world: &mut World, creature: EntityId, st: State) -> Result<f64, CommandError> {
    let top = st.top();
    let keep = (st.n as f64 - 1.0) / st.n as f64;
    let mut gone = 0.0;
    for (id, v, arm) in held(world, creature) {
        match arm {
            // Its own (or one left on an arm that is not there).
            Some(t) if t >= top => {
                world
                    .record(Actor::Driver, StoreEvent::EntityRemoved { id })
                    .map_err(internal)?;
                gone += v;
            }
            Some(_) => {}
            None if st.n == 1 || v * keep < super::memory::FORGET_BELOW => {
                world
                    .record(Actor::Driver, StoreEvent::EntityRemoved { id })
                    .map_err(internal)?;
                gone += v;
            }
            None => {
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
                gone += v * (1.0 - keep);
            }
        }
    }
    Ok(gone)
}

/// A blow has landed on `creature` and its hit points are written, `hp_before` to
/// `hp_after`. For a starfish the blow is taken by the leading arm; when that arm
/// has no more to give it comes off, what it held goes with it, and what is left
/// of the blow goes on into the next. Its hit points are then written again from
/// its arms (read them back). The line to add to the striker's reply, when an arm
/// came off.
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
    let mut dmg = (hp_before - hp_after).max(0.0);
    if dmg <= 0.0 {
        return Ok(None);
    }
    let mut st = state_at(world, creature, Some(hp_before));
    let now = world.now().0;
    // The clock an arm grows back by starts at the first wound.
    if st.whole() {
        set(world, creature, "arm_at", json!(now));
    }
    let before = st.n;
    let mut shed_all = 0.0;
    while st.n > 0 {
        let (plain, extra) = sizes(world, creature);
        let whole = size(plain, &extra, st.top());
        let left = st.fill * whole;
        if dmg < left - 1e-6 {
            st.fill = ((left - dmg) / whole).clamp(0.0, 1.0);
            break;
        }
        dmg -= left;
        shed_all += shed(world, creature, st)?;
        st.n -= 1;
        st.fill = 1.0;
        if dmg <= 1e-6 {
            break;
        }
    }
    let after = st.n;
    write(world, creature, Some(st))?;
    if after >= before {
        return Ok(None);
    }
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
        json!({"creature": creature.0, "by": actor, "arms": after, "shed": shed_all, "room": room.map(|r| r.0)}),
    );
    if after == 0 {
        // The last: what follows is the kill, which says the rest.
        return Ok(Some("The last arm comes off.".to_string()));
    }
    // What it is now: smaller by what the arm held, and a point lighter in the blow.
    super::mind::check_tier(world, creature)?;
    let hp = plain_num(world, creature, CREATURE, "hp").unwrap_or(hp_after);
    let max = plain_num(world, creature, CREATURE, "max_hp").unwrap_or(hp);
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
    if shed_all >= 1.0 {
        line.push_str(&format!(
            " What {} held of the fallen goes with {}: {shed_all:.0} of what it had taken, and the hit points that gave it ({hp:.0} of {max:.0} now).",
            if cut == 1 { "that arm" } else { "they" },
            if cut == 1 { "it" } else { "them" },
        ));
    }
    let leading = leading_holds(world, creature);
    if leading >= 1.0 {
        line.push_str(&format!(
            " The arm behind it leads now, thick with {leading:.0} of its own."
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
/// of the whole, but for each twenty minutes since the last was cut or grew, an arm
/// back at the bottom, plain; and with all five, the cut in the leading one closed.
/// Returns whether anything grew.
pub fn regrow(world: &mut World, creature: EntityId) -> Result<bool, String> {
    let mut st = state(world, creature);
    if st.whole() || st.n == 0 {
        return Ok(false);
    }
    let now = world.now().0;
    let Some(at) = plain_num(world, creature, CREATURE, "arm_at").map(|t| t as i64) else {
        set(world, creature, "arm_at", json!(now));
        return Ok(false);
    };
    let steps = (now - at) / ARM_GROWS_MS;
    if steps < 1 {
        return Ok(false);
    }
    let mut grew = 0;
    for _ in 0..steps {
        if st.n < ARMS {
            st.lo -= 1;
            st.n += 1;
            grew += 1;
        } else {
            st.fill = 1.0;
            break;
        }
    }
    set(world, creature, "arm_at", json!(at + steps * ARM_GROWS_MS));
    write(world, creature, Some(st)).map_err(|e| e.to_string())?;
    let who = super::mind::display(world, creature);
    let who = if super::mind::name(world, creature).is_some() {
        who
    } else {
        format!("The {who}")
    };
    if grew > 0 {
        let _ = world.note(
            Actor::Driver,
            ARM_GREW,
            json!({"creature": creature.0, "arms": st.n}),
        );
        super::combat::narrate_room(
            world,
            creature,
            &format!(
                "{who} has an arm back, pale and a little short: {} of five.",
                words(st.n)
            ),
        );
    } else {
        super::combat::narrate_room(
            world,
            creature,
            &format!("{who} is whole again: the cut in its leading arm has closed."),
        );
    }
    Ok(true)
}

/// How long until its next arm, if it lacks any of itself: for `retreat`'s line.
pub fn next_arm_in(world: &World, creature: EntityId) -> Option<i64> {
    if !is(world, creature) || state(world, creature).whole() {
        return None;
    }
    let at = plain_num(world, creature, CREATURE, "arm_at")? as i64;
    Some((at + ARM_GROWS_MS - world.now().0).max(0))
}

/// On boot: a room keeps one starfish. Where a second grew beside one that had
/// taken a name (the Court, 11-16), the unnamed one that nobody is fighting goes.
pub fn settle(world: &mut World) -> Result<usize, String> {
    let mut by_room: BTreeMap<EntityId, Vec<EntityId>> = Default::default();
    for e in world.store.of_kind("creature") {
        if plain_str(world, e.id, CREATURE, "name") == Some(KIND) {
            if let Some(room) = e.parent {
                by_room.entry(room).or_default().push(e.id);
            }
        }
    }
    let mut gone = 0;
    for (_, mut stars) in by_room {
        if stars.len() < 2 {
            continue;
        }
        // The one to keep: a named one, else the first.
        stars.sort_by_key(|id| super::mind::name(world, *id).is_none());
        for id in stars.into_iter().skip(1) {
            if super::mind::name(world, id).is_some()
                || super::combat::fought_by(world, id).is_some()
            {
                continue;
            }
            world
                .record(Actor::Driver, StoreEvent::EntityRemoved { id })
                .map_err(|e| e.to_string())?;
            gone += 1;
        }
    }
    Ok(gone)
}
