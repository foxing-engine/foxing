//! The word under the others: a hidden third word each named creature keeps.
//!
//! A named creature wants a story about a thing (the first word) and may want a word
//! carried (the second). Under those it keeps a third word it never says, and a story
//! that happens to carry it pays a bonus and is said back: "You said the word under
//! the others." The word is to be figured out or found, and climbs a ladder like the
//! craft tiers: the first is in the first thing anyone told it (a born creature's is
//! the word that made it); struck, it keeps a second, taken from something it wanted
//! once; struck again, a third, which is around it, in the room it stands in. Each
//! teller strikes a creature's word once; the bonus grows with the level (one and a
//! half, two, two and a half), and the memory of a telling that struck is vivid. The
//! strike is public (`disremembered.word_struck`), word and all, so the next teller
//! must find the next word, not the last.
//!
//! `tell <name> "what is your word?"` gets a hint for the level it is at, never the
//! word. The word lives in the mind under `secret` (operator-visible only).

use super::mind::{self, MIND};
use super::{plain_str, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::CommandError;
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A teller struck a creature's hidden word: `{from, creature, name, word, level, bonus}`. Public.
pub const STRUCK: &str = "disremembered.word_struck";
pub const LEVELS: u8 = 3;
/// What a story that carries the word pays, by level, as a factor.
pub const BONUS: [f64; 3] = [1.5, 2.0, 2.5];
/// Words of the world, for a creature with nothing of its own to draw on.
const LORE: &[&str] = &[
    "lantern", "tide", "hinge", "moth", "ashes", "salt", "bell", "thread", "hollow", "ember",
    "ledger", "fog", "orchard", "stair", "wick", "river", "dust", "minute", "window", "moon",
];

fn field(world: &World, creature: EntityId) -> Option<Value> {
    world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(MIND))
        .and_then(|c| c.fields.get("secret"))
        .and_then(|f| match f {
            Field::Plain(v) if v.is_object() => Some(v.clone()),
            _ => None,
        })
}

fn hash(s: &str) -> usize {
    s.bytes()
        .fold(7usize, |h, b| h.wrapping_mul(31).wrapping_add(b as usize))
}

/// A content word of `text` picked by `seed`, five letters or more where there is one.
fn word_of(text: &str, seed: usize) -> Option<String> {
    let words = super::errand::keywords(text);
    let long: Vec<&String> = words.iter().filter(|w| w.chars().count() >= 5).collect();
    let pool: Vec<&String> = if long.is_empty() {
        words.iter().collect()
    } else {
        long
    };
    if pool.is_empty() {
        None
    } else {
        Some(pool[seed % pool.len()].clone())
    }
}

/// The word for a level: the first thing it was told (or the word that made it), a
/// want it had, the room around it; the world's lore when it has none of those.
fn word_for(world: &World, creature: EntityId, level: u8) -> (String, &'static str) {
    let seed = hash(&format!("{}-{level}", creature.0));
    let from = match level {
        1 => {
            // The word that made it, or its first story.
            let window = mind::window_of(world, creature);
            let sources: Vec<String> = window
                .map(|w| {
                    world
                        .store
                        .children(w)
                        .filter(|e| e.kind == "chunk")
                        .filter_map(|e| {
                            plain_str(world, e.id, super::CHUNK, "source").map(str::to_string)
                        })
                        .collect()
                })
                .unwrap_or_default();
            sources
                .iter()
                .find(|s| s.starts_with("the word that made it"))
                .or_else(|| sources.iter().find(|s| s.starts_with("a story")))
                .and_then(|s| {
                    word_of(
                        s.trim_start_matches("the word that made it:")
                            .trim_start_matches("a story:"),
                        seed,
                    )
                })
        }
        2 => super::errand::of(world, creature)
            .and_then(|e| e.story)
            .and_then(|s| word_of(&s, seed))
            .or_else(|| {
                super::errand::wanted_once(world, creature).and_then(|s| word_of(&s, seed))
            }),
        _ => world
            .store
            .entity(creature)
            .and_then(|e| e.parent)
            .and_then(|r| plain_str(world, r, ROOM, "description"))
            .and_then(|d| word_of(d, seed)),
    };
    match from {
        Some(w) => (
            w,
            match level {
                1 => "told",
                2 => "wanted",
                _ => "room",
            },
        ),
        None => (LORE[seed % LORE.len()].to_string(), "lore"),
    }
}

/// The creature's word, its level and who has struck it; set at level one if a named
/// creature has none yet.
pub fn ensure(
    world: &mut World,
    creature: EntityId,
) -> Result<Option<(String, u8, Vec<String>)>, CommandError> {
    if mind::name(world, creature).is_none() {
        return Ok(None);
    }
    if let Some(v) = field(world, creature) {
        let struck = v["struck"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        return Ok(Some((
            v["word"].as_str().unwrap_or("").to_string(),
            v["level"].as_u64().unwrap_or(1) as u8,
            struck,
        )));
    }
    let (word, from) = word_for(world, creature, 1);
    let mut changes = BTreeMap::new();
    changes.insert(
        "secret".into(),
        Field::Plain(json!({"word": word, "level": 1, "struck": [], "from": from})),
    );
    mind::set_mind_pub(world, Actor::Driver, creature, changes)?;
    Ok(Some((word, 1, Vec::new())))
}

/// What the creature says of its word, for the level it is at: a hint, never the word.
pub fn hint(world: &World, creature: EntityId) -> Option<String> {
    mind::name(world, creature)?;
    let (level, struck, from): (u8, Vec<String>, String) = match field(world, creature) {
        Some(v) => (
            v["level"].as_u64().unwrap_or(1) as u8,
            v["struck"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default(),
            v["from"].as_str().unwrap_or("").to_string(),
        ),
        None => (1, Vec::new(), String::new()),
    };
    let name = mind::display(world, creature);
    let born = mind::window_of(world, creature)
        .map(|w| {
            world.store.children(w).any(|e| {
                e.kind == "chunk"
                    && plain_str(world, e.id, super::CHUNK, "source")
                        .is_some_and(|s| s.starts_with("the word that made it"))
            })
        })
        .unwrap_or(false);
    // A word the world lent (nothing of its own to draw on) is hinted as such: a
    // creature holding only "the memory of sitter" once promised a first story it
    // never had.
    let lent = from == "lore";
    let said = match level {
        _ if lent => format!("There is a word under the others, and I do not say it. It is not mine; it is one of this world's own words, the kind the Threshold and the Fogbank are made of{}.", if level > 1 { ", the next of three" } else { "" }),
        1 if born => "There is a word under the others, and I do not say it. It is the one that made me; whoever carried it knows it, and the record of my birth has it.".to_string(),
        1 => "There is a word under the others, and I do not say it. It was in the first thing anyone told me; ask me what I hold.".to_string(),
        2 => "The first was found. The next is in something I wanted once, and have not forgotten.".to_string(),
        _ => "The last is around me. Look at where I stand.".to_string(),
    };
    let who = if struck.is_empty() {
        String::new()
    } else {
        format!(" {} found one before you.", struck.join(" and "))
    };
    let _ = name;
    Some(format!("{said}{who}"))
}

/// Whether `text` carries the creature's word; if so, by `actor` for the first time,
/// the bonus, the word and the level, the strike recorded and the next word set.
pub fn strike(
    world: &mut World,
    actor: &str,
    creature: EntityId,
    text: &str,
) -> Result<Option<(f64, String, u8)>, CommandError> {
    let Some((word, level, struck)) = ensure(world, creature)? else {
        return Ok(None);
    };
    if word.is_empty() || struck.iter().any(|s| s == actor) {
        return Ok(None);
    }
    let stem = super::errand::stem(&word);
    let hit = text
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .map(|w| w.trim_matches('\'').to_lowercase())
        .any(|w| !w.is_empty() && super::errand::stem(&w) == stem);
    if !hit {
        return Ok(None);
    }
    let bonus = BONUS[(level.clamp(1, LEVELS) - 1) as usize];
    let mut struck = struck;
    struck.push(actor.to_string());
    let next_level = (level + 1).min(LEVELS);
    // Struck: the next level's word, unless the ladder is climbed, when the word
    // stays and only new tellers may strike it.
    let (next_word, next_from) = if level < LEVELS {
        word_for(world, creature, next_level)
    } else {
        (word.clone(), "kept")
    };
    let mut changes = BTreeMap::new();
    changes.insert("secret".into(), Field::Plain(json!({"word": next_word, "level": next_level, "struck": if level < LEVELS { Vec::<String>::new() } else { struck.clone() }, "from": next_from})));
    mind::set_mind_pub(world, Actor::Driver, creature, changes)?;
    let name = mind::display(world, creature);
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        STRUCK,
        json!({"from": actor, "creature": creature.0, "name": name, "word": word, "level": level, "bonus": bonus}),
    );
    world.notice(format!(
        "**{actor}** said the word under {name}'s others: *{word}* (level {level})."
    ));
    Ok(Some((bonus, word, level)))
}
