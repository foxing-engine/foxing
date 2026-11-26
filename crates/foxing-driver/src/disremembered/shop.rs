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
//! half, plain, half as much again, or double, with its reason said); each further
//! one of a kind it buys from anyone in a day is worth half the one before ("I have
//! one of those now"), down to half a point, the count starting again at midnight UTC;
//! it remembers what each name has sold it and says so; it gives no change (a thing
//! is bought back with one chunk worth at least the ticket, spent whole); and it
//! does not trade on a government holiday, the Clerks' Sabbath among them.
//!
//! Cashing in is for a memory of your own (`sell <thing>`) or for the world's
//! (`sell <thing> world`): the price and half again goes into the jar by the door,
//! and a name walking in, other than whoever filled it, is given up to ten of it
//! as a memory, once a day. Anyone may sell: a name, a called creature on its trade
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
/// The most the jar gives one name at the door, and it gives a name once a UTC day
/// (Watch 28: the jar and half again was free level for two names in turn).
pub const JAR_TAKE: f64 = 10.0;
/// The least the shopkeep pays for anything.
pub const LEAST: f64 = 0.5;
/// How vivid a memory the shop pays is: the shopkeep's, not yours, so it keeps a
/// little worse than a hard fight's and far worse than a story's (Watch 28: at a
/// half it was a way to make a fading memory keep).
pub const PAID_VIVIDITY: f64 = 0.3;

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

/// The shopkeep is put back at its counter if it stands anywhere else (one fed to a
/// name before feeding was refused walked out on its business).
pub fn keep_in(world: &mut World) -> Result<(), String> {
    let Some(r) = room(world) else {
        return Ok(());
    };
    let strays: Vec<EntityId> = world
        .store
        .of_kind("creature")
        .filter(|e| e.parent != Some(r) && plain_str(world, e.id, CREATURE, "name") == Some(KEEPER))
        .map(|e| e.id)
        .collect();
    for id in strays {
        if keeper(world).is_some() {
            // One is at the counter already: the stray is not the shopkeep.
            continue;
        }
        world
            .record(
                Actor::Driver,
                StoreEvent::EntityMoved {
                    id,
                    parent: Some(r),
                },
            )
            .map_err(|e| e.to_string())?;
    }
    // One shopkeep. (Watch 29: the one fed to a name before feeding was refused stood
    // beside a fresh one the room had spawned, each copying the other.) The plain one
    // stays, with everything on the shelves; the named one goes back to being it.
    let all: Vec<EntityId> = world
        .store
        .children(r)
        .filter(|e| {
            e.kind == "creature" && plain_str(world, e.id, CREATURE, "name") == Some(KEEPER)
        })
        .map(|e| e.id)
        .collect();
    if all.len() > 1 {
        let stays = all
            .iter()
            .copied()
            .find(|c| super::mind::name(world, *c).is_none())
            .unwrap_or(all[0]);
        for go in all.into_iter().filter(|c| *c != stays) {
            let things: Vec<EntityId> = world
                .store
                .children(go)
                .filter(|e| e.kind == "item")
                .map(|e| e.id)
                .collect();
            for t in things {
                world
                    .record(
                        Actor::Driver,
                        StoreEvent::EntityMoved {
                            id: t,
                            parent: Some(stays),
                        },
                    )
                    .map_err(|e| e.to_string())?;
            }
            world
                .record(Actor::Driver, StoreEvent::EntityRemoved { id: go })
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// What the shopkeep says to a memory/// What the shopkeep says to a memory or a story offered it: it deals in them and
/// does not take them in.
pub fn refuses(world: &World, creature: EntityId) -> Option<String> {
    (plain_str(world, creature, CREATURE, "name") == Some(KEEPER)).then(|| {
        "The shopkeep holds up a hand. \"I sell what things remember. I do not take it in myself; a shopkeeper who samples the stock is a customer.\" (It buys things: `appraise <thing>`.)".to_string()
    })
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
    // A tool is a made thing, or anything that does something (a spade off the
    // Mudroom's rack is taken, not made, and is a tool all the same: Watch 28).
    let made = plain_str(world, item, ITEM, "kind") == Some("made")
        || plain_str(world, item, ITEM, "effect").is_some_and(|e| !e.is_empty())
        || name.to_lowercase().contains("spade");
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
    // By what the thing is, never by a name its owner gave it (Watch 28: `name` is
    // free and repeatable, and each new name rolled the verdict again).
    let name = plain_str(world, item, ITEM, "name")
        .map(str::to_lowercase)
        .unwrap_or_else(|| super::craft::display(world, item).to_lowercase());
    REMINDS[(hash(&format!("{name}:{day}")) % REMINDS.len() as u64) as usize]
}

/// The shop's book, kept with the jar on the room: `{worth, from, sold: {name: n},
/// day, today: {thing: n}, took: {name: day}}`. (Watch 28: counted from the event
/// log, the shopkeep's count of what a name had sold it went down as the log moved
/// on, and so did its tickets.)
fn book(world: &World) -> Value {
    let Some(r) = room(world) else {
        return json!({});
    };
    match world
        .store
        .entity(r)
        .and_then(|e| e.components.get(ROOM))
        .and_then(|c| c.fields.get("jar"))
    {
        Some(Field::Plain(v)) if v.is_object() => v.clone(),
        _ => json!({}),
    }
}

fn keep_book(world: &mut World, book: Value) -> Result<(), String> {
    let Some(r) = room(world) else {
        return Ok(());
    };
    let mut changes = BTreeMap::new();
    changes.insert("jar".into(), Field::Plain(book));
    super::bindery::set_fields(world, Actor::Driver, r, ROOM, changes)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn day_of(world: &World) -> i64 {
    world.now().0 / 86_400_000
}

/// How many of this name the shop has bought today (UTC), from anyone.
fn bought_today(world: &World, name: &str) -> u64 {
    let b = book(world);
    if b["day"].as_i64() != Some(day_of(world)) {
        return 0;
    }
    b["today"][name.to_lowercase()].as_u64().unwrap_or(0)
}

/// How many things this seller has sold the shop.
fn sold_by(world: &World, by: &str) -> u64 {
    book(world)["sold"][by].as_u64().unwrap_or(0)
}

/// What the shopkeep would pay for a thing now, and what it says of it.
pub fn price(world: &World, item: EntityId) -> Result<(f64, String), String> {
    let base = worth(world, item)?;
    let (factor, reminded) = reminds(world, item);
    let name = super::craft::display(world, item);
    let had = bought_today(world, &name);
    let again = had > 0;
    let paid = (base * factor * 0.5f64.powi(had.min(8) as i32)).max(LEAST);
    let paid = ((paid * 2.0).round() / 2.0).max(LEAST);
    let said = format!(
        "\"{reminded}.{}\"",
        if again {
            " And I have had one of those today already; each one after the first is worth half the one before, until midnight by the Clocktower's UTC."
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

/// The ticket on a shelved thing: written on it when it was bought (what was paid
/// and half again), or, for a thing shelved before tickets were written down, half
/// again what it is worth.
fn ticket(world: &World, item: EntityId) -> f64 {
    plain_num(world, item, ITEM, "ticket").unwrap_or_else(|| {
        ((worth(world, item).unwrap_or(FOUND_WORTH) * MARKUP).max(2.0) * 2.0).round() / 2.0
    })
}

fn jar_of(world: &World) -> (f64, Vec<String>) {
    let v = book(world);
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

/// The shop buys a thing: it goes on the shelf with its ticket, the sale goes in the
/// book and on the record, and the price is returned for the caller to pay, or, for
/// the world, goes in the jar with half again. `by` is how the seller is styled.
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
    let mut on_it = BTreeMap::new();
    on_it.insert(
        "ticket".into(),
        Field::Plain(json!(((paid * MARKUP).max(2.0) * 2.0).round() / 2.0)),
    );
    on_it.insert("sold_by".into(), Field::Plain(json!(by)));
    super::bindery::set_fields(world, Actor::Driver, item, ITEM, on_it)
        .map_err(|e| e.to_string())?;
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
    let mut b = book(world);
    let day = day_of(world);
    if b["day"].as_i64() != Some(day) {
        b["day"] = json!(day);
        b["today"] = json!({});
    }
    let kind = name.to_lowercase();
    b["today"][&kind] = json!(b["today"][&kind].as_u64().unwrap_or(0) + 1);
    if !b["sold"].is_object() {
        b["sold"] = json!({});
    }
    b["sold"][by] = json!(b["sold"][by].as_u64().unwrap_or(0) + 1);
    if for_world {
        let mut from: Vec<Value> = b["from"].as_array().cloned().unwrap_or_default();
        if !from.iter().any(|f| f == by) {
            from.push(json!(by));
        }
        b["worth"] = json!(b["worth"].as_f64().unwrap_or(0.0) + worth);
        b["from"] = json!(from);
    }
    keep_book(world, b)?;
    let _ = world.note(
        Actor::Driver,
        SOLD,
        json!({"by": by, "item": item.0, "name": name, "paid": worth, "to": if for_world { "world" } else { "seller" }, "factor": factor, "reminded": reminded}),
    );
    Ok((worth, said))
}

/// A name walked into the shop: up to ten of what is in the jar is theirs, once a UTC
/// day, unless they helped fill it.
pub fn came_in(world: &mut World, actor: &str, to: EntityId) -> Result<Option<String>, String> {
    if Some(to) != room(world) {
        return Ok(None);
    }
    let (worth, from) = jar_of(world);
    if worth < 1.0 {
        return Ok(None);
    }
    // Said, where it was silent: why the jar is not for you this time.
    if from.iter().any(|f| f == actor) {
        return Ok(Some(format!("The jar by the door holds {worth:.1}, and some of it is yours: what you put in is for whoever comes next, and that is not you.")));
    }
    let mut b = book(world);
    let day = day_of(world);
    if b["took"][actor].as_i64() == Some(day) {
        return Ok(Some(format!("The jar by the door holds {worth:.1}. It has given to you once today, and gives a name once a day.")));
    }
    let Some(f) = find_citizen(world, actor) else {
        return Ok(None);
    };
    let take = worth.min(JAR_TAKE);
    let left = worth - take;
    let curves = super::Curves::from_env();
    super::memory::earn_chunk(
        world,
        actor,
        f.window,
        "what was left in the jar at the Souvenir Shop, for whoever came next",
        take,
        PAID_VIVIDITY,
        &curves,
    )
    .map_err(|e| e.to_string())?;
    if !b["took"].is_object() {
        b["took"] = json!({});
    }
    // Yesterday's takers are forgotten with yesterday.
    let kept: serde_json::Map<String, Value> = b["took"]
        .as_object()
        .map(|m| {
            m.iter()
                .filter(|(_, d)| d.as_i64() == Some(day))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default();
    b["took"] = Value::Object(kept);
    b["took"][actor] = json!(day);
    b["worth"] = json!(left);
    if left < 1.0 {
        b["worth"] = json!(0.0);
        b["from"] = json!([]);
    }
    keep_book(world, b)?;
    let _ = world.note(
        Actor::Driver,
        JAR,
        json!({"to": actor, "worth": take, "from": from, "left": left}),
    );
    Ok(Some(format!(
        "The shopkeep tips the jar by the door into your hands without looking up: \"For whoever came next. That is you.\" A memory, {take:.1} XP, left by {}.{} The jar gives a name once a day.",
        list(&from),
        if left >= 1.0 { format!(" It stops your hand at ten; {left:.1} stays in the jar for the next.") } else { String::new() }
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
        // Like things are said once, with how many (Watch 28: five stone rolls were
        // five entries, and sixteen more hid behind them).
        let mut groups: Vec<(String, f64, usize)> = Vec::new();
        for (_, n, t) in all.iter().rev() {
            match groups
                .iter_mut()
                .find(|g| g.0 == *n && (g.1 - *t).abs() < 0.01)
            {
                Some(g) => g.2 += 1,
                None => groups.push((n.clone(), *t, 1)),
            }
        }
        let shown: Vec<String> = groups
            .iter()
            .take(12)
            .map(|(n, t, c)| {
                if *c > 1 {
                    format!("{n} x{c} ({t:.1} each)")
                } else {
                    format!("{n} ({t:.1})")
                }
            })
            .collect();
        format!(
            "On the shelves, each with its ticket: {}{}. `appraise <thing>` says what one on a shelf is and where it came from; `buy <thing> <chunk>` takes one down for a memory worth its ticket or more, spent whole (the shopkeep gives no change).",
            shown.join(", "),
            if groups.len() > 12 { format!(", and {} more kinds behind", groups.len() - 12) } else { String::new() }
        )
    };
    let (worth, _) = jar_of(world);
    let jar = if worth >= 1.0 {
        format!(" The jar by the door holds {worth:.1}: a name walking in is given up to {JAR_TAKE:.0} of it, once a day, unless they helped fill it.")
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

/// Over the counter a thing is named in full or by a whole word of its name, or by
/// its id: a letter or two is not enough to sell by (Watch 28: `sell "wi"` sold a
/// wisp ember, and `appraise "a"` found a sword).
fn meant(w: &World, typed: &str, item: EntityId) -> Result<(), CommandError> {
    let t = typed.to_lowercase();
    if t.starts_with("item-") || t.chars().all(|c| c.is_ascii_digit()) {
        return Ok(());
    }
    let name = super::craft::display(w, item);
    let strip = |s: &str| {
        let s = s.trim().to_lowercase();
        ["a ", "an ", "the "]
            .iter()
            .find_map(|a| s.strip_prefix(a).map(str::to_string))
            .unwrap_or(s)
    };
    let (t, n) = (strip(&t), strip(&name));
    if t.len() >= 3
        && (t == n
            || n.split(|c: char| !c.is_alphanumeric())
                .any(|word| word == t)
            || n.contains(&format!("{t} "))
            || n.ends_with(&format!(" {t}")))
    {
        return Ok(());
    }
    Err(CommandError::precondition(format!(
        "The shopkeep waits. \"Which thing? Say its name, or a whole word of it.\" The nearest to '{typed}' is {name}: `{name}` in full names it."
    )))
}

/// A name as typed, without its quotes.
fn unquote(s: &str) -> String {
    s.trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .trim()
        .to_string()
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
            let what = unquote(args.str("thing").unwrap_or(""));
            let item = match super::craft::resolve_item(w, &what, sat) {
                Ok(item) => {
                    meant(w, &what, item)?;
                    item
                }
                Err(mine) => {
                    // Not yours: one on the shelves is looked at instead.
                    let on_shelf = keeper(w).and_then(|k| super::craft::resolve_item(w, &what, k).ok());
                    let Some(item) = on_shelf else {
                        return Err(mine);
                    };
                    meant(w, &what, item)?;
                    let name = super::craft::display(w, item);
                    let asked = ticket(w, item);
                    let from = plain_str(w, item, ITEM, "sold_by").map(|b| format!(" It had it off {b}.")).unwrap_or_default();
                    let origin = plain_str(w, item, ITEM, "origin").filter(|o| !o.is_empty() && *o != "material" && *o != "thing").map(|o| format!(" Of it the ticket says: {o}.")).unwrap_or_default();
                    let held = super::craft::held(w, item);
                    let holds = if held >= 1.0 { format!(" It has {held:.0} of memory in it.") } else { String::new() };
                    return Ok(Outcome::new(
                        format!("The shopkeep takes {name} down and lets you look, not letting go of it. Ticket: {asked:.1}.{from}{origin}{holds} `buy {name} <chunk>` takes it for one memory worth that or more."),
                        json!({"item": item.0, "name": name, "ticket": asked, "shelf": true}),
                    ));
                }
            };
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
        .param("what", ParamType::Text, true, "a thing you carry, by name or id; `world` after it to cash it in for the jar")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = here_in_shop(w, ctx)?;
            let sat = super::craft::satchel(w, &ctx.actor, f.citizen)?;
            // The whole line: a thing's name of any number of words, then perhaps
            // `world` (Watch 28: two words wanted quotes, and said so badly).
            let line = args.str("what").unwrap_or("").trim().to_string();
            let lower = line.to_lowercase();
            let (thing, to) = ["for the world", "the world", "world", "jar"]
                .iter()
                .find_map(|t| lower.strip_suffix(t).filter(|rest| rest.is_empty() || rest.ends_with(' ') || rest.ends_with('"')).map(|rest| (line[..rest.len()].to_string(), t.to_string())))
                .unwrap_or((line.clone(), String::new()));
            let item = super::craft::resolve_item(w, &unquote(&thing), sat)?;
            meant(w, &unquote(&thing), item)?;
            let name = super::craft::display(w, item);
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
                let chunk = super::memory::earn_chunk(w, &ctx.actor, f.window, &format!("what the Souvenir Shop gave for {name}"), worth, PAID_VIVIDITY, &curves)?;
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
        .param("what", ParamType::Text, true, "a thing on the shelves, then a chunk of yours worth its ticket or more (it is spent): buy rag thread marigold")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = here_in_shop(w, ctx)?;
            let k = keeper(w).ok_or_else(|| CommandError::precondition("The shopkeep is not at its counter."))?;
            // The whole line: the last word is the chunk, what comes before it the thing.
            let line = args.str("what").unwrap_or("").trim().to_string();
            let Some((thing, chunk_word)) = line.rsplit_once(char::is_whitespace) else {
                return Err(CommandError::precondition(format!(
                    "`buy <thing> <chunk>`: a thing on the shelves, then one memory of yours worth its ticket or more (`chunks` lists yours). On the shelves: {}.",
                    shelf(w).iter().map(|(_, n, t)| format!("{n} ({t:.1})")).collect::<Vec<_>>().join(", ")
                )));
            };
            let item = super::craft::resolve_item(w, &unquote(thing), k).map_err(|_| {
                CommandError::precondition(format!(
                    "Nothing on the shelves answers to that. On them: {}.",
                    shelf(w).iter().map(|(_, n, t)| format!("{n} ({t:.1})")).collect::<Vec<_>>().join(", ")
                ))
            })?;
            meant(w, &unquote(thing), item)?;
            let name = super::craft::display(w, item);
            // Of like things on the shelves, the cheapest comes down.
            let item = shelf(w)
                .into_iter()
                .filter(|(_, n, _)| *n == name)
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
                .map(|x| x.0)
                .unwrap_or(item);
            let asked = ticket(w, item);
            let chunk = super::memory::resolve_chunk(w, ctx, &unquote(chunk_word), &[f.window, f.palace])?;
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
