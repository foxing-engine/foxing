//! Memory that moves without a fight.
//!
//! `give <chunk> <citizen>` hands one of your chunks to a citizen standing here: it is
//! theirs now, on its own curve, and the gift is a public event. `story <creature>
//! "<text>"` tells a named creature a story. It keeps the story as a memory of its own,
//! and you come away with one: the first way to earn a chunk here without striking
//! anything. A creature listens to one story an hour from each teller, so stories are
//! a way in, not a mint; a creature without a name does not understand you.

use super::combat::narrate;
use super::speech::SAY;
use super::{internal, me, mind, presence, rooms, Curves};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::EntityId;
use serde_json::json;

/// A chunk changed hands: `{from, to, chunk, with, room}`.
pub const GIVEN: &str = "disremembered.given";
/// A story was told and kept: `{from, creature, kind, name, words, earned, kept}`.
pub const STORY: &str = "disremembered.story";

/// A story is at least this many words, and worth up to this many.
pub(crate) const MIN_WORDS: usize = 12;
/// Distinct words at which a story's vividity reaches 0.9 (D6).
pub const VIVID_WORDS: f64 = 200.0;
pub(crate) const MAX_WORDS_PAID: usize = 60;
/// XP per word for the teller; the creature keeps half as much.
pub(crate) const XP_PER_WORD: f64 = 0.3;
const MAX_STORY_CHARS: usize = 1200;
/// One story per teller per creature in this long.
pub const STORY_COOLDOWN_MS: i64 = 60 * 60 * 1000;

/// How many stories a creature turns over in an hour, from anyone: the creature-side
/// cap on a ring of fresh names, each on its own cooldown (approved 2026-11-02).
pub const STORIES_AN_HOUR: usize = 3;

/// How many stories `creature` has heard in the hour, from anyone.
pub fn heard_this_hour(world: &World, creature: EntityId) -> usize {
    let now = world.now();
    world
        .recent_events(2000)
        .iter()
        .filter(|e| {
            e.kind == STORY
                && e.payload["creature"] == creature.0
                && now.since(e.at).0 < STORY_COOLDOWN_MS
        })
        .count()
}

/// Minutes until a creature that is full for the hour takes another story: when the
/// oldest of the hour's stories is an hour old (the Watch of 11-25: every other
/// refusal gave minutes and this one said "until that hour turns").
pub fn room_in(world: &World, creature: EntityId) -> Option<i64> {
    let now = world.now();
    world
        .recent_events(2000)
        .iter()
        .filter(|e| e.kind == STORY && e.payload["creature"] == creature.0)
        .map(|e| STORY_COOLDOWN_MS - now.since(e.at).0)
        .filter(|ms| *ms > 0)
        .min()
        .map(|ms| (ms + 59_999) / 60_000)
}

/// How long until `actor`'s hour with `creature` turns: the time left on the newest
/// paid story (none when nothing is on the clock).
pub fn ready_in(world: &World, actor: &str, creature: EntityId) -> Option<i64> {
    let now = world.now();
    let events = world.recent_events(2000);
    let mine = events.iter().filter(|e| {
        e.kind == STORY
            && e.payload["from"] == actor
            && e.payload["creature"] == creature.0
            && e.payload["struck"] != true
            && e.payload["early"] != true
            && now.since(e.at).0 < STORY_COOLDOWN_MS
    });
    let mut heard: Option<i64> = None;
    let mut misses: Vec<i64> = Vec::new();
    for e in mine {
        let left = STORY_COOLDOWN_MS - now.since(e.at).0;
        if e.payload["missed"] == true {
            misses.push(left);
        } else {
            heard = Some(heard.map_or(left, |h: i64| h.max(left)));
        }
    }
    // R119: the hour spent by two misses turns too, when the older of them falls out
    // of the window (`wants` said "after the hour" of one creature and a countdown of
    // the next in the same list).
    misses.sort_unstable();
    heard.or_else(|| misses.len().checked_sub(2).map(|i| misses[i]))
}

/// Whether `actor` told `creature` a story within the hour: the cooldown. A story
/// that missed the creature's standing want does not spend the hour, once: the
/// reply said why, and the teller may try again (a second miss does).
///
/// R118: the hour runs from the story that opened it. One heard inside it for the
/// word (`struck`) or for the want at plain pay (`early`, D19 b) is left out, so the
/// hour does not slide forward with every telling.
pub fn told_this_hour(world: &World, actor: &str, creature: EntityId) -> bool {
    let now = world.now();
    let mut misses = 0;
    for e in world.recent_events(2000).iter().filter(|e| {
        e.kind == STORY
            && e.payload["from"] == actor
            && e.payload["creature"] == creature.0
            && now.since(e.at).0 < STORY_COOLDOWN_MS
    }) {
        if e.payload["struck"] == true || e.payload["early"] == true {
            // A story that struck the word under the others opens the next word: the
            // ladder is three, and an hour between rungs put the top out of a visit.
            continue;
        } else if e.payload["missed"] == true {
            misses += 1;
        } else {
            return true;
        }
    }
    misses >= 2
}

/// How long until `told_this_hour` stops being true, when it is.
pub fn hour_turns_in(world: &World, actor: &str, creature: EntityId) -> Option<i64> {
    let now = world.now();
    let mut plain: Option<i64> = None;
    let mut misses: Vec<i64> = Vec::new();
    for e in world.recent_events(2000).iter().filter(|e| {
        e.kind == STORY
            && e.payload["from"] == actor
            && e.payload["creature"] == creature.0
            && now.since(e.at).0 < STORY_COOLDOWN_MS
    }) {
        let left = STORY_COOLDOWN_MS - now.since(e.at).0;
        if e.payload["struck"] == true || e.payload["early"] == true {
            continue;
        } else if e.payload["missed"] == true {
            misses.push(left);
        } else {
            plain = Some(plain.map_or(left, |p| p.max(left)));
        }
    }
    // Two misses close the hour; it opens when all but one have aged out.
    misses.sort_unstable_by(|a, b| b.cmp(a));
    [plain, misses.get(1).copied()].into_iter().flatten().max()
}

/// Whether `actor` has already been heard by `creature` inside this hour for the
/// story it wanted, at plain pay (D19 b): once an hour, or the want is a farm (D24).
pub fn heard_early_this_hour(world: &World, actor: &str, creature: EntityId) -> bool {
    let now = world.now();
    world.recent_events(2000).iter().any(|e| {
        e.kind == STORY
            && e.payload["from"] == actor
            && e.payload["creature"] == creature.0
            && e.payload["early"] == true
            && now.since(e.at).0 < STORY_COOLDOWN_MS
    })
}

/// What a first story to the Reading Bench pays over its words.
pub const BENCH_WELCOME_XP: f64 = 5.0;
/// The bench heard a name's one story: `{from, room, words, text}`. Public.
pub const BENCH: &str = "disremembered.bench";

/// The names the Reading Bench's listener answers to.
pub fn is_bench(who: &str) -> bool {
    let w = who.trim().trim_start_matches("the ").to_ascii_lowercase();
    matches!(w.as_str(), "bench" | "reading bench" | "listener" | "lamp")
}

/// Whether the bench has heard this name.
pub fn bench_heard(world: &World, citizen: EntityId) -> bool {
    super::plain_bool(world, citizen, super::CITIZEN, "bench")
}

/// C1: a first story, told to nobody with a name. Once a name; it pays as a plain
/// story does, and a little for the telling of a first one.
fn bench_story(
    w: &mut World,
    actor: &str,
    f: &super::Found,
    here: EntityId,
    text: &str,
    curves: &Curves,
) -> Result<Outcome, CommandError> {
    if bench_heard(w, f.citizen) {
        return Err(CommandError::precondition(
            "The bench has heard your one story, and kept the place warm for the next name. The listeners with names take one an hour each: `wants` says who is wanting and where.",
        ));
    }
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() > MAX_STORY_CHARS {
        return Err(CommandError::precondition(format!(
            "That is more than {MAX_STORY_CHARS} characters; the bench is patient, not endless."
        )));
    }
    let distinct = text
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    if distinct < MIN_WORDS {
        return Err(CommandError::precondition(format!(
            "A story is at least {MIN_WORDS} different words; that was {distinct}. The bench waits; it hears one from each name, about anything."
        )));
    }
    let paid = (distinct.min(MAX_WORDS_PAID) as f64 * XP_PER_WORD + BENCH_WELCOME_XP).round();
    let earned = super::memory::earn_chunk(
        w,
        actor,
        f.window,
        "a first story, told at the Reading Bench",
        paid,
        0.3,
        curves,
    )?;
    let mut changes = std::collections::BTreeMap::new();
    changes.insert("bench".to_string(), foxing_store::Field::Plain(json!(true)));
    super::bindery::set_fields(
        w,
        Actor::Citizen(actor.to_string()),
        f.citizen,
        super::CITIZEN,
        changes,
    )?;
    w.note(
        Actor::Citizen(actor.to_string()),
        BENCH,
        json!({"from": actor, "room": here.0, "words": distinct, "text": text}),
    )
    .map_err(internal)?;
    for other in super::talk::others_here(w, here, actor) {
        narrate(
            w,
            &other,
            format!("{actor} sits on the bench and tells a story to the lamp. Something listens."),
        );
    }
    Ok(Outcome::new(
        format!(
            "You sit, and tell it to the lamp. Something under the bench listens to the end; it has no name and wants none, and it keeps nothing of yours. You come away with the telling: chunk-{} worth {paid:.0} XP ({distinct} different words, and a little for a first story). The bench hears one from each name. The next listener has a name and keeps what it is told: `wants` says who is wanting and where, and a story about what one wants pays double.",
            earned.0
        ),
        json!({"bench": true, "chunk": earned.0, "xp": paid, "words": distinct}),
    ))
}

pub fn register(world: &mut World, curves: Curves) {
    world.register(
        CommandSpec::new(
            "disremembered.give",
            "Hand one of your chunks, or a thing you carry, to a citizen standing here. A chunk is theirs then, fading on its own curve; a thing goes into their satchel with its past, and the gift is on the record.",
        )
        .param("chunk", ParamType::Entity, true, "a chunk of yours, or a thing you carry (item-12, or its name); it goes to them")
        .param("citizen", ParamType::Entity, true, "a citizen here"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let typed = args.str("citizen").unwrap_or("").trim().trim_start_matches("citizen-").to_string();
            let Some(to) = super::canonical_name(w, &typed) else {
                // R71: a creature or the Docent named here is not a receiver; say
                // what is, and how a creature is given to.
                let lower = typed.to_lowercase();
                if lower == "docent" || lower.starts_with("the docent") || lower == "hollis" {
                    return Err(CommandError::precondition(
                        "The Docent takes nothing by the hand. `give` reaches a name at a keyboard or through the gate who stands here; `look` marks each as such. To a creature, `remember <chunk> <creature>` is the giving.",
                    ));
                }
                if let Some(c) = mind::find_here(w, here, &typed) {
                    // The fourth level: a creature gathering a collection takes a
                    // thing by the hand, if it is the kind it wants.
                    let what = args.str("chunk").unwrap_or("").trim().to_string();
                    if !what.starts_with("chunk-") && super::calling::of(w, c).is_some() {
                        let mine = super::craft::satchel(w, &ctx.actor, f.citizen)?;
                        if let Ok(item) = super::craft::resolve_item(w, &what, mine) {
                            if let Some(done) = super::calling::given(w, &ctx.actor, c, item) {
                                return done;
                            }
                        }
                    }
                    return Err(CommandError::precondition(format!(
                        "{} is a creature; `give` reaches a name at a keyboard or through the gate who stands here (`look` marks each). To feed it a memory, `remember <chunk> {typed}`; a creature takes a thing by the hand only when it is gathering that kind (`look` says what one with a calling is about).",
                        mind::display(w, c)
                    )));
                }
                return Err(CommandError::precondition(format!("There is no one called {typed}.")));
            };
            if to == ctx.actor {
                return Err(CommandError::precondition("You give it to yourself. It is where it was."));
            }
            let them = super::find_citizen(w, &to).ok_or_else(|| CommandError::precondition(format!("There is no one called {typed}.")))?;
            if rooms::room_of(w, them.citizen) != Some(here) || !presence::is_here(w, them.citizen) {
                return Err(CommandError::precondition(format!(
                    "{to} is not here. A memory is given hand to hand."
                )));
            }
            // A thing you carry, by id or name: into their satchel, with its past.
            let what = args.str("chunk").unwrap_or("").trim().to_string();
            if !what.starts_with("chunk-") {
                let mine = super::craft::satchel(w, &ctx.actor, f.citizen)?;
                if let Ok(item) = super::craft::resolve_item(w, &what, mine) {
                    if super::places::own_of(w, item).is_some() {
                        return Err(CommandError::precondition(format!(
                            "{} is yours by your figure and is not put down or handed on.",
                            super::called::capital(&super::craft::display(w, item))
                        )));
                    }
                    let theirs = super::craft::satchel(w, &to, them.citizen)?;
                    let name = super::craft::display(w, item);
                    w.record(
                        Actor::Citizen(ctx.actor.clone()),
                        foxing_store::StoreEvent::EntityMoved { id: item, parent: Some(theirs) },
                    )
                    .map_err(internal)?;
                    let where_ = super::plain_str(w, here, super::ROOM, "name").unwrap_or("nowhere").to_string();
                    super::past::push(w, &ctx.actor, item, json!({"what": "given", "by": ctx.actor, "to": to, "room": where_}))?;
                    let _ = w.note(
                        Actor::Citizen(ctx.actor.clone()),
                        GIVEN,
                        json!({"from": ctx.actor, "to": to, "item": item.0, "name": name, "room": here.0}),
                    );
                    narrate(w, &to, format!("{} hands you {name}: item-{}. It is in your satchel, past and all.", ctx.actor, item.0));
                    w.notice(format!("**{}** gave {name} to **{to}**.", ctx.actor));
                    return Ok(Outcome::new(
                        format!("You hand {name} to {to}; it goes into their satchel with its past. `read` is theirs now too."),
                        json!({"item": item.0, "name": name, "to": to}),
                    ));
                }
            }
            let chunk = super::memory::resolve_chunk(w, ctx, args.str("chunk").unwrap_or(""), &[f.window, f.palace])?;
            let size = w.store.read_field(chunk, super::CHUNK, "value", w.now()).map_err(internal)?.as_f64().unwrap_or(0.0);
            // The Conclave's gift allowance: a day's gifts past it are taxed, the gift
            // thinning faster in the receiver's hands. The chunk that crosses the line is
            // taxed on the part that is over, not the whole (a 32 taxed in full for 7 over
            // read as a rounding bug). The receiver has an allowance too: free names could
            // each give 120 untaxed to one kept name, so the tax falls on what a name
            // *receives* past its day's share as well, whoever gives it.
            let before = super::conclave::given_today(w, &ctx.actor);
            let allowance = super::conclave::statute(w, "gift_allowance");
            let received = super::conclave::received_today(w, &to);
            let receipt = super::conclave::statute(w, "gift_receipt");
            let over_giving = ((before + size - allowance) / size.max(1e-9)).clamp(0.0, 1.0);
            let over_receiving = ((received + size - receipt) / size.max(1e-9)).clamp(0.0, 1.0);
            // A chunk a passing name earned carries the mark, and is taxed in full
            // whoever gives it: the allowance is for what a name earned as itself.
            let marked = super::memory::is_passing_chunk(w, chunk);
            let over = if marked { 1.0 } else { over_giving.max(over_receiving) };
            let taxed = over > 0.0;
            let (id, source) = mind::move_chunk(w, &ctx.actor, chunk, them.window, Some(to.clone()))?;
            if taxed {
                let keep_full = super::conclave::statute(w, "gift_tax");
                // Keep all of the half-life for the part under the line, `gift_tax` of it
                // for the part over.
                let keep = 1.0 - (1.0 - keep_full) * over;
                if let Some(foxing_store::Field::Timed(t)) = w.store.entity(id).and_then(|e| e.components.get(super::CHUNK)).and_then(|c| c.fields.get("value")).cloned() {
                    let curve = match t.curve {
                        foxing_time::Curve::Exponential { half_life } => foxing_time::Curve::Exponential { half_life: foxing_time::Duration((half_life.0 as f64 * keep) as i64) },
                        foxing_time::Curve::PowerLaw { scale, exponent } => foxing_time::Curve::PowerLaw { scale: foxing_time::Duration((scale.0 as f64 * keep) as i64), exponent },
                        c => c,
                    };
                    w.record(Actor::Citizen(ctx.actor.clone()), foxing_store::StoreEvent::FieldRecurved { id, component: super::CHUNK.into(), field: "value".into(), curve }).map_err(internal)?;
                }
                w.record(Actor::Citizen(ctx.actor.clone()), foxing_store::StoreEvent::FieldSet { id, component: super::CHUNK.into(), field: "taxed".into(), value: json!((keep * 1000.0).round() / 1000.0) }).map_err(internal)?;
            }
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                GIVEN,
                json!({"from": ctx.actor, "to": to, "chunk": id.0, "was": chunk.0, "with": source, "room": here.0, "size": size, "taxed": taxed, "over": over, "marked": marked, "given_today": before + size, "received_today": received + size}),
            );
            let tax_line = if marked {
                " It bears a passing name's mark, so the Conclave's gift tax is on all of it: it thins faster than it did.".to_string()
            } else if over_receiving > 0.0 {
                format!(" You have received {:.0} today against a share of {receipt:.0}; the Conclave's gift tax is on it, and it thins faster than it did.", received + size)
            } else if taxed {
                " The Conclave's gift tax is on it: it thins faster than it did.".to_string()
            } else {
                String::new()
            };
            narrate(w, &to, format!("{} gives you the memory of {source}: chunk-{}. It is yours now, and fading as it was.{tax_line}", ctx.actor, id.0));
            w.notice(format!("**{}** gave the memory of {source} to **{to}**.{}", ctx.actor, if taxed { " (taxed)" } else { "" }));
            Ok(Outcome::new(
                format!("You give {to} the memory of {source}. It is theirs now: chunk-{}.{}", id.0,
                    if marked {
                        format!(" It was earned by a passing name and bears the mark; the Conclave's gift tax is on all of it, whatever your allowance, and it thins faster in their hands. You have given {:.0} today of an allowance of {allowance:.0}.", before + size)
                    } else if taxed {
                        // Both tallies, and which one the tax went by: the larger of
                        // the two parts over the line, not their sum.
                        format!(" You have given {:.0} today of an allowance of {allowance:.0}; {to} has received {:.0} of a share of {receipt:.0}. The Conclave's gift tax is on {} of it ({}), and that part thins faster in their hands.",
                            before + size, received + size,
                            if over >= 1.0 { "all".to_string() } else { format!("{:.0}%", over * 100.0) },
                            if over_giving > over_receiving { "by your allowance" } else if over_receiving > over_giving { "by their share" } else { "by both" })
                    } else {
                        format!(" You have given {:.0} of the day's {allowance:.0}; {to} has received {:.0} of a share of {receipt:.0}.", before + size, received + size)
                    }),
                json!({"to": to, "chunk": id.0, "with": source, "size": size, "taxed": taxed, "over": over, "marked": marked, "given_today": before + size, "allowance": allowance, "received_today": received + size, "receipt": receipt}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.story",
            "Tell a named creature here a story, at least a dozen words. It keeps the story as a memory of its own, and keeps your words to tell to other named creatures, worn a little each time (`tales` says what the creatures in a room hold), and you come away with one: the way to earn without striking anything. One story an hour per creature, from the story that opened the hour; inside it, the one it wants is still heard once at plain pay (double when the hour turns), and one that carries its hidden word is heard for the word's bonus alone. A first miss of what it wants spends nothing; a second spends the hour. A creature without a name does not understand you, except in the Reading Rooms over the Archive, where a story is the blow: there it is a telling, of a fight you still remember, naming what you fought, with no hour to wait between tellings and a blow back for each (the card on the Mezzanine says how).",
        )
        .param("creature", ParamType::Entity, true, "a named creature here, or a thing of the Reading Rooms")
        .param("text", ParamType::Text, true, "the story, quoted"),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let who = args.str("creature").unwrap_or("").trim().to_string();
            // C1: the Reading Bench at the Postern hears one story from each name,
            // and asks no name of its listener.
            if is_bench(&who) && super::plain_str(w, here, super::ROOM, "key") == Some(rooms::POSTERN) {
                return bench_story(w, &ctx.actor, &f, here, args.str("text").unwrap_or(""), &curves);
            }
            let Some(creature) = mind::find_here(w, here, &who) else {
                if is_bench(&who) {
                    return Err(CommandError::precondition(
                        "The Reading Bench is at the Postern, the world's back gate (`where Postern`); it hears one story from each name. Here a story wants a named creature: `wants` says who is wanting.",
                    ));
                }
                // A teller sent to a creature by name is told where it is.
                let away = mind::find_named(w, &who)
                    .and_then(|(_, room)| room)
                    .and_then(|r| super::plain_str(w, r, super::ROOM, "name").map(str::to_string));
                return Err(CommandError::precondition(match away {
                    Some(place) => format!("There is no {who} here: {who} is in {place} (`travel \"{place}\"`), and a story is told face to face."),
                    None => match mind::near_named(w, &who) {
                        Some(n) => format!("There is no {who} here. Did you mean {n}? (`wants` says which named creatures are where.)"),
                        None => format!("There is no {who} here. `wants` says which named creatures are where."),
                    },
                }));
            };
            let display = mind::display(w, creature);
            if let Some(no) = super::shop::refuses(w, creature) {
                return Err(CommandError::precondition(no));
            }
            // The fourth level: on the road with something in hand, it may refuse.
            if let Some(no) = super::calling::busy(w, creature, &ctx.actor) {
                return Err(CommandError::precondition(no));
            }
            // The tapster at the Counting Bar takes a story about money, by the ounce.
            if super::plain_str(w, creature, super::CREATURE, "name") == Some("tapster") {
                return super::exergue::bar_story(w, ctx, creature, args.str("text").unwrap_or(""));
            }
            // C2: a thing of the Reading Rooms is fought by what it is told.
            if mind::name(w, creature).is_none()
                && super::plain_str(w, creature, super::CREATURE, "name").is_some_and(super::reading::is_told)
            {
                return super::reading::tell(w, ctx, &f, creature, args.str("text").unwrap_or(""));
            }
            let Some(name) = mind::name(w, creature) else {
                return Ok(Outcome::new(
                    format!("You begin, and the {display} {}. It has no name to listen with.", mind::goes_on(&display)),
                    json!({"creature": creature.0, "kept": false}),
                ));
            };
            let text = args.str("text").unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ");
            let words = text.split_whitespace().count();
            // Paid by distinct words: fifty-five "la"s paid as much as a real story once.
            let distinct = text
                .split_whitespace()
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
                .filter(|w| !w.is_empty())
                .collect::<std::collections::BTreeSet<_>>()
                .len();
            if distinct < MIN_WORDS {
                // With what it wants, so the next try is the one that pays double: a
                // person tried "move away" and "run" on a wispling and left.
                let wants = super::errand::of(w, creature)
                    .and_then(|e| e.story)
                    .map(|want| format!(" {} wants a story about {want}; a dozen words on that pays double.", mind::display(w, creature)))
                    .unwrap_or_default();
                return Err(CommandError::precondition(format!(
                    "A story is at least {MIN_WORDS} different words; that was {distinct}.{wants}"
                )));
            }
            // Told in sentences, or not followed: the hour is not spent on a salad.
            if let Some(why) = super::grammar::why_unfollowable(&text) {
                let mut who = display.chars();
                let who = match who.next() {
                    Some(c) => c.to_uppercase().collect::<String>() + who.as_str(),
                    None => String::new(),
                };
                return Err(CommandError::precondition(format!(
                    "{who} cannot follow that: {why}. Tell it again, in sentences; the hour is not spent."
                )));
            }
            if text.chars().count() > MAX_STORY_CHARS {
                return Err(CommandError::precondition(format!(
                    "That is more than {MAX_STORY_CHARS} characters; tell it in parts."
                )));
            }
            // X16: a story this creature already holds a copy of, told again by its
            // first teller (set right) or, its name worn off, by anyone (made theirs).
            // Paid plain, once for each creature and story; no hour is spent on it.
            // A second mending of the same copy mends and goes on as a story does.
            if let Some(r) = super::tales::retold(w, &ctx.actor, creature, &text)? {
                if r.pay {
                    let paid = (super::jitter(w, distinct.min(MAX_WORDS_PAID) as f64 * XP_PER_WORD, 0.8, 1.2)
                        * super::economy::rate(w, "pay"))
                    .round();
                    let source = if r.claimed {
                        format!("telling {name} a story nobody's name was on")
                    } else {
                        format!("setting right the story {name} had of yours")
                    };
                    let earned = super::memory::earn_chunk(w, &ctx.actor, f.window, &source, paid, 0.5, &curves)?;
                    mind::met(w, creature, &ctx.actor)?;
                    return Ok(Outcome::new(
                        format!(
                            "{} You remember it: chunk-{} worth {paid:.1} XP. (Paid once for this story with {name}; no hour of its listening is spent, and `tales` says what it holds.)",
                            r.line, earned.0
                        ),
                        json!({"creature": creature.0, "name": name, "tale": r.tale, "mended": !r.claimed, "claimed": r.claimed, "earned": earned.0, "paid": paid}),
                    ));
                }
            }
            // D50 (b): a story it already holds whole under your name, told to it
            // again, is kept in place of the old telling for five days more and pays
            // nothing (told once an hour it kept a copy in the world for good and paid
            // each time). The one it wants, one that carries its word, and one a name
            // left a want for are heard as stories are, below.
            // (The one it wants was let through too until 11-25: two of that Watch told
            // a wanted story twice in its hour and were paid for both.)
            if super::tales::holds_whole(w, creature, &ctx.actor, &text)
                && !super::secret::would_strike(w, &ctx.actor, creature, &text)
                && !super::asks::would_meet(w, &ctx.actor, creature, &text)
            {
                super::tales::heard(w, creature, &ctx.actor, &text, 0)?;
                mind::met(w, creature, &ctx.actor)?;
                return Ok(Outcome::new(
                    format!(
                        "{display} has this one of yours already, whole, and keeps this telling in its place for five days more. A story it holds pays once: nothing is paid for telling it again, and no hour of its listening is spent. (`tales` says what it holds; a new story is heard as one.)"
                    ),
                    json!({"creature": creature.0, "name": name, "renewed": true, "paid": 0}),
                ));
            }
            // D13: within its hour with you, a story that carries the word under the
            // others is still heard, paying the bonus alone (the hint once arrived
            // after the hour had closed the only creature).
            let within_hour = told_this_hour(w, &ctx.actor, creature);
            let was_wanted = super::errand::wants_story(w, creature, &text);
            let bonus_only = within_hour && super::secret::would_strike(w, &ctx.actor, creature, &text);
            // D19 (b): the story it wants is heard inside the hour too, at base pay,
            // not double.
            let wanted_early = within_hour && !bonus_only && super::errand::wants_story(w, creature, &text);
            // D24 (lean a): the want is heard early once an hour; a second wanted
            // story inside it waits with the rest.
            if wanted_early && heard_early_this_hour(w, &ctx.actor, creature) {
                return Err(CommandError::precondition(format!(
                    "{name} has heard the story it wanted from you inside this hour already, at plain pay; another waits for the hour to turn, when it pays double. `wants` says when."
                )));
            }
            // D32 a: at the third rung, one guess at the word every three minutes. A
            // story sent sooner is not weighed at all: no strike, no miss, no hint.
            // D40c: a creature full for the hour weighs a guess at its word as one
            // inside the teller's own hour does, and a story that carries the word
            // is heard by it even then (`wants` had promised so).
            let full = heard_this_hour(w, creature) >= STORIES_AN_HOUR;
            let strikes = super::secret::would_strike(w, &ctx.actor, creature, &text);
            if (within_hour || (full && !strikes)) && !wanted_early && super::secret::would_strike_any(w, &ctx.actor, creature) {
                if let Some(ms) = super::secret::next_guess_in(w, &ctx.actor, creature) {
                    return Err(CommandError::precondition(format!(
                        "{name} weighs one guess at its third and last word from a name every three minutes, and weighed one of yours {}; it will weigh another in {}. That story was not weighed: it is neither a miss nor the word. (The story it wants is still heard, at plain pay.)",
                        if super::secret::GUESS_EVERY_MS - ms < 60_000 { "just now".to_string() } else { format!("{} ago", super::combat::human(super::secret::GUESS_EVERY_MS - ms)) },
                        super::combat::human(ms)
                    )));
                }
            }
            if within_hour && !bonus_only && !wanted_early {
                let word_left = super::secret::would_strike_any(w, &ctx.actor, creature);
                // D20 b: a refused guess is a miss on the record; the hint below
                // has already narrowed by it.
                if word_left {
                    super::secret::declined(w, &ctx.actor, creature, &text)?;
                }
                // R117, and D40c for every rung and source: a story that carried a
                // word its word could be, and was not it, is told so by the word.
                let not_it = super::secret::miss_said(w, &ctx.actor, creature, &text);
                return Err(CommandError::precondition(format!(
                    "{name} has heard a story of yours within the hour and takes one an hour (its hour with you{}). {}{not_it}{}",
                    match hour_turns_in(w, &ctx.actor, creature) {
                        Some(ms) => format!(", which turns in {}", super::combat::human(ms)),
                        None => "; `wants` says when it turns".to_string(),
                    },
                    if word_left {
                        "A story that carries the word under the others is heard even now, for the bonus alone, and the one it wants is heard at plain pay; that one was neither."
                    } else {
                        "The one it wants is heard at plain pay even now; that one was not it, and you have its words already."
                    },
                    super::secret::hint(w, &ctx.actor, creature)
                        .filter(|_| word_left)
                        .map(|h| format!(" {h}"))
                        .unwrap_or_default()
                )));
            }
            if full && !bonus_only && !wanted_early && !strikes {
                // D40c: a guess at its word is weighed all the same, and answered.
                let word_left = super::secret::would_strike_any(w, &ctx.actor, creature);
                if word_left {
                    super::secret::declined(w, &ctx.actor, creature, &text)?;
                }
                let not_it = super::secret::miss_said(w, &ctx.actor, creature, &text);
                return Err(CommandError::precondition(format!(
                    "{name} has heard {STORIES_AN_HOUR} stories in its hour and is turning them over; it takes no more until that hour turns{}{}. `wants` names another.{not_it}",
                    match room_in(w, creature) {
                        Some(m) => format!(" (in {m} minute{})", if m == 1 { "" } else { "s" }),
                        None => String::new(),
                    },
                    if word_left {
                        let n = super::secret::tries(w, &ctx.actor, creature);
                        format!(", but for one that carries the word under the others, and it weighed that one for it ({n} tr{} of yours at this word so far)", if n == 1 { "y" } else { "ies" })
                    } else { String::new() }
                )));
            }
            // The story is said, as itself.
            w.note(
                Actor::Citizen(ctx.actor.clone()),
                SAY,
                json!({"from": ctx.actor, "room": here.0, "to": display, "story": true, "text": text}),
            )
            .map_err(internal)?;
            // The story it wanted is worth double, and the errand is done.
            // Inside the hour (D19 b) the wanted story is heard at plain pay and the
            // want stands: told again when the hour turns, it pays double.
            let wanted = super::errand::wants_story(w, creature, &text) && !wanted_early;
            // About a third of an XP a word: the creature's mood is a fifth either way,
            // so no length is the right length to the decimal.
            // A story the creature has heard lately pays less, by halves: each story
            // told to it in the last hour by anyone that shares half its words with
            // this one halves the pay. (A ring of fresh names told one creature the
            // same story six times in a minute, each name on its own cooldown.)
            let (repeats, alike) = heard_lately_with(w, creature, &text);
            let paid = (super::jitter(w, distinct.min(MAX_WORDS_PAID) as f64 * XP_PER_WORD, 0.8, 1.2)
                * super::economy::rate(w, "pay")
                * 0.5f64.powi(repeats as i32))
            .round()
                * if wanted {
                    // The one it wanted is worth double, and more when names are few.
                    2.0 * super::mind::scarcity(w) * super::almanac::wants(w).0
                } else {
                    1.0
                };
            let gist = gist(&text);
            // R199: a story heard as the wanted one, at double or at plain pay inside
            // the hour, is not also "not the one it wanted" (the want it met and the
            // want that stands were two, and one reply said both).
            // C7: nor is one that met the want a player left with it (one reply paid
            // a stake and said "Not the one it wanted").
            let met_ask = super::asks::would_meet(w, &ctx.actor, creature, &text);
            let missed = if wanted || wanted_early || met_ask { None } else { super::errand::why_missed(w, creature, &text) };
            // The word under the others: a story that carries it pays the level's bonus.
            let as_wanted = (wanted || wanted_early) && super::secret::wanted_waives(w, creature);
            let over = if as_wanted { None } else { super::secret::overreach(w, &ctx.actor, creature, &text) };
            // R214: what the story carried of the words its word could be, taken
            // before the strike moves the word on.
            let guessed = super::secret::carried_candidates(w, &ctx.actor, creature, &text);
            let struck = super::secret::strike_as(w, &ctx.actor, creature, &text, as_wanted)?;
            // The grace, said right: the first miss of the hour spends nothing; the
            // second spends it ("once" on both read as a lie, the Watch of 11-07). A
            // story heard for the word spends no hour at all, and is not a miss on the
            // record (R118: "That is the second miss" closed every strike reply).
            let missed = missed.map(|m| {
                if struck.is_some() {
                    return format!("{m} A story that carries the word under the others spends no hour.");
                }
                let prior = w
                    .recent_events(2000)
                    .iter()
                    .filter(|e| {
                        e.kind == STORY
                            && e.payload["from"] == ctx.actor
                            && e.payload["creature"] == creature.0
                            && e.payload["missed"] == true
                            && e.payload["struck"] != true
                            && w.now().since(e.at).0 < STORY_COOLDOWN_MS
                    })
                    .count();
                if prior == 0 {
                    format!("{m} The hour is not spent on a first miss; a second spends it.")
                } else {
                    format!("{m} That is the second miss; the hour is spent.")
                }
            });
            let paid = match &struck {
                // Within the hour, the bonus alone (D13): what the word adds, not the pay.
                Some((bonus, _, _)) if bonus_only => (paid * (bonus - 1.0)).round(),
                Some((bonus, _, _)) => paid * bonus,
                None => paid,
            };
            // L2: being known pays. A creature that knew the teller before this story
            // pays a teller of standing more (the Watch of 11-21: "a creature knowing
            // me changed no payout").
            let known = super::bard::known_bonus(w, &ctx.actor, creature);
            // C21: a teller's lute, in the hand: the first story a creature has of
            // this name in a day pays a tenth more.
            let lute = super::craft::carries(w, f.citizen, "lute").is_some() && {
                let now = w.now();
                !w.recent_events(2000).iter().any(|e| {
                    e.kind == STORY && e.payload["creature"] == creature.0 && e.payload["from"] == ctx.actor.as_str() && now.since(e.at).0 < 86_400_000
                })
            };
            let paid = (paid * known * if lute { 1.1 } else { 1.0 }).round();
            if wanted {
                super::bard::did(w, &ctx.actor, "want");
            }
            if struck.is_some() {
                super::bard::did(w, &ctx.actor, "word");
            }
            let alike_said = if alike.is_empty() {
                String::new()
            } else {
                // The words are kept as stems ("alway, arrived, became again" was said
                // to a teller): how many, not which.
                format!(" ({} of its words over again)", alike.len())
            };
            let heard_line = match repeats {
                0 => String::new(),
                1 => format!(" It has heard one much like it in its hour{alike_said}; this one pays half."),
                n => format!(" It has heard {n} much like it in its hour{alike_said}; this one pays a {} part.", 1u32 << n.min(20)),
            };
            let kind = super::plain_str(w, creature, super::CREATURE, "name").unwrap_or("?").to_string();
            // It keeps the story; you keep the telling.
            let window = mind::window_of(w, creature).ok_or_else(|| internal("a named creature has a window"))?;
            let kept = super::memory::earn_chunk(w, &ctx.actor, window, &format!("a story: \"{gist}\""), paid / 2.0, 0.6, &curves)?;
            // The creature's chunk is its own, not the teller's.
            let kept = mind::disown(w, kept)?;
            // Length buys vividity, never XP (D6, Tyler #11): words past the pay cap
            // raise the telling's vividity from 0.4 toward 0.9 at two hundred distinct
            // words; the word under the others is 0.8 at least.
            let long = ((distinct as f64 - MAX_WORDS_PAID as f64) / (VIVID_WORDS - MAX_WORDS_PAID as f64)).clamp(0.0, 1.0);
            let vivid = (0.4 + 0.5 * long).max(if struck.is_some() { 0.8 } else { 0.0 });
            let earned = super::memory::earn_chunk(w, &ctx.actor, f.window, &format!("telling {name} a story"), paid, vivid, &curves)?;
            let level = super::up(mind::level(w, creature));
            mind::met(w, creature, &ctx.actor)?;
            // X16: it keeps the words, to pass on.
            let tale_said = super::tales::heard(w, creature, &ctx.actor, &text, earned.0)?;
            // D33: where its word is a meaning, the story goes to be weighed against
            // it, and the creature says what it made of it a little later.
            let tale_said = if struck.is_none() {
                format!("{tale_said}{}", super::meaning::ask(w, &ctx.actor, creature, &text, paid)?)
            } else {
                tale_said
            };
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                STORY,
                json!({"from": ctx.actor, "creature": creature.0, "kind": kind, "name": name, "words": words, "earned": earned.0, "kept": kept.0, "level": level, "keywords": story_words(&text), "repeats": repeats, "missed": missed.is_some() && struck.is_none(), "struck": struck.is_some(), "early": wanted_early, "wanted": was_wanted}),
            );
            // A line of its own, drawn from what it holds beside the new story: four
            // creatures once thanked a teller in exactly the same words.
            let own = super::talk::line_of_its_own(w, creature, kept);
            // David, 11-22: "quiet it during the protection time". A new teller's
            // first stories each ended on a paragraph about a word it had not been
            // told existed. A strike is still said (and paid); a miss is not.
            let quiet = super::combat::sheltered(w, f.citizen);
            let under = match &struck {
                Some((_, word, lvl)) => format!(" You said the word under the others: {word}.{}{}", if *lvl < super::secret::LEVELS { " There is another under that." } else { " There is none under that; you have them all." }, if bonus_only { " It was heard inside your hour with it, for the word; the next from you that carries none waits for the hour." } else { "" }),
                None if quiet => String::new(),
                // D29: too many guesses in one story at the higher rungs.
                None => match over {
                    Some((n, heard)) => format!(
                        " You said {n} of the words my word could be; I hear {} to a story at this rung, so none of them counted.",
                        if heard == 1 { "one".to_string() } else { heard.to_string() }
                    ),
                    // R214: a story heard and not the word says so, every time: two
                    // Watches' tellers could not tell a wrong guess from one unweighed.
                    None => match &guessed {
                        Some(g) if g.is_empty() => " Of my word (I keep one, and a story that says it is paid more): you said none of the words it could be. A longer word of this room, five letters or more, and not a plural.".to_string(),
                        Some(g) if g.len() == 1 => format!(" Of my word: you said {}, which it could have been, and is not.", super::secret::as_told(&text, g)[0]),
                        Some(g) => format!(" Of my word: you said {}, which it could have been; it is none of them.", super::secret::as_told(&text, g).join(" and ")),
                        None => String::new(),
                    },
                },
            };
            let answer = if wanted {
                format!("That is the one I wanted. I will keep that. \"{gist}\"{own}{under} I hold {level:.0} now.")
            } else {
                format!("I will keep that. \"{gist}\"{own}{under} I hold {level:.0} now.")
            };
            super::talk::speak(w, creature, &ctx.actor, &answer)?;
            if wanted {
                super::errand::story_done(w, &ctx.actor, creature, &text)?;
            }
            // C7: and the want a player left with it, if the story is that one.
            let asked = super::asks::story_told(w, &ctx.actor, creature, &text, paid)?;
            if let Some(line) = &asked {
                super::combat::narrate(w, &ctx.actor, line.clone());
            }
            mind::check_tier(w, creature)?;
            // A grudge against this teller is settled by a story it kept.
            if let Some(line) = super::calling::settled_by(w, creature, &ctx.actor) {
                super::combat::narrate(w, &ctx.actor, line.trim().to_string());
            }
            Ok(Outcome::new(
                // The creature's answer reaches the teller as it reaches the room, a
                // narrator line, so the outcome does not repeat it.
                format!(
                    "You tell {display} a story of {words} words, and it answers.{}{}{heard_line} You remember the telling: chunk-{} worth {paid:.1} XP{}. {display} keeps half as much, {:.1}, as its own memory of the story.{tale_said}{}{}",
                    if wanted_early { " It was the story it wanted, heard inside its hour with you at plain pay (once an hour); the want stands, and told again when its hour turns it pays double. A story counts as the wanted one by what it is about, meant so or not, and that was this hour's one early hearing." } else if wanted { " It was the story it wanted, so the memory is worth double." } else { "" },
                    struck.as_ref().map(|(b, _, l)| format!(" It carried the word under the others, the {} of three: {}, and vivid.", ["first", "second", "third"][(*l as usize).clamp(1, 3) - 1], if bonus_only { format!("heard inside its hour with you, so the bonus alone, {paid:.1} XP: {} of what a story pays", match (b * 10.0).round() as i64 { 15 => "half again".to_string(), 20 => "as much again".to_string(), 25 => "one and a half times again".to_string(), _ => format!("{:.1} times", b - 1.0) }) } else { format!("{b:.1} times the pay, {paid:.1} XP") })).unwrap_or_default(),
                    earned.0,
                    if vivid > 0.4 { format!(", vividity {vivid:.2}{}", if long > 0.0 && struck.is_none() { " (length buys vividity, to 0.90 at two hundred words)" } else { "" }) } else { String::new() },
                    // R160: the teller's pay and the creature's keep are two figures.
                    paid / 2.0,
                    missed.as_deref().map(|m| format!(" {m}")).unwrap_or_default(),
                    if known > 1.0 { format!(" ({display} knew you already, and pays a {} {known:.1} times.)", super::bard::title(w, &ctx.actor).unwrap_or("teller")) } else { String::new() }
                ) + if lute { " (A tenth more, for the lute in your hand: its first story of yours today.)" } else { "" },
                json!({"creature": creature.0, "name": name, "words": words, "distinct": distinct, "earned": earned.0, "xp": paid, "kept": kept.0, "level": level, "wanted": wanted, "wanted_early": wanted_early, "missed": missed, "repeats": repeats, "struck": struck.as_ref().map(|(b, w, l)| json!({"word": w, "level": l, "bonus": b}))}),
            ))
        }),
    );
}

/// The first sentence of a story, or its first words up to a line's worth, ending on a
/// word: a creature that repeats "memory of a story: \"Once there was a lant…\"" told a
/// visitor nothing. The ellipsis marks a cut, and only a cut.
pub fn gist(text: &str) -> String {
    const LINE: usize = 56;
    let first = text
        .split_inclusive(['.', '!', '?'])
        .next()
        .unwrap_or(text)
        .trim();
    if first.chars().count() <= LINE {
        return first.to_string();
    }
    let mut out = String::new();
    for word in first.split_whitespace() {
        if !out.is_empty() && out.chars().count() + 1 + word.chars().count() > LINE {
            break;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    format!("{}\u{2026}", out.trim_end_matches([',', ';', ':']))
}

/// Words that are in every story and tell none apart: not counted when one story is
/// compared with the next. (A fresh story about the lantern nobody remembers
/// lighting once read as half the last one's on "every", "night" and "though".)
const STORY_FILLER: &[&str] = &[
    "about", "after", "also", "been", "being", "before", "could", "does", "done", "even", "ever",
    "every", "from", "have", "into", "just", "like", "more", "most", "only", "other", "over",
    "said", "same", "says", "some", "still", "such", "than", "that", "their", "them", "then",
    "there", "they", "this", "those", "though", "through", "under", "until", "upon", "very",
    "were", "what", "when", "where", "which", "while", "with", "would", "your",
];

/// The words a story is made of, for telling one from the next: four letters or
/// more, lowercased, plurals trimmed, the filler left out, each once.
pub fn story_words(text: &str) -> Vec<String> {
    let mut v: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .map(|w| w.trim_matches('\'').to_lowercase())
        .filter(|w| w.chars().count() >= 4)
        .filter(|w| !STORY_FILLER.contains(&w.as_str()))
        .map(|w| w.trim_end_matches('s').to_string())
        .collect();
    v.sort();
    v.dedup();
    v
}

/// How many stories told to `creature` in the last hour, by anyone, this one is
/// mostly made of: two thirds of its words or more in common, filler aside. Two
/// different answers to the same want share the want's own words (lantern, nobody,
/// remember, lighting) and must not read as one.
pub fn heard_lately(world: &World, creature: foxing_store::EntityId, text: &str) -> usize {
    heard_lately_with(world, creature, text).0
}

/// How many stories much like this one it heard in the hour, and the words the
/// likest shared (R112: said, so the halving is not invisible).
pub fn heard_lately_with(
    world: &World,
    creature: foxing_store::EntityId,
    text: &str,
) -> (usize, Vec<String>) {
    let mine = story_words(text);
    if mine.is_empty() {
        return (0, Vec::new());
    }
    let hour_ago = world.now().0 - STORY_COOLDOWN_MS;
    let mut count = 0;
    let mut best: Vec<String> = Vec::new();
    for e in world
        .recent_events(2000)
        .iter()
        .filter(|e| e.kind == STORY && e.at.0 >= hour_ago && e.payload["creature"] == creature.0)
    {
        let theirs: Vec<&str> = e.payload["keywords"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        if theirs.is_empty() {
            continue;
        }
        let shared: Vec<String> = mine
            .iter()
            .filter(|w| theirs.contains(&w.as_str()))
            .cloned()
            .collect();
        if shared.len() * 3 >= mine.len() * 2 {
            count += 1;
            if shared.len() > best.len() {
                best = shared;
            }
        }
    }
    (count, best)
}
