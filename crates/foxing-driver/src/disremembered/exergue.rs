//! The Exergue District: eight rooms under the Clocktower, one for each coin from the
//! farthing to the sovereign, that mirror the coinage in circulation. The exergue is
//! the strip under a coin's device where the date and the mintmark go: the part that
//! says when and where, never what.
//!
//! The owner's brief (2026-11-02): "8 rooms to mirror the coinage in circulation,
//! devastating guards to protect the purse, a bar where a story about money is worth
//! its weight, just not in gold." So: every room's `read` is the economy as it stands
//! (the Die Room's rack is every chunk in the world by what struck it; the Striking
//! Floor's plate is the hour's rates; the Assay Office's balance weighs what you hold;
//! the Mintmark Hall's register is everything kept, with its date); the tapster at the
//! Counting Bar takes a story about money and pays its weight (the longest-lived
//! memory in the game, a day's half-life) in something other than gold (a handful of
//! XP); and the Purse in the Purse Room, a twentieth of all memory held in the world,
//! is taken only past two Wardens, the most dangerous things the world has.

use super::{me, plain_str, CHUNK, CREATURE, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{CommandError, Outcome};
use foxing_store::EntityId;
use serde_json::json;

/// A story weighed at the Counting Bar: `{from, words, weight_xp, half_life_ms, chunk}`. Public.
pub const WEIGHED: &str = "disremembered.weighed";
/// The Purse was taken: `{by, worth, held, wardens_felled}`. Public.
pub const PURSE_TAKEN: &str = "disremembered.purse_taken";

/// What a bar story pays: little, and slow to fade.
const BAR_XP_PER_WORD: f64 = 0.12;
const BAR_XP_MAX: f64 = 12.0;
const BAR_HALF_LIFE_S: i64 = 24 * 3600;
const BAR_MIN_WORDS: usize = 12;
/// The Purse: this share of all held memory, this much at least, capped on the taker.
const PURSE_SHARE: f64 = 0.05;
const PURSE_FLOOR: f64 = 120.0;
const PURSE_OVER_LEVEL: f64 = 200.0;
/// The Purse refills this long after it is taken.
const PURSE_REFILL_MS: i64 = 3_600_000;

/// Words a story about money is known by.
const MONEY: &[&str] = &[
    "money",
    "coin",
    "coins",
    "gold",
    "silver",
    "copper",
    "purse",
    "price",
    "pay",
    "paid",
    "owe",
    "owed",
    "debt",
    "debts",
    "wage",
    "wages",
    "tax",
    "taxes",
    "rent",
    "fine",
    "fines",
    "fee",
    "fees",
    "bank",
    "mint",
    "minted",
    "penny",
    "pennies",
    "shilling",
    "crown",
    "pound",
    "dollar",
    "cent",
    "cents",
    "farthing",
    "groat",
    "sovereign",
    "florin",
    "ledger",
    "tally",
    "account",
    "accounts",
    "worth",
    "wealth",
    "rich",
    "poor",
    "cost",
    "costs",
    "bought",
    "sold",
    "sell",
    "buy",
    "trade",
    "traded",
    "change",
    "counterfeit",
    "bribe",
    "ransom",
    "tithe",
    "treasury",
    "fortune",
    "interest",
    "loan",
    "lend",
    "lent",
    "borrow",
    "borrowed",
    "cheap",
    "dear",
    "spend",
    "spent",
    "savings",
    "profit",
    "loss",
    "ounce",
    "weight",
];

/// `story tapster "…"` at the Counting Bar.
pub fn bar_story(
    world: &mut World,
    ctx: &foxing_registry::CallContext,
    tapster: EntityId,
    text: &str,
) -> Result<Outcome, CommandError> {
    let f = me(world, ctx)?;
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let words: Vec<String> = text
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();
    let distinct: std::collections::BTreeSet<&String> = words.iter().collect();
    if distinct.len() < BAR_MIN_WORDS {
        return Err(CommandError::precondition(format!(
            "The tapster weighs nothing under {BAR_MIN_WORDS} different words; that was {}.",
            distinct.len()
        )));
    }
    let money = words.iter().filter(|w| MONEY.contains(&w.as_str())).count();
    if money == 0 {
        return Ok(Outcome::new(
            "The tapster listens to the end, dries the glass, and says nothing. It weighs stories about money; that one had none in it.".to_string(),
            json!({"weighed": false}),
        ));
    }
    if super::exchange::told_this_hour(world, &ctx.actor, tapster) {
        return Err(CommandError::precondition(
            "The tapster is still weighing your last one. One an hour.".to_string(),
        ));
    }
    // Paid by weight (the words), in a coin that is not gold: few XP, a day to halve.
    let weight = (words.len() as f64 * BAR_XP_PER_WORD)
        .clamp(3.0, BAR_XP_MAX)
        .round();
    let curves = super::Curves::with_half_life(foxing_time::Duration::seconds(BAR_HALF_LIFE_S));
    let gist = super::exchange::gist(&text);
    let chunk = super::memory::earn_chunk(
        world,
        &ctx.actor,
        f.window,
        &format!("a story weighed at the Counting Bar: \"{gist}\""),
        weight,
        0.9,
        &curves,
    )?;
    let room = world.store.entity(tapster).and_then(|e| e.parent);
    let _ = world.note(
        Actor::Citizen(ctx.actor.clone()),
        super::exchange::STORY,
        json!({"from": ctx.actor, "creature": tapster.0, "kind": "tapster", "name": "the tapster", "words": words.len(), "earned": chunk.0, "level": 0, "keywords": [], "repeats": 0, "bar": true}),
    );
    let _ = world.note(
        Actor::Citizen(ctx.actor.clone()),
        WEIGHED,
        json!({"from": ctx.actor, "words": words.len(), "money_words": money, "weight_xp": weight, "half_life_ms": BAR_HALF_LIFE_S * 1000, "chunk": chunk.0, "gist": gist, "room": room.map(|r| r.0)}),
    );
    super::combat::narrate_others(
        world,
        f.citizen,
        &format!(
            "{} tells the tapster a story about money; the tapster chalks a figure on the slate.",
            ctx.actor
        ),
        &ctx.actor,
    );
    Ok(Outcome::new(
        format!(
            "You tell the tapster a story of {} words, {money} of them about money. It weighs it, and chalks the figure: {weight:.0} ounces. You remember the telling: chunk-{} worth {weight:.0} XP, which halves in a day instead of hours. Worth its weight; just not in gold.",
            words.len(),
            chunk.0
        ),
        json!({"weighed": true, "words": words.len(), "money_words": money, "xp": weight, "earned": chunk.0, "half_life_ms": BAR_HALF_LIFE_S * 1000}),
    ))
}

/// What the Purse holds now: a share of all memory held in the world, floored; nothing
/// for an hour after it was taken.
pub fn purse_holds(world: &World) -> (f64, Option<i64>) {
    let taken_at = world
        .recent_events(8000)
        .iter()
        .rev()
        .find(|e| e.kind == PURSE_TAKEN)
        .map(|e| e.at.0);
    if let Some(at) = taken_at {
        if world.now().0 - at < PURSE_REFILL_MS {
            return (0.0, Some(at + PURSE_REFILL_MS));
        }
    }
    let dash = super::economy::dashboard(world);
    let held = dash["held"].as_f64().unwrap_or(0.0);
    ((held * PURSE_SHARE).max(PURSE_FLOOR).round(), None)
}

fn wardens_standing(world: &World, room: EntityId) -> usize {
    super::combat::creatures_in(world, room)
        .iter()
        .filter(|(_, kind, _, _)| kind == "warden")
        .count()
}

/// `take purse` in the Purse Room: past the Wardens, it is yours, capped on your level.
pub fn take_purse(
    world: &mut World,
    ctx: &foxing_registry::CallContext,
    room: EntityId,
    purse: EntityId,
) -> Result<Outcome, CommandError> {
    let f = me(world, ctx)?;
    let standing = wardens_standing(world, room);
    if standing > 0 {
        return Err(CommandError::precondition(format!(
            "{} Warden{} stand{} over the Purse. It is taken past them or not at all.",
            if standing == 1 { "A" } else { "Two" },
            if standing == 1 { "" } else { "s" },
            if standing == 1 { "s" } else { "" }
        )));
    }
    let (holds, refills) = purse_holds(world);
    if holds <= 0.0 {
        return Ok(Outcome::new(
            format!(
                "The Purse is empty; it was taken within the hour. It fills again in {}.",
                super::combat::human(refills.map(|t| t - world.now().0).unwrap_or(0).max(0))
            ),
            json!({"taken": false, "refills_at": refills}),
        ));
    }
    let level = super::memory::level(world, f.window);
    let worth = holds.min(level + PURSE_OVER_LEVEL).round();
    let curves = super::Curves::with_half_life(foxing_time::Duration::seconds(6 * 3600));
    let chunk = super::memory::earn_chunk(
        world,
        &ctx.actor,
        f.window,
        "the Purse of the Exergue",
        worth,
        0.8,
        &curves,
    )?;
    let _ = world.note(
        Actor::Citizen(ctx.actor.clone()),
        PURSE_TAKEN,
        json!({"by": ctx.actor, "worth": worth, "held": holds, "purse": purse.0, "room": room.0}),
    );
    super::combat::narrate_others(
        world,
        f.citizen,
        &format!("{} unties the Purse and takes what it holds.", ctx.actor),
        &ctx.actor,
    );
    world.notice(format!(
        "**{}** took the Purse of the Exergue: {worth:.0} XP of its {holds:.0}.",
        ctx.actor
    ));
    Ok(Outcome::new(
        format!(
            "You untie the cord. The Purse holds {holds:.0}; what you can carry of it is {worth:.0}, and it is yours: chunk-{} worth {worth:.0} XP, slow to fade. The sack is empty on its plinth; it fills again in an hour, and the Wardens come back before that.{}",
            chunk.0,
            if worth < holds { " (A kill or a purse pays at most your level plus two hundred.)" } else { "" }
        ),
        json!({"taken": true, "worth": worth, "held": holds, "chunk": chunk.0}),
    ))
}

/// The coinage in circulation: every chunk in the world, grouped by what struck it.
fn circulation(world: &World) -> Vec<(String, usize, f64)> {
    let now = world.now();
    let mut by: std::collections::BTreeMap<String, (usize, f64)> =
        std::collections::BTreeMap::new();
    for e in world.store.entities().filter(|e| e.kind == "chunk") {
        let source = plain_str(world, e.id, CHUNK, "source").unwrap_or("");
        let coin = coin_of(source);
        let v = world
            .store
            .read_field(e.id, CHUNK, "value", now)
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        if v < 0.05 {
            continue;
        }
        let row = by.entry(coin.to_string()).or_insert((0, 0.0));
        row.0 += 1;
        row.1 += v;
    }
    let mut rows: Vec<(String, usize, f64)> = by.into_iter().map(|(k, (n, v))| (k, n, v)).collect();
    rows.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    rows
}

/// The coin a chunk's source was struck as.
fn coin_of(source: &str) -> &'static str {
    let s = source.to_lowercase();
    if s.contains("counting bar") {
        "the weighted coin (stories at the Counting Bar)"
    } else if s.starts_with("the purse") {
        "the sovereign (the Purse)"
    } else if s.starts_with("a story") || s.starts_with("telling ") || s.contains("story") {
        "the coin of the story"
    } else if s.starts_with("carrying ") {
        "the coin of the road (words carried)"
    } else if s.starts_with("a petition") {
        "the coin of the house (petitions)"
    } else if s.starts_with("felling ") {
        "the coin of the fall (what a creature took)"
    } else if s.starts_with("a sheaf") {
        "the sheaf (stories folded)"
    } else if super::combat::CREATURES
        .iter()
        .any(|c| s == c.key || s.starts_with(c.key))
    {
        "the coin of the fight"
    } else {
        "small change (the rest)"
    }
}

/// `read` in a room of the district: what each one has to read.
pub fn read(world: &World, room: EntityId, arg: &str) -> Option<Outcome> {
    let key = plain_str(world, room, ROOM, "key")?;
    let a = arg.trim().trim_matches('"').to_lowercase();
    let named = |words: &[&str]| a.is_empty() || words.iter().any(|w| a.contains(w));
    let text = match key {
        "exergue-stair" if named(&["step", "date", "stair"]) => {
            let strikes = super::economy::strikes_ever(world);
            let sittings = super::conclave::to_json(world)["sittings"]
                .as_f64()
                .unwrap_or(0.0);
            format!(
                "The dates on the steps, read going down: it is {} UTC; the Clocktower has struck {strikes:.0} times and the Conclave has sat {sittings:.0}; the world opened on the twenty-seventh of October. The last step is blank, for tonight.",
                super::system::clock(world.now().0)
            )
        }
        "counting-bar" if named(&["slate", "bar", "chalk", "figure"]) => {
            let day_ago = world.now().0 - 86_400_000;
            let rows: Vec<String> = world
                .recent_events(6000)
                .iter()
                .filter(|e| e.kind == WEIGHED && e.at.0 >= day_ago)
                .rev()
                .take(8)
                .map(|e| {
                    format!(
                        "{} ({} words): {} oz",
                        e.payload["from"].as_str().unwrap_or("?"),
                        e.payload["words"],
                        e.payload["weight_xp"].as_f64().unwrap_or(0.0)
                    )
                })
                .collect();
            format!(
                "The slate behind the bar, chalked in columns: {}. The tapster weighs a story about money (`story tapster \"…\"`, a dozen words or more, one an hour) and pays its weight in a coin that halves in a day, not in hours. Worth its weight; just not in gold.",
                if rows.is_empty() { "nothing weighed today".to_string() } else { rows.join("; ") }
            )
        }
        "die-room" if named(&["die", "dies", "rack", "coin"]) => {
            let rows = circulation(world);
            let total: f64 = rows.iter().map(|r| r.2).sum();
            format!(
                "The dies in the rack, read in the mirror, one for every coin in circulation tonight ({total:.0} XP in all): {}.",
                rows.iter().map(|(coin, n, v)| format!("{coin}: {n} struck, {v:.0} XP")).collect::<Vec<_>>().join("; ")
            )
        }
        "striking-floor" if named(&["plate", "press", "rate"]) => {
            format!(
                "The plate under the middle press, struck this hour: {}. The presses stamp what the Clocktower sets; the Clocktower strikes on the hour.",
                super::economy::rates_line(&super::economy::rates(world))
            )
        }
        "assay-office" if named(&["balance", "scale", "touchstone", "sample"]) => {
            // The balance weighs the reader, not the room: the reader is the one who asked.
            "The balance under its bell weighs whoever stands at it: `chunks` is the reading, each memory at its worth tonight; the Assayer keeps a sample of what it weighs, which is to say the ray. The touchstones are for telling gold from the rest, and they have had little to do.".to_string()
        }
        "mintmark-hall" if named(&["register", "frame", "mark", "record"]) => {
            let mut rows: Vec<String> = Vec::new();
            for e in world.store.entities().filter(|e| e.kind == "creature") {
                if let Some(n) = super::mind::name(world, e.id) {
                    let kind = plain_str(world, e.id, CREATURE, "name").unwrap_or("?");
                    let room_name = e
                        .parent
                        .and_then(|r| plain_str(world, r, ROOM, "name"))
                        .unwrap_or("nowhere");
                    rows.push(format!("{n} the {kind}, {room_name}"));
                }
            }
            let mut fixtures: Vec<String> = Vec::new();
            for e in world.store.entities().filter(|e| e.kind == "room") {
                for v in super::craft::fixtures_in(world, e.id) {
                    if let Some(n) = v["given_name"].as_str() {
                        fixtures.push(format!(
                            "{n} ({})",
                            plain_str(world, e.id, ROOM, "name").unwrap_or("?")
                        ));
                    }
                }
            }
            format!(
                "The register in its iron frames, every kept thing in the world under its mark. The named ({}): {}. The fixtures ({}): {}.",
                rows.len(),
                if rows.is_empty() { "none".to_string() } else { rows.join("; ") },
                fixtures.len(),
                if fixtures.is_empty() { "none".to_string() } else { fixtures.join("; ") }
            )
        }
        "purse-room" if named(&["purse", "plinth", "ledger", "sack"]) => {
            let (holds, refills) = purse_holds(world);
            let standing = wardens_standing(world, room);
            let last = world
                .recent_events(8000)
                .iter()
                .rev()
                .find(|e| e.kind == PURSE_TAKEN)
                .map(|e| {
                    format!(
                        " Last taken by {}, {} ago, for {:.0}.",
                        e.payload["by"].as_str().unwrap_or("?"),
                        super::combat::human((world.now().0 - e.at.0).max(0)),
                        e.payload["worth"].as_f64().unwrap_or(0.0)
                    )
                })
                .unwrap_or_else(|| " It has never been taken.".to_string());
            format!(
                "The Purse on its plinth{}: it holds {}. {} Warden{} stand{} over it; `take purse` past them, and what you can carry of it is yours (your level plus two hundred at most).{last}",
                if holds <= 0.0 { ", slack" } else { "" },
                if holds <= 0.0 { format!("nothing; it fills again in {}", super::combat::human(refills.map(|t| t - world.now().0).unwrap_or(0).max(0))) } else { format!("{holds:.0} XP, a twentieth of all the world remembers") },
                match standing { 0 => "No".to_string(), 1 => "One".to_string(), n => n.to_string() },
                if standing == 1 { "" } else { "s" },
                if standing == 1 { "s" } else { "" }
            )
        }
        _ => return None,
    };
    Some(Outcome::new(text, json!({"room": room.0, "key": key})))
}
