//! The Bindery: where what was appended is bound into prose, and what a creature
//! holds in scraps is bound into a sheaf.
//!
//! A room with a guest book's worth of appended sentences reads as a run-on in five
//! voices; a visitor book fills in a day; a creature told thirty stories lists them
//! four at a time. Two remedies, in the room of the Lower Stacks where "something is
//! being bound that was never a book":
//!
//! - **Sheaves**, deterministic and the engine's own: a creature's story chunks that
//!   have faded to a tenth of what they were, five or more of them, fold into one
//!   chunk, "a sheaf of N stories", worth their sum, with the gists kept inside it
//!   for `tell` to recall. Nothing is lost but the clutter.
//! - **Bindings**, by an operator's tool with a model behind it (`tools/bind.py`):
//!   `GET /v0/bindery` lists the rooms and books worth binding (long, or much
//!   appended), the tool proposes a rewrite that keeps every name and thing and
//!   credits every hand, checks it mechanically, and applies it with the admin verb
//!   `bind`; the owner may `unbind` once, back to what it was. The binding is on the
//!   record (`disremembered.bound`, with the text before), so the world replays the
//!   same with no model in the loop.

use super::craft::{self, ITEM};
use super::{internal, me, plain_str, rooms, Curves, CHUNK, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A room or a thing was bound: `{target, kind, name, by, model, before, after, hands}`. Public.
pub const BOUND: &str = "disremembered.bound";
/// A binding was undone by the owner: `{target, kind, name, by}`. Public.
pub const UNBOUND: &str = "disremembered.unbound";
/// A creature's faded stories were folded into a sheaf: `{creature, name, folded, worth, chunk}`. Public.
pub const SHEAF: &str = "disremembered.sheaf";

/// A room worth binding: this many sentences appended since it was last bound, or a
/// few appends that have carried it past this length. Length alone never qualifies: a
/// long description with no appends is the owner's own prose, and the Bindery folds
/// many hands, not one.
pub const ROOM_CHARS: usize = 700;
pub const ROOM_APPENDS: u64 = 6;
pub const ROOM_APPENDS_MIN: u64 = 2;
/// A book worth binding: this many lines, or a few lines that run this long.
pub const BOOK_LINES: usize = 8;
pub const BOOK_LINES_MIN: usize = 3;
pub const BOOK_CHARS: usize = 450;
/// A story chunk faded to this fraction of what it was is a scrap; this many scraps make a sheaf.
pub const SCRAP_BELOW: f64 = 0.1;
pub const SHEAF_AT: usize = 5;
/// The gists a sheaf keeps by name; the rest are counted.
pub const SHEAF_GISTS: usize = 8;

fn plain(world: &World, id: EntityId, component: &str, field: &str) -> Option<Value> {
    world
        .store
        .entity(id)
        .and_then(|e| e.components.get(component))
        .and_then(|c| c.fields.get(field))
        .and_then(|f| match f {
            Field::Plain(v) => Some(v.clone()),
            _ => None,
        })
}

/// The rooms and books the Bindery would bind, as `GET /v0/bindery` lists them.
pub fn candidates(world: &World) -> Value {
    let mut out: Vec<Value> = Vec::new();
    for e in world.store.entities() {
        match e.kind.as_str() {
            "room" if e.owner.is_some() => {
                let desc = plain_str(world, e.id, ROOM, "description").unwrap_or("");
                let appends = plain(world, e.id, ROOM, "appends")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if appends >= ROOM_APPENDS
                    || (appends >= ROOM_APPENDS_MIN && desc.chars().count() >= ROOM_CHARS)
                {
                    out.push(json!({
                        "kind": "room",
                        "target": format!("room-{}", e.id.0),
                        "id": e.id.0,
                        "key": plain_str(world, e.id, ROOM, "key"),
                        "name": plain_str(world, e.id, ROOM, "name"),
                        "owner": e.owner,
                        "text": desc,
                        "chars": desc.chars().count(),
                        "appends": appends,
                        "hands": plain(world, e.id, ROOM, "hands").unwrap_or(json!([])),
                        "bound": plain(world, e.id, ROOM, "bound").map(|b| b["at"].clone()),
                    }));
                }
            }
            "item" if craft::is_fixed(world, e.id) => {
                let text = plain_str(world, e.id, ITEM, "text").unwrap_or("");
                let lines = text.split(" | ").filter(|l| !l.trim().is_empty()).count();
                if lines >= BOOK_LINES
                    || (lines >= BOOK_LINES_MIN && text.chars().count() >= BOOK_CHARS)
                {
                    let hands: Vec<String> = text
                        .split(" | ")
                        .filter_map(|l| {
                            l.rsplit_once(" \u{2014} ")
                                .map(|(_, h)| h.trim().to_string())
                        })
                        .collect();
                    out.push(json!({
                        "kind": "book",
                        "target": format!("item-{}", e.id.0),
                        "id": e.id.0,
                        "name": craft::display(world, e.id),
                        "of": plain_str(world, e.id, ITEM, "name"),
                        "maker": plain_str(world, e.id, ITEM, "made_by"),
                        "room": e.parent.and_then(|r| plain_str(world, r, ROOM, "name")),
                        "text": text,
                        "chars": text.chars().count(),
                        "lines": lines,
                        "hands": hands,
                        "bound": plain(world, e.id, ITEM, "bound").map(|b| b["at"].clone()),
                    }));
                }
            }
            _ => {}
        }
    }
    json!({
        "candidates": out,
        "thresholds": {"room_chars": ROOM_CHARS, "room_appends": ROOM_APPENDS, "room_appends_min": ROOM_APPENDS_MIN, "book_lines": BOOK_LINES, "book_lines_min": BOOK_LINES_MIN, "book_chars": BOOK_CHARS},
        "note": "Rooms much appended (or appended a little and long), and books with many lines, as the Bindery would bind them; the owner's own prose alone is never a candidate. bind <target> \"text\" [model] applies a binding (operator); the owner may unbind once.",
    })
}

/// What `bind` and `unbind` take: `room-12`, a room key, or `item-12`.
fn resolve_target(world: &World, arg: &str) -> Result<(EntityId, &'static str), CommandError> {
    let arg = arg.trim();
    if let Some(n) = arg
        .strip_prefix("item-")
        .and_then(|n| n.parse::<u64>().ok())
    {
        let id = EntityId(n);
        if world.store.entity(id).is_some_and(|e| e.kind == "item") && craft::is_fixed(world, id) {
            return Ok((id, "book"));
        }
        return Err(CommandError::precondition(format!(
            "There is no fixture item-{n}."
        )));
    }
    if let Some(n) = arg
        .strip_prefix("room-")
        .and_then(|n| n.parse::<u64>().ok())
    {
        let id = EntityId(n);
        if world.store.entity(id).is_some_and(|e| e.kind == "room") {
            return Ok((id, "room"));
        }
        return Err(CommandError::precondition(format!("There is no room-{n}.")));
    }
    rooms::room_by_key(world, arg)
        .map(|id| (id, "room"))
        .ok_or_else(|| {
            CommandError::precondition(format!("There is no room or fixture called {arg}."))
        })
}

pub(crate) fn set_fields(
    world: &mut World,
    actor: Actor,
    id: EntityId,
    component: &str,
    changes: BTreeMap<String, Field>,
) -> Result<(), CommandError> {
    let mut fields = world
        .store
        .entity(id)
        .and_then(|e| e.components.get(component))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    fields.extend(changes);
    world
        .record(
            actor,
            StoreEvent::ComponentSet {
                id,
                component: component.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(internal)
}

/// Fold a creature's scraps into a sheaf: story chunks faded to [`SCRAP_BELOW`] of
/// what they were, [`SHEAF_AT`] or more of them, become one chunk worth their sum.
pub fn fold_sheaves(world: &mut World, curves: &Curves) -> Result<usize, String> {
    let now = world.now();
    let creatures: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "creature")
        .map(|e| e.id)
        .collect();
    let mut folded = 0;
    for c in creatures {
        let Some(window) = super::mind::window_of(world, c) else {
            continue;
        };
        let scraps: Vec<(EntityId, String, f64)> = world
            .store
            .children(window)
            .filter(|e| e.kind == "chunk")
            .filter_map(|e| {
                let source = plain_str(world, e.id, CHUNK, "source")?.to_string();
                let is_story = source.starts_with("a story: ") || source.contains("'s story of ");
                if !is_story {
                    return None;
                }
                let value = world
                    .store
                    .read_field(e.id, CHUNK, "value", now)
                    .ok()?
                    .as_f64()?;
                let size = super::plain_num(world, e.id, CHUNK, "size").unwrap_or(value);
                (size > 0.0 && value / size < SCRAP_BELOW).then_some((e.id, source, value))
            })
            .collect();
        if scraps.len() < SHEAF_AT {
            continue;
        }
        let worth: f64 = scraps.iter().map(|(_, _, v)| v).sum();
        let gists: Vec<String> = scraps
            .iter()
            .map(|(_, s, _)| {
                s.strip_prefix("a story: ")
                    .unwrap_or(s)
                    .trim_matches('"')
                    .to_string()
            })
            .collect();
        let shown: Vec<String> = gists
            .iter()
            .take(SHEAF_GISTS)
            .map(|g| format!("\"{g}\""))
            .collect();
        let source = format!(
            "a sheaf of {} stories: {}{}",
            gists.len(),
            shown.join("; "),
            if gists.len() > SHEAF_GISTS {
                format!("; and {} more", gists.len() - SHEAF_GISTS)
            } else {
                String::new()
            }
        );
        for (id, _, _) in &scraps {
            world
                .record(Actor::Driver, StoreEvent::EntityRemoved { id: *id })
                .map_err(|e| e.to_string())?;
        }
        let chunk = super::memory::earn_chunk(
            world,
            "nobody",
            window,
            &source,
            worth.max(0.01),
            0.5,
            curves,
        )
        .map_err(|e| e.to_string())?;
        let _ = world.note(
            Actor::Driver,
            SHEAF,
            json!({"creature": c.0, "name": super::mind::name(world, c), "kind": plain_str(world, c, super::CREATURE, "name"), "folded": scraps.len(), "worth": worth, "chunk": chunk.0, "gists": gists}),
        );
        folded += 1;
    }
    Ok(folded)
}

pub fn register(world: &mut World, curves: Curves) {
    world.register(
        CommandSpec::new(
            "disremembered.bind",
            "Operator. Bind a room's description or a book's text into the prose given: what was there is kept on the record, and the owner may unbind once.",
        )
        .param("target", ParamType::Entity, true, "room-12, a room key, or item-12 (a fixture)")
        .param("text", ParamType::Text, true, "the bound text, quoted")
        .param("model", ParamType::String, false, "what wrote it, for the record")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let (id, kind) = resolve_target(w, args.str("target").unwrap_or(""))?;
            let text = args
                .str("text")
                .unwrap_or("")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if text.chars().count() < 20 {
                return Err(CommandError::precondition("A binding is at least twenty characters."));
            }
            let model = args.str("model").unwrap_or("").trim().to_string();
            let (component, field, name, hands) = match kind {
                "room" => (
                    ROOM,
                    "description",
                    plain_str(w, id, ROOM, "name").unwrap_or("?").to_string(),
                    plain(w, id, ROOM, "hands").unwrap_or(json!([])),
                ),
                _ => (
                    ITEM,
                    "text",
                    craft::display(w, id),
                    json!(plain_str(w, id, ITEM, "text").unwrap_or("").split(" | ").filter_map(|l| l.rsplit_once(" \u{2014} ").map(|(_, h)| h.trim().to_string())).collect::<Vec<_>>()),
                ),
            };
            let max = if kind == "room" { super::building::MAX_DESCRIPTION_CHARS } else { craft::MAX_TEXT_CHARS };
            if text.chars().count() > max {
                return Err(CommandError::precondition(format!("A binding is at most {max} characters here.")));
            }
            let before = plain_str(w, id, component, field).unwrap_or("").to_string();
            let mut changes = BTreeMap::new();
            changes.insert(field.into(), Field::Plain(json!(text)));
            changes.insert(
                "bound".into(),
                Field::Plain(json!({"at": w.now().0, "by": ctx.actor, "model": model, "before": before, "hands": hands})),
            );
            if kind == "room" {
                changes.insert("appends".into(), Field::Plain(json!(0)));
            }
            set_fields(w, Actor::Citizen(ctx.actor.clone()), id, component, changes)?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                BOUND,
                json!({"target": id.0, "kind": kind, "name": name, "by": ctx.actor, "model": model, "before": before, "after": text, "hands": hands}),
            );
            w.notice(format!("The Bindery bound **{name}** ({kind}): {} characters became {}.", before.chars().count(), text.chars().count()));
            let _ = super::craft::world_writes(
                w,
                "bindery",
                "the Binding Ledger",
                "the Bindery",
                &format!(
                    "{} UTC: bound {name} ({kind}), {} hand{} folded in, {} characters to {}",
                    super::system::clock(w.now().0),
                    hands.as_array().map(|h| h.len()).unwrap_or(0),
                    if hands.as_array().map(|h| h.len()).unwrap_or(0) == 1 { "" } else { "s" },
                    before.chars().count(),
                    text.chars().count()
                ),
            );
            Ok(Outcome::new(
                format!("{name} is bound: {text}"),
                json!({"target": id.0, "kind": kind, "name": name, "before_chars": before.chars().count(), "after_chars": text.chars().count()}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.unbind",
            "Undo the Bindery's last binding of a room of yours (standing in it) or a fixture you made: the text is as it was. Once; a binding undone is not kept.",
        )
        .param("target", ParamType::Entity, false, "item-12 for a fixture here; nothing for the room you stand in"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let (id, kind, component, field) = match args.str("target").filter(|s| !s.trim().is_empty()) {
                Some(t) => {
                    let item = craft::resolve_object(w, &ctx.actor, f.citizen, t)?;
                    if !craft::is_fixed(w, item) {
                        return Err(CommandError::precondition("Only a fixture is bound; a carried thing has no binding."));
                    }
                    if plain_str(w, item, ITEM, "made_by") != Some(ctx.actor.as_str()) && ctx.kind != ActorKind::Admin {
                        return Err(CommandError::precondition("Only its maker unbinds a fixture."));
                    }
                    (item, "book", ITEM, "text")
                }
                None => {
                    let owner = w.store.entity(here).and_then(|e| e.owner.clone());
                    if owner.as_deref() != Some(ctx.actor.as_str()) && ctx.kind != ActorKind::Admin {
                        return Err(CommandError::precondition("Only the room's owner unbinds it."));
                    }
                    (here, "room", ROOM, "description")
                }
            };
            let Some(bound) = plain(w, id, component, "bound").filter(|b| b.is_object()) else {
                return Err(CommandError::precondition("Nothing here is bound."));
            };
            let before = bound["before"].as_str().unwrap_or("").to_string();
            if before.is_empty() {
                return Err(CommandError::precondition("The binding kept nothing to go back to."));
            }
            let name = if kind == "room" { plain_str(w, id, ROOM, "name").unwrap_or("?").to_string() } else { craft::display(w, id) };
            let mut changes = BTreeMap::new();
            changes.insert(field.into(), Field::Plain(json!(before)));
            changes.insert("bound".into(), Field::Plain(Value::Null));
            set_fields(w, Actor::Citizen(ctx.actor.clone()), id, component, changes)?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                UNBOUND,
                json!({"target": id.0, "kind": kind, "name": name, "by": ctx.actor}),
            );
            Ok(Outcome::new(
                format!("{name} is as it was before the Bindery: {before}"),
                json!({"target": id.0, "kind": kind, "name": name, "text": before}),
            ))
        }),
    );
    let _ = curves;
}
