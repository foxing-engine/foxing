//! The Wistful Trail's own machinery: the Bog of Second Thoughts, which takes a liking
//! to whoever lingers on its tussocks; the wicket into the Long Afternoon, which asks
//! for the word cut into the Wishing Well's rim; and the Lookout, from which you can
//! see whoever is standing anywhere on the trail. The rooms and creatures themselves
//! are in `rooms.rs` and `combat.rs` with the rest.

use super::craft::ITEM;
use super::{plain_num, plain_str, BODY, CITIZEN, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::Outcome;
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
    from: Option<EntityId>,
) -> Result<(), String> {
    world.cancel(JOB_BOG, |p| p["citizen"] == citizen.0)?;
    if key_of(world, room) != Some(BOG) {
        return Ok(());
    }
    let due = world
        .now()
        .plus(foxing_time::Duration::seconds(BOG_SECONDS));
    // The room they stepped in from, kept with the clock: "the way you came" is
    // that room, whatever the trail says (the trail is `back`'s, and a step onto
    // the room at its top retraces).
    world.schedule_once(
        JOB_BOG,
        due,
        json!({"citizen": citizen.0, "actor": actor, "from": from.map(|r| r.0)}),
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
    let from = payload["from"].as_u64().map(EntityId);
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
    // What they carry lives in the satchel under them (the first night's bog looked
    // under the citizen and found nothing to keep).
    let satchel: Option<EntityId> = world
        .store
        .children(citizen)
        .find(|e| e.kind == "satchel")
        .map(|e| e.id);
    let carried: Vec<EntityId> = satchel
        .map(|s| {
            super::craft::things_in(world, s)
                .iter()
                .filter_map(|t| t["id"].as_u64().map(EntityId))
                .collect()
        })
        .unwrap_or_default();
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
    let back = from
        .or_else(|| world.came_from(&actor))
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
    // Spat out is a step back along the trail, not a step on (R1).
    world.pop_came_from(&actor);
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
        "The bog takes a liking to you. It has you to the waist before you notice, then to the chest, and then it loses interest and spits you out at {to_name}, {taken:.0} hit points the poorer{}. The tussocks are the path; cross, do not linger. The trail forgets the bog: `back` from here is the way you came before it.",
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
        // R95: the Stile says what is ahead, which the signpost will not.
        STILE => {
            let exits = super::rooms::exits_of(world, room);
            let name_of =
                |r: EntityId| plain_str(world, r, ROOM, "name").unwrap_or("?").to_string();
            let mut arms: Vec<String> = Vec::new();
            for dir in ["north", "east"] {
                let Some(&next) = exits.get(dir) else {
                    continue;
                };
                let beyond = super::rooms::exits_of(world, next)
                    .into_iter()
                    .find(|(d, to)| d == dir && *to != room)
                    .map(|(_, to)| format!(", and past it {}", name_of(to)))
                    .unwrap_or_default();
                arms.push(format!("{dir}, {}{beyond}", name_of(next)));
            }
            if arms.is_empty() {
                None
            } else {
                Some(format!(
                    "Cut into the top step by someone who did not trust the signpost: {}.",
                    arms.join("; ")
                ))
            }
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
                // R57: a walker quiet five minutes is still on the trail, marked so;
                // one gone ten is not (the room has disremembered them).
                let who: Vec<String> = world
                    .store
                    .children(r)
                    .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
                    .filter_map(|e| plain_str(world, e.id, CITIZEN, "name").map(str::to_string))
                    .map(|n| {
                        let quiet = world
                            .last_active(&n)
                            .map(|t| world.now().since(t).0)
                            .is_some_and(|q| q >= super::presence::QUIET_MS);
                        if quiet {
                            format!("{n} (quiet)")
                        } else {
                            n
                        }
                    })
                    .collect();
                if !who.is_empty() {
                    let rname = plain_str(world, r, ROOM, "name").unwrap_or("?");
                    seen.push(format!("{} in {rname}", who.join(" and ")));
                }
            }
            // R80: the bench remembers who sat here last ("Gnomeward was here 2h ago").
            let last: Vec<String> =
                super::building::came_through_within(world, room, 24 * 3_600_000)
                    .into_iter()
                    .filter(|(n, _)| n != actor)
                    .take(3)
                    .map(|(n, at)| {
                        format!(
                            "{n} was here {} ago",
                            super::combat::human(world.now().0 - at)
                        )
                    })
                    .collect();
            let bench = if last.is_empty() {
                String::new()
            } else {
                format!(" Scratched on the bench: {}.", last.join("; "))
            };
            Some(if seen.is_empty() {
                format!("From the bench you can see the whole trail, every room of it lit, and nobody on it but you.{bench}")
            } else {
                format!(
                    "From the bench you can see the whole trail: {}.{bench}",
                    seen.join("; ")
                )
            })
        }
        _ => None,
    }
}

pub const HUSH: &str = "trail-hush";
pub const JOB_WHISPER: &str = "disremembered.whisper";
pub const WHISPER_MS: i64 = 60 * 1000;
/// A whisper from the Hush arrived: `{from, text, room, heard: [names]}`. Public.
pub const WHISPERED: &str = "disremembered.whispered";

/// Said in the Hush: the words go soft and leave; a minute later they arrive in
/// another room of the trail as a whisper, to whoever stands there, and the one who
/// said them is told where they went. Returns the line for the sayer's reply.
pub fn hush(
    world: &mut World,
    actor: &str,
    room: EntityId,
    text: &str,
) -> Result<Option<String>, String> {
    if key_of(world, room) != Some(HUSH) {
        return Ok(None);
    }
    let others: Vec<&str> = KEYS.iter().copied().filter(|k| *k != HUSH).collect();
    let to_key = others[world.roll(others.len() as u32) as usize];
    let Some(to) = super::rooms::room_by_key(world, to_key) else {
        return Ok(None);
    };
    let due = world.now().plus(foxing_time::Duration(WHISPER_MS));
    world.schedule(
        JOB_WHISPER,
        due,
        json!({"from": actor, "text": text, "to": to.0}),
    )?;
    Ok(Some(" Your words go soft before they are out, and leave for somewhere else on the trail; in a minute they arrive there as a whisper, and you will hear where.".to_string()))
}

fn whisper(world: &mut World, payload: &Value) -> Result<(), String> {
    let from = payload["from"].as_str().unwrap_or("").to_string();
    let text = payload["text"].as_str().unwrap_or("").to_string();
    let to = EntityId(payload["to"].as_u64().unwrap_or(0));
    let Some(rname) = plain_str(world, to, ROOM, "name").map(str::to_string) else {
        return Ok(());
    };
    let heard: Vec<String> = world
        .store
        .children(to)
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .collect();
    for who in &heard {
        super::combat::narrate(
            world,
            who,
            format!("A whisper comes up out of the ground, in {from}'s voice, from the Hush: \"{text}\""),
        );
    }
    let _ = world.note(
        Actor::Driver,
        WHISPERED,
        json!({"from": from, "text": text, "room": to.0, "room_name": rname, "heard": heard}),
    );
    let told = match heard.iter().filter(|h| **h != from).count() {
        0 if heard.contains(&from) => format!("Your whisper from the Hush arrives in {rname}, where you now stand, and you hear yourself say it."),
        0 => format!("Your whisper from the Hush arrives in {rname}; no one is there to hear it, and the grass has it."),
        1 => format!("Your whisper from the Hush arrives in {rname}; {} hears it.", heard.iter().find(|h| **h != from).cloned().unwrap_or_default()),
        n => format!("Your whisper from the Hush arrives in {rname}; {n} hear it."),
    };
    super::combat::narrate(world, &from, told);
    Ok(())
}

/// The trail's props answer `read`: the Stile's signpost (every arm points to a
/// room you have stood in), the Well's rim (the word), the Lookout's plate (the
/// view, with who is where), the Long Afternoon's book (about you: your session).
/// The prose named them and the world refused them, the first night.
pub fn read(world: &World, actor: &str, room: EntityId, arg: &str) -> Option<Outcome> {
    let key = key_of(world, room)?;
    let a = arg.trim().trim_matches('"').to_lowercase();
    let named = |words: &[&str]| a.is_empty() || words.iter().any(|w| a.contains(w));
    let f = super::find_citizen(world, actor)?;
    let text = match key {
        STILE if named(&["signpost", "sign", "post", "arm"]) => {
            let hl = super::Curves::from_env().active_half_life.0 as f64
                * super::atlas::ATLAS_HALF_LIVES;
            let mut names = super::atlas::known_room_names(world, f.citizen, hl);
            names.retain(|n| n != "The Stile");
            names.sort();
            if names.is_empty() {
                "The signpost has one arm, and it points back the way you came, to the Orchard. The others are blank; they fill as you go.".to_string()
            } else {
                format!(
                    "The signpost has {} arms, and every one points to somewhere you have been: {}. None points anywhere you have not.",
                    names.len(),
                    names.join(", ")
                )
            }
        }
        "trail-well" if named(&["rim", "well", "word", "stone", "letters"]) => {
            format!("Inside the rim, deep, in capitals, smoothed over and still legible: {}. The wicket at the Long Afternoon asks for it (`travel wicket \"{}\"`).", WORD.to_uppercase(), WORD)
        }
        LOOKOUT if named(&["plate", "brass", "bench", "view"]) => {
            let seen = look_line(world, actor, room).unwrap_or_default();
            format!("The brass plate names the view: THE WISTFUL TRAIL, NINE ROOMS, EVERY ONE OF THEM MISSED. Under it, in a smaller hand: {}", seen)
        }
        AFTERNOON if named(&["book", "chair", "page"]) => {
            let session = super::session::summary(world, actor)
                .map(|o| o.summary)
                .unwrap_or_default();
            format!("The book on the deck chair is open at a page about you, and it is a good one. It reads: \"{session}\" The next page is blank, and warm from the sun.")
        }
        _ => return None,
    };
    Some(Outcome::new(text, json!({"room": key, "read": a})))
}

pub fn register(world: &mut World) {
    world.on_job(JOB_BOG, Box::new(|w: &mut World, job| bog(w, &job.payload)));
    world.on_job(
        JOB_WHISPER,
        Box::new(|w: &mut World, job| whisper(w, &job.payload)),
    );
}
