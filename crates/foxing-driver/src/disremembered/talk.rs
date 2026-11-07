//! Talking to someone in particular.
//!
//! `tell <who> "<text>"` addresses one citizen or creature. A citizen hears it now if
//! they are here, and on their return if they are not: the words wait in the log as a
//! `disremembered.tell` and are delivered as narrative when they next arrive. A named
//! creature answers. At this tier it has no mind of its own: it answers from what the
//! log knows, who keeps it, what it holds, what has happened in its room, and says so
//! as itself; its words are recorded as a `disremembered.say` with the creature as
//! their source, so an agent sees them as a `[QUOTE]` like anyone's. A creature with
//! no name does not understand you. `say` reaches a named creature too, if you use
//! its name.

use super::combat::narrate;
use super::speech::SAY;
use super::{internal, me, mind, plain_str, presence, rooms, CHUNK, CITIZEN, ROOM};
use crate::world::World;
use foxing_log::{Actor, Cursor};
use foxing_registry::{Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Words left for an absent citizen: `{from, to, room, text}`.
pub const TELL: &str = "disremembered.tell";
const MAX_CHARS: usize = 600;

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.greet",
            "Set a standing greeting: while you are here, whoever arrives where you are hears it from you, with a note that tell reaches you. Empty text clears it. For a citizen at a keyboard who cannot answer within a visitor's minute, it is the first word.",
        )
        .param("text", ParamType::Text, false, "what to say to whoever arrives, quoted; nothing, or empty, to clear"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = super::me(w, ctx)?;
            let text = args
                .str("text")
                .unwrap_or("")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if text.chars().count() > 240 {
                return Err(CommandError::precondition("A greeting is at most 240 characters."));
            }
            let mut fields = BTreeMap::new();
            fields.insert(
                "greeting".into(),
                Field::Plain(if text.is_empty() { Value::Null } else { json!(text) }),
            );
            super::set_citizen_fields(w, &ctx.actor, f.citizen, fields)?;
            Ok(Outcome::new(
                if text.is_empty() {
                    "You will greet no one in particular.".to_string()
                } else {
                    format!("Whoever arrives where you are will hear: \"{text}\"")
                },
                json!({"greeting": if text.is_empty() { Value::Null } else { json!(text) }}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.tell",
            "Say something to one citizen or creature here. A citizen who is away hears it when they return. A creature with a name answers from what it remembers; one without a name does not understand you.",
        )
        .param("who", ParamType::Entity, true, "a citizen, or a creature by kind or by name")
        .param("text", ParamType::Text, true, "what you say, quoted"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let who = args.str("who").unwrap_or("").trim().to_string();
            let text = args.str("text").unwrap_or("").trim().to_string();
            if text.is_empty() {
                return Err(CommandError::precondition("Tell them what? The text is empty."));
            }
            if text.chars().count() > MAX_CHARS {
                return Err(CommandError::precondition(format!(
                    "That is more than {MAX_CHARS} characters; say it in parts."
                )));
            }
            let f = me(w, ctx)?;
            let room = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            // Morphos, when it stands here: an answer to its riddle.
            if super::morphos::is_named(&who) {
                let (line, result) = super::morphos::answer(w, &ctx.actor, &f, room, &text)?;
                if super::morphos::present(w) == Some(room) {
                    for other in others_here(w, room, &ctx.actor) {
                        narrate(w, &other, format!("{} says something to Morphos, low.", ctx.actor));
                    }
                }
                return Ok(Outcome::new(format!("You say to Morphos, \"{text}\" {line}"), result));
            }
            // The Docent, if it stands here: the question goes on its record and an
            // answer comes to you within the minute, wherever you are.
            if super::docent::is_named(&who) {
                // Asked from anywhere: at the door it is the figure in front of you;
                // elsewhere the question goes to the desk and it comes to you with the
                // page. (It was asked only at the Threshold once, and a visitor three
                // rooms in stopped asking.)
                let here_with_it = super::docent::room(w) == Some(room);
                let (line, id) = super::docent::ask(w, &ctx.actor, room, &text)?;
                if id == 0 {
                    return Ok(Outcome::new(
                        format!("You ask the Docent, \"{text}\" {line}"),
                        json!({"to": "the Docent", "text": text, "asked": Value::Null, "closed": true}),
                    ));
                }
                if here_with_it {
                    for other in others_here(w, room, &ctx.actor) {
                        narrate(w, &other, format!("{} asks the Docent something; it nods.", ctx.actor));
                    }
                }
                let nth = super::docent::nth_tonight(w, &ctx.actor);
                let said = if here_with_it {
                    format!("You ask the Docent, \"{text}\" The Docent: \"{line}\" (It finds you within a few minutes, wherever you are, and names the question; {nth}.)")
                } else {
                    format!("You ask after the Docent, \"{text}\" The question goes to the desk at the Threshold; the Docent comes to you with the page within a few minutes, wherever you are, and names the question ({nth}).")
                };
                return Ok(Outcome::new(
                    said,
                    json!({"to": "the Docent", "text": text, "asked": id, "here": here_with_it}),
                ));
            }
            // A creature here?
            if let Some(creature) = mind::find_here(w, room, &who) {
                let name = mind::display(w, creature);
                w.note(
                    Actor::Citizen(ctx.actor.clone()),
                    SAY,
                    json!({"from": ctx.actor, "room": room.0, "to": name, "text": text}),
                )
                .map_err(internal)?;
                for other in others_here(w, room, &ctx.actor) {
                    narrate(w, &other, format!("{} says to {name}, \"{text}\"", ctx.actor));
                }
                // A word carried to it, if one was wanted, before anything it would say.
                let answer = match super::errand::carried_word(w, &ctx.actor, creature, &text)? {
                    Some(a) => Some(a),
                    None => reply(w, creature, &ctx.actor, &text),
                };
                let Some(answer) = answer else {
                    // F21: a newborn that knows you (you carried the word that made
                    // it, or are its parent's keeper) is not a stranger, only wordless.
                    if mind::keepers(w, creature).iter().any(|k| k == &ctx.actor) {
                        return Ok(Outcome::new(
                            format!("You say to the {name}, \"{text}\" It turns toward your voice as if it knew it, which it does; it has no name yet, and no words until it has one. It holds {:.0}; at sixty it answers.", super::up(mind::level(w, creature))),
                            json!({"to": name, "text": text, "answered": false, "knows_you": true}),
                        ));
                    }
                    // R43: how one gets a name, in the same breath.
                    return Ok(Outcome::new(
                        format!("You say to the {name}, \"{text}\" It does not understand you. It is only {}; it {}. Only a named creature answers: feed one to sixty (`remember <chunk> {name}`) and it has a name and an ear.", super::mind::an(&name), super::mind::goes_on(&name)),
                        json!({"to": name, "text": text, "answered": false}),
                    ));
                };
                // The answer is in the outcome; the room hears it as a narrator line,
                // the teller not twice (a carried word's reply printed twice on the page).
                speak_to(w, creature, &ctx.actor, &answer, false)?;
                return Ok(Outcome::new(
                    format!("You say to {name}, \"{text}\" {name} says, \"{answer}\""),
                    json!({"to": name, "text": text, "answered": true, "reply": answer}),
                ));
            }
            // A citizen, here or away.
            let typed = who.trim_start_matches("citizen-").to_string();
            let Some(target) = super::canonical_name(w, &typed) else {
                // A kind typed where every one of the kind has a name ("tell wisp"
                // beside Fallis and Tinsel) is answered with the names, not "no such
                // creature" (the Watch of 11-06 read that as a lie about the room).
                let kind = typed.trim_start_matches("the ").to_ascii_lowercase();
                let of_kind: Vec<String> = super::combat::creatures_in(w, room)
                    .iter()
                    .filter(|c| c.1 == kind)
                    .map(|c| mind::display(w, c.0))
                    .collect();
                if !of_kind.is_empty() {
                    return Err(CommandError::precondition(format!(
                        "The {kind}{} here {} a name: {}. Tell one by its name.",
                        if of_kind.len() == 1 { "" } else { "s" },
                        if of_kind.len() == 1 { "has" } else { "each have" },
                        of_kind.join(", ")
                    )));
                }
                return Err(CommandError::precondition(format!(
                    "There is no one called {typed}, and no such creature here."
                )));
            };
            let Some(them) = super::find_citizen(w, &target) else {
                return Err(CommandError::precondition(format!("There is no one called {typed}.")));
            };
            if target == ctx.actor {
                return Err(CommandError::precondition("You tell yourself. Nothing changes."));
            }
            let their_room = rooms::room_of(w, them.citizen);
            let present = their_room == Some(room) && presence::is_here(w, them.citizen);
            if present {
                w.note(
                    Actor::Citizen(ctx.actor.clone()),
                    SAY,
                    json!({"from": ctx.actor, "room": room.0, "to": target, "text": text}),
                )
                .map_err(internal)?;
                narrate(w, &target, format!("{} says to you, \"{text}\"", ctx.actor));
                for other in others_here(w, room, &ctx.actor).into_iter().filter(|o| o != &target) {
                    narrate(w, &other, format!("{} says to {target}, \"{text}\"", ctx.actor));
                }
                // R5: the Conclave's clerks write a `tell` down as they write a `say`.
                let clerks = super::combat::creatures_in(w, room)
                    .iter()
                    .filter(|c| c.1 == "engrosser")
                    .count();
                let wrote = match clerks {
                    0 => "",
                    1 => " An engrosser writes it down, word for word.",
                    _ => " The engrossers write it down, word for word.",
                };
                return Ok(Outcome::new(
                    format!("You say to {target}, \"{text}\" They hear you.{wrote}"),
                    json!({"to": target, "text": text, "delivered": true, "written_down": clerks > 0}),
                ));
            }
            w.note(
                Actor::Citizen(ctx.actor.clone()),
                TELL,
                json!({"from": ctx.actor, "to": target, "room": room.0, "text": text}),
            )
            .map_err(internal)?;
            // In the world, in another room: the word reaches them with their next
            // reply, not on an "arrival" that never comes while they keep acting (a
            // visitor crossed four rooms and found the word only in `log`).
            if presence::is_here(w, them.citizen) {
                let here_name = plain_str(w, room, ROOM, "name").unwrap_or("somewhere").to_string();
                narrate(w, &target, format!("{} left word for you from {here_name}: \"{text}\"", ctx.actor));
                mark_heard(w, them.citizen)?;
                let how = if super::combat::through_the_gate(w, &target) {
                    "they came through the gate, and will read it with their next reply"
                } else {
                    "they are at a keyboard, and see it now"
                };
                return Ok(Outcome::new(
                    format!("{target} is elsewhere; the word goes to them ({how}): \"{text}\""),
                    json!({"to": target, "text": text, "delivered": true, "elsewhere": true, "gate": super::combat::through_the_gate(w, &target)}),
                ));
            }
            Ok(Outcome::new(
                format!("{target} is not here. They will hear it when they return: \"{text}\""),
                json!({"to": target, "text": text, "delivered": false}),
            ))
        }),
    );
    world.on_arrival(Box::new(|w: &mut World, actor: &str| deliver(w, actor)));
}

pub(crate) fn others_here(world: &World, room: EntityId, me: &str) -> Vec<String> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen" && presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .filter(|o| o != me)
        .collect()
}

/// A creature speaks: recorded as its own `say`, heard by everyone here.
/// A named creature answers `to`: the room hears who it was answering, so a visitor
/// does not read another's answer as their own (one did, and reported the world
/// inconsistent).
pub fn speak(
    world: &mut World,
    creature: EntityId,
    to: &str,
    text: &str,
) -> Result<(), CommandError> {
    speak_to(world, creature, to, text, true)
}

/// As [`speak`], with a choice of whether the one spoken to hears the narrator line
/// (false when the outcome already carries the words).
pub fn speak_to(
    world: &mut World,
    creature: EntityId,
    to: &str,
    text: &str,
    to_hears: bool,
) -> Result<(), CommandError> {
    let name = mind::display(world, creature);
    let Some(room) = world.store.entity(creature).and_then(|e| e.parent) else {
        return Ok(());
    };
    world
        .note(
            Actor::Driver,
            SAY,
            json!({"from": name, "creature": creature.0, "room": room.0, "to": to, "text": text}),
        )
        .map_err(internal)?;
    let present: Vec<String> = world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen" && presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .collect();
    for actor in present {
        let line = if actor == to {
            if !to_hears {
                continue;
            }
            format!("{name} says to you, \"{text}\"")
        } else {
            format!("{name} says to {to}, \"{text}\"")
        };
        narrate(world, &actor, line);
    }
    Ok(())
}

/// Everything left so far is heard: a word narrated to a citizen now is not read to
/// them again when they next arrive.
fn mark_heard(world: &mut World, citizen: EntityId) -> Result<(), CommandError> {
    let head = world
        .log
        .head(&world.id)
        .map_err(internal)?
        .map(|s| s.0)
        .unwrap_or(0);
    let mut fields = world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(presence::PRESENCE))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    fields.insert("heard_through".into(), Field::Plain(json!(head)));
    world
        .record(
            Actor::Driver,
            foxing_store::StoreEvent::ComponentSet {
                id: citizen,
                component: presence::PRESENCE.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(internal)
}

/// Words waiting for a citizen who just arrived.
pub fn deliver(world: &mut World, actor: &str) -> Result<(), String> {
    let Some(f) = super::find_citizen(world, actor) else {
        return Ok(());
    };
    let heard: u64 = match world
        .store
        .entity(f.citizen)
        .and_then(|e| e.components.get(presence::PRESENCE))
        .and_then(|c| c.fields.get("heard_through"))
    {
        Some(Field::Plain(v)) => v.as_u64().unwrap_or(0),
        _ => 0,
    };
    let head = world
        .log
        .head(&world.id)
        .map_err(|e| e.to_string())?
        .map(|s| s.0)
        .unwrap_or(0);
    if head <= heard {
        return Ok(());
    }
    let waiting: Vec<(String, String)> = world
        .log
        .read_all(&world.id, Cursor { after: heard })
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|e| e.kind == TELL && e.payload["to"] == actor)
        .map(|e| {
            (
                e.payload["from"].as_str().unwrap_or("?").to_string(),
                e.payload["text"].as_str().unwrap_or("").to_string(),
            )
        })
        .collect();
    for (from, text) in &waiting {
        narrate(
            world,
            actor,
            format!("While you were away, {from} left word: \"{text}\""),
        );
    }
    let mut fields = world
        .store
        .entity(f.citizen)
        .and_then(|e| e.components.get(presence::PRESENCE))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    if !fields.contains_key("here") {
        fields.insert("here".into(), Field::Plain(json!(true)));
    }
    fields.insert("heard_through".into(), Field::Plain(json!(head)));
    world
        .record(
            Actor::Driver,
            foxing_store::StoreEvent::ComponentSet {
                id: f.citizen,
                component: presence::PRESENCE.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// `say` reached a named creature by name: it answers.
pub fn overhear(
    world: &mut World,
    room: EntityId,
    actor: &str,
    text: &str,
) -> Result<Vec<String>, CommandError> {
    let lower = text.to_lowercase();
    let named: Vec<EntityId> = world
        .store
        .children(room)
        .filter(|e| e.kind == "creature")
        .map(|e| e.id)
        .filter(|&c| mind::name(world, c).is_some_and(|n| lower.contains(&n.to_lowercase())))
        .collect();
    let mut answers = Vec::new();
    for c in named {
        if let Some(a) = reply(world, c, actor, text) {
            speak(world, c, actor, &a)?;
            answers.push(format!("{} says, \"{a}\"", mind::display(world, c)));
        }
    }
    Ok(answers)
}

/// A named creature notices who walks in.
pub fn greet(world: &mut World, actor: &str, room: EntityId) {
    // Citizens here with a standing greeting say it to whoever arrives.
    let greeters: Vec<(String, String)> = world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen")
        .filter(|e| e.owner.as_deref() != Some(actor))
        .filter(|e| presence::is_here(world, e.id))
        .filter_map(|e| {
            let name = plain_str(world, e.id, super::CITIZEN, "name")?.to_string();
            let text = plain_str(world, e.id, super::CITIZEN, "greeting")?.to_string();
            Some((name, text))
        })
        .collect();
    for (name, text) in greeters {
        // Once an hour per visitor: a greeting that replays on every step through the
        // room is noise, as a visitor who heard it eight times said.
        if !world.greeted(&name, actor) {
            continue;
        }
        narrate(
            world,
            actor,
            format!("{name}, who is here, says to you as you arrive: \"{text}\" (tell {name} answers them.)"),
        );
    }
    let named: Vec<EntityId> = world
        .store
        .children(room)
        .filter(|e| e.kind == "creature")
        .map(|e| e.id)
        .filter(|&c| mind::name(world, c).is_some())
        .collect();
    for c in named {
        // Once an hour per visitor, like a citizen's greeting: said on every step
        // through the room, "You again" twelve seconds later read as a tic.
        let name = mind::display(world, c);
        if !world.greeted(&format!("creature-{}", c.0), actor) {
            continue;
        }
        let line = if mind::keepers(world, c).iter().any(|k| k == actor) {
            format!("{name} looks up. \"You again, {actor}.\"")
        } else if mind::told_by(world, c, actor) || super::errand::did_for(world, c, actor) {
            // R79: a creature that remembers you says your name before you speak.
            format!("{name} looks up and says your name before you speak: \"{actor}.\"")
        } else {
            format!("{name} looks up at you, and does not look away.")
        };
        narrate(world, actor, line);
    }
    // R79: the picnicker sets another place for whoever is new to the picnic.
    let picnicker = super::combat::creatures_in(world, room)
        .into_iter()
        .find(|c| c.1 == "picnicker")
        .map(|c| c.0);
    if let Some(p) = picnicker {
        if world.greeted(&format!("picnic-{}", p.0), actor) {
            let who = mind::display(world, p);
            narrate(
                world,
                actor,
                format!("{who} sets another place at the cloth without a word, and pours."),
            );
        }
    }
}

/// What a named creature says back: facts, from the log and the store, in its voice.
/// `None` when it has no name to answer with.
pub fn reply(world: &World, creature: EntityId, actor: &str, text: &str) -> Option<String> {
    let name = mind::name(world, creature)?;
    let kind = plain_str(world, creature, super::CREATURE, "name")
        .unwrap_or("?")
        .to_string();
    let t = text.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| t.contains(w));
    let keepers = mind::keepers(world, creature);
    // A creature knows whoever kept it, and whoever told it a story: a visitor who
    // told one and said thanks was asked "For what?", which is not how a story is received.
    let told_me =
        mind::told_by(world, creature, actor) || super::errand::did_for(world, creature, actor);
    let knows_you = keepers.iter().any(|k| k == actor) || told_me;
    let level = mind::level(world, creature);
    let room = world.store.entity(creature).and_then(|e| e.parent);
    let sources = held(world, creature);
    let list = |v: &[String]| -> String {
        match v.len() {
            0 => "nothing".into(),
            1 => v[0].clone(),
            n => format!("{}, and {}", v[..n - 1].join(", "), v[n - 1]),
        }
    };

    // A child knows whose it is, and what is strange about it.
    let parents: Option<(String, String)> = world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(super::CREATURE))
        .and_then(|c| c.fields.get("parents"))
        .and_then(|f| match f {
            foxing_store::Field::Plain(v) => v.as_array().cloned(),
            _ => None,
        })
        .filter(|p| p.len() == 2)
        .map(|p| {
            (
                p[0].as_str().unwrap_or("?").to_string(),
                p[1].as_str().unwrap_or("?").to_string(),
            )
        });
    if parents.is_some()
        && has(&[
            "who made you",
            "your parents",
            "born",
            "whose are you",
            "your mother",
            "your father",
        ])
    {
        let (a, b) = parents.clone().unwrap_or_default();
        let carrier = plain_str(world, creature, super::CREATURE, "born_of")
            .unwrap_or("someone")
            .to_string();
        let strange = plain_str(world, creature, super::CREATURE, "mutation")
            .map(|m| format!(" I am {m}; neither of them was."))
            .unwrap_or_default();
        return Some(format!(
            "I am {name}, a {kind}, of {a} and {b}. {carrier} carried the word between them, and I came of it.{strange}"
        ));
    }
    // The word under the others: a hint for the level it is at, never the word.
    if has(&[
        "your word",
        "the word under",
        "secret",
        "third word",
        "hidden word",
        "word you keep",
    ]) {
        return super::secret::hint(world, actor, creature);
    }
    // What it wanted once: the want before this one, said whole (the first word under
    // the others is drawn from it, D7).
    if has(&[
        "wanted once",
        "want once",
        "used to want",
        "wanted before",
        "once want",
    ]) {
        return Some(match super::errand::wanted_once(world, creature) {
            Some(s) => format!("Once I wanted a story about {s}. I have not forgotten it."),
            None => "I have wanted only what I want now.".to_string(),
        });
    }
    // Born of nobody: "who made you" is who fed it to its name, which is the only
    // making it knows (the rest once got the canned "I don't know that" here).
    if has(&[
        "who made you",
        "your parents",
        "born",
        "whose are you",
        "your mother",
        "your father",
        "who named you",
        "who fed you",
    ]) {
        let first = keepers.first().cloned().unwrap_or_else(|| "someone".into());
        let last = mind::namer(world, creature).unwrap_or_else(|| first.clone());
        let with = first_gift(world, creature).unwrap_or_else(|| "a memory".into());
        return Some(if keepers.len() > 1 && first != last {
            format!("No one made me; I was a {kind} before anyone looked. {first} fed me first, {with}, and {last} fed me to my name. That is the nearest I have to parents.")
        } else {
            format!("No one made me; I was a {kind} before anyone looked. {first} fed me {with}, and fed me to my name. That is the nearest I have to a parent.")
        });
    }
    if has(&["who are you", "your name", "what are you", "who is this"]) {
        if let Some((a, b)) = parents {
            return Some(format!(
                "I am {name}. I was born a {kind}, of {a} and {b}, and I have been a {kind} with a name since {} fed me to it.",
                mind::last_feeder(world, creature).unwrap_or_else(|| "someone".into())
            ));
        }
        let first = keepers.first().cloned().unwrap_or_else(|| "someone".into());
        let with = first_gift(world, creature).unwrap_or_else(|| "a memory".into());
        return Some(format!(
            "I am {name}. I was a {kind}, and nothing else, until {first} gave me {with}. Now I am a {kind} with a name."
        ));
    }
    if has(&[
        "what else",
        "the rest",
        "go on",
        "and the others",
        "more of",
    ]) {
        // The rest of what it holds, four at a time: the answer to "and 18 more".
        let asked = world
            .recent_events(400)
            .iter()
            .filter(|e| {
                e.kind == SAY
                    && e.payload["from"] == actor
                    && e.payload["to"]
                        .as_str()
                        .is_some_and(|t| t.starts_with(&name))
                    && e.payload["text"].as_str().is_some_and(|t| {
                        let t = t.to_lowercase();
                        [
                            "what else",
                            "the rest",
                            "go on",
                            "and the others",
                            "more of",
                        ]
                        .iter()
                        .any(|w| t.contains(w))
                    })
            })
            .count();
        // The ask being answered is already in the log, so the count includes it: the
        // first "what else" is page two.
        let asked = asked.max(1);
        let page: Vec<String> = sources.iter().skip(4 * asked).take(4).cloned().collect();
        return Some(if page.is_empty() {
            "That is all of it.".into()
        } else {
            let left = sources.len().saturating_sub(4 * (asked + 1));
            format!(
                "Also {}{}.",
                list(&page),
                if left > 0 {
                    format!(", and {left} more")
                } else {
                    String::new()
                }
            )
        });
    }
    // Recite: the full text of a story it was told (the hunt for the word under the
    // others pointed at a held story no verb could surface). The newest, or the one
    // the asker names a word of.
    if has(&[
        "recite",
        "whole story",
        "tell me the story",
        "story again",
        "word for word",
        "the full story",
        "read me",
    ]) {
        let display = mind::display(world, creature);
        let to_me = |to: &str| {
            display.eq_ignore_ascii_case(to)
                || mind::name(world, creature).is_some_and(|n| n.eq_ignore_ascii_case(to))
        };
        let told: Vec<(String, String)> = world
            .recent_events(4000)
            .iter()
            .filter(|e| {
                e.kind == SAY
                    && e.payload["story"] == true
                    && e.payload["to"].as_str().is_some_and(to_me)
            })
            .filter_map(|e| {
                Some((
                    e.payload["from"].as_str()?.to_string(),
                    e.payload["text"].as_str()?.to_string(),
                ))
            })
            .collect();
        if told.is_empty() {
            return Some(
                "No one has told me a story yet. I would hold one, if you had it.".to_string(),
            );
        }
        let asked: Vec<String> = t
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 3)
            .map(str::to_string)
            .collect();
        // F6: "the first thing anyone told you" is the oldest, asked for by "first".
        // R88: the oldest still held whole, said so (older ones fold into sheaves,
        // which are not recited).
        let first_asked = asked
            .iter()
            .any(|w| w == "first" || w == "earliest" || w == "oldest");
        let pick = if first_asked {
            told.first().cloned()
        } else {
            None
        }
        .or_else(|| {
            told.iter()
                .rev()
                .find(|(_, t)| {
                    let lt = t.to_lowercase();
                    asked.iter().any(|w| {
                        ![
                            "recite", "whole", "story", "tell", "again", "word", "full", "read",
                            "please",
                        ]
                        .contains(&w.as_str())
                            && lt.contains(w.as_str())
                    })
                })
                .cloned()
        })
        .or_else(|| told.last().cloned())
        .unwrap();
        return Some(format!(
            "I hold it whole{}. {} told me: \"{}\"{}",
            if first_asked {
                ", the oldest I still hold whole (older ones have folded into sheaves, which I cannot say)"
            } else {
                ""
            },
            pick.0,
            pick.1,
            if told.len() > 1 {
                format!(
                    " I hold {} stories; name a word of another and I will say that one.",
                    told.len()
                )
            } else {
                String::new()
            }
        ));
    }
    // R65: "what word did X carry?" is about the carried-word record, not the hold
    // (it matched "what" and "carry" and gave the holdings).
    if has(&[
        "what word",
        "which word",
        "word did",
        "word was",
        "carried",
        "brought",
    ]) {
        let display = mind::display(world, creature);
        let mine = |v: &serde_json::Value| {
            v.as_str()
                .is_some_and(|t| t.eq_ignore_ascii_case(&name) || t.eq_ignore_ascii_case(&display))
        };
        let carried: Vec<String> = world
            .recent_events(6000)
            .iter()
            .filter(|e| e.kind == super::errand::DONE && e.payload["errand"] == "word")
            .filter_map(|e| {
                let p = &e.payload;
                let from = p["from"].as_str()?;
                let word = p["want"].as_str()?;
                if p["creature"].as_u64() == Some(creature.0) {
                    Some(format!(
                        "{from} carried my word, \"{word}\", to {}",
                        p["to"].as_str().unwrap_or("someone")
                    ))
                } else if mine(&p["to"]) {
                    Some(format!(
                        "{from} brought me {}'s word: \"{word}\"",
                        p["name"].as_str().unwrap_or("someone")
                    ))
                } else {
                    None
                }
            })
            .collect();
        // The words under the others said to it are carries too, to the one who
        // said them: "none" right after three of them read as a contradiction
        // (the Watch of 11-06).
        let struck: Vec<String> = world
            .recent_events(6000)
            .iter()
            .filter(|e| {
                e.kind == super::secret::STRUCK
                    && e.payload["creature"].as_u64() == Some(creature.0)
            })
            .filter_map(|e| {
                Some(format!(
                    "{} said the word under the others, \"{}\"",
                    e.payload["from"].as_str()?,
                    e.payload["word"].as_str()?
                ))
            })
            .collect();
        let errands = if carried.is_empty() {
            "No errand's word has been carried for me or to me yet".to_string()
        } else {
            let n = carried.len();
            carried[n.saturating_sub(3)..].join("; ")
        };
        return Some(if struck.is_empty() {
            format!("{errands}.")
        } else {
            let n = struck.len();
            format!("{errands}; {}.", struck[n.saturating_sub(3)..].join("; "))
        });
    }
    if has(&[
        "remember",
        "hold",
        "keep",
        "memor",
        "what do you have",
        "carry",
    ]) {
        let taken = last_taken(world, creature);
        // The newest few, not the whole window: a wisp with thirty stories answered
        // "who keeps you?" with a page.
        let shown: Vec<String> = sources.iter().take(4).cloned().collect();
        let mut s = format!("I hold {level:.0}: {}", list(&shown));
        if sources.len() > shown.len() {
            s.push_str(&format!(
                ", and {} more (ask what else)",
                sources.len() - shown.len()
            ));
        }
        s.push('.');
        if keepers.is_empty() {
            s.push_str(" No one keeps me.");
        } else {
            s.push_str(&format!(
                " {} keep{} me.",
                list(&keepers),
                if keepers.len() == 1 { "s" } else { "" }
            ));
        }
        if let Some((by, with)) = taken {
            s.push_str(&format!(
                " {by} took the memory of {with} from me. I remember that it is gone, which is not the same."
            ));
        }
        return Some(s);
    }
    if has(&["where", "this place", "what room", "which room"]) {
        let (rname, desc, exits) = room
            .map(|r| {
                let d = plain_str(world, r, ROOM, "description").unwrap_or("");
                let first = d
                    .split_inclusive(['.', '!', '?'])
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                (
                    plain_str(world, r, ROOM, "name")
                        .unwrap_or("here")
                        .to_string(),
                    first,
                    rooms::exits_of(world, r)
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>(),
                )
            })
            .unwrap_or_default();
        return Some(format!(
            "{rname}. {desc} The ways out are {}.",
            list(&exits)
        ));
    }
    if has(&[
        "who has",
        "who was",
        "seen",
        "anyone",
        "been here",
        "who came",
        "what happened",
        "news",
    ]) {
        let seen = recent(world, creature, room);
        return Some(if seen.is_empty() {
            "No one, since I started counting. I started counting when I got my name.".into()
        } else {
            format!("Since I had a name: {}.", seen.join("; "))
        });
    }
    if has(&["fade", "forget", "die", "afraid", "last"]) {
        let bar = mind::TIERS
            .iter()
            .find(|(t, _, _)| *t == mind::tier(world, creature))
            .map(|(_, b, _)| b / 2.0)
            .unwrap_or(30.0);
        return Some(format!(
            "Everything fades. I hold {level:.0}; below {bar:.0} I will not know my name, and you will say \"the {kind}\" again and be right."
        ));
    }
    if has(&["hello", "hi ", "hi,", "hey", "greet", "good "])
        || t.trim_end_matches(['.', '!']) == "hi"
    {
        return Some(if knows_you {
            format!("{actor}. You again. I hold {level:.0} today.")
        } else if keepers.is_empty() {
            "I do not know you. I do not know anyone yet.".into()
        } else {
            format!("I do not know you. I know {}.", list(&keepers))
        });
    }
    if has(&["thank"]) {
        // Each creature in its own words, so a room of them does not answer a
        // farewell in chorus.
        const THANKS: &[&str] = &[
            "For the story? I hold it. That is the thanks.",
            "Keep your thanks; I keep the story. We are even.",
            "The story thanks you. I only hold it.",
            "No. Thank the story. I am where it lives now.",
            "You gave; I hold. That is the whole of it.",
        ];
        return Some(if told_me {
            THANKS[creature.0 as usize % THANKS.len()].to_string()
        } else if knows_you {
            "It is your memory. I only hold it.".into()
        } else {
            "For nothing yet. Tell me a story and there will be something of yours in me.".into()
        });
    }
    if has(&[
        "bye", "farewell", "see you", "so long", "must go", "leaving",
    ]) {
        return Some(if knows_you {
            format!("Go, then, {actor}. What you gave me holds as long as it holds.")
        } else {
            "Go, then. I will not remember you, which is nothing against you.".into()
        });
    }
    if has(&["sorry", "apolog"]) {
        return Some("Nothing is taken from me that fading would not take.".into());
    }
    Some(if knows_you {
        format!("I hold what I am given, {actor}. Ask me what I remember, where this is, or who has been here.")
    } else {
        "I hold what I am given. Ask me what I remember, where this is, or who has been here."
            .into()
    })
}

/// The sources of what a creature holds, most recent first.
fn held(world: &World, creature: EntityId) -> Vec<String> {
    let Some(window) = mind::window_of(world, creature) else {
        return Vec::new();
    };
    let mut v: Vec<(u64, String)> = world
        .store
        .children(window)
        .filter(|e| e.kind == "chunk")
        .filter_map(|e| {
            plain_str(world, e.id, CHUNK, "source")
                .map(|s| (e.id.0, format!("the memory of {}", short_source(s))))
        })
        .collect();
    v.sort_by_key(|(id, _)| std::cmp::Reverse(*id));
    // The same memory held twice (a story told twice, two tellings of Ossian's) is
    // one line with a count, not two lines.
    let mut out: Vec<(String, usize)> = Vec::new();
    for (_, s) in v {
        match out.iter_mut().find(|(t, _)| *t == s) {
            Some((_, n)) => *n += 1,
            None => out.push((s, 1)),
        }
    }
    out.into_iter()
        .map(|(s, n)| match n {
            1 => s,
            2 => format!("{s} (twice)"),
            n => format!("{s} ({n} times)"),
        })
        .collect()
}

fn first_gift(world: &World, creature: EntityId) -> Option<String> {
    match world
        .store
        .entity(creature)?
        .components
        .get(mind::MIND)?
        .fields
        .get("kept")?
    {
        Field::Plain(Value::Array(a)) => a
            .first()
            .and_then(|k| k["with"].as_str())
            .map(|s| format!("the memory of {s}")),
        _ => None,
    }
}

fn last_taken(world: &World, creature: EntityId) -> Option<(String, String)> {
    world
        .recent_events(2000)
        .into_iter()
        .rev()
        .find(|e| e.kind == mind::TAKEN && e.payload["creature"] == creature.0)
        .map(|e| {
            (
                e.payload["by"].as_str().unwrap_or("someone").to_string(),
                e.payload["with"].as_str().unwrap_or("memory").to_string(),
            )
        })
}

/// What the creature has seen in its room since it was named: arrivals, speech, digs.
fn recent(world: &World, creature: EntityId, room: Option<EntityId>) -> Vec<String> {
    let Some(room) = room else { return Vec::new() };
    let all = world.recent_events(2000);
    let named_at = all
        .iter()
        .rev()
        .find(|e| {
            e.kind == mind::TIER
                && e.payload["creature"] == creature.0
                && e.payload["to"].as_u64() >= Some(1)
        })
        .map(|e| e.seq.0)
        .unwrap_or(0);
    let citizens_here = |e: &foxing_log::Event| -> Option<String> {
        if e.kind != "store.entity_moved" || e.payload["parent"] != room.0 {
            return None;
        }
        let id = EntityId(e.payload["id"].as_u64()?);
        plain_str(world, id, CITIZEN, "name").map(str::to_string)
    };
    let mut out: Vec<String> = Vec::new();
    let mut seen_names: BTreeMap<String, ()> = BTreeMap::new();
    for e in all.iter().filter(|e| e.seq.0 > named_at).rev() {
        if out.len() >= 4 {
            break;
        }
        if let Some(n) = citizens_here(e) {
            if seen_names.insert(n.clone(), ()).is_none() {
                out.push(format!("{n} came"));
            }
        } else if e.kind == SAY
            && e.payload["room"] == room.0
            && e.payload["creature"].is_null()
            // Said to the room, or to this creature: not what someone told another.
            && match e.payload["to"].as_str() {
                None => true,
                Some(to) => {
                    // To this creature, or to a citizen: yes. To another creature: no.
                    let mine = mind::display(world, creature);
                    mine.eq_ignore_ascii_case(to)
                        || mind::name(world, creature).is_some_and(|n| n.eq_ignore_ascii_case(to))
                        || !world.store.entities().any(|c| {
                            c.kind == "creature"
                                && c.id != creature
                                && (mind::display(world, c.id).eq_ignore_ascii_case(to)
                                    || mind::name(world, c.id)
                                        .is_some_and(|n| n.eq_ignore_ascii_case(to)))
                        })
                }
            }
        {
            let from = e.payload["from"].as_str().unwrap_or("someone");
            let text = e.payload["text"].as_str().unwrap_or("");
            let text = if text.chars().count() > 60 {
                // Cut on a word, and say so.
                let cut: String = text.chars().take(60).collect();
                let at = cut.rfind(' ').unwrap_or(cut.len());
                format!("{}…", &cut[..at])
            } else {
                text.to_string()
            };
            out.push(format!("{from} said, \"{text}\""));
        } else if e.kind == super::building::REMEMBERED && e.payload["creature"] == creature.0 {
            out.push(format!(
                "{} gave me the memory of {}",
                e.payload["by"].as_str().unwrap_or("someone"),
                e.payload["with"].as_str().unwrap_or("something")
            ));
        }
    }
    out.reverse();
    out
}

/// A chunk's source as it is spoken: a sheaf says how many stories it folds, not
/// every gist (the list spliced itself into the middle of a creature's answer).
pub(crate) fn short_source(source: &str) -> String {
    if source.starts_with("a sheaf of ") {
        if let Some((head, _)) = source.split_once(':') {
            return head.to_string();
        }
    }
    // The word that made a born creature is the word under its others (secret.rs):
    // recited whole, the hunt was four tells and one packed story. The carrier
    // knows it, and the birth is on the public record.
    if source.starts_with("the word that made it") {
        return "the word that made it, which it keeps under the others".to_string();
    }
    source.to_string()
}

/// A line of the creature's own, drawn from what it holds beside the memory just
/// given (`except`): another story it keeps, a creature it remembers, something a
/// stranger brought. Empty when it holds nothing else. Which memory is by the
/// creature and how many it holds, so two creatures told the same story answer
/// differently, and the same creature differently as it fills.
pub fn line_of_its_own(world: &World, creature: EntityId, except: EntityId) -> String {
    let Some(window) = mind::window_of(world, creature) else {
        return String::new();
    };
    let now = world.now();
    // Nor one that is the same story told before (a creature once answered a story
    // with "one I am losing" and quoted the opening it had just been given).
    let same = plain_str(world, except, CHUNK, "source")
        .unwrap_or("")
        .to_string();
    let mut others: Vec<(EntityId, String, f64)> = world
        .store
        .children(window)
        .filter(|e| e.kind == "chunk" && e.id != except)
        .filter_map(|e| {
            let source = plain_str(world, e.id, CHUNK, "source")?.to_string();
            if !same.is_empty() && source == same {
                return None;
            }
            let value = world
                .store
                .read_field(e.id, CHUNK, "value", now)
                .ok()?
                .as_f64()?;
            Some((e.id, source, value))
        })
        .collect();
    if others.is_empty() {
        return " It is the first thing I have been given to keep.".to_string();
    }
    others.sort_by_key(|(id, _, _)| id.0);
    let pick = (creature.0 as usize + others.len()) % others.len();
    let (_, source, value) = &others[pick];
    let faded = *value < 5.0;
    // A sheaf first: its gist list holds "X's story of …", and the branch below
    // spliced the whole list into the sentence.
    if source.starts_with("a sheaf of ") {
        let head = short_source(source);
        return format!(
            " It goes in with {head} I have mostly let go of; they will keep each other."
        );
    }
    if let Some(gist) = source.strip_prefix("a story: \"") {
        let gist = gist.trim_end_matches('"');
        return if faded {
            format!(" It puts me in mind of one I am losing: \"{gist}\"")
        } else {
            format!(" It puts me in mind of one I keep: \"{gist}\"")
        };
    }
    if let Some((teller, rest)) = source.split_once("'s story of ") {
        return format!(" {teller} told me one like it once: {rest}.");
    }
    if source.starts_with("the word that made it") {
        return " I was made of a word; I know what it is to be given one.".to_string();
    }
    if source.starts_with("the ") || source.starts_with("a ") || source.starts_with("someone") {
        return if faded {
            format!(" I will set it beside {source}, while I still have that.")
        } else {
            format!(" I will set it beside {source}; they will get on.")
        };
    }
    // A creature's memory, given by a keeper: "hollow", "wisp".
    format!(" It goes in with the memory of {source}, which is mostly quiet now.")
}
