//! X18: old age and the Ossuary.
//!
//! A named creature ends two ways. Nobody feeds it: what it holds fades and it is
//! scenery again (mind.rs). Or it is kept, and grows old: three weeks from the day it
//! was first named it says so, and three days after that it walks to the Ossuary and
//! takes a numbered niche. The niche can be read (its name and kind, who named it,
//! how long it lived, who kept it, how its last stories began). A niche nobody reads
//! for two weeks is swept clean, as the room's text has always said.
//!
//! An heir of its kind stands up where it lived, named, holding a fifth of what the
//! old one held and one of its stories, worn. Nine times in ten the heir has a name
//! of its own and knows nobody; one time in ten it takes the old one's name and
//! knows the old one's keepers (David, 11-18).
//!
//! Time can be bought (David: "future crafted items buy more time, but it becomes
//! less effective each use"): [`extend`] is the hook, a week the first time and half
//! as much each time after. Nothing in the world calls it yet.
//!
//! The house's own (the tapster, the pilots at the door), the bosses and the
//! starfish do not age.

use super::mind::MIND;
use super::{find_citizen, plain_str, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{CommandError, Outcome};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A creature said it is old: `{creature, name, kind, room_name, goes_at}`. Public.
pub const OLD: &str = "disremembered.creature_old";
/// A creature went to the Ossuary: `{name, kind, niche, lived_ms, heir, heir_kept_name}`. Public.
pub const PASSED: &str = "disremembered.creature_passed";
/// A niche nobody read was swept: `{niche, name}`. Public.
pub const SWEPT: &str = "disremembered.niche_swept";

/// From first naming to saying it is old.
pub const LIFE_MS: i64 = 21 * 86_400_000;
/// From saying so to the walk: the keepers' warning.
pub const OLD_FOR_MS: i64 = 3 * 86_400_000;
/// A niche unread this long is swept clean.
pub const NICHE_LIFE_MS: i64 = 14 * 86_400_000;
/// The Ossuary's niches.
pub const NICHES: usize = 24;
/// What bought time gives the first time; halved each time after.
#[allow(dead_code)] // the crafted thing that calls `extend` is still to come
pub const EXTEND_MS: i64 = 7 * 86_400_000;
/// The heir takes the old one's name and its keepers one time in this.
pub const SAME_NAME_ONE_IN: u32 = 10;

fn mind_num(world: &World, creature: EntityId, field: &str) -> Option<i64> {
    match world
        .store
        .entity(creature)?
        .components
        .get(MIND)?
        .fields
        .get(field)?
    {
        Field::Plain(v) => v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)),
        _ => None,
    }
}

fn mind_val(world: &World, creature: EntityId, field: &str) -> Option<Value> {
    match world
        .store
        .entity(creature)?
        .components
        .get(MIND)?
        .fields
        .get(field)?
    {
        Field::Plain(v) if !v.is_null() => Some(v.clone()),
        _ => None,
    }
}

/// Whether this creature grows old at all.
pub fn ages(world: &World, creature: EntityId) -> bool {
    let kind = plain_str(world, creature, CREATURE, "name").unwrap_or("");
    super::mind::name(world, creature).is_some()
        && !super::combat::is_boss(kind)
        && kind != "tapster"
        && !super::starfish::is(world, creature)
        && !super::mind::is_pilot(world, creature)
}

/// When it was first named: its own mark, else the Register's earliest line for it
/// (creatures named before this was written are counted from their naming: some are
/// old on the first day).
pub fn named_since(world: &World, creature: EntityId) -> Option<i64> {
    mind_num(world, creature, "named_since").or_else(|| {
        super::namery::used(world)
            .iter()
            .filter(|u| u["creature"].as_u64() == Some(creature.0))
            .filter_map(|u| u["at"].as_i64())
            .min()
    })
}

/// How long it has lived, less the time bought for it.
pub fn age_ms(world: &World, creature: EntityId) -> Option<i64> {
    let since = named_since(world, creature)?;
    Some(world.now().0 - since - mind_num(world, creature, "life_extra_ms").unwrap_or(0))
}

/// When it said it was old, if it has.
pub fn old_since(world: &World, creature: EntityId) -> Option<i64> {
    mind_num(world, creature, "old_since").filter(|t| *t > 0)
}

fn set(
    world: &mut World,
    creature: EntityId,
    changes: BTreeMap<String, Field>,
) -> Result<(), String> {
    super::mind::set_mind_pub(world, Actor::Driver, creature, changes).map_err(|e| e.to_string())
}

fn days(ms: i64) -> String {
    let d = (ms as f64 / 86_400_000.0).round() as i64;
    match d {
        0 => "under a day".to_string(),
        1 => "a day".to_string(),
        n => format!("{n} days"),
    }
}

/// Buys a creature time: a week the first time, half as much each time after. If
/// that takes it back under old age, it is not old. Returns what was given.
#[allow(dead_code)] // called by the crafted thing to come, and by the test
pub fn extend(world: &mut World, creature: EntityId) -> Result<i64, String> {
    let times = mind_num(world, creature, "life_bought").unwrap_or(0);
    let given = EXTEND_MS >> times.clamp(0, 20);
    let extra = mind_num(world, creature, "life_extra_ms").unwrap_or(0) + given;
    let mut changes = BTreeMap::new();
    changes.insert("life_bought".into(), Field::Plain(json!(times + 1)));
    changes.insert("life_extra_ms".into(), Field::Plain(json!(extra)));
    let young_again =
        named_since(world, creature).is_some_and(|since| world.now().0 - since - extra < LIFE_MS);
    if young_again {
        changes.insert("old_since".into(), Field::Plain(json!(0)));
    }
    set(world, creature, changes)?;
    Ok(given)
}

/// The Ossuary's kept niches, as stored.
pub fn niches(world: &World) -> Vec<Value> {
    let Some(room) = super::rooms::room_by_key(world, "ossuary") else {
        return Vec::new();
    };
    match world
        .store
        .entity(room)
        .and_then(|e| e.components.get(ROOM))
        .and_then(|c| c.fields.get("niches"))
    {
        Some(Field::Plain(Value::Array(a))) => a.clone(),
        _ => Vec::new(),
    }
}

fn keep_niches(world: &mut World, niches: Vec<Value>) -> Result<(), String> {
    let Some(room) = super::rooms::room_by_key(world, "ossuary") else {
        return Ok(());
    };
    let mut changes = BTreeMap::new();
    changes.insert("niches".into(), Field::Plain(Value::Array(niches)));
    super::bindery::set_fields(world, Actor::Driver, room, ROOM, changes)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn last_read(n: &Value) -> i64 {
    n["read_at"]
        .as_i64()
        .unwrap_or(0)
        .max(n["at"].as_i64().unwrap_or(0))
}

/// The world's tick for age: who says they are old, who goes, which niche is swept.
pub fn tick(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    let named: Vec<EntityId> = world
        .store
        .of_kind("creature")
        .map(|e| e.id)
        .filter(|c| ages(world, *c))
        .collect();
    for c in named {
        // A creature named before the mark was kept gets it from the Register, or
        // from now if the Register has no line for it.
        if mind_num(world, c, "named_since").is_none() {
            let since = named_since(world, c).unwrap_or(now);
            let mut changes = BTreeMap::new();
            changes.insert("named_since".into(), Field::Plain(json!(since)));
            set(world, c, changes)?;
        }
        let Some(age) = age_ms(world, c) else {
            continue;
        };
        match old_since(world, c) {
            None if age >= LIFE_MS => say_old(world, c, age)?,
            // Not out from under a fight: it goes when it is let alone.
            Some(since)
                if now - since >= OLD_FOR_MS && super::combat::fought_by(world, c).is_none() =>
            {
                pass(world, c)?
            }
            _ => {}
        }
    }
    // Niches nobody reads are swept.
    let kept = niches(world);
    let (stay, swept): (Vec<Value>, Vec<Value>) = kept
        .into_iter()
        .partition(|n| now - last_read(n) < NICHE_LIFE_MS);
    if !swept.is_empty() {
        keep_niches(world, stay)?;
        for n in swept {
            let _ = world.note(
                Actor::Driver,
                SWEPT,
                json!({"niche": n["n"], "name": n["name"]}),
            );
            if let Some(room) = super::rooms::room_by_key(world, "ossuary") {
                narrate_in(
                    world,
                    room,
                    &format!(
                        "Something goes along the wall and sweeps niche {} clean. Whoever {} was, nobody had read it in two weeks.",
                        n["n"],
                        n["name"].as_str().unwrap_or("it")
                    ),
                );
            }
        }
    }
    Ok(())
}

fn narrate_in(world: &mut World, room: EntityId, text: &str) {
    let present: Vec<String> = world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .collect();
    super::combat::narrate_many(world, &present, text.to_string());
}

fn say_old(world: &mut World, c: EntityId, age: i64) -> Result<(), String> {
    let now = world.now().0;
    let mut changes = BTreeMap::new();
    changes.insert("old_since".into(), Field::Plain(json!(now)));
    set(world, c, changes)?;
    let name = super::mind::display(world, c);
    let room_name = world
        .store
        .entity(c)
        .and_then(|e| e.parent)
        .and_then(|r| plain_str(world, r, ROOM, "name"))
        .unwrap_or("a room")
        .to_string();
    super::combat::narrate_room(
        world,
        c,
        &format!(
            "{name} is slower getting up than it was. \"I am old,\" it says, to nobody, and then to you. \"{} with a name. I will go to the Ossuary in three days, and take a niche. Tell me something before then.\"",
            days(age)
        ),
    );
    for k in super::mind::keepers(world, c) {
        let elsewhere = find_citizen(world, &k).is_some_and(|f| {
            super::presence::is_here(world, f.citizen)
                && super::rooms::room_of(world, f.citizen)
                    != world.store.entity(c).and_then(|e| e.parent)
        });
        if elsewhere {
            super::combat::narrate(
                world,
                &k,
                format!("Word reaches you: {name}, in {room_name}, has said it is old. It goes to the Ossuary in three days."),
            );
        }
    }
    world.notice(format!(
        "**{name}** in {room_name} is old, {} named, and goes to the Ossuary in three days.",
        days(age)
    ));
    let _ = world.note(
        Actor::Driver,
        OLD,
        json!({"creature": c.0, "name": super::mind::name(world, c), "kind": plain_str(world, c, CREATURE, "name"),
               "room_name": room_name, "goes_at": now + OLD_FOR_MS}),
    );
    Ok(())
}

/// The walk: a niche for the old one, and an heir where it lived.
fn pass(world: &mut World, c: EntityId) -> Result<(), String> {
    let now = world.now().0;
    let Some(name) = super::mind::name(world, c) else {
        return Ok(());
    };
    let kind = plain_str(world, c, CREATURE, "name")
        .unwrap_or("creature")
        .to_string();
    let display = super::mind::display(world, c);
    let lived = named_since(world, c).map(|s| now - s).unwrap_or(0);
    let keepers = super::mind::keepers(world, c);
    let namer = mind_val(world, c, "namer").and_then(|v| v.as_str().map(str::to_string));
    let held = super::mind::level(world, c);
    let kept_list = mind_val(world, c, "kept");
    let mut tales = super::tales::of(world, c);
    tales.sort_by_key(|t| std::cmp::Reverse(t.at));
    let lines: Vec<String> = tales
        .iter()
        .take(3)
        .map(|t| {
            t.text
                .split_whitespace()
                .take(10)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    let here = world.store.entity(c).and_then(|e| e.parent);
    let home = super::life::home_of(world, c).or(here);

    // The niche: the lowest number free; the wall full, the one longest unread goes.
    let mut all = niches(world);
    if all.len() >= NICHES {
        if let Some(i) = (0..all.len()).min_by_key(|i| last_read(&all[*i])) {
            all.remove(i);
        }
    }
    let n = (1..=NICHES as u64)
        .find(|n| !all.iter().any(|x| x["n"].as_u64() == Some(*n)))
        .unwrap_or(1);

    // The heir, where the old one lived.
    let same_name = world.roll(SAME_NAME_ONE_IN) == 0;
    let mut heir_said = String::new();
    let mut heir_name: Option<String> = None;
    if let (Some(home), Some(def)) = (home, super::combat::def_by_name(&kind)) {
        let heir = super::combat::spawn_one(world, home, def).map_err(|e| e.to_string())?;
        let window = super::mind::ensure_window_pub(world, heir).map_err(|e| e.to_string())?;
        // A fifth of what the old one held, and enough to keep a name.
        let worth = (held / 5.0).max(super::mind::TIERS[0].1 * 1.25).round();
        let chunk = super::memory::earn_chunk(
            world,
            "nobody",
            window,
            &format!("what {name} left it"),
            worth,
            0.6,
            &super::Curves::from_env(),
        )
        .map_err(|e| e.to_string())?;
        super::mind::disown(world, chunk).map_err(|e| e.to_string())?;
        let mut changes = BTreeMap::new();
        changes.insert(
            "heir_of".into(),
            Field::Plain(json!({"name": name, "niche": n})),
        );
        changes.insert("named_since".into(), Field::Plain(json!(now)));
        if same_name {
            changes.insert("heir_name".into(), Field::Plain(json!(name)));
            if let Some(k) = kept_list.clone() {
                changes.insert("kept".into(), Field::Plain(k));
            }
        }
        set(world, heir, changes)?;
        // The old one gives up its name before the heir is dealt one.
        world
            .record(Actor::Driver, StoreEvent::EntityRemoved { id: c })
            .map_err(|e| e.to_string())?;
        super::mind::check_tier(world, heir).map_err(|e| e.to_string())?;
        super::life::set_home(world, heir, home)?;
        if let Some(t) = tales.first() {
            super::tales::bequeath(world, heir, t, home).map_err(|e| e.to_string())?;
        }
        heir_name = super::mind::name(world, heir);
        let called = super::mind::display(world, heir);
        heir_said = if same_name {
            format!(" A younger {kind} stands up where it lived, and answers to {name}: it knows the ones the old {name} knew.")
        } else {
            format!(" A younger {kind} stands up where it lived, {called}, its heir: it has one of the old one's stories, nearly, and knows nobody yet.")
        };
        narrate_in(
            world,
            home,
            &format!(
                "{display} gets up for the last time, looks the room over, and goes. It does not come back.{heir_said}"
            ),
        );
    } else {
        world
            .record(Actor::Driver, StoreEvent::EntityRemoved { id: c })
            .map_err(|e| e.to_string())?;
    }

    all.push(
        json!({"n": n, "name": name, "kind": kind, "namer": namer, "keepers": keepers,
                    "lived_ms": lived, "at": now, "read_at": now, "lines": lines,
                    "heir": heir_name, "heir_kept_name": same_name}),
    );
    all.sort_by_key(|x| x["n"].as_u64().unwrap_or(0));
    keep_niches(world, all)?;
    if let Some(oss) = super::rooms::room_by_key(world, "ossuary") {
        narrate_in(
            world,
            oss,
            &format!("{display} comes in, old, and walks the wall, and stops at niche {n}, and gets in. The niche has a name on its sill now. `read niche {n}` says whose."),
        );
    }
    for k in &keepers {
        if find_citizen(world, k).is_some_and(|f| super::presence::is_here(world, f.citizen)) {
            super::combat::narrate(
                world,
                k,
                format!("Word reaches you: {display} has gone to the Ossuary, {} named, and has niche {n}.{heir_said}", days(lived)),
            );
        }
    }
    world.notice(format!(
        "**{display}** has gone to the Ossuary after {} with a name: niche {n}.{heir_said}",
        days(lived)
    ));
    let _ = world.note(
        Actor::Driver,
        PASSED,
        json!({"name": name, "kind": kind, "niche": n, "lived_ms": lived, "heir": heir_name, "heir_kept_name": same_name}),
    );
    Ok(())
}

/// What `look` adds: in the Ossuary, which niches are kept; anywhere, who here is old.
pub fn look_line(world: &World, room: EntityId) -> Option<String> {
    let now = world.now().0;
    let mut out = Vec::new();
    if plain_str(world, room, ROOM, "key") == Some("ossuary") {
        let kept = niches(world);
        if !kept.is_empty() {
            let each: Vec<String> = kept
                .iter()
                .map(|n| {
                    format!(
                        "{} ({} the {})",
                        n["n"],
                        n["name"].as_str().unwrap_or("?"),
                        n["kind"].as_str().unwrap_or("creature")
                    )
                })
                .collect();
            out.push(format!(
                "Not every niche is empty: {} {} kept, with a name on the sill: {}. `read niche <number>` reads one, and a niche nobody reads for two weeks is swept.",
                each.len(),
                if each.len() == 1 { "is" } else { "are" },
                each.join(", ")
            ));
        }
    }
    for e in world.store.children(room).filter(|e| e.kind == "creature") {
        if let (Some(since), true) = (old_since(world, e.id), ages(world, e.id)) {
            let left = (OLD_FOR_MS - (now - since)).max(0);
            out.push(format!(
                "{} is old, and goes to the Ossuary in {}.",
                super::mind::display(world, e.id),
                if left < 3_600_000 {
                    "under an hour".to_string()
                } else {
                    super::combat::human(left)
                }
            ));
        } else if let Some(v) = mind_val(world, e.id, "heir_of") {
            if super::mind::name(world, e.id).is_some() {
                out.push(format!(
                    "{} is the heir of {} (niche {} in the Ossuary).",
                    super::mind::display(world, e.id),
                    v["name"].as_str().unwrap_or("one that went"),
                    v["niche"]
                ));
            }
        }
    }
    (!out.is_empty()).then(|| out.join(" "))
}

/// `read niche`, `read "niche 3"`, `read "niche Tinsel"` in the Ossuary. Reading one
/// is what keeps it.
pub fn read(world: &mut World, room: EntityId, arg: &str) -> Option<Result<Outcome, CommandError>> {
    if plain_str(world, room, ROOM, "key") != Some("ossuary") {
        return None;
    }
    let low = arg.trim().to_lowercase();
    let rest = low
        .strip_prefix("niches")
        .or_else(|| low.strip_prefix("niche"))?
        .trim()
        .to_string();
    let mut all = niches(world);
    if all.is_empty() {
        return None;
    }
    let now = world.now().0;
    if rest.is_empty() {
        let each: Vec<String> = all
            .iter()
            .map(|n| {
                format!(
                    "{}: {} the {}, {} with a name, read {} ago",
                    n["n"],
                    n["name"].as_str().unwrap_or("?"),
                    n["kind"].as_str().unwrap_or("creature"),
                    days(n["lived_ms"].as_i64().unwrap_or(0)),
                    super::combat::human(now - last_read(n))
                )
            })
            .collect();
        return Some(Ok(Outcome::new(
            format!(
                "The kept niches, by number. {}. `read \"niche <number>\"` reads one whole, and that reading is what keeps it: two weeks unread and it is swept.",
                each.join("; ")
            ),
            json!({"niches": all}),
        )));
    }
    let i = all.iter().position(|n| {
        rest.parse::<u64>().ok() == n["n"].as_u64()
            || n["name"]
                .as_str()
                .is_some_and(|x| x.eq_ignore_ascii_case(&rest))
    });
    let Some(i) = i else {
        return Some(Err(CommandError::precondition(format!(
            "No kept niche answers to \"{rest}\". `read niche` lists the ones with a name on the sill."
        ))));
    };
    all[i]["read_at"] = json!(now);
    let n = all[i].clone();
    if let Err(e) = keep_niches(world, all) {
        return Some(Err(CommandError::precondition(e)));
    }
    let keepers: Vec<&str> = n["keepers"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let lines: Vec<String> = n["lines"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(|l| format!("\"{l}…\""))
                .collect()
        })
        .unwrap_or_default();
    let heir = match (n["heir"].as_str(), n["heir_kept_name"].as_bool()) {
        (Some(_), Some(true)) => " A younger one answers to its name where it lived.".to_string(),
        (Some(h), _) => format!(" Its heir is {h}."),
        _ => String::new(),
    };
    Some(Ok(Outcome::new(
        format!(
            "Niche {}. Cut on the sill: {} the {}. {}{} with a name, {} ago. {}{}{} You have read it, and it is kept two weeks more.",
            n["n"],
            n["name"].as_str().unwrap_or("?"),
            n["kind"].as_str().unwrap_or("creature"),
            n["namer"].as_str().map(|x| format!("Named by {x}. ")).unwrap_or_default(),
            {
                let d = days(n["lived_ms"].as_i64().unwrap_or(0));
                let mut c = d.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => d,
                }
            },
            super::combat::human(now - n["at"].as_i64().unwrap_or(now)),
            if keepers.is_empty() { "Nobody kept it at the end.".to_string() } else { format!("Kept at the end by {}.", keepers.join(", ")) },
            if lines.is_empty() { " It held no story.".to_string() } else { format!(" The stories it held began: {}.", lines.join("; ")) },
            heir
        ),
        json!({"niche": n}),
    )))
}
