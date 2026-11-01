//! The board: who holds what, who earned, who kept, who gave.
//!
//! A leaderboard in a world named for fading is mostly a leaderboard of the present:
//! `holding` is the sum of a citizen's unfaded chunks now, and it falls while they are
//! away. The rest is the record, which does not fade: every chunk ever earned under a
//! name, what was paid to keep a room or a creature, what was given away, the stories
//! told, the words carried, the births carried, and the creatures that know the name.
//! Names the world has forgotten are off the board (their count stays, as a line); a
//! passing name is on it while it lasts, marked.

use super::{errand, exchange, kindle, memory, mind, CHUNK};
use crate::world::World;
use moondial_log::{Actor, Cursor};
use moondial_registry::{ActorKind, CommandError, CommandSpec, Outcome};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};

/// One citizen's line.
#[derive(Debug, Default, Clone)]
pub struct Line {
    pub name: String,
    pub passing: bool,
    pub here: bool,
    /// Unfaded chunks now.
    pub holding: f64,
    /// Every chunk ever created under the name, by size.
    pub earned: f64,
    /// Chunks paid into rooms and creatures.
    pub kept: u64,
    pub dug: u64,
    /// Chunks handed to another citizen.
    pub given: u64,
    pub stories: u64,
    pub errands: u64,
    /// Births carried.
    pub carried: u64,
    /// Named creatures that know the name.
    pub known_by: u64,
    /// Chunks the world forgot under the name.
    pub faded: u64,
    /// Actions through the Gate: a typed line is one, a written message its DO lines.
    pub actions: u64,
    /// Holding over earned: of all the name ever earned, what is still unfaded.
    pub retention: f64,
    /// Holding over actions: what there is to show per word through the Gate.
    pub per_action: f64,
}

/// The board, from the log and the store.
pub fn board(world: &World) -> (Vec<Line>, u64) {
    let mut lines: BTreeMap<String, Line> = BTreeMap::new();
    // Chunks by id, so a chunk's first size is credited to whoever it was made for.
    let mut chunk_owner: HashMap<u64, String> = HashMap::new();
    let mut sized: HashMap<u64, ()> = HashMap::new();
    let mut forgotten_names: std::collections::BTreeSet<String> = Default::default();
    for e in world
        .log
        .read_all(&world.id, Cursor::START)
        .unwrap_or_default()
    {
        let p = &e.payload;
        match e.kind.as_str() {
            // A dug room: the room's creation under its digger's name (the public
            // disremembered.dug came later, and the count must not double).
            "store.entity_created" if p["kind"] == "room" => {
                if let (Some(by), Actor::Citizen(a)) = (p["owner"].as_str(), &e.actor) {
                    if a == by {
                        lines.entry(by.to_string()).or_default().dug += 1;
                    }
                }
            }
            "store.entity_created" if p["kind"] == "chunk" => {
                if let (Some(id), Some(owner)) = (p["id"].as_u64(), p["owner"].as_str()) {
                    chunk_owner.insert(id, owner.to_string());
                }
            }
            "store.component_set" if p["component"] == CHUNK => {
                let Some(id) = p["id"].as_u64() else { continue };
                if sized.contains_key(&id) {
                    continue;
                }
                let size = p["fields"]["size"]
                    .as_f64()
                    .or_else(|| p["fields"]["size"]["value"].as_f64());
                if let (Some(size), Some(owner)) = (size, chunk_owner.get(&id)) {
                    sized.insert(id, ());
                    // Earned by the one it was made for, by their own act: a chunk
                    // given or kept is made anew under another owner by the giver's
                    // hand, and that is not an earning.
                    if matches!(&e.actor, Actor::Citizen(a) if a == owner) {
                        lines.entry(owner.clone()).or_default().earned += size;
                    }
                }
            }
            k if k == super::building::REMEMBERED => {
                if let Some(by) = p["by"].as_str() {
                    lines.entry(by.to_string()).or_default().kept += 1;
                }
            }
            k if k == exchange::GIVEN => {
                if let Some(from) = p["from"].as_str() {
                    lines.entry(from.to_string()).or_default().given += 1;
                }
            }
            k if k == exchange::STORY => {
                if let Some(from) = p["from"].as_str() {
                    lines.entry(from.to_string()).or_default().stories += 1;
                }
            }
            k if k == errand::DONE => {
                if let Some(from) = p["from"].as_str() {
                    lines.entry(from.to_string()).or_default().errands += 1;
                }
            }
            k if k == kindle::BORN => {
                if let Some(c) = p["carrier"].as_str() {
                    lines.entry(c.to_string()).or_default().carried += 1;
                }
            }
            k if k == memory::FADED => {
                if let Some(o) = p["owner"].as_str() {
                    lines.entry(o.to_string()).or_default().faded += 1;
                }
            }
            "gate.message" => {
                if let Some(from) = p["from"].as_str() {
                    let n = if p["compiled"] == true {
                        1
                    } else {
                        p["text"]
                            .as_str()
                            .map(|t| t.matches("[DO]").count().max(1))
                            .unwrap_or(1) as u64
                    };
                    lines.entry(from.to_string()).or_default().actions += n;
                }
            }
            k if k == super::presence::FORGOTTEN_NAME => {
                if let Some(n) = p["name"].as_str() {
                    forgotten_names.insert(n.to_string());
                }
            }
            _ => {}
        }
    }
    // The present: who is still a citizen, what they hold, who knows them.
    let mut known: HashMap<String, u64> = HashMap::new();
    for e in world.store.entities().filter(|e| e.kind == "creature") {
        if mind::name(world, e.id).is_some() {
            for k in mind::keepers(world, e.id) {
                *known.entry(k).or_default() += 1;
            }
        }
    }
    let mut out = Vec::new();
    for (name, mut l) in lines {
        let Some(f) = super::find_citizen(world, &name) else {
            forgotten_names.insert(name.clone());
            continue;
        };
        l.name = super::plain_str(world, f.citizen, super::CITIZEN, "name")
            .unwrap_or(&name)
            .to_string();
        l.passing = super::is_passing(world, f.citizen);
        l.here = super::presence::is_here(world, f.citizen);
        l.holding = memory::level(world, f.window);
        l.known_by = known.get(&l.name).copied().unwrap_or(0);
        l.retention = if l.earned > 0.0 {
            l.holding / l.earned
        } else {
            0.0
        };
        l.per_action = if l.actions > 0 {
            l.holding / l.actions as f64
        } else {
            0.0
        };
        out.push(l);
    }
    // Citizens who never earned a chunk are on the board too, at nothing.
    for e in world.store.entities().filter(|e| e.kind == "citizen") {
        let Some(name) = super::plain_str(world, e.id, super::CITIZEN, "name") else {
            continue;
        };
        if out.iter().any(|l| l.name == name) {
            continue;
        }
        let Some(f) = super::find_citizen(world, name) else {
            continue;
        };
        out.push(Line {
            name: name.to_string(),
            passing: super::is_passing(world, f.citizen),
            here: super::presence::is_here(world, f.citizen),
            holding: memory::level(world, f.window),
            known_by: known.get(name).copied().unwrap_or(0),
            ..Default::default()
        });
    }
    // Forgotten: every name the record says the world forgot, less any taken again.
    let forgotten = forgotten_names
        .iter()
        .filter(|n| super::find_citizen(world, n).is_none())
        .count() as u64;
    out.sort_by(|a, b| {
        b.holding
            .partial_cmp(&a.holding)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                b.earned
                    .partial_cmp(&a.earned)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
            .then(a.name.cmp(&b.name))
    });
    (out, forgotten)
}

/// The board as JSON.
pub fn to_json(world: &World) -> Value {
    let (lines, forgotten) = board(world);
    json!({
        "as_of": world.now().0,
        "citizens": lines.iter().map(|l| json!({
            "name": l.name, "passing": l.passing, "here": l.here,
            "holding": (l.holding * 100.0).round() / 100.0,
            "earned": (l.earned * 100.0).round() / 100.0,
            "kept": l.kept, "dug": l.dug, "given": l.given, "stories": l.stories,
            "errands": l.errands, "carried": l.carried, "known_by": l.known_by, "faded": l.faded,
            "actions": l.actions,
            "retention": (l.retention * 1000.0).round() / 1000.0,
            "per_action": (l.per_action * 100.0).round() / 100.0,
        })).collect::<Vec<_>>(),
        "forgotten": forgotten,
        "note": "holding is the sum of unfaded chunks now and falls while you are away; the rest is the record and does not fade. retention is holding over earned; per_action is holding over actions through the Gate (a typed line is one, a written message its DO lines). Names the world has forgotten are off the board.",
    })
}

/// The board as prose: the ten highest holders, and where the asker stands.
pub fn text(world: &World, asker: Option<&str>) -> String {
    let (lines, forgotten) = board(world);
    if lines.is_empty() {
        return "The board is empty: no one is here.".into();
    }
    let mut out = String::from("Holding now, highest first:");
    for (i, l) in lines.iter().take(10).enumerate() {
        out.push_str(&format!(
            "\n  {:>2}. {:<20} {:>9.2}  earned {:>9.2}  kept {:>3.0}%  per action {:>6.2}  given {:>3}  stories {:>3}  errands {:>3}{}{}",
            i + 1,
            l.name,
            l.holding,
            l.earned,
            l.retention * 100.0,
            l.per_action,
            l.given,
            l.stories,
            l.errands,
            if l.known_by > 0 { format!("  known by {}", l.known_by) } else { String::new() },
            if l.passing { "  (passing)" } else { "" }
        ));
    }
    if let Some(a) = asker {
        if let Some(i) = lines.iter().position(|l| l.name.eq_ignore_ascii_case(a)) {
            if i >= 10 {
                let l = &lines[i];
                out.push_str(&format!(
                    "\n  …\n  {:>2}. {:<20} {:>9.2}  earned {:>9.2}  kept {:>3.0}%  per action {:>6.2}  given {:>3}  stories {:>3}  errands {:>3}",
                    i + 1, l.name, l.holding, l.earned, l.retention * 100.0, l.per_action, l.given, l.stories, l.errands
                ));
            }
        }
    }
    out.push_str(&format!(
        "\n{} on the board; {forgotten} name{} the world has forgotten. Holding fades while you are away; the rest is the record. kept% is holding over all you earned; per action is holding over your lines through the Gate.",
        lines.len(),
        if forgotten == 1 { "" } else { "s" }
    ));
    out
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.board",
            "The board: who holds the most unfaded memory now, and what each name earned, kept, gave, told and carried, by the record. Holding fades while you are away; the rest does not. Names the world has forgotten are off it.",
        )
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, ctx, _| -> Result<Outcome, CommandError> {
            let v = to_json(w);
            Ok(Outcome::new(text(w, Some(&ctx.actor)), v))
        }),
    );
}
