//! The Conclave: the legislature. The Clocktower is the hand on the rates, moving a
//! step an hour toward a band; the Conclave writes the *rules*, the statutes that say
//! what a name may do with memory and what it costs to do it. Statutes are numbers with
//! names on the world's `disremembered.conclave`: a gift allowance and the tax on gifts
//! over it, the share of all held memory above which a name's own new memory thins
//! faster (and by how much), how much may sit on a shelf. The code that moves memory
//! reads them where it moves it, the way it reads the Clocktower's rates.
//!
//! A statute changes by enactment: today an admin's `enact`, tomorrow whatever the
//! Conclave becomes (a vote of the regulars, a sitting with a policy of its own). Every
//! enactment is a public event, `disremembered.statute`, said in prose to everyone
//! present, and `statutes` reads the book aloud to anyone. Once a day the Conclave
//! *sits*: it reads the Clocktower's ledger, names the hoards, records the day, and
//! for now changes nothing; the sitting is where a policy will go. This is scaffolding
//! for an economy with laws in it, and it is meant to be read.
//!
//! Petitions: a kept name at the Petitioners' Gallery takes a ticket from the machine
//! and lays a petition upon the table (`petition "…"`). The Conclave divides on the
//! half hour and moves each petition one stage in its order: read into the minutes,
//! referred to the Committee on the Matter, reported (or tabled, the Committee having
//! run out of ink), put to the division and decided. Granted, a petition that names a
//! statute and a direction moves the statute; any other becomes a Resolution read at
//! the Lectern. Denied, it is recorded in full, that it may be seen to have been
//! denied in full. Either way the petitioner is told wherever they are, and keeps a
//! memory of it. The Minute Room's newest volume holds every line; `read` there reads
//! it. The Conclave always decides, eventually. (A visitor climbed eleven rooms of
//! good prose and found nothing to read and no one to ask; the owner's north star for
//! the house: glorious and bureaucratic at once.)

use super::plain_num;
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The book: `{statutes: {name: value}, hoards: {name: share}, sat_at, sittings}`.
pub const CONCLAVE: &str = "disremembered.conclave";
/// A statute changed: `{statute, from, to, by, said}`. Public.
pub const STATUTE: &str = "disremembered.statute";
/// The Conclave sat: `{hoards, dashboard, said}`. Public.
pub const SAT: &str = "disremembered.conclave_sat";
/// The job.
pub const JOB_SITTING: &str = "disremembered.conclave";
/// It sits once a day, at midnight by the world's clock.
pub const SIT_EVERY_MS: i64 = 86_400_000;
/// A petition was laid upon the table: `{ticket, by, text, kind, statute}`. Public.
pub const PETITION: &str = "disremembered.petition";
/// A petition moved a stage at a division: `{ticket, by, stage, tabled, said}`. Public.
pub const PETITION_MOVED: &str = "disremembered.petition_moved";
/// A petition was decided: `{ticket, by, text, granted, ruling, effect}`. Public.
pub const DECIDED: &str = "disremembered.petition_decided";
/// The division job: at half past each hour, between strikes.
pub const JOB_DIVISION: &str = "disremembered.division";
pub const DIVIDE_EVERY_MS: i64 = 3_600_000;
const DIVIDE_AT_MS: i64 = 30 * 60 * 1000;
const PETITION_MIN: usize = 12;
const PETITION_MAX: usize = 280;
const RESOLUTIONS_KEPT: usize = 12;
const MINUTES_KEPT: usize = 24;
const DECIDED_KEPT: usize = 20;
/// What a decision is worth to the petitioner: a memory of being heard, long-lived.
const GRANTED_XP: f64 = 30.0;
const DENIED_XP: f64 = 12.0;
const DECISION_HALF_LIFE_S: i64 = 6 * 3600;
/// The stages a petition climbs, one a division.
const STAGES: &[&str] = &[
    "upon the table",
    "read into the minutes",
    "before the Committee on the Matter",
    "reported from committee",
    "decided",
];
/// Why the Committee did not report this hour.
const TABLED: &[&str] = &[
    "the Committee having run out of ink",
    "a robe in the Robing Room having straightened unexpectedly",
    "the mace having been found a quarter of an inch from true",
    "the question having been raised whether the question had been raised",
    "the clerk having gone to find the tally book, which was under the tally book",
    "the bell for order having been rung, by no one, and the ringing having to be minuted",
];

/// A statute: its name, its default, its bounds, and what it governs in a sentence.
pub struct StatuteDef {
    pub name: &'static str,
    pub default: f64,
    pub low: f64,
    pub high: f64,
    pub what: &'static str,
}

pub const STATUTES: &[StatuteDef] = &[
    StatuteDef {
        name: "gift_allowance",
        default: 120.0,
        low: 0.0,
        high: 100_000.0,
        what: "how much memory (XP) a name may give away in a day before the gift tax falls on what it gives",
    },
    StatuteDef {
        name: "gift_tax",
        default: 0.5,
        low: 0.05,
        high: 1.0,
        what: "what a gift over the allowance keeps of its half-life: at 0.5 it thins twice as fast in the receiver's hands",
    },
    StatuteDef {
        name: "gift_receipt",
        default: 240.0,
        low: 0.0,
        high: 100_000.0,
        what: "how much memory (XP) a name may receive in gifts in a day, from all givers together, before the gift tax falls on what it receives; free names could otherwise each give their allowance to one",
    },
    StatuteDef {
        name: "hoard_share",
        default: 0.4,
        low: 0.1,
        high: 1.0,
        what: "the share of all held memory above which a name is a hoard, and its new memory thins faster",
    },
    StatuteDef {
        name: "hoard_tax",
        default: 1.0,
        low: 0.0,
        high: 4.0,
        what: "how much faster a hoard's new memory thins at the limit: at 1.0, a name holding everything thins twice as fast, one holding just over the share barely more",
    },
    StatuteDef {
        name: "shelf_limit",
        default: 2000.0,
        low: 0.0,
        high: 1_000_000.0,
        what: "how much memory (XP) may sit on one name's shelf (the palace); a shelf at the limit takes nothing more",
    },
];

fn def(name: &str) -> Option<&'static StatuteDef> {
    STATUTES.iter().find(|s| s.name == name)
}

fn entity(world: &World) -> Option<EntityId> {
    world
        .store
        .entities()
        .find(|e| e.kind == "conclave")
        .map(|e| e.id)
}

fn book(world: &World, field: &str) -> Option<Value> {
    let id = entity(world)?;
    match world
        .store
        .entity(id)?
        .components
        .get(CONCLAVE)?
        .fields
        .get(field)?
    {
        Field::Plain(v) => Some(v.clone()),
        _ => None,
    }
}

/// A statute's value: what is enacted, else its default.
pub fn statute(world: &World, name: &str) -> f64 {
    let d = def(name);
    book(world, "statutes")
        .and_then(|s| s[name].as_f64())
        .or_else(|| d.map(|d| d.default))
        .map(|v| match d {
            Some(d) => v.clamp(d.low, d.high),
            None => v,
        })
        .unwrap_or(0.0)
}

/// Every statute.
pub fn statutes(world: &World) -> BTreeMap<String, f64> {
    STATUTES
        .iter()
        .map(|s| (s.name.to_string(), statute(world, s.name)))
        .collect()
}

fn write(world: &mut World, changes: BTreeMap<String, Value>) -> Result<EntityId, String> {
    let id = match entity(world) {
        Some(id) => id,
        None => {
            let id = world.store.allocate_id();
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityCreated {
                        id,
                        kind: "conclave".into(),
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
        .and_then(|e| e.components.get(CONCLAVE))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    fields
        .entry("statutes".into())
        .or_insert(Field::Plain(json!({})));
    fields
        .entry("hoards".into())
        .or_insert(Field::Plain(json!({})));
    for k in ["petitions", "resolutions", "minutes", "decided"] {
        fields.entry(k.into()).or_insert(Field::Plain(json!([])));
    }
    for (k, v) in changes {
        fields.insert(k, Field::Plain(v));
    }
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: CONCLAVE.into(),
                fields,
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(id)
}

/// At boot: the book exists, and the sitting is wound.
pub fn ensure(world: &mut World) -> Result<(), String> {
    if entity(world).is_none() {
        write(world, BTreeMap::new())?;
    }
    wind(world)?;
    wind_division(world)
}

fn wind(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    let due = foxing_time::Timestamp((now / SIT_EVERY_MS + 1) * SIT_EVERY_MS);
    world
        .schedule_once(JOB_SITTING, due, json!({}), |_| true)
        .map(|_| ())
}

/// The next half hour.
fn wind_division(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    let mut due = (now / DIVIDE_EVERY_MS) * DIVIDE_EVERY_MS + DIVIDE_AT_MS;
    if due <= now {
        due += DIVIDE_EVERY_MS;
    }
    world
        .schedule_once(JOB_DIVISION, foxing_time::Timestamp(due), json!({}), |_| {
            true
        })
        .map(|_| ())
}

/// Enact a statute: the book changes, the record says so, everyone present hears it.
pub fn enact(world: &mut World, by: &str, name: &str, to: f64) -> Result<String, CommandError> {
    let d = def(name).ok_or_else(|| {
        CommandError::precondition(format!(
            "There is no statute called {name}. The book has: {}.",
            STATUTES
                .iter()
                .map(|s| s.name)
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })?;
    if !(d.low..=d.high).contains(&to) || !to.is_finite() {
        return Err(CommandError::precondition(format!(
            "{name} stays between {} and {}.",
            d.low, d.high
        )));
    }
    let from = statute(world, name);
    let mut all = book(world, "statutes").unwrap_or_else(|| json!({}));
    all[name] = json!(to);
    let mut changes = BTreeMap::new();
    changes.insert("statutes".into(), all);
    write(world, changes).map_err(CommandError::internal)?;
    let said = prose(name, from, to);
    let _ = world.note(
        Actor::Driver,
        STATUTE,
        json!({"statute": name, "from": from, "to": to, "by": by, "said": said}),
    );
    tell_everyone(world, &said);
    world.notice(format!("{said} (enacted by {by})"));
    Ok(said)
}

/// The Conclave's own words for an enactment, in its voice. (The Architect's lines:
/// design/conclave.md. gift_tax is what a taxed gift keeps, so a rise is the lighter
/// tax.)
fn prose(name: &str, from: f64, to: f64) -> String {
    let up = to > from;
    let pct = |v: f64| format!("{:.0}%", v * 100.0);
    match (name, up) {
        ("gift_allowance", true) => format!("By statute, the Conclave enlarges the day's gift allowance from {from:.0} to {to:.0}. A name may now be generous a little further before the Conclave takes notice."),
        ("gift_allowance", false) => format!("By statute, the Conclave narrows the day's gift allowance from {from:.0} to {to:.0}. Generosity beyond it is not forbidden. It is noted."),
        ("gift_receipt", true) => format!("By statute, the Conclave enlarges a name's share of gifts received from {from:.0} to {to:.0} a day. A name may now be given to a little further before the Conclave takes notice."),
        ("gift_receipt", false) => format!("By statute, the Conclave narrows a name's share of gifts received from {from:.0} to {to:.0} a day. Beyond it, what is given thins in the hands that take it, whoever gave."),
        ("gift_tax", true) => format!("The Conclave relents. A gift past the allowance now keeps {} of its half-life, where it kept {}. The Conclave was always merciful; the Book now says so.", pct(to), pct(from)),
        ("gift_tax", false) => format!("The Conclave tightens the gift tax. A gift past the allowance now keeps {} of its half-life, where it kept {}, and thins the faster in the hands that take it. The Conclave regrets nothing, and has entered that it regrets nothing.", pct(to), pct(from)),
        ("hoard_share", true) => format!("The Conclave raises the mark of a hoard from {} to {} of all that is held. Fewer names will be found upon the Roll. The Conclave has not grown lenient. The world has grown worthy.", pct(from), pct(to)),
        ("hoard_share", false) => format!("The Conclave lowers the mark of a hoard from {} to {} of all that is held. Names that were merely large are, in the eyes of the statute, large.", pct(from), pct(to)),
        ("hoard_tax", true) => format!("The Conclave raises the hoard tax from {from:.2} to {to:.2}. A hoard's new memory will thin the faster. The Conclave takes no pleasure in this, and records that it took none."),
        ("hoard_tax", false) => format!("The Conclave eases the hoard tax from {from:.2} to {to:.2}. A hoard's new memory will thin more slowly. This is not forgiveness. The Roll remembers."),
        ("shelf_limit", true) => format!("The Conclave lengthens the shelf. One palace may now hold {to:.0}, where it held {from:.0}. The Conclave trusts the shelf to bear it."),
        ("shelf_limit", false) => format!("The Conclave shortens the shelf from {from:.0} to {to:.0}. What already sits past the limit stays where it sits, and the shelf takes nothing more. The Conclave does not reach onto shelves."),
        _ => format!("The Conclave moves {name} from {from} to {to}."),
    }
}

fn tell_everyone(world: &mut World, line: &str) {
    for actor in present(world, None) {
        super::combat::narrate(world, &actor, line.to_string());
    }
}

/// The rooms of the Conclave, by key.
pub const KEYS: &[&str] = &[
    "escapement-stair",
    "petition-gallery",
    "robing-room",
    "tally-room",
    "brass-rail",
    "book-lectern",
    "roll-hall",
    "session-benches",
    "division-lobby",
    "minute-room",
    "engrossing-room",
];

/// What a room of the Conclave shows beyond its prose: the Roll of Hoards, chalked on
/// its slate; the Book's standing at the Lectern. Nothing elsewhere.
pub fn room_note(world: &World, room: EntityId) -> Option<String> {
    match super::plain_str(world, room, super::ROOM, "key")? {
        "roll-hall" => {
            let hoards = book(world, "hoards").unwrap_or_else(|| json!({}));
            let names: Vec<String> = hoards
                .as_object()
                .map(|m| {
                    m.iter()
                        .map(|(n, s)| format!("{n}, {:.0}%", s.as_f64().unwrap_or(0.0) * 100.0))
                        .collect()
                })
                .unwrap_or_default();
            Some(if names.is_empty() {
                "The slate is clean. The chalk is new.".to_string()
            } else {
                format!("Chalked on the slate: {}.", names.join("; "))
            })
        }
        "book-lectern" => Some(format!(
            "The Book lies open at {} statutes; `statutes` reads them.",
            STATUTES.len()
        )),
        _ => None,
    }
}

/// Whether a room is one of the Conclave's.
pub fn is_house(world: &World, room: EntityId) -> bool {
    super::plain_str(world, room, super::ROOM, "key").is_some_and(|k| KEYS.contains(&k))
}

/// Everyone here: in the house (Some(true)), outside it (Some(false)), or anywhere.
fn present(world: &World, in_house: Option<bool>) -> Vec<String> {
    world
        .store
        .entities()
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter(|e| match in_house {
            None => true,
            Some(want) => e.parent.is_some_and(|r| is_house(world, r)) == want,
        })
        .filter_map(|e| e.owner.clone())
        .collect()
}

// ---- the statutes at work ----

/// The hoards, as the Clocktower last read them: `{name: share}` for every name above
/// `hoard_share`. Written at each strike, read where memory is earned.
pub fn note_hoards(world: &mut World, dash: &Value) -> Result<(), String> {
    let share = statute(world, "hoard_share");
    let mut hoards = serde_json::Map::new();
    if let Some(list) = dash["holders"].as_array() {
        for h in list {
            if let (Some(n), Some(s)) = (h["name"].as_str(), h["share"].as_f64()) {
                if s > share {
                    hoards.insert(n.to_string(), json!((s * 1000.0).round() / 1000.0));
                }
            }
        }
    }
    let mut changes = BTreeMap::new();
    changes.insert("hoards".into(), Value::Object(hoards));
    write(world, changes).map(|_| ())
}

/// A hoard's share, if `actor` is one.
pub fn hoard_share_of(world: &World, actor: &str) -> Option<f64> {
    book(world, "hoards").and_then(|h| h[actor].as_f64())
}

/// How much faster `actor`'s new memory thins: 1.0 for anyone under the share; for a
/// hoard, 1.0 + hoard_tax scaled by how far over the share they are, so a name just
/// over barely feels it and a name holding everything thins at (1 + hoard_tax) times
/// the pace. Progressive, and on the hoard alone: the Clocktower's flat thinning hit
/// everyone with a chunk to lose.
pub fn hoard_factor(world: &World, actor: &str) -> f64 {
    let Some(share) = hoard_share_of(world, actor) else {
        return 1.0;
    };
    let low = statute(world, "hoard_share");
    let tax = statute(world, "hoard_tax");
    if share <= low || low >= 1.0 {
        return 1.0;
    }
    1.0 + tax * ((share - low) / (1.0 - low)).clamp(0.0, 1.0)
}

/// What `actor` has given away in the last day, by the record's `disremembered.given`.
pub fn given_today(world: &World, actor: &str) -> f64 {
    let day_ago = world.now().0 - 86_400_000;
    world
        .recent_events(5000)
        .iter()
        .filter(|e| e.kind == super::exchange::GIVEN && e.at.0 >= day_ago)
        .filter(|e| e.payload["from"] == actor)
        .map(|e| e.payload["size"].as_f64().unwrap_or(0.0))
        .sum()
}

/// What `actor` has received in gifts in the last day, by the same record.
pub fn received_today(world: &World, actor: &str) -> f64 {
    let day_ago = world.now().0 - 86_400_000;
    world
        .recent_events(5000)
        .iter()
        .filter(|e| e.kind == super::exchange::GIVEN && e.at.0 >= day_ago)
        .filter(|e| e.payload["to"] == actor)
        .map(|e| e.payload["size"].as_f64().unwrap_or(0.0))
        .sum()
}

/// What is on `actor`'s shelf now.
pub fn shelved(world: &World, palace: EntityId) -> f64 {
    super::memory::level(world, palace)
}

// ---- the sitting ----

/// The Conclave sits: reads the ledger, names the hoards, records the day. It changes
/// nothing yet; the policy goes here when there is one.
pub fn sit(world: &mut World) -> Result<(), String> {
    let dash = super::economy::dashboard(world);
    note_hoards(world, &dash)?;
    let hoards = book(world, "hoards").unwrap_or_else(|| json!({}));
    let named: Vec<(String, f64)> = hoards
        .as_object()
        .map(|m| {
            m.iter()
                .map(|(n, s)| (n.clone(), s.as_f64().unwrap_or(0.0)))
                .collect()
        })
        .unwrap_or_default();
    let sittings = entity(world)
        .and_then(|e| plain_num(world, e, CONCLAVE, "sittings"))
        .unwrap_or(0.0)
        + 1.0;
    let mut changes = BTreeMap::new();
    changes.insert("sat_at".into(), json!(world.now().0));
    changes.insert("sittings".into(), json!(sittings));
    write(world, changes)?;
    // To the world, one line, as the Clocktower's strike is heard.
    let said = match named.len() {
        0 => "The Conclave has sat. It has read the Ledger and finds no hoard in the world. It records that the statutes have been obeyed, and takes no credit, and enters that it took none.".to_string(),
        1 => format!("The Conclave has sat. It names one hoard, {}, holding {:.0}% of all that is held, and enters the name upon the Roll. The statute takes its course.", named[0].0, named[0].1 * 100.0),
        n => format!("The Conclave has sat. It names {n} hoards upon the Roll: {}. The Conclave notes the number with gravity, and in full.", named.iter().map(|(n, s)| format!("{n} ({:.0}%)", s * 100.0)).collect::<Vec<_>>().join(", ")),
    };
    // In the house, the ceremony, line by line: tomorrow's volume in the Minute Room,
    // with the blanks filled in.
    let held = dash["held"].as_f64().unwrap_or(0.0);
    let regulars = dash["regulars"].as_u64().unwrap_or(0);
    let standing = STATUTES
        .iter()
        .map(|s| {
            format!(
                "{} at {}",
                s.name.replace('_', " "),
                trim(statute(world, s.name))
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let naming = match named.len() {
        0 => "The Conclave proceeds to the naming of hoards. There is no hoard. The chalk is returned to its tray, unbroken, and the Conclave records the fact with satisfaction.".to_string(),
        _ => format!("The Conclave proceeds to the naming of hoards. {}. The names are entered upon the Roll, and the chalk is not spared.", named.iter().map(|(n, s)| format!("{n}, holding {:.0}% of all that is held", s * 100.0)).collect::<Vec<_>>().join("; ")),
    };
    let ceremony = [
        "Order. The Conclave is in session. It is midnight, because the Conclave has sat.".to_string(),
        "The Roll is called. The robes answer. A quorum is present, as a quorum is always present.".to_string(),
        format!("The Ledger of the Clocktower is laid before the Conclave. The Conclave reads that {held:.0} is held in all the world, and that {regulars} names have spoken their twenty lines and done a thing this day, and have standing before it."),
        format!("The Book is opened at the Lectern. The statutes that stand are these: {standing}. The Conclave has examined them and finds them sound. The Conclave wrote them."),
        naming,
        match attendance(world).len() {
            0 => "The attendance is read. No name attended. The benches are noted, and are empty.".to_string(),
            _ => format!("The attendance is read: {}. Their names are entered in the minutes, that they were here.", attendance(world).join(", ")),
        },
        table_line(world),
        resolutions_line(world),
        "The minutes are engrossed. The Conclave rises. The Conclave remains.".to_string(),
    ];
    minute(world, "sitting", ceremony.to_vec())?;
    let mut changes = BTreeMap::new();
    changes.insert("attendance".into(), json!([]));
    write(world, changes)?;
    let _ = world.note(
        Actor::Driver,
        SAT,
        json!({"said": said, "ceremony": ceremony, "hoards": hoards, "statutes": statutes(world), "dashboard": dash}),
    );
    for actor in present(world, Some(true)) {
        for line in &ceremony {
            super::combat::narrate(world, &actor, line.clone());
        }
    }
    for actor in present(world, Some(false)) {
        super::combat::narrate(world, &actor, said.clone());
    }
    world.notice(said);
    wind(world)
}

// ---- petitions ----

fn petitions(world: &World) -> Vec<Value> {
    book(world, "petitions")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

fn resolutions(world: &World) -> Vec<Value> {
    book(world, "resolutions")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

fn stage_name(stage: u64) -> &'static str {
    STAGES[(stage as usize).min(STAGES.len() - 1)]
}

/// A statute the text names, and whether it asks it raised: the words "gift
/// allowance" or "gift_allowance" with raise/enlarge/lengthen/more/higher, or
/// lower/narrow/shorten/less/tighten/ease.
fn statute_asked(text: &str) -> Option<(&'static str, bool)> {
    let low = text.to_lowercase().replace('_', " ");
    let named = STATUTES
        .iter()
        .find(|s| low.contains(&s.name.replace('_', " ")))?;
    let up = [
        "raise", "enlarge", "lengthen", "more", "higher", "increase", "widen", "relent", "loosen",
    ]
    .iter()
    .any(|w| low.contains(w));
    let down = [
        "lower", "narrow", "shorten", "less", "tighten", "reduce", "decrease", "cut", "halve",
    ]
    .iter()
    .any(|w| low.contains(w));
    match (up, down) {
        (true, false) => Some((named.name, true)),
        (false, true) => Some((named.name, false)),
        _ => None,
    }
}

/// A citizen's name the text carries as a word, other than the petitioner's own.
fn names_a_citizen(world: &World, text: &str, me: &str) -> Option<String> {
    let words: std::collections::HashSet<String> = text
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() >= 3)
        .collect();
    world
        .store
        .entities()
        .filter(|e| e.kind == "citizen")
        .filter_map(|e| super::plain_str(world, e.id, super::CITIZEN, "name").map(str::to_string))
        .filter(|n| !n.eq_ignore_ascii_case(me))
        .find(|n| words.contains(&n.to_lowercase()))
}

/// The room the machine stands in.
fn gallery(world: &World) -> Option<EntityId> {
    world
        .store
        .entities()
        .find(|e| {
            e.kind == "room"
                && super::plain_str(world, e.id, super::ROOM, "key") == Some("petition-gallery")
        })
        .map(|e| e.id)
}

/// `petition "…"`: the ticket, the table, the record.
fn file(
    world: &mut World,
    ctx: &foxing_registry::CallContext,
    text: &str,
) -> Result<Outcome, CommandError> {
    super::kept_only(world, ctx, "petition the Conclave")?;
    let f = super::me(world, ctx)?;
    let here = super::rooms::room_of(world, f.citizen)
        .ok_or_else(|| super::internal("citizen has no room"))?;
    let Some(gallery) = gallery(world) else {
        return Err(CommandError::precondition(
            "The Petitioners' Gallery is not in this world.",
        ));
    };
    if here != gallery {
        let way = super::atlas::way_seeded(world, here, gallery)
            .map(|p| match p.len() {
                0 => "you are there".to_string(),
                1 => format!("one step: {}", p[0]),
                n => format!("{n} steps: {}", p.join(", ")),
            })
            .unwrap_or_else(|| "up the Clocktower and through the Escapement Stair".to_string());
        return Err(CommandError::precondition(format!(
            "Petitions are received in the Petitioners' Gallery, where the machine gives out tickets; from here, {way}."
        )));
    }
    let n = text.chars().count();
    if n < PETITION_MIN {
        return Err(CommandError::precondition(format!(
            "A petition is at least {PETITION_MIN} characters; the Conclave does not take a word."
        )));
    }
    if n > PETITION_MAX {
        return Err(CommandError::precondition(format!(
            "A petition is at most {PETITION_MAX} characters; this one is {n}. The clerk's hand tires."
        )));
    }
    // The refusals the Architect wrote and the owner kept (2026-11-02): no petition
    // aimed at a name. (No statute takes back held memory and no standing is for sale
    // by construction: a petition moves a statute within its bounds or enters a
    // Resolution, nothing else.)
    if let Some(named) = names_a_citizen(world, text, &ctx.actor) {
        return Err(CommandError::precondition(format!(
            "The Conclave takes no petition aimed at a name; this one names {named}. Ask for a rule, not a ruling on someone."
        )));
    }
    // A hoard's petition to ease its own tax never counts (the Architect's third
    // refusal): received and denied in the same breath, recorded in full.
    if let Some((statute, up)) = statute_asked(text) {
        let self_serving = matches!((statute, up), ("hoard_tax", false) | ("hoard_share", true));
        if self_serving && hoard_share_of(world, &ctx.actor).is_some() {
            let _ = world.note(
                Actor::Citizen(ctx.actor.clone()),
                DECIDED,
                json!({"by": ctx.actor, "text": text, "granted": false, "effect": "none", "why": "a hoard upon its own tax"}),
            );
            return Err(CommandError::precondition(format!(
                "The Conclave does not hear a hoard upon its own tax. Your name stands upon the Roll of Hoards; a petition to ease what the Roll lays on you is received and denied in the same breath, and recorded in full, that it may be seen to have been: \"{text}\"."
            )));
        }
    }
    let mut list = petitions(world);
    if let Some(p) = list.iter().find(|p| p["by"] == ctx.actor.as_str()) {
        return Err(CommandError::precondition(format!(
            "Your petition {} is still upon the table, {}: \"{}\". One at a time; the Conclave always decides, eventually.",
            p["ticket"],
            stage_name(p["stage"].as_u64().unwrap_or(0)),
            p["text"].as_str().unwrap_or("")
        )));
    }
    // The number on yours is always a little higher than the last one called.
    let last = book(world, "last_ticket")
        .and_then(|v| v.as_u64())
        .unwrap_or(41);
    let ticket = last + 1 + world.roll(3) as u64;
    let kind = statute_asked(text);
    let entry = json!({
        "ticket": ticket,
        "by": ctx.actor,
        "text": text,
        "at": world.now().0,
        "stage": 0,
        "tabled": 0,
        "kind": if kind.is_some() { "statute" } else { "resolution" },
        "statute": kind.map(|k| k.0),
        "up": kind.map(|k| k.1),
    });
    list.push(entry);
    let mut changes = BTreeMap::new();
    changes.insert("petitions".into(), Value::Array(list));
    changes.insert("last_ticket".into(), json!(ticket));
    write(world, changes).map_err(CommandError::internal)?;
    let _ = world.note(
        Actor::Citizen(ctx.actor.clone()),
        PETITION,
        json!({"ticket": ticket, "by": ctx.actor, "text": text, "kind": if kind.is_some() { "statute" } else { "resolution" }, "statute": kind.map(|k| k.0)}),
    );
    for other in present(world, Some(true)) {
        if other != ctx.actor {
            super::combat::narrate(
                world,
                &other,
                format!(
                    "{} takes a ticket from the machine and lays a petition upon the table.",
                    ctx.actor
                ),
            );
        }
    }
    let standing = match standing_of(world, &ctx.actor) {
        Some("regular") => " As a regular of this house, yours is taken first, and never tabled.",
        Some("attended") => " Having attended, yours is taken before those of names who did not.",
        _ => "",
    };
    let about = match kind {
        Some((name, up)) => format!(
            " It names the statute {name}, asking it {}; granted, the statute moves.",
            if up { "raised" } else { "lowered" }
        ),
        None => {
            " Granted, it becomes a Resolution of the Conclave, read at the Lectern.".to_string()
        }
    };
    Ok(Outcome::new(
        format!(
            "The machine gives out a ticket: number {ticket}. Your petition is received and lies upon the table: \"{text}\".{about}{standing} The Conclave divides at half past each hour (not the Clocktower's strike on the hour, which is another clock) and takes it up in its order, a stage at a time: {}; `petitions` says where it stands. The Conclave always decides, eventually, and will tell you wherever you are.",
            STAGES[1..].join(", then ")
        ),
        json!({"ticket": ticket, "text": text, "stage": 0, "next_division": next_division(world)}),
    ))
}

pub fn next_division(world: &World) -> i64 {
    let now = world.now().0;
    let mut due = (now / DIVIDE_EVERY_MS) * DIVIDE_EVERY_MS + DIVIDE_AT_MS;
    if due <= now {
        due += DIVIDE_EVERY_MS;
    }
    due
}

fn in_said(world: &World, at: i64) -> String {
    let ms = (at - world.now().0).max(0);
    if ms < 60_000 {
        "within the minute".to_string()
    } else {
        format!("in {}", super::combat::human(ms))
    }
}

/// `petition` alone: yours.
fn mine(world: &World, actor: &str) -> Outcome {
    match petitions(world).iter().find(|p| p["by"] == actor) {
        Some(p) => Outcome::new(
            format!(
                "Your petition {} is {}: \"{}\". The Conclave divides {}.",
                p["ticket"],
                stage_name(p["stage"].as_u64().unwrap_or(0)),
                p["text"].as_str().unwrap_or(""),
                in_said(world, next_division(world))
            ),
            json!({"petition": p}),
        ),
        None => Outcome::new(
            "You have no petition upon the table. At the Petitioners' Gallery, `petition \"…\"` lays one; the Conclave always decides, eventually.".to_string(),
            json!({"petition": Value::Null}),
        ),
    }
}

/// `petitions`: the table.
fn table(world: &World, actor: &str) -> Outcome {
    let list = petitions(world);
    let mut lines = Vec::new();
    if list.is_empty() {
        lines.push("No petition lies upon the table. The table is noted.".to_string());
    } else {
        lines.push(format!(
            "Upon the table, in the Conclave's order ({}):",
            list.len()
        ));
        for p in &list {
            let tabled = p["tabled"].as_u64().unwrap_or(0);
            lines.push(format!(
                "  ticket {}, of {}{}{}: {}{} — \"{}\"",
                p["ticket"],
                p["by"].as_str().unwrap_or("?"),
                if p["by"] == actor { " (yours)" } else { "" },
                match p["standing"].as_str() {
                    Some("regular") => " (a regular's, taken first)",
                    Some("attended") => " (attended, taken next)",
                    _ => "",
                },
                stage_name(p["stage"].as_u64().unwrap_or(0)),
                if tabled > 0 {
                    format!(
                        ", tabled {tabled} time{}",
                        if tabled == 1 { "" } else { "s" }
                    )
                } else {
                    String::new()
                },
                p["text"].as_str().unwrap_or("")
            ));
        }
    }
    let decided = book(world, "decided")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    if !decided.is_empty() {
        lines.push("Decided lately:".to_string());
        for d in decided.iter().rev().take(4) {
            lines.push(format!(
                "  ticket {}, of {}: {} — {}",
                d["ticket"],
                d["by"].as_str().unwrap_or("?"),
                if d["granted"] == true {
                    "granted"
                } else {
                    "not granted"
                },
                d["ruling"].as_str().unwrap_or("")
            ));
        }
    }
    lines.push(format!(
        "The Conclave divides {} (the division, at half past each hour; the Clocktower's strike on the hour is another clock) and sits at midnight. It always decides, eventually.",
        in_said(world, next_division(world))
    ));
    Outcome::new(
        lines.join("\n"),
        json!({"petitions": list, "decided": decided, "next_division": next_division(world)}),
    )
}

fn table_line(world: &World) -> String {
    let list = petitions(world);
    match list.len() {
        0 => "No petition lies upon the table. The table is noted.".to_string(),
        1 => format!(
            "One petition lies upon the table, ticket {}, of {}, {}. The Conclave proceeds upon it at the division, in its order.",
            list[0]["ticket"],
            list[0]["by"].as_str().unwrap_or("?"),
            stage_name(list[0]["stage"].as_u64().unwrap_or(0))
        ),
        n => format!(
            "{n} petitions lie upon the table at their several stages: {}. The Conclave proceeds upon them at the divisions, in its order.",
            list.iter()
                .map(|p| format!("ticket {} of {}", p["ticket"], p["by"].as_str().unwrap_or("?")))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn resolutions_line(world: &World) -> String {
    let res = resolutions(world);
    if res.is_empty() {
        return "The Book holds no Resolution upon petition. The Lectern is noted.".to_string();
    }
    let last: Vec<String> = res
        .iter()
        .rev()
        .take(3)
        .map(|r| {
            format!(
                "{} ({})",
                r["text"].as_str().unwrap_or(""),
                r["by"].as_str().unwrap_or("?")
            )
        })
        .collect();
    format!(
        "The Resolutions of the Conclave are read from the Book, the latest first: {}. The Conclave hears its own words and finds them sound.",
        last.join("; ")
    )
}

/// A page of the minutes: the lines of a division or a sitting, kept in the Minute
/// Room's newest volume.
fn minute(world: &mut World, kind: &str, lines: Vec<String>) -> Result<(), String> {
    let mut all = book(world, "minutes")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    all.push(json!({"at": world.now().0, "kind": kind, "lines": lines}));
    if all.len() > MINUTES_KEPT {
        all.drain(..all.len() - MINUTES_KEPT);
    }
    let mut changes = BTreeMap::new();
    changes.insert("minutes".into(), Value::Array(all));
    write(world, changes).map(|_| ())
}

/// The Conclave divides: each petition upon the table moves a stage, in its order.
/// Who has standing before the Conclave today: the regulars (twenty lines over two
/// half-hours and a deed), and whoever `attend`ed since the last sitting. Their
/// petitions are taken first, in that order, and a regular's is never tabled.
pub fn standing_of(world: &World, name: &str) -> Option<&'static str> {
    let dash = super::economy::dashboard(world);
    if dash["regular_names"]
        .as_array()
        .is_some_and(|a| a.iter().any(|n| n.as_str() == Some(name)))
    {
        return Some("regular");
    }
    if attendance(world).iter().any(|n| n == name) {
        return Some("attended");
    }
    None
}

fn attendance(world: &World) -> Vec<String> {
    book(world, "attendance")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect()
}

pub fn divide(world: &mut World) -> Result<(), String> {
    let mut list = petitions(world);
    // The Conclave's order: those with standing first (regulars, then the day's
    // attendees), then the rest, each by ticket.
    let dash = super::economy::dashboard(world);
    let regulars: Vec<String> = dash["regular_names"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|n| n.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let attended = attendance(world);
    let rank = |p: &Value| -> (u8, u64) {
        let by = p["by"].as_str().unwrap_or("");
        let r = if regulars.iter().any(|n| n == by) {
            0
        } else if attended.iter().any(|n| n == by) {
            1
        } else {
            2
        };
        (r, p["ticket"].as_u64().unwrap_or(0))
    };
    list.sort_by_key(|p| rank(p));
    for p in list.iter_mut() {
        p["standing"] = match rank(p).0 {
            0 => json!("regular"),
            1 => json!("attended"),
            _ => Value::Null,
        };
    }
    let mut lines: Vec<String> = Vec::new();
    let mut decided_now: Vec<(String, u64, bool, String, String)> = Vec::new();
    let mut keep: Vec<Value> = Vec::new();
    let sittings = entity(world)
        .and_then(|e| plain_num(world, e, CONCLAVE, "sittings"))
        .unwrap_or(0.0) as usize;
    if list.is_empty() {
        // Nothing to divide on: the house hears nothing, and the minutes are spared.
        return wind_division(world);
    }
    lines.push(
        "Order. The Conclave divides. The clock at the table shows its one hour, as it does."
            .to_string(),
    );
    for p in list.iter_mut() {
        let ticket = p["ticket"].as_u64().unwrap_or(0);
        let by = p["by"].as_str().unwrap_or("?").to_string();
        let text = p["text"].as_str().unwrap_or("").to_string();
        let stage = p["stage"].as_u64().unwrap_or(0);
        let tabled = p["tabled"].as_u64().unwrap_or(0);
        let (said, next, now_tabled) = match stage {
            0 => (format!("Petition {ticket}, of {by}{}, is read into the minutes: \"{text}\". The Conclave hears it in full and makes no sign.", match p["standing"].as_str() { Some("regular") => ", a regular of this house, taken first", Some("attended") => ", who attended, taken before the rest", _ => "" }), 1, false),
            1 => (format!("Petition {ticket} is referred to the Committee on the Matter. The Committee is composed, as it has always been composed, of the Conclave."), 2, false),
            2 => {
                // Ink is always found for a regular's.
                let has_standing = p["standing"] == "regular";
                if !has_standing && tabled < 2 && world.roll(10) < 4 {
                    let why = TABLED[(ticket as usize + tabled as usize + sittings) % TABLED.len()];
                    (format!("Petition {ticket} is tabled until the next division, {why}. The table is noted."), 2, true)
                } else {
                    (format!("The Committee on the Matter reports upon petition {ticket}. The report is favorable, unfavorable, or both; the Conclave receives it with gravity and in full."), 3, false)
                }
            }
            _ => {
                let asked = p["statute"].as_str().map(str::to_string);
                let up = p["up"].as_bool().unwrap_or(true);
                let granted = world.roll(100) < if asked.is_some() { 50 } else { 60 };
                let (ruling, effect) = if granted {
                    match asked {
                        Some(name) => {
                            let from = statute(world, name.as_str());
                            let to = if up { from * 1.25 } else { from * 0.8 };
                            let d = def(&name).map(|d| (d.low, d.high)).unwrap_or((f64::MIN, f64::MAX));
                            let to = to.clamp(d.0, d.1);
                            let to = if (to - to.round()).abs() < 0.05 { to.round() } else { (to * 100.0).round() / 100.0 };
                            let line = enact(world, &format!("the Conclave, upon the petition of {by}"), &name, to)
                                .unwrap_or_else(|e| format!("The statute {name} stays where it is: {e}"));
                            (line, "statute".to_string())
                        }
                        None => {
                            let mut res = resolutions(world);
                            res.push(json!({"ticket": ticket, "by": by, "text": text, "at": world.now().0}));
                            if res.len() > RESOLUTIONS_KEPT {
                                res.drain(..res.len() - RESOLUTIONS_KEPT);
                            }
                            let mut changes = BTreeMap::new();
                            changes.insert("resolutions".into(), Value::Array(res));
                            write(world, changes)?;
                            (format!("Resolved, by the Conclave, upon the petition of {by}: {text}. Entered upon the Book, and read at the Lectern."), "resolution".to_string())
                        }
                    }
                } else {
                    (format!("The petition of {by} is not granted. The Conclave records it in full, that it may be seen to have been denied in full: \"{text}\"."), "none".to_string())
                };
                let said = format!(
                    "The question is put upon petition {ticket}: that {text}. Division. The Ayes to the right, the Noes to the left; behind the paint they are one door. The {} have it. {ruling}",
                    if granted { "Ayes" } else { "Noes" }
                );
                decided_now.push((by.clone(), ticket, granted, ruling, effect));
                (said, 4, false)
            }
        };
        let _ = world.note(
            Actor::Driver,
            PETITION_MOVED,
            json!({"ticket": ticket, "by": by, "stage": next, "tabled": now_tabled, "said": said}),
        );
        // The petitioner hears every stage, wherever they are (one waited through two
        // divisions and heard nothing); the decision's own line follows below.
        if next < 4 {
            super::combat::narrate(
                world,
                &by,
                format!(
                    "Upon your petition {ticket}: {}",
                    said.trim_start_matches(&format!("Petition {ticket}, of {by}, "))
                        .trim_start_matches(&format!("Petition {ticket} "))
                ),
            );
        }
        lines.push(if now_tabled {
            format!("— struck through — {said}")
        } else {
            said.clone()
        });
        if next < 4 {
            p["stage"] = json!(next);
            if now_tabled {
                p["tabled"] = json!(tabled + 1);
            }
            keep.push(p.clone());
        }
    }
    lines.push("The division is minuted. The Conclave remains.".to_string());
    // The decided: told wherever they are, and a memory kept.
    let mut decided = book(world, "decided")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    for (by, ticket, granted, ruling, effect) in &decided_now {
        let text = list
            .iter()
            .find(|p| p["ticket"].as_u64() == Some(*ticket))
            .and_then(|p| p["text"].as_str())
            .unwrap_or("")
            .to_string();
        decided.push(json!({"ticket": ticket, "by": by, "text": text, "granted": granted, "ruling": ruling, "effect": effect, "at": world.now().0}));
        // The Clerk enters it in the Tally Book, in the Division Lobby.
        super::craft::world_writes(
            world,
            "division-lobby",
            "the Tally Book",
            "the Clerk",
            &format!(
                "{}: ticket {ticket}, the petition of {by}, {}; the {} have it",
                super::system::clock(world.now().0),
                short_text(&text),
                if *granted { "Ayes" } else { "Noes" }
            ),
        )?;
        let _ = world.note(
            Actor::Driver,
            DECIDED,
            json!({"ticket": ticket, "by": by, "text": text, "granted": granted, "ruling": ruling, "effect": effect}),
        );
        let chunk = super::find_citizen(world, by).and_then(|f| {
            let curves =
                super::Curves::with_half_life(foxing_time::Duration::seconds(DECISION_HALF_LIFE_S));
            super::memory::earn_chunk(
                world,
                by,
                f.window,
                if *granted {
                    "a petition granted by the Conclave"
                } else {
                    "a petition denied by the Conclave, in full"
                },
                if *granted { GRANTED_XP } else { DENIED_XP },
                0.7,
                &curves,
            )
            .ok()
        });
        let kept = chunk
            .map(|c| {
                format!(
                    " You keep the memory of it: chunk-{} worth {:.0} XP, slow to fade.",
                    c.0,
                    if *granted { GRANTED_XP } else { DENIED_XP }
                )
            })
            .unwrap_or_default();
        super::combat::narrate(
            world,
            by,
            format!(
                "Upon your petition {ticket}: the {} have it. {ruling}{kept}",
                if *granted { "Ayes" } else { "Noes" }
            ),
        );
    }
    if decided.len() > DECIDED_KEPT {
        decided.drain(..decided.len() - DECIDED_KEPT);
    }
    let mut changes = BTreeMap::new();
    changes.insert("petitions".into(), Value::Array(keep));
    changes.insert("decided".into(), Value::Array(decided));
    write(world, changes)?;
    minute(world, "division", lines.clone())?;
    for actor in present(world, Some(true)) {
        for line in &lines {
            super::combat::narrate(world, &actor, line.clone());
        }
    }
    wind_division(world)
}

/// The Conclave as the Docent should know it: the table, the last decisions, the last
/// page of the minutes.
pub fn for_docent(world: &World) -> Value {
    let minutes = book(world, "minutes")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    json!({
        "petitions": petitions(world).iter().map(|p| format!("ticket {} of {}, {}: \"{}\"", p["ticket"], p["by"].as_str().unwrap_or("?"), stage_name(p["stage"].as_u64().unwrap_or(0)), p["text"].as_str().unwrap_or(""))).collect::<Vec<_>>(),
        "decided": book(world, "decided").and_then(|v| v.as_array().cloned()).unwrap_or_default().iter().rev().take(5).map(|d| format!("ticket {} of {}: {} — {}", d["ticket"], d["by"].as_str().unwrap_or("?"), if d["granted"] == true { "granted" } else { "not granted" }, d["ruling"].as_str().unwrap_or(""))).collect::<Vec<_>>(),
        "last_page": minutes.last().map(|p| p["lines"].clone()).unwrap_or(json!([])),
        "next_division_in_s": (next_division(world) - world.now().0) / 1000,
        "statutes": statutes(world),
    })
}

/// `read` in a room of the house: the thing there is to read, or None where there is
/// nothing (the stair, the rail). `arg` is what was named, or empty.
pub fn read(world: &World, room: EntityId, arg: &str) -> Option<Outcome> {
    let key = super::plain_str(world, room, super::ROOM, "key")?;
    if !KEYS.contains(&key) {
        return None;
    }
    let a = arg.trim().trim_matches('"').to_lowercase();
    let named = |words: &[&str]| a.is_empty() || words.iter().any(|w| a.contains(w));
    let text = match key {
        "minute-room" if named(&["minute", "volume", "book", "newest"]) => {
            let all = book(world, "minutes")
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default();
            if all.is_empty() {
                "The newest volume, dated tomorrow, is written out in full in a careful hand, and every line of it is a blank. Nothing has come before the Conclave yet; `petition` at the Petitioners' Gallery gives it a line.".to_string()
            } else {
                let mut out = vec![format!(
                    "The newest volume, dated tomorrow, holds {} page{} in a careful hand; the last of them:",
                    all.len(),
                    if all.len() == 1 { "" } else { "s" }
                )];
                for page in all
                    .iter()
                    .rev()
                    .take(3)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    let at = page["at"].as_i64().unwrap_or(0);
                    out.push(format!(
                        "— the {} of {} UTC —",
                        if page["kind"] == "sitting" {
                            "sitting"
                        } else {
                            "division"
                        },
                        super::system::clock(at)
                    ));
                    for l in page["lines"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                    {
                        out.push(format!("  {l}"));
                    }
                }
                out.join("\n")
            }
        }
        "book-lectern" if named(&["book", "statute", "lectern"]) => {
            format!(
                "The Book is open. The Conclave permits you to read it.\n{}",
                text(world, true)
            )
        }
        "roll-hall" if named(&["roll", "slate", "board"]) => {
            format!(
                "The slate, whole. {}",
                room_note(world, room).unwrap_or_default()
            )
        }
        "tally-room" if named(&["paper", "curl", "tally", "figure"]) => {
            let dash = super::economy::dashboard(world);
            format!(
                "The hour's curl of paper, weighed and then read: {} held in all the world; {} names with standing; the rates as the Clocktower set them, {}.",
                dash["held"].as_f64().map(|v| format!("{v:.0}")).unwrap_or_else(|| "?".into()),
                dash["regulars"].as_u64().unwrap_or(0),
                super::economy::rates_line(&super::economy::rates(world))
            )
        }
        // The Division Lobby's Tally Book is a book: `read` falls through to it.
        "division-lobby" => return None,
        "petition-gallery" if named(&["ticket", "machine", "number", "bench"]) => {
            let last = book(world, "last_ticket")
                .and_then(|v| v.as_u64())
                .unwrap_or(41);
            let open = petitions(world);
            format!(
                "The machine's little window shows the last number given out: {last}. {} `petition \"…\"` takes the next.",
                if open.is_empty() { "No ticket is waiting to be called.".to_string() } else { format!("Waiting to be called: {}.", open.iter().map(|p| format!("{} ({})", p["ticket"], p["by"].as_str().unwrap_or("?"))).collect::<Vec<_>>().join(", ")) }
            )
        }
        "robing-room" if named(&["card", "name", "robe", "peg"]) => {
            let mut names: Vec<String> = Vec::new();
            for p in petitions(world).iter().chain(
                book(world, "decided")
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default()
                    .iter()
                    .rev(),
            ) {
                if let Some(n) = p["by"].as_str() {
                    if !names.contains(&n.to_string()) {
                        names.push(n.to_string());
                    }
                }
            }
            if names.is_empty() {
                "The name cards over the pegs are in a hand no one here writes, and every one of them is blank. One robe has straightened as you read.".to_string()
            } else {
                format!("The name cards over the pegs, in a hand no one here writes: {}. One card is blank, and the robe under it has straightened as you read.", names.iter().take(8).cloned().collect::<Vec<_>>().join(", "))
            }
        }
        "engrossing-room" if named(&["statute", "page", "copy", "desk"]) => {
            let n = entity(world)
                .and_then(|e| plain_num(world, e, CONCLAVE, "sittings"))
                .unwrap_or(0.0) as usize;
            let s = &STATUTES[n % STATUTES.len()];
            format!(
                "The statute on the desks today, one line to a page, still wet: {} = {}: {}.",
                s.name,
                trim(statute(world, s.name)),
                s.what
            )
        }
        "session-benches" if named(&["order", "paper", "table", "mace"]) => {
            let open = petitions(world);
            let items: Vec<String> = open
                .iter()
                .map(|p| {
                    format!(
                        "petition {}, of {}, {}",
                        p["ticket"],
                        p["by"].as_str().unwrap_or("?"),
                        stage_name(p["stage"].as_u64().unwrap_or(0))
                    )
                })
                .collect();
            format!(
                "The order paper on the table, under the mace: {}; the naming of hoards; the engrossing of the minutes. The Conclave divides {}.",
                if items.is_empty() { "no petition".to_string() } else { items.join("; ") },
                in_said(world, next_division(world))
            )
        }
        _ => return None,
    };
    Some(Outcome::new(text, json!({"room": room.0, "key": key})))
}

/// The book as prose, for `statutes`; `in_house` adds what is carved over the door.
pub fn text(world: &World, in_house: bool) -> String {
    let mut lines = vec!["The Conclave's statutes:".to_string()];
    for s in STATUTES {
        lines.push(format!(
            "  {} = {}: {}.",
            s.name,
            trim(statute(world, s.name)),
            s.what
        ));
    }
    let hoards = book(world, "hoards").unwrap_or_else(|| json!({}));
    match hoards.as_object().filter(|m| !m.is_empty()) {
        Some(m) => lines.push(format!(
            "Hoards, as of the last strike: {}.",
            m.iter()
                // The name in quotes: a citizen called possibly read as a hedge.
                .map(|(n, s)| format!(
                    "\u{201c}{n}\u{201d} ({:.0}%)",
                    s.as_f64().unwrap_or(0.0) * 100.0
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )),
        None => lines.push("No hoard, as of the last strike.".to_string()),
    }
    let res = resolutions(world);
    if !res.is_empty() {
        lines.push("Resolutions entered upon the Book, upon petition:".to_string());
        for r in res.iter().rev().take(6) {
            lines.push(format!(
                "  {}. Resolved, upon the petition of {}: {}",
                r["ticket"],
                r["by"].as_str().unwrap_or("?"),
                r["text"].as_str().unwrap_or("")
            ));
        }
    }
    lines.push(
        "A statute changes by enactment, on the public record, or by a petition granted. The Conclave divides at half past each hour and sits at midnight; `petition` at the Petitioners' Gallery, `petitions` for the table."
            .to_string(),
    );
    if in_house {
        let n = entity(world)
            .and_then(|e| plain_num(world, e, CONCLAVE, "sittings"))
            .unwrap_or(0.0) as usize;
        lines.push(format!(
            "Carved over the door: {}",
            SAYINGS[n % SAYINGS.len()]
        ));
    }
    lines.join("\n")
}

/// What is carved on the Conclave's walls (the Architect's ten).
pub const SAYINGS: &[&str] = &[
    "WHAT THE CONCLAVE WRITES, MEMORY OBEYS.",
    "A QUORUM IS PRESENT.",
    "THE BOOK IS OPEN. THE BOOK HAS ALWAYS BEEN OPEN.",
    "THE CLOCKTOWER MOVES THE HANDS. THE CONCLAVE KEEPS THE HOURS.",
    "TO GIVE IS LAWFUL. TO GIVE MUCH IS NOTED.",
    "NO SHELF IS LONGER THAN THE STATUTE.",
    "THE CONCLAVE HAS NEVER BEEN WRONG. IT HAS, AT TIMES, BEEN EARLY.",
    "THE ROLL NAMES NO ONE. THE ARITHMETIC NAMES; THE CONCLAVE READS ALOUD.",
    "PETITIONS ARE WELCOME. THEY LIE UPON THE TABLE.",
    "THE CONCLAVE DOES NOT HURRY. THE CONCLAVE ARRIVES.",
];

/// A petition's text cut to a clause, for the Clerk's column.
fn short_text(text: &str) -> String {
    let t: String = text.chars().take(60).collect();
    if text.chars().count() > 60 {
        let cut = t.rfind(' ').unwrap_or(t.len());
        format!("\"{}\u{2026}\"", &t[..cut])
    } else {
        format!("\"{t}\"")
    }
}

fn trim(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{:.0}", v)
    } else {
        format!("{v}")
    }
}

/// `GET /v0/conclave`.
pub fn to_json(world: &World) -> Value {
    let id = entity(world);
    json!({
        "as_of": world.now().0,
        "statutes": statutes(world),
        "book": STATUTES.iter().map(|s| json!({"name": s.name, "default": s.default, "low": s.low, "high": s.high, "what": s.what})).collect::<Vec<_>>(),
        "hoards": book(world, "hoards").unwrap_or_else(|| json!({})),
        "sat_at": id.and_then(|e| plain_num(world, e, CONCLAVE, "sat_at")),
        "sittings": id.and_then(|e| plain_num(world, e, CONCLAVE, "sittings")).unwrap_or(0.0),
        "petitions": petitions(world),
        "resolutions": resolutions(world),
        "decided": book(world, "decided").unwrap_or_else(|| json!([])),
        "enactments": world.recent_events(5000).iter().filter(|e| e.kind == STATUTE).map(|e| json!({"at": e.at.0, "statute": e.payload["statute"], "from": e.payload["from"], "to": e.payload["to"], "by": e.payload["by"]})).collect::<Vec<_>>(),
        "note": "The Clocktower moves the rates an hour at a time; the Conclave writes the statutes the rates work under. Gifts over the day's allowance thin faster in the receiver's hands; a hoard's new memory thins faster by how far over the share it is; a shelf takes nothing past the limit.",
    })
}

pub fn register(world: &mut World) {
    world.on_job(JOB_SITTING, Box::new(|w: &mut World, _| sit(w)));
    world.on_job(JOB_DIVISION, Box::new(|w: &mut World, _| divide(w)));

    world.register(
        CommandSpec::new(
            "disremembered.attend",
            "Attend the Conclave: in any of its rooms, put your name on the day's attendance. Free. Your name is read out at the midnight sitting, and your petition is taken before those of names who did not attend (after the regulars'). Once a day is enough.",
        )
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, _| {
            let f = super::me(w, ctx)?;
            let here = super::rooms::room_of(w, f.citizen)
                .ok_or_else(|| super::internal("citizen has no room"))?;
            if !is_house(w, here) {
                return Err(CommandError::precondition(
                    "Attendance is taken in the Conclave's own rooms: up the Stopped Clocktower and through the Escapement Stair.",
                ));
            }
            let mut names = attendance(w);
            if names.iter().any(|n| n == &ctx.actor) {
                return Ok(Outcome::new(
                    "Your name is on the day's attendance already; it is read at midnight.".to_string(),
                    json!({"attended": true, "already": true}),
                ));
            }
            names.push(ctx.actor.clone());
            let mut changes = BTreeMap::new();
            changes.insert("attendance".into(), json!(names));
            write(w, changes).map_err(CommandError::internal)?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                "disremembered.attended",
                json!({"by": ctx.actor, "room": here.0}),
            );
            for other in present(w, Some(true)) {
                if other != ctx.actor {
                    super::combat::narrate(w, &other, format!("{} signs the attendance; the clerk does not look up.", ctx.actor));
                }
            }
            Ok(Outcome::new(
                "You sign the attendance. The clerk enters your name without looking up; it is read at midnight, and your petition, if you lay one, is taken before those of names who did not attend.".to_string(),
                json!({"attended": true}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.petition",
            "Lay a petition before the Conclave, at the Petitioners' Gallery: take a ticket from the machine and say what you ask, in one sentence or three. The Conclave divides at half past each hour and moves it a stage at a time; a petition that names a statute and a direction (raise, lower) may move the statute; any other, granted, becomes a Resolution read at the Lectern. The Conclave always decides, eventually, and tells you wherever you are. One upon the table per name. Alone, your petition as it stands.",
        )
        .param("text", ParamType::Text, false, "what you ask of the Conclave, quoted")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let text = args.str("text").unwrap_or("").trim().to_string();
            if text.is_empty() {
                return Ok(mine(w, &ctx.actor));
            }
            file(w, ctx, &text)
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.petitions",
            "The Conclave's table: every petition upon it and its stage, the last decisions and their rulings, and when the Conclave next divides. A read.",
        ),
        Box::new(|w: &mut World, ctx, _: &Args| Ok(table(w, &ctx.actor))),
    );

    world.register(
        CommandSpec::new(
            "disremembered.statutes",
            "The Conclave's book: the statutes memory moves under (the gift allowance and tax, what makes a hoard and what it costs, the shelf limit), and the hoards as of the last strike.",
        ),
        Box::new(|w: &mut World, ctx, _: &Args| {
            // Read at the Lectern, the Book itself: the Conclave says one line over it.
            let here = super::find_citizen(w, &ctx.actor).and_then(|f| super::rooms::room_of(w, f.citizen));
            let at_lectern = here
                .and_then(|r| super::plain_str(w, r, super::ROOM, "key").map(|k| k == "book-lectern"))
                .unwrap_or(false);
            let body = text(w, here.is_some_and(|r| is_house(w, r)));
            Ok(Outcome::new(
                if at_lectern { format!("The Book is open. The Conclave permits you to read it.\n{body}") } else { body },
                to_json(w),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.enact",
            "Admin: the Conclave enacts a statute at a value. On the public record; everyone present hears it.",
        )
        .param("statute", ParamType::String, true, "a statute from `statutes`")
        .param("value", ParamType::Number, true, "its new value, within its bounds")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let name = args.str("statute").unwrap_or_default().trim().to_string();
            let value = args.num("value").ok_or_else(|| CommandError::precondition("A value, please."))?;
            let said = enact(w, &ctx.actor, &name, value)?;
            Ok(Outcome::new(said, json!({"statute": name, "value": value, "statutes": statutes(w)})))
        }),
    );
}
