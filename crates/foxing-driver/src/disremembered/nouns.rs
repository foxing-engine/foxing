//! D41: things a room's text names can be read.
//!
//! The cohort of 11-16 paid a quarter of a message each for "There is no X here to
//! speak of" on the Chart Room's maps, the Pneumatic Room's canisters, the Reserve's
//! list and the Swollen Drawers' drawers. A room that names a thing answers `read`
//! for it: a line each, the wing's and the Stacks' first. Nothing here pays or opens
//! anything; the lines say only what is so elsewhere in the world.

use super::{plain_str, ROOM};
use crate::world::World;
use foxing_registry::Outcome;
use foxing_store::EntityId;
use serde_json::json;

/// `(room key, the words that name the thing, what reading it gives)`. `{you}` is
/// the reader's name.
const NOUNS: &[(&str, &[&str], &str)] = &[
    (
        "tube-room",
        &["canister", "canisters", "tube", "tubes", "message", "brass"],
        "You take a canister out of the nearest tube before it can go on. The paper inside is in your hand, the hand you write in, and it says: \"{you}: you will have forgotten sending this. It is still true. Go down.\" There is no date. The canister is warm, and the tube takes it back from you with a thump, and it goes on to whoever it was really for.",
    ),
    (
        "chart-room",
        &["map", "maps", "chart", "charts", "drawer", "drawers"],
        "The top drawer slides out flat. The map of this room has you in it, a little behind where you are, and the ink is wet. Under it is a map of the floors below the Undertow drawn as if they were dry: thirteen rooms, a light well with a small star in it, and at the bottom a closet with nothing drawn in it but a flame. Under that, one sheet almost blank, a road going off its edge, lettered THE ASHEN WASTES in a hand that pressed too hard. The hand that drew them is redrawing; none of these will be right tomorrow.",
    ),
    (
        "reserve",
        &["list", "title", "titles", "book", "books", "counter"],
        "The list is typed, with corrections in pencil. \"{you}: A First Hour.\" \"What {you} Fed, and What It Was Called After.\" \"Rooms Dug by {you}, with Their Present Condition.\" \"{you}, Fallen: A Concordance.\" \"What {you} Had Forgotten by Morning (in several volumes).\" Each has a shelf mark, and each shelf mark is struck through and marked REQUESTED. Somebody is reading you already. The books are asked for at the counter, and nobody is at the counter.",
    ),
    (
        "deep-reserve",
        &["entry", "catalogue", "catalog", "card", "ledger"],
        "The cataloguer does not cover the page. Your entry is a card like any other: {you}; where you came in; what you hold, to the point; and under USE, where every other card has a word, a blank. It has left the pen on the blank. It is waiting to see what you are for.",
    ),
    (
        "wing-catalogue",
        &["drawer", "drawers", "card", "cards", "floor", "felt"],
        "You get a thumbnail under one card and lift it off the felt of the others. The ink has gone the brown of weak tea, and it is a catalogue card still: a title you nearly know, a shelf mark, and stamped across it in violet, WITHDRAWN TO THE LOWEST SHELF. KEEP WET. The next card says the same of another book. So do the ten after it. Whoever went down with the water took the catalogue's best things with them, and filed the fact.",
    ),
    (
        "wing-pool",
        &["book", "books", "lamp", "lamps", "table", "tables", "pool"],
        "You turn one of the floating books face up. The pages have kept their words and let go of their order: it reads as well from the back. A small face comes up under it, reads two lines over your thumb with one eye and two with the other, and goes down again. It had been enjoying that one.",
    ),
    (
        "wing-kelp",
        &[
            "spine", "spines", "title", "titles", "book", "books", "kelp", "weed", "shelving",
            "shelves", "shelf", "strap", "straps",
        ],
        "You part the straps. The spines are all there, shut hard as mussels, and the kelp has grown over the gilt of each title in the shape of the letters, so that it reads them out when the water moves it: TIDES, A TABLE. ON THE KEEPING OF STARFISH IN SMALL JARS. THE SEWER BOARD: MINUTES. The last is a gap where a book was taken, lately; the weed has not learned what to say there yet.",
    ),
    (
        "wing-drip",
        &["portrait", "portraits", "face", "faces", "frame", "frames", "saucer", "saucers"],
        "Each frame has a brass plate, and the plates have lasted where the faces did not: Keeper of the Lowest Shelf. Chairman, Municipal Sewer Board. The Night Reader, by subscription. Under that last one somebody has scratched into the brass with a pin: asked to be painted with the jar, and was refused. The saucers under them are full of faces.",
    ),
    (
        "wing-binder",
        &["type", "trough", "pool", "tidepool", "folder", "folders", "press", "pots"],
        "The brass type on the floor of the pool is still set, a line of it, locked in its stick and read backwards as type is: IT WILL NOT BURN DRY. The binder was setting a label for a book's box when the water came, and the water has kept the job standing. The anemone has its foot on the full stop.",
    ),
    (
        "wing-scriptorium",
        &["desk", "desks", "inkwell", "inkwells", "stool", "stools", "newt", "newts"],
        "Each desk has a name cut into its lip where a wrist would rest, and none of the names is a newt's. The inkwell at the nearest is full to the brim with something that is mostly pond. The newt in it lets you look, then puts its foot in the ink and goes back to the page it was on, which is yours to read if you ask for the footprints.",
    ),
    (
        "wing-scriptorium",
        &["manuscript", "manuscripts", "page", "pages", "footprint", "footprints", "desk", "desks", "inkwell"],
        "The footprints are very nearly letters, and on one page, where a newt has walked the same line for years, they are letters: when the vault clicks the water has gone. You read it twice. It is the only true sentence in the room, and the newt that wrote it is watching you from the inkwell to see whether you are going to spoil it.",
    ),
    (
        "wing-long",
        &["minute", "minutes", "book", "agenda", "table", "limpet", "limpets", "cloth"],
        "The minute book is pulp, but the agenda under the limpets is card, and holds. Item 1, apologies: the Night Reader. Item 2: the damp. Item 3: the request of the Sewer Board for the return of its key. Item 4: the thing in the light well, which the Docent undertook would stay small. Item 5: any other business. Beside item 5, in pencil, in a hurry: the water.",
    ),
    (
        "wing-sump",
        &["stair", "stairs", "water", "wall", "walls"],
        "The stair is cut with the same dates the Clocktower's stair has, running the other way, and the last one above the water is tomorrow's. You count nine steps down before the black takes them. Something has worn the middle of each step hollow going down, and not coming up.",
    ),
    // R252, the batch of 11-17: the Stacks' ground floor and the Clocktower.
    (
        "archive",
        &["ledger", "ledgers", "shelf", "shelves", "book", "books", "ink", "page", "pages"],
        "You take a ledger down and open it flat. The first line holds for as long as you look at it: \"{you}, admitted.\" Then the ink lifts, a word at a time, and goes up past your face to the rafters, where a good deal of ink is already going round. What is left on the page is the ruling and one pencil note in the margin that ink cannot lift: told upstairs, not struck.",
    ),
    (
        "archive",
        &["stair", "stairs", "mezzanine", "rafters", "rafter"],
        "The iron stair has a plate on its first riser, and the plate is not ink, so it stays: READING ROOMS. CREATURES ARE TOLD HERE AND NOT STRUCK. Someone has scratched under it with a key: they listen better than they look.",
    ),
    (
        "lantern-row",
        &["lantern", "lanterns", "hook", "hooks", "row", "glass"],
        "Each lantern has a tag wired to its hook, and each tag has a column of dates in it, every one in a different hand, the newest still tacky. The nearest tag's last line is today's and the hand is yours. The lantern lights while you read and is dark again before you have let the tag go.",
    ),
    (
        "clocktower",
        &["clock", "clocks", "face", "faces", "hour", "hours", "dial", "dials"],
        "North says a quarter past four. East says noon, or midnight. South has lost its small hand and says only that it is ten to something. West is the nearest to right and is wrong by the length of your visit so far, which it was a moment ago as well.",
    ),
    (
        "clocktower",
        &["works", "work", "spring", "gear", "gears", "wheel", "wheels", "stair", "stairs", "date", "dates"],
        "The works are stopped and not run down: the spring is tight, the wheels are clean, and a little brass plate on the frame says WOUND, with a date under it that is today's. The stair round the works is cut with dates on every tread, going up in order, and the top one that can be read is yesterday's.",
    ),
    (
        "overdue-desk",
        &["ledger", "fine", "fines", "column", "book", "desk"],
        "The column is in one hand and it is patient. Most lines are names you do not know and sums that have been carried so often that the carrying is most of the sum. Near the foot: \"{you}: one memory, borrowed at the door, not yet returned. Accruing.\" The line under it is being written as you read, and the pen is not in the room.",
    ),
    (
        "overdue-desk",
        &["bell"],
        "The bell is engraved round its rim, small, where a hand would cover it: RING FOR ATTENTION. Under that, smaller: ATTENTION IS ALSO DUE. It rings while you are reading it, and the fines go up.",
    ),
    (
        "bindery",
        &["press", "presses", "book", "binding", "spine", "glue", "pot", "pots", "page", "pages"],
        "What is in the press has a spine and boards and no pages you would call pages: they are stitched in as signatures and they are warm. The title is blocked on the spine in gold that has not set, and it reads \"{you}\" for as long as you look straight at it, and something longer when you look away.",
    ),
    (
        "bindery",
        &["guillotine", "blade", "trimmings", "trimming"],
        "The guillotine's bed is ruled in inches down one side and in years down the other. The trimmings in the bin under it are margins: clean strips, every one of them blank, with the places where something was written cut exactly away.",
    ),
    (
        "restoration",
        &["book", "page", "pages", "fiber", "fibre", "tissue", "bench"],
        "The page under the tissue is a room's description, and you know the room: you came through it to get here. Half of it has been mended back, fiber by fiber, and reads as it read when you stood there. The other half is the paler paper of the patch, with the words not yet laid in. A fiber moves while you watch, and one more word is so.",
    ),
    (
        "restoration",
        &["tweezers", "brush", "brushes", "hands", "hand"],
        "The tweezers lift, set down a fiber, and lift again, at the pace of someone who is not being paid by the hour. There is a card propped against the brush jar, printed: WORK IN PROGRESS. PLEASE DO NOT REMEMBER THE BENCH. Under it in pencil: a room that is visited keeps better. Thank you.",
    ),
    (
        "lost-things",
        &["tag", "name", "nametag", "label"],
        "The name tag says {you}, in the lettering the Docent's desk uses, and has a pin through it that has never been through cloth. On the back, in pencil: \"Handed in. Finder did not leave a name.\"",
    ),
    (
        "lost-things",
        &["umbrella", "umbrellas", "glove", "gloves", "spectacles", "shoe", "shelf", "shelves", "ticket", "tickets"],
        "Every thing on the shelves has a ticket tied to it, with FOUND and a place, and a line for CLAIMED BY that is blank on all of them. The umbrellas were found in the Fogbank. The single shoe was found on a stair. The spectacles were found, the ticket says, on someone, who did not want them back.",
    ),
    (
        "ossuary",
        &["niche", "niches", "rock", "wall", "walls", "dust"],
        "Each niche has a number cut over it and nothing in it, and no dust in it either, which in this building is the stranger thing. There is one mark in the whole room, on the sill of the nearest niche: a tally, four strokes and a bar, very fresh, and a sixth stroke begun.",
    ),
];

/// What `read <a thing this room's text names>` gives, if the room names it.
pub fn read(world: &World, actor: &str, room: EntityId, arg: &str) -> Option<Outcome> {
    let key = plain_str(world, room, ROOM, "key")?;
    let a = arg.trim().to_lowercase();
    let a = a
        .trim_start_matches("the ")
        .trim_start_matches("a ")
        .trim_start_matches("an ");
    if a.is_empty() {
        return None;
    }
    let (_, _, line) = NOUNS.iter().find(|(k, names, _)| {
        *k == key
            && names
                .iter()
                .any(|n| a == *n || a.split_whitespace().any(|w| w == *n))
    })?;
    Some(Outcome::new(
        line.replace("{you}", actor),
        json!({"room": key, "read": a}),
    ))
}

#[cfg(test)]
pub fn rooms() -> Vec<&'static str> {
    NOUNS.iter().map(|(k, _, _)| *k).collect()
}
