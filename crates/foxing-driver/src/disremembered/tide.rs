//! C9: the Drowned Wing, and the tide that keeps it.
//!
//! The library's wing that went under when the water came lies below the Undertow:
//! thirteen rooms at three depths. One tide rises and falls over them, and each room
//! has a height: the water is over it or it is not. The tide is computed from the
//! clock when it is asked (nothing schedules it): a cycle is forty minutes give or
//! take eight, and how low its low water gets moves with the week and the season, so
//! the deepest rooms clear on some tides and keep their water on others.
//!
//! A flooded room can be walked into and stood in. After a short grace the water
//! hurts, a little and then more (a shallow curve), and nothing mends under it. The
//! Tide Table at the entrance says when the next low waters are, roughly; a page that
//! stayed dry says it to the minute. The burning book on the lowest shelf can be read
//! only under water. Now and then the tide carries off what was left on a floor, and
//! leaves something of its own.

use super::{plain_str, CITIZEN, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{CommandError, Outcome};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const WRACK: &str = "wing-wrack";
pub const SLUICE: &str = "wing-sluice";
pub const COURT: &str = "wing-court";
pub const VAULT: &str = "wing-vault";
pub const SHELF: &str = "wing-shelf";

/// A room of the wing and how high it stands: the water is over a room while the
/// tide's level is at or above this. The Wrack Line is the high-water mark, and dry.
pub const HEIGHTS: &[(&str, f64)] = &[
    (WRACK, 2.0),
    ("wing-pool", 0.82),
    ("wing-catalogue", 0.79),
    (SLUICE, 0.76),
    ("wing-kelp", 0.40),
    ("wing-binder", 0.38),
    ("wing-drip", 0.36),
    ("wing-scriptorium", 0.33),
    ("wing-long", 0.30),
    ("wing-sump", 0.14),
    ("wing-vault", 0.11),
    (COURT, 0.09),
    (SHELF, 0.06),
];

/// The wing's own kinds: a stock one standing from before its kind was softened is
/// brought to what the kind is now, at boot (`reading::settle`).
pub const KINDS: &[&str] = &["newtling", "mudskipper", "hermit", "axolotl", "anemone"];

/// The middle of a cycle's length, and how far either end of it may be moved.
pub const CYCLE_MS: i64 = 40 * 60 * 1000;
pub const JITTER_MS: i64 = 4 * 60 * 1000;
/// How much of the lowest water a week may keep back, a cycle may keep back, and the
/// Fog Month keeps back. Together they can cover the three deepest rooms.
pub const WEEK_LOW: f64 = 0.05;
pub const CYCLE_LOW: f64 = 0.05;
pub const SEASON_LOW: f64 = 0.02;
const WEEK_MS: i64 = 7 * 24 * 60 * 60 * 1000;

/// The water gives this long before it hurts, and then hurts every so often: a point,
/// and a point more for each minute under, four at the most.
pub const GRACE_S: i64 = 30;
pub const TICK_S: i64 = 10;
pub const MOST_A_TICK: f64 = 4.0;
/// A diver's bell jar carried: three times the grace.
pub const JAR: &str = "bell-jar";
pub const JAR_GRACE: i64 = 3;
pub const DRY_PAGE: &str = "dry-page";
pub const PEARL: &str = "brine-pearl";
/// Who made what the tide leaves.
pub const BY_THE_TIDE: &str = "the tide";

/// The book pays this, this vividly, once in this long to a name.
pub const BOOK_XP: f64 = 80.0;
pub const BOOK_VIVIDITY: f64 = 0.9;
pub const BOOK_EVERY_MS: i64 = 24 * 60 * 60 * 1000;
/// A name under this is told what the water does, at the Wrack Line.
pub const WARN_UNDER_LEVEL: f64 = 30.0;

pub const JOB_WATER: &str = "disremembered.water";
pub const JOB_TIDE: &str = "disremembered.tide";
/// The water had someone: `{citizen, actor, room, taken}`; `fell` when it felled them.
pub const WATERED: &str = "disremembered.watered";
/// The tide moved a thing, or left one: `{item, name, from, to}` / `{item, name, to, left: true}`.
pub const TIDED: &str = "disremembered.tided";
/// The burning book was read: `{citizen, actor, chunk}`.
pub const BOOK_READ: &str = "disremembered.book_read";

// ---- the tide, from the clock ----

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A number in `[0, 1)` from a number and a salt.
fn unit(n: i64, salt: u64) -> f64 {
    (mix(n as u64 ^ salt) >> 11) as f64 / (1u64 << 53) as f64
}

/// When cycle `k` begins: its place on the forty-minute grid, moved by up to four
/// minutes either way. A cycle is so between 32 and 48 minutes long.
pub fn begins(k: i64) -> i64 {
    k * CYCLE_MS + ((unit(k, 0x7469_6465) * 2.0 - 1.0) * JITTER_MS as f64) as i64
}

/// The cycle a moment is in.
pub fn cycle_at(t: i64) -> i64 {
    let k = t.div_euclid(CYCLE_MS);
    (k - 1..=k + 1)
        .find(|&c| begins(c) <= t && t < begins(c + 1))
        .unwrap_or(k)
}

/// How far the low water of cycle `k` stays above the floor of the world: the week's
/// share, the cycle's own, and the season's.
pub fn low_of(k: i64, season: f64) -> f64 {
    let week = begins(k).div_euclid(WEEK_MS);
    unit(week, 0x7765_656b) * WEEK_LOW + unit(k, 0x6c6f_7721) * CYCLE_LOW + season
}

/// The water's level at a moment, 1 at high water and `low_of` at low.
pub fn level_at(t: i64, season: f64) -> f64 {
    let k = cycle_at(t);
    let (b, e) = (begins(k), begins(k + 1));
    let phase = (t - b) as f64 / (e - b) as f64;
    let low = low_of(k, season);
    low + (1.0 - low) * (0.5 + 0.5 * (std::f64::consts::TAU * phase).cos())
}

/// When a room of this height is dry in cycle `k`: `None` when that tide never
/// leaves it.
pub fn dry_in(k: i64, height: f64, season: f64) -> Option<(i64, i64)> {
    let (b, e) = (begins(k), begins(k + 1));
    let low = low_of(k, season);
    let x = 2.0 * (height - low) / (1.0 - low) - 1.0;
    if x <= -1.0 {
        return None;
    }
    if x >= 1.0 {
        return Some((b, e));
    }
    let a = x.acos() / std::f64::consts::TAU;
    let len = (e - b) as f64;
    Some((b + (a * len) as i64, b + ((1.0 - a) * len) as i64))
}

/// Whether a room of this height is under water at a moment.
pub fn flooded_at(t: i64, height: f64, season: f64) -> bool {
    if height >= 1.0 {
        return false;
    }
    !dry_in(cycle_at(t), height, season).is_some_and(|(s, e)| s <= t && t < e)
}

/// When a room dry now goes under.
pub fn floods_at(t: i64, height: f64, season: f64) -> Option<i64> {
    dry_in(cycle_at(t), height, season)
        .filter(|(s, e)| *s <= t && t < *e)
        .map(|(_, e)| e)
}

/// The middle of a cycle: its low water.
pub fn low_water(k: i64) -> i64 {
    (begins(k) + begins(k + 1)) / 2
}

// ---- the world's side ----

fn key_of(world: &World, room: EntityId) -> Option<&str> {
    plain_str(world, room, ROOM, "key")
}

/// A room's height, if it is a room of the wing.
pub fn height_of(world: &World, room: EntityId) -> Option<f64> {
    let key = key_of(world, room)?;
    HEIGHTS.iter().find(|(k, _)| *k == key).map(|(_, h)| *h)
}

/// What the season keeps back of the low water.
pub fn season(world: &World) -> f64 {
    if super::almanac::on(world, super::almanac::FOG_MONTH) {
        SEASON_LOW
    } else {
        0.0
    }
}

/// Whether a room is under water now.
pub fn flooded(world: &World, room: EntityId) -> bool {
    height_of(world, room).is_some_and(|h| flooded_at(world.now().0, h, season(world)))
}

/// Whether a citizen stands in water now: nothing mends there.
pub fn under(world: &World, citizen: EntityId) -> bool {
    world
        .store
        .entity(citizen)
        .and_then(|e| e.parent)
        .is_some_and(|r| flooded(world, r))
}

fn rooms_of_the_wing(world: &World) -> Vec<(EntityId, f64)> {
    HEIGHTS
        .iter()
        .filter_map(|(k, h)| super::rooms::room_by_key(world, k).map(|r| (r, *h)))
        .collect()
}

fn here_in(world: &World, room: EntityId) -> Vec<String> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .collect()
}

fn minutes(ms: i64) -> String {
    let m = (ms as f64 / 60_000.0).round() as i64;
    match m {
        i64::MIN..=0 => "under a minute".to_string(),
        1 => "about a minute".to_string(),
        m => format!("about {m} minutes"),
    }
}

/// A span as the Tide Table gives it: to the minute.
fn roughly(ms: i64) -> String {
    // D46 (David, 11-17: "take it down to a minute"): it gave five-minute steps past
    // ten, and a captain moved a strike four minutes on a rounding.
    minutes(ms)
}

/// How long the water waits before it hurts this citizen: three times as long with a
/// bell jar carried.
fn grace_s(world: &World, citizen: EntityId) -> i64 {
    if super::craft::carries(world, citizen, JAR).is_some() {
        GRACE_S * JAR_GRACE
    } else {
        GRACE_S
    }
}

fn schedule(
    world: &mut World,
    due_ms: i64,
    citizen: EntityId,
    actor: &str,
    room: EntityId,
    stage: &str,
    since: i64,
) -> Result<(), String> {
    world.schedule_once(
        JOB_WATER,
        foxing_time::Timestamp(due_ms),
        json!({"citizen": citizen.0, "actor": actor, "room": room.0, "stage": stage, "since": since}),
        |p| p["citizen"] == citizen.0,
    )?;
    Ok(())
}

/// The water closes over someone: said, the bell jar spent if one is carried, and the
/// first hurt set for when the grace runs out.
fn closes_over(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    room: EntityId,
    walked_in: bool,
) -> Result<(), String> {
    let now = world.now().0;
    let grace = grace_s(world, citizen);
    let rname = plain_str(world, room, ROOM, "name")
        .unwrap_or("This room")
        .to_string();
    let jar = match super::craft::carries(world, citizen, JAR) {
        Some(j) => super::craft::spend(world, actor, j, "kept the water off you")
            .map(|n| format!(" Your {n} holds a breath of air around you: three times as long."))
            .unwrap_or_default(),
        None => String::new(),
    };
    super::combat::narrate(
        world,
        actor,
        // By the room's name: a walk through one to the Wrack Line was told "this
        // room is under water" in the reply that set it down in the dry.
        format!(
            "{} You have {grace} seconds there before it hurts; then a point every {TICK_S} seconds, and more the longer you stay. Nothing mends under water.{jar}",
            if walked_in {
                format!("{rname} is under water, and you are in it to the chest.")
            } else {
                format!("The water is over the floor of {rname}, and rising past your knees.")
            }
        ),
    );
    super::combat::recurve_mending(world, citizen).map_err(|e| e.to_string())?;
    schedule(world, now + grace * 1000, citizen, actor, room, "tick", now)
}

/// Someone has come into a room, by any door: a water clock they were under stops,
/// and a room of the wing starts one.
pub fn on_arrival(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    room: EntityId,
) -> Result<(), String> {
    // A message through the gate is an arrival too, as far as a room is concerned
    // (a name logging in where something hunts is arriving): a clock already
    // running on this citizen in this room is left to run. The first night's water
    // started over with every message: the grace again, a point every seventeen
    // seconds and never two, and the entry line said after the citizen had left.
    if world.jobs.has(JOB_WATER, |p| {
        p["citizen"] == citizen.0 && p["room"] == room.0
    }) {
        return Ok(());
    }
    // Already under water in the room just left: the hurt that was coming is still
    // coming, when it was due, and the count of minutes under goes on. (A door
    // between two flooded rooms was a fresh half minute of grace each way.)
    let carried: Option<(i64, i64)> = world
        .jobs
        .pending()
        .find(|j| {
            j.kind == JOB_WATER && j.payload["citizen"] == citizen.0 && j.payload["stage"] == "tick"
        })
        .and_then(|j| Some((j.due.0, j.payload["since"].as_i64()?)));
    world.cancel(JOB_WATER, |p| p["citizen"] == citizen.0)?;
    let Some(height) = height_of(world, room) else {
        return Ok(());
    };
    if height >= 1.0 {
        return Ok(());
    }
    let now = world.now().0;
    let s = season(world);
    match floods_at(now, height, s) {
        None => match carried {
            Some((due, since)) => {
                let rname = plain_str(world, room, ROOM, "name")
                    .unwrap_or("This room")
                    .to_string();
                super::combat::narrate(
                    world,
                    actor,
                    format!("{rname} is under water too: there is no fresh breath in it, and the water goes on as it was."),
                );
                super::combat::recurve_mending(world, citizen).map_err(|e| e.to_string())?;
                schedule(
                    world,
                    due.max(now + 1000),
                    citizen,
                    actor,
                    room,
                    "tick",
                    since,
                )
            }
            None => closes_over(world, actor, citizen, room, true),
        },
        Some(at) if at - now <= 60_000 => {
            super::combat::narrate(
                world,
                actor,
                format!(
                    "The water is at the sill here: this room goes under in {}.",
                    minutes(at - now)
                ),
            );
            schedule(world, at, citizen, actor, room, "flood", 0)
        }
        Some(at) => schedule(world, at - 60_000, citizen, actor, room, "warn", 0),
    }
}

/// A water clock comes due on someone.
fn water(world: &mut World, payload: &Value) -> Result<(), String> {
    let citizen = EntityId(payload["citizen"].as_u64().unwrap_or(0));
    let actor = payload["actor"].as_str().unwrap_or("").to_string();
    let room = EntityId(payload["room"].as_u64().unwrap_or(0));
    let stage = payload["stage"].as_str().unwrap_or("");
    let since = payload["since"].as_i64().unwrap_or(0);
    // Gone from the room: wherever they went started its own clock, or none.
    if world.store.entity(citizen).and_then(|e| e.parent) != Some(room) {
        return Ok(());
    }
    let Some(height) = height_of(world, room) else {
        return Ok(());
    };
    let now = world.now().0;
    let s = season(world);
    let wet = flooded_at(now, height, s);
    // A name gone quiet is not drowned: the tide carries it up to the Wrack Line.
    if !super::presence::is_here(world, citizen) {
        if wet {
            wash_up(world, &actor, citizen)?;
        }
        return Ok(());
    }
    match stage {
        "warn" => match floods_at(now, height, s) {
            Some(at) => {
                super::combat::narrate(
                    world,
                    &actor,
                    format!(
                        "The water is coming back: this room goes under in {}. The Wrack Line is always dry.",
                        minutes(at - now)
                    ),
                );
                schedule(world, at, citizen, &actor, room, "flood", 0)
            }
            None => closes_over(world, &actor, citizen, room, false),
        },
        "flood" if wet => closes_over(world, &actor, citizen, room, false),
        "tick" if wet => {
            let grace = grace_s(world, citizen);
            let under_ms = (now - since - grace * 1000).max(0);
            let taken = (1.0 + (under_ms / 60_000) as f64).min(MOST_A_TICK);
            let before = super::combat::hp(world, citizen);
            let after = before - taken;
            super::combat::commit_hp(world, citizen, after).map_err(|e| e.to_string())?;
            let fell = after <= 0.0;
            let _ = world.note(
                Actor::Driver,
                WATERED,
                json!({"citizen": citizen.0, "actor": actor, "room": room.0, "taken": taken, "fell": fell}),
            );
            if fell {
                let Some(f) = super::find_citizen(world, &actor) else {
                    return Ok(());
                };
                let d = super::combat::die(world, &actor, &f, None).map_err(|e| e.to_string())?;
                super::combat::narrate(
                    world,
                    &actor,
                    format!(
                        "The water has all of you now, and it is very quiet. You go down.{}",
                        d.said()
                    ),
                );
                return Ok(());
            }
            super::combat::narrate(
                world,
                &actor,
                format!(
                    "Under water: it takes {taken:.0}, {:.0} \u{2192} {:.0} of {:.0}.",
                    before.ceil(),
                    after.ceil(),
                    super::combat::max_hp(world, citizen)
                ),
            );
            schedule(
                world,
                now + TICK_S * 1000,
                citizen,
                &actor,
                room,
                "tick",
                since,
            )
        }
        // The water has drawn off, or never came: the clock starts over for the
        // next tide.
        _ => {
            if stage == "tick" {
                super::combat::narrate(
                    world,
                    &actor,
                    "The water draws off this room, and leaves you standing in what it left.",
                );
            }
            super::combat::recurve_mending(world, citizen).map_err(|e| e.to_string())?;
            match floods_at(now, height, s) {
                Some(at) => schedule(
                    world,
                    (at - 60_000).max(now + 1000),
                    citizen,
                    &actor,
                    room,
                    "warn",
                    0,
                ),
                None => Ok(()),
            }
        }
    }
}

/// Carried up to the Wrack Line by the tide: a name that went quiet under water.
fn wash_up(world: &mut World, actor: &str, citizen: EntityId) -> Result<(), String> {
    let Some(to) = super::rooms::room_by_key(world, WRACK) else {
        return Ok(());
    };
    let _ = super::combat::leave_fight(world, actor, citizen);
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: citizen,
                parent: Some(to),
            },
        )
        .map_err(|e| e.to_string())?;
    let _ = super::atlas::mark_seen(world, actor, to);
    super::combat::recurve_mending(world, citizen).map_err(|e| e.to_string())?;
    super::combat::narrate(
        world,
        actor,
        "While you were away the tide came in over you, and carried you up to the Wrack Line with the rest of what floats.",
    );
    Ok(())
}

/// One line for `look` in a room of the wing: where the water is here, which of the
/// doors lead into it now (a middle room is under while the room above it is dry,
/// and the first visitor walked into one blind), and at the Wrack Line, to a name
/// with little to lose and all of it to lose, the mild word.
pub fn look_line(world: &World, actor: &str, room: EntityId) -> Option<String> {
    let height = height_of(world, room)?;
    let now = world.now().0;
    let s = season(world);
    let mut wet: Vec<String> = super::rooms::exits_of(world, room)
        .into_iter()
        .filter(|(_, to)| flooded(world, *to))
        .map(|(via, to)| {
            format!(
                "{via} ({})",
                plain_str(world, to, ROOM, "name")
                    .unwrap_or("?")
                    .trim_start_matches("The ")
            )
        })
        .collect();
    wet.sort();
    let doors = if wet.is_empty() {
        String::new()
    } else {
        format!(" Under water now: {}.", wet.join(", "))
    };
    if height >= 1.0 {
        let new = super::find_citizen(world, actor)
            .is_some_and(|f| super::memory::level(world, f.window) < WARN_UNDER_LEVEL);
        return Some(format!(
            "The high-water mark runs along the wall at your shoulder: this room is always dry. The Tide Table hangs by the stair (`read table`).{doors}{}",
            if new {
                " A word, since you are new to it: the rooms below go under with the tide, and the water hurts whoever stays. Read the table before you go down, and come back up when a room says the water is at the sill."
            } else {
                ""
            }
        ));
    }
    // Whether the tide is still going out: a deep room is never ten minutes from its
    // water, and said "on its way back" from the moment it cleared, while the table
    // said the water was falling.
    let falling = now < low_water(cycle_at(now));
    let here = match floods_at(now, height, s) {
        None => "The water is over you: this room is under water. After half a minute of it you are hurt, a little and then more, and nothing mends until you are out; the Wrack Line is always dry.",
        // The warning a minute out is said once, to whoever is listening; `look`
        // says it too, to whoever asks.
        Some(at) if at - now < 60_000 => "The water is over the sill: this room goes under inside a minute.",
        Some(at) if at - now < 3 * 60_000 => "The water is at the sill, and coming in.",
        // (Of a deep room only: a middle room said "well out" and then this.)
        Some(at) if at - now < 10 * 60_000 && falling && height < 0.2 => "The floor is wet and the water is still going out; a room this low is not clear of it long.",
        Some(at) if at - now < 10 * 60_000 => "The floor is wet and the water is on its way back: you can hear it in the next room.",
        Some(_) => "The floor is wet, and the water is well out.",
    };
    // The Court's text has a starfish lying across the door; when it has come apart,
    // the room says so until another drifts in.
    let bare = if key_of(world, room) == Some(COURT)
        && !world
            .store
            .children(room)
            .any(|e| e.kind == "creature" && super::starfish::is(world, e.id))
    {
        " The starfish is not here: it came apart, and the five signs lie bare where its arms were. The door behind is clear, south, until another drifts in."
    } else {
        ""
    };
    Some(format!("{here}{doors}{bare}"))
}

/// What the Tide Table says: the next two low waters, roughly, and how good each is.
fn table(world: &World, exact: bool) -> String {
    let now = world.now().0;
    let s = season(world);
    let k = cycle_at(now);
    let mut said: Vec<String> = Vec::new();
    for c in k..k + 4 {
        let at = low_water(c);
        if at <= now {
            continue;
        }
        let low = low_of(c, s);
        // The deep rooms, by which of them this low water uncovers; and of the Court,
        // for how long (11-17: a party of three planned a strike to the table's minute
        // and found the Court clear for three of them, the low in the middle).
        let court = HEIGHTS
            .iter()
            .find(|(k, _)| *k == COURT)
            .and_then(|(_, h)| dry_in(c, *h, s))
            .map(|(from, to)| minutes(to - from))
            .unwrap_or_else(|| "a moment".to_string());
        let depth = if low < 0.06 {
            format!("a great low: every room clears, the Starfish Court for {court} with the low in the middle of them, the lowest shelf for a moment")
        } else if low < 0.09 {
            format!("a good low: the Starfish Court clears for {court} with the low in the middle of them, and the shelf below it keeps its water")
        } else if low < 0.11 {
            "a poor low: the Barnacle Vault clears and nothing below it".to_string()
        } else {
            "a neap: the Sump clears, and the three rooms below it keep their water".to_string()
        };
        let when = if exact {
            let m = (at - now) / 60_000;
            let sec = ((at - now) % 60_000) / 1000;
            format!("in {m}m {sec:02}s")
        } else {
            format!("in {}", roughly(at - now))
        };
        said.push(format!("low water {when} ({depth})"));
        if said.len() == if exact { 3 } else { 2 } {
            break;
        }
    }
    let level = level_at(now, s);
    let state = if level > 0.8 {
        "The water is high now"
    } else if level > 0.4 {
        "The water is at half now"
    } else {
        "The water is low now"
    };
    format!(
        "{state}{}. Then: {}.",
        if begins(k + 1) - now > (begins(k + 1) - begins(k)) / 2 {
            ", and falling"
        } else {
            ", and rising"
        },
        said.join("; ")
    )
}

/// The wing's own things to read: the Tide Table at the Wrack Line, the grate in the
/// Sluice, the burning book on its shelf, and a page that stayed dry, wherever it is
/// carried.
pub fn read(
    world: &mut World,
    actor: &str,
    room: EntityId,
    arg: &str,
) -> Result<Option<Outcome>, CommandError> {
    let a = arg.trim().trim_matches('"').to_lowercase();
    let named = |words: &[&str]| words.iter().any(|w| a.contains(w));
    let Some(f) = super::find_citizen(world, actor) else {
        return Ok(None);
    };
    if named(&["page"]) && !named(&["book"]) {
        if let Some(page) = super::craft::carries(world, f.citizen, DRY_PAGE) {
            let text = format!(
                "The page that stayed dry is a leaf of the Tide Table's own book, and the ink on it moves as you read: {} It is right to the second, for as long as you hold it.",
                table(world, true)
            );
            return Ok(Some(Outcome::new(
                text,
                json!({"item": page.0, "read": "dry page"}),
            )));
        }
    }
    let key = key_of(world, room).map(str::to_string);
    let text = match key.as_deref() {
        Some(WRACK) if a.is_empty() || named(&["table", "tide", "board", "chalk", "slate"]) => format!(
            "The Tide Table is a slate in a swollen oak frame, chalked by nobody you will meet. {} Under it, smaller: THE FIGURES ARE THE WATER'S, AND THE WATER DOES NOT KEEP TO THEM. A tide is forty minutes, give or take eight; the upper rooms are dry most of one, the middle rooms under half of one, the deep rooms a few minutes at the bottom of it.",
            table(world, false)
        ),
        Some(SLUICE) if a.is_empty() || named(&["grate", "sewer", "door", "gate", "bars"]) => "The grate is a door of iron bars as thick as your wrist, green and furred, hinged on the far side and barred on the far side. Beyond it a brick culvert goes away downhill in the dark, and something in it is moving slowly the other way. A plate on the lintel, half eaten: MUNICIPAL S—WER. NO ADM—. Nothing on this side opens it. Not yet.".to_string(),
        Some(SLUICE) if named(&["gauge", "staff", "brass", "marks", "names"]) => gauge(world),
        Some(COURT) if named(&["mosaic", "floor", "zodiac", "signs", "well"]) => "The mosaic is a zodiac, and the water has been at it: the tesserae lift with every tide and settle a little wrong, so the Scales now weigh a Fish against the Reader, and the Lamp has drifted into the house of the Crab. Twelve signs, and where a thirteenth has been set in later, in newer stone, a small five-pointed star at the foot of the door going down. The starfish lies with one arm on it. Someone drew this before there was anything to draw it from.".to_string(),
        Some(VAULT) if a.is_empty() || named(&["ledger", "deposit", "book", "box", "plate", "deed", "counter"]) => "The deposit ledger is open at its last page, and the ink has spread into the paper and stayed legible, the way a bruise does. Box 7. The Night Reader. One starfish, small, in a jar of its own water, for the light well. In another hand, later: It has not stayed small. In a third, pressed hard: Its strength is in its arms. What it takes from whoever it puts under it goes into the arm it leads with, and that arm thickens; take that arm off and it lets all of it go. An arm is twenty minutes growing back. Five arms. Bring friends, and do not stop at four; it is divided evenly among those who cut it. Box 12. Municipal Sewer Board. One key, iron, for the grate at the Sluice. Withdrawn. Not returned. Box 31. The Keeper of the Lowest Shelf. One book, bound in something that does not char. TO BE KEPT WET. Do not let it dry while a reader has it open. Box 40. Depositor's name burned out of the page, neatly, from the inside. One deed, to a lot in the Ashen Wastes. Below that the page is blank, and beaded with water, and patient.".to_string(),
        Some(SHELF) if a.is_empty() || named(&["book", "burning", "shelf", "fire", "flame"]) => {
            return burning_book(world, actor, &f, room).map(Some);
        }
        _ => return Ok(None),
    };
    Ok(Some(Outcome::new(text, json!({"room": key, "read": a}))))
}

/// The Sluice's gauge: every room of the wing cut into a brass staff at its height,
/// and the water against it. Which rooms are under and which are clear, now; never
/// when (the Tide Table keeps that). It gives the Sluice a reason to be walked to,
/// and a new name a way to see the whole wing's water from one dry-shod place.
fn gauge(world: &World) -> String {
    let now = world.now().0;
    let s = season(world);
    let name = |key: &str| {
        super::rooms::room_by_key(world, key)
            .and_then(|r| plain_str(world, r, ROOM, "name"))
            .unwrap_or("?")
            .trim_start_matches("The ")
            .to_string()
    };
    let (mut under, mut clear) = (Vec::new(), Vec::new());
    for (key, h) in HEIGHTS.iter().filter(|(_, h)| *h < 1.0) {
        if flooded_at(now, *h, s) {
            under.push(name(key));
        } else {
            clear.push(name(key));
        }
    }
    let list = |v: &[String]| match v.len() {
        0 => "none of them".to_string(),
        _ => v.join(", "),
    };
    let k = cycle_at(now);
    format!(
        "The gauge is a brass staff bolted to the brick, green where the water lives and bright where hands have been. Every room of the wing is cut into it at its own height, the Reading Pool near the top and the Burning Shelf a thumb above the floor, and the water stands against it, {}. Under it now: {}. Clear of it: {}. The staff says where the water is and never when: the slate at the Wrack Line keeps the hours.",
        if now < low_water(k) { "going down" } else { "coming up" },
        list(&under),
        list(&clear)
    )
}

/// The burning book: cold and shut in the air, alight and open under water. Read
/// there, it pays once a day, and says where the fire came from.
fn burning_book(
    world: &mut World,
    actor: &str,
    f: &super::Found,
    room: EntityId,
) -> Result<Outcome, CommandError> {
    if !flooded(world, room) {
        return Ok(Outcome::new(
            "The book lies on the lowest shelf, grey as a hearth the morning after, and its pages will not part: they are one block, cold, and a little greasy with soot. There is a scorch on the stone above it in the shape of a flame that is not there. It is waiting for something, and the floor here is only damp.",
            json!({"read": "burning book", "alight": false}),
        ));
    }
    let now = world.now().0;
    let last = super::plain_num(world, f.citizen, CITIZEN, "burning_at").unwrap_or(0.0) as i64;
    let page = "Under the water the book is alight. It burns the way a coal does, from the inside, and the water around it does not boil and does not put it out; the pages turn themselves, slowly, as a thing breathes. The one it stops at is a map of no place under any water: a country drawn in ash, east of every east on the map you carry, with one road into it and the road's name burned through. THE ASHEN WASTES, the legend says, WHERE WHAT WAS FORGOTTEN ON PURPOSE WAS TAKEN TO BE BURNED. Under that, in a hand that pressed hard: it did not all burn.";
    if last > 0 && now - last < BOOK_EVERY_MS {
        return Ok(Outcome::new(
            format!(
                "{page} You have read this page today, and it gives you nothing it has not given: the book turns to it again in {}.",
                super::combat::human(BOOK_EVERY_MS - (now - last))
            ),
            json!({"read": "burning book", "alight": true, "paid": false}),
        ));
    }
    let chunk = super::memory::earn_chunk(
        world,
        actor,
        f.window,
        "the burning book, read under water",
        BOOK_XP,
        BOOK_VIVIDITY,
        &super::Curves::from_env(),
    )?;
    let mut changes = BTreeMap::new();
    changes.insert("burning_at".into(), Field::Plain(json!(now)));
    super::set_citizen_fields(world, actor, f.citizen, changes)?;
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        BOOK_READ,
        json!({"citizen": f.citizen.0, "actor": actor, "chunk": chunk.0}),
    );
    Ok(Outcome::new(
        format!(
            "{page} You will not forget this soon: chunk-{} worth {BOOK_XP:.0} XP, as vivid as a memory comes. The book gives it once a day to a name. The water is still over you.",
            chunk.0
        ),
        json!({"read": "burning book", "alight": true, "paid": true, "chunk": chunk.0, "xp": BOOK_XP}),
    ))
}

/// A door of the wing that will not be gone through just now: the way down to the
/// Burning Shelf, while the starfish lies across it.
pub fn bars(world: &World, from: EntityId, to: EntityId) -> Option<String> {
    if key_of(world, from) != Some(COURT) || key_of(world, to) != Some(SHELF) {
        return None;
    }
    let star = super::combat::creatures_in(world, from)
        .into_iter()
        .any(|(id, _, _, _)| plain_str(world, id, CREATURE, "name") == Some("starfish"));
    star.then(|| "The starfish lies across the way down to the shelf with every arm it has left. It does not move for being asked. It will have to be felled, all five arms of it, and an arm grows back in twenty minutes.".to_string())
}

/// A way out of a room of the wing that is no exit: the grate in the Sluice.
pub fn no_exit(world: &World, room: EntityId, exit: &str) -> Option<String> {
    (key_of(world, room) == Some(SLUICE)
        && ["grate", "sewer", "door", "gate", "culvert"].contains(&exit))
    .then(|| "The grate does not open from this side: it is hinged and barred on the other, and has been for longer than the water has been here. `read grate` says what can be seen through it. Nothing opens it yet.".to_string())
}

// ---- the tide's own round: what it carries off, and what it leaves ----

/// What the tide leaves, and how often it is each.
const LEFT: &[(&str, &str, u32)] = &[
    ("a diver's bell jar", JAR, 3),
    ("a page that stayed dry", DRY_PAGE, 0),
    ("a brine pearl", PEARL, 0),
];
/// At most this many of the tide's own things lie in the wing at once.
const MOST_LEFT: usize = 2;

fn plain_things(world: &World, room: EntityId) -> Vec<(EntityId, String)> {
    super::craft::things_in(world, room)
        .iter()
        .filter(|t| {
            t["given_name"].is_null()
                && t["made_by"].is_null()
                && t["tier"].as_u64().unwrap_or(0) <= 1
        })
        .filter_map(|t| {
            Some((
                EntityId(t["id"].as_u64()?),
                t["name"].as_str().unwrap_or("something").to_string(),
            ))
        })
        .collect()
}

/// The tide's round, every few minutes: names gone quiet under water are carried up,
/// a plain thing on a flooded floor is sometimes carried to another room, and at low
/// water something of the tide's own is sometimes left in a room it has uncovered.
fn round(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    let s = season(world);
    let wing = rooms_of_the_wing(world);
    if wing.is_empty() {
        return rearm(world);
    }
    // The quiet, carried up.
    for (room, h) in &wing {
        if !flooded_at(now, *h, s) {
            continue;
        }
        let asleep: Vec<(EntityId, String)> = world
            .store
            .children(*room)
            .filter(|e| e.kind == "citizen" && !super::presence::is_here(world, e.id))
            .filter_map(|e| e.owner.clone().map(|o| (e.id, o)))
            .collect();
        for (citizen, actor) in asleep {
            wash_up(world, &actor, citizen)?;
        }
    }
    // What lay on a flooded floor: one thing in three goes with the water, to
    // another room of the wing, or up into the Undertow or the Shallows.
    let mut elsewhere: Vec<EntityId> = wing.iter().map(|(r, _)| *r).collect();
    for key in ["undertow", "shallows"] {
        if let Some(r) = super::rooms::room_by_key(world, key) {
            elsewhere.push(r);
        }
    }
    for (room, h) in &wing {
        if !flooded_at(now, *h, s) {
            continue;
        }
        for (item, name) in plain_things(world, *room) {
            if world.roll(3) != 0 {
                continue;
            }
            // Half of what the water takes it leaves against the Sluice's grate, the
            // wing's drain: a room to look in for what was lost downstairs.
            let sluice = super::rooms::room_by_key(world, SLUICE);
            let to = match sluice {
                Some(sl) if world.roll(2) == 0 => sl,
                _ => elsewhere[world.roll(elsewhere.len() as u32) as usize],
            };
            if to == *room {
                continue;
            }
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityMoved {
                        id: item,
                        parent: Some(to),
                    },
                )
                .map_err(|e| e.to_string())?;
            let _ = world.note(
                Actor::Driver,
                TIDED,
                json!({"item": item.0, "name": name, "from": room.0, "to": to.0}),
            );
            let gone = here_in(world, *room);
            super::combat::narrate_many(
                world,
                &gone,
                format!("The water lifts {name} off the floor and takes it out of the room."),
            );
            let come = here_in(world, to);
            super::combat::narrate_many(
                world,
                &come,
                format!("Something turns over in the water and fetches up here: {name}."),
            );
        }
    }
    // What the tide leaves: at low water, in a room below the upper ones that it has
    // uncovered, one time in six, and never more than two lying about.
    let lying = wing
        .iter()
        .flat_map(|(r, _)| super::craft::things_in(world, *r))
        .filter(|t| t["made_by"] == BY_THE_TIDE)
        .count();
    let uncovered: Vec<EntityId> = wing
        .iter()
        .filter(|(_, h)| *h < 0.5 && !flooded_at(now, *h, s))
        .map(|(r, _)| *r)
        .collect();
    if lying < MOST_LEFT && !uncovered.is_empty() && world.roll(6) == 0 {
        // Half the time in the Barnacle Vault, when it is clear: the tide leaves
        // heavy things where the valuable things were.
        let vault = super::rooms::room_by_key(world, VAULT).filter(|v| uncovered.contains(v));
        let room = match vault {
            Some(v) if world.roll(2) == 0 => v,
            _ => uncovered[world.roll(uncovered.len() as u32) as usize],
        };
        let (name, origin, uses) = LEFT[world.roll(LEFT.len() as u32) as usize];
        let item = super::craft::make(
            world,
            &super::Curves::from_env(),
            Actor::Driver,
            None,
            room,
            name,
            "made",
            origin,
            Some(BY_THE_TIDE),
            &[],
            uses,
            false,
        )
        .map_err(|e| e.to_string())?;
        let _ = world.note(
            Actor::Driver,
            TIDED,
            json!({"item": item.0, "name": name, "to": room.0, "left": true}),
        );
        let here = here_in(world, room);
        super::combat::narrate_many(
            world,
            &here,
            format!("The water, going out, has left something on the floor that it did not make: {name}."),
        );
    }
    // The Sluice is the wing's drain, and says so to whoever stands in it: the room
    // "empties itself twice an hour, loudly", and a visitor heard nothing.
    if let Some(sl) = super::rooms::room_by_key(world, SLUICE) {
        let here = here_in(world, sl);
        if !here.is_empty() {
            let k = cycle_at(now);
            let line = if now < low_water(k) {
                "The channel is running out: the whole wing going past your feet toward the grate, grey and fast, and the bars combing it."
            } else {
                "The channel has turned: water is coming in through the grate from the culvert, slow and cold, and it smells of somewhere else."
            };
            super::combat::narrate_many(world, &here, line.to_string());
        }
    }
    rearm(world)
}

/// The next round: five to eight minutes on, by the clock and not the world's dice.
fn rearm(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    let wait = 5 * 60_000 + (unit(now / 1000, 0x726f_756e) * 180_000.0) as i64;
    world.schedule_once(
        JOB_TIDE,
        foxing_time::Timestamp(now + wait),
        json!({}),
        |_| true,
    )?;
    Ok(())
}

/// At boot: the tide's round is on the clock.
pub fn ensure(world: &mut World) -> Result<(), String> {
    rearm(world)
}

pub fn register(world: &mut World) {
    world.on_job(
        JOB_WATER,
        Box::new(|w: &mut World, job| water(w, &job.payload)),
    );
    world.on_job(JOB_TIDE, Box::new(|w: &mut World, _| round(w)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cycle_is_forty_minutes_give_or_take_eight_and_every_moment_is_in_one() {
        for k in -50..2000i64 {
            let len = begins(k + 1) - begins(k);
            assert!(
                (32 * 60_000..=48 * 60_000).contains(&len),
                "cycle {k} is {len} ms"
            );
        }
        let mut t = 1_794_800_000_000i64;
        for _ in 0..5000 {
            let k = cycle_at(t);
            assert!(begins(k) <= t && t < begins(k + 1));
            t += 61_337;
        }
    }

    #[test]
    fn rooms_are_dry_about_as_long_as_the_brief_says() {
        // Over many tides: the upper rooms most of a cycle, the middle under half,
        // the deep a few minutes, and the lowest shelf not on every tide.
        let share = |h: f64| {
            let (mut dry, mut all, mut never) = (0i64, 0i64, 0);
            for k in 1000..1400i64 {
                all += begins(k + 1) - begins(k);
                match dry_in(k, h, 0.0) {
                    Some((s, e)) => dry += e - s,
                    None => never += 1,
                }
            }
            (dry as f64 / all as f64, never)
        };
        let (upper, _) = share(0.79);
        let (middle, _) = share(0.36);
        let (sump, sump_never) = share(0.14);
        let (shelf, shelf_never) = share(0.06);
        assert!((0.62..0.75).contains(&upper), "{upper}");
        assert!((0.33..0.45).contains(&middle), "{middle}");
        assert!((0.12..0.25).contains(&sump), "{sump}");
        assert_eq!(sump_never, 0, "the Sump clears on every tide");
        assert!(shelf < 0.10, "{shelf}");
        assert!(
            shelf_never > 20 && shelf_never < 380,
            "the shelf keeps its water on some tides and not all: {shelf_never}"
        );
        // The level is high at a cycle's ends and at its lowest in the middle.
        let k = 1234;
        assert!(level_at(begins(k) + 1000, 0.0) > 0.99);
        assert!(level_at(low_water(k), 0.0) < 0.11);
        // Dry and flooded agree with the window, and a dry room says when it floods.
        let (s, e) = dry_in(k, 0.36, 0.0).unwrap();
        assert!(flooded_at(s - 1000, 0.36, 0.0));
        assert!(!flooded_at(s + 1000, 0.36, 0.0));
        assert_eq!(floods_at(s + 1000, 0.36, 0.0), Some(e));
        assert!(flooded_at(e + 1000, 0.36, 0.0));
        assert!(!flooded_at(e + 1000, 2.0, 0.0), "the Wrack Line is dry");
    }
}
