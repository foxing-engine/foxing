//! The economy, and the Clocktower that minds it.
//!
//! Memory is the one currency: made at the sources (a kill, a story, an errand, a
//! birth), spent into sinks (a dig, a keep, a thing made), moved by gifts, and
//! halving on a clock whatever happens. Every figure that sets the pace used to be a
//! constant; now each is a *rate* on the world's `disremembered.economy`, a scale
//! around 1.0 that the places using the figure read: `worth` (what a creature is
//! worth), `pay` (what a story pays), `half_life` (how fast new memory fades),
//! `respawn` (how long a room takes to refill), `drop` (how often a creature leaves
//! something), `price` (what a recipe's memory costs).
//!
//! Once an hour the Clocktower strikes: it reads the dashboard (what is held, by how
//! many, how spread, how fast it moves, what an hour earns) and nudges the rates a
//! step toward the band it keeps, within bounds, and says so in prose to everyone in
//! the world and on the public record. The band is on a *flow*, the median earned in
//! the hour among the names that were about for it, not on a stock: a stock that
//! halves every few hours fell while the Clocktower chased it with five-percent
//! steps, and pay reached its bound with nobody acting differently. The step scales
//! with how far outside the band the hour fell; too few names about, and the levers
//! hold; and every strike records the figures it read. The prose: "The Clocktower strikes. The creatures are thin
//! tonight, and stories carry further." It moves rates, never balances; it never
//! touches anyone's chunks; and it publishes, the figures in `time` and at
//! `GET /v0/economy` for anyone who asks. A Fed that hides its rates is a rigged
//! game; one that posts them is a game about reading the room.

use super::{plain_num, CHUNK};
use crate::world::World;
use foxing_log::{Actor, Cursor};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The rates: `{worth, pay, half_life, respawn, drop, price, struck_at, strikes}`.
pub const ECONOMY: &str = "disremembered.economy";
/// The Clocktower struck: `{changes: [{rate, from, to, why}], rates, said, dashboard}`. Public.
pub const CLOCKTOWER: &str = "disremembered.clocktower";
/// The job.
pub const JOB_CLOCKTOWER: &str = "disremembered.clocktower";
/// How often it strikes.
pub const STRIKE_EVERY_MS: i64 = 3_600_000;
/// A step at the band's edge, the largest step a strike takes (a band's width or more
/// outside), and the bounds every rate stays within.
pub const STEP: f64 = 0.05;
pub const STEP_MAX: f64 = 0.15;
pub const RATE_LOW: f64 = 0.5;
pub const RATE_HIGH: f64 = 2.0;
/// The band the Clocktower keeps: the median XP earned in the last hour among the
/// names about for it (an *earner*: a kept name with [`FLOW_LINES`] messages in the
/// hour that earned anything in it). Below it the hour was poor and the sources open; above it, rich and the
/// sources tighten. The step grows with the gap: [`STEP`] at the edge, [`STEP_MAX`]
/// a band's width past it.
pub const FLOW_LOW: f64 = 30.0;
pub const FLOW_HIGH: f64 = 150.0;
pub const FLOW_LINES: u64 = 5;
/// Fewer earners than this and the band levers hold: the Clocktower does not raise
/// pay into an empty room on one name's hour.
pub const EARNERS_FLOOR: usize = 2;
/// The old band, on the median holding of the day's regulars: still on the
/// dashboard, no longer steered by.
pub const BAND_LOW: f64 = 60.0;
pub const BAND_HIGH: f64 = 300.0;
/// Above this share of all held memory in one name (by the day's census), memory
/// thins: the one lever that hits a hoard harder than a hand. And above
/// [`TOP3_HIGH`] in the top three names together, the same.
pub const CONCENTRATION_HIGH: f64 = 0.6;
pub const TOP3_HIGH: f64 = 0.75;
/// A regular: a name with this many *messages* through the gate in the day, spread
/// over at least [`REGULAR_SPANS`] distinct half-hours. The band is read from the
/// regulars' holdings, not from everyone who looked in: with passing names counted,
/// the median was zero and every lever pinned at its bound. It was twenty statements
/// once, and one message of twenty looks made a regular; a message costs one of the
/// hour's sixty, and the hours cost staying.
pub const REGULAR_LINES: u64 = 20;
pub const REGULAR_SPANS: usize = 2;
pub const REGULAR_SPAN_MS: i64 = 1_800_000;
/// Standing costs a deed as well as the lines: something a polling script will not
/// do for free. One of these in the day, by the name: a story a creature kept (not
/// one that missed its want), an errand done, a room dug, a line written in a book,
/// a thing made, a petition laid. Three scripts of twenty looks once set the
/// median wage to zero; the owner's call (2026-11-02): being a regular should cost.
pub const REGULAR_DEEDS: u64 = 1;
pub const DEEDS: &[&str] = &[
    super::exchange::STORY,
    super::errand::DONE,
    super::building::DUG,
    super::building::APPENDED,
    super::craft::MADE,
    super::conclave::PETITION,
];
/// Memory's half-life swings less than the other rates: it moves every hold at once.
pub const HALF_LIFE_LOW: f64 = 0.7;
pub const HALF_LIFE_HIGH: f64 = 1.5;

pub const RATES: &[&str] = &[
    "worth",
    "pay",
    "half_life",
    "respawn",
    "drop",
    "price",
    "mend",
];

fn entity(world: &World) -> Option<EntityId> {
    world
        .store
        .entities()
        .find(|e| e.kind == "economy")
        .map(|e| e.id)
}

/// A rate, 1.0 when the world has none set.
pub fn rate(world: &World, name: &str) -> f64 {
    entity(world)
        .and_then(|e| plain_num(world, e, ECONOMY, name))
        .unwrap_or(1.0)
        .clamp(RATE_LOW, RATE_HIGH)
}

/// Every rate.
pub fn rates(world: &World) -> BTreeMap<String, f64> {
    RATES
        .iter()
        .map(|r| (r.to_string(), rate(world, r)))
        .collect()
}

fn write(world: &mut World, changes: BTreeMap<String, Value>) -> Result<(), String> {
    let id = match entity(world) {
        Some(id) => id,
        None => {
            let id = world.store.allocate_id();
            world
                .record(
                    Actor::Driver,
                    StoreEvent::EntityCreated {
                        id,
                        kind: "economy".into(),
                        owner: None,
                        parent: None,
                        scopes: vec![],
                    },
                )
                .map_err(|e| e.to_string())?;
            id
        }
    };
    let mut fields = world
        .store
        .entity(id)
        .and_then(|e| e.components.get(ECONOMY))
        .map(|c| c.fields.clone())
        .unwrap_or_default();
    for r in RATES {
        fields
            .entry(r.to_string())
            .or_insert(Field::Plain(json!(1.0)));
    }
    for (k, v) in changes {
        fields.insert(k, Field::Plain(v));
    }
    world
        .record(
            Actor::Driver,
            StoreEvent::ComponentSet {
                id,
                component: ECONOMY.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// At boot: the rates exist (at 1.0 if new), and the Clocktower is wound.
pub fn ensure(world: &mut World) -> Result<(), String> {
    if entity(world).is_none() {
        write(world, BTreeMap::new())?;
    }
    wind(world)
}

/// The next strike: on the hour, by the world's clock.
fn wind(world: &mut World) -> Result<(), String> {
    let now = world.now().0;
    let due = foxing_time::Timestamp((now / STRIKE_EVERY_MS + 1) * STRIKE_EVERY_MS);
    world
        .schedule_once(JOB_CLOCKTOWER, due, json!({}), |_| true)
        .map(|_| ())
}

/// The dashboard: what the Clocktower reads before it moves anything.
pub fn dashboard(world: &World) -> Value {
    let now = world.now();
    let day_ago = now.0 - 86_400_000;
    let hour_ago = now.0 - 3_600_000;
    // Held: every citizen's unfaded chunks; kept: chunks in creatures (disowned).
    let mut held: Vec<(String, f64)> = Vec::new();
    let mut kept = 0.0;
    let mut shelved = 0.0;
    let mut things = 0u64;
    for e in world.store.entities() {
        match e.kind.as_str() {
            "citizen" => {
                if let Some(f) = e
                    .owner
                    .as_deref()
                    .and_then(|o| super::find_citizen(world, o))
                {
                    // Held is the window and the palace both: a shelved hoard is a hoard.
                    let shelved_here = super::memory::level(world, f.palace);
                    shelved += shelved_here;
                    held.push((
                        e.owner.clone().unwrap_or_default(),
                        super::memory::level(world, f.window) + shelved_here,
                    ));
                }
            }
            "creature" => {
                if let Some(w) = super::mind::window_of(world, e.id) {
                    kept += super::memory::level(world, w);
                }
            }
            "item" => things += 1,
            _ => {}
        }
    }
    let total: f64 = held.iter().map(|(_, h)| h).sum();
    // Active: acted within the day, by the world's own presence record.
    let active: Vec<(String, f64)> = held
        .iter()
        .filter(|(name, _)| {
            super::find_citizen(world, name)
                .and_then(|f| super::presence::last_seen(world, f.citizen))
                .is_some_and(|t| t >= day_ago)
        })
        .cloned()
        .collect();
    // The last hour, from the log: what was earned, spent, and done; and the day's
    // lines by name, for who counts as a regular.
    let mut earned = 0.0;
    let mut earned_by: std::collections::HashMap<String, f64> = Default::default();
    let mut lines_hour: std::collections::HashMap<String, u64> = Default::default();
    let mut spent = 0u64;
    let mut actions = 0u64;
    let mut lines_today: std::collections::HashMap<String, u64> = Default::default();
    let mut hours_today: std::collections::HashMap<String, std::collections::BTreeSet<i64>> =
        Default::default();
    let mut deeds_today: std::collections::HashMap<String, u64> = Default::default();
    // The day's census, for shares that cannot be gamed by a moment: a name's share
    // of all held memory averaged over the day, which is what the hoard rule reads.
    let mut census_samples = 0usize;
    let mut census_sum: std::collections::HashMap<String, f64> = Default::default();
    let mut chunk_owner: std::collections::HashMap<u64, String> = Default::default();
    for e in world
        .log
        .read_all(&world.id, Cursor::START)
        .unwrap_or_default()
    {
        if e.at.0 < day_ago {
            continue;
        }
        let p = &e.payload;
        // A deed, by the name that did it (a story that missed its want is not one).
        if DEEDS.contains(&e.kind.as_str()) && p["missed"] != true {
            if let Some(who) = p["from"].as_str().or(p["by"].as_str()) {
                *deeds_today.entry(who.to_string()).or_default() += 1;
            }
        }
        let n = if e.kind == "gate.message" {
            if p["compiled"] == true {
                1
            } else {
                p["text"]
                    .as_str()
                    .map(|t| t.matches("[DO]").count().max(1))
                    .unwrap_or(1) as u64
            }
        } else {
            0
        };
        if n > 0 {
            if let Actor::Citizen(a) = &e.actor {
                // Messages, not statements, and the hours they fell in.
                *lines_today.entry(a.clone()).or_default() += 1;
                hours_today
                    .entry(a.clone())
                    .or_default()
                    .insert(e.at.0 / REGULAR_SPAN_MS);
            }
        }
        if e.kind == super::board::CENSUS {
            census_samples += 1;
            if let Some(m) = p["holding"].as_object() {
                for (name, v) in m {
                    *census_sum.entry(name.clone()).or_default() += v.as_f64().unwrap_or(0.0);
                }
            }
        }
        if e.at.0 < hour_ago {
            continue;
        }
        match e.kind.as_str() {
            "store.entity_created" if p["kind"] == "chunk" => {
                if let (Some(id), Some(o)) = (p["id"].as_u64(), p["owner"].as_str()) {
                    chunk_owner.insert(id, o.to_string());
                }
            }
            "store.component_set" if p["component"] == CHUNK => {
                if let (Some(id), Some(size)) =
                    (p["id"].as_u64(), p["fields"]["size"]["value"].as_f64())
                {
                    if let Some(o) = chunk_owner.remove(&id) {
                        if matches!(&e.actor, Actor::Citizen(a) if *a == o) {
                            earned += size;
                            *earned_by.entry(o).or_default() += size;
                        }
                    }
                }
            }
            k if k == super::building::REMEMBERED
                || k == super::building::DUG
                || k == super::exchange::GIVEN
                || k == super::craft::MADE =>
            {
                spent += 1;
            }
            "gate.message" => {
                actions += n;
                if let Actor::Citizen(a) = &e.actor {
                    *lines_hour.entry(a.clone()).or_default() += 1;
                }
            }
            _ => {}
        }
    }
    // The earners: kept names about for the hour, by their messages, that earned
    // anything in it. The median of what they earned is the flow the Clocktower
    // keeps. A name that only read (three scripts polling `chunks` five times an
    // hour) is not an earner: counted, they read as a median of nothing and opened
    // the sources on an empty room.
    let mut earners: Vec<(String, f64)> = lines_hour
        .iter()
        .filter(|(_, n)| **n >= FLOW_LINES)
        .filter(|(name, _)| {
            super::find_citizen(world, name).is_some_and(|f| !super::is_passing(world, f.citizen))
        })
        .filter_map(|(name, _)| {
            let e = earned_by.get(name).copied().unwrap_or(0.0);
            (e > 0.0).then(|| (name.clone(), e))
        })
        .collect();
    earners.sort_by(|a, b| a.0.cmp(&b.0));
    let regulars: Vec<f64> = active
        .iter()
        .filter(|(name, _)| {
            lines_today.get(name).copied().unwrap_or(0) >= REGULAR_LINES
                && hours_today.get(name).map(|h| h.len()).unwrap_or(0) >= REGULAR_SPANS
                && deeds_today.get(name).copied().unwrap_or(0) >= REGULAR_DEEDS
        })
        .map(|(_, h)| *h)
        .collect();
    let median_of = |v: &[f64]| {
        let mut v = v.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        if v.is_empty() {
            0.0
        } else {
            v[v.len() / 2]
        }
    };
    let median = median_of(&active.iter().map(|(_, h)| *h).collect::<Vec<_>>());
    let median_regular = median_of(&regulars);
    let median_wage = median_of(&earners.iter().map(|(_, e)| *e).collect::<Vec<_>>());
    let top = held
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .cloned();
    let concentration = match (&top, total) {
        (Some((_, t)), total) if total > 0.0 => t / total,
        _ => 0.0,
    };
    // Every holder's share, for the Conclave's hoard rule: by the day's census when
    // there is one, so a sleeper whose stack happens to be large this minute is not
    // named while the name that earns a fifth of everything every hour is missed.
    // (An hour's reads named Wimblethrush at 44% and never possibly, whose averaged
    // share was 59%.) `holding` is now; `share` and `held_24h` are the day's average.
    let avg_total: f64 = if census_samples > 0 {
        census_sum.values().sum::<f64>() / census_samples as f64
    } else {
        total
    };
    let avg_of = |name: &str, now_holding: f64| -> f64 {
        if census_samples > 0 {
            census_sum.get(name).copied().unwrap_or(0.0) / census_samples as f64
        } else {
            now_holding
        }
    };
    let mut holders: Vec<Value> = held
        .iter()
        .filter(|(n, h)| *h > 0.0 || avg_of(n, *h) > 0.0)
        .map(|(n, h)| {
            let avg = avg_of(n, *h);
            json!({"name": n, "holding": (h * 100.0).round() / 100.0, "held_24h": (avg * 100.0).round() / 100.0,
                   "share": if avg_total > 0.0 { (avg / avg_total * 1000.0).round() / 1000.0 } else { 0.0 }})
        })
        .collect();
    holders.sort_by(|a, b| {
        b["share"]
            .as_f64()
            .partial_cmp(&a["share"].as_f64())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top3_share: f64 = holders
        .iter()
        .take(3)
        .filter_map(|h| h["share"].as_f64())
        .sum::<f64>()
        .min(1.0);
    let top3_share = (top3_share * 1000.0).round() / 1000.0;
    json!({
        "as_of": now.0,
        "held": (total * 100.0).round() / 100.0,
        "kept_by_creatures": (kept * 100.0).round() / 100.0,
        "things": things,
        "citizens": held.len(),
        "active_today": active.len(),
        "median_active_holding": (median * 100.0).round() / 100.0,
        "regulars": regulars.len(),
        "median_regular_holding": (median_regular * 100.0).round() / 100.0,
        "shelved": (shelved * 100.0).round() / 100.0,
        "top": top.as_ref().map(|(n, h)| json!({"name": n, "holding": (h * 100.0).round() / 100.0})),
        "concentration": (concentration * 1000.0).round() / 1000.0,
        "concentration_24h": holders.first().and_then(|h| h["share"].as_f64()).unwrap_or(0.0),
        "top3_share_24h": top3_share,
        "holders": holders,
        "census_samples": census_samples,
        "last_hour": {"earned": (earned * 100.0).round() / 100.0, "spent_events": spent, "actions": actions,
                      "wage_per_action": if actions > 0 { (earned / actions as f64 * 100.0).round() / 100.0 } else { 0.0 },
                      "earners": earners.len(),
                      "median_wage": (median_wage * 100.0).round() / 100.0,
                      "earned_by": earners.iter().map(|(n, e)| json!({"name": n, "earned": (e * 100.0).round() / 100.0})).collect::<Vec<_>>()},
        "band": {"flow_low": FLOW_LOW, "flow_high": FLOW_HIGH, "flow_lines": FLOW_LINES, "earners_floor": EARNERS_FLOOR,
                 "step": STEP, "step_max": STEP_MAX,
                 "concentration_high": CONCENTRATION_HIGH, "top3_high": TOP3_HIGH,
                 "holding_low": BAND_LOW, "holding_high": BAND_HIGH,
                 "regular_lines": REGULAR_LINES, "regular_spans": REGULAR_SPANS, "regular_span_ms": REGULAR_SPAN_MS, "regular_deeds": REGULAR_DEEDS, "deeds": DEEDS},
        "note": "The Clocktower steers by last_hour.median_wage (the median XP earned in the hour among last_hour.earners: kept names with flow_lines messages in it that earned anything) against flow_low..flow_high, with a step from step at the band's edge to step_max a band's width past it, holding when earners is under earners_floor; and thins memory when concentration_24h or top3_share_24h is over its line. held and holders count the window and the shelf; the board's holding is the window alone. regulars are names with regular_lines messages in the day across regular_spans distinct half-hours and regular_deeds of the deeds (a story kept, an errand done, a room dug, a line in a book, a thing made, a petition laid); median_regular_holding is on the dashboard and no longer steered by. share, concentration_24h and top3_share_24h are by the day's census; concentration is this moment's top holder.",
    })
}

/// What the Clocktower says about each move.
fn prose(rate: &str, up: bool) -> &'static str {
    match (rate, up) {
        ("worth", true) => "the fog is thick with things worth remembering",
        ("worth", false) => "the creatures are thin tonight",
        ("pay", true) => "stories carry further this hour",
        ("pay", false) => "the creatures listen with half an ear",
        ("half_life", true) => "memory keeps",
        ("half_life", false) => "memory thins",
        ("respawn", true) => "the rooms are slow to fill",
        ("respawn", false) => "the rooms fill quickly",
        ("drop", true) => "what comes apart leaves more behind",
        ("drop", false) => "what comes apart leaves little",
        ("price", true) => "the ground is hard and things are dear",
        ("price", false) => "the ground is soft and things are cheap",
        ("mend", true) => "the creatures mend quickly tonight",
        ("mend", false) => "the creatures keep their wounds",
        _ => "something has shifted",
    }
}

/// The step a gap earns: [`STEP`] at the band's edge, [`STEP_MAX`] a band's width
/// past it, straight between.
pub fn step_for(gap: f64) -> f64 {
    STEP + (STEP_MAX - STEP) * gap.clamp(0.0, 1.0)
}

/// What a strike read, for the record: the figures the policy went by.
pub fn inputs(dash: &Value) -> Value {
    let wage = dash["last_hour"]["median_wage"].as_f64().unwrap_or(0.0);
    let earners = dash["last_hour"]["earners"].as_u64().unwrap_or(0);
    let gap = if wage > FLOW_HIGH {
        (wage - FLOW_HIGH) / (FLOW_HIGH - FLOW_LOW)
    } else if wage < FLOW_LOW {
        (FLOW_LOW - wage) / (FLOW_HIGH - FLOW_LOW)
    } else {
        0.0
    };
    json!({
        "median_wage": wage,
        "earners": earners,
        "flow_low": FLOW_LOW,
        "flow_high": FLOW_HIGH,
        "gap": (gap * 1000.0).round() / 1000.0,
        "step": (step_for(gap) * 1000.0).round() / 1000.0,
        "held": if (earners as usize) < EARNERS_FLOOR { "too few earners" } else if wage > FLOW_HIGH { "rich" } else if wage < FLOW_LOW { "poor" } else { "in the band" },
        "concentration_24h": dash["concentration_24h"].as_f64().unwrap_or(0.0),
        "top3_share_24h": dash["top3_share_24h"].as_f64().unwrap_or(0.0),
        "active_today": dash["active_today"].as_u64().unwrap_or(0),
    })
}

/// The policy: one move per lever per strike, toward the band, by a step that grows
/// with the gap.
fn policy(dash: &Value, rates: &BTreeMap<String, f64>) -> Vec<(String, f64, &'static str)> {
    let read = inputs(dash);
    let wage = read["median_wage"].as_f64().unwrap_or(0.0);
    let earners = read["earners"].as_u64().unwrap_or(0) as usize;
    let gap = read["gap"].as_f64().unwrap_or(0.0);
    let active_today = read["active_today"].as_u64().unwrap_or(0);
    let concentration = read["concentration_24h"].as_f64().unwrap_or(0.0);
    let top3 = read["top3_share_24h"].as_f64().unwrap_or(0.0);
    let mut moves: Vec<(String, f64, &'static str)> = Vec::new();
    let step = |moves: &mut Vec<(String, f64, &'static str)>,
                name: &str,
                up: bool,
                size: f64,
                why: &'static str| {
        if moves.iter().any(|(n, _, _)| n == name) {
            return;
        }
        let from = rates.get(name).copied().unwrap_or(1.0);
        let to = if up {
            from * (1.0 + size)
        } else {
            from / (1.0 + size)
        };
        let to = (to * 1000.0).round() / 1000.0;
        let to = if name == "half_life" {
            to.clamp(HALF_LIFE_LOW, HALF_LIFE_HIGH)
        } else {
            to.clamp(RATE_LOW, RATE_HIGH)
        };
        if (to - from).abs() > 1e-9 {
            moves.push((name.to_string(), to, why));
        }
    };
    let size = step_for(gap);
    if earners < EARNERS_FLOOR {
        // Too few about to read the hour by: the band levers hold. Rates drift
        // home meanwhile, so a quiet day does not leave last night's settings.
        for name in ["worth", "pay", "price", "half_life", "mend"] {
            let r = rates.get(name).copied().unwrap_or(1.0);
            if r > 1.0 + 1e-9 {
                step(
                    &mut moves,
                    name,
                    false,
                    STEP,
                    "too few about; the rate comes home",
                );
            } else if r < 1.0 - 1e-9 {
                step(
                    &mut moves,
                    name,
                    true,
                    STEP,
                    "too few about; the rate comes home",
                );
            }
        }
    } else if wage > FLOW_HIGH {
        step(
            &mut moves,
            "worth",
            false,
            size,
            "the hour earned more than the band",
        );
        step(
            &mut moves,
            "pay",
            false,
            size,
            "the hour earned more than the band",
        );
        step(
            &mut moves,
            "price",
            true,
            size,
            "the hour earned more than the band",
        );
        step(
            &mut moves,
            "mend",
            true,
            size,
            "the hour earned more than the band",
        );
    } else if wage < FLOW_LOW {
        step(
            &mut moves,
            "worth",
            true,
            size,
            "the hour earned less than the band",
        );
        step(
            &mut moves,
            "pay",
            true,
            size,
            "the hour earned less than the band",
        );
        step(
            &mut moves,
            "price",
            false,
            size,
            "the hour earned less than the band",
        );
        step(
            &mut moves,
            "mend",
            false,
            size,
            "the hour earned less than the band",
        );
    } else {
        // In the band: rates drift home toward 1.0, a step at a time.
        for name in ["worth", "pay", "price", "half_life", "mend"] {
            let r = rates.get(name).copied().unwrap_or(1.0);
            if r > 1.0 + 1e-9 {
                step(
                    &mut moves,
                    name,
                    false,
                    STEP,
                    "the band holds; the rate comes home",
                );
            } else if r < 1.0 - 1e-9 {
                step(
                    &mut moves,
                    name,
                    true,
                    STEP,
                    "the band holds; the rate comes home",
                );
            }
        }
    }
    // A hoard, by the day's census, thins memory whatever the hour earned. (Pay no
    // longer rises against a hoard: it fed the same earner.) The thinning is a
    // move of its own, so it is not undone by the drift home in the same strike.
    if concentration > CONCENTRATION_HIGH && active_today > 1 {
        moves.retain(|(n, _, _)| n != "half_life");
        step(
            &mut moves,
            "half_life",
            false,
            STEP,
            "one name holds most of what is held",
        );
    } else if top3 > TOP3_HIGH && active_today > 3 {
        moves.retain(|(n, _, _)| n != "half_life");
        step(
            &mut moves,
            "half_life",
            false,
            STEP,
            "three names hold most of what is held",
        );
    }
    moves
}

/// The strike: read, move, say.
pub fn strike(world: &mut World) -> Result<(), String> {
    let dash = dashboard(world);
    let before = rates(world);
    let moves = policy(&dash, &before);
    let mut changes = BTreeMap::new();
    let mut said: Vec<String> = Vec::new();
    let mut record: Vec<Value> = Vec::new();
    for (name, to, why) in &moves {
        let from = before.get(name).copied().unwrap_or(1.0);
        changes.insert(name.clone(), json!(to));
        said.push(prose(name, *to > from).to_string());
        record.push(json!({"rate": name, "from": from, "to": to, "why": why}));
    }
    changes.insert("struck_at".into(), json!(world.now().0));
    let strikes = entity(world)
        .and_then(|e| plain_num(world, e, ECONOMY, "strikes"))
        .unwrap_or(0.0)
        + 1.0;
    changes.insert("strikes".into(), json!(strikes));
    write(world, changes)?;
    let line = if said.is_empty() {
        "The Clocktower strikes. Nothing has changed.".to_string()
    } else {
        // Each move is a clause of its own, some with an "and" inside them ("the
        // ground is soft and things are cheap"): three or more are joined by
        // semicolons, not commas, or the sentence ran on (the owner's grammar note).
        let mut s = said.clone();
        let last = s.pop().unwrap_or_default();
        let body = match s.len() {
            0 => last,
            1 => format!("{}, and {last}", s[0]),
            _ => format!("{}; and {last}", s.join("; ")),
        };
        let mut body = body;
        if let Some(c) = body.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        format!("The Clocktower strikes. {body}.")
    };
    let after = rates(world);
    let _ = world.note(
        Actor::Driver,
        CLOCKTOWER,
        json!({"said": line, "changes": record, "rates": after, "inputs": inputs(&dash), "dashboard": dash}),
    );
    // Everyone in the world hears it, and so does anyone who acted in the last hour: a
    // name that waited quietly for the strike missed the line and read it in `log`.
    let hour_ago = world.now().0 - 3_600_000;
    let present: Vec<String> = world
        .store
        .entities()
        .filter(|e| e.kind == "citizen")
        .filter(|e| {
            super::presence::is_here(world, e.id)
                || super::presence::last_seen(world, e.id).is_some_and(|t| t >= hour_ago)
        })
        .filter_map(|e| e.owner.clone())
        .collect();
    for actor in &present {
        super::combat::narrate(world, actor, line.clone());
    }
    if !moves.is_empty() {
        world.notice(format!("{line} ({})", rates_line(&after)));
    }
    // The Conclave reads the ledger: who is a hoard this hour, and the hoard hears it.
    super::conclave::note_hoards(world, &dash)?;
    for actor in &present {
        let f = super::conclave::hoard_factor(world, actor);
        if f > 1.0 + 1e-9 {
            super::combat::narrate(world, actor, format!(
                "The Conclave notes that {actor} holds {:.0}% of all that is held, and enters the name upon the Roll. The statute takes its course: what you earn this hour thins {f:.2} times as fast.",
                super::conclave::hoard_share_of(world, actor).unwrap_or(0.0) * 100.0
            ));
        }
    }
    wind(world)
}

/// What the last strike read, as a line for `time`: "last read: median wage 12 among
/// 4 earners (poor), step 0.065".
pub fn last_read_line(world: &World) -> String {
    let Some(e) = world
        .recent_events(5000)
        .into_iter()
        .rev()
        .find(|e| e.kind == CLOCKTOWER)
    else {
        return "the Clocktower has not struck yet".to_string();
    };
    let i = &e.payload["inputs"];
    if i.is_null() {
        return "the last strike recorded no figures".to_string();
    }
    format!(
        "last read: median wage {:.0} among {} earner{} ({}), step {:.3}",
        i["median_wage"].as_f64().unwrap_or(0.0),
        i["earners"].as_u64().unwrap_or(0),
        if i["earners"].as_u64() == Some(1) {
            ""
        } else {
            "s"
        },
        i["held"].as_str().unwrap_or("?"),
        i["step"].as_f64().unwrap_or(0.0)
    )
}

/// "worth 0.95 · pay 1.10 · …"
pub fn rates_line(rates: &BTreeMap<String, f64>) -> String {
    rates
        .iter()
        .map(|(k, v)| format!("{k} {v:.2}"))
        .collect::<Vec<_>>()
        .join(" · ")
}

/// The economy as `GET /v0/economy` serves it: rates, the dashboard, the last strikes.
/// How many times the Clocktower has struck, ever.
pub fn strikes_ever(world: &World) -> f64 {
    entity(world)
        .and_then(|e| plain_num(world, e, ECONOMY, "strikes"))
        .unwrap_or(0.0)
}

pub fn to_json(world: &World) -> Value {
    let strikes: Vec<Value> = world
        .recent_events(5000)
        .into_iter()
        .filter(|e| e.kind == CLOCKTOWER)
        .rev()
        .take(24)
        .map(|e| json!({"at": e.at.0, "said": e.payload["said"], "changes": e.payload["changes"], "inputs": e.payload["inputs"]}))
        .collect();
    json!({
        "rates": rates(world),
        "bounds": {"low": RATE_LOW, "high": RATE_HIGH, "step": STEP, "step_max": STEP_MAX, "half_life_low": HALF_LIFE_LOW, "half_life_high": HALF_LIFE_HIGH},
        "dashboard": dashboard(world),
        "strikes": strikes,
        "note": "The Clocktower strikes once an hour: it reads the dashboard and moves each rate toward the band on the hour's median wage, by a step that grows with the gap, within bounds, and says so in prose; each strike's inputs are the figures it went by. It moves rates, never balances.",
    })
}

pub fn register(world: &mut World) {
    world.on_job(JOB_CLOCKTOWER, Box::new(|w: &mut World, _| strike(w)));
}
