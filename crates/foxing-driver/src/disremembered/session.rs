//! A session, and the words that reached you.
//!
//! A session begins when a name arrives (logs in, or acts again after the room had
//! disremembered them): the moment, the level and the log's position are kept on the
//! citizen, and `session` reads what has happened since, as a person leaving a table
//! would want it: how long, what the level did, what was earned, lost, dug, told,
//! written, said. The play page says it on `leave`.
//!
//! `messages` is the record of what was said to you, late or not: a `tell` that
//! reached you, one left for you while you were away, a creature's answer, the
//! Docent's. The owner once thought his were lost; they had been delivered once, at
//! the top of an arrival, and never shown again.

use super::{plain_num, CITIZEN};
use crate::world::World;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::Field;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// How many events back `messages` looks.
const MESSAGES_SPAN: usize = 8000;
const MESSAGES_DEFAULT: usize = 12;
const MESSAGES_MAX: usize = 40;

/// A session begins: kept on the citizen for `session` to read against.
pub fn begin(world: &mut World, actor: &str) -> Result<(), String> {
    let Some(f) = super::find_citizen(world, actor) else {
        return Ok(());
    };
    let head = world
        .log
        .head(&world.id)
        .ok()
        .flatten()
        .map(|s| s.0)
        .unwrap_or(0);
    let mut changes = BTreeMap::new();
    changes.insert("session_at".into(), Field::Plain(json!(world.now().0)));
    changes.insert(
        "session_level".into(),
        Field::Plain(json!(super::memory::level(world, f.window))),
    );
    changes.insert("session_seq".into(), Field::Plain(json!(head)));
    super::set_citizen_fields(world, actor, f.citizen, changes).map_err(|e| e.to_string())
}

fn ago(world: &World, at: i64) -> String {
    let ms = (world.now().0 - at).max(0);
    if ms < 60_000 {
        "just now".to_string()
    } else {
        format!("{} ago", super::combat::human(ms))
    }
}

/// What has happened since you arrived, as a line.
pub fn summary(world: &World, actor: &str) -> Result<Outcome, CommandError> {
    let f = super::find_citizen(world, actor)
        .ok_or_else(|| CommandError::precondition(format!("{actor} has not arrived.")))?;
    let at =
        plain_num(world, f.citizen, CITIZEN, "session_at").unwrap_or(world.now().0 as f64) as i64;
    let level0 = plain_num(world, f.citizen, CITIZEN, "session_level").unwrap_or(0.0);
    let seq0 = plain_num(world, f.citizen, CITIZEN, "session_seq").unwrap_or(0.0) as u64;
    let level = super::memory::level(world, f.window);
    let events = world
        .log
        .read_all(&world.id, foxing_log::Cursor { after: seq0 })
        .unwrap_or_default();
    let window = f.window.0;
    let mut n = BTreeMap::new();
    let mut bump = |k: &str| *n.entry(k.to_string()).or_insert(0usize) += 1;
    for e in &events {
        let p = &e.payload;
        match e.kind.as_str() {
            "store.entity_created" if p["kind"] == "chunk" && p["parent"] == window => {
                bump("earned")
            }
            k if k == super::combat::DIED && p["citizen"] == actor => bump("falls"),
            k if k == super::building::DUG && p["by"] == actor => {
                bump("dug");
                bump("spent")
            }
            k if (k == super::building::REMEMBERED || k == super::craft::KEPT)
                && p["by"] == actor =>
            {
                bump("given");
                bump("spent")
            }
            k if k == super::craft::MADE && p["by"] == actor => {
                bump("made");
                if !p["spent"].is_null() {
                    bump("spent")
                }
            }
            k if k == super::exchange::STORY && p["from"] == actor => bump("stories"),
            k if k == super::building::APPENDED && p["by"] == actor => bump("written"),
            k if k == super::speech::SAY && p["from"] == actor => bump("said"),
            k if k == super::speech::SAY && p["to"] == actor => bump("heard"),
            k if k == super::talk::TELL && p["to"] == actor => bump("heard"),
            k if k == super::docent::ASKED && p["from"] == actor => bump("asked"),
            _ => {}
        }
    }
    let c = |k: &str| n.get(k).copied().unwrap_or(0);
    let plural = |k: usize, one: &str, many: &str| {
        if k == 1 {
            one.to_string()
        } else {
            many.to_string()
        }
    };
    let mut parts: Vec<String> = Vec::new();
    if c("earned") > 0 {
        parts.push(format!(
            "{} {} earned",
            c("earned"),
            plural(c("earned"), "memory", "memories")
        ));
    }
    if c("falls") > 0 {
        parts.push(format!(
            "{} {}",
            c("falls"),
            plural(c("falls"), "fall", "falls")
        ));
    }
    if c("dug") > 0 {
        parts.push(format!(
            "{} {} dug",
            c("dug"),
            plural(c("dug"), "room", "rooms")
        ));
    }
    if c("stories") > 0 {
        parts.push(format!(
            "{} {} told",
            c("stories"),
            plural(c("stories"), "story", "stories")
        ));
    }
    if c("given") > 0 {
        parts.push(format!(
            "{} {} given",
            c("given"),
            plural(c("given"), "memory", "memories")
        ));
    }
    if c("made") > 0 {
        parts.push(format!(
            "{} {} made",
            c("made"),
            plural(c("made"), "thing", "things")
        ));
    }
    if c("spent") > 0 {
        parts.push(format!(
            "{} {} spent",
            c("spent"),
            plural(c("spent"), "memory", "memories")
        ));
    }
    if c("written") > 0 {
        parts.push(format!(
            "{} {} written",
            c("written"),
            plural(c("written"), "line", "lines")
        ));
    }
    if c("said") > 0 || c("heard") > 0 {
        parts.push(format!("{} said, {} heard", c("said"), c("heard")));
    }
    if c("asked") > 0 {
        parts.push(format!("{} asked of the Docent", c("asked")));
    }
    let delta = level - level0;
    let text = format!(
        "Here {} (since {}). Level {:.1} \u{2192} {:.1} ({}{:.1}). {}",
        super::combat::human((world.now().0 - at).max(0)),
        ago(world, at),
        level0,
        level,
        if delta >= 0.0 { "+" } else { "" },
        delta,
        if parts.is_empty() {
            "Nothing yet; the room is waiting.".to_string()
        } else {
            format!("Since you arrived: {}.", parts.join(", "))
        }
    );
    Ok(Outcome::new(
        text,
        json!({"since": at, "ms": (world.now().0 - at).max(0), "level_then": level0, "level": level, "counts": n}),
    ))
}

/// The words that reached you, latest last.
pub fn messages(world: &World, actor: &str, count: usize) -> Outcome {
    let count = count.clamp(1, MESSAGES_MAX);
    let recent = world.recent_events(MESSAGES_SPAN);
    let mut rows: Vec<Value> = Vec::new();
    for e in &recent {
        let p = &e.payload;
        let row = match e.kind.as_str() {
            k if k == super::speech::SAY && p["to"] == actor && p["from"] != actor => {
                let from = p["from"].as_str().unwrap_or("?");
                let who = if p["creature"].is_null() {
                    from.to_string()
                } else {
                    format!("{from} (a creature)")
                };
                json!({"at": e.at.0, "from": who, "text": p["text"], "kind": "said"})
            }
            k if k == super::talk::TELL && p["to"] == actor => {
                json!({"at": e.at.0, "from": p["from"], "text": p["text"], "kind": "left"})
            }
            k if k == super::combat::NARRATIVE
                && p["to"] == actor
                && p["text"].as_str().is_some_and(|t| {
                    t.starts_with("The Docent: ")
                        || t.starts_with("The Docent finds you")
                        || t.starts_with("The Docent comes back")
                        || t.starts_with("The Docent shakes its head")
                }) =>
            {
                json!({"at": e.at.0, "from": "the Docent", "text": p["text"], "kind": "docent"})
            }
            k if k == super::combat::NARRATIVE
                && p["to"] == actor
                && p["text"]
                    .as_str()
                    .is_some_and(|t| t.starts_with("Upon your petition")) =>
            {
                json!({"at": e.at.0, "from": "the Conclave", "text": p["text"], "kind": "conclave"})
            }
            _ => continue,
        };
        rows.push(row);
    }
    let total = rows.len();
    if rows.len() > count {
        rows.drain(..rows.len() - count);
    }
    let text = if rows.is_empty() {
        "No words have reached you in a while. `tell <name> \"…\"` leaves one for someone; theirs come here.".to_string()
    } else {
        let lines: Vec<String> = rows
            .iter()
            .map(|r| {
                let when = ago(world, r["at"].as_i64().unwrap_or(0));
                match r["kind"].as_str() {
                    Some("left") => format!(
                        "{when}: {} left word for you: \"{}\"",
                        r["from"].as_str().unwrap_or("?"),
                        r["text"].as_str().unwrap_or("")
                    ),
                    Some("docent") | Some("conclave") => {
                        format!("{when}: {}", r["text"].as_str().unwrap_or(""))
                    }
                    _ => format!(
                        "{when}: {} said to you, \"{}\"",
                        r["from"].as_str().unwrap_or("?"),
                        r["text"].as_str().unwrap_or("")
                    ),
                }
            })
            .collect();
        format!(
            "Words for you, the last {} of {total}:\n{}",
            rows.len(),
            lines.join("\n")
        )
    };
    Outcome::new(text, json!({"messages": rows, "total": total}))
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.session",
            "What has happened since you arrived: how long you have been here, what your level did, what you earned, lost, dug, told, wrote and said. A read.",
        )
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, _args: &Args| summary(w, &ctx.actor)),
    );
    world.register(
        CommandSpec::new(
            "disremembered.messages",
            "The words that reached you, latest last: what was said to you here, what was left for you while you were away, a creature's answers, the Docent's. A read.",
        )
        .param("count", ParamType::Integer, false, "how many, 12 by default, 40 at most")
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let count = args.int("count").unwrap_or(MESSAGES_DEFAULT as i64).max(1) as usize;
            Ok(messages(w, &ctx.actor, count))
        }),
    );
}
