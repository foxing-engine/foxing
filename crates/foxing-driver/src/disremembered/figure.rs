//! The four figures (C22; David, 11-25: "a new player design where they can choose
//! one of four avatars", and of which four: "follow the 4 SGLs").
//!
//! Every name was one hooded figure with a tint of its own. A name chooses one of
//! four now, one for each of the world's loops: a hunter, a teller, a keeper, a
//! reader. The choice is how it is drawn, in its own picture and in everyone
//! else's, and nothing else yet: what each begins with in its hand comes with the
//! places on the body (C21). The first choice is made anywhere; a change of figure
//! is made in the Namery, where names are kept.
//!
//! The figure is the frame of the name's sprite (`figure-hunter` and the rest),
//! which the picture already reads, and a field on the citizen for the record.

use super::{plain_str, CITIZEN, ROOM, SPRITE};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field, StoreEvent};
use serde_json::json;
use std::collections::BTreeMap;

/// The four: its word, its loop's verb, and how it is drawn, in a line.
pub const FIGURES: [(&str, &str, &str); 4] = [
    (
        "hunter",
        "hunt",
        "a short hooded cloak in fog green, a short sword and a buckler",
    ),
    (
        "teller",
        "tell",
        "a feathered cap, a ruff and a slashed doublet, a lute across the body",
    ),
    (
        "keeper",
        "keep",
        "a kerchief, rolled sleeves and a long apron, a ring of keys at the hip",
    ),
    (
        "reader",
        "read",
        "a long ink-blue gown, round spectacles and a scholar's cap, a book under the arm",
    ),
];

/// The figure a name has chosen, if it has.
pub fn of(world: &World, citizen: EntityId) -> Option<&'static str> {
    let had = plain_str(world, citizen, CITIZEN, "figure")?;
    FIGURES.iter().map(|f| f.0).find(|f| *f == had)
}

fn set(
    world: &mut World,
    actor: &str,
    citizen: EntityId,
    figure: &str,
) -> Result<(), CommandError> {
    let mut changes = BTreeMap::new();
    changes.insert("figure".to_string(), Field::Plain(json!(figure)));
    super::set_citizen_fields(world, actor, citizen, changes)?;
    let mut fields = world
        .store
        .entity(citizen)
        .and_then(|e| e.components.get(SPRITE))
        .map(|c| c.fields.clone())
        .unwrap_or_else(|| super::sprite_fields("@", "#e8e4da", "citizen"));
    fields.insert(
        "frame".into(),
        Field::Plain(json!(format!("figure-{figure}"))),
    );
    world
        .record(
            Actor::Citizen(actor.to_string()),
            StoreEvent::ComponentSet {
                id: citizen,
                component: SPRITE.into(),
                fields,
            },
        )
        .map(|_| ())
        .map_err(|e| CommandError::internal(e.to_string()))
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.figure",
            "How you are drawn. Bare: the four figures and which is yours. With one (`figure hunter`, `teller`, `keeper` or `reader`): be drawn as it, in your picture and everyone's. One for each of the world's loops; each begins with a thing of its own in the hand, small and of its loop (a short sword, a lute, a ring of keys, a primer); otherwise it is how you look. The first choice is made anywhere; to change it, stand in the Namery.",
        )
        .param("which", ParamType::String, false, "hunter, teller, keeper or reader"),

        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = super::me(w, ctx)?;
            let mine = of(w, f.citizen);
            let Some(which) = args.str("which").map(|s| s.trim().trim_start_matches("the ").to_ascii_lowercase()).filter(|s| !s.is_empty()) else {
                let rows = FIGURES
                    .iter()
                    .map(|(k, l, d)| format!("  {k}{}: {d} (the figure of those who {l}).", if mine == Some(*k) { " (yours)" } else { "" }))
                    .collect::<Vec<_>>()
                    .join("\n");
                return Ok(Outcome::new(
                    format!(
                        "{}\n{rows}\n`figure <one>` chooses{}.",
                        match mine {
                            Some(m) => format!("You are drawn as the {m}. The four figures:"),
                            None => "You are drawn as every name is until it chooses: hooded, in a pale robe. The four figures:".to_string(),
                        },
                        if mine.is_some() { "; a change is made in the Namery" } else { "" }
                    ),
                    json!({"figure": mine, "figures": FIGURES.iter().map(|f| f.0).collect::<Vec<_>>()}),
                ));
            };
            let Some((key, _, drawn)) = FIGURES.iter().find(|f| f.0 == which) else {
                return Err(CommandError::precondition(format!(
                    "There is no figure called {which}. The four: hunter, teller, keeper, reader."
                )));
            };
            if mine == Some(*key) {
                // Its own thing fades as everything does (the Watch of 11-25: "a gift
                // that rots away to nothing, with no way to get it back"): asked for
                // again when it has gone, it is given again.
                return Ok(match super::places::begin(w, &ctx.actor, f.citizen, key)? {
                    Some(name) => Outcome::new(format!("You are the {key} already, and yours had gone: you are given {name} again."), json!({"figure": key, "given": name})),
                    None => Outcome::new(format!("You are the {key} already."), json!({"figure": key})),
                });
            }
            if mine.is_some() {
                let here = super::rooms::room_of(w, f.citizen);
                if here.and_then(|r| plain_str(w, r, ROOM, "key")) != Some("namery") {
                    return Err(CommandError::precondition(format!(
                        "You are the {}, and a figure is changed where names are kept: in the Namery (`travel Namery`).",
                        mine.unwrap_or("one you chose")
                    )));
                }
            }
            set(w, &ctx.actor, f.citizen, key)?;
            // A figure's own thing is the figure's: the old one's is handed in.
            let handed = super::places::hand_in(w, &ctx.actor, f.citizen, key);
            // C21: each figure begins with its own thing, a real one, in the hand.
            let given = super::places::begin(w, &ctx.actor, f.citizen, key)?;
            let with = match given {
                Some(name) => format!(
                    " You are given {name}{}: {}.",
                    if *key == "hunter" { " and a small buckler for the other arm (the first blow of a fight lands a point lighter)" } else { "" },
                    super::places::begins_does(super::places::BEGINS.iter().find(|b| b.0 == *key).map(|b| b.1).unwrap_or("")).unwrap_or("it is yours")
                ),
                None => String::new(),
            };
            Ok(Outcome::new(
                format!(
                    "You are the {key} now: {drawn}. It is how you are drawn, in your picture and in everyone's.{}{with}",
                    if handed.is_empty() { String::new() } else { format!(" You hand in {}: it was the other figure's.", handed.join(" and ")) }
                ),
                json!({"figure": key, "given": given}),
            ))
        }),
    );
}
