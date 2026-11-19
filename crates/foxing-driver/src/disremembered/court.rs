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
        let room = EntityId(e.payload["room"].as_u64()?);
        let added = e.payload["added"].as_str()?;
        (owner_of(world, room).as_deref() == Some(by)
            && added.to_lowercase().contains(low)
            && plain_str(world, room, ROOM, "description").is_some_and(|d| d.contains(added)))
        .then(|| Line::Room(room, added.to_string()))
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
            super::combat::narrate(world, &who, format!("{said}{kept} (`docket {n}` at the Court reads the ruling, and a ruling read is a ruling kept.)"));
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
                decide(
                    world,
                    c,
                    if ok { "granted" } else { "denied" },
                    &opinion,
                    "the record",
                );
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

/// The errand: a lodged plea is stamped by arriving at the Archive with it, and
/// entered by arriving back at the Court.
pub fn on_arrival(world: &mut World, actor: &str, room: EntityId) -> Result<(), String> {
    let key = plain_str(world, room, ROOM, "key")
        .unwrap_or("")
        .to_string();
    if key != "archive" && key != ROOM_KEY {
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
    match (s(c, "stage"), key.as_str()) {
        ("lodged", "archive") => {
            move_stage(world, c, "stamped");
            write(world, all, None)?;
            super::combat::narrate(
                world,
                actor,
                format!("A ledger comes down off the Archive's shelf without being asked, opens at a page with nothing on it, and takes the impression of plea {n}. The ink stays. The plea is stamped: carry it back to the Court of Lethe."),
            );
        }
        ("stamped", k) if k == ROOM_KEY => {
            c["answer_by"] = json!(world.now().0 + ANSWER_MS);
            move_stage(world, c, "entered");
            let (against, by, kind, what) = (
                s(c, "against").to_string(),
                s(c, "by").to_string(),
                s(c, "kind").to_string(),
                s(c, "what").to_string(),
            );
            write(world, all, None)?;
            super::combat::narrate(
                world,
                actor,
                format!("The Clerk of the Court holds the stamp to the light, finds it sufficient, and enters case {n} upon the Docket. {against} is sent word, and has a day to answer. `docket` says where the case stands."),
            );
            if against != "conclave" {
                tell(
                    world,
                    &against,
                    format!("Word reaches you from the Court of Lethe: {by} has entered case {n} against you, in the matter of {} (\"{what}\"). You have a day to answer, from anywhere: defend {n} \"<your answer>\". Unanswered, the case is heard without you.", kind_said(&kind)),
                );
            }
        }
        _ => {}
    }
    Ok(())
}

fn kind_said(kind: &str) -> &'static str {
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
        "lodged" => "lodged, and waiting on its errand: the plea goes to the Unfiled Archive for the record's stamp".to_string(),
        "stamped" => "stamped at the Archive, and waiting to be carried back to the Court".to_string(),
        "entered" => format!(
            "entered; the other name has {} more to answer",
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

/// What the Court's room adds to `look`.
pub fn look_line(world: &World, room: EntityId) -> Option<String> {
    if plain_str(world, room, ROOM, "key") != Some(ROOM_KEY) {
        return None;
    }
    let all = cases(world);
    let pending = all.iter().filter(|c| open(c)).count();
    let standing = all.len() - pending;
    Some(format!(
        "The Docket hangs by the Clerk's window: {pending} case{} open, {standing} ruling{} remembered (`docket`). A kept name brings a case here with plead <kind> <against> \"<what>\", the kinds being story, line, name, thing and roll; the fee is the least memory you hold, and the plea then goes to the Archive and back.",
        if pending == 1 { "" } else { "s" },
        if standing == 1 { "" } else { "s" },
    ))
}

/// For the judge: the cases before the bench, with the record.
pub fn pending(world: &World) -> Vec<Value> {
    cases(world)
        .iter()
        .filter(|c| s(c, "stage") == "heard")
        .map(|c| {
            let (ok, facts) = record(world, c);
            json!({"case": c["n"], "kind": c["kind"], "by": c["by"], "against": c["against"], "what": c["what"],
                   "answer": c["answer"], "record_bears_it_out": ok, "record": facts,
                   "may_refer": c["no_refer"] != true, "conclave": c["conclave"]})
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
            decide(world, c, verdict, &opinion, judge_name);
            format!("Case {n}: {verdict}.")
        }
        "referred" => {
            if c["no_refer"] == true {
                return Err(CommandError::precondition(format!(
                    "Case {n} has been to the Conclave and back: the Court decides it now, granted or denied."
                )));
            }
            c["opinion"] = json!(opinion);
            move_stage(world, c, "referred");
            let by = s(c, "by").to_string();
            tell(
                world,
                &by,
                format!("The Court of Lethe has heard case {n} and will not decide it yet: {opinion} The case is referred to the Conclave. Lay a petition in the matter at the Petitioners' Gallery (petition \"…\"); when the Conclave has decided it, either way, the case comes back to the Court, which then must decide."),
            );
            format!("Case {n}: referred to the Conclave.")
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
    world.register(
        CommandSpec::new(
            "disremembered.plead",
            "Bring a case before the Court of Lethe, standing in it, as a kept name: plead <kind> <against> \"<what>\". The kinds: story (a story of yours another name tells as their own; what: the creature that holds it), line (a line another wrote in a room or book of yours; what: some of its words), name (a name another put on an object of yours; what: the name), thing (a thing left by a creature you felled, taken by another; what: the thing), roll (your name on the Roll of Hoards; against: conclave). The fee is the least memory you hold. The plea is then carried to the Unfiled Archive to be stamped, and back, and the other name has a day to answer. A judgment puts right and fines nobody.",
        )
        .param("kind", ParamType::String, true, "story, line, name, thing or roll")
        .param("against", ParamType::String, true, "the other name, or conclave for the Roll")
        .param("what", ParamType::Text, true, "the creature, the words, the name or the thing, quoted")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let kind = args.str("kind").unwrap_or("").trim().to_lowercase();
            let against = args.str("against").unwrap_or("").trim().to_string();
            let what = args.str("what").unwrap_or("").trim().to_string();
            if in_court(w, &ctx.actor).is_none() {
                return Err(CommandError::precondition(
                    "A case is brought in the Court of Lethe, through the one door behind AYE and NO in the Division Lobby (`where \"Court of Lethe\"`).".to_string(),
                ));
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
                return Err(CommandError::precondition("Say what the case is about, quoted.".to_string()));
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
            w.record(Actor::Driver, StoreEvent::EntityRemoved { id: chunk })
                .map_err(|e| CommandError::precondition(e.to_string()))?;
            let n = next_number(w);
            let mut c = json!({"n": n, "kind": kind, "by": ctx.actor, "against": against, "what": what, "at": w.now().0,
                               "fee": worth, "answer": Value::Null});
            move_stage(w, &mut c, "lodged");
            all.push(c);
            write(w, all, Some(n + 1)).map_err(CommandError::precondition)?;
            Ok(Outcome::new(
                format!(
                    "The Clerk of the Court takes the fee, your least memory (chunk-{} worth {worth:.1}), weighs it, and lodges your plea as case {n}: {} against {against}, in the matter of {} (\"{what}\"). Lodged is not entered. The Court acts on the record, and the record is in the Unfiled Archive: carry the plea there for its stamp, and bring it back here. Arriving does both.",
                    chunk.0,
                    ctx.actor,
                    kind_said(&kind)
                ),
                json!({"case": n, "stage": "lodged", "fee": worth}),
            ))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.defend",
            "Answer a case entered against you before the Court of Lethe, from anywhere: defend <case> \"<your answer>\". The judge reads it beside the plea and the world's record. One answer to a case; a name that answers keeps a small memory of the judgment whichever way it goes.",
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
                format!("Your answer is read into case {n}, and the case goes before the bench. The Court will send word of its judgment."),
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
                                format!(" The record, as it stands: {facts}")
                            }
                        ),
                        json!({"case": c}),
                    ))
                }
                None => {
                    if all.is_empty() {
                        return Ok(Outcome::new(
                            "The Docket is a clean sheet under glass. No case is open and no ruling is remembered. A kept name brings one in the Court of Lethe, off the Division Lobby: plead <kind> <against> \"<what>\".",
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
