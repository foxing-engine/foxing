//! R76b (David, 11-17: "fixed for now"): the Under-Docent, a second figure for the
//! Lower Stacks. It stands at the foot of the stair, in the Lower Stacks' first room,
//! and answers `tell under-docent "…"` there from what it knows by heart: what lives
//! below and what each does to you, the ways through, what the lantern is for, what a
//! stamp costs and how to keep a memory out of its reach, the Catalogue's index. No
//! model reads the question: it hears a word or two and says its piece, and says so.
//! The figures in its mouth are read from the creatures' own table, so they do not
//! go stale when a kind is changed.

use super::{plain_str, ROOM};
use crate::world::World;
use foxing_store::EntityId;

/// The room it keeps: the foot of the stair.
pub const KEEPS: &str = "lower-stacks";

pub fn is_named(arg: &str) -> bool {
    let a: String = arg
        .trim()
        .trim_matches('"')
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    matches!(
        a.as_str(),
        "underdocent" | "theunderdocent" | "under" | "underreader"
    )
}

pub fn here(world: &World, room: EntityId) -> bool {
    plain_str(world, room, ROOM, "key") == Some(KEEPS)
}

/// For `look` at the foot of the stair.
pub fn look_line(world: &World, room: EntityId) -> Option<String> {
    here(world, room).then(|| {
        "The Under-Docent stands at the foot of the stair with a lamp turned low, and will say what is below before you meet it: `tell under-docent \"what hunts here\"` (or the ways, the tricks, the lantern, the index).".to_string()
    })
}

fn figures(kind: &str) -> String {
    match super::combat::def_by_name(kind) {
        Some(d) => format!(
            "about {:.0} hit points, a blow of {:.0} to {:.0}",
            d.hp,
            d.attack,
            d.attack + 2.0
        ),
        None => "figures I do not have".to_string(),
    }
}

/// What it says to `text`, by the word or two it hears in it.
pub fn answer(text: &str) -> String {
    let t = text.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| t.contains(w));
    if has(&[
        "trick",
        "stamp",
        "ray",
        "bite",
        "venom",
        "shown out",
        "shows you",
    ]) {
        String::from(
            "\"Four tricks, one to a kind, and each lands only with a blow that lands. The lamprey's bite stays in you: it works for a minute after, so leaving does not end it. The usher shows you out: one blow in two moves you to another room of the Stacks, the fight over and whatever is there noticing you. The censor's stamp halves one of your chunks where it stands, one blow in two. The cataloguer has your entry open and does the same, harder. What is on your shelf they cannot reach: `shelve` what you would keep before you go past me.\""
        )
    } else if has(&[
        "hunt",
        "danger",
        "live",
        "fight",
        "creature",
        "kill",
        "strong",
        "below",
        "what is here",
    ]) {
        format!(
            "\"By the room. East, the Card Catalogue: silverfish ({}), which keep to themselves. North, the Overdue Desk: bookworms ({}), which mend by what they take. Past the Desk, the Carrels and the Pneumatic Room each keep an usher ({}); the Acid Room keeps the lamprey ({}); the Chart Room, the censor ({}); the Deep Reserve, the cataloguer ({}), and it is the worst of them. The Conclave's clerks walk here too. Nothing in this room hunts: it is why I stand in it.\"",
            figures("silverfish"),
            figures("bookworm"),
            figures("usher"),
            figures("lamprey"),
            figures("censor"),
            figures("cataloguer"),
        )
    } else if has(&[
        "way", "where", "map", "route", "room", "down", "exit", "door", "reserve",
    ]) {
        "\"Twenty rooms, and they do not all join. From here: up is the Archive and the air; east is the Card Catalogue, and the Dark Reading Room past it; north is the Overdue Desk, and from the Desk east to the Carrels or north to the Lost Things. The Reserve and the Deep Reserve are the far side, past the Chart Room. The Restoration Bench and the Bindery keep nothing that bites, if you need a room to stand in. `where \"<a room>\"` gives the way over what you remember, and `map` draws it.\"".to_string()
    } else if has(&["lantern", "lamp", "dark", "light", "forget the way", "fade"]) {
        "\"Down here a lantern keeps the map: carried, the rooms you have walked stay on it three times as long, and the ways below are not ones to walk twice by guessing. `recipes` says what a lantern asks; `read` one you carry says what it is doing.\"".to_string()
    } else if has(&["index", "catalogue", "catalog", "card", "slip", "drawer"]) {
        "\"East of me. The index has a card for every name, every creature that has one, every kind and every room, from the public record as it stands: `index \\\"wisp\\\"`, `index \\\"The Sump\\\"`. A reference slip is pinned to a drawer there, and the desk pays for its answer (`read slip`). It is the one room below where reading is the whole of the work.\"".to_string()
    } else if has(&["cure", "halved", "lost", "back", "undo", "mend", "restore"]) {
        "\"What a stamp halves stays halved; I know no cure, and the Restoration Bench mends books. What I can tell you is the order of things: shelve first, then come down; a chunk on the shelf is out of every reach but your own. And a fall here is a fall as anywhere: you wake at your door.\"".to_string()
    } else if has(&["who", "docent", "name", "you"]) {
        "\"The Docent keeps the door and carries questions to a desk. I keep the stair and carry nothing: what I say I have by heart, and it is about what is below. Ask me what hunts here, the tricks, the ways, the lantern, or the index.\"".to_string()
    } else {
        "\"I have a few things by heart and no desk behind me. Ask me what hunts here, the tricks, the ways, the lantern, the index, or whether there is a cure. For anything else the Docent is the one: `tell docent \\\"…\\\"` reaches it from here.\"".to_string()
    }
}
