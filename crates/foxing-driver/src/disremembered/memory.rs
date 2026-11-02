//! Chunks, the active window, the Memory Palace, and PTO.

use super::{internal, me, plain_num, Curves, CHUNK};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{
    ActorKind, Args, CallContext, CommandError, CommandSpec, Outcome, ParamType,
};
use foxing_store::{EntityId, Field, StoreEvent};
use foxing_time::TimedValue;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Create a chunk in a window at `size`, fading on the active curve for its vividity.
pub fn earn_chunk(
    world: &mut World,
    actor: &str,
    window: EntityId,
    source: &str,
    size: f64,
    vividity: f64,
    curves: &Curves,
) -> Result<EntityId, CommandError> {
    let id = world.store.allocate_id();
    let me = Actor::Citizen(actor.to_string());
    world
        .record(
            me.clone(),
            StoreEvent::EntityCreated {
                id,
                kind: "chunk".into(),
                owner: Some(actor.into()),
                parent: Some(window),
                scopes: vec![],
            },
        )
        .map_err(internal)?;
    let now = world.now();
    // New memory fades at the world's rate: the Clocktower's one lever on time. A
    // hoard's fades faster still, by the Conclave's statute, on the hoard alone.
    let pace =
        super::economy::rate(world, "half_life") / super::conclave::hoard_factor(world, actor);
    let curve = match curves.active(vividity) {
        foxing_time::Curve::Exponential { half_life } => foxing_time::Curve::Exponential {
            half_life: foxing_time::Duration((half_life.0 as f64 * pace) as i64),
        },
        c => c,
    };
    let mut fields = BTreeMap::new();
    fields.insert(
        "value".into(),
        Field::Timed(TimedValue::new(size, now, curve)),
    );
    fields.insert("size".into(), Field::Plain(json!(size)));
    fields.insert("vividity".into(), Field::Plain(json!(vividity)));
    fields.insert("source".into(), Field::Plain(json!(source)));
    fields.insert("tier".into(), Field::Plain(json!("active")));
    // A passing name's earnings are marked at the minting, and the mark goes with
    // the chunk wherever it is given: a kept name cannot launder a ring of fresh
    // names' stories through its own allowance.
    if super::find_citizen(world, actor)
        .map(|f| super::is_passing(world, f.citizen))
        .unwrap_or(false)
    {
        fields.insert("provenance".into(), Field::Plain(json!("passing")));
    }
    world
        .record(
            me,
            StoreEvent::ComponentSet {
                id,
                component: CHUNK.into(),
                fields,
            },
        )
        .map_err(internal)?;
    Ok(id)
}

/// Whether a chunk was earned by a passing name, wherever it is now.
pub fn is_passing_chunk(world: &World, chunk: EntityId) -> bool {
    super::plain_str(world, chunk, CHUNK, "provenance") == Some("passing")
}

/// The sum of a window's chunk values now.
pub fn level(world: &World, window: EntityId) -> f64 {
    let now = world.now();
    world
        .store
        .children(window)
        .filter(|e| e.kind == "chunk")
        .filter_map(|e| world.store.read_field(e.id, CHUNK, "value", now).ok())
        .filter_map(|v| v.as_f64())
        .sum::<f64>()
        .max(0.0)
}

/// The query behind `chunks` and `palace`: every chunk in a container, every field.
pub fn chunks_query(container: EntityId) -> Value {
    json!({
        "root": {"entity": container.0},
        "select": {},
        "relations": {"contents": {"kinds": ["chunk"], "select": {CHUNK: "*"}}}
    })
}

/// How many chunk rows a readout lists; the rest are summed in one line. The
/// observation engine refuses a relation past its breadth (a hundred), and a window
/// holds more than that after an afternoon on pto, so the driver's own readout is
/// built from the store, largest first, and never fails for a legal state.
pub const ROWS_SHOWN: usize = 100;

/// Chunk rows of a window or palace, largest value first, with an observation of the
/// same rows when the engine can give one (it cites the moment; past the breadth it
/// is null and the rows stand on their own).
fn chunk_rows(
    world: &World,
    ctx: &CallContext,
    container: EntityId,
) -> Result<(Vec<Value>, Value), CommandError> {
    let now = world.now();
    let mut rows: Vec<Value> = world
        .store
        .children(container)
        .filter(|e| e.kind == "chunk")
        .map(|e| {
            let f = |name: &str| world.store.read_field(e.id, CHUNK, name, now).ok();
            // Its own clock: the half-life (or the power law's scale) as it stands, so
            // a taxed gift shows its faster fade here and not only to observe.
            let halves = world
                .store
                .timed_field(e.id, CHUNK, "value")
                .ok()
                .and_then(|t| match t.curve {
                    foxing_time::Curve::Exponential { half_life } => Some(half_life.0),
                    foxing_time::Curve::PowerLaw { scale, .. } => Some(scale.0),
                    _ => None,
                });
            json!({"id": e.id.0, "source": f("source"), "tier": f("tier"), "size": f("size"), "vividity": f("vividity"), "value": f("value"), "halves_ms": halves, "taxed": f("taxed"), "provenance": f("provenance")})
        })
        .collect();
    rows.sort_by(|a, b| {
        b["value"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["value"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let observation = if rows.len() <= ROWS_SHOWN {
        super::observe_as(world, ctx, chunks_query(container))
            .ok()
            .and_then(|o| serde_json::to_value(o).ok())
            .unwrap_or(Value::Null)
    } else {
        Value::Null
    };
    Ok((rows, observation))
}

/// The rows past `ROWS_SHOWN`, as one line: how many, and what they are worth.
fn rest_line(rows: &[Value]) -> Option<String> {
    if rows.len() <= ROWS_SHOWN {
        return None;
    }
    let rest = &rows[ROWS_SHOWN..];
    let worth: f64 = rest.iter().filter_map(|r| r["value"].as_f64()).sum();
    Some(format!(
        "  … and {} smaller, worth {worth:.2} together (the {ROWS_SHOWN} largest are listed; observe the window with a where on value for the rest)",
        rest.len()
    ))
}

/// The one line that says how fast the window fades, so nobody has to measure it.
fn fade_rate(active_hl_ms: i64) -> String {
    format!(
        "A chunk halves every {} at vividity 0 and every {} at vividity 1; shelved in the palace it fades about five times slower.",
        super::combat::human(active_hl_ms),
        super::combat::human(active_hl_ms * 3)
    )
}

fn rows_text(rows: &[Value]) -> Vec<String> {
    let mut out: Vec<String> = rows
        .iter()
        .take(ROWS_SHOWN)
        .map(|r| {
            format!(
                "  chunk-{} {:<10} {:>9.2} of {:<6} vividity {:.2}{}{}{}",
                r["id"],
                r["source"].as_str().unwrap_or("?"),
                r["value"].as_f64().unwrap_or(0.0),
                r["size"],
                r["vividity"].as_f64().unwrap_or(0.0),
                r["halves_ms"]
                    .as_i64()
                    .map(|ms| format!("  halves in {}", super::combat::human(ms)))
                    .unwrap_or_default(),
                if r["taxed"].is_number() {
                    "  (taxed)"
                } else {
                    ""
                },
                if r["provenance"].as_str() == Some("passing") {
                    "  (passing)"
                } else {
                    ""
                }
            )
        })
        .collect();
    out.extend(rest_line(rows));
    out
}

pub(crate) fn resolve_chunk(
    world: &World,
    ctx: &CallContext,
    arg: &str,
    within: &[EntityId],
) -> Result<EntityId, CommandError> {
    let id: u64 = arg.trim_start_matches("chunk-").parse().map_err(|_| {
        CommandError::precondition(format!("'{arg}' is not a chunk id; chunks shows yours."))
    })?;
    let id = EntityId(id);
    let e = world
        .store
        .entity(id)
        .ok_or_else(|| CommandError::precondition(format!("There is no chunk {}.", id.0)))?;
    if e.kind != "chunk" || e.owner.as_deref() != Some(ctx.actor.as_str()) {
        return Err(CommandError::precondition(format!(
            "Chunk {} is not yours.",
            id.0
        )));
    }
    if !within.contains(&e.parent.unwrap_or(EntityId(0))) {
        return Err(CommandError::precondition(format!(
            "Chunk {} is not where that command expects it.",
            id.0
        )));
    }
    Ok(id)
}

/// A chunk worth less than this is forgotten outright: a halving never reaches zero,
/// and a window full of ten-thousandths is a list nobody wants to read.
pub const FORGET_BELOW: f64 = 0.05;
/// A chunk faded to nothing and was removed: `{owner, chunk, source, container}`.
pub const FADED: &str = "disremembered.faded";

/// Forget the chunks that have faded to nothing, wherever they lie: a citizen's
/// window or palace, a creature's window. Once a minute is plenty; the value is read
/// at the world's clock, so a paused owner's chunks are never touched.
pub fn sweep(world: &mut World) -> Result<(), String> {
    let now = world.now();
    if now.0 / 60_000 == world.last_forget_minute() {
        return Ok(());
    }
    world.set_last_forget_minute(now.0 / 60_000);
    let gone: Vec<(EntityId, Option<String>, Option<EntityId>, String)> = world
        .store
        .entities()
        .filter(|e| e.kind == "chunk")
        .filter_map(|e| {
            let v = world
                .store
                .read_field(e.id, CHUNK, "value", now)
                .ok()?
                .as_f64()?;
            (v < FORGET_BELOW).then(|| {
                (
                    e.id,
                    e.owner.clone(),
                    e.parent,
                    super::plain_str(world, e.id, CHUNK, "source")
                        .unwrap_or("something")
                        .to_string(),
                )
            })
        })
        .collect();
    for (id, owner, container, source) in gone {
        world
            .record(Actor::Driver, StoreEvent::EntityRemoved { id })
            .map_err(|e| e.to_string())?;
        let _ = world.note(
            Actor::Driver,
            FADED,
            json!({"owner": owner, "chunk": id.0, "source": source, "container": container.map(|c| c.0)}),
        );
        if let Some(who) = owner {
            super::combat::narrate(
                world,
                &who,
                format!("The memory of {source} is gone; you could not have said when."),
            );
        }
    }
    super::craft::sweep(world)
}

pub fn register(world: &mut World, curves: Curves) {
    world.on_tick(Box::new(sweep));
    let active_hl_ms = curves.active_half_life.0;
    world.register(
        CommandSpec::new(
            "disremembered.chunks",
            "Your active window: every chunk, its current value, and your level. The reply states the rate at which chunks fade.",
        ),
        Box::new(move |w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let (rows, observation) = chunk_rows(w, ctx, f.window)?;
            let level = level(w, f.window);
            let attack = super::combat::attack_for_level(level);
            // At the Clocktower's rate this hour, not the base figure.
            let rate = fade_rate(
                (active_hl_ms as f64 * super::economy::rate(w, "half_life")) as i64,
            );
            let text = if rows.is_empty() {
                // Name a creature that would take a story, and where it is, rather
                // than telling an empty-handed newcomer to go and ask.
                let listener = w
                    .store
                    .entities()
                    .filter(|e| e.kind == "creature")
                    .find_map(|e| {
                        let given = super::mind::name(w, e.id)?;
                        let room = e.parent.and_then(|r| super::plain_str(w, r, super::ROOM, "name"))?;
                        Some(format!("{given} the {}, in {room}, would take one", super::plain_str(w, e.id, super::CREATURE, "name").unwrap_or("?")))
                    })
                    .unwrap_or_else(|| "none has a name just now".to_string());
                format!("Your window is empty. Level 0, attack {attack:.0}. A chunk is earned by felling a creature (attack; the Fogbank east of the Threshold has wisps) or by telling a named creature a story of a dozen words (story; {listener}). {rate}")
            } else {
                format!(
                    "Level {level:.2}, attack {attack:.0} (your level is the sum of these chunks' current values, so it falls as they fade and when you spend one; attack is three and the square root of level)\n{}\n{rate}",
                    rows_text(&rows).join("\n")
                )
            };
            Ok(Outcome::new(
                text,
                json!({"level": level, "attack": attack, "chunks": rows, "active_half_life_ms": active_hl_ms, "observation": observation}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.palace",
            "Your Memory Palace: shelved chunks, fading slowly, counting for nothing.",
        ),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let (rows, observation) = chunk_rows(w, ctx, f.palace)?;
            let on_shelf: f64 = rows.iter().filter_map(|r| r["value"].as_f64()).sum();
            let limit = super::conclave::statute(w, "shelf_limit");
            let text = if rows.is_empty() {
                format!("Your palace is empty. The shelf holds {limit:.0} at most, by the Conclave's statute.")
            } else {
                format!("Palace, {on_shelf:.2} on the shelf of {limit:.0} the statute allows\n{}", rows_text(&rows).join("\n"))
            };
            Ok(Outcome::new(
                text,
                json!({"chunks": rows, "on_shelf": on_shelf, "shelf_limit": limit, "observation": observation}),
            ))
        }),
    );

    let c = curves.clone();
    world.register(
        CommandSpec::new("disremembered.earn", "Admin: conjure a chunk by hand.")
            .param("source", ParamType::String, true, "what it is a memory of")
            .param(
                "size",
                ParamType::Number,
                true,
                "its XP value at the moment of earning",
            )
            .param("vividity", ParamType::Number, false, "0 to 1; default 0")
            .allowed(&[ActorKind::Admin]),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let source = args.str("source").unwrap_or("unknown").to_string();
            let size = args.num("size").unwrap_or(0.0);
            let vividity = args.num("vividity").unwrap_or(0.0).clamp(0.0, 1.0);
            if size <= 0.0 || size > 100_000.0 {
                return Err(CommandError::precondition(
                    "size must be between 0 and 100000 XP.",
                ));
            }
            let id = earn_chunk(w, &ctx.actor, f.window, &source, size, vividity, &c)?;
            Ok(Outcome::new(
                format!(
                    "You remember the {source}: chunk-{} worth {size:.0} XP, already fading.",
                    id.0
                ),
                json!({"chunk": id.0, "size": size, "vividity": vividity}),
            ))
        }),
    );

    let c = curves.clone();
    world.register(
        CommandSpec::new("disremembered.shelve", "Move a chunk from your window into your palace, where it fades slower and counts for nothing. A kept name's verb.")
            .param("chunk", ParamType::Entity, true, "a chunk in your window"),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            super::kept_only(w, ctx, "shelve")?;
            let f = me(w, ctx)?;
            // Not mid-fight: a delver shelved the big chunks, retreated into a death
            // that cost nothing, and took them down again for a twentieth of an XP.
            if super::combat::engaged_with(w, f.citizen).is_some() {
                return Err(CommandError::precondition(
                    "You are in a fight; nothing goes on the shelf until it is over.",
                ));
            }
            let id = resolve_chunk(w, ctx, args.str("chunk").unwrap_or(""), &[f.window])?;
            let vividity = plain_num(w, id, CHUNK, "vividity").unwrap_or(0.0);
            // The Conclave's shelf limit: a shelf at the limit takes nothing more.
            let limit = super::conclave::statute(w, "shelf_limit");
            let on_shelf = super::conclave::shelved(w, f.palace);
            let this = w.store.read_field(id, CHUNK, "value", w.now()).map_err(internal)?.as_f64().unwrap_or(0.0);
            if on_shelf + this > limit {
                return Err(CommandError::precondition(format!(
                    "Your shelf holds {on_shelf:.0} and the Conclave's limit is {limit:.0}; chunk-{} at {this:.0} will not fit.", id.0
                )));
            }
            let actor = Actor::Citizen(ctx.actor.clone());
            w.record(actor.clone(), StoreEvent::FieldRecurved { id, component: CHUNK.into(), field: "value".into(), curve: c.palace(vividity) }).map_err(internal)?;
            w.record(actor.clone(), StoreEvent::FieldSet { id, component: CHUNK.into(), field: "tier".into(), value: json!("palace") }).map_err(internal)?;
            w.record(actor, StoreEvent::EntityMoved { id, parent: Some(f.palace) }).map_err(internal)?;
            let v = w.store.read_field(id, CHUNK, "value", w.now()).map_err(internal)?;
            Ok(Outcome::new(
                format!("You shelve chunk-{} at {:.2}. It will fade slowly now, and count for nothing until you reminisce.", id.0, v.as_f64().unwrap_or(0.0)),
                json!({"chunk": id.0, "value": v}),
            ))
        }),
    );

    let c = curves;
    world.register(
        CommandSpec::new(
            "disremembered.reminisce",
            "Bring a chunk back from your palace into your window, at its current value.",
        )
        .param("chunk", ParamType::Entity, true, "a chunk in your palace"),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            if super::combat::engaged_with(w, f.citizen).is_some() {
                return Err(CommandError::precondition(
                    "You are in a fight; the shelf waits until it is over.",
                ));
            }
            let id = resolve_chunk(w, ctx, args.str("chunk").unwrap_or(""), &[f.palace])?;
            // A chunk an engrosser entered into the minutes stays there a while: the
            // shelving was undone by a free reminisce (the owner's call: a short delay).
            if let Some(left) = super::stacks::minuted_for(w, id) {
                return Err(CommandError::precondition(format!(
                    "chunk-{} is in the Conclave's minutes; the engrosser's hand is still on the page for {}. Reminisce it after.",
                    id.0,
                    super::combat::human(left)
                )));
            }
            let vividity = plain_num(w, id, CHUNK, "vividity").unwrap_or(0.0);
            let actor = Actor::Citizen(ctx.actor.clone());
            w.record(
                actor.clone(),
                StoreEvent::FieldRecurved {
                    id,
                    component: CHUNK.into(),
                    field: "value".into(),
                    // On the Clocktower's pace, as a fresh chunk would be: a shelve
                    // and reminisce once came back at the base rate, fading slower
                    // than its neighbors.
                    curve: match c.active(vividity) {
                        foxing_time::Curve::Exponential { half_life } => {
                            foxing_time::Curve::Exponential {
                                half_life: foxing_time::Duration(
                                    (half_life.0 as f64 * super::economy::rate(w, "half_life")
                                        / super::conclave::hoard_factor(w, &ctx.actor))
                                        as i64,
                                ),
                            }
                        }
                        other => other,
                    },
                },
            )
            .map_err(internal)?;
            w.record(
                actor.clone(),
                StoreEvent::FieldSet {
                    id,
                    component: CHUNK.into(),
                    field: "tier".into(),
                    value: json!("active"),
                },
            )
            .map_err(internal)?;
            w.record(
                actor,
                StoreEvent::EntityMoved {
                    id,
                    parent: Some(f.window),
                },
            )
            .map_err(internal)?;
            let v = w
                .store
                .read_field(id, CHUNK, "value", w.now())
                .map_err(internal)?;
            Ok(Outcome::new(
                format!(
                    "You reminisce: chunk-{} returns at {:.2}, and counts again.",
                    id.0,
                    v.as_f64().unwrap_or(0.0)
                ),
                json!({"chunk": id.0, "value": v}),
            ))
        }),
    );

    world.register(
        CommandSpec::new("disremembered.pto", "Ask for leave: your clock stops when the room disremembers you (ten minutes after your last word), so nothing of yours fades or heals while you are gone, and it runs again the moment you act. Two windows in thirty days, each seven days at most. A kept name's verb."),
        Box::new(|w: &mut World, ctx, _| {
            super::kept_only(w, ctx, "pause its clock")?;
            let f = me(w, ctx)?;
            if super::on_pto(w, &ctx.actor) {
                return Err(CommandError::precondition("You are already on PTO."));
            }
            let taken = super::pto_taken(w, f.citizen);
            if taken.len() >= super::PTO_WINDOWS {
                let next = taken.iter().min().copied().unwrap_or(0) + super::PTO_QUOTA_MS;
                return Err(CommandError::precondition(format!(
                    "You have taken {} windows of PTO in thirty days, which is what the rules allow; the next is yours in {}.",
                    super::PTO_WINDOWS,
                    super::combat::human((next - w.now().0).max(0))
                )));
            }
            let mut fields = BTreeMap::new();
            let mut all: Vec<Value> = taken.iter().map(|t| json!(t)).collect();
            all.push(json!(w.now().0));
            fields.insert("pto".into(), Field::Plain(json!(true)));
            fields.insert("pto_taken".into(), Field::Plain(Value::Array(all)));
            super::set_citizen_fields(w, &ctx.actor, f.citizen, fields)?;
            Ok(Outcome::new(
                format!("Leave granted: your clock stops when the room disremembers you, ten minutes after your last word, and runs again the moment you act. Window {} of {} this month; seven days at most.", taken.len() + 1, super::PTO_WINDOWS),
                json!({"pto": true, "begins_after_ms": crate::world::ABSENCE.0, "windows_used": taken.len() + 1, "windows": super::PTO_WINDOWS}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.resume",
            "End your PTO, or take back the asking. Your clock runs; acting does the same.",
        ),
        Box::new(|w: &mut World, ctx, _| {
            let f = me(w, ctx)?;
            let asked = matches!(
                w.store
                    .entity(f.citizen)
                    .and_then(|e| e.components.get(super::CITIZEN))
                    .and_then(|c| c.fields.get("pto")),
                Some(Field::Plain(Value::Bool(true)))
            );
            if !asked && !super::on_pto(w, &ctx.actor) {
                return Err(CommandError::precondition("You are not on PTO."));
            }
            match super::end_pto(w, &ctx.actor, f.citizen, "you asked")? {
                Some(ms) => Ok(Outcome::new(
                    format!(
                        "Your clock runs again; it stood still for {}.",
                        super::combat::human(ms)
                    ),
                    json!({"paused": false, "for_ms": ms}),
                )),
                None => Ok(Outcome::new(
                    "The asking is taken back; your clock never stopped.",
                    json!({"paused": false, "cancelled": true}),
                )),
            }
        }),
    );
}
