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
//! `disremembered.made`). Fixtures, things placed to stay, come after this.

use super::{internal, me, plain_str, rooms, Curves};
use crate::world::World;
use moondial_log::Actor;
use moondial_registry::{Args, CommandError, CommandSpec, Outcome, ParamType};
use moondial_store::{EntityId, Field, StoreEvent};
use moondial_time::Curve;
use moondial_time::TimedValue;
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
}

pub const RECIPES: &[Recipe] = &[
    Recipe {
        key: "lantern",
        name: "a lantern",
        of: &["a wisp ember", "rag thread"],
        chunk_at_least: 10.0,
        does: "carried, the map stays in your memory twice as long",
        uses: 0,
    },
    Recipe {
        key: "charm",
        name: "a moth charm",
        of: &["moth dust", "a dab of ink"],
        chunk_at_least: 5.0,
        does: "carried, your next fight's memory is more vivid, and the charm is spent",
        uses: 1,
    },
    Recipe {
        key: "jar",
        name: "a jar of fog",
        of: &["hollow water", "river weed"],
        chunk_at_least: 0.0,
        does: "nothing yet; it is fog in a jar, and it keeps",
        uses: 0,
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
    // Things fade on the palace curve: slowly, and never to nothing by halving; the
    // sweep forgets one below the threshold.
    let c: Curve = curves.palace(0.5);
    fields.insert("memory".into(), Field::Timed(TimedValue::new(1.0, now, c)));
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
    if world.roll(DROP_ONE_IN) != 0 {
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
    json!({"id": id.0, "name": f("name"), "kind": f("kind"), "origin": f("origin"), "memory": f("memory"), "made_by": f("made_by"), "made_of": f("made_of"), "uses": f("uses")})
}

/// The things in a container, as `describe_item` gives them.
pub fn things_in(world: &World, container: EntityId) -> Vec<Value> {
    world
        .store
        .children(container)
        .filter(|e| e.kind == "item")
        .map(|e| describe_item(world, e.id))
        .collect()
}

/// One line for a list: "a lantern (item-12, made by Lute, 80%)".
pub fn item_line(v: &Value) -> String {
    let mem = v["memory"].as_f64().unwrap_or(1.0);
    let by = v["made_by"]
        .as_str()
        .map(|b| format!(", made by {b}"))
        .unwrap_or_default();
    format!(
        "{} (item-{}{by}{})",
        v["name"].as_str().unwrap_or("something"),
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
            plain_str(world, e.id, ITEM, "name")
                .map(|n| {
                    let n = n.to_lowercase();
                    n == want || n.trim_start_matches("a ") == want || n.contains(&want)
                })
                .unwrap_or(false)
        })
        .map(|e| e.id)
        .ok_or_else(|| CommandError::precondition(format!("There is no {arg} here to speak of.")))
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
            let s = satchel(w, &ctx.actor, f.citizen)?;
            w.record(
                Actor::Citizen(ctx.actor.clone()),
                StoreEvent::EntityMoved {
                    id: item,
                    parent: Some(s),
                },
            )
            .map_err(internal)?;
            let name = plain_str(w, item, ITEM, "name").unwrap_or("it").to_string();
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
            let name = plain_str(w, item, ITEM, "name").unwrap_or("it").to_string();
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
                let can = missing.is_empty() && level >= r.chunk_at_least;
                lines.push(format!(
                    "  {:<8} {}: of {}{}; {}.{}",
                    r.key,
                    r.name,
                    r.of.join(" and "),
                    if r.chunk_at_least > 0.0 { format!(" and a chunk worth {:.0} or more", r.chunk_at_least) } else { String::new() },
                    r.does,
                    if can { " You could make this now.".to_string() } else if !missing.is_empty() { format!(" You lack {}.", missing.join(" and ")) } else { " You lack the memory to spend.".to_string() }
                ));
                list.push(json!({"recipe": r.key, "name": r.name, "of": r.of, "chunk_at_least": r.chunk_at_least, "does": r.does, "uses": r.uses, "could_make": can, "missing": missing}));
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
                if worth < r.chunk_at_least {
                    return Err(CommandError::precondition(format!(
                        "Chunk {} is worth {worth:.2}; {} takes {:.0} or more.",
                        chunk.0, r.name, r.chunk_at_least
                    )));
                }
                w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::EntityRemoved { id: chunk })
                    .map_err(internal)?;
                spent = Some(chunk);
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
                s,
                r.name,
                "made",
                r.key,
                Some(&ctx.actor),
                &used_names,
                r.uses,
            )?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                MADE,
                json!({"by": ctx.actor, "item": item.0, "name": r.name, "recipe": r.key, "of": used_names, "spent": spent.map(|c| c.0)}),
            );
            super::combat::narrate_room(w, f.citizen, &format!("{} makes {}.", ctx.actor, r.name));
            Ok(Outcome::new(
                format!(
                    "You make {} of {}{}: item-{}. It {}.",
                    r.name,
                    used_names.join(" and "),
                    spent.map(|c| format!(", spending chunk-{}", c.0)).unwrap_or_default(),
                    item.0,
                    r.does.replacen("carried, ", "", 1)
                ),
                json!({"item": item.0, "name": r.name, "recipe": r.key, "of": used_names, "spent": spent.map(|c| c.0)}),
            ))
        }),
    );
}
