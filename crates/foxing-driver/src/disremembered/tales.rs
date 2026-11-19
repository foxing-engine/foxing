//! X16: stories travel and wear.
//!
//! A named creature keeps the stories it is told, in the teller's own words. When its
//! business puts it in a room with another named creature it passes one on, once a
//! day, and the copy the other keeps is shorter and a little wrong: a few words
//! dropped, one swapped for a word of the room. With each hand the teller's name has
//! one chance in two of wearing: first "Gnosis's", then "a story someone told
//! Tinsel", then nobody's. A copy nobody mends is let go after five days, and when
//! the last copy goes the story has left the world (`news` says so, and its teller is
//! told).
//!
//! The teller sets a worn copy right by telling the creature that holds it the story
//! again, close to the first telling: the creature keeps the mended words under the
//! teller's name, and pays plain story pay once for each creature and story. A copy
//! whose name has worn off can be told to its holder by anyone as their own, and
//! travels under that name from then on, until its first teller sets it right.
//!
//! `tales` (a read) lists what the named creatures in the room hold.

use super::{find_citizen, plain_str, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome};
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A story went from one creature to another: `{from, to, room, room_name, tale, hops, whose}`. Public.
pub const PASSED: &str = "disremembered.tale_passed";
/// A worn story was set right by its teller: `{by, creature, name, tale, paid}`. Public.
pub const MENDED: &str = "disremembered.tale_mended";
/// A story with no name on it was told as their own: `{by, creature, name, tale, paid}`. Public.
pub const CLAIMED: &str = "disremembered.tale_claimed";
/// The last copy of a story was let go: `{tale, first, held_by}`. Public.
pub const LOST: &str = "disremembered.tale_lost";

/// Stories a creature keeps to tell; the oldest goes first.
pub const MAX_TALES: usize = 5;
/// A copy nobody mends is let go after this.
pub const TALE_LIFE_MS: i64 = 5 * 86_400_000;
/// A creature passes one story on, and then not again for this long (David: slowly).
pub const PASS_EVERY_MS: i64 = 86_400_000;
/// The teller's name wears one step a hand, one chance in this (David: by hop, with a
/// percentage chance).
pub const NAME_WEARS_ONE_IN: u32 = 2;
/// Under this many words a story is too worn to pass on.
pub const MIN_WORDS: usize = 6;
/// How much of a copy's words a telling must carry to be that story.
const ALIKE: f64 = 0.6;

#[derive(Clone, Debug)]
pub struct Tale {
    /// The same number on every copy of one story.
    pub id: u64,
    /// Who first told it.
    pub by: String,
    /// Whose it is said to be now.
    pub credit: String,
    /// 0: `credit`'s by name; 1: "someone told `first`"; 2: nobody's.
    pub whose: u8,
    /// The creature it was first told to.
    pub first: String,
    pub hops: u32,
    pub text: String,
    /// The words of the first telling, for setting right.
    pub keys: Vec<String>,
    pub at: i64,
    /// Names paid for mending or claiming this copy.
    pub paid: Vec<String>,
}

impl Tale {
    fn json(&self) -> Value {
        json!({"id": self.id, "by": self.by, "credit": self.credit, "whose": self.whose, "first": self.first,
               "hops": self.hops, "text": self.text, "keys": self.keys, "at": self.at, "paid": self.paid})
    }
    fn from(v: &Value) -> Option<Tale> {
        let strs = |k: &str| -> Vec<String> {
            v[k].as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        };
        Some(Tale {
            id: v["id"].as_u64()?,
            by: v["by"].as_str()?.to_string(),
            credit: v["credit"].as_str().unwrap_or("").to_string(),
            whose: v["whose"].as_u64().unwrap_or(0) as u8,
            first: v["first"].as_str().unwrap_or("").to_string(),
            hops: v["hops"].as_u64().unwrap_or(0) as u32,
            text: v["text"].as_str()?.to_string(),
            keys: strs("keys"),
            at: v["at"].as_i64().unwrap_or(0),
            paid: strs("paid"),
        })
    }
    /// "it had from Gnosis", "someone told Tinsel", "nobody says whose".
    fn whose_said(&self) -> String {
        match self.whose {
            0 => format!("{}'s", self.credit),
            1 if !self.first.is_empty() => format!("one someone told {}", self.first),
            1 => "someone's".to_string(),
            _ => "nobody's now".to_string(),
        }
    }
}

pub fn of(world: &World, creature: EntityId) -> Vec<Tale> {
    match world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(super::mind::MIND))
        .and_then(|c| c.fields.get("tales"))
    {
        Some(Field::Plain(Value::Array(a))) => a.iter().filter_map(Tale::from).collect(),
        _ => Vec::new(),
    }
}

/// Whether `creature` has told a story inside the day (never is not lately).
fn told_lately(world: &World, creature: EntityId) -> bool {
    let at = told_at(world, creature);
    at > 0 && world.now().0 - at < PASS_EVERY_MS
}

fn told_at(world: &World, creature: EntityId) -> i64 {
    match world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(super::mind::MIND))
        .and_then(|c| c.fields.get("tale_told_at"))
    {
        Some(Field::Plain(v)) => v.as_i64().unwrap_or(0),
        _ => 0,
    }
}

fn keep(
    world: &mut World,
    creature: EntityId,
    mut tales: Vec<Tale>,
    told: Option<i64>,
) -> Result<(), CommandError> {
    if tales.len() > MAX_TALES {
        tales.sort_by_key(|t| t.at);
        let cut = tales.len() - MAX_TALES;
        tales.drain(..cut);
    }
    let mut changes = BTreeMap::new();
    changes.insert(
        "tales".into(),
        Field::Plain(Value::Array(tales.iter().map(Tale::json).collect())),
    );
    if let Some(t) = told {
        changes.insert("tale_told_at".into(), Field::Plain(json!(t)));
    }
    super::mind::set_mind_pub(world, Actor::Driver, creature, changes)
}

/// For tests: wears the name off every copy `creature` holds, as hops would in time.
#[cfg(test)]
pub fn wear_names(world: &mut World, creature: EntityId, whose: u8) {
    let mut tales = of(world, creature);
    for t in &mut tales {
        t.whose = whose;
    }
    keep(world, creature, tales, None).unwrap();
}

/// How much of `of` is in `told`, as a share of `of`.
fn share(of: &[String], told: &[String]) -> f64 {
    if of.is_empty() {
        return 0.0;
    }
    of.iter().filter(|k| told.contains(k)).count() as f64 / of.len() as f64
}

fn words_of(text: &str) -> usize {
    text.split_whitespace().count()
}

/// A story was told to `creature` and heard: it keeps the words, first hand. A
/// telling like one it already has under the same name takes that one's place.
pub fn heard(
    world: &mut World,
    creature: EntityId,
    actor: &str,
    text: &str,
    id: u64,
) -> Result<String, CommandError> {
    let keys = super::exchange::story_words(text);
    let mut tales = of(world, creature);
    let first = super::mind::name(world, creature).unwrap_or_default();
    let now = world.now().0;
    let said;
    if let Some(t) = tales
        .iter_mut()
        .find(|t| t.credit == actor && t.whose == 0 && share(&t.keys, &keys) >= ALIKE)
    {
        t.text = text.trim().to_string();
        t.at = now;
        t.hops = 0;
        said = " It had this one of yours already, and keeps this telling in its place, for five days more.".to_string();
    } else if let Some(t) = tales.iter().find(|t| {
        t.whose == 0
            && t.credit != actor
            && share(&t.keys, &keys) >= 0.8
            && share(&keys, &t.keys) >= 0.8
    }) {
        // The words of a story it holds under another name: heard, and not kept a
        // second time (one creature held one story three times, as three names').
        return Ok(format!(
            " It has this one already, as {}'s, and keeps that one: a story with a name on it is not taken by telling it.",
            t.credit
        ));
    } else {
        said = " It will keep your words, and may tell them to another named creature: `tales` here says what it holds.".to_string();
        tales.push(Tale {
            id,
            by: actor.to_string(),
            credit: actor.to_string(),
            whose: 0,
            first,
            hops: 0,
            text: text.trim().to_string(),
            keys,
            at: now,
            paid: Vec::new(),
        });
    }
    keep(world, creature, tales, None)?;
    Ok(said)
}

/// Whether `creature` holds a story under `actor`'s name: it knows them by it.
pub fn holds_of(world: &World, creature: EntityId, actor: &str) -> bool {
    of(world, creature)
        .iter()
        .any(|t| t.whose == 0 && t.credit == actor)
}

/// The copy the next creature keeps: a seventh of the words gone (one at least,
/// never the first), and one longer word swapped for a word of the room it is told in.
fn worn(world: &mut World, text: &str, room: EntityId) -> String {
    let mut words: Vec<String> = text.split_whitespace().map(str::to_string).collect();
    let drop = (words.len() / 7).max(1);
    for _ in 0..drop {
        if words.len() <= 2 {
            break;
        }
        let i = 1 + world.roll((words.len() - 1) as u32) as usize;
        words.remove(i);
    }
    let pool: Vec<String> = plain_str(world, room, ROOM, "description")
        .unwrap_or("")
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| w.chars().count() >= 5 && w.chars().all(|c| c.is_lowercase()))
        .map(str::to_string)
        .collect();
    let long: Vec<usize> = words
        .iter()
        .enumerate()
        .filter(|(i, w)| *i > 0 && w.chars().filter(|c| c.is_alphabetic()).count() >= 5)
        .map(|(i, _)| i)
        .collect();
    if !pool.is_empty() && !long.is_empty() {
        let i = long[world.roll(long.len() as u32) as usize];
        let new = pool[world.roll(pool.len() as u32) as usize].clone();
        let tail: String = words[i]
            .chars()
            .rev()
            .take_while(|c| !c.is_alphanumeric())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        words[i] = format!("{new}{tail}");
    }
    // A dropped word can leave its neighbors the same ("in the the room").
    words.dedup_by(|b, a| a.to_lowercase() == b.to_lowercase());
    let mut out = words.join(" ");
    if !out.ends_with(['.', '!', '?', '"']) {
        out = out.trim_end_matches([',', ';', ':']).to_string();
        out.push('.');
    }
    out
}

fn named_here(world: &World, room: EntityId, but: EntityId) -> Vec<EntityId> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "creature" && e.id != but)
        .filter(|e| super::mind::name(world, e.id).is_some())
        .filter(|e| {
            !super::mind::shaken(world, e.id) && super::combat::fought_by(world, e.id).is_none()
        })
        .map(|e| e.id)
        .collect()
}

/// One creature's turn to tell: if another named creature shares its room and it has
/// told nothing for a day, it passes on the newest story the other lacks.
pub fn pass(world: &mut World, c: EntityId) -> Result<bool, String> {
    let now = world.now().0;
    if told_lately(world, c) {
        return Ok(false);
    }
    let Some(here) = world.store.entity(c).and_then(|e| e.parent) else {
        return Ok(false);
    };
    let mine = of(world, c);
    if mine.is_empty() {
        return Ok(false);
    }
    for other in named_here(world, here, c) {
        let theirs = of(world, other);
        let Some(tale) = mine
            .iter()
            .filter(|t| words_of(&t.text) >= MIN_WORDS && !theirs.iter().any(|o| o.id == t.id))
            .max_by_key(|t| t.at)
            .cloned()
        else {
            continue;
        };
        let text = worn(world, &tale.text, here);
        let whose = if tale.whose < 2 && world.roll(NAME_WEARS_ONE_IN) == 0 {
            tale.whose + 1
        } else {
            tale.whose
        };
        let teller = super::mind::display(world, c);
        let hearer = super::mind::display(world, other);
        let said = tale.whose_said();
        let copy = Tale {
            hops: tale.hops + 1,
            whose,
            text: text.clone(),
            at: now,
            paid: Vec::new(),
            ..tale.clone()
        };
        let mut theirs = theirs;
        theirs.push(copy);
        keep(world, other, theirs, None).map_err(|e| e.to_string())?;
        keep(world, c, mine.clone(), Some(now)).map_err(|e| e.to_string())?;
        super::combat::narrate_room(
            world,
            c,
            &format!(
                "{teller} tells {hearer} a story, {said}. {hearer} gives it back to be sure, and has it nearly: \"{text}\""
            ),
        );
        let first: String = tale
            .text
            .split_whitespace()
            .take(6)
            .collect::<Vec<_>>()
            .join(" ");
        let room_name = plain_str(world, here, ROOM, "name")
            .unwrap_or("a room")
            .to_string();
        let _ = world.note(
            Actor::Driver,
            PASSED,
            json!({"from": super::mind::name(world, c), "to": super::mind::name(world, other), "room": here.0,
                   "room_name": room_name, "tale": tale.id, "hops": tale.hops + 1, "whose": whose,
                   "said": said, "first": first}),
        );
        // Its first teller hears of it, wherever they stand: where their story has
        // gone is the start of the chase (nobody could learn a story of theirs was
        // out in the world, the Watch of 11-19).
        let teller_at = find_citizen(world, &tale.by)
            .filter(|f| super::presence::is_here(world, f.citizen))
            .map(|f| super::rooms::room_of(world, f.citizen));
        if teller_at.is_some_and(|r| r != Some(here)) {
            super::combat::narrate(
                world,
                &tale.by,
                format!("Word reaches you: {teller} has told {hearer} a story of yours (\"{first}…\") in {room_name}, and {hearer} has it worn. `tales` there says how; told again to {hearer} as you first told it, it is set right."),
            );
        }
        return Ok(true);
    }
    Ok(false)
}

/// X18: an heir is left one of the old one's stories, worn as a telling wears it,
/// with whatever name it carried.
pub fn bequeath(
    world: &mut World,
    heir: EntityId,
    tale: &Tale,
    room: EntityId,
) -> Result<(), CommandError> {
    let text = worn(world, &tale.text, room);
    let copy = Tale {
        hops: tale.hops + 1,
        text,
        at: world.now().0,
        paid: Vec::new(),
        ..tale.clone()
    };
    let mut theirs = of(world, heir);
    theirs.push(copy);
    keep(world, heir, theirs, None)
}

/// A door from `here` into a room where a named creature stands that lacks a story
/// `c` could tell, for a creature at home with a day's telling unspent.
pub fn door_to_a_listener(
    world: &World,
    c: EntityId,
    here: EntityId,
) -> Option<(String, EntityId)> {
    if told_lately(world, c) {
        return None;
    }
    let mine = of(world, c);
    let tellable: Vec<&Tale> = mine
        .iter()
        .filter(|t| words_of(&t.text) >= MIN_WORDS)
        .collect();
    if tellable.is_empty() {
        return None;
    }
    super::rooms::exits_of(world, here)
        .iter()
        .filter(|(_, to)| !super::plain_bool(world, **to, ROOM, "hidden"))
        .find_map(|(via, to)| {
            named_here(world, *to, c).into_iter().find_map(|o| {
                let theirs = of(world, o);
                tellable
                    .iter()
                    .any(|t| !theirs.iter().any(|x| x.id == t.id))
                    .then(|| (via.clone(), o))
            })
        })
}

/// Lets go of copies past their life. When the last copy of a story goes, the world
/// says so and its first teller is told.
pub fn prune(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    let named: Vec<EntityId> = world
        .store
        .of_kind("creature")
        .filter(|e| super::mind::name(world, e.id).is_some())
        .map(|e| e.id)
        .collect();
    let mut gone: Vec<(Tale, String)> = Vec::new();
    for c in &named {
        let tales = of(world, *c);
        if tales.iter().all(|t| now - t.at < TALE_LIFE_MS) {
            continue;
        }
        let (kept, lost): (Vec<Tale>, Vec<Tale>) =
            tales.into_iter().partition(|t| now - t.at < TALE_LIFE_MS);
        let holder = super::mind::display(world, *c);
        keep(world, *c, kept, None).map_err(|e| e.to_string())?;
        gone.extend(lost.into_iter().map(|t| (t, holder.clone())));
    }
    let mut said: Vec<u64> = Vec::new();
    for (t, holder) in gone {
        // Two copies let go in one sweep are one story gone, said once.
        if said.contains(&t.id) {
            continue;
        }
        said.push(t.id);
        let still = world
            .store
            .of_kind("creature")
            .any(|e| of(world, e.id).iter().any(|o| o.id == t.id));
        if still {
            continue;
        }
        let first: String = t
            .text
            .split_whitespace()
            .take(8)
            .collect::<Vec<_>>()
            .join(" ");
        let _ = world.note(
            Actor::Driver,
            LOST,
            json!({"tale": t.id, "first": first, "held_by": holder}),
        );
        if find_citizen(world, &t.by).is_some() {
            super::combat::narrate(
                world,
                &t.by,
                format!("{holder} has let go of the last copy of a story you told (\"{first}…\"). Nothing in the world holds it now: it is disremembered."),
            );
        }
    }
    Ok(())
}

/// X6: the Court puts a story back under its first teller's name.
pub fn restore(
    world: &mut World,
    creature: EntityId,
    by: &str,
    against: &str,
) -> Result<bool, CommandError> {
    let mut tales = of(world, creature);
    let Some(t) = tales
        .iter_mut()
        .find(|t| t.by == by && t.credit == against && t.whose == 0)
    else {
        return Ok(false);
    };
    t.credit = by.to_string();
    t.at = world.now().0;
    keep(world, creature, tales, None)?;
    Ok(true)
}

/// What a second telling did to a copy the creature held.
pub struct Retold {
    pub line: String,
    /// Whether this name is paid for it (once for each creature and story).
    pub pay: bool,
    pub claimed: bool,
    pub tale: u64,
}

/// `actor` tells `creature` a story it already holds a copy of. The first teller,
/// telling it close to the first telling, sets a worn or renamed copy right. Anyone,
/// telling a copy whose name has worn off close to how the creature has it, makes it
/// theirs. Nothing for a story the creature does not hold, or holds whole under
/// this name already.
pub fn retold(
    world: &mut World,
    actor: &str,
    creature: EntityId,
    text: &str,
) -> Result<Option<Retold>, CommandError> {
    let told = super::exchange::story_words(text);
    let mut tales = of(world, creature);
    let now = world.now().0;
    let name = super::mind::display(world, creature);
    let mend = tales.iter().position(|t| {
        t.by == actor
            && (t.hops > 0 || t.whose > 0 || t.credit != actor)
            && share(&t.keys, &told) >= ALIKE
    });
    let claim = || {
        tales.iter().position(|t| {
            // X6: a story the Court has put back is not taken while that ruling is
            // remembered.
            let guarded = super::mind::name(world, creature)
                .is_some_and(|n| super::court::guards_story(world, &n, &t.by).is_some());
            !guarded
                && t.by != actor
                && t.whose >= 1
                && share(&super::exchange::story_words(&t.text), &told) >= ALIKE
        })
    };
    let (i, claimed) = match mend {
        Some(i) => (i, false),
        None => match claim() {
            Some(i) => (i, true),
            None => return Ok(None),
        },
    };
    let t = &mut tales[i];
    let was = t.whose_said();
    let hops = t.hops;
    let from = t.credit.clone();
    let renamed = t.whose == 0 && t.credit != actor;
    t.text = text.trim().to_string();
    t.hops = 0;
    t.whose = 0;
    t.credit = actor.to_string();
    t.at = now;
    let pay = !t.paid.iter().any(|p| p == actor);
    if pay {
        t.paid.push(actor.to_string());
    }
    let tale = t.id;
    keep(world, creature, tales, None)?;
    let line = if claimed {
        format!(
            "{name} knows this one: it had it as {was}, {hops} hand{} on. It listens to you tell it as yours, and from now it is: {name} will pass it on as {actor}'s, until whoever told it first says otherwise.",
            if hops == 1 { "" } else { "s" }
        )
    } else if renamed {
        format!(
            "{name} knows this one, and had been telling it as {from}'s. You tell it as you first told it, and {name} has it right again, and as yours."
        )
    } else {
        format!(
            "{name} knows this one: it had it {hops} hand{} on, as {was}, and with words gone. You tell it as you first told it, and {name} has it right again, and as yours.",
            if hops == 1 { "" } else { "s" }
        )
    };
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        if claimed { CLAIMED } else { MENDED },
        json!({"by": actor, "creature": creature.0, "name": super::mind::name(world, creature), "tale": tale, "paid": pay}),
    );
    Ok(Some(Retold {
        line,
        pay,
        claimed,
        tale,
    }))
}

/// `tales`: what the named creatures in the room hold to tell.
fn list(world: &World, actor: &str) -> Result<Outcome, CommandError> {
    let here = find_citizen(world, actor)
        .and_then(|f| super::rooms::room_of(world, f.citizen))
        .ok_or_else(|| CommandError::precondition("You are nowhere just now.".to_string()))?;
    let now = world.now().0;
    let mut lines = Vec::new();
    let mut data = Vec::new();
    for e in world.store.children(here).filter(|e| e.kind == "creature") {
        let Some(name) = super::mind::name(world, e.id) else {
            continue;
        };
        let kind = plain_str(world, e.id, CREATURE, "name").unwrap_or("creature");
        let mut tales = of(world, e.id);
        tales.sort_by_key(|t| std::cmp::Reverse(t.at));
        if tales.is_empty() {
            lines.push(format!("{name} the {kind} holds no story it could tell."));
            continue;
        }
        let each: Vec<String> = tales
            .iter()
            .map(|t| {
                let hands = match t.hops {
                    0 => "first hand".to_string(),
                    1 => "one hand on".to_string(),
                    n => format!("{n} hands on"),
                };
                let left = (TALE_LIFE_MS - (now - t.at)).max(0);
                format!(
                    "{}, {hands}, kept {} more: \"{}\"",
                    t.whose_said(),
                    super::combat::human(left),
                    t.text
                )
            })
            .collect();
        data.push(json!({"creature": e.id.0, "name": name, "tales": tales.iter().map(|t| json!({"tale": t.id, "hops": t.hops, "whose": t.whose, "credit": if t.whose == 0 { json!(t.credit) } else { Value::Null }, "text": t.text})).collect::<Vec<_>>()}));
        lines.push(format!(
            "{name} the {kind} holds {}: {}",
            each.len(),
            each.join("; ")
        ));
    }
    if lines.is_empty() {
        return Ok(Outcome::new(
            "No named creature stands here. A named creature keeps the stories it is told and passes one on, a day apart, to another it shares a room with; `tales` in that room says what each holds.",
            json!({"tales": []}),
        ));
    }
    lines.push("A story passed on loses words, and with each hand its teller's name may wear off. Yours, worn: tell it again to the creature that holds it, as you first told it, and it is set right and paid once. One with no teller's name on it (\"one someone told…\", \"nobody's now\"): tell it to its holder as it stands and it is yours, until its first teller sets it right. One with a name on it is not taken by telling it. A creature tells one story a day, the newest it holds that the other lacks.".to_string());
    Ok(Outcome::new(lines.join("\n"), json!({"tales": data})))
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.tales",
            "What the named creatures in this room hold to tell: each story in the words it has now, whose it is said to be, how many hands it has come through, and how long it will be kept. A creature passes one on, a day apart, to another it shares a room with; the copy is shorter and a little wrong.",
        )
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, _args: &Args| list(w, &ctx.actor)),
    );
}
