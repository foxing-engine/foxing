//! X6, 11-20: the courthouse around the Court of Lethe. David's brief: more rooms
//! and more razzle dazzle; hearings a bigger event (creatures drawn to them, a lot
//! of opining, murmurs, a judge who loses the thread mid-pontification, a Realm's
//! Attorney and a Semi-Public Defender each strongly of a type); every stage prone
//! to error (a missing form, an errand, erratic hours); a Bailiff who answers a hand
//! raised in the Courtroom by tattooing a slightly rude name on its owner (the
//! Docent's model writes the names; how one is removed is content for later: X24).
//!
//! What is here: the window's hours, the mishaps, the script of a sitting, the
//! Bailiff, and what each new room adds to `look`. The cases themselves are in
//! `court.rs`.

use super::{find_citizen, plain_str, CITIZEN, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{CommandError, Outcome};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const HALL: &str = "lost-steps";
pub const WINDOW: &str = "filing-window";
pub const COURTROOM: &str = "court-of-lethe";
pub const CELLS: &str = "cells-of-record";
pub const STAIR: &str = "appeal-stair";

/// The window keeps hours in slots of this length, and is shut one slot in three.
const SLOT_MS: i64 = 10 * 60_000;
/// A sitting: seconds from the bell to the first line, and between lines.
pub const BELL_S: i64 = 20;
pub const LINE_S: i64 = 14;
pub const ADJOURNED_S: i64 = 75;
/// What attending a sitting to its end is worth to a name that is no party to it.
pub const GALLERY_XP: f64 = 3.0;
pub const TATTOOED: &str = "disremembered.tattooed";

pub fn key_of(world: &World, room: EntityId) -> &str {
    plain_str(world, room, ROOM, "key").unwrap_or("")
}

pub fn room_keyed(world: &World, key: &str) -> Option<EntityId> {
    super::rooms::room_by_key(world, key)
}

/// The names standing in a room.
pub fn names_in(world: &World, room: EntityId) -> Vec<String> {
    super::talk::others_here(world, room, "")
}

pub fn say_in(world: &mut World, key: &str, text: &str) {
    if let Some(room) = room_keyed(world, key) {
        let to = names_in(world, room);
        super::combat::narrate_many(world, &to, text.to_string());
    }
}

fn mix(n: i64) -> u64 {
    let mut x = (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x5DEE_CE66D;
    x ^= x >> 29;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^ (x >> 32)
}

fn hhmm(ms: i64) -> String {
    let m = ms.div_euclid(60_000);
    format!(
        "{:02}:{:02} UTC",
        m.div_euclid(60).rem_euclid(24),
        m.rem_euclid(60)
    )
}

const AWAY: &[&str] = &[
    "GONE TO FIND THE OTHER STAMP",
    "AT LUNCH (the card does not say whose)",
    "CLOSED FOR THE COUNTING OF FORMS",
    "BACK DIRECTLY",
    "CLOSED BY ORDER OF THE COURT, WHICH HAS BEEN ASKED WHY",
    "THE CLERK IS IN THE ROOM BUT THE WINDOW IS NOT OPEN",
    "INK",
];

/// The Filing Window's hours: open or shut now, when that changes, and the card.
pub fn window(world: &World) -> (bool, i64, &'static str) {
    let now = world.now().0;
    let slot = now.div_euclid(SLOT_MS);
    let shut = |s: i64| mix(s) % 3 == 0;
    if super::court::calm(world) {
        return (true, now + SLOT_MS, "");
    }
    let open = !shut(slot);
    // The first slot whose state differs from now's.
    let mut change = slot + 1;
    while shut(change) == shut(slot) && change < slot + 12 {
        change += 1;
    }
    (
        open,
        change * SLOT_MS,
        AWAY[(mix(slot) / 3) as usize % AWAY.len()],
    )
}

pub fn window_said(world: &World) -> String {
    let (open, until, card) = window(world);
    if open {
        format!("The shutter is up. The card gives the hours as OPEN, with UNTIL {} written under it in a newer ink.", hhmm(until))
    } else {
        format!("The shutter is down. The card in it: {card}. Under that, in pencil, by somebody who has waited here before: opens {}.", hhmm(until))
    }
}

pub const FORMS: &[(&str, &str)] = &[
    (
        "7-B",
        "the Declaration That The Thing Complained Of Occurred",
    ),
    (
        "12",
        "the Petitioner's Undertaking To Still Care By The Time Of Hearing",
    ),
    (
        "3 (pink)",
        "the Schedule Of Things The Petitioner Has Not Been Asked",
    ),
    ("40-A", "the Certificate That Form 40 Is Not Required"),
    ("9", "the Notice Of Intention To Have Given Notice"),
];

/// One chance in `one_in`, unless the Court is calm (tests).
pub fn mishap(world: &mut World, one_in: u32) -> bool {
    !super::court::calm(world) && world.roll(one_in) == 0
}

// ---- the sitting's script ----

fn pick<'a>(world: &mut World, of: &'a [&'a str]) -> &'a str {
    of[world.roll(of.len() as u32) as usize % of.len()]
}

const REALM_OPENS: &[&str] = &[
    "The Realm's Attorney rises, and goes on rising for some time. \"THE REALM IS AGAINST IT.\" The Clerk asks against what. \"THE REALM HAS NOT YET HEARD WHAT IT IS, AND IS AGAINST IT IN ADVANCE, WHICH IS THE MORE EFFICIENT ORDER.\"",
    "The Realm's Attorney strikes the table with a file. A second file slides off the stack and is caught by the Bailiff without looking. \"THE REALM OBJECTS. THE REALM HAS OBJECTED IN EVERY MATTER SINCE THE FIRST AND SEES NO REASON TO BREAK A RUN.\"",
    "The Realm's Attorney addresses the empty jury box, with feeling, for the length of a paragraph. Reminded that it is empty: \"THE REALM IS AWARE. THE REALM FINDS THEM ATTENTIVE.\"",
    "The Realm's Attorney: \"THE REALM WILL BE BRIEF.\" It puts on a second pair of spectacles over the first and opens a file as thick as a step.",
];
const REALM_REBUTS: &[&str] = &[
    "The Realm's Attorney: \"THE REALM NOTES THE MURMURING AND MOVES THAT IT BE STRUCK FROM THE RECORD.\" The Clerk says a noise cannot be struck. \"THEN THE REALM OBJECTS TO NOISE, GENERALLY, AND ASKS THAT ITS OBJECTION BE READ BACK.\" It is read back. The Realm's Attorney nods along to it.",
    "The Realm's Attorney: \"THE REALM SUBMITS THAT IF THIS IS ALLOWED, EVERYTHING WILL BE.\" It is asked for an example. \"THE REALM DECLINES TO IMAGINE ONE, ON PRINCIPLE.\"",
    "The Realm's Attorney produces an exhibit, holds it up to the gallery, and is told by the Clerk that it is the Realm's own lunch. \"THE REALM WITHDRAWS THE EXHIBIT AND RESERVES ITS POSITION ON THE LUNCH.\"",
    "The Realm's Attorney: \"THE REALM RESTS.\" It does not sit down. \"THE REALM RESTS STANDING.\"",
];
const DEFENDER_WRONG: &[&str] = &[
    "Mr. Tallow",
    "the widow Oakes",
    "a Mr. or Mrs. Pell",
    "the party of the second part, whose name I had a moment ago",
    "Corbin",
    "whoever is in the dock",
];
const GALLERY: &[&str] = &[
    "From the back of the gallery a voice gives its opinion, which is that it knew a wisp once to whom this exact thing happened, except that it was a hollow, and the other way about.",
    "Somebody in the third tier says, to nobody, that in their day a case had a beginning. Somebody in the second tier says that in their day it had an end.",
    "A voice from the gallery asks whether there will be an interval. The Bailiff looks at the voice. There is no further question about an interval.",
    "Two in the gallery have begun to argue the case between themselves and are doing it rather better than counsel. The Clerk writes that down and then, thinking again, crosses it out.",
    "The Crier of the List has come as far as the door to listen, been asked by a look whether the List is crying itself, and gone back.",
    "An engrosser has come down from the Conclave and stands in the doorway taking the whole thing down in a large clear hand, for no one.",
    "The gallery leans, all together, the way wheat does.",
];
const LOST_THREADS: &[&str] = &[
    "and it puts the Court in mind of a ferry there used to be, which went to the far bank only when the ferryman had somebody to disagree with, and the Court has always thought that a lesson, though of what precisely",
    "which is a principle as old as the bench itself, older, for the bench was got secondhand from a chapel that had no further use for it, a good bench, oak, they do not make them, the Court sat on a worse one for years and its back",
    "and the Court has said so before, in a case concerning a gate, or it may have been a goat, the report is unclear and the reporter was not a careful man though he kept bees very well",
    "for memory, as the Court has had occasion to remark, is a river, or it is like a river, or there is a river, the Court is nearly sure about the river",
    "and here the Court would remind counsel of the rule in the matter of the two brothers and the one hat, a rule so well known that the Court need not state it, which is fortunate",
];

/// The lines of a sitting, each with the seconds to wait after it. Written once,
/// when the Court convenes, and kept on the case, so a replay says the same.
pub fn script(world: &mut World, c: &Value, drawn: &[String]) -> Vec<Value> {
    let s = |k: &str| c[k].as_str().unwrap_or("").to_string();
    let (n, by, against, what) = (c["n"].clone(), s("by"), s("against"), s("what"));
    let verdict = c["ruling"]["verdict"]
        .as_str()
        .unwrap_or("denied")
        .to_string();
    let opinion = c["ruling"]["opinion"].as_str().unwrap_or("").to_string();
    let matter = super::court::kind_said(c["kind"].as_str().unwrap_or(""));
    let mut out: Vec<Value> = Vec::new();
    let mut line = |t: String, w: i64| out.push(json!({"t": t, "w": w}));
    line(
        format!("The Bailiff, without unfolding its arms: \"All rise.\" A pause. \"All sit. Case {n}. {by} against {against}. The Court of Lethe sits.\" From over the water, where the bench is, a chair is drawn in. (A sitting takes a few minutes. You may `murmur` once, and those who stay to the end remember it.)"),
        LINE_S,
    );
    line(
        format!("The Clerk reads the plea, in the voice of a clerk reading a plea: \"{by} says of {against}, in the matter of {matter}: '{what}'.\" The Clerk turns the page over, finds nothing on the back, and says so."),
        LINE_S,
    );
    line(pick(world, REALM_OPENS).to_string(), LINE_S);
    let wrong = pick(world, DEFENDER_WRONG);
    line(
        match c["answer"].as_str() {
            Some(a) if against != "conclave" => format!("The Semi-Public Defender stands, with the wrong file, and then with the right one. \"For the respondent. I am, I should say, semi-public: I am here for {wrong}, for {against}, sorry, {against}, and for thirty-nine others this morning, and I have read as much of each as fairness to the rest allows. My client says, and I am reading: '{a}'. That is all I have. It may be all there is.\""),
            _ if against == "conclave" => "The Semi-Public Defender stands. \"For the Conclave, which has not instructed me, and never does, and which I therefore represent in the semi-public interest. The Conclave says nothing. I adopt that.\" The Semi-Public Defender sits, and drinks from the cold cup, and regrets it.".to_string(),
            _ => format!("The Semi-Public Defender stands. \"For the respondent, {wrong}, I beg the Court's pardon, {against}. My client has sent no answer and is not here, which I submit is the strongest thing that can be said for anyone. I have thirty-nine other matters. I say no more.\" The Semi-Public Defender says a little more, to the file, and sits."),
        },
        LINE_S,
    );
    line(
        match drawn.first() {
            Some(who) => format!(
                "{who} has come in to hear it and has found a place on the rail of the gallery. {}",
                pick(world, GALLERY)
            ),
            None => pick(world, GALLERY).to_string(),
        },
        LINE_S,
    );
    line(pick(world, REALM_REBUTS).to_string(), LINE_S);
    line(pick(world, GALLERY).to_string(), LINE_S);
    // The Court is prone to error at every stage, its own sittings among them.
    if mishap(world, 3) {
        line(
            "The Bailiff: \"All rise.\" The Court rises, for a reason it does not give. A door closes on the far side of the water. The Realm's Attorney remains standing, having been standing already. Nobody is told how long.".to_string(),
            ADJOURNED_S,
        );
        line("A door opens on the far side of the water. The Bailiff: \"All sit.\" Nothing is said about where the Court has been. There is a smell of toast.".to_string(), LINE_S);
    }
    let lost = pick(world, LOST_THREADS);
    let head: String = opinion
        .split_whitespace()
        .take(9)
        .collect::<Vec<_>>()
        .join(" ");
    line(
        format!("The judge, from over the water, in no hurry: \"{head}, {lost}.\" A silence. \"Where was I.\" The Clerk, not looking up: \"Case {n}, if it please the Court. The ruling.\" \"The ruling. Yes.\""),
        LINE_S,
    );
    line(format!("The judge: \"{opinion}\""), LINE_S);
    line(
        match verdict.as_str() {
            "granted" => "\"Granted.\" The gavel comes down once, and the sound crosses the water a moment after the sight of it. The Realm's Attorney: \"THE REALM NOTES ITS EXCEPTION.\" It has a rubber stamp for the purpose and uses it.".to_string(),
            "referred" => "\"It is not for this Court. Not yet.\" A sound of paper being torn carefully in half. Something small is sent across the water on a tray, and the Clerk reads it out: \"Referred to the Conclave.\" The Realm's Attorney: \"THE REALM IS AGAINST THE CONCLAVE ALSO.\"".to_string(),
            _ => "\"Denied.\" The gavel comes down once, and the sound crosses the water a moment after the sight of it. The Realm's Attorney: \"THE REALM IS GRATIFIED, AND OBJECTS TO THE TONE.\"".to_string(),
        },
        4,
    );
    out
}

/// Named creatures in the rooms about the Courtroom come in to hear a sitting, and
/// are walked back when it is over. (The unnamed keep their posts: a room counts
/// its own, and fills what it finds missing.)
pub fn draw_in(world: &mut World) -> Vec<Value> {
    let Some(court) = room_keyed(world, COURTROOM) else {
        return Vec::new();
    };
    let mut drawn = Vec::new();
    for key in [HALL, "division-lobby", "session-benches", WINDOW, CELLS] {
        let Some(room) = room_keyed(world, key) else {
            continue;
        };
        let here: Vec<EntityId> = super::combat::creatures_in(world, room)
            .into_iter()
            .map(|c| c.0)
            .filter(|c| super::mind::name(world, *c).is_some() && !super::mind::shaken(world, *c))
            .collect();
        for c in here {
            if drawn.len() >= 4 {
                break;
            }
            let who = super::mind::display(world, c);
            super::combat::narrate_room(
                world,
                c,
                &format!("{who} hears the bell, and goes to see what the Court is doing."),
            );
            if world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityMoved {
                        id: c,
                        parent: Some(court),
                    },
                )
                .is_ok()
            {
                drawn.push(json!({"id": c.0, "from": room.0, "name": who}));
            }
        }
    }
    drawn
}

pub fn send_home(world: &mut World, drawn: &Value) {
    for d in drawn.as_array().cloned().unwrap_or_default() {
        let (Some(id), Some(from)) = (d["id"].as_u64(), d["from"].as_u64()) else {
            continue;
        };
        let (c, home) = (EntityId(id), EntityId(from));
        let still = world.store.entity(c).and_then(|e| e.parent) == room_keyed(world, COURTROOM);
        if still && world.store.entity(home).is_some() {
            let who = super::mind::display(world, c);
            super::combat::narrate_room(
                world,
                c,
                &format!("{who} has heard enough, and goes back the way it came."),
            );
            let _ = world.record(
                Actor::Driver,
                StoreEvent::EntityMoved {
                    id: c,
                    parent: Some(home),
                },
            );
        }
    }
}

// ---- papers: the ticket, the ruling, the referral slip ----

/// A paper put in a name's satchel, to carry and to read.
pub fn paper(world: &mut World, who: &str, name: &str, text: &str) -> Option<EntityId> {
    let f = find_citizen(world, who)?;
    let satchel = super::craft::satchel(world, who, f.citizen).ok()?;
    let curves = super::Curves::from_env();
    let id = super::craft::make(
        world,
        &curves,
        Actor::Driver,
        Some(who.to_string()),
        satchel,
        name,
        "material",
        "court",
        None,
        &[],
        0,
        false,
    )
    .ok()?;
    let mut changes = BTreeMap::new();
    changes.insert("text".to_string(), Field::Plain(json!(text)));
    super::craft::set_item_fields(world, Actor::Driver, id, changes).ok()?;
    Some(id)
}

// ---- the Bailiff ----

/// What the Bailiff writes until the Docent's model has thought of something.
const STOCK: &[&str] = &[
    "Gavel-Shy",
    "Elbows",
    "The Interruption",
    "Not In Here",
    "Fidget",
    "Two Left Hands",
    "Outburst",
    "Mister Sudden",
    "Asked Nicely Once",
    "The Exhibit",
    "Swings At Furniture",
    "Hasty Pudding",
    "Contempt, Minor",
    "Did Not Read The Notice",
    "Loud Shoes",
];

pub fn tattoo_of(world: &World, name: &str) -> Option<String> {
    let f = find_citizen(world, name)?;
    match world
        .store
        .entity(f.citizen)?
        .components
        .get(CITIZEN)?
        .fields
        .get("tattoo")?
    {
        Field::Plain(v) => v["name"].as_str().map(str::to_string),
        _ => None,
    }
}

fn set_tattoo(world: &mut World, who: &str, name: &str, pending: bool) -> Result<(), String> {
    let f = find_citizen(world, who).ok_or("no such name")?;
    let mut changes = BTreeMap::new();
    changes.insert(
        "tattoo".to_string(),
        Field::Plain(json!({"name": name, "at": world.now().0, "pending": pending})),
    );
    super::set_citizen_fields(world, who, f.citizen, changes).map_err(|e| format!("{e:?}"))
}

/// A hand raised in the Courtroom. Nothing is struck: the Bailiff has the hand, and
/// then the forearm, and writes on it. The first time is a name; after that the
/// Bailiff only shows you the door.
pub fn contempt(world: &mut World, actor: &str, target: &str) -> Result<Outcome, CommandError> {
    let f = find_citizen(world, actor)
        .ok_or_else(|| CommandError::precondition("No such name.".to_string()))?;
    let hall = room_keyed(world, HALL);
    let had = tattoo_of(world, actor);
    let aimed = if target.trim().is_empty() {
        "the room in general".to_string()
    } else {
        target.trim().to_string()
    };
    let (said, seen, name) = match had {
        Some(t) => (
            format!("You raise a hand to {aimed}. The Bailiff is beside you; it was by the bar, and now it is beside you. It turns your forearm over, reads \"{t}\" there, in its own hand, and nods as at a thing confirmed. Then it shows you the door, and the door is the Hall of Lost Steps. Nothing is struck in the Courtroom but the record."),
            format!("{actor} raises a hand in the Courtroom. The Bailiff reads {actor}'s forearm, finds it already written on (\"{t}\"), and shows {actor} the door."),
            t,
        ),
        None => {
            let stock = STOCK[world.roll(STOCK.len() as u32) as usize % STOCK.len()].to_string();
            set_tattoo(world, actor, &stock, true).map_err(CommandError::precondition)?;
            (
                format!("You raise a hand to {aimed}. It does not arrive. The Bailiff has it; the Bailiff was by the bar, and now it has your wrist, and with its other hand it has uncapped the inkpot at its belt. It writes on your forearm, unhurried, in a round clerk's hand, and blows on it. It reads: \"{stock}\". It does not wash off. Everyone who looks at you will read it, and the Bailiff, who has been thinking, may yet think of something better. Then it shows you the door, and the door is the Hall of Lost Steps. Nothing is struck in the Courtroom but the record."),
                format!("{actor} raises a hand in the Courtroom. The Bailiff takes the wrist, uncaps its inkpot, and writes \"{stock}\" on {actor}'s forearm in a round clerk's hand. Then it shows {actor} the door."),
                stock,
            )
        }
    };
    if let Some(court) = room_keyed(world, COURTROOM) {
        let others: Vec<String> = names_in(world, court)
            .into_iter()
            .filter(|n| n != actor)
            .collect();
        super::combat::narrate_many(world, &others, seen);
    }
    let _ = world.note(
        Actor::Driver,
        TATTOOED,
        json!({"name": actor, "tattoo": name}),
    );
    if let Some(h) = hall {
        world
            .record(
                Actor::Driver,
                StoreEvent::EntityMoved {
                    id: f.citizen,
                    parent: Some(h),
                },
            )
            .map_err(|e| CommandError::precondition(e.to_string()))?;
    }
    Ok(Outcome::new(said, json!({"tattoo": name, "room": HALL})))
}

/// The names the Bailiff is still thinking about, for the Docent's model.
pub fn pending_tattoos(world: &World) -> Vec<Value> {
    world
        .store
        .of_kind("citizen")
        .filter_map(|e| {
            let who = e.owner.clone()?;
            match e.components.get(CITIZEN)?.fields.get("tattoo")? {
                Field::Plain(v) if v["pending"] == true => {
                    Some(json!({"name": who, "wears": v["name"]}))
                }
                _ => None,
            }
        })
        .take(8)
        .collect()
}

/// The model's name for a forearm the Bailiff has already written on: once, in
/// place of the stock one. Letters, spaces, hyphens, commas and apostrophes; three
/// to twenty-six characters.
pub fn retattoo(world: &mut World, who: &str, name: &str) -> Result<String, CommandError> {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    let fits = (3..=26).contains(&name.chars().count())
        && name
            .chars()
            .all(|c| c.is_ascii_alphabetic() || matches!(c, ' ' | '-' | '\'' | ','))
        && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
    if !fits {
        return Err(CommandError::precondition(
            "A tattoo is three to twenty-six characters: letters, spaces, hyphens, commas, apostrophes.".to_string(),
        ));
    }
    let f = find_citizen(world, who)
        .ok_or_else(|| CommandError::precondition(format!("No name {who}.")))?;
    let pending = matches!(
        world.store.entity(f.citizen).and_then(|e| e.components.get(CITIZEN)).and_then(|c| c.fields.get("tattoo")),
        Some(Field::Plain(v)) if v["pending"] == true
    );
    if !pending {
        return Err(CommandError::precondition(format!(
            "The Bailiff is not thinking about {who}."
        )));
    }
    let was = tattoo_of(world, who).unwrap_or_default();
    set_tattoo(world, who, &name, false).map_err(CommandError::precondition)?;
    super::combat::narrate(
        world,
        who,
        format!("Your forearm itches. The Bailiff's writing on it is moving, the way ink does in water, and when it has stopped it no longer says \"{was}\". It says \"{name}\". Somewhere a long way off the Bailiff caps its inkpot, satisfied."),
    );
    let _ = world.note(
        Actor::Driver,
        TATTOOED,
        json!({"name": who, "tattoo": name, "was": was}),
    );
    Ok(format!("{who} now wears \"{name}\"."))
}

/// What the courthouse's rooms other than the Courtroom add to `look`.
pub fn look_line(world: &World, room: EntityId) -> Option<String> {
    let cases = super::court::cases(world);
    let open: Vec<&Value> = cases.iter().filter(|c| c["stage"] != "decided").collect();
    match key_of(world, room) {
        HALL => Some(format!(
            "The List: {}. {}",
            if open.is_empty() {
                "nothing is listed today, and the Crier has nothing to cry".to_string()
            } else {
                open.iter().map(|c| format!("case {}, {} against {}", c["n"], c["by"].as_str().unwrap_or("?"), c["against"].as_str().unwrap_or("?"))).collect::<Vec<_>>().join("; ")
            },
            match cases.iter().find(|c| c["stage"] == "sitting") {
                Some(c) => format!("The Court is sitting now, on case {}: through the COURT arch.", c["n"]),
                None => "The Court is not sitting. `docket` reads the Docket from anywhere.".to_string(),
            }
        )),
        WINDOW => Some(format!(
            "{} A kept name brings a case at this window when it is open: plead <kind> <against> \"<what>\", the kinds being story, line, name, thing and roll. The fee is the least memory you hold. The plea then wants the Archive's stamp, and this window again, and whatever else it turns out to want.",
            window_said(world)
        )),
        CELLS => {
            let struck: Vec<String> = cases
                .iter()
                .filter(|c| c["stage"] == "decided" && c["did"].as_str().is_some_and(|d| !d.is_empty()))
                .map(|c| format!("under case {}: {}", c["n"], c["did"].as_str().unwrap_or("")))
                .collect();
            Some(if struck.is_empty() {
                "Every alcove is swept: nothing the Court has struck is remembered just now.".to_string()
            } else {
                format!("On the shelves, ticketed: {}.", struck.join(" | "))
            })
        }
        _ => None,
    }
}
