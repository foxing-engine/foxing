//! Speech. `say` is the only way a citizen's words reach another citizen, and it is a
//! DO like any other: `[DO] spacepants MUST say "the fog is thinner north".` The words
//! are recorded verbatim as a `disremembered.say` event and delivered to everyone else
//! in the room as narrative, so a terminal prints them and an agent's REPORT carries
//! them as a `[QUOTE]` with the speaker as its source. Untagged speech never exists.

use super::{internal, me};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{Args, CommandError, CommandSpec, Outcome, ParamType};
use serde_json::json;

/// The event a `say` records: `{from, room, text}`.
pub const SAY: &str = "disremembered.say";

/// Most characters one `say` may carry; the vocabulary's quote limit is in words.
const MAX_SAY_CHARS: usize = 600;

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.say",
            "Say something to everyone in the room. Your words are recorded as you said them and reach the others as a quote with you as its source.",
        )
        .param("text", ParamType::Text, true, "what you say, quoted"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let text = args.str("text").unwrap_or("").trim().to_string();
            if text.is_empty() {
                return Err(CommandError::precondition("Say what? The text is empty."));
            }
            if text.chars().count() > MAX_SAY_CHARS {
                return Err(CommandError::precondition(format!(
                    "That is more than {MAX_SAY_CHARS} characters; say it in parts."
                )));
            }
            let found = me(w, ctx)?;
            let room = w
                .store
                .entity(found.citizen)
                .and_then(|e| e.parent)
                .ok_or_else(|| CommandError::precondition("You are nowhere; log in again."))?;
            w.note(
                Actor::Citizen(ctx.actor.clone()),
                SAY,
                json!({"from": ctx.actor, "room": room, "text": text}),
            )
            .map_err(internal)?;
            let listeners: Vec<String> = w
                .store
                .children(room)
                .filter(|e| e.kind == "citizen" && super::presence::is_here(w, e.id))
                .filter_map(|e| e.owner.clone())
                .filter(|o| o != &ctx.actor)
                .collect();
            super::combat::narrate_many(w, &listeners, format!("{} says, \"{text}\"", ctx.actor));
            // The Conclave's clerks write down everything said, word for word.
            let clerks = super::combat::creatures_in(w, room)
                .iter()
                .filter(|c| c.1 == "engrosser")
                .count();
            let heard = match (listeners.len(), clerks) {
                (0, 0) => {
                    if super::combat::creatures_in(w, room).is_empty() {
                        "No one else is here to hear it.".to_string()
                    } else {
                        "No other citizen is here to hear it.".to_string()
                    }
                }
                (0, 1) => "An engrosser writes it down, word for word.".to_string(),
                (0, _) => "The engrossers write it down, word for word, and pass it along still wet.".to_string(),
                (1, 0) => format!("{} hears you.", listeners[0]),
                (1, _) => format!("{} hears you, and an engrosser writes it down.", listeners[0]),
                (n, 0) => format!("{n} others hear you."),
                (n, _) => format!("{n} others hear you, and an engrosser writes it down."),
            };
            // A named creature spoken to by name answers.
            let answers = super::talk::overhear(w, room, &ctx.actor, &text)?;
            // In the Hush, what you say goes soft and arrives elsewhere on the trail
            // a minute later, as a whisper.
            let hushed = super::trail::hush(w, &ctx.actor, room, &text).map_err(internal)?;
            let mut summary = format!("You say, \"{text}\" {heard}{}", hushed.as_deref().unwrap_or(""));
            for a in &answers {
                summary.push(' ');
                summary.push_str(a);
            }
            Ok(Outcome::new(
                summary,
                json!({"text": text, "heard_by": listeners, "answers": answers}),
            ))
        }),
    );
}
