//! D33: the second and third words by meaning.
//!
//! With the switch on (`sense meanings`, the operator's), a named creature's second and
//! third rungs hold a meaning, not a word ("something lent and never returned"), and
//! a story finds it by being about that, in any words. The first rung stays a word.
//!
//! Nothing here asks a model. A story told to a creature at a meaning rung is paid
//! as a story at once, and put on the Docent's record to be weighed; the tool outside
//! (`tools/docent.py`, the same that answers questions and sits on the Court's bench)
//! reads `GET /v0/docent`, asks a small model one question under a fixed rubric (is
//! this story about this meaning: 0 to 3, and one line in the creature's voice), and
//! says the result with the operator's verb `weigh`. The verdict reaches the teller
//! the way the Docent's answers do, a little later, wherever they stand. A 3 finds
//! the meaning: the bonus is paid then, the strike is public, and the creature keeps
//! the next one.
//!
//! What a late judge costs, and how each is met:
//! - It cannot be predicted, so no `check` and no reading it off the record: a
//!   strike names the meaning found, and the creature has moved on to another.
//! - A story will address the judge. The story goes in as data; only a number and a
//!   short line come out, and the world checks both. A line that says the meaning
//!   short of a 3 is thrown away for a plain one.
//! - Two like stories may be weighed apart, so the score is coarse and the creature
//!   says what it made of it.
//! - When the tool is not listening the rung sleeps: the story is told so, is paid as
//!   a story, and nothing guessable stands in. A verdict that never comes is said to
//!   have never come.
//! - One story an hour from a name is weighed by a creature (it hears one an hour).

use super::mind::{self, MIND};
use super::Curves;
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A story was weighed against a meaning: `{id, from, creature, name, level, score,
/// model}`; with `meaning` when it was found. Public.
pub const WEIGHED: &str = "disremembered.meaning_weighed";
const JOB_SLEPT: &str = "disremembered.meaning_slept";
/// A verdict not here this long after the story is said to have never come.
pub const SLEPT_AFTER_S: i64 = 240;
/// The tool is listening if it read the record this lately.
const AWAKE_MS: i64 = 90_000;
const KEPT: usize = 60;
/// The creature's line, at most.
pub const LINE_MAX: usize = 200;
/// What goes to the judge of a story, at most.
const STORY_MAX: usize = 1500;

/// A meaning a creature may keep: its key (what the mind stores), the meaning as it
/// is said when found, and three hints, each plainer than the last.
pub struct Theme {
    pub key: &'static str,
    pub meaning: &'static str,
    pub hints: [&'static str; 3],
}

macro_rules! themes {
    ($( $k:literal, $m:literal, $a:literal, $b:literal, $c:literal; )*) => {
        &[ $( Theme { key: $k, meaning: $m, hints: [$a, $b, $c] }, )* ]
    };
}

pub const THEMES: &[Theme] = themes![
    "lent", "something lent and never returned", "It went from one hand to another and was meant to come back.", "Someone is still owed it.", "A loan, and the lender still waiting.";
    "late", "arriving too late for the thing one came for", "Somebody hurried, and it did not help.", "The door was shut, or the bed empty, by the time.", "Coming too late.";
    "kept-promise", "a promise kept long after anyone would have minded it broken", "Nobody was holding them to it any more.", "It was done anyway, years on, for someone who had stopped expecting it.", "A promise kept when no one was left to ask.";
    "wrong-name", "being called by a name that is not one's own, and answering", "Somebody was mistaken about who, and was not corrected.", "It was easier to turn round than to explain.", "Answering to the wrong name.";
    "last-of", "being the last one who remembers how a thing was done", "When this one goes, the knack goes.", "There is nobody left to ask whether it is being done right.", "The last one who knows how.";
    "unsent", "a letter written and never sent", "It was meant for someone and stayed in a drawer.", "Every word of it was finished; only the going was not.", "A letter never sent.";
    "trade-down", "giving up something good for something worse, and being glad of it", "The bargain was a bad one and nobody regretted it.", "What was given was worth more than what was got, and that was the point.", "A bad trade, gladly made.";
    "borrowed-light", "finding the way by a light that belongs to someone else", "They could see, and it was not their doing.", "Somebody else's window, or lamp, or fire, and a road by it.", "Walking by another's light.";
    "small-theft", "a small theft that nobody ever noticed", "Something went missing and was never missed.", "The one who took it is the only one who knows.", "A theft too small to notice.";
    "wait", "waiting for someone who is not coming", "A place was kept, and stayed kept.", "The one who waits half knows, and waits.", "Waiting for one who will not come.";
    "mended", "a thing mended so often that nothing of the first one is left", "It is the same one, and none of it is.", "Each part was replaced in turn, and nobody ever called it new.", "Mended until nothing original remains.";
    "inherit", "inheriting a thing one does not know how to use", "It came down to them, and they could only keep it.", "The one who knew what it was for is not there to say.", "An inheritance nobody can work.";
    "first-lie", "a kind lie told to spare someone, and believed", "It was not true, and it was the gentlest thing said that day.", "They never found out, and were happier.", "A kind lie, believed.";
    "return", "coming home and finding it smaller than it was", "The place had not changed, and it did not fit.", "Everything was where it had been left, and lower, and nearer together.", "Home, and it has shrunk.";
    "overheard", "learning something important by overhearing what was not meant for one", "It was said to somebody else.", "A door was not quite shut, and it changed things.", "Overhearing what was not meant for you.";
    "shared-secret", "a secret that two people keep by never speaking of it", "Both of them know, and neither has said.", "It holds only as long as nobody mentions it.", "A secret kept by silence between two.";
    "wrong-turn", "a wrong turning that turned out better than the right one", "They were lost, and it was the best thing that happened.", "The road they meant to take would have been worse.", "A mistake that was a mercy.";
    "counting", "counting something over and over because one is afraid of the number", "The sum does not change, and is taken again.", "As long as it is being counted it has not been lost.", "Counting, for fear of the count.";
    "gift-refused", "a gift that was refused", "It was held out, and not taken.", "The giver carried it home again.", "A gift refused.";
    "namesake", "being named after someone one never met", "The name was somebody's before, and they are only a story.", "People look for the other one in the face.", "Named for a stranger.";
    "watch", "keeping watch over something that no longer needs it", "The danger passed long ago, and the post was never stood down.", "It is guarded out of habit, or love.", "Guarding what no longer needs a guard.";
    "forgiven", "being forgiven by someone who never knew what for", "The pardon was real, and the offense was never told.", "They said it was nothing, and did not know how much it was.", "Forgiven, unconfessed.";
    "unfinished", "work left unfinished on purpose", "It could have been done, and was set down one step short.", "Finishing it would have meant it was over.", "Left undone, deliberately.";
    "echo", "hearing one's own words come back from someone else, changed", "They said it once, and it returned a stranger.", "Somebody repeated it, and it was not what was meant any more.", "One's own words, come back altered.";
];

fn theme(key: &str) -> Option<&'static Theme> {
    THEMES.iter().find(|t| t.key == key)
}

fn hash(s: &str) -> usize {
    s.bytes()
        .fold(5381usize, |h, b| h.wrapping_mul(33) ^ b as usize)
}

/// The meaning for a creature at a rung, on its `turn`th finding (the top rung moves
/// on each time it is found), and not `not`.
pub fn pick(creature: EntityId, level: u8, turn: usize, not: &str) -> &'static Theme {
    let start = hash(&format!("meaning-{}-{level}-{turn}", creature.0));
    (0..THEMES.len())
        .map(|i| &THEMES[(start + i) % THEMES.len()])
        .find(|t| t.key != not)
        .unwrap_or(&THEMES[0])
}

/// Whether the switch is on.
pub fn on(world: &World) -> bool {
    super::docent::field(world, "sense").is_some_and(|v| v == true)
}

fn secret(world: &World, creature: EntityId) -> Option<Value> {
    match world
        .store
        .entity(creature)?
        .components
        .get(MIND)?
        .fields
        .get("secret")?
    {
        Field::Plain(v) if v.is_object() => Some(v.clone()),
        _ => None,
    }
}

/// The meaning `creature` keeps now, if its rung is one.
pub fn kept(world: &World, creature: EntityId) -> Option<&'static Theme> {
    let v = secret(world, creature)?;
    (v["from"] == "meaning")
        .then(|| theme(v["word"].as_str().unwrap_or("")))
        .flatten()
}

// ---- whether the tool is listening ----
#[cfg(not(test))]
static LISTENED: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
#[cfg(test)]
thread_local! { static LISTENED: std::cell::Cell<i64> = const { std::cell::Cell::new(0) }; }

/// The tool read the record just now (`GET /v0/docent`).
pub fn listened(world: &World) {
    let now = world.now().0;
    #[cfg(not(test))]
    LISTENED.store(now, std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    LISTENED.with(|l| l.set(now));
}

fn awake(world: &World) -> bool {
    #[cfg(not(test))]
    let at = LISTENED.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    let at = LISTENED.with(|l| l.get());
    at > 0 && world.now().0 - at < AWAKE_MS
}

fn weighing(world: &World) -> Vec<Value> {
    super::docent::field(world, "weighing")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

fn nth(level: u8) -> &'static str {
    ["first", "second", "third"][(level.clamp(1, 3) - 1) as usize]
}

/// After a story has been heard and paid as one: if the creature's rung is a meaning
/// the teller has not found, the story goes to be weighed. Returns what to tell the
/// teller of it (empty when the rung is a word, or the switch is off).
pub fn ask(
    world: &mut World,
    actor: &str,
    creature: EntityId,
    text: &str,
    paid: f64,
) -> Result<String, CommandError> {
    if !on(world) {
        return Ok(String::new());
    }
    let Some(v) = secret(world, creature) else {
        return Ok(String::new());
    };
    let Some(t) = kept(world, creature) else {
        return Ok(String::new());
    };
    let level = v["level"].as_u64().unwrap_or(2) as u8;
    let name = mind::display(world, creature);
    if v["struck"]
        .as_array()
        .is_some_and(|a| a.iter().any(|s| s == actor))
    {
        return Ok(String::new());
    }
    let now = world.now().0;
    let mut list = weighing(world);
    if list.iter().any(|j| {
        j["from"] == actor
            && j["creature"] == creature.0
            && j["at"].as_i64().is_some_and(|at| now - at < 3_600_000)
    }) {
        return Ok(format!(
            " Its {} word is a meaning, and it weighs one story an hour from a name against it; it has weighed one of yours within the hour, so this one was heard as a story.",
            nth(level)
        ));
    }
    if !awake(world) {
        return Ok(format!(
            " Its {} word is a meaning, not a word, and what weighs a meaning is asleep tonight: the story was heard as a story, and nothing of the word was spent. (`tell {} \"what is your word?\"` says what it will say of it.)",
            nth(level),
            mind::name(world, creature).unwrap_or_default()
        ));
    }
    let id = list
        .iter()
        .filter_map(|j| j["id"].as_u64())
        .max()
        .unwrap_or(0)
        + 1;
    let story: String = text.trim().chars().take(STORY_MAX).collect();
    list.push(json!({"id": id, "from": actor, "creature": creature.0, "name": name, "level": level, "key": t.key,
                     "story": story, "paid": paid, "at": now, "done": Value::Null}));
    if list.len() > KEPT {
        list.drain(..list.len() - KEPT);
    }
    let mut changes = BTreeMap::new();
    changes.insert("weighing".into(), json!(list));
    super::docent::write(world, changes).map_err(CommandError::precondition)?;
    let due = world
        .now()
        .plus(foxing_time::Duration::seconds(SLEPT_AFTER_S));
    world
        .schedule_once(JOB_SLEPT, due, json!({"id": id}), |p| p["id"] == id)
        .map_err(CommandError::precondition)?;
    Ok(format!(
        " Its {} word is a meaning, not a word: it is turning your story over to see whether that is what it was about, and will say what it made of it in a few minutes, wherever you are (`messages` keeps it).",
        nth(level)
    ))
}

/// What the creature says of the meaning it keeps, plainer with each story of this
/// name's it has weighed and found short.
pub fn hint(world: &World, actor: &str, creature: EntityId) -> Option<String> {
    let t = kept(world, creature)?;
    let v = secret(world, creature)?;
    let level = v["level"].as_u64().unwrap_or(2) as u8;
    if v["struck"]
        .as_array()
        .is_some_and(|a| a.iter().any(|s| s == actor))
    {
        return Some("You found what I keep under the others, and I keep another now for whoever comes next. It is not yours to find twice.".to_string());
    }
    let short = weighing(world)
        .iter()
        .filter(|j| {
            j["from"] == actor
                && j["creature"] == creature.0
                && j["key"] == t.key
                && j["score"].as_u64().is_some_and(|s| s < 3)
        })
        .count();
    let said = t.hints[short.min(2)];
    let sleeping = if awake(world) {
        ""
    } else {
        " (What weighs it is asleep tonight; it will keep.)"
    };
    Some(format!(
        "{} What I keep under it now is not a word but a meaning, the {} of three. {said} Tell me a story that is about that, in any words you like, and I will know it; I weigh one story an hour from a name, and say what I made of it a little after.{sleeping}",
        if level == 2 { "The first was found." } else { "Two were found." },
        nth(level),
    ))
}

/// The switch thrown, or a creature come to a rung: its secret is made to match. A
/// word at the second or third rung becomes a meaning while the switch is on, and a
/// meaning becomes a word again when it is off. Returns whether the secret is a
/// meaning now.
pub fn settle(world: &mut World, creature: EntityId) -> Result<bool, CommandError> {
    let Some(v) = secret(world, creature) else {
        return Ok(false);
    };
    let level = v["level"].as_u64().unwrap_or(1) as u8;
    let is = v["from"] == "meaning";
    let should = on(world) && level >= 2;
    if is == should {
        return Ok(is);
    }
    let struck = v["struck"].clone();
    let turn = struck.as_array().map_or(0, Vec::len);
    let next = if should {
        json!({"word": pick(creature, level, turn, "").key, "level": level, "struck": struck, "from": "meaning"})
    } else {
        let (word, from) = super::secret::word_again(world, creature, level);
        json!({"word": word, "level": level, "struck": struck, "from": from})
    };
    let mut changes = BTreeMap::new();
    changes.insert("secret".into(), Field::Plain(next));
    mind::set_mind_pub(world, Actor::Driver, creature, changes)?;
    Ok(should)
}

fn line_ok(line: &str, t: &Theme, score: u64) -> bool {
    let n = line.chars().count();
    if !(6..=LINE_MAX).contains(&n)
        || line.contains(['{', '}', '`', '<', '>', '\n'])
        || line.to_lowercase().contains("http")
    {
        return false;
    }
    if score >= 3 {
        return true;
    }
    // Short of a find, the line does not say the meaning: three of its longer words
    // in one line is saying it.
    let low = line.to_lowercase();
    let said = t
        .meaning
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 5)
        .filter(|w| low.contains(&w.to_lowercase()))
        .count();
    said < 3
}

const PLAIN: [&str; 4] = [
    "I turned it over. It is not about what I keep.",
    "There is a thing in it that brushes mine, and no more than brushes.",
    "Near. What I keep is in your story, and your story is about something else.",
    "That is it. That is what I keep.",
];

/// The verdict: said to the teller, and at 3 the meaning is found.
fn weigh(
    world: &mut World,
    by: &str,
    id: u64,
    score: u64,
    line: &str,
    model: &str,
    curves: &Curves,
) -> Result<String, CommandError> {
    if score > 3 {
        return Err(CommandError::precondition("A weighing is 0, 1, 2 or 3."));
    }
    let mut list = weighing(world);
    let Some(j) = list.iter_mut().find(|j| j["id"] == id) else {
        return Err(CommandError::precondition(format!(
            "No story {id} is being weighed."
        )));
    };
    if !j["done"].is_null() && j["done"] != false {
        return Err(CommandError::precondition(format!(
            "Story {id} was weighed already."
        )));
    }
    let late = j["done"] == false;
    let (from, creature, level, key, paid) = (
        j["from"].as_str().unwrap_or("").to_string(),
        EntityId(j["creature"].as_u64().unwrap_or(0)),
        j["level"].as_u64().unwrap_or(2) as u8,
        j["key"].as_str().unwrap_or("").to_string(),
        j["paid"].as_f64().unwrap_or(0.0),
    );
    let gist = super::exchange::gist(j["story"].as_str().unwrap_or(""));
    let Some(t) = theme(&key) else {
        return Err(CommandError::precondition(
            "That meaning is not one the world keeps.",
        ));
    };
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    let line = if line_ok(&line, t, score) {
        line
    } else {
        PLAIN[score as usize].to_string()
    };
    // The creature may have come apart, or another name found this meaning first.
    let still = kept(world, creature).is_some_and(|k| k.key == key)
        && secret(world, creature).is_some_and(|v| {
            !v["struck"]
                .as_array()
                .is_some_and(|a| a.iter().any(|s| *s == from))
        });
    let score = if score == 3 && !still { 2 } else { score };
    j["done"] = json!(world.now().0);
    j["score"] = json!(score);
    let name = j["name"].as_str().unwrap_or("it").to_string();
    let mut changes = BTreeMap::new();
    changes.insert("weighing".into(), json!(list));
    super::docent::write(world, changes).map_err(CommandError::precondition)?;
    world
        .cancel(JOB_SLEPT, |p| p["id"] == id)
        .map_err(CommandError::precondition)?;
    let after = if late { ", late" } else { "" };
    let mut found = Value::Null;
    let told = if score == 3 {
        let v = secret(world, creature).unwrap_or(json!({}));
        let mut struck: Vec<String> = v["struck"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        struck.push(from.clone());
        let top = level >= super::secret::LEVELS;
        let next_level = (level + 1).min(super::secret::LEVELS);
        let next = pick(
            creature,
            next_level,
            if top { struck.len() } else { 0 },
            t.key,
        );
        let mut changes = BTreeMap::new();
        changes.insert(
            "secret".into(),
            Field::Plain(json!({"word": next.key, "level": next_level, "struck": if top { struck.clone() } else { Vec::new() }, "from": "meaning"})),
        );
        mind::set_mind_pub(world, Actor::Driver, creature, changes)?;
        let bonus = super::secret::BONUS[(level.clamp(1, 3) - 1) as usize];
        let extra = (paid * (bonus - 1.0)).round().max(1.0);
        let earned = match super::find_citizen(world, &from) {
            Some(f) => Some(super::memory::earn_chunk(
                world,
                &from,
                f.window,
                &format!("finding what {name} kept under its others"),
                extra,
                0.8,
                curves,
            )?),
            None => None,
        };
        super::bard::did(world, &from, "word");
        let _ = world.note(
            Actor::Citizen(from.clone()),
            super::secret::STRUCK,
            json!({"from": from, "creature": creature.0, "name": name, "word": t.meaning, "level": level, "bonus": bonus, "meaning": true}),
        );
        world.notice(format!("**{from}** told {name} a story about what it kept under its others: *{}* (level {level}).", t.meaning));
        found = json!(t.meaning);
        format!(
            "{name} has turned your story over (\"{gist}\"){after}: \"{line}\" It was about what it kept under the others, the {} of three: {}. {}{}",
            nth(level),
            t.meaning,
            match earned {
                Some(c) => format!("You remember it: chunk-{} worth {extra:.1} XP, {} of what the story paid, and vivid.", c.0, if bonus >= 2.0 { format!("{:.1} times", bonus - 1.0) } else { "half".to_string() }),
                None => String::new(),
            },
            if top { " There is none under that; it keeps another now for the next teller." } else { " There is another under that." }
        )
    } else {
        format!(
            "{name} has turned your story over (\"{gist}\"){after}: \"{line}\" {} Its hint is plainer for it: `tell {} \"what is your word?\"`.",
            ["Not about what it keeps.", "Far from what it keeps: a likeness on the surface.", "Near what it keeps, and not about it."][score as usize],
            mind::name(world, creature).unwrap_or(name.clone())
        )
    };
    super::combat::narrate(world, &from, told);
    let _ = world.note(
        Actor::Citizen(by.to_string()),
        WEIGHED,
        json!({"id": id, "from": from, "creature": creature.0, "name": name, "level": level, "score": score, "model": model, "meaning": found}),
    );
    Ok(from)
}

/// For the tool: the stories waiting to be weighed, each with the meaning and who
/// keeps it.
pub fn pending(world: &World) -> Vec<Value> {
    let now = world.now().0;
    weighing(world)
        .iter()
        .filter(|j| j["done"].is_null() || j["done"] == false)
        .filter(|j| j["at"].as_i64().is_some_and(|at| now - at < if j["done"] == false { 1_800_000 } else { SLEPT_AFTER_S * 1000 }))
        .filter_map(|j| {
            let t = theme(j["key"].as_str()?)?;
            let creature = EntityId(j["creature"].as_u64()?);
            Some(json!({
                "id": j["id"], "meaning": t.meaning, "story": j["story"],
                "creature": {"name": j["name"], "kind": super::plain_str(world, creature, super::CREATURE, "name")},
            }))
        })
        .collect()
}

pub fn register(world: &mut World, curves: Curves) {
    world.on_job(
        JOB_SLEPT,
        Box::new(|w: &mut World, job| {
            let id = job.payload["id"].as_u64().unwrap_or(0);
            let mut list = weighing(w);
            let Some(j) = list.iter_mut().find(|j| j["id"] == id) else {
                return Ok(());
            };
            if !j["done"].is_null() {
                return Ok(());
            }
            j["done"] = json!(false);
            let (to, name) = (j["from"].as_str().unwrap_or("").to_string(), j["name"].as_str().unwrap_or("It").to_string());
            // Not counted against the hour: the next story may be weighed.
            j["at"] = json!(0);
            let mut changes = BTreeMap::new();
            changes.insert("weighing".into(), json!(list));
            super::docent::write(w, changes)?;
            super::combat::narrate(
                w,
                &to,
                format!("{name} has not weighed your story after all: what weighs a meaning fell asleep over it. Nothing of its word was spent; the next story you tell it may be weighed."),
            );
            Ok(())
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.weigh",
            "Operator. The verdict on a story being weighed against a creature's meaning (GET /v0/docent lists them under weighing): 0 not about it, 1 a surface likeness, 2 near, 3 about it; and one line the creature says, at most 200 characters. Said to the teller wherever they stand; a 3 finds the meaning.",
        )
        .param("id", ParamType::Integer, true, "the story's id")
        .param("score", ParamType::Integer, true, "0 to 3")
        .param("line", ParamType::Text, true, "what the creature says of it, quoted")
        .param("model", ParamType::String, false, "what weighed it, for the record")
        .allowed(&[ActorKind::Admin]),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            let id = args.int("id").unwrap_or(0).max(0) as u64;
            let score = args.int("score").unwrap_or(-1);
            if score < 0 {
                return Err(CommandError::precondition("A weighing is 0, 1, 2 or 3."));
            }
            let line = args.str("line").unwrap_or("").to_string();
            let model = args.str("model").unwrap_or("").trim().to_string();
            let to = weigh(w, &ctx.actor, id, score as u64, &line, &model, &curves)?;
            Ok(Outcome::new(format!("Weighed: story {id}, {score} of 3, said to {to}."), json!({"id": id, "score": score, "to": to})))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.sense",
            "Operator. Whether a named creature's second and third words are meanings, weighed by the Docent's model (`sense meanings`), or words, as the first is (`sense words`). Bare: which it is, whether the model is listening, and what is being weighed. Creatures change over as they are next told a story or asked their word.",
        )
        .param("as", ParamType::String, false, "meanings or words")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _ctx, args: &Args| {
            // (Said as `meanings` and `words`: the gate reads a bare `on` as a
            // preposition, and `sense on` arrived as `sense`.)
            let to = args.str("as").unwrap_or("").trim().to_lowercase();
            match to.as_str() {
                "meanings" | "words" | "on" | "off" => {
                    let mut changes = BTreeMap::new();
                    changes.insert("sense".into(), json!(to == "meanings" || to == "on"));
                    super::docent::write(w, changes).map_err(CommandError::precondition)?;
                    // Every named creature at a second or third rung changes over now.
                    let named: Vec<EntityId> = w.store.of_kind("creature").map(|e| e.id).filter(|c| mind::name(w, *c).is_some()).collect();
                    for c in named {
                        settle(w, c)?;
                    }
                }
                "" => {}
                _ => return Err(CommandError::precondition("`sense meanings` or `sense words`.")),
            }
            let open = weighing(w).iter().filter(|j| j["done"].is_null()).count();
            Ok(Outcome::new(
                format!(
                    "The second and third words are {}. The model is {}. {open} being weighed, {} themes.",
                    if on(w) { "meanings" } else { "words" },
                    if awake(w) { "listening" } else { "not listening" },
                    THEMES.len()
                ),
                json!({"on": on(w), "awake": awake(w), "open": open}),
            ))
        }),
    );
}

#[cfg(test)]
pub fn wake_for_test(world: &World) {
    listened(world);
}
