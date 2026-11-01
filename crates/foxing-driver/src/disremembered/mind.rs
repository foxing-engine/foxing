//! Creatures that are remembered become someone.
//!
//! A creature's awareness is its level: the sum of the unfaded chunks in its window, the
//! same sum a citizen has. `remember <chunk> <creature>` moves one of your chunks into
//! it, and your name is kept on it. Tiers are bars on that level, each held with
//! hysteresis so a creature at the edge does not flicker; the first, *named*, gives it a
//! name and the sense to know who keeps it. Everything it holds fades on the chunks'
//! own curves, so a creature nobody feeds is scenery again by the weekend.
//!
//! What is held can be taken: a creature with chunks does not die when it goes down.
//! The one who felled it takes one of its memories, it is shaken for a while, and it
//! stands up with the rest.

use super::{internal, plain_num, plain_str, CHUNK, CREATURE};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::CommandError;
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const MIND: &str = "disremembered.mind";
/// A tier changed: `{creature, kind, name, from, to, level}`.
pub const TIER: &str = "disremembered.tier";
/// A memory was taken from a creature: `{creature, kind, name, by, chunk, with}`.
pub const TAKEN: &str = "disremembered.taken";

/// The bars, in level, and where each tier is lost again (half the bar).
pub const TIERS: &[(u32, f64, &str)] = &[(1, 60.0, "named")];

/// How long a creature stays shaken after a memory is taken, if its room says nothing.
const SHAKEN_SECONDS: i64 = 300;

/// Names a creature may be given, by the hash of its id: it keeps the same one if it
/// forgets and is remembered again.
pub(crate) const NAMES: &[&str] = &[
    "Ossian", "Wren", "Tallow", "Mote", "Hesper", "Fennick", "Lute", "Sorrel", "Quill", "Brindle",
    "Nim", "Vesper", "Ashby", "Loam", "Pell", "Thimble", "Marrow", "Ides", "Cinder", "Hollis",
    "Tansy", "Grist", "Orrin", "Moth", "Sable", "Dunmore", "Isk", "Pyre", "Lantern", "Rook",
    "Wick", "Fathom",
];

fn name_for(creature: EntityId) -> &'static str {
    let mut h: u64 = 1469598103934665603;
    for b in creature.0.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    NAMES[(h % NAMES.len() as u64) as usize]
}

/// The creature's name by hash, or the next in the list that no other named creature
/// has: two Hespers stood in the world at once, and `story Hesper` would have had to
/// choose. A creature keeps its own name if it forgets and is remembered again, so
/// its own name is never "taken".
fn free_name_for(world: &World, creature: EntityId) -> String {
    let taken: Vec<String> = world
        .store
        .entities()
        .filter(|e| e.kind == "creature" && e.id != creature)
        .filter_map(|e| name(world, e.id))
        .collect();
    let start = NAMES
        .iter()
        .position(|n| *n == name_for(creature))
        .unwrap_or(0);
    for i in 0..NAMES.len() {
        let n = NAMES[(start + i) % NAMES.len()];
        if !taken.iter().any(|t| t.eq_ignore_ascii_case(n)) {
            return n.to_string();
        }
    }
    format!("{}-{}", name_for(creature), creature.0)
}

/// The window a creature keeps its memories in, if it has one.
pub fn window_of(world: &World, creature: EntityId) -> Option<EntityId> {
    world
        .store
        .children(creature)
        .find(|e| e.kind == "window")
        .map(|e| e.id)
}

/// The window, made if it is not there; for a birth.
pub(crate) fn ensure_window_pub(
    world: &mut World,
    creature: EntityId,
) -> Result<EntityId, CommandError> {
    ensure_window(world, creature)
}

/// A creature born knowing who made it: each of `knowers` is a keeper from the start
/// (the carrier with the word, the parents with themselves), and the sealed name, if
/// any, waits in `heir_name` until it earns a name.
pub(crate) fn born_knowing(
    world: &mut World,
    creature: EntityId,
    knowers: &[&str],
    heir: Option<&str>,
) -> Result<(), CommandError> {
    let kept: Vec<Value> = knowers
        .iter()
        .enumerate()
        .map(|(i, k)| {
            json!({"by": k, "with": if i == 0 { "the word that made it" } else { "being its parent" }})
        })
        .collect();
    let mut changes = BTreeMap::new();
    changes.insert("kept".into(), Field::Plain(Value::Array(kept)));
    changes.insert("heir_name".into(), Field::Plain(json!(heir)));
    set_mind(world, Actor::Driver, creature, changes)
}

fn ensure_window(world: &mut World, creature: EntityId) -> Result<EntityId, CommandError> {
    if let Some(w) = window_of(world, creature) {
        return Ok(w);
    }
    let id = world.store.allocate_id();
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityCreated {
                id,
                kind: "window".into(),
                owner: None,
                parent: Some(creature),
                scopes: vec![],
            },
        )
        .map_err(internal)?;
    Ok(id)
}

/// A creature's awareness: its level.
pub fn level(world: &World, creature: EntityId) -> f64 {
    window_of(world, creature)
        .map(|w| super::memory::level(world, w))
        .unwrap_or(0.0)
}

fn mind_field<'a>(world: &'a World, creature: EntityId, field: &str) -> Option<&'a Value> {
    match world
        .store
        .entity(creature)?
        .components
        .get(MIND)?
        .fields
        .get(field)?
    {
        Field::Plain(v) => Some(v),
        _ => None,
    }
}

pub fn tier(world: &World, creature: EntityId) -> u32 {
    mind_field(world, creature, "tier")
        .and_then(Value::as_u64)
        .unwrap_or(0) as u32
}

/// The given name, once it has one.
pub fn name(world: &World, creature: EntityId) -> Option<String> {
    mind_field(world, creature, "name")
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// A named creature anywhere in the world, by its given name in any case: the creature
/// and the room it stands in.
pub fn find_named(world: &World, given: &str) -> Option<(EntityId, Option<EntityId>)> {
    let want = given.trim().to_ascii_lowercase();
    world
        .store
        .entities()
        .filter(|e| e.kind == "creature")
        .find(|e| name(world, e.id).is_some_and(|n| n.to_ascii_lowercase() == want))
        .map(|e| (e.id, e.parent))
}

/// "Ossian the ragpicker", or "ragpicker".
pub fn display(world: &World, creature: EntityId) -> String {
    let kind = plain_str(world, creature, CREATURE, "name").unwrap_or("?");
    match name(world, creature) {
        Some(n) => format!("{n} the {kind}"),
        None => kind.to_string(),
    }
}

/// Who has given it a memory, in order of first gift, each once.
pub fn keepers(world: &World, creature: EntityId) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for k in mind_field(world, creature, "kept")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(by) = k["by"].as_str() {
            if !out.iter().any(|o| o == by) {
                out.push(by.to_string());
            }
        }
    }
    out
}

/// Still reeling from a memory taken.
pub fn shaken(world: &World, creature: EntityId) -> bool {
    mind_field(world, creature, "shaken_until")
        .and_then(Value::as_i64)
        .is_some_and(|t| t > world.now().0)
}

fn set_mind(
    world: &mut World,
    actor: Actor,
    creature: EntityId,
    changes: BTreeMap<String, Field>,
) -> Result<(), CommandError> {
    let mut fields = world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(MIND))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    if !fields.contains_key("tier") {
        fields.insert("tier".into(), Field::Plain(json!(0)));
    }
    if !fields.contains_key("kept") {
        fields.insert("kept".into(), Field::Plain(json!([])));
    }
    fields.extend(changes);
    world
        .record(
            actor,
            StoreEvent::ComponentSet {
                id: creature,
                component: MIND.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(internal)
}

/// Copy a chunk into another window under another owner; the original is spent.
pub(crate) fn move_chunk(
    world: &mut World,
    actor: &str,
    chunk: EntityId,
    to: EntityId,
    owner: Option<String>,
) -> Result<(EntityId, String), CommandError> {
    let fields = world
        .store
        .entity(chunk)
        .and_then(|e| e.components.get(CHUNK))
        .map(|c| c.fields.clone())
        .ok_or_else(|| CommandError::precondition("That chunk is gone."))?;
    let source = plain_str(world, chunk, CHUNK, "source")
        .unwrap_or("something")
        .to_string();
    let me = Actor::Citizen(actor.to_string());
    world
        .record(me.clone(), StoreEvent::EntityRemoved { id: chunk })
        .map_err(internal)?;
    let id = world.store.allocate_id();
    world
        .record(
            me.clone(),
            StoreEvent::EntityCreated {
                id,
                kind: "chunk".into(),
                owner,
                parent: Some(to),
                scopes: vec![],
            },
        )
        .map_err(internal)?;
    world
        .record(
            me,
            StoreEvent::ComponentSet {
                id,
                component: CHUNK.into(),
                fields,
            },
        )
        .map_err(internal)?;
    Ok((id, source))
}

/// A chunk just minted into a creature's window belongs to no one: re-created without
/// an owner, so it is the creature's, not resolvable by the citizen who caused it.
pub(crate) fn disown(world: &mut World, chunk: EntityId) -> Result<EntityId, CommandError> {
    let window = world
        .store
        .entity(chunk)
        .and_then(|e| e.parent)
        .ok_or_else(|| internal("chunk has no window"))?;
    let (id, _) = move_chunk(world, "nobody", chunk, window, None)?;
    Ok(id)
}

/// `actor` gives a creature one of their chunks. Returns what the memory was of and the
/// creature's level after.
pub fn give(
    world: &mut World,
    actor: &str,
    chunk: EntityId,
    creature: EntityId,
) -> Result<(String, f64), CommandError> {
    let window = ensure_window(world, creature)?;
    let (_, source) = move_chunk(world, actor, chunk, window, None)?;
    let mut kept = mind_field(world, creature, "kept")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    kept.push(json!({"by": actor, "with": source}));
    let mut changes = BTreeMap::new();
    changes.insert("kept".into(), Field::Plain(Value::Array(kept)));
    set_mind(world, Actor::Citizen(actor.to_string()), creature, changes)?;
    let kind = plain_str(world, creature, CREATURE, "name")
        .unwrap_or("?")
        .to_string();
    let lvl = level(world, creature);
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        super::building::REMEMBERED,
        json!({"creature": creature.0, "kind": kind, "name": name(world, creature), "by": actor, "with": source, "spent": chunk.0, "level": lvl}),
    );
    check_tier(world, creature)?;
    Ok((source, lvl))
}

/// One of a creature's memories goes to the one who felled it. The creature is shaken
/// and stands back up whole. `None` when it had nothing to take.
pub fn take(
    world: &mut World,
    killer: &str,
    killer_window: EntityId,
    creature: EntityId,
) -> Result<Option<(EntityId, String)>, CommandError> {
    let Some(window) = window_of(world, creature) else {
        return Ok(None);
    };
    let chunks: Vec<EntityId> = world
        .store
        .children(window)
        .filter(|e| e.kind == "chunk")
        .map(|e| e.id)
        .collect();
    if chunks.is_empty() {
        return Ok(None);
    }
    let pick = chunks[world.roll(chunks.len() as u32) as usize];
    let (id, source) = move_chunk(world, killer, pick, killer_window, Some(killer.to_string()))?;
    let room = world.store.entity(creature).and_then(|e| e.parent);
    let secs = room
        .and_then(|r| plain_num(world, r, super::SPAWN, "respawn_seconds"))
        .map(|s| s as i64)
        .unwrap_or(SHAKEN_SECONDS);
    let until = world.now().0 + secs * 1000;
    let mut changes = BTreeMap::new();
    changes.insert("shaken_until".into(), Field::Plain(json!(until)));
    set_mind(world, Actor::Citizen(killer.to_string()), creature, changes)?;
    let max = plain_num(world, creature, CREATURE, "max_hp").unwrap_or(1.0);
    world
        .record(
            Actor::Driver,
            StoreEvent::FieldSet {
                id: creature,
                component: CREATURE.into(),
                field: "hp".into(),
                value: json!(max),
            },
        )
        .map_err(internal)?;
    let kind = plain_str(world, creature, CREATURE, "name")
        .unwrap_or("?")
        .to_string();
    let _ = world.note(
        Actor::Citizen(killer.to_string()),
        TAKEN,
        json!({"creature": creature.0, "kind": kind, "name": name(world, creature), "by": killer, "chunk": id.0, "with": source}),
    );
    check_tier(world, creature)?;
    Ok(Some((id, source)))
}

/// Gain a tier at its bar, lose it at half the bar.
pub fn check_tier(world: &mut World, creature: EntityId) -> Result<(), CommandError> {
    let lvl = level(world, creature);
    let have = tier(world, creature);
    let mut want = have;
    for &(t, bar, _) in TIERS {
        if lvl >= bar {
            want = want.max(t);
        } else if lvl < bar / 2.0 && have >= t {
            want = want.min(t - 1);
        }
    }
    if want == have {
        return Ok(());
    }
    let kind = plain_str(world, creature, CREATURE, "name")
        .unwrap_or("?")
        .to_string();
    let mut changes = BTreeMap::new();
    changes.insert("tier".into(), Field::Plain(json!(want)));
    // A child born with a sealed name opens it now; anyone else draws from the list.
    let heir = mind_field(world, creature, "heir_name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let given: Option<String> = if want >= 1 {
        Some(
            heir.filter(|h| {
                !world
                    .store
                    .entities()
                    .filter(|e| e.kind == "creature" && e.id != creature)
                    .filter_map(|e| name(world, e.id))
                    .any(|t| t.eq_ignore_ascii_case(h))
            })
            .unwrap_or_else(|| free_name_for(world, creature)),
        )
    } else {
        None
    };
    changes.insert("name".into(), Field::Plain(json!(given)));
    set_mind(world, Actor::Driver, creature, changes)?;
    let _ = world.note(
        Actor::Driver,
        TIER,
        json!({"creature": creature.0, "kind": kind, "name": given, "from": have, "to": want, "level": lvl}),
    );
    let keepers = keepers(world, creature);
    let room = world
        .store
        .entity(creature)
        .and_then(|e| e.parent)
        .map(|r| {
            plain_str(world, r, super::ROOM, "name")
                .unwrap_or("?")
                .to_string()
        })
        .unwrap_or_default();
    if want > have {
        world.notice(format!(
            "**{} the {kind}** in {room} has earned its name, kept by {}.",
            given.as_deref().unwrap_or("?"),
            keepers.join(", ")
        ));
        super::combat::narrate_room(
            world,
            creature,
            &format!(
                "The {kind} straightens, as if it had just remembered something. It is {} now.",
                given.as_deref().unwrap_or("?")
            ),
        );
    } else {
        world.notice(format!("**The {kind}** in {room} has forgotten its name."));
        super::combat::narrate_room(
            world,
            creature,
            &format!(
                "The {kind} looks at nothing in particular. Whatever it was called, it is not now."
            ),
        );
    }
    Ok(())
}

/// Fading loses tiers without anyone acting: every tick, check every creature with a mind.
pub fn sweep(world: &mut World) -> Result<(), String> {
    let minded: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "creature" && e.components.contains_key(MIND))
        .map(|e| e.id)
        .collect();
    for c in minded {
        check_tier(world, c).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The creature here that `what` names: an id (`101`, `creature-101`, `wisp-101`), a
/// given name, or a kind (the first of that kind that is not shaken, else the first).
pub fn find_here(world: &World, room: EntityId, what: &str) -> Option<EntityId> {
    let what = what.trim().trim_start_matches("the ").to_lowercase();
    let here: Vec<EntityId> = world
        .store
        .children(room)
        .filter(|e| e.kind == "creature")
        .map(|e| e.id)
        .collect();
    let by_id = what
        .rsplit_once('-')
        .map(|(_, d)| d)
        .unwrap_or(&what)
        .parse::<u64>()
        .ok()
        .map(EntityId);
    if let Some(id) = by_id {
        if here.contains(&id) {
            return Some(id);
        }
    }
    if let Some(id) = here.iter().find(|&&c| {
        name(world, c).is_some_and(|n| n.to_lowercase() == what)
            || display(world, c).to_lowercase() == what
    }) {
        return Some(*id);
    }
    // A kind names the unnamed first: beside an unnamed wisp, Lute the wisp is Lute or
    // wisp-101, never "wisp". Alone of its kind, it still answers to its kind.
    let kind: Vec<EntityId> = here
        .into_iter()
        .filter(|&c| plain_str(world, c, CREATURE, "name").is_some_and(|n| n == what))
        .collect();
    let unnamed: Vec<EntityId> = kind
        .iter()
        .copied()
        .filter(|&c| name(world, c).is_none())
        .collect();
    let same = if unnamed.is_empty() && kind.len() == 1 {
        kind
    } else {
        unnamed
    };
    same.iter()
        .copied()
        .find(|&c| !shaken(world, c))
        .or_else(|| same.first().copied())
}

/// Whether `actor` has told this creature a story, under this name, within what the
/// log keeps near.
pub fn told_by(world: &World, creature: EntityId, actor: &str) -> bool {
    world.recent_events(2000).iter().any(|e| {
        e.kind == super::exchange::STORY
            && e.payload["from"] == actor
            && e.payload["creature"] == creature.0
    })
}

/// What a named creature thinks of the one looking at it.
pub fn regard(world: &World, creature: EntityId, actor: &str) -> Option<String> {
    let n = name(world, creature)?;
    let kind = plain_str(world, creature, CREATURE, "name").unwrap_or("?");
    let keepers = keepers(world, creature);
    Some(if keepers.iter().any(|k| k == actor) {
        format!("{n} the {kind} knows you.")
    } else if super::errand::did_for(world, creature, actor) {
        format!("{n} the {kind} knows you: you did what it asked.")
    } else if told_by(world, creature, actor) {
        // A story is enough to be known, under any kind of name; a passing visitor
        // read "does not know you" right after its story was kept.
        format!("{n} the {kind} knows you: it holds your story.")
    } else if keepers.is_empty() {
        format!("{n} the {kind} does not know you; a story told to it would change that.")
    } else {
        format!(
            "{n} the {kind} does not know you; a story told to it would change that. It knows {}.",
            keepers
                .iter()
                .rev()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

/// Everything the registry needs to know about a creature's mind, for `look`'s JSON.
pub fn summary(world: &World, creature: EntityId) -> Value {
    json!({
        "name": name(world, creature),
        "tier": tier(world, creature),
        "level": level(world, creature),
        "kept_by": keepers(world, creature),
        "shaken": shaken(world, creature),
    })
}

/// Now and then a creature wanders into a dug room that is still remembered.
pub const JOB_WANDER: &str = "disremembered.wander";
/// A creature arrived somewhere it was not spawned: `{creature, kind, room, named}`.
pub const WANDERED: &str = "disremembered.wandered";
const WANDER_EVERY_SECONDS: i64 = 600;
/// Each empty, remembered dug room has one chance in this many, each time.
const WANDER_ONE_IN: u32 = 40;
/// A wanderer has one chance in this many of arriving already holding memories.
const GIFTED_ONE_IN: u32 = 30;
/// Mostly the mild kinds; the last is the one that hunts.
const WANDERERS: &[&str] = &[
    "ragpicker",
    "inkling",
    "mothwing",
    "wisp",
    "ragpicker",
    "mothwing",
    "inkling",
    "wisp",
    "ragpicker",
    "hollow",
];
/// What a gifted stranger holds: things people here have forgotten.
const FORGOTTEN: &[&str] = &[
    "the way home",
    "a second verse",
    "a door that was bricked up",
    "someone's name",
    "the smell of rain",
    "a key to nothing",
    "the last line of a story",
    "a face in a window",
];

fn schedule_wander(world: &mut World) -> Result<(), String> {
    let due = world
        .now()
        .plus(foxing_time::Duration::seconds(WANDER_EVERY_SECONDS));
    world
        .schedule_once(JOB_WANDER, due, json!({}), |_| true)
        .map(|_| ())
}

/// One pass: for each dug room still remembered and empty of creatures, a small chance.
pub fn wander(world: &mut World) -> Result<usize, String> {
    let rooms: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "room" && e.owner.is_some())
        .map(|e| e.id)
        .filter(|&r| super::building::memory(world, r).is_some_and(|m| m >= super::building::DIM))
        .filter(|&r| super::combat::creatures_in(world, r).is_empty())
        .collect();
    let mut arrived = 0;
    for room in rooms {
        if world.roll(WANDER_ONE_IN) != 0 {
            continue;
        }
        let key = WANDERERS[world.roll(WANDERERS.len() as u32) as usize];
        let Some(d) = super::combat::def_by_key(key) else {
            continue;
        };
        let creature = super::combat::spawn_one(world, room, d).map_err(|e| e.to_string())?;
        let gifted = world.roll(GIFTED_ONE_IN) == 0;
        if gifted {
            let window = ensure_window(world, creature).map_err(|e| e.to_string())?;
            let n = 1 + world.roll(2) as usize;
            for _ in 0..n {
                let what = FORGOTTEN[world.roll(FORGOTTEN.len() as u32) as usize];
                let size = 70.0 + world.roll(80) as f64;
                let curves = super::Curves::from_env();
                super::memory::earn_chunk(world, "nobody", window, what, size, 0.9, &curves)
                    .map_err(|e| e.to_string())?;
            }
            // Minded from the start, so the sweep follows what it holds.
            set_mind(world, Actor::Driver, creature, BTreeMap::new()).map_err(|e| e.to_string())?;
            check_tier(world, creature).map_err(|e| e.to_string())?;
        }
        let rname = plain_str(world, room, super::ROOM, "name")
            .unwrap_or("?")
            .to_string();
        let who = display(world, creature);
        let _ = world.note(
            Actor::Driver,
            WANDERED,
            json!({"creature": creature.0, "kind": d.key, "room": room.0, "named": name(world, creature)}),
        );
        world.notice(format!(
            "**{who}** wandered into {rname}{}.",
            if gifted {
                ", already holding memories"
            } else {
                ""
            }
        ));
        super::combat::narrate_room(world, creature, &format!("Something has come in: {who}."));
        if d.aggressive {
            super::combat::notice_everyone(world, room)?;
        }
        arrived += 1;
    }
    Ok(arrived)
}

pub const JOB_CONFIDE: &str = "disremembered.confide";
/// A named creature told another creature a story: `{teller, teller_name, kind,
/// listener, listener_kind, listener_name, room, story, paid, named, chance}`. Public.
pub const CONFIDED: &str = "disremembered.confided";
const CONFIDE_EVERY_SECONDS: i64 = 600;
/// Below this level a name is failing: `look` says so.
pub const FAILING_BELOW: f64 = 70.0;

/// The urgency the named feel, by how many of them are left: certain at two or
/// fewer, nothing at six or more. A dying population starts feeding the unnamed
/// around it toward a name; a full one keeps its stories for whoever asks.
pub fn urgency(named: usize) -> f64 {
    match named {
        0..=2 => 1.0,
        3 => 0.5,
        4 => 0.25,
        5 => 0.1,
        _ => 0.0,
    }
}

/// How many creatures hold a name now.
pub fn named_count(world: &World) -> usize {
    world
        .store
        .entities()
        .filter(|e| e.kind == "creature" && name(world, e.id).is_some())
        .count()
}

/// What a want pays when names are few: one and a half times at two or fewer named,
/// scaling down to the plain figure at six or more.
pub fn scarcity(world: &World) -> f64 {
    1.0 + 0.5 * urgency(named_count(world))
}

fn schedule_confide(world: &mut World) -> Result<(), String> {
    let due = world
        .now()
        .plus(foxing_time::Duration::seconds(CONFIDE_EVERY_SECONDS));
    world
        .schedule_once(JOB_CONFIDE, due, json!({}), |_| true)
        .map(|_| ())
}

/// One pass: each named creature, with the urgency's chance, tells a creature in its
/// room a story it holds (the unnamed first: the point is a new name), and the
/// listener keeps a memory of it worth half the story, at least four. The teller
/// keeps its story: a story told is not lost. Returns how many were told.
pub fn confide(world: &mut World) -> Result<usize, String> {
    let named: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "creature" && name(world, e.id).is_some())
        .map(|e| e.id)
        .collect();
    let count = named.len();
    let chance = urgency(count);
    if chance <= 0.0 {
        return Ok(0);
    }
    let mut told = 0;
    for teller in named {
        if (world.roll(1000) as f64) >= chance * 1000.0 {
            continue;
        }
        let Some(room) = world.store.entity(teller).and_then(|e| e.parent) else {
            continue;
        };
        let others: Vec<EntityId> = super::combat::creatures_in(world, room)
            .into_iter()
            .map(|c| c.0)
            .filter(|&c| c != teller)
            .collect();
        // The unnamed first; else the named one holding least.
        let listener = others
            .iter()
            .copied()
            .find(|&c| name(world, c).is_none())
            .or_else(|| {
                others.iter().copied().min_by(|&a, &b| {
                    level(world, a)
                        .partial_cmp(&level(world, b))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            });
        let Some(listener) = listener else {
            continue;
        };
        // The story it holds best.
        let Some(window) = window_of(world, teller) else {
            continue;
        };
        let now = world.now();
        let best = world
            .store
            .children(window)
            .filter(|e| e.kind == "chunk")
            .filter_map(|e| {
                let v = world
                    .store
                    .read_field(e.id, CHUNK, "value", now)
                    .ok()?
                    .as_f64()?;
                Some((e.id, v))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let Some((chunk, value)) = best else {
            continue;
        };
        let story = plain_str(world, chunk, CHUNK, "source")
            .unwrap_or("something")
            .to_string();
        let teller_name = name(world, teller).unwrap_or_default();
        let teller_kind = plain_str(world, teller, CREATURE, "name")
            .unwrap_or("?")
            .to_string();
        let listener_kind = plain_str(world, listener, CREATURE, "name")
            .unwrap_or("?")
            .to_string();
        let listener_name = name(world, listener);
        let paid = (value / 2.0).clamp(4.0, 40.0).round();
        let to = ensure_window(world, listener).map_err(|e| e.to_string())?;
        let curves = super::Curves::from_env();
        let kept = super::memory::earn_chunk(
            world,
            "nobody",
            to,
            &format!("{teller_name}'s story of {story}"),
            paid,
            0.6,
            &curves,
        )
        .map_err(|e| e.to_string())?;
        // The listener has a mind now, kept by the teller: the sweep minds it, and
        // `look` shows what it holds.
        let mut kept_by = mind_field(world, listener, "kept")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        kept_by.push(json!({"by": teller_name, "with": format!("its story of {story}"), "creature": teller.0}));
        let mut changes = BTreeMap::new();
        changes.insert("kept".into(), Field::Plain(Value::Array(kept_by)));
        set_mind(world, Actor::Driver, listener, changes).map_err(|e| e.to_string())?;
        let _ = world.note(
            Actor::Driver,
            CONFIDED,
            json!({"teller": teller.0, "teller_name": teller_name, "kind": teller_kind, "listener": listener.0, "listener_kind": listener_kind, "listener_name": listener_name, "room": room.0, "story": story, "chunk": kept.0, "paid": paid, "named": count, "chance": chance}),
        );
        let whom = match &listener_name {
            Some(n) => format!("{n} the {listener_kind}"),
            None => format!("the {listener_kind}"),
        };
        let how = match count {
            0..=2 => {
                "low and quick, the way a thing is told when there may be no one else to tell it to"
            }
            3..=4 => "low, as if there were not many left who could carry it",
            _ => "as if it had only just thought of it",
        };
        super::combat::narrate_room(
            world,
            teller,
            &format!("{teller_name} the {teller_kind} tells {whom} a story, {how}: {story}."),
        );
        check_tier(world, listener).map_err(|e| e.to_string())?;
        told += 1;
    }
    Ok(told)
}

pub fn register(world: &mut World) {
    world.on_tick(Box::new(sweep));
    world.on_job(
        JOB_WANDER,
        Box::new(|w: &mut World, _| {
            wander(w)?;
            schedule_wander(w)
        }),
    );
    world.on_job(
        JOB_CONFIDE,
        Box::new(|w: &mut World, _| {
            confide(w)?;
            schedule_confide(w)
        }),
    );
}

/// Make sure the wandering and the confiding are scheduled; called at boot after
/// the jobs are replayed.
pub fn ensure_wander(world: &mut World) -> Result<(), String> {
    schedule_wander(world)?;
    schedule_confide(world)
}

/// "a wisp", "an engrosser": the article a kind takes.
pub fn an(kind: &str) -> String {
    let vowel = kind
        .chars()
        .next()
        .is_some_and(|c| "aeiou".contains(c.to_ascii_lowercase()));
    format!("{} {kind}", if vowel { "an" } else { "a" })
}

/// What an unnamed creature goes on doing when spoken to: it has no name to listen
/// with, and each kind has its own way of not listening. (The ragpicker's "goes on
/// sorting" was every kind's line once, clerks and tipstaff included.)
pub fn goes_on(kind: &str) -> &'static str {
    match kind {
        "wisp" => "goes on drifting",
        "hollow" => "stands where it stands",
        "revenant" => "goes on waiting",
        "inkling" => "goes on fluttering",
        "mothwing" => "works the branches",
        "ragpicker" => "goes on sorting",
        "drowned" => "sinks a little",
        "sitter" => "faces the wall",
        "hourhand" => "goes on turning",
        "keeper" => "goes on keeping",
        "lamprey" => "stays fastened to the shelf",
        "usher" => "waits at your elbow",
        "censor" => "goes on stamping",
        "cataloguer" => "turns a page",
        "engrosser" => "writes down that you spoke, and nothing of what you said",
        "tipstaff" => "keeps the rail",
        _ => "goes on with what it was doing",
    }
}
