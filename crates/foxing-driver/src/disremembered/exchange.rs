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
const MAX_WORDS_PAID: usize = 60;
/// XP per word for the teller; the creature keeps half as much.
const XP_PER_WORD: f64 = 0.3;
const MAX_STORY_CHARS: usize = 1200;
/// One story per teller per creature in this long.
pub const STORY_COOLDOWN_MS: i64 = 60 * 60 * 1000;

/// Whether `actor` told `creature` a story within the hour: the cooldown.
pub fn told_this_hour(world: &World, actor: &str, creature: EntityId) -> bool {
    let now = world.now();
    world.recent_events(2000).iter().any(|e| {
        e.kind == STORY
            && e.payload["from"] == actor
            && e.payload["creature"] == creature.0
            && now.since(e.at).0 < STORY_COOLDOWN_MS
    })
}

pub fn register(world: &mut World, curves: Curves) {
    world.register(
        CommandSpec::new(
            "disremembered.give",
            "Hand one of your chunks to a citizen standing here. It is theirs then, fading on its own curve, and the gift is on the record.",
        )
        .param("chunk", ParamType::Entity, true, "a chunk of yours; it goes to them")
        .param("citizen", ParamType::Entity, true, "a citizen here"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let here = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let typed = args.str("citizen").unwrap_or("").trim().trim_start_matches("citizen-").to_string();
            let Some(to) = super::canonical_name(w, &typed) else {
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
            "Tell a named creature here a story, at least a dozen words. It keeps the story as a memory of its own, and you come away with one: the way to earn without striking anything. One story an hour per creature; a creature without a name does not understand you.",
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
            if text.chars().count() > MAX_STORY_CHARS {
                return Err(CommandError::precondition(format!(
                    "That is more than {MAX_STORY_CHARS} characters; tell it in parts."
                )));
            }
            if told_this_hour(w, &ctx.actor, creature) {
                return Err(CommandError::precondition(format!(
                    "{name} is still turning your last story over. One an hour."
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
            let wanted = super::errand::wants_story(w, creature, &text);
            // About a third of an XP a word: the creature's mood is a fifth either way,
            // so no length is the right length to the decimal.
            // A story the creature has heard lately pays less, by halves: each story
            // told to it in the last hour by anyone that shares half its words with
            // this one halves the pay. (A ring of fresh names told one creature the
            // same story six times in a minute, each name on its own cooldown.)
            let repeats = heard_lately(w, creature, &text);
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
            let heard_line = match repeats {
                0 => String::new(),
                1 => " It has heard one much like it this hour; this one pays half.".to_string(),
                n => format!(" It has heard {n} much like it this hour; this one pays a {} part.", 1u32 << n.min(20)),
            };
            let kind = super::plain_str(w, creature, super::CREATURE, "name").unwrap_or("?").to_string();
            // It keeps the story; you keep the telling.
            let window = mind::window_of(w, creature).ok_or_else(|| internal("a named creature has a window"))?;
            let kept = super::memory::earn_chunk(w, &ctx.actor, window, &format!("a story: \"{gist}\""), paid / 2.0, 0.6, &curves)?;
            // The creature's chunk is its own, not the teller's.
            let kept = mind::disown(w, kept)?;
            let earned = super::memory::earn_chunk(w, &ctx.actor, f.window, &format!("telling {name} a story"), paid, 0.4, &curves)?;
            let level = mind::level(w, creature);
            let _ = w.note(
                Actor::Citizen(ctx.actor.clone()),
                STORY,
                json!({"from": ctx.actor, "creature": creature.0, "kind": kind, "name": name, "words": words, "earned": earned.0, "kept": kept.0, "level": level, "keywords": story_words(&text), "repeats": repeats}),
            );
            // A line of its own, drawn from what it holds beside the new story: four
            // creatures once thanked a teller in exactly the same words.
            let own = super::talk::line_of_its_own(w, creature, kept);
            let answer = if wanted {
                format!("That is the one I wanted. I will keep that. \"{gist}\"{own} I hold {level:.0} now.")
            } else {
                format!("I will keep that. \"{gist}\"{own} I hold {level:.0} now.")
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
                    "You tell {display} a story of {words} words, and it answers.{}{heard_line} You remember the telling: chunk-{} worth {paid:.0} XP.",
                    if wanted { " It was the story it wanted, so the memory is worth double." } else { "" },
                    earned.0
                ),
                json!({"creature": creature.0, "name": name, "words": words, "earned": earned.0, "xp": paid, "kept": kept.0, "level": level, "wanted": wanted, "repeats": repeats}),
            ))
        }),
    );
}

/// The first sentence of a story, or its first words up to a line's worth, ending on a
/// word: a creature that repeats "memory of a story: \"Once there was a lant…\"" told a
/// visitor nothing. The ellipsis marks a cut, and only a cut.
fn gist(text: &str) -> String {
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
    let mine = story_words(text);
    if mine.is_empty() {
        return 0;
    }
    let hour_ago = world.now().0 - STORY_COOLDOWN_MS;
    world
        .recent_events(2000)
        .iter()
        .filter(|e| e.kind == STORY && e.at.0 >= hour_ago && e.payload["creature"] == creature.0)
        .filter(|e| {
            let theirs: Vec<&str> = e.payload["keywords"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();
            if theirs.is_empty() {
                return false;
            }
            let shared = mine.iter().filter(|w| theirs.contains(&w.as_str())).count();
            shared * 3 >= mine.len() * 2
        })
        .count()
}
