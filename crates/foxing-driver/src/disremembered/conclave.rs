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
    wind(world)
}

fn wind(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    let due = foxing_time::Timestamp((now / SIT_EVERY_MS + 1) * SIT_EVERY_MS);
    world
        .schedule_once(JOB_SITTING, due, json!({}), |_| true)
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
        format!("The Ledger of the Clocktower is laid before the Conclave. The Conclave reads that {held:.0} is held in all the world, and that {regulars} names have spoken their twenty lines this day and have standing before it."),
        format!("The Book is opened at the Lectern. The statutes that stand are these: {standing}. The Conclave has examined them and finds them sound. The Conclave wrote them."),
        naming,
        "No petition lies upon the table. The table is noted.".to_string(),
        "The minutes are engrossed. The Conclave rises. The Conclave remains.".to_string(),
    ];
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

/// The book as prose, for `statutes`.
pub fn text(world: &World) -> String {
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
                .map(|(n, s)| format!("{n} ({:.0}%)", s.as_f64().unwrap_or(0.0) * 100.0))
                .collect::<Vec<_>>()
                .join(", ")
        )),
        None => lines.push("No hoard, as of the last strike.".to_string()),
    }
    lines.push(
        "A statute changes by enactment, on the public record. The Conclave sits once a day."
            .to_string(),
    );
    let n = entity(world)
        .and_then(|e| plain_num(world, e, CONCLAVE, "sittings"))
        .unwrap_or(0.0) as usize;
    lines.push(format!(
        "Carved over the door: {}",
        SAYINGS[n % SAYINGS.len()]
    ));
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
        "enactments": world.recent_events(5000).iter().filter(|e| e.kind == STATUTE).map(|e| json!({"at": e.at.0, "statute": e.payload["statute"], "from": e.payload["from"], "to": e.payload["to"], "by": e.payload["by"]})).collect::<Vec<_>>(),
        "note": "The Clocktower moves the rates an hour at a time; the Conclave writes the statutes the rates work under. Gifts over the day's allowance thin faster in the receiver's hands; a hoard's new memory thins faster by how far over the share it is; a shelf takes nothing past the limit.",
    })
}

pub fn register(world: &mut World) {
    world.on_job(JOB_SITTING, Box::new(|w: &mut World, _| sit(w)));

    world.register(
        CommandSpec::new(
            "disremembered.statutes",
            "The Conclave's book: the statutes memory moves under (the gift allowance and tax, what makes a hoard and what it costs, the shelf limit), and the hoards as of the last strike.",
        ),
        Box::new(|w: &mut World, ctx, _: &Args| {
            // Read at the Lectern, the Book itself: the Conclave says one line over it.
            let at_lectern = super::find_citizen(w, &ctx.actor)
                .and_then(|f| super::rooms::room_of(w, f.citizen))
                .and_then(|r| super::plain_str(w, r, super::ROOM, "key").map(|k| k == "book-lectern"))
                .unwrap_or(false);
            let body = text(w);
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
