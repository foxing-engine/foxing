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
        &["drawer", "drawers", "card", "cards", "floor"],
        "You get a thumbnail under one card and lift it off the felt of the others. The ink has gone the brown of weak tea, and it is a catalogue card still: a title you nearly know, a shelf mark, and stamped across it in violet, WITHDRAWN TO THE LOWEST SHELF. KEEP WET. The next card says the same of another book. So do the ten after it. Whoever went down with the water took the catalogue's best things with them, and filed the fact.",
    ),
    (
        "wing-pool",
        &["book", "books", "lamp", "lamps", "table", "tables"],
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
