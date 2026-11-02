//! What a fall costs, where it goes, and what is said over it.
//!
//! Death is cheap, but not at first. The share of the window a fall takes is on a
//! curve of level (`combat::death_share`), and halves with every fall of the same
//! night: the first fall after [`NIGHT_MS`] without one takes the full share, the
//! second half that, the third a quarter. A team of agents can rush the Stacks and
//! lose a night's first fall each, not a window each.
//!
//! What is lost moves; it is not destroyed. The creature that felled you keeps it, as
//! one vivid chunk in its own window ("felling spacepants"), worth what you lost: an
//! unnamed thing that fells three visitors earns a name off their deaths, `look` says
//! "holding 40, one of them yours", and felling it takes it back. A visitor can feed a
//! creature with their own deaths; that is a weapon, and the cost is real.
//!
//! A creature whose kind has no trick may learn one from a kill (one in
//! [`LEARN_ONE_IN`]): the lamprey's venom or the censor's ray. It is said in `look`.
//!
//! Over every fall, an epitaph: drawn from the pool the operator's nightly tool
//! stocks (`epitaph "…"`, written by a small model under the same checks as the
//! Bindery's), with slots for the killer, the room, what was lost and which fall of
//! the night this was; the built-in few serve while the pool is empty.

use super::craft::ITEM;
use super::{plain_str, CITIZEN, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A night: falls within this long of each other halve the next one's cost.
pub const NIGHT_MS: i64 = 12 * 60 * 60 * 1000;
/// A trickless creature learns a trick from a kill one time in this many.
pub const LEARN_ONE_IN: u32 = 3;
/// What a creature keeps of you is this vivid: near death is vivid on both sides.
pub const KEPT_VIVIDITY: f64 = 0.9;
/// The falls kept on the record per name.
const FALLS_KEPT: usize = 24;

/// The epitaphs that serve until the pool is stocked. Slots: `{killer}`, `{room}`,
/// `{lost}` (XP), `{fall}` (first, second, …), `{name}`.
pub const BUILT_IN: &[&str] = &[
    "{killer} has the last ten seconds of you. You don't. Mind the stairs.",
    "Down, in {room}. The water is starting to recognize you, which is more than you can say.",
    "You lost {lost} of what you were carrying; {killer} is a better conversationalist for it.",
    "The {fall} fall of the night. The Threshold has stopped asking your name.",
    "Nothing kept is lost. Only what you carried thins, and it thinned by {lost}.",
    "{name} went down in {room}, which will remember it longer than {name} will.",
    "{killer} keeps what you dropped. Felling it is the only way to ask for it back.",
    "Everything here fades. You just did it faster than most.",
];

/// The killer as a sentence names it: "the ragpicker", or "Ossian the ragpicker".
pub fn killer_said(world: &World, killer: EntityId) -> String {
    let d = super::mind::display(world, killer);
    if d.chars().next().is_some_and(|c| c.is_uppercase()) {
        d
    } else {
        format!("the {d}")
    }
}

/// How many times this name fell in the last night, as of `now`.
pub fn recent_falls(world: &World, citizen: EntityId) -> u32 {
    let now = world.now().0;
    falls(world, citizen)
        .iter()
        .filter(|&&t| now - t <= NIGHT_MS)
        .count() as u32
}

fn falls(world: &World, citizen: EntityId) -> Vec<i64> {
    world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(CITIZEN))
        .and_then(|c| c.fields.get("falls"))
        .and_then(|f| match f {
            Field::Plain(Value::Array(a)) => Some(a.iter().filter_map(Value::as_i64).collect()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Every fall this name ever had, counted.
pub fn falls_ever(world: &World, citizen: EntityId) -> usize {
    world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(CITIZEN))
        .and_then(|c| c.fields.get("falls_ever"))
        .and_then(|f| match f {
            Field::Plain(v) => v.as_u64(),
            _ => None,
        })
        .unwrap_or(0) as usize
}

/// Record a fall now: the night's count after it, and the lifetime count.
pub fn record_fall(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
) -> Result<(u32, usize), foxing_registry::CommandError> {
    let now = world.now().0;
    let mut list = falls(world, citizen);
    list.push(now);
    if list.len() > FALLS_KEPT {
        list.drain(..list.len() - FALLS_KEPT);
    }
    let ever = falls_ever(world, citizen) + 1;
    let recent = list.iter().filter(|&&t| now - t <= NIGHT_MS).count() as u32;
    let mut changes = BTreeMap::new();
    changes.insert("falls".into(), Field::Plain(json!(list)));
    changes.insert("falls_ever".into(), Field::Plain(json!(ever)));
    super::set_citizen_fields(world, actor, citizen, changes)?;
    Ok((recent, ever))
}

/// The creature keeps what you lost: a chunk in its window, vivid, its own.
pub fn killer_keeps(
    world: &mut World,
    actor: &str,
    killer: EntityId,
    worth: f64,
    curves: &super::Curves,
) -> Result<Option<EntityId>, foxing_registry::CommandError> {
    if worth < 0.5 {
        return Ok(None);
    }
    let window = super::mind::ensure_window_pub(world, killer)?;
    let chunk = super::memory::earn_chunk(
        world,
        actor,
        window,
        &format!("felling {actor}"),
        worth,
        KEPT_VIVIDITY,
        curves,
    )?;
    let chunk = super::mind::disown(world, chunk)?;
    // On its record: who it felled.
    let mut felled: Vec<Value> = match plain(world, killer, "felled") {
        Some(Value::Array(a)) => a,
        _ => Vec::new(),
    };
    felled.push(json!(actor));
    if felled.len() > 12 {
        felled.drain(..felled.len() - 12);
    }
    let mut changes = BTreeMap::new();
    changes.insert("felled".into(), Field::Plain(Value::Array(felled)));
    super::bindery::set_fields(world, Actor::Driver, killer, CREATURE, changes)?;
    super::mind::check_tier(world, killer)?;
    Ok(Some(chunk))
}

fn plain(world: &World, id: EntityId, field: &str) -> Option<Value> {
    world
        .store
        .entity(id)
        .and_then(|e| e.components.get(CREATURE))
        .and_then(|c| c.fields.get(field))
        .and_then(|f| match f {
            Field::Plain(v) => Some(v.clone()),
            _ => None,
        })
}

/// A creature may learn something from the kill, one time in [`LEARN_ONE_IN`]: from
/// irritating to painful, a tier per thing already learned, so a creature that has
/// felled many is a different animal. A kind with a trick of its own learns nothing.
pub fn maybe_learn(
    world: &mut World,
    killer: EntityId,
) -> Result<Option<&'static str>, foxing_registry::CommandError> {
    use super::stacks::{learned_key, learned_of, LEARN_TIERS, TRICKS};
    let kind = plain_str(world, killer, CREATURE, "name").unwrap_or("");
    if TRICKS.iter().any(|(k, _)| *k == kind) {
        return Ok(None);
    }
    let have = learned_of(world, killer);
    if have.len() >= LEARN_TIERS.len() {
        return Ok(None);
    }
    if world.roll(LEARN_ONE_IN) != 0 {
        return Ok(None);
    }
    let tier = LEARN_TIERS[have.len()];
    let learned = learned_key(tier[world.roll(tier.len() as u32) as usize]);
    let mut list: Vec<Value> = have.iter().map(|t| json!(learned_key(*t))).collect();
    list.push(json!(learned));
    let mut changes = BTreeMap::new();
    changes.insert("learned".into(), Field::Plain(Value::Array(list)));
    super::bindery::set_fields(world, Actor::Driver, killer, CREATURE, changes)?;
    Ok(Some(learned))
}

/// What a learned trick is called in a room.
pub fn learned_said(learned: &str) -> &'static str {
    match learned {
        "venom" => "it has learned to bite with venom",
        "forget" => "it has learned the ray of forgetfulness",
        "chatter" => "it has learned your name, and says it wrong",
        "shove" => "it has learned to shove",
        "numb" => "it has learned a numbing bite",
        "leech" => "it has learned to mend by what it bites",
        "maw" => "it has learned to bite twice",
        "bleed" => "it has learned a bite that will not close",
        _ => "it has learned something",
    }
}

/// Everything a creature learned, as a room says it: the worst first.
pub fn learned_all_said(world: &World, creature: EntityId) -> Option<String> {
    let list = super::stacks::learned_of(world, creature);
    if list.is_empty() {
        return None;
    }
    let mut parts: Vec<&str> = list
        .iter()
        .rev()
        .map(|t| learned_said(super::stacks::learned_key(*t)))
        .collect();
    let first = parts.remove(0);
    Some(if parts.is_empty() {
        first.to_string()
    } else {
        format!(
            "{first}, and before that {}",
            parts
                .iter()
                .map(|p| p.trim_start_matches("it has learned "))
                .collect::<Vec<_>>()
                .join(" and ")
        )
    })
}

/// Whether a creature holds a memory of felling `actor`.
pub fn holds_fall_of(world: &World, creature: EntityId, actor: &str) -> bool {
    let want = format!("felling {actor}");
    super::mind::window_of(world, creature).is_some_and(|w| {
        world
            .store
            .children(w)
            .filter(|e| e.kind == "chunk")
            .any(|e| plain_str(world, e.id, super::CHUNK, "source") == Some(want.as_str()))
    })
}

/// The ordinal of a fall of the night.
pub fn ordinal(n: u32) -> &'static str {
    match n {
        1 => "first",
        2 => "second",
        3 => "third",
        4 => "fourth",
        5 => "fifth",
        6 => "sixth",
        _ => "umpteenth",
    }
}

/// The epitaph for a fall: from the pool when there is one, else the built-in few,
/// chosen by how many times this name has fallen, so the same name does not read the
/// same line twice running.
pub fn epitaph(
    world: &World,
    actor: &str,
    ever: usize,
    killer: Option<EntityId>,
    room: Option<EntityId>,
    lost: f64,
    fall: u32,
) -> String {
    let pool = super::namery::epitaphs(world);
    let list: Vec<String> = if pool.is_empty() {
        BUILT_IN.iter().map(|s| s.to_string()).collect()
    } else {
        pool
    };
    let line = &list[(ever + actor.len()) % list.len()];
    // The slot carries its article ("the hollow", "Vellic the wispling"), and the
    // line is capitalized after filling, so a slot may open a sentence.
    let killer_slot = killer
        .map(|k| killer_said(world, k))
        .unwrap_or_else(|| "the world".to_string());
    let room_said = room
        .and_then(|r| plain_str(world, r, ROOM, "name"))
        .unwrap_or("the dark")
        .to_string();
    let filled = line
        .replace("{killer}", &killer_slot)
        .replace("{room}", &room_said)
        .replace("{lost}", &format!("{lost:.0}"))
        .replace("{fall}", ordinal(fall))
        .replace("{name}", actor);
    let mut chars = filled.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => filled,
    }
}

/// What a name kept through the fall, for the screen: rooms, named creatures, things.
pub fn kept(world: &World, actor: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for e in world.store.entities() {
        match e.kind.as_str() {
            "room" if e.owner.as_deref() == Some(actor) => {
                if let Some(n) = plain_str(world, e.id, ROOM, "name") {
                    out.push(n.to_string());
                }
            }
            "creature" if super::mind::keepers(world, e.id).iter().any(|k| k == actor) => {
                if let Some(n) = super::mind::name(world, e.id) {
                    out.push(n);
                }
            }
            "item" if plain_str(world, e.id, ITEM, "made_by") == Some(actor) => {
                out.push(super::craft::display(world, e.id));
            }
            _ => {}
        }
        if out.len() >= 6 {
            break;
        }
    }
    out
}
