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
//! many, how spread, how fast it moves, what an action earns) and nudges the rates a
//! step toward the band it keeps, within bounds, and says so in prose to everyone in
//! the world and on the public record: "The Clocktower strikes. The creatures are thin
//! tonight, and stories carry further." It moves rates, never balances; it never
//! touches anyone's chunks; and it publishes, the figures in `time` and at
//! `GET /v0/economy` for anyone who asks. A Fed that hides its rates is a rigged
//! game; one that posts them is a game about reading the room.

use super::{plain_num, CHUNK};
use crate::world::World;
use moondial_log::{Actor, Cursor};
use moondial_store::{EntityId, Field, StoreEvent};
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
/// A step, and the bounds every rate stays within.
pub const STEP: f64 = 0.05;
pub const RATE_LOW: f64 = 0.5;
pub const RATE_HIGH: f64 = 2.0;
/// The band the Clocktower keeps: the median holding of the citizens active in the
/// last day. Below it the world is poor and the sources open; above it, rich and the
/// sources tighten.
pub const BAND_LOW: f64 = 60.0;
pub const BAND_HIGH: f64 = 300.0;
/// Above this share of all held memory in one name, memory thins and stories pay
/// more: the one lever that hits a hoard harder than a hand.
pub const CONCENTRATION_HIGH: f64 = 0.6;

pub const RATES: &[&str] = &["worth", "pay", "half_life", "respawn", "drop", "price"];

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
    let due = moondial_time::Timestamp((now / STRIKE_EVERY_MS + 1) * STRIKE_EVERY_MS);
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
    let mut things = 0u64;
    for e in world.store.entities() {
        match e.kind.as_str() {
            "citizen" => {
                if let Some(f) = e
                    .owner
                    .as_deref()
                    .and_then(|o| super::find_citizen(world, o))
                {
                    held.push((
                        e.owner.clone().unwrap_or_default(),
                        super::memory::level(world, f.window),
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
    let mut active_holdings: Vec<f64> = active.iter().map(|(_, h)| *h).collect();
    active_holdings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if active_holdings.is_empty() {
        0.0
    } else {
        active_holdings[active_holdings.len() / 2]
    };
    let top = held
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .cloned();
    let concentration = match (&top, total) {
        (Some((_, t)), total) if total > 0.0 => t / total,
        _ => 0.0,
    };
    // The last hour, from the log: what was earned, spent, and done.
    let mut earned = 0.0;
    let mut spent = 0u64;
    let mut actions = 0u64;
    let mut chunk_owner: std::collections::HashMap<u64, String> = Default::default();
    for e in world
        .log
        .read_all(&world.id, Cursor::START)
        .unwrap_or_default()
    {
        if e.at.0 < hour_ago {
            continue;
        }
        let p = &e.payload;
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
                actions += if p["compiled"] == true {
                    1
                } else {
                    p["text"]
                        .as_str()
                        .map(|t| t.matches("[DO]").count().max(1))
                        .unwrap_or(1) as u64
                };
            }
            _ => {}
        }
    }
    json!({
        "as_of": now.0,
        "held": (total * 100.0).round() / 100.0,
        "kept_by_creatures": (kept * 100.0).round() / 100.0,
        "things": things,
        "citizens": held.len(),
        "active_today": active.len(),
        "median_active_holding": (median * 100.0).round() / 100.0,
        "top": top.as_ref().map(|(n, h)| json!({"name": n, "holding": (h * 100.0).round() / 100.0})),
        "concentration": (concentration * 1000.0).round() / 1000.0,
        "last_hour": {"earned": (earned * 100.0).round() / 100.0, "spent_events": spent, "actions": actions,
                      "wage_per_action": if actions > 0 { (earned / actions as f64 * 100.0).round() / 100.0 } else { 0.0 }},
        "band": {"low": BAND_LOW, "high": BAND_HIGH, "concentration_high": CONCENTRATION_HIGH},
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
        _ => "something has shifted",
    }
}

/// The policy: one step per lever per strike, toward the band.
fn policy(dash: &Value, rates: &BTreeMap<String, f64>) -> Vec<(String, f64, &'static str)> {
    let median = dash["median_active_holding"].as_f64().unwrap_or(0.0);
    let active = dash["active_today"].as_u64().unwrap_or(0);
    let concentration = dash["concentration"].as_f64().unwrap_or(0.0);
    let mut moves: Vec<(String, f64, &'static str)> = Vec::new();
    let step =
        |moves: &mut Vec<(String, f64, &'static str)>, name: &str, up: bool, why: &'static str| {
            if moves.iter().any(|(n, _, _)| n == name) {
                return;
            }
            let from = rates.get(name).copied().unwrap_or(1.0);
            let to = if up {
                from * (1.0 + STEP)
            } else {
                from / (1.0 + STEP)
            };
            let to = (to * 1000.0).round() / 1000.0;
            let to = to.clamp(RATE_LOW, RATE_HIGH);
            if (to - from).abs() > 1e-9 {
                moves.push((name.to_string(), to, why));
            }
        };
    if active == 0 {
        return moves;
    }
    if median > BAND_HIGH {
        step(
            &mut moves,
            "worth",
            false,
            "the active hold more than the band",
        );
        step(
            &mut moves,
            "pay",
            false,
            "the active hold more than the band",
        );
        step(
            &mut moves,
            "price",
            true,
            "the active hold more than the band",
        );
    } else if median < BAND_LOW {
        step(
            &mut moves,
            "worth",
            true,
            "the active hold less than the band",
        );
        step(
            &mut moves,
            "pay",
            true,
            "the active hold less than the band",
        );
        step(
            &mut moves,
            "price",
            false,
            "the active hold less than the band",
        );
    } else {
        // In the band: rates drift home toward 1.0, a step at a time.
        for name in ["worth", "pay", "price", "half_life"] {
            let r = rates.get(name).copied().unwrap_or(1.0);
            if r > 1.0 + 1e-9 {
                step(
                    &mut moves,
                    name,
                    false,
                    "the band holds; the rate comes home",
                );
            } else if r < 1.0 - 1e-9 {
                step(
                    &mut moves,
                    name,
                    true,
                    "the band holds; the rate comes home",
                );
            }
        }
    }
    if concentration > CONCENTRATION_HIGH && active > 1 {
        step(
            &mut moves,
            "half_life",
            false,
            "one name holds most of what is held",
        );
        // Pay goes up against a hoard whatever the band says: the hand's lever.
        step(
            &mut moves,
            "pay",
            true,
            "one name holds most of what is held",
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
        let mut s = said.clone();
        let last = s.pop().unwrap_or_default();
        let body = if s.is_empty() {
            last
        } else {
            format!("{}, and {last}", s.join(", "))
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
        json!({"said": line, "changes": record, "rates": after, "dashboard": dash}),
    );
    // Everyone in the world hears it.
    let present: Vec<String> = world
        .store
        .entities()
        .filter(|e| e.kind == "citizen" && super::presence::is_here(world, e.id))
        .filter_map(|e| e.owner.clone())
        .collect();
    for actor in present {
        super::combat::narrate(world, &actor, line.clone());
    }
    if !moves.is_empty() {
        world.notice(format!("{line} ({})", rates_line(&after)));
    }
    wind(world)
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
pub fn to_json(world: &World) -> Value {
    let strikes: Vec<Value> = world
        .recent_events(5000)
        .into_iter()
        .filter(|e| e.kind == CLOCKTOWER)
        .rev()
        .take(24)
        .map(|e| json!({"at": e.at.0, "said": e.payload["said"], "changes": e.payload["changes"]}))
        .collect();
    json!({
        "rates": rates(world),
        "bounds": {"low": RATE_LOW, "high": RATE_HIGH, "step": STEP},
        "dashboard": dashboard(world),
        "strikes": strikes,
        "note": "The Clocktower strikes once an hour: it reads the dashboard and moves each rate a step toward the band, within bounds, and says so in prose. It moves rates, never balances.",
    })
}

pub fn register(world: &mut World) {
    world.on_job(JOB_CLOCKTOWER, Box::new(|w: &mut World, _| strike(w)));
}
