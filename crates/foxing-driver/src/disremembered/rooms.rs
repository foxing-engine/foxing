//! The map: fourteen rooms off the Threshold, twenty below the Archive, eleven up
//! inside the Clocktower's works, exits between them, and what spawns where.
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

pub const ROOMS: &[RoomDef] = &[
    RoomDef {
        key: ARRIVAL,
        name: "The Threshold",
        description: "A threshold of pale stone under a moon that is always up. Everything you earn here will fade; what you do with it is yours. Nothing hunts here.",
        exits: &[("east", "fogbank"), ("north", "lantern-row")],
        spawn: None,
        former: &["gate"],
        tint: "#d9d4c7",
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
        exits: &[("south", ARRIVAL), ("north", "archive"), ("down", "stairwell")],
        spawn: None,
        former: &[],
        tint: "#c9a86a",
    },
    RoomDef {
        key: "archive",
        name: "The Unfiled Archive",
        description: "Shelves to the ceiling and ledgers on every one, but the ink lifts off the page the moment you read it and drifts for the rafters. Some of it has learned to flutter.",
        exits: &[("south", "lantern-row"), ("down", "lower-stacks")],
        spawn: Some(("inkling", 3, 45)),
        former: &[],
        tint: "#4a4e6b",
    },
    RoomDef {
        key: "stairwell",
        name: "The Downward Stair",
        description: "A spiral stair cut into the rock, going down further than the hill is tall. Every landing has a door, and every door is bricked up from the other side.",
        exits: &[("up", "lantern-row"), ("down", "cistern")],
        spawn: None,
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
        exits: &[("south", "shallows"), ("up", "clocktower")],
        spawn: Some(("mothwing", 3, 40)),
        former: &[],
        tint: "#7d9a5a",
    },
    RoomDef {
        key: "clocktower",
        name: "The Stopped Clocktower",
        description: "Four clock faces, one to each wall, each showing a different hour, and all of them wrong. The works fill the middle of the room, and something in the works is still wound.",
        exits: &[("down", "orchard"), ("escapement", "escapement-stair")],
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
        spawn: None,
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
        spawn: None,
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
        spawn: None,
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
        spawn: None,
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
        description: "A brass tube comes down through the ceiling from the Clocktower and ends over a slanted desk. Once an hour a curl of paper drops out of it, figures on one side, and it is weighed on a small scale before anyone reads it. The scale is kept very clean.",
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
];

/// How leaving reads: "You go east." for a compass word, "You go back.", and "You
/// take the orrery." for a door with a name of its own ("You go orrery." was wrong).
pub fn went(exit: &str) -> String {
    match exit {
        "north" | "south" | "east" | "west" | "up" | "down" | "back" | "in" | "out" => {
            format!("You go {exit}.")
        }
        _ => format!("You take the {exit}."),
    }
}

/// The room entity with this key.
pub fn room_by_key(world: &World, key: &str) -> Option<EntityId> {
    world
        .store
        .entities()
        .find(|e| e.kind == "room" && plain_str(world, e.id, ROOM, "key") == Some(key))
        .map(|e| e.id)
}

/// The arrival room.
pub fn arrival(world: &World) -> Option<EntityId> {
    room_by_key(world, ARRIVAL)
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
                    .entities()
                    .find(|e| {
                        e.kind == "room"
                            && plain_str(world, e.id, ROOM, "key").is_none()
                            && plain_str(world, e.id, ROOM, "name") == Some(def.name)
                    })
                    .map(|e| e.id)
            });
        let id = match existing {
            Some(id) if plain_str(world, id, ROOM, "key") == Some(def.key) => id,
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
                COMPASS.contains(&name.as_str())
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
        "select": {ROOM: ["name", "description", "memory", "dug_with", "kept", "bound"], EXITS: "*"},
        "relations": {
            "contents": {
                "kinds": ["citizen", "creature"],
                "select": {super::CITIZEN: ["name"], super::CREATURE: ["name", "hp", "max_hp", "parents", "born_of", "mutation"], super::mind::MIND: ["name", "tier"]}
            }
        }
    })
}

/// Describe a room as `actor` sees it. Runs the observation query and renders prose
/// from the structure, which is the text client's job: the driver sends components,
/// the client makes sentences.
pub fn describe(
    world: &mut World,
    actor: &str,
    ctx: &foxing_registry::CallContext,
    room: EntityId,
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
    let desc = super::building::remembered_description(
        root["components"][ROOM]["description"]
            .as_str()
            .unwrap_or(""),
        memory,
    );
    let desc = if desc.ends_with(['.', '!', '?', '"', '”', ')']) || desc.is_empty() {
        desc
    } else {
        format!("{desc}.")
    };
    let dug_by = root["owner"].as_str().map(str::to_string);
    let exits: Vec<String> = root["components"][EXITS]["exits"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
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
            let mut v =
                json!({"id": n["id"], "name": c["name"], "hp": c["hp"], "max_hp": c["max_hp"]});
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
    let mut text = format!("{name}. {desc}");
    if let Some(who) = &dug_by {
        let with = root["components"][ROOM]["dug_with"]
            .as_str()
            .map(|s| format!(" with the memory of {s}"))
            .unwrap_or_default();
        text.push_str(&format!(" Dug by {who}{with}."));
        if let Some(kept) = root["components"][ROOM]["kept"].as_array() {
            let keepers: Vec<String> = kept
                .iter()
                .filter_map(|k| {
                    Some(format!(
                        "{} ({})",
                        k["by"].as_str()?,
                        k["with"].as_str().unwrap_or("something")
                    ))
                })
                .collect();
            if !keepers.is_empty() {
                text.push_str(&format!(" Kept in memory by {}.", keepers.join(", ")));
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
    if !exits.is_empty() {
        // Each exit says where it leads: three visitors in one night asked, because a
        // dug exit is named by its digger and tells a stranger nothing.
        let by_name = exits_of(world, room);
        let shown: Vec<String> = exits
            .iter()
            .map(|e| {
                match by_name
                    .get(e)
                    .and_then(|&to| super::plain_str(world, to, ROOM, "name"))
                {
                    Some(n) => format!("{e} ({})", n.trim_start_matches("The ")),
                    None => e.clone(),
                }
            })
            .collect();
        text.push_str(&format!(" Exits: {}.", shown.join(", ")));
    }
    if let Some(standing) = super::craft::fixtures_line(world, room) {
        text.push(' ');
        text.push_str(&standing);
    }
    if let Some(ground) = super::craft::ground_line(world, room) {
        text.push(' ');
        text.push_str(&ground);
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
                let holds = match c["mind"]["level"].as_f64() {
                    Some(l)
                        if l >= 0.5 && c["mind"]["name"].is_string() && l < mind::FAILING_BELOW =>
                    {
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
                let shaken = if c["mind"]["shaken"] == true {
                    ", not itself yet"
                } else {
                    ""
                };
                // What it keeps of you, and what it learned from a kill.
                let cid = EntityId(c["id"].as_u64().unwrap_or(0));
                let yours = if super::death::holds_fall_of(world, cid, actor) {
                    ", one of them yours"
                } else {
                    ""
                };
                let learned = match super::death::learned_all_said(world, cid) {
                    Some(l) => format!("; {l}"),
                    None => String::new(),
                };
                format!(
                    "{who} ({}/{}{holds}{yours}{shaken}{born}{learned})",
                    c["hp"], c["max_hp"]
                )
            })
            .collect();
        text.push_str(&format!(" Here: {}.", list.join(", ")));
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
    if let Some(note) = super::conclave::room_note(world, room) {
        text.push(' ');
        text.push_str(&note);
    }
    text.push_str(&if others.is_empty() {
        " You are alone.".to_string()
    } else {
        format!(" Also here: {}.", others.join(", "))
    });
    if !remembered.is_empty() {
        let names: Vec<&str> = remembered.iter().map(|(n, _)| n.as_str()).collect();
        text.push_str(&format!(" Remembered here: {}.", names.join(", ")));
    }
    let remembered: Vec<Value> = remembered
        .iter()
        .map(|(n, t)| json!({"name": n, "trace": t}))
        .collect();
    Ok(Outcome::new(
        text,
        json!({"room": name, "description": desc, "memory": memory, "dug_by": dug_by, "exits": exits, "creatures": creatures, "citizens": others, "remembered": remembered, "things": super::craft::things_in(world, room), "fixtures": super::craft::fixtures_in(world, room), "observation": obs}),
    ))
}

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
            "Take an exit. `back` is the way you came, whatever the exit is called. You cannot travel while something is fighting you; retreat first.",
        )
        .param("exit", ParamType::String, true, "an exit name from look, or back"),
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
            // `back`: the exit that leads to the room you came from, unless the room has
            // an exit named back of its own (diggers name exits as they like).
            if exit == "back" && !exits.contains_key("back") {
                match w.came_from(&ctx.actor).and_then(|from| {
                    exits.iter().find(|(_, &to)| to == from).map(|(via, _)| via.clone())
                }) {
                    Some(via) => exit = via,
                    None => {
                        return Err(foxing_registry::CommandError::precondition(
                            "There is no way back from here that the world remembers you taking; name an exit.".to_string(),
                        ))
                    }
                }
            }
            // Not an exit: a room's name, if you remember the way (a person typed
            // "the fog bank" straight off the map). The first step is taken now and
            // the rest said, so each room is still a room passed through.
            let mut onward: Option<String> = None;
            if !exits.contains_key(&exit) {
                if let Some(target) = super::atlas::room_named(w, &exit).filter(|&t| t != room) {
                    let hl = super::Curves::from_env().active_half_life.0 as f64 * super::atlas::ATLAS_HALF_LIVES;
                    match super::atlas::way(w, f.citizen, room, target, hl) {
                        Some(path) if !path.is_empty() => {
                            if path.len() > 1 {
                                onward = Some(format!(
                                    " Toward {}: then {}.",
                                    plain_str(w, target, ROOM, "name").unwrap_or("?"),
                                    path[1..].join(", ")
                                ));
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
            let Some(&to) = exits.get(&exit) else {
                let names: Vec<&String> = exits.keys().collect();
                return Err(foxing_registry::CommandError::precondition(format!(
                    "There is no exit '{exit}'. Exits: {}.",
                    names
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            };
            w.record(
                Actor::Citizen(ctx.actor.clone()),
                StoreEvent::EntityMoved {
                    id: f.citizen,
                    parent: Some(to),
                },
            )
            .map_err(internal)?;
            w.set_came_from(&ctx.actor, room);
            super::combat::on_arrival(w, &ctx.actor, f.citizen, to).map_err(internal)?;
            super::building::visited(w, to).map_err(internal)?;
            super::talk::greet(w, &ctx.actor, to);
            let mut out = describe(w, &ctx.actor, ctx, to)?;
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
