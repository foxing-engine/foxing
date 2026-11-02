//! Things: materials and what is made of them. The scaffolding of a crafting economy.
//!
//! A creature that comes apart leaves something of itself behind, one time in three:
//! a wisp its ember, a ragpicker a thread, a hollow its water. What is left lies in
//! the room until someone takes it (`take`) into their satchel (`satchel`), and
//! fades there like everything else, slowly, on the palace curve, unless it is made
//! into something (`craft`, from the recipes `recipes` lists: a thing is made of
//! materials you carry and a memory you spend, and remembers who made it). A made
//! thing does something while carried: a lantern keeps the map in your memory twice
//! as long; a moth charm makes your next fight's memory more vivid, once. `drop`
//! leaves a thing where you stand for whoever comes next; things are on the record
//! like everything else (`disremembered.dropped`, `disremembered.took`,
//! `disremembered.made`).
//!
//! Fixtures are things placed to stay: made by a recipe marked `fixed` in a room you
//! may build in (yours, or one you are permitted in; the world's own rooms take only
//! the operator's), standing in the room for `look` to show, never taken, at most
//! three to a room. A thing you made, or a fixture you placed, is yours to shape:
//! `name <name> <object>` once it holds sixty (`remember <chunk> <object>` writes
//! memory into it; a name is written in memory, and goes when what it holds falls
//! under thirty), `describe <text> <object>` a line of your own; `append <text>
//! <fixture>` a line for anyone the room's owner welcomed (a visitor book, at last),
//! or anyone at all on the world's own books. A found thing never takes a name: a
//! wisp ember is a wisp ember, and a thing you did not make is not yours to name.
//! Everything fades: a fixture nobody feeds goes the way of a dug room.

use super::{internal, me, plain_str, rooms, Curves};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use foxing_time::Curve;
use foxing_time::TimedValue;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A thing: `{name, kind (material|made), origin, made_by, made_of, uses, memory}`.
pub const ITEM: &str = "disremembered.item";
/// A creature left something behind: `{creature, kind, room, item, name}`. Public.
pub const LEFT: &str = "disremembered.left";
/// Someone took a thing: `{by, item, name, room}`. Public.
pub const TOOK: &str = "disremembered.took";
/// Someone dropped a thing: `{by, item, name, room}`. Public.
pub const DROPPED: &str = "disremembered.dropped";
/// Someone made a thing: `{by, item, name, recipe, of, spent}`. Public.
pub const MADE: &str = "disremembered.made";
/// One creature in this many leaves something behind.
pub const DROP_ONE_IN: u32 = 3;
/// Fixtures a room holds, at most.
pub const FIXTURES_PER_ROOM: usize = 3;
/// What a thing must hold to take a name, and below what it loses it.
pub const NAME_AT: f64 = 60.0;
pub const NAME_LOST_BELOW: f64 = 30.0;
/// A thing was named, described, or lost its name: `{item, by, name, was, room}`. Public.
pub const NAMED: &str = "disremembered.item_named";
/// Memory went into a thing: `{by, item, name, with, held}`. Public.
pub const KEPT: &str = "disremembered.item_kept";
/// The most a thing's text runs to.
pub const MAX_TEXT_CHARS: usize = 600;

/// The world's own fixtures, placed when the rooms are: `(room key, recipe, name,
/// text, open)`; `open` means any hand may append (the world welcomes everyone to
/// its own books).
pub const SEED_FIXTURES: &[(&str, &str, &str, &str, bool)] = &[
    (
        "overdue-desk",
        "book",
        "the Ledger of Fines",
        "A column of fines that does not end. The hand changes every few lines; the fines go up.",
        true,
    ),
    (
        "division-lobby",
        "book",
        "the Tally Book",
        "Ruled in a single column. Nothing has been entered. The clerk's pen lies across it, capped.",
        false,
    ),
    (
        "dark-reading-room",
        "plaque",
        "the Book Nobody Is Reading",
        "Open at a page about you. The page turns when you look away.",
        false,
    ),
];

/// What each kind leaves behind when it comes apart.
pub const MATERIALS: &[(&str, &str)] = &[
    ("wisp", "a wisp ember"),
    ("hollow", "hollow water"),
    ("revenant", "grave dust"),
    ("inkling", "a dab of ink"),
    ("mothwing", "moth dust"),
    ("ragpicker", "rag thread"),
    ("drowned", "river weed"),
    ("sitter", "a sitter's patience"),
    ("hourhand", "a spare minute"),
    ("keeper", "a small key"),
];

/// What can be made, of what, for what memory, and what it does.
pub struct Recipe {
    pub key: &'static str,
    pub name: &'static str,
    pub of: &'static [&'static str],
    /// The least a spent chunk must be worth; 0 for none needed.
    pub chunk_at_least: f64,
    pub does: &'static str,
    /// How many times it works before it is used up; 0 for as long as it lasts.
    pub uses: u32,
    /// Made in the room, to stay: a fixture.
    pub fixed: bool,
    /// Takes `append`ed lines from other hands (a book); else only its maker's text.
    pub takes_lines: bool,
}

pub const RECIPES: &[Recipe] = &[
    Recipe {
        key: "lantern",
        name: "a lantern",
        of: &["a wisp ember", "rag thread"],
        chunk_at_least: 10.0,
        does: "keeps the map in your memory twice as long while carried",
        uses: 0,
        fixed: false,
        takes_lines: false,
    },
    Recipe {
        key: "charm",
        name: "a moth charm",
        of: &["moth dust", "a dab of ink"],
        chunk_at_least: 5.0,
        does: "makes your next fight's memory more vivid while carried, and is spent by it",
        uses: 1,
        fixed: false,
        takes_lines: false,
    },
    Recipe {
        key: "jar",
        name: "a jar of fog",
        of: &["hollow water", "river weed"],
        chunk_at_least: 0.0,
        does: "does nothing yet; it is fog in a jar, and it keeps",
        uses: 0,
        fixed: false,
        takes_lines: false,
    },
    Recipe {
        key: "book",
        name: "a visitor book",
        of: &["rag thread", "a dab of ink"],
        chunk_at_least: 60.0,
        does: "stands in a room of yours; whoever you welcome may append a line to it, and it fades unless remembered",
        uses: 0,
        fixed: true,
        takes_lines: true,
    },
    Recipe {
        key: "plaque",
        name: "a plaque",
        of: &["a sitter's patience", "moth dust"],
        chunk_at_least: 40.0,
        does: "stands in a room of yours and says what you describe on it, and fades unless remembered",
        uses: 0,
        fixed: true,
        takes_lines: false,
    },
];

/// The material a creature of this kind leaves, by its kind or the kind a hybrid
/// kind begins or ends with.
pub fn material_for(kind: &str) -> Option<&'static str> {
    MATERIALS
        .iter()
        .find(|(k, _)| *k == kind)
        // A hybrid: the kind it begins with, else the kind it ends with.
        .or_else(|| {
            MATERIALS
                .iter()
                .find(|(k, _)| kind.starts_with(&k[..k.len().min(3)]))
        })
        .or_else(|| {
            MATERIALS
                .iter()
                .find(|(k, _)| kind.ends_with(&k[k.len().saturating_sub(3)..]))
        })
        .map(|(_, m)| *m)
}

/// Make a thing in a container (a room, a satchel).
#[allow(clippy::too_many_arguments)]
fn make(
    world: &mut World,
    curves: &Curves,
    actor: Actor,
    owner: Option<String>,
    container: EntityId,
    name: &str,
    kind: &str,
    origin: &str,
    made_by: Option<&str>,
    made_of: &[String],
    uses: u32,
    fixed: bool,
) -> Result<EntityId, CommandError> {
    let id = world.store.allocate_id();
    world
        .record(
            actor.clone(),
            StoreEvent::EntityCreated {
                id,
                kind: "item".into(),
                owner,
                parent: Some(container),
                scopes: vec![],
            },
        )
        .map_err(internal)?;
    let now = world.now();
    let mut fields = BTreeMap::new();
    fields.insert("name".into(), Field::Plain(json!(name)));
    fields.insert("kind".into(), Field::Plain(json!(kind)));
    fields.insert("origin".into(), Field::Plain(json!(origin)));
    fields.insert("made_by".into(), Field::Plain(json!(made_by)));
    fields.insert("made_of".into(), Field::Plain(json!(made_of)));
    fields.insert("uses".into(), Field::Plain(json!(uses)));
    fields.insert("fixed".into(), Field::Plain(json!(fixed)));
    // Things fade on the palace curve: slowly, and never to nothing by halving; the
    // sweep forgets one below the threshold.
    let c: Curve = curves.palace(0.5);
    fields.insert(
        "memory".into(),
        Field::Timed(TimedValue::new(1.0, now, c.clone())),
    );
    fields.insert("held".into(), Field::Timed(TimedValue::new(0.0, now, c)));
    world
        .record(
            actor,
            StoreEvent::ComponentSet {
                id,
                component: ITEM.into(),
                fields,
            },
        )
        .map_err(internal)?;
    Ok(id)
}

/// A creature came apart here: one time in three, it leaves something.
pub fn leave_behind(
    world: &mut World,
    curves: &Curves,
    room: EntityId,
    creature: EntityId,
    kind: &str,
) -> Result<Option<(EntityId, String)>, CommandError> {
    // One in three at the world's drop rate of 1.0; the Clocktower moves it.
    let chance = super::economy::rate(world, "drop") / DROP_ONE_IN as f64;
    if world.roll(1000) as f64 >= chance * 1000.0 {
        return Ok(None);
    }
    let Some(name) = material_for(kind) else {
        return Ok(None);
    };
    let id = make(
        world,
        curves,
        Actor::Driver,
        None,
        room,
        name,
        "material",
        kind,
        None,
        &[],
        0,
        false,
    )?;
    let _ = world.note(
        Actor::Driver,
        LEFT,
        json!({"creature": creature.0, "kind": kind, "room": room.0, "item": id.0, "name": name}),
    );
    Ok(Some((id, name.to_string())))
}

/// The satchel a citizen carries things in, made on first need.
pub fn satchel(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
) -> Result<EntityId, CommandError> {
    if let Some(s) = world
        .store
        .children(citizen)
        .find(|e| e.kind == "satchel")
        .map(|e| e.id)
    {
        return Ok(s);
    }
    let id = world.store.allocate_id();
    world
        .record(
            Actor::Citizen(actor.to_string()),
            StoreEvent::EntityCreated {
                id,
                kind: "satchel".into(),
                owner: Some(actor.to_string()),
                parent: Some(citizen),
                scopes: vec![],
            },
        )
        .map_err(internal)?;
    Ok(id)
}

/// One thing as `{id, name, kind, memory, made_by, made_of, uses}`.
fn describe_item(world: &World, id: EntityId) -> Value {
    let now = world.now();
    let f = |n: &str| {
        world
            .store
            .read_field(id, ITEM, n, now)
            .ok()
            .unwrap_or(Value::Null)
    };
    json!({"id": id.0, "name": f("name"), "kind": f("kind"), "origin": f("origin"), "memory": f("memory"), "made_by": f("made_by"), "made_of": f("made_of"), "uses": f("uses"),
           "fixed": f("fixed") == json!(true), "given_name": f("given_name"), "text": f("text"), "held": f("held"), "kept": f("kept"), "open": f("open") == json!(true)})
}

/// Whether a thing is a fixture: placed to stay, never taken.
pub fn is_fixed(world: &World, id: EntityId) -> bool {
    world
        .store
        .entity(id)
        .and_then(|e| e.components.get(ITEM))
        .and_then(|c| c.fields.get("fixed"))
        .is_some_and(|f| matches!(f, Field::Plain(Value::Bool(true))))
}

/// What a thing holds now, in XP: the memory remembered into it.
pub fn held(world: &World, id: EntityId) -> f64 {
    world
        .store
        .read_field(id, ITEM, "held", world.now())
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

/// A thing as it is called: its given name, else its kind.
pub fn display(world: &World, id: EntityId) -> String {
    plain_str(world, id, ITEM, "given_name")
        .map(str::to_string)
        .unwrap_or_else(|| {
            plain_str(world, id, ITEM, "name")
                .unwrap_or("something")
                .to_string()
        })
}

/// The things in a container that can be carried, as `describe_item` gives them.
pub fn things_in(world: &World, container: EntityId) -> Vec<Value> {
    world
        .store
        .children(container)
        .filter(|e| e.kind == "item" && !is_fixed(world, e.id))
        .map(|e| describe_item(world, e.id))
        .collect()
}

/// The fixtures standing in a room.
pub fn fixtures_in(world: &World, room: EntityId) -> Vec<Value> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "item" && is_fixed(world, e.id))
        .map(|e| describe_item(world, e.id))
        .collect()
}

/// One line per fixture, for `look`: "Standing here: the Ledger of Fines, a visitor
/// book (item-9, open to any hand): \"…\"".
pub fn fixtures_line(world: &World, room: EntityId) -> Option<String> {
    let fx = fixtures_in(world, room);
    if fx.is_empty() {
        return None;
    }
    let parts: Vec<String> = fx
        .iter()
        .map(|v| {
            let kind = v["name"].as_str().unwrap_or("something");
            let who = match v["given_name"].as_str() {
                Some(n) => format!("{n}, {kind}"),
                None => kind.to_string(),
            };
            let by = v["made_by"]
                .as_str()
                .map(|b| format!(", made by {b}"))
                .unwrap_or_default();
            let open = if v["open"] == true {
                ", open to any hand"
            } else {
                ""
            };
            let held = v["held"].as_f64().unwrap_or(0.0);
            let held = if held >= 0.5 {
                format!(", holding {held:.0}")
            } else {
                String::new()
            };
            let text = v["text"]
                .as_str()
                .map(|t| format!(": \"{t}\""))
                .unwrap_or_default();
            format!("{who} (item-{}{by}{open}{held}){text}", v["id"])
        })
        .collect();
    Some(format!("Standing here: {}.", parts.join("; ")))
}

/// One line for a list: "a lantern (item-12, made by Lute, 80%)".
pub fn item_line(v: &Value) -> String {
    let mem = v["memory"].as_f64().unwrap_or(1.0);
    let by = v["made_by"]
        .as_str()
        .map(|b| format!(", made by {b}"))
        .unwrap_or_default();
    let called = match v["given_name"].as_str() {
        Some(n) => format!("{n}, {}", v["name"].as_str().unwrap_or("something")),
        None => v["name"].as_str().unwrap_or("something").to_string(),
    };
    let held = v["held"].as_f64().unwrap_or(0.0);
    let held = if held >= 0.5 {
        format!(", holding {held:.0}")
    } else {
        String::new()
    };
    let text = v["text"]
        .as_str()
        .map(|t| format!(": \"{t}\""))
        .unwrap_or_default();
    format!(
        "{} (item-{}{by}{held}{}){text}",
        called,
        v["id"],
        if mem < 0.95 {
            format!(", {:.0}% remembered", mem * 100.0)
        } else {
            String::new()
        }
    )
}

/// What lies here, for `look`: "On the ground: a wisp ember (item-9)."
pub fn ground_line(world: &World, room: EntityId) -> Option<String> {
    let things = things_in(world, room);
    if things.is_empty() {
        return None;
    }
    Some(format!(
        "On the ground: {}.",
        things.iter().map(item_line).collect::<Vec<_>>().join(", ")
    ))
}

/// Whether a citizen carries a made thing of this recipe.
pub fn carries(world: &World, citizen: EntityId, recipe: &str) -> Option<EntityId> {
    let satchel = world
        .store
        .children(citizen)
        .find(|e| e.kind == "satchel")
        .map(|e| e.id)?;
    world
        .store
        .children(satchel)
        .filter(|e| e.kind == "item")
        .find(|e| plain_str(world, e.id, ITEM, "origin") == Some(recipe))
        .map(|e| e.id)
}

/// Spend one use of a carried thing: gone when its uses run out. The name, for the
/// line that says so.
pub fn spend(world: &mut World, actor: &str, item: EntityId) -> Result<String, CommandError> {
    let name = plain_str(world, item, ITEM, "name")
        .unwrap_or("something")
        .to_string();
    let uses = world
        .store
        .read_field(item, ITEM, "uses", world.now())
        .ok()
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if uses <= 1 {
        world
            .record(
                Actor::Citizen(actor.to_string()),
                StoreEvent::EntityRemoved { id: item },
            )
            .map_err(internal)?;
    } else {
        let mut fields = world
            .store
            .entity(item)
            .and_then(|e| e.components.get(ITEM))
            .map(|c| c.fields.clone())
            .unwrap_or_default();
        fields.insert("uses".into(), Field::Plain(json!(uses - 1)));
        world
            .record(
                Actor::Citizen(actor.to_string()),
                StoreEvent::ComponentSet {
                    id: item,
                    component: ITEM.into(),
                    fields,
                },
            )
            .map_err(internal)?;
    }
    Ok(name)
}

fn resolve_item(world: &World, arg: &str, within: EntityId) -> Result<EntityId, CommandError> {
    let arg = arg.trim();
    let by_id = arg
        .trim_start_matches("item-")
        .parse::<u64>()
        .ok()
        .map(EntityId)
        .filter(|id| {
            world
                .store
                .entity(*id)
                .is_some_and(|e| e.kind == "item" && e.parent == Some(within))
        });
    if let Some(id) = by_id {
        return Ok(id);
    }
    // By name, the first that matches, "a" and "the" aside.
    let want = arg.to_lowercase();
    let want = want
        .trim_start_matches("a ")
        .trim_start_matches("the ")
        .to_string();
    world
        .store
        .children(within)
        .filter(|e| e.kind == "item")
        .find(|e| {
            let by_kind = plain_str(world, e.id, ITEM, "name")
                .map(|n| {
                    let n = n.to_lowercase();
                    n == want || n.trim_start_matches("a ") == want || n.contains(&want)
                })
                .unwrap_or(false);
            let by_given = plain_str(world, e.id, ITEM, "given_name")
                .map(|n| {
                    let n = n.to_lowercase();
                    n == want || n.trim_start_matches("the ") == want || n.contains(&want)
                })
                .unwrap_or(false);
            by_kind || by_given
        })
        .map(|e| e.id)
        .ok_or_else(|| CommandError::precondition(format!("There is no {arg} here to speak of.")))
}

/// An object the verbs that shape things take: a thing in your satchel, or a fixture
/// standing in the room you are in.
pub fn resolve_object(
    world: &World,
    actor: &str,
    citizen: EntityId,
    arg: &str,
) -> Result<EntityId, CommandError> {
    let room = rooms::room_of(world, citizen).ok_or_else(|| internal("citizen has no room"))?;
    if let Some(s) = world
        .store
        .children(citizen)
        .find(|e| e.kind == "satchel")
        .map(|e| e.id)
    {
        if let Ok(id) = resolve_item(world, arg, s) {
            return Ok(id);
        }
    }
    let _ = actor;
    match resolve_item(world, arg, room) {
        Ok(id) if is_fixed(world, id) => Ok(id),
        Ok(_) => Err(CommandError::precondition(format!(
            "{} lies on the ground here; take it first, if it is yours to shape.",
            arg.trim()
        ))),
        Err(_) => Err(CommandError::precondition(format!(
            "There is no {} in your satchel or standing here; satchel and look list them.",
            arg.trim()
        ))),
    }
}

/// Whether `actor` may shape a thing: its maker, and a made thing (a found thing is
/// what it is).
fn mine_to_shape(world: &World, actor: &str, item: EntityId) -> Result<(), CommandError> {
    if plain_str(world, item, ITEM, "kind") == Some("material") {
        return Err(CommandError::precondition(format!(
            "{} is what it is; a found thing takes no name and no words. Make something of it.",
            display(world, item)
        )));
    }
    match plain_str(world, item, ITEM, "made_by") {
        Some(by) if by == actor => Ok(()),
        Some(by) => Err(CommandError::precondition(format!(
            "{} was made by {by}, and a thing you did not make is not yours to shape.",
            display(world, item)
        ))),
        None => Err(CommandError::precondition(format!(
            "{} is the world's own; only an operator shapes it.",
            display(world, item)
        ))),
    }
}

fn set_item_fields(
    world: &mut World,
    actor: Actor,
    item: EntityId,
    changes: BTreeMap<String, Field>,
) -> Result<(), CommandError> {
    let mut fields = world
        .store
        .entity(item)
        .and_then(|e| e.components.get(ITEM))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    fields.extend(changes);
    world
        .record(
            actor,
            StoreEvent::ComponentSet {
                id: item,
                component: ITEM.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(internal)
}

/// Memory into a thing: what it holds grows by the chunk's worth (on the palace
/// curve, slowly), its own memory is renewed, and the giver's name is kept with it.
/// Returns what it was of and what the thing holds now.
pub fn remember_into(
    world: &mut World,
    curves: &Curves,
    actor: &str,
    chunk: EntityId,
    item: EntityId,
) -> Result<(String, f64), CommandError> {
    let now = world.now();
    let worth = world
        .store
        .read_field(chunk, super::CHUNK, "value", now)
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let source = plain_str(world, chunk, super::CHUNK, "source")
        .unwrap_or("something")
        .to_string();
    let me_ = Actor::Citizen(actor.to_string());
    world
        .record(me_.clone(), StoreEvent::EntityRemoved { id: chunk })
        .map_err(internal)?;
    let total = held(world, item) + worth;
    let mut kept: Vec<Value> = world
        .store
        .entity(item)
        .and_then(|e| e.components.get(ITEM))
        .and_then(|c| c.fields.get("kept"))
        .and_then(|f| match f {
            Field::Plain(Value::Array(a)) => Some(a.clone()),
            _ => None,
        })
        .unwrap_or_default();
    kept.push(json!({"by": actor, "with": source, "worth": (worth * 100.0).round() / 100.0}));
    let c: Curve = curves.palace(0.5);
    let mut changes = BTreeMap::new();
    changes.insert(
        "held".into(),
        Field::Timed(TimedValue::new(total, now, c.clone())),
    );
    changes.insert("memory".into(), Field::Timed(TimedValue::new(1.0, now, c)));
    changes.insert("kept".into(), Field::Plain(Value::Array(kept)));
    set_item_fields(world, me_, item, changes)?;
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        KEPT,
        json!({"by": actor, "item": item.0, "name": display(world, item), "with": source, "worth": worth, "held": total}),
    );
    Ok((source, total))
}

/// Whether `actor` may describe a thing (the dry run asks too).
pub fn shape_check(world: &World, actor: &str, item: EntityId) -> Result<(), CommandError> {
    mine_to_shape(world, actor, item)
}

/// Whether `actor` may name a thing now: theirs, and holding enough.
pub fn name_check(world: &World, actor: &str, item: EntityId) -> Result<(), CommandError> {
    mine_to_shape(world, actor, item)?;
    let holds = held(world, item);
    if holds < NAME_AT {
        return Err(CommandError::precondition(format!(
            "{} holds {holds:.0}; a name is written in memory, and it takes {NAME_AT:.0}. remember a chunk into it (remember <chunk> item-{}).",
            display(world, item),
            item.0
        )));
    }
    Ok(())
}

/// Name a thing you made, once it holds enough.
pub fn name_object(
    world: &mut World,
    actor: &str,
    item: EntityId,
    name: &str,
) -> Result<String, CommandError> {
    name_check(world, actor, item)?;
    let name = super::building::check_name(name)?;
    let was = plain_str(world, item, ITEM, "given_name").map(str::to_string);
    let mut changes = BTreeMap::new();
    changes.insert("given_name".into(), Field::Plain(json!(name)));
    set_item_fields(world, Actor::Citizen(actor.to_string()), item, changes)?;
    let room = world.store.entity(item).and_then(|e| e.parent);
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        NAMED,
        json!({"item": item.0, "by": actor, "name": name, "was": was, "room": room.map(|r| r.0)}),
    );
    Ok(name)
}

/// Describe a thing you made: its text, afresh.
pub fn describe_object(
    world: &mut World,
    actor: &str,
    item: EntityId,
    text: &str,
) -> Result<String, CommandError> {
    mine_to_shape(world, actor, item)?;
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() < 3 || text.chars().count() > MAX_TEXT_CHARS {
        return Err(CommandError::precondition(format!(
            "A thing's text is 3 to {MAX_TEXT_CHARS} characters."
        )));
    }
    let mut changes = BTreeMap::new();
    changes.insert("text".into(), Field::Plain(json!(text)));
    set_item_fields(world, Actor::Citizen(actor.to_string()), item, changes)?;
    Ok(text)
}

/// Whether a line may go in a thing from this hand: a fixture that takes lines, open,
/// or the maker's, or the room owner's welcome (the dry run asks too).
pub fn append_check(
    world: &World,
    ctx: &foxing_registry::CallContext,
    item: EntityId,
) -> Result<(), CommandError> {
    if !is_fixed(world, item) {
        return Err(CommandError::precondition(format!(
            "{} is carried, not standing; only a fixture takes lines from others.",
            display(world, item)
        )));
    }
    let room = world
        .store
        .entity(item)
        .and_then(|e| e.parent)
        .ok_or_else(|| internal("a fixture stands in a room"))?;
    let open = world
        .store
        .entity(item)
        .and_then(|e| e.components.get(ITEM))
        .and_then(|c| c.fields.get("open"))
        .is_some_and(|f| matches!(f, Field::Plain(Value::Bool(true))));
    let takes_lines = RECIPES
        .iter()
        .find(|r| Some(r.key) == plain_str(world, item, ITEM, "origin"))
        .is_some_and(|r| r.takes_lines);
    if !takes_lines {
        return Err(CommandError::precondition(format!(
            "{} takes no lines; its maker describes it, and that is all it says.",
            display(world, item)
        )));
    }
    if !open {
        let maker = plain_str(world, item, ITEM, "made_by").map(str::to_string);
        if maker.is_none() {
            return Err(CommandError::precondition(format!(
                "{} is the world's own, and closed; it takes no hand.",
                display(world, item)
            )));
        }
        if maker.as_deref() != Some(ctx.actor.as_str()) {
            super::building::may_append(world, ctx, room).map_err(|_| {
                CommandError::precondition(format!(
                    "{} takes a line only from a hand the room's owner welcomed.",
                    display(world, item)
                ))
            })?;
        }
    }
    Ok(())
}

/// Add a line to a fixture, signed.
pub fn append_object(
    world: &mut World,
    ctx: &foxing_registry::CallContext,
    item: EntityId,
    more: &str,
) -> Result<String, CommandError> {
    append_check(world, ctx, item)?;
    let more = more.split_whitespace().collect::<Vec<_>>().join(" ");
    if more.chars().count() < 3 {
        return Err(CommandError::precondition("Add at least a few characters."));
    }
    let current = plain_str(world, item, ITEM, "text")
        .unwrap_or("")
        .to_string();
    let joined = if current.is_empty() {
        more.clone()
    } else {
        let sep = if current.ends_with(['.', '!', '?', '"', '”']) {
            " "
        } else {
            ". "
        };
        format!("{current}{sep}{more}")
    };
    if joined.chars().count() > MAX_TEXT_CHARS {
        return Err(CommandError::precondition(format!(
            "That would run past {MAX_TEXT_CHARS} characters; the page is full."
        )));
    }
    let mut changes = BTreeMap::new();
    changes.insert("text".into(), Field::Plain(json!(joined)));
    set_item_fields(world, Actor::Citizen(ctx.actor.clone()), item, changes)?;
    Ok(joined)
}

/// The world's own fixtures, placed once; re-running places only what is missing.
pub fn ensure_fixtures(world: &mut World, curves: &Curves) -> Result<(), String> {
    for (room_key, recipe, name, text, open) in SEED_FIXTURES {
        let Some(room) = rooms::room_by_key(world, room_key) else {
            continue;
        };
        let already = fixtures_in(world, room)
            .iter()
            .any(|v| v["given_name"].as_str() == Some(name));
        if already {
            continue;
        }
        let Some(r) = RECIPES.iter().find(|r| r.key == *recipe) else {
            continue;
        };
        let id = make(
            world,
            curves,
            Actor::Driver,
            None,
            room,
            r.name,
            "made",
            r.key,
            None,
            &[],
            0,
            true,
        )
        .map_err(|e| e.to_string())?;
        // The world's own neither fades nor needs feeding.
        let mut changes = BTreeMap::new();
        changes.insert("given_name".into(), Field::Plain(json!(name)));
        changes.insert("text".into(), Field::Plain(json!(text)));
        changes.insert("open".into(), Field::Plain(json!(open)));
        changes.insert(
            "memory".into(),
            Field::Timed(TimedValue::new(1.0, world.now(), Curve::Constant)),
        );
        changes.insert(
            "held".into(),
            Field::Timed(TimedValue::new(NAME_AT, world.now(), Curve::Constant)),
        );
        set_item_fields(world, Actor::Driver, id, changes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Things that have faded past the threshold are forgotten, wherever they lie.
pub fn sweep(world: &mut World) -> Result<(), String> {
    let now = world.now();
    let gone: Vec<(EntityId, String, Option<EntityId>)> = world
        .store
        .entities()
        .filter(|e| e.kind == "item")
        .filter_map(|e| {
            let m = world
                .store
                .read_field(e.id, ITEM, "memory", now)
                .ok()
                .and_then(|v| v.as_f64())?;
            (m < super::memory::FORGET_BELOW).then(|| {
                (
                    e.id,
                    plain_str(world, e.id, ITEM, "name")
                        .unwrap_or("something")
                        .to_string(),
                    e.parent,
                )
            })
        })
        .collect();
    for (id, name, container) in gone {
        world
            .record(Actor::Driver, StoreEvent::EntityRemoved { id })
            .map_err(|e| e.to_string())?;
        let _ = world.note(
            Actor::Driver,
            super::memory::FADED,
            json!({"owner": Value::Null, "chunk": Value::Null, "item": id.0, "source": name, "container": container.map(|c| c.0)}),
        );
    }
    // A name is written in memory: under the bar, it is gone, and the thing is its
    // kind again.
    let unnamed: Vec<(EntityId, String, String, Option<EntityId>)> = world
        .store
        .entities()
        .filter(|e| e.kind == "item")
        .filter(|e| plain_str(world, e.id, ITEM, "given_name").is_some())
        .filter(|e| held(world, e.id) < NAME_LOST_BELOW)
        .map(|e| {
            (
                e.id,
                plain_str(world, e.id, ITEM, "given_name")
                    .unwrap_or("?")
                    .to_string(),
                plain_str(world, e.id, ITEM, "name")
                    .unwrap_or("something")
                    .to_string(),
                e.parent,
            )
        })
        .collect();
    for (id, was, kind, container) in unnamed {
        let mut changes = BTreeMap::new();
        changes.insert("given_name".into(), Field::Plain(Value::Null));
        set_item_fields(world, Actor::Driver, id, changes).map_err(|e| e.to_string())?;
        let _ = world.note(
            Actor::Driver,
            NAMED,
            json!({"item": id.0, "by": Value::Null, "name": Value::Null, "was": was, "room": container.map(|c| c.0)}),
        );
        if is_fixed(world, id) {
            super::combat::narrate_room(
                world,
                id,
                &format!("{kind} here was called {was}. Whatever it was called, it is not now."),
            );
        }
        world.notice(format!(
            "**{was}** ({kind}) has lost its name: what it held fell under {NAME_LOST_BELOW:.0}."
        ));
    }
    Ok(())
}

pub fn register(world: &mut World, curves: Curves) {
    world.register(
        CommandSpec::new(
            "disremembered.satchel",
            "What you carry: materials creatures left behind and things made of them, each with how well it is remembered. Things fade slowly; a made thing does something while carried.",
        ),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let s = satchel(w, &ctx.actor, f.citizen)?;
            let things = things_in(w, s);
            let text = if things.is_empty() {
                "Your satchel is empty. A creature that comes apart leaves something behind one time in three; take it, and recipes says what it makes.".to_string()
            } else {
                format!(
                    "You carry: {}.",
                    things.iter().map(item_line).collect::<Vec<_>>().join(", ")
                )
            };
            Ok(Outcome::new(text, json!({"things": things})))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.take",
            "Pick up a thing lying here, by name or id (item-12), into your satchel.",
        )
        .param(
            "item",
            ParamType::Entity,
            true,
            "a thing here: its name, or item-12",
        ),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let room =
                rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let item = resolve_item(w, args.str("item").unwrap_or(""), room)?;
            if is_fixed(w, item) {
                return Err(CommandError::precondition(format!(
                    "{} stands where it is; a fixture is not taken.",
                    display(w, item)
                )));
            }
            let s = satchel(w, &ctx.actor, f.citizen)?;
            w.record(
                Actor::Citizen(ctx.actor.clone()),
                StoreEvent::EntityMoved {
                    id: item,
                    parent: Some(s),
                },
            )
            .map_err(internal)?;
            let name = display(w, item);
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                TOOK,
                json!({"by": ctx.actor, "item": item.0, "name": name, "room": room.0}),
            );
            super::combat::narrate_room(w, f.citizen, &format!("{} picks up {name}.", ctx.actor));
            Ok(Outcome::new(
                format!("You take {name}. It is in your satchel."),
                json!({"item": item.0, "name": name}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.drop",
            "Leave a thing you carry where you stand, for whoever comes next.",
        )
        .param(
            "item",
            ParamType::Entity,
            true,
            "a thing in your satchel: its name, or item-12",
        ),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let room =
                rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let s = satchel(w, &ctx.actor, f.citizen)?;
            let item = resolve_item(w, args.str("item").unwrap_or(""), s).map_err(|_| {
                CommandError::precondition("You carry no such thing; satchel lists what you have.")
            })?;
            w.record(
                Actor::Citizen(ctx.actor.clone()),
                StoreEvent::EntityMoved {
                    id: item,
                    parent: Some(room),
                },
            )
            .map_err(internal)?;
            let name = display(w, item);
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                DROPPED,
                json!({"by": ctx.actor, "item": item.0, "name": name, "room": room.0}),
            );
            super::combat::narrate_room(w, f.citizen, &format!("{} sets down {name}.", ctx.actor));
            Ok(Outcome::new(
                format!("You set {name} down here."),
                json!({"item": item.0, "name": name, "room": room.0}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.recipes",
            "What can be made: each thing, what it is made of, the memory it costs, and what it does. Marks the ones you could make now.",
        ),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let s = satchel(w, &ctx.actor, f.citizen)?;
            let have: Vec<String> = things_in(w, s)
                .iter()
                .filter_map(|t| t["name"].as_str().map(str::to_string))
                .collect();
            let level = super::memory::level(w, f.window);
            let mut lines = Vec::new();
            let mut list = Vec::new();
            for r in RECIPES {
                let missing: Vec<&str> = r.of.iter().copied().filter(|m| !have.iter().any(|h| h == m)).collect();
                let asks = r.chunk_at_least * super::economy::rate(w, "price");
                let can = missing.is_empty() && level >= asks;
                lines.push(format!(
                    "  {:<8} {}{}: of {}{}; {}.{}",
                    r.key,
                    r.name,
                    if r.fixed { " (a fixture)" } else { "" },
                    r.of.join(" and "),
                    if r.chunk_at_least > 0.0 { format!(" and a chunk worth {asks:.0} or more") } else { String::new() },
                    r.does,
                    if can { " You could make this now.".to_string() } else if !missing.is_empty() { format!(" You lack {}.", missing.join(" and ")) } else { " You lack the memory to spend.".to_string() }
                ));
                list.push(json!({"recipe": r.key, "name": r.name, "of": r.of, "chunk_at_least": asks, "does": r.does, "uses": r.uses, "fixed": r.fixed, "could_make": can, "missing": missing}));
            }
            Ok(Outcome::new(
                format!("What can be made (craft <recipe> [<chunk>]):\n{}", lines.join("\n")),
                json!({"recipes": list}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.craft",
            "Make a thing from a recipe: the materials leave your satchel, the chunk is spent, and the thing is yours, remembering who made it.",
        )
        .param("recipe", ParamType::String, true, "a recipe key from recipes")
        .param("chunk", ParamType::Entity, false, "a chunk of yours to spend, where the recipe asks one"),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let key = args.str("recipe").unwrap_or("").trim().to_lowercase();
            let Some(r) = RECIPES.iter().find(|r| r.key == key || r.name.trim_start_matches("a ") == key) else {
                return Err(CommandError::precondition(format!(
                    "No recipe called {key}. recipes lists them: {}.",
                    RECIPES.iter().map(|r| r.key).collect::<Vec<_>>().join(", ")
                )));
            };
            let s = satchel(w, &ctx.actor, f.citizen)?;
            // A fixture stands in the room: one you may build in, with room to stand.
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            if r.fixed {
                super::kept_only(w, ctx, "place a fixture")?;
                super::building::may_edit(w, ctx, here).map_err(|_| {
                    CommandError::precondition(format!(
                        "{} stands in a room you may build in: yours, or one you are permitted in. This one is not.",
                        r.name
                    ))
                })?;
                if fixtures_in(w, here).len() >= FIXTURES_PER_ROOM {
                    return Err(CommandError::precondition(format!(
                        "This room already holds {FIXTURES_PER_ROOM} fixtures; nothing more stands here."
                    )));
                }
            }
            // The materials, one of each.
            let mut used: Vec<EntityId> = Vec::new();
            let mut used_names: Vec<String> = Vec::new();
            for m in r.of {
                let found = w
                    .store
                    .children(s)
                    .filter(|e| e.kind == "item" && !used.contains(&e.id))
                    .find(|e| plain_str(w, e.id, ITEM, "name") == Some(*m))
                    .map(|e| e.id);
                match found {
                    Some(id) => {
                        used.push(id);
                        used_names.push(m.to_string());
                    }
                    None => {
                        return Err(CommandError::precondition(format!(
                            "{} needs {m}, and you carry none.",
                            r.name
                        )))
                    }
                }
            }
            // The memory spent.
            let mut spent: Option<EntityId> = None;
            let mut spent_worth = 0.0;
            if r.chunk_at_least > 0.0 {
                let arg = args.str("chunk").unwrap_or("").trim().to_string();
                if arg.is_empty() {
                    return Err(CommandError::precondition(format!(
                        "{} takes a chunk worth {:.0} or more; name one (craft {} chunk-N).",
                        r.name, r.chunk_at_least, r.key
                    )));
                }
                let chunk = super::memory::resolve_chunk(w, ctx, &arg, &[f.window])?;
                let worth = w
                    .store
                    .read_field(chunk, super::CHUNK, "value", w.now())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let asks = r.chunk_at_least * super::economy::rate(w, "price");
                if worth < asks {
                    return Err(CommandError::precondition(format!(
                        "Chunk {} is worth {worth:.2}; {} takes {asks:.0} or more at today's prices.",
                        chunk.0, r.name
                    )));
                }
                w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::EntityRemoved { id: chunk })
                    .map_err(internal)?;
                spent = Some(chunk);
                spent_worth = worth;
            }
            for id in &used {
                w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::EntityRemoved { id: *id })
                    .map_err(internal)?;
            }
            let item = make(
                w,
                &curves,
                Actor::Citizen(ctx.actor.clone()),
                Some(ctx.actor.clone()),
                if r.fixed { here } else { s },
                r.name,
                "made",
                r.key,
                Some(&ctx.actor),
                &used_names,
                r.uses,
                r.fixed,
            )?;
            if r.fixed {
                // What was spent is what it holds at the start, so a book made of a
                // sixty can be named at once.
                let mut changes = BTreeMap::new();
                changes.insert("held".into(), Field::Timed(TimedValue::new(spent_worth, w.now(), curves.palace(0.5))));
                set_item_fields(w, Actor::Citizen(ctx.actor.clone()), item, changes)?;
            }
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                MADE,
                json!({"by": ctx.actor, "item": item.0, "name": r.name, "recipe": r.key, "of": used_names, "spent": spent.map(|c| c.0)}),
            );
            super::combat::narrate_room(w, f.citizen, &format!("{} makes {}{}.", ctx.actor, r.name, if r.fixed { ", and it stands here" } else { "" }));
            Ok(Outcome::new(
                format!(
                    "You make {} of {}{}: item-{}. It {}.{}",
                    r.name,
                    used_names.join(" and "),
                    spent.map(|c| format!(", spending chunk-{}", c.0)).unwrap_or_default(),
                    item.0,
                    r.does,
                    if r.fixed { format!(" It stands here, holding {spent_worth:.0}; name it (name \"…\" item-{}) once it holds {NAME_AT:.0}, describe it, remember chunks into it so it lasts.", item.0) } else { String::new() }
                ),
                json!({"item": item.0, "name": r.name, "recipe": r.key, "of": used_names, "spent": spent.map(|c| c.0)}),
            ))
        }),
    );
}
