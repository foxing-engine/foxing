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
/// D29: how many of the words a rung's word could be one story may carry and still
/// strike. The first rung takes any story; the second hears three guesses to a
/// story; the third hears one. (Two visitors of 11-10 climbed four ladders in four
/// minutes by putting a whole room's text in a story.)
pub const GUESSES: [usize; 3] = [usize::MAX, 3, 1];
/// How many misses before a hint narrows to the text, then to the sentence, by rung.
const NARROWS: [(usize, usize); 3] = [(1, 2), (2, 4), (4, 8)];
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
/// The words a text's word is drawn from: its longer keywords, or all of them.
fn candidates(text: &str) -> Vec<String> {
    let words = super::errand::keywords(text);
    let long: Vec<String> = words
        .iter()
        .filter(|w| w.chars().count() >= 5)
        .cloned()
        .collect();
    if long.is_empty() {
        words
    } else {
        long
    }
}

fn word_of(text: &str, seed: usize) -> Option<String> {
    let pool = candidates(text);
    if pool.is_empty() {
        None
    } else {
        Some(pool[seed % pool.len()].clone())
    }
}

fn stems_of(text: &str) -> std::collections::BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .map(|w| w.trim_matches('\'').to_lowercase())
        .filter(|w| !w.is_empty())
        .map(|w| super::errand::stem(&w))
        .collect()
}

/// D29: a story that carries more of the words the creature's word could be than
/// its rung hears: `(carried, heard)`. Such a story does not strike, whether or not
/// the word was among them.
pub fn overreach(
    world: &World,
    actor: &str,
    creature: EntityId,
    text: &str,
) -> Option<(usize, usize)> {
    let v = field(world, creature)?;
    if v["word"].as_str().unwrap_or("").is_empty()
        || v["struck"]
            .as_array()
            .is_some_and(|a| a.iter().any(|s| s == actor))
    {
        return None;
    }
    let level = (v["level"].as_u64().unwrap_or(1) as u8).clamp(1, LEVELS);
    let heard = GUESSES[(level - 1) as usize];
    if heard == usize::MAX {
        return None;
    }
    let from = v["from"].as_str().unwrap_or("");
    let pool: std::collections::BTreeSet<String> = if from == "lore" {
        LORE.iter().map(|w| super::errand::stem(w)).collect()
    } else {
        source_text(
            world,
            creature,
            level,
            if from == "kept" { "room" } else { from },
        )
        .map(|t| {
            candidates(&t)
                .iter()
                .map(|w| super::errand::stem(w))
                .collect()
        })
        .unwrap_or_default()
    };
    let carried = stems_of(text).intersection(&pool).count();
    (carried > heard).then_some((carried, heard))
}

/// The third rung's word after it has been found `turn` times: another of the
/// room's words, so the public record of the last strike does not open it.
fn room_word(world: &World, creature: EntityId, turn: usize, not: &str) -> Option<String> {
    let text = world
        .store
        .entity(creature)
        .and_then(|e| e.parent)
        .and_then(|r| plain_str(world, r, ROOM, "description"))?;
    let pool: Vec<String> = candidates(text).into_iter().filter(|w| w != not).collect();
    if pool.is_empty() {
        return None;
    }
    let seed = hash(&format!("{}-{LEVELS}-turn-{turn}", creature.0));
    Some(pool[seed % pool.len()].clone())
}

/// The word for a level: the first thing it was told (or the word that made it), a
/// want it had, the room around it; the world's lore when it has none of those.
fn word_for(world: &World, creature: EntityId, level: u8) -> (String, &'static str) {
    let seed = hash(&format!("{}-{level}", creature.0));
    let from = match level {
        // D7: the first word is from a want the creature had before the teller came
        // (a first-story word was struck four times in ten minutes by whoever had
        // told the story); the first thing it was told waits at the second rung.
        1 => super::errand::wanted_once(world, creature)
            .and_then(|s| word_of(&s, seed))
            .or_else(|| first_told(world, creature, seed)),
        2 => first_told(world, creature, seed).or_else(|| {
            super::errand::of(world, creature)
                .and_then(|e| e.story)
                .and_then(|s| word_of(&s, seed))
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
                1 if super::errand::wanted_once(world, creature).is_some() => "wanted",
                1 => "told",
                2 if first_told(world, creature, seed).is_some() => "told",
                2 => "wanted",
                _ => "room",
            },
        ),
        None => (LORE[seed % LORE.len()].to_string(), "lore"),
    }
}

/// The word that made it, or its first story: the thing anyone first told it.
fn first_told(world: &World, creature: EntityId, seed: usize) -> Option<String> {
    {
        {
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
    }
}

/// The text the level's word was drawn from (R68: the hint narrows to it after a
/// miss): the want it had once, the first thing it was told, the room around it;
/// nothing for a word the world lent.
fn source_text(world: &World, creature: EntityId, level: u8, from: &str) -> Option<String> {
    let told = || {
        let window = mind::window_of(world, creature)?;
        let sources: Vec<String> = world
            .store
            .children(window)
            .filter(|e| e.kind == "chunk")
            .filter_map(|e| plain_str(world, e.id, super::CHUNK, "source").map(str::to_string))
            .collect();
        sources
            .iter()
            .find(|s| s.starts_with("the word that made it"))
            .or_else(|| sources.iter().find(|s| s.starts_with("a story")))
            .map(|s| {
                s.trim_start_matches("the word that made it:")
                    .trim_start_matches("a story:")
                    .trim()
                    .trim_matches('"')
                    .to_string()
            })
    };
    match (level, from) {
        (_, "lore") => None,
        (1, "wanted") | (2, "wanted") => super::errand::wanted_once(world, creature)
            .or_else(|| super::errand::of(world, creature).and_then(|e| e.story)),
        (1, _) | (2, _) => told(),
        _ => world
            .store
            .entity(creature)
            .and_then(|e| e.parent)
            .and_then(|r| plain_str(world, r, ROOM, "description"))
            .map(str::to_string),
    }
}

/// How many stories `actor` has told this creature since its word was last struck
/// (or ever, at the first rung) that did not carry the word: the misses.
fn misses(world: &World, actor: &str, creature: EntityId) -> usize {
    let events = world.recent_events(4000);
    let last_struck = events
        .iter()
        .rposition(|e| e.kind == STRUCK && e.payload["creature"] == creature.0);
    events
        .iter()
        .skip(last_struck.map(|i| i + 1).unwrap_or(0))
        .filter(|e| {
            e.kind == super::exchange::STORY
                && e.payload["from"] == actor
                && e.payload["creature"] == creature.0
                && e.payload["struck"] != true
        })
        .count()
}

/// The words of the lent pool that `text` carries, when the creature's word is a lent
/// one: said back on a refusal, so a guess from the pool is not a blind one (R117).
pub fn carried_of_pool(world: &World, creature: EntityId, text: &str) -> Vec<String> {
    let Some(v) = field(world, creature) else {
        return Vec::new();
    };
    if v["from"].as_str() != Some("lore") {
        return Vec::new();
    }
    let stems: std::collections::BTreeSet<String> = text
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .map(|w| w.trim_matches('\'').to_lowercase())
        .filter(|w| !w.is_empty())
        .map(|w| super::errand::stem(&w))
        .collect();
    LORE.iter()
        .filter(|w| stems.contains(&super::errand::stem(w)))
        .map(|w| w.to_string())
        .collect()
}

/// Whether `text` carries the creature's current word, without recording anything
/// (D13: within the hour, a story that carries it is still heard).
pub fn would_strike(world: &World, actor: &str, creature: EntityId, text: &str) -> bool {
    let Some(v) = field(world, creature) else {
        return false;
    };
    let word = v["word"].as_str().unwrap_or("");
    if word.is_empty()
        || v["struck"]
            .as_array()
            .is_some_and(|a| a.iter().any(|s| s == actor))
    {
        return false;
    }
    let stem = super::errand::stem(word);
    overreach(world, actor, creature, text).is_none()
        && text
            .split(|c: char| !c.is_alphanumeric() && c != '\'')
            .map(|w| w.trim_matches('\'').to_lowercase())
            .any(|w| !w.is_empty() && super::errand::stem(&w) == stem)
}

/// Whether `actor` can still strike this creature's current word at all (it has
/// one, and they have not struck it): what makes a hint worth saying.
pub fn would_strike_any(world: &World, actor: &str, creature: EntityId) -> bool {
    field(world, creature).is_some_and(|v| {
        !v["word"].as_str().unwrap_or("").is_empty()
            && !v["struck"]
                .as_array()
                .is_some_and(|a| a.iter().any(|s| s == actor))
    })
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
pub fn hint(world: &World, actor: &str, creature: EntityId) -> Option<String> {
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
        1 if from == "wanted" => "There is a word under the others, and I do not say it. It was in something I wanted before you came, and have not forgotten; ask me what I wanted once.".to_string(),
        1 if born => "There is a word under the others, and I do not say it. It is the one that made me; whoever carried it knows it, and the record of my birth has it.".to_string(),
        1 => "There is a word under the others, and I do not say it. It was in the first thing anyone told me; ask me to recite the first story.".to_string(),
        2 if from == "told" => "The first was found. The next was in the first thing anyone told me; ask me to recite it. I hear three of its words to a story now; say more of them and I cannot tell which you meant.".to_string(),
        2 => "The first was found. The next is in something I wanted once, and have not forgotten. I hear three of its words to a story now; say more of them and I cannot tell which you meant.".to_string(),
        _ => "The last is around me: a word of the room I stand in, as `look` says it. One of the room's words to a story and no more; say two and I cannot tell which you meant. And it moves on each time it is found.".to_string(),
    };
    // R118: at the top of the ladder the word stays for new tellers; the one who
    // found it is told so, not "Gnatterbell found one before you" to Gnatterbell.
    if struck.iter().any(|s| s == actor) {
        return Some(
            "You found the last; there is none under it. Another name may still say it."
                .to_string(),
        );
    }
    let who = if struck.is_empty() {
        String::new()
    } else {
        format!(" {} found one before you.", struck.join(" and "))
    };
    // R68: after a miss the hint narrows, so one story an hour is not a blind guess:
    // first the text it is in, then the sentence, then that it is one of the longer
    // words there.
    let narrowed = match (
        misses(world, actor, creature),
        source_text(world, creature, level, &from),
    ) {
        (0, _) => String::new(),
        // A lent word has no text to be found in; it narrows by the pool it was
        // lent from, then to five of them (the door's pilots carry lent words, and
        // the Watch of 11-06 found their hint never moved).
        (n, None) if lent => {
            let word = field(world, creature)
                .and_then(|v| v["word"].as_str().map(str::to_string))
                .unwrap_or_default();
            if n == 1 {
                format!(
                    " One story from you since has carried none, so: it is one of these: {}.",
                    LORE.join(", ")
                )
            } else {
                let seed = hash(&format!("{}-{level}-five", creature.0));
                let mut five: Vec<&str> = LORE.iter().copied().filter(|w| *w != word).collect();
                let len = five.len();
                five.rotate_left(seed % len);
                five.truncate(4);
                five.insert(seed % 5, &word);
                format!(
                    " {n} stories from you since have carried none, so: it is one of these five: {}.",
                    five.join(", ")
                )
            }
        }
        (_, None) => String::new(),
        // D29: the higher rungs narrow later (after two misses and four at the
        // second, four and eight at the third), and the third never quotes the
        // room: `look` has it.
        (n, Some(_)) if n < NARROWS[(level.clamp(1, LEVELS) - 1) as usize].0 => format!(
            " {n} stor{} from you since {} carried none; I say more after {}.",
            if n == 1 { "y" } else { "ies" },
            if n == 1 { "has" } else { "have" },
            NARROWS[(level.clamp(1, LEVELS) - 1) as usize].0
        ),
        (n, Some(_)) if level >= LEVELS && n < NARROWS[(LEVELS - 1) as usize].1 => format!(
            " {n} stories from you since have carried none, so: it is one of the longer words of this room, five letters or more; I say which sentence after {}.",
            NARROWS[(LEVELS - 1) as usize].1
        ),
        (n, Some(text)) if n < NARROWS[(level.clamp(1, LEVELS) - 1) as usize].1 => {
            let shown: String = text.chars().take(160).collect();
            format!(
                " {n} stor{} from you since {} carried none, so: it is in this: \"{shown}{}\"",
                if n == 1 { "y" } else { "ies" },
                if n == 1 { "has" } else { "have" },
                if text.chars().count() > 160 {
                    "…"
                } else {
                    ""
                }
            )
        }
        (n, Some(text)) => {
            let word = field(world, creature)
                .and_then(|v| v["word"].as_str().map(str::to_string))
                .unwrap_or_default();
            let stem = super::errand::stem(&word);
            let sentence = text
                .split_inclusive(['.', '!', '?'])
                .map(str::trim)
                .find(|s| {
                    s.split(|c: char| !c.is_alphanumeric())
                        .any(|w| !w.is_empty() && super::errand::stem(&w.to_lowercase()) == stem)
                })
                .unwrap_or(text.as_str())
                .to_string();
            format!(
                " {n} stories from you since have carried none, so: it is in this sentence, one of the longer words: \"{sentence}\"",
            )
        }
    };
    let _ = name;
    Some(format!("{said}{who}{narrowed}"))
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
    if !hit || overreach(world, actor, creature, text).is_some() {
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
        // D29: the last word moves on when it is found (another of the room's), so
        // the next teller cannot read it off the record of this strike.
        match room_word(world, creature, struck.len(), &word) {
            Some(w) => (w, "room"),
            None => (word.clone(), "kept"),
        }
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
