//! The Lower Stacks: twenty rooms under the Archive where the higher things live, each
//! with a trick of its own. A lamprey's bite stays in you (venom works for a minute
//! after); an usher shows you out (one hit in two moves you to another room of the
//! Stacks, the fight over, whatever is there noticing you); a censor's stamp is a ray
//! of forgetfulness (one hit in two halves one of your chunks where it stands); the
//! cataloguer has your entry open and does the same, harder. The rooms are in
//! rooms.rs, the creatures in combat.rs; this is the tricks, called after a creature's
//! strike lands and you are still standing. The Conclave's clerks and Sergeant keep
//! theirs here too: an engrosser enters a chunk of yours into the minutes (shelved,
//! not halved), the tipstaff shows you below the bar.

use super::{plain_num, plain_str, CHUNK, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_store::{EntityId, StoreEvent};
use serde_json::{json, Value};

/// The rooms of the Stacks, by key.
pub const KEYS: &[&str] = &[
    "lower-stacks",
    "catalogue",
    "dark-reading-room",
    "periodicals",
    "dumbwaiter",
    "overdue-desk",
    "carrels",
    "index-stair",
    "bindery",
    "tube-room",
    "lost-things",
    "acid-room",
    "chart-room",
    "censor-office",
    "restoration",
    "pulp-sump",
    "the-leak",
    "reserve",
    "strongroom",
    "deep-reserve",
];

/// What each kind does when its strike lands.
/// How long a chunk entered into the minutes stays there before `reminisce` has it back.
pub const MINUTED_MS: i64 = 5 * 60 * 1000;

/// How much longer `chunk` is held in the minutes, if an engrosser entered it within
/// [`MINUTED_MS`]; None when it is free to reminisce.
pub fn minuted_for(world: &World, chunk: EntityId) -> Option<i64> {
    let now = world.now();
    world
        .recent_events(3000)
        .iter()
        .rev()
        .find(|e| {
            e.kind == TRICK && e.payload["trick"] == "entered" && e.payload["chunk"] == chunk.0
        })
        .map(|e| MINUTED_MS - now.since(e.at).0)
        .filter(|left| *left > 0)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trick {
    /// Venom: damage on a clock after the bite.
    Venom,
    /// Shown out: moved to another room of the Stacks, the fight over.
    Usher,
    /// A ray of forgetfulness: one chunk halved.
    Forget,
    /// Entered into the minutes: one chunk shelved, not halved (the Conclave's clerk).
    Enter,
    /// Shown below the bar: moved to the Petitioners' Gallery, the fight over (the
    /// Conclave's Sergeant). Never out of the Conclave.
    Below,
    // Learned from kills, from irritating to painful (`death::maybe_learn`):
    /// It says your name back wrong. Nothing else; it is the principle of the thing.
    Chatter,
    /// Shoved through a random exit of the room, the fight over.
    Shove,
    /// Your next strike does half: your arm is asleep for a while.
    Numb,
    /// It mends by what it bit.
    Leech,
    /// It bites a second time in the same breath, and stops short of the kill.
    Maw,
    /// A bleeding bite: nothing mends for a while.
    Bleed,
}

/// What a learned trick is called in the record and on the creature.
pub fn learned_key(t: Trick) -> &'static str {
    match t {
        Trick::Venom => "venom",
        Trick::Forget => "forget",
        Trick::Chatter => "chatter",
        Trick::Shove => "shove",
        Trick::Numb => "numb",
        Trick::Leech => "leech",
        Trick::Maw => "maw",
        Trick::Bleed => "bleed",
        Trick::Usher => "usher",
        Trick::Enter => "enter",
        Trick::Below => "below",
    }
}

pub fn learned_trick(key: &str) -> Option<Trick> {
    Some(match key {
        "venom" => Trick::Venom,
        "forget" => Trick::Forget,
        "chatter" => Trick::Chatter,
        "shove" => Trick::Shove,
        "numb" => Trick::Numb,
        "leech" => Trick::Leech,
        "maw" => Trick::Maw,
        "bleed" => Trick::Bleed,
        _ => return None,
    })
}

/// Tiers a creature learns through, kill by kill: the first kill teaches something
/// irritating, the second something worse, the third something painful.
pub const LEARN_TIERS: &[&[Trick]] = &[
    &[Trick::Chatter, Trick::Shove],
    &[Trick::Numb, Trick::Leech, Trick::Bleed],
    &[Trick::Venom, Trick::Forget, Trick::Maw],
];

/// Your arm is asleep this long after a numbing bite.
pub const NUMB_MS: i64 = 45_000;

pub const TRICKS: &[(&str, Trick)] = &[
    ("lamprey", Trick::Venom),
    ("usher", Trick::Usher),
    ("censor", Trick::Forget),
    ("cataloguer", Trick::Forget),
    ("engrosser", Trick::Enter),
    ("tipstaff", Trick::Below),
];

/// Venom: this many ticks, this far apart, this much each.
pub const VENOM_TICKS: i64 = 4;
pub const VENOM_EVERY_S: i64 = 15;
pub const VENOM_DAMAGE: f64 = 2.0;
pub const JOB_VENOM: &str = "disremembered.venom";
/// The ray's aim: the chance it finds your largest memory is level/(level+AIM_SCALE).
pub const AIM_SCALE: f64 = 300.0;
/// The ushering and the ray each land one hit in this many.
pub const ONE_IN: u32 = 2;
/// A trick on the record: `{trick, creature, kind, citizen, ...}`. Public.
pub const TRICK: &str = "disremembered.trick";

/// What a creature has learned from its kills, oldest first.
pub fn learned_of(world: &World, creature: EntityId) -> Vec<Trick> {
    match world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(CREATURE))
        .and_then(|c| c.fields.get("learned"))
    {
        Some(foxing_store::Field::Plain(Value::Array(a))) => a
            .iter()
            .filter_map(|v| v.as_str().and_then(learned_trick))
            .collect(),
        Some(foxing_store::Field::Plain(Value::String(s))) => {
            learned_trick(s).into_iter().collect()
        }
        _ => Vec::new(),
    }
}

pub fn trick_of(world: &World, creature: EntityId) -> Option<Trick> {
    // The worst thing it learned from a kill plays; else what its kind does.
    if let Some(t) = learned_of(world, creature).last() {
        return Some(*t);
    }
    let kind = plain_str(world, creature, CREATURE, "name")?;
    TRICKS.iter().find(|(k, _)| *k == kind).map(|(_, t)| *t)
}

/// After a creature's strike that left the citizen standing: the trick, if its kind
/// has one and it lands. Returns what to say and whether the fight is over.
pub fn trick(
    world: &mut World,
    curves: &super::Curves,
    creature: EntityId,
    citizen: EntityId,
    actor: &str,
) -> Result<(Option<String>, bool), String> {
    play(world, curves, creature, citizen, actor, false)
}

/// Dying throes: as a creature of the Stacks comes apart, its trick fires once more,
/// and the venom and the ray always land. A strong name felled everything in one
/// hit and the stamp never got a turn; now it gets the last one.
pub fn throes(
    world: &mut World,
    curves: &super::Curves,
    creature: EntityId,
    citizen: EntityId,
    actor: &str,
) -> Result<(Option<String>, bool), String> {
    let (line, over) = play(world, curves, creature, citizen, actor, true)?;
    Ok((
        line.map(|l| format!("Even as it comes apart: {}", lower_first(&l))),
        over,
    ))
}

fn lower_first(s: &str) -> String {
    // "A ray…" stays; "Its bite…" and "The usher…" lose their capital.
    let mut c = s.chars();
    match c.next() {
        Some(f) if s.starts_with("A ray") || s.starts_with("An engrosser") => {
            format!("{f}{}", c.as_str())
        }
        Some(f) => format!("{}{}", f.to_lowercase(), c.as_str()),
        None => String::new(),
    }
}

fn play(
    world: &mut World,
    curves: &super::Curves,
    creature: EntityId,
    citizen: EntityId,
    actor: &str,
    throes: bool,
) -> Result<(Option<String>, bool), String> {
    let Some(t) = trick_of(world, creature) else {
        return Ok((None, false));
    };
    let kind = plain_str(world, creature, CREATURE, "name")
        .unwrap_or("creature")
        .to_string();
    match t {
        Trick::Venom => {
            // A bite resets the venom rather than stacking it.
            let due = world
                .now()
                .plus(foxing_time::Duration::seconds(VENOM_EVERY_S));
            world.schedule_once(
                JOB_VENOM,
                due,
                json!({"citizen": citizen.0, "actor": actor, "left": VENOM_TICKS, "creature": creature.0}),
                |p| p["citizen"] == citizen.0,
            )?;
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "venom", "creature": creature.0, "kind": kind, "citizen": actor}),
            );
            Ok((
                Some("Its bite stays in you: the venom will work for a minute.".to_string()),
                false,
            ))
        }
        Trick::Usher => {
            if world.roll(ONE_IN) != 0 {
                return Ok((None, false));
            }
            let here = world.store.entity(citizen).and_then(|e| e.parent);
            let rooms: Vec<EntityId> = KEYS
                .iter()
                .filter_map(|k| super::rooms::room_by_key(world, k))
                .filter(|r| Some(*r) != here)
                .collect();
            let Some(&to) = rooms.get(world.roll(rooms.len().max(1) as u32) as usize) else {
                return Ok((None, false));
            };
            super::combat::leave_fight(world, actor, citizen)?;
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityMoved {
                        id: citizen,
                        parent: Some(to),
                    },
                )
                .map_err(|e| e.to_string())?;
            super::building::visited(world, to)?;
            let _ = super::atlas::mark_seen(world, actor, to);
            super::combat::on_arrival(world, actor, citizen, to)?;
            let rname = plain_str(world, to, ROOM, "name")
                .unwrap_or("?")
                .to_string();
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "ushered", "creature": creature.0, "kind": kind, "citizen": actor, "to": to.0, "room_name": rname}),
            );
            Ok((
                Some(format!(
                    "The usher takes your elbow and the shelves go past very fast. You are in {rname}."
                )),
                true,
            ))
        }
        Trick::Forget => {
            if !throes && world.roll(ONE_IN) != 0 {
                return Ok((None, false));
            }
            let Some(f) = super::find_citizen(world, actor) else {
                return Ok((None, false));
            };
            let chunks: Vec<EntityId> = world
                .store
                .children(f.window)
                .filter(|e| e.kind == "chunk")
                .map(|e| e.id)
                .collect();
            if chunks.is_empty() {
                return Ok((
                    Some("The stamp comes down on nothing: you had nothing to forget.".to_string()),
                    false,
                ));
            }
            // The ray reaches for what matters, more surely the more you hold: with
            // a chance on the death curve (level/(level+300): one in two at three
            // hundred, three in four at nine hundred) it finds your largest memory;
            // otherwise it lands by weight of value, so a tenth of an XP is seldom
            // what it takes. (A delver at 780 watched two rays halve chaff while a
            // 358 sat untouched, then found the Stacks a net loss when they did not.)
            let now = world.now();
            let value_of = |w: &World, id: EntityId| {
                w.store
                    .read_field(id, CHUNK, "value", now)
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            };
            let level = super::memory::level(world, f.window);
            let aim = level.max(0.0) / (level.max(0.0) + AIM_SCALE);
            let aimed = (world.roll(1000) as f64) < aim * 1000.0;
            let id = if aimed {
                chunks
                    .iter()
                    .copied()
                    .max_by(|a, b| {
                        value_of(world, *a)
                            .partial_cmp(&value_of(world, *b))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap_or(chunks[0])
            } else {
                let weights: Vec<f64> = chunks
                    .iter()
                    .map(|&c| value_of(world, c).max(0.01))
                    .collect();
                let total: f64 = weights.iter().sum();
                let mut at = (world.roll(1_000_000) as f64 / 1_000_000.0) * total;
                let mut pick = chunks[chunks.len() - 1];
                for (c, w) in chunks.iter().zip(&weights) {
                    if at < *w {
                        pick = *c;
                        break;
                    }
                    at -= w;
                }
                pick
            };
            let was = value_of(world, id);
            // The cataloguer takes two thirds; a censor, half.
            let keep = if kind == "cataloguer" { 1.0 / 3.0 } else { 0.5 };
            world
                .record(
                    Actor::Driver,
                    StoreEvent::FieldCommitted {
                        id,
                        component: CHUNK.into(),
                        field: "value".into(),
                        value: was * keep,
                    },
                )
                .map_err(|e| e.to_string())?;
            let source = plain_str(world, id, CHUNK, "source")
                .unwrap_or("?")
                .to_string();
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "forget", "creature": creature.0, "kind": kind, "citizen": actor, "chunk": id.0, "source": source, "was": was, "now": was * keep, "aimed": aimed, "aim": (aim * 1000.0).round() / 1000.0}),
            );
            Ok((
                Some(format!(
                    "A ray of forgetfulness{}: chunk-{} ({source}) is {} what it was, {:.1} now.",
                    if aimed && chunks.len() > 1 {
                        ", and it finds what matters"
                    } else {
                        ""
                    },
                    id.0,
                    if keep < 0.5 { "a third of" } else { "half" },
                    was * keep
                )),
                false,
            ))
        }
        Trick::Enter => {
            if !throes && world.roll(ONE_IN) != 0 {
                return Ok((None, false));
            }
            let Some(f) = super::find_citizen(world, actor) else {
                return Ok((None, false));
            };
            let chunks: Vec<EntityId> = world
                .store
                .children(f.window)
                .filter(|e| e.kind == "chunk")
                .map(|e| e.id)
                .collect();
            if chunks.is_empty() {
                return Ok((
                    Some("The engrosser writes down that you had nothing to say.".to_string()),
                    false,
                ));
            }
            let id = chunks[world.roll(chunks.len() as u32) as usize];
            let vividity = super::plain_num(world, id, CHUNK, "vividity").unwrap_or(0.0);
            world
                .record(
                    Actor::Driver,
                    StoreEvent::FieldRecurved {
                        id,
                        component: CHUNK.into(),
                        field: "value".into(),
                        curve: curves.palace(vividity),
                    },
                )
                .map_err(|e| e.to_string())?;
            world
                .record(
                    Actor::Driver,
                    StoreEvent::FieldSet {
                        id,
                        component: CHUNK.into(),
                        field: "tier".into(),
                        value: json!("palace"),
                    },
                )
                .map_err(|e| e.to_string())?;
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityMoved {
                        id,
                        parent: Some(f.palace),
                    },
                )
                .map_err(|e| e.to_string())?;
            let source = plain_str(world, id, CHUNK, "source")
                .unwrap_or("?")
                .to_string();
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "entered", "creature": creature.0, "kind": kind, "citizen": actor, "chunk": id.0, "source": source}),
            );
            Ok((
                Some(format!(
                    "An engrosser enters chunk-{} ({source}) into the minutes: it is on your shelf now, slow to fade and counting for nothing until you reminisce it.",
                    id.0
                )),
                false,
            ))
        }
        Trick::Below => {
            if world.roll(ONE_IN) != 0 {
                return Ok((None, false));
            }
            let Some(to) = super::rooms::room_by_key(world, "petition-gallery") else {
                return Ok((None, false));
            };
            if world.store.entity(citizen).and_then(|e| e.parent) == Some(to) {
                return Ok((None, false));
            }
            super::combat::leave_fight(world, actor, citizen)?;
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityMoved {
                        id: citizen,
                        parent: Some(to),
                    },
                )
                .map_err(|e| e.to_string())?;
            super::building::visited(world, to)?;
            let _ = super::atlas::mark_seen(world, actor, to);
            super::combat::on_arrival(world, actor, citizen, to)?;
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "below", "creature": creature.0, "kind": kind, "citizen": actor, "to": to.0, "room_name": "The Petitioners' Gallery"}),
            );
            Ok((
                Some("The tipstaff shows you below the bar. You are in the Petitioners' Gallery, and your number has not been called.".to_string()),
                true,
            ))
        }
        Trick::Chatter => {
            if world.roll(ONE_IN) != 0 {
                return Ok((None, false));
            }
            let wrong: String = actor.chars().rev().collect();
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "chatter", "creature": creature.0, "kind": kind, "citizen": actor, "said": wrong}),
            );
            Ok((
                Some(format!(
                    "The {kind} says your name back, wrong: \"{wrong}.\" It seems pleased."
                )),
                false,
            ))
        }
        Trick::Shove => {
            if world.roll(ONE_IN) != 0 {
                return Ok((None, false));
            }
            let Some(here) = world.store.entity(citizen).and_then(|e| e.parent) else {
                return Ok((None, false));
            };
            let exits: Vec<(String, EntityId)> =
                super::rooms::exits_of(world, here).into_iter().collect();
            let Some((via, to)) = exits
                .get(world.roll(exits.len().max(1) as u32) as usize)
                .cloned()
            else {
                return Ok((None, false));
            };
            super::combat::leave_fight(world, actor, citizen)?;
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityMoved {
                        id: citizen,
                        parent: Some(to),
                    },
                )
                .map_err(|e| e.to_string())?;
            super::building::visited(world, to)?;
            let _ = super::atlas::mark_seen(world, actor, to);
            super::combat::on_arrival(world, actor, citizen, to)?;
            let rname = plain_str(world, to, ROOM, "name")
                .unwrap_or("?")
                .to_string();
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "shoved", "creature": creature.0, "kind": kind, "citizen": actor, "to": to.0, "room_name": rname, "via": via}),
            );
            Ok((
                Some(format!(
                    "The {kind} shoves you {via}, and the door does the rest. You are in {rname}."
                )),
                true,
            ))
        }
        Trick::Numb => {
            if world.roll(ONE_IN) != 0 {
                return Ok((None, false));
            }
            let until = world.now().0 + NUMB_MS;
            world
                .record(
                    Actor::Driver,
                    StoreEvent::FieldSet {
                        id: citizen,
                        component: super::BODY.into(),
                        field: "numb_until".into(),
                        value: json!(until),
                    },
                )
                .map_err(|e| e.to_string())?;
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "numb", "creature": creature.0, "kind": kind, "citizen": actor, "until": until}),
            );
            Ok((Some(format!("The {kind}'s bite leaves your arm asleep: your strikes do half for a little while.")), false))
        }
        Trick::Leech => {
            if world.roll(ONE_IN) != 0 {
                return Ok((None, false));
            }
            let bite = plain_num(world, creature, CREATURE, "attack").unwrap_or(1.0);
            let max = plain_num(world, creature, CREATURE, "max_hp").unwrap_or(1.0);
            let hp = plain_num(world, creature, CREATURE, "hp").unwrap_or(0.0);
            let mended = (hp + bite).min(max);
            super::combat::set_creature_hp(world, creature, mended)?;
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "leech", "creature": creature.0, "kind": kind, "citizen": actor, "mended": mended - hp}),
            );
            Ok((Some(format!("The {kind} is the better for the bite: it mends by what it took ({mended:.0} of {max:.0}).")), false))
        }
        Trick::Bleed => {
            if world.roll(ONE_IN) != 0 {
                return Ok((None, false));
            }
            // A bandage carried takes the bite: spent, and the bleeding never starts.
            if let Some(bandage) = super::craft::carries(world, citizen, "bandage") {
                super::craft::spend(world, actor, bandage).map_err(|e| e.to_string())?;
                let _ = world.note(
                    Actor::Driver,
                    TRICK,
                    json!({"trick": "bleed", "creature": creature.0, "kind": kind, "citizen": actor, "bandaged": true}),
                );
                return Ok((
                    Some(format!(
                        "The {kind}'s bite would not close; your bandage takes it, and is spent."
                    )),
                    false,
                ));
            }
            let until = world.now().0 + super::combat::BLEED_MS;
            world
                .record(
                    Actor::Driver,
                    StoreEvent::FieldSet {
                        id: citizen,
                        component: super::BODY.into(),
                        field: "bleed_until".into(),
                        value: json!(until),
                    },
                )
                .map_err(|e| e.to_string())?;
            super::combat::recurve_mending(world, citizen).map_err(|e| e.to_string())?;
            let citizen_id = citizen.0;
            let _ = world.cancel(super::combat::JOB_STAUNCH, |p| p["citizen"] == citizen_id);
            world.schedule_once(
                super::combat::JOB_STAUNCH,
                foxing_time::Timestamp(until),
                json!({"citizen": citizen.0, "actor": actor}),
                |p| p["citizen"] == citizen_id,
            )?;
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "bleed", "creature": creature.0, "kind": kind, "citizen": actor, "until": until}),
            );
            Ok((
                Some(format!(
                    "The {kind}'s bite will not close: nothing mends for {}.",
                    super::combat::human(super::combat::BLEED_MS)
                )),
                false,
            ))
        }
        Trick::Maw => {
            if world.roll(ONE_IN) != 0 {
                return Ok((None, false));
            }
            // A second bite in the same breath; it stops short of the kill, so you remember.
            let bite = plain_num(world, creature, CREATURE, "attack").unwrap_or(1.0);
            let before = super::combat::hp(world, citizen);
            let after = (before - bite).max(1.0);
            if before - after < 0.5 {
                return Ok((None, false));
            }
            super::combat::commit_hp(world, citizen, after).map_err(|e| e.to_string())?;
            let _ = world.note(
                Actor::Driver,
                TRICK,
                json!({"trick": "maw", "creature": creature.0, "kind": kind, "citizen": actor, "bite": before - after}),
            );
            Ok((Some(format!("And again, before you can step back: {:.0} \u{2192} {:.0}. It stops there, on purpose.", before.ceil(), after.ceil())), false))
        }
    }
}

pub fn register(world: &mut World) {
    world.on_job(
        JOB_VENOM,
        Box::new(|w: &mut World, job| {
            let citizen = EntityId(job.payload["citizen"].as_u64().unwrap_or(0));
            let actor = job.payload["actor"].as_str().unwrap_or("").to_string();
            let left = job.payload["left"].as_i64().unwrap_or(0);
            if w.store.entity(citizen).is_none() || left <= 0 {
                return Ok(());
            }
            let Some(f) = super::find_citizen(w, &actor) else {
                return Ok(());
            };
            let after = super::combat::hp(w, citizen) - VENOM_DAMAGE;
            super::combat::commit_hp(w, citizen, after).map_err(|e| e.to_string())?;
            if after <= 0.0 {
                // The one whose venom it was keeps what you lost, if it still stands.
                let biter = job.payload["creature"].as_u64().map(EntityId).filter(|c| w.store.entity(*c).is_some());
                let d = super::combat::die(w, &actor, &f, biter).map_err(|e| e.to_string())?;
                super::combat::narrate(w, &actor, format!("The venom finishes what the lamprey started. You go down.{}", d.said()));
                return Ok(());
            }
            super::combat::narrate(
                w,
                &actor,
                format!(
                    "The venom works: {VENOM_DAMAGE:.0}, {:.0} \u{2192} {:.0} of {:.0}.{}",
                    (after + VENOM_DAMAGE).ceil(),
                    after.ceil(),
                    super::MAX_HP,
                    if left == 1 {
                        " It is out of you now.".to_string()
                    } else {
                        format!(" {} more tick{} to come, {VENOM_EVERY_S} s apart.", left - 1, if left == 2 { "" } else { "s" })
                    }
                ),
            );
            if left > 1 {
                let due = w.now().plus(foxing_time::Duration::seconds(VENOM_EVERY_S));
                w.schedule_once(
                    JOB_VENOM,
                    due,
                    json!({"citizen": citizen.0, "actor": actor, "left": left - 1, "creature": job.payload["creature"]}),
                    |p| p["citizen"] == citizen.0,
                )?;
            }
            Ok(())
        }),
    );
}
