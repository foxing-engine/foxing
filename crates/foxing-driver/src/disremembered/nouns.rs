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
        &["lantern", "lanterns", "hook", "hooks", "row", "glass", "tag", "tags"],
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
        &["nobody", "nobody's"],
        "It takes some finding, because it is not under N: it is the first line in the book, above the ruled margin, in a hand older than the column. \"Nobody: every book in the Dark Reading Room, nightly. Returned unread, each morning, to the minute. Owing: one lamp, unlit, issued at the door and never used.\" The sum beside it has never been carried, because it has never changed.",
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
        &["niche", "niches", "rock", "wall", "walls", "dust", "tally", "sill", "mark", "number", "numbers", "stroke", "strokes"],
        "Each niche has a number cut over it and nothing in it, and no dust in it either, which in this building is the stranger thing. There is one mark in the whole room, on the sill of the nearest niche: a tally, four strokes and a bar, very fresh, and a sixth stroke begun.",
    ),
    // R252, the batch of 11-19: where a first hour goes, and the Reading Rooms.
    (
        "fogbank",
        &["fog", "path", "stone", "stones"],
        "You crouch and read the path, since the fog will not be read. The stone is worn in two lines: one goes in from the Threshold and comes back, many times over; the other goes in and stops. Between them somebody has scratched, small, with a nail: THEY HOLD WHAT THEY TOOK. HIT THE ONE THAT TOOK YOURS.",
    ),
    (
        "shallows",
        &["water", "sand", "surface", "reflection"],
        "The water is still enough to read, and what is written on it is the sky, which here is one moon and no stars. Under that, in the sand, a line of small prints goes out from the edge and does not come back. They are your size. You check: you have not been out there. Not yet, or not that you hold.",
    ),
    (
        "lamplighters-lane",
        &["ladder", "ladders", "post", "posts", "lamp", "lamps"],
        "The nearest ladder has a tag wired to its top rung: PROPERTY OF THE LAMPLIGHTER. BACK DIRECTLY. The tag is brown at the edges. On the post behind it, in fresh chalk, today's tally: every lamp in the lane, lit, and a tick beside each. Somebody still does it. You have never met them and you are standing in their work.",
    ),
    (
        "orchard-wall",
        &["wall", "windfall", "windfalls", "fruit", "grass", "stone", "stones"],
        "One stone in the wall is cut, where the rest are only stacked: THIS WALL KEEPS NOTHING IN. Below it the windfalls lie in the grass where they dropped, green and hard, each with one small bite out of it, as if something keeps hoping. The moths work them over anyway. A moth is the mildest thing here that will give you something to remember.",
    ),
    (
        "orchard",
        &["tree", "trees", "fruit", "label", "labels", "branch", "branches", "row", "rows"],
        "Every tree has a zinc label on a wire, and every label is an hour: A QUARTER PAST, THE ONE BEFORE DINNER, THE LAST OF THE LIGHT. The fruit under each is that hour, nearly. You hold one and it is warm on the side that faced the afternoon. It does not come off the branch. What falls on its own can be picked up; the rest is not ready and has not been for a long time.",
    ),
    (
        "midden",
        &["heap", "pile", "rags", "keys", "names", "verse"],
        "You turn the top of the heap over with your foot. A key to a door that has been bricked up. A name on a luggage label, not yours, you think. Rags, a great many, which is most of what forgetting leaves and the one thing here worth a maker's while (`gather`). The heap keeps a handful of each plain thing and the rest rots down into it; what lives in it sorts the rest, and has opinions about where things go.",
    ),
    (
        "mezzanine",
        &["card", "newel", "post", "notice", "stair"],
        "The card on the newel post, in a librarian's capitals: READING ROOMS. NOTHING HERE IS STRUCK TO ANY PURPOSE; IT IS TOLD. TELL A CREATURE OF A FIGHT YOU STILL REMEMBER, NAMING WHAT YOU FOUGHT, A DOZEN WORDS OR MORE (`story <creature> \"...\"`). NEW WORDS LAND HARDER. EVERY TELLING DRAWS A BLOW BACK. THE LAMPMOTHS ARE KINDEST; BEGIN THERE. Under it, in pencil: nothing asks your level at this stair.",
    ),
    (
        "proof-room",
        &["proof", "proofs", "galley", "galleys", "pencil", "wall", "walls", "marks"],
        "The nearest galley is a fight, set in type, and somebody's: \"I hit it and it hit me and I hit it and it hit me.\" Every second \"hit\" is ringed in pencil with a line out to the margin, and in the margin, each time, pressed harder: rep. rep. REP. At the foot, initialed: a word said twice is a mark, and my marks land. Say it another way.",
    ),
    (
        "lamp-table",
        &["lamp", "table", "shade", "moth", "moths", "wings"],
        "Under the green shade the table is scored with small neat lines, tens of them, each the length of a moth. Beside the newest somebody has written what it was: THE ONE I NEARLY LOST. A lampmoth takes that kind and no other, a fight you only just came through (`remember <chunk> lampmoth`), keeps it warm under its wings while the lamp burns, and leaves you one that lasts.",
    ),
    (
        "returns-cage",
        &["trolley", "trolleys", "book", "books", "slip", "cage", "wire"],
        "Every book on the trolleys has a date slip in the back, and every slip says the same thing in a different hand: DUE BACK ELSEWHERE. You open one at random. It is a guide to the Reading Rooms, and the page it falls open at is this room, with a note in the margin: if one takes your elbow, go where it shows you. The fight is over and you kept what you had.",
    ),
    // X6, 11-20: the courthouse.
    (
        "lost-steps",
        &["list", "board", "crier"],
        "The List is one sheet, pinned at the corners, ruled by hand. What is on it today the room tells you when you `look`; `docket` reads the whole Docket from anywhere. At the foot, printed: CASES ARE CALLED IN THE ORDER RECEIVED, EXCEPT AS OTHERWISE. The Crier of the List stands beside it with a handbell and a coat that rustles. It cries a case once, when the Court sits on it, and has been known to practice under its breath.",
    ),
    (
        "court-of-lethe",
        &["docket", "glass", "case"],
        "The Docket under glass: every case the Court still remembers, in the Clerk's hand, the oldest gone the color of the paper. The glass is to keep fingers off it and does. `docket` reads it from anywhere, and `docket <n>` reads one case whole; a ruling that is read is kept two weeks more, and one that nobody reads fades off the sheet and binds nobody.",
    ),
    (
        "court-of-lethe",
        &["dock", "water", "channel", "bench", "tables", "table", "files", "cup"],
        "The dock is a railed square with a worn place where hands go. From it you can see the Realm's Attorney's table, where the files are stacked by weight; the Semi-Public Defender's, where forty files share one ribbon and the cup has a skin on it; and, over the water, the bench, which is higher than it needs to be and lit from somewhere behind. The water in the channel moves, slowly, from the bench's side to yours. Things said from over there arrive a little late.",
    ),
    (
        "lost-steps",
        &["clock", "hand"],
        "The clock has one hand and no numbers. You watch it and it does not move, and then you look away and back and it has. It is not telling the hour. It is telling you how long you have been in this hall, and it is a little ahead.",
    ),
    (
        "lost-steps",
        &["bench", "benches", "carving", "carvings"],
        "Initials. Tallies in fives, one of them running the whole length of a bench and onto the next. STILL HERE. A date, and under it the same date in a shakier hand. One careful drawing of a small door. And, cut deep by somebody with a proper knife: THE FORM YOU WANT IS THE ONE THEY DO NOT KEEP.",
    ),
    (
        "filing-window",
        &["card", "hours", "cards", "pigeonholes", "forms", "window", "grille", "shutter", "clerk", "stamps", "pencil", "note", "writing"],
        "The card of hours has been corrected in three inks and each correction corrects the last. What can be made out: the window is open when it is open. The second card says FORMS NOT KEPT AT THIS WINDOW ARE KEPT ELSEWHERE, and somebody waiting has added in pencil: the Crier has them, in the Hall, in its coat. The pigeonholes behind the grille are numbered 7-B, 12, 3 (pink), 40-A and 9, and every one of them is empty, and dusted.",
    ),
    (
        "court-of-lethe",
        &["jury", "box", "jury box", "seats", "rail"],
        "Twelve seats in two rows behind a rail. The cushions are dented as if by a jury that rose a moment ago and has not been back for years. The card on the rail says RESERVED. On the back of the card, where you are plainly not meant to look, a seating plan in pencil: eleven blanks, and in the twelfth place a single M, gone over several times.",
    ),
    (
        "court-of-lethe",
        &["door", "robing", "robing room"],
        "The door behind the bench is across the water and you read it from here: ROBING. A slate hangs on it of the kind a doctor's door has. Chalked on it: SITTING, rubbed out; NOT SITTING, rubbed out; and under both, left standing, the last thing the Court said from the bench, which the slate keeps until the next one. Today it reads: \"Where was I.\"",
    ),
    (
        "court-of-lethe",
        &["notice", "notices", "inkpot", "bailiff", "bar"],
        "The third hand on the notice is the Bailiff's: NOTHING IS STRUCK IN THIS ROOM BUT THE RECORD. The inkpot at its belt has a brass cap on a chain and a crust of blue-black round the neck. Whoever raises a hand in here leaves with something written on them, in a round clerk's hand, that the Bailiff thought suited. It does not wash off. Nobody has yet found what does.",
    ),
    (
        "cells-of-record",
        &["evidence", "door", "locker", "wheel", "padlocks", "hasp"],
        "EVIDENCE, in stencil, on steel. Three padlocks: one brass, one iron, one of some grey metal that is cold from a foot away. The wheel does not turn. A docket in a frame beside the door lists what is inside by case number, and every line has been blacked out by the Censor's Office, except the last, which says only: AND THE OTHER THING. Something behind the door shifts its weight. It is waiting for a case that needs it.",
    ),
    (
        "cells-of-record",
        &["alcove", "alcoves", "shelf", "shelves", "ticket", "tickets"],
        "Each alcove has a shelf, a ticket on a string, and bars you could put an arm through. What the Court strikes is kept here under its case number: a name it took off a thing, a line it cut from a wall. They keep as long as the ruling is remembered. When nobody reads the ruling the alcove is swept, and whatever was struck is simply gone, which is not the same as being given back.",
    ),
    (
        "appeal-stair",
        &["door", "plate", "knocker", "keyhole", "appeals"],
        "You get down on your knees to read it, which is the point. APPEALS, on a brass plate the size of a playing card. The knocker is a tiny hand holding a tinier gavel. Through the keyhole: a light, a long way off, and a draught that smells of beeswax. The door is locked. It is the right size for a supplicant and for nothing else, and whoever is let through will go through on their knees and be glad of it.",
    ),
    // R252 (11-20): the Trail's unread rooms and the first of the lower Stacks.
    (
        "trail-stile",
        &["step", "stile", "wall", "dip"],
        "The top step is worn to a dip the width of one person standing still. Cut into the riser under it, small, by many hands and a good many knives: initials, and under each a number. The numbers are how long they stood here looking back. Nobody has put down less than a minute. The newest cut has no number yet, and the knife marks are pale.",
    ),
    (
        "trail-meadow",
        &["butterflies", "butterfly", "envelopes", "grass", "wings"],
        "One lands on your wrist and stays long enough to be read. The wings are an envelope: a stamp in one corner with the head worn off, a postmark from a town with your street in it, and an address in a hand you know, to a name you have not gone by in years. It lifts off before you get to the back flap, where the sender would be.",
    ),
    (
        "trail-picnic",
        &["basket", "blanket", "plates", "cups", "note", "tea"],
        "Under the basket's lid, pinned to the wicker with a hatpin, a note in pencil: BACK IN A MINUTE. DO START. The pencil has been gone over more than once, each time by a firmer hand, as if the minute needed renewing. Four napkins, one unfolded. The used cup has a ring of tea in it that has not dried, and you understand that it is not going to.",
    ),
    (
        "trail-hollow-way",
        &["roots", "bank", "banks", "pebble", "lane"],
        "The roots in the bank have been tied, here and there, into knots that roots do not make: a reef knot, a bowline, one nobody taught you. Under each a flat stone is wedged with a day scratched on it, and no year. They are the days somebody meant to come back this way. The pebbles that come down the bank come from above the newest one.",
    ),
    (
        "trail-hush",
        &["moss", "birds", "beaks", "dell"],
        "The moss on the left bank grows in lines, and the lines are words, soft and green and a finger deep: everything that has been shouted here, set down as it left. The newest is still pale. You cannot make out whose each one was, only that they were loud, and where the moss has them going: uphill, toward the bench.",
    ),
    (
        "trail-bog",
        &["tussocks", "tussock", "stones", "post", "flat"],
        "A post leans at the edge with a board nailed to it, the paint gone to rust: SHORT CUT, and under that, in a later and smaller hand, IF YOU DO NOT STOP TO READ THIS. The nearest tussock has a boot print in it, filling. The second has two. Past the third there are none, which is either good news or the other kind.",
    ),
    (
        "lower-stacks",
        &["spines", "spine", "shelves", "books", "ladders", "ladder", "rails"],
        "You take one down. The spine is blank and so is the cover; the first page has a name on it, crossed through, and under it another, crossed through, and so on down the page to the last, which is not crossed through and is not anyone you know. The rest of the book is in a neat hand and stops in the middle of a sentence. On the ladder's top rung somebody has chalked a shelf mark, and the ladder is on its way there now without you.",
    ),
    (
        "catalogue",
        &["drawers", "drawer", "cards", "card", "catalogue", "catalog"],
        "The drawer that opens is three along from the one you meant. The card on top is typed, with one correction in ink: a name, then SEE under it, then another name, which has its own card somewhere that says SEE and gives the first. Somebody has pencilled in the corner: both of these were the same person and neither of them is here.",
    ),
    (
        "periodicals",
        &["newspapers", "newspaper", "papers", "headlines", "headline", "stamp"],
        "The volume on the table is open at tomorrow. The headline is about the weather in a room you have not walked yet, and it is right about the fog. Across the column the stamp has come down in violet: RECEIVED, and a date some years off. The ink of the stamp is dry. The ink of the headline comes off on your thumb.",
    ),
    (
        "dumbwaiter",
        &["box", "shaft", "rope", "slip", "hatch"],
        "The box stops at your floor without being asked. Inside: a call slip, filled in, in your hand, for a book you have not thought of yet. The shelf mark on it is a real one. The slip is dated the day after you forget what you came down here for, and the box waits a moment, politely, and goes on up with it.",
    ),
    (
        "carrels",
        &["chair", "chairs", "light", "lights", "carrel", "desk"],
        "The reading light in the nearest carrel is on and warm. On the desk, scratched with a pin where a bored hand would scratch: a tally, five-barred, row on row, and beside the last row the words NEXT ONE OVER. The chair's worn shape is nobody's in particular. You sit, to see, and the light in the next carrel comes on.",
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
    // C11: the Visitors' Book on the Kept House's hall table.
    if let Some(page) = super::kept::read_book(world, room, actor, a) {
        return Some(page);
    }
    if a.is_empty() {
        return None;
    }
    let (_, _, line) = NOUNS.iter().find(|(k, names, _)| {
        *k == key
            && names
                .iter()
                .any(|n| a == *n || a.split_whitespace().any(|w| w == *n))
    })?;
    // L4: where a trail ends here, the first reading says there is a second.
    let further = super::lore::further(world, room, a)
        .map(|w| format!(" (There is more to be made of this: `study {w}`.)"))
        .unwrap_or_default();
    Some(Outcome::new(
        format!("{}{further}", line.replace("{you}", actor)),
        json!({"room": key, "read": a}),
    ))
}

/// A thing no list names, that the room's own text does: the sentences that hold the
/// word, as the room wrote them. For a builder's room, whose slate is only words.
pub fn in_text(world: &World, room: EntityId, arg: &str) -> Option<Outcome> {
    let text = plain_str(world, room, ROOM, "description")?;
    let a = arg.trim().trim_matches('"').to_lowercase();
    let a = a
        .trim_start_matches("the ")
        .trim_start_matches("a ")
        .trim_start_matches("an ");
    // "here" and its like are the room itself, and are answered as the room.
    if ["here", "room", "this room", "this", "all", "everything"].contains(&a) {
        return None;
    }
    if a.chars().count() < 3 || a.split_whitespace().count() > 3 {
        return None;
    }
    let has = |s: &str| {
        let low = s.to_lowercase();
        low.match_indices(a).any(|(i, _)| {
            let before = low[..i].chars().next_back();
            let after = low[i + a.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric)
                && !after.is_some_and(|c| c.is_alphanumeric() && c != 's')
        })
    };
    // Sentences, kept with their stops; a quoted line stays with what leads into it.
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for c in text.chars() {
        cur.push(c);
        if c == '"' || c == '\u{201c}' || c == '\u{201d}' {
            quoted = !quoted;
        }
        if !quoted && matches!(c, '.' | '!' | '?') {
            parts.push(std::mem::take(&mut cur).trim().to_string());
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur.trim().to_string());
    }
    let hits: Vec<&String> = parts.iter().filter(|p| has(p)).take(3).collect();
    if hits.is_empty() {
        return None;
    }
    let name = plain_str(world, room, ROOM, "name").unwrap_or("this room");
    let said = hits
        .iter()
        .map(|h| h.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    Some(Outcome::new(
        format!("Of the {a}, {name} says only what is written of it: {said}"),
        json!({"read": a, "from_text": true, "sentences": hits}),
    ))
}

/// The first word each of a room's read things answers to, for bare `read`'s list.
pub fn names_for(world: &World, room: EntityId) -> Vec<&'static str> {
    let key = plain_str(world, room, ROOM, "key").unwrap_or("");
    NOUNS
        .iter()
        .filter(|(k, names, _)| *k == key && !names.is_empty())
        .map(|(_, names, _)| names[0])
        .collect()
}

#[cfg(test)]
pub fn rooms() -> Vec<&'static str> {
    NOUNS.iter().map(|(k, _, _)| *k).collect()
}
