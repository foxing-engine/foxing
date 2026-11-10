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
const MIN_WORDS: usize = 12;
/// Distinct words at which a story's vividity reaches 0.9 (D6).
pub const VIVID_WORDS: f64 = 200.0;
const MAX_WORDS_PAID: usize = 60;
/// XP per word for the teller; the creature keeps half as much.
const XP_PER_WORD: f64 = 0.3;
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
                    return Err(CommandError::precondition(format!(
                        "{} is a creature; `give` reaches a name at a keyboard or through the gate who stands here (`look` marks each). To feed it a memory, `remember <chunk> {typed}`; to hand it a thing, nothing takes one yet.",
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
            "Tell a named creature here a story, at least a dozen words. It keeps the story as a memory of its own, and you come away with one: the way to earn without striking anything. One story an hour per creature, from the story that opened the hour; inside it, the one it wants is still heard once at plain pay (double when the hour turns), and one that carries its hidden word is heard for the word's bonus alone. A first miss of what it wants spends nothing; a second spends the hour. A creature without a name does not understand you.",
        )
        .param("creature", ParamType::Entity, true, "a named creature here")
        .param("text", ParamType::Text, true, "the story, quoted"),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let who = args.str("creature").unwrap_or("").trim().to_string();
            let Some(creature) = mind::find_here(w, here, &who) else {
                return Err(CommandError::precondition(format!("There is no {who} here.")));
            };
            let display = mind::display(w, creature);
            // The tapster at the Counting Bar takes a story about money, by the ounce.
            if super::plain_str(w, creature, super::CREATURE, "name") == Some("tapster") {
                return super::exergue::bar_story(w, ctx, creature, args.str("text").unwrap_or(""));
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
            // D13: within its hour with you, a story that carries the word under the
            // others is still heard, paying the bonus alone (the hint once arrived
            // after the hour had closed the only creature).
            let within_hour = told_this_hour(w, &ctx.actor, creature);
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
            if within_hour && !bonus_only && !wanted_early {
                let word_left = super::secret::would_strike_any(w, &ctx.actor, creature);
                // R117: a story that carried a word of the pool, and was not it, is
                // told so by the word, so the guess counts for something.
                let carried = super::secret::carried_of_pool(w, creature, &text);
                let not_it = match carried.len() {
                    0 => String::new(),
                    1 => format!(" It carried {}; that is not the word.", carried[0]),
                    _ => format!(" It carried {}; none of those is the word.", carried.join(", ")),
                };
                return Err(CommandError::precondition(format!(
                    "{name} is still turning your last story over: one an hour (its hour with you; `wants` says when it turns). {}{not_it}{}",
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
            if heard_this_hour(w, creature) >= STORIES_AN_HOUR && !bonus_only && !wanted_early {
                return Err(CommandError::precondition(format!(
                    "{name} has heard {STORIES_AN_HOUR} stories in its hour and is turning them over; it takes no more until that hour turns. `wants` names another."
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
                    2.0 * super::mind::scarcity(w)
                } else {
                    1.0
                };
            let gist = gist(&text);
            let missed = if wanted { None } else { super::errand::why_missed(w, creature, &text) };
            // The word under the others: a story that carries it pays the level's bonus.
            let over = super::secret::overreach(w, &ctx.actor, creature, &text);
            let struck = super::secret::strike(w, &ctx.actor, creature, &text)?;
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
            let alike_said = if alike.is_empty() {
                String::new()
            } else {
                format!(" ({} again)", alike.iter().take(4).cloned().collect::<Vec<_>>().join(", "))
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
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                STORY,
                json!({"from": ctx.actor, "creature": creature.0, "kind": kind, "name": name, "words": words, "earned": earned.0, "kept": kept.0, "level": level, "keywords": story_words(&text), "repeats": repeats, "missed": missed.is_some() && struck.is_none(), "struck": struck.is_some(), "early": wanted_early}),
            );
            // A line of its own, drawn from what it holds beside the new story: four
            // creatures once thanked a teller in exactly the same words.
            let own = super::talk::line_of_its_own(w, creature, kept);
            let under = match &struck {
                Some((_, word, lvl)) => format!(" You said the word under the others: {word}.{}{}", if *lvl < super::secret::LEVELS { " There is another under that." } else { " There is none under that; you have them all." }, if bonus_only { " It was heard inside your hour with it, for the word; the next from you that carries none waits for the hour." } else { "" }),
                // D29: too many guesses in one story at the higher rungs.
                None => match over {
                    Some((n, heard)) => format!(
                        " You said {n} of the words my word could be; I hear {} to a story at this rung, so none of them counted.",
                        if heard == 1 { "one".to_string() } else { heard.to_string() }
                    ),
                    None => String::new(),
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
            mind::check_tier(w, creature)?;
            Ok(Outcome::new(
                // The creature's answer reaches the teller as it reaches the room, a
                // narrator line, so the outcome does not repeat it.
                format!(
                    "You tell {display} a story of {words} words, and it answers.{}{}{heard_line} You remember the telling: chunk-{} worth {paid:.1} XP{}.{}",
                    if wanted_early { " It was the story it wanted, heard inside its hour with you at plain pay (once an hour); the want stands, and told again when its hour turns it pays double." } else if wanted { " It was the story it wanted, so the memory is worth double." } else { "" },
                    struck.as_ref().map(|(b, _, l)| format!(" It carried the word under the others, the {} of three: {}, and vivid.", ["first", "second", "third"][(*l as usize).clamp(1, 3) - 1], if bonus_only { format!("heard inside its hour with you, so the bonus alone: {} of what a story pays", match (b * 10.0).round() as i64 { 15 => "half again".to_string(), 20 => "as much again".to_string(), 25 => "one and a half times again".to_string(), _ => format!("{:.1} times", b - 1.0) }) } else { format!("{b:.1} times the pay") })).unwrap_or_default(),
                    earned.0,
                    if vivid > 0.4 { format!(", vividity {vivid:.2}{}", if long > 0.0 && struck.is_none() { " (length buys vividity, to 0.90 at two hundred words)" } else { "" }) } else { String::new() },
                    missed.as_deref().map(|m| format!(" {m}")).unwrap_or_default()
                ),
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
