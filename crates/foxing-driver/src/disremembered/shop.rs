//! C17: the Souvenir Shop (David, 11-25: "The Souvenir Shop with an idiosyncratic
//! shopkeep. Creatures with business / Docents may move certain special objects to
//! the shop and cash them in for world or personal memory chunks").
//!
//! One room off Lantern Row and one shopkeep. A *souvenir* is a thing that is of
//! somewhere or of someone, not a tool: anything found (an ember, a thread, a dab of
//! ink, a ripe hour) and any made thing with a past (a given name, memory remembered
//! into it, a tier in past.rs). A plain tool is refused: the shop sells what things
//! were, not what they do. A figure's own thing is never sold.
//!
//! The shopkeep is idiosyncratic by rule, so every reply can be checked: it prices by
//! what a thing reminds it of (the same thing, the same day, the same verdict:
//! half, plain, half as much again, or double, with its reason said); it pays half
//! for a second of what it bought from anyone that day ("I have one of those now");
//! it remembers what each name has sold it and says so; it gives no change (a thing
//! is bought back with one chunk worth at least the ticket, spent whole); and it
//! does not trade on a government holiday, the Clerks' Sabbath among them.
//!
//! Cashing in is for a memory of your own (`sell <thing>`) or for the world's
//! (`sell <thing> world`): the price and half again goes into the jar by the door,
//! and the next name to walk in, other than whoever filled it, is given all of it
//! as a memory. Anyone may sell: a name, a called creature on its trade
//! (calling.rs), and the Docent, who now and then carries something off the
//! Midden's heap up the Row and cashes it in for the jar. What the shop bought
//! stands on its shelves with a ticket, to be looked at and bought back.

use super::craft::ITEM;
use super::{find_citizen, me, plain_num, plain_str, CHUNK, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const KEY: &str = "souvenir-shop";
pub const KEEPER: &str = "shopkeep";
/// The shop bought a thing: `{by, item, name, paid, to ("seller", "world"), factor,
/// reminded}`. Public.
pub const SOLD: &str = "disremembered.sold";
/// A thing was bought back off the shelves: `{by, item, name, ticket, paid}`. Public.
pub const BOUGHT: &str = "disremembered.bought";
/// The jar was emptied into a name's hands: `{to, worth}`. Public.
pub const JAR: &str = "disremembered.jar";
/// What the shelves hold; the oldest goes to make room.
pub const SHELF_MAX: usize = 24;
/// What is cashed in for the world is worth this much more than for oneself.
pub const WORLD_BONUS: f64 = 1.5;
/// A ticket is the price paid and this much again.
pub const MARKUP: f64 = 1.5;
/// What a found thing is worth before the shopkeep has looked at it, and the most
/// any souvenir is.
pub const FOUND_WORTH: f64 = 4.0;
pub const WORTH_MAX: f64 = 40.0;

/// What a thing may remind the shopkeep of: the factor, and what it says.
const REMINDS: &[(f64, &str)] = &[
    (0.5, "It reminds me of a Tuesday I did not enjoy"),
    (
        0.5,
        "I had an aunt who kept one of these, and I did not care for her",
    ),
    (1.0, "It reminds me of nothing, which is restful"),
    (1.0, "I have seen its like and expect to again"),
    (
        1.0,
        "It is exactly what it is; I respect that and pay for it accordingly",
    ),
    (1.5, "It reminds me of a door I meant to go back to"),
    (1.5, "Somebody was happy near this once; it shows"),
    (
        2.0,
        "It reminds me of the first thing I ever sold, which I have regretted selling ever since",
    ),
];

pub fn room(world: &World) -> Option<EntityId> {
    super::rooms::room_by_key(world, KEY)
}

/// The shopkeep, standing in its shop.
pub fn keeper(world: &World) -> Option<EntityId> {
    let r = room(world)?;
    world
        .store
        .children(r)
        .find(|e| e.kind == "creature" && plain_str(world, e.id, CREATURE, "name") == Some(KEEPER))
        .map(|e| e.id)
}

/// Why the shop is shut, when it is.
pub fn closed(world: &World) -> Option<String> {
    super::life::holiday(world).map(|h| {
        format!("The blind is down and a card hangs in the door: \"Shut for {h}. Things keep.\" The shopkeep does not trade on a government holiday.")
    })
}

/// What a thing is worth as a souvenir before the shopkeep's own opinion of it, or
/// why it is not one.
pub fn worth(world: &World, item: EntityId) -> Result<f64, String> {
    let name = super::craft::display(world, item);
    if super::places::own_of(world, item).is_some() {
        return Err(format!(
            "\"{name} is yours the way your hands are. I do not buy hands.\""
        ));
    }
    if super::craft::is_fixed(world, item) {
        return Err("\"That stays where it stands.\"".to_string());
    }
    let tier = super::past::tier(world, item) as f64;
    let held = super::craft::held(world, item);
    let named = plain_str(world, item, ITEM, "given_name").is_some();
    let made = plain_str(world, item, ITEM, "kind") == Some("made");
    // (Every made thing is of the first tier; the second and third are the ones
    // with a past of their own.)
    let tier = if made { (tier - 1.0).max(0.0) } else { tier };
    if made && tier < 1.0 && held < 5.0 && !named {
        return Err(format!(
            "\"{name} is a tool. I sell what things were, not what they do. Give it a name, or remember something into it, and bring it back when it has been somewhere.\""
        ));
    }
    let base = if made { 6.0 } else { FOUND_WORTH };
    Ok((base + held / 3.0 + tier * 6.0 + if named { 3.0 } else { 0.0 }).min(WORTH_MAX))
}

fn hash(s: &str) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

/// What this thing reminds the shopkeep of today: the same thing, the same day, the
/// same verdict.
fn reminds(world: &World, item: EntityId) -> (f64, &'static str) {
    let day = world.now().0 / 86_400_000;
    let name = super::craft::display(world, item).to_lowercase();
    REMINDS[(hash(&format!("{name}:{day}")) % REMINDS.len() as u64) as usize]
}

/// How many of this name the shop has bought today, from anyone.
fn bought_today(world: &World, name: &str) -> usize {
    let day = world.now().0 / 86_400_000;
    world
        .recent_events(5000)
        .iter()
        .filter(|e| e.kind == SOLD && e.at.0 / 86_400_000 == day && e.payload["name"] == name)
        .count()
}

/// How many things this seller has sold the shop, as far back as it remembers.
fn sold_by(world: &World, by: &str) -> usize {
    world
        .recent_events(5000)
        .iter()
        .filter(|e| e.kind == SOLD && e.payload["by"] == by)
        .count()
}

/// What the shopkeep would pay for a thing now, and what it says of it.
pub fn price(world: &World, item: EntityId) -> Result<(f64, String), String> {
    let base = worth(world, item)?;
    let (factor, reminded) = reminds(world, item);
    let name = super::craft::display(world, item);
    let again = bought_today(world, &name) > 0;
    let paid = (base * factor * if again { 0.5 } else { 1.0 }).max(1.0);
    let paid = (paid * 2.0).round() / 2.0;
    let said = format!(
        "\"{reminded}.{}\"",
        if again {
            " And I have one of those now, since this morning; the second is worth half."
        } else {
            ""
        }
    );
    Ok((paid, said))
}

/// What stands on the shelves: the thing, its name and its ticket, oldest first.
pub fn shelf(world: &World) -> Vec<(EntityId, String, f64)> {
    let Some(k) = keeper(world) else {
        return Vec::new();
    };
    let mut all: Vec<(EntityId, String, f64)> = world
        .store
        .children(k)
        .filter(|e| e.kind == "item")
        .map(|e| {
            (
                e.id,
                super::craft::display(world, e.id),
                ticket(world, e.id),
            )
        })
        .collect();
    all.sort_by_key(|x| x.0 .0);
    all
}

/// The ticket on a shelved thing: what was last paid for it and half again, or, the
/// sale forgotten, what it is worth.
fn ticket(world: &World, item: EntityId) -> f64 {
    let paid = world
        .recent_events(5000)
        .iter()
        .rev()
        .find(|e| e.kind == SOLD && e.payload["item"] == item.0)
        .and_then(|e| e.payload["paid"].as_f64())
        .unwrap_or_else(|| worth(world, item).unwrap_or(FOUND_WORTH));
    ((paid * MARKUP).max(2.0) * 2.0).round() / 2.0
}

fn jar_of(world: &World) -> (f64, Vec<String>) {
    let Some(r) = room(world) else {
        return (0.0, Vec::new());
    };
    let v = match world
        .store
        .entity(r)
        .and_then(|e| e.components.get(ROOM))
        .and_then(|c| c.fields.get("jar"))
    {
        Some(Field::Plain(v)) => v.clone(),
        _ => Value::Null,
    };
    (
        v["worth"].as_f64().unwrap_or(0.0),
        v["from"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
    )
}

fn set_jar(world: &mut World, worth: f64, from: Vec<String>) -> Result<(), String> {
    let Some(r) = room(world) else {
        return Ok(());
    };
    let mut changes = BTreeMap::new();
    changes.insert(
        "jar".into(),
        Field::Plain(json!({"worth": worth, "from": from})),
    );
    super::bindery::set_fields(world, Actor::Driver, r, ROOM, changes)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// The shop buys a thing: it goes on the shelf, the sale is on the record, and the
/// price is returned for the caller to pay, or, for the world, goes in the jar with
/// half again. `by` is how the seller is styled.
pub fn buy_from(
    world: &mut World,
    by: &str,
    item: EntityId,
    for_world: bool,
) -> Result<(f64, String), String> {
    let k = keeper(world).ok_or("The shopkeep is not at its counter.")?;
    let (paid, said) = price(world, item)?;
    let (factor, reminded) = reminds(world, item);
    let name = super::craft::display(world, item);
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: item,
                parent: Some(k),
            },
        )
        .map_err(|e| e.to_string())?;
    // The oldest goes to make room; the shopkeep does not say where.
    let all = shelf(world);
    if all.len() > SHELF_MAX {
        for (old, _, _) in all.iter().take(all.len() - SHELF_MAX) {
            let _ = world.record(Actor::Driver, StoreEvent::EntityRemoved { id: *old });
        }
    }
    let worth = if for_world {
        ((paid * WORLD_BONUS) * 2.0).round() / 2.0
    } else {
        paid
    };
    if for_world {
        let (have, mut from) = jar_of(world);
        if !from.iter().any(|f| f == by) {
            from.push(by.to_string());
        }
        set_jar(world, have + worth, from)?;
    }
    let _ = world.note(
        Actor::Driver,
        SOLD,
        json!({"by": by, "item": item.0, "name": name, "paid": worth, "to": if for_world { "world" } else { "seller" }, "factor": factor, "reminded": reminded}),
    );
    Ok((worth, said))
}

/// A name walked into the shop: what is in the jar is theirs, unless they filled it.
pub fn came_in(world: &mut World, actor: &str, to: EntityId) -> Result<Option<String>, String> {
    if Some(to) != room(world) {
        return Ok(None);
    }
    let (worth, from) = jar_of(world);
    if worth < 1.0 || from.iter().any(|f| f == actor) {
        return Ok(None);
    }
    let Some(f) = find_citizen(world, actor) else {
        return Ok(None);
    };
    let curves = super::Curves::from_env();
    super::memory::earn_chunk(
        world,
        actor,
        f.window,
        "what was left in the jar at the Souvenir Shop, for whoever came next",
        worth,
        0.6,
        &curves,
    )
    .map_err(|e| e.to_string())?;
    set_jar(world, 0.0, Vec::new())?;
    let _ = world.note(
        Actor::Driver,
        JAR,
        json!({"to": actor, "worth": worth, "from": from}),
    );
    Ok(Some(format!(
        "The shopkeep tips the jar by the door into your hands without looking up: \"For whoever came next. That is you.\" A memory, {worth:.1} XP, left by {}.",
        list(&from)
    )))
}

fn list(names: &[String]) -> String {
    match names.len() {
        0 => "nobody in particular".to_string(),
        1 => names[0].clone(),
        n => format!("{} and {}", names[..n - 1].join(", "), names[n - 1]),
    }
}

/// The shop's line under `look`: the shelves, the jar, the blind.
pub fn look_line(world: &World, room_id: EntityId) -> Option<String> {
    if Some(room_id) != room(world) {
        return None;
    }
    if let Some(shut) = closed(world) {
        return Some(shut);
    }
    let all = shelf(world);
    let shelves = if all.is_empty() {
        "The shelves are bare, each with a ticket waiting on it.".to_string()
    } else {
        let shown: Vec<String> = all
            .iter()
            .rev()
            .take(8)
            .map(|(_, n, t)| format!("{n} ({t:.1})"))
            .collect();
        format!(
            "On the shelves, each with its ticket: {}{}. `buy <thing> <chunk>` takes one down for a memory worth its ticket or more, spent whole; the shopkeep gives no change.",
            shown.join(", "),
            if all.len() > 8 { format!(", and {} more behind", all.len() - 8) } else { String::new() }
        )
    };
    let (worth, _) = jar_of(world);
    let jar = if worth >= 1.0 {
        format!(" The jar by the door holds {worth:.1}, for whoever comes next.")
    } else {
        " The jar by the door is empty.".to_string()
    };
    Some(format!(
        "{shelves}{jar} `appraise <thing>` asks what a thing of yours would fetch; `sell <thing>` cashes it in for a memory of your own, `sell <thing> world` for the jar, at half as much again."
    ))
}

/// Now and then the Docent carries something off the Midden's heap up the Row and
/// cashes it in for the jar (life.rs, one turn of business in six).
pub fn docent_brings(world: &mut World) -> Result<bool, String> {
    if closed(world).is_some() || keeper(world).is_none() {
        return Ok(false);
    }
    // From the Midden, where the swept go and what is too much rots: never from the
    // Threshold, where a thing carried back to the door is waiting for its owner.
    let Some(threshold) = super::rooms::room_by_key(world, "midden") else {
        return Ok(false);
    };
    let found = world
        .store
        .children(threshold)
        .filter(|e| e.kind == "item")
        .map(|e| e.id)
        .find(|&i| {
            plain_str(world, i, ITEM, "kind") != Some("made")
                && worth(world, i).is_ok()
                && super::past::tier(world, i) < 3
        });
    let Some(item) = found else {
        return Ok(false);
    };
    let name = super::craft::display(world, item);
    super::combat::narrate_room(
        world,
        item,
        &format!("The Docent lifts {name} off the heap before it can rot down, turns it over, and goes off toward Lantern Row with it. Somebody will want that."),
    );
    let (worth, said) = buy_from(world, "the Docent", item, true)?;
    if let Some(k) = keeper(world) {
        super::combat::narrate_room(
            world,
            k,
            &format!("The Docent comes in with {name} and sets it on the counter. The shopkeep looks at it for some time. {said} {worth:.1} goes into the jar by the door, the Docent having no use for it."),
        );
    }
    Ok(true)
}

fn here_in_shop(
    w: &World,
    ctx: &foxing_registry::CallContext,
) -> Result<super::Found, CommandError> {
    let f = me(w, ctx)?;
    let here = super::rooms::room_of(w, f.citizen);
    if here.is_none() || here != room(w) {
        return Err(CommandError::precondition(
            "That is done over the counter of the Souvenir Shop, off Lantern Row (`where \"Souvenir Shop\"`).",
        ));
    }
    if let Some(shut) = closed(w) {
        return Err(CommandError::precondition(shut));
    }
    if keeper(w).is_none() {
        return Err(CommandError::precondition(
            "The shopkeep is not at its counter; it will be back.",
        ));
    }
    Ok(f)
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.appraise",
            "At the Souvenir Shop: ask what a thing you carry would fetch, and why. The shopkeep prices by what a thing reminds it of, the same all day; a plain tool it will not buy.",
        )
        .param("thing", ParamType::Text, true, "a thing you carry, by name or id")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = here_in_shop(w, ctx)?;
            let sat = super::craft::satchel(w, &ctx.actor, f.citizen)?;
            let item = super::craft::resolve_item(w, args.str("thing").unwrap_or(""), sat)?;
            let name = super::craft::display(w, item);
            match price(w, item) {
                Ok((paid, said)) => Ok(Outcome::new(
                    format!(
                        "The shopkeep holds {name} up to the lamp. {said} It would give {paid:.1} for it as a memory of your own (`sell {name}`), or put {:.1} in the jar for whoever comes next (`sell {name} world`).",
                        ((paid * WORLD_BONUS) * 2.0).round() / 2.0
                    ),
                    json!({"item": item.0, "name": name, "price": paid}),
                )),
                Err(why) => Ok(Outcome::new(
                    format!("The shopkeep does not pick {name} up. {why}"),
                    json!({"item": item.0, "name": name, "price": Value::Null}),
                )),
            }
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.sell",
            "At the Souvenir Shop: cash in a souvenir you carry (a found thing, or a made thing with a name or a memory in it). Alone, for a memory of your own; with `world`, the price and half again goes in the jar by the door for the next name to walk in.",
        )
        .param("thing", ParamType::String, true, "a thing you carry, by name or id (quote a name of two words)")
        .param("to", ParamType::String, false, "`world` to cash it in for the jar")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = here_in_shop(w, ctx)?;
            let sat = super::craft::satchel(w, &ctx.actor, f.citizen)?;
            let item = super::craft::resolve_item(w, args.str("thing").unwrap_or(""), sat)?;
            let name = super::craft::display(w, item);
            let to = args.str("to").unwrap_or("").trim().to_lowercase();
            let for_world = matches!(to.as_str(), "world" | "jar" | "the world" | "for the world");
            if !to.is_empty() && !for_world {
                return Err(CommandError::precondition("`sell <thing>` is for a memory of your own; `sell <thing> world` is for the jar."));
            }
            let before = sold_by(w, &ctx.actor);
            let (worth, said) = buy_from(w, &ctx.actor, item, for_world).map_err(|why| {
                CommandError::precondition(format!("The shopkeep does not pick {name} up. {why}"))
            })?;
            let knows = match before {
                0 => " \"I have not bought from you before. I will remember that I have now.\"".to_string(),
                n => format!(" \"That is {} I have had off you. I keep count.\"", if n == 1 { "two".to_string() } else { format!("{} things", n + 1) }),
            };
            let paid = if for_world {
                format!("It drops {worth:.1} into the jar by the door, for whoever comes next, and writes a ticket.")
            } else {
                let curves = super::Curves::from_env();
                let chunk = super::memory::earn_chunk(w, &ctx.actor, f.window, &format!("what the Souvenir Shop gave for {name}"), worth, 0.5, &curves)?;
                format!("It counts {worth:.1} into your hand as a memory (chunk-{}), and writes a ticket.", chunk.0)
            };
            for other in w
                .store
                .children(room(w).unwrap_or(EntityId(0)))
                .filter(|e| e.kind == "citizen" && super::presence::is_here(w, e.id))
                .filter_map(|e| e.owner.clone())
                .filter(|o| o != &ctx.actor)
                .collect::<Vec<_>>()
            {
                super::combat::narrate(w, &other, format!("{} sells the shopkeep {name}; it goes up on a shelf with a ticket.", ctx.actor));
            }
            Ok(Outcome::new(
                format!("The shopkeep holds {name} up to the lamp. {said} {paid} {name} goes up on a shelf.{knows}"),
                json!({"item": item.0, "name": name, "paid": worth, "to": if for_world { "world" } else { "seller" }}),
            ))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.buy",
            "At the Souvenir Shop: take a thing down from the shelves for one chunk of yours worth its ticket or more. The chunk is spent whole; the shopkeep gives no change.",
        )
        .param("thing", ParamType::String, true, "a thing on the shelves, by name or id (quote a name of two words)")
        .param("chunk", ParamType::Entity, true, "a chunk of yours worth the ticket or more; it is spent")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = here_in_shop(w, ctx)?;
            let k = keeper(w).ok_or_else(|| CommandError::precondition("The shopkeep is not at its counter."))?;
            let item = super::craft::resolve_item(w, args.str("thing").unwrap_or(""), k).map_err(|_| {
                CommandError::precondition(format!(
                    "Nothing on the shelves answers to that. On them: {}.",
                    shelf(w).iter().map(|(_, n, t)| format!("{n} ({t:.1})")).collect::<Vec<_>>().join(", ")
                ))
            })?;
            let name = super::craft::display(w, item);
            let asked = ticket(w, item);
            let chunk = super::memory::resolve_chunk(w, ctx, args.str("chunk").unwrap_or(""), &[f.window, f.palace])?;
            let value = w.store.read_field(chunk, CHUNK, "value", w.now()).ok().and_then(|v| v.as_f64()).or_else(|| plain_num(w, chunk, CHUNK, "value")).unwrap_or(0.0);
            if value + 0.05 < asked {
                return Err(CommandError::precondition(format!(
                    "The shopkeep taps the ticket: {asked:.1}. \"That memory is worth {value:.1}. I do not take two small ones for one large one; they are not the same thing.\""
                )));
            }
            let sat = super::craft::satchel(w, &ctx.actor, f.citizen)?;
            w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::EntityRemoved { id: chunk }).map_err(super::internal)?;
            w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::EntityMoved { id: item, parent: Some(sat) }).map_err(super::internal)?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                BOUGHT,
                json!({"by": ctx.actor, "item": item.0, "name": name, "ticket": asked, "paid": value}),
            );
            Ok(Outcome::new(
                format!(
                    "The shopkeep takes {name} down, blows on it, and takes your memory for it ({value:.1} against a ticket of {asked:.1}){}. {name} is in your satchel. \"It was somebody's. Now it is yours, which is a different thing.\"",
                    if value > asked + 0.5 { "; it gives no change, and looks as if change had never been invented" } else { "" }
                ),
                json!({"item": item.0, "name": name, "ticket": asked, "paid": value}),
            ))
        }),
    );
}
