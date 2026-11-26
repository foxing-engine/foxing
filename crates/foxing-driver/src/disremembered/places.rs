//! The places on the body (C21; David, 11-25: "let's plan for a better equipment
//! experience", and of the two calls: "1:limit / 2:players to place"; then "3:a",
//! a second hand).
//!
//! A made thing used to work by being somewhere in the satchel: three charms were
//! three charms at work, nothing was put on, and a name could not say what it had
//! on because it had nothing on. A thing that does something in a fight or on the
//! road works from its place now, and there are five, one thing to each:
//!
//!   the hand      a dart, and what each figure begins with (a short sword, a lute,
//!                 a ring of keys, a primer)
//!   the other arm a buckler
//!   the neck      a charm, a lens, a brine glass
//!   the pocket    the one thing used up for you when its moment comes: a bandage,
//!                 a jar, a salve, a splint
//!   the light     a lantern
//!
//! `hold` or `wear` puts a thing you carry in its place (it knows which; whatever
//! was there goes back in the satchel); `stow` takes it out. What has no place (a
//! material, a ribbon marker, a commonplace book) works from the satchel as it did.
//!
//! A thing newly made or taken goes to its place by itself when the place is empty,
//! so a first lantern lights the map as it always has. What a name carried before
//! this was built is left in the satchel for it to place (David's call): the world
//! keeps the number of the first thing made after (`places_from`, on the Docent's
//! entity, set when the world first wakes with this built), and each name the newest thing it has
//! been asked about (`places_seen`).
//!
//! The record is the name's: `places`, `{place: {id, origin}}`. A thing that leaves
//! the satchel (dropped, given, traded, spent) has left its place with it; a spent
//! thing's place is taken by another of its kind if one is carried.

use super::craft::{self, ITEM};
use super::{plain_str, CITIZEN};
use crate::world::World;
use foxing_registry::{Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The five, in the order they are said: its key, where a thing in it is, and the
/// place said bare.
pub const PLACES: [(&str, &str, &str); 5] = [
    ("hand", "in your hand", "your hand"),
    ("off", "on your other arm", "your other arm"),
    ("neck", "at your neck", "your neck"),
    ("pocket", "in your pocket", "your pocket"),
    ("light", "lit, as your light", "your light"),
];

/// What each figure begins with in its hand (David, 11-25: "b", each a real thing,
/// and "1 yes" to what each does): the figure, the thing's kind, its name, and
/// what it does, small and of its loop.
pub const BEGINS: [(&str, &str, &str, &str); 4] = [
    ("hunter", "sword", "a short sword", "in your hand, your blows land a little more often (one more in twenty)"),
    ("teller", "lute", "a small lute", "in your hand, the first story you tell a creature in a day pays a tenth more"),
    ("keeper", "keys", "a ring of keys", "in your hand, you mend half again as fast resting in a room of your own"),
    ("reader", "primer", "a primer", "in your hand, your first wrong answer to a slip each day costs nothing: the slip hears you again at once"),
];

/// What a figure's own thing does, by its kind.
pub fn begins_does(kind: &str) -> Option<&'static str> {
    BEGINS.iter().find(|b| b.1 == kind).map(|b| b.3)
}

/// Give a figure the thing it begins with, if it carries none of that kind: into
/// the satchel, and `settle` puts it in the hand if the hand is empty.
pub fn begin(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    figure: &str,
) -> Result<Option<&'static str>, CommandError> {
    let Some((_, kind, name, _)) = BEGINS.iter().find(|b| b.0 == figure) else {
        return Ok(None);
    };
    if carried(world, citizen)
        .iter()
        .any(|i| origin(world, *i) == *kind)
    {
        return Ok(None);
    }
    let s = craft::satchel(world, actor, citizen)?;
    let curves = super::Curves::from_env();
    craft::make(
        world,
        &curves,
        foxing_log::Actor::Driver,
        Some(actor.to_string()),
        s,
        name,
        "made",
        kind,
        None,
        &[],
        0,
        false,
    )?;
    Ok(Some(name))
}

/// The place a thing of this family works from, if it has one.
pub fn place_of(family: &str) -> Option<&'static str> {
    Some(match family {
        "dart" | "sword" | "lute" | "keys" | "primer" => "hand",
        "buckler" => "off",
        "charm" | "lens" | "brine" | "brine-glass" => "neck",
        "bandage" | "jar" | "salve" | "splint" => "pocket",
        "lantern" => "light",
        _ => return None,
    })
}

fn family(world: &World, item: EntityId) -> String {
    plain_str(world, item, ITEM, "effect")
        .filter(|e| !e.is_empty())
        .or_else(|| plain_str(world, item, ITEM, "origin"))
        .unwrap_or("")
        .to_string()
}

fn origin(world: &World, item: EntityId) -> String {
    plain_str(world, item, ITEM, "origin")
        .unwrap_or("")
        .to_string()
}

fn satchel_of(world: &World, citizen: EntityId) -> Option<EntityId> {
    world
        .store
        .children(citizen)
        .find(|e| e.kind == "satchel")
        .map(|e| e.id)
}

fn carried(world: &World, citizen: EntityId) -> Vec<EntityId> {
    let Some(s) = satchel_of(world, citizen) else {
        return Vec::new();
    };
    let mut ids: Vec<EntityId> = world
        .store
        .children(s)
        .filter(|e| e.kind == "item")
        .map(|e| e.id)
        .collect();
    ids.sort_by_key(|i| i.0);
    ids
}

/// What is in each place: `place -> (item, origin)`.
pub fn map(world: &World, citizen: EntityId) -> BTreeMap<String, (EntityId, String)> {
    let v = match world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(CITIZEN))
        .and_then(|c| c.fields.get("places"))
    {
        Some(Field::Plain(v)) => v.clone(),
        _ => Value::Null,
    };
    v.as_object()
        .map(|o| {
            o.iter()
                .filter_map(|(k, e)| {
                    Some((
                        k.clone(),
                        (
                            EntityId(e["id"].as_u64()?),
                            e["origin"].as_str().unwrap_or("").to_string(),
                        ),
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn keep(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    m: &BTreeMap<String, (EntityId, String)>,
    seen: Option<u64>,
) -> Result<(), CommandError> {
    let mut changes = BTreeMap::new();
    let obj: serde_json::Map<String, Value> = m
        .iter()
        .map(|(k, (id, o))| (k.clone(), json!({"id": id.0, "origin": o})))
        .collect();
    changes.insert("places".to_string(), Field::Plain(Value::Object(obj)));
    if let Some(s) = seen {
        changes.insert("places_seen".to_string(), Field::Plain(json!(s)));
    }
    super::set_citizen_fields(world, actor, citizen, changes)
}

/// Whether a carried thing is at work: in its place, or of a kind that has none.
pub fn at_work(world: &World, citizen: EntityId, item: EntityId) -> bool {
    match place_of(&family(world, item)) {
        None => true,
        Some(p) => map(world, citizen)
            .get(p)
            .is_some_and(|(id, _)| *id == item),
    }
}

/// The thing in a place, if it is still carried.
#[allow(dead_code)] // the picture's (the thing in the hand is drawn there next)
pub fn in_place(world: &World, citizen: EntityId, place: &str) -> Option<EntityId> {
    let (id, _) = map(world, citizen).get(place)?.clone();
    carried(world, citizen).contains(&id).then_some(id)
}

fn name_of(world: &World, item: EntityId) -> String {
    plain_str(world, item, ITEM, "name")
        .unwrap_or("something")
        .to_string()
}

fn where_said(place: &str) -> &'static str {
    PLACES
        .iter()
        .find(|p| p.0 == place)
        .map(|p| p.1)
        .unwrap_or("in its place")
}

/// The line between what a name carried before the places and what it comes by
/// after: drawn once, the first time the world wakes with this built (every thing
/// there is then is older than it).
pub fn ensure(world: &mut World) -> u64 {
    if let Some(n) = super::docent::field(world, "places_from").and_then(|v| v.as_u64()) {
        return n;
    }
    let n = world.store.allocate_id().0;
    let mut c = BTreeMap::new();
    c.insert("places_from".to_string(), json!(n));
    let _ = super::docent::write(world, c);
    n
}

/// After anything a name does: what left the satchel leaves its place (another of
/// its kind takes it, if one is carried), and a thing newly come by goes to its
/// place when that is empty. Says what it did.
pub fn settle(world: &mut World, actor: &str) {
    let Some(f) = super::find_citizen(world, actor) else {
        return;
    };
    let citizen = f.citizen;
    let from = ensure(world);
    let items = carried(world, citizen);
    let mut m = map(world, citizen);
    let before = m.clone();
    let seen_was = plain_str_u64(world, citizen, "places_seen");
    let seen = seen_was.unwrap_or(from).max(from.saturating_sub(1));
    let mut said = Vec::new();
    // What has gone has left its place; another of its kind takes it.
    for (place, (id, org)) in before.iter() {
        if items.contains(id) {
            continue;
        }
        let placed: Vec<EntityId> = m.values().map(|v| v.0).collect();
        match items.iter().find(|i| {
            !placed.contains(i)
                && origin(world, **i) == *org
                && place_of(&family(world, **i)) == Some(place.as_str())
        }) {
            Some(next) => {
                said.push(format!(
                    "Another {} goes {}.",
                    name_of(world, *next)
                        .trim_start_matches("a ")
                        .trim_start_matches("an "),
                    where_said(place)
                ));
                m.insert(place.clone(), (*next, org.clone()));
            }
            None => {
                m.remove(place);
            }
        }
    }
    // What is newly come by goes to its place, if the place is empty.
    let mut newest = seen;
    for i in items.iter().filter(|i| i.0 > seen) {
        newest = newest.max(i.0);
        if let Some(p) = place_of(&family(world, *i)) {
            if !m.contains_key(p) {
                m.insert(p.to_string(), (*i, origin(world, *i)));
                said.push(format!(
                    "{} goes {} (`satchel` says what is at work; `stow` puts a thing away).",
                    super::called::capital(&name_of(world, *i)),
                    where_said(p)
                ));
            } else {
                said.push(format!(
                    "{} stays in the satchel: {} is taken (`hold {}` changes them over).",
                    super::called::capital(&name_of(world, *i)),
                    PLACES
                        .iter()
                        .find(|x| x.0 == p)
                        .map(|x| x.2)
                        .unwrap_or("its place"),
                    name_of(world, *i)
                        .trim_start_matches("a ")
                        .trim_start_matches("an ")
                ));
            }
        }
    }
    // A name met for the first time since the places were built, carrying things
    // from before: told once that they wait for it (David: "players to place").
    if seen_was.is_none() {
        let waiting: Vec<String> = items
            .iter()
            .filter(|i| i.0 <= seen && place_of(&family(world, **i)).is_some())
            .map(|i| name_of(world, *i))
            .collect();
        if !waiting.is_empty() {
            let names: Vec<&str> = waiting.iter().map(String::as_str).collect();
            said.push(format!(
                "Things work from their places now: the hand, the other arm, the neck, the pocket and your light, one thing to each. What you carried before waits in the satchel and does nothing there: {}. `hold <thing>` puts one in its place; `satchel` says what is at work.",
                super::called::listed(&names)
            ));
        }
    }
    if m != before || seen_was != Some(newest) {
        let _ = keep(world, actor, citizen, &m, Some(newest));
    }
    for s in said {
        super::combat::narrate(world, actor, s);
    }
}

fn plain_str_u64(world: &World, citizen: EntityId, field: &str) -> Option<u64> {
    match world
        .store
        .entity(citizen)?
        .components
        .get(CITIZEN)?
        .fields
        .get(field)?
    {
        Field::Plain(v) => v.as_u64(),
        _ => None,
    }
}

/// For `satchel`: what is at work, place by place, and what waits to be placed.
pub fn said(world: &World, citizen: EntityId) -> (String, Value) {
    let m = map(world, citizen);
    let items = carried(world, citizen);
    let mut parts = Vec::new();
    let mut data = serde_json::Map::new();
    for (key, _, bare) in PLACES {
        match m.get(key).filter(|(id, _)| items.contains(id)) {
            Some((id, _)) => {
                parts.push(format!(
                    "{}: {}",
                    bare.trim_start_matches("your "),
                    name_of(world, *id)
                ));
                data.insert(
                    key.to_string(),
                    json!({"id": id.0, "name": name_of(world, *id), "origin": origin(world, *id)}),
                );
            }
            None => {
                parts.push(format!("{}: nothing", bare.trim_start_matches("your ")));
                data.insert(key.to_string(), Value::Null);
            }
        }
    }
    let idle: Vec<String> = items
        .iter()
        .filter(|i| place_of(&family(world, **i)).is_some() && !m.values().any(|v| v.0 == **i))
        .map(|i| name_of(world, *i))
        .collect();
    let mut text = format!("At work: {}.", parts.join("; "));
    if !idle.is_empty() {
        let names: Vec<&str> = idle.iter().map(String::as_str).collect();
        text.push_str(&format!(
            " In the satchel and not at work until it is put in its place (`hold <thing>`, or `wear`): {}.",
            super::called::listed(&names)
        ));
    }
    (text, json!({"places": Value::Object(data), "idle": idle}))
}

pub fn register(world: &mut World) {
    for (verb, about) in [
        ("hold", "Put a thing you carry in its place, where it works from: a dart or your figure's own thing in the hand, a buckler on the other arm, a charm or a lens at the neck, a bandage, jar, salve or splint in the pocket, a lantern as your light. One thing to a place; what was there goes back in the satchel. `wear` is the same word. Bare: what is at work."),
        ("wear", "As `hold`: put a thing you carry in its place (the thing knows which), where it works from. Bare: what is at work."),
    ] {
        world.register(
            CommandSpec::new(format!("disremembered.{verb}"), about).param("thing", ParamType::Entity, false, "a thing in your satchel, by its name"),
            Box::new(|w: &mut World, ctx, args: &Args| {
                let f = super::me(w, ctx)?;
                let what = args.str("thing").unwrap_or("").trim().to_string();
                if what.is_empty() {
                    let (text, data) = said(w, f.citizen);
                    return Ok(Outcome::new(text, data));
                }
                let s = craft::satchel(w, &ctx.actor, f.citizen)?;
                let item = craft::resolve_item(w, &what, s).map_err(|_| {
                    CommandError::precondition(format!("You carry nothing called {what}. `satchel` lists what you carry."))
                })?;
                let name = name_of(w, item);
                let Some(place) = place_of(&family(w, item)) else {
                    return Err(CommandError::precondition(format!(
                        "{} has no place on you: it does what it does from the satchel.",
                        super::called::capital(&name)
                    )));
                };
                let mut m = map(w, f.citizen);
                if m.get(place).is_some_and(|(id, _)| *id == item) {
                    return Ok(Outcome::new(format!("{} is {} already.", super::called::capital(&name), where_said(place)), json!({"place": place, "item": item.0})));
                }
                let was = m.get(place).map(|(id, _)| *id).filter(|id| w.store.entity(*id).is_some());
                m.insert(place.to_string(), (item, origin(w, item)));
                keep(w, &ctx.actor, f.citizen, &m, None)?;
                Ok(Outcome::new(
                    format!(
                        "{} is {} now, and at work.{}",
                        super::called::capital(&name),
                        where_said(place),
                        match was {
                            Some(w0) => format!(" {} goes back in the satchel.", super::called::capital(&name_of(w, w0))),
                            None => String::new(),
                        }
                    ),
                    json!({"place": place, "item": item.0, "stowed": was.map(|i| i.0)}),
                ))
            }),
        );
    }
    world.register(
        CommandSpec::new(
            "disremembered.stow",
            "Take a thing out of its place and put it back in the satchel, where it does nothing until it is held or worn again: by the thing's name, or the place's (`stow hand`, `stow pocket`).",
        )
        .param("thing", ParamType::Entity, true, "the thing, or its place: hand, arm, neck, pocket, light"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = super::me(w, ctx)?;
            let what = args.str("thing").unwrap_or("").trim().to_ascii_lowercase();
            let mut m = map(w, f.citizen);
            let by_place = match what.trim_start_matches("my ").trim_start_matches("the ") {
                "hand" => Some("hand"),
                "arm" | "other arm" | "off hand" | "off" => Some("off"),
                "neck" => Some("neck"),
                "pocket" => Some("pocket"),
                "light" => Some("light"),
                _ => None,
            };
            let place = match by_place {
                Some(p) => p.to_string(),
                None => {
                    let s = craft::satchel(w, &ctx.actor, f.citizen)?;
                    let item = craft::resolve_item(w, &what, s).map_err(|_| {
                        CommandError::precondition(format!("You carry nothing called {what}, and there is no such place. The places: hand, arm, neck, pocket, light."))
                    })?;
                    match m.iter().find(|(_, v)| v.0 == item) {
                        Some((p, _)) => p.clone(),
                        None => {
                            return Err(CommandError::precondition(format!(
                                "{} is in the satchel already, and not at work.",
                                super::called::capital(&name_of(w, item))
                            )))
                        }
                    }
                }
            };
            let Some((item, _)) = m.remove(&place) else {
                return Err(CommandError::precondition(format!("There is nothing {}.", where_said(&place))));
            };
            keep(w, &ctx.actor, f.citizen, &m, None)?;
            Ok(Outcome::new(
                format!("{} goes back in the satchel, and does nothing there until you `hold` it again.", super::called::capital(&name_of(w, item))),
                json!({"place": place, "item": item.0}),
            ))
        }),
    );
}
