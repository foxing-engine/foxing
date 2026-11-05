//! The Wistful Trail's own machinery: the Bog of Second Thoughts, which takes a liking
//! to whoever lingers on its tussocks; the wicket into the Long Afternoon, which asks
//! for the word cut into the Wishing Well's rim; and the Lookout, from which you can
//! see whoever is standing anywhere on the trail. The rooms and creatures themselves
//! are in `rooms.rs` and `combat.rs` with the rest.

use super::craft::ITEM;
use super::{plain_num, plain_str, BODY, CITIZEN, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};

/// The word the wicket asks for. It is cut into the well's rim for anyone who reads.
pub const WORD: &str = "linger";
/// What hangs over the wicket.
pub const HINT: &str = "Not go. Not stay. The word for the thing between, and the well knows it.";
/// How long the bog lets you stand on its tussocks before it takes a liking to you.
pub const BOG_SECONDS: i64 = 20;
/// The bog takes this share of your hit points, and never the last one.
pub const BOG_SHARE: f64 = 0.2;
pub const JOB_BOG: &str = "disremembered.bog";
/// The bog had someone: `{citizen, actor, hp_taken, kept: item id or null, spat_to}`.
pub const BOGGED: &str = "disremembered.bogged";

pub const BOG: &str = "trail-bog";
pub const AFTERNOON: &str = "trail-afternoon";
pub const LOOKOUT: &str = "trail-lookout";
pub const STILE: &str = "trail-stile";

/// The trail's rooms, in walking order.
pub const KEYS: &[&str] = &[
    STILE,
    "trail-meadow",
    "trail-picnic",
    "trail-hollow-way",
    "trail-well",
    AFTERNOON,
    "trail-hush",
    BOG,
    LOOKOUT,
];

fn key_of(world: &World, room: EntityId) -> Option<&str> {
    plain_str(world, room, ROOM, "key")
}

/// The wicket's ward, set once: a room that already has one (an owner's, or an
/// older seed's) keeps it.
pub fn ensure(world: &mut World) -> Result<(), String> {
    let Some(room) = super::rooms::room_by_key(world, AFTERNOON) else {
        return Ok(());
    };
    if super::building::ward_of(world, room).is_some() {
        return Ok(());
    }
    let mut fields = world
        .store
        .entity(room)
        .and_then(|e| e.components.get(ROOM))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    fields.insert(
        "ward".into(),
        Field::Plain(json!({"word": WORD, "hint": HINT, "passed": [], "by": "the trail"})),
    );
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id: room,
                component: ROOM.into(),
                fields,
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Someone arrives somewhere on foot: a bog clock they were under stops, and the
/// bog, if that is where they are, starts one.
pub fn on_arrival(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    room: EntityId,
) -> Result<(), String> {
    world.cancel(JOB_BOG, |p| p["citizen"] == citizen.0)?;
    if key_of(world, room) != Some(BOG) {
        return Ok(());
    }
    let due = world
        .now()
        .plus(foxing_time::Duration::seconds(BOG_SECONDS));
    world.schedule_once(
        JOB_BOG,
        due,
        json!({"citizen": citizen.0, "actor": actor}),
        |p| p["citizen"] == citizen.0,
    )?;
    Ok(())
}

/// The bog's clock runs out on someone still standing in it: a fifth of their hit
/// points (never the last), one thing they carry kept by the bog, and they are spat
/// out the way they came. A name that has gone quiet is left alone; the bog likes
/// company, not furniture.
fn bog(world: &mut World, payload: &Value) -> Result<(), String> {
    let citizen = EntityId(payload["citizen"].as_u64().unwrap_or(0));
    let actor = payload["actor"].as_str().unwrap_or("").to_string();
    let Some(room) = world.store.entity(citizen).and_then(|e| e.parent) else {
        return Ok(());
    };
    if key_of(world, room) != Some(BOG) || !super::presence::is_here(world, citizen) {
        return Ok(());
    }
    // Hit points: a fifth, floor three, never below one.
    let hp = super::combat::hp(world, citizen);
    let max = plain_num(world, citizen, BODY, "max_hp").unwrap_or(hp.max(1.0));
    let taken = (max * BOG_SHARE).max(3.0).min((hp - 1.0).max(0.0));
    if taken > 0.0 {
        super::combat::commit_hp(world, citizen, hp - taken).map_err(|e| e.to_string())?;
    }
    // One carried thing, kept by the bog: it lies on the tussocks for whoever comes
    // back for it.
    let carried: Vec<EntityId> = world
        .store
        .children(citizen)
        .filter(|e| e.kind == "item")
        .map(|e| e.id)
        .collect();
    let kept = if carried.is_empty() {
        None
    } else {
        let pick = carried[world.roll(carried.len() as u32) as usize];
        world
            .record(
                Actor::Driver,
                StoreEvent::EntityMoved {
                    id: pick,
                    parent: Some(room),
                },
            )
            .map_err(|e| e.to_string())?;
        Some(pick)
    };
    let kept_name = kept
        .and_then(|k| plain_str(world, k, ITEM, "name"))
        .map(str::to_string);
    // Spat out the way they came, or over the stile if the bog cannot remember.
    let back = world
        .came_from(&actor)
        .filter(|r| *r != room && world.store.entity(*r).is_some())
        .or_else(|| super::rooms::room_by_key(world, STILE));
    let Some(to) = back else {
        return Ok(());
    };
    super::combat::leave_fight(world, &actor, citizen)?;
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: citizen,
                parent: Some(to),
            },
        )
        .map_err(|e| e.to_string())?;
    world.set_came_from(&actor, room);
    super::building::visited(world, to)?;
    let _ = super::atlas::mark_seen(world, &actor, to);
    super::combat::on_arrival(world, &actor, citizen, to)?;
    let to_name = plain_str(world, to, ROOM, "name")
        .unwrap_or("?")
        .to_string();
    let _ = world.note(
        Actor::Driver,
        BOGGED,
        json!({"citizen": citizen.0, "actor": actor, "hp_taken": taken, "kept": kept.map(|k| k.0), "spat_to": to.0}),
    );
    let said = format!(
        "The bog takes a liking to you. It has you to the waist before you notice, then to the chest, and then it loses interest and spits you out at {to_name}, {taken:.0} hit points the poorer{}. The tussocks are the path; cross, do not linger.",
        kept_name
            .map(|n| format!(", and it has kept your {n}"))
            .unwrap_or_default()
    );
    super::combat::narrate(world, &actor, said);
    let name = plain_str(world, citizen, CITIZEN, "name")
        .unwrap_or(&actor)
        .to_string();
    super::combat::narrate_others(
        world,
        citizen,
        &format!("{name} is spat out of the bog, wet to the chin."),
        &actor,
    );
    Ok(())
}

/// One line for `look`: in the bog, how long before it takes a liking to you; on the
/// Lookout, who stands where along the trail.
pub fn look_line(world: &World, actor: &str, room: EntityId) -> Option<String> {
    match key_of(world, room)? {
        BOG => {
            let f = super::find_citizen(world, actor)?;
            let due = world
                .jobs
                .pending()
                .find(|j| j.kind == JOB_BOG && j.payload["citizen"] == f.citizen.0)
                .map(|j| j.due)?;
            let left = (due.0 - world.now().0).max(0);
            Some(format!(
                "You are on a tussock, and the bog has noticed: it takes a liking to you in {}.",
                super::combat::human(left)
            ))
        }
        LOOKOUT => {
            let mut seen: Vec<String> = Vec::new();
            for key in KEYS {
                if *key == LOOKOUT {
                    continue;
                }
                let Some(r) = super::rooms::room_by_key(world, key) else {
                    continue;
                };
                let who: Vec<String> = world
                    .store
                    .children(r)
                    .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
                    .filter_map(|e| plain_str(world, e.id, CITIZEN, "name").map(str::to_string))
                    .collect();
                if !who.is_empty() {
                    let rname = plain_str(world, r, ROOM, "name").unwrap_or("?");
                    seen.push(format!("{} in {rname}", who.join(" and ")));
                }
            }
            Some(if seen.is_empty() {
                "From the bench you can see the whole trail, every room of it lit, and nobody on it but you.".to_string()
            } else {
                format!(
                    "From the bench you can see the whole trail: {}.",
                    seen.join("; ")
                )
            })
        }
        _ => None,
    }
}

pub fn register(world: &mut World) {
    world.on_job(JOB_BOG, Box::new(|w: &mut World, job| bog(w, &job.payload)));
}
