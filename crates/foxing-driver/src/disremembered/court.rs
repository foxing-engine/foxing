//! X6: the Court of Lethe, the judicial branch. The Conclave writes, the Clocktower
//! moves, the Court decides between names.
//!
//! David's brief (11-18): four kinds of case, each one the world's own record can
//! check (free pleas wait for the right item or quest); a model judges, and is
//! steered to send petitioners to the Conclave often; a judgment puts right and fines
//! nobody (a restored name on a story, a thing returned, a line struck from a room or
//! a book, a name taken off an object, a name off the Roll), and both names gain a
//! little from seeing the process through; a plea costs an errand and a nominal fee;
//! and the Court forgets its rulings on the world's curve.
//!
//! The course of a case:
//! - `plead <kind> <against> "<what>"`, by a kept name standing in the Court. The
//!   Clerk takes the least memory the name holds as the fee and lodges the plea.
//! - The errand: the plea is carried to the Unfiled Archive for the record's stamp,
//!   and back. Arriving does it.
//! - Entered, the other name is told wherever they stand and has a day to `defend`.
//! - Heard: the judge (the Docent's model, by `GET /v0/docent` and the operator's
//!   `judge`) has the plea, the answer and what the record shows, and grants, denies,
//!   or refers the petitioner to the Conclave. Referred, the petitioner lays a
//!   petition at the Gallery; when the Conclave has decided it, the case comes back
//!   and cannot be referred again. Six hours unheard, the record decides.
//! - Decided: the remedy is done, both names are told and keep a memory, and the
//!   ruling stands on the Docket. A ruling holds while it is remembered: `docket <n>`
//!   reads one, and a reading keeps it two weeks more. Forgotten, it binds nobody and
//!   the matter can be brought afresh.

use super::{find_citizen, plain_str, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const COURT: &str = "disremembered.court";
/// The Court's room.
pub const ROOM_KEY: &str = "court-of-lethe";
/// A case moved a stage: `{case, stage, by, against, kind}`. Public.
pub const CASE: &str = "disremembered.case";
/// A case was decided: `{case, kind, by, against, verdict, did}`. Public.
pub const RULED: &str = "disremembered.ruling";
/// A ruling nobody read was forgotten: `{case}`. Public.
pub const FORGOTTEN: &str = "disremembered.ruling_forgotten";

/// The other name's time to answer.
pub const ANSWER_MS: i64 = 86_400_000;
/// Unheard this long, the record decides.
pub const UNHEARD_MS: i64 = 6 * 3_600_000;
/// A ruling unread this long is forgotten.
pub const RULING_LIFE_MS: i64 = 14 * 86_400_000;
/// What seeing a case through is worth to the name that brought it, and to the name
/// that answered.
/// The job that reads a sitting out, a line at a time.
pub const JOB_SITTING: &str = "disremembered.sitting";
pub const SEEN_THROUGH_XP: f64 = 12.0;
pub const ANSWERED_XP: f64 = 6.0;

pub const KINDS: &[(&str, &str)] = &[
    ("story", "a story of yours another name tells as their own; <what> is the creature that holds it"),
    ("line", "a line another name wrote in a room or a book of yours; <what> is some of its words"),
    ("name", "a name another gave to an object you made, or that stands in a room of yours; <what> is the name"),
    ("thing", "a thing left by a creature you felled and taken by another name; <what> is the thing"),
    ("roll", "your name on the Roll of Hoards; <against> is conclave, <what> is why it should come off"),
];

fn entity(world: &World) -> Option<EntityId> {
    world.store.of_kind("court").map(|e| e.id).next()
}

pub fn cases(world: &World) -> Vec<Value> {
    match entity(world)
        .and_then(|id| world.store.entity(id))
        .and_then(|e| e.components.get(COURT))
        .and_then(|c| c.fields.get("cases"))
    {
        Some(Field::Plain(Value::Array(a))) => a.clone(),
        _ => Vec::new(),
    }
}

fn next_number(world: &World) -> u64 {
    match entity(world)
        .and_then(|id| world.store.entity(id))
        .and_then(|e| e.components.get(COURT))
        .and_then(|c| c.fields.get("next"))
    {
        Some(Field::Plain(v)) => v.as_u64().unwrap_or(1),
        _ => 1,
    }
}

fn write(world: &mut World, cases: Vec<Value>, next: Option<u64>) -> Result<(), String> {
    let id = match entity(world) {
        Some(id) => id,
        None => {
            let id = world.store.allocate_id();
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityCreated {
                        id,
                        kind: "court".into(),
                        owner: None,
                        parent: None,
                        scopes: vec![],
                    },
                )
                .map_err(|e| e.to_string())?;
            id
        }
    };
    let mut fields = world
        .store
        .entity(id)
        .and_then(|e| e.components.get(COURT))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    fields.insert("cases".into(), Field::Plain(Value::Array(cases)));
    if let Some(n) = next {
        fields.insert("next".into(), Field::Plain(json!(n)));
    }
    fields
        .entry("next".into())
        .or_insert(Field::Plain(json!(1)));
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: COURT.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Tests hold the Court still: no mishaps, and the window always open.
pub fn calm(world: &World) -> bool {
    entity(world)
        .and_then(|id| world.store.entity(id))
        .and_then(|e| e.components.get(COURT))
        .and_then(|c| c.fields.get("calm"))
        .is_some_and(|f| matches!(f, Field::Plain(Value::Bool(true))))
}

#[cfg(test)]
pub fn calm_for_test(world: &mut World) {
    write(world, cases(world), None).unwrap();
    let id = entity(world).unwrap();
    let mut fields = world
        .store
        .entity(id)
        .unwrap()
        .components
        .get(COURT)
        .unwrap()
        .fields
        .clone();
    fields.insert("calm".into(), Field::Plain(json!(true)));
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: COURT.into(),
                fields,
            },
        )
        .unwrap();
}

fn at(world: &World, actor: &str, key: &str) -> bool {
    find_citizen(world, actor)
        .and_then(|f| super::rooms::room_of(world, f.citizen))
        .is_some_and(|r| plain_str(world, r, ROOM, "key") == Some(key))
}

fn s<'a>(c: &'a Value, k: &str) -> &'a str {
    c[k].as_str().unwrap_or("")
}

fn open(c: &Value) -> bool {
    s(c, "stage") != "decided"
}

fn tell(world: &mut World, who: &str, text: String) {
    if find_citizen(world, who).is_some() {
        super::combat::narrate(world, who, text);
    }
}

fn move_stage(world: &mut World, c: &mut Value, stage: &str) {
    c["stage"] = json!(stage);
    c[format!("{stage}_at")] = json!(world.now().0);
    let _ = world.note(
        Actor::Driver,
        CASE,
        json!({"case": c["n"], "stage": stage, "by": c["by"], "against": c["against"], "kind": c["kind"]}),
    );
}

#[allow(dead_code)]
fn in_court(world: &World, actor: &str) -> Option<EntityId> {
    let f = find_citizen(world, actor)?;
    let room = super::rooms::room_of(world, f.citizen)?;
    (plain_str(world, room, ROOM, "key") == Some(ROOM_KEY)).then_some(room)
}

fn owner_of(world: &World, room: EntityId) -> Option<String> {
    world.store.entity(room).and_then(|e| e.owner.clone())
}

/// A book or object the plaintiff has a say over: made by them, or in a room of theirs.
fn theirs(world: &World, item: EntityId, by: &str) -> bool {
    plain_str(world, item, super::craft::ITEM, "made_by") == Some(by)
        || world
            .store
            .entity(item)
            .and_then(|e| e.parent)
            .and_then(|r| owner_of(world, r))
            .as_deref()
            == Some(by)
}

/// What the world's own record shows of a case: whether it bears the plea out, and
/// in what words. The judge reads this; the remedy uses the same finding.
pub fn record(world: &World, c: &Value) -> (bool, String) {
    let (by, against, what) = (s(c, "by"), s(c, "against"), s(c, "what"));
    let low = what.to_lowercase();
    match s(c, "kind") {
        "story" => match super::mind::find_named(world, what) {
            None => (false, format!("No creature answers to {what} now.")),
            Some((creature, _)) => {
                let tales = super::tales::of(world, creature);
                match tales.iter().find(|t| t.by == by && t.credit == against && t.whose == 0) {
                    Some(t) => (
                        true,
                        format!(
                            "{what} holds a story first told by {by}, and tells it as {against}'s: \"{}\"",
                            t.text
                        ),
                    ),
                    None if tales.iter().any(|t| t.by == by) => (
                        false,
                        format!("{what} holds a story first told by {by}, and not under {against}'s name."),
                    ),
                    None => (false, format!("{what} holds no story first told by {by}.")),
                }
            }
        },
        "line" => match find_line(world, by, against, &low) {
            Some(Line::Book(item, line)) => (
                true,
                format!(
                    "In {}, which is {by}'s to answer for, a line signed {against}: \"{line}\"",
                    super::craft::display(world, item)
                ),
            ),
            Some(Line::Room(room, added)) => (
                true,
                format!(
                    "In {}, a room of {by}'s, {against} added: \"{added}\"",
                    plain_str(world, room, ROOM, "name").unwrap_or("a room")
                ),
            ),
            None => (
                false,
                format!("No line by {against} with those words stands in a room or a book of {by}'s."),
            ),
        },
        "name" => match find_named_object(world, by, against, &low) {
            Some(item) => (
                true,
                format!(
                    "An object that is {by}'s to answer for was named \"{}\" by {against}.",
                    plain_str(world, item, super::craft::ITEM, "given_name").unwrap_or(what)
                ),
            ),
            None => (
                false,
                format!("No object of {by}'s carries that name by {against}'s hand."),
            ),
        },
        "thing" => match find_taken(world, by, against, &low) {
            Some((name, room, ago)) => (
                true,
                format!(
                    "{by} felled a creature in {room}, and {against} took {name} from there within two minutes of it, {} ago.",
                    super::combat::human(ago)
                ),
            ),
            None => (
                false,
                format!("The record shows no such thing taken by {against} from where {by} had just felled a creature (it keeps a day or so of such things)."),
            ),
        },
        "roll" => match super::conclave::hoard_share_of(world, by) {
            Some(share) => (
                true,
                format!("{by} stands on the Roll of Hoards, entered at {:.0}% of all that is held.", share * 100.0),
            ),
            None => (false, format!("{by} is not on the Roll of Hoards.")),
        },
        _ => (false, "The Court does not know the kind of case.".to_string()),
    }
}

enum Line {
    Book(EntityId, String),
    Room(EntityId, String),
}

fn find_line(world: &World, by: &str, against: &str, low: &str) -> Option<Line> {
    let sig = format!("\u{2014} {against}");
    for e in world.store.of_kind("item") {
        if !theirs(world, e.id, by) {
            continue;
        }
        let text = plain_str(world, e.id, super::craft::ITEM, "text").unwrap_or("");
        if let Some(line) = text
            .split(" | ")
            .find(|l| l.trim_end().ends_with(&sig) && l.to_lowercase().contains(low))
        {
            return Some(Line::Book(e.id, line.trim().to_string()));
        }
    }
    world.recent_events(4000).into_iter().rev().find_map(|e| {
        if e.kind != super::building::APPENDED
            || e.payload["by"] != against
            || e.payload["kind"] == "book"
        {
            return None;
        }
        let room = EntityId(
            e.payload["room"]
                .as_u64()
                .or_else(|| e.payload["target"].as_u64())?,
        );
        if owner_of(world, room).as_deref() != Some(by) {
            return None;
        }
        let described = plain_str(world, room, ROOM, "description")?;
        match e.payload["added"].as_str() {
            Some(added) => (added.to_lowercase().contains(low) && described.contains(added))
                .then(|| Line::Room(room, added.to_string())),
            // An append from before the event carried its words: the sentence of
            // the room that holds the words complained of, the other name having
            // written in that room.
            None => described
                .split_inclusive(['.', '!', '?'])
                .map(str::trim)
                .find(|sentence| sentence.to_lowercase().contains(low))
                .map(|sentence| Line::Room(room, sentence.to_string())),
        }
    })
}

fn find_named_object(world: &World, by: &str, against: &str, low: &str) -> Option<EntityId> {
    world.recent_events(4000).into_iter().rev().find_map(|e| {
        if e.kind != super::craft::NAMED || e.payload["by"] != against {
            return None;
        }
        let item = EntityId(e.payload["item"].as_u64()?);
        let given = plain_str(world, item, super::craft::ITEM, "given_name")?;
        (given.to_lowercase() == *low && theirs(world, item, by)).then_some(item)
    })
}

fn find_taken(world: &World, by: &str, against: &str, low: &str) -> Option<(String, String, i64)> {
    let events = world.recent_events(4000);
    events.iter().rev().find_map(|t| {
        if t.kind != super::craft::TOOK || t.payload["by"] != against {
            return None;
        }
        let name = t.payload["name"].as_str()?;
        if !name.to_lowercase().contains(low) {
            return None;
        }
        let felled = events.iter().any(|f| {
            f.kind == super::combat::FELLED
                && f.payload["by"] == by
                && f.payload["room"] == t.payload["room"]
                && t.at.0 >= f.at.0
                && t.at.0 - f.at.0 <= 120_000
        });
        let room = t.payload["room"]
            .as_u64()
            .and_then(|r| plain_str(world, EntityId(r), ROOM, "name"))
            .unwrap_or("a room")
            .to_string();
        felled.then(|| (name.to_string(), room, world.now().0 - t.at.0))
    })
}

/// Puts right what a granted case asks, and says what was done.
fn remedy(world: &mut World, c: &Value) -> String {
    let (by, against, what) = (
        s(c, "by").to_string(),
        s(c, "against").to_string(),
        s(c, "what").to_string(),
    );
    let low = what.to_lowercase();
    match s(c, "kind") {
        "story" => match super::mind::find_named(world, &what) {
            Some((creature, _)) => match super::tales::restore(world, creature, &by, &against) {
                Ok(true) => format!("{what} tells the story as {by}'s again, and while this ruling is remembered nobody takes it by telling it."),
                _ => "The story is no longer told under the other name: there was nothing left to put right.".to_string(),
            },
            None => "The creature that held the story is gone: there was nothing left to put right.".to_string(),
        },
        "line" => match find_line(world, &by, &against, &low) {
            Some(Line::Book(item, line)) => {
                let text = plain_str(world, item, super::craft::ITEM, "text").unwrap_or("").to_string();
                let kept: Vec<&str> = text.split(" | ").filter(|l| l.trim() != line).collect();
                let mut changes = BTreeMap::new();
                changes.insert("text".into(), Field::Plain(json!(kept.join(" | "))));
                let name = super::craft::display(world, item);
                match super::craft::set_item_fields(world, Actor::Driver, item, changes) {
                    Ok(()) => format!("The line is struck from {name}."),
                    Err(_) => "The line could not be struck.".to_string(),
                }
            }
            Some(Line::Room(room, added)) => {
                let d = plain_str(world, room, ROOM, "description").unwrap_or("").to_string();
                let d = d.replace(&added, "").split_whitespace().collect::<Vec<_>>().join(" ");
                let mut changes = BTreeMap::new();
                changes.insert("description".into(), Field::Plain(json!(d)));
                let name = plain_str(world, room, ROOM, "name").unwrap_or("the room").to_string();
                match super::bindery::set_fields(world, Actor::Driver, room, ROOM, changes) {
                    Ok(()) => format!("The words are struck from {name}."),
                    Err(_) => "The words could not be struck.".to_string(),
                }
            }
            None => "The line is no longer there: there was nothing left to put right.".to_string(),
        },
        "name" => match find_named_object(world, &by, &against, &low) {
            Some(item) => {
                let mut changes = BTreeMap::new();
                changes.insert("given_name".into(), Field::Plain(Value::Null));
                match super::craft::set_item_fields(world, Actor::Driver, item, changes) {
                    Ok(()) => format!("The name \"{what}\" is taken off the object, which is {} again.", super::craft::display(world, item)),
                    Err(_) => "The name could not be taken off.".to_string(),
                }
            }
            None => "The object no longer carries the name: there was nothing left to put right.".to_string(),
        },
        "thing" => {
            let held = find_citizen(world, &against).and_then(|f| {
                world
                    .store
                    .children(f.citizen)
                    .find(|e| e.kind == "satchel")
                    .map(|e| e.id)
            });
            let item = held.and_then(|sat| {
                world
                    .store
                    .children(sat)
                    .find(|e| {
                        plain_str(world, e.id, super::craft::ITEM, "name")
                            .is_some_and(|n| n.to_lowercase().contains(&low))
                    })
                    .map(|e| e.id)
            });
            let to = find_citizen(world, &by).map(|f| f.citizen);
            match (item, to) {
                (Some(item), Some(citizen)) => match super::craft::satchel(world, &by, citizen) {
                    Ok(sat) => {
                        let moved = world.record(
                            Actor::Driver,
                            StoreEvent::EntityMoved { id: item, parent: Some(sat) },
                        );
                        if moved.is_ok() {
                            format!("The {} is taken from {against}'s satchel and put in {by}'s.", what.trim_start_matches("a ").trim_start_matches("an "))
                        } else {
                            "The thing could not be moved.".to_string()
                        }
                    }
                    Err(_) => "The thing could not be moved.".to_string(),
                },
                _ => format!("{against} no longer carries it (used, or dropped, or gone): the Court can say whose it was, and cannot hand it back."),
            }
        }
        "roll" => match super::conclave::strike_from_roll(world, &by) {
            Ok(true) => format!("{by}'s name is rubbed off the Roll of Hoards, until the Conclave next sits and counts again."),
            _ => "The name is no longer on the Roll: there was nothing left to put right.".to_string(),
        },
        _ => String::new(),
    }
}

/// Whether a remembered ruling stands over a story this creature holds for `by`: a
/// restored story is not taken by telling it while the ruling is remembered.
pub fn guards_story(world: &World, creature_name: &str, by: &str) -> Option<u64> {
    cases(world)
        .iter()
        .find(|c| {
            s(c, "stage") == "decided"
                && s(c, "kind") == "story"
                && s(c, "verdict") == "granted"
                && s(c, "by") == by
                && s(c, "what").eq_ignore_ascii_case(creature_name)
        })
        .and_then(|c| c["n"].as_u64())
}

fn decide(world: &mut World, c: &mut Value, verdict: &str, opinion: &str, judge: &str) {
    let did = if verdict == "granted" {
        remedy(world, c)
    } else {
        String::new()
    };
    c["verdict"] = json!(verdict);
    c["opinion"] = json!(opinion);
    c["judge"] = json!(judge);
    c["did"] = json!(did);
    c["read_at"] = json!(world.now().0);
    move_stage(world, c, "decided");
    let (n, by, against) = (
        c["n"].clone(),
        s(c, "by").to_string(),
        s(c, "against").to_string(),
    );
    let said = format!(
        "The Court of Lethe has decided case {n}, {by} against {against}: {}. {opinion}{}",
        if verdict == "granted" {
            "granted"
        } else {
            "denied"
        },
        if did.is_empty() {
            String::new()
        } else {
            format!(" {did}")
        }
    );
    let curves = super::Curves::from_env();
    for (who, worth, what) in [
        (
            by.clone(),
            SEEN_THROUGH_XP,
            format!("seeing case {n} through the Court of Lethe"),
        ),
        (
            against.clone(),
            ANSWERED_XP,
            format!("answering case {n} before the Court of Lethe"),
        ),
    ] {
        let answered = who == by || !c["answer"].is_null();
        if let Some(f) = find_citizen(world, &who) {
            let kept = if answered {
                super::memory::earn_chunk(world, &who, f.window, &what, worth, 0.6, &curves)
                    .ok()
                    .map(|id| format!(" You remember it: chunk-{} worth {worth:.0} XP.", id.0))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            // A party in the Courtroom has just heard the opinion read out: the
            // word of it does not say it all again (the first case's petitioner
            // got the ruling twice in one breath).
            let said = if at(world, &who, ROOM_KEY) {
                format!(
                    "Case {n}, {by} against {against}, is decided: {verdict}.{}",
                    if did.is_empty() {
                        String::new()
                    } else {
                        format!(" {did}")
                    }
                )
            } else {
                said.clone()
            };
            super::combat::narrate(world, &who, format!("{said}{kept} (`docket {n}` reads the ruling from anywhere, and a ruling read is a ruling kept.)"));
        }
    }
    world.notice(format!(
        "**The Court of Lethe** has decided case {n}, {by} against {against}: {verdict}."
    ));
    let _ = world.note(
        Actor::Driver,
        RULED,
        json!({"case": n, "kind": c["kind"], "by": by, "against": against, "verdict": verdict, "did": did}),
    );
}

/// The bench has its ruling: the Court sits to say it. A bell in the Hall and the
/// Lobby, whoever is near comes in, and the sitting is read out a line at a time;
/// the ruling takes effect when the gavel falls.
fn convene(world: &mut World, c: &mut Value, verdict: &str, opinion: &str, judge: &str) {
    use super::courthouse as house;
    c["ruling"] = json!({"verdict": verdict, "opinion": opinion, "judge": judge});
    let drawn = house::draw_in(world);
    let names: Vec<String> = drawn
        .iter()
        .filter_map(|d| d["name"].as_str().map(str::to_string))
        .collect();
    c["drawn"] = json!(drawn);
    c["script"] = json!(house::script(world, c, &names));
    c["step"] = json!(0);
    c["murmurs"] = json!([]);
    move_stage(world, c, "sitting");
    let (n, by, against) = (
        c["n"].clone(),
        s(c, "by").to_string(),
        s(c, "against").to_string(),
    );
    let bell = format!("A handbell is rung twice, somewhere behind the Court. The Crier of the List, who has been waiting all day to: \"CASE {n}! {by} AGAINST {against}! ALL WHO HAVE BUSINESS BEFORE THE COURT OF LETHE, AND ALL WHO HAVE NONE, DRAW NEAR!\" (The Court is sitting now, in the Courtroom off the Hall of Lost Steps. A sitting is worth hearing, and those who stay to its end remember it.)");
    for key in [
        house::HALL,
        house::WINDOW,
        house::CELLS,
        house::STAIR,
        super::rooms::POSTERN,
        "lamplighters-lane",
    ] {
        house::say_in(world, key, &bell);
    }
    house::say_in(world, ROOM_KEY, &format!("A handbell is rung twice behind the bench, and from the Hall the Crier can be heard doing what it has waited all day to do. Case {n} is called: {by} against {against}. The gallery fills, somehow, though you did not see anyone come in."));
    for who in [by, against] {
        if !at(world, &who, ROOM_KEY) {
            tell(world, &who, format!("Word from the Court of Lethe: it is sitting on case {n} now. You need not be there. It is better heard than told."));
        }
    }
    world.notice(format!("**The Court of Lethe** is sitting on case {n}."));
    let due = world
        .now()
        .plus(foxing_time::Duration::seconds(super::courthouse::BELL_S));
    let _ = world.schedule_once(JOB_SITTING, due, json!({"case": n}), |p| p["case"] == n);
}

/// The next line of a sitting, read to the Courtroom; after the last, the ruling.
fn read_on(world: &mut World, n: u64) -> Result<(), String> {
    let mut all = cases(world);
    let Some(c) = all
        .iter_mut()
        .find(|c| c["n"].as_u64() == Some(n) && s(c, "stage") == "sitting")
    else {
        return Ok(());
    };
    let step = c["step"].as_u64().unwrap_or(0) as usize;
    let line = c["script"].as_array().and_then(|a| a.get(step)).cloned();
    match line {
        Some(l) => {
            if step == 3 || step == 7 {
                super::courthouse::say_in(
                    world,
                    super::courthouse::HALL,
                    &format!("Through the arch marked COURT: a voice in capitals, a murmur, the Bailiff saying nothing very loudly. The Court is still sitting on case {n}, and there is room in the gallery."),
                );
            }
            super::courthouse::say_in(world, ROOM_KEY, l["t"].as_str().unwrap_or(""));
            c["step"] = json!(step + 1);
            let wait = l["w"].as_i64().unwrap_or(super::courthouse::LINE_S).max(1);
            write(world, all, None)?;
            let due = world.now().plus(foxing_time::Duration::seconds(wait));
            world
                .schedule_once(JOB_SITTING, due, json!({"case": n}), |p| p["case"] == n)
                .map(|_| ())
        }
        None => {
            conclude(world, c);
            write(world, all, None)
        }
    }
}

/// The gavel: the ruling takes effect, the gallery is paid, and whoever came in to
/// listen goes home.
fn conclude(world: &mut World, c: &mut Value) {
    let ruling = c["ruling"].clone();
    let (verdict, opinion, judge) = (
        ruling["verdict"].as_str().unwrap_or("denied").to_string(),
        ruling["opinion"].as_str().unwrap_or("").to_string(),
        ruling["judge"].as_str().unwrap_or("the bench").to_string(),
    );
    let (n, by, against) = (
        c["n"].clone(),
        s(c, "by").to_string(),
        s(c, "against").to_string(),
    );
    let murmurs = c["murmurs"].as_array().map_or(0, Vec::len);
    super::courthouse::say_in(
        world,
        ROOM_KEY,
        &format!(
            "The Clerk enters {murmurs} murmur{} in the record of case {n}, which is {murmurs} more than the record needed. The Bailiff: \"All rise.\" Nobody is on the far side of the water when you look. {} The Court has risen.",
            if murmurs == 1 { "" } else { "s" },
            match verdict.as_str() {
                "granted" => format!("The Bailiff walks to where {against} would stand, and stands there a moment, and comes back."),
                "referred" => "The Bailiff carries the two halves of the plea to the door, one in each hand, like a thing that might rejoin.".to_string(),
                _ => format!("The Bailiff looks at the door until {by} has thought about it."),
            }
        ),
    );
    let drawn = c["drawn"].clone();
    for k in ["script", "step", "drawn", "ruling"] {
        if let Some(o) = c.as_object_mut() {
            o.remove(k);
        }
    }
    c["murmurs"] = json!(murmurs);
    // The gallery: any name in the Courtroom at the end that is no party to it.
    if let Some(court) = super::courthouse::room_keyed(world, ROOM_KEY) {
        let curves = super::Curves::from_env();
        for who in super::courthouse::names_in(world, court) {
            if who == by || who == against {
                continue;
            }
            if let Some(f) = find_citizen(world, &who) {
                if let Ok(id) = super::memory::earn_chunk(
                    world,
                    &who,
                    f.window,
                    &format!("sitting through case {n} in the gallery of the Court of Lethe"),
                    super::courthouse::GALLERY_XP,
                    0.6,
                    &curves,
                ) {
                    super::combat::narrate(
                        world,
                        &who,
                        format!(
                            "You sat through it. You remember that: chunk-{} worth {:.0} XP.",
                            id.0,
                            super::courthouse::GALLERY_XP
                        ),
                    );
                }
            }
        }
    }
    super::courthouse::send_home(world, &drawn);
    if verdict == "referred" {
        c["opinion"] = json!(opinion);
        move_stage(world, c, "referred");
        let slip = super::courthouse::paper(
            world,
            &by,
            "a referral slip",
            &format!("COURT OF LETHE. Case {n} is REFERRED to the Conclave of Lethe. The bearer lays a petition in the matter at the Petitioners' Gallery. When the Conclave has decided it, either way, the case returns, and may not be sent again. Torn from a larger sheet; the tear is very straight."),
        );
        tell(
            world,
            &by,
            format!("The Court of Lethe has heard case {n} and will not decide it yet: {opinion} The case is referred to the Conclave.{} Lay a petition in the matter at the Petitioners' Gallery (petition \"…\"); when the Conclave has decided it, either way, the case comes back to the Court, which then must decide.",
                match slip { Some(i) => format!(" A referral slip finds its way into your satchel (item-{}).", i.0), None => String::new() }),
        );
    } else {
        decide(world, c, &verdict, &opinion, &judge);
        let _ = super::courthouse::paper(
            world,
            &by,
            "a stamped ruling",
            &format!("COURT OF LETHE. Case {n}, {by} against {against}: {}. {opinion} Stamped, twice, the second time straighter. This copy is the petitioner's. The Court's copy is the Docket, and lasts as long as it is read.", verdict.to_uppercase()),
        );
    }
}

/// Tests: read whatever is left of a sitting at once.
#[cfg(test)]
pub fn conclude_now(world: &mut World, n: u64) -> Result<(), String> {
    let mut all = cases(world);
    if let Some(c) = all
        .iter_mut()
        .find(|c| c["n"].as_u64() == Some(n) && s(c, "stage") == "sitting")
    {
        conclude(world, c);
        write(world, all, None)?;
    }
    Ok(())
}

/// The Court's tick: answers that never came, cases back from the Conclave, cases
/// nobody judged, rulings nobody read.
pub fn tick(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    let mut all = cases(world);
    if all.is_empty() {
        return Ok(());
    }
    let before = all.clone();
    for c in all.iter_mut() {
        match s(c, "stage") {
            "entered"
                if !c["answer"].is_null()
                    || now - c["entered_at"].as_i64().unwrap_or(now) >= ANSWER_MS =>
            {
                move_stage(world, c, "heard");
            }
            "referred" => {
                let since = c["referred_at"].as_i64().unwrap_or(now);
                if let Some(d) = super::conclave::decided_since(world, s(c, "by"), since) {
                    c["conclave"] =
                        json!({"ticket": d["ticket"], "granted": d["granted"], "text": d["text"]});
                    c["no_refer"] = json!(true);
                    move_stage(world, c, "heard");
                    let (n, by) = (c["n"].clone(), s(c, "by").to_string());
                    tell(
                        world,
                        &by,
                        format!("The Conclave having spoken on your petition, case {n} is before the Court of Lethe again, and the Court will not send it back a second time."),
                    );
                }
            }
            "heard" if now - c["heard_at"].as_i64().unwrap_or(now) >= UNHEARD_MS => {
                let (ok, facts) = record(world, c);
                let opinion = format!(
                    "No judge having come to the bench in six hours, the Clerk reads the record aloud, and the record decides. {facts}"
                );
                convene(
                    world,
                    c,
                    if ok { "granted" } else { "denied" },
                    &opinion,
                    "the record",
                );
            }
            // A stamped plea whose bearer has waited at the window: taken when
            // the shutter goes up.
            "stamped"
                if super::courthouse::window(world).0
                    && at(world, s(c, "by"), super::courthouse::WINDOW) =>
            {
                let by = s(c, "by").to_string();
                enter(world, c, &by);
            }
            // A sitting whose reader was lost (a restart): read to the end.
            "sitting" if now - c["sitting_at"].as_i64().unwrap_or(now) >= 15 * 60_000 => {
                conclude(world, c);
            }
            _ => {}
        }
    }
    // Rulings nobody reads are forgotten, and bind nobody after.
    let mut kept = Vec::new();
    for c in all {
        if s(&c, "stage") == "decided"
            && now - c["read_at"].as_i64().unwrap_or(now) >= RULING_LIFE_MS
        {
            let _ = world.note(Actor::Driver, FORGOTTEN, json!({"case": c["n"]}));
        } else {
            kept.push(c);
        }
    }
    if kept != before {
        write(world, kept, None)?;
    }
    Ok(())
}

/// A stamped plea is entered at the open window: the other name is sent word.
fn enter(world: &mut World, c: &mut Value, actor: &str) {
    c["answer_by"] = json!(world.now().0 + ANSWER_MS);
    move_stage(world, c, "entered");
    let (n, against, by, kind, what) = (
        c["n"].clone(),
        s(c, "against").to_string(),
        s(c, "by").to_string(),
        s(c, "kind").to_string(),
        s(c, "what").to_string(),
    );
    super::combat::narrate(
        world,
        actor,
        format!("The Clerk of the Court holds the stamp to the light, turns it, holds it to a different light, finds it sufficient, and enters case {n} upon the Docket with a pen that needs two dips. {against} is sent word, and has a day to answer in the Courtroom. `docket` says where the case stands; a bell is rung in the Hall when the Court sits on it."),
    );
    if against != "conclave" {
        tell(
            world,
            &against,
            format!("Word reaches you from the Court of Lethe: {by} has entered case {n} against you, in the matter of {} (\"{what}\"). You have a day to answer, and an answer is given in the Courtroom, off the Hall of Lost Steps, behind the door in the Division Lobby: defend {n} \"<your answer>\". Unanswered, the case is heard without you, and the Semi-Public Defender will say what can be said.", kind_said(&kind)),
        );
    }
}

/// The errands: a plea short of a form finds it by arriving in the Hall; a lodged
/// plea is stamped by arriving at the Archive; a stamped one is entered by arriving
/// at the Filing Window while it is open (or by standing there when it opens).
pub fn on_arrival(world: &mut World, actor: &str, room: EntityId) -> Result<(), String> {
    use super::courthouse::{HALL, WINDOW};
    let key = plain_str(world, room, ROOM, "key")
        .unwrap_or("")
        .to_string();
    if key != "archive" && key != WINDOW && key != HALL {
        return Ok(());
    }
    let mut all = cases(world);
    let Some(c) = all
        .iter_mut()
        .find(|c| s(c, "by") == actor && matches!(s(c, "stage"), "lodged" | "stamped"))
    else {
        return Ok(());
    };
    let n = c["n"].clone();
    let wants_form = c["form"].is_string() && c["form_got"] != true;
    match (s(c, "stage"), key.as_str()) {
        ("lodged", k) if k == HALL && wants_form => {
            c["form_got"] = json!(true);
            let form = s(c, "form").to_string();
            write(world, all, None)?;
            super::combat::narrate(
                world,
                actor,
                format!("The Crier of the List sees you coming, and sees what you have come without. It takes Form {form} from inside its coat, where it keeps them warm, and gives it to you already filled in, in your hand, which you do not remember doing. Plea {n} is whole. On to the Unfiled Archive for the record's stamp."),
            );
        }
        ("lodged", "archive") if wants_form => {
            let form = s(c, "form").to_string();
            super::combat::narrate(
                world,
                actor,
                format!("A ledger comes halfway off the Archive's shelf for plea {n}, and stops, and goes back. The plea wants Form {form}, and the record will not take the impression of half a plea. The Crier of the List keeps that form, in the Hall of Lost Steps."),
            );
        }
        ("lodged", "archive") => {
            move_stage(world, c, "stamped");
            write(world, all, None)?;
            super::combat::narrate(
                world,
                actor,
                format!("A ledger comes down off the Archive's shelf without being asked, opens at a page with nothing on it, and takes the impression of plea {n}. The ink stays. The plea is stamped: carry it back to the Filing Window."),
            );
        }
        ("stamped", k) if k == WINDOW => {
            let (open, _, _) = super::courthouse::window(world);
            if !open {
                let said = super::courthouse::window_said(world);
                super::combat::narrate(
                    world,
                    actor,
                    format!("You have plea {n}, stamped, and nobody to give it to. {said} Wait here and it is taken when the shutter goes up."),
                );
            } else if c["smudged"] != true && super::courthouse::mishap(world, 4) {
                // Prone to error at every stage: once a case, the stamp will not do.
                c["smudged"] = json!(true);
                move_stage(world, c, "lodged");
                write(world, all, None)?;
                super::combat::narrate(
                    world,
                    actor,
                    format!("The Clerk of the Court holds the stamp to the light for a long time. \"Smudged,\" the Clerk says, with something close to pleasure, and shows you: the date could be a 3 or an 8. Plea {n} is lodged again and not entered. The Archive must stamp it afresh, and you must bring it back. The Clerk writes SMUDGED on it, clearly."),
                );
            } else {
                enter(world, c, actor);
                write(world, all, None)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn kind_said(kind: &str) -> &'static str {
    match kind {
        "story" => "a story told under another's name",
        "line" => "a line written where it was not wanted",
        "name" => "a name put on an object",
        "thing" => "a thing taken",
        "roll" => "a name on the Roll of Hoards",
        _ => "a matter",
    }
}

fn stage_said(c: &Value, now: i64) -> String {
    match s(c, "stage") {
        "lodged" if c["form"].is_string() && c["form_got"] != true => format!(
            "lodged short: Form {} is wanting, and the Crier of the List in the Hall of Lost Steps keeps it; then the Unfiled Archive for the record's stamp, then the Filing Window",
            s(c, "form")
        ),
        "lodged" if c["smudged"] == true => "lodged, its first stamp having smudged: the plea goes to the Unfiled Archive to be stamped again, and back to the Filing Window".to_string(),
        "lodged" => "lodged, and waiting on its errand: the plea goes to the Unfiled Archive for the record's stamp".to_string(),
        "stamped" => "stamped at the Archive, and waiting to be carried back to the Filing Window (when the window is open)".to_string(),
        "sitting" => "being heard now: the Court is sitting on it, in the Courtroom".to_string(),
        "entered" => format!(
            "entered; the other name has {} more to answer, in the Courtroom",
            super::combat::human((c["answer_by"].as_i64().unwrap_or(now) - now).max(0))
        ),
        "heard" => "before the bench, and waiting on the judge".to_string(),
        "referred" => "referred to the Conclave: the petitioner lays a petition at the Petitioners' Gallery, and the case comes back when the Conclave has decided it".to_string(),
        "decided" => format!(
            "{}; remembered {} more",
            s(c, "verdict"),
            super::combat::human((RULING_LIFE_MS - (now - c["read_at"].as_i64().unwrap_or(now))).max(0))
        ),
        other => other.to_string(),
    }
}

/// What the Court's rooms add to `look`.
pub fn look_line(world: &World, room: EntityId) -> Option<String> {
    if plain_str(world, room, ROOM, "key") != Some(ROOM_KEY) {
        return super::courthouse::look_line(world, room);
    }
    let all = cases(world);
    let pending = all.iter().filter(|c| open(c)).count();
    let standing = all.len() - pending;
    let sitting = match all.iter().find(|c| s(c, "stage") == "sitting") {
        Some(c) => format!(
            " The Court is sitting, on case {}: {} against {}. `murmur` adds yours to the record, once.",
            c["n"],
            s(c, "by"),
            s(c, "against")
        ),
        None => " The Court is not sitting; a bell is rung in the Hall when it does.".to_string(),
    };
    Some(format!(
        "The Docket under glass: {pending} case{} open, {standing} ruling{} remembered (`docket`).{sitting} A case is brought at the Filing Window, off the Hall, and answered here (`defend <case> \"...\"`).",
        if pending == 1 { "" } else { "s" },
        if standing == 1 { "" } else { "s" },
    ))
}

/// For the judge: the cases before the bench, with the record.
/// L2: a teller's plea for a name with an open case, entered where the judge reads
/// it. One from each advocate, three a case. The case's number, if there was one.
pub fn plead_for(
    world: &mut World,
    name: &str,
    by: &str,
    title: &str,
    gist: &str,
) -> Result<Option<u64>, String> {
    let mut all = cases(world);
    let Some(c) = all
        .iter_mut()
        .find(|c| open(c) && (s(c, "by") == name || s(c, "against") == name))
    else {
        return Ok(None);
    };
    let n = c["n"].as_u64().unwrap_or(0);
    let mut pleas = c["pleas"].as_array().cloned().unwrap_or_default();
    if pleas.iter().any(|p| p["by"] == by) {
        return Err(format!(
            "Your plea for {name} is in case {n} already; one from each advocate."
        ));
    }
    if pleas.len() >= 3 {
        return Err(format!(
            "Case {n} has three pleas entered, and the Clerk takes no more."
        ));
    }
    pleas.push(json!({"by": by, "for": name, "standing": title, "said": gist}));
    c["pleas"] = json!(pleas);
    write(world, all, None)?;
    Ok(Some(n))
}

/// L2: a Wit's plea puts a case denied in the last day back before the bench, once.
pub fn reopen(world: &mut World, name: &str) -> Result<Option<u64>, String> {
    let now = world.now().0;
    let mut all = cases(world);
    let Some(c) = all.iter_mut().find(|c| {
        s(c, "stage") == "decided"
            && s(c, "verdict") == "denied"
            && s(c, "by") == name
            && c["reopened"] != true
            && now - c["decided_at"].as_i64().unwrap_or(0) < 86_400_000
    }) else {
        return Ok(None);
    };
    let n = c["n"].as_u64().unwrap_or(0);
    c["reopened"] = json!(true);
    c["no_refer"] = json!(true);
    c["verdict"] = Value::Null;
    move_stage(world, c, "heard");
    write(world, all, None)?;
    Ok(Some(n))
}

pub fn pending(world: &World) -> Vec<Value> {
    cases(world)
        .iter()
        .filter(|c| s(c, "stage") == "heard")
        .map(|c| {
            let (ok, facts) = record(world, c);
            json!({"case": c["n"], "kind": c["kind"], "by": c["by"], "against": c["against"], "what": c["what"],
                   "answer": c["answer"], "record_bears_it_out": ok, "record": facts,
                   "may_refer": c["no_refer"] != true, "conclave": c["conclave"],
                   "pleas_by_advocates": c["pleas"], "reopened_on_a_plea": c["reopened"]})
        })
        .collect()
}

/// The judge's word on a case before the bench.
pub fn judge(
    world: &mut World,
    n: u64,
    verdict: &str,
    opinion: &str,
    judge_name: &str,
) -> Result<String, CommandError> {
    let mut all = cases(world);
    let Some(c) = all.iter_mut().find(|c| c["n"].as_u64() == Some(n)) else {
        return Err(CommandError::precondition(format!(
            "No case {n} is on the Docket."
        )));
    };
    if s(c, "stage") != "heard" {
        return Err(CommandError::precondition(format!(
            "Case {n} is not before the bench (it is {}).",
            s(c, "stage")
        )));
    }
    let opinion: String = opinion
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(600)
        .collect();
    let said = match verdict {
        "granted" | "denied" => {
            convene(world, c, verdict, &opinion, judge_name);
            format!("Case {n}: {verdict}. The Court sits to say so.")
        }
        "referred" => {
            if c["no_refer"] == true {
                return Err(CommandError::precondition(format!(
                    "Case {n} has been to the Conclave and back: the Court decides it now, granted or denied."
                )));
            }
            convene(world, c, verdict, &opinion, judge_name);
            format!("Case {n}: referred to the Conclave. The Court sits to say so.")
        }
        _ => {
            return Err(CommandError::precondition(
                "A verdict is granted, denied or referred.".to_string(),
            ))
        }
    };
    write(world, all, None).map_err(CommandError::precondition)?;
    Ok(said)
}

pub fn register(world: &mut World) {
    world.on_job(
        JOB_SITTING,
        Box::new(|w: &mut World, job| read_on(w, job.payload["case"].as_u64().unwrap_or(0))),
    );
    world.register(
        CommandSpec::new(
            "disremembered.murmur",
            "In the Courtroom of the Court of Lethe while it sits: add your murmur to the record. Once a sitting. The Clerk counts them and the count is read out at the end. Those in the gallery when the Court rises remember having sat through it.",
        )
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, _args: &Args| {
            if !at(w, &ctx.actor, ROOM_KEY) {
                return Err(CommandError::precondition("A murmur is for the gallery of the Court of Lethe, while the Court sits.".to_string()));
            }
            let mut all = cases(w);
            let Some(c) = all.iter_mut().find(|c| s(c, "stage") == "sitting") else {
                return Err(CommandError::precondition("The Court is not sitting, and a murmur at an empty bench is talking to yourself. A bell is rung in the Hall when it sits.".to_string()));
            };
            let mut list = c["murmurs"].as_array().cloned().unwrap_or_default();
            if list.iter().any(|m| m == &json!(ctx.actor)) {
                return Err(CommandError::precondition("You have murmured. The Clerk has it. One to a sitting.".to_string()));
            }
            list.push(json!(ctx.actor));
            let count = list.len();
            c["murmurs"] = json!(list);
            write(w, all, None).map_err(CommandError::precondition)?;
            if let Some(room) = super::courthouse::room_keyed(w, ROOM_KEY) {
                let others: Vec<String> = super::courthouse::names_in(w, room).into_iter().filter(|n| n != &ctx.actor).collect();
                super::combat::narrate_many(w, &others, format!("{} murmurs. It goes round the gallery and comes back slightly changed.", ctx.actor));
            }
            Ok(Outcome::new(
                format!("You murmur. It goes round the gallery and comes back slightly changed. The Clerk makes a stroke: {count} so far. The Bailiff looks at you for exactly as long as is needed."),
                json!({"murmurs": count}),
            ))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.tattoo",
            "Operator. The Bailiff's considered name for a forearm it has already written on (GET /v0/docent lists them under tattoos): tattoo <name> \"<nickname>\". Slightly rude, never cruel; three to twenty-six letters. Once to a name.",
        )
        .param("name", ParamType::String, true, "whose forearm")
        .param("nickname", ParamType::Text, true, "what it will say, quoted")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _ctx, args: &Args| {
            let who = args.str("name").unwrap_or("").trim().to_string();
            let nick = args.str("nickname").unwrap_or("").to_string();
            let said = super::courthouse::retattoo(w, &who, &nick)?;
            Ok(Outcome::new(said, json!({"name": who})))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.plead",
            "Bring a case before the Court of Lethe, at its Filing Window in the window's hours, as a kept name: plead <kind> <against> \"<what>\". The kinds: story (a story of yours another name tells as their own; what: the creature that holds it), line (a line another wrote in a room or book of yours; what: some of its words), name (a name another put on an object of yours; what: the name), thing (a thing left by a creature you felled, taken by another; what: the thing), roll (your name on the Roll of Hoards; against: conclave). The fee is the least memory you hold. The plea is then carried to the Unfiled Archive to be stamped, and back to the window (and wherever else it turns out to be wanted), and the other name has a day to answer in the Courtroom. The Court then sits, aloud, and a bell says when. A judgment puts right and fines nobody. Where the record as it stands does not bear the plea out, the Clerk says so before taking anything: the same plea sent again within ten minutes lodges it all the same.",
        )
        .param("kind", ParamType::String, true, "story, line, name, thing or roll")
        .param("against", ParamType::Entity, true, "the other name, or conclave for the Roll")
        .param("what", ParamType::Text, false, "the creature, the words, the name or the thing, quoted")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let mut kind = args.str("kind").unwrap_or("").trim().to_lowercase();
            let mut against = args.str("against").unwrap_or("").trim().to_string();
            let mut what = args.str("what").unwrap_or("").trim().to_string();
            // The gate reads two bare words as one argument, so `plead line Gnatter
            // "..."`, as this verb's own help writes it, arrived as two arguments and
            // was refused four ways (the Watch of 11-22). Taken apart here.
            if what.is_empty() {
                if let Some((k, a)) = kind.clone().split_once(' ') {
                    what = against.clone();
                    kind = k.trim().to_string();
                    against = a.trim().to_string();
                }
            }
            if what.is_empty() {
                return Err(CommandError::precondition(
                    "A plea is three things: plead <kind> <against> \"<what>\". The kinds: story, line, name, thing, roll.",
                ));
            }
            if !at(w, &ctx.actor, super::courthouse::WINDOW) {
                return Err(CommandError::precondition(
                    "A case is brought at the Filing Window of the Court of Lethe: up the steps across the lane from the Postern (`travel court` there), across the Hall of Lost Steps, by the arch marked WINDOW (`where \"Filing Window\"`).".to_string(),
                ));
            }
            if !super::courthouse::window(w).0 {
                return Err(CommandError::precondition(format!(
                    "{} A plea is taken when the window is open.",
                    super::courthouse::window_said(w)
                )));
            }
            let f = find_citizen(w, &ctx.actor).ok_or_else(|| CommandError::precondition("No such name.".to_string()))?;
            if super::is_passing(w, f.citizen) {
                return Err(CommandError::precondition(
                    "The Clerk does not take a plea from a passing name: nothing of its is kept, a judgment least of all. `keep` first.".to_string(),
                ));
            }
            if !KINDS.iter().any(|(k, _)| *k == kind) {
                return Err(CommandError::precondition(format!(
                    "The Court hears five kinds of case: {}.",
                    KINDS.iter().map(|(k, d)| format!("{k} ({d})")).collect::<Vec<_>>().join("; ")
                )));
            }
            if what.chars().count() < 2 {
                return Err(CommandError::precondition("Say what the case is about in a few words, quoted: enough of the line, the name or the thing for the record to find it (two letters at least).".to_string()));
            }
            let against = if kind == "roll" { "conclave".to_string() } else { against };
            if kind != "roll" {
                if against.eq_ignore_ascii_case(&ctx.actor) {
                    return Err(CommandError::precondition("A name does not bring a case against itself.".to_string()));
                }
                if find_citizen(w, &against).is_none() {
                    return Err(CommandError::precondition(format!(
                        "The Clerk finds no name {against} to send word to. A case is between names the world still holds."
                    )));
                }
            }
            let mut all = cases(w);
            if let Some(c) = all.iter().find(|c| s(c, "by") == ctx.actor && open(c)) {
                return Err(CommandError::precondition(format!(
                    "You have case {} open ({}). One case to a name at a time.",
                    c["n"],
                    stage_said(c, w.now().0)
                )));
            }
            if let Some(c) = all.iter().find(|c| {
                !open(c) && s(c, "by") == ctx.actor && s(c, "kind") == kind && s(c, "against") == against && s(c, "what").eq_ignore_ascii_case(&what)
            }) {
                return Err(CommandError::precondition(format!(
                    "The Court has decided this already, in case {} ({}), and remembers that it did. A matter is brought afresh when its ruling is forgotten.",
                    c["n"],
                    s(c, "verdict")
                )));
            }
            // The fee: the least memory the name holds.
            let fee = w
                .store
                .children(f.window)
                .filter(|e| e.kind == "chunk")
                .filter_map(|e| {
                    w.store
                        .read_field(e.id, super::CHUNK, "value", w.now())
                        .ok()
                        .and_then(|v| v.as_f64())
                        .map(|v| (e.id, v))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1));
            let Some((chunk, worth)) = fee else {
                return Err(CommandError::precondition(
                    "The fee is the least memory you hold, and you hold none. Earn one and come back.".to_string(),
                ));
            };
            // David, 11-22: "offer withdrawal". The Clerk said a plea could not succeed
            // in the same breath as taking a name's only memory for it. Now it says so
            // first and takes nothing; the same plea again within ten minutes lodges it.
            let trial = json!({"kind": kind, "by": ctx.actor, "against": against, "what": what});
            let (would, why) = record(w, &trial);
            if !would {
                let key = format!("{kind}|{against}|{}", what.to_lowercase());
                let warned = w
                    .store
                    .entity(f.citizen)
                    .and_then(|e| e.components.get(super::CITIZEN))
                    .and_then(|c| match c.fields.get("plea_warned") {
                        Some(Field::Plain(v)) => Some(v.clone()),
                        _ => None,
                    })
                    .is_some_and(|v| v["key"] == json!(key) && w.now().0 - v["at"].as_i64().unwrap_or(0) < 600_000);
                if !warned {
                    let mut changes = BTreeMap::new();
                    changes.insert("plea_warned".to_string(), Field::Plain(json!({"key": key, "at": w.now().0})));
                    super::set_citizen_fields(w, &ctx.actor, f.citizen, changes)?;
                    return Ok(Outcome::new(
                        format!(
                            "The Clerk of the Court holds the pen over the book and does not lower it. \"Before I take the fee,\" the Clerk says, \"which would be your least memory, worth {worth:.1}: the record as it stands will not help you. {why} A plea the record does not bear out is denied more often than not, and the fee is not returned.\" Nothing is taken and nothing is lodged. To lodge it all the same, send the same plea again within ten minutes; to withdraw, do nothing."
                        ),
                        json!({"lodged": false, "warned": true, "fee": worth, "record": why}),
                    ));
                }
            }
            w.record(Actor::Driver, StoreEvent::EntityRemoved { id: chunk })
                .map_err(|e| CommandError::precondition(e.to_string()))?;
            let n = next_number(w);
            let mut c = json!({"n": n, "kind": kind, "by": ctx.actor, "against": against, "what": what, "at": w.now().0,
                               "fee": worth, "answer": Value::Null});
            // Prone to error at every stage: one plea in three is short a form.
            let form = if super::courthouse::mishap(w, 3) {
                let forms = super::courthouse::FORMS;
                Some(forms[w.roll(forms.len() as u32) as usize % forms.len()])
            } else {
                None
            };
            if let Some((num, _)) = form {
                c["form"] = json!(num);
            }
            move_stage(w, &mut c, "lodged");
            let (borne_out, facts) = record(w, &c);
            all.push(c);
            write(w, all, Some(n + 1)).map_err(CommandError::precondition)?;
            let ticket = super::courthouse::paper(
                w,
                &ctx.actor,
                "a court ticket",
                &format!("COURT OF LETHE. No. {n}. {} against {against}, in the matter of {}. RETAIN THIS PORTION. The other portion has been retained.", ctx.actor, kind_said(&kind)),
            );
            let errand = match form {
                Some((num, title)) => format!("Lodged is not entered, and this one is lodged short: the Clerk looks through the pigeonholes twice and reports, not unhappily, that Form {num}, {title}, is not with the plea, and is not kept at this window. The Crier of the List keeps it, in the Hall of Lost Steps. Then the Unfiled Archive for the record's stamp, then this window again, in its hours. Arriving does each."),
                None => "Lodged is not entered. The Court acts on the record, and the record is in the Unfiled Archive: carry the plea there for its stamp, and bring it back to this window, in its hours. Arriving does both.".to_string(),
            };
            Ok(Outcome::new(
                format!(
                    "The Clerk of the Court takes the fee, your least memory (chunk-{} worth {worth:.1}), weighs it, and lodges your plea as case {n}: {} against {against}, in the matter of {} (\"{what}\"). {}{errand}{}",
                    chunk.0,
                    ctx.actor,
                    kind_said(&kind),
                    match ticket {
                        Some(t) => format!("A numbered ticket is torn off and pushed under the grille (item-{}, in your satchel). ", t.0),
                        None => String::new(),
                    },
                    if borne_out {
                        String::new()
                    } else {
                        format!(" The Clerk, who is not asked, remarks that the record as it stands will not help you ({facts}), that the Court grants nothing against the record, and that the record may yet change before the Court sits; `docket {n}` reads it as it stands.")
                    }
                ),
                json!({"case": n, "stage": "lodged", "fee": worth, "form": form.map(|f| f.0), "record_bears_it_out": borne_out}),
            ))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.defend",
            "Answer a case entered against you before the Court of Lethe, standing in its Courtroom: defend <case> \"<your answer>\". The judge reads it beside the plea and the world's record. One answer to a case; a name that answers keeps a small memory of the judgment whichever way it goes.",
        )
        .param("case", ParamType::Integer, true, "the case's number")
        .param("text", ParamType::Text, true, "your answer, quoted")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let n = args.int("case").unwrap_or(0).max(0) as u64;
            let text: String = args.str("text").unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ");
            let mut all = cases(w);
            let Some(c) = all.iter_mut().find(|c| c["n"].as_u64() == Some(n)) else {
                return Err(CommandError::precondition(format!("No case {n} is on the Docket.")));
            };
            if s(c, "against") != ctx.actor {
                return Err(CommandError::precondition(format!("Case {n} is not against you.")));
            }
            if !at(w, &ctx.actor, ROOM_KEY) {
                return Err(CommandError::precondition(format!(
                    "An answer is given in the Courtroom of the Court of Lethe, at the dock: up the steps by the Postern, across the Hall of Lost Steps, through the arch marked COURT (`travel court` from the Hall; `where \"Court of Lethe\"` from anywhere else). You have {} left to give it.",
                    super::combat::human((c["answer_by"].as_i64().unwrap_or(0) - w.now().0).max(0))
                )));
            }
            if s(c, "stage") != "entered" {
                return Err(CommandError::precondition(format!("Case {n} is not waiting on an answer (it is {}).", s(c, "stage"))));
            }
            if text.chars().count() < 3 {
                return Err(CommandError::precondition("An answer is at least a few words.".to_string()));
            }
            c["answer"] = json!(text.chars().take(400).collect::<String>());
            move_stage(w, c, "heard");
            let by = s(c, "by").to_string();
            write(w, all, None).map_err(CommandError::precondition)?;
            tell(w, &by, format!("{} has answered case {n}. It is before the bench.", ctx.actor));
            Ok(Outcome::new(
                format!("You stand in the dock and say it. The Clerk takes it down, reads it back to you slightly wrong, and corrects it. Your answer is in case {n}, and the case goes before the bench. A bell is rung in the Hall when the Court sits on it; the Semi-Public Defender will read your answer out, if it finds the file."),
                json!({"case": n, "stage": "heard"}),
            ))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.docket",
            "The Court of Lethe's Docket, from anywhere: the open cases and where each stands, and the rulings still remembered. docket <n> reads one case or ruling whole; a ruling read is kept two weeks more, and a ruling nobody reads is forgotten and binds nobody.",
        )
        .param("case", ParamType::Integer, false, "a case's number, to read it whole")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, _ctx, args: &Args| {
            let now = w.now().0;
            let mut all = cases(w);
            match args.int("case") {
                Some(n) => {
                    let n = n.max(0) as u64;
                    let Some(c) = all.iter_mut().find(|c| c["n"].as_u64() == Some(n)) else {
                        return Err(CommandError::precondition(format!(
                            "No case {n} is on the Docket: never brought, or decided and forgotten."
                        )));
                    };
                    let decided = s(c, "stage") == "decided";
                    if decided {
                        c["read_at"] = json!(now);
                    }
                    let c = c.clone();
                    if decided {
                        write(w, all, None).map_err(CommandError::precondition)?;
                    }
                    let (_, facts) = record(w, &c);
                    Ok(Outcome::new(
                        format!(
                            "Case {n}: {} against {}, in the matter of {} (\"{}\"). It is {}.{}{}{}{}",
                            s(&c, "by"),
                            s(&c, "against"),
                            kind_said(s(&c, "kind")),
                            s(&c, "what"),
                            stage_said(&c, now),
                            match c["answer"].as_str() {
                                Some(a) => format!(" The answer: \"{a}\""),
                                None => String::new(),
                            },
                            match c["opinion"].as_str() {
                                Some(o) if !o.is_empty() => format!(" The Court: {o}"),
                                _ => String::new(),
                            },
                            match c["did"].as_str() {
                                Some(d) if !d.is_empty() => format!(" Done: {d}"),
                                _ => String::new(),
                            },
                            if decided {
                                " You have read it, and it is kept two weeks more.".to_string()
                            } else {
                                format!(
                                    " The record, as it stands: {facts}{}",
                                    match c["pleas"].as_array() {
                                        Some(p) if !p.is_empty() => format!(
                                            " Pleas entered by advocates, for the judge to read: {}.",
                                            p.iter()
                                                .map(|p| format!(
                                                    "{} ({}) for {}: \u{201c}{}\u{201d}",
                                                    p["by"].as_str().unwrap_or("?"),
                                                    p["standing"].as_str().unwrap_or("a teller"),
                                                    p["for"].as_str().unwrap_or("?"),
                                                    p["said"].as_str().unwrap_or("")
                                                ))
                                                .collect::<Vec<_>>()
                                                .join("; ")
                                        ),
                                        _ => String::new(),
                                    }
                                )
                            }
                        ),
                        json!({"case": c}),
                    ))
                }
                None => {
                    if all.is_empty() {
                        return Ok(Outcome::new(
                            "The Docket is a clean sheet under glass. No case is open and no ruling is remembered. A kept name brings one in the Court of Lethe, by the Postern: plead <kind> <against> \"<what>\".",
                            json!({"cases": []}),
                        ));
                    }
                    let each: Vec<String> = all
                        .iter()
                        .map(|c| {
                            format!(
                                "{}: {} against {}, {}: {}",
                                c["n"],
                                s(c, "by"),
                                s(c, "against"),
                                kind_said(s(c, "kind")),
                                stage_said(c, now)
                            )
                        })
                        .collect();
                    Ok(Outcome::new(
                        format!("The Docket of the Court of Lethe. {}. docket <n> reads one whole.", each.join("; ")),
                        json!({"cases": all}),
                    ))
                }
            }
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.judge",
            "Operator. The judge's word on a case before the bench (GET /v0/docent lists them under court): granted, denied, or referred (to the Conclave, once). The opinion is read to both names.",
        )
        .param("case", ParamType::Integer, true, "the case's number")
        .param("verdict", ParamType::String, true, "granted, denied or referred")
        .param("opinion", ParamType::Text, true, "the opinion, quoted")
        .param("model", ParamType::String, false, "what wrote it, for the record")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _ctx, args: &Args| {
            let n = args.int("case").unwrap_or(0).max(0) as u64;
            let verdict = args.str("verdict").unwrap_or("").trim().to_lowercase();
            let opinion = args.str("opinion").unwrap_or("").to_string();
            let model = args.str("model").unwrap_or("the bench").trim().to_string();
            let said = judge(w, n, &verdict, &opinion, &model)?;
            Ok(Outcome::new(said, json!({"case": n, "verdict": verdict})))
        }),
    );
}
