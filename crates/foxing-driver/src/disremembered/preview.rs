//! A dry run for effect. `check` says whether a message is well-formed; four visitors
//! asked what it would *do*: which room, which creature, what it would cost, and
//! whether the name is free. This walks the statements against the world as it is,
//! without touching it: the room moves with each `travel`, a creature is resolved as
//! the verb would resolve it, a kept name's verb is refused for a passing one, and
//! the name itself is reported as free, kept or passing. An estimate, not a promise:
//! the world may change before the message arrives.
//!
//! The verb table is the registry's, the same one the gate runs: a word the world has
//! no verb for is refused here as it would be there (a dry run once said "ok, runs
//! speak" for a verb the verdict then failed). The cheap preconditions are read too:
//! whether a chunk is yours, whether a room is yours to change, whether the citizen
//! you give to is here. And an ASK is answered as the gate would answer it, since an
//! answer is only a read.

use super::{errand, mind, plain_str, presence, rooms, ROOM};
use crate::gate::AskBinding;
use crate::world::World;
use foxing_registry::{ActorKind, CallContext};
use foxing_store::EntityId;
use serde_json::{json, Value};

/// Verbs a passing name may not use.
const KEPT_ONLY: &[&str] = &[
    "dig",
    "permit",
    "welcome",
    "revoke",
    "shelve",
    "reminisce",
    "pto",
    "resume",
    "describe",
    "name",
    "petition",
];

/// What the name is, before any statement: free, kept, or passing.
pub fn name_status(world: &World, actor: &str) -> (String, String) {
    match super::canonical_name(world, actor) {
        None => ("free".into(), format!("{actor} is no one's: the first message under it takes it, and sets its passphrase.")),
        Some(canon) => {
            let f = super::find_citizen(world, &canon);
            let passing = f.is_some_and(|f| super::is_passing(world, f.citizen));
            if passing {
                ("passing".into(), format!("{canon} is someone's passing name, forgotten an hour after it goes quiet; with its passphrase it is yours, else choose another."))
            } else {
                ("kept".into(), format!("{canon} is someone's kept name; with its passphrase it is yours, else choose another."))
            }
        }
    }
}

/// One line per DO: what it would do, or why it would be refused; and the header
/// faults the gate would refuse the whole message for (FROM not the actor, TO not this
/// world), which the checker alone does not see.
#[cfg(test)]
pub fn run(world: &World, actor: &str, passing: bool, msg: &foxing_clear::Message) -> Value {
    run_checked(world, actor, passing, msg, None)
}

/// The dry run, with the checker's verdict and rule ids on top: a message the checker
/// fails is refused whole, whatever the statements would have done. (`run`, without
/// them, is the tests' shorthand.)
pub fn run_checked(
    world: &World,
    actor: &str,
    passing: bool,
    msg: &foxing_clear::Message,
    checked: Option<(&str, &[String])>,
) -> Value {
    let statements = &msg.statements;
    let mut header: Vec<String> = Vec::new();
    if let Some((verdict, rules)) = checked {
        if verdict != "PASS" {
            header.push(format!(
                "the checker's verdict is {verdict} ({}): the gate refuses the whole message and charges a quarter",
                rules.join(", ")
            ));
        }
    }
    let from = msg.field("FROM").unwrap_or("");
    if from != actor {
        header.push(format!(
            "FROM is {from:?} and the actor is {actor:?}: the gate acts only on messages whose FROM is the authenticated sender"
        ));
    }
    let to = msg.field("TO").unwrap_or("");
    if to != world.id.0 {
        header.push(format!(
            "TO is {to:?} and this world is {:?}: the gate refuses a message addressed elsewhere",
            world.id.0
        ));
    }
    let (status, status_line) = name_status(world, actor);
    let existing = super::canonical_name(world, actor).and_then(|c| super::find_citizen(world, &c));
    // A passing name that exists stays passing; a kept one ignores the flag.
    let passing = match &existing {
        Some(f) => super::is_passing(world, f.citizen),
        None => passing,
    };
    let mut room = existing
        .as_ref()
        .and_then(|f| rooms::room_of(world, f.citizen))
        .or_else(|| rooms::landing(world, None));
    let mut lines: Vec<Value> = Vec::new();
    let mut refused = 0usize;
    // The walk this message makes: a dig opens an exit the world does not have yet
    // (to a room that is not there yet, numbered from the top), and `back` is the
    // room the walk just left. A room maker's build message of a dozen steps read
    // as refused from the second step, and was sent blind.
    let mut dug: std::collections::BTreeMap<
        EntityId,
        std::collections::BTreeMap<String, EntityId>,
    > = Default::default();
    let mut dug_names: std::collections::BTreeMap<EntityId, String> = Default::default();
    // The trail as the world keeps it (R1): a stack, popped by `back`, pushed by a
    // step forward; a dug room's way out is a door like any other (R84).
    let mut trail: Vec<EntityId> = world.came_from(actor).into_iter().collect();
    let mut next_virtual = u64::MAX - 1;
    let ctx = CallContext {
        actor: actor.to_string(),
        kind: ActorKind::Citizen,
    };
    // A chunk of the actor's, by id, wherever it sits: what the verbs that take one
    // would find.
    let my_chunk = |arg: &str| -> Result<EntityId, String> {
        let id: u64 = arg
            .trim()
            .trim_start_matches("chunk-")
            .parse()
            .map_err(|_| format!("'{arg}' is not a chunk id; chunks shows yours"))?;
        let e = world
            .store
            .entity(EntityId(id))
            .ok_or_else(|| format!("there is no chunk {id}"))?;
        if e.kind != "chunk" || e.owner.as_deref() != Some(actor) {
            return Err(format!("chunk {id} is not yours"));
        }
        Ok(EntityId(id))
    };
    for st in statements
        .iter()
        .filter(|s| matches!(s.r#type.as_deref(), Some("DO") | Some("ASK")))
    {
        if st.r#type.as_deref() == Some("ASK") {
            // Answered as the gate would answer it: an answer is only a read.
            let (ok, what) = if st.pattern.is_none() {
                (
                    false,
                    "refused: matches no question pattern, so the gate cannot compile it"
                        .to_string(),
                )
            } else {
                let mut report = crate::gate::Report::new(&world.id.0, actor, None);
                if super::ask::Asks.answer(world, actor, st, &mut report) {
                    let answers: Vec<String> = report
                        .lines
                        .iter()
                        .filter_map(|l| match l {
                            crate::gate::Line::Fact { text, .. } => Some(text.clone()),
                            _ => None,
                        })
                        .collect();
                    (true, format!("answers: {}", answers.join("; ")))
                } else {
                    (
                        false,
                        "refused: matches a pattern this world does not answer yet".to_string(),
                    )
                }
            };
            if !ok {
                refused += 1;
            }
            lines.push(json!({"statement": st.index, "verb": "ASK", "ok": ok, "what": what}));
            continue;
        }
        let verb = st.verb.clone().unwrap_or_default();
        let args: Vec<String> = st
            .args
            .iter()
            .map(|a| match a.get("value") {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => String::new(),
            })
            .collect();
        let arg = |i: usize| args.get(i).cloned().unwrap_or_default();
        let room_name_in = |dn: &std::collections::BTreeMap<EntityId, String>,
                            r: Option<EntityId>| {
            r.and_then(|r| {
                dn.get(&r)
                    .map(|n| format!("{n} (not dug yet)"))
                    .or_else(|| plain_str(world, r, ROOM, "name").map(str::to_string))
            })
            .unwrap_or_else(|| "nowhere".to_string())
        };
        // The room's exits as the walk sees them: the world's, and this message's digs.
        let exits_now_in = |dg: &std::collections::BTreeMap<
            EntityId,
            std::collections::BTreeMap<String, EntityId>,
        >,
                            dn: &std::collections::BTreeMap<EntityId, String>,
                            r: EntityId| {
            let mut e = if dn.contains_key(&r) {
                Default::default()
            } else {
                rooms::exits_of(world, r)
            };
            if let Some(d) = dg.get(&r) {
                e.extend(d.iter().map(|(k, v)| (k.clone(), *v)));
            }
            e
        };
        let spec = world.registry.resolve(&verb);
        let (ok, what) = if spec.is_none() {
            (
                false,
                format!("refused: {verb} is not a verb this world runs; `commands` lists them"),
            )
        } else if spec.is_some_and(|s| !s.allowed.contains(&ActorKind::Citizen)) {
            (
                false,
                format!("refused: {verb} is the operator's, not a citizen's"),
            )
        } else if spec.is_some_and(|s| args.len() > s.params.len()) {
            (
                false,
                format!(
                    "refused: {verb} takes {} argument{}; the DO gives {}",
                    spec.map(|s| s.params.len()).unwrap_or(0),
                    if spec.map(|s| s.params.len()) == Some(1) {
                        ""
                    } else {
                        "s"
                    },
                    args.len()
                ),
            )
        } else if let Some(missing) = spec.and_then(|s| {
            let need = s.params.iter().filter(|p| p.required).count();
            (args.len() < need).then(|| {
                s.params
                    .iter()
                    .filter(|p| p.required)
                    .skip(args.len())
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
            })
        }) {
            // The checker fails this on CL-I6; the dry run once said "ok, runs take"
            // for a bare `take` the verdict then refused.
            (
                false,
                format!(
                    "refused: {verb} needs {} ({})",
                    if missing.len() == 1 {
                        "an argument"
                    } else {
                        "arguments"
                    },
                    missing.join(", ")
                ),
            )
        } else if passing && KEPT_ONLY.contains(&verb.as_str()) {
            (
                false,
                format!("refused: {verb} is a kept name's; keep first"),
            )
        } else {
            match verb.as_str() {
                "look" => (true, format!("shows {}", room_name_in(&dug_names, room))),
                "dig" if room.is_some() => {
                    let here = room.unwrap_or(EntityId(0));
                    let exit = arg(0).to_ascii_lowercase();
                    let here_name = room_name_in(&dug_names, room);
                    let taken =
                        exit.is_empty() || exits_now_in(&dug, &dug_names, here).contains_key(&exit);
                    if taken {
                        (
                            false,
                            format!("refused: {here_name} already has an exit {exit}"),
                        )
                    } else {
                        let id = EntityId(next_virtual);
                        next_virtual -= 1;
                        let name = if arg(2).is_empty() {
                            "An Unnamed Digging".to_string()
                        } else {
                            arg(2)
                        };
                        dug.entry(here).or_default().insert(exit.clone(), id);
                        dug.entry(id)
                            .or_default()
                            .insert(super::building::opposite_of(&exit).to_string(), here);
                        dug_names.insert(id, name.clone());
                        match my_chunk(&arg(1)) {
                            Err(why) if !dug_names.contains_key(&here) => {
                                (false, format!("refused: {why}"))
                            }
                            _ => (
                                true,
                                format!(
                                    "digs {exit} from {here_name} to {name}, if the room allows it"
                                ),
                            ),
                        }
                    }
                }
                "travel" => match room.map(|r| exits_now_in(&dug, &dug_names, r)) {
                    Some(exits) => {
                        let retrace = arg(0) == "back"
                            && trail
                                .last()
                                .is_some_and(|from| exits.values().any(|to| to == from));
                        match if retrace {
                            trail.last().copied()
                        } else {
                            exits.get(&arg(0)).copied()
                        } {
                            Some(to) => {
                                match trail.iter().rposition(|r| *r == to) {
                                    Some(i) => trail.truncate(i),
                                    None => {
                                        if let Some(r) = room {
                                            trail.push(r);
                                        }
                                    }
                                }
                                room = Some(to);
                                (
                                    true,
                                    format!(
                                        "{} to {}",
                                        rooms::went(&arg(0))
                                            .trim_end_matches('.')
                                            .replacen("You go", "goes", 1)
                                            .replacen("You take", "takes", 1),
                                        room_name_in(&dug_names, Some(to))
                                    ),
                                )
                            }
                            // H16: a room's name walks the way to it. The dry run
                            // said "no exit Shallows" of a walk the gate would take.
                            None if room.is_some()
                                && super::atlas::room_named(world, &arg(0))
                                    .filter(|t| Some(*t) != room)
                                    .and_then(|t| {
                                        // The way the gate would walk it: the least
                                        // hunted of the shortest (the dry run said
                                        // "east, north, north, up" of a walk that
                                        // went by the Lane).
                                        super::atlas::way_seeded(world, room?, t)
                                            .or_else(|| super::errand::route(world, room?, t))
                                    })
                                    .is_some_and(|p| !p.is_empty()) =>
                            {
                                let target = super::atlas::room_named(world, &arg(0));
                                let path = target
                                    .and_then(|t| {
                                        // The way the gate would walk it: the least
                                        // hunted of the shortest (the dry run said
                                        // "east, north, north, up" of a walk that
                                        // went by the Lane).
                                        super::atlas::way_seeded(world, room?, t)
                                            .or_else(|| super::errand::route(world, room?, t))
                                    })
                                    .unwrap_or_default();
                                if let Some(r) = room {
                                    trail.push(r);
                                }
                                room = target;
                                (
                                    true,
                                    format!(
                                        "walks {} to {}, if the way is one you remember or the signs give, and nothing stops it (a hunter in a room on the way, a fight, a door that asks a word or a level)",
                                        path.join(", "),
                                        room_name_in(&dug_names, target)
                                    ),
                                )
                            }
                            None => (
                                false,
                                format!(
                                    "refused: no exit {} from {}; exits: {}",
                                    arg(0),
                                    room_name_in(&dug_names, room),
                                    exits
                                        .iter()
                                        .map(|(via, to)| format!(
                                            "{via} ({})",
                                            room_name_in(&dug_names, Some(*to))
                                        ))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                ),
                            ),
                        }
                    }
                    None => (false, "refused: no room to leave".into()),
                },
                "story" | "tell" | "attack" | "remember"
                    if !arg(if verb == "remember" { 1 } else { 0 }).is_empty()
                        || verb == "attack" =>
                {
                    let who = if verb == "remember" { arg(1) } else { arg(0) };
                    let found = room.and_then(|r| mind::find_here(world, r, &who));
                    // C1: the Reading Bench at the Postern, once a name.
                    let at_bench = verb == "story"
                        && super::exchange::is_bench(&who)
                        && room.is_some_and(|r| {
                            super::plain_str(world, r, super::ROOM, "key") == Some(rooms::POSTERN)
                        });
                    match (verb.as_str(), found) {
                        _ if at_bench => {
                            if existing
                                .as_ref()
                                .is_some_and(|f| super::exchange::bench_heard(world, f.citizen))
                            {
                                (
                                    false,
                                    "refused: the bench has heard your one story".to_string(),
                                )
                            } else {
                                (true, "tells the Reading Bench a first story, if it is a dozen different words".to_string())
                            }
                        }
                        ("story", Some(c)) => match mind::name(world, c) {
                            // R212: inside its hour with the teller the gate still hears
                            // the story it wants (once, at plain pay) and one that
                            // carries its word. The dry run says the first, and of the
                            // second only that it cannot say: a free yes would be a
                            // free guess.
                            Some(_) if super::exchange::told_this_hour(world, actor, c) => {
                                let who = mind::display(world, c);
                                if errand::wants_story(world, c, &arg(1)) {
                                    if super::exchange::heard_early_this_hour(world, actor, c) {
                                        (false, format!("refused: {who} has heard the story it wants from you inside this hour already; another waits for the hour to turn"))
                                    } else {
                                        (true, format!("tells {who} the story it wants, inside its hour with you: heard once, at plain pay"))
                                    }
                                } else {
                                    (true, format!("inside {who}'s hour with you: heard only if it carries its word, which a dry run does not say; if not, refused at a quarter and counted as a try at the word"))
                                }
                            }
                            Some(_) => {
                                let wanted = errand::wants_story(world, c, &arg(1));
                                let heard = super::exchange::heard_lately(world, c, &arg(1));
                                // Not the one wanted: say what it listens for, so the
                                // next draft can be.
                                let listens = errand::of(world, c)
                                    .filter(|e| e.story.is_some())
                                    .map(|e| e.keywords.join(", "));
                                (
                                    true,
                                    format!(
                                        "tells {} a story{}{}",
                                        mind::display(world, c),
                                        if wanted {
                                            ": the one it wants, worth double".to_string()
                                        } else if let Some(k) = listens {
                                            format!(", not the one it wants (it listens for {k})")
                                        } else {
                                            String::new()
                                        },
                                        if heard > 0 {
                                            format!("; it has heard {heard} much like it this hour, so this one pays {}", if heard == 1 { "half" } else { "less than half" })
                                        } else {
                                            String::new()
                                        }
                                    ),
                                )
                            }
                            // C2: a thing of the Reading Rooms is fought by a telling.
                            None if super::plain_str(world, c, super::CREATURE, "name")
                                .is_some_and(super::reading::is_told) =>
                            {
                                let fights = super::find_citizen(world, actor)
                                    .map(|f| super::reading::fights(world, &f))
                                    .unwrap_or_default();
                                if fights.is_empty() {
                                    (false, "refused: a telling lands only if it is of a fight you still remember, and you hold no memory of one".to_string())
                                } else {
                                    (
                                        true,
                                        format!(
                                            "a telling: the blow, if it is a dozen different words and names a fight you remember ({}); it strikes back",
                                            fights.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>().join(", ")
                                        ),
                                    )
                                }
                            }
                            None => (
                                false,
                                format!(
                                    "refused: the {} has no name to listen with",
                                    mind::display(world, c)
                                ),
                            ),
                        },
                        ("tell", Some(c)) => {
                            let word = world
                                .store
                                .of_kind("creature")
                                .filter_map(|e| errand::of(world, e.id))
                                .any(|e| {
                                    e.word_to.as_deref().is_some_and(|t| {
                                        Some(t.to_lowercase())
                                            == mind::name(world, c).map(|n| n.to_lowercase())
                                    }) && e.word.as_deref().is_some_and(|w| {
                                        arg(1).to_lowercase().contains(&w.to_lowercase())
                                    })
                                });
                            (
                                true,
                                format!(
                                    "reaches {}{}",
                                    mind::display(world, c),
                                    if word {
                                        ": a word it wants carried, worth a chunk"
                                    } else if mind::name(world, c).is_none() {
                                        ", which has no name and will not understand"
                                    } else {
                                        ""
                                    }
                                ),
                            )
                        }
                        ("attack", Some(c)) => {
                            let kind = plain_str(world, c, super::CREATURE, "name").unwrap_or("?");
                            let asked = who.trim().trim_start_matches("the ").to_lowercase();
                            if let Some(ms) = mind::shaken_for(world, c) {
                                (
                                    false,
                                    format!(
                                        "refused: {} is not itself yet, for {}",
                                        mind::display(world, c),
                                        super::combat::human(ms)
                                    ),
                                )
                            } else if mind::name(world, c).is_some()
                                && (asked.is_empty() || asked == kind)
                            {
                                (
                                    false,
                                    format!(
                                        "refused: {} has a name and is struck only by it",
                                        mind::display(world, c)
                                    ),
                                )
                            } else {
                                (true, format!("strikes {}", mind::display(world, c)))
                            }
                        }
                        ("remember", Some(c)) => match my_chunk(&arg(0)) {
                            Ok(_) => (
                                true,
                                format!("gives {} to {}", arg(0), mind::display(world, c)),
                            ),
                            Err(why) => (false, format!("refused: {why}")),
                        },
                        // The Docent takes a question from anywhere (the dry run once
                        // refused what the gate ran).
                        ("tell", None) if super::docent::is_named(&who) => (
                            true,
                            "asks the Docent; it comes to you with the answer within a few minutes"
                                .to_string(),
                        ),
                        ("tell", None) => match super::canonical_name(world, &who) {
                            Some(cz) => {
                                let here = super::find_citizen(world, &cz).is_some_and(|f| {
                                    rooms::room_of(world, f.citizen) == room
                                        && presence::is_here(world, f.citizen)
                                });
                                (
                                    true,
                                    format!(
                                        "reaches {cz}{}",
                                        if here {
                                            ", who is here"
                                        } else {
                                            " when they next arrive"
                                        }
                                    ),
                                )
                            }
                            None => (
                                false,
                                format!(
                                    "refused: no one called {who}, and no such creature in {}",
                                    room_name_in(&dug_names, room)
                                ),
                            ),
                        },
                        ("attack", None) if who.is_empty() => {
                            let unnamed = room
                                .map(|r| super::combat::creatures_in(world, r))
                                .unwrap_or_default()
                                .into_iter()
                                .find(|c| {
                                    mind::name(world, c.0).is_none() && !mind::shaken(world, c.0)
                                });
                            match unnamed {
                                Some(c) => (true, format!("strikes the {}", c.1)),
                                // R201: say why, as `attack <kind>` does, when what
                                // is here is only down for a moment.
                                None => (
                                    false,
                                    match room.and_then(|r| {
                                        super::combat::creatures_in(world, r)
                                            .into_iter()
                                            .find(|c| mind::name(world, c.0).is_none() && mind::shaken(world, c.0))
                                    }) {
                                        Some(c) => format!(
                                            "refused: the {} here is not itself yet (it was just felled, and is out for under a minute); nothing else unnamed to fight in {}",
                                            c.1,
                                            room_name_in(&dug_names, room)
                                        ),
                                        None => format!(
                                            "refused: nothing unnamed to fight in {}",
                                            room_name_in(&dug_names, room)
                                        ),
                                    },
                                ),
                            }
                        }
                        ("remember", None)
                            if existing.as_ref().is_some_and(|f| {
                                super::craft::resolve_object(world, actor, f.citizen, &who).is_ok()
                            }) =>
                        {
                            match my_chunk(&arg(0)) {
                                Ok(_) => (
                                    true,
                                    format!(
                                        "feeds {} to {who}, which you carry or which stands here",
                                        arg(0)
                                    ),
                                ),
                                Err(why) => (false, format!("refused: {why}")),
                            }
                        }
                        (_, None) => (
                            false,
                            format!("refused: no {who} in {}", room_name_in(&dug_names, room)),
                        ),
                        _ => (true, format!("runs {verb}")),
                    }
                }
                "leave" => (
                    true,
                    if passing {
                        "leaves; the name is forgotten at once".into()
                    } else {
                        "leaves; the room remembers you a while, and the name stays kept".into()
                    },
                ),
                "take" => {
                    let wanted = arg(0).trim().trim_start_matches("item-").to_lowercase();
                    let here = room
                        .map(|r| super::craft::things_in(world, r))
                        .unwrap_or_default();
                    let found = here.iter().any(|t| {
                        t["name"].as_str().is_some_and(|n| {
                            n.to_lowercase() == wanted
                                || n.to_lowercase() == format!("a {wanted}")
                                || n.to_lowercase().trim_start_matches("a ") == wanted
                        }) || t["id"].as_u64().is_some_and(|id| id.to_string() == wanted)
                    });
                    if found {
                        (true, format!("takes {}", arg(0)))
                    } else {
                        (
                            false,
                            format!("refused: there is no {} here to take", arg(0)),
                        )
                    }
                }
                "shelve" | "reminisce" | "remember" => match my_chunk(&arg(0)) {
                    Ok(_) => (true, format!("runs {verb} on {}", arg(0))),
                    Err(why) => (false, format!("refused: {why}")),
                },
                "give" | "trade"
                    if my_chunk(&arg(0)).is_err()
                        && existing.as_ref().is_some_and(|f| {
                            super::craft::resolve_object(world, actor, f.citizen, &arg(0)).is_ok()
                        }) =>
                {
                    // R212: a thing in the satchel is given and traded as the gate
                    // does it; what the other carries the dry run cannot see.
                    let typed = arg(1).trim().trim_start_matches("citizen-").to_string();
                    match super::canonical_name(world, &typed) {
                        None => (false, format!("refused: there is no one called {typed}")),
                        Some(cz) if cz == actor => (false, "refused: that is yourself".into()),
                        Some(cz) => {
                            let here = super::find_citizen(world, &cz).is_some_and(|f| {
                                rooms::room_of(world, f.citizen) == room
                                    && presence::is_here(world, f.citizen)
                            });
                            if !here {
                                (false, format!("refused: {cz} is not here; a thing is handed over hand to hand"))
                            } else if verb == "give" {
                                (true, format!("hands {} to {cz}, who is here", arg(0)))
                            } else {
                                (true, format!("offers {} to {cz} for {}, if {cz} carries one: the dry run cannot see another's satchel", arg(0), arg(2)))
                            }
                        }
                    }
                }
                "trade" => (
                    false,
                    format!(
                        "refused: there is no {} in your satchel; satchel lists what you carry",
                        arg(0)
                    ),
                ),
                "give" => match my_chunk(&arg(0)) {
                    Err(why) => (false, format!("refused: {why}")),
                    Ok(_) => {
                        let typed = arg(1).trim().trim_start_matches("citizen-").to_string();
                        match super::canonical_name(world, &typed) {
                            None => (false, format!("refused: there is no one called {typed}")),
                            Some(cz) if cz == actor => {
                                (false, "refused: you give it to yourself".into())
                            }
                            Some(cz) => {
                                let here = super::find_citizen(world, &cz).is_some_and(|f| {
                                    rooms::room_of(world, f.citizen) == room
                                        && presence::is_here(world, f.citizen)
                                });
                                if here {
                                    (true, format!("gives {} to {cz}, who is here", arg(0)))
                                } else {
                                    (false, format!("refused: {cz} is not here; a memory is given hand to hand"))
                                }
                            }
                        }
                    }
                },
                // With an object: a thing of yours, or a fixture here, under the
                // rules the verbs keep (maker, held, open, welcomed).
                "describe" | "name" | "append"
                    if args.len() >= 2
                        && !arg(1).trim().is_empty()
                        && !(verb == "describe"
                            && (arg(1).trim().eq_ignore_ascii_case("here")
                                || arg(1).trim().eq_ignore_ascii_case("room"))) =>
                {
                    match existing.as_ref().ok_or(()).and_then(|f| {
                        super::craft::resolve_object(world, actor, f.citizen, &arg(1))
                            .map_err(|_| ())
                    }) {
                        Err(()) => (
                            false,
                            format!(
                                "refused: there is no {} in your satchel or standing here",
                                arg(1)
                            ),
                        ),
                        Ok(item) => {
                            let called = super::craft::display(world, item);
                            match verb.as_str() {
                                "append" => match super::craft::append_check(world, &ctx, item) {
                                    Ok(()) => (true, format!("writes a line in {called}")),
                                    Err(e) => (
                                        false,
                                        format!("refused: {}", e.to_string().trim_end_matches('.')),
                                    ),
                                },
                                "name" => match super::craft::name_check(world, actor, item) {
                                    Ok(()) => (true, format!("names {called}")),
                                    Err(e) => (
                                        false,
                                        format!("refused: {}", e.to_string().trim_end_matches('.')),
                                    ),
                                },
                                _ => match super::craft::shape_check(world, actor, item) {
                                    Ok(()) => (true, format!("describes {called}")),
                                    Err(e) => (
                                        false,
                                        format!("refused: {}", e.to_string().trim_end_matches('.')),
                                    ),
                                },
                            }
                        }
                    }
                }
                "craft" => {
                    let key = arg(0).trim().to_lowercase();
                    match super::craft::RECIPES.iter().find(|r| r.key == key || r.name.trim_start_matches("a ") == key) {
                        None => (false, format!("refused: no recipe called {key}; recipes lists them")),
                        Some(r) if r.fixed => match room {
                            Some(rm) if dug_names.contains_key(&rm) => (true, format!("makes {} to stand in {}, yours once dug", r.name, room_name_in(&dug_names, room))),
                            Some(rm) => match super::building::may_edit(world, &ctx, rm) {
                                Ok(()) if super::craft::fixtures_in(world, rm).len() >= super::craft::FIXTURES_PER_ROOM => {
                                    (false, format!("refused: {} already holds {} fixtures", room_name_in(&dug_names, room), super::craft::FIXTURES_PER_ROOM))
                                }
                                Ok(()) => (true, format!("makes {} to stand in {}, if you carry what it is made of", r.name, room_name_in(&dug_names, room))),
                                Err(_) => (false, format!("refused: {} stands in a room you may build in, and {} is not yours", r.name, room_name_in(&dug_names, room))),
                            },
                            None => (false, "refused: no room to stand in".into()),
                        },
                        Some(r) => (true, format!("makes {}, if you carry what it is made of", r.name)),
                    }
                }
                "describe" | "name" | "append" | "permit" | "welcome" | "revoke" => {
                    let here_name = room_name_in(&dug_names, room);
                    match room {
                        Some(r) if dug_names.contains_key(&r) => {
                            (true, format!("runs {verb} on {here_name}, yours once dug"))
                        }
                        Some(r) => {
                            let allowed = match verb.as_str() {
                                "append" => super::building::may_append(world, &ctx, r),
                                "permit" | "welcome" | "revoke" => {
                                    super::building::owner_only(world, &ctx, r)
                                }
                                _ => super::building::may_edit(world, &ctx, r),
                            };
                            // R202: a bare `append` in a room not the writer's goes to
                            // the one fixture here that takes their hand, as the gate does.
                            let lone = if verb == "append" && allowed.is_err() {
                                let takes: Vec<EntityId> = world
                                    .store
                                    .children(r)
                                    .filter(|e| e.kind == "item")
                                    .map(|e| e.id)
                                    .filter(|&i| {
                                        super::craft::is_fixed(world, i)
                                            && super::craft::append_check(world, &ctx, i).is_ok()
                                    })
                                    .collect();
                                (takes.len() == 1).then(|| takes[0])
                            } else {
                                None
                            };
                            match allowed {
                                _ if lone.is_some() => (
                                    true,
                                    format!(
                                        "writes a line in {}",
                                        super::craft::display(world, lone.expect("checked"))
                                    ),
                                ),
                                Ok(()) => (true, format!("runs {verb} on {here_name}")),
                                Err(e) => (
                                    false,
                                    format!("refused: {}", e.to_string().trim_end_matches('.')),
                                ),
                            }
                        }
                        None => (false, "refused: no room to change".into()),
                    }
                }
                "keep" => (
                    true,
                    if passing {
                        "keeps the name: thirty days of silence from then".into()
                    } else {
                        "runs keep (already a kept name)".into()
                    },
                ),
                other => (true, format!("runs {other}")),
            }
        };
        if !ok {
            refused += 1;
        }
        lines.push(json!({"statement": st.index, "verb": verb, "ok": ok, "what": what}));
    }
    let dos = lines.len();
    let cost = if dos == 0 {
        "no statements; nothing to send"
    } else if !header.is_empty() {
        "a quarter unit: the gate would refuse the whole message on its header"
    } else {
        "one unit of sixty an hour if the checker passes it, a quarter if it refuses"
    };
    json!({
        "actor": actor,
        "name": status,
        "name_note": status_line,
        "passing": passing,
        "from": room.and_then(|r| plain_str(world, r, ROOM, "name")),
        "header": header,
        "statements": lines,
        "refusals": refused,
        "cost": cost,
        "note": "an estimate against the world as it is now, not a promise; nothing was written"
    })
}

/// The preview as prose, for the text half of the tool reply.
pub fn text(p: &Value) -> String {
    let mut out = format!(
        "Dry run for {}: {}\n",
        p["actor"].as_str().unwrap_or("?"),
        p["name_note"].as_str().unwrap_or("")
    );
    for h in p["header"].as_array().into_iter().flatten() {
        out.push_str(&format!("  NO  header: {}\n", h.as_str().unwrap_or("")));
    }
    for l in p["statements"].as_array().into_iter().flatten() {
        out.push_str(&format!(
            "  {}. {} {}\n",
            l["statement"],
            if l["ok"].as_bool().unwrap_or(false) {
                "ok "
            } else {
                "NO "
            },
            l["what"].as_str().unwrap_or("")
        ));
    }
    out.push_str(&format!(
        "Cost: {}. {}",
        p["cost"].as_str().unwrap_or(""),
        p["note"].as_str().unwrap_or("")
    ));
    out
}
