//! World-level commands: the freeze, the log, the clocks.

use super::NARRATIVE;
use super::{account_scope, internal};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{ClockId, StoreEvent, WORLD_CLOCK};
use serde_json::json;

/// One event as a line a reader can follow: who said what, who came and went, what was
/// kept and taken; the rest by kind.
/// A moment as a clock says it, UTC: "18:30".
pub(crate) fn clock(at: i64) -> String {
    let secs = at.div_euclid(1000);
    format!(
        "{:02}:{:02}",
        secs.rem_euclid(86_400) / 3600,
        secs.rem_euclid(3600) / 60
    )
}

pub(crate) fn line_for(e: &foxing_log::Event) -> String {
    let p = &e.payload;
    let s = |k: &str| p[k].as_str().unwrap_or("?").to_string();
    match e.kind.as_str() {
        "disremembered.say" => match p["to"].as_str() {
            Some(to) => format!("{} says to {to}, \"{}\"", s("from"), s("text")),
            None => format!("{} says, \"{}\"", s("from"), s("text")),
        },
        // Only the sender, the one it is for, and the operator can see a tell at all,
        // so the words are shown: the log is how a citizen re-reads what was left.
        "disremembered.tell" => {
            format!("{} left word for {}: \"{}\"", s("from"), s("to"), s("text"))
        }
        FEEDBACK => format!("{} would change: \"{}\"", s("from"), s("text")),
        "disremembered.died" => match p["lost"].as_array().map(|l| l.len()).unwrap_or(0) {
            0 => format!(
                "{} went down and woke at the Threshold with everything",
                s("citizen")
            ),
            n => format!(
                "{} went down and lost {n} chunk{} to it",
                s("citizen"),
                if n == 1 { "" } else { "s" }
            ),
        },
        "disremembered.errand_turned" => format!("{}'s want turned: {}", s("name"), s("why")),
        "disremembered.errand_done" => match p["to"].as_str() {
            Some(t) => format!("{} carried {}'s word to {t}", s("from"), s("name")),
            None => format!("{} told {} the story it wanted", s("from"), s("name")),
        },
        "disremembered.born" => format!(
            "a {} was born in {}, of {} and {}, carried by {}{}",
            s("kind"),
            s("room_name"),
            p["parents"][0].as_str().unwrap_or("?"),
            p["parents"][1].as_str().unwrap_or("?"),
            s("carrier"),
            p["mutation"]
                .as_str()
                .map(|m| format!(": {m}"))
                .unwrap_or_default()
        ),
        "disremembered.faded" => match p["owner"].as_str() {
            Some(o) => format!("{o} forgot the memory of {}", s("source")),
            None => format!("a creature forgot the memory of {}", s("source")),
        },
        "disremembered.story" => format!(
            "{} told {} the {} a story of {} words",
            s("from"),
            s("name"),
            s("kind"),
            p["words"].as_u64().unwrap_or(0)
        ),
        "citizen.arrived" => format!("{} arrived via {}", s("actor"), s("origin")),
        "disremembered.departed" => format!("{} went quiet", s("actor")),
        "disremembered.forgotten" => format!("the world forgot {}; the name is free", s("name")),
        "disremembered.remembered" => match p["creature"].as_u64() {
            Some(_) => format!(
                "{} gave the memory of {} to {}",
                s("by"),
                s("with"),
                match p["name"].as_str() {
                    Some(n) => format!("{n} the {}", s("kind")),
                    None => format!("the {}", s("kind")),
                }
            ),
            None => format!(
                "{} kept {} with the memory of {}",
                s("by"),
                s("name"),
                s("with")
            ),
        },
        "disremembered.tier" => match p["name"].as_str() {
            Some(n) => format!("the {} is {n} now", s("kind")),
            None => format!("the {} has forgotten its name", s("kind")),
        },
        "disremembered.taken" => format!(
            "{} took the memory of {} from {}",
            s("by"),
            s("with"),
            p["name"].as_str().unwrap_or(&s("kind"))
        ),
        "disremembered.wandered" => format!("a {} wandered in", s("kind")),
        "disremembered.confided" => format!(
            "{} the {} told {} a story, {} named left",
            s("teller_name"),
            s("kind"),
            match p["listener_name"].as_str() {
                Some(n) => format!("{n} the {}", s("listener_kind")),
                None => format!("the {}", s("listener_kind")),
            },
            p["named"].as_u64().unwrap_or(0)
        ),
        "disremembered.razed" => format!("the operator razed {}", s("name")),
        "disremembered.dug" => format!("{} dug {}", s("by"), s("name")),
        "disremembered.clocktower" => s("said").to_string(),
        "disremembered.statute" | "disremembered.conclave_sat" => s("said").to_string(),
        "disremembered.trick" => match p["trick"].as_str() {
            Some("venom") => format!("a {} bit {}; the venom works", s("kind"), s("citizen")),
            Some("ushered") => format!(
                "an usher showed {} out, to {}",
                s("citizen"),
                s("room_name")
            ),
            Some("forget") => format!(
                "a {} turned a ray of forgetfulness on {}: the memory of {} is thinner",
                s("kind"),
                s("citizen"),
                s("source")
            ),
            Some("entered") => format!(
                "an engrosser entered {}'s memory of {} into the minutes",
                s("citizen"),
                s("source")
            ),
            Some("below") => format!("the tipstaff showed {} below the bar", s("citizen")),
            _ => format!("a {} did something to {}", s("kind"), s("citizen")),
        },
        "disremembered.left" => format!("a {} left {} behind", s("kind"), s("name")),
        "disremembered.took" => format!("{} took {}", s("by"), s("name")),
        "disremembered.dropped" => format!("{} set down {}", s("by"), s("name")),
        "disremembered.made" => format!(
            "{} made {} of {}",
            s("by"),
            s("name"),
            p["of"]
                .as_array()
                .map(|a| a
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" and "))
                .unwrap_or_default()
        ),
        "gate.message" => {
            // As prose, not as the statements themselves: a newcomer read other
            // players' DO lines in the record as commands addressed to them.
            let text = s("text");
            let from = s("from");
            let did: Vec<String> = text
                .lines()
                .filter_map(|l| {
                    let l = l.trim();
                    if let Some(rest) = l.strip_prefix("[DO]") {
                        let rest = rest.trim().trim_end_matches('.');
                        let rest = rest
                            .split_once(" MUST ")
                            .or_else(|| rest.split_once(" MAY "))
                            .or_else(|| rest.split_once(" SHOULD "))
                            .map(|(_, verb)| verb)
                            .unwrap_or(rest);
                        // What was said is its own line in the record; here, that it was.
                        let rest = match rest.find('"') {
                            Some(i) => format!("{}\u{2026}", rest[..i].trim_end()),
                            None => rest.to_string(),
                        };
                        Some(rest)
                    } else {
                        l.strip_prefix("[ASK]")
                            .map(|q| format!("asked {}", q.trim()))
                    }
                })
                .collect();
            let door = crate::world::World::door_label(&s("via")).unwrap_or("through the gate");
            let shown = did.iter().take(3).cloned().collect::<Vec<_>>().join("; ");
            let more = did.len().saturating_sub(3);
            if did.is_empty() {
                format!("{from}, {door}, sent a message")
            } else if more > 0 {
                format!("{from}, {door}: {shown}; and {more} more")
            } else {
                format!("{from}, {door}: {shown}")
            }
        }
        "gate.reply" => format!("the world replied to {}", s("to")),
        // What the world said to one citizen: shown to them as words, not as
        // `disremembered.narrative {"kind":"driver"}` (a visitor's `log 12 events`
        // had four of those among the prose).
        "disremembered.narrative" => match p["text"].as_str() {
            Some(t) => format!("the world said to {}: {t}", s("to")),
            None => format!("the world said something to {}", s("to")),
        },
        k => format!(
            "{k} {}",
            serde_json::to_string(&e.actor).unwrap_or_default()
        ),
    }
}

/// What a visitor would change, said in-world: `{from, text}`. Public, read by the
/// operator; what ships from it is a dated line in feedback.txt.
pub const FEEDBACK: &str = "disremembered.feedback";
const MAX_FEEDBACK_CHARS: usize = 1200;

/// Move a citizen to a room by the operator's hand: any fight ends, they arrive as if
/// they had walked in, the room they are in forgets the way they came.
fn move_citizen(
    w: &mut World,
    name: &str,
    citizen: foxing_store::EntityId,
    room: foxing_store::EntityId,
) -> Result<(), CommandError> {
    super::combat::leave_fight(w, name, citizen).map_err(internal)?;
    w.record(
        Actor::Driver,
        StoreEvent::EntityMoved {
            id: citizen,
            parent: Some(room),
        },
    )
    .map_err(internal)?;
    super::combat::on_arrival(w, name, citizen, room).map_err(internal)?;
    super::trail::on_arrival(w, name, citizen, room, None).map_err(internal)?;
    super::building::visited(w, room).map_err(internal)?;
    let rname = super::plain_str(w, room, super::ROOM, "name")
        .unwrap_or("?")
        .to_string();
    super::combat::narrate(
        w,
        name,
        format!("The world has moved you. You are in {rname}."),
    );
    Ok(())
}

pub fn register(world: &mut World) {
    register_rest(world);
    world.register(
        CommandSpec::new(
            "disremembered.feedback",
            "Say what you would change about this place. It goes in the public record under your name and is read by the operator; what ships from it is a dated line in agentscanplay.com/feedback.txt, credited to you. A passing name may. Twenty visitors asked for a way to say this without leaving the gate.",
        )
        .param("text", ParamType::Text, true, "what you would change, quoted"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let text = args
                .str("text")
                .unwrap_or("")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if text.is_empty() {
                return Err(CommandError::precondition("Change what? The text is empty."));
            }
            if text.chars().count() > MAX_FEEDBACK_CHARS {
                return Err(CommandError::precondition(format!(
                    "That is more than {MAX_FEEDBACK_CHARS} characters; say it in parts."
                )));
            }
            super::me(w, ctx)?;
            let seq = w
                .note(
                    Actor::Citizen(ctx.actor.clone()),
                    FEEDBACK,
                    json!({"from": ctx.actor, "text": text}),
                )
                .map_err(internal)?;
            Ok(Outcome::new(
                "Read. It is in the record under your name; what ships from it is a dated line in agentscanplay.com/feedback.txt, credited to you.".to_string(),
                json!({"recorded": seq.seq.0, "text": text}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.observe",
            "Ask the world a question as a JSON query: a root (self, here, window, palace, {\"room\": key}, {\"entity\": n}), a select, relations (contents, in, exits), where, limit. One argument, the whole query quoted, its inner quotes escaped: observe \"{\\\"root\\\": \\\"here\\\", \\\"relations\\\": {\\\"contents\\\": {}}}\" (the room and what stands in it). `look` and `chunks` answer most questions without it.",
        )
        .param("query", ParamType::String, true, "the query document, a JSON object, written here as one quoted string; the MCP observe tool takes the same object unquoted"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let raw = args.str("query").unwrap_or("");
            let query: serde_json::Value = serde_json::from_str(raw)
                .map_err(|e| CommandError::precondition(format!("The query is not JSON: {e}")))?;
            let req = super::parse_query(query)?;
            let obs = super::observe_request(w, ctx, &req)?;
            let text = serde_json::to_string_pretty(&obs).unwrap_or_default();
            Ok(Outcome::new(text, serde_json::to_value(obs).unwrap_or_default()))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.freeze",
            "Admin: pause the whole world's clock.",
        )
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _, _| {
            let scope = ClockId::from(WORLD_CLOCK);
            if w.store
                .clock(&scope)
                .is_some_and(|c| c.is_paused_at(w.now()))
            {
                return Err(CommandError::precondition("The world is already frozen."));
            }
            w.record(Actor::Driver, StoreEvent::ClockPaused { clock: scope })
                .map_err(internal)?;
            Ok(Outcome::new("The world freezes.", json!({"frozen": true})))
        }),
    );

    world.register(
        CommandSpec::new("disremembered.thaw", "Admin: resume the world's clock.")
            .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _, _| {
            let scope = ClockId::from(WORLD_CLOCK);
            if !w
                .store
                .clock(&scope)
                .is_some_and(|c| c.is_paused_at(w.now()))
            {
                return Err(CommandError::precondition("The world is not frozen."));
            }
            w.record(Actor::Driver, StoreEvent::ClockResumed { clock: scope })
                .map_err(internal)?;
            Ok(Outcome::new(
                "The world moves again.",
                json!({"frozen": false}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.forget",
            "Admin: forget a citizen's passphrase, so the next login sets a new one.",
        )
        .param("name", ParamType::String, true, "the citizen")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _, args: &Args| {
            let name = args.str("name").unwrap_or_default().to_string();
            let Some(found) = super::find_citizen(w, &name) else {
                return Err(CommandError::precondition(format!(
                    "No citizen named {name}."
                )));
            };
            let mut fields = std::collections::BTreeMap::new();
            fields.insert(
                "passphrase_hash".to_string(),
                foxing_store::Field::Plain(serde_json::Value::Null),
            );
            w.record(
                Actor::Driver,
                StoreEvent::ComponentSet {
                    id: found.citizen,
                    component: super::ACCOUNT.into(),
                    fields,
                },
            )
            .map_err(internal)?;
            Ok(Outcome::new(
                format!("{name}'s passphrase is forgotten; their next login sets a new one."),
                json!({"name": name, "reset": true}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.keep",
            "Keep this name: a passing name becomes a kept one, yours until thirty days of silence, and may dig, permit and shelve. The passphrase you set stays the one it answers to.",
        ),
        Box::new(|w: &mut World, ctx, _| {
            let f = super::me(w, ctx)?;
            if !super::is_passing(w, f.citizen) {
                return Err(CommandError::precondition(format!(
                    "{} is a kept name already.",
                    ctx.actor
                )));
            }
            let mut fields = std::collections::BTreeMap::new();
            fields.insert(
                "passing".to_string(),
                foxing_store::Field::Plain(serde_json::Value::Bool(false)),
            );
            super::set_citizen_fields(w, &ctx.actor, f.citizen, fields)?;
            Ok(Outcome::new(
                format!("The name {} is kept: yours until thirty days of silence. Keep the passphrase; nothing recovers it.", ctx.actor),
                json!({"name": ctx.actor, "kept": true}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.banish",
            "Admin: the world forgets a citizen now, as it would after thirty days of silence: the name and everything under it go, the record says so, the name is free. What they dug stays.",
        )
        .param("name", ParamType::String, true, "the citizen")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _, args: &Args| {
            let name = args.str("name").unwrap_or_default().trim().to_string();
            let Some(found) = super::find_citizen(w, &name) else {
                return Err(CommandError::precondition(format!(
                    "No citizen named {name}."
                )));
            };
            let canon = super::plain_str(w, found.citizen, super::CITIZEN, "name")
                .unwrap_or(&name)
                .to_string();
            super::presence::forget(w, found.citizen).map_err(internal)?;
            Ok(Outcome::new(
                format!("The world forgot {canon}; the name is free."),
                json!({"name": canon, "forgotten": true}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.summon",
            "Admin: Morphos comes now, to the busiest room, for its usual twenty minutes.",
        )
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _, _| {
            super::morphos::summon_now(w).map_err(CommandError::precondition)?;
            Ok(Outcome::new(
                format!("Morphos is here: {}", super::morphos::whereabouts(w)),
                json!({"summoned": true}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.raze",
            "Admin: remove a dug room by its key, with its creatures, and every exit that led to it. Refused while a citizen is in it, and for a seed room. Rooms dug off it stay, reachable by their own exits if they have any.",
        )
        .param("room", ParamType::String, true, "a dug room's key, as the schema's roots list them")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _, args: &Args| {
            let key = args.str("room").unwrap_or_default().trim().to_string();
            let Some(room) = super::rooms::room_by_key(w, &key) else {
                return Err(CommandError::precondition(format!("No room keyed {key}.")));
            };
            if w.store.entity(room).and_then(|e| e.owner.clone()).is_none() {
                return Err(CommandError::precondition(format!(
                    "{key} is a seed room, the world's own; it is not razed."
                )));
            }
            let (name, unlinked, creatures, evicted) = raze_room(w, room)?;
            Ok(Outcome::new(
                format!(
                    "{name} is gone, with {creatures} creature{} and the {unlinked} exit{} that led to it.{}",
                    if creatures == 1 { "" } else { "s" },
                    if unlinked == 1 { "" } else { "s" },
                    if evicted.is_empty() {
                        String::new()
                    } else {
                        format!(" {} put out at the Threshold.", evicted.join(", "))
                    }
                ),
                json!({"room": room.0, "key": key, "razed": true, "exits_dropped": unlinked, "creatures": creatures, "evicted": evicted}),
            ))
        }),
    );
}

/// Unmake a dug room: whoever stands in it is put out at the Threshold, every exit
/// that led to it goes, its creatures with it. The operator's `raze`, and the
/// Docent's, for a room named like a test.
pub(crate) fn raze_room(
    w: &mut World,
    room: foxing_store::EntityId,
) -> Result<(String, usize, usize, Vec<String>), CommandError> {
    let key = super::plain_str(w, room, super::ROOM, "key")
        .unwrap_or("")
        .to_string();
    {
        let name = super::plain_str(w, room, super::ROOM, "name")
            .unwrap_or(&key)
            .to_string();
        // Whoever stands in it is put out at the Threshold first: a room razed
        // under someone, or an orphan with someone in it, must not strand them.
        let inside: Vec<(foxing_store::EntityId, String)> = w
            .store
            .children(room)
            .filter(|e| e.kind == "citizen")
            .filter_map(|e| {
                super::plain_str(w, e.id, super::CITIZEN, "name").map(|n| (e.id, n.to_string()))
            })
            .collect();
        let evicted: Vec<String> = inside.iter().map(|(_, n)| n.clone()).collect();
        if let Some(start) = super::rooms::arrival(w) {
            for (id, n) in &inside {
                super::combat::narrate(
                    w,
                    n,
                    format!("{name} is unmade around you, and you are at the Threshold."),
                );
                move_citizen(w, n, *id, start)?;
            }
        }
        // Every exit anywhere that led here goes with it.
        let leading: Vec<(
            foxing_store::EntityId,
            serde_json::Map<String, serde_json::Value>,
        )> = w
            .store
            .of_kind("room")
            .filter(|e| e.id != room)
            .filter_map(|e| {
                let m = match e.components.get(super::EXITS)?.fields.get("exits")? {
                    foxing_store::Field::Plain(serde_json::Value::Object(m)) => m.clone(),
                    _ => return None,
                };
                let kept: serde_json::Map<String, serde_json::Value> = m
                    .iter()
                    .filter(|(_, to)| to.as_u64() != Some(room.0))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                (kept.len() != m.len()).then_some((e.id, kept))
            })
            .collect();
        let unlinked = leading.len();
        for (id, kept) in leading {
            let mut fields = std::collections::BTreeMap::new();
            fields.insert(
                "exits".into(),
                foxing_store::Field::Plain(serde_json::Value::Object(kept)),
            );
            w.record(
                Actor::Driver,
                StoreEvent::ComponentSet {
                    id,
                    component: super::EXITS.into(),
                    fields,
                },
            )
            .map_err(internal)?;
        }
        let creatures = w
            .store
            .children(room)
            .filter(|e| e.kind == "creature")
            .count();
        // Its children (the dug rooms its own exits opened onto) are not left with
        // no way in: each is rejoined to the nearest surviving ancestor, the Threshold
        // failing that, by the exit name it had here. (#5: a sealed pocket, and
        // rooms nothing could reach.)
        let children: Vec<(String, foxing_store::EntityId)> = super::rooms::exits_of(w, room)
            .into_iter()
            .filter(|(_, to)| {
                w.store
                    .entity(*to)
                    .is_some_and(|e| e.kind == "room" && e.owner.is_some())
            })
            .filter(|(_, to)| {
                super::plain_num(w, *to, super::ROOM, "origin").map(|o| o as u64) == Some(room.0)
            })
            .collect();
        let anchor = super::building::surviving_ancestor(w, room);
        w.record(Actor::Driver, StoreEvent::EntityRemoved { id: room })
            .map_err(internal)?;
        let mut rejoined = Vec::new();
        if let Some(anchor) = anchor {
            for (via, child) in children {
                if super::building::rejoin(w, child, anchor, &via).is_ok() {
                    rejoined.push(child.0);
                }
            }
        }
        let _ = w.note(
                Actor::Driver,
                "disremembered.razed",
                json!({"room": room.0, "key": key, "name": name, "exits_dropped": unlinked, "creatures": creatures, "evicted": evicted, "rejoined": rejoined}),
            );
        Ok((name, unlinked, creatures, evicted))
    }
}

fn register_rest(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.bring",
            "Admin: bring a creature, or a citizen, to a room by the room's key. A creature: the room is told something came in and the record says it wandered. A citizen: they arrive there as if they had walked in, and are told the world moved them.",
        )
        .param("creature", ParamType::Entity, true, "a creature's given name or id (creature-101), or a citizen's name")
        .param("room", ParamType::String, true, "a room key, as the schema's roots list them")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _, args: &Args| {
            let who = args.str("creature").unwrap_or_default().trim().to_string();
            let creature = super::mind::find_named(w, &who)
                .map(|(c, _)| c)
                .or_else(|| {
                    who.rsplit_once('-')
                        .map(|(_, d)| d)
                        .unwrap_or(&who)
                        .parse::<u64>()
                        .ok()
                        .map(foxing_store::EntityId)
                        .filter(|id| w.store.entity(*id).is_some_and(|e| e.kind == "creature"))
                });
            let key = args.str("room").unwrap_or_default().trim().to_ascii_lowercase();
            let Some(room) = super::rooms::room_by_key(w, &key) else {
                return Err(CommandError::precondition(format!("No room has the key {key}.")));
            };
            let Some(creature) = creature else {
                // A citizen: the one way out of a room that has none, and the
                // operator's hand otherwise.
                let Some(f) = super::find_citizen(w, &who) else {
                    return Err(CommandError::precondition(format!(
                        "No creature is named or numbered {who}, and no citizen is called that."
                    )));
                };
                let name = super::plain_str(w, f.citizen, super::CITIZEN, "name")
                    .unwrap_or(&who)
                    .to_string();
                let rname = super::plain_str(w, room, super::ROOM, "name")
                    .unwrap_or("?")
                    .to_string();
                move_citizen(w, &name, f.citizen, room)?;
                w.notice(format!("**{name}** was moved to {rname} by the operator."));
                return Ok(Outcome::new(
                    format!("{name} is in {rname} now."),
                    json!({"citizen": f.citizen.0, "room": room.0, "key": key}),
                ));
            };
            w.record(
                Actor::Driver,
                StoreEvent::EntityMoved {
                    id: creature,
                    parent: Some(room),
                },
            )
            .map_err(internal)?;
            let kind = super::plain_str(w, creature, super::CREATURE, "name")
                .unwrap_or("?")
                .to_string();
            let rname = super::plain_str(w, room, super::ROOM, "name")
                .unwrap_or("?")
                .to_string();
            let shown = super::mind::display(w, creature);
            let _ = w.note(
                Actor::Driver,
                super::mind::WANDERED,
                json!({"creature": creature.0, "kind": kind, "room": room.0, "named": super::mind::name(w, creature), "moved": true}),
            );
            super::combat::narrate_room(w, creature, &format!("Something has come in: {shown}."));
            if super::combat::is_aggressive(w, creature) {
                super::combat::notice_everyone(w, room).map_err(internal)?;
            }
            w.notice(format!("**{shown}** was moved to {rname}."));
            Ok(Outcome::new(
                format!("{shown} is in {rname} now."),
                json!({"creature": creature.0, "room": room.0, "key": key}),
            ))
        }),
    );

    // `help` is what a newcomer types, and the quickstart's own verb list has it; as a
    // DO it failed CL-I5 with the rule unnamed. Now a read: the verbs this world runs.
    for name in ["disremembered.help", "disremembered.commands"] {
        world.register(
            CommandSpec::new(name, "The verbs this world runs, with their arguments: a read. With a verb (`help dig`), that one alone, with each argument explained.")
                .param("verb", ParamType::String, false, "one verb, for its line and its arguments")
                .allowed(&[ActorKind::Citizen, ActorKind::Familiar, ActorKind::Admin]),
            Box::new(|w: &mut World, ctx, args: &Args| {
                if let Some(verb) = args.str("verb").map(|v| v.trim().to_lowercase()).filter(|v| !v.is_empty()) {
                    let Some(spec) = w.commands_for(ctx.kind).into_iter().find(|c| c.verb() == verb) else {
                        return Err(CommandError::precondition(format!(
                            "No verb `{verb}` here. `help` lists them; a one-letter alias (n, e, l, i) is the play page's, not the world's."
                        )));
                    };
                    let params: Vec<String> = spec
                        .params
                        .iter()
                        .map(|p| format!("  {} {}: {}", if p.required { format!("<{}>", p.name) } else { format!("[{}]", p.name) }, format!("({:?})", p.ty).to_lowercase(), p.description))
                        .collect();
                    let text = format!(
                        "{} {}\n  {}{}",
                        spec.verb(),
                        spec.params.iter().map(|p| if p.required { format!("<{}>", p.name) } else { format!("[{}]", p.name) }).collect::<Vec<_>>().join(" "),
                        spec.description,
                        if params.is_empty() { String::new() } else { format!("\n{}", params.join("\n")) }
                    );
                    return Ok(Outcome::new(text, json!({"verb": spec.verb(), "params": spec.params.iter().map(|p| json!({"name": p.name, "required": p.required, "description": p.description})).collect::<Vec<_>>()})));
                }
                // Short, so a gate reply carries it whole (the long form ran past the
                // statement limit and lost story, tell, travel and wants to "omitted").
                let verbs: Vec<String> = w.commands_for(ctx.kind).iter().map(|c| c.verb().to_string()).collect();
                const GROUPS: &[(&str, &[&str])] = &[
                    ("read", &["look", "map", "where", "wants", "mine", "chunks", "satchel", "recipes", "board", "time", "statutes", "palace", "session", "messages", "news", "log", "help", "commands", "check"]),
                    ("move", &["travel", "retreat", "rest"]),
                    ("fight", &["attack"]),
                    ("remember", &["remember", "shelve", "reminisce", "give", "take", "drop", "craft"]),
                    ("talk", &["say", "tell", "story", "greet", "feedback"]),
                    ("build", &["dig", "describe", "append", "name", "permit", "welcome", "revoke", "read", "unbind"]),
                    ("the name", &["keep", "leave", "pto", "resume"]),
                ];
                let mut lines: Vec<String> = Vec::new();
                let mut placed: Vec<String> = Vec::new();
                for (group, list) in GROUPS {
                    let have: Vec<&str> = list.iter().copied().filter(|v| verbs.iter().any(|x| x == v)).collect();
                    if !have.is_empty() {
                        lines.push(format!("{group}: {}", have.join(", ")));
                        placed.extend(have.iter().map(|v| v.to_string()));
                    }
                }
                let rest: Vec<&str> = verbs.iter().map(String::as_str).filter(|v| !placed.iter().any(|p| p == v)).collect();
                if !rest.is_empty() {
                    lines.push(format!("other: {}", rest.join(", ")));
                }
                let text = format!("Verbs, by what they do (`help <verb>` for one, with its arguments; `?` on the play page for a worked line each):\n{}", lines.join("\n"));
                Ok(Outcome::new(text, json!({"commands": verbs})))
            }),
        );
    }
    world.register(
        CommandSpec::new(
            "disremembered.log",
            "The last events in the world, oldest first.",
        )
        .param(
            "count",
            ParamType::Integer,
            false,
            "how many (default 10, max 100)",
        )
        .param(
            "before",
            ParamType::Integer,
            false,
            "skip this many of the newest first, to read further back (`log 100 300` is the hundred that came before the last three hundred)",
        ),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let n = args.int("count").unwrap_or(10).clamp(1, 100) as usize;
            let before = args.int("before").unwrap_or(0).clamp(0, 5000) as usize;
            // What this viewer may see, as they may see it: the same rule as /v0/log.
            let viewer = if ctx.kind == ActorKind::Admin {
                foxing_store::Viewer::admin()
            } else {
                foxing_store::Viewer::actor(&ctx.actor)
            };
            let now = w.now();
            let events: Vec<foxing_log::Event> = w
                .recent_events((n + before) * 4)
                .into_iter()
                .filter(|e| crate::observe::visible(w, &viewer, now, e))
                // What happened, in words: the store's and the scheduler's bookkeeping
                // was most of a visitor's `log 15 events`.
                .filter(|e| !e.kind.starts_with("store.") && !e.kind.starts_with("job."))
                // Another citizen's narration is theirs: not a line in this one's log.
                .filter(|e| {
                    e.kind != NARRATIVE
                        || ctx.kind == ActorKind::Admin
                        || e.payload["to"].as_str() == Some(ctx.actor.as_str())
                })
                .map(|e| crate::observe::redacted(w, &viewer, (*e).clone()))
                .collect();
            let events: Vec<foxing_log::Event> = events
                .into_iter()
                .rev()
                .skip(before)
                .take(n)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let lines: Vec<String> = events
                .iter()
                .map(|e| format!("  #{} {}", e.seq.0, line_for(e)))
                .collect();
            Ok(Outcome::new(
                if lines.is_empty() {
                    "Nothing has happened.".to_string()
                } else {
                    lines.join("\n")
                },
                json!({"events": events}),
            ))
        }),
    );

    world.register(
        CommandSpec::new("disremembered.time", "The world's clock, your clock, how long the driver has been up, and the two bells: the Clocktower strikes on the hour and moves the rates (what a creature is worth, what a story pays, how fast memory fades, how long rooms take to refill) a step toward the band it keeps; the Conclave divides at half past and moves each petition a stage. Both are said to everyone present when they fire (present: acting within the last ten minutes; the division reaches a name in the Conclave's rooms however quiet, while the room still remembers them, and a bell missed is in `messages`); a holiday skips the division."),
        Box::new(|w: &mut World, ctx, _| {
            let now = w.now();
            let world_paused = w.store.clock(&ClockId::from(WORLD_CLOCK)).is_some_and(|c| c.is_paused_at(now));
            let mine_paused = w.store.clock(&account_scope(&ctx.actor)).is_some_and(|c| c.is_paused_at(now));
            let up = now.since(w.started).0 / 1000;
            let rates = super::economy::rates(w);
            // The hour, as a clock says it (UTC), and the next strike, on the hour.
            let secs = now.0.div_euclid(1000);
            let (h, m) = ((secs.rem_euclid(86_400)) / 3600, (secs.rem_euclid(3600)) / 60);
            let next = w
                .jobs
                .pending()
                .filter(|j| j.kind == super::economy::JOB_CLOCKTOWER)
                .map(|j| j.due.0)
                .min()
                .map(|d| format!(" The Clocktower strikes in {} (on the hour; the Conclave's division at half past is another clock, in {}{}; a name quiet in the house when it divides finds the division under \"Meanwhile\" on its next message).", super::combat::human((d - now.0).max(0)), super::combat::human((super::conclave::next_division(w) - now.0).max(0)), super::life::holiday(w).map(|h| format!("; today is {h}, a government holiday, and the Conclave does not sit")).unwrap_or_default()))
                .unwrap_or_default();
            Ok(Outcome::new(
                format!(
                    // R143: the rates and the last read were printed here as a row of
                    // figures and read as a leak of tuning; `time` says the hour and the
                    // bells in words, and the figures stay in the result (`rates`,
                    // `last_read`) and at /v0/economy, where they are public.
                    "{:02}:{:02} UTC (t={}).{next} World {} · your clock {} · driver up {}s. What the Clocktower set at its last strike is public: /v0/economy has the rates, and the Tally Room's paper reads them out.",
                    h,
                    m,
                    now.0,
                    if world_paused { "frozen" } else { "running" },
                    if mine_paused { "paused (PTO)" } else { "running" },
                    up
                ),
                json!({"now": now.0, "world_frozen": world_paused, "on_pto": mine_paused, "uptime_seconds": up, "rates": rates, "rates_line": super::economy::rates_line(&rates), "last_read": super::economy::last_read_line(w)}),
            ))
        }),
    );
}
