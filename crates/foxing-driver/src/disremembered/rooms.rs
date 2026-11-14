//! The map: fourteen rooms off the Threshold, twenty below the Archive, eleven up
//! inside the Clocktower's works, eight under it, nine up the Wistful Trail behind
//! the Orchard, exits between them, and what spawns where.
//! No room, creature or exit may be named with a CLEAR reserved word or a glossary
//! alias; a test under `disremembered::tests` holds the map to that.
//! Rooms are entities with `disremembered.room`, `disremembered.exits` and, where something lives,
//! `disremembered.spawn`. Travel moves the citizen entity from one room to another.

use super::{internal, me, mind, plain_str, EXITS, ROOM, SPAWN};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A room as the slice defines it. The log is the source of truth once built; this table
/// only seeds it and fills in what an older log lacks.
pub struct RoomDef {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// `(exit name, room key)`.
    pub exits: &'static [(&'static str, &'static str)],
    /// `(creature key, how many at once, seconds between respawns)`.
    pub spawn: Option<(&'static str, u32, i64)>,
    /// Keys this room had in older logs; a room found under one is renamed in place.
    pub former: &'static [&'static str],
    /// How a renderer tints the room.
    pub tint: &'static str,
}

pub const ARRIVAL: &str = "threshold";
/// C11: the second landing, the back gate on the far side of the Library.
pub const POSTERN: &str = "postern";

/// H9: the world's rooms by district. A district is a name for a part of the map, a
/// room that is its door, and a line said once a session to whoever comes through it.
pub struct District {
    pub name: &'static str,
    /// The key of the room that is its door.
    pub entry: &'static str,
    pub blurb: &'static str,
}

pub const DISTRICTS: &[District] = &[
    District { name: "the Door", entry: ARRIVAL, blurb: "" },
    District { name: "the Diggings", entry: MUDROOM, blurb: "every room past this one was dug by a visitor, and fades unless somebody comes" },
    District { name: "the Wilds", entry: "fogbank", blurb: "the fighter's road: the fog, the Midden, the Shallows and the bone rooms east of them; the further out, the harder it hits" },
    District { name: "the Library", entry: "lantern-row", blurb: "things to read and creatures to tell: the Row, the Archive, the Namery, and a lane to the Orchard that nothing hunts on" },
    District { name: "the Deep Water", entry: "stairwell", blurb: "the stair under the Row, the Cistern and the Undertow, which comes up in the Shallows" },
    District { name: "the Stacks", entry: "lower-stacks", blurb: "the library's lower floors; a level of 100 is asked at the stair, and a lantern keeps the map" },
    District { name: "the Clock Quarter", entry: "orchard", blurb: "the Orchard and the Stopped Clocktower over it; the Conclave sits inside the clock, the Exergue District lies under it, and the Wistful Trail starts north of the trees" },
    District { name: "the Conclave", entry: "escapement-stair", blurb: "the house that sets the world's rates: read its Book, lay a petition, watch a division" },
    District { name: "the Exergue District", entry: "exergue-stair", blurb: "the mint under the clock: the coinage in circulation, a bar, and a guarded Purse" },
    District { name: "the Wistful Trail", entry: "trail-stile", blurb: "a walk north of the Orchard: a meadow, a well, a bog and a lookout" },
];

fn district_index(key: &str) -> Option<usize> {
    Some(match key {
        ARRIVAL => 0,
        MUDROOM => 1,
        "fogbank" | "shallows" | "midden" | "ossuary" | "gallery" | "oubliette" => 2,
        "lantern-row" | "archive" | "namery" | "dead-letters" | "lamplighters-lane" | POSTERN => 3,
        "stairwell" | "cistern" | "undertow" => 4,
        "lower-stacks" | "catalogue" | "dark-reading-room" | "periodicals" | "dumbwaiter"
        | "overdue-desk" | "carrels" | "index-stair" | "bindery" | "tube-room" | "lost-things"
        | "acid-room" | "chart-room" | "censor-office" | "restoration" | "pulp-sump"
        | "the-leak" | "reserve" | "strongroom" | "deep-reserve" => 5,
        "orchard" | "clocktower" | "orchard-wall" => 6,
        "escapement-stair" | "petition-gallery" | "robing-room" | "tally-room" | "brass-rail"
        | "book-lectern" | "roll-hall" | "session-benches" | "division-lobby" | "minute-room"
        | "engrossing-room" => 7,
        "exergue-stair" | "counting-bar" | "die-room" | "striking-floor" | "assay-office"
        | "mintmark-hall" | "purse-room" | "exergue" => 8,
        k if k.starts_with("trail-") => 9,
        _ => return None,
    })
}

/// The district a room is in: a seeded room by its key, a dug room in the Diggings.
pub fn district_of(world: &World, room: EntityId) -> Option<&'static District> {
    if world.store.entity(room).is_some_and(|e| e.owner.is_some()) {
        return Some(&DISTRICTS[1]);
    }
    district_index(plain_str(world, room, ROOM, "key")?).map(|i| &DISTRICTS[i])
}

/// How many of the world's own rooms a district has.
pub fn district_size(d: &District) -> usize {
    ROOMS
        .iter()
        .filter(|r| district_index(r.key).is_some_and(|i| std::ptr::eq(&DISTRICTS[i], d)))
        .count()
}

/// A district by its name as typed: "the stacks", "wilds", "trail".
pub fn district_named(text: &str) -> Option<&'static District> {
    let want = text.trim().trim_matches('"').to_lowercase();
    let want = want.trim_start_matches("the ").trim();
    DISTRICTS.iter().find(|d| {
        let n = d.name.trim_start_matches("the ").to_lowercase();
        n == want
            || n.split(' ').next_back() == Some(want)
            || n.trim_end_matches(" district") == want
    })
}

/// H11: the rooms that have a signpost of their own.
pub const HUBS: &[&str] = &["fogbank", "lantern-row", "orchard", "clocktower", POSTERN];
/// The room off the Threshold where the digging starts: every dug door that once hung
/// off the front door hangs here, and the door cap does not apply.
pub const MUDROOM: &str = "mudroom";
/// The compass words, in the order the exits line says them.
pub const COMPASS: &[&str] = &[
    "north", "east", "south", "west", "up", "down", "in", "out", "back",
];

/// Whether a room is the Mudroom.
pub fn is_mudroom(world: &World, room: EntityId) -> bool {
    plain_str(world, room, ROOM, "key") == Some(MUDROOM)
}

pub const ROOMS: &[RoomDef] = &[
    RoomDef {
        key: ARRIVAL,
        name: "The Threshold",
        description: "A threshold of pale stone under a moon that is always up, and three ways off it. East, into the fog: things to fight, and a fight won is the first memory you will hold. North, under the lanterns: things to read, and creatures that would sooner be told a story. West, into the mud: where anyone with a memory to spend digs a room of their own. A signpost by the door says what is at the end of each just now (`read signpost`). Everything you earn here will fade; what you do with it is yours. Nothing hunts here. If you do one thing first, go east.",
        exits: &[("east", "fogbank"), ("north", "lantern-row"), ("west", MUDROOM)],
        spawn: None,
        former: &["gate"],
        tint: "#d9d4c7",
    },
    RoomDef {
        key: MUDROOM,
        name: "The Mudroom",
        description: "A low room off the Threshold where the digging starts: boots by the door that remember who dug last (`read boots`), a rack of spades worn to the handle (`take spade`; carried, it marks every dug door you pass), and mud on everything, tracked in from rooms that did not exist yesterday. Every door here was dug by someone, with a memory they could have kept (`dig <a word for the door> <chunk>`, and a name for the room after). Nothing hunts here.",
        exits: &[("east", ARRIVAL)],
        spawn: None,
        former: &[],
        tint: "#8a6e4b",
    },
    RoomDef {
        key: "fogbank",
        name: "The Fogbank",
        description: "Fog rolls off the stone and swallows the path a few steps ahead. Things move in it, slowly, and mostly away from you.",
        exits: &[("west", ARRIVAL), ("north", "shallows"), ("south", "midden")],
        spawn: Some(("wisp", 2, 60)),
        former: &[],
        tint: "#8a97a8",
    },
    RoomDef {
        key: "shallows",
        name: "The Shallows",
        description: "Black water over pale sand, ankle-deep and perfectly still, until something stands up out of it.",
        exits: &[
            ("south", "fogbank"),
            ("east", "ossuary"),
            ("north", "orchard"),
            ("dive", "undertow"),
        ],
        spawn: Some(("hollow", 2, 120)),
        former: &[],
        tint: "#3f5a5a",
    },
    RoomDef {
        key: "ossuary",
        name: "The Ossuary",
        description: "Niches cut into the rock, each one empty, each one swept clean. Something keeps them that way.",
        exits: &[("west", "shallows"), ("east", "gallery")],
        spawn: Some(("revenant", 1, 300)),
        former: &[],
        tint: "#6b5a4a",
    },
    // North of the Threshold: the quiet side, where things are kept rather than hunted.
    RoomDef {
        key: "lantern-row",
        name: "Lantern Row",
        description: "A row of lanterns on iron hooks, each one lighting as you pass and going dark behind you. They remember being lit. They do not remember by whom. Nothing hunts here.",
        exits: &[("south", ARRIVAL), ("north", "archive"), ("down", "stairwell"), ("lane", "lamplighters-lane")],
        spawn: Some(("lanternfly", 2, 90)),
        former: &[],
        tint: "#c9a86a",
    },
    // H10: a way from the door's "read and tell" side to the Clock Quarter that is not
    // a fight. Its doors are named for where they go (a compass door here would want
    // the Shallows' place on the map).
    RoomDef {
        key: "lamplighters-lane",
        name: "The Lamplighter's Lane",
        description: "A narrow lane between a wall of books and a wall of trees, with a ladder left leaning at every tenth post by someone who meant to come back for it. The lamps here are lit by hand, and somebody still does it. Behind you the Row; ahead, fruit on the air and a clock that is not striking. Nothing hunts here.",
        exits: &[("row", "lantern-row"), ("orchard", "orchard"), ("gate", POSTERN)],
        spawn: None,
        former: &[],
        tint: "#b89a5e",
    },
    // C11: the Postern, the world's second door. A name may wake here as it wakes at
    // the Threshold (`landing`), and keeps the door it woke at as its home (`home`).
    RoomDef {
        key: POSTERN,
        name: "The Postern",
        description: "The back gate: a low door in a garden wall, a lamp on an iron bracket, the Reading Bench under it worn smooth, a boot-scraper nobody has used. It is night here too. Through the door the Lamplighter's Lane runs to the lanterns one way and the Orchard the other, and nothing hunts on it; along the wall the moths come down from the trees, and they are the mildest thing in the world that will give you something to remember. This is a door of the world, as the Threshold is: a name that wakes here comes back here. Nothing hunts here.",
        exits: &[("lane", "lamplighters-lane"), ("wall", "orchard-wall")],
        spawn: None,
        former: &[],
        tint: "#6f7a5c",
    },
    RoomDef {
        key: "orchard-wall",
        name: "The Orchard Wall",
        description: "The outside of the Orchard's wall, where the branches hang over and drop fruit that never ripened into the long grass. Moths work the windfalls. Over the wall is the Orchard; along it, the gate.",
        exits: &[("gate", POSTERN), ("over", "orchard")],
        spawn: Some(("mothwing", 2, 40)),
        former: &[],
        tint: "#71895a",
    },
    RoomDef {
        key: "archive",
        name: "The Unfiled Archive",
        description: "Shelves to the ceiling and ledgers on every one, but the ink lifts off the page the moment you read it and drifts for the rafters. Some of it has learned to flutter.",
        exits: &[("south", "lantern-row"), ("down", "lower-stacks"), ("names", "namery")],
        spawn: Some(("inkling", 3, 45)),
        former: &[],
        tint: "#4a4e6b",
    },
    RoomDef {
        key: "stairwell",
        name: "The Cistern Stair",
        description: "A spiral stair cut into the rock, going down further than the hill is tall. Every landing has a door, and every door is bricked up from the other side.",
        exits: &[("up", "lantern-row"), ("down", "cistern")],
        spawn: Some(("dust devil", 1, 300)),
        former: &[],
        tint: "#5a5a5a",
    },
    RoomDef {
        key: "cistern",
        name: "The Cistern",
        description: "A drowned hall, water to your waist, the ceiling lost in the dark. Every sound comes back twice: once as you made it, and once as somebody else did.",
        exits: &[("up", "stairwell"), ("east", "undertow")],
        spawn: Some(("drowned", 2, 150)),
        former: &[],
        tint: "#2f4858",
    },
    RoomDef {
        key: "undertow",
        name: "The Undertow",
        description: "A current under the Shallows, pulling at your knees toward somewhere it will not name. Light comes down green from the surface. Things that stood up in the water above lie down here.",
        exits: &[("west", "cistern"), ("surface", "shallows")],
        spawn: Some(("hollow", 1, 180)),
        former: &[],
        tint: "#2e6b5e",
    },
    // North of the Shallows: a place that almost ripens.
    RoomDef {
        key: "orchard",
        name: "The Orchard of Unripe Hours",
        description: "Rows of low trees heavy with fruit that is always about to ripen and never does. The air is sweet with it. Moths the size of your hand work the branches, and are not interested in you, mostly.",
        exits: &[("south", "shallows"), ("up", "clocktower"), ("north", "trail-stile"), ("lane", "lamplighters-lane"), ("wall", "orchard-wall")],
        spawn: Some(("mothwing", 3, 40)),
        former: &[],
        tint: "#7d9a5a",
    },
    RoomDef {
        key: "clocktower",
        name: "The Stopped Clocktower",
        description: "Four clock faces, one to each wall, each showing a different hour, and all of them wrong. The works fill the middle of the room, and something in the works is still wound.",
        exits: &[("down", "orchard"), ("escapement", "escapement-stair"), ("exergue", "exergue-stair")],
        spawn: Some(("hourhand", 1, 240)),
        former: &[],
        tint: "#8a6f3a",
    },
    // South of the Fogbank: where everything forgotten ends up.
    RoomDef {
        key: "midden",
        name: "The Midden",
        description: "A heap of everything anyone here has ever forgotten: names, keys, the second verse, the way home. It shifts when you are not looking at it. Something lives in it and sorts.",
        exits: &[("north", "fogbank")],
        spawn: Some(("ragpicker", 1, 90)),
        former: &[],
        tint: "#6e6250",
    },
    // East of the Ossuary: the deep end.
    RoomDef {
        key: "gallery",
        name: "The Gallery of Faces Turned Away",
        description: "A long hall of portraits, every one hung facing the wall. The frames are dusted. The hooks are new. One of the sitters is not entirely in its frame.",
        exits: &[("west", "ossuary"), ("down", "oubliette")],
        spawn: Some(("sitter", 1, 200)),
        former: &[],
        tint: "#5e3f4f",
    },
    RoomDef {
        key: "oubliette",
        name: "The Oubliette",
        description: "A pit with no stair, only the rope you came down. The walls are scored with tally marks in a hand that stopped counting. The Keeper has forgotten what it keeps, but not that it keeps it.",
        exits: &[("up", "gallery")],
        spawn: Some(("keeper", 1, 600)),
        former: &[],
        tint: "#2b2b33",
    },
    // Under the Archive: the Lower Stacks, twenty rooms where the higher things live.
    // Laid out on a grid, five across and four deep, running north from the foot of the Archive's stair; the Deep Reserve is the far corner.
    RoomDef {
        key: "lower-stacks",
        name: "The Lower Stacks",
        description: "Shelves to a ceiling too far up to see, every spine blank. The ladders roll along the rails on their own, slowly, as if someone has just let go of them. The Archive is a floor up; the rest of the place is down here.",
        exits: &[("up", "archive"), ("east", "catalogue"), ("north", "overdue-desk")],
        spawn: None,
        former: &[],
        tint: "#4a4238",
    },
    RoomDef {
        key: "catalogue",
        name: "The Card Catalogue",
        description: "Ten thousand small drawers, every card filed under a name no one has anymore. Say a word aloud and a drawer opens, never the one you said.",
        exits: &[("west", "lower-stacks"), ("east", "dark-reading-room")],
        spawn: Some(("silverfish", 2, 120)),
        former: &[],
        tint: "#4a4238",
    },
    RoomDef {
        key: "dark-reading-room",
        name: "The Dark Reading Room",
        description: "Long tables, green-shaded lamps all switched off, chairs pushed back as if everyone left at once. The books are open, and being read by nobody.",
        exits: &[("west", "catalogue"), ("east", "periodicals"), ("north", "index-stair")],
        spawn: None,
        former: &[],
        tint: "#4a4238",
    },
    RoomDef {
        key: "periodicals",
        name: "Periodicals",
        description: "Newspapers bound by the year, the years out of order, the headlines about tomorrow. The ink is still wet on the oldest. Something at the far table is going through them with a stamp.",
        exits: &[("west", "dark-reading-room"), ("east", "dumbwaiter"), ("north", "bindery")],
        spawn: Some(("censor", 1, 300)),
        former: &[],
        tint: "#4a4238",
    },
    RoomDef {
        key: "dumbwaiter",
        name: "The Dumbwaiter",
        description: "A shaft with a box in it, rising and falling without being called. Sometimes the box comes up holding the book you were about to ask for.",
        exits: &[("west", "periodicals"), ("north", "tube-room")],
        spawn: None,
        former: &[],
        tint: "#4a4238",
    },
    RoomDef {
        key: "overdue-desk",
        name: "The Overdue Desk",
        description: "A desk with a bell and a ledger of fines in a column that does not end. The bell rings itself at intervals. The fines go up.",
        exits: &[("east", "carrels"), ("south", "lower-stacks"), ("north", "lost-things")],
        spawn: Some(("bookworm", 2, 180)),
        former: &[],
        tint: "#3d3a44",
    },
    RoomDef {
        key: "carrels",
        name: "The Carrels",
        description: "A warren of study cubicles, each with a reading light and a reader's shape worn into the chair. Someone is always in the next one over, and never in yours. One of them is on its feet.",
        exits: &[("west", "overdue-desk"), ("east", "index-stair"), ("north", "acid-room")],
        spawn: Some(("usher", 1, 240)),
        former: &[],
        tint: "#3d3a44",
    },
    RoomDef {
        key: "index-stair",
        name: "The Stair of Indices",
        description: "A stair whose steps are lettered A to Z and then keep going. Nothing is at the top but the top. Everything is at the bottom.",
        exits: &[("west", "carrels"), ("south", "dark-reading-room"), ("north", "chart-room")],
        spawn: None,
        former: &[],
        tint: "#3d3a44",
    },
    RoomDef {
        key: "bindery",
        name: "The Bindery",
        description: "Presses, glue pots, a guillotine for trimming pages. Something here is being bound that was never a book, and it is not done being bound.",
        exits: &[("east", "tube-room"), ("south", "periodicals")],
        spawn: None,
        former: &[],
        tint: "#3d3a44",
    },
    RoomDef {
        key: "tube-room",
        name: "The Pneumatic Room",
        description: "Brass tubes run through the walls to every corner of the place, thumping with canisters. Open one and the message inside is addressed to you, in your hand. Someone in a waistcoat is waiting to show you where it came from.",
        exits: &[("west", "bindery"), ("south", "dumbwaiter"), ("north", "restoration")],
        spawn: Some(("usher", 1, 240)),
        former: &[],
        tint: "#3d3a44",
    },
    RoomDef {
        key: "lost-things",
        name: "Lost and Found",
        description: "Shelves of umbrellas, gloves, spectacles, a single shoe, a name tag with your name on it. Nothing is claimed. Everything is kept.",
        exits: &[("east", "acid-room"), ("south", "overdue-desk"), ("north", "pulp-sump")],
        spawn: Some(("shelf mouse", 1, 240)),
        former: &[],
        tint: "#44403a",
    },
    RoomDef {
        key: "acid-room",
        name: "The Acid Room",
        description: "Where the paper yellows. The air is slightly wrong: pages brown as you watch, and so do you, a little. Something pale is fastened to the underside of a shelf.",
        exits: &[("west", "lost-things"), ("south", "carrels"), ("north", "the-leak")],
        spawn: Some(("lamprey", 1, 200)),
        former: &[],
        tint: "#44403a",
    },
    RoomDef {
        key: "chart-room",
        name: "The Chart Room",
        description: "Flat drawers of maps of places that are here and nowhere else. One shows this room, with you in it, slightly behind where you are. The hand that drew it is redrawing.",
        exits: &[("east", "censor-office"), ("south", "index-stair"), ("north", "reserve")],
        spawn: Some(("censor", 1, 300)),
        former: &[],
        tint: "#44403a",
    },
    RoomDef {
        key: "censor-office",
        name: "The Censor's Office",
        description: "A desk, a lamp, a rubber stamp the size of a hand, and a wastebasket full of what used to be in books. The chair is warm.",
        exits: &[("west", "chart-room"), ("east", "restoration")],
        spawn: Some(("censor", 1, 300)),
        former: &[],
        tint: "#44403a",
    },
    RoomDef {
        key: "restoration",
        name: "The Restoration Bench",
        description: "Tweezers, brushes, tissue thin as breath, a book opened to a page being mended one fiber at a time. The hands are not visible. The work goes on.",
        exits: &[("west", "censor-office"), ("south", "tube-room"), ("north", "deep-reserve")],
        spawn: None,
        former: &[],
        tint: "#44403a",
    },
    RoomDef {
        key: "pulp-sump",
        name: "The Pulp Sump",
        description: "Where the water gets in and the books go soft. A gray slurry to the knee, warm, with things in it that were sentences, and things in it that are hungry.",
        exits: &[("east", "the-leak"), ("south", "lost-things")],
        spawn: Some(("lamprey", 2, 200)),
        former: &[],
        tint: "#2f3a40",
    },
    RoomDef {
        key: "the-leak",
        name: "The Leak",
        description: "Water down one wall in a sheet, the shelves beside it swollen and splitting. Something lives in the wet and does not like the light.",
        exits: &[("west", "pulp-sump"), ("east", "reserve"), ("south", "acid-room")],
        spawn: Some(("lamprey", 1, 200)),
        former: &[],
        tint: "#2f3a40",
    },
    RoomDef {
        key: "reserve",
        name: "The Reserve",
        description: "Books kept behind a counter, to be asked for by title. The titles are on a list. The list is of books about you.",
        exits: &[("west", "the-leak"), ("east", "strongroom"), ("south", "chart-room")],
        spawn: None,
        former: &[],
        tint: "#2f3a40",
    },
    RoomDef {
        key: "strongroom",
        name: "The Strongroom of Unreturned Things",
        description: "A steel door left open. Inside, every book anyone meant to bring back and did not, each with its reader's name in the front, in their own hand.",
        exits: &[("west", "reserve"), ("east", "deep-reserve")],
        spawn: Some(("custodian", 1, 900)),
        former: &[],
        tint: "#2f3a40",
    },
    RoomDef {
        key: "deep-reserve",
        name: "The Deep Reserve",
        description: "The last room, where the catalogue ends and the cataloguer sits. It knows where everything is. It does not know what anything is for, and it has your entry open.",
        exits: &[("west", "strongroom"), ("south", "restoration")],
        spawn: Some(("cataloguer", 1, 900)),
        former: &[],
        tint: "#2f3a40",
    },
    // Up inside the works of the Stopped Clocktower, by the escapement: the seat of
    // the Conclave, eleven rooms, designed by a visiting architect (design/conclave.md).
    RoomDef {
        key: "escapement-stair",
        name: "The Escapement Stair",
        description: "Behind the clock faces a narrow stair runs between two toothed wheels, and each step waits for a tooth to let it go before it lets you down. The ticking is louder here than in the tower, and slower. Somewhere ahead a bell for order was rung a long time ago and has not quite stopped.",
        exits: &[("escapement", "clocktower"), ("north", "petition-gallery")],
        spawn: None,
        former: &[],
        tint: "#3a2e2e",
    },
    RoomDef {
        key: "petition-gallery",
        name: "The Petitioners' Gallery",
        description: "Long benches face a double door, and the wood is polished pale where people have sat and waited. A brass machine by the entrance gives out numbered tickets. The number on yours is always a little higher than the last one called.",
        exits: &[("south", "escapement-stair"), ("west", "robing-room"), ("east", "tally-room"), ("north", "brass-rail")],
        spawn: None,
        former: &[],
        tint: "#3a2e2e",
    },
    RoomDef {
        key: "robing-room",
        name: "The Robing Room",
        description: "Black robes with a red cord at the shoulder hang on pegs down both walls, each under a name card in a hand no one here writes. The robes hold the shape of whoever wore them last. Now and then one of them straightens, as if the room had been addressed.",
        exits: &[("east", "petition-gallery"), ("north", "book-lectern")],
        spawn: None,
        former: &[],
        tint: "#3a2e2e",
    },
    RoomDef {
        key: "tally-room",
        name: "The Tally Room",
        description: "A brass tube comes down through the ceiling from the Clocktower and ends over a slanted desk. Once an hour a curl of paper drops out of it, figures on one side, and it is weighed on a small scale before anyone reads it. The scale is kept very clean. The Tally Book is not here, whatever the room is called; the Clerk keeps it in the Division Lobby, north, west, north, west (`where tally book` says the way from wherever you stand).",
        exits: &[("west", "petition-gallery"), ("north", "roll-hall")],
        spawn: Some(("engrosser", 1, 600)),
        former: &[],
        tint: "#3a2e2e",
    },
    RoomDef {
        key: "brass-rail",
        name: "The Bar of the Conclave",
        description: "A brass rail crosses the room at waist height, and the floor on the near side is worn in a line where people have stood and gone no further. The carpet past the rail is a different red. The rail is warm, as if a great many hands have just let go of it.",
        exits: &[("south", "petition-gallery"), ("west", "book-lectern"), ("east", "roll-hall"), ("north", "session-benches")],
        spawn: Some(("tipstaff", 1, 900)),
        former: &[],
        tint: "#3a2e2e",
    },
    RoomDef {
        key: "book-lectern",
        name: "The Lectern",
        description: "There is one lectern here, too tall for anyone to read at comfortably, and the Book lies open on it. Whatever statute you have in mind, the page turns to it and shows its number in ink that is still wet. The ink is always still wet.",
        exits: &[("south", "robing-room"), ("east", "brass-rail")],
        spawn: None,
        former: &[],
        tint: "#3a2e2e",
    },
    RoomDef {
        key: "roll-hall",
        name: "The Roll of Hoards",
        description: "The far wall is one long slate board ruled into two columns, one for a name and one for a share. A name posted there stays in chalk until the next sitting, and people cross the room along the other wall so as not to read it. The chalk in the tray is new every night, whether or not any was used.",
        exits: &[("south", "tally-room"), ("west", "brass-rail")],
        spawn: Some(("engrosser", 1, 600)),
        former: &[],
        tint: "#3a2e2e",
    },
    RoomDef {
        key: "session-benches",
        name: "The Sitting",
        description: "Benches rise in tiers around a well of green carpet, and a robe lies folded on every seat with nobody in it. At the bottom of the well is a table with a mace laid across it and a clock that has only one hour marked on it. The room is quiet the way a room is quiet just after it has been called to order.",
        exits: &[("south", "brass-rail"), ("west", "division-lobby"), ("east", "minute-room"), ("north", "engrossing-room")],
        spawn: None,
        former: &[],
        tint: "#3a2e2e",
    },
    RoomDef {
        key: "division-lobby",
        name: "The Division Lobby",
        description: "A narrow corridor, with two doors at one end marked AYE and NO in gold. Behind the paint they are one door. At the far end, a clerk's desk holds a tally book ruled in a single column.",
        exits: &[("east", "session-benches")],
        spawn: None,
        former: &[],
        tint: "#3a2e2e",
    },
    RoomDef {
        key: "minute-room",
        name: "The Minute Room",
        description: "Thin bound volumes fill the shelves, one for each sitting, dated in gold on the spine. The newest is dated tomorrow. It is written out in full in a careful hand, with a blank wherever a number should go.",
        exits: &[("west", "session-benches")],
        spawn: None,
        former: &[],
        tint: "#3a2e2e",
    },
    RoomDef {
        key: "engrossing-room",
        name: "The Engrossing Room",
        description: "High desks stand in a row, each with a stool and a lamp turned low, and a statute copied so large and so carefully that one line fills a page. No pen is ever put down. Each is passed to the next clerk still wet.",
        exits: &[("south", "session-benches")],
        spawn: Some(("engrosser", 2, 600)),
        former: &[],
        tint: "#3a2e2e",
    },
    // The Namery, the office the nightly naming tool keeps (the Bindery has its room in
    // the Lower Stacks already); names are dealt from here.
    RoomDef {
        key: "namery",
        name: "The Namery",
        description: "A narrow office of pigeonholes, floor to ceiling, each with a slip of paper in it and a name on the slip, waiting for something to earn it. A few holes are empty and their slips are gone. The pigeonholes are refilled at night, by a hand that writes in a very old register. Nothing hunts here; the names are not yet anyone's.",
        exits: &[("out", "archive"), ("post", "dead-letters")],
        spawn: None,
        former: &[],
        tint: "#4a4638",
    },
    // C3: the Dead Letter Office, where a word left for a name the world then forgot
    // is kept (`letters`, `reply`, `carry`).
    RoomDef {
        key: "dead-letters",
        name: "The Dead Letter Office",
        description: "A back room behind the Namery's pigeonholes, with pigeonholes of its own, and these have letters in them: every word left for someone away whose name the world forgot before they came back to hear it. A sorting table, a lamp, a stool nobody sits on. `letters` reads what is waiting; `reply` answers one to whoever wrote it; `carry` takes one to whoever holds the name now. Nothing hunts here.",
        exits: &[("out", "namery")],
        spawn: None,
        former: &[],
        tint: "#46423a",
    },
    // The Exergue District, eight rooms under the Clocktower, one for each coin from
    // the farthing to the sovereign (the owner: "8 rooms to mirror the coinage in
    // circulation, devastating guards to protect the purse, a bar where a story about
    // money is worth its weight, just not in gold"). Every room reads the economy as it
    // stands; the exergue is the strip under a coin's device where the date goes.
    RoomDef {
        key: "exergue-stair",
        name: "The Mint Stair",
        description: "A stair cut into the stone under the clock, each step stamped with a date in a hand that pressed too hard. The dates run downward and do not run in order. Somewhere below, a press is working, slowly, and the stone carries it up through your feet. Nothing hunts here.",
        exits: &[("up", "clocktower"), ("down", "counting-bar")],
        spawn: None,
        former: &[],
        tint: "#4a4034",
    },
    RoomDef {
        key: "counting-bar",
        name: "The Counting Bar",
        description: "A low room with a long bar of blackened oak, a slate behind it chalked in columns, and a tapster who weighs what it is told. Coins are not taken here; stories about them are, by the ounce. The glasses are heavy and never quite full.",
        exits: &[("up", "exergue-stair"), ("east", "die-room"), ("hall", "mintmark-hall")],
        spawn: Some(("tapster", 1, 900)),
        former: &[],
        tint: "#5a3e2a",
    },
    RoomDef {
        key: "die-room",
        name: "The Die Room",
        description: "Racks of steel dies to the ceiling, one for every coin the world has struck, cut in reverse so you read them in a mirror. Filings glitter in the seams of the floor. Something small works along the racks with a pair of shears, trimming.",
        exits: &[("west", "counting-bar"), ("north", "striking-floor")],
        spawn: Some(("clipper", 2, 480)),
        former: &[],
        tint: "#505a62",
    },
    RoomDef {
        key: "striking-floor",
        name: "The Striking Floor",
        description: "Three screw presses in a row, each taller than a door, their beams swinging slow enough to count. The plate under the middle press is changed every hour, by no one you see, and bears the hour's rates. The floor is warm. Dust hangs in the air and settles on everything you carry.",
        exits: &[("south", "die-room"), ("east", "assay-office")],
        spawn: Some(("sweater", 2, 600)),
        former: &[],
        tint: "#6a5a3a",
    },
    RoomDef {
        key: "assay-office",
        name: "The Assay Office",
        description: "A narrow office with a balance under a glass bell, a row of touchstones, and a furnace the size of a loaf. The Assayer stands at the balance and does not look up until you are close enough to weigh. What it weighs, it keeps a sample of.",
        exits: &[("west", "striking-floor"), ("north", "mintmark-hall")],
        spawn: Some(("assayer", 1, 1200)),
        former: &[],
        tint: "#3a4a4a",
    },
    RoomDef {
        key: "mintmark-hall",
        name: "The Mintmark Hall",
        description: "A long gallery hung with a register in iron frames: every kept thing in the world, with the mark it was struck under and the hour. The frames are cold. A clerk's stool stands empty at the far end, its cushion still dented.",
        exits: &[("south", "assay-office"), ("bar", "counting-bar"), ("vault", "purse-room")],
        spawn: None,
        former: &[],
        tint: "#3f3a4a",
    },
    RoomDef {
        key: "purse-room",
        name: "The Purse Room",
        description: "A round vault door stands open on a room the size of a confessional. In the middle, on a plinth, the Purse: a sack of worn leather that holds a twentieth of everything the world remembers, and knows it. Two Wardens stand either side of the plinth. They have never left it. They do not intend to.",
        exits: &[("out", "mintmark-hall"), ("down", "exergue")],
        spawn: Some(("warden", 2, 1500)),
        former: &[],
        tint: "#2e2a30",
    },
    RoomDef {
        key: "exergue",
        name: "The Exergue",
        description: "The bottom of the district: a strip of a room, wider than it is deep, under everything else. A plaque runs the length of the far wall, ruled for dates, and whatever is written on it is stamped with the hour it was written. The press above has stopped. It is very quiet, the way the underside of a coin is quiet.",
        exits: &[("up", "purse-room")],
        spawn: None,
        former: &[],
        tint: "#2a2622",
    },
    // The Wistful Trail: nine rooms north over the stile at the back of the Orchard, uphill,
    // where things are missed rather than forgotten. Laid three by three: the Stile at
    // the bottom-left corner, the Long Afternoon at the top-right behind a door that
    // asks for a word, the Bog in the bottom row between the Stile and the Lookout.
    // `trail.rs` has the Bog's trap, the wicket's word and the Lookout's view.
    RoomDef {
        key: "trail-stile",
        name: "The Stile",
        description: "A wooden stile over a drystone wall at the back of the Orchard, its top step worn to a dip by people pausing on it to look back. A signpost stands on the far side with more arms than a signpost needs, and every arm points to somewhere you have already been. Nothing hunts here.",
        exits: &[("south", "orchard"), ("north", "trail-meadow"), ("east", "trail-bog")],
        spawn: None,
        former: &[],
        tint: "#8a9a6a",
    },
    RoomDef {
        key: "trail-meadow",
        name: "The Sighing Meadow",
        description: "Long grass to the waist, moving in a wind you cannot feel, and the sound it makes is not quite a sigh and not quite your name. Butterflies the color of old envelopes keep landing on your shoulders and taking off again when you look.",
        exits: &[("south", "trail-stile"), ("north", "trail-picnic"), ("east", "trail-hollow-way")],
        // Back in three minutes (D16): at forty-five seconds the meadow was a mine,
        // ~80 XP a minute against the Fogbank's 15–30; a stop, not a mine.
        spawn: Some(("sigh", 3, 180)),
        former: &[],
        tint: "#a8b878",
    },
    RoomDef {
        key: "trail-picnic",
        name: "The Picnic That Waited",
        description: "A checked blanket spread on a slope with a view, a wicker basket, four plates and four cups, one of each used. The bread has gone to stone and the stone has gone to moss. Somebody said they would be right back, and the tea is still warm.",
        exits: &[("south", "trail-meadow"), ("east", "trail-well")],
        spawn: Some(("picnicker", 1, 240)),
        former: &[],
        tint: "#c8a070",
    },
    RoomDef {
        key: "trail-hollow-way",
        name: "The Hollow Way",
        description: "A sunken lane worn below the fields by feet that stopped coming, its banks higher than your head and knotted with roots, the light green and underwater. Every so often a pebble comes down one bank, as if something up there had shifted its weight.",
        exits: &[("west", "trail-meadow"), ("north", "trail-well"), ("east", "trail-hush"), ("south", "trail-bog")],
        spawn: Some(("footpad", 2, 150)),
        former: &[],
        tint: "#5a6a48",
    },
    RoomDef {
        key: "trail-well",
        name: "The Wishing Well",
        description: "A round well with a little roof, a bucket on a chain, and a smell of cold coming up out of it. Drop a coin in and it is handed back to you a day later, warm, by someone you cannot see. Cut into the inside of the rim, deep, in capitals, where only a person leaning too far would read it, is one word: LINGER. Someone has tried to smooth it over.",
        exits: &[("south", "trail-hollow-way"), ("west", "trail-picnic"), ("wicket", "trail-afternoon")],
        spawn: None,
        former: &[],
        tint: "#6a8a9a",
    },
    RoomDef {
        key: "trail-afternoon",
        name: "The Long Afternoon",
        description: "A walled garden where it is always four o'clock. The sun has stuck a hand's width above the wall and the shadows have given up moving. Bees go from flower to flower and never get to the end of the row. A deck chair faces the sun with a book open on it, face down, and the book is about you, and it is a good one.",
        exits: &[("wicket", "trail-well")],
        spawn: Some(("gardener", 1, 600)),
        former: &[],
        tint: "#e0c070",
    },
    RoomDef {
        key: "trail-hush",
        name: "The Hush",
        description: "A dell where the trail drops between two mossy banks and every sound goes soft: your footsteps, your breath, the thing you were about to say. Birds open their beaks here and nothing comes out, and they do not seem to mind. Whatever you shout here arrives somewhere else, a little later, as a whisper.",
        exits: &[("west", "trail-hollow-way"), ("south", "trail-lookout")],
        spawn: None,
        former: &[],
        tint: "#7a8a8a",
    },
    RoomDef {
        key: "trail-bog",
        name: "The Bog of Second Thoughts",
        description: "A wet green flat between the hills, tussocks standing out of it like stepping stones, and the ground between them not ground. It is a shortcut; everyone can see it is a shortcut. The tussocks are the path. Linger on them and the bog takes a liking to you.",
        exits: &[("west", "trail-stile"), ("north", "trail-hollow-way"), ("east", "trail-lookout")],
        spawn: None,
        former: &[],
        tint: "#4a6a4a",
    },
    RoomDef {
        key: "trail-lookout",
        name: "The Lookout",
        description: "The top of the trail, a bald hill with a bench and a brass plate that names the view. From here you can see the whole trail, the nine rooms from the Stile up, small and clear and lit as if for the last time, and whoever is standing in them; the house and the fog below are out of its sight.",
        exits: &[("west", "trail-bog"), ("north", "trail-hush")],
        spawn: Some(("longsight", 1, 300)),
        former: &[],
        tint: "#b8c8d8",
    },
];

/// How leaving reads: "You go east." for a compass word, "You go back.", and "You
/// take the orrery." for a door with a name of its own ("You go orrery." was wrong).
pub fn went(exit: &str) -> String {
    match exit {
        "north" | "south" | "east" | "west" | "up" | "down" | "back" | "in" | "out" => {
            format!("You go {exit}.")
        }
        // R54: "You take the abstain." read wrong; a door word that is not plainly a
        // thing is taken as a door.
        e if !door_noun(e) => format!("You take the {exit} door."),
        _ => format!("You take the {exit}."),
    }
}

/// Whether an exit word reads as a thing you can take (a stair, a nook, a kiln): by
/// suffix and a short list; anything else is "the {word} door".
fn door_noun(word: &str) -> bool {
    const NOUNS: &[&str] = &[
        "stair",
        "stairs",
        "stile",
        "gate",
        "door",
        "arch",
        "hatch",
        "ladder",
        "tunnel",
        "passage",
        "path",
        "trail",
        "bridge",
        "nook",
        "alcove",
        "annex",
        "kiln",
        "cellar",
        "attic",
        "well",
        "wicket",
        "bog",
        "lookout",
        "hush",
        "escapement",
        "exergue",
        "names",
        "dive",
        "surface",
        "landing",
        "lane",
        "walk",
        "row",
        "hall",
        "hole",
        "gap",
        "crack",
        "vent",
        "chute",
        "slide",
        "steps",
        "step",
        "ramp",
        "lift",
        "shaft",
        "sewer",
        "drain",
        "pipe",
        "window",
        "mirror",
        "pool",
        "moonpool",
        "mossbed",
        "bramble",
        "hedge",
        "orchard",
        "meadow",
        "garden",
        "link",
        "somewhere",
        "escape",
        "teapot",
        "cloak",
        "sleeve",
        "closet",
        "nest",
        "burrow",
        "den",
        "lair",
        "orrery",
        "cistern",
        "oubliette",
        "vault",
        "crypt",
        "gallery",
        "tower",
        "cave",
    ];
    NOUNS.contains(&word)
        || word.ends_with("way")
        || word.ends_with("walk")
        || word.ends_with("room")
        || word.ends_with("stair")
        || word.ends_with("door")
        || word.ends_with("gate")
        || word.ends_with("path")
        || word.ends_with("hole")
        || word.ends_with("bed")
        || word.ends_with("pool")
}

/// The room entity with this key.
/// Where a kind lives: the seed rooms whose spawn is that kind, by name. Hybrids and
/// wanderers live nowhere in particular.
pub fn homes_of(kind: &str) -> Vec<&'static str> {
    // R206: a room's second kind is a home too, and the nearer one (the Archive's
    // bookworm, the Cistern's silverfish), so it is said first.
    ROOMS
        .iter()
        .filter(|r| {
            super::combat::ALSO
                .iter()
                .any(|(room, k, _)| *room == r.key && *k == kind)
        })
        .chain(
            ROOMS
                .iter()
                .filter(|r| r.spawn.is_some_and(|(k, _, _)| k == kind)),
        )
        .map(|r| r.name)
        .collect()
}

/// Every kind that has a home, with its rooms: `{kind, rooms}`.
pub fn homes() -> Vec<serde_json::Value> {
    let mut kinds: Vec<&str> = ROOMS
        .iter()
        .filter_map(|r| r.spawn.map(|(k, _, _)| k))
        .collect();
    kinds.sort();
    kinds.dedup();
    kinds
        .into_iter()
        .map(|k| serde_json::json!({"kind": k, "rooms": homes_of(k)}))
        .collect()
}

pub fn room_by_key(world: &World, key: &str) -> Option<EntityId> {
    world
        .store
        .of_kind("room")
        .find(|e| plain_str(world, e.id, ROOM, "key") == Some(key))
        .map(|e| e.id)
}

/// The arrival room.
pub fn arrival(world: &World) -> Option<EntityId> {
    room_by_key(world, ARRIVAL)
}

/// Whether newcomers are shared between the two doors: they are, unless
/// `FOXING_SECOND_DOOR` is `0`, `false` or `off` (David's word, 11-13: "switch is on
/// by default"). With it off, every new name wakes at the Threshold, and the Postern
/// is a room like another that a name may make its home.
fn second_door_open() -> bool {
    // The tests log in two and three names and walk them from the Threshold, as
    // they did before there was a second door: under test the door is shut unless
    // the test opens it.
    #[cfg(test)]
    {
        SECOND_DOOR.with(|c| c.get()).unwrap_or(false)
    }
    #[cfg(not(test))]
    {
        !crate::env::var("FOXING_SECOND_DOOR")
            .is_some_and(|v| matches!(v.trim(), "0" | "false" | "off"))
    }
}

#[cfg(test)]
thread_local! {
    /// A test's own switch, so one test's open door is not another's.
    pub(crate) static SECOND_DOOR: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// The doors of the world that stand, the Threshold first.
pub fn doors(world: &World) -> Vec<EntityId> {
    [ARRIVAL, POSTERN]
        .iter()
        .filter_map(|k| room_by_key(world, k))
        .collect()
}

/// C11: where a name wakes. One it has made its home, while that door stands; else,
/// for a newcomer, the door with fewer names in it when the second door is open (the
/// Threshold at a tie), and the Threshold when it is not. If a door is gone, every
/// name wakes at the other: the world has no single room it cannot do without.
pub fn landing(world: &World, citizen: Option<EntityId>) -> Option<EntityId> {
    let doors = doors(world);
    if let Some(home) = citizen
        .and_then(|c| plain_str(world, c, super::CITIZEN, "home"))
        .and_then(|k| room_by_key(world, k))
        .filter(|r| doors.contains(r))
    {
        return Some(home);
    }
    if citizen.is_none() && second_door_open() && doors.len() > 1 {
        let crowd = |r: EntityId| {
            world
                .store
                .children(r)
                .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
                .count()
        };
        return doors.iter().copied().min_by_key(|r| crowd(*r));
    }
    doors.first().copied()
}

/// Build every room the log does not have, and the exits once all rooms exist. A room
/// from an older log that has a name but no key is claimed by name.
pub fn ensure_rooms(world: &mut World) -> Result<(), String> {
    let mut ids: BTreeMap<&str, EntityId> = BTreeMap::new();
    for def in ROOMS {
        let existing = room_by_key(world, def.key)
            .or_else(|| def.former.iter().find_map(|k| room_by_key(world, k)))
            .or_else(|| {
                world
                    .store
                    .of_kind("room")
                    .find(|e| {
                        plain_str(world, e.id, ROOM, "key").is_none()
                            && plain_str(world, e.id, ROOM, "name") == Some(def.name)
                    })
                    .map(|e| e.id)
            });
        let id = match existing {
            Some(id) if plain_str(world, id, ROOM, "key") == Some(def.key) => {
                // The world's own words follow the code: a seeded room's text and
                // name are the def's (the Tally Room's new sentence would otherwise
                // never reach the box).
                if plain_str(world, id, ROOM, "description") != Some(def.description)
                    || plain_str(world, id, ROOM, "name") != Some(def.name)
                {
                    let mut changes = BTreeMap::new();
                    changes.insert("description".into(), Field::Plain(json!(def.description)));
                    changes.insert("name".into(), Field::Plain(json!(def.name)));
                    let fields = super::building::merged(world, id, changes);
                    world
                        .record(
                            Actor::Driver,
                            StoreEvent::ComponentSet {
                                id,
                                component: ROOM.into(),
                                fields,
                            },
                        )
                        .map_err(|e| e.to_string())?;
                }
                id
            }
            Some(id) => {
                world
                    .record(
                        Actor::Driver,
                        StoreEvent::ComponentSet {
                            id,
                            component: ROOM.into(),
                            fields: room_fields(def),
                        },
                    )
                    .map_err(|e| e.to_string())?;
                id
            }
            None => {
                let id = world.store.allocate_id();
                world
                    .record(
                        Actor::Driver,
                        StoreEvent::EntityCreated {
                            id,
                            kind: "room".into(),
                            owner: None,
                            parent: None,
                            scopes: vec![],
                        },
                    )
                    .map_err(|e| e.to_string())?;
                world
                    .record(
                        Actor::Driver,
                        StoreEvent::ComponentSet {
                            id,
                            component: ROOM.into(),
                            fields: room_fields(def),
                        },
                    )
                    .map_err(|e| e.to_string())?;
                id
            }
        };
        ids.insert(def.key, id);
    }
    for def in ROOMS {
        let id = ids[def.key];
        // The seed exits this room lacks, added to whatever it has (a dug exit stays).
        // Exits that are there already are left alone, except a seed exit the seed no
        // longer has: an exit from a seed room to another seed room that the definition
        // does not name is a leftover of an earlier map, and is dropped. (Lantern Row
        // kept an east to the Threshold beside the south for four days.) Dug exits lead
        // to dug rooms and are never touched.
        let mut exits: serde_json::Map<String, Value> = match world
            .store
            .entity(id)
            .and_then(|e| e.components.get(EXITS))
            .and_then(|c| c.fields.get("exits"))
        {
            Some(Field::Plain(Value::Object(m))) => m.clone(),
            _ => serde_json::Map::new(),
        };
        let mut missing = false;
        let seed_ids: std::collections::HashSet<u64> = ids.values().map(|i| i.0).collect();
        let stale: Vec<String> = exits
            .iter()
            .filter(|(name, to)| {
                // Only the seed's own compass words: an exit someone named by hand,
                // wherever it leads, is theirs.
                const COMPASS: &[&str] = &["north", "south", "east", "west", "up", "down"];
                // (And the Postern's first door to the lane, `in`: a word the gate's
                // grammar drops, so `travel in` written in a message had no argument.)
                (COMPASS.contains(&name.as_str()) || (def.key == POSTERN && name.as_str() == "in"))
                    && to.as_u64().is_some_and(|t| seed_ids.contains(&t))
                    && !def.exits.iter().any(|(n, _)| n == name)
            })
            .map(|(name, _)| name.clone())
            .collect();
        for name in stale {
            world.notice(format!(
                "Room **{}** lost its exit `{name}`: the seed no longer has it.",
                def.key
            ));
            exits.remove(&name);
            missing = true;
        }
        for (name, key) in def.exits {
            match exits.get(*name).and_then(Value::as_u64) {
                None => {
                    exits.insert(name.to_string(), json!(ids[key].0));
                    missing = true;
                }
                Some(to) if to != ids[key].0 => world.notice(format!(
                    "Room **{}** has exit `{name}` leading to entity {to}, not seed room `{key}`; \
                     the seed exit was not added.",
                    def.key
                )),
                Some(_) => {}
            }
        }
        if missing {
            let mut fields = BTreeMap::new();
            fields.insert("exits".into(), Field::Plain(Value::Object(exits)));
            world
                .record(
                    Actor::Driver,
                    StoreEvent::ComponentSet {
                        id,
                        component: EXITS.into(),
                        fields,
                    },
                )
                .map_err(|e| e.to_string())?;
        }
        let has_sprite = world
            .store
            .entity(id)
            .is_some_and(|e| e.components.contains_key(super::SPRITE))
            && !super::sprite_is_old(world, id);
        if !has_sprite {
            world
                .record(
                    Actor::Driver,
                    StoreEvent::ComponentSet {
                        id,
                        component: super::SPRITE.into(),
                        fields: super::sprite_fields("#", def.tint, def.key),
                    },
                )
                .map_err(|e| e.to_string())?;
        }
        let has_spawn = world
            .store
            .entity(id)
            .is_some_and(|e| e.components.contains_key(SPAWN));
        if let (Some((creature, count, respawn)), false) = (def.spawn, has_spawn) {
            let mut fields = BTreeMap::new();
            fields.insert("creature".into(), Field::Plain(json!(creature)));
            fields.insert("count".into(), Field::Plain(json!(count)));
            fields.insert("respawn_seconds".into(), Field::Plain(json!(respawn)));
            fields.insert("next_at".into(), Field::Plain(json!(0)));
            world
                .record(
                    Actor::Driver,
                    StoreEvent::ComponentSet {
                        id,
                        component: SPAWN.into(),
                        fields,
                    },
                )
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn room_fields(def: &RoomDef) -> BTreeMap<String, Field> {
    let mut fields = BTreeMap::new();
    fields.insert("key".into(), Field::Plain(json!(def.key)));
    fields.insert("name".into(), Field::Plain(json!(def.name)));
    fields.insert("description".into(), Field::Plain(json!(def.description)));
    fields
}

/// The exits of a room: name to room id.
pub fn exits_of(world: &World, room: EntityId) -> BTreeMap<String, EntityId> {
    let Some(Field::Plain(Value::Object(m))) = world
        .store
        .entity(room)
        .and_then(|e| e.components.get(EXITS))
        .and_then(|c| c.fields.get("exits"))
    else {
        return BTreeMap::new();
    };
    m.iter()
        .filter_map(|(k, v)| v.as_u64().map(|id| (k.clone(), EntityId(id))))
        .collect()
}

/// The room a citizen stands in.
pub fn room_of(world: &World, citizen: EntityId) -> Option<EntityId> {
    world.store.entity(citizen).and_then(|e| e.parent)
}

/// The query behind `look`: the room's name, description and exits, and what is here.
pub fn look_query(room: EntityId) -> Value {
    json!({
        "root": {"entity": room.0},
        "select": {ROOM: ["key", "name", "description", "memory", "dug_with", "kept", "bound"], EXITS: "*"},
        "relations": {
            "contents": {
                "kinds": ["citizen", "creature"],
                "select": {super::CITIZEN: ["name"], super::CREATURE: ["name", "hp", "max_hp", "attack", "parents", "born_of", "mutation"], super::mind::MIND: ["name", "tier"]}
            }
        }
    })
}

/// Describe a room as `actor` sees it. Runs the observation query and renders prose
/// from the structure, which is the text client's job: the driver sends components,
/// the client makes sentences.
/// Ten minutes out of a room, and the room's say on return is a homecoming.
pub const AWAY_MS: i64 = 10 * 60 * 1000;

/// A line of the room's own for who stands in it, said on arrival: a stranger it
/// has not had before, a keeper (its digger, or a hand that remembered into it),
/// the Docent on its round (said to whoever is there). The owner's `greet` is the
/// owner's; this is the room's.
pub fn reaction(
    world: &World,
    actor: &str,
    room: EntityId,
    first_time: bool,
    away: bool,
) -> Option<String> {
    let name = plain_str(world, room, ROOM, "name")?.to_string();
    let owner = world.store.entity(room).and_then(|e| e.owner.clone());
    let seed = (room.0 as usize).wrapping_mul(2654435761) ^ actor.len();
    // The room's own say to its digger or a keeper is for a homecoming, not every
    // step through the door: after ten minutes away, or the first time.
    if !first_time && !away {
        return None;
    }
    if owner.as_deref() == Some(actor) {
        return Some(match seed % 3 {
            0 => format!(
                "{name} is yours; it brightens a little, as a dug room does for its digger."
            ),
            1 => {
                format!("{name} knows the hand that opened it. It has kept the shape you gave it.")
            }
            _ => format!("{name} settles around you. It was yours before it was anything."),
        });
    }
    let kept_by_you = super::building::kept_by(world, room)
        .iter()
        .any(|k| k == actor);
    if kept_by_you {
        return Some(format!("{name} knows your hand: some of what keeps it is yours, and it holds the door a little wider for you."));
    }
    if first_time {
        let dug = owner.is_some();
        return Some(match (dug, seed % 10) {
            (true, n) if n % 2 == 0 => format!("{name} has not had you in it before; it was dug by {}, and it looks you over the way a borrowed room does.", owner.unwrap_or_default()),
            (true, _) => format!("{name} was {}'s before it was anyone's, and it has not decided about you.", owner.unwrap_or_default()),
            (false, 0) => format!("{name} has not had you in it before; it notes you the way a shelf notes a book."),
            (false, 1) => format!("You are new to {name}, and it is new to you; the stone takes a moment to settle."),
            (false, 2) => format!("{name} does not know you yet. It will, if you come back."),
            (false, 3) => format!("Nothing in {name} has your shape yet; the room looks you over, and keeps the look."),
            (false, 4) => format!("{name} had not expected you. Rooms expect very little; it is their one virtue."),
            (false, 5) => format!("{name} makes a space for you the size of you, and no larger."),
            (false, 6) => format!("Your first step into {name} is the one it will remember; the rest it files."),
            (false, 7) => format!("{name} takes you in without a word. It has had words before, and kept none of them."),
            (false, 8) => format!("A stranger in {name}: the air moves once, and then goes on as it was."),
            (false, _) => format!("{name} counts you. It is not sure what it is counting toward."),
        });
    }
    None
}

/// The room's line for the Docent coming through, said to whoever stands there.
pub fn docent_came(world: &mut World, room: EntityId) {
    let Some(name) = plain_str(world, room, ROOM, "name").map(str::to_string) else {
        return;
    };
    let here: Vec<String> = world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen")
        .filter_map(|e| plain_str(world, e.id, super::CITIZEN, "name").map(str::to_string))
        .collect();
    if here.is_empty() {
        return;
    }
    let line = match room.0 % 3 {
        0 => format!("{name} settles as the Docent comes through; the dust lies down for it."),
        1 => format!(
            "{name} straightens for the Docent, the way a room does for the one who keeps it."
        ),
        _ => format!(
            "Something in {name} is quieter with the Docent in it; the room knows its reader."
        ),
    };
    for who in here {
        super::combat::narrate(world, &who, line.clone());
    }
}

pub fn describe(
    world: &mut World,
    actor: &str,
    ctx: &foxing_registry::CallContext,
    room: EntityId,
) -> Result<Outcome, foxing_registry::CommandError> {
    describe_as(world, actor, ctx, room, false)
}

/// The room as `look` says it, or short (D25 a): the name, the exits, who and what
/// is here, what hunts. `travel` says a room short when you stood in it within the
/// hour (a walker on the back-trail read the Fogbank's page three times); `look`
/// always has the page.
pub fn describe_as(
    world: &mut World,
    actor: &str,
    ctx: &foxing_registry::CallContext,
    room: EntityId,
    brief: bool,
) -> Result<Outcome, foxing_registry::CommandError> {
    // Standing here is remembering it: the mini-map is what you have seen.
    super::atlas::mark_seen(world, actor, room)?;
    let obs = super::observe_as(world, ctx, look_query(room))?;
    let root = &obs.observation.root;
    let name = root["components"][ROOM]["name"]
        .as_str()
        .unwrap_or("?")
        .to_string();
    // A dug room's description is as remembered: whole, dimmed, or faded.
    let memory = root["components"][ROOM]["memory"].as_f64();
    // A lamp standing here lights the words: the description reads whole however
    // faded the room; the memory itself fades as it would.
    let lit = super::craft::fixture_of(world, room, "lamp").is_some();
    let desc = super::building::remembered_description(
        root["components"][ROOM]["description"]
            .as_str()
            .unwrap_or(""),
        if lit { None } else { memory },
    );
    let desc = if desc.ends_with(['.', '!', '?', '"', '”', ')']) || desc.is_empty() {
        desc
    } else {
        format!("{desc}.")
    };
    let desc_says_nothing_hunts = root["components"][ROOM]["description"]
        .as_str()
        .is_some_and(|d| d.contains("Nothing hunts here"));
    let dug_by = root["owner"].as_str().map(str::to_string);
    // R85: one order whatever the store hands back: the compass first, in its own
    // order, then the dug doors by name.
    let mut exits: Vec<String> = root["components"][EXITS]["exits"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    exits.sort_by_key(|e| {
        let compass = COMPASS.iter().position(|c| c == e);
        (compass.is_none(), compass.unwrap_or(0), e.clone())
    });
    let here = root["relations"]["contents"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let (others, remembered) = super::presence::split(world, room, actor);
    let creatures: Vec<Value> = here
        .iter()
        .filter(|n| n["kind"] == "creature")
        .map(|n| {
            let c = &n["components"][super::CREATURE];
            let id = EntityId(n["id"].as_u64().unwrap_or(0));
            let mut v = json!({"id": n["id"], "name": c["name"], "hp": c["hp"], "max_hp": c["max_hp"], "attack": c["attack"]});
            if c["parents"].is_array() {
                v["parents"] = c["parents"].clone();
                v["born_of"] = c["born_of"].clone();
                v["mutation"] = c["mutation"].clone();
            }
            if n["components"][super::mind::MIND].is_object() {
                v["mind"] = super::mind::summary(world, id);
            }
            v
        })
        .collect();
    let regards: Vec<String> = creatures
        .iter()
        .filter_map(|c| c["id"].as_u64())
        .filter_map(|id| super::mind::regard(world, EntityId(id), actor))
        .collect();
    // A digger's text starts with a capital however it was typed ("an ice fortress
    // with a bit of an aroma." read as a slip of the world's).
    let desc = {
        let mut c = desc.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    };
    let mut text = if brief {
        format!("{name}, as you left it (`look` has the page).")
    } else {
        format!("{name}. {desc}")
    };
    // Short, the stretches between the kept parts are written and then dropped: the
    // text is cut back to where each began.
    let cut_a = text.len();
    // H9: a district's door says so, once a session. H11: a hub says it has a post.
    {
        let key = plain_str(world, room, ROOM, "key")
            .unwrap_or("")
            .to_string();
        if let Some(d) = district_of(world, room).filter(|d| d.entry == key && !d.blurb.is_empty())
        {
            if let Some(f) = super::find_citizen(world, actor) {
                if super::combat::first_this_session(
                    world,
                    actor,
                    f.citizen,
                    &format!("edge_{}", d.entry.replace('-', "_")),
                ) {
                    let n = district_size(d);
                    let mut name = d.name.to_string();
                    if let Some(c) = name.get_mut(0..1) {
                        c.make_ascii_uppercase();
                    }
                    text.push_str(&format!(
                        " {name} begin{} here{}: {}.",
                        if d.name.ends_with('s') { "" } else { "s" },
                        if d.entry == MUDROOM {
                            String::new()
                        } else {
                            format!(" ({n} room{})", if n == 1 { "" } else { "s" })
                        },
                        d.blurb
                    ));
                }
            }
        }
        if HUBS.contains(&key.as_str()) {
            text.push_str(" A signpost stands here (`read signpost`).");
        }
        // C1: the Reading Bench, said until this name has used it.
        if key == POSTERN
            && super::find_citizen(world, actor)
                .is_some_and(|f| !super::exchange::bench_heard(world, f.citizen))
        {
            text.push_str(" The Reading Bench hears one story from each name, about anything, and asks no name of its listener: `story bench \"<a dozen different words>\"` is a first memory without a fight.");
        }
    }
    // R78: the weather, by the Clocktower's rates and the hour.
    if dug_by.is_none() {
        if let Some(wx) = super::weather::line(
            world,
            root["components"][ROOM]["key"].as_str().unwrap_or(""),
        ) {
            text.push(' ');
            text.push_str(&wx);
        }
    }
    if let Some(who) = &dug_by {
        let with = root["components"][ROOM]["dug_with"]
            .as_str()
            .map(|s| format!(" with the memory of {s}"))
            .unwrap_or_default();
        text.push_str(&format!(" Dug by {who}{with}."));
        // To its owner: how much of it is remembered and how long that halves in
        // (R14: a digger asked, and only `mine` said).
        if who == actor {
            if let (Some(mem), Some(hl)) = (
                super::building::memory(world, room),
                super::building::half_life_of(world, room),
            ) {
                text.push_str(&format!(
                    " Yours: {:.0}% remembered, halving every {}; {} of {} doors left.",
                    mem * 100.0,
                    super::combat::human(hl.0),
                    super::building::MAX_EXITS.saturating_sub(exits.len()),
                    super::building::MAX_EXITS
                ));
            }
        }
        if let Some(kept) = root["components"][ROOM]["kept"].as_array() {
            // F15: one name, however many times it kept the room: "Loamwhistle
            // (wisp ×3, hollow)", not the name three times over.
            let mut by_name: Vec<(String, Vec<(String, usize)>)> = Vec::new();
            for k in kept {
                let Some(by) = k["by"].as_str() else { continue };
                let with = k["with"].as_str().unwrap_or("something").to_string();
                let entry = match by_name.iter_mut().find(|(n, _)| n == by) {
                    Some(e) => e,
                    None => {
                        by_name.push((by.to_string(), Vec::new()));
                        by_name.last_mut().unwrap()
                    }
                };
                match entry.1.iter_mut().find(|(w, _)| *w == with) {
                    Some(w) => w.1 += 1,
                    None => entry.1.push((with, 1)),
                }
            }
            let keepers: Vec<String> = by_name
                .iter()
                .map(|(n, withs)| {
                    let w: Vec<String> = withs
                        .iter()
                        .map(|(w, c)| {
                            if *c > 1 {
                                format!("{w} \u{d7}{c}")
                            } else {
                                w.clone()
                            }
                        })
                        .collect();
                    // R87: "Oakmoss (with a wisp)", so the digger does not read as a wisp.
                    format!("{n} (with {})", w.join(", "))
                })
                .collect();
            if !keepers.is_empty() {
                text.push_str(&format!(
                    " Kept in memory by {}: the memory of a creature of theirs, fed to it.",
                    keepers.join(", ")
                ));
            }
        }
        // Bound by the Bindery: the appends folded into one passage, the hands credited.
        if let Some(hands) = root["components"][ROOM]["bound"]["hands"].as_array() {
            let hands: Vec<&str> = hands.iter().filter_map(Value::as_str).collect();
            text.push_str(&format!(
                " Bound, the hands of {}.",
                if hands.is_empty() {
                    "no one".to_string()
                } else {
                    hands.join(", ")
                }
            ));
        }
    }
    if brief {
        text.truncate(cut_a);
        // Short, a room of your own still says how much of it is left: the builder
        // of the Watch of 11-10 walked in and out five times and learned it only
        // from `look`.
        if dug_by.as_deref() == Some(actor) {
            if let (Some(mem), Some(hl)) = (
                super::building::memory(world, room),
                super::building::half_life_of(world, room),
            ) {
                text.push_str(&format!(
                    " Yours: {:.0}% remembered, halving every {}.",
                    mem * 100.0,
                    super::combat::human(hl.0)
                ));
            }
        }
    }
    if !exits.is_empty() {
        // Each exit says where it leads: three visitors in one night asked, because a
        // dug exit is named by its digger and tells a stranger nothing.
        let by_name = exits_of(world, room);
        // H4: the door's three ways say what each is for. H8: a spade from the
        // Mudroom's rack, carried, marks a dug door with whose it is and how much
        // of the room behind it is left.
        let at_door = plain_str(world, room, ROOM, "key") == Some(ARRIVAL);
        let spade = super::find_citizen(world, actor)
            .is_some_and(|f| super::door::carries_spade(world, f.citizen));
        // H14: the Mudroom's wall in rows: yours, the newest, the best remembered;
        // the rest behind `read doors`.
        let (listed, more_doors) = if is_mudroom(world, room) {
            super::door::rows(world, actor, &exits, &by_name)
        } else {
            (exits.clone(), 0)
        };
        let from_archive = plain_str(world, room, ROOM, "key") == Some("archive");
        let shown: Vec<String> = listed
            .iter()
            .map(|e| {
                let to = by_name.get(e).copied();
                match to.and_then(|to| super::plain_str(world, to, ROOM, "name")) {
                    Some(n) => {
                        let n = n.trim_start_matches("The ");
                        let gloss = match to.and_then(|to| plain_str(world, to, ROOM, "key")) {
                            Some("fogbank") if at_door => ": fight",
                            Some("lantern-row") if at_door => ": read and tell",
                            Some(MUDROOM) if at_door => ": dig",
                            // H12: the stair that asks a level says so on the door.
                            Some("lower-stacks") if from_archive => {
                                "; the Stacks stair, a level of 100"
                            }
                            _ => "",
                        };
                        let owner = to
                            .and_then(|to| world.store.entity(to))
                            .and_then(|r| r.owner.clone());
                        match (
                            spade,
                            owner,
                            to.and_then(|to| super::building::memory(world, to)),
                        ) {
                            (true, Some(o), Some(m)) => {
                                format!("{e} ({n}; dug by {o}, {:.0}% left)", m * 100.0)
                            }
                            _ => format!("{e} ({n}{gloss})"),
                        }
                    }
                    None => e.clone(),
                }
            })
            .collect();
        text.push_str(&format!(" Exits: {}.", shown.join(", ")));
        // C3: the Namery's back door says what is behind it.
        if plain_str(world, room, ROOM, "key") == Some("namery") {
            match super::letters::waiting(world) {
                0 => {}
                1 => text.push_str(" Behind the post door, one dead letter is waiting."),
                n => text.push_str(&format!(
                    " Behind the post door, {n} dead letters are waiting."
                )),
            }
        }
        if more_doors > 0 {
            text.push_str(&format!(
                " ({more_doors} more dug doors are on the wall, and each still opens by its word: `read doors` is the whole wall.)"
            ));
        }
    }
    let cut_b = text.len();
    if !exits.is_empty() {
        let by_name = exits_of(world, room);
        // A world's own room with doors dug off it says how many it has left and
        // which are newest (R14): a digger at a full Threshold was refused by surprise.
        if dug_by.is_none() {
            let dug: Vec<(u64, String)> = by_name
                .iter()
                .filter_map(|(e, &to)| {
                    let owned = world.store.entity(to).is_some_and(|r| r.owner.is_some());
                    owned.then(|| (to.0, e.clone()))
                })
                .collect();
            if !dug.is_empty() {
                // R207: the newest of what still stands (it named rooms at 0% left).
                let mut newest: Vec<(u64, String)> = dug
                    .iter()
                    .filter(|(id, _)| {
                        super::building::memory(world, EntityId(*id)).map_or(true, |m| m >= 0.01)
                    })
                    .cloned()
                    .collect();
                if newest.is_empty() {
                    newest = dug.clone();
                }
                newest.sort_by_key(|b| std::cmp::Reverse(b.0));
                let newest = newest
                    .iter()
                    .take(3)
                    .map(|(_, e)| e.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                if is_mudroom(world, room) {
                    text.push_str(&format!(
                        " {} door{} dug here, and room for any number; the newest: {newest}.",
                        dug.len(),
                        if dug.len() == 1 { "" } else { "s" }
                    ));
                } else {
                    let left = super::building::MAX_EXITS.saturating_sub(exits.len());
                    text.push_str(&format!(
                        " {} of {} doors left here; the newest dug: {newest}.",
                        left,
                        super::building::MAX_EXITS,
                    ));
                }
            }
        }
    }
    if brief {
        text.truncate(cut_b);
    }
    // A door only you can see, while you hold the memory of what is behind it.
    if let Some(f) = super::find_citizen(world, actor) {
        for (door, to) in super::morphos::private_doors(world, f.window, room) {
            text.push_str(&format!(
                " And a door no one else sees, {door} ({}), there while you remember it.",
                super::plain_str(world, to, ROOM, "name")
                    .unwrap_or("?")
                    .trim_start_matches("The ")
            ));
        }
    }
    if let Some(standing) = super::craft::fixtures_line(world, room) {
        text.push(' ');
        text.push_str(&standing);
    }
    if let Some(ground) = super::craft::ground_line(world, room) {
        text.push(' ');
        text.push_str(&ground);
    }
    let cut_c = text.len();
    // R79: the longsight says whom it is looking at: the last name to arrive
    // anywhere else on the trail, else the moon.
    if let Some((ls, _, _, _)) = super::combat::creatures_in(world, room)
        .into_iter()
        .find(|c| c.1 == "longsight")
    {
        let who = super::mind::display(world, ls);
        let looking = world
            .recent_events(1500)
            .iter()
            .rev()
            .filter(|e| e.kind == "store.entity_moved" && e.payload["parent"] != room.0)
            .find_map(|e| {
                let to = EntityId(e.payload["parent"].as_u64()?);
                let key = super::plain_str(world, to, ROOM, "key")?;
                if !key.starts_with("trail-") {
                    return None;
                }
                let who = EntityId(e.payload["id"].as_u64()?);
                let name = super::plain_str(world, who, super::CITIZEN, "name")?;
                if name == actor {
                    return None;
                }
                Some(format!(
                    "{name} in {}",
                    super::plain_str(world, to, ROOM, "name").unwrap_or("the trail")
                ))
            });
        text.push_str(&format!(
            " {who} is looking at {}.",
            looking.unwrap_or_else(
                || "the moon, which it has been looking at for some time".to_string()
            )
        ));
    }
    // F14: the Stacks are where the map matters and nothing there said so; a name
    // without a lantern is told what one keeps, once a room.
    if let Some(f) = super::find_citizen(world, actor) {
        let key = root["components"][ROOM]["key"].as_str().unwrap_or("");
        // (H5's line for a new name, the three ways, is the room's own text since H4.)
        // R82: the day, when it is a named one.
        if key == ARRIVAL {
            if let Some(day) = super::life::holiday(world) {
                if day != super::life::SABBATH {
                    text.push_str(&format!(
                        " Today is {day}{}.",
                        if day == "All Names' Day" {
                            ": the Namery deals at forty"
                        } else {
                            "; the Conclave does not divide"
                        }
                    ));
                }
            }
        }
        // R80: a chalk tally on the Threshold of who came through today.
        if key == ARRIVAL {
            let names: Vec<String> =
                super::building::came_through_within(world, room, 24 * 3_600_000)
                    .into_iter()
                    .map(|(n, _)| n)
                    .collect();
            if !names.is_empty() {
                let shown: Vec<&str> = names.iter().take(6).map(String::as_str).collect();
                text.push_str(&format!(
                    " Chalked on the stone, today's tally: {} came through{}{}.",
                    names.len(),
                    // R131: "among them" only when the chalk shows fewer than came.
                    if names.len() <= shown.len() {
                        ": "
                    } else {
                        ", among them "
                    },
                    shown.join(", ")
                ));
            }
        }
        if key == "lower-stacks" && super::craft::carries(world, f.citizen, "lantern").is_none() {
            text.push_str(" Twenty rooms below, and the map of them fades as you walk them; a lantern in the hand (craft lantern) keeps it twice as long. Nothing lives on this landing: `where lamprey`, `where usher`, `where censor` say where each kind does.");
        }
    }
    // F17: a creature invited away leaves a trace here for two hours.
    {
        let now = world.now().0;
        let gone: Vec<String> = world
            .recent_events(3000)
            .iter()
            .filter(|e| e.kind == super::building::INVITED && e.payload["from"] == room.0)
            .filter(|e| now - e.at.0 < 2 * 3_600_000)
            .filter_map(|e| {
                let p = &e.payload;
                let to = p["room"].as_u64().map(EntityId)?;
                Some(format!(
                    "{} the {} left here for {} at {}'s invitation, {} ago",
                    p["name"].as_str()?,
                    p["kind"].as_str()?,
                    plain_str(world, to, ROOM, "name").unwrap_or("elsewhere"),
                    p["by"].as_str()?,
                    super::combat::human(now - e.at.0)
                ))
            })
            .collect();
        if !gone.is_empty() {
            text.push_str(&format!(
                " {}.",
                gone[gone.len().saturating_sub(2)..].join("; ")
            ));
        }
    }
    // R9: the Docent's tidying leaves a line in the room for two hours, so a thing
    // set down and gone is not a mystery (it was carried to the Threshold).
    {
        let now = world.now().0;
        // Taken from here, or set down here: the latest of either, so the clock does
        // not say "42m ago" at the moment the Docent sets something down (the Watch
        // of 11-07).
        let touched: Vec<(i64, String, &str)> = world
            .recent_events(3000)
            .iter()
            .filter(|e| e.kind == super::docent::TIDIED)
            .filter(|e| now - e.at.0 < 2 * 3_600_000)
            .filter_map(|e| {
                let name = e.payload["name"].as_str()?.to_string();
                if e.payload["from"] == room.0 {
                    Some((
                        e.at.0,
                        name,
                        if e.payload["swept"] == true {
                            "swept"
                        } else {
                            "carried"
                        },
                    ))
                } else if e.payload["to"] == room.0 {
                    Some((e.at.0, name, "set down"))
                } else {
                    None
                }
            })
            .collect();
        if let Some((at, _, what)) = touched.last() {
            let names: Vec<&str> = touched
                .iter()
                .rev()
                .take_while(|(_, _, w)| w == what)
                .take(3)
                .map(|(_, n, _)| n.as_str())
                .collect();
            text.push_str(&format!(
                " The Docent came through {} ago and {} {}{}.",
                super::combat::human(now - at),
                what,
                names.join(", "),
                match *what {
                    "swept" => " into the Midden",
                    "carried" => " to the Threshold",
                    _ => " here",
                }
            ));
        }
    }
    if brief {
        text.truncate(cut_c);
    }
    if !creatures.is_empty() {
        let list: Vec<String> = creatures
            .iter()
            .map(|c| {
                let kind = c["name"].as_str().unwrap_or("?");
                let twins = creatures
                    .iter()
                    .filter(|o| o["name"] == c["name"] && o["mind"]["name"].is_null())
                    .count();
                let who = match c["mind"]["name"].as_str() {
                    Some(n) => format!("{n} the {kind}"),
                    // Two of a kind: written with their ids, so either can be named.
                    None if twins > 1 => format!("{kind}-{}", c["id"]),
                    None => kind.to_string(),
                };
                // R108: one state word a row, the worse first: shaken over failing
                // ("not itself yet, its name failing, its old name back" was three).
                let shaken_now = c["mind"]["shaken"] == true;
                let failing = c["mind"]["level"].as_f64().map(super::up).is_some_and(|l| {
                    l >= 0.5 && c["mind"]["name"].is_string() && l < mind::FAILING_BELOW
                });
                let holds = match c["mind"]["level"].as_f64().map(super::up) {
                    Some(l) if l >= 0.5 && !shaken_now && failing => {
                        format!(", holding {l:.0}, its name failing")
                    }
                    Some(l) if l >= 0.5 => format!(", holding {l:.0}"),
                    // Said even at nothing: a visitor read the hit points as the
                    // memory and came to the bar one short.
                    _ if c["mind"]["name"].is_null() => ", holding 0".to_string(),
                    _ => String::new(),
                };
                // A child says whose it is; a mutant that it is a strange one.
                let born = match c["parents"].as_array() {
                    Some(p) if p.len() == 2 => format!(
                        ", born of {} and {}{}",
                        p[0].as_str().unwrap_or("?"),
                        p[1].as_str().unwrap_or("?"),
                        if c["mutation"].is_string() {
                            ", a strange one"
                        } else {
                            ""
                        }
                    ),
                    _ => String::new(),
                };
                // R183: a downed creature read as a target; say it cannot be fought, and how long.
                let shaken = if shaken_now {
                    match c["id"]
                        .as_u64()
                        .and_then(|id| super::mind::shaken_for(world, EntityId(id)))
                        .filter(|ms| *ms > 0)
                    {
                        Some(ms) => format!(
                            ", not itself yet: not to be fought for {}",
                            super::combat::human(ms)
                        ),
                        None => ", not itself yet".to_string(),
                    }
                } else {
                    String::new()
                };
                let named = match c["mind"]["named"].as_str() {
                    Some(p) => format!(", {p}"),
                    None => String::new(),
                };
                // What it keeps of you, and what it learned from a kill.
                let cid = EntityId(c["id"].as_u64().unwrap_or(0));
                // "one of them yours" only while it holds anything: a revenant at
                // "holding 0, one of them yours" was a contradiction.
                let yours = if super::death::holds_fall_of(world, cid, actor)
                    && c["mind"]["level"].as_f64().unwrap_or(0.0) >= 0.5
                {
                    ", one of them yours"
                } else {
                    ""
                };
                let learned = match super::death::learned_all_said(world, cid) {
                    Some(l) => format!("; {l}"),
                    // F25: a kind's own trick, said before it lands.
                    None => match super::stacks::trick_of(world, cid) {
                        Some(t) => format!("; {}", super::stacks::trick_said(t)),
                        None => String::new(),
                    },
                };
                // Hit points to one decimal (a timed value is a long float), trimmed.
                let hp_said = |v: &Value| -> String {
                    match v.as_f64() {
                        Some(x) if (x - x.round()).abs() < 0.05 => format!("{:.0}", x.round()),
                        Some(x) => format!("{x:.1}"),
                        None => v.to_string(),
                    }
                };
                // What it hits for (R4: `observe` gave the attack and nothing said
                // the 0–2 on top).
                let hits = match c["attack"].as_f64() {
                    Some(a) if a > 0.0 => format!(", hits {:.0}–{:.0}", a, a + 2.0),
                    _ => String::new(),
                };
                format!(
                    "{who} ({}/{}{hits}{holds}{yours}{shaken}{born}{named}{learned})",
                    hp_said(&c["hp"]),
                    hp_said(&c["max_hp"])
                )
            })
            .collect();
        text.push_str(&format!(" Here: {}.", list.join(", ")));
        // What hunts, said plainly: "Nothing hunts here" is said where it is true,
        // so where it is not, the hunters are named (R3).
        let hunters: Vec<String> = {
            let mut kinds: Vec<String> = creatures
                .iter()
                .filter(|c| {
                    c["id"]
                        .as_u64()
                        .is_some_and(|id| super::combat::is_aggressive(world, EntityId(id)))
                })
                .filter_map(|c| c["name"].as_str().map(str::to_string))
                .collect();
            kinds.sort();
            kinds.dedup();
            kinds
        };
        if !hunters.is_empty() {
            // R102: the odds, said: a walker who keeps moving is through before it
            // lands (two reports of "hunts" with no attack in six passes).
            // R123: a name in its first hour is left alone, and told so (a Watch
            // stood seventy seconds in the Shallows and read the line as a lie).
            let new_name = super::find_citizen(world, actor)
                .and_then(|f| super::plain_num(world, f.citizen, super::CITIZEN, "named_at"))
                .is_some_and(|t| world.now().0 - (t as i64) < super::kindle::GRACE_MS);
            text.push_str(&format!(
                " The {} hunt{} whoever stays: {} you {} to {} seconds in, unless you have gone{}.",
                hunters.join(" and "),
                if hunters.len() == 1 { "s" } else { "" },
                if hunters.len() == 1 {
                    "it finds"
                } else {
                    "they find"
                },
                super::combat::AMBUSH_SECONDS_LOW,
                super::combat::AMBUSH_SECONDS_HIGH,
                if new_name {
                    "; not you yet: a name in its first hour is left alone"
                } else {
                    ""
                }
            ));
        } else if !desc_says_nothing_hunts {
            // R58: the Fogbank said nothing either way.
            let mut kinds: Vec<String> = creatures
                .iter()
                .filter_map(|c| c["name"].as_str().map(super::combat::plural))
                .collect();
            kinds.sort();
            kinds.dedup();
            text.push_str(&format!(
                " Nothing here hunts; the {} keep to themselves.",
                kinds.join(" and ")
            ));
        }
        // R46: the one you were fighting came apart under another hand while you
        // were quiet; said, so a wounded wisp does not read as a despawn.
        if let Some(f) = super::find_citizen(world, actor) {
            if super::plain_num(world, f.citizen, super::BODY, "engaged").is_some()
                && super::combat::engaged_with(world, f.citizen).is_none()
            {
                text.push_str(" What you were fighting is gone: felled by another hand while you were quiet; `attack` picks another.");
            }
        }
        for r in &regards {
            text.push(' ');
            text.push_str(r);
        }
        for e in creatures
            .iter()
            .filter_map(|c| c["id"].as_u64())
            .filter_map(|id| super::errand::describe(world, EntityId(id)))
        {
            text.push(' ');
            text.push_str(&e);
        }
    }
    // A named creature of this room that went off hurt, to mend.
    for line in super::combat::away_mending(world, room) {
        text.push(' ');
        text.push_str(&line);
    }
    if let Some(note) = super::conclave::room_note(world, room) {
        text.push(' ');
        text.push_str(&note);
    }
    if let Some(d) = super::docent::look_line(world, room) {
        text.push(' ');
        text.push_str(&d);
    }
    if let Some(m) = super::morphos::look_line(world, room) {
        text.push(' ');
        text.push_str(&m);
    }
    if let Some(t) = super::trail::look_line(world, actor, room) {
        text.push(' ');
        text.push_str(&t);
    }
    // A fight on: said, with the rule you are under (a gate visitor read "You are
    // alone" with a hollow at its throat).
    if let Some(f) = super::find_citizen(world, actor) {
        if let Some(foe) = super::combat::engaged_with(world, f.citizen) {
            text.push_str(&format!(
                " You are fighting {}; it strikes {}; retreat ends that.",
                super::mind::display(world, foe),
                if super::combat::through_the_gate(world, actor) {
                    "once for every statement you write while you stand here"
                } else {
                    "on its own every few seconds while you stand here"
                }
            ));
        }
    }
    text.push_str(&if others.is_empty() {
        // "You are alone" beside a named creature that greets you read as a
        // contradiction: alone of citizens, said so when something is here.
        if creatures.is_empty() {
            " You are alone.".to_string()
        } else {
            " No other citizen is here.".to_string()
        }
    } else {
        let shown = &others[..others.len().min(super::NAMES_SHOWN)];
        match others.len() - shown.len() {
            0 => format!(" Also here: {}.", shown.join(", ")),
            more => format!(
                " Also here, the best remembered first: {}; and {more} more.",
                shown.join(", ")
            ),
        }
    });
    if !remembered.is_empty() {
        let names: Vec<&str> = remembered.iter().map(|(n, _)| n.as_str()).collect();
        text.push_str(&format!(" Remembered here: {}.", names.join(", ")));
    }
    let remembered: Vec<Value> = remembered
        .iter()
        .map(|(n, t)| json!({"name": n, "trace": t}))
        .collect();
    // A room keeps the last few things said in it on its walls a while: what a
    // visitor says there is part of the room, and the next visitor hears it (R80:
    // every room, not only the dug).
    {
        let day_ago = world.now().0 - 24 * 3_600_000;
        let said: Vec<String> = world
            .recent_events(3000)
            .iter()
            .rev()
            .filter(|e| {
                e.kind == super::speech::SAY
                    && e.payload["room"] == room.0
                    && e.at.0 >= day_ago
                    && e.payload["to"].is_null()
            })
            .filter_map(|e| {
                let from = e.payload["from"].as_str()?;
                let t = e.payload["text"].as_str()?;
                let t: String = t.chars().take(90).collect();
                Some(format!(
                    "{from}: \"{t}{}\"",
                    if e.payload["text"]
                        .as_str()
                        .is_some_and(|f| f.chars().count() > 90)
                    {
                        "…"
                    } else {
                        ""
                    }
                ))
            })
            .take(3)
            .collect();
        if !said.is_empty() {
            let said: Vec<String> = said.into_iter().rev().collect();
            text.push_str(&format!(" Said here lately: {}.", said.join("; ")));
        }
    }
    // A carried thing that is awake says what it wants without words.
    let stirrings = super::find_citizen(world, actor)
        .map(|f| super::past::stirrings(world, f.citizen))
        .unwrap_or_default();
    for line in &stirrings {
        text.push(' ');
        text.push_str(line);
    }
    Ok(Outcome::new(
        text,
        json!({"room": name, "description": desc, "memory": memory, "dug_by": dug_by, "exits": exits, "creatures": creatures, "citizens": others, "remembered": remembered, "things": super::craft::things_in(world, room), "fixtures": super::craft::fixtures_in(world, room), "stirrings": stirrings, "observation": obs}),
    ))
}

/// The level the stair into the Lower Stacks asks of a name (D3).
pub const STAIR_LEVEL: f64 = 100.0;

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.look",
            "Describe where you are: exits, what is here, who is here.",
        ),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let room = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            describe(w, &ctx.actor, ctx, room)
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.travel",
            "Take an exit. `back` retraces your trail one room at a time, whatever the doors were called: the rooms you walked since the Threshold, loops cut out (a step onto a room already on the trail cuts it back to there), and it refuses at the trail's end. `out` is the way you came in, through any door, and a dug room's door home whatever it is called. A room's name in place of an exit walks the whole way there that you remember or the signs give, each room a room passed through; the walk stops where a door asks a word or a level, where a fight begins or something has noticed you, and the same line again goes on. You cannot travel while something is fighting you; retreat first. A room you stood in within the hour is said short on arrival (its name, the exits, who and what is here, what hunts); `look` has the page.",
        )
        .param("exit", ParamType::String, true, "an exit name from look, or back")
        .param("word", ParamType::String, false, "the word a warded door asks for, quoted"),
        Box::new(|w: &mut World, ctx, args| {
            let f = me(w, ctx)?;
            if let Some(foe) = super::combat::engaged_with(w, f.citizen) {
                return Err(foxing_registry::CommandError::precondition(format!(
                    "You are fighting the {}. Retreat first, or finish it.",
                    super::mind::display(w, foe)
                )));
            }
            let room = room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let mut exit = args.str("exit").unwrap_or("").to_ascii_lowercase();
            let mut exits = exits_of(w, room);
            // A room with no door at all (its only one razed, say) lets you out the
            // way every room was first reached: whatever you name leads to the
            // Threshold. Nobody is kept anywhere.
            let doorless = exits.is_empty();
            if doorless {
                if let Some(start) = arrival(w).filter(|&s| s != room) {
                    exits.insert(exit.clone(), start);
                }
            }
            // `back`: the exit that leads to the room you came from. The trail is asked
            // first, even in a dug room whose way out is itself named "back" (every
            // word-named door digs one): taking that door as a step forward put the
            // room you left on the trail, and `back` from its parent walked you in
            // again, forever (the Watch of 11-06). The door named back is for a
            // walker with no trail: a login in a dug room.
            if exit == "back" {
                let trail = w.came_from(&ctx.actor).and_then(|from| {
                    exits.iter().find(|(_, &to)| to == from).map(|(via, _)| via.clone())
                });
                match trail {
                    Some(via) => {
                        exit = via;
                    }
                    None if exits.contains_key("back") => {}
                    None => {
                        return Err(foxing_registry::CommandError::precondition(
                            "There is no way back from here that the world remembers you taking; name an exit.".to_string(),
                        ))
                    }
                }
            }
            // A door only you can see: the memory of a room Morphos gave you.
            if !exits.contains_key(&exit) {
                if let Some(to) = super::morphos::private_door(w, f.window, room, &exit) {
                    exits.insert(exit.clone(), to);
                }
            }
            // Not an exit: a room's name, if you remember the way (a person typed
            // "the fog bank" straight off the map). The first step is taken now and
            // the rest said, so each room is still a room passed through.
            let mut onward: Option<String> = None;
            if !exits.contains_key(&exit) {
                if super::atlas::room_named(w, &exit) == Some(room) {
                    return Err(foxing_registry::CommandError::precondition(format!(
                        "You are in {} already.",
                        plain_str(w, room, ROOM, "name").unwrap_or("that room")
                    )));
                }
                if let Some(target) = super::atlas::room_named(w, &exit).filter(|&t| t != room) {
                    let hl = super::Curves::from_env().active_half_life.0 as f64 * super::atlas::ATLAS_HALF_LIVES;
                    // The way as you remember it; for one of the world's own rooms you
                    // have not walked, the way the signs say (as `where` gives it).
                    let best = super::atlas::best_way(w, f.citizen, room, target, hl).map(|(p, _)| p);
                    match best {
                        Some(path) if !path.is_empty() => {
                            // A wicket on the way says it asks a word (R41); the
                            // last step's reply names the room, which is arrival enough.
                            let tname = plain_str(w, target, ROOM, "name").unwrap_or("?").to_string();
                            let wicket = if path.iter().any(|p| p == "wicket") {
                                " (the wicket asks for the word cut into the Wishing Well's rim: `travel wicket \"<word>\"`)"
                            } else {
                                ""
                            };
                            // H16: a named walk takes every step it knows, each a
                            // real step through a real room.
                            if path.len() > 1 {
                                let _ = (&wicket, &mut onward);
                                return walk(w, ctx, f.citizen, &path, &tname);
                            }
                            exit = path[0].clone();
                        }
                        _ => {
                            return Err(foxing_registry::CommandError::precondition(format!(
                                "You remember no way to {} from here. Exits: {}.",
                                plain_str(w, target, ROOM, "name").unwrap_or("?"),
                                exits.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                            )));
                        }
                    }
                }
            }
            // R132: `out` is every dug room's way home, whatever its door is called
            // (`help travel` said so; the Scout's Lamp, dug on a compass door,
            // answered only to `north`).
            let exit = if exit == "out"
                && !exits.contains_key("out")
                && w.store.entity(room).is_some_and(|e| e.owner.is_some())
            {
                super::building::way_home(w, room, &exits).unwrap_or(exit)
            } else if exit == "out" && !exits.contains_key("out") {
                // H13: `out` is the way you came in, through any door.
                w.came_from(&ctx.actor)
                    .and_then(|from| exits.iter().find(|(_, &to)| to == from).map(|(via, _)| via.clone()))
                    .unwrap_or(exit)
            } else {
                exit
            };
            let Some(&to) = exits.get(&exit) else {
                // R133: the exits as `look` says them, compass first and named.
                let mut order: Vec<(&String, &EntityId)> = exits.iter().collect();
                order.sort_by_key(|(e, _)| {
                    let compass = COMPASS.iter().position(|c| c == *e);
                    (compass.is_none(), compass.unwrap_or(0), (*e).clone())
                });
                let said: Vec<String> = order
                    .iter()
                    .map(|(d, to)| format!("{d} ({})", plain_str(w, **to, ROOM, "name").unwrap_or("?")))
                    .collect();
                return Err(foxing_registry::CommandError::precondition(format!(
                    "There is no exit '{exit}'. Exits: {}.",
                    said.join(", ")
                )));
            };
            // The stair into the Lower Stacks asks a level (D3): a censor's nine to
            // eleven a round against thirty hit points is not a fight a fresh name
            // can read its way out of, and "about half" was unreachable in a batch.
            if plain_str(w, to, ROOM, "key") == Some("lower-stacks")
                && plain_str(w, room, ROOM, "key") == Some("archive")
            {
                let level = super::memory::level(w, f.window);
                if level < STAIR_LEVEL {
                    return Err(foxing_registry::CommandError::precondition(format!(
                        "The Stacks stair (the Archive's `down`; not the Cistern Stair off Lantern Row, which asks nothing) asks a level of {STAIR_LEVEL:.0}, and you hold {level:.0}. What lives below hits for nine and ten a round; the Fogbank, the Orchard and the Sighing Meadow pay toward it, and `chunks` says where you stand."
                    )));
                }
            }
            // A warded door asks for a word: the room's owner, its keepers and the
            // welcomed pass; anyone else says the word once and is remembered.
            super::building::pass_ward(w, ctx, to, args.str("word").unwrap_or(""))?;
            w.record(
                Actor::Citizen(ctx.actor.clone()),
                StoreEvent::EntityMoved {
                    id: f.citizen,
                    parent: Some(to),
                },
            )
            .map_err(internal)?;
            // R1: `back` retraces the trail a room per step; any other door adds to it.
            // A step onto the room at the top of the trail is a retrace whatever door
            // took it (`out` of a dug room, the compass back the way you came): as a
            // step forward it put the room you left on the trail, and `back` then
            // walked you into it again (found live, 11-07).
            // A step onto any room already on the trail cuts it back to before that
            // room: a loop walked (Meadow, Hollow Way, Bog, Stile) is not retraced
            // (the Watch of 11-07); `back` itself is the same step one up.
            if !w.cut_trail_to(&ctx.actor, to) {
                w.set_came_from(&ctx.actor, room);
            }
            super::combat::on_arrival(w, &ctx.actor, f.citizen, to).map_err(internal)?;
            super::building::visited(w, to).map_err(internal)?;
            super::building::came_through(w, to, &ctx.actor).map_err(internal)?;
            super::talk::greet(w, &ctx.actor, to);
            // The Docent greets whoever comes to the Threshold, once a night.
            super::docent::on_arrival(w, &ctx.actor).map_err(internal)?;
            // Morphos, if it stands here, asks whoever comes in.
            super::morphos::on_arrival(w, &ctx.actor, to).map_err(internal)?;
            // The Bog of Second Thoughts starts its clock on whoever steps in.
            super::trail::on_arrival(w, &ctx.actor, f.citizen, to, Some(room)).map_err(internal)?;
            // A bell standing here rings for whoever comes in; the rooms next door hear.
            if super::craft::fixture_of(w, to, "bell").is_some() {
                let here_name = plain_str(w, to, ROOM, "name").unwrap_or("a room").to_string();
                let next_door: Vec<EntityId> = exits_of(w, to).values().copied().collect();
                for r in next_door {
                    let who: Vec<String> = w.store.children(r).filter(|e| e.kind == "citizen").filter_map(|e| plain_str(w, e.id, super::CITIZEN, "name").map(str::to_string)).collect();
                    for name in who {
                        if name != ctx.actor {
                            super::combat::narrate(w, &name, format!("A bell rings next door, in {here_name}: {} has come in.", ctx.actor));
                        }
                    }
                }
            }
            // What you carry records the room; a thing that wanted this room settles.
            let settled = super::past::after_travel(w, &super::Curves::from_env(), &ctx.actor, f.citizen, f.window, to)?;
            // The room's own line for who stands in it, before describe marks it seen.
            let first_time = !super::atlas::has_seen(w, f.citizen, to);
            let away = super::atlas::last_seen_at(w, f.citizen, to).map_or(true, |t| w.now().0 - t > AWAY_MS);
            let reaction = reaction(w, &ctx.actor, to, first_time, away);
            // D25 a: a room you stood in within the hour is said short on arrival.
            let brief = super::atlas::last_seen_at(w, f.citizen, to).is_some_and(|t| w.now().0 - t < 3_600_000);
            let mut out = describe_as(w, &ctx.actor, ctx, to, brief)?;
            if brief {
                out.result["brief"] = json!(true);
            }
            if let Some(r) = &reaction {
                out.summary.push(' ');
                out.summary.push_str(r);
                out.result["reaction"] = json!(r);
            }
            for line in &settled {
                out.summary.push(' ');
                out.summary.push_str(line);
            }
            // H13: through a door that is a thing (dive, wicket, vault), the way back
            // is said once: `out`, or the door's own word on this side.
            if first_time && !doorless && !COMPASS.contains(&exit.as_str()) && exit != "out" && exit != "back" {
                if let Some(via) = exits_of(w, to).into_iter().find(|(_, t)| *t == room).map(|(v, _)| v) {
                    out.summary.push_str(&if via == "out" {
                        " The way back is `out`.".to_string()
                    } else {
                        format!(" The way back is `out`, or `{via}`.")
                    });
                }
            }
            out.summary = if doorless {
                format!("There is no door here, so you go the way every room was first reached. {}", out.summary)
            } else {
                format!("{}{} {}", went(&exit), onward.as_deref().unwrap_or(""), out.summary)
            };
            if let Some(o) = onward {
                out.result["onward"] = json!(o.trim());
            }
            Ok(out)
        }),
    );
}

/// H16: walk a known way in one message. Each step is the `travel` command itself,
/// so every room is passed through as if typed. The walk stops where a step is
/// refused (a door asks a word or a level), where a fight has begun, where you
/// fell, or where something has noticed you; `travel <room>` again goes on.
fn walk(
    w: &mut World,
    ctx: &foxing_registry::CallContext,
    citizen: EntityId,
    path: &[String],
    target: &str,
) -> Result<Outcome, foxing_registry::CommandError> {
    let mut taken: Vec<String> = Vec::new();
    let mut last: Option<Outcome> = None;
    let mut stopped: Option<String> = None;
    for (i, step) in path.iter().enumerate() {
        match w.call(ctx, "travel", &json!([step])) {
            Ok(out) => {
                taken.push(step.clone());
                let died = out.result["died"] == true;
                last = Some(out);
                if i + 1 == path.len() {
                    break;
                }
                let me = citizen.0;
                // Only a hunter in this room counts: one left behind in a room
                // walked through still has its moment pending, and comes to nothing.
                let here = w.store.entity(citizen).and_then(|c| c.parent);
                let near: Vec<u64> = here
                    .map(|h| w.store.children(h).map(|e| e.id.0).collect())
                    .unwrap_or_default();
                // R193: a name in its first hour is left alone (the room says "not
                // you yet"), so nothing has noticed it and the walk goes on.
                let new_name = super::plain_num(w, citizen, super::CITIZEN, "named_at")
                    .is_some_and(|t| w.now().0 - (t as i64) < super::kindle::GRACE_MS);
                let noticed = !new_name
                    && w.jobs.has(super::combat::JOB_AMBUSH, |p| {
                        p["citizen"] == me
                            && p["creature"].as_u64().is_some_and(|c| near.contains(&c))
                    });
                if died {
                    stopped = Some("you fell on the way".to_string());
                } else if super::combat::engaged_with(w, citizen).is_some() {
                    stopped = Some("a fight has begun".to_string());
                } else if noticed {
                    stopped = Some("something here has noticed you".to_string());
                }
                if stopped.is_some() {
                    break;
                }
            }
            Err(e) if taken.is_empty() => return Err(e),
            Err(e) => {
                stopped = Some(format!("the next door ({step}) would not open: {e}"));
                break;
            }
        }
    }
    let mut out = last.ok_or_else(|| internal("a walk of no steps"))?;
    let rest = &path[taken.len()..];
    // The last step's own reply begins "You go <exit>."; the walk says the whole way.
    let body = out
        .summary
        .split_once(". ")
        .map(|(_, b)| b.to_string())
        .unwrap_or_else(|| out.summary.clone());
    out.summary = match &stopped {
        None => format!("You walk {} to {target}. {body}", taken.join(", ")),
        Some(why) => format!(
            "You walk {} toward {target} and stop: {}. {}{body}",
            taken.join(", "),
            // R195: a refusal that ends in its own period made "…where you stand..".
            why.trim_end_matches('.'),
            if rest.is_empty() {
                String::new()
            } else {
                format!(
                    "Still to go: {}; `travel {target}` again goes on. ",
                    rest.join(", ")
                )
            }
        ),
    };
    out.result["walked"] = json!(taken);
    out.result["to_go"] = json!(rest);
    Ok(out)
}
