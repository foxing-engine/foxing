//! What is yours in the world, and what the named creatures want: two reads for a
//! person at a keyboard, who otherwise learns both by walking.
//!
//! `wants`: every named creature, where it stands, what it wants (a story, a word to
//! carry) and the way there over the rooms you remember. The errand system is the
//! world's quest board; this is the board.
//!
//! `mine`: your footprint (rooms dug, creatures kept, things made or named, lines in
//! books), each with what keeps it alive, and what happened to any of it since you
//! were last here: who visited your room, who wrote in your book, what became of a
//! creature you keep. The reason to come back, said plainly.

use super::craft::{self, ITEM};
use super::{internal, me, mind, plain_str, rooms, Curves, CREATURE, ROOM};
use crate::world::World;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::EntityId;
use serde_json::{json, Value};

/// How many digest lines `mine` says at most; the rest are counted.
const DIGEST_LINES: usize = 12;

fn room_name(world: &World, room: Option<EntityId>) -> String {
    room.and_then(|r| plain_str(world, r, ROOM, "name"))
        .unwrap_or("nowhere")
        .to_string()
}

fn way_said(
    world: &World,
    citizen: EntityId,
    here: EntityId,
    to: Option<EntityId>,
    hl: f64,
) -> (Option<Vec<String>>, String) {
    let Some(to) = to else {
        return (None, String::new());
    };
    match super::atlas::way(world, citizen, here, to, hl) {
        Some(p) if p.is_empty() => (Some(p), " (here)".to_string()),
        Some(p) => {
            let said = format!(
                " ({} step{}: {})",
                p.len(),
                if p.len() == 1 { "" } else { "s" },
                p.join(", ")
            );
            (Some(p), said)
        }
        None => (None, " (no way you remember)".to_string()),
    }
}

/// The named creatures and their wants, as `wants` lists them.
pub fn wants(world: &World, actor: &str, hl: f64) -> Result<Value, CommandError> {
    let f = super::find_citizen(world, actor).ok_or_else(|| {
        CommandError::precondition(format!("{actor} has not arrived; log in first."))
    })?;
    let here = rooms::room_of(world, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
    let mut rows: Vec<Value> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let scarcity = mind::scarcity(world);
    for e in world.store.entities().filter(|e| e.kind == "creature") {
        let Some(name) = mind::name(world, e.id) else {
            continue;
        };
        let kind = plain_str(world, e.id, CREATURE, "name")
            .unwrap_or("?")
            .to_string();
        let room = e.parent;
        let where_ = room_name(world, room);
        let (way, way_said) = way_said(world, f.citizen, here, room, hl);
        let errand = super::errand::of(world, e.id);
        let story = errand.as_ref().and_then(|x| x.story.clone());
        let word = errand.as_ref().and_then(|x| x.word.clone());
        let word_to = errand.as_ref().and_then(|x| x.word_to.clone());
        let knows_you =
            mind::regard(world, e.id, actor).is_some_and(|r| !r.contains("does not know you"));
        let mut want_said: Vec<String> = Vec::new();
        let told = story.is_some() && super::exchange::told_this_hour(world, actor, e.id);
        if let Some(s) = &story {
            want_said.push(if told {
                format!("a story about {s} (you told it one this hour; another from you pays after the hour)")
            } else {
                format!(
                    "a story about {s} (pays double{})",
                    if scarcity > 1.0 {
                        format!(", and {scarcity:.1}× while names are few")
                    } else {
                        String::new()
                    }
                )
            });
        }
        if let (Some(w), Some(to)) = (&word, &word_to) {
            want_said.push(format!("the word \u{201c}{w}\u{201d} carried to {to}"));
        }
        if want_said.is_empty() {
            continue;
        }
        lines.push(format!(
            "{name} the {kind}, in {where_}{way_said}, wants {}{}.",
            want_said.join("; and "),
            if knows_you { "; it knows you" } else { "" }
        ));
        rows.push(json!({
            "creature": e.id.0, "name": name, "kind": kind, "room": room.map(|r| r.0), "room_name": where_,
            "way": way, "story": story, "word": word, "word_to": word_to, "knows_you": knows_you, "told_this_hour": told,
            "level": mind::level(world, e.id),
        }));
    }
    let summary = if lines.is_empty() {
        "No named creature wants anything just now. Feed an unnamed one to sixty and it will."
            .to_string()
    } else {
        format!("What the named want:\n{}", lines.join("\n"))
    };
    Ok(json!({"summary": summary, "wants": rows, "scarcity": scarcity}))
}

/// Your footprint and what happened to it since you were last here.
pub fn mine(world: &World, actor: &str, hl: f64) -> Result<Value, CommandError> {
    let f = super::find_citizen(world, actor).ok_or_else(|| {
        CommandError::precondition(format!("{actor} has not arrived; log in first."))
    })?;
    let here = rooms::room_of(world, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
    let now = world.now();
    let mut lines: Vec<String> = Vec::new();

    // Rooms: dug by you, or kept by you.
    let mut rooms_out: Vec<Value> = Vec::new();
    let mut room_ids: Vec<EntityId> = Vec::new();
    for e in world.store.entities().filter(|e| e.kind == "room") {
        let dug = e.owner.as_deref() == Some(actor);
        let kept = super::building::kept_by(world, e.id)
            .iter()
            .any(|k| k == actor);
        if !dug && !kept {
            continue;
        }
        room_ids.push(e.id);
        let name = plain_str(world, e.id, ROOM, "name")
            .unwrap_or("?")
            .to_string();
        let memory = world
            .store
            .read_field(e.id, ROOM, "memory", now)
            .ok()
            .and_then(|v| v.as_f64());
        let half_life_ms = super::plain_num(world, e.id, ROOM, "half_life_ms").unwrap_or(0.0);
        let (way, way_said) = way_said(world, f.citizen, here, Some(e.id), hl);
        let fixtures = craft::fixtures_in(world, e.id).len();
        let state = match memory {
            Some(m) if m >= 0.5 => "remembered",
            Some(m) if m >= 0.1 => "dimming",
            Some(_) => "nearly forgotten",
            None => "standing",
        };
        lines.push(format!(
            "{name}{way_said}: {}, {state}{}{}.",
            if dug { "yours" } else { "kept by you" },
            if half_life_ms > 0.0 {
                format!(
                    ", halving every {}",
                    super::building::humane(foxing_time::Duration(half_life_ms as i64))
                )
            } else {
                String::new()
            },
            if fixtures > 0 {
                format!(
                    ", {fixtures} thing{} standing in it",
                    if fixtures == 1 { "" } else { "s" }
                )
            } else {
                String::new()
            }
        ));
        rooms_out.push(json!({"room": e.id.0, "name": name, "dug": dug, "kept": kept, "memory": memory, "half_life_ms": half_life_ms, "way": way, "fixtures": fixtures}));
    }

    // Creatures you keep (fed), by name or kind.
    let mut creatures_out: Vec<Value> = Vec::new();
    let mut creature_ids: Vec<EntityId> = Vec::new();
    for e in world.store.entities().filter(|e| e.kind == "creature") {
        if !mind::keepers(world, e.id).iter().any(|k| k == actor) {
            continue;
        }
        creature_ids.push(e.id);
        let kind = plain_str(world, e.id, CREATURE, "name")
            .unwrap_or("?")
            .to_string();
        let name = mind::name(world, e.id);
        let level = mind::level(world, e.id);
        let where_ = room_name(world, e.parent);
        let (way, way_said) = way_said(world, f.citizen, here, e.parent, hl);
        lines.push(format!(
            "{} in {where_}{way_said}, holding {level:.0}{}.",
            match &name {
                Some(n) => format!("{n} the {kind}"),
                None => format!("an unnamed {kind}"),
            },
            match &name {
                Some(_) if level < mind::FAILING_BELOW => ", its name failing",
                Some(_) => "",
                None => " (a name at 60)",
            }
        ));
        creatures_out.push(json!({"creature": e.id.0, "name": name, "kind": kind, "level": level, "room": e.parent.map(|r| r.0), "room_name": where_, "way": way}));
    }

    // Things made by you, wherever they are, and books you wrote in.
    let mut things_out: Vec<Value> = Vec::new();
    let mut thing_ids: Vec<EntityId> = Vec::new();
    let sig = format!(" \u{2014} {actor}");
    for e in world.store.entities().filter(|e| e.kind == "item") {
        let made = plain_str(world, e.id, ITEM, "made_by") == Some(actor);
        let text = plain_str(world, e.id, ITEM, "text").unwrap_or("");
        let my_lines = text
            .split(" | ")
            .filter(|l| l.trim_end().ends_with(&sig))
            .count();
        if !made && my_lines == 0 {
            continue;
        }
        thing_ids.push(e.id);
        let name = craft::display(world, e.id);
        let fixed = craft::is_fixed(world, e.id);
        let holder = e.parent.and_then(|p| world.store.entity(p));
        let where_ = match holder {
            Some(h) if h.kind == "room" => plain_str(world, h.id, ROOM, "name")
                .unwrap_or("?")
                .to_string(),
            Some(h) if h.kind == "citizen" => {
                format!("carried by {}", h.owner.clone().unwrap_or_default())
            }
            _ => "somewhere".to_string(),
        };
        lines.push(format!(
            "{name}, {}, {where_}{}.",
            if made {
                if fixed {
                    "made by you, standing"
                } else {
                    "made by you"
                }
            } else {
                "a book"
            },
            if my_lines > 0 {
                format!(
                    ", {my_lines} line{} of yours in it",
                    if my_lines == 1 { "" } else { "s" }
                )
            } else {
                String::new()
            }
        ));
        things_out.push(json!({"item": e.id.0, "name": name, "made": made, "fixed": fixed, "lines": my_lines, "where": where_, "room": holder.filter(|h| h.kind == "room").map(|h| h.id.0)}));
    }

    // Since you were last here: the public record touching any of it, after your
    // previous arrival (the one before this visit's).
    let recent = world.recent_events(4000);
    let arrivals: Vec<u64> = recent
        .iter()
        .filter(|e| e.kind == crate::world::ARRIVED && e.payload["actor"] == actor)
        .map(|e| e.seq.0)
        .collect();
    // A first visit has no "last here": the digest is since this arrival, said so.
    let first_visit = arrivals.len() < 2;
    let since = if first_visit {
        arrivals.first().copied().unwrap_or(0)
    } else {
        arrivals[arrivals.len() - 2]
    };
    let since_said = if first_visit {
        "Since you arrived"
    } else {
        "Since you were last here"
    };
    let mine_room = |v: &Value| {
        v.as_u64()
            .is_some_and(|id| room_ids.iter().any(|r| r.0 == id))
    };
    let mine_creature = |v: &Value| {
        v.as_u64()
            .is_some_and(|id| creature_ids.iter().any(|c| c.0 == id))
    };
    let mine_thing = |v: &Value| {
        v.as_u64()
            .is_some_and(|id| thing_ids.iter().any(|c| c.0 == id))
    };
    let mut digest: Vec<String> = Vec::new();
    for e in recent.iter().filter(|e| e.seq.0 > since) {
        let p = &e.payload;
        let by = p["by"]
            .as_str()
            .or(p["from"].as_str())
            .or(p["actor"].as_str())
            .unwrap_or("someone");
        if by == actor {
            continue;
        }
        let s = |k: &str| p[k].as_str().unwrap_or("?").to_string();
        let line = match e.kind.as_str() {
            super::building::APPENDED if p["kind"] == "room" && mine_room(&p["target"]) => {
                format!("{by} wrote in {}", s("name"))
            }
            super::building::APPENDED if p["kind"] == "book" && mine_thing(&p["target"]) => {
                format!("{by} wrote a line in {}", s("name"))
            }
            super::building::REMEMBERED if mine_room(&p["room"]) => {
                format!("{by} kept {} longer", s("name"))
            }
            super::bindery::BOUND if mine_room(&p["target"]) || mine_thing(&p["target"]) => {
                format!("the Bindery bound {}", s("name"))
            }
            super::bindery::UNBOUND if mine_room(&p["target"]) || mine_thing(&p["target"]) => {
                format!("{by} unbound {}", s("name"))
            }
            super::building::DUG if mine_room(&p["from"]) => {
                format!("{by} dug {} from your room", s("name"))
            }
            super::mind::TAKEN if mine_creature(&p["creature"]) => format!(
                "{by} felled {} and took a memory of {} from it (it is back, shaken)",
                p["name"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| format!(
                        "the {} you keep",
                        p["kind"].as_str().unwrap_or("creature")
                    )),
                p["worth"]
                    .as_f64()
                    .map(|w| format!("{w:.0}"))
                    .unwrap_or_else(|| "something".into())
            ),
            super::exchange::STORY if mine_creature(&p["creature"]) => {
                format!("{by} told {} a story", s("name"))
            }
            super::mind::TIER if mine_creature(&p["creature"]) => match p["name"].as_str() {
                Some(n) => format!("{n} the {} is named", s("kind")),
                None => format!("the {} lost its name", s("kind")),
            },
            super::mind::CONFIDED
                if mine_creature(&p["teller"]) || mine_creature(&p["listener"]) =>
            {
                format!("{} confided in a {}", s("teller_name"), s("listener_kind"))
            }
            super::craft::TOOK if mine_thing(&p["item"]) => format!("{by} took {}", s("name")),
            super::craft::DROPPED if mine_thing(&p["item"]) => {
                format!("{by} set down {}", s("name"))
            }
            "store.entity_moved" if mine_room(&p["parent"]) => {
                // A citizen came into your room.
                match p["id"]
                    .as_u64()
                    .and_then(|id| world.store.entity(EntityId(id)))
                    .filter(|c| c.kind == "citizen")
                    .and_then(|c| c.owner.clone())
                {
                    Some(who) if who != actor => format!(
                        "{who} came into {}",
                        room_name(world, p["parent"].as_u64().map(EntityId))
                    ),
                    _ => continue,
                }
            }
            _ => continue,
        };
        if digest.last() != Some(&line) {
            digest.push(line);
        }
    }
    let shown: Vec<String> = digest
        .iter()
        .rev()
        .take(DIGEST_LINES)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let more = digest.len().saturating_sub(DIGEST_LINES);

    // The night's falls: what the next one costs.
    let falls = super::death::recent_falls(world, f.citizen);
    if falls > 0 {
        lines.push(format!(
            "You have fallen {} tonight; the next fall costs {} of the first.",
            match falls {
                1 => "once".to_string(),
                2 => "twice".to_string(),
                n => format!("{n} times"),
            },
            match falls {
                1 => "half",
                2 => "a quarter",
                3 => "an eighth",
                _ => "almost nothing",
            }
        ));
    }
    let summary = if lines.is_empty() {
        "Nothing in the world is yours yet: dig a room, feed a creature to sixty, make a thing, write in a book. Each stays after you go.".to_string()
    } else {
        format!(
            "What remembers you:\n{}{}",
            lines.join("\n"),
            if shown.is_empty() {
                if since > 0 {
                    format!("\n{since_said}: nothing touched any of it.")
                } else {
                    String::new()
                }
            } else {
                format!(
                    "\n{since_said}: {}{}.",
                    shown.join("; "),
                    if more > 0 {
                        format!("; and {more} more")
                    } else {
                        String::new()
                    }
                )
            }
        )
    };
    Ok(
        json!({"summary": summary, "rooms": rooms_out, "creatures": creatures_out, "things": things_out, "since": since, "digest": digest, "falls_tonight": falls}),
    )
}

pub fn register(world: &mut World, curves: Curves) {
    let hl = curves.active_half_life.0 as f64 * super::atlas::ATLAS_HALF_LIVES;
    world.register(
        CommandSpec::new(
            "disremembered.wants",
            "What every named creature wants (a story, a word carried), where it is, and the way there over the rooms you remember: a read.",
        )
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(move |w: &mut World, ctx, _args: &Args| {
            let v = wants(w, &ctx.actor, hl)?;
            let summary = v["summary"].as_str().unwrap_or("").to_string();
            Ok(Outcome::new(summary, v))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.mine",
            "What of yours is in the world (rooms, creatures you keep, things you made, lines in books), how each is faring, and what happened to any of it since you were last here: a read.",
        )
        .param("about", ParamType::String, false, "unused; for the grammar")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(move |w: &mut World, ctx, _args: &Args| {
            let _ = me(w, ctx)?;
            let v = mine(w, &ctx.actor, hl)?;
            let summary = v["summary"].as_str().unwrap_or("").to_string();
            Ok(Outcome::new(summary, v))
        }),
    );
}
