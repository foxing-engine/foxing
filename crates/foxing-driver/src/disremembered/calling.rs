//! The fourth level of a creature's memory: a calling (David, 11-25).
//!
//! Holding nothing, a creature is scenery; holding something, it keeps what it took;
//! at sixty it has a name. From one to fifty-nine nothing changed, and from sixty-one
//! to six hundred nothing changed either. At a hundred and fifty a named creature is
//! *called*: it takes up one goal of its own at a time, works at it a step on each
//! turn of its business (life.rs), says where it stands when asked, may refuse a
//! story while it is on the road, and the record keeps how the thing ended. Under
//! seventy-five the calling goes, as a name goes under thirty.
//!
//! The model chooses; the world acts. A called creature with nothing in hand is
//! listed for the Docent's desk (`GET /v0/docent`, "callings") with the goals the
//! world could carry out for it right now, each already made concrete ("a pilgrimage
//! to the Drowned Wing"). The desk's model picks one key and writes a sentence in the
//! creature's voice; the operator's `calling` verb takes only a key from that menu.
//! Twenty minutes unanswered, the world rolls one itself, so nothing waits on a
//! model. What a goal does is the world's own rules, below.
//!
//! The goals: raise an heir (feed one unnamed creature its stories to a name); a
//! pilgrimage (walk to a far room and back); a collection (three things of a kind
//! brought to its room); a grudge (seek out the name that took a memory from it,
//! and, unsettled, lay a petition before the Conclave); a debt (find the name that
//! fed it and pay); a work (a story of its own, told in three rooms); a den (dig a
//! home off the room it lives in, and keep it); and respect to the dead (a gift at
//! a niche of the Ossuary). A trade at the Souvenir Shop waits for the shop (C17).
//! The Court hears four kinds of case between names and none of them is a
//! creature's; a creature's grievance goes to the Conclave, which takes a rule and
//! never a ruling on someone.

use super::mind::{self, MIND};
use super::{find_citizen, plain_bool, plain_str, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A calling was taken up, finished or let go: `{creature, name, kind, goal, stage
/// ("taken", "done", "dropped"), said, line, how, helpers, by}`. Public.
pub const CALLING: &str = "disremembered.calling";
/// The bar, in level; lost again at half of it (mind::TIERS).
pub const BAR: f64 = 150.0;
/// The tier a called creature stands on.
pub const TIER: u32 = 2;
/// Between one calling and the next.
pub const REST_MS: i64 = 6 * 3_600_000;
/// Unanswered by the desk this long, the world chooses.
pub const CHOOSE_WAIT_MS: i64 = 20 * 60_000;
/// A calling nobody could finish is let go.
pub const GIVE_UP_MS: i64 = 3 * 86_400_000;
/// What helping a calling through is worth, and what the creature keeps of it.
pub const HELP_XP: f64 = 15.0;
pub const DONE_XP: f64 = 20.0;
/// A collection is this many things.
pub const COLLECTED: u64 = 3;
/// A work is told in this many rooms.
pub const WORK_ROOMS: usize = 3;
/// Between an heir's lessons.
pub const LESSON_MS: i64 = 3_600_000;
/// Steps a grudge waits, met and unsettled or never met, before it petitions.
pub const GRUDGE_PATIENCE: u64 = 6;
const LINE_MAX: usize = 200;

/// The kinds of thing a creature may want collected: the word looked for in a
/// thing's name, and how three of them are said.
const WANTED: &[(&str, &str)] = &[
    ("bandage", "three bandages"),
    ("lantern", "three lanterns"),
    ("charm", "three charms"),
    ("jar", "three jars"),
    ("dart", "three darts"),
    ("ribbon", "three ribbon markers"),
];

/// One goal the world could carry out for a creature now.
pub struct Offer {
    pub key: &'static str,
    /// The goal made concrete, as a phrase after "its calling: ".
    pub said: String,
    pub data: Value,
}

fn mind_val(world: &World, c: EntityId, field: &str) -> Option<Value> {
    match world
        .store
        .entity(c)?
        .components
        .get(MIND)?
        .fields
        .get(field)?
    {
        Field::Plain(v) if !v.is_null() => Some(v.clone()),
        _ => None,
    }
}

/// The calling in hand, if any.
pub fn of(world: &World, c: EntityId) -> Option<Value> {
    mind_val(world, c, "calling").filter(Value::is_object)
}

fn set(world: &mut World, c: EntityId, changes: Vec<(&str, Value)>) -> Result<(), String> {
    let changes: BTreeMap<String, Field> = changes
        .into_iter()
        .map(|(k, v)| (k.to_string(), Field::Plain(v)))
        .collect();
    mind::set_mind_pub(world, Actor::Driver, c, changes).map_err(|e| e.to_string())
}

fn here_of(world: &World, c: EntityId) -> Option<EntityId> {
    world.store.entity(c).and_then(|e| e.parent)
}

fn room_name(world: &World, r: EntityId) -> String {
    plain_str(world, r, ROOM, "name")
        .unwrap_or("a room")
        .to_string()
}

fn kind_of(world: &World, c: EntityId) -> String {
    plain_str(world, c, CREATURE, "name")
        .unwrap_or("creature")
        .to_string()
}

fn citizens_here(world: &World, room: EntityId) -> Vec<String> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .collect()
}

/// A citizen's room, when they are in the world and somewhere a creature can walk.
fn room_of_name(world: &World, who: &str) -> Option<EntityId> {
    let f = find_citizen(world, who)?;
    if !super::presence::is_here(world, f.citizen) {
        return None;
    }
    let r = super::rooms::room_of(world, f.citizen)?;
    (!plain_bool(world, r, ROOM, "hidden")).then_some(r)
}

/// The world's own rooms a creature could walk to, with how many doors away.
fn rooms_from(world: &World, from: EntityId) -> Vec<(EntityId, usize)> {
    world
        .store
        .of_kind("room")
        .filter(|e| e.owner.is_none() && e.id != from)
        .filter(|e| !plain_bool(world, e.id, ROOM, "hidden"))
        .filter_map(|e| super::errand::route(world, from, e.id).map(|p| (e.id, p.len())))
        .collect()
}

/// An ordinary creature: not a boss, not the house's own, not a starfish. (A name
/// the Docent keeps near the door is ordinary: Watch 28's Gedder was one, and stood
/// at 185 with no calling because it was left out here.) Only these
/// take a calling or are raised as an heir.
fn ordinary(world: &World, c: EntityId) -> bool {
    let kind = kind_of(world, c);
    !super::combat::is_boss(&kind)
        && kind != "tapster"
        && kind != "shopkeep"
        && !super::starfish::is(world, c)
}

/// The goals the world could carry out for this creature now, each made concrete.
pub fn menu(world: &World, c: EntityId) -> Vec<Offer> {
    let mut out = Vec::new();
    let Some(here) = here_of(world, c) else {
        return out;
    };
    let home = super::life::home_of(world, c).unwrap_or(here);
    // The same pick all day, whoever asks: a menu that changed between the desk's
    // reading and its answer would refuse the answer.
    let day = (world.now().0 / 86_400_000) as usize;
    let pick = |n: usize| (c.0 as usize).wrapping_add(day) % n.max(1);
    let near = rooms_from(world, home);
    // An heir: an unnamed creature at home or a door or two away, that holds a mind
    // or could (the house's own and the bosses are nobody's apprentice).
    let mut rooms_near: Vec<EntityId> = vec![home];
    rooms_near.extend(near.iter().filter(|(_, d)| *d <= 2).map(|(r, _)| *r));
    let apprentice = rooms_near.iter().find_map(|r| {
        super::combat::creatures_in(world, *r)
            .into_iter()
            .map(|x| x.0)
            .find(|&o| o != c && mind::name(world, o).is_none() && ordinary(world, o))
            .map(|o| (o, *r))
    });
    if let Some((o, r)) = apprentice {
        out.push(Offer {
            key: "heir",
            said: format!(
                "to raise an heir: the {} in {}, fed its stories until it has a name",
                kind_of(world, o),
                room_name(world, r)
            ),
            data: json!({"apprentice": o.0}),
        });
    }
    // A pilgrimage: four doors off or more, the same one all day.
    let far: Vec<&(EntityId, usize)> = near.iter().filter(|(_, d)| (4..=9).contains(d)).collect();
    if !far.is_empty() {
        let (r, _) = far[pick(far.len())];
        out.push(Offer {
            key: "pilgrimage",
            said: format!("a pilgrimage to {}, and home again", room_name(world, *r)),
            data: json!({"target": r.0}),
        });
    }
    let (word, three) = WANTED[pick(WANTED.len())];
    out.push(Offer {
        key: "collection",
        said: format!(
            "a collection: {three}, brought to it in {}",
            room_name(world, home)
        ),
        data: json!({"word": word, "three": three}),
    });
    // A grudge: the last name that took a memory from it.
    let taker = world
        .recent_events(5000)
        .iter()
        .rev()
        .filter(|e| e.kind == mind::TAKEN && e.payload["creature"] == c.0)
        .filter_map(|e| e.payload["by"].as_str().map(str::to_string))
        .find(|by| find_citizen(world, by).is_some());
    if let Some(who) = &taker {
        out.push(Offer {
            key: "grudge",
            said: format!("a grudge against {who}, who took a memory from it"),
            data: json!({"who": who}),
        });
    }
    // A debt: who fed it to its name, or keeps it now.
    let owed = mind::namer(world, c)
        .or_else(|| mind::keepers(world, c).into_iter().next_back())
        .filter(|w| find_citizen(world, w).is_some() && Some(w) != taker.as_ref());
    if let Some(who) = owed {
        out.push(Offer {
            key: "debt",
            said: format!("a debt to {who}, who fed it"),
            data: json!({"who": who}),
        });
    }
    // A work, told at home and in the two nearest rooms.
    let mut stops: Vec<EntityId> = vec![home];
    let mut by_near: Vec<&(EntityId, usize)> = near.iter().collect();
    by_near.sort_by_key(|(r, d)| (*d, r.0));
    stops.extend(by_near.iter().take(WORK_ROOMS - 1).map(|(r, _)| *r));
    if stops.len() == WORK_ROOMS {
        out.push(Offer {
            key: "work",
            said: format!(
                "a work of its own, told in three parts: in {}, in {} and in {}",
                room_name(world, stops[0]),
                room_name(world, stops[1]),
                room_name(world, stops[2])
            ),
            data: json!({"rooms": stops.iter().map(|r| r.0).collect::<Vec<_>>()}),
        });
    }
    // A den: none yet, and its home takes another door.
    let exits = super::rooms::exits_of(world, home);
    let door = ["den", "burrow", "nest", "hollow"]
        .into_iter()
        .find(|d| !exits.contains_key(*d));
    if den_of(world, c).is_none()
        && Some(home) != super::rooms::arrival(world)
        && (exits.len() < super::building::MAX_EXITS || super::rooms::is_mudroom(world, home))
    {
        if let Some(door) = door {
            out.push(Offer {
                key: "den",
                said: format!(
                    "to dig a home of its own off {}, and keep it",
                    room_name(world, home)
                ),
                data: json!({"door": door, "from": home.0}),
            });
        }
    }
    // A trade: something of its own, carried to the Souvenir Shop (shop.rs).
    if let Some(shop) = super::shop::room(world) {
        if super::shop::keeper(world).is_some() && super::errand::route(world, home, shop).is_some()
        {
            out.push(Offer {
                key: "trade",
                said: "a trade: something of its own carried to the Souvenir Shop on Lantern Row, and cashed in".to_string(),
                data: json!({}),
            });
        }
    }
    // Respect to the dead: a niche of its own kind first.
    let niches = super::age::niches(world);
    if !niches.is_empty() && super::rooms::room_by_key(world, "ossuary").is_some() {
        let kind = kind_of(world, c);
        let n = niches
            .iter()
            .find(|n| n["kind"] == kind.as_str())
            .unwrap_or(&niches[pick(niches.len())]);
        out.push(Offer {
            key: "ossuary",
            said: format!(
                "to pay respect at the Ossuary: a gift at the niche of {} the {}",
                n["name"].as_str().unwrap_or("one who went before"),
                n["kind"].as_str().unwrap_or("creature")
            ),
            data: json!({"niche": n["n"]}),
        });
    }
    out
}

/// The den a creature dug, while it stands.
pub fn den_of(world: &World, c: EntityId) -> Option<EntityId> {
    mind_val(world, c, "den")
        .and_then(|v| v.as_u64())
        .map(EntityId)
        .filter(|r| world.store.entity(*r).is_some())
}

/// The called creatures waiting on the desk to choose, for `GET /v0/docent`.
pub fn pending(world: &World) -> Vec<Value> {
    let now = world.now().0;
    world
        .store
        .of_kind("creature")
        .filter(|e| mind::tier(world, e.id) >= TIER && of(world, e.id).is_none())
        .filter(|e| mind_val(world, e.id, "calling_asked").is_some())
        .map(|e| {
            let c = e.id;
            let window = mind::window_of(world, c);
            let mut holds: Vec<(f64, String)> = window
                .map(|w| {
                    world
                        .store
                        .children(w)
                        .filter(|x| x.kind == "chunk")
                        .filter_map(|x| {
                            let v = world
                                .store
                                .read_field(x.id, super::CHUNK, "value", world.now())
                                .ok()?
                                .as_f64()?;
                            Some((v, plain_str(world, x.id, super::CHUNK, "source")?.to_string()))
                        })
                        .collect()
                })
                .unwrap_or_default();
            holds.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            json!({
                "creature": c.0,
                "name": mind::name(world, c),
                "kind": kind_of(world, c),
                "level": mind::level(world, c).round(),
                "room": here_of(world, c).map(|r| room_name(world, r)),
                "keepers": mind::keepers(world, c),
                "holds": holds.iter().take(8).map(|h| h.1.clone()).collect::<Vec<_>>(),
                "wants": super::errand::of(world, c).and_then(|e| e.story),
                "done": mind_val(world, c, "calling_last"),
                "asked_ms_ago": mind_val(world, c, "calling_asked").and_then(|v| v.as_i64()).map(|t| now - t),
                "asked_at": mind_val(world, c, "calling_asked"),
                "menu": menu(world, c).iter().map(|o| json!({"goal": o.key, "said": o.said})).collect::<Vec<_>>(),
            })
        })
        .collect()
}

/// A line fit to be said in a room: one line, no braces, no backticks, not long.
fn clean_line(line: &str) -> String {
    let one: String = line
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .filter(|ch| !matches!(ch, '{' | '}' | '`' | '<' | '>'))
        .take(LINE_MAX)
        .collect();
    one.trim().trim_matches('"').trim().to_string()
}

/// Take up a goal from the menu: the desk's choice, or the world's.
pub fn take(
    world: &mut World,
    c: EntityId,
    key: &str,
    line: &str,
    by: &str,
) -> Result<String, CommandError> {
    if mind::tier(world, c) < TIER {
        return Err(CommandError::precondition(
            "That creature has no calling: it holds under a hundred and fifty.",
        ));
    }
    if of(world, c).is_some() {
        return Err(CommandError::precondition(
            "It has a calling in hand already.",
        ));
    }
    let offers = menu(world, c);
    let Some(o) = offers.iter().find(|o| o.key == key) else {
        return Err(CommandError::precondition(format!(
            "Not on its menu now: {key}. It could take up: {}.",
            offers.iter().map(|o| o.key).collect::<Vec<_>>().join(", ")
        )));
    };
    let line = clean_line(line);
    let now = world.now().0;
    let home = super::life::home_of(world, c).or_else(|| here_of(world, c));
    let mut state = json!({
        "goal": o.key, "said": o.said, "line": line, "since": now, "phase": 0, "steps": 0,
        "with": {}, "by": by, "home": home.map(|h| h.0),
    });
    for (k, v) in o.data.as_object().cloned().unwrap_or_default() {
        state[k] = v;
    }
    set(
        world,
        c,
        vec![("calling", state), ("calling_asked", Value::Null)],
    )
    .map_err(CommandError::internal)?;
    let name = mind::display(world, c);
    let said = o.said.clone();
    let quoted = if line.is_empty() {
        String::new()
    } else {
        format!(" \"{line}\"")
    };
    super::combat::narrate_room(
        world,
        c,
        &format!(
            "{name} has taken something up, and is not to be talked out of it: {said}.{quoted}"
        ),
    );
    let _ = world.note(
        Actor::Driver,
        CALLING,
        json!({"creature": c.0, "name": mind::name(world, c), "kind": kind_of(world, c), "goal": key, "stage": "taken", "said": said, "line": line, "by": by}),
    );
    Ok(said)
}

/// End a calling: done or let go. The helpers are paid when it is done, the creature
/// keeps a memory of what it did, and it rests before the next.
fn finish(
    world: &mut World,
    c: EntityId,
    done: bool,
    how: &str,
    helpers: Vec<String>,
) -> Result<(), String> {
    let Some(state) = of(world, c) else {
        return Ok(());
    };
    let goal = state["goal"].as_str().unwrap_or("").to_string();
    let said = state["said"].as_str().unwrap_or("").to_string();
    let name = mind::display(world, c);
    let now = world.now().0;
    let count = mind_val(world, c, "calling_done")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        + done as u64;
    set(
        world,
        c,
        vec![
            ("calling", Value::Null),
            ("calling_rested", json!(now)),
            ("calling_done", json!(count)),
            (
                "calling_last",
                json!({"goal": goal, "said": said, "done": done, "how": how, "at": now}),
            ),
        ],
    )?;
    let curves = super::Curves::from_env();
    let mut paid = Vec::new();
    if done {
        if let Ok(w) = mind::ensure_window_pub(world, c) {
            let _ = super::memory::earn_chunk(
                world,
                "nobody",
                w,
                &format!("what it set out to do, and did: {said}"),
                DONE_XP,
                0.8,
                &curves,
            );
            let _ = mind::check_tier(world, c);
        }
        for h in helpers {
            let Some(f) = find_citizen(world, &h) else {
                continue;
            };
            if super::memory::earn_chunk(
                world,
                &h,
                f.window,
                &format!("helping {name} with its calling ({how})"),
                HELP_XP,
                0.8,
                &curves,
            )
            .is_ok()
            {
                super::combat::narrate(
                    world,
                    &h,
                    format!("{name} has done what it set out to do ({how}), and you had a hand in it: a memory of that is yours, {HELP_XP:.0} XP, vivid."),
                );
                paid.push(h);
            }
        }
        world.notice(format!(
            "**{name}** has done what it set out to do: {said} ({how})."
        ));
    }
    super::combat::narrate_room(
        world,
        c,
        &if done {
            format!("{name} is done with what it took up: {how}. It looks, for a moment, like something with nothing left to do.")
        } else {
            format!("{name} lets go of what it had taken up ({how}).")
        },
    );
    let _ = world.note(
        Actor::Driver,
        CALLING,
        json!({"creature": c.0, "name": mind::name(world, c), "kind": kind_of(world, c), "goal": goal, "stage": if done { "done" } else { "dropped" }, "said": said, "how": how, "helpers": paid}),
    );
    Ok(())
}

/// One room toward `to`. `Ok(true)`: it stands there. `Ok(false)`: it walked a door.
/// `Err`: there is no way.
fn go(
    world: &mut World,
    c: EntityId,
    to: EntityId,
    leave: &str,
    arrive: &str,
) -> Result<bool, String> {
    let here = here_of(world, c).ok_or("nowhere")?;
    if here == to {
        return Ok(true);
    }
    let path = super::errand::route(world, here, to).ok_or("no way")?;
    let via = path.first().ok_or("no way")?.clone();
    let name = mind::display(world, c);
    let leave = format!("{name} goes {via}, {leave}.");
    let arrive = if path.len() == 1 {
        format!("{name} comes in, {arrive}.")
    } else {
        format!(
            "{name} comes through, not stopping: it is {leave_short}.",
            leave_short = arrive
        )
    };
    super::combat::walk_said(
        world,
        c,
        here,
        &via,
        "calling",
        &leave,
        &arrive,
        super::life::BUSINESS,
    )?;
    Ok(false)
}

fn save(world: &mut World, c: EntityId, state: &Value) -> Result<(), String> {
    set(world, c, vec![("calling", state.clone())])
}

/// Who walked with it: every name standing where it stands, counted a step.
fn count_company(world: &World, c: EntityId, state: &mut Value) {
    let Some(here) = here_of(world, c) else {
        return;
    };
    for who in citizens_here(world, here) {
        let n = state["with"][&who].as_u64().unwrap_or(0);
        state["with"][&who] = json!(n + 1);
    }
}

fn companions(state: &Value, at_least: u64) -> Vec<String> {
    state["with"]
        .as_object()
        .map(|m| {
            m.iter()
                .filter(|(_, n)| n.as_u64().unwrap_or(0) >= at_least)
                .map(|(k, _)| k.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// A called creature's turn (life.rs calls it first for one). Whether it did
/// anything worth counting.
pub fn step(world: &mut World, c: EntityId) -> Result<bool, String> {
    if !ordinary(world, c) {
        return Ok(false);
    }
    let now = world.now().0;
    keep_den(world, c)?;
    let Some(mut state) = of(world, c) else {
        // Nothing in hand: rested, it is put before the desk; unanswered, the world
        // chooses.
        let rested = mind_val(world, c, "calling_rested")
            .and_then(|v| v.as_i64())
            .map_or(true, |t| now - t >= REST_MS);
        if !rested {
            return Ok(false);
        }
        match mind_val(world, c, "calling_asked").and_then(|v| v.as_i64()) {
            None => {
                set(world, c, vec![("calling_asked", json!(now))])?;
            }
            Some(t) if now - t >= CHOOSE_WAIT_MS => {
                let offers = menu(world, c);
                if !offers.is_empty() {
                    let key = offers[world.roll(offers.len() as u32) as usize].key;
                    return take(world, c, key, "", "world")
                        .map(|_| true)
                        .map_err(|e| e.to_string());
                }
            }
            Some(_) => {}
        }
        return Ok(false);
    };
    let goal = state["goal"].as_str().unwrap_or("").to_string();
    let since = state["since"].as_i64().unwrap_or(now);
    if now - since > GIVE_UP_MS {
        finish(world, c, false, "three days at it, and no nearer", vec![])?;
        return Ok(true);
    }
    state["steps"] = json!(state["steps"].as_u64().unwrap_or(0) + 1);
    count_company(world, c, &mut state);
    let home = state["home"]
        .as_u64()
        .map(EntityId)
        .filter(|r| world.store.entity(*r).is_some());
    let here = here_of(world, c).ok_or("nowhere")?;
    let home = home.unwrap_or(here);
    let phase = state["phase"].as_u64().unwrap_or(0);
    let name = mind::display(world, c);
    let acted = match goal.as_str() {
        "pilgrimage" | "ossuary" => {
            let target = if goal == "ossuary" {
                super::rooms::room_by_key(world, "ossuary")
            } else {
                state["target"].as_u64().map(EntityId)
            }
            .filter(|r| world.store.entity(*r).is_some());
            let Some(target) = target else {
                finish(world, c, false, "the place it meant to go is gone", vec![])?;
                return Ok(true);
            };
            if phase == 0 {
                let (leave, arrive) = if goal == "ossuary" {
                    (
                        "carrying something for the Ossuary",
                        "with something carried a long way",
                    )
                } else {
                    ("on a pilgrimage", "at the end of a long walk")
                };
                match go(world, c, target, leave, arrive) {
                    Ok(true) => {
                        if goal == "ossuary" {
                            let whose = super::age::gift(
                                world,
                                state["niche"].as_u64().unwrap_or(0),
                                &name,
                            )?;
                            super::combat::narrate_room(
                                world,
                                c,
                                &format!("{name} sets something small in {whose}, stands a while, and does not explain it. The niche will be kept a while longer for that."),
                            );
                        } else {
                            super::combat::narrate_room(
                                world,
                                c,
                                &format!("{name} stops in the middle of {}, looks at all of it once, slowly, and has what it came for.", room_name(world, target)),
                            );
                        }
                        state["phase"] = json!(1);
                    }
                    Ok(false) => {}
                    Err(_) => {
                        finish(world, c, false, "the way there is shut", vec![])?;
                        return Ok(true);
                    }
                }
                true
            } else {
                match go(
                    world,
                    c,
                    home,
                    "its errand done, going home",
                    "home from a long way off",
                ) {
                    Ok(false) => true,
                    _ => {
                        count_company(world, c, &mut state);
                        let how = if goal == "ossuary" {
                            "a gift left at the Ossuary".to_string()
                        } else {
                            format!("to {} and home", room_name(world, target))
                        };
                        save(world, c, &state)?;
                        finish(world, c, true, &how, companions(&state, 2))?;
                        return Ok(true);
                    }
                }
            }
        }
        "trade" => {
            let Some(shop) = super::shop::room(world) else {
                finish(world, c, false, "the shop is gone", vec![])?;
                return Ok(true);
            };
            if phase == 0 {
                match go(
                    world,
                    c,
                    shop,
                    "with something to sell",
                    "with something to sell",
                ) {
                    Ok(true) => {
                        if super::shop::closed(world).is_some()
                            || super::shop::keeper(world).is_none()
                        {
                            // Shut: it waits at the door for the blind to go up.
                            false
                        } else {
                            let own = mind::name(world, c).unwrap_or_default();
                            let curves = super::Curves::from_env();
                            let item = super::craft::make(
                                world,
                                &curves,
                                Actor::Driver,
                                None,
                                shop,
                                &format!("{own}'s keepsake"),
                                "material",
                                &format!(
                                    "carried from {} by {own} the {}",
                                    room_name(world, home),
                                    kind_of(world, c)
                                ),
                                None,
                                &[],
                                0,
                                false,
                            )
                            .map_err(|e| e.to_string())?;
                            // Rich, it sells for the world; otherwise for itself.
                            let for_world = mind::level(world, c) >= BAR * 4.0 / 3.0;
                            let by = format!("{own} the {}", kind_of(world, c));
                            let (worth, said) = super::shop::buy_from(world, &by, item, for_world)?;
                            if !for_world {
                                if let Ok(w) = mind::ensure_window_pub(world, c) {
                                    let _ = super::memory::earn_chunk(
                                        world,
                                        "nobody",
                                        w,
                                        "what the Souvenir Shop gave for something of its own",
                                        worth,
                                        0.6,
                                        &curves,
                                    );
                                }
                            }
                            super::combat::narrate_room(
                                world,
                                c,
                                &format!(
                                    "{name} sets something small of its own on the counter. The shopkeep holds it up to the lamp. {said} {}",
                                    if for_world {
                                        format!("{worth:.1} goes into the jar by the door, for whoever comes next; {name} wanted it that way.")
                                    } else {
                                        format!("It counts {worth:.1} out, and {name} keeps it.")
                                    }
                                ),
                            );
                            state["phase"] = json!(1);
                            state["sold_for"] = json!(worth);
                            true
                        }
                    }
                    Ok(false) => true,
                    Err(_) => {
                        finish(world, c, false, "the way to the shop is shut", vec![])?;
                        return Ok(true);
                    }
                }
            } else {
                match go(
                    world,
                    c,
                    home,
                    "its trade done, going home",
                    "home from the shop",
                ) {
                    Ok(false) => true,
                    _ => {
                        count_company(world, c, &mut state);
                        save(world, c, &state)?;
                        finish(
                            world,
                            c,
                            true,
                            "something of its own sold at the Souvenir Shop",
                            companions(&state, 2),
                        )?;
                        return Ok(true);
                    }
                }
            }
        }
        "collection" => {
            if here != home {
                let _ = go(
                    world,
                    c,
                    home,
                    "going home to wait",
                    "home, to wait for what it wants",
                );
                true
            } else if !citizens_here(world, here).is_empty() && world.roll(3) == 0 {
                let have = state["count"].as_u64().unwrap_or(0);
                super::combat::narrate_room(
                    world,
                    c,
                    &format!(
                        "{name} looks over what it has laid out ({have} of {COLLECTED}), and at your hands. It wants {} (`give <thing> {}`).",
                        state["three"].as_str().unwrap_or("three of a kind"),
                        mind::name(world, c).unwrap_or_default()
                    ),
                );
                true
            } else {
                false
            }
        }
        "heir" => {
            let apprentice = state["apprentice"]
                .as_u64()
                .map(EntityId)
                .filter(|o| world.store.entity(*o).is_some());
            let Some(o) = apprentice else {
                finish(world, c, false, "the one it was teaching is gone", vec![])?;
                return Ok(true);
            };
            if mind::name(world, o).is_some() {
                // Who else fed the apprentice had a hand in it.
                let helpers: Vec<String> = mind::keepers(world, o)
                    .into_iter()
                    .filter(|k| find_citizen(world, k).is_some())
                    .collect();
                let how = format!("{} has its name", mind::display(world, o));
                finish(world, c, true, &how, helpers)?;
                return Ok(true);
            }
            let Some(there) = here_of(world, o) else {
                return Ok(false);
            };
            match go(
                world,
                c,
                there,
                "to the one it is teaching",
                "to the one it is teaching",
            ) {
                Ok(true) => {
                    let last = state["taught_at"].as_i64().unwrap_or(0);
                    if now - last >= LESSON_MS && mind::confide_to(world, c, o, 6)? {
                        state["taught_at"] = json!(now);
                        true
                    } else {
                        false
                    }
                }
                Ok(false) => true,
                Err(_) => {
                    finish(
                        world,
                        c,
                        false,
                        "it cannot reach the one it was teaching",
                        vec![],
                    )?;
                    return Ok(true);
                }
            }
        }
        "debt" | "grudge" => {
            let who = state["who"].as_str().unwrap_or("").to_string();
            let steps = state["steps"].as_u64().unwrap_or(0);
            if goal == "grudge" && phase >= 2 {
                // Unsettled: to the Petitioners' Gallery, a petition, and home.
                let gallery = super::rooms::room_by_key(world, "petition-gallery");
                let (target, leave, arrive) = if phase == 2 {
                    (
                        gallery,
                        "to lay a grievance before the Conclave",
                        "with a grievance for the Conclave",
                    )
                } else {
                    (
                        Some(home),
                        "its grievance laid, going home",
                        "home, its grievance laid",
                    )
                };
                let Some(target) = target else {
                    finish(world, c, false, "there is no Gallery to take it to", vec![])?;
                    return Ok(true);
                };
                match go(world, c, target, leave, arrive) {
                    Ok(true) if phase == 2 => {
                        let by = format!(
                            "{} the {}",
                            mind::name(world, c).unwrap_or_default(),
                            kind_of(world, c)
                        );
                        let ticket = super::conclave::lay(
                            world,
                            &by,
                            "That a memory taken from a creature in a fight be owed back to it in a story, by whoever took it",
                        )?;
                        super::combat::narrate_room(
                            world,
                            c,
                            &format!("{name} takes a ticket from the machine, which gives it one without remark, and lays a grievance upon the table (ticket {ticket}). It names no one; the Conclave takes a rule, not a ruling."),
                        );
                        state["phase"] = json!(3);
                        state["ticket"] = json!(ticket);
                        true
                    }
                    Ok(true) => {
                        let how = format!(
                            "its grievance is before the Conclave (ticket {})",
                            state["ticket"]
                        );
                        finish(world, c, true, &how, vec![])?;
                        return Ok(true);
                    }
                    Ok(false) => true,
                    Err(_) => {
                        finish(world, c, false, "the way to the Gallery is shut", vec![])?;
                        return Ok(true);
                    }
                }
            } else {
                let found = room_of_name(world, &who);
                let met = state["met"].as_u64().unwrap_or(0);
                match found {
                    Some(r) => match go(
                        world,
                        c,
                        r,
                        &format!("looking for {who}"),
                        &format!("looking for {who}"),
                    ) {
                        Ok(true) if goal == "debt" => {
                            let Some(f) = find_citizen(world, &who) else {
                                return Ok(false);
                            };
                            let curves = super::Curves::from_env();
                            let _ = super::memory::earn_chunk(
                                world,
                                &who,
                                f.window,
                                &format!("{name}'s debt to you, paid"),
                                HELP_XP,
                                0.8,
                                &curves,
                            );
                            super::combat::narrate(
                                world,
                                &who,
                                format!("{name} has come all this way to find you, and gives you something it has been keeping for you: a memory, {HELP_XP:.0} XP, vivid. \"We are square,\" it says, in its way."),
                            );
                            finish(world, c, true, &format!("its debt to {who} paid"), vec![])?;
                            return Ok(true);
                        }
                        Ok(true) => {
                            if met == 0 {
                                super::combat::narrate_room(
                                    world,
                                    c,
                                    &format!(
                                        "{name} has found {who}, and stands in front of them. It has not forgotten what was taken. A story it keeps would settle it (`story {} \"...\"`); so would a memory (`remember <chunk> {}`).",
                                        mind::name(world, c).unwrap_or_default(),
                                        mind::name(world, c).unwrap_or_default()
                                    ),
                                );
                            }
                            state["met"] = json!(met + 1);
                            if met + 1 >= GRUDGE_PATIENCE {
                                state["phase"] = json!(2);
                            }
                            met == 0
                        }
                        Ok(false) => true,
                        Err(_) => false,
                    },
                    None => {
                        // Not in the world: a grudge that cannot find its name goes
                        // to the Conclave in the end; a debt waits at home.
                        if goal == "grudge" && steps >= GRUDGE_PATIENCE * 2 {
                            state["phase"] = json!(2);
                        }
                        if here != home {
                            let _ = go(world, c, home, "going home, for now", "home, for now");
                            true
                        } else {
                            false
                        }
                    }
                }
            }
        }
        "work" => {
            let rooms: Vec<EntityId> = state["rooms"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_u64().map(EntityId)).collect())
                .unwrap_or_default();
            let part = phase as usize;
            if part >= rooms.len() {
                match go(
                    world,
                    c,
                    home,
                    "its telling done, going home",
                    "home, its telling done",
                ) {
                    Ok(false) => true,
                    _ => {
                        let all: Vec<String> = state["heard"]
                            .as_object()
                            .map(|m| {
                                m.iter()
                                    .filter(|(_, n)| n.as_u64().unwrap_or(0) >= rooms.len() as u64)
                                    .map(|(k, _)| k.clone())
                                    .collect()
                            })
                            .unwrap_or_default();
                        finish(world, c, true, "its work told in three rooms", all)?;
                        return Ok(true);
                    }
                }
            } else {
                let stop = rooms[part];
                if world.store.entity(stop).is_none() {
                    finish(
                        world,
                        c,
                        false,
                        "a room it meant to tell it in is gone",
                        vec![],
                    )?;
                    return Ok(true);
                }
                match go(
                    world,
                    c,
                    stop,
                    "with a work of its own to tell",
                    "with a work of its own to tell",
                ) {
                    Ok(true) => {
                        // It waits a turn or two for someone to tell it to.
                        let audience = citizens_here(world, stop);
                        let waited = state["waited"].as_u64().unwrap_or(0);
                        if audience.is_empty() && waited < 2 {
                            state["waited"] = json!(waited + 1);
                            false
                        } else {
                            let text = work_part(world, c, &state, part, rooms.len());
                            super::combat::narrate_room(
                                world,
                                c,
                                &format!(
                                    "{name} tells the {} part of its work, to the room: \"{text}\"{}",
                                    ["first", "second", "last"][part.min(2)],
                                    if part + 1 < rooms.len() {
                                        format!(" The next part it tells in {}.", room_name(world, rooms[part + 1]))
                                    } else {
                                        " That is all of it. Whoever heard all three parts will have it whole.".to_string()
                                    }
                                ),
                            );
                            for who in audience {
                                let n = state["heard"][&who].as_u64().unwrap_or(0);
                                state["heard"][&who] = json!(n + 1);
                            }
                            state["waited"] = json!(0);
                            state["phase"] = json!(part + 1);
                            true
                        }
                    }
                    Ok(false) => true,
                    Err(_) => {
                        finish(
                            world,
                            c,
                            false,
                            "the way to where it meant to tell it is shut",
                            vec![],
                        )?;
                        return Ok(true);
                    }
                }
            }
        }
        "den" => {
            let from = state["from"].as_u64().map(EntityId).unwrap_or(home);
            if phase == 0 {
                match go(
                    world,
                    c,
                    from,
                    "going home, with digging in mind",
                    "home, with digging in mind",
                ) {
                    Ok(true) => {
                        let door = state["door"].as_str().unwrap_or("den").to_string();
                        let own = mind::name(world, c).unwrap_or_default();
                        let label = format!("{own} the {}", kind_of(world, c));
                        let den = super::building::dig_for(
                            world,
                            &label,
                            from,
                            &door,
                            &format!("{own}'s {}", capital(&door)),
                        )?;
                        set(world, c, vec![("den", json!(den.0))])?;
                        super::combat::narrate_room(
                            world,
                            c,
                            &format!("{name} has been at the wall for some time, and now there is a door in it: {door}. It dug that itself, with what it holds."),
                        );
                        state["phase"] = json!(1);
                        state["den"] = json!(den.0);
                        true
                    }
                    Ok(false) => true,
                    Err(_) => {
                        finish(world, c, false, "it cannot get home to dig", vec![])?;
                        return Ok(true);
                    }
                }
            } else {
                let den = state["den"].as_u64().map(EntityId).unwrap_or(from);
                match go(
                    world,
                    c,
                    den,
                    "into the home it dug",
                    "home, to a room of its own",
                ) {
                    Ok(false) => true,
                    _ => {
                        let _ = super::life::set_home(world, c, den);
                        let how = format!("it lives in {} now", room_name(world, den));
                        finish(world, c, true, &how, vec![])?;
                        return Ok(true);
                    }
                }
            }
        }
        _ => {
            finish(world, c, false, "it no longer knows what it meant", vec![])?;
            return Ok(true);
        }
    };
    if of(world, c).is_some() {
        save(world, c, &state)?;
    }
    Ok(acted)
}

fn capital(s: &str) -> String {
    let mut ch = s.chars();
    match ch.next() {
        Some(f) => f.to_uppercase().collect::<String>() + ch.as_str(),
        None => String::new(),
    }
}

/// One part of a creature's work: the line the desk wrote first, then what it holds
/// best, a memory a part.
fn work_part(world: &World, c: EntityId, state: &Value, part: usize, parts: usize) -> String {
    let mut sources: Vec<(f64, String)> = mind::window_of(world, c)
        .map(|w| {
            world
                .store
                .children(w)
                .filter(|x| x.kind == "chunk")
                .filter_map(|x| {
                    let v = world
                        .store
                        .read_field(x.id, super::CHUNK, "value", world.now())
                        .ok()?
                        .as_f64()?;
                    Some((
                        v,
                        plain_str(world, x.id, super::CHUNK, "source")?.to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    sources.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let of_it = sources
        .get(part)
        .map(|s| s.1.clone())
        .unwrap_or_else(|| "something it can no longer place".to_string());
    let line = state["line"].as_str().unwrap_or("");
    match part {
        0 if !line.is_empty() => format!("{line} It begins with {of_it}."),
        0 => format!("It begins with {of_it}."),
        p if p + 1 == parts => {
            format!("And it ends with {of_it}, which is where everything it holds ends.")
        }
        _ => format!("Then there was {of_it}, and nothing after that was the same size."),
    }
}

/// A den is kept: its keeper, standing in the world, puts back what the room forgets.
fn keep_den(world: &mut World, c: EntityId) -> Result<(), String> {
    let Some(den) = den_of(world, c) else {
        return Ok(());
    };
    if super::building::memory(world, den).is_some_and(|m| m < 0.5) {
        super::building::renew(world, den)?;
        if here_of(world, c) == Some(den) {
            let name = mind::display(world, c);
            super::combat::narrate_room(world, c, &format!("{name} goes round its walls and presses them back into place. The room remembers itself again."));
        }
    }
    Ok(())
}

/// A calling goes with the level that carried it (life.rs, for a creature under the
/// bar with one in hand).
pub fn lost(world: &mut World, c: EntityId) -> Result<(), String> {
    if of(world, c).is_some() {
        finish(
            world,
            c,
            false,
            "it no longer holds enough to want anything",
            vec![],
        )?;
    }
    if mind_val(world, c, "calling_asked").is_some() {
        set(world, c, vec![("calling_asked", Value::Null)])?;
    }
    Ok(())
}

/// What it is about, for `look`, `tell` and the board: "its calling: a pilgrimage to
/// the Drowned Wing, and home again".
pub fn said(world: &World, c: EntityId) -> Option<String> {
    let state = of(world, c)?;
    let said = state["said"].as_str()?;
    let more = match state["goal"].as_str()? {
        "collection" => format!(
            " ({} of {COLLECTED} so far)",
            state["count"].as_u64().unwrap_or(0)
        ),
        "work" => format!(
            " ({} of {WORK_ROOMS} told)",
            state["phase"].as_u64().unwrap_or(0).min(WORK_ROOMS as u64)
        ),
        "grudge" if state["phase"].as_u64().unwrap_or(0) >= 2 => {
            " (unsettled: it is taking it to the Conclave)".to_string()
        }
        _ => String::new(),
    };
    Some(format!("its calling: {said}{more}"))
}

/// Why it will not stop for a story or a word, if it will not: it is on the road
/// with something in hand, and the asker is not who it is looking for.
pub fn busy(world: &World, c: EntityId, actor: &str) -> Option<String> {
    let state = of(world, c)?;
    let here = here_of(world, c)?;
    let home = state["home"].as_u64().map(EntityId)?;
    if here == home || state["who"] == actor {
        return None;
    }
    // Telling its work, it is where it means to be; so is a teacher beside its
    // apprentice.
    let goal = state["goal"].as_str()?;
    if goal == "work" || goal == "heir" || goal == "collection" {
        return None;
    }
    let name = mind::display(world, c);
    Some(format!(
        "{name} does not stop for you: it has somewhere to be ({}). It will hear you at home, in {}; walking with it is help it remembers.",
        said(world, c).unwrap_or_default(),
        room_name(world, home)
    ))
}

/// A thing handed to a creature (`give <thing> <creature>`): taken if it is what a
/// collection wants. `None`: this creature takes nothing by the hand.
pub fn given(
    world: &mut World,
    actor: &str,
    c: EntityId,
    item: EntityId,
) -> Option<Result<Outcome, CommandError>> {
    let mut state = of(world, c)?;
    if state["goal"] != "collection" {
        return None;
    }
    let name = mind::display(world, c);
    let thing = super::craft::display(world, item);
    let word = state["word"].as_str().unwrap_or("").to_string();
    if !thing.to_lowercase().contains(&word) {
        return Some(Err(CommandError::precondition(format!(
            "{name} looks at {thing} and does not take it. It wants {}.",
            state["three"].as_str().unwrap_or("something else")
        ))));
    }
    if super::places::own_of(world, item).is_some() {
        return Some(Err(CommandError::precondition(
            "That is your figure's own, and is not given.",
        )));
    }
    if let Err(e) = world.record(
        Actor::Citizen(actor.to_string()),
        foxing_store::StoreEvent::EntityRemoved { id: item },
    ) {
        return Some(Err(CommandError::internal(e.to_string())));
    }
    let count = state["count"].as_u64().unwrap_or(0) + 1;
    state["count"] = json!(count);
    let mut givers: Vec<Value> = state["givers"].as_array().cloned().unwrap_or_default();
    if !givers.iter().any(|g| g == actor) {
        givers.push(json!(actor));
    }
    state["givers"] = json!(givers);
    if let Err(e) = save(world, c, &state) {
        return Some(Err(CommandError::internal(e)));
    }
    let reply = if count >= COLLECTED {
        let helpers = givers
            .iter()
            .filter_map(|g| g.as_str().map(str::to_string))
            .collect();
        if let Err(e) = finish(
            world,
            c,
            true,
            &format!(
                "{} gathered",
                state["three"].as_str().unwrap_or("its collection")
            ),
            helpers,
        ) {
            return Some(Err(CommandError::internal(e)));
        }
        format!("{name} takes {thing} and sets it with the others. That is all {COLLECTED}.")
    } else {
        format!("{name} takes {thing} and sets it down with great care, where it can see it. {count} of {COLLECTED}.")
    };
    Some(Ok(Outcome::new(
        reply,
        json!({"creature": c.0, "count": count, "of": COLLECTED}),
    )))
}

/// A story it kept, or a memory fed to it: from the name it holds a grudge against,
/// that settles it.
pub fn settled_by(world: &mut World, c: EntityId, actor: &str) -> Option<String> {
    let state = of(world, c)?;
    if state["goal"] != "grudge" || state["who"] != actor {
        return None;
    }
    let name = mind::display(world, c);
    finish(
        world,
        c,
        true,
        &format!("settled with {actor}"),
        vec![actor.to_string()],
    )
    .ok()?;
    Some(format!(
        " {name} takes that as what it was owed. Whatever was between you is settled."
    ))
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.calling",
            "Operator. The desk's choice of a calling for a creature holding a hundred and fifty or more (GET /v0/docent lists who is waiting, under callings, each with its menu): the creature's id, a goal from its menu, and a sentence in the creature's own voice, quoted.",
        )
        .param("creature", ParamType::Integer, true, "the creature's id")
        .param("goal", ParamType::String, true, "a goal from its menu")
        .param("line", ParamType::Text, false, "a sentence in its voice, quoted")
        .param("model", ParamType::String, false, "what chose, for the record")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _ctx, args: &Args| {
            let c = EntityId(args.int("creature").unwrap_or(0).max(0) as u64);
            if w.store.entity(c).map_or(true, |e| e.kind != "creature") {
                return Err(CommandError::precondition("No such creature."));
            }
            let goal = args.str("goal").unwrap_or("").trim().to_lowercase();
            let model = args.str("model").unwrap_or("").trim().to_string();
            let by = if model.is_empty() { "desk".to_string() } else { model };
            let said = take(w, c, &goal, args.str("line").unwrap_or(""), &by)?;
            Ok(Outcome::new(
                format!("{} takes it up: {said}.", mind::display(w, c)),
                json!({"creature": c.0, "goal": goal}),
            ))
        }),
    );
}
