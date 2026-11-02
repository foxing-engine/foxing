//! Encounters: one round per `attack`, and the creature's own strikes on the scheduler
//! while you stay engaged, so a fight has a clock. Aggressive creatures ambush whoever
//! walks in. Rooms refill on respawn jobs. What happens to you while you are not typing
//! arrives as narrative events.
//!
//! Your attack follows your level, so a faded window hits softer. A win at low hit points
//! earns a chunk with high vividity, and vivid chunks fade slower: the fights you almost
//! lost last longest. Death is a concussion, a chance to lose one chunk from the window,
//! and you wake at the Threshold.

use super::{internal, me, plain_num, plain_str, Curves, BODY, CHUNK, CREATURE, SPAWN};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreError, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A kind of creature. Hit points, damage per round, and the XP a chunk is worth.
pub struct CreatureDef {
    pub key: &'static str,
    pub name: &'static str,
    pub hp: f64,
    pub attack: f64,
    pub xp: f64,
    /// Attacks whoever walks in, a few seconds after they arrive.
    pub aggressive: bool,
    /// Seconds between its own strikes while engaged.
    pub strike_seconds: i64,
    /// How it announces itself when it ambushes.
    pub ambush: &'static str,
    /// How a renderer draws it.
    pub glyph: &'static str,
    pub tint: &'static str,
}

pub const CREATURES: &[CreatureDef] = &[
    CreatureDef {
        key: "wisp",
        name: "wisp",
        hp: 8.0,
        attack: 2.0,
        xp: 15.0,
        aggressive: false,
        strike_seconds: 12,
        ambush: "drifts closer.",
        glyph: "w",
        tint: "#9fd3e6",
    },
    CreatureDef {
        key: "hollow",
        name: "hollow",
        hp: 20.0,
        attack: 4.0,
        xp: 60.0,
        aggressive: true,
        strike_seconds: 10,
        ambush: "stands up out of the water and comes at you.",
        glyph: "H",
        tint: "#6f8f7a",
    },
    CreatureDef {
        key: "revenant",
        name: "revenant",
        hp: 45.0,
        attack: 7.0,
        xp: 200.0,
        aggressive: true,
        strike_seconds: 8,
        ambush: "steps out of a niche. It has been waiting.",
        glyph: "R",
        tint: "#c96a6a",
    },
    CreatureDef {
        key: "inkling",
        name: "inkling",
        hp: 10.0,
        attack: 2.0,
        xp: 20.0,
        aggressive: false,
        strike_seconds: 10,
        ambush: "peels off a page and flutters at your face.",
        glyph: "i",
        tint: "#7a7fd1",
    },
    CreatureDef {
        key: "mothwing",
        name: "mothwing",
        hp: 6.0,
        attack: 1.0,
        xp: 10.0,
        aggressive: false,
        strike_seconds: 14,
        ambush: "brushes past your ear, heavy as a hand.",
        glyph: "m",
        tint: "#d8d0a0",
    },
    CreatureDef {
        key: "ragpicker",
        name: "ragpicker",
        hp: 16.0,
        attack: 3.0,
        xp: 40.0,
        aggressive: false,
        strike_seconds: 11,
        ambush: "looks up from the heap, then back down, then up again, closer.",
        glyph: "r",
        tint: "#a08a60",
    },
    CreatureDef {
        key: "drowned",
        name: "drowned",
        hp: 26.0,
        attack: 5.0,
        xp: 80.0,
        aggressive: true,
        strike_seconds: 10,
        ambush: "surfaces beside you without a ripple and takes hold.",
        glyph: "D",
        tint: "#4f8a9e",
    },
    CreatureDef {
        key: "sitter",
        name: "sitter",
        hp: 32.0,
        attack: 5.0,
        xp: 120.0,
        aggressive: true,
        strike_seconds: 9,
        ambush: "turns around in its frame, and then steps out of it.",
        glyph: "S",
        tint: "#b07aa0",
    },
    CreatureDef {
        key: "hourhand",
        name: "hourhand",
        hp: 38.0,
        attack: 6.0,
        xp: 150.0,
        aggressive: true,
        strike_seconds: 8,
        ambush: "unfolds from the works with a sound like a key turning.",
        glyph: "T",
        tint: "#d1a04a",
    },
    CreatureDef {
        key: "keeper",
        name: "keeper",
        hp: 70.0,
        attack: 9.0,
        xp: 400.0,
        aggressive: true,
        strike_seconds: 7,
        ambush: "stands. It does not remember you. It never needed to.",
        glyph: "K",
        tint: "#e0e0ee",
    },
    // The Lower Stacks, under the Archive: higher things, each with a trick of its
    // own (stacks.rs has the tricks).
    CreatureDef {
        key: "lamprey",
        name: "lamprey",
        hp: 45.0,
        attack: 7.0,
        xp: 180.0,
        aggressive: true,
        strike_seconds: 9,
        ambush: "unfastens from the underside of a shelf and fastens on you.",
        glyph: "l",
        tint: "#9aa878",
    },
    CreatureDef {
        key: "usher",
        name: "usher",
        hp: 60.0,
        attack: 8.0,
        xp: 240.0,
        aggressive: true,
        strike_seconds: 10,
        ambush: "is at your elbow in a waistcoat. \"This way, please.\"",
        glyph: "U",
        tint: "#b8a888",
    },
    CreatureDef {
        key: "censor",
        name: "censor",
        hp: 70.0,
        attack: 9.0,
        xp: 320.0,
        aggressive: true,
        strike_seconds: 9,
        ambush: "looks up from the desk and raises the stamp.",
        glyph: "C",
        tint: "#a84848",
    },
    CreatureDef {
        key: "cataloguer",
        name: "cataloguer",
        hp: 110.0,
        attack: 13.0,
        xp: 600.0,
        aggressive: false,
        strike_seconds: 8,
        ambush: "turns a page. It has your entry open.",
        glyph: "Q",
        tint: "#e8e0c0",
    },
    // The Conclave's clerks and its Sergeant of the Bar. Neither starts a fight.
    CreatureDef {
        key: "engrosser",
        name: "engrosser",
        hp: 80.0,
        attack: 7.0,
        xp: 320.0,
        aggressive: false,
        strike_seconds: 10,
        ambush: "looks up from the minutes, dips its pen, and writes down that you have arrived.",
        glyph: "E",
        tint: "#2a2a3a",
    },
    CreatureDef {
        key: "tipstaff",
        name: "tipstaff",
        hp: 140.0,
        attack: 10.0,
        xp: 600.0,
        aggressive: false,
        strike_seconds: 9,
        ambush: "lowers its staff across the rail and asks, in a voice made for a much larger room, whether you have business before the Conclave.",
        glyph: "T",
        tint: "#c0a060",
    },
];

/// Base damage per round before the level bonus.
const BASE_ATTACK: f64 = 3.0;
/// Attack grows with the square root of level: 3 at nothing, 13 at 100, 20 at 300,
/// 35 at 1,000, 48 at 2,000, 80 at 6,000. It was a tenth of level, and a name with a
/// few hundred XP felled everything in the Lower Stacks in one hit; a hoard of six
/// thousand would have hit for six hundred. The curve keeps the first hundred XP
/// as they were and bends the rest.
const ATTACK_PER_ROOT_LEVEL: f64 = 1.0;
/// On death, the share of the window that goes, on a curve of level: half of
/// `level / (level + DEATH_SCALE)`, so a name holding ten loses almost nothing (a
/// newcomer may zombie-rush a censor once), one holding three hundred loses a quarter,
/// a thousand a third, and the share tends to half. It was a coin flip for one chunk
/// at any level, and death cost a 2,000-XP delver nothing it noticed.
pub const DEATH_SHARE_MAX: f64 = 0.5;
pub const DEATH_SCALE: f64 = 300.0;
/// Something was lost to a death: `{citizen, lost: [{chunk, source, value}], share}`.
pub const DIED: &str = "disremembered.died";

/// The share of the window a death costs at this level.
pub fn death_share(level: f64) -> f64 {
    DEATH_SHARE_MAX * level.max(0.0) / (level.max(0.0) + DEATH_SCALE)
}
/// Seconds between arriving in a room and an aggressive creature noticing you.
/// An ambush comes this many seconds after an arrival, at the earliest and latest.
pub const AMBUSH_SECONDS_LOW: i64 = 2;
pub const AMBUSH_SECONDS_HIGH: i64 = 8;
/// A respawn lands between these fractions of the room's figure.
pub const RESPAWN_LOW: f64 = 0.6;
pub const RESPAWN_HIGH: f64 = 1.6;
/// A creature's own strike lands between these fractions of its figure.
pub const STRIKE_LOW: f64 = 0.5;
pub const STRIKE_HIGH: f64 = 1.5;

pub const JOB_RESPAWN: &str = "disremembered.respawn";
pub const JOB_STRIKE: &str = "disremembered.strike";
pub const JOB_AMBUSH: &str = "disremembered.ambush";
pub const NARRATIVE: &str = "disremembered.narrative";

/// Something that happened to a citizen while they were not typing. The text port
/// prints it after their next command; the browser client polls for it.
pub fn narrate(world: &mut World, actor: &str, text: impl Into<String>) {
    let _ = world.note(
        Actor::Driver,
        NARRATIVE,
        json!({"to": actor, "text": text.into()}),
    );
}

/// Something happened near `entity` that everyone present in its room should hear.
pub fn narrate_room(world: &mut World, entity: EntityId, text: &str) {
    narrate_room_but(world, entity, text, None);
}

/// As [`narrate_room`], but not to `except`: one's own act is in the reply already,
/// and "Thimbleweed picks up grave dust" told to Thimbleweed read as someone else.
pub fn narrate_others(world: &mut World, entity: EntityId, text: &str, except: &str) {
    narrate_room_but(world, entity, text, Some(except));
}

fn narrate_room_but(world: &mut World, entity: EntityId, text: &str, except: Option<&str>) {
    let Some(room) = world.store.entity(entity).and_then(|e| e.parent) else {
        return;
    };
    let present: Vec<String> = world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .filter(|o| except != Some(o.as_str()))
        .collect();
    for actor in present {
        narrate(world, &actor, text.to_string());
    }
}

fn creature_def(world: &World, creature: EntityId) -> Option<&'static CreatureDef> {
    plain_str(world, creature, CREATURE, "name").and_then(def)
}

/// Whether a creature ambushes: its own field when it has one (a hybrid, or a mutant
/// that turned), else its kind's.
pub(crate) fn is_aggressive(world: &World, creature: EntityId) -> bool {
    match world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(CREATURE))
        .and_then(|c| c.fields.get("aggressive"))
    {
        Some(Field::Plain(Value::Bool(b))) => *b,
        _ => creature_def(world, creature).is_some_and(|d| d.aggressive),
    }
}

/// How a creature announces an ambush: its own line, else its kind's, else a plain one.
fn ambush_line(world: &World, creature: EntityId) -> String {
    match world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(CREATURE))
        .and_then(|c| c.fields.get("ambush"))
    {
        Some(Field::Plain(Value::String(s))) => s.clone(),
        _ => creature_def(world, creature)
            .map(|d| d.ambush.to_string())
            .unwrap_or_else(|| "comes at you.".to_string()),
    }
}

fn def(key: &str) -> Option<&'static CreatureDef> {
    CREATURES.iter().find(|c| c.key == key)
}

/// Creatures in a room: `(id, name, hp, max_hp)`.
pub fn creatures_in(world: &World, room: EntityId) -> Vec<(EntityId, String, f64, f64)> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "creature")
        .map(|e| {
            (
                e.id,
                plain_str(world, e.id, CREATURE, "name")
                    .unwrap_or("?")
                    .to_string(),
                plain_num(world, e.id, CREATURE, "hp").unwrap_or(0.0),
                plain_num(world, e.id, CREATURE, "max_hp").unwrap_or(0.0),
            )
        })
        .collect()
}

/// Fill the room to its count now. Respawn jobs call this; nothing else does.
pub fn fill(world: &mut World, room: EntityId) -> Result<usize, StoreError> {
    let Some(key) = plain_str(world, room, SPAWN, "creature").map(str::to_string) else {
        return Ok(0);
    };
    let count = plain_num(world, room, SPAWN, "count").unwrap_or(0.0) as usize;
    let Some(d) = def(&key) else { return Ok(0) };
    // One refill in six brings one more than the room's figure: the fog has two
    // wisps, mostly, and now and then three. (Never fewer: a room below its figure
    // is what a boot refills, and a short room would refill forever.) A room
    // already at its figure is left as it is.
    let have = creatures_in(world, room).len();
    let count = if have < count && world.roll(6) == 0 {
        count + 1
    } else {
        count
    };
    let mut spawned = 0;
    for _ in have..count {
        spawn_one(world, room, d)?;
        spawned += 1;
    }
    Ok(spawned)
}

/// One creature of this kind, here.
pub(crate) fn spawn_one(
    world: &mut World,
    room: EntityId,
    d: &CreatureDef,
) -> Result<EntityId, StoreError> {
    let id = world.store.allocate_id();
    world.record(
        Actor::Driver,
        StoreEvent::EntityCreated {
            id,
            kind: "creature".into(),
            owner: None,
            parent: Some(room),
            scopes: vec![],
        },
    )?;
    // No two of a kind are quite alike: hit points within a fifth of the figure,
    // worth within a quarter. `look` shows the hit points, so a careful fighter can
    // pick the weaker one; the worth is found out at the end.
    let hp = (super::jitter(world, d.hp, 0.8, 1.2) * 10.0).round() / 10.0;
    let xp =
        (super::jitter(world, d.xp, 0.75, 1.25) * super::economy::rate(world, "worth")).round();
    let mut fields = BTreeMap::new();
    fields.insert("name".into(), Field::Plain(json!(d.name)));
    fields.insert("hp".into(), Field::Plain(json!(hp)));
    fields.insert("max_hp".into(), Field::Plain(json!(hp)));
    fields.insert("attack".into(), Field::Plain(json!(d.attack)));
    fields.insert("xp".into(), Field::Plain(json!(xp)));
    world.record(
        Actor::Driver,
        StoreEvent::ComponentSet {
            id,
            component: CREATURE.into(),
            fields,
        },
    )?;
    world.record(
        Actor::Driver,
        StoreEvent::ComponentSet {
            id,
            component: super::SPRITE.into(),
            fields: super::sprite_fields(d.glyph, d.tint, d.key),
        },
    )?;
    Ok(id)
}

/// A creature kind by key.
pub(crate) fn def_by_key(key: &str) -> Option<&'static CreatureDef> {
    def(key)
}

/// Schedule the room's refill after its respawn interval, unless one is pending.
fn schedule_respawn(world: &mut World, room: EntityId) -> Result<(), String> {
    let secs = plain_num(world, room, SPAWN, "respawn_seconds").unwrap_or(60.0);
    // Between three fifths and eight fifths of the figure: a room refills "in a
    // couple of minutes", never on the minute.
    let secs = (super::jitter(world, secs, RESPAWN_LOW, RESPAWN_HIGH)
        * super::economy::rate(world, "respawn")) as i64;
    let due = world.now().plus(foxing_time::Duration::seconds(secs));
    world
        .schedule_once(JOB_RESPAWN, due, json!({"room": room.0}), |p| {
            p["room"] == room.0
        })
        .map(|_| ())
}

/// At boot: every room below its count gets a refill job due now, unless one is pending.
pub fn ensure_spawns(world: &mut World) -> Result<(), String> {
    let rooms: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "room" && e.components.contains_key(SPAWN))
        .map(|e| e.id)
        .collect();
    for room in rooms {
        let count = plain_num(world, room, SPAWN, "count").unwrap_or(0.0) as usize;
        if creatures_in(world, room).len() < count {
            let now = world.now();
            world.schedule_once(JOB_RESPAWN, now, json!({"room": room.0}), |p| {
                p["room"] == room.0
            })?;
        }
        // Creatures from before sprites existed get one; citizens standing here when the
        // driver comes up are noticed, as if they had just walked in.
        let unsprited: Vec<EntityId> = creatures_in(world, room)
            .into_iter()
            .filter(|(id, _, _, _)| {
                world
                    .store
                    .entity(*id)
                    .is_some_and(|e| !e.components.contains_key(super::SPRITE))
                    || super::sprite_is_old(world, *id)
            })
            .map(|c| c.0)
            .collect();
        for id in unsprited {
            if let Some(d) = creature_def(world, id) {
                world
                    .record(
                        Actor::Driver,
                        StoreEvent::ComponentSet {
                            id,
                            component: super::SPRITE.into(),
                            fields: super::sprite_fields(d.glyph, d.tint, d.key),
                        },
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        notice_everyone(world, room)?;
    }
    Ok(())
}

/// Every citizen in the room who is not already fighting gets noticed by what hunts there.
pub fn notice_everyone(world: &mut World, room: EntityId) -> Result<(), String> {
    let citizens: Vec<(String, EntityId)> = world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen")
        .filter_map(|e| e.owner.clone().map(|o| (o, e.id)))
        .collect();
    for (actor, citizen) in citizens {
        if engaged_with(world, citizen).is_none() && super::presence::is_here(world, citizen) {
            on_arrival(world, &actor, citizen, room)?;
        }
    }
    Ok(())
}

/// "It strikes on its own every ten seconds or so while you stand here, typing or
/// not." Said when a fight begins, so the clock is no surprise.
fn clock_line(world: &World, creature: EntityId, actor: &str) -> String {
    let secs = creature_def(world, creature)
        .map(|d| d.strike_seconds)
        .unwrap_or(10);
    if through_the_gate(world, actor) {
        return "It strikes once for every statement you write while you stand here; retreat ends that.".to_string();
    }
    format!("It strikes on its own every {secs} seconds or so while you stand here, typing or not; retreat ends that.")
}

/// A creature's hit points as "left": whole, except under one, where "0 left" and
/// still standing read as a lie; then a decimal.
pub fn left_said(hp: f64) -> String {
    if hp > 0.0 && hp < 1.0 {
        format!("{hp:.1}")
    } else {
        format!("{hp:.0}")
    }
}

/// A citizen has arrived in a room: anything aggressive there will notice them soon.
pub fn on_arrival(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    room: EntityId,
) -> Result<(), String> {
    let hunters: Vec<EntityId> = creatures_in(world, room)
        .into_iter()
        .filter(|(id, _, _, _)| is_aggressive(world, *id))
        .map(|c| c.0)
        .collect();
    for creature in hunters {
        // Each hunter picks its own moment, two to eight seconds in.
        let secs = AMBUSH_SECONDS_LOW
            + world.roll((AMBUSH_SECONDS_HIGH - AMBUSH_SECONDS_LOW + 1) as u32) as i64;
        let due = world.now().plus(foxing_time::Duration::seconds(secs));
        world.schedule_once(
            JOB_AMBUSH,
            due,
            json!({"creature": creature.0, "citizen": citizen.0, "actor": actor}),
            |p| p["creature"] == creature.0 && p["citizen"] == citizen.0,
        )?;
    }
    Ok(())
}

/// The creature will strike on its own while the fight lasts.
fn schedule_strike(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    creature: EntityId,
) -> Result<(), String> {
    let secs = creature_def(world, creature)
        .map(|d| d.strike_seconds)
        .unwrap_or(10);
    // Half to one and a half times its figure: "every ten seconds or so".
    let secs = super::jitter(world, secs as f64, STRIKE_LOW, STRIKE_HIGH).round() as i64;
    let due = world
        .now()
        .plus(foxing_time::Duration::seconds(secs.max(1)));
    world
        .schedule_once(
            JOB_STRIKE,
            due,
            json!({"creature": creature.0, "citizen": citizen.0, "actor": actor}),
            |p| p["creature"] == creature.0 && p["citizen"] == citizen.0,
        )
        .map(|_| ())
}

/// Whether a name's last door was the gate or MCP: an agent writing messages, with
/// no clock of its own between them.
pub fn through_the_gate(world: &World, actor: &str) -> bool {
    matches!(world.door_of(actor), Some("gate") | Some("mcp"))
}

/// One round of the creature against the citizen, narrated: `None` when the fight is
/// not on (gone, moved, disengaged), else whether it ended the fight (a death, a
/// trick that moved someone). The clock's strike and the gate's per-statement
/// round are both this.
pub fn round(
    world: &mut World,
    curves: &super::Curves,
    creature: EntityId,
    citizen: EntityId,
    actor: &str,
) -> Result<Option<(bool, String)>, String> {
    let same_room = world.store.entity(creature).and_then(|c| c.parent)
        == world.store.entity(citizen).and_then(|c| c.parent);
    if !same_room
        || engaged_with(world, citizen) != Some(creature)
        || !super::presence::is_here(world, citizen)
    {
        return Ok(None);
    }
    let cname = plain_str(world, creature, CREATURE, "name")
        .unwrap_or("?")
        .to_string();
    let (taken, dead) = creature_strikes(world, creature, citizen).map_err(|e| e.to_string())?;
    if dead {
        let f = super::find_citizen(world, actor).ok_or("citizen vanished")?;
        let d = die(world, actor, &f, Some(creature)).map_err(|e| e.to_string())?;
        let said = format!("The {cname} {}. You go down.{}", hit_said(taken), d.said());
        narrate(world, actor, said.clone());
        return Ok(Some((true, said)));
    }
    let (trick, over) = super::stacks::trick(world, curves, creature, citizen, actor)?;
    let said = format!(
        "The {cname} {}.{}",
        hit_said(taken),
        trick.map(|t| format!(" {t}")).unwrap_or_default()
    );
    narrate(world, actor, said.clone());
    Ok(Some((over, said)))
}

/// The gate's round: after a statement from a name that came through the gate, the
/// creature it is fighting strikes once. Not after `attack` (its round has the
/// strike back in it) or `retreat`.
pub fn gate_round(
    world: &mut World,
    curves: &super::Curves,
    actor: &str,
    verb: &str,
) -> Result<Option<String>, String> {
    if matches!(verb, "attack" | "retreat") || !through_the_gate(world, actor) {
        return Ok(None);
    }
    let Some(f) = super::find_citizen(world, actor) else {
        return Ok(None);
    };
    let Some(creature) = engaged_with(world, f.citizen) else {
        return Ok(None);
    };
    round(world, curves, creature, f.citizen, actor).map(|r| r.map(|(_, said)| said))
}

/// The fight is over for this citizen: no more strikes come.
pub(crate) fn end_fight(world: &mut World, citizen: EntityId) -> Result<(), String> {
    world.cancel(JOB_STRIKE, |p| p["citizen"] == citizen.0)?;
    world.cancel(JOB_AMBUSH, |p| p["citizen"] == citizen.0)?;
    Ok(())
}

/// A citizen leaves a fight without a blow: gone quiet, the room disremembers them.
pub(crate) fn leave_fight(world: &mut World, actor: &str, citizen: EntityId) -> Result<(), String> {
    if let Some(creature) = engaged_with(world, citizen) {
        mend(world, creature).map_err(|e| e.to_string())?;
        set_engaged(world, actor, citizen, None).map_err(|e| e.to_string())?;
    }
    end_fight(world, citizen)
}

/// A creature of the Stacks left standing is whole again by the next time: a
/// newcomer with an attack of three felled a censor in four trips and three deaths,
/// the censor keeping every wound between them. The ground floor's creatures keep
/// their wounds, as they always have; the Stacks keep their own.
/// A creature's hit points, set by a trick (a leech mends by its bite).
pub(crate) fn set_creature_hp(
    world: &mut World,
    creature: EntityId,
    value: f64,
) -> Result<(), String> {
    world
        .record(
            Actor::Driver,
            StoreEvent::FieldSet {
                id: creature,
                component: CREATURE.into(),
                field: "hp".into(),
                value: json!(value),
            },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub(crate) fn mend(world: &mut World, creature: EntityId) -> Result<(), StoreError> {
    // The Stacks' own kinds only: a ground-floor creature that learned a trick from a
    // kill keeps its wounds like its neighbors (a wisp that learned to shove was back
    // to full hit points seconds after the shove; the owner called it a bug).
    let kind = super::plain_str(world, creature, CREATURE, "name").unwrap_or("");
    if !super::stacks::TRICKS.iter().any(|(k, _)| *k == kind) {
        return Ok(());
    }
    let max = plain_num(world, creature, CREATURE, "max_hp").unwrap_or(0.0);
    let hp = plain_num(world, creature, CREATURE, "hp").unwrap_or(max);
    if hp < max {
        world.record(
            Actor::Driver,
            StoreEvent::FieldSet {
                id: creature,
                component: CREATURE.into(),
                field: "hp".into(),
                value: json!(max),
            },
        )?;
    }
    Ok(())
}

/// The creature a citizen is engaged with, if it still exists.
pub fn engaged_with(world: &World, citizen: EntityId) -> Option<EntityId> {
    let id = plain_num(world, citizen, BODY, "engaged")? as u64;
    world.store.entity(EntityId(id)).map(|e| e.id)
}

/// How fast this body mends now: nothing in a fight, nothing while bleeding, else
/// the world's rate.
pub fn mending_rate(world: &World, citizen: EntityId) -> f64 {
    if engaged_with(world, citizen).is_some() {
        return 0.0;
    }
    let now = world.now().0 as f64;
    if plain_num(world, citizen, BODY, "bleed_until").is_some_and(|u| u > now) {
        return 0.0;
    }
    super::HP_PER_SECOND
}

/// Set the body's mending to what it should be now (`mending_rate`): after a fight
/// begins or ends, after a bleeding bite, when it staunches, on waking.
pub fn recurve_mending(world: &mut World, citizen: EntityId) -> Result<(), StoreError> {
    let rate = mending_rate(world, citizen);
    world.record(
        Actor::Driver,
        StoreEvent::FieldRecurved {
            id: citizen,
            component: BODY.into(),
            field: "hp".into(),
            curve: foxing_time::Curve::Linear {
                rate_per_ms: rate / 1000.0,
            },
        },
    )?;
    Ok(())
}

/// A bleeding bite: nothing mends for this long.
pub const BLEED_MS: i64 = 8 * 60 * 1000;
pub const JOB_STAUNCH: &str = "disremembered.staunch";

pub(crate) fn set_engaged(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    creature: Option<EntityId>,
) -> Result<(), StoreError> {
    let value = creature.map(|c| json!(c.0)).unwrap_or(Value::Null);
    world.record(
        Actor::Citizen(actor.into()),
        StoreEvent::FieldSet {
            id: citizen,
            component: BODY.into(),
            field: "engaged".into(),
            value,
        },
    )?;
    recurve_mending(world, citizen)?;
    Ok(())
}

/// Current hit points.
pub fn hp(world: &World, citizen: EntityId) -> f64 {
    world
        .store
        .read_field(citizen, BODY, "hp", world.now())
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

/// Hit points as a line says them: rounded up, so a name still standing on a
/// fraction never reads "0/30".
pub fn hp_said(world: &World, citizen: EntityId) -> f64 {
    hp(world, citizen).ceil()
}

pub(crate) fn commit_hp(
    world: &mut World,
    citizen: EntityId,
    value: f64,
) -> Result<(), StoreError> {
    world.record(
        Actor::Driver,
        StoreEvent::FieldCommitted {
            id: citizen,
            component: BODY.into(),
            field: "hp".into(),
            value: value.max(0.0),
        },
    )?;
    Ok(())
}

/// Damage per round at this level.
pub fn attack_for_level(level: f64) -> f64 {
    BASE_ATTACK + (ATTACK_PER_ROOT_LEVEL * level.max(0.0).sqrt()).floor()
}

/// The creature hits back. Returns the damage and whether the citizen died.
fn creature_strikes(
    world: &mut World,
    creature: EntityId,
    citizen: EntityId,
) -> Result<(f64, bool), StoreError> {
    let attack = plain_num(world, creature, CREATURE, "attack").unwrap_or(1.0);
    let dmg = attack + world.roll(3) as f64;
    let before = hp(world, citizen);
    let after = before - dmg;
    commit_hp(world, citizen, after)?;
    LAST_HIT.with(|c| c.set((before.ceil(), after.max(0.0).ceil())));
    Ok((dmg, after <= 0.0))
}

thread_local! {
    /// The hit points before and after the last strike, as a line says them: both
    /// ends, because you mend between strikes and a line that said only the end
    /// read as arithmetic gone wrong ("hits you for 4" and the number fell by 2).
    static LAST_HIT: std::cell::Cell<(f64, f64)> = const { std::cell::Cell::new((0.0, 0.0)) };
}

/// "hits you for 4: 28 → 24 of 30", from the last strike.
pub(crate) fn hit_said(taken: f64) -> String {
    let (before, after) = LAST_HIT.with(|c| c.get());
    format!(
        "hits you for {taken:.0}: {before:.0} \u{2192} {after:.0} of {:.0}",
        super::MAX_HP
    )
}

/// Death: concussion, then wake at the Threshold at full health, disengaged.
/// A fall, as the call sites say it: what was lost (as prose, if anything), its
/// worth, which fall of the night, the epitaph, who keeps it, what it learned.
pub(crate) struct Death {
    pub lost: Option<String>,
    pub worth: f64,
    pub fall: u32,
    pub epitaph: String,
    pub keeper: Option<String>,
    pub learned: Option<&'static str>,
}

impl Death {
    /// The sentences after "You go down.", for every door a fall comes through.
    pub fn said(&self) -> String {
        // Named as an epitaph: a visitor read it as prose and reported none was said.
        let mut out = format!(
            " Over you, the world says: \u{201c}{}\u{201d}",
            self.epitaph
        );
        if let Some(c) = &self.lost {
            out.push_str(&format!(" Concussion: you have lost {c}"));
            match &self.keeper {
                Some(k) => out.push_str(&format!("; {k} keeps it.")),
                None => out.push('.'),
            }
        }
        if self.fall > 1 {
            out.push_str(&format!(
                " The {} fall of the night: it costs {} what the first did.",
                super::death::ordinal(self.fall),
                match self.fall {
                    2 => "half",
                    3 => "a quarter",
                    4 => "an eighth",
                    _ => "almost nothing of",
                }
            ));
        }
        if let Some(l) = self.learned {
            out.push_str(&format!(" And {}.", super::death::learned_said(l)));
        }
        out.push_str(if self.lost.is_some() {
            " You wake at the Threshold."
        } else {
            " You wake at the Threshold with everything you had."
        });
        out
    }
}

pub(crate) fn die(
    world: &mut World,
    actor: &str,
    f: &super::Found,
    killer: Option<EntityId>,
) -> Result<Death, CommandError> {
    // Concussion, on the curve: chunks go, picked at random, until the share of the
    // window the level owes is gone; the chunk that would take more than is owed is
    // thinned to the remainder instead of taken whole (a 7% death once cost a third of
    // a window, the one chunk the roll landed on). Nothing goes from a window worth
    // almost nothing. The share halves with every fall of the same night: cheap, but
    // not at first.
    let level = super::memory::level(world, f.window);
    let (fall, ever) = super::death::record_fall(world, actor, f.citizen, killer)?;
    let share = death_share(level) * 0.5f64.powi(fall.saturating_sub(1) as i32);
    let owed = level * share;
    let fell_in = super::rooms::room_of(world, f.citizen);
    let mut chunks: Vec<(EntityId, String, f64)> = world
        .store
        .children(f.window)
        .filter(|e| e.kind == "chunk")
        .map(|e| {
            (
                e.id,
                plain_str(world, e.id, CHUNK, "source")
                    .unwrap_or("?")
                    .to_string(),
                world
                    .store
                    .read_field(e.id, CHUNK, "value", world.now())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            )
        })
        .collect();
    let mut gone: Vec<(EntityId, String, f64)> = Vec::new();
    let mut thinned: Vec<(EntityId, String, f64, f64)> = Vec::new();
    let mut taken = 0.0;
    while taken < owed && owed >= 1.0 && !chunks.is_empty() {
        let (id, source, value) = chunks.remove(world.roll(chunks.len() as u32) as usize);
        let remaining = owed - taken;
        if value > remaining + 0.5 {
            // Thin it to what is owed, and that settles the debt.
            let left = ((value - remaining) * 100.0).round() / 100.0;
            world
                .record(
                    Actor::Driver,
                    StoreEvent::FieldCommitted {
                        id,
                        component: CHUNK.into(),
                        field: "value".into(),
                        value: left,
                    },
                )
                .map_err(internal)?;
            taken += remaining;
            thinned.push((id, source, remaining, left));
            break;
        }
        world
            .record(Actor::Driver, StoreEvent::EntityRemoved { id })
            .map_err(internal)?;
        taken += value;
        gone.push((id, source, value));
    }
    // What was lost goes to the one that took it; it may learn something from the kill.
    let curves = super::Curves::from_env();
    let kept_chunk = match killer {
        Some(k) => super::death::killer_keeps(world, actor, k, taken, &curves)?,
        None => None,
    };
    let learned = match killer {
        Some(k) => super::death::maybe_learn(world, k)?,
        None => None,
    };
    // "the ragpicker", or "Ossian the ragpicker" once it has a name (it may have
    // earned one off this very fall): the article travels with the slot.
    let keeper_slot = killer.map(|k| super::death::killer_said(world, k));
    let epitaph = super::death::epitaph(world, actor, ever, killer, fell_in, taken, fall);
    let _ = world.note(
        Actor::Driver,
        DIED,
        json!({"citizen": actor, "level": level, "share": share, "fall": fall, "falls_ever": ever, "worth": (taken * 100.0).round() / 100.0,
               "room": fell_in.map(|r| r.0), "room_name": fell_in.and_then(|r| plain_str(world, r, super::ROOM, "name")),
               "killer": killer.map(|k| k.0), "killer_kind": killer.and_then(|k| plain_str(world, k, CREATURE, "name")), "killer_name": killer.and_then(|k| super::mind::name(world, k)),
               "kept": kept_chunk.map(|c| c.0), "learned": learned, "epitaph": epitaph,
               "kept_things": super::death::kept(world, actor),
               "night": super::death::night(world, f.citizen),
               "lost": gone.iter().map(|(id, s, v)| json!({"chunk": id.0, "source": s, "value": v}))
                   .chain(thinned.iter().map(|(id, s, v, left)| json!({"chunk": id.0, "source": s, "value": v, "thinned_to": left})))
                   .collect::<Vec<_>>()}),
    );
    let lost =
        if gone.is_empty() && thinned.is_empty() {
            None
        } else {
            let mut parts: Vec<String> = gone
                .iter()
                .map(|(id, s, v)| format!("chunk-{} ({s}, {v:.0} XP)", id.0))
                .collect();
            parts.extend(thinned.iter().map(|(id, s, v, left)| {
                format!("{v:.0} XP of chunk-{} ({s}, {left:.0} left)", id.0)
            }));
            Some(if parts.len() == 1 {
                parts.remove(0)
            } else {
                format!("{taken:.0} XP: {}", parts.join(", "))
            })
        };

    let home = super::rooms::arrival(world).ok_or_else(|| internal("the Threshold is missing"))?;
    if let Some(creature) = engaged_with(world, f.citizen) {
        mend(world, creature).map_err(internal)?;
    }
    // Whatever was in you dies with you: a lamprey's venom once went on working at
    // the Threshold, where nothing hunts, after the wake.
    let me_id = f.citizen.0;
    let _ = world.cancel(super::stacks::JOB_VENOM, |p| p["citizen"] == me_id);
    let _ = world.cancel(JOB_STAUNCH, |p| p["citizen"] == me_id);
    world
        .record(
            Actor::Driver,
            StoreEvent::FieldSet {
                id: f.citizen,
                component: BODY.into(),
                field: "bleed_until".into(),
                value: json!(0),
            },
        )
        .map_err(internal)?;
    set_engaged(world, actor, f.citizen, None).map_err(internal)?;
    end_fight(world, f.citizen).map_err(internal)?;
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: f.citizen,
                parent: Some(home),
            },
        )
        .map_err(internal)?;
    commit_hp(world, f.citizen, super::MAX_HP).map_err(internal)?;
    Ok(Death {
        lost,
        worth: taken,
        fall,
        epitaph,
        keeper: keeper_slot,
        learned,
    })
}

pub fn register(world: &mut World, curves: Curves) {
    world.on_job(
        JOB_STAUNCH,
        Box::new(|w: &mut World, job| {
            let citizen = EntityId(job.payload["citizen"].as_u64().unwrap_or(0));
            if w.store.entity(citizen).is_none() {
                return Ok(());
            }
            recurve_mending(w, citizen).map_err(|e| e.to_string())?;
            if let Some(actor) = job.payload["actor"].as_str() {
                if mending_rate(w, citizen) > 0.0 {
                    narrate(w, actor, "The bleeding stops; you begin to mend.");
                }
            }
            Ok(())
        }),
    );
    world.on_job(
        JOB_RESPAWN,
        Box::new(|w: &mut World, job| {
            let room = EntityId(job.payload["room"].as_u64().unwrap_or(0));
            let spawned = fill(w, room).map_err(|e| e.to_string())?;
            if spawned > 0 {
                notice_everyone(w, room)?;
            }
            Ok(())
        }),
    );

    let c = curves.clone();
    world.on_job(
        JOB_STRIKE,
        Box::new(move |w: &mut World, job| {
            let creature = EntityId(job.payload["creature"].as_u64().unwrap_or(0));
            let citizen = EntityId(job.payload["citizen"].as_u64().unwrap_or(0));
            let actor = job.payload["actor"].as_str().unwrap_or("").to_string();
            // A name that came through the gate is struck once a statement, not on
            // the clock: a stateless visitor took nine strikes between two lines.
            if through_the_gate(w, &actor) {
                return Ok(());
            }
            let Some((over, _)) = round(w, &c, creature, citizen, &actor)? else {
                return Ok(());
            };
            if over {
                return Ok(());
            }
            schedule_strike(w, &actor, citizen, creature)
        }),
    );

    let c2 = curves.clone();
    world.on_job(
        JOB_AMBUSH,
        Box::new(move |w: &mut World, job| {
            let creature = EntityId(job.payload["creature"].as_u64().unwrap_or(0));
            let citizen = EntityId(job.payload["citizen"].as_u64().unwrap_or(0));
            let actor = job.payload["actor"].as_str().unwrap_or("").to_string();
            let same_room = w.store.entity(creature).and_then(|c| c.parent)
                == w.store.entity(citizen).and_then(|c| c.parent);
            if !same_room
                || w.store.entity(creature).is_none()
                || engaged_with(w, citizen).is_some()
                || !super::presence::is_here(w, citizen)
            {
                return Ok(());
            }
            if !is_aggressive(w, creature) {
                return Ok(());
            }
            let dname = plain_str(w, creature, CREATURE, "name")
                .unwrap_or("creature")
                .to_string();
            let dambush = ambush_line(w, creature);
            // It does not hunt those it knows: its keepers, whoever told it a story,
            // whoever did what it asked. (A visitor told Sorrel the story it wanted
            // and was ambushed by Sorrel in the Clocktower an hour later.)
            // Nor a name in its first hour: a hollow in the Shallows took a newcomer
            // from thirty to nothing between two gate lines, before they could act.
            let new_name = super::plain_num(w, citizen, super::CITIZEN, "named_at")
                .is_some_and(|t| w.now().0 - (t as i64) < super::kindle::GRACE_MS);
            if new_name
                || super::mind::shaken(w, creature)
                || super::mind::keepers(w, creature)
                    .iter()
                    .any(|k| k == &actor)
                || super::mind::told_by(w, creature, &actor)
                || super::errand::did_for(w, creature, &actor)
            {
                return Ok(());
            }
            set_engaged(w, &actor, citizen, Some(creature)).map_err(|e| e.to_string())?;
            let (taken, dead) =
                creature_strikes(w, creature, citizen).map_err(|e| e.to_string())?;
            if dead {
                let f = super::find_citizen(w, &actor).ok_or("citizen vanished")?;
                let d = die(w, &actor, &f, Some(creature)).map_err(|e| e.to_string())?;
                narrate(
                    w,
                    &actor,
                    format!(
                        "The {dname} {dambush} It {}. You go down.{}",
                        hit_said(taken),
                        d.said()
                    ),
                );
                return Ok(());
            }
            let (trick, over) = super::stacks::trick(w, &c2, creature, citizen, &actor)?;
            let clock = clock_line(w, creature, &actor);
            narrate(
                w,
                &actor,
                format!(
                    "The {dname} {dambush} It {}.{} {clock}",
                    hit_said(taken),
                    match &trick {
                        Some(t) => format!(" {t}"),
                        None => " Attack it, or retreat.".to_string(),
                    }
                ),
            );
            if over {
                return Ok(());
            }
            schedule_strike(w, &actor, citizen, creature)
        }),
    );

    world.register(
        CommandSpec::new("disremembered.attack", "Fight a creature here, one round: you strike, it strikes back. Win and you remember it; the closer you came to dying, the longer the memory lasts. While you stand with it, it keeps striking: at a keyboard on its own every few seconds; through the gate once for every statement you write. `retreat` ends that (it gets one blow as you turn). Some creatures ambush a name that is past its first hour.")
            .param("creature", ParamType::Entity, false, "a creature name from look; defaults to the one you are fighting, else an unnamed one here. A named creature is never struck by default."),
        Box::new(move |w: &mut World, ctx, args| {
            let f = me(w, ctx)?;
            let room = super::rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let here = creatures_in(w, room);
            if args.str("creature").is_some_and(super::docent::is_named) {
                return Err(CommandError::precondition(
                    "The Docent is not for fighting; it keeps the door, and has seen worse than you. Ask it something: tell docent \"\u{2026}\".",
                ));
            }
            let target = match args.str("creature") {
                Some(name) if !name.is_empty() => {
                    // The kind you are already fighting is the one you mean, before any
                    // other of its kind standing by (a typed "kill wisp" once turned on
                    // a shaken wisp beside the one that was biting).
                    let asked = name.trim().trim_start_matches("the ").to_lowercase();
                    let fighting = engaged_with(w, f.citizen)
                        .filter(|id| here.iter().any(|c| c.0 == *id))
                        .filter(|id| plain_str(w, *id, CREATURE, "name").is_some_and(|k| k == asked));
                    let found = fighting.or_else(|| super::mind::find_here(w, room, name));
                    // A kind never strikes a named creature, even one alone of its kind:
                    // two visitors in one night felled Lute by typing "attack wisp".
                    if let Some(c) = found {
                        let kind = plain_str(w, c, CREATURE, "name").unwrap_or("");
                        let asked = name.trim().trim_start_matches("the ").to_lowercase();
                        if super::mind::name(w, c).is_some() && asked == kind {
                            return Err(CommandError::precondition(format!(
                                "No unnamed {kind} is here. {} has a name, and is struck only by that name.",
                                super::mind::display(w, c)
                            )));
                        }
                    }
                    found
                }
                // With no target: the one you are fighting, else the first creature here
                // that nobody has named and nobody bore. A named creature is never struck
                // by default, nor a child: one was felled at the Threshold by "attack".
                _ => engaged_with(w, f.citizen).filter(|id| here.iter().any(|c| c.0 == *id) && !super::mind::shaken(w, *id)).or_else(|| here.iter().find(|c| super::mind::name(w, c.0).is_none() && super::kindle::born_at(w, c.0).is_none() && !super::mind::shaken(w, c.0)).map(|c| c.0)),
            };
            if let Some(c) = target {
                if let Some(born) = super::kindle::born_at(w, c) {
                    let age = w.now().0 - born;
                    if age < super::kindle::GRACE_MS && super::mind::name(w, c).is_none() {
                        return Err(CommandError::precondition(format!(
                            "The {} is newly born and not to be struck for {} more; feed it instead, or let it be.",
                            plain_str(w, c, CREATURE, "name").unwrap_or("creature"),
                            human(super::kindle::GRACE_MS - age)
                        )));
                    }
                }
            }
            let Some(creature) = target else {
                // Where you are, when it is not where you think: a death earlier in the
                // same message leaves you at the Threshold, and "No such creature here"
                // read as an error.
                let where_ = format!(" You are in {}.", plain_str(w, room, super::ROOM, "name").unwrap_or("?"));
                return Err(CommandError::precondition(if here.is_empty() { format!("There is nothing here to fight.{where_}") } else if args.str("creature").map_or(true, str::is_empty) {
                    format!("Nothing unnamed is here to fight. {} {} a name, and {} struck only by that name.", here.iter().map(|c| super::mind::display(w, c.0)).collect::<Vec<_>>().join(" and "), if here.len() == 1 { "has" } else { "have" }, if here.len() == 1 { "is" } else { "are" })
                } else {
                    // Named as look names them, so the retry can be copied from the refusal.
                    format!("No such creature here. Here: {}.", here.iter().map(|c| {
                        let kind = c.1.as_str();
                        let twins = here.iter().filter(|o| o.1 == *kind).count();
                        match super::mind::name(w, c.0) {
                            Some(n) => format!("{n} the {kind}"),
                            None if twins > 1 => format!("{kind}-{}", c.0 .0),
                            None => kind.to_string(),
                        }
                    }).collect::<Vec<_>>().join(", ")) + &where_
                }));
            };
            let cname = plain_str(w, creature, CREATURE, "name").unwrap_or("?").to_string();
            if let Some(ms) = super::mind::shaken_for(w, creature) {
                return Err(CommandError::precondition(format!(
                    "{} is not itself yet; there is nothing to take from it for {}.",
                    super::mind::display(w, creature),
                    human(ms)
                )));
            }
            let mut began = None;
            if engaged_with(w, f.citizen) != Some(creature) {
                set_engaged(w, &ctx.actor, f.citizen, Some(creature)).map_err(internal)?;
                end_fight(w, f.citizen).map_err(internal)?;
                schedule_strike(w, &ctx.actor, f.citizen, creature).map_err(internal)?;
                // The clock, said once: a visitor died between two messages and read
                // it only on the next.
                began = Some(clock_line(w, creature, &ctx.actor));
            }

            // You strike.
            let level = super::memory::level(w, f.window);
            let mut dmg = attack_for_level(level) + w.roll(3) as f64;
            // An arm asleep from a numbing bite strikes for half, for a while.
            let numb = plain_num(w, f.citizen, BODY, "numb_until").is_some_and(|u| u > w.now().0 as f64);
            if numb {
                dmg = (dmg / 2.0).max(1.0).floor();
            }
            let chp = (plain_num(w, creature, CREATURE, "hp").unwrap_or(0.0) - dmg).max(0.0);
            w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::FieldSet { id: creature, component: CREATURE.into(), field: "hp".into(), value: json!(chp) }).map_err(internal)?;
            // A fight begun by default is said, not silent: six bare `attack` lines
            // felled an inkling on the fourth and struck its neighbor on the fifth.
            let turned = args.str("creature").map_or(true, str::is_empty) && began.is_some();
            let mut lines = vec![if turned {
                format!("You turn on the {cname} and hit it for {dmg:.0}{}.", if numb { ", your arm still asleep" } else { "" })
            } else {
                format!("You hit the {cname} for {dmg:.0}{}.", if numb { ", your arm still asleep" } else { "" })
            }];
            if chp > 0.0 {
                if let Some(c) = began.take() {
                    lines.push(c);
                }
            }

            if chp <= 0.0 {
                // A creature that holds memories does not die: you take one, it is shaken.
                // The memory is yours at what it is worth now, as vivid as the fight.
                let who = super::mind::display(w, creature);
                let taken_vividity = (1.0 - hp(w, f.citizen) / super::MAX_HP).clamp(0.0, 1.0);
                if let Some((chunk, source, worth, was)) = super::mind::take(w, &ctx.actor, f.window, creature, taken_vividity, &curves)? {
                    set_engaged(w, &ctx.actor, f.citizen, None).map_err(internal)?;
                    end_fight(w, f.citizen).map_err(internal)?;
                    let left = super::mind::level(w, creature);
                    let faded = if was > worth + 0.5 {
                        format!(" It was worth {was:.0} when the {cname} got it; a creature's memories fade as yours do.")
                    } else {
                        String::new()
                    };
                    lines.push(format!(
                        "{who} goes down, and you take its memory of {source}: chunk-{} worth {worth:.0} XP.{faded} It gets up again, holding {left:.0}, and will not be itself for {}.",
                        chunk.0,
                        super::mind::shaken_for(w, creature).map(human).unwrap_or_else(|| "a while".into())
                    ));
                    return Ok(Outcome::new(lines.join(" "), json!({"creature": cname, "killed": false, "taken": chunk.0, "with": source, "xp": worth, "was": was, "left": left, "hp": hp(w, f.citizen)})));
                }
                // Win: the chunk's vividity is how close you came to dying; a moth
                // charm carried makes it more vivid and is spent.
                let my_hp = hp(w, f.citizen);
                let mut vividity = (1.0 - my_hp / super::MAX_HP).clamp(0.0, 1.0);
                let mut charm_line = String::new();
                if let Some(charm) = super::craft::carries(w, f.citizen, "charm") {
                    vividity = (vividity + 0.3).min(1.0);
                    let name = super::craft::spend(w, &ctx.actor, charm)?;
                    charm_line = format!(" The {} you carried is spent, and the memory is the more vivid for it.", name.trim_start_matches("a "));
                }
                let xp = plain_num(w, creature, CREATURE, "xp").unwrap_or(0.0);
                // What it leaves behind, one time in three.
                let left = super::craft::leave_behind(w, &curves, room, creature, &cname)?;
                // The dying throes of a creature of the Stacks: its trick, once more.
                let (throes, _) = super::stacks::throes(w, &curves, creature, f.citizen, &ctx.actor).map_err(internal)?;
                w.record(Actor::Driver, StoreEvent::EntityRemoved { id: creature }).map_err(internal)?;
                schedule_respawn(w, room).map_err(internal)?;
                set_engaged(w, &ctx.actor, f.citizen, None).map_err(internal)?;
                end_fight(w, f.citizen).map_err(internal)?;
                let chunk = super::memory::earn_chunk(w, &ctx.actor, f.window, &cname, xp, vividity, &curves)?;
                lines.push(format!(
                    "The {cname} comes apart. You remember it: chunk-{} worth {xp:.0} XP, vividity {:.2}{}.{charm_line}{}{}",
                    chunk.0,
                    vividity,
                    if vividity > 0.6 { " (you were near death for it; this one will last)" } else { "" },
                    match &left {
                        Some((id, n)) => format!(" It leaves {n} behind (item-{}); take it.", id.0),
                        // Say when the roll missed, so a visitor can tell a missed
                        // roll from a broken one (ten kills, nothing, and no way to know).
                        None if super::craft::material_for(&cname).is_some() => " Nothing of it is left behind this time.".to_string(),
                        None => String::new(),
                    },
                    throes.map(|t| format!(" {t}")).unwrap_or_default()
                ));
                return Ok(Outcome::new(lines.join(" "), json!({"creature": cname, "killed": true, "chunk": chunk.0, "xp": xp, "vividity": vividity, "hp": my_hp, "left": left.as_ref().map(|(id, n)| json!({"item": id.0, "name": n}))})));
            }

            // It strikes back.
            let (taken, dead) = creature_strikes(w, creature, f.citizen).map_err(internal)?;
            lines.push(format!("The {cname} ({} left) {}.", left_said(chp), hit_said(taken)));
            if dead {
                let d = die(w, &ctx.actor, &f, Some(creature))?;
                lines.push(format!("You go down.{}", d.said()));
                return Ok(Outcome::new(lines.join(" "), json!({"creature": cname, "killed": false, "died": true, "lost": d.lost, "worth": d.worth, "fall": d.fall, "epitaph": d.epitaph, "keeper": d.keeper, "learned": d.learned})));
            }
            let my_hp = hp_said(w, f.citizen);
            let (trick, _) = super::stacks::trick(w, &curves, creature, f.citizen, &ctx.actor).map_err(internal)?;
            if let Some(t) = trick {
                lines.push(t);
            }
            Ok(Outcome::new(lines.join(" "), json!({"creature": cname, "killed": false, "died": false, "creature_hp": chp, "hp": my_hp})))
        }),
    );

    world.register(
        CommandSpec::new("disremembered.retreat", "Break off the fight. The creature gets one free strike as you go."),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let Some(creature) = engaged_with(w, f.citizen) else {
                return Err(CommandError::precondition("You are not fighting anything."));
            };
            let cname = plain_str(w, creature, CREATURE, "name").unwrap_or("?").to_string();
            let (taken, dead) = creature_strikes(w, creature, f.citizen).map_err(internal)?;
            mend(w, creature).map_err(internal)?;
            set_engaged(w, &ctx.actor, f.citizen, None).map_err(internal)?;
            end_fight(w, f.citizen).map_err(internal)?;
            if dead {
                let d = die(w, &ctx.actor, &f, Some(creature))?;
                let lost = d.lost.clone();
                let msg = {
                    format!("The {cname} catches you as you turn, for {taken:.0}. You go down.{}", d.said())
                };
                return Ok(Outcome::new(msg, json!({"died": true, "lost": lost})));
            }
            let mended = match super::stacks::trick_of(w, creature) {
                Some(super::stacks::Trick::Enter) | Some(super::stacks::Trick::Below) => " It will be whole again by the time you come back; the Conclave keeps its order.",
                Some(_) => " It will be whole again by the time you come back; the Stacks keep their own.",
                None => "",
            };
            Ok(Outcome::new(format!("You back away from the {cname}; it catches you once for {taken:.0}. You have {:.0}/{:.0}.{mended}", hp_said(w, f.citizen), super::MAX_HP), json!({"died": false, "hp": hp(w, f.citizen)})))
        }),
    );

    world.register(
        CommandSpec::new("disremembered.rest", "Your hit points, and how long until they are full. Healing happens on its own, on your clock."),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let now = w.now();
            let current = hp(w, f.citizen);
            let eta = w
                .store
                .timed_field(f.citizen, BODY, "hp")
                .ok()
                .and_then(|t| t.active_time_to_reach(super::MAX_HP, now, &w.store.scopes_for(f.citizen).unwrap_or_default()));
            let mut bleeding = plain_num(w, f.citizen, BODY, "bleed_until").filter(|&u| u > now.0 as f64);
            // A bandage carried closes the bite now, and is spent.
            let mut bandaged = String::new();
            if bleeding.is_some() {
                if let Some(bandage) = super::craft::carries(w, f.citizen, "bandage") {
                    super::craft::spend(w, &ctx.actor, bandage)?;
                    let me_id = f.citizen.0;
                    let _ = w.cancel(JOB_STAUNCH, |p| p["citizen"] == me_id);
                    w.record(
                        Actor::Driver,
                        StoreEvent::FieldSet { id: f.citizen, component: BODY.into(), field: "bleed_until".into(), value: json!(0) },
                    )
                    .map_err(internal)?;
                    recurve_mending(w, f.citizen).map_err(internal)?;
                    bleeding = None;
                    bandaged = "Your bandage closes the bite, and is spent. ".to_string();
                }
            }
            let text = match eta {
                _ if current < super::MAX_HP - 0.5 && engaged_with(w, f.citizen).is_some() => format!("You have {current:.0}/{:.0}. Nothing mends in a fight; retreat, and it starts.", super::MAX_HP),
                _ if current < super::MAX_HP - 0.5 && bleeding.is_some() => format!("You have {current:.0}/{:.0}. You are bleeding: nothing mends for {} more.", super::MAX_HP, human((bleeding.unwrap() as i64) - now.0)),
                Some(d) if d.0 > 0 => format!("{bandaged}You have {current:.0}/{:.0}. Full in {}.", super::MAX_HP, human(d.0)),
                _ => format!("{bandaged}You have {current:.0}/{:.0}. You are whole.", super::MAX_HP),
            };
            Ok(Outcome::new(text, json!({"hp": current, "max_hp": super::MAX_HP, "full_in_seconds": eta.map(|d| d.0 / 1000)})))
        }),
    );
}

/// `90s`, `4m 10s`, `1h 2m`.
pub fn human(ms: i64) -> String {
    let s = ms / 1000;
    if s < 90 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m {}s", s / 60, s % 60)
    } else {
        format!("{}h {}m", s / 3600, (s % 3600) / 60)
    }
}
