//! C7: what players write for players.
//!
//! Rooms, recipes and wards were player-made; wants were the world's alone. A kept
//! name that keeps a named creature (it holds a memory that name gave it) may leave
//! one want with it, in the name's own words, and stake one of its own memories on
//! it:
//!
//! - `entrust <creature> <chunk> "<what about>"`: a story about that;
//! - `entrust <creature> <chunk> "<the word>" <another named creature>`: that word,
//!   carried to the other.
//!
//! David, 11-20 ("c7a:1 / c7b:3 / c7c:1 / c7d:2 / c7e:3"): one want a creature and one
//! standing want a name; the stake is held by the creature, goes to whoever does the
//! want with the world's usual pay on top, and comes back, as faded as it then is, if
//! the want lapses; both of the world's kinds; a short free line, checked as a name
//! is, shown in quotation marks and marked as that name's words; and no limit on who
//! does it but that it is not the one who left it.
//!
//! A player's want sits beside the world's own on the same creature and does not
//! displace it. The line is the only prose of one player's that `wants` and `look`
//! put in front of another, which is why it is short, quoted, and attributed every
//! time it is said.

use super::{internal, mind, plain_str, CREATURE};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A player's want, on the creature that holds it:
/// `{kind, by, line, keywords, to, stake, stake_at, stake_hl_ms, with, at, heard}`.
pub const ASK: &str = "disremembered.ask";
/// A want was left: `{by, creature, name, kind, line, to, stake}`. Public.
pub const LEFT: &str = "disremembered.ask_left";
/// A player's want was done: `{by, from, creature, name, kind, line, to, paid}`.
pub const DONE: &str = "disremembered.ask_done";
/// A player's want lapsed or was taken back: `{by, creature, name, why, back}`.
pub const LAPSED: &str = "disremembered.ask_lapsed";

/// The line a player writes: at most this many characters.
pub const LINE_MAX: usize = 80;
/// A stake is worth at least this when it is left.
pub const STAKE_MIN: f64 = 10.0;
/// And the want lapses when what is left of it is under this.
pub const STAKE_FLOOR: f64 = 3.0;

#[derive(Debug, Clone)]
pub struct Ask {
    pub kind: String,
    pub by: String,
    pub line: String,
    pub keywords: Vec<String>,
    pub to: Option<String>,
    stake: f64,
    stake_at: i64,
    stake_hl_ms: i64,
    pub with: String,
    heard: Vec<String>,
}

impl Ask {
    /// What the stake is worth now.
    pub fn worth(&self, now: i64) -> f64 {
        let hl = self.stake_hl_ms.max(60_000) as f64;
        self.stake * 0.5f64.powf((now - self.stake_at).max(0) as f64 / hl)
    }
}

pub fn of(world: &World, creature: EntityId) -> Option<Ask> {
    let f = &world.store.entity(creature)?.components.get(ASK)?.fields;
    let v = |k: &str| match f.get(k) {
        Some(Field::Plain(v)) if !v.is_null() => Some(v.clone()),
        _ => None,
    };
    let s = |k: &str| v(k).and_then(|x| x.as_str().map(str::to_string));
    let list = |k: &str| -> Vec<String> {
        v(k).and_then(|x| x.as_array().cloned())
            .unwrap_or_default()
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect()
    };
    Some(Ask {
        kind: s("kind")?,
        by: s("by")?,
        line: s("line")?,
        keywords: list("keywords"),
        to: s("to"),
        stake: v("stake").and_then(|x| x.as_f64()).unwrap_or(0.0),
        stake_at: v("stake_at").and_then(|x| x.as_i64()).unwrap_or(0),
        stake_hl_ms: v("stake_hl_ms")
            .and_then(|x| x.as_i64())
            .unwrap_or(3_600_000),
        with: s("with").unwrap_or_else(|| "something".into()),
        heard: list("heard"),
    })
}

/// Every standing want a player has left, with the creature that holds it.
pub fn all(world: &World) -> Vec<(EntityId, Ask)> {
    world
        .store
        .of_kind("creature")
        .filter_map(|e| of(world, e.id).map(|a| (e.id, a)))
        .collect()
}

fn set(
    world: &mut World,
    creature: EntityId,
    fields: BTreeMap<String, Field>,
) -> Result<(), CommandError> {
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id: creature,
                component: ASK.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(internal)
}

fn clear(world: &mut World, creature: EntityId) -> Result<(), CommandError> {
    let mut fields = BTreeMap::new();
    for k in [
        "kind",
        "by",
        "line",
        "keywords",
        "to",
        "stake",
        "stake_at",
        "stake_hl_ms",
        "with",
        "at",
        "heard",
    ] {
        fields.insert(k.to_string(), Field::Plain(Value::Null));
    }
    set(world, creature, fields)
}

/// The line as it is always said: in quotation marks, and whose words they are.
fn quoted(a: &Ask) -> String {
    format!("\u{201c}{}\u{201d} ({}'s words)", a.line, a.by)
}

/// A player's line: short, one line, printable, and checked as a name is (no
/// brackets or quotation marks, no word the protocol reserves).
fn check_line(line: &str) -> Result<String, CommandError> {
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    let n = line.chars().count();
    if !(3..=LINE_MAX).contains(&n) {
        return Err(CommandError::precondition(format!(
            "A want is 3 to {LINE_MAX} characters: what the story is about, or the word to carry."
        )));
    }
    if line.contains([
        '[', ']', '"', '\u{201c}', '\u{201d}', '`', '<', '>', '{', '}', '|', '\\',
    ]) || line.chars().any(char::is_control)
    {
        return Err(CommandError::precondition(
            "A want carries no brackets, quotation marks, backticks or marks of that kind: plain words.",
        ));
    }
    if line.contains("://") || line.to_lowercase().contains("www.") {
        return Err(CommandError::precondition(
            "A want carries no links: plain words.",
        ));
    }
    if let Some(word) = crate::gate::world_checker().collision(&line) {
        return Err(CommandError::precondition(format!(
            "'{word}' is a word CLEAR reserves; a want may not carry it."
        )));
    }
    Ok(line)
}

/// Mint what is left of a stake as a memory of `to`'s.
fn mint(
    world: &mut World,
    to: &str,
    worth: f64,
    source: &str,
) -> Result<Option<EntityId>, CommandError> {
    if worth < super::memory::FORGET_BELOW {
        return Ok(None);
    }
    let Some(f) = super::find_citizen(world, to) else {
        return Ok(None);
    };
    let curves = super::Curves::with_half_life(foxing_time::Duration::seconds(3 * 3600));
    super::memory::earn_chunk(
        world,
        to,
        f.window,
        source,
        (worth * 10.0).round() / 10.0,
        0.5,
        &curves,
    )
    .map(Some)
}

/// What a named creature says of the want a player left with it, for `look`.
pub fn describe(world: &World, creature: EntityId, actor: &str) -> Option<String> {
    let a = of(world, creature)?;
    let name = mind::name(world, creature)?;
    let worth = a.worth(world.now().0);
    let whose = if a.by == actor {
        "you left it".to_string()
    } else {
        format!("{} left it", a.by)
    };
    Some(match (a.kind.as_str(), &a.to) {
        ("word", Some(to)) => {
            let word = if a.heard.iter().any(|h| h == actor) || a.by == actor {
                format!(": {}", quoted(&a))
            } else {
                String::new()
            };
            format!("{name} also holds a word for {to}{word}; {whose}, with a memory worth {worth:.0} for the carrier.")
        }
        _ => format!(
            "{name} also wants a story about {}; {whose}, with a memory worth {worth:.0} for the teller.",
            quoted(&a)
        ),
    })
}

/// The same, as one clause of a `wants` line, and the row beside it.
pub fn clause(world: &World, creature: EntityId, actor: &str) -> Option<(String, Value)> {
    let a = of(world, creature)?;
    let worth = a.worth(world.now().0);
    let mine = a.by == actor;
    let has = mine || a.heard.iter().any(|h| h == actor);
    let said = match (a.kind.as_str(), &a.to) {
        ("word", Some(to)) => format!(
            "{} word carried to {to}{}, which pays {}'s staked memory (worth {worth:.0} now) and the carrying besides",
            if mine { "your".to_string() } else { format!("{}'s", a.by) },
            if has { format!(": {}", quoted(&a)) } else { " (stand with it or tell it, and it gives you the word)".to_string() },
            if mine { "your".to_string() } else { a.by.clone() },
        ),
        _ => format!(
            "{}a story about {}, which pays {} staked memory (worth {worth:.0} now) on top of the telling, doubled",
            if mine { "for you, " } else { "" },
            quoted(&a),
            if mine { "your".to_string() } else { format!("{}'s", a.by) },
        ),
    };
    let said = if mine {
        format!("{said}; not yours to do")
    } else {
        said
    };
    Some((
        said,
        json!({"by": a.by, "kind": a.kind, "line": if a.kind == "word" && !has { Value::Null } else { json!(a.line) }, "to": a.to, "worth": (worth * 10.0).round() / 10.0, "yours": mine}),
    ))
}

/// Standing with a creature, or telling it anything, is being given the word a
/// player left with it (as with the world's own words: D30).
pub fn hear(world: &mut World, creature: EntityId, actor: &str) -> Result<(), CommandError> {
    let Some(a) = of(world, creature) else {
        return Ok(());
    };
    if a.kind != "word" || a.heard.iter().any(|h| h == actor) {
        return Ok(());
    }
    let Some(e) = world.store.entity(creature) else {
        return Ok(());
    };
    let mut fields = e
        .components
        .get(ASK)
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    let mut heard = a.heard.clone();
    heard.push(actor.to_string());
    fields.insert("heard".into(), Field::Plain(json!(heard)));
    set(world, creature, fields)
}

pub fn hear_here(world: &mut World, room: EntityId, actor: &str) -> Result<(), CommandError> {
    let here: Vec<EntityId> = world
        .store
        .of_kind("creature")
        .filter(|e| e.parent == Some(room))
        .map(|e| e.id)
        .collect();
    for c in here {
        hear(world, c, actor)?;
    }
    Ok(())
}

/// Whether a story would meet the want a player left with `creature`, told by `actor`.
pub fn would_meet(world: &World, actor: &str, creature: EntityId, text: &str) -> bool {
    let Some(a) = of(world, creature) else {
        return false;
    };
    if a.kind != "story" || a.by == actor {
        return false;
    }
    let told: Vec<String> = super::errand::keywords(text)
        .iter()
        .map(|w| super::errand::stem(w))
        .collect();
    a.keywords
        .iter()
        .map(|w| super::errand::stem(w))
        .any(|k| told.contains(&k))
}

/// A story was told to `creature` and heard. If a player left a want with it that
/// the story touches, and the teller is not that player, the teller is paid: the
/// stake, and the world's usual on top (the telling over again, as a wanted story
/// doubles). Returns the line to say.
pub fn story_told(
    world: &mut World,
    actor: &str,
    creature: EntityId,
    text: &str,
    paid: f64,
) -> Result<Option<String>, CommandError> {
    let Some(a) = of(world, creature) else {
        return Ok(None);
    };
    if a.kind != "story" {
        return Ok(None);
    }
    let told: Vec<String> = super::errand::keywords(text)
        .iter()
        .map(|w| super::errand::stem(w))
        .collect();
    let keys: Vec<String> = a.keywords.iter().map(|w| super::errand::stem(w)).collect();
    if keys.is_empty() || !keys.iter().any(|k| told.contains(k)) {
        return Ok(None);
    }
    let name = mind::name(world, creature).unwrap_or_else(|| "it".into());
    if a.by == actor {
        return Ok(Some(format!(
            "That touches the want you left with {name} yourself; your own want is another name's to do, and the stake stays where it is."
        )));
    }
    let worth = a.worth(world.now().0);
    let stake = mint(
        world,
        actor,
        worth,
        &format!("{}'s want, told to {name}", a.by),
    )?;
    let on_top = if paid >= 0.1 {
        mint(
            world,
            actor,
            paid,
            &format!("the story {name} was asked for"),
        )?
    } else {
        None
    };
    finish(
        world,
        actor,
        creature,
        &a,
        worth + if on_top.is_some() { paid } else { 0.0 },
    )?;
    Ok(Some(format!(
        "It was the story {} asked {name} for, {}: {}'s staked memory of {} is yours{}{}.",
        a.by,
        quoted(&a),
        a.by,
        a.with,
        stake
            .map(|c| format!(", chunk-{} worth {worth:.1} XP", c.0))
            .unwrap_or_default(),
        on_top
            .map(|c| format!(
                ", and the telling over again on top, chunk-{} worth {paid:.1} XP",
                c.0
            ))
            .unwrap_or_default(),
    )))
}

/// A word was told to `receiver`. If a player left that word with another creature
/// to be carried here, and the carrier had it from that creature, the carrier is
/// paid: the stake, and the world's usual carrying on top. Returns the reply.
pub fn carried(
    world: &mut World,
    actor: &str,
    receiver: EntityId,
    text: &str,
) -> Result<Option<String>, CommandError> {
    let Some(receiver_name) = mind::name(world, receiver) else {
        return Ok(None);
    };
    let said = text.to_lowercase();
    let Some((sender, a)) = all(world).into_iter().find(|(c, a)| {
        *c != receiver
            && a.kind == "word"
            && a.to
                .as_deref()
                .is_some_and(|t| t.eq_ignore_ascii_case(&receiver_name))
            && said.contains(&a.line.to_lowercase())
    }) else {
        return Ok(None);
    };
    let sender_name = mind::name(world, sender).unwrap_or_else(|| "someone".into());
    if a.by == actor {
        return Ok(Some(format!(
            "That is the word you left with {sender_name} yourself. Your own want is another name's to carry; the stake stays where it is."
        )));
    }
    if !a.heard.iter().any(|h| h == actor) {
        return Ok(Some(format!(
            "That is the word {} left with {sender_name}, and you did not have it from {sender_name}. A word is carried by one it was given to: stand with {sender_name} or tell it anything, and it gives you the word; then bring it.",
            a.by
        )));
    }
    let worth = a.worth(world.now().0);
    let stake = mint(
        world,
        actor,
        worth,
        &format!(
            "{}'s word, carried from {sender_name} to {receiver_name}",
            a.by
        ),
    )?;
    let word_xp =
        (super::errand::WORD_XP * mind::scarcity(world) * super::almanac::wants(world).0).round();
    let on_top = mint(
        world,
        actor,
        word_xp,
        &format!("carrying a word to {receiver_name}"),
    )?;
    finish(world, actor, sender, &a, worth + word_xp)?;
    Ok(Some(format!(
        "So {} says, by {sender_name}: {}. I have it. {actor}, I will know you by that. {}'s staked memory of {} is yours{}{}.",
        a.by,
        quoted(&a),
        a.by,
        a.with,
        stake.map(|c| format!(", chunk-{} worth {worth:.1} XP", c.0)).unwrap_or_default(),
        on_top.map(|c| format!(", and the carrying on top, chunk-{} worth {word_xp:.0} XP", c.0)).unwrap_or_default(),
    )))
}

fn finish(
    world: &mut World,
    actor: &str,
    creature: EntityId,
    a: &Ask,
    paid: f64,
) -> Result<(), CommandError> {
    let name = mind::name(world, creature);
    let kind = plain_str(world, creature, CREATURE, "name")
        .unwrap_or("?")
        .to_string();
    world
        .note(
            Actor::Citizen(actor.to_string()),
            DONE,
            json!({"by": a.by, "from": actor, "creature": creature.0, "name": name, "creature_kind": kind, "kind": a.kind, "line": a.line, "to": a.to, "paid": (paid * 10.0).round() / 10.0}),
        )
        .map_err(internal)?;
    let shown = name.unwrap_or_else(|| format!("a {kind}"));
    super::combat::narrate(
        world,
        &a.by,
        format!(
            "{actor} did the want you left with {shown}, {}: your staked memory of {} went to {actor}.",
            match (a.kind.as_str(), &a.to) {
                ("word", Some(to)) => format!("and carried your word to {to}"),
                _ => "and told it the story".to_string(),
            },
            a.with
        ),
    );
    world.notice(format!(
        "**{actor}** did the want **{}** left with **{shown}**.",
        a.by
    ));
    clear(world, creature)
}

/// Give a stake back, as faded as it is, and clear the want.
fn lapse(world: &mut World, creature: EntityId, a: &Ask, why: &str) -> Result<f64, CommandError> {
    let worth = a.worth(world.now().0);
    let name = mind::name(world, creature);
    let shown = name
        .clone()
        .unwrap_or_else(|| "a creature that has lost its name".into());
    let back = mint(
        world,
        &a.by,
        worth,
        &format!("{} (staked with {shown}, come back)", a.with),
    )?;
    let _ = world.note(
        Actor::Driver,
        LAPSED,
        json!({"by": a.by, "creature": creature.0, "name": name, "why": why, "back": back.map(|_| (worth * 10.0).round() / 10.0)}),
    );
    super::combat::narrate(
        world,
        &a.by,
        format!(
            "The want you left with {shown} has lapsed ({why}){}.",
            match back {
                Some(c) => format!(
                    "; what was left of your staked memory came back: chunk-{} worth {worth:.1} XP",
                    c.0
                ),
                None => "; nothing was left of the staked memory to come back".to_string(),
            }
        ),
    );
    clear(world, creature)?;
    Ok(worth)
}

/// Once a minute: a want whose stake has faded under the floor lapses, as does one
/// whose creature has lost its name, whose word has nobody left to go to, or whose
/// player the world has forgotten.
pub fn sweep(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    for (creature, a) in all(world) {
        let why = if mind::name(world, creature).is_none() {
            Some("its name failed")
        } else if super::find_citizen(world, &a.by).is_none() {
            Some("the world has forgotten who left it")
        } else if a.kind == "word"
            && a.to
                .as_deref()
                .and_then(|t| mind::find_named(world, t))
                .is_none()
        {
            Some("the one the word was for has no name any more")
        } else if a.worth(now) < STAKE_FLOOR {
            Some("the staked memory faded")
        } else {
            None
        };
        if let Some(why) = why {
            lapse(world, creature, &a, why).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn register(world: &mut World) {
    world.on_tick(Box::new(sweep));
    world.register(
        CommandSpec::new(
            "disremembered.withdraw",
            "Take back the want you left with a named creature (`entrust`): it lets the want go and gives back what is left of the memory you staked.",
        )
        .param("creature", ParamType::Entity, true, "the named creature that holds your want")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            super::me(w, ctx)?;
            let who = args.str("creature").unwrap_or_default().trim().to_string();
            let Some((creature, _)) = mind::find_named(w, &who) else {
                return Err(CommandError::precondition(format!("No creature is named {who}.")));
            };
            let name = mind::name(w, creature).unwrap_or_default();
            let Some(a) = of(w, creature).filter(|a| a.by == ctx.actor) else {
                return Err(CommandError::precondition(format!(
                    "You have no want standing with {name}."
                )));
            };
            let worth = lapse(w, creature, &a, "you took it back")?;
            Ok(Outcome::new(
                format!("{name} lets your want go, and gives back what is left of the memory you staked: {worth:.1} XP."),
                json!({"creature": creature.0, "name": name, "back": (worth * 10.0).round() / 10.0}),
            ))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.entrust",
            "Leave a want of your own with a named creature you keep, and stake one of your memories on it. With a line alone, it wants a story about that; with a second creature's name, the line is a word to carry to that creature. Whoever does it (not you) gets the staked memory and the world's usual pay on top. The creature must already hold a memory you gave it (`remember <chunk> <creature>`), and the stake must be worth 10 or more and earned as a kept name. One want a creature, one standing want a name; it stands until done or until the stake fades, and what is left comes back to you. `withdraw <creature>` takes it back.",
        )
        .param("creature", ParamType::Entity, true, "a named creature that holds a memory you gave it")
        .param("chunk", ParamType::Entity, true, "one of your chunks, the stake")
        .param("line", ParamType::Text, false, "what the story is about, or the word to carry: 3 to 80 characters, plain words")
        .param("to", ParamType::Entity, false, "a second named creature: the line is then a word carried to it")
        .allowed(&[ActorKind::Citizen]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = super::me(w, ctx)?;
            super::kept_only(w, ctx, "leave a want")?;
            let who = args.str("creature").unwrap_or_default().trim().to_string();
            let Some((creature, _)) = mind::find_named(w, &who) else {
                return Err(CommandError::precondition(format!(
                    "No creature is named {who}. A want is left with a named creature (`wants` lists them)."
                )));
            };
            let name = mind::name(w, creature).unwrap_or_default();
            let chunk_arg = args.str("chunk").unwrap_or("").trim().to_string();
            if !mind::keepers(w, creature).iter().any(|k| k == &ctx.actor) {
                return Err(CommandError::precondition(format!(
                    "{name} holds nothing of yours, so it will not hold a want of yours either. Give it a memory first (`remember <chunk> {name}`, in its room); then it is one you keep."
                )));
            }
            if let Some(a) = of(w, creature) {
                return Err(CommandError::precondition(if a.by == ctx.actor {
                    format!("{name} already holds a want of yours, {}. `withdraw {name}` takes it back first.", quoted(&a))
                } else {
                    format!("{name} already holds a want of {}'s; one a creature. It frees when that one is done or lapses.", a.by)
                }));
            }
            if let Some((c, a)) = all(w).into_iter().find(|(_, a)| a.by == ctx.actor) {
                let other = mind::name(w, c).unwrap_or_else(|| "another creature".into());
                return Err(CommandError::precondition(format!(
                    "You have a want standing already, with {other}: {}. One at a time; `withdraw {other}` takes it back.",
                    quoted(&a)
                )));
            }
            let line = check_line(args.str("line").unwrap_or(""))?;
            let to = match args.str("to").map(str::trim).filter(|t| !t.is_empty()) {
                Some(t) => {
                    let Some((c, _)) = mind::find_named(w, t) else {
                        return Err(CommandError::precondition(format!("No creature is named {t}.")));
                    };
                    if c == creature {
                        return Err(CommandError::precondition("A word is carried to another creature, not back to the one that holds it."));
                    }
                    mind::name(w, c)
                }
                None => None,
            };
            let keywords = super::errand::keywords(&line);
            if to.is_none() && keywords.is_empty() {
                return Err(CommandError::precondition(
                    "A story is asked for by what it is about: name a thing (the line had no word a story could be about).",
                ));
            }
            let chunk = super::memory::resolve_chunk(w, ctx, &chunk_arg, &[f.window, f.palace])?;
            if super::memory::is_passing_chunk(w, chunk) {
                return Err(CommandError::precondition("That memory was earned while your name was a passing one, and carries the mark (`chunks` shows it): a passing name's memory is not staked. One earned since you kept your name is."));
            }
            let now = w.now();
            let worth = w
                .store
                .read_field(chunk, super::CHUNK, "value", now)
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            if worth < STAKE_MIN {
                return Err(CommandError::precondition(format!(
                    "That memory is worth {worth:.1} now; a stake is worth at least {STAKE_MIN:.0}, or nobody walks for it."
                )));
            }
            let with = plain_str(w, chunk, super::CHUNK, "source").unwrap_or("something").to_string();
            let hl = super::memory::halving_ms(w, chunk).unwrap_or(3_600_000);
            w.record(Actor::Citizen(ctx.actor.clone()), StoreEvent::EntityRemoved { id: chunk })
                .map_err(internal)?;
            let kind = if to.is_some() { "word" } else { "story" };
            let mut fields = BTreeMap::new();
            for (k, v) in [
                ("kind", json!(kind)),
                ("by", json!(ctx.actor)),
                ("line", json!(line)),
                ("keywords", json!(keywords)),
                ("to", json!(to)),
                ("stake", json!(worth)),
                ("stake_at", json!(now.0)),
                ("stake_hl_ms", json!(hl)),
                ("with", json!(with)),
                ("at", json!(now.0)),
                ("heard", json!([])),
            ] {
                fields.insert(k.to_string(), Field::Plain(v));
            }
            set(w, creature, fields)?;
            let ckind = plain_str(w, creature, CREATURE, "name").unwrap_or("?").to_string();
            w.note(
                Actor::Citizen(ctx.actor.clone()),
                LEFT,
                json!({"by": ctx.actor, "creature": creature.0, "name": name, "creature_kind": ckind, "kind": kind, "line": line, "to": to, "stake": (worth * 10.0).round() / 10.0}),
            )
            .map_err(internal)?;
            w.notice(format!(
                "**{}** left a want with **{name}**: {}.",
                ctx.actor,
                match &to {
                    Some(t) => format!("a word to carry to {t}"),
                    None => format!("a story about \u{201c}{line}\u{201d}"),
                }
            ));
            let fades = super::combat::human((hl as f64 * (worth / STAKE_FLOOR).log2()) as i64);
            Ok(Outcome::new(
                match &to {
                    Some(t) => format!("{name} takes the memory of {with} from you and holds it, worth {worth:.1} now, against your word for {t}: \u{201c}{line}\u{201d}. Whoever has the word from {name} and tells it to {t} gets the memory and the carrying besides. It stands until then, or about {fades}, when the memory will have faded under {STAKE_FLOOR:.0} and what is left comes back to you. `wants` lists it; `withdraw {name}` takes it back."),
                    None => format!("{name} takes the memory of {with} from you and holds it, worth {worth:.1} now, against a story about \u{201c}{line}\u{201d}. Whoever tells {name} that story (not you) gets the memory, and the telling over again. It stands until then, or about {fades}, when the memory will have faded under {STAKE_FLOOR:.0} and what is left comes back to you. `wants` lists it; `withdraw {name}` takes it back."),
                },
                json!({"creature": creature.0, "name": name, "kind": kind, "line": line, "to": to, "stake": (worth * 10.0).round() / 10.0, "with": with, "keywords": keywords}),
            ))
        }),
    );
}
