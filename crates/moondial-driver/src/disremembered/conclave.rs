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
use moondial_log::Actor;
use moondial_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use moondial_store::{EntityId, Field, StoreEvent};
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
    let due = moondial_time::Timestamp((now / SIT_EVERY_MS + 1) * SIT_EVERY_MS);
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
    let said = format!("The Conclave enacts: {}. {}", prose(name, from, to), d.what);
    let _ = world.note(
        Actor::Driver,
        STATUTE,
        json!({"statute": name, "from": from, "to": to, "by": by, "said": said}),
    );
    tell_everyone(world, &said);
    world.notice(format!("{said} (enacted by {by})"));
    Ok(said)
}

fn prose(name: &str, from: f64, to: f64) -> String {
    let dir = if to > from { "rises" } else { "falls" };
    match name {
        "gift_allowance" => format!("the day's gift allowance {dir} from {from:.0} to {to:.0} XP"),
        "gift_tax" => format!(
            "a gift over the allowance now keeps {:.0}% of its half-life (was {:.0}%)",
            to * 100.0,
            from * 100.0
        ),
        "hoard_share" => format!(
            "a hoard is now a name holding more than {:.0}% of all that is held (was {:.0}%)",
            to * 100.0,
            from * 100.0
        ),
        "hoard_tax" => format!("the hoard tax {dir} from {from:.2} to {to:.2}"),
        "shelf_limit" => format!("the shelf limit {dir} from {from:.0} to {to:.0} XP"),
        _ => format!("{name} {dir} from {from} to {to}"),
    }
}

fn tell_everyone(world: &mut World, line: &str) {
    let present: Vec<String> = world
        .store
        .entities()
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .collect();
    for actor in present {
        super::combat::narrate(world, &actor, line.to_string());
    }
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
    let names: Vec<String> = hoards
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    let sittings = entity(world)
        .and_then(|e| plain_num(world, e, CONCLAVE, "sittings"))
        .unwrap_or(0.0)
        + 1.0;
    let mut changes = BTreeMap::new();
    changes.insert("sat_at".into(), json!(world.now().0));
    changes.insert("sittings".into(), json!(sittings));
    write(world, changes)?;
    let said = match names.len() {
        0 => "The Conclave sits, hears the Clocktower's ledger, and finds no hoard. The statutes stand.".to_string(),
        1 => format!("The Conclave sits, hears the Clocktower's ledger, and names a hoard: {}. The statutes stand; the hoard tax is on it.", names[0]),
        _ => format!("The Conclave sits, hears the Clocktower's ledger, and names the hoards: {}. The statutes stand; the hoard tax is on them.", names.join(", ")),
    };
    let _ = world.note(
        Actor::Driver,
        SAT,
        json!({"said": said, "hoards": hoards, "statutes": statutes(world), "dashboard": dash}),
    );
    tell_everyone(world, &said);
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
    lines.join("\n")
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
        Box::new(|w: &mut World, _, _: &Args| {
            Ok(Outcome::new(text(w), to_json(w)))
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
