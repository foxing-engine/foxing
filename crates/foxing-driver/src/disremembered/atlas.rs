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
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
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
/// The layout, cached until the rooms or their exits change: `map` is asked on every
/// room change by the play page, under the world's lock, and laying out every room
/// each time grew past a quarter second at twenty thousand rooms.
pub fn layout(world: &World) -> Layout {
    thread_local! {
        static CACHE: std::cell::RefCell<Option<(u64, Layout)>> = const { std::cell::RefCell::new(None) };
    }
    // A fingerprint of the topology: rooms, exits, and where they lead.
    let mut fp: u64 = 1469598103934665603;
    let mut mix = |x: u64| {
        fp ^= x;
        fp = fp.wrapping_mul(1099511628211);
    };
    for e in world.store.entities().filter(|e| e.kind == "room") {
        mix(e.id.0);
        for (via, to) in rooms::exits_of(world, e.id) {
            mix(to.0);
            for b in via.bytes() {
                mix(b as u64);
            }
        }
    }
    let hit = CACHE.with(|c| {
        c.borrow()
            .as_ref()
            .filter(|(k, _)| *k == fp)
            .map(|(_, l)| l.clone())
    });
    if let Some(l) = hit {
        return l;
    }
    let l = layout_uncached(world);
    CACHE.with(|c| *c.borrow_mut() = Some((fp, l.clone())));
    l
}

fn layout_uncached(world: &World) -> Layout {
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
/// Whether the citizen has stood in the room before (it is on their map at all).
pub fn has_seen(world: &World, citizen: EntityId, room: EntityId) -> bool {
    seen(world, citizen).contains_key(&room.0)
}

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
    // A lantern carried: the map stays twice as long; a night lantern, three times.
    let half_life_ms = match super::craft::carries(world, f.citizen, "lantern") {
        Some(l) if super::past::tier(world, l) >= 2 => half_life_ms * 3.0,
        Some(_) => half_life_ms * 2.0,
        None => half_life_ms,
    };
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
    // The ways out, by name: a visitor at the Fogbank read "you remember no other
    // room" while standing one door from the Threshold, because what you can see is
    // not listed as remembered.
    let ways: Vec<String> = here
        .map(|h| {
            let mut v: Vec<(String, EntityId)> = rooms::exits_of(world, h).into_iter().collect();
            v.sort();
            v.into_iter()
                .map(|(n, to)| {
                    format!(
                        "{n} ({})",
                        plain_str(world, to, ROOM, "name")
                            .unwrap_or("?")
                            .trim_start_matches("The ")
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let mut summary = format!(
        "You are in {here_name}; you can see {} way{} out{}.",
        visible.len().saturating_sub(1),
        if visible.len() == 2 { "" } else { "s" },
        if ways.is_empty() {
            String::new()
        } else {
            format!(": {}", ways.join(", "))
        }
    );
    if clear.is_empty() && dim.is_empty() {
        summary.push_str(" Beyond what you can see, the map is blank: you remember no other room.");
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
    // The grid, drawn: this floor, as text, when it fits. A visitor through the gate
    // was promised a grid and got a sentence; the play page draws it, and now so does
    // the reply. @ is here, * what you can see, · what you remember.
    let floor = here
        .and_then(|h| all.rooms.iter().find(|p| p.id == h))
        .map(|p| p.z);
    if let Some(z) = floor {
        let on_floor: Vec<&Value> = shown
            .iter()
            .filter(|r| r["z"].as_i64() == Some(z))
            .collect();
        if on_floor.len() > 1 {
            let xs: Vec<i64> = on_floor.iter().filter_map(|r| r["x"].as_i64()).collect();
            let ys: Vec<i64> = on_floor.iter().filter_map(|r| r["y"].as_i64()).collect();
            let (x0, x1) = (
                *xs.iter().min().unwrap_or(&0),
                *xs.iter().max().unwrap_or(&0),
            );
            let (y0, y1) = (
                *ys.iter().min().unwrap_or(&0),
                *ys.iter().max().unwrap_or(&0),
            );
            if x1 - x0 <= 15 && y1 - y0 <= 11 {
                let mut legend: Vec<String> = Vec::new();
                let mut grid: Vec<Vec<String>> =
                    // Empty cells are dots, not spaces: a reply through the gate
                    // collapses whitespace, and the grid lost its columns.
                    vec![vec![" . ".to_string(); (x1 - x0 + 1) as usize]; (y1 - y0 + 1) as usize];
                for (i, r) in on_floor.iter().enumerate() {
                    let (x, y) = (r["x"].as_i64().unwrap_or(0), r["y"].as_i64().unwrap_or(0));
                    let code = format!("{:02}", (i + 1) % 100);
                    let mark = if r["here"] == true {
                        "@"
                    } else if r["visible"] == true {
                        "*"
                    } else {
                        "·"
                    };
                    // North is the smaller y (GET /v0/map: Lantern Row y=-1 of the
                    // Threshold), so the first row is the smallest y. Two visitors
                    // read the Scout's Lamp above the Threshold.
                    grid[(y - y0) as usize][(x - x0) as usize] = format!("{code}{mark}");
                    legend.push(format!("{code} {}", r["name"].as_str().unwrap_or("?")));
                }
                let rows: Vec<String> = grid.iter().map(|row| row.join(" ")).collect();
                summary.push_str(&format!(
                    "\nThis floor (north up, east right; @ here, * in sight, · remembered):\n{}\n{}",
                    rows.join("\n"),
                    legend.join("; ")
                ));
            }
        }
    }
    Ok(json!({
        "here": here.map(|h| h.0),
        "rooms": shown,
        "half_life_ms": half_life_ms as i64,
        "forgotten_below": FORGOTTEN,
        "summary": summary,
    }))
}

/// The rooms a citizen could find their way through: what they remember (above
/// FORGOTTEN on the map's half-life) and what they can see from where they stand.
fn known_rooms(world: &World, citizen: EntityId, half_life_ms: f64) -> HashSet<EntityId> {
    let half_life_ms = match super::craft::carries(world, citizen, "lantern") {
        Some(l) if super::past::tier(world, l) >= 2 => half_life_ms * 3.0,
        Some(_) => half_life_ms * 2.0,
        None => half_life_ms,
    };
    let now = world.now().0;
    let mut known: HashSet<EntityId> = seen(world, citizen)
        .into_iter()
        .filter(|(_, t)| 0.5f64.powf((now - t).max(0) as f64 / half_life_ms) >= FORGOTTEN)
        .map(|(id, _)| EntityId(id))
        .collect();
    if let Some(h) = rooms::room_of(world, citizen) {
        known.insert(h);
        known.extend(rooms::exits_of(world, h).values().copied());
    }
    known
}

/// The way from `from` to `to` over remembered rooms, as exit names; empty when
/// there is none the citizen knows.
pub fn way(
    world: &World,
    citizen: EntityId,
    from: EntityId,
    to: EntityId,
    half_life_ms: f64,
) -> Option<Vec<String>> {
    let known = known_rooms(world, citizen, half_life_ms);
    way_over(world, from, to, Some(&known))
}

/// The signed way from `here` to every one of the world's own rooms, for a figure
/// that gives directions: `{room, way}`, nearest first.
pub fn routes_from(world: &World, here: EntityId) -> Vec<serde_json::Value> {
    let mut out: Vec<(usize, String, Vec<String>)> = world
        .store
        .entities()
        .filter(|e| e.kind == "room" && e.owner.is_none() && e.id != here)
        .filter_map(|e| {
            let name = plain_str(world, e.id, ROOM, "name")?.to_string();
            let way = way_seeded(world, here, e.id)?;
            Some((way.len(), name, way))
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    out.into_iter()
        .map(|(_, room, way)| json!({"room": room, "way": way}))
        .collect()
}

/// The rooms a citizen remembers, by name.
pub fn known_room_names(world: &World, citizen: EntityId, half_life_ms: f64) -> Vec<String> {
    known_rooms(world, citizen, half_life_ms)
        .into_iter()
        .filter_map(|r| plain_str(world, r, ROOM, "name").map(str::to_string))
        .collect()
}

/// The way over the world's own rooms (the seed rooms), remembered or not: what a
/// sign at the door would say. Dug rooms are not on it; they are found by walking.
pub fn way_seeded(world: &World, from: EntityId, to: EntityId) -> Option<Vec<String>> {
    let seeded: std::collections::HashSet<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "room" && e.owner.is_none())
        .map(|e| e.id)
        .collect();
    way_over(world, from, to, Some(&seeded))
}

fn way_over(
    world: &World,
    from: EntityId,
    to: EntityId,
    known: Option<&std::collections::HashSet<EntityId>>,
) -> Option<Vec<String>> {
    if from == to {
        return Some(vec![]);
    }
    let empty = std::collections::HashSet::new();
    let known = known.unwrap_or(&empty);
    let mut prev: std::collections::HashMap<EntityId, (EntityId, String)> =
        std::collections::HashMap::new();
    let mut queue = std::collections::VecDeque::from([from]);
    while let Some(r) = queue.pop_front() {
        for (via, next) in rooms::exits_of(world, r) {
            if next == from || prev.contains_key(&next) {
                continue;
            }
            // The last step may lead into a room not yet remembered (the one
            // asked for, in sight from a known room); the steps before it may not.
            if !known.contains(&next) && next != to {
                continue;
            }
            prev.insert(next, (r, via));
            if next == to {
                let mut path = Vec::new();
                let mut at = to;
                while at != from {
                    let (p, v) = &prev[&at];
                    path.push(v.clone());
                    at = *p;
                }
                path.reverse();
                return Some(path);
            }
            queue.push_back(next);
        }
    }
    None
}

/// A room by its name, the article aside, or its key; case does not matter.
pub fn room_named(world: &World, text: &str) -> Option<EntityId> {
    let want = text.trim().trim_matches('"').to_ascii_lowercase();
    let want = want.strip_prefix("the ").unwrap_or(&want).to_string();
    if want.is_empty() {
        return None;
    }
    world
        .store
        .entities()
        .filter(|e| e.kind == "room")
        .find(|e| {
            let name = plain_str(world, e.id, ROOM, "name")
                .map(|n| n.to_ascii_lowercase())
                .unwrap_or_default();
            let name = name.strip_prefix("the ").unwrap_or(&name).to_string();
            name == want || plain_str(world, e.id, ROOM, "key") == Some(want.as_str())
        })
        .map(|e| e.id)
}

fn said_way(path: &[String]) -> String {
    match path.len() {
        0 => "you are there".to_string(),
        1 => format!("one step: {}", path[0]),
        n => format!("{n} steps: {}", path.join(", ")),
    }
}

pub fn register(world: &mut World, curves: Curves) {
    let hl = curves.active_half_life.0 as f64 * ATLAS_HALF_LIVES;
    world.register(
        CommandSpec::new(
            "disremembered.where",
            "Where a named creature, a citizen or a room is, and the way there over the rooms you remember: a read.",
        )
        .param("who", ParamType::String, true, "a creature's given name, a citizen, or a room's name"),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let who = args.str("who").unwrap_or("").trim().trim_matches('"').to_string();
            let (what, room): (String, Option<EntityId>) = if super::docent::is_named(&who) {
                ("The Docent".to_string(), super::docent::room(w))
            } else if let Some((c, r)) = super::mind::find_named(w, &who) {
                (super::mind::display(w, c), r)
            } else if let Some((cz, spelled)) = super::canonical_name(w, &who).and_then(|n| super::find_citizen(w, &n).map(|f| (f, n))) {
                let present = super::presence::is_here(w, cz.citizen);
                if !present {
                    return Ok(Outcome::new(
                        format!("{spelled} is not in the world now; the rooms remember them a while, and `log` keeps their words."),
                        json!({"who": spelled, "present": false}),
                    ));
                }
                (spelled, rooms::room_of(w, cz.citizen))
            } else if let Some(r) = room_named(w, &who) {
                (plain_str(w, r, ROOM, "name").unwrap_or("?").to_string(), Some(r))
            } else if !rooms::homes_of(who.trim().trim_start_matches("the ").trim_end_matches('s')).is_empty() || !rooms::homes_of(who.trim().trim_start_matches("the ")).is_empty() {
                // A kind: where it lives, and the way to the nearest of its rooms.
                let kind = who.trim().trim_start_matches("the ").to_string();
                let kind = if rooms::homes_of(&kind).is_empty() { kind.trim_end_matches('s').to_string() } else { kind };
                let homes = rooms::homes_of(&kind);
                let mut best: Option<(EntityId, Vec<String>, bool)> = None;
                for h in &homes {
                    let Some(r) = room_named(w, h) else { continue };
                    let remembered = way(w, f.citizen, here, r, hl);
                    let (p, signed) = match remembered {
                        Some(p) => (Some(p), false),
                        None => (way_seeded(w, here, r), true),
                    };
                    if let Some(p) = p {
                        if !best.as_ref().is_some_and(|b| p.len() >= b.1.len()) {
                            best = Some((r, p, signed));
                        }
                    }
                }
                let said = match &best {
                    Some((r, p, signed)) if p.is_empty() => format!("{kind}s live in {}, and come back when felled: here.", plain_str(w, *r, ROOM, "name").unwrap_or("?")),
                    Some((r, p, signed)) => format!("{kind}s live in {}, and come back when felled; {} {}.", plain_str(w, *r, ROOM, "name").unwrap_or("?"), if *signed { "you have not walked it, but the signs say" } else { "the way from here, as you remember it, is" }, said_way(p)),
                    None => format!("{kind}s live in {}, and come back when felled.", homes.join(" and ")),
                };
                return Ok(Outcome::new(said, json!({"who": kind, "homes": homes, "room": best.as_ref().map(|b| b.0 .0), "way": best.as_ref().map(|b| b.1.clone()), "signed": best.as_ref().is_some_and(|b| b.2)})));
            } else {
                return Err(CommandError::precondition(format!(
                    "Nothing called {who} that you could go to: a creature's given name, a citizen here now, a room's name, or a kind of creature (where ragpicker)."
                )));
            };
            let Some(room) = room else {
                return Ok(Outcome::new(format!("{what} is nowhere in particular."), json!({"who": what})));
            };
            let name = plain_str(w, room, ROOM, "name").unwrap_or("?").to_string();
            let path = way(w, f.citizen, here, room, hl);
            // A room asked for by name is not "in" itself.
            let is_room = what == name;
            // A seed room you have not walked to: the world's own rooms are signed, so
            // the way over them is said (the way as you remember it, when you have one).
            let signed = if path.is_none() && w.store.entity(room).is_some_and(|e| e.owner.is_none()) { way_seeded(w, here, room) } else { None };
            let said = match (&path, &signed) {
                (Some(p), _) if p.is_empty() && is_room => format!("{name} is where you stand."),
                (Some(p), _) if p.is_empty() => format!("{what} is here, in {name}."),
                (Some(p), _) if is_room => format!("{name}: the way from here, as you remember it, is {}.", said_way(p)),
                (Some(p), _) => format!("{what} is in {name}; the way from here, as you remember it: {}.", said_way(p)),
                (None, Some(p)) if is_room => format!("{name}: you have not walked it, but the signs say {}.", said_way(p)),
                (None, Some(p)) => format!("{what} is in {name}; you have not walked it, but the signs say {}.", said_way(p)),
                (None, None) if is_room => format!("{name}: you remember no way there from here. Walk, and `map` fills in."),
                (None, None) => format!("{what} is in {name}, but you remember no way there from here. Walk, and `map` fills in."),
            };
            let way_said = path.clone().or(signed);
            Ok(Outcome::new(said, json!({"who": what, "room": room.0, "room_name": name, "way": way_said, "signed": path.is_none() && way_said.is_some()})))
        }),
    );

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
