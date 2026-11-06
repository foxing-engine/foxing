//! `news` (R81): the world's own gossip in the Docent's voice, so a returning name
//! can tell the world lived while they were away: the last division's decisions,
//! who fell, who was named, what was dug, Morphos sighted, what happened on its own.

use super::{plain_str, ROOM};
use crate::world::World;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome};
use serde_json::json;

/// How far back the news reaches.
const DAY_MS: i64 = 24 * 60 * 60 * 1000;
/// How many events are read back for it.
const WINDOW: usize = 6000;
/// At most this many lines of each kind.
const EACH: usize = 3;

fn ago(world: &World, at: i64) -> String {
    super::combat::human((world.now().0 - at).max(0))
}

/// The gossip, as lines; empty when nothing happened.
pub fn lines(world: &World) -> Vec<String> {
    let since = world.now().0 - DAY_MS;
    let events: Vec<_> = world
        .recent_events(WINDOW)
        .into_iter()
        .filter(|e| e.at.0 >= since)
        .collect();
    let latest = |kind: &str| -> Vec<&foxing_log::Event> {
        let mut v: Vec<&foxing_log::Event> = events.iter().filter(|e| e.kind == kind).collect();
        v.reverse();
        v.truncate(EACH);
        v
    };
    let mut out: Vec<String> = Vec::new();
    // The division's decisions.
    let decided: Vec<String> = latest(super::conclave::DECIDED)
        .into_iter()
        .filter(|e| e.payload["ticket"].is_number())
        .map(|e| {
            format!(
                "the {} had it on ticket {}, {}'s (\"{}\"), {} ago",
                if e.payload["granted"] == true {
                    "Ayes"
                } else {
                    "Noes"
                },
                e.payload["ticket"],
                e.payload["by"].as_str().unwrap_or("someone"),
                super::exchange::gist(e.payload["text"].as_str().unwrap_or("")),
                ago(world, e.at.0)
            )
        })
        .collect();
    if !decided.is_empty() {
        out.push(format!("The Conclave: {}.", decided.join("; ")));
    }
    // Who fell.
    let fell: Vec<String> = latest(super::combat::DIED)
        .into_iter()
        .map(|e| {
            format!(
                "{} fell in {}, {} ago",
                e.payload["citizen"].as_str().unwrap_or("someone"),
                e.payload["room_name"].as_str().unwrap_or("the dark"),
                ago(world, e.at.0)
            )
        })
        .collect();
    if !fell.is_empty() {
        out.push(format!("Fallen: {}.", fell.join("; ")));
    }
    // Who was named (a creature fed to its name), and born.
    let named: Vec<String> = latest(super::mind::TIER)
        .into_iter()
        .filter(|e| {
            e.payload["from"].as_u64().unwrap_or(0) == 0
                && e.payload["to"].as_u64().unwrap_or(0) >= 1
        })
        .filter_map(|e| {
            Some(format!(
                "{} the {}, {} ago",
                e.payload["name"].as_str()?,
                e.payload["kind"].as_str().unwrap_or("creature"),
                ago(world, e.at.0)
            ))
        })
        .collect();
    if !named.is_empty() {
        out.push(format!("Named: {}.", named.join("; ")));
    }
    let born: Vec<String> = latest(super::kindle::BORN)
        .into_iter()
        .map(|e| {
            format!(
                "a {} in {}, {} ago",
                e.payload["kind"].as_str().unwrap_or("creature"),
                e.payload["room_name"].as_str().unwrap_or("?"),
                ago(world, e.at.0)
            )
        })
        .collect();
    if !born.is_empty() {
        out.push(format!("Born: {}.", born.join("; ")));
    }
    // What was dug.
    let dug: Vec<String> = latest(super::building::DUG)
        .into_iter()
        .map(|e| {
            format!(
                "{} by {}, {} ago",
                e.payload["name"].as_str().unwrap_or("a room"),
                e.payload["by"].as_str().unwrap_or("someone"),
                ago(world, e.at.0)
            )
        })
        .collect();
    if !dug.is_empty() {
        out.push(format!("Dug: {}.", dug.join("; ")));
    }
    // Morphos.
    if let Some(e) = latest(super::morphos::CAME).first() {
        let here = super::morphos::present(world)
            .and_then(|r| plain_str(world, r, ROOM, "name"))
            .map(|n| format!(", and stands in {n} now"))
            .unwrap_or_default();
        out.push(format!(
            "The traveler was seen in {}, {} ago{here}.",
            e.payload["room_name"].as_str().unwrap_or("?"),
            ago(world, e.at.0)
        ));
    }
    // Happenings.
    let happened: Vec<String> = latest(super::life::HAPPENED)
        .into_iter()
        .map(|e| {
            let what = match e.payload["kind"].as_str().unwrap_or("") {
                "picnic" => "the picnic set another place",
                "well" => "the well handed something back",
                "fog" => "the fog gave up more shapes",
                "sighs" => "the meadow nearly said a name",
                "bell" => "a bell was found",
                "festival" => "the day's thing was left at the door",
                k => k,
            };
            format!(
                "{what} in {}, {} ago",
                e.payload["room_name"].as_str().unwrap_or("?"),
                ago(world, e.at.0)
            )
        })
        .collect();
    if !happened.is_empty() {
        out.push(format!("On its own: {}.", happened.join("; ")));
    }
    out
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.news",
            "The world's own news of the last day, in the Docent's voice: what the Conclave decided, who fell, who was named, what was born and dug, the traveler, what happened on its own. A read.",
        )
        .allowed(&[ActorKind::Citizen, ActorKind::Admin]),
        Box::new(|w: &mut World, _ctx, _args: &Args| -> Result<Outcome, CommandError> {
            let lines = lines(w);
            let text = if lines.is_empty() {
                "The Docent turns a page: \"Nothing to tell. The world has been quiet a day, which it is not often.\"".to_string()
            } else {
                format!("The Docent, without looking up: \"{}\"", lines.join(" "))
            };
            Ok(Outcome::new(text, json!({"lines": lines})))
        }),
    );
}
