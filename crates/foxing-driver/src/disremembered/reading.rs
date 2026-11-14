//! C2: the Reading Rooms, a mezzanine over the Unfiled Archive: six rooms for a name
//! between the Trail and the Stacks' stair. Their creatures are fought by what you
//! tell them and told by what you fought. A blade goes through them like a hand
//! through a paragraph; `story <creature> "…"` is the blow, and it lands only if it
//! tells of a fight you still remember (it names a kind you hold a memory of), as
//! hard as that memory is vivid, by the words the creature has not heard this fight.
//!
//! Each room has one rule of its own. The proofreader strikes back harder for every
//! word you repeat. The lampmoth takes the memory of a fight you nearly lost
//! (`remember <chunk> lampmoth`) and pays for it with one that lasts. The erratum,
//! the shusher and the redactor each carry one trick of the Stacks (venom, being
//! shown out, a chunk halved), met here one at a time before they come three to a
//! room below. The rooms are in rooms.rs, the creatures in combat.rs, the tricks in
//! stacks.rs; this is the telling and the vigil.

use super::{internal, plain_num, plain_str, Found, BODY, CHUNK, CREATURE};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{CallContext, CommandError, Outcome};
use foxing_store::{EntityId, StoreEvent};
use serde_json::json;
use std::collections::BTreeSet;

/// The rooms, by key; the first is the landing over the Archive.
pub const KEYS: &[&str] = &[
    "mezzanine",
    "proof-room",
    "errata-desk",
    "returns-cage",
    "lamp-table",
    "blue-pencil",
];
/// The kinds that are fought by telling.
pub const KINDS: &[&str] = &["proofreader", "lampmoth", "erratum", "shusher", "redactor"];
pub const PROOFREADER: &str = "proofreader";
pub const LAMPMOTH: &str = "lampmoth";

/// A blade's share of its blow against a thing of the Reading Rooms.
pub const BLADE_SHARE: f64 = 0.25;
/// New words to a point of damage, before the memory's vividity; and the most words
/// one telling is counted for.
const WORDS_A_POINT: f64 = 2.5;
const MOST_WORDS: usize = 60;
/// D39 a: what the memory's vividity is worth to a telling: a quarter of the words'
/// weight at vividity 0, one and three quarters at 1, so a dull memory lands a
/// seventh of a vivid one and the way to hit hard is a fight nearly lost. (It ran
/// from three quarters to one and a half, and word count swamped it.)
const VIVIDITY_FLOOR: f64 = 0.25;
const VIVIDITY_SPAN: f64 = 1.5;
/// The most a proofreader adds to its blow for repeats.
const MOST_BACK: usize = 10;
/// A fight this vivid or more was nearly lost: what the lampmoth takes.
pub const NEARLY_LOST: f64 = 0.6;
/// The lampmoth pays the memory's worth and half again, as vivid as this.
const VIGIL_PAYS: f64 = 1.5;
const VIGIL_VIVIDITY: f64 = 0.9;
/// One vigil a name an hour.
const VIGIL_EVERY_MS: i64 = 60 * 60 * 1000;
/// The lampmoth kept a fight: `{by, creature, chunk, with, vividity, paid}`. Public.
pub const VIGIL: &str = "disremembered.vigil";
/// A telling landed: `{by, creature, kind, of, new_words, repeats, damage}`. Public.
pub const TOLD: &str = "disremembered.told";

pub fn is_told(kind: &str) -> bool {
    KINDS.contains(&kind)
}

/// Whether a room is one of the Reading Rooms.
pub fn is_room(world: &World, room: EntityId) -> bool {
    plain_str(world, room, super::ROOM, "key").is_some_and(|k| KEYS.contains(&k))
}

/// The line `attack` adds when a blade meets one of these.
pub fn blade_said(kind: &str) -> String {
    format!(
        "Your blow goes through it like a hand through a paragraph (a quarter lands): the Reading Rooms' things are fought by what you tell them. `story {kind} \"\u{2026}\"`, a dozen words or more on a fight you still remember, naming what you fought."
    )
}

/// The fights a name remembers: `(kind, vividity)`, the most vivid of each kind.
pub fn fights(world: &World, f: &Found) -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = Vec::new();
    for holder in [f.window, f.palace] {
        for e in world.store.children(holder).filter(|e| e.kind == "chunk") {
            let Some(source) = plain_str(world, e.id, CHUNK, "source") else {
                continue;
            };
            if super::combat::def_by_name(source).is_none() {
                continue;
            }
            let v = plain_num(world, e.id, CHUNK, "vividity").unwrap_or(0.0);
            match out.iter_mut().find(|(k, _)| k == source) {
                Some((_, best)) => *best = best.max(v),
                None => out.push((source.to_string(), v)),
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn words_of(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// The words this name has told this creature this fight.
fn told_so_far(world: &World, citizen: EntityId, creature: EntityId) -> BTreeSet<String> {
    world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(BODY))
        .and_then(|c| c.fields.get("told"))
        .and_then(|f| match f {
            foxing_store::Field::Plain(v) => Some(v.clone()),
            _ => None,
        })
        .filter(|v| v["creature"] == creature.0)
        .and_then(|v| v["words"].as_array().cloned())
        .map(|a| {
            a.iter()
                .filter_map(|w| w.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Which remembered fight a telling is of: the most vivid kind it names.
fn fight_named(text_words: &[String], fights: &[(String, f64)]) -> Option<(String, f64)> {
    let joined = format!(" {} ", text_words.join(" "));
    fights
        .iter()
        .filter(|(kind, _)| {
            let plural = super::combat::plural(kind);
            joined.contains(&format!(" {kind} ")) || joined.contains(&format!(" {plural} "))
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .cloned()
}

/// A telling, weighed: what it is of, what it would do.
pub struct Weighed {
    pub kind: String,
    pub of: String,
    pub vividity: f64,
    pub new_words: usize,
    pub repeats: usize,
    /// The words marked, for the reply: a count alone taught nothing.
    pub marked: Vec<String>,
    /// What a proofreader adds to its blow.
    pub back: f64,
    pub damage: f64,
    words: BTreeSet<String>,
}

/// Weigh a telling without telling it: why it would be refused (nothing spent, no
/// blow), or what it would land for. The gate and the dry run both ask here, so
/// they agree (the dry run passed five tellings the gate refused, the Watch of
/// 11-14).
pub fn weigh(w: &World, f: &Found, creature: EntityId, text: &str) -> Result<Weighed, String> {
    let kind = plain_str(w, creature, CREATURE, "name")
        .unwrap_or("?")
        .to_string();
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let words = words_of(&text);
    let distinct: BTreeSet<String> = words.iter().cloned().collect();
    if distinct.len() < super::exchange::MIN_WORDS {
        return Err(format!(
            "A telling is at least {} different words; that was {}. Nothing is spent and the {kind} has not struck.",
            super::exchange::MIN_WORDS,
            distinct.len()
        ));
    }
    if let Some(why) = super::grammar::why_unfollowable(&text) {
        return Err(format!(
            "The {kind} cannot follow that: {why}. Tell it again, in sentences (small words between the others, a stop now and then); it has not struck."
        ));
    }
    let mine = fights(w, f);
    if mine.is_empty() {
        return Err(format!(
            "The {kind} does not look up. In the Reading Rooms a telling lands only if it is of a fight you still remember, and you hold no memory of one: win a fight (the Orchard's mothwings, the Fogbank's wisps), keep the memory, and come back."
        ));
    }
    let Some((of, vividity)) = fight_named(&words, &mine) else {
        return Err(format!(
            "The {kind} does not look up: a telling lands only if it is of a fight you still remember, and names what you fought. You remember: {}. Tell it one of those; it has not struck.",
            mine.iter()
                .map(|(k, v)| format!("{k} (vividity {v:.2})"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    };
    let before = told_so_far(w, f.citizen, creature);
    let new_words = distinct.iter().filter(|x| !before.contains(*x)).count();
    // R229: a telling it has heard every word of is not a blow thrown away and a
    // blow taken; it is refused, unstruck (a word-for-word retelling was a fall).
    if new_words == 0 {
        return Err(format!(
            "The {kind} has heard every word of that from you this fight. Tell it something new; nothing is spent and it has not struck."
        ));
    }
    // The words that count against you with a proofreader: any of four letters or
    // more said twice in this telling, or said already this fight. R228: not the
    // names of what you fought, which a telling must say, nor the listener's own.
    let mut exempt: BTreeSet<String> = BTreeSet::new();
    for k in mine.iter().map(|(k, _)| k.as_str()).chain([kind.as_str()]) {
        exempt.extend(words_of(k));
        exempt.extend(words_of(&super::combat::plural(k)));
    }
    let counts = |x: &String| x.chars().count() >= 4 && !exempt.contains(x);
    // (A word said twice here and also earlier in the fight is marked for each.)
    let twice: Vec<&String> = distinct
        .iter()
        .filter(|x| counts(x) && words.iter().filter(|y| y == x).count() > 1)
        .collect();
    let again: Vec<&String> = distinct
        .iter()
        .filter(|x| counts(x) && before.contains(*x))
        .collect();
    let repeats = twice.len() + again.len();
    let mut marked: Vec<String> = twice
        .iter()
        .chain(again.iter())
        .map(|x| x.to_string())
        .collect();
    marked.dedup();
    let back = if kind == PROOFREADER {
        repeats.min(MOST_BACK) as f64
    } else {
        0.0
    };
    let counted = new_words.min(MOST_WORDS) as f64;
    let damage = ((counted / WORDS_A_POINT) * (VIVIDITY_FLOOR + VIVIDITY_SPAN * vividity))
        .round()
        .max(1.0);
    let mut all = before;
    all.extend(distinct);
    Ok(Weighed {
        kind,
        of,
        vividity,
        new_words,
        repeats,
        marked,
        back,
        damage,
        words: all,
    })
}

/// `story` at an unnamed thing of the Reading Rooms: the telling is the blow.
pub fn tell(
    w: &mut World,
    ctx: &CallContext,
    f: &Found,
    creature: EntityId,
    text: &str,
) -> Result<Outcome, CommandError> {
    let Weighed {
        kind,
        of,
        vividity,
        new_words,
        repeats,
        marked,
        back,
        damage: dmg,
        words,
    } = weigh(w, f, creature, text).map_err(CommandError::precondition)?;
    w.record(
        Actor::Driver,
        StoreEvent::FieldSet {
            id: f.citizen,
            component: BODY.into(),
            field: "told".into(),
            value: json!({"creature": creature.0, "words": words.into_iter().collect::<Vec<_>>()}),
        },
    )
    .map_err(internal)?;
    let _ = w.note(
        Actor::Citizen(ctx.actor.clone()),
        TOLD,
        json!({"by": ctx.actor, "creature": creature.0, "kind": kind, "of": of, "new_words": new_words, "repeats": repeats, "damage": dmg}),
    );
    let out = super::combat::with_telling(dmg, back, || {
        w.call(
            ctx,
            "disremembered.attack",
            &json!({"creature": format!("{kind}-{}", creature.0)}),
        )
    })?;
    let mut said = format!(
        "You tell it of the {of} (your memory of that fight: vividity {vividity:.2}; {new_words} word{} it had not heard). {}",
        if new_words == 1 { "" } else { "s" },
        out.summary
    );
    if kind == PROOFREADER && repeats > 0 {
        said.push_str(&format!(
            " The proofreader marked {repeats} repeated word{} ({}) and struck {} harder for {}.",
            if repeats == 1 { "" } else { "s" },
            marked.join(", "),
            back as i64,
            if repeats == 1 { "it" } else { "them" }
        ));
    }
    // A proofreader's blow is its own and a point a repeat: said while it could
    // still be the last (a tester at 13 took 14 unwarned).
    if kind == PROOFREADER && super::combat::engaged_with(w, f.citizen) == Some(creature) {
        let base = plain_num(w, creature, CREATURE, "attack").unwrap_or(4.0);
        let hp = super::combat::hp(w, f.citizen);
        if hp <= base + 2.0 + MOST_BACK as f64 {
            said.push_str(&format!(
                " You have {hp:.0}: a proofreader hits {base:.0}\u{2013}{:.0} and a point more for each word of four letters or more that you repeat, in a telling or across the fight, ten at most (the names of what you fought do not count).",
                base + 2.0
            ));
        }
    }
    let mut data = out.result;
    data["told"] = json!({"of": of, "vividity": vividity, "new_words": new_words, "repeats": repeats, "damage": dmg});
    Ok(Outcome::new(said, data))
}

/// Why a lampmoth would not take this chunk, or nothing: asked by the gate and by
/// the dry run.
pub fn vigil_why_not(w: &World, actor: &str, f: &Found, chunk: EntityId) -> Option<String> {
    let source = plain_str(w, chunk, CHUNK, "source")
        .unwrap_or("?")
        .to_string();
    let vividity = plain_num(w, chunk, CHUNK, "vividity").unwrap_or(0.0);
    let fight = super::combat::def_by_name(&source).is_some();
    if !fight || vividity < NEARLY_LOST {
        let have: Vec<String> = [f.window, f.palace]
            .iter()
            .flat_map(|h| w.store.children(*h).filter(|e| e.kind == "chunk"))
            .filter(|e| {
                plain_str(w, e.id, CHUNK, "source")
                    .is_some_and(|s| super::combat::def_by_name(s).is_some())
                    && plain_num(w, e.id, CHUNK, "vividity").unwrap_or(0.0) >= NEARLY_LOST
            })
            .map(|e| format!("chunk-{}", e.id.0))
            .collect();
        return Some(format!(
            "The lampmoth turns from it. It takes only the memory of a fight you nearly lost: a fight's chunk of vividity {NEARLY_LOST:.1} or more (`chunks` shows each; a memory is that vivid when the fight had most of your blood). {} {}",
            if fight {
                format!("That one, of the {source}, is {vividity:.2}.")
            } else {
                format!("That one, of {source}, is not a fight.")
            },
            if have.is_empty() {
                "You hold none that would do; nothing is spent.".to_string()
            } else {
                format!("It would take: {}. Nothing is spent.", have.join(", "))
            }
        ));
    }
    let now = w.now();
    let last = w
        .recent_events(4000)
        .iter()
        .rev()
        .find(|e| e.kind == VIGIL && e.payload["by"] == actor)
        .map(|e| now.since(e.at).0);
    if let Some(ago) = last.filter(|a| *a < VIGIL_EVERY_MS) {
        return Some(format!(
            "A lampmoth keeps one vigil an hour for a name, and kept one of yours {} ago; it will take another in {}. Nothing is spent.",
            super::combat::human(ago),
            super::combat::human(VIGIL_EVERY_MS - ago)
        ));
    }
    None
}

/// `remember <chunk> lampmoth`: it takes the memory of a fight you nearly lost, and
/// pays with one that lasts.
pub fn vigil(
    w: &mut World,
    ctx: &CallContext,
    f: &Found,
    creature: EntityId,
    chunk: EntityId,
    curves: &super::Curves,
) -> Result<Outcome, CommandError> {
    if let Some(why) = vigil_why_not(w, ctx.actor.as_str(), f, chunk) {
        return Err(CommandError::precondition(why));
    }
    let source = plain_str(w, chunk, CHUNK, "source")
        .unwrap_or("?")
        .to_string();
    let vividity = plain_num(w, chunk, CHUNK, "vividity").unwrap_or(0.0);
    let now = w.now();
    let worth = w
        .store
        .read_field(chunk, CHUNK, "value", now)
        .map_err(internal)?
        .as_f64()
        .unwrap_or(0.0);
    let cap = super::memory::level(w, f.window) + super::combat::KILL_PAY_OVER_LEVEL;
    let paid = (worth * VIGIL_PAYS).min(cap).max(1.0);
    super::mind::give(w, ctx.actor.as_str(), chunk, creature)?;
    let earned = super::memory::earn_chunk(
        w,
        &ctx.actor,
        f.window,
        "a lampmoth's vigil",
        paid,
        VIGIL_VIVIDITY,
        curves,
    )?;
    let _ = w.note(
        Actor::Citizen(ctx.actor.clone()),
        VIGIL,
        json!({"by": ctx.actor, "creature": creature.0, "chunk": chunk.0, "with": source, "vividity": vividity, "worth": worth, "paid": paid, "earned": earned.0}),
    );
    // C6: the book remembers the vigil.
    super::craft::commonplace(
        w,
        &ctx.actor,
        f.citizen,
        &format!(
            "A lampmoth kept a vigil over {}'s fight with the {source}.",
            ctx.actor
        ),
    );
    let halves = super::memory::halving_ms(w, earned)
        .map(|ms| format!(", halves every {}", super::combat::human(ms)))
        .unwrap_or_default();
    Ok(Outcome::new(
        format!(
            "The lampmoth settles on the memory of the {source} (vividity {vividity:.2}, worth {worth:.1}) and keeps it, wings shut, as long as a lamp burns. What it leaves you is the vigil: chunk-{} worth {paid:.1} XP, vividity {VIGIL_VIVIDITY:.2}{halves}. It keeps one an hour for a name.",
            earned.0
        ),
        json!({"vigil": true, "spent": chunk.0, "with": source, "chunk": earned.0, "xp": paid, "vividity": VIGIL_VIVIDITY}),
    ))
}

/// The sign on the landing, in `look`.
pub fn landing_said(world: &World, f: &Found) -> String {
    let mine = fights(world, f);
    let level = super::memory::level(world, f.window);
    format!(
        "A card on the newel post: \"READING ROOMS. Sized for a name holding 20 to 100 (you hold {level:.0}). Nothing here is struck to any purpose; it is told. `story <creature> \"\u{2026}\"` of a fight you still remember, naming what you fought, a dozen words or more: the more words it has not heard this fight and the more vivid that memory, the harder the telling lands (your most vivid memory of the kind you name is the one told). Every telling draws a blow back; a proofreader's is a point harder for each word you repeat. The lampmoth takes the memory of a fight you nearly lost, vividity 0.6 or more (`remember <chunk> lampmoth`). Nothing asks your level at the stair: under 20, the lampmoths are the ones to tell.\" {}",
        if mine.is_empty() {
            "You remember no fight just now; win one below first.".to_string()
        } else {
            format!(
                "You remember: {}.",
                mine.iter()
                    .map(|(k, v)| format!("{k} ({v:.2})"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    )
}

/// D37 b, for those already standing: a creature of these rooms that holds nothing
/// of its own and is worth more than its kind now is (it stood before the kinds
/// were halved, and two of the Watch of 11-14 were paid 88 and 48 by such) is worth
/// what its kind is. On boot; returns how many.
pub fn settle(world: &mut World) -> Result<usize, String> {
    let ids: Vec<(EntityId, f64)> = world
        .store
        .of_kind("creature")
        .filter_map(|e| {
            let kind = plain_str(world, e.id, CREATURE, "name")?;
            if !is_told(kind) || super::mind::name(world, e.id).is_some() {
                return None;
            }
            let def = super::combat::def_by_name(kind)?;
            let xp = plain_num(world, e.id, CREATURE, "xp")?;
            (super::mind::level(world, e.id) < 0.5 && xp > def.xp * 1.25).then_some((e.id, def.xp))
        })
        .collect();
    for (id, xp) in &ids {
        world
            .record(
                Actor::Driver,
                StoreEvent::FieldSet {
                    id: *id,
                    component: CREATURE.into(),
                    field: "xp".into(),
                    value: json!(xp),
                },
            )
            .map_err(|e| e.to_string())?;
    }
    Ok(ids.len())
}
