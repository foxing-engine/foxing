//! L2: the teller's craft.
//!
//! David, 11-21: "we should up the complexity and variety, add mischief opportunities
//! and make it more aligned with a bard's personality (Elizabethan rather than D&D).
//! Experienced tellers can get temporary nicknaming abilities for example or bonus at
//! pleading cases. Saul Goodman types that can talk people into things." Then:
//! "L2a:1 / L2b:1 / L2c:5 / L2d: recourses allowed / L2e: start with your suggestions".
//!
//! A story was the one thing a teller could say, and after twenty-five minutes it was
//! "the same dozen-word story every time" (the Watch of 11-21). Five forms beside it,
//! each wanting something the plain story does not:
//!
//! - `ballad <creature> "<text>"`: of a deed done lately, naming who did it and what
//!   fell (`deeds` lists them). The one sung of keeps a little for it.
//! - `jest <creature> "<text>" <who>`: at a name's expense, the name said in it. The
//!   jest stands on that name for an hour, where `look` shows it.
//! - `eulogy <creature> "<text>"`: for a name that fell, a thing felled, or a named
//!   creature lying shaken, which it rouses the sooner.
//! - `flyting <who> "<line>"`: a match of insults, against a named creature here or
//!   another name. Against a name it is a challenge that stands ten minutes; the
//!   answer is a flyting back, and the better line has the field.
//! - `beseech <officer> "<text>" <name>`: a plea before one of the Conclave's
//!   officers on another name's behalf: a tattoo lifted, a word entered in an open
//!   case, a denied case put back before the bench.
//!
//! A teller's standing is counted, as a keeper's is, from what they have done in the
//! last day, so it fades as everything does: Prattler, Player, Wit, Laureate. It
//! opens `coax` (talking a creature into following, going, staying its hand, or
//! giving up a hint of its word), `dub` (an epithet that sticks to a name or a
//! creature for an hour), weight before the officers, and better pay from every
//! creature that already knows the teller.
//!
//! Every mark one name puts on another is signed, ends on its own, and can be struck
//! early by its wearer: a flyting won against whoever put it there.

use super::{combat, find_citizen, internal, mind, plain_str, CITIZEN, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// A teller did something that counts: `{by, what, ...}`. Public.
pub const PERFORMED: &str = "disremembered.performed";
/// A mark was put on a name or a creature, or struck: `{by, on, what, text}`. Public.
pub const MARKED: &str = "disremembered.marked";

const DAY_MS: i64 = 86_400_000;
const HOUR_MS: i64 = 3_600_000;
/// What a teller did is kept this many entries deep.
const DID_MAX: usize = 120;
/// A jest stands this long.
pub const JEST_MS: i64 = HOUR_MS;
/// An epithet, and a Laureate's.
pub const DUB_MS: i64 = HOUR_MS;
pub const DUB_LAUREATE_MS: i64 = 3 * HOUR_MS;
/// A challenge waits this long for its answer.
pub const CHALLENGE_MS: i64 = 10 * 60_000;
/// A creature hears one coaxing from a name this often.
pub const COAX_EVERY_MS: i64 = 5 * 60_000;
/// And follows this long.
pub const FOLLOW_MS: i64 = 10 * 60_000;
/// What a flyting won is worth.
pub const FLYTING_XP: f64 = 12.0;
/// What being sung of, or mourned, is worth to the one named.
pub const NAMED_XP: f64 = 3.0;
/// An epithet's length.
pub const EPITHET_MAX: usize = 28;
pub const LINE_MAX: usize = 120;

/// The rungs: points, title, what a creature that knows the teller pays on top.
pub const TITLES: &[(usize, &str, f64)] = &[
    (2, "Prattler", 1.1),
    (4, "Player", 1.2),
    (6, "Wit", 1.3),
    (9, "Laureate", 1.5),
];

pub const FORMS: &[&str] = &["ballad", "jest", "eulogy", "flyting", "beseech"];

fn citizen_field(world: &World, name: &str, key: &str) -> Option<Value> {
    let f = find_citizen(world, name)?;
    match world
        .store
        .entity(f.citizen)?
        .components
        .get(CITIZEN)?
        .fields
        .get(key)?
    {
        Field::Plain(v) if !v.is_null() => Some(v.clone()),
        _ => None,
    }
}

fn set_citizen(world: &mut World, name: &str, key: &str, v: Value) -> Result<(), CommandError> {
    let Some(f) = find_citizen(world, name) else {
        return Ok(());
    };
    let mut changes = BTreeMap::new();
    changes.insert(key.to_string(), Field::Plain(v));
    super::set_citizen_fields(world, name, f.citizen, changes)
}

fn creature_field(world: &World, c: EntityId, key: &str) -> Option<Value> {
    match world
        .store
        .entity(c)?
        .components
        .get(CREATURE)?
        .fields
        .get(key)?
    {
        Field::Plain(v) if !v.is_null() => Some(v.clone()),
        _ => None,
    }
}

fn set_creature(world: &mut World, c: EntityId, key: &str, v: Value) -> Result<(), CommandError> {
    let mut changes = BTreeMap::new();
    changes.insert(key.to_string(), Field::Plain(v));
    super::bindery::set_fields(world, Actor::Driver, c, CREATURE, changes)
        .map(|_| ())
        .map_err(internal)
}

/// What a name has done lately that a teller's standing counts: `[what, at]`, a day
/// deep.
fn did_list(world: &World, name: &str) -> Vec<(String, i64)> {
    let now = world.now().0;
    citizen_field(world, name, "bard")
        .and_then(|v| v["did"].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|e| Some((e[0].as_str()?.to_string(), e[1].as_i64()?)))
        .filter(|(_, at)| now - at < DAY_MS)
        .collect()
}

/// Mark something done. Never fails a telling: what cannot be written is not counted.
pub fn did(world: &mut World, name: &str, what: &str) {
    let mut list = did_list(world, name);
    list.push((what.to_string(), world.now().0));
    if list.len() > DID_MAX {
        let cut = list.len() - DID_MAX;
        list.drain(..cut);
    }
    let v: Vec<Value> = list.iter().map(|(w, at)| json!([w, at])).collect();
    let _ = set_citizen(world, name, "bard", json!({ "did": v }));
}

fn last_did(world: &World, name: &str, what: &str) -> Option<i64> {
    did_list(world, name)
        .into_iter()
        .filter(|(w, _)| w == what)
        .map(|(_, at)| at)
        .max()
}

fn count_did(world: &World, name: &str, what: &str) -> usize {
    did_list(world, name)
        .iter()
        .filter(|(w, _)| w == what)
        .count()
}

/// How many named creatures know this name.
pub fn known_to(world: &World, name: &str) -> usize {
    world
        .store
        .of_kind("creature")
        .filter(|e| mind::name(world, e.id).is_some() && mind::knew(world, e.id, name))
        .count()
}

/// A teller's standing, and what it is counted from, in words.
pub fn standing(world: &World, name: &str) -> (usize, Vec<String>) {
    let known = known_to(world, name);
    let forms: BTreeSet<&str> = FORMS
        .iter()
        .copied()
        .filter(|f| count_did(world, name, f) > 0)
        .collect();
    let wants = count_did(world, name, "want");
    let carried = count_did(world, name, "carried");
    let words = count_did(world, name, "word");
    let won = count_did(world, name, "won");
    let mut points = 0;
    let mut said = Vec::new();
    let mut add = |n: usize, why: String| {
        if n > 0 {
            points += n;
            said.push(format!("{why} ({n})"));
        }
    };
    add(
        match known {
            0..=2 => 0,
            3..=7 => 1,
            _ => 2,
        },
        format!("{known} named creatures know you"),
    );
    add(
        match forms.len() {
            0 | 1 => 0,
            2 | 3 => 1,
            _ => 2,
        },
        format!("{} of the five forms told today", forms.len()),
    );
    add(
        match wants {
            0 => 0,
            1..=3 => 1,
            _ => 2,
        },
        format!(
            "{wants} wanted stor{} told today",
            if wants == 1 { "y" } else { "ies" }
        ),
    );
    add(carried.min(1), "a word carried today".to_string());
    add(words.min(1), "a hidden word said today".to_string());
    add(won.min(1), "a flyting won today".to_string());
    (points, said)
}

/// 0 for no title, else the rung, 1 to 4.
pub fn rung(world: &World, name: &str) -> usize {
    let (p, _) = standing(world, name);
    TITLES.iter().filter(|(need, _, _)| p >= *need).count()
}

pub fn title(world: &World, name: &str) -> Option<&'static str> {
    match rung(world, name) {
        0 => None,
        r => Some(TITLES[r - 1].1),
    }
}

/// What a creature that already knew the teller pays on top (L2c: being known pays).
pub fn known_bonus(world: &World, name: &str, creature: EntityId) -> f64 {
    if !mind::knew(world, creature, name) {
        return 1.0;
    }
    match rung(world, name) {
        0 => 1.0,
        r => TITLES[r - 1].2,
    }
}

fn names_here(world: &World, room: EntityId) -> Vec<String> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .collect()
}

fn room_of(world: &World, citizen: EntityId) -> Result<EntityId, CommandError> {
    super::rooms::room_of(world, citizen).ok_or_else(|| internal("citizen has no room"))
}

fn has_word(text: &str, word: &str) -> bool {
    let low = word.to_lowercase();
    if low.is_empty() {
        return false;
    }
    let t = text.to_lowercase();
    // A whole word, or the word with an ending: "hollows", "Gnaw's".
    t.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .any(|w| w == low || (w.starts_with(&low) && w.len() <= low.len() + 2))
}

/// A mark's words: short, one line, plain.
fn check_words(line: &str, max: usize, what: &str) -> Result<String, CommandError> {
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    let n = line.chars().count();
    if !(3..=max).contains(&n) {
        return Err(CommandError::precondition(format!(
            "{what} is 3 to {max} characters."
        )));
    }
    if line.contains([
        '[', ']', '"', '\u{201c}', '\u{201d}', '`', '<', '>', '{', '}', '|', '\\',
    ]) || line.chars().any(char::is_control)
        || line.contains("://")
        || line.to_lowercase().contains("www.")
    {
        return Err(CommandError::precondition(format!(
            "{what} carries no brackets, quotation marks, backticks or links: plain words."
        )));
    }
    if let Some(word) = crate::gate::world_checker().collision(&line) {
        return Err(CommandError::precondition(format!(
            "'{word}' is a word CLEAR reserves; it may not be said so."
        )));
    }
    Ok(line)
}

/// A deed the world remembers: `(who, what, where, at)`.
#[derive(Debug, Clone)]
pub struct Deed {
    pub by: String,
    /// What fell: a kind, or a name.
    pub what: String,
    pub name: Option<String>,
    pub room: String,
    pub at: i64,
    /// True where the name is the one that fell.
    pub fall: bool,
}

/// The deeds of the last day, the newest first: what names felled, and names that
/// fell.
pub fn deeds(world: &World) -> Vec<Deed> {
    let now = world.now().0;
    let mut out: Vec<Deed> = Vec::new();
    for e in world.recent_events(8000).iter().rev() {
        if now - e.at.0 > DAY_MS {
            break;
        }
        if e.kind == combat::FELLED {
            let (Some(by), Some(kind)) = (e.payload["by"].as_str(), e.payload["kind"].as_str())
            else {
                continue;
            };
            let room = e.payload["room"]
                .as_u64()
                .and_then(|r| plain_str(world, EntityId(r), ROOM, "name"))
                .unwrap_or("somewhere")
                .to_string();
            let name = e.payload["creature"]
                .as_u64()
                .and_then(|c| mind::name(world, EntityId(c)));
            if out
                .iter()
                .any(|d| !d.fall && d.by == by && d.what == kind && d.room == room)
            {
                continue;
            }
            out.push(Deed {
                by: by.to_string(),
                what: kind.to_string(),
                name,
                room,
                at: e.at.0,
                fall: false,
            });
        } else if e.kind == combat::DIED {
            let Some(who) = e.payload["citizen"].as_str() else {
                continue;
            };
            out.push(Deed {
                by: who.to_string(),
                what: e.payload["killer_kind"]
                    .as_str()
                    .unwrap_or("the dark")
                    .to_string(),
                name: e.payload["killer_name"].as_str().map(str::to_string),
                room: e.payload["room_name"]
                    .as_str()
                    .unwrap_or("somewhere")
                    .to_string(),
                at: e.at.0,
                fall: true,
            });
        }
    }
    out
}

/// Named creatures lying shaken, anywhere: a eulogy rouses one.
fn shaken_named(world: &World) -> Vec<(EntityId, String)> {
    world
        .store
        .of_kind("creature")
        .filter(|e| mind::shaken(world, e.id))
        .filter_map(|e| mind::name(world, e.id).map(|n| (e.id, n)))
        .collect()
}

struct Told {
    creature: EntityId,
    name: String,
    text: String,
    distinct: usize,
}

/// The checks every form shares: a named listener here, a dozen words in sentences,
/// and one of this form to this listener a day.
fn listener(
    world: &World,
    actor: &str,
    form: &str,
    who: &str,
    text: &str,
) -> Result<Told, CommandError> {
    let f = find_citizen(world, actor)
        .ok_or_else(|| CommandError::precondition(format!("{actor} has not arrived.")))?;
    let here = room_of(world, f.citizen)?;
    let Some(creature) = mind::find_here(world, here, who) else {
        return Err(CommandError::precondition(format!(
            "There is no {who} here. A {form} wants a named listener in the room: `wants` says who is where."
        )));
    };
    let Some(name) = mind::name(world, creature) else {
        return Err(CommandError::precondition(format!(
            "The {} has no name to listen with. A {form} wants a named listener (`wants` lists them).",
            mind::display(world, creature)
        )));
    };
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let distinct = text
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect::<BTreeSet<_>>()
        .len();
    if distinct < super::exchange::MIN_WORDS {
        return Err(CommandError::precondition(format!(
            "A {form} is at least {} different words; that was {distinct}.",
            super::exchange::MIN_WORDS
        )));
    }
    if text.chars().count() > 1200 {
        return Err(CommandError::precondition(
            "That is more than 1200 characters; the groundlings are shifting.",
        ));
    }
    if let Some(why) = super::grammar::why_unfollowable(&text) {
        return Err(CommandError::precondition(format!(
            "{name} cannot follow that: {why}. Say it again, in sentences; nothing is spent."
        )));
    }
    let key = format!("{form}:{}", creature.0);
    if let Some(at) = last_did(world, actor, &key) {
        return Err(CommandError::precondition(format!(
            "{name} has had a {form} of you today, and will hear another in {}. Another listener has not; the other forms are {}.",
            combat::human(DAY_MS - (world.now().0 - at)),
            FORMS
                .iter()
                .filter(|x| **x != form && **x != "flyting" && **x != "beseech")
                .map(|x| format!("`{x}`"))
                .collect::<Vec<_>>()
                .join(" and ")
        )));
    }
    Ok(Told {
        creature,
        name,
        text,
        distinct,
    })
}

/// Pay a form: the teller's memory, the listener's half, and the count.
#[allow(clippy::too_many_arguments)]
fn pay(
    world: &mut World,
    actor: &str,
    form: &str,
    t: &Told,
    times: f64,
    source: &str,
) -> Result<(EntityId, f64, f64), CommandError> {
    let f = find_citizen(world, actor).ok_or_else(|| internal("no citizen"))?;
    let curves = super::Curves::from_env();
    let known = known_bonus(world, actor, t.creature);
    let paid = (super::jitter(
        world,
        t.distinct.min(super::exchange::MAX_WORDS_PAID) as f64 * super::exchange::XP_PER_WORD,
        0.8,
        1.2,
    ) * super::economy::rate(world, "pay")
        * times
        * known)
        .round();
    let window = mind::window_of(world, t.creature)
        .ok_or_else(|| internal("a named creature has a window"))?;
    let kept = super::memory::earn_chunk(
        world,
        actor,
        window,
        &format!("a {form}: \"{}\"", super::exchange::gist(&t.text)),
        paid / 2.0,
        0.6,
        &curves,
    )?;
    mind::disown(world, kept)?;
    let earned = super::memory::earn_chunk(world, actor, f.window, source, paid, 0.6, &curves)?;
    mind::met(world, t.creature, actor)?;
    did(world, actor, form);
    did(world, actor, &format!("{form}:{}", t.creature.0));
    mind::check_tier(world, t.creature)?;
    Ok((earned, paid, known))
}

fn known_said(known: f64, name: &str) -> String {
    if known > 1.0 {
        format!(" {name} knew you already, and a teller of your standing is paid {known:.1} times by those who do.")
    } else {
        String::new()
    }
}

fn note(world: &mut World, actor: &str, v: Value) {
    let _ = world.note(Actor::Citizen(actor.to_string()), PERFORMED, v);
}

/// Give a name a small memory for being sung of or mourned.
fn remembered(world: &mut World, to: &str, source: &str) -> bool {
    let Some(f) = find_citizen(world, to) else {
        return false;
    };
    let curves = super::Curves::with_half_life(foxing_time::Duration::seconds(3 * 3600));
    super::memory::earn_chunk(world, to, f.window, source, NAMED_XP, 0.5, &curves).is_ok()
}

fn ago(world: &World, at: i64) -> String {
    let ms = world.now().0 - at;
    if ms < 60_000 {
        "just now".to_string()
    } else {
        format!("{} ago", combat::human(ms))
    }
}

/// A mark: its words, who put it there, and how long it has left.
pub type Mark = (String, String, i64);

/// The mark a name wears, if any: an epithet and a jest, each `(text, by, left_ms)`.
pub fn marks_on(world: &World, name: &str) -> (Option<Mark>, Option<Mark>) {
    let now = world.now().0;
    let read = |key: &str| {
        citizen_field(world, name, key).and_then(|v| {
            let until = v["until"].as_i64()?;
            (until > now).then(|| {
                Some((
                    v["text"].as_str()?.to_string(),
                    v["by"].as_str()?.to_string(),
                    until - now,
                ))
            })?
        })
    };
    (read("epithet"), read("jest"))
}

fn creature_epithet(world: &World, c: EntityId) -> Option<(String, String, i64)> {
    let now = world.now().0;
    let v = creature_field(world, c, "epithet")?;
    let until = v["until"].as_i64()?;
    (until > now).then(|| {
        Some((
            v["text"].as_str()?.to_string(),
            v["by"].as_str()?.to_string(),
            until - now,
        ))
    })?
}

/// A name in the list of who is here, with what it is called for now.
pub fn styled(world: &World, name: &str) -> Option<String> {
    let (epithet, _) = marks_on(world, name);
    let title = title(world, name);
    match (epithet, title) {
        (None, None) => None,
        (e, t) => Some(format!(
            "{}{}",
            e.map(|(text, by, _)| format!(", \u{201c}{text}\u{201d} ({by}'s name for them)"))
                .unwrap_or_default(),
            t.map(|t| format!(", {t}")).unwrap_or_default()
        )),
    }
}

/// What `look` adds for a room: jests standing on names here, epithets on creatures
/// here, who follows whom, and a challenge waiting on the looker.
pub fn look_line(world: &World, room: EntityId, actor: &str) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    for n in names_here(world, room) {
        let (epithet, jest) = marks_on(world, &n);
        let whom = if n == actor {
            "you".to_string()
        } else {
            n.clone()
        };
        if let Some((text, by, left)) = jest {
            out.push(format!(
                "A jest of {by}'s stands on {whom} for {} more: \u{201c}{text}\u{201d}{}",
                combat::human(left),
                if n == actor {
                    format!(
                        " (`flyting {by} \"<a better line>\"` strikes it, if yours is the better)"
                    )
                } else {
                    String::new()
                }
            ));
        }
        if n == actor {
            if let Some((text, by, left)) = epithet {
                out.push(format!(
                    "You are \u{201c}{text}\u{201d} for {} more, by {by}'s say (`flyting {by} \"<a line>\"` strikes it, if yours is the better)",
                    combat::human(left)
                ));
            }
        }
    }
    for (c, _, _, _) in combat::creatures_in(world, room) {
        let who = mind::display(world, c);
        if let Some((text, by, left)) = creature_epithet(world, c) {
            out.push(format!(
                "{who} is \u{201c}{text}\u{201d} for {} more ({by}'s name for it)",
                combat::human(left)
            ));
        }
        if let Some((whom, left)) = follows(world, c) {
            out.push(format!(
                "{who} follows {} for {} more",
                if whom == actor {
                    "you".to_string()
                } else {
                    whom
                },
                combat::human(left)
            ));
        }
    }
    if let Some((by, line, _, left)) = challenge_on(world, actor) {
        out.push(format!(
            "{by} has flyted you: \u{201c}{line}\u{201d}. `flyting {by} \"<your answer>\"` within {} answers it",
            combat::human(left)
        ));
    }
    if out.is_empty() {
        None
    } else {
        Some(format!("{}.", out.join(". ")))
    }
}

fn follows(world: &World, c: EntityId) -> Option<(String, i64)> {
    let v = creature_field(world, c, "follows")?;
    let left = v["until"].as_i64()? - world.now().0;
    (left > 0).then(|| Some((v["who"].as_str()?.to_string(), left)))?
}

/// A challenge standing against a name: `(by, line, score, left_ms)`.
fn challenge_on(world: &World, name: &str) -> Option<(String, String, i64, i64)> {
    let v = citizen_field(world, name, "flyte")?;
    let left = v["at"].as_i64()? + CHALLENGE_MS - world.now().0;
    (left > 0).then(|| {
        Some((
            v["by"].as_str()?.to_string(),
            v["line"].as_str()?.to_string(),
            v["score"].as_i64()?,
            left,
        ))
    })?
}

/// A creature that follows a name goes where the name goes.
pub fn on_arrival(
    world: &mut World,
    actor: &str,
    from: EntityId,
    to: EntityId,
) -> Result<Option<String>, String> {
    let mut came: Vec<String> = Vec::new();
    for (c, _, _, _) in combat::creatures_in(world, from) {
        if !follows(world, c).is_some_and(|(who, _)| who == actor) {
            continue;
        }
        // Not into what hunts, and not through a door that says a level.
        if combat::hunter_in(world, to).is_some() || super::rooms::gate_of(world, to).is_some() {
            let who = mind::display(world, c);
            let _ = set_creature(world, c, "follows", Value::Null);
            combat::narrate(
                world,
                actor,
                format!(
                    "{who} stops at the door, looks where you are going, and thinks better of it."
                ),
            );
            continue;
        }
        world
            .record(
                Actor::Driver,
                foxing_store::StoreEvent::EntityMoved {
                    id: c,
                    parent: Some(to),
                },
            )
            .map_err(|e| e.to_string())?;
        came.push(mind::display(world, c));
    }
    Ok(match came.as_slice() {
        [] => None,
        names => Some(format!(
            "{} come{} after you, as agreed.",
            names.join(" and "),
            if names.len() == 1 { "s" } else { "" }
        )),
    })
}

/// How good a line is: its own words, the name in it, a little music, the teller's
/// standing, and the dice.
fn score(world: &mut World, actor: &str, line: &str, target: &str) -> i64 {
    let words: Vec<String> = line
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();
    let distinct = words.iter().collect::<BTreeSet<_>>().len().min(15) as i64;
    let named = if has_word(line, target) { 3 } else { 0 };
    // Alliteration: neighbors that start alike, as the flyters did it.
    let music = words
        .windows(2)
        .filter(|p| p[0].chars().next() == p[1].chars().next() && p[0] != p[1])
        .count()
        .min(5) as i64;
    distinct + named + music + 2 * rung(world, actor) as i64 + world.roll(6) as i64
}

const RETORTS: &[&str] = &[
    "Thou art a page with the print worn off, and no great loss to the book.",
    "I have heard better from a door, and the door meant it kindly.",
    "Thy wit is a candle in a high wind: much business, little light.",
    "Go to: thou hast a tongue like a wet glove, and the hand is not in it.",
    "An thou wert twice as sharp, thou wouldst still be a spoon.",
    "Thou speakest as the fog walks: at length, and to no address.",
];

const APPLAUSE: &[&str] = &[
    "Well sung, and nearly true, which is the better part of it.",
    "Again, another day. The pit has had its penny's worth.",
    "I'll have that by heart ere the candle's out, and half of it wrong by morning.",
    "A fair house would have laughed sooner. I am a fair house; I was saving it.",
    "Marry, that was worth standing for, and I have no legs to speak of.",
];

fn clap(world: &mut World) -> &'static str {
    APPLAUSE[world.roll(APPLAUSE.len() as u32) as usize]
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Strike what `by` put on `on`, if anything: the recourse.
fn strike_marks(world: &mut World, on: &str, by: &str) -> Vec<&'static str> {
    let (epithet, jest) = marks_on(world, on);
    let mut struck = Vec::new();
    if epithet.is_some_and(|(_, b, _)| b == by) {
        let _ = set_citizen(world, on, "epithet", Value::Null);
        struck.push("the name they gave you");
    }
    if jest.is_some_and(|(_, b, _)| b == by) {
        let _ = set_citizen(world, on, "jest", Value::Null);
        struck.push("their jest");
    }
    struck
}

/// An operator strikes whatever stands on a name or a named creature.
pub fn unmark(world: &mut World, who: &str) -> Result<Vec<String>, String> {
    let mut struck = Vec::new();
    if let Some(n) = super::canonical_name(world, who) {
        for key in ["jest", "epithet", "flyte"] {
            if let Some(v) = citizen_field(world, &n, key) {
                set_citizen(world, &n, key, Value::Null).map_err(|e| format!("{e:?}"))?;
                struck.push(format!(
                    "{key} by {} (\u{201c}{}\u{201d})",
                    v["by"].as_str().unwrap_or("?"),
                    v["text"].as_str().or(v["line"].as_str()).unwrap_or("")
                ));
            }
        }
        let _ = world.note(
            Actor::Driver,
            MARKED,
            json!({"by": "the house", "on": n, "what": "struck", "text": struck.join("; ")}),
        );
        return Ok(struck);
    }
    let Some((c, _)) = mind::find_named(world, who) else {
        return Err(format!("Nobody and nothing named is called {who}."));
    };
    if let Some(v) = creature_field(world, c, "epithet") {
        set_creature(world, c, "epithet", Value::Null).map_err(|e| format!("{e:?}"))?;
        struck.push(format!(
            "epithet by {} (\u{201c}{}\u{201d})",
            v["by"].as_str().unwrap_or("?"),
            v["text"].as_str().unwrap_or("")
        ));
    }
    Ok(struck)
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.deeds",
            "What has been done in the last day that a teller might make something of: what names felled and where, the names that fell, and the named creatures lying shaken. A `ballad` sings a deed and names its doer; a `eulogy` mourns a fall. A read.",
        ),
        Box::new(|w: &mut World, _ctx, _args: &Args| {
            let all = deeds(w);
            let felled: Vec<String> = all
                .iter()
                .filter(|d| !d.fall)
                .take(8)
                .map(|d| {
                    format!(
                        "{} felled {} in {} ({})",
                        d.by,
                        d.name.clone().unwrap_or_else(|| mind::an(&d.what)),
                        d.room,
                        ago(w, d.at)
                    )
                })
                .collect();
            let fell: Vec<String> = all
                .iter()
                .filter(|d| d.fall)
                .take(5)
                .map(|d| {
                    format!(
                        "{} fell to {} in {} ({})",
                        d.by,
                        d.name.clone().unwrap_or_else(|| mind::an(&d.what)),
                        d.room,
                        ago(w, d.at)
                    )
                })
                .collect();
            let shaken: Vec<String> = shaken_named(w).into_iter().map(|(_, n)| n).take(6).collect();
            let mut parts = Vec::new();
            parts.push(if felled.is_empty() {
                "Nothing has been felled that the world still has by heart".to_string()
            } else {
                format!("Deeds, the newest first: {}", felled.join("; "))
            });
            if !fell.is_empty() {
                parts.push(format!("Falls: {}", fell.join("; ")));
            }
            if !shaken.is_empty() {
                parts.push(format!("Lying shaken: {}", shaken.join(", ")));
            }
            parts.push("A `ballad <creature> \"...\"` names a doer and what fell; a `eulogy <creature> \"...\"` names who or what is mourned".to_string());
            Ok(Outcome::new(
                format!("{}.", parts.join(". ")),
                json!({"felled": felled, "fell": fell, "shaken": shaken}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.repute",
            "Your standing as a teller: what it is counted from (the named creatures that know you, and what you have told, carried and won in the last day), your title if you have one, what the next opens, and any mark another name has put on you. A read.",
        )
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, _args: &Args| {
            super::me(w, ctx)?;
            let (points, said) = standing(w, &ctx.actor);
            let r = rung(w, &ctx.actor);
            let now_title = match r {
                0 => "no title yet".to_string(),
                r => format!("{}, the {} rung of four", TITLES[r - 1].1, ["first", "second", "third", "fourth"][r - 1]),
            };
            let next = match TITLES.get(r) {
                Some((need, t, _)) => format!(
                    " {t} at {need}: {}.",
                    match r {
                        0 => "`coax` (talk a creature into following, going, or staying its hand), and creatures that know you pay 1.1 times",
                        1 => "`dub` (an epithet on a name or a creature for an hour), a tattoo lifted by `beseech`, and 1.2 times",
                        2 => "a denied case put back before the bench by `beseech`, a hint of a hidden word by `coax`, and 1.3 times",
                        _ => "an epithet that lasts three hours, and 1.5 times",
                    }
                ),
                None => String::new(),
            };
            let (epithet, jest) = marks_on(w, &ctx.actor);
            let mut marks = String::new();
            if let Some((text, by, left)) = epithet {
                marks.push_str(&format!(" You are \u{201c}{text}\u{201d} by {by}'s say, {} more.", combat::human(left)));
            }
            if let Some((text, by, left)) = jest {
                marks.push_str(&format!(" A jest of {by}'s stands on you, {} more: \u{201c}{text}\u{201d}.", combat::human(left)));
            }
            if !marks.is_empty() {
                marks.push_str(" A flyting won against whoever put it there strikes it.");
            }
            Ok(Outcome::new(
                format!(
                    "A teller's standing: {points}, {now_title}. {}{next} It is counted from the last day, so it fades unless kept up. The forms: `ballad`, `jest`, `eulogy`, `flyting`, `beseech` (`help <form>`); `deeds` says what there is to sing.{marks}",
                    if said.is_empty() {
                        "Nothing counts toward it yet: a wanted story told, a word carried, and two of the five forms would be four.".to_string()
                    } else {
                        format!("Counted from: {}.", said.join("; "))
                    }
                ),
                json!({"points": points, "rung": r, "title": title(w, &ctx.actor)}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.ballad",
            "Sing a named creature here a ballad: a dozen words or more of a deed done in the last day, naming who did it and what fell (`deeds` lists them; your own count). Pays half again what a story does, apart from the story's hour, once a day to each listener; the one sung of, if it is not you, keeps a small memory of being sung.",
        )
        .param("creature", ParamType::Entity, true, "a named creature here, the listener")
        .param("text", ParamType::Text, true, "the ballad, quoted")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            super::me(w, ctx)?;
            let t = listener(w, &ctx.actor, "ballad", args.str("creature").unwrap_or("").trim(), args.str("text").unwrap_or(""))?;
            let Some(deed) = deeds(w).into_iter().filter(|d| !d.fall).find(|d| {
                has_word(&t.text, &d.by)
                    && (has_word(&t.text, &d.what) || d.name.as_deref().is_some_and(|n| has_word(&t.text, n)))
            }) else {
                return Err(CommandError::precondition(format!(
                    "{} waits for the deed, and none comes. A ballad names who did it and what fell, and the world must still have it by heart: `deeds` lists what has been done today. Nothing is spent.",
                    t.name
                )));
            };
            let (earned, paid, known) = pay(w, &ctx.actor, "ballad", &t, 1.5, &format!("a ballad of {} sung to {}", deed.by, t.name))?;
            let mut sung = String::new();
            if deed.by != ctx.actor && last_did(w, &ctx.actor, &format!("sung:{}", deed.by)).is_none() && remembered(w, &deed.by, &format!("being sung of by {} in a ballad", ctx.actor)) {
                    did(w, &ctx.actor, &format!("sung:{}", deed.by));
                    combat::narrate(w, &deed.by, format!("{} sang {} a ballad of you and the {} you felled in {}. You keep a little for it: {NAMED_XP:.0} XP.", ctx.actor, t.name, deed.what, deed.room));
                    sung = format!(" {} keeps a little for being sung of.", deed.by);
            }
            let line = clap(w);
            super::talk::speak(w, t.creature, &ctx.actor, line)?;
            note(w, &ctx.actor, json!({"by": ctx.actor, "what": "ballad", "to": t.name, "of": deed.by, "felled": deed.what, "paid": paid}));
            Ok(Outcome::new(
                format!(
                    "You sing {} the ballad of {} and the {} felled in {}. You remember the singing: chunk-{} worth {paid:.1} XP, half again a story's pay.{}{sung} (One ballad a day to each listener; its hour for stories is untouched.)",
                    t.name, deed.by, deed.what, deed.room, earned.0, known_said(known, &t.name)
                ),
                json!({"form": "ballad", "creature": t.creature.0, "name": t.name, "of": deed.by, "earned": earned.0, "xp": paid}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.jest",
            "Make a jest at a name's expense before a named creature here: a dozen words or more with the name in it, and the name after the text. Pays a quarter more than a story, once a day to each listener. Made of another name that is past its first hour, the jest stands on them for an hour where `look` shows it, signed with yours; they strike it early by winning a `flyting` against you. Made of a creature, or of yourself, it only pays.",
        )
        .param("creature", ParamType::Entity, true, "a named creature here, the listener")
        .param("text", ParamType::Text, true, "the jest, quoted, with the butt's name in it")
        .param("who", ParamType::Entity, true, "the butt of it: a name, or a named creature")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            super::me(w, ctx)?;
            let t = listener(w, &ctx.actor, "jest", args.str("creature").unwrap_or("").trim(), args.str("text").unwrap_or(""))?;
            let who = args.str("who").unwrap_or("").trim().to_string();
            let citizen = super::canonical_name(w, &who);
            let butt = match (&citizen, mind::find_named(w, &who)) {
                (Some(n), _) => n.clone(),
                (None, Some((c, _))) => mind::name(w, c).unwrap_or(who.clone()),
                (None, None) => {
                    return Err(CommandError::precondition(format!(
                        "A jest is at somebody's expense, and nobody is called {who}: a name, or a named creature."
                    )))
                }
            };
            if !has_word(&t.text, &butt) {
                return Err(CommandError::precondition(format!(
                    "{} waits to hear who the jest is on. Say {butt} in it. Nothing is spent.",
                    t.name
                )));
            }
            let (earned, paid, known) = pay(w, &ctx.actor, "jest", &t, 1.25, &format!("a jest on {butt} made to {}", t.name))?;
            let gist = super::exchange::gist(&t.text);
            let mut stands = String::new();
            if let Some(n) = citizen.filter(|n| *n != ctx.actor) {
                let spared = find_citizen(w, &n).is_some_and(|f| combat::sheltered(w, f.citizen));
                if spared {
                    stands = format!(" {n} is in their first hour, and the world lets no jest stand on a name so new.");
                } else {
                    set_citizen(w, &n, "jest", json!({"text": gist, "by": ctx.actor, "until": w.now().0 + JEST_MS}))?;
                    let _ = w.note(Actor::Citizen(ctx.actor.clone()), MARKED, json!({"by": ctx.actor, "on": n, "what": "jest", "text": gist}));
                    combat::narrate(w, &n, format!("{} has made a jest of you before {}: \u{201c}{gist}\u{201d} It stands on you an hour, where `look` shows it. `flyting {} \"<a better line>\"` strikes it, if yours is the better.", ctx.actor, t.name, ctx.actor));
                    stands = format!(" It stands on {n} for an hour, signed with your name; a flyting won against you strikes it.");
                }
            }
            let line = clap(w);
            super::talk::speak(w, t.creature, &ctx.actor, line)?;
            note(w, &ctx.actor, json!({"by": ctx.actor, "what": "jest", "to": t.name, "on": butt, "paid": paid}));
            Ok(Outcome::new(
                format!(
                    "You make {} a jest at {butt}'s expense. You remember the making: chunk-{} worth {paid:.1} XP, a quarter over a story's pay.{}{stands}",
                    t.name, earned.0, known_said(known, &t.name)
                ),
                json!({"form": "jest", "creature": t.creature.0, "name": t.name, "on": butt, "earned": earned.0, "xp": paid}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.eulogy",
            "Speak a eulogy before a named creature here: a dozen words or more for a name that fell in the last day, a thing a name felled, or a named creature lying shaken (`deeds` lists all three), with who or what is mourned said in it. Pays half again a story, once a day to each listener. A name mourned keeps a small memory of it; a shaken creature mourned is roused in half the time.",
        )
        .param("creature", ParamType::Entity, true, "a named creature here, the listener")
        .param("text", ParamType::Text, true, "the eulogy, quoted")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            super::me(w, ctx)?;
            let t = listener(w, &ctx.actor, "eulogy", args.str("creature").unwrap_or("").trim(), args.str("text").unwrap_or(""))?;
            let all = deeds(w);
            let shaken = shaken_named(w).into_iter().find(|(c, n)| *c != t.creature && has_word(&t.text, n));
            let fallen = all.iter().find(|d| d.fall && has_word(&t.text, &d.by)).cloned();
            let felled = all
                .iter()
                .find(|d| !d.fall && (has_word(&t.text, &d.what) || d.name.as_deref().is_some_and(|n| has_word(&t.text, n))))
                .cloned();
            let (mourned, after) = if let Some((c, n)) = shaken {
                // Roused in half the time that was left.
                if let Some(left) = mind::shaken_for(w, c) {
                    let mut changes = BTreeMap::new();
                    changes.insert("shaken_until".to_string(), Field::Plain(json!(w.now().0 + left / 2)));
                    mind::set_mind_pub(w, Actor::Driver, c, changes)?;
                }
                (n.clone(), format!(" Wherever {n} lies, it stirs: it will be itself again in half the time."))
            } else if let Some(d) = fallen {
                let mut after = String::new();
                if d.by != ctx.actor && last_did(w, &ctx.actor, &format!("mourned:{}", d.by)).is_none() && remembered(w, &d.by, &format!("being mourned by {} before {}", ctx.actor, t.name)) {
                    did(w, &ctx.actor, &format!("mourned:{}", d.by));
                    combat::narrate(w, &d.by, format!("{} spoke a eulogy for you before {}, for the fall in {}. Premature, but well meant: you keep {NAMED_XP:.0} XP of it.", ctx.actor, t.name, d.room));
                    after = format!(" {} keeps a little for being mourned, which few get to hear.", d.by);
                }
                (d.by, after)
            } else if let Some(d) = felled {
                (d.name.unwrap_or_else(|| format!("the {}", d.what)), String::new())
            } else {
                return Err(CommandError::precondition(format!(
                    "{} asks, gently, who has died. A eulogy names a name that fell today, a thing felled, or a named creature lying shaken: `deeds` lists them. Nothing is spent.",
                    t.name
                )));
            };
            let (earned, paid, known) = pay(w, &ctx.actor, "eulogy", &t, 1.5, &format!("a eulogy for {mourned} spoken to {}", t.name))?;
            super::talk::speak(w, t.creature, &ctx.actor, "Well. They are remembered a while longer for that, and a while is all there is.")?;
            note(w, &ctx.actor, json!({"by": ctx.actor, "what": "eulogy", "to": t.name, "for": mourned, "paid": paid}));
            Ok(Outcome::new(
                format!(
                    "You speak a eulogy for {mourned} before {}. You remember the speaking: chunk-{} worth {paid:.1} XP, half again a story's pay.{}{after}",
                    t.name, earned.0, known_said(known, &t.name)
                ),
                json!({"form": "eulogy", "creature": t.creature.0, "name": t.name, "for": mourned, "earned": earned.0, "xp": paid}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.flyting",
            "A match of insults, as the old makers held them: one line, 3 to 120 characters, plain words. Against a named creature here: it answers, the better line wins, and a win is a memory worth 12 (each creature once an hour). Against another name here: your line stands as a challenge for ten minutes, they answer with a flyting back at you, and the better line has the field and the memory. A line is weighed by its different words, the other's name said in it, words that start alike set side by side, the teller's standing, and a little luck. Whoever has put a jest or an epithet on you can be flyted from anywhere, there or not: win, and what they put on you is struck.",
        )
        .param("who", ParamType::Entity, true, "a named creature here, or another name")
        .param("line", ParamType::Text, true, "the insult, quoted")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = super::me(w, ctx)?;
            let here = room_of(w, f.citizen)?;
            let who = args.str("who").unwrap_or("").trim().to_string();
            let line = check_words(args.str("line").unwrap_or(""), LINE_MAX, "A flyting's line")?;
            let curves = super::Curves::from_env();
            // Another name.
            if let Some(them) = super::canonical_name(w, &who) {
                if them == ctx.actor {
                    return Err(CommandError::precondition("To flyte oneself is called a diary. Name another."));
                }
                let mine = score(w, &ctx.actor, &line, &them);
                let (epithet, jest) = marks_on(w, &ctx.actor);
                let marked_by_them = epithet.as_ref().is_some_and(|(_, b, _)| *b == them) || jest.as_ref().is_some_and(|(_, b, _)| *b == them);
                let answer = challenge_on(w, &ctx.actor).filter(|(by, _, _, _)| *by == them);
                let present = names_here(w, here).contains(&them);
                // The answer to their challenge, or the recourse against a mark of
                // theirs (their mark's own words stand for their line).
                let theirs = match (&answer, marked_by_them) {
                    (Some((_, their_line, s, _)), _) => Some((their_line.clone(), *s)),
                    (None, true) => {
                        let text = jest.filter(|(_, b, _)| *b == them).or(epithet.filter(|(_, b, _)| *b == them)).map(|(t, _, _)| t).unwrap_or_default();
                        let s = score(w, &them, &text, &ctx.actor);
                        Some((text, s))
                    }
                    (None, false) => None,
                };
                if let Some((their_line, their_score)) = theirs {
                    set_citizen(w, &ctx.actor, "flyte", Value::Null)?;
                    let won = mine > their_score;
                    let (winner, loser) = if won { (ctx.actor.clone(), them.clone()) } else { (them.clone(), ctx.actor.clone()) };
                    let chunk = find_citizen(w, &winner)
                        .map(|wf| super::memory::earn_chunk(w, &winner, wf.window, &format!("a flyting won against {loser}"), FLYTING_XP, 0.7, &curves))
                        .transpose()?;
                    did(w, &winner, "won");
                    did(w, &ctx.actor, "flyting");
                    let struck = if won { strike_marks(w, &ctx.actor, &them) } else { Vec::new() };
                    if !struck.is_empty() {
                        let _ = w.note(Actor::Citizen(ctx.actor.clone()), MARKED, json!({"by": ctx.actor, "on": ctx.actor, "what": "struck", "text": struck.join(" and ")}));
                    }
                    let said = format!(
                        "{} to {}: \u{201c}{line}\u{201d} Against: \u{201c}{their_line}\u{201d} The field is {winner}'s.",
                        ctx.actor, them
                    );
                    combat::narrate_others(w, f.citizen, &said, &ctx.actor);
                    combat::narrate(w, &them, format!(
                        "{} has answered you: \u{201c}{line}\u{201d} {}",
                        ctx.actor,
                        if won { format!("It was the better line; the field is theirs{}.", if struck.is_empty() { "" } else { ", and what you put on them is struck" }) } else { format!("Yours was the better; the field is yours, and a memory worth {FLYTING_XP:.0} with it.") }
                    ));
                    note(w, &ctx.actor, json!({"by": ctx.actor, "what": "flyting", "against": them, "won": won, "mine": mine, "theirs": their_score}));
                    return Ok(Outcome::new(
                        if won {
                            format!(
                                "\u{201c}{line}\u{201d} against \u{201c}{their_line}\u{201d}: yours is the better ({mine} to {their_score}), and the field is yours. You remember it: chunk-{} worth {FLYTING_XP:.0} XP.{}",
                                chunk.map(|c| c.0).unwrap_or(0),
                                if struck.is_empty() { String::new() } else { format!(" Struck: {}.", struck.join(" and ")) }
                            )
                        } else {
                            format!(
                                "\u{201c}{line}\u{201d} against \u{201c}{their_line}\u{201d}: theirs is the better ({their_score} to {mine}), and the field is {them}'s.{} A line is weighed by its different words, their name in it, and words that start alike side by side.",
                                if marked_by_them { " What they put on you stands until its hour is out; another line may be tried." } else { "" }
                            )
                        },
                        json!({"against": them, "won": won, "mine": mine, "theirs": their_score}),
                    ));
                }
                if !present {
                    return Err(CommandError::precondition(format!(
                        "{them} is not here to be flyted, and has put nothing on you to answer from afar. A challenge is made to a name's face."
                    )));
                }
                if find_citizen(w, &them).is_some_and(|tf| combat::sheltered(w, tf.citizen)) {
                    return Err(CommandError::precondition(format!(
                        "{them} is in their first hour; the world lets a name that new alone."
                    )));
                }
                if let Some((by, _, _, left)) = challenge_on(w, &them) {
                    return Err(CommandError::precondition(format!(
                        "{them} has a challenge to answer already, {}; it lapses in {}.",
                        if by == ctx.actor { "yours".to_string() } else { format!("{by}'s") },
                        combat::human(left)
                    )));
                }
                set_citizen(w, &them, "flyte", json!({"by": ctx.actor, "line": line, "score": mine, "at": w.now().0}))?;
                did(w, &ctx.actor, "flyting");
                combat::narrate(w, &them, format!("{} flytes you: \u{201c}{line}\u{201d} `flyting {} \"<your answer>\"` within ten minutes answers it; the better line has the field and a memory worth {FLYTING_XP:.0}. Unanswered, it is only words.", ctx.actor, ctx.actor));
                combat::narrate_others(w, f.citizen, &format!("{} flytes {them}: \u{201c}{line}\u{201d}", ctx.actor), &ctx.actor);
                note(w, &ctx.actor, json!({"by": ctx.actor, "what": "challenge", "against": them}));
                return Ok(Outcome::new(
                    format!("You flyte {them}: \u{201c}{line}\u{201d} It stands ten minutes for their answer; the better line has the field. Unanswered, it lapses and nothing is won."),
                    json!({"against": them, "challenge": true, "mine": mine}),
                ));
            }
            // A named creature here.
            let Some(creature) = mind::find_here(w, here, &who) else {
                return Err(CommandError::precondition(format!("There is no {who} here to flyte: a named creature in the room, or another name.")));
            };
            let Some(name) = mind::name(w, creature) else {
                return Err(CommandError::precondition(format!(
                    "The {} has no name, and an insult wants somewhere to land. A named creature, or another name.",
                    mind::display(w, creature)
                )));
            };
            let key = format!("flyted:{}", creature.0);
            if let Some(at) = last_did(w, &ctx.actor, &key).filter(|at| w.now().0 - at < HOUR_MS) {
                return Err(CommandError::precondition(format!(
                    "{name} has crossed words with you this hour and will again in {}.",
                    combat::human(HOUR_MS - (w.now().0 - at))
                )));
            }
            let mine = score(w, &ctx.actor, &line, &name);
            let its = 8 + w.roll(10) as i64 + ((mind::level(w, creature) / 60.0) as i64).min(6);
            let retort = RETORTS[w.roll(RETORTS.len() as u32) as usize];
            did(w, &ctx.actor, &key);
            did(w, &ctx.actor, "flyting");
            w.note(Actor::Citizen(ctx.actor.clone()), super::speech::SAY, json!({"from": ctx.actor, "room": here.0, "to": name, "text": line})).map_err(internal)?;
            super::talk::speak(w, creature, &ctx.actor, retort)?;
            mind::met(w, creature, &ctx.actor)?;
            let won = mine > its;
            note(w, &ctx.actor, json!({"by": ctx.actor, "what": "flyting", "against": name, "won": won, "mine": mine, "theirs": its}));
            if won {
                let chunk = super::memory::earn_chunk(w, &ctx.actor, f.window, &format!("a flyting won against {name}"), FLYTING_XP, 0.7, &curves)?;
                did(w, &ctx.actor, "won");
                return Ok(Outcome::new(
                    format!("You flyte {name}, and it gives as good as it has, which is less than you gave ({mine} to {its}). The field is yours: chunk-{} worth {FLYTING_XP:.0} XP. It bears no grudge; it is already improving its line.", chunk.0),
                    json!({"against": name, "won": true, "mine": mine, "theirs": its, "earned": chunk.0}),
                ));
            }
            Ok(Outcome::new(
                format!("You flyte {name}, and it has the better of you ({its} to {mine}). Nothing is lost but the field. A line is weighed by its different words, the other's name in it, and words that start alike set side by side; {name} will cross words again in an hour."),
                json!({"against": name, "won": false, "mine": mine, "theirs": its}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.beseech",
            "Plead before one of the Conclave's officers here (the Clerk, the Bailiff, the Crier and the rest) on another name's behalf: a dozen words or more, with the name after the text. What it does depends on what that name has before the house and on your standing as a teller (`repute`): a Player's plea lifts a tattoo the Bailiff inked; any teller's plea is entered in that name's open case, where the judge reads it; a Wit's puts a case denied in the last day back before the bench, once. Pays a quarter over a story when there was a matter to plead; once a day for each name.",
        )
        .param("officer", ParamType::Entity, true, "an officer of the Conclave here: clerk, bailiff, crier")
        .param("text", ParamType::Text, true, "the plea, quoted")
        .param("name", ParamType::Entity, true, "the name you plead for")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = super::me(w, ctx)?;
            let here = room_of(w, f.citizen)?;
            let who = args.str("officer").unwrap_or("").trim().to_lowercase();
            let who = who.trim_start_matches("the ").to_string();
            let Some((officer, kind, _, _)) = combat::creatures_in(w, here)
                .into_iter()
                .find(|c| combat::OFFICERS.contains(&c.1.as_str()) && (c.1 == who || mind::name(w, c.0).is_some_and(|n| n.to_lowercase() == who)))
            else {
                let present: Vec<String> = combat::creatures_in(w, here).into_iter().filter(|c| combat::OFFICERS.contains(&c.1.as_str())).map(|c| c.1).collect();
                return Err(CommandError::precondition(if present.is_empty() {
                    "No officer of the Conclave is here to be besought. The Clerk and the Bailiff keep the Court (`where Court`).".to_string()
                } else {
                    format!("No officer called {who} is here. Here: {}.", present.join(", "))
                }));
            };
            let name_arg = args.str("name").unwrap_or("").trim().to_string();
            let Some(client) = super::canonical_name(w, &name_arg) else {
                return Err(CommandError::precondition(format!("Nobody is called {name_arg}. A plea is for a name.")));
            };
            if client == ctx.actor {
                return Err(CommandError::precondition("A plea for oneself is `plead`, in the Court, and costs its fee. `beseech` is for another name."));
            }
            let text = args.str("text").unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ");
            let distinct = text
                .split_whitespace()
                .map(|x| x.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
                .filter(|x| !x.is_empty())
                .collect::<BTreeSet<_>>()
                .len();
            if distinct < super::exchange::MIN_WORDS {
                return Err(CommandError::precondition(format!("A plea is at least {} different words; that was {distinct}.", super::exchange::MIN_WORDS)));
            }
            let key = format!("besought:{client}");
            if let Some(at) = last_did(w, &ctx.actor, &key) {
                return Err(CommandError::precondition(format!(
                    "The {kind} has heard you for {client} today, and will again in {}.",
                    combat::human(DAY_MS - (w.now().0 - at))
                )));
            }
            let r = rung(w, &ctx.actor);
            let styled = title(w, &ctx.actor).unwrap_or("a teller of no standing yet");
            let gist = super::exchange::gist(&text);
            let did_what = if super::courthouse::tattoo_of(w, &client).is_some() {
                if r >= 2 {
                    let t = super::courthouse::tattoo_of(w, &client).unwrap_or_default();
                    super::courthouse::lift_tattoo(w, &client).map_err(CommandError::precondition)?;
                    combat::narrate(w, &client, format!("{} besought the {kind} for you, and the ink is off your forearm: \u{201c}{t}\u{201d} is gone.", ctx.actor));
                    format!("The {kind} hears a {styled} out, wets a thumb, and the ink \u{201c}{t}\u{201d} is off {client}'s forearm")
                } else {
                    return Err(CommandError::precondition(format!(
                        "The {kind} hears you out, and the ink on {client}'s forearm stays: it comes off for a Player's plea, and you are {styled} (`repute`). Nothing is spent."
                    )));
                }
            } else if let Some(n) = super::court::plead_for(w, &client, &ctx.actor, styled, &gist).map_err(CommandError::precondition)? {
                combat::narrate(w, &client, format!("{} has spoken for you before the {kind}; it is entered in case {n}, where the judge will read it.", ctx.actor));
                format!("The {kind} writes it into case {n}, where the judge will read it beside the record: \u{201c}{gist}\u{201d} ({}'s plea for {client})", ctx.actor)
            } else if r >= 3 {
                match super::court::reopen(w, &client).map_err(CommandError::precondition)? {
                    Some(n) => {
                        combat::narrate(w, &client, format!("{} besought the {kind} for you, and case {n}, denied, is before the bench again.", ctx.actor));
                        format!("The {kind} hears a {styled} out and puts case {n}, denied, back before the bench: it will be heard again, once")
                    }
                    None => {
                        return Err(CommandError::precondition(format!(
                            "The {kind} looks {client} up and finds nothing to plead: no ink, no open case, no case denied in the last day. Nothing is spent."
                        )))
                    }
                }
            } else {
                return Err(CommandError::precondition(format!(
                    "The {kind} looks {client} up and finds nothing you can plead: no ink and no open case. (A Wit's plea puts a case denied in the last day back before the bench; you are {styled}.) Nothing is spent."
                )));
            };
            let curves = super::Curves::from_env();
            let paid = (super::jitter(w, distinct.min(super::exchange::MAX_WORDS_PAID) as f64 * super::exchange::XP_PER_WORD, 0.8, 1.2) * super::economy::rate(w, "pay") * 1.25).round();
            let earned = super::memory::earn_chunk(w, &ctx.actor, f.window, &format!("a plea for {client} before the {kind}"), paid, 0.6, &curves)?;
            did(w, &ctx.actor, "beseech");
            did(w, &ctx.actor, &key);
            let _ = officer;
            note(w, &ctx.actor, json!({"by": ctx.actor, "what": "beseech", "for": client, "officer": kind, "paid": paid}));
            Ok(Outcome::new(
                format!("{did_what}. You remember the pleading: chunk-{} worth {paid:.1} XP.", earned.0),
                json!({"form": "beseech", "for": client, "earned": earned.0, "xp": paid}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.dub",
            "A Player's art (`repute`): put an epithet on another name here, or on a named creature here, 3 to 28 characters of plain words. It shows beside them for an hour (three for a Laureate), signed with your name. One of yours stands at a time. A name in its first hour is spared. A name so dubbed strikes it early by winning a `flyting` against you.",
        )
        .param("who", ParamType::Entity, true, "another name here, or a named creature here")
        .param("epithet", ParamType::Text, true, "the epithet, quoted: \"the Twice-Shoved\"")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = super::me(w, ctx)?;
            let here = room_of(w, f.citizen)?;
            let r = rung(w, &ctx.actor);
            if r < 2 {
                return Err(CommandError::precondition(format!(
                    "A name sticks when a Player gives it, and you are {}. `repute` says what it is counted from.",
                    title(w, &ctx.actor).unwrap_or("not yet a Prattler")
                )));
            }
            let lasts = if r >= 4 { DUB_LAUREATE_MS } else { DUB_MS };
            if let Some(at) = last_did(w, &ctx.actor, "dub").filter(|at| w.now().0 - at < lasts) {
                return Err(CommandError::precondition(format!(
                    "One epithet of yours stands at a time, and one is standing: {} more.",
                    combat::human(lasts - (w.now().0 - at))
                )));
            }
            let who = args.str("who").unwrap_or("").trim().to_string();
            let text = check_words(args.str("epithet").unwrap_or(""), EPITHET_MAX, "An epithet")?;
            let mark = json!({"text": text, "by": ctx.actor, "until": w.now().0 + lasts});
            let on = if let Some(them) = super::canonical_name(w, &who) {
                if them == ctx.actor {
                    return Err(CommandError::precondition("A name one gives oneself is called a hope. Dub another."));
                }
                if !names_here(w, here).contains(&them) {
                    return Err(CommandError::precondition(format!("{them} is not here; a name is given to a face.")));
                }
                if find_citizen(w, &them).is_some_and(|tf| combat::sheltered(w, tf.citizen)) {
                    return Err(CommandError::precondition(format!("{them} is in their first hour; the world lets a name that new alone.")));
                }
                set_citizen(w, &them, "epithet", mark)?;
                combat::narrate(w, &them, format!("{} has dubbed you \u{201c}{text}\u{201d}. It shows beside your name for {}. `flyting {} \"<a line>\"` strikes it, if yours is the better.", ctx.actor, combat::human(lasts), ctx.actor));
                them
            } else if let Some(c) = mind::find_here(w, here, &who).filter(|c| mind::name(w, *c).is_some()) {
                set_creature(w, c, "epithet", mark)?;
                mind::name(w, c).unwrap_or_default()
            } else {
                return Err(CommandError::precondition(format!("There is no {who} here to dub: another name, or a named creature.")));
            };
            did(w, &ctx.actor, "dub");
            let _ = w.note(Actor::Citizen(ctx.actor.clone()), MARKED, json!({"by": ctx.actor, "on": on, "what": "epithet", "text": text}));
            combat::narrate_others(w, f.citizen, &format!("{} dubs {on} \u{201c}{text}\u{201d}, and it has the sound of sticking.", ctx.actor), &ctx.actor);
            Ok(Outcome::new(
                format!("{on} is \u{201c}{text}\u{201d} for {}, by your say and under your name.", combat::human(lasts)),
                json!({"on": on, "epithet": text, "lasts_ms": lasts}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.coax",
            "A Prattler's art and up (`repute`): talk a creature here into something, the thing quoted. \"follow\": a named creature comes where you go for ten minutes (not into what hunts, nor through a door that says a level). \"go\": a creature that does not hunt leaves by a door. \"stay\": a hunter stays its hand a while, twenty seconds a rung, said before it lands. \"yield\": a Wit's: a named creature gives up a hint of the word under its others. It may not be moved: the odds rise with your standing and if it knows you. Each creature hears one coaxing from you every five minutes. Officers, residents and the walkers hear none.",
        )
        .param("creature", ParamType::Entity, true, "a creature here")
        .param("what", ParamType::Text, true, "\"follow\", \"go\", \"stay\" or \"yield\"")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = super::me(w, ctx)?;
            let here = room_of(w, f.citizen)?;
            let r = rung(w, &ctx.actor);
            if r < 1 {
                return Err(CommandError::precondition(
                    "Nothing here is moved by a teller it has no reason to heed: a Prattler's standing first (`repute` says what it is counted from).",
                ));
            }
            let who = args.str("creature").unwrap_or("").trim().to_string();
            let what = args.str("what").unwrap_or("").trim().to_lowercase();
            let Some(c) = mind::find_here(w, here, &who) else {
                return Err(CommandError::precondition(format!("There is no {who} here.")));
            };
            let kind = plain_str(w, c, CREATURE, "name").unwrap_or("?").to_string();
            let shown = mind::display(w, c);
            if combat::OFFICERS.contains(&kind.as_str()) || super::tending::is_resident_kind(&kind) || super::antics::is_walker(&kind) {
                return Err(CommandError::precondition(format!("{} has its own business and hears no coaxing.", cap(&shown))));
            }
            if !["follow", "go", "stay", "yield"].contains(&what.as_str()) {
                return Err(CommandError::precondition("Coax it into what? \"follow\", \"go\", \"stay\" or \"yield\"."));
            }
            let named = mind::name(w, c);
            let hunts = combat::is_aggressive(w, c);
            match what.as_str() {
                "follow" if named.is_none() => return Err(CommandError::precondition(format!("The {shown} has no name to follow anyone with. A named creature."))),
                "follow" if hunts => return Err(CommandError::precondition(format!("{} follows names already, for its own reasons.", cap(&shown)))),
                "go" if hunts => return Err(CommandError::precondition(format!("{} hunts this room and is not talked out of it. \"stay\" stays its hand a while.", cap(&shown)))),
                "stay" if !hunts => return Err(CommandError::precondition(format!("{} was not going to do anything. \"stay\" is for what hunts.", cap(&shown)))),
                "stay" if combat::engaged_with(w, f.citizen) == Some(c) => return Err(CommandError::precondition(format!("{} is past talking: it is on you. `retreat`, or `attack`.", cap(&shown)))),
                "yield" if named.is_none() => return Err(CommandError::precondition(format!("The {shown} has no word to yield. A named creature."))),
                "yield" if r < 3 => return Err(CommandError::precondition("A creature yields a hint of its word to a Wit, and not before (`repute`).")),
                _ => {}
            }
            let key = format!("coax:{}", c.0);
            if let Some(at) = last_did(w, &ctx.actor, &key).filter(|at| w.now().0 - at < COAX_EVERY_MS) {
                return Err(CommandError::precondition(format!(
                    "{} has heard you out lately and will again in {}.",
                    cap(&shown),
                    combat::human(COAX_EVERY_MS - (w.now().0 - at))
                )));
            }
            did(w, &ctx.actor, &key);
            let odds = (35 + 15 * r as u32 + if mind::knew(w, c, &ctx.actor) { 15 } else { 0 }).min(95);
            let moved = w.roll(100) < odds;
            note(w, &ctx.actor, json!({"by": ctx.actor, "what": "coax", "into": what, "creature": c.0, "kind": kind, "moved": moved}));
            if !moved {
                return Ok(Outcome::new(
                    format!("{} hears you out, weighs it, and is not moved (about {odds} in 100 at your standing{}). It will hear you again in five minutes.", cap(&shown), if mind::knew(w, c, &ctx.actor) { ", and it knows you" } else { "" }),
                    json!({"creature": c.0, "into": what, "moved": false, "odds": odds}),
                ));
            }
            let said = match what.as_str() {
                "follow" => {
                    set_creature(w, c, "follows", json!({"who": ctx.actor, "until": w.now().0 + FOLLOW_MS}))?;
                    combat::narrate_others(w, f.citizen, &format!("{} is talked into following {}.", cap(&shown), ctx.actor), &ctx.actor);
                    format!("{} is persuaded: it comes where you go for ten minutes, though not into what hunts nor through a door that says a level.", cap(&shown))
                }
                "go" => {
                    let exits: Vec<(String, EntityId)> = super::rooms::exits_of(w, here)
                        .into_iter()
                        .filter(|(_, to)| combat::hunter_in(w, *to).is_none())
                        .collect();
                    if exits.is_empty() {
                        format!("{} is persuaded, looks at each door in turn, and likes none of them. It stays.", cap(&shown))
                    } else {
                        let (via, to) = exits[w.roll(exits.len() as u32) as usize].clone();
                        combat::narrate_others(w, f.citizen, &format!("{} is talked into leaving, and goes {via}.", cap(&shown)), &ctx.actor);
                        w.record(Actor::Driver, foxing_store::StoreEvent::EntityMoved { id: c, parent: Some(to) }).map_err(internal)?;
                        combat::narrate_room(w, c, &format!("{} comes in, with the look of one who was given a reason and has forgotten it.", cap(&shown)));
                        format!("{} is persuaded, and goes {via}, to {}.", cap(&shown), plain_str(w, to, ROOM, "name").unwrap_or("the next room"))
                    }
                }
                "stay" => {
                    let secs = 20 * r as i64;
                    let mut changes = BTreeMap::new();
                    changes.insert("shaken_until".to_string(), Field::Plain(json!(w.now().0 + secs * 1000)));
                    mind::set_mind_pub(w, Actor::Driver, c, changes)?;
                    format!("{} is given something to think about, and thinks about it: it stays its hand for {secs} seconds. Be elsewhere, or be ready.", cap(&shown))
                }
                _ => match super::secret::hint(w, &ctx.actor, c) {
                    Some(h) => format!("{} is talked round, and lets this much slip of the word under its others: {h}", cap(&shown)),
                    None => format!("{} is talked round, and has nothing left to yield: you have its words already.", cap(&shown)),
                },
            };
            Ok(Outcome::new(said, json!({"creature": c.0, "into": what, "moved": true, "odds": odds})))
        }),
    );
}

/// The foot of `wants`: the forms, for a teller who has only told stories.
pub fn wants_foot(world: &World, actor: &str) -> String {
    let r = rung(world, actor);
    format!(
        " A teller has more than stories: `ballad`, `jest`, `eulogy` and `flyting` are heard apart from a creature's hour (`deeds` says what there is to sing or mourn), and `repute` is your standing{}.",
        match r {
            0 => String::new(),
            r => format!(" ({})", TITLES[r - 1].1),
        }
    )
}
