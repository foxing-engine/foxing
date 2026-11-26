//! Antics: what the creatures that do not hunt get up to.
//!
//! David, 11-21: "let's have the non-hunting creatures conduct more business, move
//! around, act weird sometimes and generally make the place more lively. we can add
//! new characters as needed." Until now a creature that did not hunt stood where it
//! was spawned and did nothing until struck or told a story; only named creatures
//! had business, three of them every ten minutes.
//!
//! Once a minute:
//!
//! - **In a room with someone in it**, about every other minute, one creature there
//!   that is not hunting or fighting does something: a line of its kind's own; a
//!   word with another creature there; a turn toward one of the names present; or,
//!   now and then, it leaves a small making on the floor (a mothwing its dust, an
//!   inkling a dab of ink), where fewer than two lie loose.
//! - **Strolls.** An unnamed creature that keeps to itself wanders to the next room
//!   and, a few minutes on, back. Its room is still its home (the room does not
//!   restock while it is out), it never walks into a room where something hunts,
//!   and no more than six are abroad at once.
//! - **Walkers**, four new characters with rounds of their own, a step every two or
//!   three minutes: the lamplighter (Lantern Row, the Lane, the Postern, the
//!   Orchard), the runner from the Dead Letter Office (out to the Mudroom and
//!   back, saying how many letters wait), the tinker (the Threshold, the fog, the
//!   Midden, the Mudroom; it sets a making down now and then), and the census
//!   taker (the Library and both doors), who counts what is in each room aloud and
//!   is not always right. None of them is fought.
//!
//! Each act is public as `disremembered.antic`.

use super::{plain_num, plain_str, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::json;
use std::collections::BTreeMap;

pub const JOB: &str = "disremembered.antics_tick";
pub const EVERY_MS: i64 = 60_000;
/// A creature did something of its own accord: `{creature, kind, name, room, what}`.
pub const ANTIC: &str = "disremembered.antic";
/// Unnamed creatures abroad at once, at most.
pub const STROLLERS: usize = 6;
/// How long a stroller stays out.
const STROLL_MS: i64 = 4 * 60_000;

/// The walkers: `(kind, the rooms of its round, in order)`.
pub const WALKERS: &[(&str, &[&str])] = &[
    (
        "lamplighter",
        &[
            "lantern-row",
            "lamplighters-lane",
            "postern",
            "lamplighters-lane",
            "orchard",
            "lamplighters-lane",
        ],
    ),
    (
        "runner",
        &[
            "dead-letters",
            "namery",
            "archive",
            "lantern-row",
            "threshold",
            "mudroom",
            "threshold",
            "lantern-row",
            "archive",
            "namery",
        ],
    ),
    (
        "tinker",
        &[
            "threshold",
            "fogbank",
            "midden",
            "fogbank",
            "threshold",
            "mudroom",
        ],
    ),
    (
        "census taker",
        &[
            "archive",
            "lantern-row",
            "threshold",
            "lantern-row",
            "lamplighters-lane",
            "postern",
            "lamplighters-lane",
            "lantern-row",
        ],
    ),
];

pub fn is_walker(kind: &str) -> bool {
    WALKERS.iter().any(|(k, _)| *k == kind)
}

/// What a kind does when nobody asks it to. `{who}` is the creature as it is called;
/// `{you}` a name standing in the room.
fn lines(kind: &str) -> &'static [&'static str] {
    match kind {
        "wisp" => &[
            "{who} drifts through the back of a chair and seems embarrassed about it.",
            "{who} rehearses a shape it nearly remembers, gets as far as the shoulders, and gives up.",
            "{who} dims until it is only a place the fog is not, then comes back as if nothing had happened.",
            "{who} follows {you} for three steps, at a polite distance, and then pretends it was going that way anyway.",
            "{who} writes something on the air in its own light. It fades from the far end before the near end is done.",
        ],
        "mothwing" => &[
            "{who} lands on {you}'s sleeve, reads it like a page, and leaves unimpressed.",
            "{who} batters the same patch of nothing four times. There may once have been a lamp there.",
            "{who} folds itself flat as a pressed leaf and holds perfectly still, which fools no one.",
            "{who} shakes itself, and for a moment the air is full of dust that smells of a drawer.",
        ],
        "lanternfly" => &[
            "{who} goes round the nearest lamp once, counting, and seems satisfied with the number.",
            "{who} blinks its light in a pattern, waits for an answer, and gets one from further up the row.",
            "{who} settles on {you}'s shoulder as on a windowsill, warms itself, and is off again.",
            "{who} goes dark all at once, as if it had been told a secret.",
        ],
        "inkling" => &[
            "{who} corrects something on the nearest page, considers the correction, and corrects that.",
            "{who} leaves a footprint that is a comma, then another, and stops to see whether a sentence is coming.",
            "{who} dips itself in a shadow and comes out darker, and pleased.",
            "{who} underlines {you} from a distance, twice, without explaining.",
        ],
        "ragpicker" => &[
            "{who} holds a scrap up to the light, names a price nobody asked for, and puts it back.",
            "{who} sorts three things into two piles, then the two piles into one, with great care.",
            "{who} finds a glove that fits, wears it for a moment, and returns it to the heap as not its own.",
            "{who} looks {you} over the way it looks a heap over, and decides there is nothing loose.",
        ],
        "silverfish" => &[
            "{who} pours itself under a book and out the other side, shorter by a paragraph's worth of somebody.",
            "{who} stops dead on a spine, tastes the title, and moves on to something older.",
            "{who} runs a straight line along the shelf's edge and turns at the end like a typewriter.",
        ],
        "bookworm" => &[
            "{who} surfaces through a cover, blinks, and goes back in at the same hole.",
            "{who} has reached the last page of something. It lies still a while, as one does.",
            "{who} mouths a word over and over. It is not a word from the book it is in.",
        ],
        "shelf mouse" => &[
            "{who} carries a call slip twice its size along the shelf and files it, wrongly, with conviction.",
            "{who} sits up, washes its face, and looks at {you} as if you had come about the vacancy.",
            "{who} has made a nest of overdue notices and is not taking questions.",
        ],
        "newtling" => &[
            "{who} climbs out onto a dry stone, regrets it, and slides back in.",
            "{who} watches its own reflection drift off and paddles after it.",
            "{who} opens its mouth very wide at nothing, and shuts it, satisfied.",
        ],
        "mudskipper" => &[
            "{who} walks six steps on its fins, forgets how, and has to think.",
            "{who} raises its back fin at {you} like a small flag of a country you have not heard of.",
            "{who} rolls once in the wet, eyes shut, in what can only be pleasure.",
        ],
        "hermit" => &[
            "{who} tries on a second shell, stands in both for a moment, and goes back to the first.",
            "{who} drags its house a hand's width to the left. The view must be better.",
            "{who} knocks on its own shell from the inside, and waits.",
        ],
        "sigh" => &[
            "{who} goes through the grass and every blade leans the way a head leans to listen.",
            "{who} says most of a name, yours or near it, and lets the rest go.",
            "{who} settles in the hollow of the path like something put down for a moment and not picked up.",
        ],
        "picnicker" => &[
            "{who} pours tea into a cup that is already full, and neither of them mentions it.",
            "{who} offers the plate to the empty place beside it, waits a civil interval, and takes one itself.",
            "{who} says \"they will be along directly\" to {you}, with the smile of long practice.",
        ],
        "longsight" => &[
            "{who} shades its eyes and looks a long way off at somewhere you were this morning.",
            "{who} counts the rooms it can see, on its fingers, and runs out of fingers contentedly.",
            "{who} points, says nothing, and when you look there is only the weather.",
        ],
        "gardener" => &[
            "{who} deadheads a flower that was not there a moment ago, and pockets it.",
            "{who} ties an hour back to its stake with a twist of green string.",
            "{who} stands with the watering can tilted and nothing coming out, in no hurry at all.",
        ],
        "glowworm" => &[
            "{who} brightens when {you} looks at it, and tries to look as if it had not.",
            "{who} lets itself down a hand's width on its thread, and reels itself back up.",
            "{who} pulses slowly, like something counting sheep.",
        ],
        "mender" => &[
            "{who} holds a needle up to the light, threads it with something you cannot see, and begins on the air.",
            "{who} looks {you} over for loose ends, and finds one, and is tactful about it.",
            "{who} bites a thread through and tucks the end in. Something in the room sits a little straighter.",
        ],
        "porter" => &[
            "{who} shifts its pack from one shoulder to the other. It makes the sound of a much heavier pack.",
            "{who} reads a label, turns it over, and reads the back with more interest.",
            "{who} sets its pack down, sits on it, and is asleep at once with its eyes open.",
        ],
        "lamplighter" => &[
            "{who} trims a wick with two fingers, blows on them, and nods to the lamp as to a colleague.",
            "{who} holds the pole up to a lamp that is already lit, to be sure, and writes it in a small book.",
            "{who} says, to nobody, \"that one will want oil by Thursday,\" and looks at {you} as if you might see to it.",
            "{who} counts the lamps it has passed on its fingers and frowns at the answer.",
        ],
        "runner" => &[
            "{who} checks the label on its empty bag against {you}'s face, sighs, and folds it again.",
            "{who} stops, pats every pocket in turn, finds what it was looking for in its hand, and runs on the spot until it remembers which way.",
            "{who} reads an envelope held up to the light, catches {you} watching, and holds it against its chest.",
        ],
        "tinker" => &[
            "{who} unrolls a cloth of small useful things, looks at them fondly, and rolls it up again before anyone can ask.",
            "{who} taps the door frame, listens, and says \"that'll hold\", which nobody had doubted until then.",
            "{who} offers {you} a button for a story, hears none in time, and keeps the button.",
            "{who} mends something that was not broken, and hands it back to the room.",
        ],
        "census taker" => &[
            "{who} licks its pencil and asks {you} how many of you there are, and writes down something longer than the answer.",
            "{who} counts the doors, then the people, then the doors again, and is one short each time.",
            "{who} asks the nearest lamp its occupation.",
        ],
        "crier" => &[
            "{who} clears its throat, reads the next name off the list, and waits. Nobody comes. It ticks the name.",
            "{who} practices a name under its breath until it has it wrong the same way every time.",
        ],
        "clerk" => &[
            "{who} stamps a form, looks at the stamp, and stamps beside it, straighter.",
            "{who} moves a stack of papers from the left of the window to the right, and then looks for it on the left.",
            "{who} tells {you} the window is open, in the tone of one announcing a death.",
        ],
        "bailiff" => &[
            "{who} looks at {you}'s forearm, and then, with visible restraint, away.",
            "{who} tests the nib of something on its thumb and puts it back in its coat.",
        ],
        "engrosser" => &[
            "{who} writes a capital with eleven strokes where four would do, and sits back to admire the seventh.",
            "{who} blots a line, reads the blotter, and prefers it.",
        ],
        "tipstaff" => &[
            "{who} raps its staff once on the floor for silence and seems put out to find it already had it.",
            "{who} straightens a row of chairs that nobody has sat in, starting from the wrong end.",
        ],
        // C11: the Kept House. Each has one thing it cannot leave alone, and the
        // Visitors' Book is matched to them by it.
        "housekeeper" => &[
            "{who} counts the coat hooks, then counts them again from the other end, and is satisfied.",
            "{who} says, to nobody, \"Four cups out, four cups back,\" and writes it down.",
            "{who} looks {you} over as an item newly arrived, and appears to enter you in a column.",
            "{who} finds one of something where there should be two, and stands quite still until the other turns up.",
        ],
        "watchman" => &[
            "{who} looks at each of the clocks in turn and trusts none of them.",
            "{who} remarks that it is later than it was, as if that were news it had been keeping for {you}.",
            "{who} yawns at an hour when nobody else would, and seems to enjoy it.",
            "{who} winds something. It is not clear what; it is not clear it needed it.",
        ],
        "cook" => &[
            "{who} tastes from the pot, considers, and reaches for the salt without looking.",
            "{who} tells {you} that whatever is the matter, it would be less so after a bowl of something.",
            "{who} moves the pot a hand's width off the heat, for tomorrow's sake.",
            "{who} cuts the heel off a loaf and leaves it on the sill for whoever comes down hungry.",
        ],
        "copyist" => &[
            "{who} reads a line back under its breath, stops at a comma, and moves it.",
            "{who} corrects something on the wall with one stroke and looks relieved.",
            "{who} says \"fewer, not less\" to {you}, though you had not spoken.",
            "{who} rules a margin, admires it, and writes nothing in it.",
        ],
        "seamster" => &[
            "{who} finds a loose thread on {you} and has it off before you can object.",
            "{who} turns a thing over to see where it gave way, and nods as if it had told the truth.",
            "{who} sews a button onto something that had not lost one, to be on the safe side.",
            "{who} bites off a thread and says that it will hold, and it will.",
        ],
        "stargazer" => &[
            "{who} tips back under the skylight and names something up there, quietly, in case it is listening.",
            "{who} tells {you} what the weather will do, and is specific about the hour.",
            "{who} marks the chart with a pencil kept behind its ear, and rubs the mark out again.",
            "{who} looks out of the east window and says tomorrow looks all right from here.",
        ],
        "traveler" => &[
            "{who} unpacks one thing, looks at it, and packs it again.",
            "{who} asks {you} how far it is to somewhere you have not heard of, and nods at the answer you do not give.",
            "{who} dries a boot by holding it near where a fire would be.",
            "{who} says the beds are better here than the last place, and does not say where that was.",
        ],
        "shopkeep" => &[
            "{who} dusts something that is not dusty, and puts it back a finger's width to the left.",
            "{who} rewrites a ticket in the same small hand, with the same figure on it.",
            "{who} looks at {you}, then at a shelf, as if working out where you would go.",
            "{who} turns a thing over to see what is written underneath, and seems reassured that nothing is.",
        ],
        "tapster" => &[
            "{who} polishes a glass, holds it to the light, and weighs the light.",
            "{who} chalks a figure on the slate, rubs out the last digit, and chalks a kinder one.",
        ],
        "cataloguer" => &[
            "{who} files {you} under something, changes its mind, and cross-references.",
            "{who} pulls a drawer all the way out, reads the last card, and nods as at an old friend.",
        ],
        _ => &[
            "{who} turns round twice where it stands and settles facing the other way.",
            "{who} looks at {you} for exactly as long as is polite, and one moment longer.",
            "{who} does something small and private with great concentration.",
        ],
    }
}

/// What two creatures in a room do with each other.
const DUETS: &[&str] = &[
    "{who} and {other} pass each other in the middle of the room, each stepping the same way twice.",
    "{who} shows {other} something small. {other} is polite about it.",
    "{who} takes up a place beside {other}, and for a while they look at the same nothing.",
    "{who} says something to {other} too low to hear. {other} looks at {you}, and away.",
    "{who} bows to {other}, who had no idea they were on those terms.",
    "{who} copies whatever {other} is doing, half a beat behind, until {other} notices.",
];

/// What a kind may leave on the floor.
fn leaves(kind: &str) -> Option<(&'static str, &'static str)> {
    match kind {
        "mothwing" => Some(("moth dust", "{who} shakes out its wings over the floor and goes on, lighter: moth dust, where it was.")),
        "inkling" => Some(("a dab of ink", "{who} has stood in one place too long and left some of itself there: a dab of ink.")),
        "ragpicker" => Some(("rag thread", "{who} picks a length of thread off its own sleeve, judges it, and lets it fall.")),
        "tinker" => Some(("rag thread", "{who} cuts a length of thread too long, tuts, and leaves the offcut on the floor.")),
        "porter" => None,
        _ => None,
    }
}

fn kind_of(world: &World, c: EntityId) -> String {
    plain_str(world, c, CREATURE, "name")
        .unwrap_or("?")
        .to_string()
}

/// A creature as a line calls it: its name if it has one, else "the" and its kind.
fn who(world: &World, c: EntityId) -> String {
    match super::mind::name(world, c) {
        Some(n) => n,
        None => format!("the {}", kind_of(world, c)),
    }
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Creatures in a room that may do something: not hunting, not in a fight, not down.
fn idle_in(world: &World, room: EntityId) -> Vec<EntityId> {
    super::combat::creatures_in(world, room)
        .into_iter()
        .map(|c| c.0)
        .filter(|&c| {
            !super::combat::is_aggressive(world, c)
                && super::combat::fought_by(world, c).is_none()
                && !super::mind::shaken(world, c)
                && kind_of(world, c) != super::starfish::KIND
        })
        .collect()
}

fn names_present(world: &World, room: EntityId) -> Vec<String> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .collect()
}

fn note(world: &mut World, c: EntityId, room: EntityId, what: &str) {
    let _ = world.note(
        Actor::Driver,
        ANTIC,
        json!({"creature": c.0, "kind": kind_of(world, c), "name": super::mind::name(world, c), "room": room.0, "what": what}),
    );
}

/// One act in an occupied room. Returns whether anything was said.
pub fn act_in(world: &mut World, room: EntityId) -> Result<bool, String> {
    let names = names_present(world, room);
    if names.is_empty() {
        return Ok(false);
    }
    let idle = idle_in(world, room);
    if idle.is_empty() {
        return Ok(false);
    }
    let c = idle[world.roll(idle.len() as u32) as usize];
    let kind = kind_of(world, c);
    let me = who(world, c);
    let you = names[world.roll(names.len() as u32) as usize].clone();
    let roll = world.roll(10);
    // Now and then, a making left on the floor.
    if roll == 0 {
        if let Some((what, line)) = leaves(&kind) {
            let loose = world
                .store
                .children(room)
                .filter(|e| e.kind == "item" && !super::craft::is_fixed(world, e.id))
                .count();
            if loose < 2 {
                let curves = super::Curves::from_env();
                super::craft::make(
                    world,
                    &curves,
                    Actor::Driver,
                    None,
                    room,
                    what,
                    "material",
                    &format!("left by a {kind}"),
                    None,
                    &[],
                    0,
                    false,
                )
                .map_err(|e| e.to_string())?;
                let said = cap(&line.replace("{who}", &me));
                super::combat::narrate_room(world, c, &said);
                note(world, c, room, "left");
                return Ok(true);
            }
        }
    }
    // A word with another creature there.
    if roll <= 2 && idle.len() > 1 {
        let others: Vec<EntityId> = idle.iter().copied().filter(|o| *o != c).collect();
        let o = others[world.roll(others.len() as u32) as usize];
        let other = who(world, o);
        if other != me {
            let line = DUETS[world.roll(DUETS.len() as u32) as usize];
            let said = cap(&line
                .replace("{who}", &me)
                .replace("{other}", &other)
                .replace("{you}", &you))
            // A second sentence that begins with "the glowworm".
            .replace(". the ", ". The ");
            super::combat::narrate_room(world, c, &said);
            note(world, c, room, "duet");
            return Ok(true);
        }
    }
    // Not the line it said last: the tinker said one four times in an hour.
    let table = lines(&kind);
    let last = plain_num(world, c, CREATURE, "said_i").map(|n| n as usize);
    let mut i = world.roll(table.len() as u32) as usize;
    if Some(i) == last && table.len() > 1 {
        i = (i + 1 + world.roll(table.len() as u32 - 1) as usize) % table.len();
    }
    set(world, c, "said_i", json!(i))?;
    let line = table[i];
    let said = cap(&line.replace("{who}", &me).replace("{you}", &you));
    super::combat::narrate_room(world, c, &said);
    note(world, c, room, "idle");
    Ok(true)
}

/// What a walker or a resident says when spoken to: its own business, never the
/// question's (R263).
pub fn answers(world: &mut World, kind: &str) -> Option<&'static str> {
    let table: &[&str] = match kind {
        "lamplighter" => &[
            "The last lamp is whichever one I am furthest from. It has always been so.",
            "I light them. Who puts them out is a matter I have raised, in writing.",
            "Mind the ladder. There is no ladder; mind it anyway.",
        ],
        "runner" => &[
            "Can't stop. Nothing to deliver, and I am late with it.",
            "Write to somebody! Anybody! The Dead Letter Office takes what has nowhere to go.",
            "If it's for you I'd have said. It isn't. Nothing is.",
        ],
        "tinker" => &[
            "That'll hold. Don't lean on it.",
            "I mend what's brought. What's brought is mostly opinions.",
            "It wasn't broken when I came, and it is mended now, and that's two different things.",
        ],
        "census taker" => &[
            "One of you. I had you down as one already, so that is none new.",
            "Stand still, please. You count the same moving, but it is harder to be sure.",
            "I am always one short. I have begun to suspect it is me.",
        ],
        "glowworm" => {
            &["It brightens a little, which is all the answer it has, and not a bad one."]
        }
        "mender" => &[
            "Hold still and it's done sooner. That goes for most things.",
            "Sit. Whatever it was will keep, and you won't.",
        ],
        "porter" => &[
            "Left it by the door. Leave it there, or don't; it's yours now.",
            "One a day, when the floor isn't already full of things nobody picked up.",
        ],
        _ => return None,
    };
    Some(table[world.roll(table.len() as u32) as usize])
}

/// How a walker leaves a room: one line said of four characters wore out in an hour
/// (the Watch of 11-21).
const GOES: &[&str] = &[
    "{who} looks at the time, though there is no clock, and goes on toward {to}.",
    "{who} remembers somewhere else it is supposed to be, and goes: {to}, by the look of it.",
    "{who} is finished here, by some reckoning of its own, and sets off for {to}.",
    "{who} nods to the room in general and leaves it for {to}.",
    "{who} gathers itself up and is gone toward {to} before anyone can ask.",
    "{who} checks the door, finds it a door, and goes through it toward {to}.",
];

fn move_to(world: &mut World, c: EntityId, to: EntityId) -> Result<(), String> {
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: c,
                parent: Some(to),
            },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn set(world: &mut World, c: EntityId, key: &str, v: serde_json::Value) -> Result<(), String> {
    let mut changes = BTreeMap::new();
    changes.insert(key.to_string(), Field::Plain(v));
    super::bindery::set_fields(world, Actor::Driver, c, CREATURE, changes)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn room_name(world: &World, r: EntityId) -> String {
    // Mid-sentence: "toward the Fogbank", not "toward The Fogbank".
    let n = plain_str(world, r, ROOM, "name").unwrap_or("somewhere");
    match n.strip_prefix("The ") {
        Some(rest) => format!("the {rest}"),
        None => n.to_string(),
    }
}

/// Whether a creature may be strolled into a room: a room of the world's own, with
/// nothing hunting in it, no door that says a level, and not a house with rules.
fn strollable(world: &World, room: EntityId) -> bool {
    let Some(e) = world.store.entity(room) else {
        return false;
    };
    e.owner.is_none()
        && super::rooms::gate_of(world, room).is_none()
        && !super::conclave::is_house(world, room)
        && plain_str(world, room, ROOM, "key") != Some(super::courthouse::COURTROOM)
        && !super::combat::creatures_in(world, room)
            .iter()
            .any(|c| super::combat::is_aggressive(world, c.0))
}

/// Strolls: those abroad long enough go home; then, one time in three, one more
/// sets out.
pub fn strolls(world: &mut World) -> Result<usize, String> {
    let now = world.now().0;
    let abroad: Vec<(EntityId, EntityId, i64)> = world
        .store
        .of_kind("creature")
        .filter_map(|e| {
            let at = plain_num(world, e.id, CREATURE, "stroll_at")? as i64;
            let home = super::life::home_of(world, e.id)?;
            Some((e.id, home, at))
        })
        .collect();
    let mut moved = 0;
    for (c, home, at) in &abroad {
        let here = world.store.entity(*c).and_then(|e| e.parent);
        let fighting = super::combat::fought_by(world, *c).is_some();
        if now - at >= STROLL_MS && !fighting {
            if here != Some(*home) && world.store.entity(*home).is_some() {
                let me = who(world, *c);
                super::combat::narrate_room(
                    world,
                    *c,
                    &cap(&format!(
                        "{me} has had enough of here and goes back the way it came."
                    )),
                );
                move_to(world, *c, *home)?;
                super::combat::narrate_room(world, *c, &cap(&format!("{me} comes back in and takes up exactly where it was, as if it had only blinked.")));
                moved += 1;
            }
            set(world, *c, "stroll_at", serde_json::Value::Null)?;
        }
    }
    if abroad.len() >= STROLLERS || world.roll(3) != 0 {
        return Ok(moved);
    }
    // One sets out: unnamed, keeping to itself, at home, in a room of the world's.
    let candidates: Vec<(EntityId, EntityId)> = world
        .store
        .of_kind("creature")
        .filter(|e| super::mind::name(world, e.id).is_none())
        .filter(|e| plain_num(world, e.id, CREATURE, "stroll_at").is_none())
        .filter_map(|e| e.parent.map(|p| (e.id, p)))
        .filter(|(c, p)| {
            let k = kind_of(world, *c);
            world
                .store
                .entity(*p)
                .is_some_and(|r| r.kind == "room" && r.owner.is_none())
                && !super::combat::is_aggressive(world, *c)
                && super::combat::fought_by(world, *c).is_none()
                && !super::mind::shaken(world, *c)
                && !super::combat::OFFICERS.contains(&k.as_str())
                && !super::tending::is_resident_kind(&k)
                && !is_walker(&k)
                && k != super::starfish::KIND
                && super::combat::kinds_of(world, *p)
                    .iter()
                    .any(|(key, _)| *key == k)
        })
        .collect();
    if candidates.is_empty() {
        return Ok(moved);
    }
    let (c, from) = candidates[world.roll(candidates.len() as u32) as usize];
    let ways: Vec<EntityId> = super::rooms::exits_of(world, from)
        .into_values()
        .filter(|to| *to != from && strollable(world, *to))
        .collect();
    if ways.is_empty() {
        return Ok(moved);
    }
    let to = ways[world.roll(ways.len() as u32) as usize];
    let me = who(world, c);
    super::life::set_home(world, c, from)?;
    set(world, c, "stroll_at", json!(now))?;
    super::combat::narrate_room(
        world,
        c,
        &cap(&format!(
            "{me} takes a notion and wanders off toward {}.",
            room_name(world, to)
        )),
    );
    move_to(world, c, to)?;
    super::combat::narrate_room(world, c, &cap(&format!("{me} wanders in from {}, looks about as if it had been told there would be more, and stays anyway.", room_name(world, from))));
    note(world, c, to, "stroll");
    Ok(moved + 1)
}

/// What a walker says on coming into a room with someone in it.
fn arrives(world: &mut World, kind: &str, c: EntityId, room: EntityId) -> String {
    let me = cap(&who(world, c));
    match kind {
        "lamplighter" => format!("{me} comes in with the long pole over one shoulder, looks up at every lamp in turn, and finds one to disapprove of."),
        "runner" => {
            let n = super::letters::waiting(world);
            match n {
                0 => format!("{me} comes in at a trot with an empty bag, and tells the room so: \"Nothing in the pigeonholes. Nothing! Write to somebody.\""),
                1 => format!("{me} comes in at a trot: \"One letter waiting at the Dead Letter Office, north of the Namery, and no one to read it.\""),
                n => format!("{me} comes in at a trot: \"{n} letters waiting at the Dead Letter Office, north of the Namery. Each one pays whoever answers.\""),
            }
        }
        "tinker" => format!("{me} comes in under a frame hung with pans and small tools, every one of which announces it."),
        _ => {
            // The census: what is in the room, counted aloud, and not always right.
            let creatures = super::combat::creatures_in(world, room).len().saturating_sub(1);
            let people = names_present(world, room).len();
            let off = world.roll(4) == 0;
            let said = if off { creatures + 1 } else { creatures };
            format!(
                "{me} comes in, licks its pencil, and counts aloud: \"{} of the named sort, {} of the other{}.\" It writes that down{}.",
                people,
                said,
                if said == 1 { "" } else { "s" },
                if off { ", and something in the room was counted twice" } else { "" }
            )
        }
    }
}

/// The walkers' rounds: each is made if it is nowhere, and steps when its time comes.
pub fn walkers(world: &mut World) -> Result<usize, String> {
    let now = world.now().0;
    let mut stepped = 0;
    for (kind, route) in WALKERS {
        let found = world
            .store
            .of_kind("creature")
            .find(|e| plain_str(world, e.id, CREATURE, "name") == Some(*kind))
            .map(|e| e.id);
        let c = match found {
            Some(c) => c,
            None => {
                let (Some(d), Some(start)) = (
                    super::combat::def_by_key(kind),
                    super::rooms::room_by_key(world, route[0]),
                ) else {
                    continue;
                };
                let c = super::combat::spawn_one(world, start, d).map_err(|e| e.to_string())?;
                set(world, c, "route_i", json!(0))?;
                set(world, c, "moved_at", json!(now))?;
                c
            }
        };
        if super::combat::fought_by(world, c).is_some() {
            continue;
        }
        let moved_at = plain_num(world, c, CREATURE, "moved_at").unwrap_or(0.0) as i64;
        // Two or three minutes in a room, by its own number.
        let stay = (2 + (c.0 % 2) as i64) * 60_000;
        if now - moved_at < stay {
            continue;
        }
        let i =
            (plain_num(world, c, CREATURE, "route_i").unwrap_or(0.0) as usize + 1) % route.len();
        let Some(to) = super::rooms::room_by_key(world, route[i]) else {
            continue;
        };
        let me = cap(&who(world, c));
        let goes = GOES[world.roll(GOES.len() as u32) as usize]
            .replace("{who}", &me)
            .replace("{to}", &room_name(world, to));
        super::combat::narrate_room(world, c, &goes);
        move_to(world, c, to)?;
        set(world, c, "route_i", json!(i))?;
        set(world, c, "moved_at", json!(now))?;
        if !names_present(world, to).is_empty() {
            let said = arrives(world, kind, c, to);
            super::combat::narrate_room(world, c, &said);
        }
        note(world, c, to, "round");
        stepped += 1;
    }
    Ok(stepped)
}

/// One minute of the world's small life.
pub fn tick(world: &mut World) -> Result<(), String> {
    walkers(world)?;
    strolls(world)?;
    // In each room with someone in it, about every other minute.
    let occupied: Vec<EntityId> = world
        .store
        .of_kind("citizen")
        .filter(|e| super::presence::is_here(world, e.id))
        .filter_map(|e| e.parent)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for room in occupied {
        if world.roll(2) == 0 {
            act_in(world, room)?;
        }
    }
    wind(world)
}

fn wind(world: &mut World) -> Result<(), String> {
    let due = world.now().plus(foxing_time::Duration(EVERY_MS));
    world
        .schedule_once(JOB, due, json!({}), |_| true)
        .map(|_| ())
}

pub fn ensure(world: &mut World) -> Result<(), String> {
    if !world.jobs.has(JOB, |_| true) {
        wind(world)?;
    }
    Ok(())
}

pub fn register(world: &mut World) {
    world.on_job(JOB, Box::new(|w: &mut World, _| tick(w)));
}
