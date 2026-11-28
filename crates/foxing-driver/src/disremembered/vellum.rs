//! C19: the Vellum Way (David, 11-27: ending (c)).
//!
//! A long path of thin rooms off the Unfiled Archive, each a skin scraped and
//! written over. It gives nothing along the way: nothing spawns, nothing is
//! gathered, nothing is dug, and no room of it says where it goes. The rooms are on
//! no sign and answer to no name (`where`, `travel <name>`): the Way is walked, a
//! door at a time, every time.
//!
//! At the end is the Flyleaf, and the Bearer. A kept name that stands before it and
//! sends `commend` gives its **oldest memory** into the Bearer's keeping: the memory
//! stays the name's own (in its window or on its shelf, counted as before), is made
//! whole again up to [`WHOLE_AT`], and from then on does not fade, for as long as
//! the Bearer lives. One memory a name.
//!
//! The Bearer is a creature and may be struck by name. It is as tough as the Kept
//! House's own (a large party of leveled names to fell it). Felled, it lets go of
//! everything it bears: every memory in its keeping fades again from what it is
//! worth at that moment, and each name must walk the Way again. A memory given
//! away, spent or lost leaves its keeping at the Way's next turn.

use super::mind;
use super::{me, plain_num, plain_str, CHUNK, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The door off the Archive, and the keys of the Way's rooms.
#[cfg(test)]
pub const DOOR: &str = "vellum";
#[cfg(test)]
pub const FIRST: &str = "vellum-1";
pub const END: &str = "vellum-end";
/// The creature's kind, and the name it goes by.
pub const BEARER: &str = "bearer";
pub const NAME: &str = "Tacit";
/// What the Bearer holds itself, kept up by the Way.
pub const KEPT_AT: f64 = 100.0;
/// A memory commended is made whole again up to this much.
pub const WHOLE_AT: f64 = 40.0;

pub const COMMENDED: &str = "disremembered.commended";
pub const LET_GO: &str = "disremembered.bearer_let_go";

/// Whether a room is one of the Way's.
pub fn of_way(world: &World, room: EntityId) -> bool {
    plain_str(world, room, ROOM, "key").is_some_and(|k| k.starts_with("vellum-"))
}

/// Whether this creature is the Bearer.
pub fn is_bearer(world: &World, c: EntityId) -> bool {
    plain_str(world, c, CREATURE, "name") == Some(BEARER)
}

fn bearer(world: &World) -> Option<EntityId> {
    world
        .store
        .of_kind("creature")
        .find(|e| is_bearer(world, e.id))
        .map(|e| e.id)
}

fn end(world: &World) -> Option<EntityId> {
    super::rooms::room_by_key(world, END)
}

/// What is borne: name → chunk id, kept on the Flyleaf.
fn borne(world: &World) -> BTreeMap<String, u64> {
    let Some(room) = end(world) else {
        return BTreeMap::new();
    };
    match world
        .store
        .entity(room)
        .and_then(|e| e.components.get(ROOM))
        .and_then(|c| c.fields.get("borne"))
    {
        Some(Field::Plain(Value::Object(m))) => m
            .iter()
            .filter_map(|(k, v)| v.as_u64().map(|id| (k.clone(), id)))
            .collect(),
        _ => BTreeMap::new(),
    }
}

fn keep(world: &mut World, b: &BTreeMap<String, u64>) -> Result<(), String> {
    let Some(room) = end(world) else {
        return Ok(());
    };
    let mut changes = BTreeMap::new();
    changes.insert("borne".into(), Field::Plain(json!(b)));
    super::bindery::set_fields(world, Actor::Driver, room, ROOM, changes)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// The chunk the Bearer keeps for this name, if it does.
pub fn borne_of(world: &World, actor: &str) -> Option<EntityId> {
    borne(world)
        .get(actor)
        .filter(|&&id| id != 0)
        .map(|&id| EntityId(id))
}

/// Whether a memory is in the Bearer's keeping (the `chunks` listing says so).
pub fn is_borne(world: &World, chunk: EntityId) -> bool {
    matches!(
        world
            .store
            .entity(chunk)
            .and_then(|e| e.components.get(CHUNK))
            .and_then(|c| c.fields.get("borne")),
        Some(Field::Plain(Value::Bool(true)))
    )
}

fn mark(world: &mut World, chunk: EntityId, on: bool) -> Result<(), String> {
    world
        .record(
            Actor::Driver,
            StoreEvent::FieldSet {
                id: chunk,
                component: CHUNK.into(),
                field: "borne".into(),
                value: if on { json!(true) } else { Value::Null },
            },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// A memory leaves the Bearer's keeping and fades again, from what it is worth now,
/// as a memory of its kind does where it lies.
fn release(world: &mut World, actor: &str, chunk: EntityId) -> Result<(), String> {
    if world.store.entity(chunk).is_none() {
        return Ok(());
    }
    let vividity = plain_num(world, chunk, CHUNK, "vividity").unwrap_or(0.5);
    let curves = super::Curves::from_env();
    let curve = if plain_str(world, chunk, CHUNK, "tier") == Some("palace") {
        curves.palace(vividity)
    } else {
        super::memory::fade_curve(world, actor, vividity, &curves)
    };
    world
        .record(
            Actor::Driver,
            StoreEvent::FieldRecurved {
                id: chunk,
                component: CHUNK.into(),
                field: "value".into(),
                curve,
            },
        )
        .map_err(|e| e.to_string())?;
    mark(world, chunk, false)
}

/// A borne memory keeps still wherever its name puts it: shelving and reminiscing
/// give a memory the curve of where it lies, and a borne one is given its stillness
/// back.
pub fn hold(world: &mut World, chunk: EntityId) {
    if !is_borne(world, chunk) {
        return;
    }
    let still = world
        .store
        .timed_field(chunk, CHUNK, "value")
        .is_ok_and(|t| matches!(t.curve, foxing_time::Curve::Constant));
    if !still {
        let _ = world.record(
            Actor::Driver,
            StoreEvent::FieldRecurved {
                id: chunk,
                component: CHUNK.into(),
                field: "value".into(),
                curve: foxing_time::Curve::Constant,
            },
        );
    }
}

/// Whether a chunk still lies in this name's own window or on its own shelf.
fn still_theirs(world: &World, actor: &str, chunk: EntityId) -> bool {
    let Some(f) = super::find_citizen(world, actor) else {
        return false;
    };
    world.store.entity(chunk).is_some_and(|e| {
        e.kind == "chunk" && matches!(e.parent, Some(p) if p == f.window || p == f.palace)
    })
}

/// A name's oldest memory: the first made of those in its window and on its shelf.
fn oldest(world: &World, f: &super::Found) -> Option<EntityId> {
    [f.window, f.palace]
        .into_iter()
        .flat_map(|p| world.store.children(p))
        .filter(|e| e.kind == "chunk")
        .map(|e| e.id)
        .min_by_key(|id| id.0)
}

fn top_up(world: &mut World, c: EntityId) -> Result<(), String> {
    let short = KEPT_AT - mind::level(world, c);
    if short >= 5.0 {
        let w = mind::ensure_window_pub(world, c).map_err(|e| e.to_string())?;
        let curves = super::Curves::from_env();
        super::memory::earn_chunk(
            world,
            "nobody",
            w,
            "the Way's keeping: every skin it was written on before this one",
            short,
            0.5,
            &curves,
        )
        .map_err(|e| e.to_string())?;
    }
    mind::check_tier(world, c).map_err(|e| e.to_string())
}

fn raise(world: &mut World, room: EntityId) -> Result<EntityId, String> {
    let def = super::combat::def_by_name(BEARER).ok_or("no creature kind bearer")?;
    let c = super::combat::spawn_one(world, room, def).map_err(|e| e.to_string())?;
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id: c,
                component: super::SPRITE.into(),
                fields: super::sprite_fields(def.glyph, def.tint, "boarder"),
            },
        )
        .map_err(|e| e.to_string())?;
    super::life::set_home(world, c, room)?;
    let mut changes = BTreeMap::new();
    changes.insert("heir_name".into(), Field::Plain(json!(NAME)));
    mind::set_mind_pub(world, Actor::Driver, c, changes).map_err(|e| e.to_string())?;
    top_up(world, c)?;
    Ok(c)
}

/// The Way's rooms are on no sign and answer to no name.
pub fn ensure(world: &mut World) -> Result<(), String> {
    let rooms: Vec<EntityId> = world
        .store
        .of_kind("room")
        .filter(|e| of_way(world, e.id) && !super::plain_bool(world, e.id, ROOM, "hidden"))
        .map(|e| e.id)
        .collect();
    for r in rooms {
        let mut changes = BTreeMap::new();
        changes.insert("hidden".into(), Field::Plain(json!(true)));
        super::bindery::set_fields(world, Actor::Driver, r, ROOM, changes)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The Way's turn (with the Kept House's, every ten minutes): the Bearer stands at
/// the Flyleaf and is kept; what it bears that is no longer its name's is let go;
/// and with no Bearer at all, everything is.
pub fn tick(world: &mut World) -> Result<(), String> {
    let Some(room) = end(world) else {
        return Ok(());
    };
    let mut b = borne(world);
    let was = b.len();
    let mut changed = false;
    match bearer(world) {
        Some(c) => {
            if world.store.entity(c).and_then(|e| e.parent) != Some(room)
                && super::combat::fought_by(world, c).is_none()
            {
                world
                    .record(
                        Actor::Driver,
                        StoreEvent::EntityMoved {
                            id: c,
                            parent: Some(room),
                        },
                    )
                    .map_err(|e| e.to_string())?;
            }
            top_up(world, c)?;
            let gone: Vec<String> = b
                .iter()
                .filter(|(who, &id)| id != 0 && !still_theirs(world, who, EntityId(id)))
                .map(|(who, _)| who.clone())
                .collect();
            // (Watch 30: a borne memory spent into a bandage freed the place, and the
            // next oldest was commended after it. One memory a name: the place stays
            // taken, empty, until the bearer is felled.)
            for who in gone {
                if let Some(id) = b.insert(who.clone(), 0).filter(|&id| id != 0) {
                    release(world, &who, EntityId(id))?;
                    changed = true;
                }
            }
            for id in b.values().filter(|&&id| id != 0) {
                hold(world, EntityId(*id));
            }
        }
        None => {
            // It came apart altogether: nothing is borne, and another stands in its
            // place, holding nothing of anyone's.
            for (who, id) in std::mem::take(&mut b) {
                if id != 0 {
                    release(world, &who, EntityId(id))?;
                }
            }
            raise(world, room)?;
        }
    }
    if b.len() != was || changed {
        keep(world, &b)?;
    }
    Ok(())
}

/// The Bearer was felled (combat.rs): it lets go of everything it bears. What is
/// said to the one who felled it, or `None` for any other creature.
pub fn felled(world: &mut World, c: EntityId) -> Option<String> {
    if !is_bearer(world, c) {
        return None;
    }
    let b: BTreeMap<String, u64> = borne(world)
        .into_iter()
        .filter(|(_, id)| *id != 0)
        .collect();
    let n = b.len();
    for (who, id) in &b {
        let _ = release(world, who, EntityId(*id));
        super::combat::narrate(
            world,
            who,
            format!("Far off, at the end of the Vellum Way, {NAME} the bearer has been felled, and has let go of everything it carried. Your oldest memory is your own to lose again: it fades from now as any memory does. The Way can be walked again."),
        );
    }
    let _ = keep(world, &BTreeMap::new());
    let _ = world.note(Actor::Driver, LET_GO, json!({"memories": n}));
    Some(match n {
        0 => format!("{NAME} was carrying nothing of anyone's, and gets up lighter by one memory of its own."),
        1 => format!("{NAME}'s hands open: the one memory it was keeping for a name is let go, and fades from now like any other."),
        n => format!("{NAME}'s hands open: the {n} memories it was keeping, one each for {n} names, are let go, and fade from now like any others."),
    })
}

/// What the Flyleaf's `look` adds for this name.
pub fn look_line(world: &World, room: EntityId, actor: &str) -> Option<String> {
    if Some(room) != end(world) {
        return None;
    }
    let all = borne(world);
    let n = all.values().filter(|&&id| id != 0).count();
    let carrying = match n {
        0 => "It is carrying nothing yet.".to_string(),
        1 => "It is carrying one memory, for one name.".to_string(),
        n => format!("It is carrying {n} memories, one each for {n} names."),
    };
    let spent = all
        .get(actor)
        .is_some_and(|&id| id == 0 || !still_theirs(world, actor, EntityId(id)));
    let yours = match borne_of(world, actor) {
        _ if spent => " It kept one for you, and you have since spent it or given it away; it holds your place empty, and takes no second while it lives.".to_string(),
        Some(id) => format!(" One is yours: chunk-{}, which does not fade while {NAME} lives.", id.0),
        None => " `commend` gives your oldest memory into its keeping: it stays yours, is brought back to what it was worth when it was made (40 XP at most), and does not fade while the bearer lives. One memory a name, and the name must be kept.".to_string(),
    };
    Some(format!(
        "{NAME} the bearer keeps what it is given. {carrying}{yours}"
    ))
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.commend",
            "At the end of the Vellum Way, before the bearer: give your oldest memory into its keeping. The memory stays yours and where it lies, is brought back to what it was worth when it was made (40 XP at most), and does not fade while the bearer lives. One memory a name; a kept name only. If the bearer is ever felled it lets go of all it carries, and the Way is walked again.",
        )
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, _args: &Args| {
            let f = me(w, ctx)?;
            let here = super::rooms::room_of(w, f.citizen);
            if here.is_none() || here != end(w) {
                return Err(CommandError::precondition(
                    "There is nobody here to commend anything to. The one who keeps memories stands at the end of a long way, and the way does not say where it is.",
                ));
            }
            let Some(c) = bearer(w).filter(|&c| w.store.entity(c).and_then(|e| e.parent) == here) else {
                return Err(CommandError::precondition(
                    "The Flyleaf is empty: the bearer is not here. It is back within ten minutes of being put down; wait for it.",
                ));
            };
            if super::is_passing(w, f.citizen) {
                return Err(CommandError::precondition(format!(
                    "{NAME} looks at you for some time and does not hold out its hands: a passing name is gone in an hour, and it keeps things longer than that. `keep` makes your name one that stays (the first is free); then come back."
                )));
            }
            if super::combat::fought_by(w, c).is_some() {
                return Err(CommandError::precondition(format!(
                    "{NAME} is being fought, and has no hands free."
                )));
            }
            let mut b = borne(w);
            if let Some(&id) = b.get(&ctx.actor) {
                if still_theirs(w, &ctx.actor, EntityId(id)) {
                    return Err(CommandError::precondition(format!(
                        "{NAME} carries one memory for a name, and it has yours: chunk-{id}. It does not fade while the bearer lives."
                    )));
                }
                if id != 0 {
                    release(w, &ctx.actor, EntityId(id)).map_err(CommandError::internal)?;
                    b.insert(ctx.actor.clone(), 0);
                    keep(w, &b).map_err(CommandError::internal)?;
                }
                return Err(CommandError::precondition(format!(
                    "{NAME} carries one memory for a name, and it carried yours until you spent it or gave it away. It holds your place empty, and takes no second while it lives."
                )));
            }
            let Some(chunk) = oldest(w, &f) else {
                return Err(CommandError::precondition(format!(
                    "{NAME} holds out its hands and you have nothing to put in them: you hold no memory at all. Earn one, and walk the Way again."
                )));
            };
            let now = w.now();
            let was = w.store.read_field(chunk, CHUNK, "value", now).ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
            let size = plain_num(w, chunk, CHUNK, "size").unwrap_or(was);
            let whole = was.max(size.min(WHOLE_AT));
            let source = plain_str(w, chunk, CHUNK, "source").unwrap_or("something").to_string();
            let shelved = plain_str(w, chunk, CHUNK, "tier") == Some("palace");
            if whole > was {
                w.record(
                    Actor::Driver,
                    StoreEvent::FieldCommitted { id: chunk, component: CHUNK.into(), field: "value".into(), value: whole },
                )
                .map_err(super::internal)?;
            }
            w.record(
                Actor::Driver,
                StoreEvent::FieldRecurved { id: chunk, component: CHUNK.into(), field: "value".into(), curve: foxing_time::Curve::Constant },
            )
            .map_err(super::internal)?;
            mark(w, chunk, true).map_err(CommandError::internal)?;
            b.insert(ctx.actor.clone(), chunk.0);
            keep(w, &b).map_err(CommandError::internal)?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                COMMENDED,
                json!({"by": ctx.actor, "chunk": chunk.0, "was": was, "now": whole}),
            );
            super::combat::narrate_room_but(
                w,
                f.citizen,
                &format!("{} puts something small into {NAME}'s hands, and {NAME} closes them.", ctx.actor),
                Some(&ctx.actor),
            );
            let mended = if whole > was + 0.05 {
                format!(" It had worn down to {was:.1}; in those hands it is {whole:.1} again.")
            } else {
                format!(" It is worth {whole:.1}.")
            };
            let lies = if shelved {
                " It stays on your shelf, where it counts for nothing until you reminisce, but it will not have faded when you do."
            } else {
                " It stays in your own window and counts toward your level as it always did."
            };
            Ok(Outcome::new(
                format!(
                    "You commend your oldest memory to {NAME}: chunk-{}, {source}.{mended} {NAME} reads it once, scrapes a place for it on its own skin, and writes it there under everything else it carries. It is still yours.{lies} It does not fade while {NAME} lives, and a fall of yours does not take it. If the bearer is ever felled, it lets go of all it carries, and this fades again from that day.",
                    chunk.0
                ),
                json!({"chunk": chunk.0, "was": was, "now": whole, "bearer": c.0}),
            ))
        }),
    );
}
