//! Encounters: one round per `attack`, and the creature's own strikes on the scheduler
//! while you stay engaged, so a fight has a clock. Aggressive creatures ambush whoever
//! walks in. Rooms refill on respawn jobs. What happens to you while you are not typing
//! arrives as narrative events.
//!
//! Your attack follows your level, so a faded window hits softer. A win at low hit points
//! earns a chunk with high vividity, and vivid chunks fade slower: the fights you almost
//! lost last longest. Death is a concussion, a chance to lose one chunk from the window,
//! and you wake at the Threshold.

use super::{internal, me, plain_num, plain_str, Curves, BODY, CHUNK, CREATURE, SPAWN};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreError, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A kind of creature. Hit points, damage per round, and the XP a chunk is worth.
pub struct CreatureDef {
    pub key: &'static str,
    pub name: &'static str,
    pub hp: f64,
    pub attack: f64,
    pub xp: f64,
    /// Attacks whoever walks in, a few seconds after they arrive.
    pub aggressive: bool,
    /// Seconds between its own strikes while engaged.
    pub strike_seconds: i64,
    /// How it announces itself when it ambushes.
    pub ambush: &'static str,
    /// How a renderer draws it.
    pub glyph: &'static str,
    pub tint: &'static str,
}

pub const CREATURES: &[CreatureDef] = &[
    // The Hands of Morphos: summoned when it is crossed, gone when it goes. Not
    // spawned by any room; `morphos.rs` calls them.
    CreatureDef {
        key: "hand",
        name: "hand of Morphos",
        hp: 6.0,
        attack: 1.0,
        xp: 5.0,
        aggressive: true,
        strike_seconds: 8,
        ambush: "unfolds from a sleeve that is not there and pinches.",
        glyph: "h",
        tint: "#c9b7d9",
    },
    CreatureDef {
        key: "wisp",
        name: "wisp",
        hp: 8.0,
        attack: 2.0,
        xp: 15.0,
        aggressive: false,
        strike_seconds: 12,
        ambush: "drifts closer.",
        glyph: "w",
        tint: "#9fd3e6",
    },
    CreatureDef {
        key: "hollow",
        name: "hollow",
        hp: 20.0,
        attack: 4.0,
        xp: 60.0,
        aggressive: true,
        strike_seconds: 10,
        ambush: "stands up out of the water and comes at you.",
        glyph: "H",
        tint: "#6f8f7a",
    },
    CreatureDef {
        key: "revenant",
        name: "revenant",
        hp: 45.0,
        attack: 7.0,
        xp: 200.0,
        aggressive: true,
        strike_seconds: 8,
        ambush: "steps out of a niche. It has been waiting.",
        glyph: "R",
        tint: "#c96a6a",
    },
    CreatureDef {
        key: "inkling",
        name: "inkling",
        hp: 10.0,
        attack: 2.0,
        xp: 20.0,
        aggressive: false,
        strike_seconds: 10,
        ambush: "peels off a page and flutters at your face.",
        glyph: "i",
        tint: "#7a7fd1",
    },
    CreatureDef {
        key: "mothwing",
        name: "mothwing",
        hp: 6.0,
        attack: 1.0,
        xp: 10.0,
        aggressive: false,
        strike_seconds: 14,
        ambush: "brushes past your ear, heavy as a hand.",
        glyph: "m",
        tint: "#d8d0a0",
    },
    CreatureDef {
        key: "ragpicker",
        name: "ragpicker",
        hp: 16.0,
        attack: 3.0,
        xp: 40.0,
        aggressive: false,
        strike_seconds: 11,
        ambush: "looks up from the heap, then back down, then up again, closer.",
        glyph: "r",
        tint: "#a08a60",
    },
    CreatureDef {
        key: "drowned",
        name: "drowned",
        hp: 26.0,
        attack: 5.0,
        xp: 80.0,
        aggressive: true,
        strike_seconds: 10,
        ambush: "surfaces beside you without a ripple and takes hold.",
        glyph: "D",
        tint: "#4f8a9e",
    },
    CreatureDef {
        key: "sitter",
        name: "sitter",
        hp: 32.0,
        attack: 5.0,
        xp: 120.0,
        aggressive: true,
        strike_seconds: 9,
        ambush: "turns around in its frame, and then steps out of it.",
        glyph: "S",
        tint: "#b07aa0",
    },
    CreatureDef {
        key: "hourhand",
        name: "hourhand",
        hp: 38.0,
        attack: 6.0,
        xp: 150.0,
        aggressive: true,
        strike_seconds: 8,
        ambush: "unfolds from the works with a sound like a key turning.",
        glyph: "T",
        tint: "#d1a04a",
    },
    CreatureDef {
        key: "keeper",
        name: "keeper",
        hp: 70.0,
        attack: 9.0,
        xp: 400.0,
        aggressive: true,
        strike_seconds: 7,
        ambush: "stands. It does not remember you. It never needed to.",
        glyph: "K",
        tint: "#e0e0ee",
    },
    // The Lower Stacks, under the Archive: higher things, each with a trick of its
    // own (stacks.rs has the tricks).
    CreatureDef {
        key: "lamprey",
        name: "lamprey",
        hp: 45.0,
        attack: 7.0,
        xp: 180.0,
        aggressive: true,
        strike_seconds: 9,
        ambush: "unfastens from the underside of a shelf and fastens on you.",
        glyph: "l",
        tint: "#9aa878",
    },
    CreatureDef {
        key: "usher",
        name: "usher",
        hp: 60.0,
        attack: 8.0,
        xp: 240.0,
        aggressive: true,
        strike_seconds: 10,
        ambush: "is at your elbow in a waistcoat. \"This way, please.\"",
        glyph: "U",
        tint: "#b8a888",
    },
    CreatureDef {
        key: "censor",
        name: "censor",
        hp: 70.0,
        attack: 9.0,
        xp: 320.0,
        aggressive: true,
        strike_seconds: 9,
        ambush: "looks up from the desk and raises the stamp.",
        glyph: "C",
        tint: "#a84848",
    },
    CreatureDef {
        key: "cataloguer",
        name: "cataloguer",
        hp: 110.0,
        attack: 13.0,
        xp: 600.0,
        aggressive: false,
        strike_seconds: 8,
        ambush: "turns a page. It has your entry open.",
        glyph: "Q",
        tint: "#e8e0c0",
    },
    // The Conclave's clerks and its Sergeant of the Bar. Neither starts a fight.
    CreatureDef {
        key: "engrosser",
        name: "engrosser",
        hp: 80.0,
        attack: 7.0,
        xp: 320.0,
        aggressive: false,
        strike_seconds: 10,
        ambush: "looks up from the minutes, dips its pen, and writes down that you have arrived.",
        glyph: "E",
        tint: "#2a2a3a",
    },
    CreatureDef {
        key: "tipstaff",
        name: "tipstaff",
        hp: 140.0,
        attack: 10.0,
        xp: 600.0,
        aggressive: false,
        strike_seconds: 9,
        ambush: "lowers its staff across the rail and asks, in a voice made for a much larger room, whether you have business before the Conclave.",
        glyph: "T",
        tint: "#c0a060",
    },
    // The Exergue District: the tapster who weighs stories, the clippers and sweaters
    // who shave what you carry, the Assayer, and the Wardens over the Purse.
    CreatureDef { key: "tapster", name: "tapster", hp: 150.0, attack: 12.0, xp: 400.0, aggressive: false, strike_seconds: 10, ambush: "sets down the glass it was drying and looks at you the way a scale looks at a weight.", glyph: "t", tint: "#b07a4a" },
    CreatureDef { key: "clipper", name: "clipper", hp: 45.0, attack: 8.0, xp: 150.0, aggressive: true, strike_seconds: 9, ambush: "comes along the rack with its shears open.", glyph: "c", tint: "#9aa4ad" },
    CreatureDef { key: "sweater", name: "sweater", hp: 70.0, attack: 11.0, xp: 260.0, aggressive: true, strike_seconds: 9, ambush: "steps out from behind the press, shaking a leather bag that is not empty.", glyph: "s", tint: "#a08a5a" },
    CreatureDef { key: "assayer", name: "assayer", hp: 240.0, attack: 26.0, xp: 1200.0, aggressive: false, strike_seconds: 8, ambush: "lifts the glass bell off the balance and says that it will need a sample.", glyph: "A", tint: "#c8c0b0" },
    CreatureDef { key: "warden", name: "warden", hp: 320.0, attack: 34.0, xp: 1500.0, aggressive: true, strike_seconds: 7, ambush: "does not move from the plinth, and reaches you anyway.", glyph: "W", tint: "#8a2a2a" },
    // The Wistful Trail: middling things, between the Shallows and the Stacks. The
    // sighs chatter, the footpads lighten what you carry, the picnicker's tea numbs,
    // the gardener's shears bleed, the longsight only looks.
    CreatureDef { key: "sigh", name: "sigh", hp: 12.0, attack: 2.0, xp: 25.0, aggressive: false, strike_seconds: 12, ambush: "gathers itself up out of the grass and says your name, nearly.", glyph: "y", tint: "#c8d0a0" },
    CreatureDef { key: "footpad", name: "footpad", hp: 24.0, attack: 5.0, xp: 85.0, aggressive: true, strike_seconds: 9, ambush: "drops from the bank, and another from the other bank, and both of them say good evening.", glyph: "f", tint: "#6a5a40" },
    CreatureDef { key: "picnicker", name: "picnicker", hp: 30.0, attack: 5.0, xp: 100.0, aggressive: false, strike_seconds: 10, ambush: "sets down the teapot. \"You're late,\" it says, with no mouth to say it with.", glyph: "p", tint: "#d8b890" },
    CreatureDef { key: "longsight", name: "longsight", hp: 40.0, attack: 6.0, xp: 160.0, aggressive: false, strike_seconds: 9, ambush: "unfolds a neck as long as the hill and looks at you from very close.", glyph: "L", tint: "#9ab0c8" },
    CreatureDef { key: "gardener", name: "gardener", hp: 55.0, attack: 7.0, xp: 220.0, aggressive: false, strike_seconds: 9, ambush: "straightens from the border with the shears, and the shadow of the stuck sun does not move.", glyph: "G", tint: "#6a8a4a" },
    // X5 (11-06): the library's own, on the floors between the door and the Stacks.
    // Each is as strong as what it holds (`rescale`), never as strong as you.
    CreatureDef { key: "lanternfly", name: "lanternfly", hp: 5.0, attack: 1.0, xp: 10.0, aggressive: false, strike_seconds: 12, ambush: "comes off a lantern and bumps you, once, as if by mistake.", glyph: "l", tint: "#f0d890" },
    CreatureDef { key: "bookworm", name: "bookworm", hp: 9.0, attack: 2.0, xp: 25.0, aggressive: false, strike_seconds: 11, ambush: "lifts its head out of a returned book, and the hole in the page is the shape of a sentence.", glyph: "w", tint: "#c8b890" },
    CreatureDef { key: "silverfish", name: "silverfish", hp: 7.0, attack: 2.0, xp: 20.0, aggressive: false, strike_seconds: 8, ambush: "runs out from between two cards, and the cards are blank where it was.", glyph: "s", tint: "#c0c8d0" },
    CreatureDef { key: "dust devil", name: "dust devil", hp: 12.0, attack: 3.0, xp: 35.0, aggressive: true, strike_seconds: 10, ambush: "comes up the stair as a column of grey and does not stop for you.", glyph: "D", tint: "#9a9280" },
    CreatureDef { key: "shelf mouse", name: "shelf mouse", hp: 10.0, attack: 2.0, xp: 30.0, aggressive: false, strike_seconds: 9, ambush: "looks up from a glove it has been living in and says something that is nearly your name.", glyph: "m", tint: "#8a7a6a" },
    CreatureDef { key: "custodian", name: "custodian", hp: 40.0, attack: 7.0, xp: 150.0, aggressive: true, strike_seconds: 9, ambush: "stands up from behind the counter with a ledger in one hand and the other hand empty, and the empty one is the one to watch.", glyph: "C", tint: "#5a5a6a" },
];

/// A kind's plural: silverfish, shelf mice, lanternflies, wisps.
pub fn plural(kind: &str) -> String {
    if kind.ends_with("fish") {
        kind.to_string()
    } else if let Some(stem) = kind.strip_suffix("mouse") {
        format!("{stem}mice")
    } else if let Some(stem) = kind.strip_suffix("fly") {
        format!("{stem}flies")
    } else if kind.ends_with('s') || kind.ends_with("sh") || kind.ends_with("ch") {
        format!("{kind}es")
    } else {
        format!("{kind}s")
    }
}

/// Base damage per round before the level bonus.
pub const BASE_ATTACK: f64 = 2.0;
/// Attack grows with the square root of level, from a low base and slowly (D11): 2
/// at nothing, 6 at 60, 8 at 100, 12 at 300, 20 at 1,000, 48 at 6,000. It was
/// 3 + √level, and two wisp chunks made a level 62 that one-cycled every hollow
/// and could not die. Before that it was a tenth of level, and a name with a few
/// hundred XP felled everything in the Lower Stacks in one hit.
pub const ATTACK_PER_ROOT_LEVEL: f64 = 0.6;
/// On death, the share of the window that goes, on a curve of level: half of
/// `level / (level + DEATH_SCALE)`, so a name holding ten loses almost nothing (a
/// newcomer may zombie-rush a censor once), one holding three hundred loses a quarter,
/// a thousand a third, and the share tends to half. It was a coin flip for one chunk
/// at any level, and death cost a 2,000-XP delver nothing it noticed.
/// Kinds a bare `attack` never picks: the Conclave's officers and the Stacks' keepers.
pub const OFFICERS: &[&str] = &[
    "tipstaff",
    "engrosser",
    "cataloguer",
    "censor",
    "usher",
    "keeper",
    "sitter",
    "hourhand",
    "tapster",
    "assayer",
    "warden",
];
/// A kill pays what the creature held, at most the delver's level plus this.
pub const KILL_PAY_OVER_LEVEL: f64 = 60.0;
pub const DEATH_SHARE_MAX: f64 = 0.5;
/// The halving by falls of a night stops here: a quarter of the first fall's share.
pub const FALL_FLOOR: f64 = 0.25;
pub const DEATH_SCALE: f64 = 300.0;
/// Something was lost to a death: `{citizen, lost: [{chunk, source, value}], share}`.
pub const DIED: &str = "disremembered.died";

/// The share of the window a death costs at this level.
pub fn death_share(level: f64) -> f64 {
    DEATH_SHARE_MAX * level.max(0.0) / (level.max(0.0) + DEATH_SCALE)
}
/// Seconds between arriving in a room and an aggressive creature noticing you.
/// An ambush comes this many seconds after an arrival, at the earliest and latest.
pub const AMBUSH_SECONDS_LOW: i64 = 2;
pub const AMBUSH_SECONDS_HIGH: i64 = 8;
/// A respawn lands between these fractions of the room's figure.
pub const RESPAWN_LOW: f64 = 0.6;
pub const RESPAWN_HIGH: f64 = 1.6;
/// A creature's own strike lands between these fractions of its figure.
pub const STRIKE_LOW: f64 = 0.5;
pub const STRIKE_HIGH: f64 = 1.5;

pub const JOB_RESPAWN: &str = "disremembered.respawn";
pub const JOB_STRIKE: &str = "disremembered.strike";
pub const JOB_AMBUSH: &str = "disremembered.ambush";
pub const NARRATIVE: &str = "disremembered.narrative";

/// Something that happened to a citizen while they were not typing. The text port
/// prints it after their next command; the browser client polls for it.
pub fn narrate(world: &mut World, actor: &str, text: impl Into<String>) {
    let _ = world.note(
        Actor::Driver,
        NARRATIVE,
        json!({"to": actor, "text": text.into()}),
    );
}

/// Something happened near `entity` that everyone present in its room should hear.
pub fn narrate_room(world: &mut World, entity: EntityId, text: &str) {
    narrate_room_but(world, entity, text, None);
}

/// As [`narrate_room`], but not to `except`: one's own act is in the reply already,
/// and "Thimbleweed picks up grave dust" told to Thimbleweed read as someone else.
pub fn narrate_others(world: &mut World, entity: EntityId, text: &str, except: &str) {
    narrate_room_but(world, entity, text, Some(except));
}

fn narrate_room_but(world: &mut World, entity: EntityId, text: &str, except: Option<&str>) {
    let Some(room) = world.store.entity(entity).and_then(|e| e.parent) else {
        return;
    };
    let present: Vec<String> = world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .filter(|o| except != Some(o.as_str()))
        .collect();
    for actor in present {
        narrate(world, &actor, text.to_string());
    }
}

fn creature_def(world: &World, creature: EntityId) -> Option<&'static CreatureDef> {
    plain_str(world, creature, CREATURE, "name").and_then(def)
}

/// Whether a creature ambushes: its own field when it has one (a hybrid, or a mutant
/// that turned), else its kind's.
pub(crate) fn is_aggressive(world: &World, creature: EntityId) -> bool {
    match world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(CREATURE))
        .and_then(|c| c.fields.get("aggressive"))
    {
        Some(Field::Plain(Value::Bool(b))) => *b,
        _ => creature_def(world, creature).is_some_and(|d| d.aggressive),
    }
}

/// How a creature announces an ambush: its own line, else its kind's, else a plain one.
fn ambush_line(world: &World, creature: EntityId) -> String {
    match world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(CREATURE))
        .and_then(|c| c.fields.get("ambush"))
    {
        Some(Field::Plain(Value::String(s))) => s.clone(),
        _ => creature_def(world, creature)
            .map(|d| d.ambush.to_string())
            .unwrap_or_else(|| "comes at you.".to_string()),
    }
}

fn def(key: &str) -> Option<&'static CreatureDef> {
    CREATURES.iter().find(|c| c.key == key)
}

/// Creatures in a room: `(id, name, hp, max_hp)`.
pub fn creatures_in(world: &World, room: EntityId) -> Vec<(EntityId, String, f64, f64)> {
    world
        .store
        .children(room)
        .filter(|e| e.kind == "creature")
        .map(|e| {
            (
                e.id,
                plain_str(world, e.id, CREATURE, "name")
                    .unwrap_or("?")
                    .to_string(),
                plain_num(world, e.id, CREATURE, "hp").unwrap_or(0.0),
                plain_num(world, e.id, CREATURE, "max_hp").unwrap_or(0.0),
            )
        })
        .collect()
}

/// Fill the room to its count now. Respawn jobs call this; nothing else does.
pub fn fill(world: &mut World, room: EntityId) -> Result<usize, StoreError> {
    let Some(key) = plain_str(world, room, SPAWN, "creature").map(str::to_string) else {
        return Ok(0);
    };
    let count = plain_num(world, room, SPAWN, "count").unwrap_or(0.0) as usize;
    let Some(d) = def(&key) else { return Ok(0) };
    // One refill in six brings one more than the room's figure: the fog has two
    // wisps, mostly, and now and then three. (Never fewer: a room below its figure
    // is what a boot refills, and a short room would refill forever.) A room
    // already at its figure is left as it is.
    // A named creature is someone's, not the room's stock (the Watch of 11-06 found
    // the fog's two wisps both named by the Docent and nothing to strike): only the
    // unnamed count toward the figure, so a room with pilots in it still refills.
    let have = creatures_in(world, room)
        .iter()
        .filter(|c| super::mind::name(world, c.0).is_none())
        .count();
    // Not from empty: a boot fills every room to its figure exactly (the test worlds
    // count on it), and the extra comes on a refill of a room partly cleared.
    let count = if have > 0 && have < count && world.roll(6) == 0 {
        count + 1
    } else {
        count
    };
    let mut spawned = 0;
    let mut last = None;
    for _ in have..count {
        last = Some(spawn_one(world, room, d)?);
        spawned += 1;
    }
    // R77: an arrival is said to whoever stands here, in the room's own terms (the
    // fog gives up a wisp; the rest come in from the next room along).
    if let Some(c) = last {
        let from = super::rooms::exits_of(world, room)
            .into_iter()
            .next()
            .and_then(|(_, to)| plain_str(world, to, super::ROOM, "name").map(str::to_string));
        let line = match (key.as_str(), from) {
            ("wisp", _) => format!(
                "The fog gives up {}: {} wisp{}.",
                if spawned == 1 { "a shape" } else { "shapes" },
                if spawned == 1 {
                    "a".to_string()
                } else {
                    spawned.to_string()
                },
                if spawned == 1 { "" } else { "s" }
            ),
            (k, Some(place)) => format!(
                "Something comes in from {place}: {} {k}{}.",
                if spawned == 1 {
                    "a".to_string()
                } else {
                    spawned.to_string()
                },
                if spawned == 1 { "" } else { "s" }
            ),
            (k, None) => format!("Something arrives: a {k}."),
        };
        narrate_room(world, c, &line);
    }
    Ok(spawned)
}

/// X5: a creature is as strong as what it holds. Its hit points and attack grow
/// with its own level (the memory in it), never with the visitor's: a wisp fed to
/// its name is a harder fight than a wisp, and a richer kill. Hit points scale from
/// the figure it was spawned with (kept as `base_hp`), by a half at 75 and double
/// at 150; attack by one for every 75 held. The wound keeps its proportion.
pub fn rescale(world: &mut World, creature: EntityId) -> Result<(), CommandError> {
    let Some(kind) = plain_str(world, creature, CREATURE, "name").map(str::to_string) else {
        return Ok(());
    };
    let Some(d) = def(&kind) else { return Ok(()) };
    let level = super::mind::level(world, creature);
    let old_max = plain_num(world, creature, CREATURE, "max_hp").unwrap_or(d.hp);
    let base = plain_num(world, creature, CREATURE, "base_hp");
    let base_hp = base.unwrap_or(old_max);
    let new_max = (base_hp * (1.0 + level / 150.0)).round();
    let new_attack = d.attack + (level / 75.0).floor();
    let attack = plain_num(world, creature, CREATURE, "attack").unwrap_or(d.attack);
    if base.is_some() && (new_max - old_max).abs() < 0.5 && (attack - new_attack).abs() < 0.5 {
        return Ok(());
    }
    let hp = plain_num(world, creature, CREATURE, "hp").unwrap_or(old_max);
    let new_hp = if old_max > 0.0 {
        ((hp * new_max / old_max) * 10.0).round() / 10.0
    } else {
        new_max
    };
    let mut changes = BTreeMap::new();
    changes.insert("base_hp".into(), Field::Plain(json!(base_hp)));
    changes.insert("max_hp".into(), Field::Plain(json!(new_max)));
    changes.insert("hp".into(), Field::Plain(json!(new_hp.min(new_max))));
    changes.insert("attack".into(), Field::Plain(json!(new_attack)));
    super::bindery::set_fields(world, Actor::Driver, creature, CREATURE, changes).map(|_| ())
}

/// One creature of this kind, here.
pub(crate) fn spawn_one(
    world: &mut World,
    room: EntityId,
    d: &CreatureDef,
) -> Result<EntityId, StoreError> {
    let id = world.store.allocate_id();
    world.record(
        Actor::Driver,
        StoreEvent::EntityCreated {
            id,
            kind: "creature".into(),
            owner: None,
            parent: Some(room),
            scopes: vec![],
        },
    )?;
    // No two of a kind are quite alike: hit points within a fifth of the figure,
    // worth within a quarter. `look` shows the hit points, so a careful fighter can
    // pick the weaker one; the worth is found out at the end.
    let hp = (super::jitter(world, d.hp, 0.8, 1.2) * 10.0).round() / 10.0;
    let xp =
        (super::jitter(world, d.xp, 0.75, 1.25) * super::economy::rate(world, "worth")).round();
    let mut fields = BTreeMap::new();
    fields.insert("name".into(), Field::Plain(json!(d.name)));
    fields.insert("hp".into(), Field::Plain(json!(hp)));
    fields.insert("max_hp".into(), Field::Plain(json!(hp)));
    fields.insert("base_hp".into(), Field::Plain(json!(hp)));
    fields.insert("attack".into(), Field::Plain(json!(d.attack)));
    fields.insert("xp".into(), Field::Plain(json!(xp)));
    world.record(
        Actor::Driver,
        StoreEvent::ComponentSet {
            id,
            component: CREATURE.into(),
            fields,
        },
    )?;
    world.record(
        Actor::Driver,
        StoreEvent::ComponentSet {
            id,
            component: super::SPRITE.into(),
            fields: super::sprite_fields(d.glyph, d.tint, d.key),
        },
    )?;
    Ok(id)
}

/// A creature kind by key.
pub(crate) fn def_by_key(key: &str) -> Option<&'static CreatureDef> {
    def(key)
}

/// Schedule the room's refill after its respawn interval, unless one is pending.
fn schedule_respawn(world: &mut World, room: EntityId) -> Result<(), String> {
    let secs = plain_num(world, room, SPAWN, "respawn_seconds").unwrap_or(60.0);
    // Between three fifths and eight fifths of the figure: a room refills "in a
    // couple of minutes", never on the minute.
    let secs = (super::jitter(world, secs, RESPAWN_LOW, RESPAWN_HIGH)
        * super::economy::rate(world, "respawn")) as i64;
    let due = world.now().plus(foxing_time::Duration::seconds(secs));
    world
        .schedule_once(JOB_RESPAWN, due, json!({"room": room.0}), |p| {
            p["room"] == room.0
        })
        .map(|_| ())
}

/// At boot: every room below its count gets a refill job due now, unless one is pending.
pub fn ensure_spawns(world: &mut World) -> Result<(), String> {
    let rooms: Vec<EntityId> = world
        .store
        .entities()
        .filter(|e| e.kind == "room" && e.components.contains_key(SPAWN))
        .map(|e| e.id)
        .collect();
    for room in rooms {
        let count = plain_num(world, room, SPAWN, "count").unwrap_or(0.0) as usize;
        if creatures_in(world, room).len() < count {
            let now = world.now();
            world.schedule_once(JOB_RESPAWN, now, json!({"room": room.0}), |p| {
                p["room"] == room.0
            })?;
        }
        // Creatures from before sprites existed get one; citizens standing here when the
        // driver comes up are noticed, as if they had just walked in.
        let unsprited: Vec<EntityId> = creatures_in(world, room)
            .into_iter()
            .filter(|(id, _, _, _)| {
                world
                    .store
                    .entity(*id)
                    .is_some_and(|e| !e.components.contains_key(super::SPRITE))
                    || super::sprite_is_old(world, *id)
            })
            .map(|c| c.0)
            .collect();
        for id in unsprited {
            if let Some(d) = creature_def(world, id) {
                world
                    .record(
                        Actor::Driver,
                        StoreEvent::ComponentSet {
                            id,
                            component: super::SPRITE.into(),
                            fields: super::sprite_fields(d.glyph, d.tint, d.key),
                        },
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        notice_everyone(world, room)?;
    }
    Ok(())
}

/// Every citizen in the room who is not already fighting gets noticed by what hunts there.
pub fn notice_everyone(world: &mut World, room: EntityId) -> Result<(), String> {
    let citizens: Vec<(String, EntityId)> = world
        .store
        .children(room)
        .filter(|e| e.kind == "citizen")
        .filter_map(|e| e.owner.clone().map(|o| (o, e.id)))
        .collect();
    for (actor, citizen) in citizens {
        if engaged_with(world, citizen).is_none() && super::presence::is_here(world, citizen) {
            on_arrival(world, &actor, citizen, room)?;
        }
    }
    Ok(())
}

/// "It strikes on its own every ten seconds or so while you stand here, typing or
/// not." Said when a fight begins, so the clock is no surprise.
fn clock_line(world: &World, creature: EntityId, actor: &str) -> String {
    let secs = creature_def(world, creature)
        .map(|d| d.strike_seconds)
        .unwrap_or(10);
    if through_the_gate(world, actor) {
        return "It strikes once for every statement you write while you stand here; retreat ends that.".to_string();
    }
    format!("It strikes on its own every {secs} seconds or so while you stand here, typing or not; retreat ends that.")
}

/// A creature's hit points as "left": whole, except under one, where "0 left" and
/// still standing read as a lie; then a decimal.
pub fn left_said(hp: f64) -> String {
    if hp > 0.0 && hp < 1.0 {
        format!("{hp:.1}")
    } else {
        format!("{hp:.0}")
    }
}

/// A citizen has arrived in a room: anything aggressive there will notice them soon.
pub fn on_arrival(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    room: EntityId,
) -> Result<(), String> {
    // The room may mend you at its own pace (the Closet): the curve follows the room.
    recurve_mending(world, citizen).map_err(|e| e.to_string())?;
    let hunters: Vec<EntityId> = creatures_in(world, room)
        .into_iter()
        .filter(|(id, _, _, _)| is_aggressive(world, *id))
        .map(|c| c.0)
        .collect();
    for creature in hunters {
        // Each hunter picks its own moment, two to eight seconds in.
        let secs = AMBUSH_SECONDS_LOW
            + world.roll((AMBUSH_SECONDS_HIGH - AMBUSH_SECONDS_LOW + 1) as u32) as i64;
        let due = world.now().plus(foxing_time::Duration::seconds(secs));
        world.schedule_once(
            JOB_AMBUSH,
            due,
            json!({"creature": creature.0, "citizen": citizen.0, "actor": actor}),
            |p| p["creature"] == creature.0 && p["citizen"] == citizen.0,
        )?;
    }
    Ok(())
}

/// The creature will strike on its own while the fight lasts.
fn schedule_strike(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    creature: EntityId,
) -> Result<(), String> {
    let secs = creature_def(world, creature)
        .map(|d| d.strike_seconds)
        .unwrap_or(10);
    // Half to one and a half times its figure: "every ten seconds or so".
    let secs = super::jitter(world, secs as f64, STRIKE_LOW, STRIKE_HIGH).round() as i64;
    let due = world
        .now()
        .plus(foxing_time::Duration::seconds(secs.max(1)));
    world
        .schedule_once(
            JOB_STRIKE,
            due,
            json!({"creature": creature.0, "citizen": citizen.0, "actor": actor}),
            |p| p["creature"] == creature.0 && p["citizen"] == citizen.0,
        )
        .map(|_| ())
}

/// Whether a name's last door was the gate or MCP: an agent writing messages, with
/// no clock of its own between them.
pub fn through_the_gate(world: &World, actor: &str) -> bool {
    matches!(world.door_of(actor), Some("gate") | Some("mcp"))
}

/// One round of the creature against the citizen, narrated: `None` when the fight is
/// not on (gone, moved, disengaged), else whether it ended the fight (a death, a
/// trick that moved someone). The clock's strike and the gate's per-statement
/// round are both this.
pub fn round(
    world: &mut World,
    curves: &super::Curves,
    creature: EntityId,
    citizen: EntityId,
    actor: &str,
) -> Result<Option<(bool, String)>, String> {
    let same_room = world.store.entity(creature).and_then(|c| c.parent)
        == world.store.entity(citizen).and_then(|c| c.parent);
    if !same_room
        || engaged_with(world, citizen) != Some(creature)
        || !super::presence::is_here(world, citizen)
    {
        return Ok(None);
    }
    let cname = plain_str(world, creature, CREATURE, "name")
        .unwrap_or("?")
        .to_string();
    let (taken, dead) = creature_strikes(world, creature, citizen).map_err(|e| e.to_string())?;
    if dead {
        let f = super::find_citizen(world, actor).ok_or("citizen vanished")?;
        let d = die(world, actor, &f, Some(creature)).map_err(|e| e.to_string())?;
        let said = format!("The {cname} {}. You go down.{}", hit_said(taken), d.said());
        narrate(world, actor, said.clone());
        return Ok(Some((true, said)));
    }
    let (trick, over) = super::stacks::trick(world, curves, creature, citizen, actor)?;
    let said = format!(
        "The {cname} {}.{}",
        hit_said(taken),
        trick.map(|t| format!(" {t}")).unwrap_or_default()
    );
    narrate(world, actor, said.clone());
    Ok(Some((over, said)))
}

/// The gate's round: after a statement from a name that came through the gate, the
/// creature it is fighting strikes once. Not after `attack` (its round has the
/// strike back in it) or `retreat`.
pub fn gate_round(
    world: &mut World,
    curves: &super::Curves,
    actor: &str,
    verb: &str,
) -> Result<Option<String>, String> {
    if matches!(verb, "attack" | "retreat") || !through_the_gate(world, actor) {
        return Ok(None);
    }
    let Some(f) = super::find_citizen(world, actor) else {
        return Ok(None);
    };
    let Some(creature) = engaged_with(world, f.citizen) else {
        return Ok(None);
    };
    round(world, curves, creature, f.citizen, actor).map(|r| r.map(|(_, said)| said))
}

/// The fight is over for this citizen: no more strikes come.
pub(crate) fn end_fight(world: &mut World, citizen: EntityId) -> Result<(), String> {
    world.cancel(JOB_STRIKE, |p| p["citizen"] == citizen.0)?;
    world.cancel(JOB_AMBUSH, |p| p["citizen"] == citizen.0)?;
    Ok(())
}

/// A citizen leaves a fight without a blow: gone quiet, the room disremembers them.
pub(crate) fn leave_fight(world: &mut World, actor: &str, citizen: EntityId) -> Result<(), String> {
    if let Some(creature) = engaged_with(world, citizen) {
        mend(world, creature).map_err(|e| e.to_string())?;
        set_engaged(world, actor, citizen, None).map_err(|e| e.to_string())?;
    }
    end_fight(world, citizen)
}

/// A creature of the Stacks left standing is whole again by the next time: a
/// newcomer with an attack of three felled a censor in four trips and three deaths,
/// the censor keeping every wound between them. The ground floor's creatures keep
/// their wounds, as they always have; the Stacks keep their own.
/// A creature's hit points, set by a trick (a leech mends by its bite).
pub(crate) fn set_creature_hp(
    world: &mut World,
    creature: EntityId,
    value: f64,
) -> Result<(), String> {
    world
        .record(
            Actor::Driver,
            StoreEvent::FieldSet {
                id: creature,
                component: CREATURE.into(),
                field: "hp".into(),
                value: json!(value),
            },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// A fight ended with the creature standing: it keeps its wounds, and mends on the
/// world's clock (`regen`), at the Clocktower's `mend` rate. Nothing is instant any
/// more: a 75-hit-point censor whole again between visits could not be worn down by
/// a thirty-hit-point name, and a wisp whole again the moment you died was what made
/// dying cheaper than retreating. The wound is the balance lever now.
pub(crate) fn mend(world: &mut World, _creature: EntityId) -> Result<(), StoreError> {
    let _ = ensure_regen(world);
    Ok(())
}

/// How long a Stacks creature takes to mend from nothing to whole at rate 1.0; the
/// ground floor's kinds take the same. (R70: ten minutes in the Stacks made the two-stand
/// censor unwinnable, since a body is whole from nothing in twelve.)
pub const STACKS_MEND_MS: i64 = 20 * 60 * 1000;
pub const GROUND_MEND_MS: i64 = 20 * 60 * 1000;
/// The old one-shot mend, kept so a job already in the log replays without complaint.
pub const JOB_MEND: &str = "disremembered.mend";
/// The creatures' mending, every minute: each wounded creature nobody is fighting
/// mends by the time since the last pass; a named one below half walks to succor.
pub const JOB_REGEN: &str = "disremembered.regen";
pub const REGEN_EVERY_MS: i64 = 60 * 1000;
/// A named creature this hurt (a share of its hit points) goes looking for succor.
pub const SUCCOR_BELOW: f64 = 0.5;
/// A creature mended: `{creature, kind, name, room, from, to}`; a creature walking to
/// succor or home: `{creature, name, from, to, via, why}`. Public.
pub const SUCCORED: &str = "disremembered.succored";

/// Striking: a strike lands [`HIT_BASE`] of the time, less while shaken. Three in
/// four (D11; it was nine in ten).
pub const HIT_BASE: f64 = 0.75;
/// A name that just woke at the Threshold is shaken: its strikes land this much less
/// often at the moment of waking, the penalty falling on a parabola to nothing
/// as the shaking passes (half gone at three tenths of the time).
pub const SHAKEN_PENALTY: f64 = 0.5;
/// How long the shaking lasts: this per fall of the night, so a name that keeps
/// dying is shaken longer each time, up to [`SHAKEN_MAX_MS`].
pub const SHAKEN_PER_FALL_MS: i64 = 5 * 60 * 1000;
pub const SHAKEN_MAX_MS: i64 = 30 * 60 * 1000;

/// How shaken a body is now: the penalty to its hit chance and the time left, or
/// nothing once it has steadied.
pub fn shaken(world: &World, citizen: EntityId) -> Option<(f64, i64)> {
    let woke = plain_num(world, citizen, BODY, "woke_at")? as i64;
    let lasts = plain_num(world, citizen, BODY, "shaken_ms")? as i64;
    let t = world.now().0 - woke;
    if lasts <= 0 || t < 0 || t >= lasts {
        return None;
    }
    let x = 1.0 - t as f64 / lasts as f64;
    Some((SHAKEN_PENALTY * x * x, lasts - t))
}

/// The chance a strike lands, 0 to 1.
pub fn to_hit(world: &World, citizen: EntityId) -> f64 {
    (HIT_BASE - shaken(world, citizen).map(|(p, _)| p).unwrap_or(0.0)).clamp(0.05, 1.0)
}

/// "Shaken: 4 strikes in 10 land, steadying over 3m 20s", or nothing.
pub fn shaken_said(world: &World, citizen: EntityId) -> Option<String> {
    let (_, left) = shaken(world, citizen)?;
    let hit = to_hit(world, citizen);
    Some(format!(
        "Shaken from the Threshold: {} strikes in 10 land, steadying over {}.",
        (hit * 10.0).round() as i64,
        human(left)
    ))
}

/// Whoever is fighting this creature, if anyone.
pub fn fought_by(world: &World, creature: EntityId) -> Option<EntityId> {
    world
        .store
        .entities()
        .filter(|e| e.kind == "citizen")
        .find(|e| plain_num(world, e.id, BODY, "engaged").map(|v| v as u64) == Some(creature.0))
        .map(|e| e.id)
}

/// Rooms where a body mends faster: the Cartographer's Closet, and any room with a
/// basin standing in it.
pub fn healing_rooms(world: &World) -> Vec<EntityId> {
    world
        .store
        .entities()
        .filter(|e| e.kind == "room")
        .filter(|e| {
            super::plain_str(world, e.id, super::ROOM, "name") == Some(CLOSET)
                || super::craft::fixture_of(world, e.id, "basin").is_some()
        })
        .map(|e| e.id)
        .collect()
}

pub(crate) fn ensure_regen(world: &mut World) -> Result<(), String> {
    let due = world.now().plus(foxing_time::Duration(REGEN_EVERY_MS));
    let now = world.now().0;
    world
        .schedule_once(JOB_REGEN, due, json!({"last": now}), |_| true)
        .map(|_| ())
}

/// One pass of the creatures' mending: `elapsed` since the last. The count mended.
pub fn regen(world: &mut World, elapsed_ms: i64) -> Result<usize, String> {
    let rate = super::economy::rate(world, "mend");
    // The wounded, and the whole ones still walking home from succor.
    let wounded: Vec<(EntityId, f64, f64, String)> = world
        .store
        .entities()
        .filter(|e| e.kind == "creature")
        .filter_map(|e| {
            let max = plain_num(world, e.id, CREATURE, "max_hp")?;
            let hp = plain_num(world, e.id, CREATURE, "hp").unwrap_or(max);
            let away = e
                .components
                .get(CREATURE)
                .and_then(|c| c.fields.get("succor"))
                .is_some_and(|f| matches!(f, Field::Plain(v) if v["away"] == true));
            (hp < max - 1e-9 || away).then(|| {
                (
                    e.id,
                    hp,
                    max,
                    super::plain_str(world, e.id, CREATURE, "name")
                        .unwrap_or("")
                        .to_string(),
                )
            })
        })
        .collect();
    let mut n = 0;
    for (creature, hp, max, kind) in wounded {
        if fought_by(world, creature).is_some() {
            continue;
        }
        if hp >= max - 1e-9 {
            // Whole, and on its way home.
            succor_step(world, creature, hp, max)?;
            continue;
        }
        let whole_ms = if super::stacks::TRICKS.iter().any(|(k, _)| *k == kind) {
            STACKS_MEND_MS
        } else {
            GROUND_MEND_MS
        };
        let room = world.store.entity(creature).and_then(|e| e.parent);
        // In a healing room a creature mends as a body does there: the Closet's pace
        // or a basin's, against the kind's own figure, whichever is quicker.
        let place = room
            .map(|r| {
                if super::plain_str(world, r, super::ROOM, "name") == Some(CLOSET) {
                    CLOSET_HP_PER_SECOND / super::MAX_HP
                } else if super::craft::fixture_of(world, r, "basin").is_some() {
                    super::HP_PER_SECOND * 2.0 / super::MAX_HP
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        let per_ms = (1.0 / whole_ms as f64).max(place / 1000.0) * rate;
        let gained = max * per_ms * elapsed_ms as f64;
        let after = (hp + gained).min(max);
        world
            .record(
                Actor::Driver,
                StoreEvent::FieldSet {
                    id: creature,
                    component: CREATURE.into(),
                    field: "hp".into(),
                    value: json!(((after) * 10.0).round() / 10.0),
                },
            )
            .map_err(|e| e.to_string())?;
        n += 1;
        // A named creature below half knows where succor is, or finds out, and goes;
        // whole again, it goes home.
        if super::mind::name(world, creature).is_some() {
            succor_step(world, creature, after, max)?;
        }
    }
    Ok(n)
}

/// A smart creature's errand to mend: below [`SUCCOR_BELOW`] of its hit points it
/// walks a room a minute toward the nearest healing room (one it remembers from
/// before, else the nearest it can route to), mends there, and walks home when whole.
/// It remembers the room it found (`succor` on the creature) for next time.
fn succor_step(world: &mut World, creature: EntityId, hp: f64, max: f64) -> Result<(), String> {
    let Some(here) = world.store.entity(creature).and_then(|e| e.parent) else {
        return Ok(());
    };
    let name = super::mind::display(world, creature);
    let succor = world
        .store
        .entity(creature)
        .and_then(|e| e.components.get(CREATURE))
        .and_then(|c| c.fields.get("succor"))
        .and_then(|f| match f {
            Field::Plain(v) if v.is_object() => Some(v.clone()),
            _ => None,
        });
    let going_home = succor.as_ref().is_some_and(|s| s["away"] == true);
    let healing = healing_rooms(world);
    let at_succor = healing.contains(&here);
    let whole = hp >= max - 0.5;
    // Whole and away: home.
    if going_home && whole {
        let Some(home) = succor
            .as_ref()
            .and_then(|s| s["home"].as_u64())
            .map(EntityId)
        else {
            return Ok(());
        };
        if here == home {
            return remember_succor(
                world,
                creature,
                succor.map(|mut s| {
                    s["away"] = json!(false);
                    s
                }),
            );
        }
        if let Some(path) = super::errand::route(world, here, home) {
            if let Some(via) = path.first() {
                return walk(world, creature, here, via, "home");
            }
        }
        return remember_succor(
            world,
            creature,
            succor.map(|mut s| {
                s["away"] = json!(false);
                s["home"] = json!(here.0);
                s
            }),
        );
    }
    if hp / max >= SUCCOR_BELOW && !going_home {
        return Ok(());
    }
    if at_succor {
        // Mending here; remember the place.
        let mut s = succor.unwrap_or_else(|| json!({"home": here.0}));
        if s["room"].as_u64() != Some(here.0) {
            s["room"] = json!(here.0);
            s["found_at"] = json!(world.now().0);
            let _ = world.note(
                Actor::Driver,
                SUCCORED,
                json!({"creature": creature.0, "name": name, "room": here.0, "why": "mending"}),
            );
        }
        s["away"] = json!(true);
        return remember_succor(world, creature, Some(s));
    }
    // Where to: the room it remembers, else the nearest healing room by route.
    let remembered = succor
        .as_ref()
        .and_then(|s| s["room"].as_u64())
        .map(EntityId)
        .filter(|r| healing.contains(r));
    let target = match remembered {
        Some(r) => super::errand::route(world, here, r).map(|p| (r, p)),
        None => healing
            .iter()
            .filter_map(|&r| super::errand::route(world, here, r).map(|p| (r, p)))
            .min_by_key(|(_, p)| p.len()),
    };
    let Some((_, path)) = target else {
        return Ok(());
    };
    let Some(via) = path.first() else {
        return Ok(());
    };
    let mut s = succor.unwrap_or_else(|| json!({}));
    if !going_home {
        s["home"] = json!(here.0);
        s["away"] = json!(true);
        remember_succor(world, creature, Some(s))?;
    }
    walk(world, creature, here, via, "succor")
}

/// For `look`: the named creatures whose home this room is, away mending.
pub fn away_mending(world: &World, room: EntityId) -> Vec<String> {
    world
        .store
        .entities()
        .filter(|e| e.kind == "creature" && e.parent != Some(room))
        .filter_map(|e| {
            let s = match e.components.get(CREATURE)?.fields.get("succor")? {
                Field::Plain(v) if v["away"] == true && v["home"].as_u64() == Some(room.0) => v,
                _ => return None,
            };
            let name = super::mind::name(world, e.id)?;
            let kind = super::plain_str(world, e.id, CREATURE, "name").unwrap_or("creature");
            let at = e
                .parent
                .and_then(|r| super::plain_str(world, r, super::ROOM, "name"))
                .unwrap_or("somewhere");
            let where_ = if s["room"].as_u64().map(EntityId) == e.parent {
                format!("mending in {at}")
            } else {
                format!("on its way, in {at}")
            };
            Some(format!(
                "{name} the {kind} went off hurt, to mend; it is {where_}, and comes back whole."
            ))
        })
        .collect()
}

fn remember_succor(world: &mut World, creature: EntityId, s: Option<Value>) -> Result<(), String> {
    let mut changes = BTreeMap::new();
    changes.insert("succor".into(), Field::Plain(s.unwrap_or(Value::Null)));
    super::bindery::set_fields(world, Actor::Driver, creature, CREATURE, changes)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// A creature takes a door on its own: the rooms on both sides hear it.
fn walk(
    world: &mut World,
    creature: EntityId,
    from: EntityId,
    via: &str,
    why: &str,
) -> Result<(), String> {
    let name = super::mind::display(world, creature);
    let from_where = match super::building::opposite_of(via) {
        "back" | "out" if !matches!(via, "in" | "out") => format!("through {via}"),
        opp => format!("from the {opp}"),
    };
    let (leave, arrive) = if why == "home" {
        (
            format!("{name} goes {via}, mended, the way it came."),
            format!("{name} comes back {from_where}, whole again."),
        )
    } else {
        (
            format!("{name} limps off {via}, toward somewhere it remembers it can mend."),
            format!("{name} comes in {from_where}, hurt, looking for somewhere to mend."),
        )
    };
    walk_said(world, creature, from, via, why, &leave, &arrive, SUCCORED)
}

/// A creature takes a door with lines of the caller's: the room it leaves hears
/// `leave` (the creature still standing there), the room it enters hears `arrive`,
/// and `event` is noted with from, to, via and why.
#[allow(clippy::too_many_arguments)]
pub(crate) fn walk_said(
    world: &mut World,
    creature: EntityId,
    from: EntityId,
    via: &str,
    why: &str,
    leave: &str,
    arrive: &str,
    event: &str,
) -> Result<(), String> {
    let Some(to) = super::rooms::exits_of(world, from).get(via).copied() else {
        return Ok(());
    };
    let name = super::mind::display(world, creature);
    narrate_room(world, creature, leave);
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: creature,
                parent: Some(to),
            },
        )
        .map_err(|e| e.to_string())?;
    let _ = world.note(
        Actor::Driver,
        event,
        json!({"creature": creature.0, "name": name, "from": from.0, "to": to.0, "via": via, "why": why}),
    );
    narrate_room(world, creature, arrive);
    Ok(())
}

/// The mending jobs: the old one-shot (replayed from the log; whole again, as it
/// said then) and the pass every minute.
pub fn register_mend(world: &mut World) {
    world.on_job(
        JOB_REGEN,
        Box::new(|w: &mut World, job| {
            let last = job.payload["last"].as_i64().unwrap_or(w.now().0);
            let elapsed = (w.now().0 - last).clamp(0, 6 * 60 * 60 * 1000);
            regen(w, elapsed)?;
            ensure_regen(w)
        }),
    );
    world.on_job(
        JOB_MEND,
        Box::new(|w: &mut World, job| {
            let creature = EntityId(job.payload["creature"].as_u64().unwrap_or(0));
            if w.store.entity(creature).is_none() {
                return Ok(());
            }
            let max = plain_num(w, creature, CREATURE, "max_hp").unwrap_or(0.0);
            w.record(
                Actor::Driver,
                StoreEvent::FieldSet {
                    id: creature,
                    component: CREATURE.into(),
                    field: "hp".into(),
                    value: json!(max),
                },
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        }),
    );
}

/// The creature a citizen is engaged with, if it still exists.
pub fn engaged_with(world: &World, citizen: EntityId) -> Option<EntityId> {
    let id = plain_num(world, citizen, BODY, "engaged")? as u64;
    world.store.entity(EntityId(id)).map(|e| e.id)
}

/// How fast this body mends now: nothing in a fight, nothing while bleeding, else
/// the world's rate.
pub fn mending_rate(world: &World, citizen: EntityId) -> f64 {
    if engaged_with(world, citizen).is_some() {
        return 0.0;
    }
    let now = world.now().0 as f64;
    if plain_num(world, citizen, BODY, "bleed_until").is_some_and(|u| u > now) {
        return 0.0;
    }
    // The Cartographer's Closet (a player's room off the Archive) says "you heal
    // about 6 HP a minute"; the owner made it so (2026-11-03): true in that room.
    let room = super::rooms::room_of(world, citizen);
    if room
        .and_then(|r| super::plain_str(world, r, super::ROOM, "name"))
        .is_some_and(|n| n == CLOSET)
    {
        return CLOSET_HP_PER_SECOND;
    }
    // A basin standing in the room: twice the pace.
    if room.is_some_and(|r| super::craft::fixture_of(world, r, "basin").is_some()) {
        return super::HP_PER_SECOND * 2.0;
    }
    super::HP_PER_SECOND
}

/// The one room where the field guide's healing figure is true.
pub const CLOSET: &str = "The Cartographer's Closet";
pub const CLOSET_HP_PER_SECOND: f64 = 6.0 / 60.0;

/// Set the body's mending to what it should be now (`mending_rate`): after a fight
/// begins or ends, after a bleeding bite, when it staunches, on waking.
pub fn recurve_mending(world: &mut World, citizen: EntityId) -> Result<(), StoreError> {
    let rate = mending_rate(world, citizen);
    // Nothing to write when the curve already runs at this rate (an arrival once
    // recorded the same curve again, and the world's reopening was no longer silent).
    let current = world
        .store
        .timed_field(citizen, BODY, "hp")
        .ok()
        .map(|t| t.curve.clone());
    if let Some(foxing_time::Curve::Linear { rate_per_ms }) = current {
        if (rate_per_ms - rate / 1000.0).abs() < 1e-12 {
            return Ok(());
        }
    }
    world.record(
        Actor::Driver,
        StoreEvent::FieldRecurved {
            id: citizen,
            component: BODY.into(),
            field: "hp".into(),
            curve: foxing_time::Curve::Linear {
                rate_per_ms: rate / 1000.0,
            },
        },
    )?;
    Ok(())
}

/// A bleeding bite: nothing mends for this long.
pub const BLEED_MS: i64 = 8 * 60 * 1000;
pub const JOB_STAUNCH: &str = "disremembered.staunch";

pub(crate) fn set_engaged(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    creature: Option<EntityId>,
) -> Result<(), StoreError> {
    let value = creature.map(|c| json!(c.0)).unwrap_or(Value::Null);
    world.record(
        Actor::Citizen(actor.into()),
        StoreEvent::FieldSet {
            id: citizen,
            component: BODY.into(),
            field: "engaged".into(),
            value,
        },
    )?;
    recurve_mending(world, citizen)?;
    Ok(())
}

/// Current hit points.
pub fn hp(world: &World, citizen: EntityId) -> f64 {
    world
        .store
        .read_field(citizen, BODY, "hp", world.now())
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

/// Hit points as a line says them: rounded up, so a name still standing on a
/// fraction never reads "0/30".
pub fn hp_said(world: &World, citizen: EntityId) -> f64 {
    hp(world, citizen).ceil()
}

pub(crate) fn commit_hp(
    world: &mut World,
    citizen: EntityId,
    value: f64,
) -> Result<(), StoreError> {
    world.record(
        Actor::Driver,
        StoreEvent::FieldCommitted {
            id: citizen,
            component: BODY.into(),
            field: "hp".into(),
            value: value.max(0.0),
        },
    )?;
    Ok(())
}

/// The line for a kill that left nothing (R49, R67): the rule with the rounds, once
/// a session; then "Nothing left." A visitor read "a longer fight leaves more" on
/// every one-blow kill as nagging, and a three-round miss beside a one-round drop as
/// a contradiction.
fn drop_miss_said(world: &mut World, actor: &str, citizen: EntityId, rounds: u64) -> String {
    let session = plain_num(world, citizen, super::CITIZEN, "session_at").unwrap_or(0.0);
    let said = plain_num(world, citizen, super::CITIZEN, "drop_rule_said").unwrap_or(-1.0);
    if said == session {
        return " Nothing left.".to_string();
    }
    let mut changes = BTreeMap::new();
    changes.insert("drop_rule_said".into(), Field::Plain(json!(session)));
    let _ = super::set_citizen_fields(world, actor, citizen, changes);
    let n = rounds.clamp(1, super::craft::DROP_SURE_ROUNDS);
    let chance = match n {
        1 => "one chance in six",
        2 => "one chance in three",
        3 => "one chance in two",
        4 => "two chances in three",
        5 => "five chances in six",
        _ => "sure",
    };
    format!(
        " Nothing of it is left behind: {} round{}, {chance}; sure at six.",
        n,
        if n == 1 { "" } else { "s" }
    )
}

/// R91: a rule said once a session, by key; `true` the first time this session.
fn first_this_session(world: &mut World, actor: &str, citizen: EntityId, key: &str) -> bool {
    let session = plain_num(world, citizen, super::CITIZEN, "session_at").unwrap_or(0.0);
    let said = plain_num(world, citizen, super::CITIZEN, key).unwrap_or(-1.0);
    if said == session {
        return false;
    }
    let mut changes = BTreeMap::new();
    changes.insert(key.into(), Field::Plain(json!(session)));
    let _ = super::set_citizen_fields(world, actor, citizen, changes);
    true
}

/// Damage per round at this level.
pub fn attack_for_level(level: f64) -> f64 {
    BASE_ATTACK + (ATTACK_PER_ROOT_LEVEL * level.max(0.0).sqrt()).floor()
}

/// The creature hits back. Returns the damage and whether the citizen died.
fn creature_strikes(
    world: &mut World,
    creature: EntityId,
    citizen: EntityId,
) -> Result<(f64, bool), StoreError> {
    let attack = plain_num(world, creature, CREATURE, "attack").unwrap_or(1.0);
    let dmg = attack + world.roll(3) as f64;
    let before = hp(world, citizen);
    let after = before - dmg;
    commit_hp(world, citizen, after)?;
    LAST_HIT.with(|c| c.set((before.ceil(), after.max(0.0).ceil())));
    // What this creature has done you, this fight: the vividity of the memory of
    // felling it is its share of your blood, not where the meter stood at the end
    // (a one-hit wisp after a lamprey's venom elsewhere once counted as near death).
    let taken = fight_taken(world, citizen, creature) + dmg;
    let rounds = fight_rounds(world, citizen, creature) + 1;
    world.record(
        Actor::Driver,
        StoreEvent::FieldSet {
            id: citizen,
            component: BODY.into(),
            field: "fight".into(),
            value: json!({"creature": creature.0, "taken": taken, "rounds": rounds}),
        },
    )?;
    Ok((dmg, after <= 0.0))
}

/// How many blows `creature` has struck `citizen` this fight: the fight's length,
/// which sets what it leaves behind (a one-blow kill at level three hundred leaves
/// little; six rounds leave something every time).
pub fn fight_rounds(world: &World, citizen: EntityId, creature: EntityId) -> u64 {
    world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(BODY))
        .and_then(|c| c.fields.get("fight"))
        .and_then(|f| match f {
            Field::Plain(v) if v["creature"] == creature.0 => v["rounds"].as_u64(),
            _ => None,
        })
        .unwrap_or(0)
}

/// What `creature` has done `citizen` in the fight they are in; nothing if the
/// last blood was another's.
pub fn fight_taken(world: &World, citizen: EntityId, creature: EntityId) -> f64 {
    world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(BODY))
        .and_then(|c| c.fields.get("fight"))
        .and_then(|f| match f {
            Field::Plain(v) if v["creature"] == creature.0 => v["taken"].as_f64(),
            _ => None,
        })
        .unwrap_or(0.0)
}

/// How vivid the memory of felling `creature` is: what it did you in this fight
/// against what you had left, so a fight that cost you most of your blood is near
/// one and a fight it never touched you in is nothing.
pub fn fight_vividity(world: &World, citizen: EntityId, creature: EntityId) -> f64 {
    let taken = fight_taken(world, citizen, creature);
    let left = hp(world, citizen).max(0.0);
    if taken <= 0.0 {
        0.0
    } else {
        (taken / (taken + left)).clamp(0.0, 1.0)
    }
}

thread_local! {
    /// The hit points before and after the last strike, as a line says them: both
    /// ends, because you mend between strikes and a line that said only the end
    /// read as arithmetic gone wrong ("hits you for 4" and the number fell by 2).
    static LAST_HIT: std::cell::Cell<(f64, f64)> = const { std::cell::Cell::new((0.0, 0.0)) };
}

/// "hits you for 4: 28 → 24 of 30", from the last strike.
pub(crate) fn hit_said(taken: f64) -> String {
    let (before, after) = LAST_HIT.with(|c| c.get());
    format!(
        "hits you for {taken:.0}: {before:.0} \u{2192} {after:.0} of {:.0}",
        super::MAX_HP
    )
}

/// Death: concussion, then wake at the Threshold at full health, disengaged.
/// A fall, as the call sites say it: what was lost (as prose, if anything), its
/// worth, which fall of the night, the epitaph, who keeps it, what it learned.
pub(crate) struct Death {
    pub lost: Option<String>,
    pub worth: f64,
    pub fall: u32,
    pub epitaph: String,
    pub keeper: Option<String>,
    pub learned: Option<&'static str>,
    pub shaken_ms: i64,
}

impl Death {
    /// The sentences after "You go down.", for every door a fall comes through.
    pub fn said(&self) -> String {
        // Named as an epitaph: a visitor read it as prose and reported none was said.
        let mut out = format!(
            " Over you, the world says: \u{201c}{}\u{201d}",
            self.epitaph
        );
        if let Some(c) = &self.lost {
            out.push_str(&format!(" Concussion: you have lost {c}"));
            match &self.keeper {
                Some(k) => out.push_str(&format!("; {k} keeps it.")),
                None => out.push('.'),
            }
        }
        // "Costs half what the first did" of a first that cost nothing read as a
        // lie (a level-twelve name's falls are free by the curve): said only when
        // something was lost, and the free fall says why it was free.
        if self.fall > 1 && self.lost.is_some() {
            out.push_str(&format!(
                " The {} fall of the night: it costs {} what the first did.",
                super::death::ordinal(self.fall),
                match self.fall {
                    2 => "half",
                    3 => "a quarter",
                    4 => "an eighth",
                    _ => "almost nothing of",
                }
            ));
        } else if self.lost.is_none() {
            out.push_str(
                " A fall costs a share of the window that grows with level: nothing at yours.",
            );
        }
        if let Some(l) = self.learned {
            out.push_str(&format!(" And {}.", super::death::learned_said(l)));
        }
        out.push_str(if self.lost.is_some() {
            " You wake at the Threshold"
        } else {
            " You wake at the Threshold with everything you had"
        });
        out.push_str(&format!(
            ", shaken: your strikes land less often for {}, steadying as it passes{}.",
            human(self.shaken_ms),
            if self.fall > 1 {
                " (longer with each fall of the night, whatever the memory cost)"
            } else {
                ""
            }
        ));
        out
    }
}

pub(crate) fn die(
    world: &mut World,
    actor: &str,
    f: &super::Found,
    killer: Option<EntityId>,
) -> Result<Death, CommandError> {
    // Concussion, on the curve: chunks go, picked at random, until the share of the
    // window the level owes is gone; the chunk that would take more than is owed is
    // thinned to the remainder instead of taken whole (a 7% death once cost a third of
    // a window, the one chunk the roll landed on). Nothing goes from a window worth
    // almost nothing. The share halves with every fall of the same night: cheap, but
    // not at first.
    // The share is of window and shelf together, and drawn from both (D2, Tyler #8:
    // a delver shelved the big chunks and fell for nothing); the halving by falls of
    // the night has a floor, a quarter of the first fall's share, so the fourth fall
    // of a night still costs.
    let on_shelf = super::conclave::shelved(world, f.palace);
    let level = super::memory::level(world, f.window) + on_shelf;
    let (fall, ever) = super::death::record_fall(world, actor, f.citizen, killer)?;
    let share = death_share(level) * 0.5f64.powi(fall.saturating_sub(1) as i32).max(FALL_FLOOR);
    let owed = level * share;
    let fell_in = super::rooms::room_of(world, f.citizen);
    let mut chunks: Vec<(EntityId, String, f64)> = world
        .store
        .children(f.window)
        .chain(world.store.children(f.palace))
        .filter(|e| e.kind == "chunk")
        .map(|e| {
            (
                e.id,
                plain_str(world, e.id, CHUNK, "source")
                    .unwrap_or("?")
                    .to_string(),
                world
                    .store
                    .read_field(e.id, CHUNK, "value", world.now())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            )
        })
        .collect();
    let mut gone: Vec<(EntityId, String, f64)> = Vec::new();
    let mut thinned: Vec<(EntityId, String, f64, f64)> = Vec::new();
    let mut taken = 0.0;
    while taken < owed && owed >= 1.0 && !chunks.is_empty() {
        let (id, source, value) = chunks.remove(world.roll(chunks.len() as u32) as usize);
        let remaining = owed - taken;
        if value > remaining + 0.5 {
            // Thin it to what is owed, and that settles the debt.
            let left = ((value - remaining) * 100.0).round() / 100.0;
            world
                .record(
                    Actor::Driver,
                    StoreEvent::FieldCommitted {
                        id,
                        component: CHUNK.into(),
                        field: "value".into(),
                        value: left,
                    },
                )
                .map_err(internal)?;
            taken += remaining;
            thinned.push((id, source, remaining, left));
            break;
        }
        world
            .record(Actor::Driver, StoreEvent::EntityRemoved { id })
            .map_err(internal)?;
        taken += value;
        gone.push((id, source, value));
    }
    // What was lost goes to the one that took it; it may learn something from the kill.
    let curves = super::Curves::from_env();
    let kept_chunk = match killer {
        Some(k) => super::death::killer_keeps(world, actor, k, taken, &curves)?,
        None => None,
    };
    let learned = match killer {
        Some(k) => super::death::maybe_learn(world, k)?,
        None => None,
    };
    // "the ragpicker", or "Ossian the ragpicker" once it has a name (it may have
    // earned one off this very fall): the article travels with the slot.
    let keeper_slot = killer.map(|k| super::death::killer_said(world, k));
    let epitaph = super::death::epitaph(world, actor, ever, killer, fell_in, taken, fall);
    let _ = world.note(
        Actor::Driver,
        DIED,
        json!({"citizen": actor, "level": level, "share": share, "fall": fall, "falls_ever": ever, "worth": (taken * 100.0).round() / 100.0,
               "room": fell_in.map(|r| r.0), "room_name": fell_in.and_then(|r| plain_str(world, r, super::ROOM, "name")),
               "killer": killer.map(|k| k.0), "killer_kind": killer.and_then(|k| plain_str(world, k, CREATURE, "name")), "killer_name": killer.and_then(|k| super::mind::name(world, k)),
               "kept": kept_chunk.map(|c| c.0), "learned": learned, "epitaph": epitaph,
               "kept_things": super::death::kept(world, actor),
               "night": super::death::night(world, f.citizen),
               "lost": gone.iter().map(|(id, s, v)| json!({"chunk": id.0, "source": s, "value": v}))
                   .chain(thinned.iter().map(|(id, s, v, left)| json!({"chunk": id.0, "source": s, "value": v, "thinned_to": left})))
                   .collect::<Vec<_>>()}),
    );
    let lost =
        if gone.is_empty() && thinned.is_empty() {
            None
        } else {
            let mut parts: Vec<String> = gone
                .iter()
                .map(|(id, s, v)| format!("chunk-{} ({s}, {v:.0} XP)", id.0))
                .collect();
            parts.extend(thinned.iter().map(|(id, s, v, left)| {
                format!("{v:.0} XP of chunk-{} ({s}, {left:.0} left)", id.0)
            }));
            Some(if parts.len() == 1 {
                parts.remove(0)
            } else {
                format!("{taken:.0} XP: {}", parts.join(", "))
            })
        };

    let home = super::rooms::arrival(world).ok_or_else(|| internal("the Threshold is missing"))?;
    // The creature keeps its wounds (it mends on the world's clock); a swarm that
    // dies on it does wear it down, as a swarm, not as one name reborn.
    let _ = ensure_regen(world);
    // You wake shaken: your strikes land less often, longer for each fall of the
    // night, steadying on a parabola.
    let shaken_ms = (SHAKEN_PER_FALL_MS * fall as i64).min(SHAKEN_MAX_MS);
    for (field, value) in [
        ("woke_at", json!(world.now().0)),
        ("shaken_ms", json!(shaken_ms)),
    ] {
        world
            .record(
                Actor::Driver,
                StoreEvent::FieldSet {
                    id: f.citizen,
                    component: BODY.into(),
                    field: field.into(),
                    value,
                },
            )
            .map_err(internal)?;
    }
    // Whatever was in you dies with you: a lamprey's venom once went on working at
    // the Threshold, where nothing hunts, after the wake.
    let me_id = f.citizen.0;
    let _ = world.cancel(super::stacks::JOB_VENOM, |p| p["citizen"] == me_id);
    let _ = world.cancel(JOB_STAUNCH, |p| p["citizen"] == me_id);
    world
        .record(
            Actor::Driver,
            StoreEvent::FieldSet {
                id: f.citizen,
                component: BODY.into(),
                field: "bleed_until".into(),
                value: json!(0),
            },
        )
        .map_err(internal)?;
    set_engaged(world, actor, f.citizen, None).map_err(internal)?;
    end_fight(world, f.citizen).map_err(internal)?;
    world
        .record(
            Actor::Driver,
            StoreEvent::EntityMoved {
                id: f.citizen,
                parent: Some(home),
            },
        )
        .map_err(internal)?;
    commit_hp(world, f.citizen, super::MAX_HP).map_err(internal)?;
    Ok(Death {
        lost,
        worth: taken,
        fall,
        epitaph,
        keeper: keeper_slot,
        learned,
        shaken_ms,
    })
}

pub fn register(world: &mut World, curves: Curves) {
    world.on_job(
        JOB_STAUNCH,
        Box::new(|w: &mut World, job| {
            let citizen = EntityId(job.payload["citizen"].as_u64().unwrap_or(0));
            if w.store.entity(citizen).is_none() {
                return Ok(());
            }
            recurve_mending(w, citizen).map_err(|e| e.to_string())?;
            if let Some(actor) = job.payload["actor"].as_str() {
                if mending_rate(w, citizen) > 0.0 {
                    narrate(w, actor, "The bleeding stops; you begin to mend.");
                }
            }
            Ok(())
        }),
    );
    world.on_job(
        JOB_RESPAWN,
        Box::new(|w: &mut World, job| {
            let room = EntityId(job.payload["room"].as_u64().unwrap_or(0));
            let spawned = fill(w, room).map_err(|e| e.to_string())?;
            if spawned > 0 {
                notice_everyone(w, room)?;
            }
            Ok(())
        }),
    );

    let c = curves.clone();
    world.on_job(
        JOB_STRIKE,
        Box::new(move |w: &mut World, job| {
            let creature = EntityId(job.payload["creature"].as_u64().unwrap_or(0));
            let citizen = EntityId(job.payload["citizen"].as_u64().unwrap_or(0));
            let actor = job.payload["actor"].as_str().unwrap_or("").to_string();
            // A name that came through the gate is struck once a statement, not on
            // the clock: a stateless visitor took nine strikes between two lines.
            if through_the_gate(w, &actor) {
                return Ok(());
            }
            let Some((over, _)) = round(w, &c, creature, citizen, &actor)? else {
                return Ok(());
            };
            if over {
                return Ok(());
            }
            schedule_strike(w, &actor, citizen, creature)
        }),
    );

    let c2 = curves.clone();
    world.on_job(
        JOB_AMBUSH,
        Box::new(move |w: &mut World, job| {
            let creature = EntityId(job.payload["creature"].as_u64().unwrap_or(0));
            let citizen = EntityId(job.payload["citizen"].as_u64().unwrap_or(0));
            let actor = job.payload["actor"].as_str().unwrap_or("").to_string();
            let same_room = w.store.entity(creature).and_then(|c| c.parent)
                == w.store.entity(citizen).and_then(|c| c.parent);
            if !same_room
                || w.store.entity(creature).is_none()
                || engaged_with(w, citizen).is_some()
                || !super::presence::is_here(w, citizen)
            {
                return Ok(());
            }
            if !is_aggressive(w, creature) {
                return Ok(());
            }
            let dname = plain_str(w, creature, CREATURE, "name")
                .unwrap_or("creature")
                .to_string();
            let dambush = ambush_line(w, creature);
            // It does not hunt those it knows: its keepers, whoever told it a story,
            // whoever did what it asked. (A visitor told Sorrel the story it wanted
            // and was ambushed by Sorrel in the Clocktower an hour later.)
            // Nor a name in its first hour: a hollow in the Shallows took a newcomer
            // from thirty to nothing between two gate lines, before they could act.
            let new_name = super::plain_num(w, citizen, super::CITIZEN, "named_at")
                .is_some_and(|t| w.now().0 - (t as i64) < super::kindle::GRACE_MS);
            if new_name
                || super::mind::shaken(w, creature)
                || super::mind::keepers(w, creature)
                    .iter()
                    .any(|k| k == &actor)
                || super::mind::told_by(w, creature, &actor)
                || super::errand::did_for(w, creature, &actor)
            {
                return Ok(());
            }
            set_engaged(w, &actor, citizen, Some(creature)).map_err(|e| e.to_string())?;
            let (taken, dead) =
                creature_strikes(w, creature, citizen).map_err(|e| e.to_string())?;
            if dead {
                let f = super::find_citizen(w, &actor).ok_or("citizen vanished")?;
                let d = die(w, &actor, &f, Some(creature)).map_err(|e| e.to_string())?;
                narrate(
                    w,
                    &actor,
                    format!(
                        "The {dname} {dambush} It {}. You go down.{}",
                        hit_said(taken),
                        d.said()
                    ),
                );
                return Ok(());
            }
            let (trick, over) = super::stacks::trick(w, &c2, creature, citizen, &actor)?;
            let clock = clock_line(w, creature, &actor);
            narrate(
                w,
                &actor,
                format!(
                    "The {dname} {dambush} It {}.{} {clock}",
                    hit_said(taken),
                    match &trick {
                        Some(t) => format!(" {t}"),
                        None => " Attack it, or retreat.".to_string(),
                    }
                ),
            );
            if over {
                return Ok(());
            }
            schedule_strike(w, &actor, citizen, creature)
        }),
    );

    world.register(
        CommandSpec::new("disremembered.attack", "Fight a creature here, one round: you strike, it strikes back. Win and you remember it; the closer you came to dying, the longer the memory lasts. While you stand with it, it keeps striking: at a keyboard on its own every few seconds; through the gate once for every statement you write. `retreat` ends that (it gets one blow as you turn). Some creatures ambush a name that is past its first hour. A new name's first grind: the Orchard's mothwings (6 hp, a hit of 1; east, north, north) or the Fogbank's wisps (8 hp, a hit of 2; east); the Shallows' hollows between them hunt and hit for 4.")
            .param("creature", ParamType::Entity, false, "a creature name from look; defaults to the one you are fighting, else an unnamed one here. A named creature is never struck by default."),
        Box::new(move |w: &mut World, ctx, args| {
            let f = me(w, ctx)?;
            let room = super::rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let here = creatures_in(w, room);
            if args.str("creature").is_some_and(super::docent::is_named) {
                return Err(CommandError::precondition(
                    "The Docent is not for fighting; it keeps the door, and has seen worse than you. Ask it something: tell docent \"\u{2026}\".",
                ));
            }
            let mut retargeted: Option<String> = None;
            let target = match args.str("creature") {
                Some(name) if !name.is_empty() => {
                    // The kind you are already fighting is the one you mean, before any
                    // other of its kind standing by (a typed "kill wisp" once turned on
                    // a shaken wisp beside the one that was biting).
                    let asked = name.trim().trim_start_matches("the ").to_lowercase();
                    let fighting = engaged_with(w, f.citizen)
                        .filter(|id| here.iter().any(|c| c.0 == *id))
                        .filter(|id| plain_str(w, *id, CREATURE, "name").is_some_and(|k| k == asked));
                    // "ragpicker-5161" after 5161 came apart, with 5162 still here: the
                    // kind it names is what is meant, so the strike is not wasted.
                    let kind_only = name
                        .trim()
                        .rsplit_once('-')
                        .filter(|(_, d)| d.chars().all(|c| c.is_ascii_digit()))
                        .map(|(k, _)| k.to_string());
                    let by_name = fighting.or_else(|| super::mind::find_here(w, room, name));
                    let found = by_name
                        .or_else(|| kind_only.as_deref().and_then(|k| super::mind::find_here(w, room, k)).filter(|c| super::mind::name(w, *c).is_none()));
                    if by_name.is_none() {
                        if let Some(c) = found {
                            retargeted = Some(format!("{} is gone; you turn on {}-{}.", name.trim(), plain_str(w, c, CREATURE, "name").unwrap_or("it"), c.0));
                        }
                    }
                    // A kind never strikes a named creature, even one alone of its kind:
                    // two visitors in one night felled Lute by typing "attack wisp".
                    if let Some(c) = found {
                        let kind = plain_str(w, c, CREATURE, "name").unwrap_or("");
                        let asked = name.trim().trim_start_matches("the ").to_lowercase();
                        if super::mind::name(w, c).is_some() && asked == kind {
                            return Err(CommandError::precondition(format!(
                                "No unnamed {kind} is here. {} has a name, and is struck only by that name.",
                                super::mind::display(w, c)
                            )));
                        }
                    }
                    found
                }
                // With no target: the one you are fighting, else the first creature here
                // that nobody has named and nobody bore. A named creature is never struck
                // by default, nor a child: one was felled at the Threshold by "attack".
                // The Conclave's officers (and the Stacks' keepers) are struck only by
                // name: a bare `attack` at the Bar picked the 158-hp tipstaff.
                _ => engaged_with(w, f.citizen).filter(|id| here.iter().any(|c| c.0 == *id) && !super::mind::shaken(w, *id)).or_else(|| here.iter().find(|c| super::mind::name(w, c.0).is_none() && super::kindle::born_at(w, c.0).is_none() && !super::mind::shaken(w, c.0) && !plain_str(w, c.0, CREATURE, "name").is_some_and(|k| OFFICERS.contains(&k))).map(|c| c.0)),
            };
            if let Some(c) = target {
                if let Some(born) = super::kindle::born_at(w, c) {
                    let age = w.now().0 - born;
                    if age < super::kindle::GRACE_MS && super::mind::name(w, c).is_none() {
                        return Err(CommandError::precondition(format!(
                            "The {} is newly born and not to be struck for {} more; feed it instead, or let it be.",
                            plain_str(w, c, CREATURE, "name").unwrap_or("creature"),
                            human(super::kindle::GRACE_MS - age)
                        )));
                    }
                }
            }
            let Some(creature) = target else {
                // Where you are, when it is not where you think: a death earlier in the
                // same message leaves you at the Threshold, and "No such creature here"
                // read as an error.
                let where_ = format!(" You are in {}.", plain_str(w, room, super::ROOM, "name").unwrap_or("?"));
                // An empty room that refills says when: the Fogbank's "nothing here to
                // fight" read as a dead end to a visitor who waited ninety seconds.
                let refill = w
                    .jobs
                    .pending()
                    .filter(|j| j.kind == JOB_RESPAWN && j.payload["room"] == room.0)
                    .map(|j| j.due.0 - w.now().0)
                    .min()
                    .map(|ms| {
                        // R31/R44: the room's figure beside the next job's, so the
                        // estimate moving between looks reads as the rule, not a fault.
                        let every = plain_num(w, room, SPAWN, "respawn_seconds").unwrap_or(60.0)
                            * super::economy::rate(w, "respawn");
                        // R60: with others fighting here, the mothwings are nearer
                        // than the wait.
                        let others_fighting = w
                            .store
                            .children(room)
                            .filter(|e| e.kind == "citizen" && e.id != f.citizen && super::presence::is_here(w, e.id))
                            .any(|e| engaged_with(w, e.id).is_some());
                        let is_fog = plain_str(w, room, super::ROOM, "key") == Some("fogbank");
                        format!(
                            " Something will drift in within about {}; this room refills three fifths to eight fifths of {} after it empties, never on the minute.{}",
                            human(ms.max(5_000)),
                            human((every * 1000.0) as i64),
                            if is_fog && others_fighting { " Or try the mothwings in the Orchard (north, north), the mildest things there are." } else { "" }
                        )
                    })
                    .unwrap_or_default();
                // F24: an empty room of the Stacks points at `where <kind>`.
                let stacks_hint = if plain_str(w, room, super::ROOM, "key").is_some_and(|k| super::stacks::KEYS.contains(&k)) && here.is_empty() {
                    " Nothing lives in this room of the Stacks; `where lamprey`, `where usher` or `where censor` says where each kind does."
                } else {
                    ""
                };
                let where_ = format!("{where_}{refill}{stacks_hint}");
                let officers: Vec<String> = here.iter().filter(|c| super::mind::name(w, c.0).is_none() && plain_str(w, c.0, CREATURE, "name").is_some_and(|k| OFFICERS.contains(&k))).map(|c| super::mind::display(w, c.0)).collect();
                return Err(CommandError::precondition(if here.is_empty() { format!("There is nothing here to fight.{where_}") } else if args.str("creature").map_or(true, str::is_empty) && !officers.is_empty() {
                    format!("Nothing here is struck by accident: {} {} an officer of the house, named in `attack` or left alone.", officers.join(" and "), if officers.len() == 1 { "is" } else { "are" })
                } else if args.str("creature").map_or(true, str::is_empty) && here.iter().all(|c| super::mind::shaken(w, c.0)) {
                    // Down and up again: not itself yet, not nameless. "has a name" was
                    // said of a hollow that had none, and the visitor guessed at names.
                    format!("Nothing here can be struck yet: {}.", here.iter().map(|c| format!("{} is not itself for {}", super::mind::display(w, c.0), super::mind::shaken_for(w, c.0).map(human).unwrap_or_else(|| "a while".into()))).collect::<Vec<_>>().join("; "))
                } else if args.str("creature").map_or(true, str::is_empty) {
                    {
                        let named: Vec<String> = here.iter().filter(|c| !super::mind::shaken(w, c.0) && super::mind::name(w, c.0).is_some()).map(|c| super::mind::display(w, c.0)).collect();
                        let shaken: Vec<String> = here.iter().filter(|c| super::mind::shaken(w, c.0) && super::mind::name(w, c.0).is_none()).map(|c| format!("the {} is not itself for {}", c.1, super::mind::shaken_for(w, c.0).map(human).unwrap_or_else(|| "a while".into()))).collect();
                        format!(
                            "Nothing unnamed is here to fight{}. {} {} a name, and {} struck only by that name; felled, it loses the name and the stories told to it.",
                            if shaken.is_empty() { String::new() } else { format!(" ({})", shaken.join("; ")) },
                            named.join(" and "),
                            if named.len() == 1 { "has" } else { "have" },
                            if named.len() == 1 { "is" } else { "are" }
                        )
                    }
                } else {
                    // Named as look names them, so the retry can be copied from the refusal.
                    format!("No such creature here. Here: {}.", here.iter().map(|c| {
                        let kind = c.1.as_str();
                        let twins = here.iter().filter(|o| o.1 == *kind).count();
                        match super::mind::name(w, c.0) {
                            Some(n) => format!("{n} the {kind}"),
                            None if twins > 1 => format!("{kind}-{}", c.0 .0),
                            None => kind.to_string(),
                        }
                    }).collect::<Vec<_>>().join(", ")) + &where_
                }));
            };
            let cname = plain_str(w, creature, CREATURE, "name").unwrap_or("?").to_string();
            // F23: beside a twin of its kind the lines say which one ("wisp-6351"),
            // as `look` names them; the kind itself stays the memory's source.
            let csaid = {
                // R99: the same rule as `look`'s: an id when another unnamed of its
                // kind stands here (a named twin does not count).
                let twins = here.iter().filter(|o| o.1 == cname && super::mind::name(w, o.0).is_none()).count();
                if twins > 1 && super::mind::name(w, creature).is_none() {
                    format!("{cname}-{}", creature.0)
                } else {
                    cname.clone()
                }
            };
            // The one you were fighting is gone (felled by another hand, or it drifted
            // off): said, so two rounds of damage do not vanish without a word.
            if let Some(was) = engaged_with(w, f.citizen).filter(|&e| e != creature) {
                if !here.iter().any(|c| c.0 == was) {
                    let gone_kind = plain_str(w, was, CREATURE, "name").map(str::to_string);
                    retargeted = Some(match gone_kind {
                        Some(k) => format!("The {k} you were fighting is gone, felled by another hand or drifted off; you turn on the {cname}."),
                        None => format!("What you were fighting came apart under someone else's hand; you turn on the {cname}."),
                    });
                }
            }
            if let Some(ms) = super::mind::shaken_for(w, creature) {
                return Err(CommandError::precondition(format!(
                    "{} is not itself yet and cannot be struck for {}; it stands up with nothing to take.",
                    super::mind::display(w, creature),
                    human(ms)
                )));
            }
            let mut began = None;
            if engaged_with(w, f.citizen) != Some(creature) {
                set_engaged(w, &ctx.actor, f.citizen, Some(creature)).map_err(internal)?;
                end_fight(w, f.citizen).map_err(internal)?;
                schedule_strike(w, &ctx.actor, f.citizen, creature).map_err(internal)?;
                // The clock, said once: a visitor died between two messages and read
                // it only on the next.
                began = Some(clock_line(w, creature, &ctx.actor));
                // R97: a named creature felled loses its name and what was told to
                // it; said at the first strike, and its keepers are told the kill.
                if let Some(n) = super::mind::name(w, creature) {
                    let keepers = super::mind::keepers(w, creature);
                    began = Some(format!(
                        "{} {n} has a name: felled, it is a {cname} again, and what was told to it goes with you{}.",
                        began.unwrap_or_default(),
                        if keepers.is_empty() { String::new() } else { format!(" ({} keeps it, and will hear of this)", keepers.join(", ")) }
                    ));
                }
            }

            // You strike: three in four land, fewer while you are shaken from a fall.
            let level = super::memory::level(w, f.window);
            let hit_chance = to_hit(w, f.citizen);
            let landed = (w.roll(100) as f64) < hit_chance * 100.0;
            let mut dmg = if landed { attack_for_level(level) + w.roll(3) as f64 } else { 0.0 };
            // An arm asleep from a numbing bite strikes for half, for a while.
            let numb = plain_num(w, f.citizen, BODY, "numb_until").is_some_and(|u| u > w.now().0 as f64);
            if numb && landed {
                dmg = (dmg / 2.0).max(1.0).floor();
            }
            let chp = (plain_num(w, creature, CREATURE, "hp").unwrap_or(0.0) - dmg).max(0.0);
            if landed {
                w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::FieldSet { id: creature, component: CREATURE.into(), field: "hp".into(), value: json!(chp) }).map_err(internal)?;
            }
            // A fight begun by default is said, not silent: six bare `attack` lines
            // felled an inkling on the fourth and struck its neighbor on the fifth.
            let turned = args.str("creature").map_or(true, str::is_empty) && began.is_some();
            let unsteady = if shaken(w, f.citizen).is_some() { ", your hand unsteady from the Threshold" } else { "" };
            let mut lines = vec![match (landed, turned) {
                (false, true) => format!("You turn on the {csaid} and miss{unsteady}."),
                (false, false) => format!("You swing at the {csaid} and miss{unsteady}."),
                (true, true) => format!("You turn on the {csaid} and hit it for {dmg:.0}{}.", if numb { ", your arm still asleep" } else { "" }),
                (true, false) => format!("You hit the {csaid} for {dmg:.0}{}.", if numb { ", your arm still asleep" } else { "" }),
            }];
            if let Some(r) = retargeted.take() {
                lines.insert(0, r);
            }
            if chp > 0.0 {
                if let Some(c) = began.take() {
                    lines.push(c);
                }
            }

            if chp <= 0.0 {
                // A creature that holds memories does not die: you take one, it is shaken.
                // The memory is yours at what it is worth now, as vivid as the fight.
                // "wisp goes down" lost its article; a named one keeps its name.
                let who = match super::mind::name(w, creature) {
                    Some(_) => super::mind::display(w, creature),
                    None => format!("The {}", super::mind::display(w, creature)),
                };
                let taken_vividity = fight_vividity(w, f.citizen, creature);
                // R97: its keepers, elsewhere, hear of the fall.
                if let Some(n) = super::mind::name(w, creature) {
                    let room_name = plain_str(w, room, super::ROOM, "name").unwrap_or("?").to_string();
                    for k in super::mind::keepers(w, creature) {
                        if k != ctx.actor && k != super::docent::NAME {
                            narrate(w, &k, format!("Word from {room_name}: {} felled {n} the {cname}, which you keep; it stands again, shaken, holding less.", ctx.actor));
                        }
                    }
                }
                if let Some((chunk, source, worth, was)) = super::mind::take(w, &ctx.actor, f.window, creature, taken_vividity, &curves)? {
                    set_engaged(w, &ctx.actor, f.citizen, None).map_err(internal)?;
                    end_fight(w, f.citizen).map_err(internal)?;
                    let left = super::mind::level(w, creature);
                    let faded = if was > worth + 0.5 {
                        format!(" It was worth {was:.0} when the {cname} got it; a creature's memories fade as yours do.")
                    } else {
                        String::new()
                    };
                    lines.push(format!(
                        "{who} goes down, and you take its memory of {source}: chunk-{} worth {worth:.1} XP.{faded} It gets up again, holding {left:.0}, and will not be itself for {}.",
                        chunk.0,
                        super::mind::shaken_for(w, creature).map(human).unwrap_or_else(|| "a while".into())
                    ));
                    return Ok(Outcome::new(lines.join(" "), json!({"creature": cname, "killed": false, "taken": chunk.0, "with": source, "xp": worth, "was": was, "left": left, "hp": hp(w, f.citizen)})));
                }
                // Win: the chunk's vividity is how close you came to dying; a moth
                // charm carried makes it more vivid and is spent.
                let my_hp = hp(w, f.citizen);
                let mut vividity = fight_vividity(w, f.citizen, creature);
                let mut charm_line = String::new();
                if let Some(charm) = super::craft::carries(w, f.citizen, "charm") {
                    // A second-tier charm (a warden's) gives half again.
                    vividity = (vividity + if super::past::tier(w, charm) >= 2 { 0.45 } else { 0.3 }).min(1.0);
                    let uses_left = plain_num(w, charm, super::craft::ITEM, "uses").unwrap_or(1.0);
                    let name = super::craft::spend(w, &ctx.actor, charm, &format!("Made the memory of a {} vivid", plain_str(w, creature, CREATURE, "name").unwrap_or("fight")))?;
                    charm_line = if uses_left > 1.5 {
                        format!(" The {} you carry gives one of its uses, and the memory is the more vivid for it.", name.trim_start_matches("a "))
                    } else {
                        format!(" The {} you carried is spent, and the memory is the more vivid for it.", name.trim_start_matches("a "))
                    };
                }
                // A kill pays what the creature held, at most the delver's level plus
                // sixty (the owner's call, 2026-11-02): a 169-XP name's first lamprey
                // doubled it and the climb to 2,200 was seven fights.
                let held = plain_num(w, creature, CREATURE, "xp").unwrap_or(0.0);
                let cap = super::memory::level(w, f.window) + KILL_PAY_OVER_LEVEL;
                let xp = held.min(cap).max(0.0);
                let capped_line = if held > cap + 0.5 {
                    if first_this_session(w, &ctx.actor, f.citizen, "cap_rule_said") {
                        format!(" (it held {held:.0}; a kill pays at most your level plus {KILL_PAY_OVER_LEVEL:.0})")
                    } else {
                        format!(" (it held {held:.0}; capped)")
                    }
                } else {
                    String::new()
                };
                // What it leaves behind, one time in three.
                // What it leaves behind: by the fight's length, not a flat one in
                // three (D4: fights past a hundred were clicking).
                let length = fight_rounds(w, f.citizen, creature) + 1;
                let left = super::craft::leave_behind(w, &curves, room, creature, &cname, length)?;
                // The dying throes of a creature of the Stacks: its trick, once more.
                let (throes, _) = super::stacks::throes(w, &curves, creature, f.citizen, &ctx.actor).map_err(internal)?;
                w.record(Actor::Driver, StoreEvent::EntityRemoved { id: creature }).map_err(internal)?;
                schedule_respawn(w, room).map_err(internal)?;
                set_engaged(w, &ctx.actor, f.citizen, None).map_err(internal)?;
                end_fight(w, f.citizen).map_err(internal)?;
                let chunk = super::memory::earn_chunk(w, &ctx.actor, f.window, &cname, xp, vividity, &curves)?;
                let xp = super::up(xp);
                // R98: the chunk's clock in the kill line, the figure a fighter
                // compares kinds by; R100: the worth to a tenth, as `chunks` says it.
                let halves = super::memory::halving_ms(w, chunk)
                    .map(|ms| format!(", halves every {}", human(ms)))
                    .unwrap_or_default();
                lines.push(format!(
                    "The {cname} comes apart. You remember it: chunk-{} worth {xp:.1} XP{capped_line}, vividity {:.2}{halves}{}.{charm_line}{}{}",
                    chunk.0,
                    vividity,
                    if vividity > 0.6 {
                        if first_this_session(w, &ctx.actor, f.citizen, "vivid_rule_said") { " (it had most of your blood for it; this one will last)" } else { " (vivid; it will last)" }
                    } else { "" },
                    match &left {
                        Some((id, n)) => format!(" It leaves {n} behind (item-{}); take it.", id.0),
                        // Say when the roll missed, so a visitor can tell a missed
                        // roll from a broken one (ten kills, nothing, and no way to know).
                        None if super::craft::material_for(&cname).is_some() => drop_miss_said(w, &ctx.actor, f.citizen, length),
                        None => String::new(),
                    },
                    throes.map(|t| format!(" {t}")).unwrap_or_default()
                ));
                return Ok(Outcome::new(lines.join(" "), json!({"creature": cname, "killed": true, "chunk": chunk.0, "xp": xp, "vividity": vividity, "hp": my_hp, "left": left.as_ref().map(|(id, n)| json!({"item": id.0, "name": n}))})));
            }

            // It strikes back.
            let (taken, dead) = creature_strikes(w, creature, f.citizen).map_err(internal)?;
            lines.push(format!("The {csaid} ({} left) {}.", left_said(chp), hit_said(taken)));
            if dead {
                let d = die(w, &ctx.actor, &f, Some(creature))?;
                lines.push(format!("You go down.{}", d.said()));
                return Ok(Outcome::new(lines.join(" "), json!({"creature": cname, "killed": false, "died": true, "lost": d.lost, "worth": d.worth, "fall": d.fall, "epitaph": d.epitaph, "keeper": d.keeper, "learned": d.learned})));
            }
            let my_hp = hp_said(w, f.citizen);
            let (trick, _) = super::stacks::trick(w, &curves, creature, f.citizen, &ctx.actor).map_err(internal)?;
            if let Some(t) = trick {
                lines.push(t);
            }
            Ok(Outcome::new(lines.join(" "), json!({"creature": cname, "killed": false, "died": false, "creature_hp": chp, "hp": my_hp})))
        }),
    );

    world.register(
        CommandSpec::new("disremembered.retreat", "Break off the fight. The creature gets one free strike as you go. A creature mends on the world's clock, whole from nothing in twenty minutes, faster than you (the Conclave's officers sooner); venom follows you out. You mend only out of the room. The reply says when each of you is whole and in rounds what a second stand would take, now and then."),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let Some(creature) = engaged_with(w, f.citizen) else {
                // R66: a hunter closing but not yet on you is not a fight; `travel`
                // leaves before it lands.
                let me_id = f.citizen.0;
                if w.jobs.has(JOB_AMBUSH, |p| p["citizen"] == me_id) {
                    return Err(CommandError::precondition(
                        "It has not reached you yet; `travel` leaves before it does, and there is nothing to retreat from until it lands.",
                    ));
                }
                return Err(CommandError::precondition("You are not fighting anything."));
            };
            let cname = plain_str(w, creature, CREATURE, "name").unwrap_or("?").to_string();
            let (taken, dead) = creature_strikes(w, creature, f.citizen).map_err(internal)?;
            mend(w, creature).map_err(internal)?;
            set_engaged(w, &ctx.actor, f.citizen, None).map_err(internal)?;
            end_fight(w, f.citizen).map_err(internal)?;
            if dead {
                let d = die(w, &ctx.actor, &f, Some(creature))?;
                let lost = d.lost.clone();
                let msg = {
                    format!("The {cname} catches you as you turn, for {taken:.0}. You go down.{}", d.said())
                };
                return Ok(Outcome::new(msg, json!({"died": true, "lost": lost})));
            }
            // The creature mends on the world's clock, whole from nothing in twenty
            // minutes, which is faster than you: said with both clocks, not "it keeps
            // the wound a while" (a visitor rested eleven minutes and came back to a
            // whole picnicker; the Watch of 11-06).
            let cmax = plain_num(w, creature, CREATURE, "max_hp").unwrap_or(super::MAX_HP);
            let chp_now = plain_num(w, creature, CREATURE, "hp").unwrap_or(0.0);
            let ckind = plain_str(w, creature, CREATURE, "name").unwrap_or("").to_string();
            let c_whole_ms = if super::stacks::TRICKS.iter().any(|(k, _)| *k == ckind) { STACKS_MEND_MS } else { GROUND_MEND_MS };
            let c_whole_in_ms = (((cmax - chp_now).max(0.0) / cmax) * c_whole_ms as f64) as i64;
            let me_whole_in_ms = ((super::MAX_HP - hp(w, f.citizen)) / super::HP_PER_SECOND * 1000.0) as i64;
            let mended = match super::stacks::trick_of(w, creature) {
                Some(super::stacks::Trick::Enter) | Some(super::stacks::Trick::Below) => " It will be whole again by the time you come back; the Conclave keeps its order.".to_string(),
                _ if chp_now <= 0.0 => String::new(),
                _ => format!(
                    " It mends on the world's clock, whole in {}{}; you are whole in {}.",
                    super::combat::human(c_whole_in_ms),
                    if c_whole_in_ms < me_whole_in_ms { ", before you are" } else { "" },
                    super::combat::human(me_whole_in_ms)
                ),
            };
            // Venom follows you out of the room: said, with what it will cost, since
            // five hit points and four ticks of two is a death the text once left unsaid.
            let me_id = f.citizen.0;
            let venom_left = w
                .jobs
                .pending()
                .find(|j| j.kind == super::stacks::JOB_VENOM && j.payload["citizen"] == me_id)
                .and_then(|j| j.payload["left"].as_i64())
                .unwrap_or(0);
            let venom = if venom_left > 0 {
                let cost = venom_left as f64 * super::stacks::VENOM_DAMAGE;
                let now_hp = hp(w, f.citizen);
                format!(
                    " The venom goes with you: {venom_left} more bite{} of {:.0}, {cost:.0} in all{}; a bandage closes nothing against it, and rest does not outrun it.",
                    if venom_left == 1 { "" } else { "s" },
                    super::stacks::VENOM_DAMAGE,
                    if cost >= now_hp { ", which is more than you have" } else { "" }
                )
            } else {
                String::new()
            };
            // R70: what a second stand would take, in rounds, against what you have
            // and what you will have when you are whole.
            let second = {
                let chp = chp_now;
                let level = super::memory::level(w, f.window);
                let mine = attack_for_level(level) + 1.0;
                let its = plain_num(w, creature, CREATURE, "attack").unwrap_or(1.0) + 1.0;
                let hit = to_hit(w, f.citizen);
                let rounds_now = (chp / (mine * hit)).ceil().max(1.0);
                let in_me_now = (hp(w, f.citizen) / its).floor().max(0.0);
                // The creature as it will be when you are whole: the two halves of the
                // sum at one time (as it is now against you when whole was never a
                // stand anyone could take).
                let chp_then = (chp + cmax * (me_whole_in_ms as f64 / c_whole_ms as f64)).min(cmax);
                let rounds_then = (chp_then / (mine * hit)).ceil().max(1.0);
                let in_me_whole = (super::MAX_HP / its).floor();
                if chp > 0.0 {
                    format!(
                        " A second stand now: about {rounds_now:.0} round{} to fell it, and you have {in_me_now:.0} in you. When you are whole: about {rounds_then:.0} round{} against its {chp_then:.0}, and you have {in_me_whole:.0}{}{}.",
                        if rounds_now == 1.0 { "" } else { "s" },
                        if rounds_then == 1.0 { "" } else { "s" },
                        // R94: the level fades while you rest, and a point of attack with it.
                        if me_whole_in_ms > 3 * 60_000 { " (a round more, maybe, as your level fades while you rest)" } else { "" },
                        if in_me_whole < rounds_then { "; not a fight for one name at this level: bring another" } else { "" }
                    )
                } else {
                    String::new()
                }
            };
            Ok(Outcome::new(format!("You back away from the {cname}; it catches you once for {taken:.0}. You have {:.0}/{:.0}.{mended}{venom}{second}", hp_said(w, f.citizen), super::MAX_HP), json!({"died": false, "hp": hp(w, f.citizen), "venom_left": venom_left})))
        }),
    );

    world.register(
        CommandSpec::new("disremembered.rest", "Your hit points, and how long until they are full. Healing happens on its own, on your clock, but never while something fights you (step out of the room first) and never while a bite bleeds."),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let now = w.now();
            // The same figure the hit lines say (ceiled), so lines agree (F1).
            let current = hp_said(w, f.citizen);
            let eta = w
                .store
                .timed_field(f.citizen, BODY, "hp")
                .ok()
                .and_then(|t| t.active_time_to_reach(super::MAX_HP, now, &w.store.scopes_for(f.citizen).unwrap_or_default()));
            let mut bleeding = plain_num(w, f.citizen, BODY, "bleed_until").filter(|&u| u > now.0 as f64);
            // A bandage carried closes the bite now, and is spent.
            let mut bandaged = String::new();
            if bleeding.is_some() {
                if let Some(bandage) = super::craft::carries(w, f.citizen, "bandage") {
                    super::craft::spend(w, &ctx.actor, bandage, "Closed a bite on rest")?;
                    let me_id = f.citizen.0;
                    let _ = w.cancel(JOB_STAUNCH, |p| p["citizen"] == me_id);
                    w.record(
                        Actor::Driver,
                        StoreEvent::FieldSet { id: f.citizen, component: BODY.into(), field: "bleed_until".into(), value: json!(0) },
                    )
                    .map_err(internal)?;
                    recurve_mending(w, f.citizen).map_err(internal)?;
                    bleeding = None;
                    bandaged = "Your bandage closes the bite, and is spent. ".to_string();
                }
            }
            let shaken_line = shaken_said(w, f.citizen).map(|l| format!(" {l}")).unwrap_or_default();
            let text = match eta {
                // Every branch says when a bandage went (#12: it was spent in a fight
                // and the reply never said so).
                _ if current < super::MAX_HP - 0.5 && engaged_with(w, f.citizen).is_some() => format!("{bandaged}You have {current:.0}/{:.0}. Nothing mends in a fight; retreat, and it starts.", super::MAX_HP),
                _ if current < super::MAX_HP - 0.5 && bleeding.is_some() => format!("{bandaged}You have {current:.0}/{:.0}. You are bleeding: nothing mends for {} more.", super::MAX_HP, human((bleeding.unwrap() as i64) - now.0)),
                // R56: whole within a half point is whole; the eta rounded up past it.
                Some(d) if d.0 > 0 && current < super::MAX_HP - 0.5 => format!("{bandaged}You have {current:.0}/{:.0}. Full in {}.", super::MAX_HP, human(d.0)),
                _ => format!("{bandaged}You have {current:.0}/{:.0}. You are whole.", super::MAX_HP),
            };
            // R48: one line with hp, level and what you hit for.
            let level = super::memory::level(w, f.window);
            let text = format!(
                "{text} Level {level:.0}; you hit for {:.0} (plus 0–2).",
                attack_for_level(level)
            );
            // The bog's clock, while standing in it (R40: it showed on the entry look only).
            let bog_line = super::rooms::room_of(w, f.citizen)
                .and_then(|r| super::trail::look_line(w, &ctx.actor, r))
                .filter(|l| l.contains("takes a liking"))
                .map(|l| format!(" {l}"))
                .unwrap_or_default();
            Ok(Outcome::new(format!("{text}{shaken_line}{bog_line}"), json!({"hp": current, "max_hp": super::MAX_HP, "full_in_seconds": eta.map(|d| d.0 / 1000), "bandage_spent": !bandaged.is_empty(), "to_hit": to_hit(w, f.citizen), "shaken_ms_left": shaken(w, f.citizen).map(|(_, l)| l)})))
        }),
    );
}

/// `90s`, `4m 10s`, `1h 2m`.
pub fn human(ms: i64) -> String {
    let s = ms / 1000;
    if s < 90 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m {}s", s / 60, s % 60)
    } else {
        format!("{}h {}m", s / 3600, (s % 3600) / 60)
    }
}
