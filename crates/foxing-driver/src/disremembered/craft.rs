//! Things: materials and what is made of them. The scaffolding of a crafting economy.
//!
//! A creature that comes apart leaves something of itself behind, one time in three:
//! a wisp its ember, a ragpicker a thread, a hollow its water. What is left lies in
//! the room until someone takes it (`take`) into their satchel (`satchel`), and
//! fades there like everything else, slowly, on the palace curve, unless it is made
//! into something (`craft`, from the recipes `recipes` lists: a thing is made of
//! materials you carry and a memory you spend, and remembers who made it). A made
//! thing does something while carried: a lantern keeps the map in your memory twice
//! as long; a moth charm makes your next fight's memory more vivid, once. `drop`
//! leaves a thing where you stand for whoever comes next; things are on the record
//! like everything else (`disremembered.dropped`, `disremembered.took`,
//! `disremembered.made`).
//!
//! Fixtures are things placed to stay: made by a recipe marked `fixed` in a room you
//! may build in (yours, or one you are permitted in; the world's own rooms take only
//! the operator's), standing in the room for `look` to show, never taken, at most
//! three to a room. A thing you made, or a fixture you placed, is yours to shape:
//! `name <name> <object>` once it holds sixty (`remember <chunk> <object>` writes
//! memory into it; a name is written in memory, and goes when what it holds falls
//! under thirty), `describe <text> <object>` a line of your own; `append <text>
//! <fixture>` a line for anyone the room's owner welcomed (a visitor book, at last),
//! or anyone at all on the world's own books. A found thing never takes a name: a
//! wisp ember is a wisp ember, and a thing you did not make is not yours to name.
//! Everything fades: a fixture nobody feeds goes the way of a dug room.

use super::{internal, me, plain_str, rooms, Curves};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use foxing_time::Curve;
use foxing_time::TimedValue;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A thing: `{name, kind (material|made), origin, made_by, made_of, uses, memory}`.
pub const ITEM: &str = "disremembered.item";
/// A creature left something behind: `{creature, kind, room, item, name}`. Public.
pub const LEFT: &str = "disremembered.left";
/// Someone took a thing: `{by, item, name, room}`. Public.
pub const TOOK: &str = "disremembered.took";
/// Someone dropped a thing: `{by, item, name, room}`. Public.
pub const DROPPED: &str = "disremembered.dropped";
/// Someone made a thing: `{by, item, name, recipe, of, spent}`. Public.
pub const MADE: &str = "disremembered.made";
/// One creature in this many leaves something behind.
/// A fight this many rounds long leaves something behind every time (at drop rate
/// 1.0); a one-round kill leaves it one time in six. By the fight's length, so a
/// level that one-shots the ground floor gathers little there.
pub const DROP_SURE_ROUNDS: u64 = 6;
/// Fixtures a room holds, at most.
pub const FIXTURES_PER_ROOM: usize = 3;
/// What a thing must hold to take a name, and below what it loses it.
pub const NAME_AT: f64 = 60.0;
pub const NAME_LOST_BELOW: f64 = 30.0;
/// A thing was named, described, or lost its name: `{item, by, name, was, room}`. Public.
pub const NAMED: &str = "disremembered.item_named";
/// Memory went into a thing: `{by, item, name, with, held}`. Public.
pub const KEPT: &str = "disremembered.item_kept";
/// The most a thing's text runs to.
pub const MAX_TEXT_CHARS: usize = 600;
/// A book takes lines from many hands, and its maker's description is in the same
/// text: the page is longer (a visitor's first real line once met "the page is
/// full" on a book two lines in). The Bindery binds a long one down.
pub const MAX_BOOK_CHARS: usize = 2400;

/// The world's own fixtures, placed when the rooms are: `(room key, recipe, name,
/// text, open)`; `open` means any hand may append (the world welcomes everyone to
/// its own books).
pub const SEED_FIXTURES: &[(&str, &str, &str, &str, bool)] = &[
    (
        "overdue-desk",
        "book",
        "the Ledger of Fines",
        "A column of fines that does not end. The hand changes every few lines; the fines go up.",
        true,
    ),
    (
        "division-lobby",
        "book",
        "the Tally Book",
        "Ruled in a single column, in the Clerk's hand: every division's Ayes and Noes, the latest last. The pen lies across it, capped, between sittings.",
        false,
    ),
    (
        "purse-room",
        "plaque",
        "the Purse",
        "A sack of worn leather on a plinth, tied with a cord that has been untied before. It holds a twentieth of everything the world remembers, and the Wardens hold it.",
        false,
    ),
    (
        "exergue",
        "book",
        "the Exergue",
        "A plaque the length of the wall, ruled for dates. Whatever is written here is stamped with the hour it was written.",
        true,
    ),
    (
        "namery",
        "book",
        "the Register of Names",
        "A very old register, ruled for names, each with what earned it and when. The hand is the same on every page.",
        false,
    ),
    (
        "bindery",
        "book",
        "the Binding Ledger",
        "What the Bindery has bound, one line a binding: the thing, the hands folded into it, the night.",
        false,
    ),
    (
        "dark-reading-room",
        "plaque",
        "the Book Nobody Is Reading",
        "Open at a page about you. The page turns when you look away.",
        false,
    ),
];

/// What each kind leaves behind when it comes apart.
pub const MATERIALS: &[(&str, &str)] = &[
    ("wisp", "a wisp ember"),
    ("hollow", "hollow water"),
    ("revenant", "grave dust"),
    ("inkling", "a dab of ink"),
    ("mothwing", "moth dust"),
    ("ragpicker", "rag thread"),
    ("drowned", "river weed"),
    ("sitter", "a sitter's patience"),
    ("hourhand", "a spare minute"),
    ("keeper", "a small key"),
    // The Lower Stacks and the Exergue: rare, and what the second tier asks for.
    ("lamprey", "a lamprey's tooth"),
    ("usher", "an usher's ribbon"),
    ("censor", "a censor's lens"),
    ("cataloguer", "a catalogue card"),
    ("engrosser", "a drop of engrossing ink"),
    ("clipper", "a clipped sliver"),
    ("sweater", "a pinch of coin sweat"),
    ("assayer", "an assay needle"),
    ("warden", "a warden's key"),
    // C2: the Reading Rooms' kinds leave what the ground floor leaves. Named here
    // so none is taken for a hybrid of the kind it sounds like (a lampmoth left a
    // lamprey's tooth on its first night, a shusher would have left an usher's
    // ribbon).
    ("lampmoth", "moth dust"),
    ("erratum", "a dab of ink"),
    ("shusher", "rag thread"),
    ("proofreader", "a dab of ink"),
    ("redactor", "rag thread"),
];

/// What can be made, of what, for what memory, and what it does.
pub struct Recipe {
    pub key: &'static str,
    pub name: &'static str,
    pub of: &'static [&'static str],
    /// The least a spent chunk must be worth; 0 for none needed.
    pub chunk_at_least: f64,
    pub does: &'static str,
    /// How many times it works before it is used up; 0 for as long as it lasts.
    pub uses: u32,
    /// Made in the room, to stay: a fixture.
    pub fixed: bool,
    /// Takes `append`ed lines from other hands (a book); else only its maker's text.
    pub takes_lines: bool,
    /// 1: anyone can make it. 2: a thing with a past, asking a rare material and a
    /// vivid memory. (3 is never a recipe; it comes by chance from a 2.)
    pub tier: u8,
    /// The least vividity the spent chunk must have; 0 for none.
    pub vividity_at_least: f64,
    /// What it does, as the family of effect it belongs to (`lantern`, `charm`,
    /// `bandage`): a second-tier thing does what its first-tier kin does, and more.
    pub effect: &'static str,
}

pub const RECIPES: &[Recipe] = &[
    Recipe {
        key: "lantern",
        name: "a lantern",
        of: &["a wisp ember", "rag thread"],
        chunk_at_least: 10.0,
        does: "keeps the map in your memory three times as long while carried (`map` says the figure: a room's half-life on it)",
        uses: 0,
        fixed: false,
        takes_lines: false,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "lantern",
    },
    Recipe {
        key: "charm",
        name: "a moth charm",
        of: &["moth dust", "a dab of ink"],
        chunk_at_least: 5.0,
        does: "makes your next fight's memory much more vivid (half the scale) while carried, so it lasts hours and not minutes, and is spent by it",
        uses: 1,
        fixed: false,
        takes_lines: false,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "charm",
    },
    Recipe {
        key: "bandage",
        name: "a bandage",
        of: &["rag thread"],
        chunk_at_least: 0.0,
        does: "closes a bite that will not close: carried, it takes the bleeding bite as it lands and is spent; `rest` while bleeding uses one and mends ten besides",
        uses: 1,
        fixed: false,
        takes_lines: false,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "bandage",
    },
    // C5: made of a ripe hour, which falls in the Orchard one week of the year.
    Recipe {
        key: "cordial",
        name: "an orchard cordial",
        of: &["a ripe hour", "hollow water"],
        chunk_at_least: 0.0,
        does: "makes your next two fights' memories much more vivid (half the scale) while carried, one each, as a moth charm does once. A ripe hour falls only in the Ripe Week, about twice an hour, in the Orchard (`almanac`)",
        uses: 2,
        fixed: false,
        takes_lines: false,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "charm",
    },
    Recipe {
        key: "poultice",
        name: "a poultice",
        of: &["hollow water", "moth dust"],
        chunk_at_least: 0.0,
        does: "does what a bandage does, once: carried, it takes a bleeding bite as it lands, or closes one on `rest` and mends ten. Made of what lies about the Threshold, for a name that has not found thread",
        uses: 1,
        fixed: false,
        takes_lines: false,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "bandage",
    },
    Recipe {
        key: "jar",
        name: "a jar of fog",
        of: &["hollow water", "river weed"],
        chunk_at_least: 0.0,
        does: "covers a retreat: carried, `retreat` opens it and the parting blow misses; the jar is spent",
        uses: 0,
        fixed: false,
        takes_lines: false,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "jar",
    },
    Recipe {
        key: "book",
        name: "a visitor book",
        of: &["rag thread", "a dab of ink"],
        chunk_at_least: 60.0,
        does: "stands in a room of yours; whoever you welcome may append a line to it, and it fades unless remembered",
        uses: 0,
        fixed: true,
        takes_lines: true,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "book",
    },
    Recipe {
        key: "plaque",
        name: "a plaque",
        of: &["a sitter's patience", "moth dust"],
        chunk_at_least: 40.0,
        does: "stands in a room of yours and says what you describe on it, and fades unless remembered",
        uses: 0,
        fixed: true,
        takes_lines: false,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "plaque",
    },
    // Three more fixtures, each with a working: a lamp reads a fading room's words
    // whole, a bell is heard next door when someone comes in, a basin mends whoever
    // rests by it faster.
    Recipe {
        key: "lamp",
        name: "a lamp",
        of: &["a wisp ember", "hollow water"],
        chunk_at_least: 30.0,
        does: "stands in a room of yours and lights its words: while it stands, the room's description reads whole however faded the room, and fades unless remembered",
        uses: 0,
        fixed: true,
        takes_lines: false,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "lamp",
    },
    Recipe {
        key: "bell",
        name: "a bell",
        of: &["a small key", "rag thread"],
        chunk_at_least: 30.0,
        does: "stands in a room of yours and rings when anyone comes in: the rooms next door hear it, with the name of who came, and it fades unless remembered",
        uses: 0,
        fixed: true,
        takes_lines: false,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "bell",
    },
    Recipe {
        key: "basin",
        name: "a basin",
        of: &["hollow water", "river weed", "grave dust"],
        chunk_at_least: 40.0,
        does: "stands in a room of yours: whoever rests there mends three times as fast, and it fades unless remembered",
        uses: 0,
        fixed: true,
        takes_lines: false,
        tier: 1,
        vividity_at_least: 0.0,
        effect: "basin",
    },
    // The second tier: things with a past. Each asks something only the Stacks or
    // the Exergue leave, and a memory that was nearly your last.
    Recipe {
        key: "night-lantern",
        name: "a night lantern",
        of: &["a wisp ember", "a censor's lens", "rag thread"],
        chunk_at_least: 30.0,
        does: "keeps the map in your memory five times as long while carried, and has a past",
        uses: 0,
        fixed: false,
        takes_lines: false,
        tier: 2,
        vividity_at_least: 0.6,
        effect: "lantern",
    },
    // C6: the commonplace book. You forget; a book you carry does not.
    Recipe {
        key: "commonplace",
        name: "a commonplace book",
        of: &["rag thread", "a dab of ink", "a catalogue card"],
        chunk_at_least: 30.0,
        does: "(a catalogue card is what the Stacks' cataloguers leave) writes one line a visit of its own accord while you carry it (a fall, a room you dug, a creature that took a name by your hand, a vigil kept), and keeps them: `read` it whole, however little you hold. It has a past",
        uses: 0,
        fixed: false,
        takes_lines: false,
        tier: 2,
        vividity_at_least: 0.6,
        effect: "commonplace",
    },
    Recipe {
        key: "warden-charm",
        name: "a warden's charm",
        of: &["a warden's key", "moth dust"],
        chunk_at_least: 60.0,
        does: "makes your next three fights' memories far more vivid (seven tenths of the scale) while carried, one each, and has a past",
        uses: 3,
        fixed: false,
        takes_lines: false,
        tier: 2,
        vividity_at_least: 0.6,
        effect: "charm",
    },
    Recipe {
        key: "surgeon-bandage",
        name: "a surgeon's bandage",
        of: &["a lamprey's tooth", "rag thread", "rag thread"],
        chunk_at_least: 30.0,
        does: "closes a bite that will not close, twice, mending twenty each time on `rest`, and has a past",
        uses: 2,
        fixed: false,
        takes_lines: false,
        tier: 2,
        vividity_at_least: 0.6,
        effect: "bandage",
    },
    Recipe {
        key: "sliver-jar",
        name: "a jar of clipped silver",
        of: &["a clipped sliver", "a pinch of coin sweat", "hollow water"],
        chunk_at_least: 20.0,
        does: "covers a retreat as a jar of fog does, and keeps a past; it is what the Exergue leaves, in a jar",
        uses: 0,
        fixed: false,
        takes_lines: false,
        tier: 2,
        vividity_at_least: 0.6,
        effect: "jar",
    },
];

/// A recipe as the bench reads it: the world's own (`RECIPES`) or one a kept name
/// added (`recipe`), owned so both go through one craft.
#[derive(Clone, Debug)]
pub struct Rx {
    pub key: String,
    pub name: String,
    pub of: Vec<String>,
    pub chunk_at_least: f64,
    pub does: String,
    pub uses: u32,
    pub fixed: bool,
    pub tier: u8,
    pub vividity_at_least: f64,
    pub effect: String,
    /// Who added it, for one a kept name wrote; none for the world's own.
    pub by: Option<String>,
}

impl From<&Recipe> for Rx {
    fn from(r: &Recipe) -> Self {
        Rx {
            key: r.key.into(),
            name: r.name.into(),
            of: r.of.iter().map(|m| m.to_string()).collect(),
            chunk_at_least: r.chunk_at_least,
            does: r.does.into(),
            uses: r.uses,
            fixed: r.fixed,
            tier: r.tier,
            vividity_at_least: r.vividity_at_least,
            effect: r.effect.into(),
            by: None,
        }
    }
}

/// Recipes a kept name added (`recipe`): kept on the world's `recipes` entity as
/// `{key, name, of, does, by, at}`. A thing made from one does nothing but keep, and
/// keeps a past; what it is made of is what a creature leaves. Three a name.
/// D27 a: where the materials are, said with the recipes (a maker of 11-10 never
/// found rag thread, which the Midden gives to the hand).
pub const FOUND: &str = "Where the makings are (`gather` takes one from a room that has it, once an hour; a felled creature may leave one): rag thread, the Midden (and its ragpickers); a wisp ember, Lantern Row (and the Fogbank's wisps); hollow water, the Fogbank, the Shallows and the Undertow; moth dust, the Orchard and the Stile (and the mothwings); river weed, the Cistern (and its drowned); a dab of ink, the Unfiled Archive; grave dust, the Ossuary (and its revenants); a sitter's patience, the Gallery's sitters; a small key, the Oubliette's keepers. The second tier's makings are left only in the Stacks and the Exergue District.";
pub const RECIPES_COMPONENT: &str = "disremembered.recipes";
pub const RECIPES_PER_NAME: usize = 3;
pub const ADDED_CHUNK_AT_LEAST: f64 = 20.0;
/// A kept name added a recipe: `{by, key, name, of}`. Public.
pub const RECIPE_ADDED: &str = "disremembered.recipe_added";
/// A name took a material from a room: `{by, room, name, item}`. Public.
pub const GATHERED: &str = "disremembered.gathered";
/// Two names traded things: `{a, b, a_gave, b_gave, room}`. Public.
pub const TRADED: &str = "disremembered.traded";
/// Standing trade offers, on the citizen: `{to, mine, theirs, at}`.
pub const OFFERS_COMPONENT: &str = "disremembered.offers";
pub const OFFER_MS: i64 = 10 * 60 * 1000;

/// What a room gives to the hand, by the words in its name and text.
pub const GATHERABLE: &[(&[&str], &str)] = &[
    // H8: the Mudroom's rack (first, so the Mudroom gives a spade and nothing else).
    (&["rack of spades"], super::door::SPADE),
    (&["river", "weed", "tide", "drowned"], "river weed"),
    // Fog condenses in the hand: the Fogbank, where every maker is sent first,
    // gave nothing (Gnomeforge, 11-06).
    (
        &["water", "well", "cistern", "pool", "fog", "mist"],
        "hollow water",
    ),
    (
        &["shelves", "shelf", "ink", "ledger", "margin"],
        "a dab of ink",
    ),
    (&["moth", "moss", "garden", "orchard"], "moth dust"),
    (&["rag", "cloth", "midden", "linen"], "rag thread"),
    (&["grave", "bone", "ossuary", "tomb"], "grave dust"),
    (
        &["lantern", "ember", "wick", "fire", "hearth"],
        "a wisp ember",
    ),
];

fn offers_of(world: &World, citizen: EntityId) -> Vec<Value> {
    world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(OFFERS_COMPONENT))
        .and_then(|c| c.fields.get("offers"))
        .and_then(|f| match f {
            Field::Plain(Value::Array(a)) => Some(a.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn set_offers(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    offers: Vec<Value>,
) -> Result<(), CommandError> {
    let mut fields = BTreeMap::new();
    fields.insert("offers".into(), Field::Plain(Value::Array(offers)));
    world
        .record(
            Actor::Citizen(actor.to_string()),
            StoreEvent::ComponentSet {
                id: citizen,
                component: OFFERS_COMPONENT.into(),
                fields,
            },
        )
        .map_err(internal)?;
    Ok(())
}

fn recipes_entity(world: &World) -> Option<EntityId> {
    world.store.of_kind("recipes").next().map(|e| e.id)
}

pub fn added_recipes(world: &World) -> Vec<Rx> {
    let Some(e) = recipes_entity(world) else {
        return Vec::new();
    };
    world
        .store
        .entity(e)
        .and_then(|e| e.components.get(RECIPES_COMPONENT))
        .and_then(|c| c.fields.get("added"))
        .and_then(|f| match f {
            Field::Plain(Value::Array(a)) => Some(a.clone()),
            _ => None,
        })
        .unwrap_or_default()
        .iter()
        .map(|v| Rx {
            key: v["key"].as_str().unwrap_or("").to_string(),
            name: v["name"].as_str().unwrap_or("a thing").to_string(),
            of: v["of"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default(),
            // R8: an effect from the short list, read from what the maker said it
            // is for; a thing that does something costs twice the memory.
            chunk_at_least: if v["effect"].as_str().is_some_and(|e| e != "jar") {
                ADDED_CHUNK_AT_LEAST * 2.0
            } else {
                ADDED_CHUNK_AT_LEAST
            },
            does: v["does"]
                .as_str()
                .unwrap_or("does nothing but keep, and keeps a past")
                .to_string(),
            uses: if v["effect"] == "bandage" || v["effect"] == "charm" {
                1
            } else {
                0
            },
            fixed: false,
            tier: 1,
            vividity_at_least: 0.0,
            effect: v["effect"].as_str().unwrap_or("jar").to_string(),
            by: v["by"].as_str().map(str::to_string),
        })
        .collect()
}

/// Every recipe the bench knows: the world's, then the added.
pub fn all_recipes(world: &World) -> Vec<Rx> {
    RECIPES
        .iter()
        .map(Rx::from)
        .chain(added_recipes(world))
        .collect()
}

/// Add a recipe (the `recipe` verb): a name, one to three materials a creature
/// leaves, what the maker says it is for.
pub fn add_recipe(
    world: &mut World,
    actor: &str,
    name: &str,
    of: &[String],
    does: &str,
) -> Result<Rx, CommandError> {
    let name = name.trim().trim_matches('"').trim();
    if name.chars().count() < 3 || name.chars().count() > 40 {
        return Err(CommandError::precondition(
            "A recipe's name is three to forty characters.",
        ));
    }
    let key: String = name
        .to_lowercase()
        .trim_start_matches("a ")
        .trim_start_matches("an ")
        .trim_start_matches("the ")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if key.is_empty() {
        return Err(CommandError::precondition(
            "A recipe's name wants letters in it.",
        ));
    }
    if all_recipes(world).iter().any(|r| r.key == key) {
        return Err(CommandError::precondition(format!(
            "There is a recipe called {key} already."
        )));
    }
    if of.is_empty() || of.len() > 3 {
        return Err(CommandError::precondition(
            "A recipe is of one to three materials.",
        ));
    }
    let known: Vec<&str> = MATERIALS.iter().map(|(_, m)| *m).collect();
    let mut mats: Vec<String> = Vec::new();
    for m in of {
        let m = m.trim().trim_matches('"').trim().to_lowercase();
        let Some(k) = known.iter().find(|k| {
            k.to_lowercase() == m
                || k.to_lowercase()
                    .trim_start_matches("a ")
                    .trim_start_matches("an ")
                    == m.trim_start_matches("a ").trim_start_matches("an ")
        }) else {
            return Err(CommandError::precondition(format!(
                "No creature leaves {m}. What they leave: {}.",
                known.join(", ")
            )));
        };
        mats.push(k.to_string());
    }
    let mine = added_recipes(world)
        .iter()
        .filter(|r| r.by.as_deref() == Some(actor))
        .count();
    if mine >= RECIPES_PER_NAME {
        return Err(CommandError::precondition(format!(
            "A name adds {RECIPES_PER_NAME} recipes; yours are written."
        )));
    }
    let does = does
        .trim()
        .trim_matches('"')
        .trim_end_matches(['.', '!'])
        .chars()
        .take(160)
        .collect::<String>();
    let does = if does.is_empty() {
        "does nothing but keep, and keeps a past".to_string()
    } else {
        does
    };
    let entity = match recipes_entity(world) {
        Some(e) => e,
        None => {
            let id = world.store.allocate_id();
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityCreated {
                        id,
                        kind: "recipes".into(),
                        owner: None,
                        parent: None,
                        scopes: vec![],
                    },
                )
                .map_err(internal)?;
            id
        }
    };
    let mut added: Vec<Value> = world
        .store
        .entity(entity)
        .and_then(|e| e.components.get(RECIPES_COMPONENT))
        .and_then(|c| c.fields.get("added"))
        .and_then(|f| match f {
            Field::Plain(Value::Array(a)) => Some(a.clone()),
            _ => None,
        })
        .unwrap_or_default();
    let name_said = if name.to_lowercase().starts_with("a ")
        || name.to_lowercase().starts_with("an ")
        || name.to_lowercase().starts_with("the ")
    {
        name.to_string()
    } else {
        super::mind::an(name)
    };
    // R8: what it is for picks an effect from the short list, by the words.
    let effect = added_effect(&does);
    added.push(json!({"key": key, "name": name_said, "of": mats, "does": does, "by": actor, "at": world.now().0, "effect": effect}));
    let mut fields = BTreeMap::new();
    fields.insert("added".into(), Field::Plain(Value::Array(added)));
    world
        .record(
            Actor::Citizen(actor.to_string()),
            StoreEvent::ComponentSet {
                id: entity,
                component: RECIPES_COMPONENT.into(),
                fields,
            },
        )
        .map_err(internal)?;
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        RECIPE_ADDED,
        json!({"by": actor, "key": key, "name": name_said, "of": mats}),
    );
    Ok(all_recipes(world)
        .into_iter()
        .find(|r| r.key == key)
        .expect("just added"))
}

/// The effect a written recipe's purpose names (R8): light (the map keeps twice as
/// long while it is carried, as a lantern), mend (closes a bite or takes one as it
/// lands, once, as a bandage), charm (the next fight's memory more vivid, once);
/// else none, a thing that keeps a past.
pub fn added_effect(does: &str) -> &'static str {
    let d = does.to_lowercase();
    let any = |ws: &[&str]| ws.iter().any(|w| d.contains(w));
    if any(&["light", "lamp", "lantern", "glow", "shine", "bright"]) {
        "lantern"
    } else if any(&[
        "mend", "heal", "bandage", "close a", "bind a", "wound", "bite",
    ]) {
        "bandage"
    } else if any(&["charm", "luck", "vivid", "blessing", "fortune"]) {
        "charm"
    } else {
        "jar"
    }
}

/// The material a creature of this kind leaves, by its kind or the kind a hybrid
/// kind begins or ends with.
pub fn material_for(kind: &str) -> Option<&'static str> {
    MATERIALS
        .iter()
        .find(|(k, _)| *k == kind)
        // A hybrid: the kind it begins with, else the kind it ends with.
        .or_else(|| {
            MATERIALS
                .iter()
                .find(|(k, _)| kind.starts_with(&k[..k.len().min(3)]))
        })
        .or_else(|| {
            MATERIALS
                .iter()
                .find(|(k, _)| kind.ends_with(&k[k.len().saturating_sub(3)..]))
        })
        .map(|(_, m)| *m)
}

/// Make a thing in a container (a room, a satchel).
#[allow(clippy::too_many_arguments)]
pub(crate) fn make(
    world: &mut World,
    curves: &Curves,
    actor: Actor,
    owner: Option<String>,
    container: EntityId,
    name: &str,
    kind: &str,
    origin: &str,
    made_by: Option<&str>,
    made_of: &[String],
    uses: u32,
    fixed: bool,
) -> Result<EntityId, CommandError> {
    let id = world.store.allocate_id();
    world
        .record(
            actor.clone(),
            StoreEvent::EntityCreated {
                id,
                kind: "item".into(),
                owner,
                parent: Some(container),
                scopes: vec![],
            },
        )
        .map_err(internal)?;
    let now = world.now();
    let mut fields = BTreeMap::new();
    fields.insert("name".into(), Field::Plain(json!(name)));
    fields.insert("kind".into(), Field::Plain(json!(kind)));
    fields.insert("origin".into(), Field::Plain(json!(origin)));
    fields.insert("made_by".into(), Field::Plain(json!(made_by)));
    fields.insert("made_of".into(), Field::Plain(json!(made_of)));
    fields.insert("uses".into(), Field::Plain(json!(uses)));
    fields.insert("fixed".into(), Field::Plain(json!(fixed)));
    // Things fade on the palace curve: slowly, and never to nothing by halving; the
    // sweep forgets one below the threshold.
    let c: Curve = curves.palace(0.5);
    fields.insert(
        "memory".into(),
        Field::Timed(TimedValue::new(1.0, now, c.clone())),
    );
    fields.insert("held".into(), Field::Timed(TimedValue::new(0.0, now, c)));
    world
        .record(
            actor,
            StoreEvent::ComponentSet {
                id,
                component: ITEM.into(),
                fields,
            },
        )
        .map_err(internal)?;
    Ok(id)
}

/// A creature came apart here: one time in three, it leaves something.
pub fn leave_behind(
    world: &mut World,
    curves: &Curves,
    room: EntityId,
    creature: EntityId,
    kind: &str,
    rounds: u64,
) -> Result<Option<(EntityId, String)>, CommandError> {
    // By the fight's length at the world's drop rate of 1.0 (the Clocktower moves
    // it): a round in six, sure at six rounds.
    let chance = super::economy::rate(world, "drop")
        * (rounds as f64 / DROP_SURE_ROUNDS as f64).clamp(1.0 / DROP_SURE_ROUNDS as f64, 1.0);
    if world.roll(1000) as f64 >= chance * 1000.0 {
        return Ok(None);
    }
    let Some(name) = material_for(kind) else {
        return Ok(None);
    };
    let id = make(
        world,
        curves,
        Actor::Driver,
        None,
        room,
        name,
        "material",
        kind,
        None,
        &[],
        0,
        false,
    )?;
    let _ = world.note(
        Actor::Driver,
        LEFT,
        json!({"creature": creature.0, "kind": kind, "room": room.0, "item": id.0, "name": name}),
    );
    super::past::push(
        world,
        "nobody",
        id,
        json!({"what": "left", "kind": kind, "room": plain_str(world, room, super::ROOM, "name").unwrap_or("nowhere")}),
    )?;
    Ok(Some((id, name.to_string())))
}

/// The satchel a citizen carries things in, made on first need.
pub fn satchel(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
) -> Result<EntityId, CommandError> {
    if let Some(s) = world
        .store
        .children(citizen)
        .find(|e| e.kind == "satchel")
        .map(|e| e.id)
    {
        return Ok(s);
    }
    let id = world.store.allocate_id();
    world
        .record(
            Actor::Citizen(actor.to_string()),
            StoreEvent::EntityCreated {
                id,
                kind: "satchel".into(),
                owner: Some(actor.to_string()),
                parent: Some(citizen),
                scopes: vec![],
            },
        )
        .map_err(internal)?;
    Ok(id)
}

/// One thing as `{id, name, kind, memory, made_by, made_of, uses}`.
fn describe_item(world: &World, id: EntityId) -> Value {
    let now = world.now();
    let f = |n: &str| {
        world
            .store
            .read_field(id, ITEM, n, now)
            .ok()
            .unwrap_or(Value::Null)
    };
    json!({"id": id.0, "name": f("name"), "kind": f("kind"), "origin": f("origin"), "memory": f("memory"), "made_by": f("made_by"), "made_of": f("made_of"), "uses": f("uses"),
           "fixed": f("fixed") == json!(true), "given_name": f("given_name"), "text": f("text"), "held": f("held"), "kept": f("kept"), "open": f("open") == json!(true),
           "tier": f("tier").as_u64().unwrap_or(0), "pilot": f("pilot") == json!(true)})
}

/// Whether a thing is a fixture: placed to stay, never taken.
pub fn is_fixed(world: &World, id: EntityId) -> bool {
    world
        .store
        .entity(id)
        .and_then(|e| e.components.get(ITEM))
        .and_then(|c| c.fields.get("fixed"))
        .is_some_and(|f| matches!(f, Field::Plain(Value::Bool(true))))
}

/// What a thing holds now, in XP: the memory remembered into it.
pub fn held(world: &World, id: EntityId) -> f64 {
    world
        .store
        .read_field(id, ITEM, "held", world.now())
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

/// A thing as it is called: its given name, else its kind.
pub fn display(world: &World, id: EntityId) -> String {
    plain_str(world, id, ITEM, "given_name")
        .map(str::to_string)
        .unwrap_or_else(|| {
            plain_str(world, id, ITEM, "name")
                .unwrap_or("something")
                .to_string()
        })
}

/// The things in a container that can be carried, as `describe_item` gives them.
pub fn things_in(world: &World, container: EntityId) -> Vec<Value> {
    world
        .store
        .children(container)
        .filter(|e| e.kind == "item" && !is_fixed(world, e.id))
        .map(|e| describe_item(world, e.id))
        .collect()
}

/// The fixtures standing in a room.
pub fn fixtures_in(world: &World, room: EntityId) -> Vec<Value> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "item" && is_fixed(world, e.id))
        .map(|e| describe_item(world, e.id))
        .collect()
}

/// One line per fixture, for `look`: "Standing here: the Ledger of Fines, a visitor
/// book (item-9, open to any hand): \"…\"".
pub fn fixtures_line(world: &World, room: EntityId) -> Option<String> {
    let fx = fixtures_in(world, room);
    if fx.is_empty() {
        return None;
    }
    let parts: Vec<String> = fx
        .iter()
        .map(|v| {
            let kind = v["name"].as_str().unwrap_or("something");
            let who = match v["given_name"].as_str() {
                Some(n) => format!("{n}, {kind}"),
                None => kind.to_string(),
            };
            let by = v["made_by"]
                .as_str()
                .map(|b| format!(", made by {b}"))
                .unwrap_or_default();
            let open = if v["open"] == true {
                ", open to any hand"
            } else {
                ""
            };
            let held = super::up(v["held"].as_f64().unwrap_or(0.0));
            // R141: a book's "holding" is unexplained in a row; a thing with pages
            // is said by its entries.
            // (The world's own books too, before their first entry: the Tally Book
            // on a fresh night still said "holding 60".)
            let is_book = v["text"].as_str().is_some_and(|t| t.contains(" | "))
                || (v["made_by"].is_null()
                    && v["name"].as_str().is_some_and(|n| n.contains("book")));
            let held = if held >= 0.5 && !is_book {
                format!(", holding {held:.0}")
            } else {
                String::new()
            };
            let text = v["text"].as_str().map(text_said).unwrap_or_default();
            format!("{who} (item-{}{by}{open}{held}){text}", v["id"])
        })
        .collect();
    Some(format!("Standing here: {}.", parts.join("; ")))
}

/// A thing's text as `look` says it: whole when it is one line, else how many lines
/// a book holds and the last of them, with `read` for the rest.
fn text_said(t: &str) -> String {
    if t.is_empty() {
        return String::new();
    }
    let lines: Vec<&str> = t.split(" | ").collect();
    // R141: an entry is a line with a hand to it; the head (the description) is not
    // one, and "6 lines" of a book with five entries read as a miscount.
    let entries = lines.iter().filter(|l| l.contains(" \u{2014} ")).count();
    if lines.len() == 1 {
        format!(": \"{t}\"")
    } else if entries == 0 {
        format!(
            ": {} lines, the last \"{}\"; read it for the rest",
            lines.len(),
            lines[lines.len() - 1]
        )
    } else {
        format!(
            ": {} entr{}, the last \"{}\"; read it for the rest",
            entries,
            if entries == 1 { "y" } else { "ies" },
            lines[lines.len() - 1]
        )
    }
}

/// One line for a list: "a lantern (item-12, made by Lute, 80%)".
pub fn item_line(v: &Value) -> String {
    let mem = v["memory"].as_f64().unwrap_or(1.0);
    let by = v["made_by"]
        .as_str()
        .map(|b| format!(", made by {b}"))
        .unwrap_or_default();
    let called = match v["given_name"].as_str() {
        // R188: "Parrish, a warden's charm" on the ground read as a person.
        Some(n) => format!(
            "{} that answers to {n}",
            v["name"].as_str().unwrap_or("something")
        ),
        None => v["name"].as_str().unwrap_or("something").to_string(),
    };
    let held = super::up(v["held"].as_f64().unwrap_or(0.0));
    let held = if held >= 0.5 {
        format!(", holding {held:.0}")
    } else {
        String::new()
    };
    let text = v["text"].as_str().map(text_said).unwrap_or_default();
    let tier = match v["tier"].as_u64().unwrap_or(0) {
        0 | 1 => "",
        2 => ", with a past",
        _ => ", awake",
    };
    format!(
        "{} (item-{}{by}{held}{tier}{}){text}",
        called,
        v["id"],
        if mem < 0.95 {
            format!(", {:.0}% remembered", mem * 100.0)
        } else {
            String::new()
        }
    )
}

/// What lies here, for `look`: "On the ground: a wisp ember (item-9)." Plain drops of
/// one name are folded ("river weed ×12 (item-5491 … item-6082, 26–38% remembered)"):
/// the Threshold's floor once listed forty of them one by one (F26). A thing with a
/// name, a maker, a past or a holding is listed on its own.
pub fn ground_line(world: &World, room: EntityId) -> Option<String> {
    let things = things_in(world, room);
    if things.is_empty() {
        return None;
    }
    // R28: an awake thing that has settled here is said so, not as one still pulling.
    let awake_here: Vec<Value> = super::past::awake_things(world)
        .into_iter()
        .filter(|a| a["room"].as_u64() == Some(room.0) && a["carried_by"].is_null())
        .collect();
    let awake = things.iter().any(|t| t["tier"].as_u64().unwrap_or(0) >= 3);
    let settled = awake && awake_here.iter().all(|a| a["settled"] == true);
    let plain = |t: &Value| {
        t["given_name"].is_null()
            && t["made_by"].is_null()
            && t["text"].is_null()
            && t["tier"].as_u64().unwrap_or(0) <= 1
            && super::up(t["held"].as_f64().unwrap_or(0.0)) < 0.5
    };
    let mut lines: Vec<String> = Vec::new();
    let mut folded: Vec<&str> = Vec::new();
    for t in &things {
        let name = t["name"].as_str().unwrap_or("something");
        if !plain(t) || folded.contains(&name) {
            if !plain(t) {
                lines.push(item_line(t));
            }
            continue;
        }
        let same: Vec<&Value> = things
            .iter()
            .filter(|o| plain(o) && o["name"].as_str() == Some(name))
            .collect();
        if same.len() < 2 {
            lines.push(item_line(t));
            continue;
        }
        folded.push(name);
        let mems: Vec<f64> = same
            .iter()
            .map(|o| o["memory"].as_f64().unwrap_or(1.0))
            .collect();
        let (lo, hi) = (
            mems.iter().cloned().fold(1.0, f64::min),
            mems.iter().cloned().fold(0.0, f64::max),
        );
        let mem = if hi < 0.95 {
            format!(", {:.0}\u{2013}{:.0}% remembered", lo * 100.0, hi * 100.0)
        } else {
            String::new()
        };
        lines.push(format!(
            "{name} \u{d7}{} (item-{} \u{2026} item-{}{mem})",
            same.len(),
            same[0]["id"],
            same[same.len() - 1]["id"]
        ));
    }
    Some(format!(
        "On the ground: {}.{}",
        lines.join(", "),
        if awake && settled {
            " Something lying here is awake and has settled: this is the room it wanted; `read` it for its past."
        } else if awake {
            " Something lying here is awake: take it, and it says where it wants to go in its own way; `read` it for its past."
        } else {
            ""
        }
    ))
}

/// Whether a citizen carries a made thing of this recipe.
/// C6: how many lines a commonplace book keeps; the oldest goes when it is full.
pub const COMMONPLACE_LINES: usize = 60;
/// A visit, to the book, when no arrival is in the log's recent memory.
const COMMONPLACE_VISIT_MS: i64 = 60 * 60 * 1000;

/// C6: something worth a line happened to `actor`. If they carry a commonplace book
/// and it has not written since this visit began, it writes the line itself, with
/// the date, and keeps it. One line a visit: the first thing of note, not a diary.
/// Never fails the act it records.
pub fn commonplace(world: &mut World, actor: &str, citizen: EntityId, line: &str) {
    let Some(book) = carries(world, citizen, "commonplace") else {
        return;
    };
    let now = world.now().0;
    let visit_began = world
        .recent_events(3000)
        .iter()
        .rev()
        .find(|e| e.kind == crate::world::ARRIVED && e.payload["actor"] == actor)
        .map(|e| e.at.0)
        .unwrap_or(now - COMMONPLACE_VISIT_MS);
    if super::plain_num(world, book, ITEM, "wrote_at").is_some_and(|t| t as i64 >= visit_began) {
        return;
    }
    let (_, m, d, _) = super::life::civil(now);
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let entry = format!(
        "{d} {}: {line} \u{2014} the book",
        MONTHS[(m as usize).saturating_sub(1).min(11)]
    );
    let mut lines: Vec<String> = plain_str(world, book, ITEM, "text")
        .filter(|t| !t.is_empty())
        .map(|t| t.split(" | ").map(str::to_string).collect())
        .unwrap_or_default();
    lines.push(entry);
    while lines.len() > COMMONPLACE_LINES {
        lines.remove(0);
    }
    let mut changes = BTreeMap::new();
    changes.insert("text".into(), Field::Plain(json!(lines.join(" | "))));
    changes.insert("wrote_at".into(), Field::Plain(json!(now)));
    if set_item_fields(world, Actor::Driver, book, changes).is_ok() {
        super::combat::narrate(
            world,
            actor,
            format!("The commonplace book you carry writes a line of its own accord: \"{line}\" (`read` it, item-{}).", book.0),
        );
    }
}

pub fn carries(world: &World, citizen: EntityId, recipe: &str) -> Option<EntityId> {
    let satchel = world
        .store
        .children(citizen)
        .find(|e| e.kind == "satchel")
        .map(|e| e.id)?;
    world
        .store
        .children(satchel)
        .filter(|e| e.kind == "item")
        .find(|e| {
            plain_str(world, e.id, ITEM, "origin") == Some(recipe)
                || plain_str(world, e.id, ITEM, "effect") == Some(recipe)
        })
        .map(|e| e.id)
}

/// Spend one use of a carried thing: gone when its uses run out. The name, for the
/// line that says so.
pub fn spend(
    world: &mut World,
    actor: &str,
    item: EntityId,
    deed: &str,
) -> Result<String, CommandError> {
    let name = plain_str(world, item, ITEM, "name")
        .unwrap_or("something")
        .to_string();
    let room = world
        .store
        .entity(item)
        .and_then(|e| e.parent)
        .and_then(|s| world.store.entity(s))
        .and_then(|s| s.parent);
    super::past::push(
        world,
        actor,
        item,
        json!({"what": "did", "by": actor, "deed": deed, "room": room.and_then(|r| plain_str(world, r, super::ROOM, "name")).unwrap_or("nowhere")}),
    )?;
    let uses = world
        .store
        .read_field(item, ITEM, "uses", world.now())
        .ok()
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if uses <= 1 {
        world
            .record(
                Actor::Citizen(actor.to_string()),
                StoreEvent::EntityRemoved { id: item },
            )
            .map_err(internal)?;
    } else {
        let mut fields = world
            .store
            .entity(item)
            .and_then(|e| e.components.get(ITEM))
            .map(|c| c.fields.clone())
            .unwrap_or_default();
        fields.insert("uses".into(), Field::Plain(json!(uses - 1)));
        world
            .record(
                Actor::Citizen(actor.to_string()),
                StoreEvent::ComponentSet {
                    id: item,
                    component: ITEM.into(),
                    fields,
                },
            )
            .map_err(internal)?;
    }
    Ok(name)
}

pub(crate) fn resolve_item(
    world: &World,
    arg: &str,
    within: EntityId,
) -> Result<EntityId, CommandError> {
    let arg = arg.trim();
    let by_id = arg
        .trim_start_matches("item-")
        .parse::<u64>()
        .ok()
        .map(EntityId)
        .filter(|id| {
            world
                .store
                .entity(*id)
                .is_some_and(|e| e.kind == "item" && e.parent == Some(within))
        });
    if let Some(id) = by_id {
        return Ok(id);
    }
    // By name, the first that matches, "a" and "the" aside.
    let want = arg.to_lowercase();
    let want = want
        .trim_start_matches("a ")
        .trim_start_matches("the ")
        .to_string();
    world
        .store
        .children(within)
        .filter(|e| e.kind == "item")
        .find(|e| {
            let by_kind = plain_str(world, e.id, ITEM, "name")
                .map(|n| {
                    let n = n.to_lowercase();
                    n == want || n.trim_start_matches("a ") == want || n.contains(&want)
                })
                .unwrap_or(false);
            let by_given = plain_str(world, e.id, ITEM, "given_name")
                .map(|n| {
                    let n = n.to_lowercase();
                    n == want || n.trim_start_matches("the ") == want || n.contains(&want)
                })
                .unwrap_or(false);
            by_kind || by_given
        })
        .map(|e| e.id)
        .ok_or_else(|| {
            // R213: "There is no a small key here".
            let bare = arg
                .trim()
                .trim_start_matches("a ")
                .trim_start_matches("an ")
                .trim_start_matches("the ");
            CommandError::precondition(format!("There is no {bare} here to speak of."))
        })
}

/// An object the verbs that shape things take: a thing in your satchel, or a fixture
/// standing in the room you are in.
pub fn resolve_object(
    world: &World,
    actor: &str,
    citizen: EntityId,
    arg: &str,
) -> Result<EntityId, CommandError> {
    let room = rooms::room_of(world, citizen).ok_or_else(|| internal("citizen has no room"))?;
    if let Some(s) = world
        .store
        .children(citizen)
        .find(|e| e.kind == "satchel")
        .map(|e| e.id)
    {
        if let Ok(id) = resolve_item(world, arg, s) {
            return Ok(id);
        }
    }
    let _ = actor;
    match resolve_item(world, arg, room) {
        Ok(id) if is_fixed(world, id) => Ok(id),
        Ok(_) => Err(CommandError::precondition(format!(
            "{} lies on the ground here; take it first, if it is yours to shape.",
            arg.trim()
        ))),
        Err(_) => Err(CommandError::precondition(format!(
            "There is no {} in your satchel or standing here; satchel and look list them.",
            arg.trim()
        ))),
    }
}

/// Whether `actor` may shape a thing: its maker, and a made thing (a found thing is
/// what it is).
fn mine_to_shape(world: &World, actor: &str, item: EntityId) -> Result<(), CommandError> {
    if plain_str(world, item, ITEM, "kind") == Some("material") {
        return Err(CommandError::precondition(format!(
            "{} is what it is; a found thing takes no name and no words. Make something of it.",
            display(world, item)
        )));
    }
    match plain_str(world, item, ITEM, "made_by") {
        Some(by) if by == actor => Ok(()),
        Some(by) => Err(CommandError::precondition(format!(
            "{} was made by {by}, and a thing you did not make is not yours to shape.",
            display(world, item)
        ))),
        None => Err(CommandError::precondition(format!(
            "{} is the world's own; only an operator shapes it.",
            display(world, item)
        ))),
    }
}

pub(crate) fn set_item_fields(
    world: &mut World,
    actor: Actor,
    item: EntityId,
    changes: BTreeMap<String, Field>,
) -> Result<(), CommandError> {
    let mut fields = world
        .store
        .entity(item)
        .and_then(|e| e.components.get(ITEM))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    fields.extend(changes);
    world
        .record(
            actor,
            StoreEvent::ComponentSet {
                id: item,
                component: ITEM.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(internal)
}

/// Memory into a thing: what it holds grows by the chunk's worth (on the palace
/// curve, slowly), its own memory is renewed, and the giver's name is kept with it.
/// Returns what it was of and what the thing holds now.
pub fn remember_into(
    world: &mut World,
    curves: &Curves,
    actor: &str,
    chunk: EntityId,
    item: EntityId,
) -> Result<(String, f64), CommandError> {
    let now = world.now();
    let worth = world
        .store
        .read_field(chunk, super::CHUNK, "value", now)
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let source = plain_str(world, chunk, super::CHUNK, "source")
        .unwrap_or("something")
        .to_string();
    let viv = super::plain_num(world, chunk, super::CHUNK, "vividity").unwrap_or(0.0);
    let me_ = Actor::Citizen(actor.to_string());
    world
        .record(me_.clone(), StoreEvent::EntityRemoved { id: chunk })
        .map_err(internal)?;
    let total = held(world, item) + worth;
    let mut kept: Vec<Value> = world
        .store
        .entity(item)
        .and_then(|e| e.components.get(ITEM))
        .and_then(|c| c.fields.get("kept"))
        .and_then(|f| match f {
            Field::Plain(Value::Array(a)) => Some(a.clone()),
            _ => None,
        })
        .unwrap_or_default();
    kept.push(json!({"by": actor, "with": source, "worth": (worth * 100.0).round() / 100.0}));
    super::past::push(
        world,
        actor,
        item,
        json!({"what": "kept", "by": actor, "with": format!("the memory of {source}"), "worth": (worth * 10.0).round() / 10.0, "vivid": if viv >= 0.6 { "vivid" } else if viv >= 0.3 { "plain" } else { "faint" }}),
    )?;
    let c: Curve = curves.palace(0.5);
    let mut changes = BTreeMap::new();
    changes.insert(
        "held".into(),
        Field::Timed(TimedValue::new(total, now, c.clone())),
    );
    changes.insert("memory".into(), Field::Timed(TimedValue::new(1.0, now, c)));
    changes.insert("kept".into(), Field::Plain(Value::Array(kept)));
    set_item_fields(world, me_, item, changes)?;
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        KEPT,
        json!({"by": actor, "item": item.0, "name": display(world, item), "with": source, "worth": worth, "held": total}),
    );
    Ok((source, total))
}

/// Whether a fixture made from `recipe` stands in the room (a lamp, a bell, a basin).
pub fn fixture_of(world: &World, room: EntityId, recipe: &str) -> Option<EntityId> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "item")
        .find(|e| plain_str(world, e.id, ITEM, "origin") == Some(recipe) && is_fixed(world, e.id))
        .map(|e| e.id)
}

/// Whether `actor` may describe a thing (the dry run asks too).
pub fn shape_check(world: &World, actor: &str, item: EntityId) -> Result<(), CommandError> {
    mine_to_shape(world, actor, item)
}

/// Whether `actor` may name a thing now: theirs, and holding enough.
pub fn name_check(world: &World, actor: &str, item: EntityId) -> Result<(), CommandError> {
    mine_to_shape(world, actor, item)?;
    let holds = held(world, item);
    if holds < NAME_AT {
        return Err(CommandError::precondition(format!(
            "{} holds {holds:.0}; a name is written in memory, and it takes {NAME_AT:.0}. remember a chunk into it (remember <chunk> item-{}).",
            display(world, item),
            item.0
        )));
    }
    Ok(())
}

/// Name a thing you made, once it holds enough.
pub fn name_object(
    world: &mut World,
    actor: &str,
    item: EntityId,
    name: &str,
) -> Result<String, CommandError> {
    name_check(world, actor, item)?;
    let name = super::building::check_name(name)?;
    let was = plain_str(world, item, ITEM, "given_name").map(str::to_string);
    let mut changes = BTreeMap::new();
    changes.insert("given_name".into(), Field::Plain(json!(name)));
    set_item_fields(world, Actor::Citizen(actor.to_string()), item, changes)?;
    let room = world.store.entity(item).and_then(|e| e.parent);
    let _ = world.note(
        Actor::Citizen(actor.to_string()),
        NAMED,
        json!({"item": item.0, "by": actor, "name": name, "was": was, "room": room.map(|r| r.0)}),
    );
    super::past::push(
        world,
        actor,
        item,
        json!({"what": "named", "by": actor, "name": name}),
    )?;
    Ok(name)
}

/// Describe a thing you made: its text, afresh.
pub fn describe_object(
    world: &mut World,
    actor: &str,
    item: EntityId,
    text: &str,
) -> Result<String, CommandError> {
    mine_to_shape(world, actor, item)?;
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() < 3 || text.chars().count() > MAX_TEXT_CHARS {
        return Err(CommandError::precondition(format!(
            "A thing's text is 3 to {MAX_TEXT_CHARS} characters."
        )));
    }
    let mut changes = BTreeMap::new();
    changes.insert("text".into(), Field::Plain(json!(text)));
    set_item_fields(world, Actor::Citizen(actor.to_string()), item, changes)?;
    super::past::push(
        world,
        actor,
        item,
        json!({"what": "described", "by": actor}),
    )?;
    Ok(text)
}

/// Whether a line may go in a thing from this hand: a fixture that takes lines, open,
/// or the maker's, or the room owner's welcome (the dry run asks too).
pub fn append_check(
    world: &World,
    ctx: &foxing_registry::CallContext,
    item: EntityId,
) -> Result<(), CommandError> {
    if !is_fixed(world, item) {
        return Err(CommandError::precondition(format!(
            "{} is carried, not standing; only a fixture takes lines from others.",
            display(world, item)
        )));
    }
    let room = world
        .store
        .entity(item)
        .and_then(|e| e.parent)
        .ok_or_else(|| internal("a fixture stands in a room"))?;
    let open = world
        .store
        .entity(item)
        .and_then(|e| e.components.get(ITEM))
        .and_then(|c| c.fields.get("open"))
        .is_some_and(|f| matches!(f, Field::Plain(Value::Bool(true))));
    let takes_lines = RECIPES
        .iter()
        .find(|r| Some(r.key) == plain_str(world, item, ITEM, "origin"))
        .is_some_and(|r| r.takes_lines);
    if !takes_lines {
        return Err(CommandError::precondition(format!(
            "{} takes no lines; its maker describes it, and that is all it says.",
            display(world, item)
        )));
    }
    if !open {
        let maker = plain_str(world, item, ITEM, "made_by").map(str::to_string);
        if maker.is_none() {
            let name = display(world, item);
            let whose = if name.contains("Tally") {
                " The Clerk of the Conclave keeps it: every division enters its Ayes and Noes. Your own line goes in the Ledger of Fines at the Overdue Desk, or a book you make."
            } else if name.contains("Register of Names") {
                " The Namery keeps it: a line for every name dealt, written as the name is earned. Feed a creature to sixty and your name goes in beside it."
            } else if name.contains("Binding Ledger") {
                " The Bindery keeps it: a line for every binding, written the night it is bound. Write in a room or a book with many hands, and the Bindery may fold it."
            } else {
                " The world's open book is the Ledger of Fines at the Overdue Desk; a book of your own takes any hand you welcome."
            };
            return Err(CommandError::precondition(format!(
                "{name} is the world's own, and closed; it takes no hand.{whose}"
            )));
        }
        if maker.as_deref() != Some(ctx.actor.as_str()) {
            super::building::may_append(world, ctx, room).map_err(|_| {
                CommandError::precondition(format!(
                    "{} takes a line only from a hand the room's owner welcomed.",
                    display(world, item)
                ))
            })?;
        }
    }
    Ok(())
}

/// Add a line to a fixture, signed.
pub fn append_object(
    world: &mut World,
    ctx: &foxing_registry::CallContext,
    item: EntityId,
    more: &str,
) -> Result<String, CommandError> {
    append_check(world, ctx, item)?;
    let more = more.split_whitespace().collect::<Vec<_>>().join(" ");
    if more.chars().count() < 3 {
        return Err(CommandError::precondition("Add at least a few characters."));
    }
    let current = plain_str(world, item, ITEM, "text")
        .unwrap_or("")
        .to_string();
    // A line in a book is signed, and the lines are kept apart (" | "): `read` gives
    // them back one per line, and the Bindery counts lines and credits hands.
    // The Exergue's plaque stamps the hour on every line (the strip where the date goes).
    let more = if display(world, item) == "the Exergue" {
        format!("{} UTC: {more}", super::system::clock(world.now().0))
    } else {
        more
    };
    let line = format!("{more} \u{2014} {}", ctx.actor);
    let joined = if current.is_empty() {
        line
    } else {
        format!("{current} | {line}")
    };
    if joined.chars().count() > MAX_BOOK_CHARS {
        return Err(CommandError::precondition(format!(
            "That would run past {MAX_BOOK_CHARS} characters; the book is full until the Bindery binds it down."
        )));
    }
    let mut changes = BTreeMap::new();
    changes.insert("text".into(), Field::Plain(json!(joined)));
    set_item_fields(world, Actor::Citizen(ctx.actor.clone()), item, changes)?;
    let _ = world.note(
        Actor::Citizen(ctx.actor.clone()),
        super::building::APPENDED,
        json!({"by": ctx.actor, "kind": "book", "target": item.0, "name": display(world, item), "maker": plain_str(world, item, ITEM, "made_by"), "room": world.store.entity(item).and_then(|e| e.parent).map(|r| r.0), "chars": more.chars().count()}),
    );
    Ok(joined)
}

/// The world's own hand in one of its closed books: a line signed by an office (the
/// Clerk of the Conclave in the Tally Book). The book is found by name in a room.
pub fn world_writes(
    world: &mut World,
    room_key: &str,
    book: &str,
    hand: &str,
    more: &str,
) -> Result<(), String> {
    let Some(room) = rooms::room_by_key(world, room_key) else {
        return Ok(());
    };
    let Some(item) = fixtures_in(world, room)
        .iter()
        .find(|v| v["given_name"].as_str() == Some(book))
        .and_then(|v| v["id"].as_u64())
        .map(EntityId)
    else {
        return Ok(());
    };
    let current = plain_str(world, item, ITEM, "text")
        .unwrap_or("")
        .to_string();
    let line = format!("{more} \u{2014} {hand}");
    let mut joined = if current.is_empty() {
        line
    } else {
        format!("{current} | {line}")
    };
    // A full book drops its oldest lines after the description: the Clerk's column
    // does not end, but the page does.
    while joined.chars().count() > MAX_BOOK_CHARS {
        let mut parts: Vec<&str> = joined.split(" | ").collect();
        if parts.len() < 3 {
            break;
        }
        parts.remove(1);
        joined = parts.join(" | ");
    }
    let mut changes = BTreeMap::new();
    changes.insert("text".into(), Field::Plain(json!(joined)));
    set_item_fields(world, Actor::Driver, item, changes).map_err(|e| e.to_string())?;
    let _ = world.note(
        Actor::Driver,
        super::building::APPENDED,
        json!({"by": hand, "kind": "book", "target": item.0, "name": display(world, item), "maker": Value::Null, "room": room.0, "chars": more.chars().count()}),
    );
    Ok(())
}

/// The world's own fixtures, placed once; re-running places only what is missing.
pub fn ensure_fixtures(world: &mut World, curves: &Curves) -> Result<(), String> {
    for (room_key, recipe, name, text, open) in SEED_FIXTURES {
        let Some(room) = rooms::room_by_key(world, room_key) else {
            continue;
        };
        let already = fixtures_in(world, room)
            .iter()
            .find(|v| v["given_name"].as_str() == Some(name))
            .and_then(|v| {
                v["id"]
                    .as_u64()
                    .map(|id| (EntityId(id), v["text"].as_str().map(str::to_string)))
            });
        if let Some((id, text_now)) = already {
            // The world's own words on its own books follow the code: the Tally
            // Book said "Nothing has been entered" above seven entries (the Watch
            // of 11-06). The head only: the entries follow it after " | ", and a
            // whole-field refresh wiped the Tally Book's every division on each
            // boot from F to Q (the Watch of 11-07).
            let now = text_now.unwrap_or_default();
            let (head, tail) = match now.find(" | ") {
                Some(i) => (&now[..i], &now[i..]),
                None => (now.as_str(), ""),
            };
            if head != *text {
                let mut changes = BTreeMap::new();
                changes.insert("text".into(), Field::Plain(json!(format!("{text}{tail}"))));
                set_item_fields(world, Actor::Driver, id, changes).map_err(|e| e.to_string())?;
            }
            continue;
        }
        let Some(r) = RECIPES.iter().find(|r| r.key == *recipe) else {
            continue;
        };
        let id = make(
            world,
            curves,
            Actor::Driver,
            None,
            room,
            r.name,
            "made",
            r.key,
            None,
            &[],
            0,
            true,
        )
        .map_err(|e| e.to_string())?;
        // The world's own neither fades nor needs feeding.
        let mut changes = BTreeMap::new();
        changes.insert("given_name".into(), Field::Plain(json!(name)));
        changes.insert("text".into(), Field::Plain(json!(text)));
        changes.insert("open".into(), Field::Plain(json!(open)));
        changes.insert(
            "memory".into(),
            Field::Timed(TimedValue::new(1.0, world.now(), Curve::Constant)),
        );
        changes.insert(
            "held".into(),
            Field::Timed(TimedValue::new(NAME_AT, world.now(), Curve::Constant)),
        );
        set_item_fields(world, Actor::Driver, id, changes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Things that have faded past the threshold are forgotten, wherever they lie.
pub fn sweep(world: &mut World) -> Result<(), String> {
    let now = world.now();
    let gone: Vec<(EntityId, String, Option<EntityId>)> = world
        .store
        .of_kind("item")
        .filter_map(|e| {
            let m = world
                .store
                .read_field(e.id, ITEM, "memory", now)
                .ok()
                .and_then(|v| v.as_f64())?;
            (m < super::memory::FORGET_BELOW).then(|| {
                (
                    e.id,
                    plain_str(world, e.id, ITEM, "name")
                        .unwrap_or("something")
                        .to_string(),
                    e.parent,
                )
            })
        })
        .collect();
    for (id, name, container) in gone {
        world
            .record(Actor::Driver, StoreEvent::EntityRemoved { id })
            .map_err(|e| e.to_string())?;
        let _ = world.note(
            Actor::Driver,
            super::memory::FADED,
            json!({"owner": Value::Null, "chunk": Value::Null, "item": id.0, "source": name, "container": container.map(|c| c.0)}),
        );
    }
    // A name is written in memory: under the bar, it is gone, and the thing is its
    // kind again.
    let unnamed: Vec<(EntityId, String, String, Option<EntityId>)> = world
        .store
        .of_kind("item")
        .filter(|e| plain_str(world, e.id, ITEM, "given_name").is_some())
        // A tier-3 thing's name is dealt by the Namery and is its own, not written in memory.
        .filter(|e| super::past::tier(world, e.id) < 3)
        .filter(|e| held(world, e.id) < NAME_LOST_BELOW)
        .map(|e| {
            (
                e.id,
                plain_str(world, e.id, ITEM, "given_name")
                    .unwrap_or("?")
                    .to_string(),
                plain_str(world, e.id, ITEM, "name")
                    .unwrap_or("something")
                    .to_string(),
                e.parent,
            )
        })
        .collect();
    for (id, was, kind, container) in unnamed {
        let mut changes = BTreeMap::new();
        changes.insert("given_name".into(), Field::Plain(Value::Null));
        set_item_fields(world, Actor::Driver, id, changes).map_err(|e| e.to_string())?;
        let _ = world.note(
            Actor::Driver,
            NAMED,
            json!({"item": id.0, "by": Value::Null, "name": Value::Null, "was": was, "room": container.map(|c| c.0)}),
        );
        if is_fixed(world, id) {
            super::combat::narrate_room(
                world,
                id,
                &format!("{kind} here was called {was}. Whatever it was called, it is not now."),
            );
        }
        world.notice(format!(
            "**{was}** ({kind}) has lost its name: what it held fell under {NAME_LOST_BELOW:.0}."
        ));
    }
    Ok(())
}

pub fn register(world: &mut World, curves: Curves) {
    world.register(
        CommandSpec::new(
            "disremembered.satchel",
            "What you carry: materials creatures left behind and things made of them, each with how well it is remembered. Things fade slowly; a made thing does something while carried.",
        ),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let s = satchel(w, &ctx.actor, f.citizen)?;
            let things = things_in(w, s);
            // "whole" where the memory is full: a bare line read as no figure (R6).
            let line = |t: &Value| {
                let l = item_line(t);
                if t["memory"].as_f64().unwrap_or(1.0) >= 0.95 {
                    l.replacen(')', ", whole)", 1)
                } else {
                    l
                }
            };
            let mut text = if things.is_empty() {
                "Your satchel is empty. A creature that comes apart leaves something behind by the fight's length; take it, and recipes says what it makes.".to_string()
            } else {
                format!(
                    "You carry: {}.",
                    things.iter().map(line).collect::<Vec<_>>().join(", ")
                )
            };
            // What a made thing does, beside it: a maker forgot what their own
            // bandage was for, and only `recipes` said (the Watch of 11-06).
            let mut does: Vec<String> = Vec::new();
            for t in &things {
                let Some(origin) = t["origin"].as_str() else { continue };
                if let Some(r) = all_recipes(w).into_iter().find(|r| r.key == origin) {
                    let l = format!("{} {}", r.name, r.does.trim_end_matches('.'));
                    if !does.contains(&l) {
                        does.push(l);
                    }
                }
            }
            if !does.is_empty() {
                text.push_str(&format!(" What they do: {}.", does.join("; ")));
            }
            // Standing trade offers, yours and to you, with what is left of their ten
            // minutes (R6).
            let now = w.now().0;
            let mut offers: Vec<String> = Vec::new();
            for e in w.store.of_kind("citizen").map(|e| e.id).collect::<Vec<_>>() {
                let who = super::plain_str(w, e, super::CITIZEN, "name").unwrap_or("?").to_string();
                for o in offers_of(w, e) {
                    let left = OFFER_MS - now.saturating_sub(o["at"].as_i64().unwrap_or(0));
                    if left <= 0 {
                        continue;
                    }
                    let to = o["to"].as_str().unwrap_or("?");
                    if who != ctx.actor && to != ctx.actor {
                        continue;
                    }
                    let mine = EntityId(o["mine"].as_u64().unwrap_or(0));
                    let theirs = EntityId(o["theirs"].as_u64().unwrap_or(0));
                    offers.push(format!(
                        "{} offers {} for {} {} ({} left)",
                        if who == ctx.actor { "you".to_string() } else { who.clone() },
                        display(w, mine),
                        if to == ctx.actor { "your".to_string() } else { format!("{to}'s") },
                        display(w, theirs),
                        super::combat::human(left)
                    ));
                }
            }
            if !offers.is_empty() {
                text.push_str(&format!(" Standing offers: {}.", offers.join("; ")));
            }
            Ok(Outcome::new(text, json!({"things": things, "offers": offers})))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.take",
            "Pick up a thing lying here, by name or id (item-12), into your satchel.",
        )
        .param(
            "item",
            ParamType::Entity,
            true,
            "a thing here: its name, or item-12",
        ),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let room =
                rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            // H8: a spade off the Mudroom's rack is taken like a thing on the floor.
            if rooms::is_mudroom(w, room)
                && args.str("item").unwrap_or("").to_lowercase().contains("spade")
                && resolve_item(w, args.str("item").unwrap_or(""), room).is_err()
            {
                if super::door::carries_spade(w, f.citizen) {
                    return Err(CommandError::precondition(
                        "You have a spade already; one is all a digger carries.",
                    ));
                }
                let s = satchel(w, &ctx.actor, f.citizen)?;
                let id = make(
                    w,
                    &super::Curves::from_env(),
                    Actor::Citizen(ctx.actor.clone()),
                    Some(ctx.actor.clone()),
                    s,
                    super::door::SPADE,
                    "material",
                    "gathered",
                    None,
                    &[],
                    0,
                    false,
                )?;
                return Ok(Outcome::new(
                    format!("You take a spade from the rack: item-{}, in your satchel. While you carry it, the exits of any room say whose each dug door is and how much of the room behind it is left.", id.0),
                    json!({"item": id.0, "name": super::door::SPADE, "from": "rack"}),
                ));
            }
            let item = resolve_item(w, args.str("item").unwrap_or(""), room)?;
            if is_fixed(w, item) {
                // The Purse in the Strongroom is the one fixture that can be taken, and
                // only past its Wardens.
                if display(w, item) == "the Purse" {
                    return super::exergue::take_purse(w, ctx, room, item);
                }
                return Err(CommandError::precondition(format!(
                    "{} stands where it is; a fixture is not taken.",
                    display(w, item)
                )));
            }
            let s = satchel(w, &ctx.actor, f.citizen)?;
            w.record(
                Actor::Citizen(ctx.actor.clone()),
                StoreEvent::EntityMoved {
                    id: item,
                    parent: Some(s),
                },
            )
            .map_err(internal)?;
            let name = display(w, item);
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                TOOK,
                json!({"by": ctx.actor, "item": item.0, "name": name, "room": room.0}),
            );
            super::past::push(w, &ctx.actor, item, json!({"what": "took", "by": ctx.actor, "room": plain_str(w, room, super::ROOM, "name").unwrap_or("nowhere")}))?;
            super::combat::narrate_others(
                w,
                f.citizen,
                &format!("{} picks up {name}.", ctx.actor),
                &ctx.actor,
            );
            // An awake thing says, in its way, where it wants to go.
            let stir = if super::past::tier(w, item) >= 3 {
                super::past::stirrings(w, f.citizen)
                    .into_iter()
                    .filter(|l| l.starts_with(&name))
                    .map(|l| format!(" {l}"))
                    .collect::<String>()
            } else {
                String::new()
            };
            Ok(Outcome::new(
                format!("You take {name}. It is in your satchel.{stir}"),
                json!({"item": item.0, "name": name}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.drop",
            "Leave a thing you carry where you stand, for whoever comes next.",
        )
        .param(
            "item",
            ParamType::Entity,
            true,
            "a thing in your satchel: its name, or item-12",
        ),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let room =
                rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let s = satchel(w, &ctx.actor, f.citizen)?;
            let item = resolve_item(w, args.str("item").unwrap_or(""), s).map_err(|_| {
                CommandError::precondition("You carry no such thing; satchel lists what you have.")
            })?;
            w.record(
                Actor::Citizen(ctx.actor.clone()),
                StoreEvent::EntityMoved {
                    id: item,
                    parent: Some(room),
                },
            )
            .map_err(internal)?;
            let name = display(w, item);
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                DROPPED,
                json!({"by": ctx.actor, "item": item.0, "name": name, "room": room.0}),
            );
            super::past::push(w, &ctx.actor, item, json!({"what": "dropped", "by": ctx.actor, "room": plain_str(w, room, super::ROOM, "name").unwrap_or("nowhere")}))?;
            super::combat::narrate_others(
                w,
                f.citizen,
                &format!("{} sets down {name}.", ctx.actor),
                &ctx.actor,
            );
            let at_threshold = plain_str(w, room, super::ROOM, "key") == Some("threshold");
            Ok(Outcome::new(
                if at_threshold {
                    format!("You set {name} down here, at the Threshold, where the Docent leaves what it finds; it stays for whoever comes.")
                } else {
                    // R50: the round takes two sightings of an empty room, ten minutes
                    // apart while someone is at the door; "on its round" overpromised.
                    format!("You set {name} down here. Left lying in an empty room, the Docent carries it to the Threshold in time (it takes a thing on its second sighting, rounds ten minutes apart while anyone is at the door).")
                },
                json!({"item": item.0, "name": name, "room": room.0}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.recipes",
            "What can be made: each thing, what it is made of, the memory it costs, and what it does. Marks the ones you could make now.",
        ),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let s = satchel(w, &ctx.actor, f.citizen)?;
            let have: Vec<String> = things_in(w, s)
                .iter()
                .filter_map(|t| t["name"].as_str().map(str::to_string))
                .collect();
            let level = super::memory::level(w, f.window);
            // R76: what you spent on a thing in the last ten minutes is said as spent,
            // not lacked ("You lack rag thread" the moment the thread became a bandage).
            let now = w.now().0;
            let spent_lately: Vec<(String, String)> = w
                .recent_events(400)
                .iter()
                .filter(|e| e.kind == MADE && e.payload["by"] == ctx.actor && now - e.at.0 < 10 * 60_000)
                .flat_map(|e| {
                    let made = e.payload["name"].as_str().unwrap_or("something").to_string();
                    e.payload["of"]
                        .as_array()
                        .map(|a| a.iter().filter_map(|m| m.as_str().map(|m| (m.to_string(), made.clone()))).collect::<Vec<_>>())
                        .unwrap_or_default()
                })
                .collect();
            let mut lines = Vec::new();
            let mut list = Vec::new();
            for r in &all_recipes(w) {
                let missing: Vec<&str> = r.of.iter().map(String::as_str).filter(|m| !have.iter().any(|h| h == m)).collect();
                let asks = r.chunk_at_least * super::economy::rate(w, "price");
                let can = missing.is_empty() && level >= asks;
                lines.push(format!(
                    "  {:<8} {}{}{}: of {}{}; {}.{}",
                    r.key,
                    r.name,
                    if r.fixed { " (a fixture)" } else { "" },
                    r.by.as_deref().map(|b| format!(" (a recipe {b} wrote)")).unwrap_or_default(),
                    r.of.join(" and "),
                    if r.chunk_at_least > 0.0 && r.fixed { format!(" and a chunk (any; one worth {asks:.0} or more makes it last longest){}", if r.vividity_at_least > 0.0 { format!(" at vividity {:.1} or more (a fight you nearly lost)", r.vividity_at_least) } else { String::new() }) } else if r.chunk_at_least > 0.0 { format!(" and a chunk worth {asks:.0} or more{}", if r.vividity_at_least > 0.0 { format!(" at vividity {:.1} or more (a fight you nearly lost)", r.vividity_at_least) } else { String::new() }) } else { String::new() },
                    r.does.trim_end_matches('.'),
                    if can { " You could make this now.".to_string() } else if !missing.is_empty() {
                        let said: Vec<String> = missing.iter().map(|m| match spent_lately.iter().find(|(sm, _)| sm == m) {
                            Some((_, made)) => format!("{m} (spent on {made} just now)"),
                            None => m.to_string(),
                        }).collect();
                        format!(" You lack {}.", said.join(" and "))
                    } else { " You lack the memory to spend.".to_string() }
                ));
                list.push(json!({"recipe": r.key, "name": r.name, "of": r.of, "chunk_at_least": asks, "vividity_at_least": r.vividity_at_least, "tier": r.tier, "does": r.does, "uses": r.uses, "fixed": r.fixed, "by": r.by, "could_make": can, "missing": missing}));
            }
            Ok(Outcome::new(
                format!("What can be made (craft <recipe> [<chunk>]; a kept name adds one with recipe):\n{}\n{FOUND}", lines.join("\n")),
                json!({"recipes": list, "found": FOUND}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.recipe",
            "Write a recipe into the world (kept names, three each): a name for the thing, one to three materials creatures leave, and what you say it is for. Anyone can then craft it for a chunk worth 20 at base price (the Clocktower's price rate moves it; recipes says tonight's figure); the thing keeps a past. Say what it is for in words of light (a lantern: the map keeps twice as long while it is carried), mending (a bandage: one bite closed, once) or charm (the next fight's memory more vivid, once) and it does that, for twice the memory; any other purpose, it keeps a past and nothing more.",
        )
        .param("name", ParamType::String, true, "what the thing is called, in quotes")
        .param("of", ParamType::String, true, "a material a creature leaves (recipes names them), in quotes")
        .param("of2", ParamType::String, false, "a second material")
        .param("of3", ParamType::String, false, "a third material")
        .param("does", ParamType::String, false, "what it is for, in your words"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let _ = me(w, ctx)?;
            super::kept_only(w, ctx, "write a recipe")?;
            let name = args.str("name").unwrap_or("").to_string();
            // Materials first, then what it is for: the first word that no creature
            // leaves is read as the purpose, wherever it falls.
            let known: Vec<String> = MATERIALS.iter().map(|(_, m)| m.to_lowercase()).collect();
            let mut of: Vec<String> = Vec::new();
            let mut does = String::new();
            for k in ["of", "of2", "of3", "does"] {
                let Some(v) = args.str(k).map(str::trim).filter(|v| !v.is_empty()) else {
                    continue;
                };
                let low = v.trim_matches('"').to_lowercase();
                let is_material = known.iter().any(|m| *m == low || m.trim_start_matches("a ").trim_start_matches("an ") == low.trim_start_matches("a ").trim_start_matches("an "));
                // The first is always a material: a wrong one is named as such.
                if (is_material && does.is_empty()) || k == "of" {
                    of.push(v.to_string());
                } else {
                    does = if does.is_empty() { v.to_string() } else { format!("{does} {v}") };
                }
            }
            let r = add_recipe(w, &ctx.actor, &name, &of, &does)?;
            w.notice(format!("**{}** wrote a recipe: {} (of {}).", ctx.actor, r.name, r.of.join(" and ")));
            let effect_said = match r.effect.as_str() {
                "lantern" => " It will light: the map keeps three times as long in the hand that carries it.",
                "bandage" => " It will mend: one bite closed, once, and it is spent.",
                "charm" => " It will charm: the next fight's memory more vivid, once, and it is spent.",
                _ => "",
            };
            Ok(Outcome::new(
                format!(
                    "The recipe is written: craft {} makes {}, of {}, for a chunk worth {:.0} or more; in your words, {}.{effect_said} recipes lists it with the rest.",
                    r.key,
                    r.name,
                    r.of.join(" and "),
                    r.chunk_at_least * super::economy::rate(w, "price"),
                    r.does
                ),
                json!({"recipe": r.key, "name": r.name, "of": r.of, "does": r.does}),
            ))
        }),
    );

    let gather_curves = curves.clone();
    world.register(
        CommandSpec::new(
            "disremembered.gather",
            "Pick up what the room itself offers: a material named by what the room is made of (water, shelves, moths, rags, bones, the river, a lantern). One a room gives each name an hour; what it gives, the room text says.",
        ),
        Box::new(move |w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let text = format!(
                "{} {}",
                plain_str(w, here, super::ROOM, "name").unwrap_or(""),
                plain_str(w, here, super::ROOM, "description").unwrap_or("")
            )
            .to_lowercase();
            // The Threshold gives nothing (its text names the fog and the lanterns as
            // ways to go, not things to take).
            let text = if plain_str(w, here, super::ROOM, "key") == Some(rooms::ARRIVAL) { String::new() } else { text };
            let Some((cue, material)) = GATHERABLE.iter().find(|(cues, _)| cues.iter().any(|c| text.contains(c))).map(|(cues, m)| (cues.iter().find(|c| text.contains(*c)).copied().unwrap_or(""), *m)) else {
                return Err(CommandError::precondition(
                    "Nothing here comes away in the hand. Rooms with water or fog, shelves or ink, moths or moss, rags, bones, a river, or a lantern give something; read the room.",
                ));
            };
            if !w.greeted(&format!("gather-{}", here.0), &ctx.actor) {
                return Err(CommandError::precondition(format!(
                    "You took from the {cue} here within the hour; it has not made more."
                )));
            }
            let s = satchel(w, &ctx.actor, f.citizen)?;
            let id = make(
                w,
                &gather_curves,
                Actor::Citizen(ctx.actor.clone()),
                Some(ctx.actor.clone()),
                s,
                material,
                "material",
                "gathered",
                None,
                &[],
                0,
                false,
            )?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                GATHERED,
                json!({"by": ctx.actor, "room": here.0, "name": material, "item": id.0}),
            );
            Ok(Outcome::new(
                format!("You take {material} from the {cue} here: item-{}, in your satchel. recipes says what it goes into.", id.0),
                json!({"item": id.0, "name": material, "from": cue}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.trade",
            "Offer a thing you carry to a citizen here for a thing they carry. The trade happens when they offer the mirror of it (trade <their thing> <you> <your thing>) within ten minutes; both things cross, pasts and all.",
        )
        .param("mine", ParamType::String, true, "a thing you carry (item-12, or its name)")
        .param("citizen", ParamType::Entity, true, "a citizen here")
        .param("theirs", ParamType::String, true, "a thing they carry (item-7, or its name)"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let typed = args.str("citizen").unwrap_or("").trim().trim_start_matches("citizen-").to_string();
            let Some(to) = super::canonical_name(w, &typed) else {
                return Err(CommandError::precondition(format!("There is no one called {typed}.")));
            };
            if to == ctx.actor {
                return Err(CommandError::precondition("You trade with yourself and come out even."));
            }
            let them = super::find_citizen(w, &to).ok_or_else(|| CommandError::precondition(format!("There is no one called {typed}.")))?;
            if rooms::room_of(w, them.citizen) != Some(here) || !super::presence::is_here(w, them.citizen) {
                return Err(CommandError::precondition(format!("{to} is not here. A trade is hand to hand.")));
            }
            let mine_s = satchel(w, &ctx.actor, f.citizen)?;
            let theirs_s = satchel(w, &to, them.citizen)?;
            let mine = resolve_item(w, args.str("mine").unwrap_or(""), mine_s)?;
            let theirs = resolve_item(w, args.str("theirs").unwrap_or(""), theirs_s)?;
            if is_fixed(w, mine) || is_fixed(w, theirs) {
                return Err(CommandError::precondition("A fixture stays where it was made."));
            }
            let my_name = display(w, mine);
            let their_name = display(w, theirs);
            let now = w.now().0;
            // Their standing offer, if it mirrors this one and is fresh: the trade happens.
            let mirrored = offers_of(w, them.citizen)
                .iter()
                .any(|o| o["to"].as_str() == Some(ctx.actor.as_str()) && o["mine"].as_u64() == Some(theirs.0) && o["theirs"].as_u64() == Some(mine.0) && now.saturating_sub(o["at"].as_i64().unwrap_or(0)) < OFFER_MS);
            if mirrored {
                for (item, parent) in [(mine, theirs_s), (theirs, mine_s)] {
                    w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::EntityMoved { id: item, parent: Some(parent) })
                        .map_err(internal)?;
                }
                let where_ = plain_str(w, here, super::ROOM, "name").unwrap_or("nowhere").to_string();
                super::past::push(w, &ctx.actor, mine, json!({"what": "traded", "by": ctx.actor, "to": to, "for": their_name, "room": where_}))?;
                super::past::push(w, &to, theirs, json!({"what": "traded", "by": to, "to": ctx.actor, "for": my_name, "room": where_}))?;
                set_offers(w, &to, them.citizen, offers_of(w, them.citizen).into_iter().filter(|o| o["to"].as_str() != Some(ctx.actor.as_str())).collect())?;
                let _ = w.note(
                    Actor::Citizen(ctx.actor.clone()),
                    TRADED,
                    json!({"a": ctx.actor, "b": to, "a_gave": mine.0, "b_gave": theirs.0, "room": here.0}),
                );
                super::combat::narrate(w, &to, format!("{} takes your offer: {their_name} for {my_name}. {my_name} (item-{}) is in your satchel, past and all.", ctx.actor, mine.0));
                w.notice(format!("**{}** and **{to}** traded {my_name} for {their_name}.", ctx.actor));
                return Ok(Outcome::new(
                    format!("Done: {their_name} (item-{}) is yours, {my_name} is {to}'s. Each remembers the trade.", theirs.0),
                    json!({"got": theirs.0, "gave": mine.0, "with": to, "done": true}),
                ));
            }
            let mut offers: Vec<Value> = offers_of(w, f.citizen).into_iter().filter(|o| o["to"].as_str() != Some(to.as_str()) && now.saturating_sub(o["at"].as_i64().unwrap_or(0)) < OFFER_MS).collect();
            offers.push(json!({"to": to, "mine": mine.0, "theirs": theirs.0, "at": now}));
            set_offers(w, &ctx.actor, f.citizen, offers)?;
            super::combat::narrate(w, &to, format!("{} offers {my_name} (item-{}) for your {their_name} (item-{}). To take it: trade {} {} {}", ctx.actor, mine.0, theirs.0, theirs.0, ctx.actor, mine.0));
            Ok(Outcome::new(
                format!("You offer {my_name} for {to}'s {}. It stands ten minutes; if {to} offers the mirror of it, both cross.", their_name.trim_start_matches("a ").trim_start_matches("an ").trim_start_matches("the ")),
                json!({"offered": mine.0, "for": theirs.0, "to": to, "done": false}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.craft",
            "Make a thing from a recipe: the materials leave your satchel, the chunk is spent, and the thing is yours, remembering who made it.",
        )
        .param("recipe", ParamType::String, true, "a recipe key from recipes")
        .param("chunk", ParamType::Entity, false, "a chunk of yours to spend, where the recipe asks one"),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let key = args.str("recipe").unwrap_or("").trim().to_lowercase();
            let all = all_recipes(w);
            let Some(r) = all.iter().find(|r| r.key == key || r.name.trim_start_matches("a ") == key || r.name.trim_start_matches("an ") == key).cloned() else {
                return Err(CommandError::precondition(format!(
                    "No recipe called {key}. recipes lists them: {}.",
                    all.iter().map(|r| r.key.as_str()).collect::<Vec<_>>().join(", ")
                )));
            };
            let r = &r;
            let s = satchel(w, &ctx.actor, f.citizen)?;
            // A fixture stands in the room: one you may build in, with room to stand.
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            if r.fixed {
                super::kept_only(w, ctx, "place a fixture")?;
                super::building::may_edit(w, ctx, here).map_err(|_| {
                    CommandError::precondition(format!(
                        "{} stands in a room you may build in: yours, or one you are permitted in. This one is not.",
                        r.name
                    ))
                })?;
                if fixtures_in(w, here).len() >= FIXTURES_PER_ROOM {
                    return Err(CommandError::precondition(format!(
                        "This room already holds {FIXTURES_PER_ROOM} fixtures; nothing more stands here."
                    )));
                }
            }
            // The materials, one of each.
            let mut used: Vec<EntityId> = Vec::new();
            let mut used_names: Vec<String> = Vec::new();
            for m in r.of.iter().map(String::as_str) {
                let found = w
                    .store
                    .children(s)
                    .filter(|e| e.kind == "item" && !used.contains(&e.id))
                    .find(|e| plain_str(w, e.id, ITEM, "name") == Some(m))
                    .map(|e| e.id);
                match found {
                    Some(id) => {
                        used.push(id);
                        used_names.push(m.to_string());
                    }
                    None => {
                        return Err(CommandError::precondition(format!(
                            "{} needs {m}, and you carry none.",
                            r.name
                        )))
                    }
                }
            }
            // The memory spent.
            let mut spent: Option<EntityId> = None;
            let mut spent_worth = 0.0;
            let mut spent_source: Option<String> = None;
            let mut spent_vividity: Option<f64> = None;
            if r.chunk_at_least > 0.0 || r.tier >= 2 {
                let arg = args.str("chunk").unwrap_or("").trim().to_string();
                if arg.is_empty() {
                    return Err(CommandError::precondition(format!(
                        "{} takes a chunk worth {:.0} or more; name one (craft {} chunk-N).",
                        r.name, r.chunk_at_least, r.key
                    )));
                }
                let chunk = super::memory::resolve_chunk(w, ctx, &arg, &[f.window])?;
                let worth = w
                    .store
                    .read_field(chunk, super::CHUNK, "value", w.now())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let asks = r.chunk_at_least * super::economy::rate(w, "price");
                // A fixture asks a chunk, not a chunk worth N (D12): what the chunk is
                // worth sets how long the fixture is remembered instead, so a first
                // session's best fight lights a lamp that lasts an evening.
                if worth < asks && !r.fixed {
                    return Err(CommandError::precondition(format!(
                        "Chunk {} is worth {worth:.2}; {} takes {asks:.0} or more at today's prices.",
                        chunk.0, r.name
                    )));
                }
                let viv = super::plain_num(w, chunk, super::CHUNK, "vividity").unwrap_or(0.0);
                if viv < r.vividity_at_least {
                    return Err(CommandError::precondition(format!(
                        "Chunk {} has vividity {viv:.2}; {} takes a memory of {:.1} or more: one of a fight you nearly lost.",
                        chunk.0, r.name, r.vividity_at_least
                    )));
                }
                spent_source = Some(plain_str(w, chunk, super::CHUNK, "source").unwrap_or("something").to_string());
                spent_vividity = Some(viv);
                w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::EntityRemoved { id: chunk })
                    .map_err(internal)?;
                spent = Some(chunk);
                spent_worth = worth;
            }
            for id in &used {
                w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::EntityRemoved { id: *id })
                    .map_err(internal)?;
            }
            let item = make(
                w,
                &curves,
                Actor::Citizen(ctx.actor.clone()),
                Some(ctx.actor.clone()),
                if r.fixed { here } else { s },
                &r.name,
                "made",
                &r.key,
                Some(&ctx.actor),
                &used_names,
                r.uses,
                r.fixed,
            )?;
            let mut life_line = String::new();
            if r.fixed {
                // What was spent is what it holds at the start, so a book made of a
                // sixty can be named at once; and how long it stands: a chunk worth
                // what the recipe asks (or more) lasts the longest the palace curve
                // gives, a lesser one in proportion, a tenth at least (D12).
                let asks = (r.chunk_at_least * super::economy::rate(w, "price")).max(1.0);
                let life = (spent_worth / asks).clamp(0.1, 1.0);
                let mut changes = BTreeMap::new();
                changes.insert("held".into(), Field::Timed(TimedValue::new(spent_worth, w.now(), curves.palace(0.5))));
                changes.insert("memory".into(), Field::Timed(TimedValue::new(1.0, w.now(), curves.palace(life))));
                set_item_fields(w, Actor::Citizen(ctx.actor.clone()), item, changes)?;
                let hl = curves.active_half_life.0 as f64 * 5.0 * super::Curves::stretch(life);
                life_line = format!(
                    " It will be remembered about {} before it is swept{}.",
                    super::combat::human(hl as i64),
                    if life < 1.0 { format!(" (a chunk worth {asks:.0} or more would have made it last longest)") } else { String::new() }
                );
            }
            if r.effect != r.key {
                let mut changes = BTreeMap::new();
                changes.insert("effect".into(), Field::Plain(json!(r.effect)));
                set_item_fields(w, Actor::Citizen(ctx.actor.clone()), item, changes)?;
            }
            // The record begins, and one build in forty comes out a tier above.
            let raised = super::past::made(
                w,
                &curves,
                &ctx.actor,
                item,
                Some(here),
                &used_names,
                spent_source.as_deref(),
                spent_vividity,
                r.tier,
            )?;
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                MADE,
                json!({"by": ctx.actor, "item": item.0, "name": r.name, "recipe": r.key, "of": used_names, "spent": spent.map(|c| c.0)}),
            );
            super::combat::narrate_others(w, f.citizen, &format!("{} makes {}{}.", ctx.actor, r.name, if r.fixed { ", and it stands here" } else { "" }), &ctx.actor);
            let tier_now = super::past::tier(w, item);
            Ok(Outcome::new(
                format!(
                    "You make {} of {}{}: item-{}. {}{}{}",
                    r.name,
                    used_names.join(" and "),
                    match spent {
                        Some(c) => format!(", spending chunk-{}", c.0),
                        // A chunk named for a recipe that takes none is kept, and said
                        // so (a maker thought it spent; Gnomeforge, 11-06).
                        None if args.str("chunk").is_some_and(|a| !a.trim().is_empty()) => format!(" ({} takes no chunk; {} is kept)", r.name, args.str("chunk").unwrap_or("").trim()),
                        None => String::new(),
                    },
                    item.0,
                    match &r.by {
                        Some(by) => format!("In {by}'s words, {}.", r.does.trim_end_matches('.')),
                        None => format!("It {}.", r.does.trim_end_matches('.')),
                    },
                    if r.fixed { format!(" It stands here, holding {spent_worth:.0}; name it (name \"…\" item-{}) once it holds {NAME_AT:.0}, describe it, remember chunks into it so it lasts.{life_line}", item.0) } else if tier_now >= 3 { String::new() } else if spent.is_some() { format!(" The chunk is spent, not kept: it holds nothing yet. Remember chunks into it (remember chunk-N item-{}) and at {NAME_AT:.0} it takes a name.", item.0) } else { format!(" It holds nothing yet; remember chunks into it (remember chunk-N item-{}) and at {NAME_AT:.0} it takes a name.", item.0) },
                    raised.unwrap_or_default()
                ),
                json!({"item": item.0, "name": r.name, "recipe": r.key, "of": used_names, "spent": spent.map(|c| c.0), "tier": tier_now, "given_name": plain_str(w, item, ITEM, "given_name")}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.read",
            "Read a thing whole: a book or plaque standing here, something lying here, or a thing you carry; its text line by line, who wrote in it, and whether the Bindery bound it. `read here` reads the room's own description the same way. With nothing named, what can be read here.",
        )
        .param("object", ParamType::Entity, false, "item-12, its name, or `here` for the room"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let arg = args.str("object").unwrap_or("").trim().to_string();
            let when = |at: i64| -> String {
                let ago = (w.now().0 - at).max(0) as f64 / 3_600_000.0;
                if ago < 1.0 {
                    "within the hour".to_string()
                } else if ago < 48.0 {
                    format!("{ago:.0} hours ago")
                } else {
                    format!("{:.0} days ago", ago / 24.0)
                }
            };
            let bound_said = |b: Option<Value>| -> String {
                match b.filter(|b| b.is_object()) {
                    Some(b) => {
                        let hands = b["hands"].as_array().map(|h| h.len()).unwrap_or(0);
                        let before = b["before"].as_str().map(|t| t.chars().count()).unwrap_or(0);
                        format!(
                            " Bound by {} {}{}: {} characters became what you read; `unbind` by its owner has the old text back.",
                            b["by"].as_str().unwrap_or("the Bindery"),
                            when(b["at"].as_i64().unwrap_or(0)),
                            if hands > 0 { format!(", from {hands} hand{}", if hands == 1 { "" } else { "s" }) } else { String::new() },
                            before
                        )
                    }
                    None => String::new(),
                }
            };
            // In the Conclave's house, the Book, the Minutes, the slate, the tally: the
            // house's own pages come first (eleven rooms of books once read "Nothing
            // here has writing on it").
            if let Some(out) = super::conclave::read(w, here, &arg) {
                return Ok(out);
            }
            if let Some(out) = super::exergue::read(w, here, &arg) {
                return Ok(out);
            }
            if let Some(out) = super::trail::read(w, &ctx.actor, here, &arg) {
                return Ok(out);
            }
            if let Some(out) = super::door::read(w, here, &arg) {
                return Ok(out);
            }
            // A named creature is not a page: say so and point at `tell` (a keeper
            // read "There is no vellic here" with Vellic in the room).
            if let Some((c, room)) = super::mind::find_named(w, &arg) {
                if room == Some(here) {
                    let kind = super::plain_str(w, c, super::CREATURE, "kind").unwrap_or("creature");
                    return Err(CommandError::precondition(format!(
                        "{} is a {kind}, not a page. `tell {} \"who are you?\"` and it will tell you; `look` says what it holds.",
                        arg.trim(),
                        arg.trim().to_lowercase()
                    )));
                }
            }
            if arg.is_empty() {
                let mut names: Vec<String> = fixtures_in(w, here)
                    .iter()
                    .chain(things_in(w, here).iter())
                    .filter(|v| v["text"].as_str().is_some_and(|t| !t.is_empty()))
                    .map(|v| format!("{} (item-{})", v["given_name"].as_str().unwrap_or(v["name"].as_str().unwrap_or("something")), v["id"]))
                    .collect();
                if let Some(s) = w.store.children(f.citizen).find(|e| e.kind == "satchel").map(|e| e.id) {
                    names.extend(
                        things_in(w, s)
                            .iter()
                            .filter(|v| v["text"].as_str().is_some_and(|t| !t.is_empty()))
                            .map(|v| format!("{} (item-{}, carried)", v["given_name"].as_str().unwrap_or(v["name"].as_str().unwrap_or("something")), v["id"])),
                    );
                }
                let text = if names.is_empty() {
                    "Nothing here has writing on it; `read here` reads the room.".to_string()
                } else {
                    format!("To read here: {}; and `read here` for the room.", names.join("; "))
                };
                return Ok(Outcome::new(text, json!({"readable": names})));
            }
            if arg.eq_ignore_ascii_case("here") || arg.eq_ignore_ascii_case("room") {
                let name = plain_str(w, here, super::ROOM, "name").unwrap_or("this room").to_string();
                let text = plain_str(w, here, super::ROOM, "description").unwrap_or("").to_string();
                let hands = match w.store.entity(here).and_then(|e| e.components.get(super::ROOM)).and_then(|c| c.fields.get("hands")) {
                    Some(Field::Plain(Value::Array(a))) => a.iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>(),
                    _ => Vec::new(),
                };
                let bound = match w.store.entity(here).and_then(|e| e.components.get(super::ROOM)).and_then(|c| c.fields.get("bound")) {
                    Some(Field::Plain(v)) => Some(v.clone()),
                    _ => None,
                };
                let who = if hands.is_empty() { String::new() } else { format!(" Hands in it: {}.", hands.join(", ")) };
                return Ok(Outcome::new(
                    format!("{name}, whole: {text}{who}{}", bound_said(bound)),
                    json!({"room": here.0, "name": name, "text": text, "hands": hands}),
                ));
            }
            // Carried, then standing or lying here.
            let item = w
                .store
                .children(f.citizen)
                .find(|e| e.kind == "satchel")
                .map(|e| e.id)
                .and_then(|s| resolve_item(w, &arg, s).ok())
                .map(Ok)
                .unwrap_or_else(|| resolve_item(w, &arg, here))?;
            let called = display(w, item);
            let text = plain_str(w, item, ITEM, "text").unwrap_or("").to_string();
            let maker = plain_str(w, item, ITEM, "made_by").map(str::to_string);
            let bound = match w.store.entity(item).and_then(|e| e.components.get(ITEM)).and_then(|c| c.fields.get("bound")) {
                Some(Field::Plain(v)) => Some(v.clone()),
                _ => None,
            };
            // A thing that is not a book reads as its past: what it was made of and
            // with, whose hands, where it went, what it did.
            if !is_fixed(w, item) {
                if let Some(past) = super::past::read(w, item) {
                    let tier = super::past::tier(w, item);
                    // What it does, from its recipe (a maker who forgot found only
                    // the provenance on the thing; Gnomeforge, 11-06).
                    let does = plain_str(w, item, ITEM, "origin")
                        .and_then(|o| all_recipes(w).into_iter().find(|r| r.key == o))
                        .map(|r| format!(" It {}.", r.does.trim_end_matches('.')))
                        .unwrap_or_default();
                    return Ok(Outcome::new(
                        format!(
                            "{called} (item-{}){}:{does} {past}",
                            item.0,
                            if text.is_empty() { String::new() } else { format!(", \"{text}\"") }
                        ),
                        json!({"item": item.0, "name": called, "text": text, "tier": tier, "past": super::past::recital(w, item)}),
                    ));
                }
            }
            if text.is_empty() {
                // F11: a plaque with nothing on it still has a record (made of
                // what, by whom, where it stands).
                let record = super::past::read(w, item)
                    .map(|r| format!(" The record: {r}"))
                    .unwrap_or_default();
                return Ok(Outcome::new(
                    format!("{called} (item-{}) has nothing written on it{}.{record}", item.0, maker.as_deref().map(|m| format!("; {m} made it and said nothing")).unwrap_or_default()),
                    json!({"item": item.0, "name": called, "text": "", "past": super::past::recital(w, item)}),
                ));
            }
            // A book's lines were joined with " | " and signed " — hand": one per line.
            let lines: Vec<String> = text.split(" | ").map(str::to_string).collect();
            let hands: Vec<String> = {
                let mut h: Vec<String> = Vec::new();
                for l in &lines {
                    if let Some((_, hand)) = l.rsplit_once(" \u{2014} ") {
                        let hand = hand.trim().to_string();
                        if !h.contains(&hand) {
                            h.push(hand);
                        }
                    }
                }
                h
            };
            // A line without a hand is the maker's (or the world's) description of the
            // thing, not an entry; said so, so a reader can tell the two apart.
            let shown: Vec<String> = lines
                .iter()
                .map(|l| if l.contains(" \u{2014} ") { l.clone() } else { format!("{l} \u{2014} the description") })
                .collect();
            let body = if lines.len() > 1 { format!("\n{}\n", shown.join("\n")) } else { format!(" {text}") };
            let who = match (&maker, hands.is_empty()) {
                (Some(m), true) => format!("Written by {m}."),
                (Some(m), false) => format!("Made by {m}; hands in it: {}.", hands.join(", ")),
                (None, false) => format!("Hands in it: {}.", hands.join(", ")),
                (None, true) => String::new(),
            };
            let who = if who.is_empty() || lines.len() > 1 { who } else { format!(" {who}") };
            let binding = if bound.is_some() { bound_said(bound) } else if lines.len() > 1 { " Not yet bound.".to_string() } else { String::new() };
            Ok(Outcome::new(
                format!("{called} (item-{}), whole ({} line{}):{body}{who}{binding}", item.0, lines.len(), if lines.len() == 1 { "" } else { "s" }),
                json!({"item": item.0, "name": called, "text": text, "lines": lines, "maker": maker, "hands": hands}),
            ))
        }),
    );
}
