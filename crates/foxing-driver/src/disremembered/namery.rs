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
    world.store.of_kind("namery").next().map(|e| e.id)
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

/// The epitaphs stocked, in order; said over falls in rotation.
pub fn epitaphs(world: &World) -> Vec<String> {
    strings(field(world, "epitaphs"))
}

/// An epitaph is this long at most, and may use these slots only.
pub const EPITAPH_MAX: usize = 180;
pub const EPITAPH_SLOTS: &[&str] = &["{killer}", "{room}", "{lost}", "{fall}", "{name}"];
/// The pool is low under this many.
pub const EPITAPHS_LOW: usize = 24;

/// Why an epitaph may not be stocked, or nothing.
pub fn epitaph_refusal(line: &str, have: &[String]) -> Option<String> {
    let t = line.trim();
    if t.chars().count() < 24 {
        return Some("too short".into());
    }
    if t.chars().count() > EPITAPH_MAX {
        return Some(format!("over {EPITAPH_MAX} characters"));
    }
    // Only the known slots between braces.
    let mut rest = t;
    while let Some(i) = rest.find('{') {
        let Some(j) = rest[i..].find('}') else {
            return Some("an unclosed brace".into());
        };
        let slot = &rest[i..i + j + 1];
        if !EPITAPH_SLOTS.contains(&slot) {
            return Some(format!("an unknown slot {slot}"));
        }
        rest = &rest[i + j + 1..];
    }
    if have.iter().any(|h| h.eq_ignore_ascii_case(t)) {
        return Some("already stocked".into());
    }
    None
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
    fields
        .entry("epitaphs".into())
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
    let kind = super::plain_str(world, creature, super::CREATURE, "name")
        .unwrap_or("creature")
        .to_string();
    let keepers = super::mind::keepers(world, creature);
    draw_for(world, creature, &kind, &keepers)
}

/// A name dealt to anything: a creature at sixty, or a made thing that woke on the
/// bench (the third tier). The Register says what it went to and who fed it there.
pub fn draw_for(
    world: &mut World,
    who: EntityId,
    kind: &str,
    keepers: &[String],
) -> Result<Option<String>, CommandError> {
    let mut pool = pool(world);
    if pool.is_empty() {
        return Ok(None);
    }
    let name = pool.remove(0);
    let mut changes = BTreeMap::new();
    changes.insert("pool".into(), json!(pool));
    write(world, changes).map_err(CommandError::precondition)?;
    record_dealt(world, who, kind, keepers, &name, "pool")?;
    Ok(Some(name))
}

/// Which stocking a name came in with, counted from one; none for a stocking before
/// the Register kept names.
pub fn stocking_of(world: &World, name: &str) -> Option<usize> {
    field(world, "stockings")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .position(|s| {
            s["names"]
                .as_array()
                .is_some_and(|ns| ns.iter().any(|n| n.as_str() == Some(name)))
        })
        .map(|i| i + 1)
}

/// Every name dealt goes through here, however it was found: from the pool, an heir
/// name sealed at birth, a former name taken back, or one minted from the kind's own
/// syllables when the pool was empty. The Register of Names takes the line.
pub fn record_dealt(
    world: &mut World,
    who: EntityId,
    kind: &str,
    keepers: &[String],
    name: &str,
    how: &str,
) -> Result<(), CommandError> {
    let stocking = stocking_of(world, name);
    let mut used = used(world);
    used.push(json!({"name": name, "creature": who.0, "at": world.now().0, "how": how, "stocking": stocking}));
    let mut changes = BTreeMap::new();
    changes.insert("used".into(), json!(used));
    write(world, changes).map_err(CommandError::precondition)?;
    let by = match how {
        "heir" => ", a name that carries its parents'".to_string(),
        "former" => ", its own name taken back".to_string(),
        "minted" => ", minted from the kind's syllables, the pool being empty".to_string(),
        "reserved" => ", one of the world's first names, the pool being empty".to_string(),
        _ => match stocking {
            Some(n) => format!(", of the {} stocking", ordinal(n)),
            None => String::new(),
        },
    };
    let _ = super::craft::world_writes(
        world,
        "namery",
        "the Register of Names",
        "the Namery",
        &format!(
            "{} UTC: {name}, to {}{}{}",
            super::system::clock(world.now().0),
            super::mind::an(kind),
            if keepers.is_empty() {
                String::new()
            } else {
                format!(", fed to its name by {}", keepers.join(" and "))
            },
            by
        ),
    );
    Ok(())
}

/// A name's provenance, for `look`: where the Namery got it.
pub fn provenance(world: &World, name: &str) -> Option<String> {
    let u = used(world);
    let rec = u.iter().rev().find(|u| u["name"].as_str() == Some(name))?;
    Some(match rec["how"].as_str() {
        Some("heir") => "named for its parents".to_string(),
        Some("former") => "its old name back".to_string(),
        Some("minted") => "a name minted from its kind".to_string(),
        Some("reserved") => "one of the world's first names".to_string(),
        _ => match rec["stocking"].as_u64() {
            Some(n) => format!("named from the Namery's {} stocking", ordinal(n as usize)),
            None => "named by the Namery".to_string(),
        },
    })
}

fn ordinal(n: usize) -> String {
    const WORDS: &[&str] = &[
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
        "tenth", "eleventh", "twelfth",
    ];
    WORDS
        .get(n.wrapping_sub(1))
        .map(|w| w.to_string())
        .unwrap_or_else(|| format!("{n}th"))
}

/// A name minted from a kind's syllables and a diminutive, for when the pool is
/// empty: a wispling's child is a Wispkin, a Wisplet, a Wispling the Younger of
/// sorts. Never one the world has.
pub fn mint(world: &World, kind: &str, taken: &[String]) -> String {
    let syl = super::kindle::syllables_of(kind);
    let heads: Vec<String> = {
        let mut h = Vec::new();
        for n in 1..=syl.len().max(1) {
            h.push(syl.iter().take(n).cloned().collect::<String>());
        }
        h
    };
    let free = |c: &str| {
        !taken.iter().any(|t| t.eq_ignore_ascii_case(c)) && refusal(world, c, taken).is_none()
    };
    let cap = |s: &str| {
        let mut c = s.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    };
    for suffix in ["kin", "let", "ling", "et", "ock", "by", "wick", "mere"] {
        for h in heads.iter().rev() {
            let c = cap(&format!("{h}{suffix}"));
            if c.chars().count() >= NAME_MIN && c.chars().count() <= NAME_MAX && free(&c) {
                return c;
            }
        }
    }
    let base = cap(heads.last().map(String::as_str).unwrap_or(kind));
    (2..999)
        .map(|i| format!("{base}{i}"))
        .find(|c| free(c))
        .unwrap_or(base)
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
        "epitaphs": epitaphs(world),
        "epitaphs_low": epitaphs(world).len() < EPITAPHS_LOW,
        "epitaphs_low_at": EPITAPHS_LOW,
        "epitaph_rules": {"max_chars": EPITAPH_MAX, "slots": EPITAPH_SLOTS},
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
            stockings.push(json!({"at": w.now().0, "by": ctx.actor, "model": model, "count": added.len(), "names": added}));
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

pub fn register_epitaphs(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.epitaph",
            "Operator. Stock one epitaph, quoted: said over a fall, with the slots {killer}, {room}, {lost}, {fall} and {name} filled in. Said back with why if refused.",
        )
        .param("text", ParamType::Text, true, "the epitaph, quoted")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let text = args.str("text").unwrap_or("").trim().trim_matches('"').to_string();
            let have = epitaphs(w);
            if let Some(why) = epitaph_refusal(&text, &have) {
                return Err(CommandError::precondition(format!("Not stocked: {why}.")));
            }
            let mut list = have;
            list.push(text.clone());
            let mut changes = BTreeMap::new();
            changes.insert("epitaphs".into(), json!(list));
            write(w, changes).map_err(CommandError::precondition)?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                STOCKED,
                json!({"count": 1, "by": ctx.actor, "kind": "epitaph", "text": text}),
            );
            let n = epitaphs(w).len();
            Ok(Outcome::new(
                format!("The epitaph is stocked; {n} in the book."),
                json!({"epitaphs": n}),
            ))
        }),
    );
}

fn pool_len(world: &World) -> usize {
    pool(world).len()
}
