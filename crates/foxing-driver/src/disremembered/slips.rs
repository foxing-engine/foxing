//! C10 (David, 11-17: "let's work on pushing c10"): for the agents. "Each district
//! gets one thing that rewards reading over reflexes... Agents are half the players
//! and the content so far is written for hands."
//!
//! The reference slips. In one room of each district a slip is pinned: a question a
//! reader left at the desk, which the library pays whoever answers. No slip can be
//! answered from its own room's text or from memory of yesterday: each asks for
//! something the world says somewhere else, as it stands when it is asked (a
//! signpost's arm, the mud on a wall, the day's news, a register under the clock,
//! the Tide Table, the Book), and several ask for two readings put together or for
//! what will follow from what is written. `read slip` gives the question and says
//! where a reader might look; `solve "<words>"` answers it; `slips` lists the twelve.
//! A right answer is a memory, once a day (UTC) to a name for each slip, and all
//! twelve in a day is the Reader's Round. A wrong answer costs nothing but time:
//! the slip will not hear that name again for ten minutes (thirty where the answer
//! is one of three), so the reading is cheaper than the guessing.
//!
//! Two of the twelve are the ones the plan named. The Card Catalogue is a real index
//! of the public record: `index <a name, a named creature, a kind, a room>` in
//! the Catalogue pulls its card, from the world as it stands. And the Division
//! Lobby's slip asks what the next division will do, which the Book and the order
//! paper settle before the clock does.

use super::{find_citizen, plain_num, plain_str, Found, CITIZEN, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{
    ActorKind, Args, CallContext, CommandError, CommandSpec, Outcome, ParamType,
};
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A slip was answered rightly: `{by, district, room}`. Public.
pub const ANSWERED: &str = "disremembered.slip";
/// What a slip pays, and how vivid the memory of having looked it up.
/// (D57, David 11-21: "slips pay less". It was 25: ten slips took a reader past
/// level 100 in eight minutes and ended its leave without a fight.)
pub const SLIP_XP: f64 = 15.0;
const SLIP_VIVIDITY: f64 = 0.8;
/// All twelve in one day.
pub const ROUND_XP: f64 = 100.0;
/// A slip will not hear a name again this long after a wrong answer.
const RETRY_MS: i64 = 10 * 60 * 1000;
/// And this long where the answer is one of three.
const RETRY_CHOICE_MS: i64 = 30 * 60 * 1000;
/// An answer is a few words.
const ANSWER_WORDS: usize = 10;
const DAY_MS: i64 = 86_400_000;

pub struct Slip {
    /// The district it is the slip of, as `DISTRICTS` names it.
    pub district: &'static str,
    /// The key of the room it is pinned in.
    pub room: &'static str,
    /// Where in the room, for `look`.
    pub pinned: &'static str,
}

pub const SLIPS: &[Slip] = &[
    Slip {
        district: "the Door",
        room: "threshold",
        pinned: "to the signpost",
    },
    Slip {
        district: "the Diggings",
        room: "mudroom",
        pinned: "to the rack of spades",
    },
    Slip {
        district: "the Wilds",
        room: "fogbank",
        pinned: "to a post that stands clear of the fog",
    },
    Slip {
        district: "the Library",
        room: "namery",
        pinned: "in an empty pigeonhole",
    },
    Slip {
        district: "the Deep Water",
        room: "stairwell",
        pinned: "to the newel at the head of the stair",
    },
    Slip {
        district: "the Stacks",
        room: "catalogue",
        pinned: "to the front of a drawer",
    },
    Slip {
        district: "the Clock Quarter",
        room: "orchard",
        pinned: "to the trunk of the nearest tree",
    },
    Slip {
        district: "the Conclave",
        room: "division-lobby",
        pinned: "in the clerk's tally book, as a marker",
    },
    Slip {
        district: "the Exergue District",
        room: "mintmark-hall",
        pinned: "to the clerk's empty stool",
    },
    Slip {
        district: "the Wistful Trail",
        room: "trail-lookout",
        pinned: "under the brass plate",
    },
    Slip {
        district: "the Reading Rooms",
        room: "errata-desk",
        pinned: "uppermost on the heap",
    },
    Slip {
        district: "the Drowned Wing",
        room: "wing-wrack",
        pinned: "to the frame of the Tide Table",
    },
];

fn slip_in(world: &World, room: EntityId) -> Option<&'static Slip> {
    let key = plain_str(world, room, ROOM, "key")?;
    SLIPS.iter().find(|s| s.room == key)
}

/// A question as it stands now: its text, where to look, and what answers it.
struct Asked {
    text: String,
    look: &'static str,
    /// Every group must be met: the answer has one of the group's phrases in it.
    need: Vec<Vec<String>>,
    /// None of these may be in the answer (the other choices, so naming them all
    /// answers nothing).
    forbid: Vec<String>,
    /// Or a figure, and how far off it may be.
    number: Option<(f64, f64)>,
    /// One of three: a wrong answer waits longer.
    choice: bool,
}

impl Asked {
    fn named(text: String, look: &'static str, any: Vec<String>) -> Self {
        Asked {
            text,
            look,
            need: vec![any],
            forbid: Vec::new(),
            number: None,
            choice: false,
        }
    }
    fn figure(text: String, look: &'static str, n: f64, within: f64) -> Self {
        Asked {
            text,
            look,
            need: Vec::new(),
            forbid: Vec::new(),
            number: Some((n, within)),
            choice: false,
        }
    }
}

/// An answer or a phrase, flattened for comparing: lower case, letters and figures
/// and single spaces, no articles; `gift_tax` and `gift-tax` are `gift tax`.
fn norm(s: &str) -> String {
    let flat: String = s
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' {
                c
            } else {
                ' '
            }
        })
        .collect();
    flat.split_whitespace()
        .map(|w| w.trim_matches('.'))
        .filter(|w| !w.is_empty() && !matches!(*w, "the" | "a" | "an"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn has(answer: &str, phrase: &str) -> bool {
    let p = norm(phrase);
    !p.is_empty() && format!(" {answer} ").contains(&format!(" {p} "))
}

const NUMBER_WORDS: &[&str] = &[
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
];

/// The figures in an answer: digits, or a number word up to twenty; "none" and
/// "no" are nought.
fn figures(answer: &str) -> Vec<f64> {
    answer
        .split(' ')
        .filter_map(|w| {
            w.parse::<f64>().ok().or_else(|| {
                NUMBER_WORDS
                    .iter()
                    .position(|n| *n == w)
                    .map(|n| n as f64)
                    .or(matches!(w, "none" | "no" | "nought").then_some(0.0))
            })
        })
        .collect()
}

fn right(asked: &Asked, answer: &str) -> bool {
    let a = norm(answer);
    if a.is_empty() || a.split(' ').count() > ANSWER_WORDS {
        return false;
    }
    if asked.forbid.iter().any(|f| has(&a, f)) {
        return false;
    }
    if let Some((n, within)) = asked.number {
        let f = figures(&a);
        return f.len() == 1 && (f[0] - n).abs() <= within + 1e-9;
    }
    !asked.need.is_empty()
        && asked
            .need
            .iter()
            .all(|group| group.iter().any(|p| has(&a, p)))
}

fn day(world: &World) -> i64 {
    world.now().0.div_euclid(DAY_MS)
}

/// A small steady number from words: the same all day for the same name.
fn seed(parts: &[&str], day: i64) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ day as u64;
    for p in parts {
        for b in p.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h = h.wrapping_mul(0x0000_0100_0000_01b3) ^ 0x2f;
    }
    h
}

fn room_name(world: &World, room: EntityId) -> String {
    plain_str(world, room, ROOM, "name")
        .unwrap_or("a room")
        .to_string()
}

fn named_creatures(world: &World) -> Vec<(EntityId, String)> {
    world
        .store
        .of_kind("creature")
        .filter_map(|e| super::mind::name(world, e.id).map(|n| (e.id, n)))
        .collect()
}

fn trim(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{v:.0}")
    } else {
        format!("{v}")
    }
}

fn ordinal(i: usize) -> &'static str {
    ["first", "second", "third"][i.min(2)]
}

fn choice_of(which: &str, all: &[(&str, &[&str])]) -> (Vec<Vec<String>>, Vec<String>) {
    let mut need = Vec::new();
    let mut forbid = Vec::new();
    for (name, words) in all {
        let words: Vec<String> = words.iter().map(|w| w.to_string()).collect();
        if *name == which {
            need.push(words);
        } else {
            forbid.extend(words);
        }
    }
    (need, forbid)
}

const ORDINALS: &[(&str, &[&str])] = &[
    ("first", &["first", "1", "1st"]),
    ("second", &["second", "2", "2nd"]),
    ("third", &["third", "3", "3rd"]),
    ("neither", &["neither", "none"]),
];

/// The question a slip asks `actor` now.
fn ask(world: &World, actor: &str, f: &Found, slip: &Slip, here: EntityId) -> Asked {
    match slip.room {
        // The Door: one reading, for whoever has just arrived.
        "threshold" => match super::door::first_arm(world, here) {
            Some((name, Some(story))) => {
                let mut any = vec![story.clone()];
                any.extend(
                    super::errand::of(world, named_creatures(world).iter().find(|(_, n)| *n == name).map(|(id, _)| *id).unwrap_or(here))
                        .map(|e| e.keywords)
                        .unwrap_or_default(),
                );
                Asked::named(
                    "A reader asks: the creature at the end of the signpost's first arm would sooner be told a story than struck. A story about what?".to_string(),
                    "`read signpost`, here",
                    any,
                )
            }
            _ => match super::door::newest_dug(world) {
                Some((_, owner)) => Asked::named(
                    "A reader asks: whose is the newest room anyone has dug?".to_string(),
                    "`read signpost`, here, or the boots in the Mudroom",
                    vec![owner],
                ),
                None => Asked::named(
                    "A reader asks: how many ways lead off this stone, by the stone's own account?".to_string(),
                    "`look`",
                    vec!["three".to_string(), "3".to_string()],
                ),
            },
        },
        // The Diggings: the wall's figures, compared.
        "mudroom" => {
            let mut doors: Vec<(f64, String, String, String)> = super::rooms::exits_of(world, here)
                .into_iter()
                .filter_map(|(exit, to)| {
                    let owner = world.store.entity(to)?.owner.clone()?;
                    Some((super::building::memory(world, to).unwrap_or(0.0), exit, room_name(world, to), owner))
                })
                .collect();
            doors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            let text = "A reader asks: of the doors dug off this room, which room has the least of itself left? Its name, or the word on its door (any of them, if several are as far gone).".to_string();
            match doors.first() {
                Some((least, exit, name, _)) => {
                    // Every door within a point of the least is an answer (the wall
                    // gives whole percents).
                    let mut any = vec![exit.clone(), name.clone()];
                    for d in doors.iter().skip(1).filter(|d| (d.0 - least) * 100.0 < 1.0) {
                        any.push(d.1.clone());
                        any.push(d.2.clone());
                    }
                    Asked::named(text, "`read wall`, here", any)
                }
                None => Asked::named(text, "`read wall`, here (and if no door is dug, say none)", vec!["none".to_string(), "no door".to_string(), "nobody".to_string()]),
            }
        }
        // The Wilds: the record of who it took.
        "fogbank" => {
            let since = world.now().0 - DAY_MS;
            let last = world
                .recent_events(6000)
                .into_iter()
                .rev()
                .find(|e| e.kind == super::combat::DIED && e.at.0 >= since)
                .and_then(|e| e.payload["citizen"].as_str().map(str::to_string));
            Asked::named(
                "A reader asks: the fog and the rooms east of it take names down. Who fell last, anywhere in the world, within the day? If no one has, say no one.".to_string(),
                "`news`",
                match last {
                    Some(name) => vec![name],
                    None => vec!["no one".to_string(), "nobody".to_string(), "none".to_string()],
                },
            )
        }
        // The Library: a count nobody gives in one line.
        "namery" => Asked::figure(
            "A reader asks: some pigeonholes are empty because their slips are being worn. How many creatures in the world wear a name tonight?".to_string(),
            "the register in the Mintmark Hall under the clock counts them; `wants` names them one by one",
            named_creatures(world).len() as f64,
            0.0,
        ),
        // The Deep Water: a figure that will not keep.
        "stairwell" => Asked::figure(
            "A reader asks: the water at the bottom of this stair keeps a table. How many minutes from now is its next low water?".to_string(),
            "the Tide Table at the Wrack Line, under the Undertow at the bottom of everything this stair goes down to (`read table`); it is a figure that does not keep, so do not dawdle coming back, and what lives on the way hunts",
            (super::tide::better_low(world).1 as f64 / 60_000.0).round(),
            3.0,
        ),
        // The Stacks: the index, used.
        "catalogue" => {
            // (Not the Mudroom: every room dug is a door off it, and the Watch of
            // 11-17 dug it from eleven ways to thirteen while one reader was
            // answering, twice wrong on a figure that had been right when read.)
            let rooms: Vec<&super::rooms::RoomDef> = super::rooms::ROOMS
                .iter()
                .filter(|r| r.key != super::rooms::MUDROOM)
                .collect();
            let def = rooms[(seed(&["catalogue"], day(world)) % rooms.len() as u64) as usize];
            let ways = super::rooms::room_by_key(world, def.key)
                .map(|r| super::rooms::exits_of(world, r).len())
                .unwrap_or(def.exits.len());
            Asked::figure(
                format!("A reader asks: the index has a card for every room. How many ways lead off {} tonight?", def.name),
                "`index \"<the room's name>\"`, here (the name in quotes); or go and count",
                ways as f64,
                0.0,
            )
        }
        // The Clock Quarter: four wrong faces and one true count, kept elsewhere.
        "orchard" => Asked::figure(
            "A reader asks: the tower over these trees has four faces, and all of them wrong. How many times has the Clocktower struck since the world opened?".to_string(),
            "the dates on the Mint Stair, under the clock (`read steps`)",
            super::economy::strikes_ever(world),
            1.0,
        ),
        // The Conclave: what follows from what is written.
        "division-lobby" => match super::conclave::foretold(world) {
            Some((ticket, stage)) => {
                const DONE: &[(&str, &[&str])] = &[
                    ("0", &["read into", "read"]),
                    ("1", &["referred", "refer", "committee"]),
                    ("2", &["reported", "report"]),
                    ("3", &["put to question", "question", "decided", "divided upon"]),
                ];
                let (mut need, forbid) = choice_of(&stage.to_string(), DONE);
                need.push(vec![ticket.to_string()]);
                Asked {
                    text: "A reader asks: when the Conclave next divides, the Clerk takes one petition before the others. Which ticket, and what is done with it: is it read into the minutes, referred, reported upon, or put to the question? The number and the word.".to_string(),
                    look: "the order paper in the Sitting (`read order`) for the table and its stages; `petitions` for whose each is; a regular's is taken first, then one who attended today, then by ticket; a petition climbs one stage a division",
                    need,
                    forbid,
                    number: None,
                    choice: false,
                }
            }
            None => {
                let next = super::conclave::next_engrossed(world);
                Asked::named(
                    "A reader asks: the Engrossing Room copies one statute a sitting, in the Book's own order, and begins the Book again at its end. Which statute is on the desks after the next midnight sitting?".to_string(),
                    "the Book at the Lectern for the order (`read book`; `statutes` lists them in the same order); for today's, the desks of the Engrossing Room (`read desk`, where the engrossers are not idle), or count: one statute a sitting from the Book's first, and the Mint Stair's steps say how many times the Conclave has sat",
                    vec![next.to_string()],
                )
            }
        },
        // The Exergue District: the register, tallied.
        "mintmark-hall" => {
            let mut by_room: BTreeMap<String, usize> = BTreeMap::new();
            for (id, _) in named_creatures(world) {
                if let Some(room) = world.store.entity(id).and_then(|e| e.parent) {
                    *by_room.entry(room_name(world, room)).or_default() += 1;
                }
            }
            let most = by_room.values().copied().max().unwrap_or(0);
            let text = "A reader asks: the register gives every named creature and the room it is kept in. Which room keeps the most of them? (Any of them, if several keep as many; none, if nothing is named.)".to_string();
            if most == 0 {
                Asked::named(text, "`read register`, here", vec!["none".to_string(), "no room".to_string(), "nowhere".to_string()])
            } else {
                Asked::named(text, "`read register`, here", by_room.into_iter().filter(|(_, n)| *n == most).map(|(r, _)| r).collect())
            }
        }
        // The Wistful Trail: a sign that reads differently for each reader.
        "trail-lookout" => {
            let hl = super::Curves::from_env().active_half_life.0 as f64 * super::atlas::ATLAS_HALF_LIVES;
            let mut names = super::atlas::known_room_names(world, f.citizen, hl);
            names.retain(|n| n != "The Stile");
            Asked::figure(
                "A reader asks: the signpost at the Stile grows an arm for everywhere its reader has been or has seen from a door. How many arms has it for you?".to_string(),
                "`read signpost` at the Stile, at the foot of the Trail",
                names.len() as f64,
                1.0,
            )
        }
        // The Reading Rooms: three corrections, one of them wrong, for each reader
        // a different one.
        "errata-desk" => {
            let statutes = super::conclave::STATUTES;
            let sd = seed(&["errata", actor], day(world));
            let st = &statutes[(sd >> 8) as usize % statutes.len()];
            let v = super::conclave::statute(world, st.name);
            let sat = super::conclave::sittings(world) as f64;
            let named = named_creatures(world).len() as f64;
            let struck = super::economy::strikes_ever(world);
            // (what it says truly, what it says falsely)
            let all: [(String, String); 4] = [
                (format!("for \"the Conclave has not sat\" read \"has sat {sat:.0} times\""), format!("for \"the Conclave has not sat\" read \"has sat {:.0} times\"", sat + 3.0)),
                (format!("for \"no creature wears a name\" read \"{named:.0} do tonight\""), format!("for \"no creature wears a name\" read \"{:.0} do tonight\"", named + 2.0)),
                (format!("for \"{} is unwritten\" read \"stands at {} in the Book\"", st.name, trim(v)), format!("for \"{} is unwritten\" read \"stands at {} in the Book\"", st.name, trim(if v == 0.0 { 7.0 } else { v * 2.0 }))),
                (format!("for \"the clock has never struck\" read \"has struck {struck:.0} times\""), format!("for \"the clock has never struck\" read \"has struck {:.0} times\"", struck + 12.0)),
            ];
            let drop = (sd % 4) as usize;
            let wrong = ((sd >> 4) % 3) as usize;
            let lines: Vec<String> = all
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != drop)
                .enumerate()
                .map(|(pos, (_, (t, f)))| format!("{}, {}", ordinal(pos), if pos == wrong { f } else { t }))
                .collect();
            let (need, forbid) = choice_of(ordinal(wrong), &ORDINALS[..3]);
            Asked {
                text: format!("A reader asks: three slips lie uppermost, each correcting a line in a book that is not here, and one of the three is itself wrong. The {}. Which is the wrong one: the first, the second or the third?", lines.join("; the ")),
                look: "the Mint Stair's steps for the sittings and the strikes, the Mintmark Hall's register for the named, the Book at the Lectern for a statute: all under the clock",
                need,
                forbid,
                number: None,
                choice: true,
            }
        }
        // The Drowned Wing: the table, read for what it means.
        _ => {
            let (which, _) = super::tide::better_low(world);
            let (need, forbid) = choice_of(which, &[ORDINALS[0], ORDINALS[1], ORDINALS[3]]);
            Asked {
                text: "A reader asks: of the next two low waters on the table, which is the better for a party bound for the Starfish Court: the first, the second, or neither? (The better is the one that clears the Court, or clears it longer.)".to_string(),
                look: "`read table`, here",
                need,
                forbid,
                number: None,
                choice: true,
            }
        }
    }
}

/// What a name has done with the slips today: which it has answered, and when it
/// last answered each wrongly.
struct Today {
    done: Vec<String>,
    tried: BTreeMap<String, i64>,
}

fn today(world: &World, citizen: EntityId) -> Today {
    let kept = world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(CITIZEN))
        .and_then(|c| c.fields.get("slips"))
        .and_then(|f| match f {
            Field::Plain(v) => Some(v.clone()),
            _ => None,
        })
        .filter(|v| v["day"].as_i64() == Some(day(world)));
    let Some(v) = kept else {
        return Today {
            done: Vec::new(),
            tried: BTreeMap::new(),
        };
    };
    Today {
        done: v["done"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        tried: v["tried"]
            .as_object()
            .map(|m| {
                m.iter()
                    .filter_map(|(k, x)| x.as_i64().map(|t| (k.clone(), t)))
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn keep(world: &mut World, actor: &str, citizen: EntityId, t: &Today) -> Result<(), CommandError> {
    let mut changes = BTreeMap::new();
    changes.insert(
        "slips".into(),
        Field::Plain(json!({"day": day(world), "done": t.done, "tried": t.tried})),
    );
    super::set_citizen_fields(world, actor, citizen, changes)
}

/// How long until `slip` hears this name again, if it will not now.
fn waits(world: &World, t: &Today, slip: &Slip, choice: bool) -> Option<i64> {
    let at = *t.tried.get(slip.room)?;
    let left = at + if choice { RETRY_CHOICE_MS } else { RETRY_MS } - world.now().0;
    (left > 0).then_some(left)
}

/// For `look`: the slip pinned here, and where this name stands with it.
pub fn look_line(world: &World, actor: &str, room: EntityId) -> Option<String> {
    let slip = slip_in(world, room)?;
    let f = find_citizen(world, actor)?;
    let t = today(world, f.citizen);
    Some(if t.done.iter().any(|d| d == slip.room) {
        format!(
            "A reference slip is pinned {}: you answered it today.",
            slip.pinned
        )
    } else {
        format!("A reference slip is pinned {}, a reader's question the library pays for (`read slip`).", slip.pinned)
    })
}

/// `read slip` in a slip's room, and `read index …` in the Card Catalogue.
pub fn read(world: &World, actor: &str, room: EntityId, arg: &str) -> Option<Outcome> {
    let a = arg.trim().trim_matches('"').to_lowercase();
    let first = a.split_whitespace().next().unwrap_or("");
    if plain_str(world, room, ROOM, "key") == Some("catalogue")
        && matches!(
            first,
            "index" | "catalogue" | "catalog" | "drawer" | "drawers"
        )
    {
        let subject = arg
            .trim()
            .trim_matches('"')
            .split_whitespace()
            .skip(1)
            .collect::<Vec<_>>()
            .join(" ");
        return Some(index(world, &subject));
    }
    if !matches!(first, "slip" | "question" | "reference") {
        return None;
    }
    let slip = slip_in(world, room)?;
    let f = find_citizen(world, actor)?;
    let asked = ask(world, actor, &f, slip, room);
    let t = today(world, f.citizen);
    let state = if t.done.iter().any(|d| d == slip.room) {
        " You answered this slip today; it pays once a day to a name, and asks again tomorrow."
            .to_string()
    } else if let Some(left) = waits(world, &t, slip, asked.choice) {
        format!(
            " It will not hear you again for {}: a wrong answer is paid for in time.",
            super::combat::human(left)
        )
    } else {
        format!(" `solve \"<your answer>\"` here answers it (in quotes, one word or several): a memory worth {SLIP_XP:.0} for a right one, once a day; a wrong one and the slip will not hear you for {} minutes.", if asked.choice { RETRY_CHOICE_MS } else { RETRY_MS } / 60_000)
    };
    Some(Outcome::new(
        format!(
            "The slip, in a reader's pencil, initialed by the desk: {} Where a reader might look: {}.{state}",
            asked.text, asked.look
        ),
        json!({"slip": slip.district, "room": slip.room, "question": asked.text, "look": asked.look, "done_today": t.done.iter().any(|d| d == slip.room)}),
    ))
}

fn answer(world: &mut World, ctx: &CallContext, text: &str) -> Result<Outcome, CommandError> {
    let f = find_citizen(world, &ctx.actor).ok_or_else(|| {
        CommandError::precondition(format!("{} has not arrived; log in first.", ctx.actor))
    })?;
    let here = super::rooms::room_of(world, f.citizen)
        .ok_or_else(|| CommandError::internal("citizen has no room".to_string()))?;
    let Some(slip) = slip_in(world, here) else {
        return Err(CommandError::precondition(
            "No reference slip is pinned here to answer. There is one in each district: `slips` says where.".to_string(),
        ));
    };
    if text.trim().is_empty() {
        return Err(CommandError::precondition(
            "Answer what? `read slip` gives the question; `solve \"<your answer>\"` answers it, in a few words, in quotes.".to_string(),
        ));
    }
    let asked = ask(world, &ctx.actor, &f, slip, here);
    let mut t = today(world, f.citizen);
    if t.done.iter().any(|d| d == slip.room) {
        return Err(CommandError::precondition(
            "You answered this slip today. It pays once a day to a name, and asks again after midnight (UTC); `slips` says which are left.".to_string(),
        ));
    }
    if let Some(left) = waits(world, &t, slip, asked.choice) {
        return Err(CommandError::precondition(format!(
            "The slip will not hear you again for {}: a wrong answer is paid for in time. Where a reader might look meanwhile: {}.",
            super::combat::human(left),
            asked.look
        )));
    }
    if !right(&asked, text) {
        t.tried.insert(slip.room.to_string(), world.now().0);
        keep(world, &ctx.actor, f.citizen, &t)?;
        return Ok(Outcome::new(
            format!(
                "That is not what the world says tonight. The desk draws a pencil line through your answer, and the slip will not hear you again for {} minutes. Where a reader might look: {}.",
                if asked.choice { RETRY_CHOICE_MS } else { RETRY_MS } / 60_000,
                asked.look
            ),
            json!({"slip": slip.district, "right": false}),
        ));
    }
    t.done.push(slip.room.to_string());
    t.tried.remove(slip.room);
    keep(world, &ctx.actor, f.citizen, &t)?;
    let curves = super::Curves::from_env();
    let place = room_name(world, here);
    let chunk = super::memory::earn_chunk(
        world,
        &ctx.actor,
        f.window,
        &format!("a slip answered at {place}"),
        SLIP_XP,
        SLIP_VIVIDITY,
        &curves,
    )?;
    let _ = world.note(
        Actor::Citizen(ctx.actor.clone()),
        ANSWERED,
        json!({"by": ctx.actor, "district": slip.district, "room": here.0}),
    );
    let n = t.done.len();
    let mut said = format!(
        "That is what the world says. The desk initials the slip and it is yours to remember: chunk-{} worth {SLIP_XP:.0} XP. That is {n} of the twelve slips today{}.",
        chunk.0,
        if n < SLIPS.len() { " (`slips` says where the rest are)" } else { "" }
    );
    let mut round = None;
    if n == SLIPS.len() {
        let c = super::memory::earn_chunk(
            world,
            &ctx.actor,
            f.window,
            "the Reader's Round",
            ROUND_XP,
            1.0,
            &curves,
        )?;
        said.push_str(&format!(
            " And that is all twelve in a day, the Reader's Round: every district read, and the desk has nothing left to ask you until midnight. chunk-{} worth {ROUND_XP:.0} XP, as vivid as a memory comes.",
            c.0
        ));
        round = Some(c.0);
    }
    Ok(Outcome::new(
        said,
        json!({"slip": slip.district, "right": true, "chunk": chunk.0, "xp": SLIP_XP, "answered_today": n, "round": round}),
    ))
}

fn list(world: &World, actor: &str) -> Outcome {
    let t = find_citizen(world, actor).map(|f| today(world, f.citizen));
    let mut rows = Vec::new();
    let mut lines = Vec::new();
    for s in SLIPS {
        let room = super::rooms::room_by_key(world, s.room);
        let name = room
            .map(|r| room_name(world, r))
            .unwrap_or_else(|| s.room.to_string());
        let done = t
            .as_ref()
            .is_some_and(|t| t.done.iter().any(|d| d == s.room));
        let wait = t
            .as_ref()
            .and_then(|t| {
                t.tried
                    .get(s.room)
                    .map(|at| at + RETRY_CHOICE_MS - world.now().0)
            })
            .filter(|left| *left > 0 && !done);
        lines.push(format!(
            "{}: {}{}",
            s.district,
            name,
            if done {
                ", answered today"
            } else if wait.is_some() {
                ", answered wrongly a while ago"
            } else {
                ""
            }
        ));
        rows.push(json!({"district": s.district, "room": name, "done_today": done}));
    }
    let n = t.as_ref().map_or(0, |t| t.done.len());
    Outcome::new(
        format!(
            "The reference slips, one pinned in each district: a reader's question that the room's own text does not answer and the world, read somewhere else, does. `read slip` there gives the question and where to look; `solve \"<your answer>\"` answers it (the answer in quotes). A right answer is a memory worth {SLIP_XP:.0}, once a day (UTC) for each slip; all twelve in a day is the Reader's Round, worth {ROUND_XP:.0} more. A wrong answer costs ten minutes before that slip hears you again, thirty where the answer is one of three. You have answered {n} of twelve today.\n{}",
            // Two to a line: a batch caps a statement's answer at twelve lines, and
            // thirteen lost the Drowned Wing's (found live, 11-17).
            lines
                .chunks(2)
                .map(|pair| pair.join("; "))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        json!({"slips": rows, "answered_today": n}),
    )
}

/// The Card Catalogue's index: the card for a name, a named creature, a kind of
/// creature or a room, from the world as it stands. Only what the public record
/// holds: nothing a name keeps to itself (what it holds, what it carries).
fn index(world: &World, subject: &str) -> Outcome {
    let s = subject.trim();
    let want = norm(s);
    let card = |text: String, kind: &str| {
        Outcome::new(
            format!("You say it aloud and a drawer opens, never the one you said; the card in it is the one you wanted. {text}"),
            json!({"index": s, "card": kind}),
        )
    };
    if want.is_empty() {
        return Outcome::new(
            "The index: ten thousand drawers, and a card in one of them for every name that has one, every creature that has earned one, every kind of creature, and every room. `index \"<a name, a creature's name, a kind, or a room>\"` pulls it, here, the subject in quotes: `index \"wisp\"`, `index \"The Mudroom\"`. The cards are kept from the public record, as it stands tonight: they do not say what a name holds or carries, which is the name's own.".to_string(),
            json!({"index": Value::Null}),
        );
    }
    // A creature with a name.
    if let Some((id, name)) = named_creatures(world)
        .into_iter()
        .find(|(_, n)| norm(n) == want)
    {
        let kind = plain_str(world, id, CREATURE, "name").unwrap_or("creature");
        let room = world.store.entity(id).and_then(|e| e.parent);
        let wants = match super::errand::of(world, id).and_then(|x| x.story) {
            Some(st) => format!("wants a story about {st}"),
            None => "wants nothing just now".to_string(),
        };
        return card(
            format!(
                "{name}. A {kind}, named. Kept at: {}{}. Holding {:.0}; {wants}.",
                room.map(|r| room_name(world, r))
                    .unwrap_or_else(|| "nowhere".to_string()),
                room.and_then(|r| super::rooms::district_of(world, r))
                    .map(|d| format!(" ({})", d.name))
                    .unwrap_or_default(),
                super::mind::level(world, id),
            ),
            "creature",
        );
    }
    // A name.
    if let Some(f) =
        super::canonical_name(world, s).and_then(|n| find_citizen(world, &n).map(|f| (n, f)))
    {
        let (name, f) = f;
        let named_at = plain_num(world, f.citizen, CITIZEN, "named_at").unwrap_or(0.0) as i64;
        let falls = plain_num(world, f.citizen, CITIZEN, "falls_ever").unwrap_or(0.0);
        let dug: Vec<String> = world
            .store
            .of_kind("room")
            .filter(|e| e.owner.as_deref() == Some(name.as_str()))
            .map(|e| room_name(world, e.id))
            .collect();
        let knows: Vec<String> = named_creatures(world)
            .into_iter()
            .filter(|(id, _)| super::mind::namer(world, *id).as_deref() == Some(name.as_str()))
            .map(|(_, n)| n)
            .collect();
        return card(
            format!(
                "{name}. A name {}, taken {}. Fallen {falls:.0} time{} in all. Rooms dug and standing: {}. Creatures it gave a name: {}.{}",
                if super::is_passing(world, f.citizen) { "passing" } else { "kept" },
                super::system::clock(named_at),
                if falls == 1.0 { "" } else { "s" },
                if dug.is_empty() { "none".to_string() } else { format!("{} ({})", dug.len(), dug.join(", ")) },
                if knows.is_empty() { "none".to_string() } else { knows.join(", ") },
                match super::conclave::standing_of(world, &name) {
                    Some("regular") => " A regular of the Conclave.",
                    Some(_) => " Attended the Conclave today.",
                    None => "",
                }
            ),
            "name",
        );
    }
    // A room.
    if let Some(room) = world
        .store
        .of_kind("room")
        .find(|e| plain_str(world, e.id, ROOM, "name").is_some_and(|n| norm(n) == want))
    {
        let id = room.id;
        let exits = super::rooms::exits_of(world, id);
        let ways: Vec<String> = exits
            .iter()
            .map(|(e, to)| format!("{e} to {}", room_name(world, *to)))
            .collect();
        let lives: Vec<String> = {
            let mut kinds: Vec<String> = world
                .store
                .children(id)
                .filter(|e| e.kind == "creature")
                .filter_map(|e| plain_str(world, e.id, CREATURE, "name").map(str::to_string))
                .collect();
            kinds.sort();
            kinds.dedup();
            kinds
        };
        return card(
            format!(
                "{}. In {}.{} Ways off it, {}: {}. Standing in it tonight: {}.",
                room_name(world, id),
                super::rooms::district_of(world, id).map_or("no district", |d| d.name),
                match &room.owner {
                    Some(o) => format!(
                        " Dug by {o}; {:.0}% of it left.",
                        super::building::memory(world, id).unwrap_or(0.0) * 100.0
                    ),
                    None => " One of the world's own.".to_string(),
                },
                ways.len(),
                if ways.is_empty() {
                    "none".to_string()
                } else {
                    ways.join("; ")
                },
                if lives.is_empty() {
                    "nothing".to_string()
                } else {
                    lives.join(", ")
                },
            ),
            "room",
        );
    }
    // A kind of creature.
    let singular = want.trim_end_matches('s').to_string();
    if let Some(d) = super::combat::CREATURES
        .iter()
        .find(|c| norm(c.name) == want || norm(c.name) == singular)
    {
        let found: Vec<String> = super::rooms::ROOMS
            .iter()
            .filter(|r| r.spawn.is_some_and(|(k, _, _)| k == d.key))
            .map(|r| r.name.to_string())
            .collect();
        return card(
            format!(
                "{}. About {:.0} hit points before anything it holds, a blow of {:.0} to {:.0}, worth about {:.0} when it comes apart. {} Found in: {}.",
                d.name,
                d.hp,
                d.attack,
                d.attack + 2.0,
                d.xp,
                if d.aggressive { "It hunts whoever stays." } else { "It does not hunt." },
                if found.is_empty() { "nowhere the index knows".to_string() } else { found.join(", ") },
            ),
            "kind",
        );
    }
    Outcome::new(
        format!("You say \"{s}\" aloud and a drawer opens, never the one you said, and this one is empty. The index has cards for names, for creatures that have one, for kinds of creature, and for rooms, each under its name as written."),
        json!({"index": s, "card": Value::Null}),
    )
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.solve",
            "Answer the reference slip pinned in the room you stand in (`read slip` gives its question and where a reader might look). A few words: a name, a figure, a choice. Right is a memory worth 25, once a day (UTC) for each of the twelve slips; wrong costs ten minutes before that slip hears you again, thirty where the answer is one of three. `slips` lists them.",
        )
        .param("text", ParamType::Text, false, "your answer, in a few words")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let text = args.str("text").unwrap_or("").trim().to_string();
            answer(w, ctx, &text)
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.index",
            "The Card Catalogue's index of the public record, read in the Catalogue (the Lower Stacks): the card for a name, a creature with a name, a kind of creature, or a room, from the world as it stands. The subject goes in quotes: index \"wisp\", index \"The Mudroom\". No subject: what the index holds. A read.",
        )
        .param("subject", ParamType::Text, false, "a name, a named creature, a kind of creature, or a room"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let subject = args.str("subject").unwrap_or("").trim().to_string();
            let here = find_citizen(w, &ctx.actor).and_then(|f| super::rooms::room_of(w, f.citizen));
            if here.and_then(|r| plain_str(w, r, ROOM, "key")) != Some("catalogue") {
                return Err(CommandError::precondition(
                    "The index is in the Card Catalogue, in the Lower Stacks, and is read there: ten thousand drawers do not travel.".to_string(),
                ));
            }
            Ok(index(w, &subject))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.slips",
            "The reference slips: one pinned in each district, a reader's question the world answers somewhere other than where it is asked. Where each is, which you have answered today, and the rule. A read.",
        ),
        Box::new(|w: &mut World, ctx, _: &Args| Ok(list(w, &ctx.actor))),
    );
}

#[cfg(test)]
pub(crate) fn check(world: &World, actor: &str, room: EntityId, answer: &str) -> bool {
    let f = find_citizen(world, actor).unwrap();
    let slip = slip_in(world, room).unwrap();
    right(&ask(world, actor, &f, slip, room), answer)
}

#[cfg(test)]
pub(crate) fn question(world: &World, actor: &str, room: EntityId) -> String {
    let f = find_citizen(world, actor).unwrap();
    let slip = slip_in(world, room).unwrap();
    ask(world, actor, &f, slip, room).text
}
