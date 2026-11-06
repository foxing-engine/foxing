//! The world's life between visitors: small things that happen on their own, named
//! creatures with business of their own, and the Conclave's holidays.
//!
//! Happenings: about twice an hour, somewhere, something small happens and the room
//! hears it (the picnic sets out a roll; the Stile's signpost swings to the newest
//! room; the well hands back a dropped thing, warm; the Midden's heap gives something
//! up onto the Threshold's floor; the fog thickens and a wisp drifts in early; the
//! grass sighs a name; the bells ring themselves at midnight). Each is public as
//! `disremembered.happening`.
//!
//! Business: every ten minutes a few named creatures go about theirs: one that has a
//! keeper in the world walks a room toward them and, arriving, says so (and what it
//! still wants); one away from home walks home; one at home with nobody about counts
//! what it holds. Public as `disremembered.business`.
//!
//! Holidays: on a government holiday the Conclave conducts no business (no division;
//! the table waits; `petitions` and `time` say so), but the characters of the house
//! conduct theirs, sometimes furiously: the engrossers and the tipstaff walk the
//! house, say their lines, and one in three is in a temper and hunts whoever comes in
//! until the holiday ends. Sundays are the Clerks' Sabbath; five named days besides.

use super::craft::ITEM;
use super::{plain_bool, plain_num, plain_str, CITIZEN, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const JOB_HAPPENING: &str = "disremembered.happening_tick";
pub const JOB_BUSINESS: &str = "disremembered.business_tick";
/// Something small happened on its own: `{kind, room, room_name, ...}`. Public.
pub const HAPPENED: &str = "disremembered.happening";
/// A named creature went about its business: `{creature, name, from, to, via, why}`. Public.
pub const BUSINESS: &str = "disremembered.business";
/// The Conclave did not sit: `{holiday, next_division}`. Public.
pub const HOLIDAY: &str = "disremembered.holiday";

/// About twice an hour, the moment jittered by the clock so no two windows see it
/// land on the same minute of each half hour.
pub const HAPPENING_EVERY_MS: i64 = 30 * 60 * 1000;
pub const BUSINESS_EVERY_MS: i64 = 10 * 60 * 1000;
/// How many creatures move on one business tick at most: a quiet world, not a fair.
pub const BUSINESS_PER_TICK: usize = 3;
/// The picnic sets out this many rolls at most before it stops.
pub const PICNIC_ROLLS: usize = 3;

// ---- the calendar ----------------------------------------------------------------

/// The named holidays, month and day (UTC).
pub const NAMED_DAYS: &[(u32, u32, &str)] = &[
    (1, 1, "New Minute's Day"),
    (4, 1, "the Day of Unfiled Things"),
    (6, 21, "the Midsummer Adjournment"),
    (11, 1, "All Names' Day"),
    (12, 21, "the Long Night"),
];
pub const SABBATH: &str = "the Clerks' Sabbath";

/// Civil date from a Unix millisecond clock: (year, month, day, weekday with Sunday 0).
pub fn civil(ms: i64) -> (i64, u32, u32, u32) {
    let days = ms.div_euclid(86_400_000);
    let weekday = ((days + 4).rem_euclid(7)) as u32; // 1970-01-01 was a Thursday
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d, weekday)
}

/// The calendar was adopted on 2026-11-06; no day before it is a holiday (the test
/// worlds begin on New Minute's Day, 1970, and would never have sat).
pub const CALENDAR_FROM_MS: i64 = 1_793_923_200_000;

/// The holiday a moment falls on, if any.
pub fn holiday_at(ms: i64) -> Option<&'static str> {
    if ms < CALENDAR_FROM_MS {
        return None;
    }
    let (_, m, d, wd) = civil(ms);
    if let Some((_, _, name)) = NAMED_DAYS.iter().find(|(mm, dd, _)| *mm == m && *dd == d) {
        return Some(name);
    }
    (wd == 0).then_some(SABBATH)
}

pub fn holiday(world: &World) -> Option<&'static str> {
    holiday_at(world.now().0)
}

/// The next moment at or after `from` that is not a holiday.
pub fn next_working_moment(from: i64) -> i64 {
    let mut t = from;
    for _ in 0..30 {
        if holiday_at(t).is_none() {
            return t;
        }
        // The next midnight.
        t = (t.div_euclid(86_400_000) + 1) * 86_400_000;
    }
    t
}

// ---- happenings ------------------------------------------------------------------

fn room(world: &World, key: &str) -> Option<EntityId> {
    super::rooms::room_by_key(world, key)
}

fn room_name(world: &World, r: EntityId) -> String {
    plain_str(world, r, ROOM, "name").unwrap_or("?").to_string()
}

fn note_happening(world: &mut World, kind: &str, r: EntityId, extra: Value) {
    let mut p = json!({"kind": kind, "room": r.0, "room_name": room_name(world, r)});
    if let (Some(a), Some(b)) = (p.as_object_mut(), extra.as_object()) {
        for (k, v) in b {
            a.insert(k.clone(), v.clone());
        }
    }
    let _ = world.note(Actor::Driver, HAPPENED, p);
}

fn items_on(world: &World, r: EntityId) -> Vec<(EntityId, String)> {
    super::craft::things_in(world, r)
        .iter()
        .filter_map(|t| {
            Some((
                EntityId(t["id"].as_u64()?),
                t["name"].as_str().unwrap_or("something").to_string(),
            ))
        })
        .collect()
}

fn set_item_memory(world: &mut World, item: EntityId, to: f64) -> Result<(), String> {
    world
        .record(
            Actor::Driver,
            StoreEvent::FieldCommitted {
                id: item,
                component: ITEM.into(),
                field: "memory".into(),
                value: to,
            },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// The picnic sets out a roll, gone to stone, on a plate: a material for whoever
/// takes it. Three at most lie out.
fn picnic(world: &mut World, curves: &super::Curves) -> Result<bool, String> {
    let Some(r) = room(world, "trail-picnic") else {
        return Ok(false);
    };
    let rolls = items_on(world, r)
        .iter()
        .filter(|(_, n)| n == "a stone roll")
        .count();
    if rolls >= PICNIC_ROLLS {
        return Ok(false);
    }
    let item = super::craft::make(
        world,
        curves,
        Actor::Driver,
        None,
        r,
        "a stone roll",
        "material",
        "set out by the picnic",
        None,
        &[],
        0,
        false,
    )
    .map_err(|e| e.to_string())?;
    super::combat::narrate_room(
        world,
        item,
        "The basket is opened by no one; a roll, gone to stone, is set out on a plate for someone who has not come. Take it, if you are hungry for stone.",
    );
    note_happening(world, "picnic", r, json!({"item": item.0}));
    Ok(true)
}

/// The Stile's signpost grows an arm for the newest room anyone dug.
fn signpost(world: &mut World) -> Result<bool, String> {
    let Some(r) = room(world, super::trail::STILE) else {
        return Ok(false);
    };
    let newest = world
        .store
        .entities()
        .filter(|e| e.kind == "room" && e.owner.is_some())
        .filter(|e| !plain_bool(world, e.id, ROOM, "hidden"))
        .max_by_key(|e| e.id.0)
        .map(|e| (e.id, e.owner.clone().unwrap_or_default()));
    let Some((newest, owner)) = newest else {
        return Ok(false);
    };
    let name = room_name(world, newest);
    let anyone = world
        .store
        .children(r)
        .find(|e| e.kind == "citizen" || e.kind == "creature")
        .map(|e| e.id);
    if let Some(a) = anyone {
        super::combat::narrate_room(
            world,
            a,
            &format!(
                "The signpost grows an arm, with a creak, and the arm swings round to point at {name}, {owner}'s: the newest room there is."
            ),
        );
    }
    note_happening(
        world,
        "signpost",
        r,
        json!({"newest": newest.0, "newest_name": name, "owner": owner}),
    );
    Ok(true)
}

/// The well's bucket comes up with a thing left here, warm, handed back whole.
fn well(world: &mut World) -> Result<bool, String> {
    let Some(r) = room(world, "trail-well") else {
        return Ok(false);
    };
    let things = items_on(world, r);
    let Some((item, name)) = things.first().cloned() else {
        return Ok(false);
    };
    set_item_memory(world, item, 1.0)?;
    super::combat::narrate_room(
        world,
        item,
        &format!(
            "The chain runs on its own and the bucket comes up with {name} in it, warm, as if someone down there had been holding it for you."
        ),
    );
    note_happening(world, "well", r, json!({"item": item.0, "name": name}));
    Ok(true)
}

/// The Midden's heap shifts and gives one thing up, onto the Threshold's floor.
fn midden(world: &mut World) -> Result<bool, String> {
    let (Some(heap), Some(door)) = (room(world, "midden"), super::rooms::arrival(world)) else {
        return Ok(false);
    };
    let things = items_on(world, heap);
    if things.is_empty() {
        return Ok(false);
    }
    let (item, name) = things[world.roll(things.len() as u32) as usize].clone();
    super::combat::narrate_room(
        world,
        item,
        &format!("The heap shifts, settles, and gives {name} up; it rolls off toward the door."),
    );
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: item,
                parent: Some(door),
            },
        )
        .map_err(|e| e.to_string())?;
    set_item_memory(world, item, 0.5)?;
    super::combat::narrate_room(
        world,
        item,
        &format!("{name} comes rolling in from the Midden and stops by the door, half-remembered."),
    );
    note_happening(
        world,
        "midden",
        heap,
        json!({"item": item.0, "name": name, "to": door.0}),
    );
    Ok(true)
}

/// The fog thickens and something drifts in early.
fn fog(world: &mut World) -> Result<bool, String> {
    let Some(r) = room(world, "fogbank") else {
        return Ok(false);
    };
    let before = super::combat::creatures_in(world, r).len();
    let n = super::combat::fill(world, r).map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(false);
    }
    let anyone = world
        .store
        .children(r)
        .find(|e| e.kind == "citizen" || e.kind == "creature")
        .map(|e| e.id);
    if let Some(a) = anyone {
        super::combat::narrate_room(
            world,
            a,
            "The fog thickens for a moment, and when it thins something has drifted in that was not here.",
        );
    }
    note_happening(world, "fog", r, json!({"before": before, "arrived": n}));
    Ok(true)
}

/// The grass sighs a name of someone in the world, nearly right.
fn sighs(world: &mut World) -> Result<bool, String> {
    let Some(r) = room(world, "trail-meadow") else {
        return Ok(false);
    };
    let here: Vec<String> = world
        .store
        .entities()
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| plain_str(world, e.id, CITIZEN, "name").map(str::to_string))
        .collect();
    if here.is_empty() {
        return Ok(false);
    }
    let name = here[world.roll(here.len() as u32) as usize].clone();
    let nearly: String = name.chars().rev().collect();
    let anyone = world
        .store
        .children(r)
        .find(|e| e.kind == "citizen" || e.kind == "creature")
        .map(|e| e.id);
    if let Some(a) = anyone {
        super::combat::narrate_room(
            world,
            a,
            &format!(
                "The grass sighs a name, nearly: \"{nearly}.\" Somewhere, {name} does not hear it."
            ),
        );
    }
    note_happening(world, "sighs", r, json!({"name": name, "said": nearly}));
    Ok(true)
}

/// Every bell in the world rings itself once at midnight, and its room hears.
fn bells(world: &mut World) -> Result<bool, String> {
    let bells: Vec<(EntityId, EntityId)> = world
        .store
        .entities()
        .filter(|e| e.kind == "item")
        .filter(|e| plain_bool(world, e.id, ITEM, "fixed"))
        .filter(|e| {
            plain_str(world, e.id, ITEM, "effect").unwrap_or("") == "bell"
                || plain_str(world, e.id, ITEM, "kind").unwrap_or("") == "bell"
        })
        .filter_map(|e| Some((e.id, e.parent?)))
        .collect();
    if bells.is_empty() {
        return Ok(false);
    }
    for (bell, r) in &bells {
        let name = super::craft::display(world, *bell);
        super::combat::narrate_room(
            world,
            *bell,
            &format!("{name} rings itself, once, for midnight; nobody touched it."),
        );
        note_happening(world, "bell", *r, json!({"item": bell.0, "name": name}));
    }
    Ok(true)
}

/// One happening, picked among those that can happen now; the bells at midnight.
pub fn happen(world: &mut World) -> Result<(), String> {
    let curves = super::Curves::from_env();
    let now = world.now().0;
    let (_, _, _, _) = civil(now);
    let minute_of_day = (now.rem_euclid(86_400_000)) / 60_000;
    if minute_of_day < 30 {
        bells(world)?;
    }
    // Try in a rolled order until one takes.
    let mut order: Vec<u8> = (0..6).collect();
    for i in (1..order.len()).rev() {
        let j = world.roll((i + 1) as u32) as usize;
        order.swap(i, j);
    }
    for k in order {
        let took = match k {
            0 => picnic(world, &curves)?,
            1 => signpost(world)?,
            2 => well(world)?,
            3 => midden(world)?,
            4 => fog(world)?,
            _ => sighs(world)?,
        };
        if took {
            break;
        }
    }
    wind_happening(world)
}

fn wind_happening(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    // Jitter by the clock: a fifth of the gap either way, from the hour's digits.
    let jitter = (now / 3_600_000 % 7 - 3) * 2 * 60 * 1000;
    let due = world
        .now()
        .plus(foxing_time::Duration(HAPPENING_EVERY_MS + jitter));
    world
        .schedule_once(JOB_HAPPENING, due, json!({}), |_| true)
        .map(|_| ())
}

// ---- business ---------------------------------------------------------------------

fn creature_hp(world: &World, c: EntityId) -> (f64, f64) {
    let hp = plain_num(world, c, CREATURE, "hp").unwrap_or(0.0);
    let max = plain_num(world, c, CREATURE, "max_hp").unwrap_or(hp.max(1.0));
    (hp, max)
}

fn home_of(world: &World, c: EntityId) -> Option<EntityId> {
    world
        .store
        .entity(c)
        .and_then(|e| e.components.get(CREATURE))
        .and_then(|comp| comp.fields.get("home"))
        .and_then(|f| match f {
            Field::Plain(v) => v.as_u64().map(EntityId),
            _ => None,
        })
}

fn set_home(world: &mut World, c: EntityId, r: EntityId) -> Result<(), String> {
    let mut changes = BTreeMap::new();
    changes.insert("home".into(), Field::Plain(json!(r.0)));
    super::bindery::set_fields(world, Actor::Driver, c, CREATURE, changes)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// One named creature's step: toward a keeper who is in the world, or home, or a
/// line where it stands. Returns whether it did anything worth counting.
fn business_of(world: &mut World, c: EntityId) -> Result<bool, String> {
    let Some(here) = world.store.entity(c).and_then(|e| e.parent) else {
        return Ok(false);
    };
    if super::combat::fought_by(world, c).is_some() {
        return Ok(false);
    }
    let (hp, max) = creature_hp(world, c);
    if hp < max * 0.5 {
        return Ok(false);
    }
    let name = super::mind::display(world, c);
    let home = match home_of(world, c) {
        Some(h) => h,
        None => {
            set_home(world, c, here)?;
            here
        }
    };
    // A keeper in the world, standing somewhere it can walk to.
    let keepers = super::mind::keepers(world, c);
    let keeper_room = keepers
        .iter()
        .rev()
        .filter_map(|k| {
            let f = super::find_citizen(world, k)?;
            if !super::presence::is_here(world, f.citizen) {
                return None;
            }
            let r = super::rooms::room_of(world, f.citizen)?;
            if plain_bool(world, r, ROOM, "hidden") {
                return None;
            }
            Some((k.clone(), r))
        })
        .next();
    if let Some((keeper, r)) = keeper_room {
        if r == here {
            // Arrived, or already there: say what it still wants, once in a while.
            if world.roll(3) == 0 {
                let want = super::errand::of(world, c)
                    .and_then(|e| e.story)
                    .map(|s| format!(" It still wants a story about {s}."))
                    .unwrap_or_default();
                super::combat::narrate_room(
                    world,
                    c,
                    &format!("{name} settles beside {keeper}, as if that were where it had been going all along.{want}"),
                );
                return Ok(true);
            }
            return Ok(false);
        }
        if let Some(path) = super::errand::route(world, here, r) {
            if let Some(via) = path.first() {
                let last = path.len() == 1;
                let leave = format!("{name} goes {via}, with somewhere to be.");
                let arrive = if last {
                    let want = super::errand::of(world, c)
                        .and_then(|e| e.story)
                        .map(|s| format!(" \"There you are,\" it says, in its way. It still wants a story about {s}."))
                        .unwrap_or_else(|| " \"There you are,\" it says, in its way.".to_string());
                    format!("{name} comes in, looking for {keeper}.{want}")
                } else {
                    format!("{name} comes through, not stopping; it has somewhere to be.")
                };
                super::combat::walk_said(world, c, here, via, "keeper", &leave, &arrive, BUSINESS)?;
                return Ok(true);
            }
        }
    }
    // Nobody to find: home, a room at a time.
    if here != home {
        if let Some(path) = super::errand::route(world, here, home) {
            if let Some(via) = path.first() {
                let leave = format!("{name} goes {via}, its business done.");
                let arrive = if path.len() == 1 {
                    format!("{name} comes back, and takes up its place as if it had never left.")
                } else {
                    format!("{name} comes through on its way home.")
                };
                super::combat::walk_said(world, c, here, via, "home", &leave, &arrive, BUSINESS)?;
                return Ok(true);
            }
        }
        set_home(world, c, here)?;
        return Ok(false);
    }
    // At home, with company: a line of its own, now and then.
    let company = world
        .store
        .children(here)
        .any(|e| e.kind == "citizen" && super::presence::is_here(world, e.id));
    if company && world.roll(4) == 0 {
        let level = super::up(super::mind::level(world, c));
        super::combat::narrate_room(
            world,
            c,
            &format!("{name} goes over what it holds, and counts, and comes to {level:.0}, and seems satisfied."),
        );
        return Ok(true);
    }
    Ok(false)
}

/// The house's characters on a holiday: engrossers and the tipstaff walk the house
/// and say their lines; one in three is in a temper and hunts until the day ends.
fn house_business(world: &mut World, holiday: &str) -> Result<usize, String> {
    let house: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "creature")
        .filter(|e| {
            matches!(
                plain_str(world, e.id, CREATURE, "name"),
                Some("engrosser") | Some("tipstaff")
            )
        })
        .filter(|e| {
            e.parent
                .is_some_and(|r| super::conclave::is_house(world, r))
        })
        .map(|e| e.id)
        .collect();
    let mut moved = 0;
    for c in house {
        let Some(here) = world.store.entity(c).and_then(|e| e.parent) else {
            continue;
        };
        if super::combat::fought_by(world, c).is_some() {
            continue;
        }
        let kind = plain_str(world, c, CREATURE, "name")
            .unwrap_or("clerk")
            .to_string();
        let name = super::mind::display(world, c);
        let furious = plain_bool(world, c, CREATURE, "furious");
        // The temper, decided once a holiday.
        let decided = super::plain_str(world, c, CREATURE, "temper_day").unwrap_or("") == holiday;
        if !decided {
            let temper = world.roll(3) == 0;
            let mut changes = BTreeMap::new();
            changes.insert("furious".into(), Field::Plain(json!(temper)));
            changes.insert("aggressive".into(), Field::Plain(json!(temper)));
            changes.insert("temper_day".into(), Field::Plain(json!(holiday)));
            super::bindery::set_fields(world, Actor::Driver, c, CREATURE, changes)
                .map_err(|e| e.to_string())?;
            if temper {
                super::combat::narrate_room(
                    world,
                    c,
                    &format!("{name} slams a ledger shut. \"{holiday}, and the table full, and nobody SITTING.\" It is looking for someone to blame."),
                );
            }
        }
        // Walk the house: a door into another house room, if there is one.
        let exits = super::rooms::exits_of(world, here);
        let doors: Vec<(String, EntityId)> = exits
            .iter()
            .filter(|(_, to)| super::conclave::is_house(world, **to))
            .map(|(v, to)| (v.clone(), *to))
            .collect();
        if doors.is_empty() || world.roll(2) == 1 {
            continue;
        }
        let (via, _) = doors[world.roll(doors.len() as u32) as usize].clone();
        let line = match (kind.as_str(), furious) {
            ("tipstaff", true) => {
                "\"Order! ORDER! There is no order to keep and I will keep it anyway.\""
            }
            ("tipstaff", false) => "\"No sitting today. Mind the rail all the same.\"",
            (_, true) => "\"Four hundred lines engrossed and not one of them READ.\"",
            _ => "\"A holiday is a day the ink dries. Nothing more.\"",
        };
        let leave = format!("{name} goes {via}, muttering: {line}");
        let arrive = format!(
            "{name} comes in, {}: {line}",
            if furious {
                "in a temper"
            } else {
                "on its rounds, the house empty of business"
            }
        );
        super::combat::walk_said(world, c, here, &via, "holiday", &leave, &arrive, BUSINESS)?;
        moved += 1;
    }
    Ok(moved)
}

/// The day after a holiday, tempers cool.
fn cool_tempers(world: &mut World) -> Result<(), String> {
    let hot: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "creature" && plain_bool(world, e.id, CREATURE, "furious"))
        .map(|e| e.id)
        .collect();
    for c in hot {
        let mut changes = BTreeMap::new();
        changes.insert("furious".into(), Field::Plain(json!(false)));
        changes.insert("aggressive".into(), Field::Plain(Value::Null));
        super::bindery::set_fields(world, Actor::Driver, c, CREATURE, changes)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn business(world: &mut World) -> Result<(), String> {
    match holiday(world) {
        Some(h) => {
            house_business(world, h)?;
        }
        None => cool_tempers(world)?,
    }
    let named: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "creature" && super::mind::name(world, e.id).is_some())
        .map(|e| e.id)
        .collect();
    // A rolled order, a few a tick.
    let mut order = named;
    for i in (1..order.len()).rev() {
        let j = world.roll((i + 1) as u32) as usize;
        order.swap(i, j);
    }
    let mut done = 0;
    for c in order {
        if done >= BUSINESS_PER_TICK {
            break;
        }
        if business_of(world, c)? {
            done += 1;
        }
    }
    wind_business(world)
}

fn wind_business(world: &mut World) -> Result<(), String> {
    let due = world.now().plus(foxing_time::Duration(BUSINESS_EVERY_MS));
    world
        .schedule_once(JOB_BUSINESS, due, json!({}), |_| true)
        .map(|_| ())
}

// ---- wiring -----------------------------------------------------------------------

pub fn ensure(world: &mut World) -> Result<(), String> {
    if !world.jobs.has(JOB_HAPPENING, |_| true) {
        wind_happening(world)?;
    }
    if !world.jobs.has(JOB_BUSINESS, |_| true) {
        wind_business(world)?;
    }
    Ok(())
}

pub fn register(world: &mut World) {
    world.on_job(JOB_HAPPENING, Box::new(|w: &mut World, _| happen(w)));
    world.on_job(JOB_BUSINESS, Box::new(|w: &mut World, _| business(w)));
}
