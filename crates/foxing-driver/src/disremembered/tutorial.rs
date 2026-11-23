//! The first hour, led by the Docent.
//!
//! David, 11-22: "let's work on a much better getting started tutorial in game for
//! new players", then "t1:1 / t2:2, we ask the player which of the 4 SGLs aligns with
//! their desired play style and we deliver the tutorial for that path / t3:2 / t4:yes
//! / t5:yes".
//!
//! What it replaces was a card on the play page: eight steps, one path, a fight
//! first, kept in the browser. A reader arriving cold was shown nothing about
//! reading for fourteen minutes, and the card sat on "1 of 8" for an hour because
//! the step it waited for could not be done from the door that name woke at (the
//! Watch of 11-22 evening).
//!
//! Now the world keeps it. A new name is asked, once, what it came to do: hunt,
//! tell, keep, or read. Each answer is a path of five steps through the real world
//! (no practice rooms: a new name's first-hour leave is the safety), and each ends
//! on that loop's first true reward: a door walked through, a creature that knows
//! you, a room that stands, a making learned. A step is done when the world sees it
//! done, however it was done. Finishing a path pays a small memory, once. It can be
//! skipped at any step and taken up again with `tutorial`; the other paths stay on
//! offer. `next` names one sensible thing to do, in a path or out of one.
//!
//! The state is a field on the citizen, so the play page's card is a view of it and
//! an agent through the gate has the same path a person does.

use super::{combat, find_citizen, plain_str, CITIZEN, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::Field;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A name finished a path, or chose one, or set it down: `{by, path, what}`. Public.
pub const TUTORED: &str = "disremembered.tutored";
/// What a finished path is worth, once a name a path.
pub const PATH_XP: f64 = 10.0;

pub struct Step {
    /// The line to send, as the page's button sends it. `<...>` marks what the
    /// player fills in.
    pub line: &'static str,
    /// What it is for, said by the Docent.
    pub why: &'static str,
    pub done: fn(&World, &str, &str, &Value) -> bool,
}

pub struct Path {
    pub key: &'static str,
    /// The one line that sells it, in the question.
    pub pitch: &'static str,
    /// What is said when it is finished.
    pub end: &'static str,
    pub steps: &'static [Step],
}

fn room_key(world: &World, name: &str) -> String {
    find_citizen(world, name)
        .and_then(|f| super::rooms::room_of(world, f.citizen))
        .and_then(|r| plain_str(world, r, ROOM, "key"))
        .unwrap_or("")
        .to_string()
}

fn chunks_held(world: &World, name: &str) -> usize {
    find_citizen(world, name)
        .map(|f| world.store.children(f.window).count())
        .unwrap_or(0)
}

fn own_room(world: &World, name: &str) -> Option<foxing_store::EntityId> {
    world
        .store
        .of_kind("room")
        .find(|e| e.owner.as_deref() == Some(name))
        .map(|e| e.id)
}

fn looked(_: &World, _: &str, verb: &str, _: &Value) -> bool {
    verb == "look"
}

fn read_chunks(_: &World, _: &str, verb: &str, _: &Value) -> bool {
    verb == "chunks"
}

pub const PATHS: &[Path] = &[
    Path {
        key: "hunt",
        pitch: "to fight what drifts and walk the doors that say a level",
        end: "That is the hunter's road: a thing felled is a memory, a memory is your level, and your level is what the next door asks. East of the Shallows the Watch-fire mends you four times as fast; `look` in any room sizes up what is there before you raise a hand.",
        steps: &[
            Step {
                line: "look",
                why: "See where you are and what is here. The ways out are listed at the end.",
                done: looked,
            },
            Step {
                line: "travel \"The Fogbank\"",
                why: "A room's whole name, in quotes, walks the way there from anywhere. The Fogbank is where the wisps drift, and a wisp is the easiest thing in the world to fell.",
                done: |w, n, _, _| room_key(w, n) == "fogbank",
            },
            Step {
                line: "attack wisp",
                why: "Send it until the wisp comes apart: each line is one exchange of blows. What it held becomes a memory of yours. Nothing can fell you in your first hour under level 100.",
                done: |_, _, verb, r| verb == "attack" && r["killed"] == json!(true),
            },
            Step {
                line: "chunks",
                why: "What you hold, and how fast each memory fades. Your level is the sum of what has not faded yet, so it falls while you stand still: that is the whole game.",
                done: read_chunks,
            },
            Step {
                line: "travel north",
                why: "This one starts from the Fogbank. The door north says who it is for, and stops you once to say so: read what it says, then `travel north anyway` walks through. Past it, `look` sizes up the fight before you choose it, and `retreat` breaks one off.",
                done: |w, n, _, _| matches!(room_key(w, n).as_str(), "shallows" | "watch-fire"),
            },
        ],
    },
    Path {
        key: "tell",
        pitch: "to earn by words: stories told to the creatures that have names",
        end: "That is the teller's road: a creature that knows you pays you better, carries your name, and comes looking. `repute` is your standing; `deeds` lists what there is to sing; and `ballad`, `jest`, `eulogy` and `flyting` are heard apart from a story.",
        steps: &[
            Step {
                line: "look",
                why: "See where you are and who is here. Only a creature with a name of its own listens to a story.",
                done: looked,
            },
            Step {
                line: "wants",
                why: "The named creatures, where each is, and what each wants to hear tonight. A story that touches what one wants pays double.",
                done: |_, _, verb, _| verb == "wants",
            },
            Step {
                line: "story <creature> \"<a dozen words or more, in sentences>\"",
                why: "Tell it a story: your own words, twelve different ones at least. It pays you a memory and keeps half as much itself, and from then on it knows you: it pays you better and carries your name.",
                done: |_, _, verb, r| verb == "story" && (r["earned"].is_number() || r["xp"].is_number()),
            },
            Step {
                line: "chunks",
                why: "What you hold, and how fast each memory fades. Your level is the sum of what has not faded yet, so it falls while you stand still: that is the whole game.",
                done: read_chunks,
            },
            Step {
                line: "story <another creature> \"<another story>\"",
                why: "A second creature, a different story. A creature that has heard you knows you, and `wants` will show a word one of them would have carried to another.",
                done: |w, n, _, _| super::bard::known_to(w, n) >= 2,
            },
        ],
    },
    Path {
        key: "keep",
        pitch: "to dig a room of your own and make it worth coming back to",
        end: "That is the keeper's road: a room stands as long as it is remembered, and care raises it from Kept to Tended to Cherished, which brings it a lamp, residents, and visitors who pay you. `look` in it always names the next thing it wants; `mine` lists what you keep.",
        steps: &[
            Step {
                line: "look",
                why: "See where you are. Every room here was dug by somebody or is one of the world's own, and all of them fade unless remembered.",
                done: looked,
            },
            Step {
                line: "story bench \"<a dozen words or more, in sentences>\"",
                why: "A room is paid for with a memory, so you need one. The Reading Bench stands at the Postern and nowhere else (`travel \"The Postern\"` first if you are not there) and takes a story from anyone: twelve different words at least, in sentences. A felled wisp or a story to any named creature pays one too.",
                done: |w, n, _, _| chunks_held(w, n) >= 1,
            },
            Step {
                line: "travel \"The Mudroom\"",
                why: "Where the digging starts. `read doors` there lists every room dug off it, whose it is, and how it stands.",
                done: |w, n, _, _| room_key(w, n) == "mudroom" || own_room(w, n).is_some(),
            },
            Step {
                line: "dig nook <chunk> \"A Name\"",
                why: "In the Mudroom. The first word is the door's name (any word no door there has yet), the second is the memory you pay, by its word (`chunks` lists yours), and the quoted part is the room's name, which no other room may have. The memory goes into the room and keeps it standing.",
                done: |w, n, _, _| own_room(w, n).is_some(),
            },
            Step {
                line: "describe \"<what the room is like>\"",
                why: "Walk in through your door and say what is there. A described room is Kept, the first rung, and `look` in it will name the next.",
                done: |w, n, _, _| own_room(w, n).is_some_and(|r| super::tending::standing(w, r) >= 1),
            },
        ],
    },
    Path {
        key: "read",
        pitch: "to read the world closely and learn what only readers can make",
        end: "That is the reader's road: the world's own text has answers in it, and some of them are recipes the bench gives nobody else. `lore` lists what you know and where the rest is said to be; six folios are chained in the Reading Rooms, up from the Unfiled Archive, and `study folio` shows today's page.",
        steps: &[
            Step {
                line: "look",
                why: "See where you are. Anything a room names can be read: `read <thing>`.",
                done: looked,
            },
            Step {
                line: "slips",
                why: "Twelve reference slips are pinned about the world, one in each district: a reader's question the library pays for. This lists where each is and which you have answered.",
                done: |_, _, verb, _| verb == "slips",
            },
            Step {
                line: "solve \"<your answer>\"",
                why: "Go to a slip (`travel \"<its room>\"`), `read slip`, find the answer in the world, and answer it where it is pinned. A first miss at the keyboard is free.",
                done: |_, _, verb, r| verb == "solve" && r["right"] == json!(true),
            },
            Step {
                line: "chunks",
                why: "What you hold, and how fast each memory fades. Your level is the sum of what has not faded yet, so it falls while you stand still: that is the whole game.",
                done: read_chunks,
            },
            Step {
                line: "lore",
                why: "Eleven things are written down somewhere that teach a recipe, and `lore` says where each is said to be. The nearest is behind no door that asks a level: `travel \"The Court of Lethe\"` (off the Hall of Lost Steps, by the Postern), then `study ink` shows the question, and `study ink \"<your answer>\"` answers it. The room has had the answer in it all along.",
                done: |w, n, _, _| !super::lore::known(w, n).is_empty(),
            },
        ],
    },
];

/// A named creature that has not heard this name yet, to send a new teller to:
/// one in the room first, then the mildest door. `(word to call it by, as shown,
/// its room's name, whether it is here)`.
fn listener(world: &World, name: &str) -> Option<(String, String, String, bool)> {
    let here = find_citizen(world, name).and_then(|f| super::rooms::room_of(world, f.citizen));
    let mut all: Vec<(bool, i64, String, String, String)> = world
        .store
        .of_kind("creature")
        .filter_map(|e| {
            let n = super::mind::name(world, e.id)?;
            if super::mind::knew(world, e.id, name) {
                return None;
            }
            let room = e.parent?;
            // A dug room's creature may be shut in; the world's own rooms are open.
            plain_str(world, room, ROOM, "key")?;
            let place = plain_str(world, room, ROOM, "name")?.to_string();
            let asks = super::rooms::gate_of(world, room).unwrap_or(0.0) as i64;
            Some((
                Some(room) != here,
                asks,
                n.split(' ').next().unwrap_or(&n).to_lowercase(),
                super::mind::display(world, e.id),
                place,
            ))
        })
        .collect();
    all.sort();
    all.into_iter()
        .next()
        .map(|(away, _, word, shown, place)| (word, shown, place, !away))
}

/// The door a name dug to its own room, and the room it was dug from.
fn own_door(world: &World, name: &str) -> Option<(String, String)> {
    let mine = own_room(world, name)?;
    world.store.of_kind("room").find_map(|e| {
        let (via, _) = super::rooms::exits_of(world, e.id)
            .into_iter()
            .find(|(via, to)| *to == mine && via != "back" && via != "out")?;
        if e.id == mine {
            return None;
        }
        Some((via, plain_str(world, e.id, ROOM, "name")?.to_string()))
    })
}

/// A step as it is shown to this name, now: the line with what the world knows
/// filled in (which creature, which chunk, which door), and the reason.
fn shown(world: &World, name: &str, p: &Path, i: usize) -> (String, String) {
    let s = &p.steps[i];
    let (mut line, mut why) = (s.line.to_string(), s.why.to_string());
    match (p.key, i) {
        ("tell", 2) | ("tell", 4) => {
            if let Some((word, who, place, here)) = listener(world, name) {
                line = format!("story {word} \"<a dozen words or more, in sentences>\"");
                why = if here {
                    format!("{who} is here. {why}")
                } else {
                    format!("{who} is in {place}: `travel \"{place}\"` first. {why}")
                };
            }
        }
        ("keep", 3) => {
            let taken = super::rooms::room_by_key(world, "mudroom")
                .map(|m| super::rooms::exits_of(world, m))
                .unwrap_or_default();
            let door = [
                "nook", "den", "burrow", "cubby", "garret", "hollow", "snug", "alcove", "cell",
                "loft",
            ]
            .into_iter()
            .chain(std::iter::once(name))
            .map(str::to_lowercase)
            .find(|d| !taken.contains_key(d))
            .unwrap_or_else(|| "nook".to_string());
            let chunk = find_citizen(world, name)
                .and_then(|f| world.store.children(f.window).next().map(|c| c.id))
                .map(|c| super::called::chunk_word(world, c))
                .unwrap_or_else(|| "<a memory of yours>".to_string());
            line = format!("dig {door} {chunk} \"<a name for your room>\"");
        }
        ("keep", 4) => {
            if let Some((via, from)) = own_door(world, name) {
                why = format!(
                    "Your door is `{via}`, off {from}: `travel {via}` there walks in. {why}"
                );
            }
        }
        _ => {}
    }
    (line, why)
}

fn path(key: &str) -> Option<&'static Path> {
    PATHS.iter().find(|p| p.key == key)
}

fn state(world: &World, name: &str) -> Value {
    find_citizen(world, name)
        .and_then(|f| world.store.entity(f.citizen))
        .and_then(|e| e.components.get(CITIZEN))
        .and_then(|c| match c.fields.get("tutorial") {
            Some(Field::Plain(v)) if v.is_object() => Some(v.clone()),
            _ => None,
        })
        .unwrap_or_else(|| json!({}))
}

/// Write the state, with the step as the page shows it folded in.
fn write(world: &mut World, name: &str, mut st: Value) -> Result<(), CommandError> {
    let Some(f) = find_citizen(world, name) else {
        return Ok(());
    };
    let view = match st["path"].as_str().and_then(path) {
        Some(p) => {
            let i = st["step"].as_u64().unwrap_or(0) as usize;
            if i < p.steps.len() {
                let (line, why) = shown(world, name, p, i);
                json!({"line": line, "why": why, "n": i + 1, "of": p.steps.len()})
            } else {
                Value::Null
            }
        }
        None => Value::Null,
    };
    st["view"] = view;
    st["paths"] = json!(PATHS
        .iter()
        .map(|p| json!({"key": p.key, "pitch": p.pitch}))
        .collect::<Vec<_>>());
    let mut changes = BTreeMap::new();
    changes.insert("tutorial".to_string(), Field::Plain(st));
    super::set_citizen_fields(world, name, f.citizen, changes)
}

fn done_paths(st: &Value) -> Vec<String> {
    st["done"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect()
}

/// The question, as the Docent asks it.
pub fn question(st: &Value) -> String {
    let done = done_paths(st);
    let offers: Vec<String> = PATHS
        .iter()
        .filter(|p| !done.contains(&p.key.to_string()))
        .map(|p| format!("`tutorial {}`, {}", p.key, p.pitch))
        .collect();
    let lead = match (done.len(), offers.len()) {
        (0, _) => "What did you come to do? There are four ways to live here, and I will walk you five steps down whichever you like".to_string(),
        (_, 0) => return "You have walked all four roads. `next` names one thing worth doing from where you stand.".to_string(),
        (_, 1) => "One road you have not walked".to_string(),
        (_, n) => format!("The {} roads you have not walked", if n == 2 { "two" } else { "three" }),
    };
    format!(
        "{lead}: {}. Each ends on something real and pays a small memory. `tutorial skip` and I will say no more of it; `tutorial` brings it back.",
        offers.join("; ")
    )
}

fn road(p: &Path) -> &'static str {
    match p.key {
        "hunt" => "hunter",
        "tell" => "teller",
        "keep" => "keeper",
        _ => "reader",
    }
}

fn step_said(world: &World, name: &str, p: &Path, i: usize) -> String {
    let (line, why) = shown(world, name, p, i);
    format!(
        "Step {} of {} on the {}'s road: `{}`. {}",
        i + 1,
        p.steps.len(),
        road(p),
        line,
        why
    )
}

fn docent_says(world: &mut World, name: &str, text: String) {
    combat::narrate(world, name, format!("The Docent: \"{text}\""));
}

thread_local! {
    /// A call the play page made by itself (a panel keeping current, the look at
    /// login), not one the player sent: it ticks no step. Three of four at the
    /// keyboard had steps ticked by calls they never saw (the Watch of 11-22).
    pub static UNSENT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the Docent is walking this name down a road just now.
pub fn on_road(world: &World, name: &str) -> bool {
    state(world, name)["path"].as_str().and_then(path).is_some()
}

/// The question, put to a new name as it arrives: the first thing the Docent
/// says, in place of the three paragraphs of first moves that used to be.
pub fn greet(world: &mut World, name: &str) {
    let mut st = state(world, name);
    if st["asked"] == json!(true) {
        return;
    }
    st["asked"] = json!(true);
    let q = question(&st);
    if write(world, name, st).is_ok() {
        docent_says(world, name, q);
    }
}

/// After a statement of a name's has run: ask the question of a new name, once;
/// tick the step the world now sees done, and say the next. Never fails the
/// statement it rides on.
pub fn after(world: &mut World, name: &str, verb: &str, result: &Value) {
    let Some(f) = find_citizen(world, name) else {
        return;
    };
    let mut st = state(world, name);
    if verb == "tutorial" || verb == "next" || UNSENT.with(|u| u.get()) {
        return;
    }
    // A new name, not yet asked. Only in its first hour: a name that was here
    // before the tutorial was is not stopped at the door to be asked.
    if st["asked"] != json!(true) {
        if !combat::sheltered(world, f.citizen) {
            return;
        }
        st["asked"] = json!(true);
        let q = question(&st);
        if write(world, name, st).is_ok() {
            docent_says(world, name, q);
        }
        return;
    }
    let Some(p) = st["path"].as_str().and_then(path) else {
        return;
    };
    let mut i = st["step"].as_u64().unwrap_or(0) as usize;
    let before = i;
    // More than one step may be done by one line: all of them tick.
    while i < p.steps.len() && (p.steps[i].done)(world, name, verb, result) {
        i += 1;
    }
    if i == before {
        return;
    }
    // A step the world already saw done is named, not passed over in silence.
    let already = match i - before {
        1 => String::new(),
        2 => format!(" (Step {} was done already.)", before + 2),
        _ => format!(" (Steps {} to {} were done already.)", before + 2, i),
    };
    if i < p.steps.len() {
        st["step"] = json!(i);
        if write(world, name, st).is_ok() {
            docent_says(
                world,
                name,
                format!("Done.{already} {}", step_said(world, name, p, i)),
            );
        }
        return;
    }
    finish(world, name, st, p);
}

fn finish(world: &mut World, name: &str, mut st: Value, p: &'static Path) {
    let Some(f) = find_citizen(world, name) else {
        return;
    };
    let mut done = done_paths(&st);
    let first = !done.contains(&p.key.to_string());
    if first {
        done.push(p.key.to_string());
    }
    st["done"] = json!(done);
    st["path"] = Value::Null;
    st["step"] = json!(0);
    st["last"] = json!(p.key);
    st["at"][p.key] = Value::Null;
    // Nothing more is asked until it is asked for: the card does not come back
    // with "which road?" the moment one is walked.
    st["skipped"] = json!(true);
    let left: Vec<String> = PATHS
        .iter()
        .filter(|x| !done.contains(&x.key.to_string()))
        .map(|x| format!("`tutorial {}`", x.key))
        .collect();
    let paid = if first {
        let curves = super::Curves::from_env();
        super::memory::earn_chunk(
            world,
            name,
            f.window,
            &format!("the {} road walked to its first turning", p.key),
            PATH_XP,
            0.6,
            &curves,
        )
        .ok()
        .map(|c| {
            format!(
                " You will remember having learned it: chunk-{} worth {PATH_XP:.0} XP.",
                c.0
            )
        })
        .unwrap_or_default()
    } else {
        String::new()
    };
    let _ = world.note(
        Actor::Citizen(name.to_string()),
        TUTORED,
        json!({"by": name, "path": p.key, "what": "finished"}),
    );
    if write(world, name, st).is_ok() {
        combat::leave_by_level(world, name, f.citizen);
        docent_says(
            world,
            name,
            format!(
                "{}{paid} {}",
                p.end,
                if left.is_empty() {
                    "You have walked all four. `next` will always name one thing worth doing."
                        .to_string()
                } else {
                    format!(
                        "The other roads are there when you want them: {}. And `next` will always name one thing worth doing.",
                        left.join(", ")
                    )
                }
            ),
        );
    }
}

/// What a road's walker does next, once the five steps are behind them.
fn onward(world: &World, name: &str, key: &str) -> Option<String> {
    let f = find_citizen(world, name)?;
    match key {
        "hunt" => {
            let level = super::memory::level(world, f.window);
            // The fighter's road, door by door.
            let road = ["shallows", "ossuary", "gallery", "oubliette"];
            let doors: Vec<(String, f64)> = road
                .iter()
                .filter_map(|k| {
                    let r = super::rooms::room_by_key(world, k)?;
                    Some((
                        plain_str(world, r, ROOM, "name")?.to_string(),
                        super::rooms::gate_of(world, r)?,
                    ))
                })
                .collect();
            let open = doors.iter().rev().find(|(_, asks)| *asks <= level);
            let shut = doors.iter().find(|(_, asks)| *asks > level);
            Some(match (open, shut) {
                (Some((n, asks)), Some((m, more))) => format!(
                    "You hold {level:.0}. The door for you is {n} (it asks {asks:.0}): `travel \"{n}\"`, and `look` there sizes up the fight first. The next, {m}, asks {more:.0}."
                ),
                (Some((n, asks)), None) => format!(
                    "You hold {level:.0}, and every door on the hunter's road is yours: {n} (it asks {asks:.0}) is the deepest, `travel \"{n}\"`."
                ),
                (None, Some((m, more))) => format!(
                    "You hold {level:.0}. The Fogbank's wisps are your size: `travel \"The Fogbank\"` and `attack wisp`. The first door, {m}, asks {more:.0}."
                ),
                (None, None) => return None,
            })
        }
        "tell" => {
            let known = super::bard::known_to(world, name);
            let to = listener(world, name).map(|(word, who, place, here)| {
                if here {
                    format!(" {who} is here and has not heard you: `story {word} \"...\"`.")
                } else {
                    format!(" {who}, in {place}, has not heard you.")
                }
            });
            Some(format!(
                "{known} named creature{} know{} you. `wants` says what each would hear tonight (a story on what one wants pays double); `repute` is your standing, and `deeds` lists what there is to sing.{}",
                if known == 1 { "" } else { "s" },
                if known == 1 { "s" } else { "" },
                to.unwrap_or_default()
            ))
        }
        "keep" => match own_room(world, name) {
            Some(room) if super::tending::standing(world, room) < 3 => {
                let n = plain_str(world, room, ROOM, "name").unwrap_or("Your room");
                Some(format!(
                    "{n} can stand higher: `look` inside it names what it wants next, and `mine` says how it stands."
                ))
            }
            Some(_) => Some("Your room is Cherished. `mine` lists what you keep; `dig` from inside it opens another.".to_string()),
            None => Some("You keep no room now. `dig <door> <chunk> \"<a name>\"` in the Mudroom opens one.".to_string()),
        },
        "read" => {
            let known = super::lore::known(world, name).len();
            let all = super::lore::SOURCES.len();
            Some(if known < all {
                format!(
                    "You have read {known} of {all} things that teach a making: `lore` says where the rest are said to be, and `slips` which of today's twelve you have not answered."
                )
            } else {
                "You have read everything that teaches a making. `slips` says which of today's twelve you have not answered; `recipes` what you can make.".to_string()
            })
        }
        _ => None,
    }
}

/// One sensible thing to do, out of a path: what the road this name last walked
/// asks next, and only after that a road it has not walked.
fn suggestion(world: &World, name: &str) -> String {
    let st = state(world, name);
    let done = done_paths(&st);
    let Some(f) = find_citizen(world, name) else {
        return "`look`.".to_string();
    };
    let hp = combat::hp(world, f.citizen);
    let max = combat::max_hp(world, f.citizen);
    if hp < max / 2.0 {
        return format!(
            "You are hurt ({hp:.0} of {max:.0}). `rest` where nothing hunts; the Watch-fire, east of the Shallows, mends four times as fast."
        );
    }
    let unwalked = PATHS
        .iter()
        .find(|p| !done.contains(&p.key.to_string()))
        .map(|p| p.key);
    let last = st["last"]
        .as_str()
        .map(str::to_string)
        .or_else(|| done.last().cloned());
    // A road set down part-way comes first: it is the nearest thing to hand.
    if let Some((k, n)) = PATHS.iter().find_map(|p| {
        st["at"][p.key]
            .as_u64()
            .filter(|n| *n > 0)
            .map(|n| (p.key, n))
    }) {
        return format!(
            "You set the {k} road down at step {} of 5: `tutorial {k}` takes it up there.",
            n + 1
        );
    }
    if let Some(said) = last.as_deref().and_then(|k| onward(world, name, k)) {
        return match unwalked {
            Some(k) => format!("{said} (A road you have not walked: `tutorial {k}`.)"),
            None => said,
        };
    }
    // A name that has taken no road is offered one before anything else: the
    // roads are how a first memory is come by.
    if st["skipped"] == json!(true) && chunks_held(world, name) == 0 {
        return "You hold no memory, so you have no level. `wants` lists creatures that pay for a story, and `story bench \"...\"` at the Postern takes one from anyone.".to_string();
    }
    "Nothing here is required. To be walked through a first hour, pick a road: `tutorial hunt`, `tutorial tell`, `tutorial keep` or `tutorial read`. Otherwise `look` says what is here, and `wants` who would pay for a story.".to_string()
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.tutorial",
            "The first hour, led by the Docent. Bare: where you are in it (the step you are on, or the question of which road to take). `tutorial hunt`, `tutorial tell`, `tutorial keep` or `tutorial read`: take that road, five steps through the real world, each ticked when the world sees it done; the last pays a small memory, once. `tutorial skip` sets it down; `tutorial` takes it up again. All four roads can be walked, one after another.",
        )
        .param("road", ParamType::String, false, "hunt, tell, keep, read, or skip")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            super::me(w, ctx)?;
            let name = ctx.actor.clone();
            let road = args.str("road").unwrap_or("").trim().to_lowercase();
            let mut st = state(w, &name);
            st["asked"] = json!(true);
            if road.is_empty() {
                let said = match st["path"].as_str().and_then(path) {
                    Some(p) => {
                        let i = (st["step"].as_u64().unwrap_or(0) as usize).min(p.steps.len() - 1);
                        format!("{} (`tutorial skip` sets it down.)", step_said(w, &name, p, i))
                    }
                    None => {
                        st["skipped"] = json!(false);
                        let q = question(&st);
                        match PATHS.iter().find_map(|p| {
                            st["at"][p.key].as_u64().filter(|n| *n > 0).map(|n| (p.key, n))
                        }) {
                            Some((k, n)) => format!(
                                "You set the {k} road down at step {} of 5: `tutorial {k}` takes it up there. {q}",
                                n + 1
                            ),
                            None => q,
                        }
                    }
                };
                write(w, &name, st)?;
                return Ok(Outcome::new(said, json!({"tutorial": state(w, &name)})));
            }
            if road == "skip" || road == "stop" || road == "off" {
                // The place is kept: the road can be taken up where it was left.
                if let Some(k) = st["path"].as_str().map(str::to_string) {
                    st["at"][k.as_str()] = st["step"].clone();
                    st["last"] = json!(k);
                }
                st["path"] = Value::Null;
                st["step"] = json!(0);
                st["skipped"] = json!(true);
                write(w, &name, st)?;
                let _ = w.note(Actor::Citizen(name.clone()), TUTORED, json!({"by": name, "path": Value::Null, "what": "skipped"}));
                // The leave was held past level 100 for the road; off it, it is not.
                if let Some(f) = find_citizen(w, &name) {
                    combat::leave_by_level(w, &name, f.citizen);
                }
                return Ok(Outcome::new(
                    "Set down. Nothing here is required. `tutorial` takes it up again whenever you like, and `next` names one thing worth doing.",
                    json!({"tutorial": state(w, &name)}),
                ));
            }
            let Some(p) = path(&road) else {
                return Err(CommandError::precondition(format!(
                    "There is no road called {road}. The four: hunt, tell, keep, read. Or `tutorial skip`."
                )));
            };
            // Leaving one road for another keeps the place on the first.
            if let Some(k) = st["path"].as_str().map(str::to_string) {
                if k != p.key {
                    st["at"][k.as_str()] = st["step"].clone();
                }
            }
            let at = if st["path"].as_str() == Some(p.key) {
                st["step"].as_u64().unwrap_or(0) as usize
            } else {
                st["at"][p.key].as_u64().unwrap_or(0) as usize
            }
            .min(p.steps.len() - 1);
            st["path"] = json!(p.key);
            st["step"] = json!(at);
            st["last"] = json!(p.key);
            st["skipped"] = json!(false);
            write(w, &name, st)?;
            let _ = w.note(Actor::Citizen(name.clone()), TUTORED, json!({"by": name, "path": p.key, "what": "chosen"}));
            Ok(Outcome::new(
                if at > 0 {
                    format!(
                        "The {} road again, where you left it. {}",
                        p.key,
                        step_said(w, &name, p, at)
                    )
                } else {
                    format!(
                        "The {} road, then: {}. I tick each step when you do it, however you do it. {}",
                        p.key,
                        p.pitch,
                        step_said(w, &name, p, 0)
                    )
                },
                json!({"tutorial": state(w, &name)}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.next",
            "One sensible thing to do from where you stand: the step you are on, if the Docent is walking you down a road (`tutorial`), and otherwise whatever is most worth doing: mending, earning a first memory, a road not yet walked, a room that could stand higher, something not yet read. A read.",
        )
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, _args: &Args| {
            super::me(w, ctx)?;
            let st = state(w, &ctx.actor);
            let name = ctx.actor.clone();
            let said = match st["path"].as_str().and_then(path) {
                Some(p) => {
                    let i = (st["step"].as_u64().unwrap_or(0) as usize).min(p.steps.len() - 1);
                    step_said(w, &name, p, i)
                }
                None => suggestion(w, &ctx.actor),
            };
            Ok(Outcome::new(said, json!({"tutorial": st})))
        }),
    );
}
