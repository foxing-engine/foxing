//! The Namery: where the names a creature may be given are kept.
//!
//! A creature earning its first tier takes a name. The world began with a fixed list
//! of thirty-two (`mind::NAMES`), dealt by the hash of the creature's id, which ran
//! short the first week there were more named creatures than names and would have
//! handed out `Hesper-412`. The Namery holds a pool stocked from outside: the
//! operator's nightly tool asks a small model for names in the world's register,
//! refuses what fails the mechanical checks here (one capitalized word, three to
//! twelve letters, not a name the world already has or a word it uses for something
//! else) and `stock`s the rest. A name drawn from the pool is drawn once: it goes to
//! the used list with the creature and the time, and is never dealt again, so every
//! named creature the world has ever had has a name of its own. A creature that
//! loses its name and earns it back takes its former name, if no one took it meanwhile.
//! The pool empty, the fixed list serves as it always did.
//!
//! Public record: `disremembered.stocked` (how many, by whom, what wrote them).

use super::{plain_str, CITIZEN, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const NAMERY: &str = "disremembered.namery";
/// The Namery took in names: `{count, by, model, names}`. Public.
pub const STOCKED: &str = "disremembered.stocked";

/// A name: one capitalized word of letters, this long.
pub const NAME_MIN: usize = 3;
pub const NAME_MAX: usize = 12;
/// Fewer unused names than this and `GET /v0/namery` says the pool is low.
pub const LOW: usize = 30;

fn entity(world: &World) -> Option<EntityId> {
    world
        .store
        .entities()
        .find(|e| e.kind == "namery")
        .map(|e| e.id)
}

fn field(world: &World, name: &str) -> Option<Value> {
    let id = entity(world)?;
    match world
        .store
        .entity(id)?
        .components
        .get(NAMERY)?
        .fields
        .get(name)?
    {
        Field::Plain(v) => Some(v.clone()),
        _ => None,
    }
}

fn strings(v: Option<Value>) -> Vec<String> {
    v.and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|s| s.as_str().map(str::to_string))
        .collect()
}

/// The unused names, in the order they were stocked.
pub fn pool(world: &World) -> Vec<String> {
    strings(field(world, "pool"))
}

/// Every name ever drawn from the pool: `{name, creature, at}`.
pub fn used(world: &World) -> Vec<Value> {
    field(world, "used")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
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
                        kind: "namery".into(),
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
        .and_then(|e| e.components.get(NAMERY))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    fields
        .entry("pool".into())
        .or_insert(Field::Plain(json!([])));
    fields
        .entry("used".into())
        .or_insert(Field::Plain(json!([])));
    fields
        .entry("stockings".into())
        .or_insert(Field::Plain(json!([])));
    for (k, v) in changes {
        fields.insert(k, Field::Plain(v));
    }
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: NAMERY.into(),
                fields,
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(id)
}

/// At boot: the Namery exists, empty if nothing was ever stocked.
pub fn ensure(world: &mut World) -> Result<(), String> {
    if entity(world).is_none() {
        write(world, BTreeMap::new())?;
    }
    Ok(())
}

/// The words the world already uses, lower-cased: creature names and kinds, citizens,
/// rooms, the fixed list, the pool and everything ever drawn. A new name may be none
/// of them.
pub fn taken(world: &World) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for e in world.store.entities() {
        match e.kind.as_str() {
            "creature" => {
                if let Some(k) = plain_str(world, e.id, CREATURE, "name") {
                    out.push(k.to_string());
                }
                if let Some(n) = super::mind::name(world, e.id) {
                    out.push(n);
                }
            }
            "citizen" => {
                if let Some(n) = plain_str(world, e.id, CITIZEN, "name") {
                    out.push(n.to_string());
                }
                if let Some(o) = &e.owner {
                    out.push(o.clone());
                }
            }
            "room" => {
                if let Some(n) = plain_str(world, e.id, ROOM, "name") {
                    out.push(n.trim_start_matches("The ").to_string());
                }
            }
            _ => {}
        }
    }
    out.extend(super::mind::NAMES.iter().map(|s| s.to_string()));
    out.extend(pool(world));
    out.extend(
        used(world)
            .iter()
            .filter_map(|u| u["name"].as_str().map(str::to_string)),
    );
    let mut out: Vec<String> = out.into_iter().map(|s| s.to_lowercase()).collect();
    out.sort();
    out.dedup();
    out
}

/// Why a name may not be stocked, or nothing.
pub fn refusal(world: &World, name: &str, taken: &[String]) -> Option<String> {
    let n = name.trim();
    let letters = n.chars().count();
    if !(NAME_MIN..=NAME_MAX).contains(&letters) {
        return Some(format!("{NAME_MIN} to {NAME_MAX} letters"));
    }
    let mut chars = n.chars();
    let first = chars.next()?;
    if !first.is_ascii_uppercase() || !chars.all(|c| c.is_ascii_lowercase()) {
        return Some("one capitalized word of letters".into());
    }
    let low = n.to_lowercase();
    if taken.binary_search(&low).is_ok() {
        return Some("the world has it".into());
    }
    // A word the vocabulary uses is not a name either.
    if world
        .commands_for(ActorKind::Admin)
        .iter()
        .any(|c| c.name.rsplit('.').next() == Some(low.as_str()))
    {
        return Some("a verb".into());
    }
    None
}

/// Draw the next unused name for a creature: it goes to the used list and is never
/// dealt again. Nothing when the pool is empty.
pub fn draw(world: &mut World, creature: EntityId) -> Result<Option<String>, CommandError> {
    let mut pool = pool(world);
    if pool.is_empty() {
        return Ok(None);
    }
    let name = pool.remove(0);
    let mut used = used(world);
    used.push(json!({"name": name, "creature": creature.0, "at": world.now().0}));
    let mut changes = BTreeMap::new();
    changes.insert("pool".into(), json!(pool));
    changes.insert("used".into(), json!(used));
    write(world, changes).map_err(CommandError::precondition)?;
    Ok(Some(name))
}

/// What `GET /v0/namery` shows the operator: the pool and whether it is low, what was
/// drawn, what the world already uses (for the tool's exclusions), the stockings.
pub fn report(world: &World) -> Value {
    let pool = pool(world);
    json!({
        "pool": pool,
        "unused": pool.len(),
        "low": pool.len() < LOW,
        "low_at": LOW,
        "used": used(world),
        "taken": taken(world),
        "stockings": field(world, "stockings").unwrap_or(json!([])),
        "rules": {"letters": [NAME_MIN, NAME_MAX], "shape": "one capitalized word of ASCII letters, not a name, kind, citizen, room or verb the world has"},
        "note": "stock \"Name, Name, ...\" [model] (operator) adds what passes; each name is dealt once, to the next creature that earns its first tier.",
    })
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.stock",
            "Operator. Stock the Namery with names, comma-separated and quoted: what passes the checks joins the pool, each dealt once; what fails is said back with why.",
        )
        .param("names", ParamType::Text, true, "the names, comma-separated, quoted")
        .param("model", ParamType::String, false, "what wrote them, for the record")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let text = args.str("names").unwrap_or("").to_string();
            let model = args.str("model").unwrap_or("").trim().to_string();
            let taken = taken(w);
            let mut added: Vec<String> = Vec::new();
            let mut refused: Vec<Value> = Vec::new();
            for raw in text.split([',', '\n', ';']) {
                let n = raw.trim().trim_matches('"').trim();
                if n.is_empty() {
                    continue;
                }
                let low = n.to_lowercase();
                if added.iter().any(|a| a.to_lowercase() == low) {
                    refused.push(json!({"name": n, "why": "twice in the list"}));
                    continue;
                }
                match refusal(w, n, &taken) {
                    Some(why) => refused.push(json!({"name": n, "why": why})),
                    None => added.push(n.to_string()),
                }
            }
            if added.is_empty() {
                return Err(CommandError::precondition(format!(
                    "Nothing to stock: {}.",
                    refused
                        .iter()
                        .map(|r| format!("{} ({})", r["name"].as_str().unwrap_or("?"), r["why"].as_str().unwrap_or("?")))
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
            let mut pool = pool(w);
            pool.extend(added.iter().cloned());
            let mut stockings = field(w, "stockings")
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default();
            stockings.push(json!({"at": w.now().0, "by": ctx.actor, "model": model, "count": added.len()}));
            if stockings.len() > 60 {
                stockings.drain(..stockings.len() - 60);
            }
            let mut changes = BTreeMap::new();
            changes.insert("pool".into(), json!(pool));
            changes.insert("stockings".into(), json!(stockings));
            write(w, changes).map_err(CommandError::precondition)?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                STOCKED,
                json!({"count": added.len(), "by": ctx.actor, "model": model, "names": added, "refused": refused.len()}),
            );
            let unused = pool_len(w);
            Ok(Outcome::new(
                format!(
                    "The Namery took in {} name{}; {} unused now.{}",
                    added.len(),
                    if added.len() == 1 { "" } else { "s" },
                    unused,
                    if refused.is_empty() {
                        String::new()
                    } else {
                        format!(" Refused {}: {}.", refused.len(), refused.iter().map(|r| format!("{} ({})", r["name"].as_str().unwrap_or("?"), r["why"].as_str().unwrap_or("?"))).collect::<Vec<_>>().join(", "))
                    }
                ),
                json!({"added": added, "refused": refused, "unused": unused}),
            ))
        }),
    );
}

fn pool_len(world: &World) -> usize {
    pool(world).len()
}
