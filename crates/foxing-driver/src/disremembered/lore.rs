//! L4: what a reader knows.
//!
//! David, 11-21: "For Read: more puzzles, inside jokes, harder tricks, crafting
//! bonuses, secret knowledge and all of the things an avid reader enjoys", and of
//! making, "reading should improve crafting capability". Then, 11-22: "l4a:1 / l4b:1 /
//! l4c:1 / l4d:yes / l4e: knowledge must be earned and cannot be traded".
//!
//! Lore is a thing read and understood, kept on the name that read it. Each piece
//! teaches one recipe that the bench will not otherwise give: the seven things "with
//! a past", and four reader's preparations that make a common material out of
//! commoner ones (the Watch of 11-21: "Reading never affects crafting, and the
//! Archive's ink text has no tie to the 'dab of ink' ingredient").
//!
//! It is had two ways, both by `study`:
//!
//! - **The five ends.** The world's text had five trails that stopped (the Watch of
//!   11-21: "The Crier 'forms' clue chain ... has no verb to follow"): the forms in
//!   the Crier's coat, the twelfth seat in the jury box, the Bailiff's ink that does
//!   not wash off, the Book Nobody Is Reading, the Ledger of Fines. Each now ends in
//!   a question that the trail answers.
//! - **The six folios.** Chained in the Reading Rooms and the Bindery: questions
//!   that want two or three readings put together, from rooms far apart. A folio
//!   asks each name a different question each day, so an answer overheard is the
//!   answer to somebody else's.
//!
//! Nothing here can be given, sold or told into another name: what is made from it
//! can be traded like any thing, and the knowing cannot. It fades as everything
//! does, three days from the reading, and reading it again keeps it.
//!
//! A reader's standing (`lore`) is counted from the last day as a keeper's and a
//! teller's are: Browser, Reader, Scholar, Antiquary.

use super::{combat, find_citizen, internal, plain_str, CITIZEN, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// A name learned something: `{by, lore, teaches}`. Public.
pub const LEARNED: &str = "disremembered.learned";

const DAY_MS: i64 = 86_400_000;
/// What is known keeps this long from the reading; a Scholar's, a week.
pub const LORE_MS: i64 = 3 * DAY_MS;
pub const LORE_SCHOLAR_MS: i64 = 7 * DAY_MS;
/// What an end pays when it is newly understood, and a folio once a day.
// (20 and 30 at first: three answers took a new name from nothing to level 72
// with no fight, and five folios ended a first hour's leave. David, 11-22: "halve it".)
pub const END_XP: f64 = 10.0;
pub const FOLIO_XP: f64 = 15.0;
/// A wrong answer, and the thing will not hear that name again this long.
pub const RETRY_MS: i64 = 10 * 60_000;

/// The rungs: points, title, what slips and folios pay a reader of that standing.
pub const TITLES: &[(usize, &str, f64)] = &[
    (3, "Browser", 1.0),
    (6, "Reader", 1.25),
    (9, "Scholar", 1.25),
    (12, "Antiquary", 1.5),
];

/// Which recipe each piece of lore teaches: `(recipe key, lore key)`.
pub const TAUGHT: &[(&str, &str)] = &[
    ("commonplace", "coat"),
    ("smoked-lens", "seat"),
    ("ink", "ink"),
    ("night-lantern", "book"),
    ("ribbon-marker", "ledger"),
    ("warden-charm", "doors"),
    ("surgeon-bandage", "ways"),
    ("sliver-jar", "letters"),
    ("thread", "makings"),
    ("card", "pins"),
    ("ember", "highest"),
];

pub struct Source {
    pub key: &'static str,
    /// What it is called in `lore`.
    pub title: &'static str,
    pub room: &'static str,
    /// The words `study` and `read` know it by, in its room.
    pub words: &'static [&'static str],
    /// A folio asks each name its own question each day.
    pub folio: bool,
    /// What studying it shows (an end's own text and question; a folio's heading).
    pub shown: &'static str,
    /// An end's right answers; a folio's are worked out.
    pub answers: &'static [&'static str],
    /// What is said when it is understood.
    pub learned: &'static str,
    /// Where `lore` says to look, for one who has not found it.
    pub rumor: &'static str,
}

pub const SOURCES: &[Source] = &[
    Source {
        key: "coat",
        title: "the forms in the Crier's coat",
        room: "lost-steps",
        words: &["coat", "crier", "forms", "form", "pocket", "pockets"],
        folio: false,
        shown: "The Crier of the List lets you look, as one lets rain. The coat is all pockets on the inside, each sewn with a number in the Clerk's hand, and each with a form folded in it to the size of a playing card: 7-B, 12, 40-A, 9. A fifth pocket is sewn with nothing and holds nothing, and the stitching round it has been unpicked and done again in a different thread. The Filing Window knows what the Clerk's pigeonholes are numbered. Which form does the Crier not have?",
        answers: &["3", "three", "pink", "3 pink", "3 (pink)", "the pink", "the pink one"],
        learned: "The Crier looks at the empty pocket, and then at you, for the first time. \"Three. Pink. It was never issued; they only numbered the hole.\" It takes the other four out and lets you see how each is ruled, folded and sewn at the spine, which is all a book of blanks is. You know how a commonplace book is made.",
        rumor: "Somebody waiting at the Filing Window wrote in pencil where the Court's missing forms are kept.",
    },
    Source {
        key: "seat",
        title: "the twelfth seat",
        room: "court-of-lethe",
        words: &["seat", "seats", "m", "twelfth", "plan", "juror", "jury"],
        folio: false,
        shown: "You turn the RESERVED card over again. Eleven blanks and an M, gone over until the pencil has shone the card. The twelfth cushion is the only one not dented: whoever sits there is not heavy, or is not there for long. Scratched under the rail in front of it, small, by somebody kept waiting through two sittings: comes twice a day, to wherever most of us are, and asks more than it answers. The Docent knows everyone's hours (`tell docent \"...\"`). Who is the twelfth juror?",
        answers: &["morphos"],
        learned: "The pencilled M takes a little of the light from the lamps, as if it had been listening for its name. Morphos sits on every jury and has never once been seen to arrive, and from the twelfth seat you can see why: the glass in the rail is smoked, and through it the room has one more in it than it has. You know how a smoked lens is made.",
        rumor: "The jury box in the Courtroom has a card that is not meant to be turned over.",
    },
    Source {
        key: "ink",
        title: "what takes the Bailiff's ink off",
        room: "court-of-lethe",
        words: &["ink", "inkpot", "bailiff", "tattoo", "forearm"],
        folio: false,
        shown: "The Bailiff lets you look at the inkpot and not touch it. Blue-black, with a brass cap on a chain; the crust round the neck has a green edge, like a ledger left in a cellar. NOTHING IS STRUCK IN THIS ROOM BUT THE RECORD, and what the Bailiff writes on a forearm does not wash off: \"nobody has yet found what does.\" The Bailiff has been in this room a long time, and has never once looked down at the floor. What takes the ink off?",
        answers: &["lethe", "the lethe", "water", "the water", "channel", "the channel", "river", "the river", "slow water", "the slow water", "lethe water"],
        learned: "You say it, and the Bailiff looks down at the channel of slow water in the floor for what may be the first time, and then away, quickly. The Court is the Court of Lethe: what runs through it takes anything written off anything. The ink is only lampblack, gall and that water's opposite, and you know how a dab of it is made without waiting on an inkling.",
        rumor: "The Bailiff's notice in the Courtroom says nobody has found what washes its ink off. The room has had the answer in it all along.",
    },
    Source {
        key: "book",
        title: "the Book Nobody Is Reading",
        room: "dark-reading-room",
        words: &["book", "books", "page", "nobody", "plaque"],
        folio: false,
        shown: "The Book Nobody Is Reading is open at a page about you. You get as far as your name, and the page turns; you look back, and it is a page about you again, a different one. The lamps are all switched off, and the print is perfectly clear. Every other book on the long tables is open too, and each is being read, attentively, by nobody, and the chairs are pushed back. Nobody is a borrower like any other, and the Overdue Desk keeps a line for every borrower (`read nobody`, there). What does Nobody owe the house?",
        answers: &["lamp", "a lamp", "one lamp", "one lamp unlit", "a lamp unlit", "an unlit lamp", "one unlit lamp", "unlit lamp", "the lamp", "lamp unlit"],
        learned: "\"One lamp, unlit,\" you say, and a chair two places down is drawn in an inch, politely, to let you past. Nobody is a regular. Nobody reads here every night with every lamp off, has never needed one, and owes the house the one it never lit; the page that would not stay still for you lies flat under that attention like a cat. You see how a light is made that is only a light to its bearer. You know how a night lantern is made.",
        rumor: "In the Dark Reading Room every book is open and the lamps are off. The room says who reads there, and the Overdue Desk says what that reader owes.",
    },
    Source {
        key: "ledger",
        title: "the fine in the Ledger",
        room: "overdue-desk",
        words: &["ledger", "fine", "fines", "column", "line"],
        folio: false,
        shown: "Your line is still there near the foot of the column: \"one memory, borrowed at the door, not yet returned. Accruing.\" Beside it, in the narrow column ruled for the thing lent, the same patient hand has written the shelf it came from: L. & F., the room to the north. The fine is struck when what was borrowed is named. What did you borrow at the door?",
        answers: &["name", "a name", "my name", "the name", "a name tag", "name tag", "the name tag", "my name tag"],
        learned: "The pen that is not in the room rules a line through yours, twice, and writes RETURNED in the margin, which is a lie and a kindness. You came in on a borrowed name; Lost and Found has had the tag all this time. In the column for the thing lent the desk has tied a ribbon, to mark the place where you stopped owing. You know how a ribbon marker is made.",
        rumor: "The Overdue Desk has a line in its ledger with your name on it, and the room to the north of it has something else with your name on it.",
    },
    Source {
        key: "doors",
        title: "the Folio of Doors",
        room: "lamp-table",
        words: &["folio", "doors"],
        folio: true,
        shown: "A folio chained to the table, bound in the grey of a door nobody painted. THE FOLIO OF DOORS. Every page is a door and what it says of who it is for.",
        answers: &[],
        learned: "The folio falls open at its last page, which is a door with a keyhole drawn the wrong way up, and a note: a warden's key opens nothing; it reminds the lock of having been shut. You know how a warden's charm is made.",
        rumor: "A folio about doors is chained in the Reading Rooms, at the Lamp Table.",
    },
    Source {
        key: "ways",
        title: "the Folio of Ways Out",
        room: "mezzanine",
        words: &["folio", "ways"],
        folio: true,
        shown: "A folio chained to the newel post, its corners rounded by thumbs. THE FOLIO OF WAYS OUT. Every page is a room, drawn as a dot with its ways out as spokes.",
        answers: &[],
        learned: "The last page is a room with one way out, drawn very small, and round it somebody has drawn a bandage as a surgeon ties one: twice round, so that what would not close has only one way left to go. You know how a surgeon's bandage is made.",
        rumor: "A folio about the ways out of rooms is chained on the Mezzanine.",
    },
    Source {
        key: "letters",
        title: "the Folio of Initials",
        room: "bindery",
        words: &["folio", "initials", "letters"],
        folio: true,
        shown: "A folio chained to the sewing frame, stamped with letters in blind. THE FOLIO OF INITIALS. The binders filed every room under the first letter of its name, leaving out The.",
        answers: &[],
        learned: "The folio's own spine is stamped with three letters you do not need, and under the pastedown the binders hid what they clipped from every coin that paid them: a sliver, kept in a jar, with a pinch of the sweat that came off it. You know how a sliver jar is made.",
        rumor: "A folio stamped with initials is chained in the Bindery, down in the Lower Stacks.",
    },
    Source {
        key: "makings",
        title: "the Folio of Makings",
        room: "returns-cage",
        words: &["folio", "makings"],
        folio: true,
        shown: "A folio chained inside the cage, its cover a patchwork of whatever was returned. THE FOLIO OF MAKINGS. Every page is a thing that can be made and what it is made of.",
        answers: &[],
        learned: "The last page is not a thing to make but a thing to unmake: river weed, dried and rolled against moth dust until it holds, is rag thread, and nobody need fight a ragpicker for it. You know how thread is twisted.",
        rumor: "A folio of things that can be made is chained in the Returns Cage, in the Reading Rooms.",
    },
    Source {
        key: "pins",
        title: "the Folio of Pins",
        room: "errata-desk",
        words: &["folio", "pins"],
        folio: true,
        shown: "A folio chained to the Errata Desk, full of pinholes. THE FOLIO OF PINS. Every page is a district, with a hole where its reference slip is pinned.",
        answers: &[],
        learned: "Held to the light, the pinholes of the last page spell a shelf mark, and the page itself is a catalogue card somebody cut down from rag thread and ink, ruled by hand. Nobody need go down to a cataloguer for one. You know how a card is cut.",
        rumor: "A folio full of pinholes is chained at the Errata Desk, in the Reading Rooms.",
    },
    Source {
        key: "highest",
        title: "the Folio of Thresholds",
        room: "proof-room",
        words: &["folio", "thresholds"],
        folio: true,
        shown: "A folio chained to the proofing bench, each page marked up in red. THE FOLIO OF THRESHOLDS. Every page is three doors, and a proofreader's query in the margin: which is the highest?",
        answers: &[],
        learned: "The last page has a scorch through it the size of a thumb. Margin, in red: grave dust and moth dust, struck together, hold a light as long as a wisp does, and query whether anybody has told the wisps. You know how an ember is struck.",
        rumor: "A folio marked up in red is chained in the Proof Room, in the Reading Rooms.",
    },
];

fn source(key: &str) -> Option<&'static Source> {
    SOURCES.iter().find(|s| s.key == key)
}

fn state(world: &World, name: &str) -> Value {
    find_citizen(world, name)
        .and_then(|f| world.store.entity(f.citizen))
        .and_then(|e| e.components.get(CITIZEN))
        .and_then(|c| match c.fields.get("lore") {
            Some(Field::Plain(v)) if v.is_object() => Some(v.clone()),
            _ => None,
        })
        .unwrap_or_else(|| json!({}))
}

fn write(world: &mut World, name: &str, v: Value) -> Result<(), CommandError> {
    let Some(f) = find_citizen(world, name) else {
        return Ok(());
    };
    let mut changes = BTreeMap::new();
    changes.insert("lore".to_string(), Field::Plain(v));
    super::set_citizen_fields(world, name, f.citizen, changes)
}

fn did_list(world: &World, name: &str) -> Vec<(String, i64)> {
    let now = world.now().0;
    state(world, name)["did"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|e| Some((e[0].as_str()?.to_string(), e[1].as_i64()?)))
        .filter(|(_, at)| now - at < DAY_MS)
        .collect()
}

/// Mark something a reader did, for the standing. Never fails what it rides on.
pub fn did(world: &mut World, name: &str, what: &str) {
    let mut list = did_list(world, name);
    list.push((what.to_string(), world.now().0));
    if list.len() > 80 {
        let cut = list.len() - 80;
        list.drain(..cut);
    }
    let mut st = state(world, name);
    st["did"] = json!(list
        .iter()
        .map(|(w, at)| json!([w, at]))
        .collect::<Vec<_>>());
    let _ = write(world, name, st);
}

fn count(world: &World, name: &str, what: &str) -> usize {
    did_list(world, name)
        .iter()
        .filter(|(w, _)| w == what || w.starts_with(&format!("{what}:")))
        .count()
}

fn keeps_ms(world: &World, name: &str) -> i64 {
    if rung(world, name) >= 3 {
        LORE_SCHOLAR_MS
    } else {
        LORE_MS
    }
}

/// What a name knows now: `(lore key, ms left)`.
pub fn known(world: &World, name: &str) -> Vec<(&'static str, i64)> {
    let now = world.now().0;
    let st = state(world, name);
    // Counted against the plain three days here, and the Scholar's week is applied
    // by `knows`, so that the standing (which counts what is known) does not turn
    // on itself.
    SOURCES
        .iter()
        .filter_map(|s| {
            let at = st["known"][s.key].as_i64()?;
            let left = at + LORE_MS - now;
            (left > 0).then_some((s.key, left))
        })
        .collect()
}

/// Whether a name knows a piece of lore, a Scholar's week allowed for.
pub fn knows(world: &World, name: &str, key: &str) -> Option<i64> {
    let at = state(world, name)["known"][key].as_i64()?;
    let left = at + keeps_ms(world, name) - world.now().0;
    (left > 0).then_some(left)
}

/// A reader's standing and what it is counted from.
pub fn standing(world: &World, name: &str) -> (usize, Vec<String>) {
    let lore = known(world, name).len();
    let slips = count(world, name, "slip");
    let folios = count(world, name, "folio");
    let ends = count(world, name, "end");
    let made = count(world, name, "made");
    let mut points = 0;
    let mut said = Vec::new();
    let mut add = |n: usize, why: String| {
        if n > 0 {
            points += n;
            said.push(format!(
                "{why} ({n} point{})",
                if n == 1 { "" } else { "s" }
            ));
        }
    };
    add(
        match lore {
            0 | 1 => 0,
            2..=4 => 1,
            5..=7 => 2,
            8..=10 => 3,
            _ => 4,
        },
        format!("{lore} of the eleven things known"),
    );
    add(
        match slips {
            0 => 0,
            1..=5 => 1,
            6..=11 => 2,
            _ => 3,
        },
        format!(
            "{slips} slip{} answered today",
            if slips == 1 { "" } else { "s" }
        ),
    );
    add(
        match folios {
            0 => 0,
            1 | 2 => 1,
            3..=5 => 2,
            _ => 3,
        },
        format!(
            "{folios} folio{} answered today",
            if folios == 1 { "" } else { "s" }
        ),
    );
    // (One point for any number of them at first: all five trails moved a reader
    // one point, the Watch of 11-22 evening.)
    add(
        match ends {
            0 => 0,
            1 | 2 => 1,
            3 | 4 => 2,
            _ => 3,
        },
        format!(
            "{ends} trail{} followed to the end today",
            if ends == 1 { "" } else { "s" }
        ),
    );
    add(
        match made {
            0 => 0,
            1 | 2 => 1,
            _ => 2,
        },
        format!(
            "{made} thing{} made today from what you know",
            if made == 1 { "" } else { "s" }
        ),
    );
    (points, said)
}

pub fn rung(world: &World, name: &str) -> usize {
    let (p, _) = standing(world, name);
    TITLES.iter().filter(|(need, _, _)| p >= *need).count()
}

pub fn title(world: &World, name: &str) -> Option<&'static str> {
    match rung(world, name) {
        0 => None,
        r => Some(TITLES[r - 1].1),
    }
}

/// What slips and folios pay this reader, times.
pub fn pay_times(world: &World, name: &str) -> f64 {
    match rung(world, name) {
        0 => 1.0,
        r => TITLES[r - 1].2,
    }
}

/// How long a wrong answer shuts a slip or a folio to this reader: a Browser's, half.
pub fn retry_ms(world: &World, name: &str, plain: i64) -> i64 {
    if rung(world, name) >= 1 {
        plain / 2
    } else {
        plain
    }
}

/// The bench will not make this for one who has not read how: what to say, or None
/// where the recipe is free or known.
pub fn locked(world: &World, name: &str, recipe: &str) -> Option<String> {
    let (_, key) = TAUGHT.iter().find(|(r, _)| *r == recipe)?;
    if knows(world, name, key).is_some() {
        return None;
    }
    let s = source(key)?;
    Some(format!(
        "its making is written down, and you have not read it. {} (`lore` lists what there is to know; `study` in the right room reads it.)",
        s.rumor
    ))
}

/// A thing was made: counted for a reader where the recipe came of lore.
pub fn made(world: &mut World, name: &str, recipe: &str) {
    if TAUGHT.iter().any(|(r, _)| *r == recipe) {
        did(world, name, "made");
    }
}

fn room_key(world: &World, room: EntityId) -> String {
    plain_str(world, room, ROOM, "key")
        .unwrap_or("")
        .to_string()
}

fn room_name(world: &World, key: &str) -> String {
    super::rooms::room_by_key(world, key)
        .and_then(|r| plain_str(world, r, ROOM, "name"))
        .unwrap_or(key)
        .to_string()
}

fn here_sources(world: &World, room: EntityId) -> Vec<&'static Source> {
    let key = room_key(world, room);
    SOURCES.iter().filter(|s| s.room == key).collect()
}

fn find_here(world: &World, room: EntityId, arg: &str) -> Option<&'static Source> {
    let a = arg.trim().trim_matches('"').to_lowercase();
    let a = a
        .trim_start_matches("the ")
        .trim_start_matches("a ")
        .trim_start_matches("an ");
    let all = here_sources(world, room);
    if a.is_empty() {
        return if all.len() == 1 { Some(all[0]) } else { None };
    }
    all.into_iter().find(|s| {
        s.words
            .iter()
            .any(|w| a == *w || a.split_whitespace().any(|x| x == *w))
    })
}

fn day(world: &World) -> i64 {
    world.now().0.div_euclid(DAY_MS)
}

/// A number that is this name's own, today, for this folio.
fn salt(name: &str, day: i64, what: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in name
        .to_lowercase()
        .bytes()
        .chain(day.to_le_bytes())
        .chain(what.bytes())
    {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h ^ (h >> 29)
}

/// Distinct picks from a list of `n`, this name's own for today.
fn picks(name: &str, day: i64, what: &str, n: usize, k: usize) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    let mut i = 0u64;
    while out.len() < k.min(n) {
        let p = (salt(name, day, &format!("{what}{i}")) % n as u64) as usize;
        if !out.contains(&p) {
            out.push(p);
        }
        i += 1;
    }
    out
}

/// Rooms whose ways out are counted and whose neighbors are named: the world's own,
/// near the Door, that every name can reach.
const NEAR: &[&str] = &[
    "fogbank",
    "lantern-row",
    "orchard",
    "archive",
    "midden",
    "lost-steps",
    "orchard-wall",
    "lamplighters-lane",
    "mezzanine",
    "watch-fire",
];

/// Recipes any name can read at the bench.
const OPEN_RECIPES: &[&str] = &[
    "lantern",
    "charm",
    "bandage",
    "cordial",
    "poultice",
    "jar",
    "book",
    "plaque",
    "lamp",
    "bell",
    "basin",
    "buckler",
    "salve",
    "dart",
    "splint",
    "brine-glass",
];

fn initial(name: &str) -> char {
    name.trim_start_matches("The ")
        .trim_start_matches("the ")
        .chars()
        .find(|c| c.is_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('?')
}

/// The doors that say who they are for: a room with a level, entered from at least
/// one room that has none or a lower one.
fn shown_gates(world: &World) -> Vec<(&'static str, f64)> {
    super::rooms::GATES
        .iter()
        .copied()
        .filter(|(key, need)| {
            let Some(room) = super::rooms::room_by_key(world, key) else {
                return false;
            };
            world.store.of_kind("room").any(|e| {
                e.owner.is_none()
                    && super::rooms::exits_of(world, e.id)
                        .values()
                        .any(|to| *to == room)
                    && super::rooms::gate_of(world, e.id).map_or(true, |g| g < *need)
            })
        })
        .collect()
}

/// A folio's question for this name today, and what answers it.
pub struct Asked {
    pub question: String,
    /// Every one of these must be in the answer (words), or the number must match.
    pub words: Vec<String>,
    pub number: Option<f64>,
    /// How far off a number may be.
    pub within: f64,
}

pub fn ask(world: &World, name: &str, key: &str) -> Option<Asked> {
    let d = day(world);
    // Only the doors that say a level to one who walks up to them: a room entered
    // from one as deep says nothing (the Watch of 11-22 evening was asked what "the
    // door of The Leak says", and no door to it says anything).
    let gates = &shown_gates(world)[..];
    match key {
        "doors" => {
            let p = picks(name, d, key, gates.len(), 2);
            let (a, b) = (gates[p[0]], gates[p[1]]);
            Some(Asked {
                question: format!(
                    "Today's page, in your hand already, as if you had copied it out and forgotten: add the level the door of {} says to the level the door of {} says. (A door says who it is for to whoever walks up to it; `where <room>` finds one.)",
                    room_name(world, a.0),
                    room_name(world, b.0)
                ),
                words: Vec::new(),
                number: Some(a.1 + b.1),
                within: 0.0,
            })
        }
        "highest" => {
            // Three doors of three different levels.
            let mut chosen: Vec<(&str, f64)> = Vec::new();
            for i in picks(name, d, key, gates.len(), gates.len()) {
                if chosen.iter().all(|c| c.1 != gates[i].1) {
                    chosen.push(gates[i]);
                }
                if chosen.len() == 3 {
                    break;
                }
            }
            let top = chosen.iter().max_by(|a, b| a.1.total_cmp(&b.1))?;
            let top_name = room_name(world, top.0);
            Some(Asked {
                question: format!(
                    "Today's page, the three doors on it ringed in red: {}, {}, {}. Which asks the highest level? Answer with the room's name.",
                    room_name(world, chosen[0].0),
                    room_name(world, chosen[1].0),
                    room_name(world, chosen[2].0)
                ),
                words: significant(&top_name),
                number: None,
                within: 0.0,
            })
        }
        "ways" => {
            let p = picks(name, d, key, NEAR.len(), 2);
            let rooms: Vec<EntityId> = p
                .iter()
                .filter_map(|i| super::rooms::room_by_key(world, NEAR[*i]))
                .collect();
            if rooms.len() < 2 {
                return None;
            }
            let n: usize = rooms
                .iter()
                .map(|r| super::rooms::exits_of(world, *r).len())
                .sum();
            Some(Asked {
                question: format!(
                    "Today's page is two dots with their spokes rubbed out: {} and {}. How many ways lead out of the two together, as they stand today? (`look` in a room lists its ways out; a door dug since this morning counts.)",
                    room_name(world, NEAR[p[0]]),
                    room_name(world, NEAR[p[1]])
                ),
                words: Vec::new(),
                number: Some(n as f64),
                // A door may be dug while the reader walks back.
                within: 1.0,
            })
        }
        "letters" => {
            let p = picks(name, d, key, NEAR.len(), 3);
            let mut asked = Vec::new();
            let mut letters = String::new();
            for i in p {
                let room = super::rooms::room_by_key(world, NEAR[i])?;
                let exits = super::rooms::exits_of(world, room);
                // The world's own ways only: a dug door comes and goes.
                let (via, to) = exits
                    .iter()
                    .find(|(_, to)| world.store.entity(**to).is_some_and(|e| e.owner.is_none()))?;
                asked.push(format!("the room {via} of {}", room_name(world, NEAR[i])));
                letters.push(initial(plain_str(world, *to, ROOM, "name").unwrap_or("?")));
            }
            Some(Asked {
                question: format!(
                    "Today's page wants three initials, in this order, each the first letter of a room's name, The left out: {}; {}; {}. Answer with the three letters.",
                    asked[0], asked[1], asked[2]
                ),
                words: vec![letters.to_lowercase()],
                number: None,
                within: 0.0,
            })
        }
        "makings" => {
            // Hardened (David, 11-22: "harden them"): it was two `recipes` reads and no
            // walking. Now the count of materials, and a walk to where one is found.
            let all = super::craft::all_recipes(world);
            let walked: Vec<(&super::craft::Rx, &str, &str)> = OPEN_RECIPES
                .iter()
                .filter_map(|k| all.iter().find(|r| r.key == *k))
                .filter_map(|r| {
                    if r.of.iter().any(|m| m == "rag thread") {
                        Some((r, "rag thread", "midden"))
                    } else if r.of.iter().any(|m| m == "hollow water") {
                        Some((r, "hollow water", "fogbank"))
                    } else {
                        None
                    }
                })
                .collect();
            if walked.is_empty() {
                return None;
            }
            let (a, material, room_key) = walked[picks(name, d, key, walked.len(), 1)[0]];
            let others: Vec<&super::craft::Rx> = OPEN_RECIPES
                .iter()
                .filter_map(|k| all.iter().find(|r| r.key == *k))
                .filter(|r| r.key != a.key)
                .collect();
            let b = others[picks(name, d, "makings-b", others.len(), 1)[0]];
            let distinct: BTreeSet<&str> =
                a.of.iter().chain(b.of.iter()).map(String::as_str).collect();
            let room = super::rooms::room_by_key(world, room_key)?;
            let ways = super::rooms::exits_of(world, room).len();
            Some(Asked {
                question: format!(
                    "Today's page is two things, their makings torn off: {} and {}. Count the different materials the two ask between them, one they share counted once and no memory counted (`recipes {}` and `recipes {}`). Then go to the first room the bench names for {material} (`recipes {}` says where it is found), and add the ways that lead out of that room as it stands today. What is the sum?",
                    a.name, b.name, a.key, b.key, a.key
                ),
                words: Vec::new(),
                number: Some((distinct.len() + ways) as f64),
                within: 1.0,
            })
        }
        "pins" => {
            // Hardened: it was one read of `slips`. Now the two rooms are gone to.
            let open: Vec<&super::slips::Slip> = super::slips::SLIPS
                .iter()
                .filter(|sl| {
                    super::rooms::room_by_key(world, sl.room)
                        .is_some_and(|r| super::rooms::gate_of(world, r).is_none())
                })
                .collect();
            if open.len() < 2 {
                return None;
            }
            let p = picks(name, d, key, open.len(), 2);
            let (a, b) = (open[p[0]], open[p[1]]);
            let rooms = [
                super::rooms::room_by_key(world, a.room)?,
                super::rooms::room_by_key(world, b.room)?,
            ];
            let ways: usize = rooms
                .iter()
                .map(|r| super::rooms::exits_of(world, *r).len())
                .sum();
            let (ia, ib) = (
                initial(&room_name(world, a.room)),
                initial(&room_name(world, b.room)),
            );
            Some(Asked {
                question: format!(
                    "Today's page is two districts with the pinholes filled in: {} and {}. Find the room each one's reference slip is pinned in (`slips` lists the twelve) and go to both. Answer with the two rooms' initials, The left out, in that order, and then the number of ways that lead out of the two rooms together as they stand today: two letters and a number, as in \"AB 7\".",
                    a.district, b.district
                ),
                words: vec![format!("{ia}{ib}").to_lowercase()],
                number: Some(ways as f64),
                within: 1.0,
            })
        }
        _ => None,
    }
}

/// The words of a room's name that an answer must have: The and the small words out.
fn significant(name: &str) -> Vec<String> {
    name.split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|w| w.len() > 2 && !["the", "and", "of", "for"].contains(&w.as_str()))
        .collect()
}

fn norm(answer: &str) -> String {
    answer
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn right(world: &World, name: &str, s: &Source, answer: &str) -> bool {
    let a = norm(answer);
    if !s.folio {
        let me = name.to_lowercase();
        // "THE 3 (PINK)" was turned away where "3 (pink)" was right.
        let bare = |x: &str| {
            let n = norm(x);
            n.trim_start_matches("the ")
                .trim_start_matches("a ")
                .to_string()
        };
        let a = bare(&a);
        return s.answers.iter().any(|x| bare(x) == a) || (s.key == "ledger" && a == me);
    }
    let Some(asked) = ask(world, name, s.key) else {
        return false;
    };
    if let Some(n) = asked.number {
        let said = a
            .split_whitespace()
            .find_map(|w| w.parse::<f64>().ok())
            .or_else(|| number_word(&a));
        if !said.is_some_and(|x| (x - n).abs() <= asked.within) {
            return false;
        }
        if asked.words.is_empty() {
            return true;
        }
        // Letters and a number: "AB 7", "a b 7", "ab7".
        let letters: String = a.chars().filter(|c| c.is_alphabetic()).collect();
        return asked.words.iter().all(|w| letters == *w);
    }
    // Letters run together or spaced; names by their words.
    let squeezed: String = a.split_whitespace().collect();
    asked
        .words
        .iter()
        .all(|w| a.split_whitespace().any(|x| x == w) || squeezed == *w)
}

fn number_word(a: &str) -> Option<f64> {
    const WORDS: &[&str] = &[
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
        "eleven", "twelve",
    ];
    a.split_whitespace()
        .find_map(|w| WORDS.iter().position(|x| *x == w))
        .map(|n| n as f64)
}

/// David, 11-22: "first mistakes for players on keyboard can be free". One miss a
/// day at each thing costs a name at the keyboard nothing: true, and marked, where
/// this is that one. `what` is the thing: a lore key, or `slip:<room>`.
pub fn free_miss(world: &mut World, name: &str, what: &str) -> bool {
    if !combat::at_a_keyboard(world, name) {
        return false;
    }
    let today = day(world);
    let mut st = state(world, name);
    if st["missed"][what].as_i64() == Some(today) {
        return false;
    }
    st["missed"][what] = json!(today);
    write(world, name, st).is_ok()
}

fn shut_for(world: &World, name: &str, key: &str) -> Option<i64> {
    let at = state(world, name)["tried"][key].as_i64()?;
    let left = at + retry_ms(world, name, RETRY_MS) - world.now().0;
    (left > 0).then_some(left)
}

fn folio_done_today(world: &World, name: &str, key: &str) -> bool {
    state(world, name)["folio_day"][key].as_i64() == Some(day(world))
}

/// What studying a thing shows, with no answer given: its text and its question.
fn shown(world: &World, name: &str, s: &Source) -> String {
    let question = if s.folio {
        match ask(world, name, s.key) {
            // On its own line: the question was lost in the paragraph on a phone.
            Some(a) => format!("\n{}\n", a.question),
            None => " The page for today has been torn out; come back tomorrow.".to_string(),
        }
    } else {
        String::new()
    };
    let state_said = match knows(world, name, s.key) {
        Some(left) => format!(
            " (You know this already, {} more; answering it again keeps it.)",
            combat::human(left)
        ),
        None => String::new(),
    };
    let word = s.words[0];
    format!(
        "{}{question} `study {word} \"<your answer>\"` answers it: right, and you know something the bench will make for you; wrong, and it will not hear you for {} minutes.{state_said}",
        s.shown,
        retry_ms(world, name, RETRY_MS) / 60_000
    )
}

/// `read coat`, `read folio` and the rest answer with the thing's own text.
pub fn read(world: &World, name: &str, room: EntityId, arg: &str) -> Option<Outcome> {
    if arg.trim().is_empty() {
        return None;
    }
    let s = find_here(world, room, arg)?;
    // The jury box, the ledger and the inkpot have their first reading in the
    // room's own nouns; this is the second, for the words that reach further.
    Some(Outcome::new(
        shown(world, name, s),
        json!({"lore": s.key, "known": knows(world, name, s.key).is_some()}),
    ))
}

/// The word to `study` a thing by, where this room has one that answers to `arg`.
pub fn further(world: &World, room: EntityId, arg: &str) -> Option<&'static str> {
    find_here(world, room, arg).map(|s| s.words[0])
}

/// What `look` adds: a folio chained here, or a trail's end.
pub fn look_line(world: &World, room: EntityId, name: &str) -> Option<String> {
    let all = here_sources(world, room);
    if all.is_empty() {
        return None;
    }
    let said: Vec<String> = all
        .iter()
        .map(|s| {
            let known = knows(world, name, s.key).is_some();
            if s.folio {
                format!(
                    "{} is chained here (`study folio`){}",
                    cap(s.title),
                    if folio_done_today(world, name, s.key) {
                        ": answered today"
                    } else if known {
                        ": you know what it teaches, and it asks a new page each day"
                    } else {
                        ""
                    }
                )
            } else if known {
                format!("You have made something of {} (`lore`)", s.title)
            } else {
                format!(
                    "There is more to be made of {} than reading it once (`study {}`)",
                    s.title, s.words[0]
                )
            }
        })
        .collect();
    Some(format!("{}.", said.join(". ")))
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn recipe_of(key: &str) -> &'static str {
    TAUGHT
        .iter()
        .find(|(_, l)| *l == key)
        .map(|(r, _)| *r)
        .unwrap_or("?")
}

/// Every piece, known as of now: the tests of things made, which are not about the
/// reading.
#[cfg(test)]
pub fn teach_all(world: &mut World, name: &str) {
    let mut st = state(world, name);
    for s in SOURCES {
        st["known"][s.key] = json!(world.now().0);
    }
    let _ = write(world, name, st);
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.lore",
            "What you know that the bench will make for you, and how long you will know it; what there is still to read, and where it is said to be; and your standing as a reader (Browser, Reader, Scholar, Antiquary), counted from the last day. Knowledge is earned by reading (`study`) and stays with the name that read it: it cannot be given, sold or told. A read.",
        )
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, _args: &Args| {
            super::me(w, ctx)?;
            let name = ctx.actor.clone();
            let mine: Vec<String> = SOURCES
                .iter()
                .filter_map(|s| {
                    knows(w, &name, s.key).map(|left| {
                        format!(
                            "{} (`craft {}`; {} more; read again with `study {}` in {})",
                            s.title,
                            recipe_of(s.key),
                            combat::human(left),
                            s.words[0],
                            room_name(w, s.room)
                        )
                    })
                })
                .collect();
            let unread: Vec<String> = SOURCES
                .iter()
                .filter(|s| knows(w, &name, s.key).is_none())
                .map(|s| {
                    // What it teaches is no secret; how to read it is the puzzle.
                    let made = super::craft::RECIPES
                        .iter()
                        .find(|r| r.key == recipe_of(s.key))
                        .map(|r| r.name)
                        .unwrap_or("something");
                    format!(
                        "{} It teaches {} (`recipes {}`).",
                        s.rumor,
                        made,
                        recipe_of(s.key)
                    )
                })
                .collect();
            let (points, said) = standing(w, &name);
            let r = rung(w, &name);
            let now_title = match r {
                0 => "no title yet".to_string(),
                r => TITLES[r - 1].1.to_string(),
            };
            let next = match TITLES.get(r) {
                Some((need, t, _)) => format!(
                    " {t} at {need}: {}.",
                    match r {
                        0 => "a wrong answer shuts a slip or a folio to you for half as long",
                        1 => "slips and folios pay a quarter more",
                        2 => "what you know keeps a week, not three days",
                        _ => "slips and folios pay half again",
                    }
                ),
                None => String::new(),
            };
            Ok(Outcome::new(
                format!(
                    "{} {} A reader's standing: {points}, {now_title}. {}{next} It is counted from the last day. What is known is known for three days from the reading, and reading it again keeps it; it cannot be given or told.",
                    if mine.is_empty() {
                        "You know nothing yet that the bench does not give everyone.".to_string()
                    } else {
                        format!("You know:\n- {}", mine.join("\n- "))
                    },
                    if unread.is_empty() {
                        "There is nothing left to read that you have not.".to_string()
                    } else {
                        format!("\nStill to be read, as it is told:\n- {}\n", unread.join("\n- "))
                    },
                    if said.is_empty() {
                        "Nothing counts toward it yet: a slip answered and two things known would be two.".to_string()
                    } else {
                        format!("Counted from: {}.", said.join("; "))
                    }
                ),
                json!({
                    "known": SOURCES.iter().filter(|s| knows(w, &name, s.key).is_some()).map(|s| s.key).collect::<Vec<_>>(),
                    "points": points,
                    "rung": r,
                    "title": title(w, &name),
                }),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.study",
            "Read a thing closely, where it is: a folio chained in a room, or one of the things the world's text leaves a trail to. Bare, or with the thing alone (`study folio`, `study coat`): its text and the question it ends in. With an answer in quotes (`study folio \"12\"`): right, and you know how something is made that the bench gives nobody else, for three days, and are paid for the reading (an end 10 when newly understood, a folio 15 once a day); wrong, and it will not hear you for ten minutes (a first miss a day at each is free to a name at the keyboard). A folio asks each name its own question each day, so nobody else's answer is yours. `lore` says what you know and what is still to read.",
        )
        .param("thing", ParamType::Entity, false, "what to study here: folio, or the thing's name")
        .param("answer", ParamType::Text, false, "your answer, quoted")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = super::me(w, ctx)?;
            let name = ctx.actor.clone();
            let here = super::rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let mut thing = args.str("thing").unwrap_or("").trim().to_string();
            let mut answer = args.str("answer").unwrap_or("").trim().to_string();
            let all = here_sources(w, here);
            if all.is_empty() {
                return Err(CommandError::precondition(
                    "There is nothing here that repays study: a folio chained in a room does, and a few things the world's text leaves a trail to. `lore` says where they are said to be.",
                ));
            }
            // The gate reads two bare words as one argument: `study folio 12`.
            if answer.is_empty() {
                if let Some((t, a)) = thing.clone().split_once(' ') {
                    let rest_is_the_thing = find_here(w, here, t)
                        .is_some_and(|s| a.split_whitespace().all(|x| s.words.contains(&x.to_lowercase().as_str())));
                    if find_here(w, here, t).is_some() && !rest_is_the_thing {
                        thing = t.to_string();
                        answer = a.to_string();
                    }
                }
            }
            let Some(s) = find_here(w, here, &thing) else {
                return Err(CommandError::precondition(format!(
                    "Study what? Here: {}.",
                    all.iter()
                        .map(|s| format!("`study {}` ({})", s.words[0], s.title))
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            };
            if answer.is_empty() {
                return Ok(Outcome::new(
                    shown(w, &name, s),
                    json!({"lore": s.key, "asked": true}),
                ));
            }
            if let Some(left) = shut_for(w, &name, s.key) {
                return Err(CommandError::precondition(format!(
                    "{} will not hear you again for {}: a wrong answer shuts it a while, so that the reading is cheaper than the guessing.",
                    cap(s.title),
                    combat::human(left)
                )));
            }
            if s.folio && folio_done_today(w, &name, s.key) && knows(w, &name, s.key).is_some() {
                return Err(CommandError::precondition(format!(
                    "You have answered {} today, and it has a new page for you tomorrow.",
                    s.title
                )));
            }
            if !right(w, &name, s, &answer) {
                if free_miss(w, &name, s.key) {
                    return Ok(Outcome::new(
                        format!(
                            "That is not it, and a first miss at the keyboard costs nothing: {} hears you again at once. The next miss today shuts it for {} minutes. The answer is in the world, not on this page.",
                            s.title,
                            retry_ms(w, &name, RETRY_MS) / 60_000
                        ),
                        json!({"lore": s.key, "right": false, "free": true}),
                    ));
                }
                let mut st = state(w, &name);
                st["tried"][s.key] = json!(w.now().0);
                write(w, &name, st)?;
                return Ok(Outcome::new(
                    format!(
                        "That is not it. {} will not hear you again for {} minutes; the answer is in the world, not on this page.",
                        cap(s.title),
                        retry_ms(w, &name, RETRY_MS) / 60_000
                    ),
                    json!({"lore": s.key, "right": false}),
                ));
            }
            let was_known = knows(w, &name, s.key).is_some();
            let times = pay_times(w, &name);
            let mut st = state(w, &name);
            st["known"][s.key] = json!(w.now().0);
            if s.folio {
                st["folio_day"][s.key] = json!(day(w));
            }
            write(w, &name, st)?;
            let curves = super::Curves::from_env();
            // A folio pays once a day; an end pays when it is newly understood.
            let paid = if s.folio {
                Some((FOLIO_XP * times).round())
            } else if !was_known {
                Some(END_XP)
            } else {
                None
            };
            let chunk = match paid {
                Some(xp) => Some(super::memory::earn_chunk(
                    w,
                    &name,
                    f.window,
                    &format!("{} understood", s.title),
                    xp,
                    0.8,
                    &curves,
                )?),
                None => None,
            };
            did(w, &name, if s.folio { "folio" } else { "end" });
            let _ = w.note(
                Actor::Citizen(name.clone()),
                LEARNED,
                json!({"by": name, "lore": s.key, "teaches": recipe_of(s.key), "again": was_known}),
            );
            // The one piece of this knowledge that is a use and not a recipe: a name
            // that knows what takes the ink off, and wears some, is standing beside it.
            let washed = if s.key == "ink" && super::courthouse::tattoo_of(w, &name).is_some() {
                let _ = super::courthouse::lift_tattoo(w, &name);
                " You kneel and put your forearm in the channel. The Bailiff's writing lifts off it in one piece, like a leaf, and goes downstream. The Bailiff does not watch."
            } else {
                ""
            };
            let keeps = combat::human(keeps_ms(w, &name));
            Ok(Outcome::new(
                format!(
                    "{}{} `craft {}` is yours to make for {keeps}, and {} (`recipes {}` says what it asks and where its makings are). What you know cannot be given or told: it is yours by the reading.{}{washed}",
                    if was_known { "Right again. " } else { "" },
                    s.learned,
                    recipe_of(s.key),
                    if s.folio { "answering tomorrow's page, or any day's before it fades, keeps it" } else { "answering it again at any time keeps it" },
                    recipe_of(s.key),
                    match (chunk, paid) {
                        (Some(c), Some(xp)) => format!(" You remember working it out: chunk-{} worth {xp:.0} XP.", c.0),
                        _ => String::new(),
                    }
                ),
                json!({"lore": s.key, "right": true, "teaches": recipe_of(s.key), "xp": paid}),
            ))
        }),
    );
}
