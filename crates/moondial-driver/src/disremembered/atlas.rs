//! The map, and what you remember of it.
//!
//! Rooms have no coordinates of their own: the map is the graph of exits, and this
//! module lays it out the same way every time, so a mini-map drawn at two keyboards
//! agrees. A breadth-first walk from the Threshold places each room by the compass word
//! that reached it (north is up the page, east is right, up and down are floors); a
//! room reached by a named exit (kiln, annex, a dug door called "teapot") is set in the
//! nearest free cell beside the room it opens from; and a room whose cell is already
//! taken is shoved to the nearest free one, which the layout says, because a map that
//! disagrees with the compass is worth knowing about. The coherence report lists those
//! shoves and every one-way exit.
//!
//! The map you see is your memory of it. Every room you stand in is marked seen on
//! your `disremembered.atlas`, and the mark halves on the atlas half-life (four times
//! the active one: a day's walk is gone by the next evening). `map` returns the rooms
//! you remember with how well, and the room you stand in with its exits, which you can
//! see; what you have forgotten is not on it. Seed rooms are not spared.

use super::{internal, me, plain_str, rooms, Curves, ROOM};
use crate::world::World;
use moondial_log::Actor;
use moondial_registry::{ActorKind, CommandError, CommandSpec, Outcome};
use moondial_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

/// A citizen's memory of the map: `seen` is `{room id: last seen, ms}`.
pub const ATLAS: &str = "disremembered.atlas";
/// How many active half-lives a room stays on your map.
pub const ATLAS_HALF_LIVES: f64 = 4.0;
/// Below this, a room is off the map.
pub const FORGOTTEN: f64 = 0.05;

/// A room placed on the grid.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub id: EntityId,
    pub key: String,
    pub name: String,
    pub x: i64,
    pub y: i64,
    pub z: i64,
    /// `compass` (where its exit says), `beside` (a named exit; the nearest free cell
    /// by its parent), or `shoved` (its compass cell was taken).
    pub placed: &'static str,
    /// The room it was reached from, and the exit.
    pub from: Option<(EntityId, String)>,
    pub owner: Option<String>,
    pub exits: BTreeMap<String, EntityId>,
}

/// The whole map, and what is wrong with it.
#[derive(Debug, Clone, Default)]
pub struct Layout {
    pub rooms: Vec<Placed>,
    /// Rooms that could not take their compass cell: "A Cellar (down from the Stacks) is
    /// shoved to (1,2,-1)".
    pub shoved: Vec<String>,
    /// Exits with no way back: "north from the Fogbank to the Shallows".
    pub one_way: Vec<String>,
}

fn step(exit: &str) -> Option<(i64, i64, i64)> {
    Some(match exit {
        "north" => (0, -1, 0),
        "south" => (0, 1, 0),
        "east" => (1, 0, 0),
        "west" => (-1, 0, 0),
        "up" => (0, 0, 1),
        "down" => (0, 0, -1),
        _ => return None,
    })
}

/// The nearest free cell to `(x, y, z)` on its floor, east first, then clockwise, then
/// the next ring out.
fn nearest_free(taken: &HashSet<(i64, i64, i64)>, x: i64, y: i64, z: i64) -> (i64, i64, i64) {
    if !taken.contains(&(x, y, z)) {
        return (x, y, z);
    }
    for r in 1..64 {
        let ring = [
            (r, 0),
            (r, r),
            (0, r),
            (-r, r),
            (-r, 0),
            (-r, -r),
            (0, -r),
            (r, -r),
        ];
        for (dx, dy) in ring {
            if !taken.contains(&(x + dx, y + dy, z)) {
                return (x + dx, y + dy, z);
            }
        }
        for dx in -r..=r {
            for dy in -r..=r {
                if (dx.abs() == r || dy.abs() == r) && !taken.contains(&(x + dx, y + dy, z)) {
                    return (x + dx, y + dy, z);
                }
            }
        }
    }
    (x, y, z)
}

/// Lay the map out from the arrival room. Rooms the walk cannot reach (a razed link,
/// a dug room whose only door is gone) are placed last, each on its own floor below
/// everything, so nothing is lost and nothing is confused with the map proper.
pub fn layout(world: &World) -> Layout {
    let mut out = Layout::default();
    let Some(start) = rooms::arrival(world) else {
        return out;
    };
    let mut taken: HashSet<(i64, i64, i64)> = HashSet::new();
    let mut at: HashMap<EntityId, (i64, i64, i64)> = HashMap::new();
    let mut queue: VecDeque<EntityId> = VecDeque::new();
    let room_name = |id: EntityId| {
        plain_str(world, id, ROOM, "name")
            .unwrap_or("?")
            .to_string()
    };
    let push = |out: &mut Layout,
                id: EntityId,
                pos: (i64, i64, i64),
                placed: &'static str,
                from: Option<(EntityId, String)>| {
        out.rooms.push(Placed {
            id,
            key: plain_str(world, id, ROOM, "key").unwrap_or("").to_string(),
            name: room_name(id),
            x: pos.0,
            y: pos.1,
            z: pos.2,
            placed,
            from,
            owner: world.store.entity(id).and_then(|e| e.owner.clone()),
            exits: rooms::exits_of(world, id),
        });
    };
    taken.insert((0, 0, 0));
    at.insert(start, (0, 0, 0));
    push(&mut out, start, (0, 0, 0), "compass", None);
    // Two passes: the compass alone first, over the whole map, so a named door (the
    // Shallows' "dive") never takes a cell a compass exit will want (the Undertow is
    // east of the Cistern, two floors down; placed beside the Shallows it shoved the
    // Gallery off the Ossuary's east). Then the named doors, beside what they open
    // from, until nothing more is reachable.
    for pass in 0..2 {
        loop {
            queue.clear();
            queue.extend(at.keys().copied());
            let before = at.len();
            while let Some(here) = queue.pop_front() {
                let (hx, hy, hz) = at[&here];
                let exits = rooms::exits_of(world, here);
                let mut order: Vec<(&String, &EntityId)> = exits.iter().collect();
                order.sort_by_key(|(name, _)| (step(name).is_none(), (*name).clone()));
                for (name, &to) in order {
                    if at.contains_key(&to) || world.store.entity(to).is_none() {
                        continue;
                    }
                    let (want, kind) = match step(name) {
                        Some((dx, dy, dz)) => ((hx + dx, hy + dy, hz + dz), "compass"),
                        None if pass == 1 => ((hx + 1, hy, hz), "beside"),
                        None => continue,
                    };
                    let pos = nearest_free(&taken, want.0, want.1, want.2);
                    let placed = if kind == "compass" && pos != want {
                        out.shoved.push(format!(
                            "{} ({name} from {}) is shoved to ({},{},{}), its cell taken",
                            room_name(to),
                            room_name(here),
                            pos.0,
                            pos.1,
                            pos.2
                        ));
                        "shoved"
                    } else {
                        kind
                    };
                    taken.insert(pos);
                    at.insert(to, pos);
                    push(&mut out, to, pos, placed, Some((here, name.clone())));
                    queue.push_back(to);
                }
            }
            if at.len() == before {
                break;
            }
        }
    }
    // The unreachable, each on a floor of its own below the map.
    let mut floor = -1_000;
    let mut orphans: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "room" && !at.contains_key(&e.id))
        .map(|e| e.id)
        .collect();
    orphans.sort_by_key(|e| e.0);
    for id in orphans {
        push(&mut out, id, (0, 0, floor), "orphan", None);
        at.insert(id, (0, 0, floor));
        floor -= 1;
    }
    // One-way exits: a door with no door back.
    for p in &out.rooms {
        for (name, to) in &p.exits {
            let back = out
                .rooms
                .iter()
                .find(|q| q.id == *to)
                .is_some_and(|q| q.exits.values().any(|&v| v == p.id));
            if !back {
                out.one_way
                    .push(format!("{name} from {} to {}", p.name, room_name(*to)));
            }
        }
    }
    out
}

impl Layout {
    /// The map as JSON: rooms with their cells and exits by room id, and the report.
    pub fn to_json(&self) -> Value {
        json!({
            "rooms": self.rooms.iter().map(|p| json!({
                "id": p.id.0, "key": p.key, "name": p.name,
                "x": p.x, "y": p.y, "z": p.z,
                "placed": p.placed,
                "from": p.from.as_ref().map(|(r, e)| json!({"room": r.0, "exit": e})),
                "dug_by": p.owner,
                "exits": p.exits.iter().map(|(n, to)| (n.clone(), json!(to.0))).collect::<Map<String, Value>>(),
            })).collect::<Vec<_>>(),
            "shoved": self.shoved,
            "one_way": self.one_way,
        })
    }
}

/// What a citizen remembers: room id to the moment they last stood in it.
fn seen(world: &World, citizen: EntityId) -> BTreeMap<u64, i64> {
    world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(ATLAS))
        .and_then(|c| c.fields.get("seen"))
        .and_then(|f| match f {
            Field::Plain(Value::Object(m)) => Some(m.clone()),
            _ => None,
        })
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((k.parse().ok()?, v.as_i64()?)))
                .collect()
        })
        .unwrap_or_default()
}

/// Mark a room seen by a citizen, now. Called wherever a room is described to them.
pub fn mark_seen(world: &mut World, actor: &str, room: EntityId) -> Result<(), CommandError> {
    let Some(f) = super::find_citizen(world, actor) else {
        return Ok(());
    };
    let now = world.now().0;
    let mut m = seen(world, f.citizen);
    // A mark within the minute is a mark; the log is not written for every look.
    if m.get(&room.0).is_some_and(|&t| now - t < 60_000) {
        return Ok(());
    }
    m.insert(room.0, now);
    let obj: Map<String, Value> = m.iter().map(|(k, v)| (k.to_string(), json!(v))).collect();
    let mut fields = BTreeMap::new();
    fields.insert("seen".into(), Field::Plain(Value::Object(obj)));
    world
        .record(
            Actor::Citizen(actor.to_string()),
            StoreEvent::ComponentSet {
                id: f.citizen,
                component: ATLAS.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(internal)
}

/// The map as one citizen remembers it: the room they stand in and its exits in full,
/// every other room at its memory, nothing below the threshold.
pub fn remembered_map(
    world: &World,
    actor: &str,
    half_life_ms: f64,
) -> Result<Value, CommandError> {
    let f = super::find_citizen(world, actor).ok_or_else(|| {
        CommandError::precondition(format!("{actor} has not arrived; log in first."))
    })?;
    let here = rooms::room_of(world, f.citizen);
    let visible: HashSet<EntityId> = here
        .map(|h| {
            let mut v: HashSet<EntityId> = rooms::exits_of(world, h).values().copied().collect();
            v.insert(h);
            v
        })
        .unwrap_or_default();
    let all = layout(world);
    let marks = seen(world, f.citizen);
    let now = world.now().0;
    let mut shown: Vec<Value> = Vec::new();
    let mut by_memory: Vec<(f64, String)> = Vec::new();
    for p in &all.rooms {
        let memory = marks
            .get(&p.id.0)
            .map(|&t| 0.5f64.powf((now - t).max(0) as f64 / half_life_ms))
            .unwrap_or(0.0);
        let is_visible = visible.contains(&p.id);
        if memory < FORGOTTEN && !is_visible {
            continue;
        }
        let mut v = json!({
            "id": p.id.0, "key": p.key, "name": p.name,
            "x": p.x, "y": p.y, "z": p.z,
            "placed": p.placed,
            "memory": if is_visible { 1.0 } else { memory },
            "seen_at": marks.get(&p.id.0),
            "here": Some(p.id) == here,
            "visible": is_visible,
        });
        // Exits only to rooms on this map: the rest you do not remember.
        v["exits"] = p
            .exits
            .iter()
            .filter(|(_, to)| {
                is_visible
                    || visible.contains(to)
                    || marks.get(&to.0).is_some_and(|&t| {
                        0.5f64.powf((now - t).max(0) as f64 / half_life_ms) >= FORGOTTEN
                    })
            })
            .map(|(n, to)| (n.clone(), json!(to.0)))
            .collect::<Map<String, Value>>()
            .into();
        if !is_visible {
            by_memory.push((memory, p.name.clone()));
        }
        shown.push(v);
    }
    by_memory.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let here_name = here
        .and_then(|h| plain_str(world, h, ROOM, "name"))
        .unwrap_or("nowhere");
    let clear: Vec<String> = by_memory
        .iter()
        .filter(|(m, _)| *m >= 0.5)
        .map(|(_, n)| n.clone())
        .collect();
    let dim: Vec<String> = by_memory
        .iter()
        .filter(|(m, _)| *m < 0.5)
        .map(|(_, n)| n.clone())
        .collect();
    let mut summary = format!(
        "You are in {here_name}; you can see {} way{} out.",
        visible.len().saturating_sub(1),
        if visible.len() == 2 { "" } else { "s" }
    );
    if clear.is_empty() && dim.is_empty() {
        summary.push_str(" The rest of the map is blank: you remember no other room.");
    } else {
        if !clear.is_empty() {
            summary.push_str(&format!(" You remember the way to {}.", clear.join(", ")));
        }
        if !dim.is_empty() {
            summary.push_str(&format!(" Half-remembered: {}.", dim.join(", ")));
        }
    }
    let hours = half_life_ms / 3_600_000.0;
    summary.push_str(&format!(
        " A room fades from your map by half every {hours:.1} hours since you stood in it."
    ));
    Ok(json!({
        "here": here.map(|h| h.0),
        "rooms": shown,
        "half_life_ms": half_life_ms as i64,
        "forgotten_below": FORGOTTEN,
        "summary": summary,
    }))
}

pub fn register(world: &mut World, curves: Curves) {
    let half_life_ms = curves.active_half_life.0 as f64 * ATLAS_HALF_LIVES;
    world.register(
        CommandSpec::new(
            "disremembered.map",
            "The map as you remember it: the room you stand in and its exits in full, every room you have stood in at how well you remember it (it halves every few hours), laid out on a grid (north is up, east is right, up and down are floors). What you have forgotten is not on it.",
        )
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(move |w: &mut World, ctx, _| {
            let _ = me(w, ctx)?;
            let v = remembered_map(w, &ctx.actor, half_life_ms)?;
            let summary = v["summary"].as_str().unwrap_or("").to_string();
            Ok(Outcome::new(summary, v))
        }),
    );
}
