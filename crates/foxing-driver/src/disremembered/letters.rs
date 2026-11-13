//! The Dead Letter Office (C3).
//!
//! A word left with `tell` for someone who is away waits for them. If the world
//! forgets the name before it is heard, the word has nobody to go to: it is a dead
//! letter, and it is pigeonholed in a room off the Namery. There anyone may read it
//! (`letters`), answer it to whoever wrote it (`reply`), or, when the name has been
//! taken again, carry it to whoever holds the name now (`carry`). Each pays a small
//! memory, once a letter. A letter leaves its pigeonhole when it is answered or
//! handed over, or thirty days after its name was forgotten.

use super::combat::{human, narrate};
use super::talk::{Letter, Tells, TELL};
use super::{internal, me, plain_str, presence, rooms, ROOM};
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{Args, CommandError, CommandSpec, Outcome, ParamType};
use serde_json::json;

/// The room's key.
pub const OFFICE: &str = "dead-letters";
/// What became of a dead letter: `{letter, what, by, text?}`; `what` is `answered`,
/// `taken`, `returned` or `delivered`. Public.
pub const DEAD_LETTER: &str = "disremembered.dead_letter";
/// A letter is kept this long after its name was forgotten.
pub const KEPT_MS: i64 = 30 * 86_400_000;
/// What answering one pays, and what carrying one to its name's new holder pays.
pub const REPLY_XP: f64 = 3.0;
pub const CARRY_XP: f64 = 6.0;
/// How many the pigeonholes show at once, newest first.
const SHOWN: usize = 12;
const MAX_CHARS: usize = 600;

/// Every dead letter with its number (its place in the order they died, from 1).
fn all(world: &World) -> Vec<(usize, Letter)> {
    world.folded("tells", |t: &Tells| {
        t.dead
            .iter()
            .enumerate()
            .map(|(i, l)| (i + 1, l.clone()))
            .collect()
    })
}

/// Whether a letter is still in the Office's keeping: not answered, not handed over,
/// not too old.
fn open(world: &World, l: &Letter) -> bool {
    l.answered.is_none() && l.delivered.is_none() && world.now().0 - l.died < KEPT_MS
}

/// Who has a letter out, if they are still a citizen (a carrier the world forgot
/// carried nothing anywhere: the letter is back in its hole).
fn out_with(world: &World, l: &Letter) -> Option<String> {
    l.taken
        .clone()
        .filter(|by| super::find_citizen(world, by).is_some())
}

/// The letter a citizen is carrying, if any.
fn carried(world: &World, actor: &str) -> Option<(usize, Letter)> {
    all(world)
        .into_iter()
        .find(|(_, l)| open(world, l) && l.taken.as_deref() == Some(actor))
}

/// How many letters are in the pigeonholes now; the Namery's door says it.
pub fn waiting(world: &World) -> usize {
    all(world).iter().filter(|(_, l)| open(world, l)).count()
}

fn clip(text: &str, most: usize) -> String {
    if text.chars().count() <= most {
        text.to_string()
    } else {
        let cut: String = text.chars().take(most).collect();
        format!("{}…", cut.trim_end())
    }
}

fn in_office(world: &World, room: foxing_store::EntityId) -> bool {
    plain_str(world, room, ROOM, "key") == Some(OFFICE)
}

fn not_here() -> CommandError {
    CommandError::precondition(
        "The dead letters are pigeonholed in the Dead Letter Office, one door off the Namery (from the Archive: names, then post). Read, answer and take them there.",
    )
}

fn find(world: &World, no: i64) -> Result<(usize, Letter), CommandError> {
    let found = all(world)
        .into_iter()
        .find(|(n, l)| *n as i64 == no && open(world, l));
    found.ok_or_else(|| {
        CommandError::precondition(format!(
            "No letter numbered {no} is in the pigeonholes (answered, handed over, or never here). `letters` lists what is."
        ))
    })
}

fn note(
    world: &mut World,
    actor: &str,
    no: usize,
    what: &str,
    text: Option<&str>,
) -> Result<(), CommandError> {
    world
        .note(
            Actor::Citizen(actor.to_string()),
            DEAD_LETTER,
            json!({"letter": no, "what": what, "by": actor, "text": text}),
        )
        .map(|_| ())
        .map_err(internal)
}

fn pay(world: &mut World, actor: &str, source: &str, xp: f64) -> Result<(), CommandError> {
    let f = super::find_citizen(world, actor).ok_or_else(|| internal("no such citizen"))?;
    let curves = super::Curves::with_half_life(foxing_time::Duration::seconds(3 * 3600));
    super::memory::earn_chunk(world, actor, f.window, source, xp, 0.5, &curves).map(|_| ())
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.letters",
            "In the Dead Letter Office (off the Namery): the words left for names the world then forgot, unheard. Bare, the pigeonholes; with a number, that letter whole. Elsewhere, the letter you are carrying. `reply` answers one to its writer; `carry` takes one to whoever holds the name now.",
        )
        .param("no", ParamType::Integer, false, "a letter's number"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let room = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let now = w.now().0;
            if !in_office(w, room) {
                return match carried(w, &ctx.actor) {
                    Some((no, l)) => Ok(Outcome::new(
                        format!(
                            "You carry dead letter no. {no}, left for {} by {} {} ago: \"{}\" Stand with whoever holds that name now and `carry` hands it over.",
                            l.to, l.word.from, human(now - l.word.at), l.word.text
                        ),
                        json!({"carrying": no, "for": l.to, "from": l.word.from, "text": l.word.text}),
                    )),
                    None => Err(not_here()),
                };
            }
            if let Some(no) = args.int("no") {
                let (no, l) = find(w, no)?;
                let held = super::find_citizen(w, &l.to_actor).is_some();
                let status = match (out_with(w, &l), held) {
                    (Some(by), _) => format!(" It is out of its pigeonhole: {by} is carrying it to whoever holds the name now."),
                    (None, true) => format!(" The name is held again, by someone who has never read this: `carry {no}` takes it to them."),
                    (None, false) => " Nobody holds the name now.".to_string(),
                };
                return Ok(Outcome::new(
                    format!(
                        "Letter no. {no}. For {}, from {}, left {} ago; the world forgot {} {} ago, and it was never heard: \"{}\"{status} `reply {no} \"…\"` answers it to {}.",
                        l.to, l.word.from, human(now - l.word.at), l.to, human(now - l.died), l.word.text, l.word.from
                    ),
                    json!({"letter": no, "for": l.to, "from": l.word.from, "text": l.word.text, "left_at": l.word.at, "forgotten_at": l.died, "name_held": held, "out_with": out_with(w, &l)}),
                ));
            }
            let mut open_now: Vec<(usize, Letter)> = all(w).into_iter().filter(|(_, l)| open(w, l)).collect();
            open_now.reverse();
            if open_now.is_empty() {
                return Ok(Outcome::new(
                    "Every pigeonhole is empty: no word is waiting on a name the world forgot. A word left with `tell` for someone away comes here if their name is forgotten before they hear it.",
                    json!({"letters": []}),
                ));
            }
            let total = open_now.len();
            let rows: Vec<String> = open_now
                .iter()
                .take(SHOWN)
                .map(|(no, l)| {
                    let mark = if out_with(w, l).is_some() {
                        " (out, being carried)"
                    } else if super::find_citizen(w, &l.to_actor).is_some() {
                        " (the name is held again)"
                    } else {
                        ""
                    };
                    format!(
                        "  no. {no}  for {}, from {}, {} ago{mark}: \"{}\"",
                        l.to, l.word.from, human(now - l.word.at), clip(&l.word.text, 90)
                    )
                })
                .collect();
            let more = if total > SHOWN {
                format!(" The newest {SHOWN} are shown.")
            } else {
                String::new()
            };
            Ok(Outcome::new(
                format!(
                    "The pigeonholes hold {total} dead letter{}: words left for names the world forgot before they were heard.{more}\n{}\n`letters <no>` reads one whole; `reply <no> \"…\"` answers it to whoever wrote it; `carry <no>` takes one whose name is held again to its new holder.",
                    if total == 1 { "" } else { "s" },
                    rows.join("\n")
                ),
                json!({"letters": open_now.iter().take(SHOWN).map(|(no, l)| json!({"letter": no, "for": l.to, "from": l.word.from, "text": l.word.text, "left_at": l.word.at, "name_held": super::find_citizen(w, &l.to_actor).is_some()})).collect::<Vec<_>>(), "total": total}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.reply",
            "In the Dead Letter Office: answer a dead letter. Your words go to whoever wrote it, with the letter they answer; the letter leaves its pigeonhole, and you remember having answered it.",
        )
        .param("no", ParamType::Integer, true, "the letter's number, from `letters`")
        .param("text", ParamType::Text, true, "what you answer, quoted"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let room = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            if !in_office(w, room) {
                return Err(not_here());
            }
            let text = args.str("text").unwrap_or("").trim().to_string();
            if text.is_empty() {
                return Err(CommandError::precondition("Answer it with what? The text is empty."));
            }
            if text.chars().count() > MAX_CHARS {
                return Err(CommandError::precondition(format!(
                    "That is more than {MAX_CHARS} characters; a pigeonhole is small."
                )));
            }
            let (no, l) = find(w, args.int("no").unwrap_or(0))?;
            if l.word.from.eq_ignore_ascii_case(&ctx.actor) {
                return Err(CommandError::precondition(
                    "That one is in your own hand. It waits for someone else to answer it.",
                ));
            }
            if let Some(by) = out_with(w, &l) {
                return Err(CommandError::precondition(format!(
                    "Letter no. {no} is out of its pigeonhole: {by} is carrying it to whoever holds the name now."
                )));
            }
            note(w, &ctx.actor, no, "answered", Some(&text))?;
            let writer = super::canonical_name(w, &l.word.from);
            let how = match writer.as_deref().and_then(|n| super::find_citizen(w, n).map(|c| (n.to_string(), c))) {
                Some((name, them)) => {
                    w.note(
                        Actor::Citizen(ctx.actor.clone()),
                        TELL,
                        json!({"from": ctx.actor, "to": name, "room": room.0, "text": text, "re": {"for": l.to, "text": l.word.text, "letter": no}}),
                    )
                    .map_err(internal)?;
                    if presence::is_here(w, them.citizen) {
                        narrate(
                            w,
                            &name,
                            format!(
                                "{} answered a word of yours from the Dead Letter Office (you left it for {}, who never heard it: \"{}\"): \"{text}\"",
                                ctx.actor, l.to, l.word.text
                            ),
                        );
                        super::talk::mark_heard(w, them.citizen)?;
                        format!("{name} is in the world, and has it now")
                    } else {
                        format!("{name} is away, and hears it on their return")
                    }
                }
                None => format!(
                    "the world has forgotten {} too, so the answer is filed with the letter and both are put away",
                    l.word.from
                ),
            };
            pay(w, &ctx.actor, &format!("answering a dead letter for {}", l.to), REPLY_XP)?;
            Ok(Outcome::new(
                format!(
                    "You answer letter no. {no}, the word {} left for {}: \"{text}\" It goes to its writer: {how}. The pigeonhole is empty, and you remember having answered ({REPLY_XP:.0}).",
                    l.word.from, l.to
                ),
                json!({"letter": no, "answered": true, "to": l.word.from, "writer_known": writer.is_some(), "earned": REPLY_XP}),
            ))
        }),
    );

    world.register(
        CommandSpec::new(
            "disremembered.carry",
            "With a number, in the Dead Letter Office: take a dead letter whose name is held again, to carry to its new holder (one at a time). Bare, standing with that holder: hand it over; they read it and you remember the errand.",
        )
        .param("no", ParamType::Integer, false, "the letter's number, from `letters`"),
        Box::new(|w: &mut World, ctx, args: &Args| {
            let f = me(w, ctx)?;
            let room = rooms::room_of(w, f.citizen).ok_or_else(|| internal("citizen has no room"))?;
            let now = w.now().0;
            let Some(no) = args.int("no") else {
                let Some((no, l)) = carried(w, &ctx.actor) else {
                    return Err(CommandError::precondition(
                        "You are carrying no letter. `carry <no>` in the Dead Letter Office takes one whose name is held again.",
                    ));
                };
                let Some(them) = super::find_citizen(w, &l.to_actor) else {
                    note(w, &ctx.actor, no, "returned", None)?;
                    return Ok(Outcome::new(
                        format!("The world has forgotten {} again; nobody holds the name. Letter no. {no} goes back to its pigeonhole.", l.to),
                        json!({"letter": no, "returned": true}),
                    ));
                };
                let holder = super::canonical_name(w, &l.to_actor).unwrap_or_else(|| l.to.clone());
                if rooms::room_of(w, them.citizen) != Some(room) || !presence::is_here(w, them.citizen) {
                    return Err(CommandError::precondition(format!(
                        "{holder} is not here. Letter no. {no} stays with you until you stand with them (`where {holder}` if they are in the world)."
                    )));
                }
                note(w, &ctx.actor, no, "delivered", None)?;
                narrate(
                    w,
                    &holder,
                    format!(
                        "{} hands you a dead letter from the Office. It was left for the name you hold, before it was yours, by {} {} ago: \"{}\"",
                        ctx.actor, l.word.from, human(now - l.word.at), l.word.text
                    ),
                );
                pay(w, &ctx.actor, &format!("carrying a dead letter to {holder}"), CARRY_XP)?;
                return Ok(Outcome::new(
                    format!(
                        "You hand letter no. {no} to {holder}: the word {} left for that name {} ago. They read it. You remember the errand ({CARRY_XP:.0}).",
                        l.word.from, human(now - l.word.at)
                    ),
                    json!({"letter": no, "delivered": true, "to": holder, "earned": CARRY_XP}),
                ));
            };
            if !in_office(w, room) {
                return Err(not_here());
            }
            let (no, l) = find(w, no)?;
            if let Some(by) = out_with(w, &l) {
                return Err(CommandError::precondition(if by == ctx.actor {
                    format!("You already have letter no. {no}.")
                } else {
                    format!("Letter no. {no} is out already: {by} is carrying it.")
                }));
            }
            if super::find_citizen(w, &l.to_actor).is_none() {
                return Err(CommandError::precondition(format!(
                    "Nobody holds the name {} now, so there is nobody to carry it to. `reply {no} \"…\"` answers it to {} instead.",
                    l.to, l.word.from
                )));
            }
            if l.to_actor.eq_ignore_ascii_case(&ctx.actor) {
                // The name's new holder came for it themselves.
                note(w, &ctx.actor, no, "delivered", None)?;
                return Ok(Outcome::new(
                    format!(
                        "Letter no. {no} is for the name you hold, from before it was yours. {} left it {} ago: \"{}\" You keep it; the pigeonhole is empty.",
                        l.word.from, human(now - l.word.at), l.word.text
                    ),
                    json!({"letter": no, "delivered": true, "to": ctx.actor, "text": l.word.text}),
                ));
            }
            if let Some((mine, _)) = carried(w, &ctx.actor) {
                return Err(CommandError::precondition(format!(
                    "You are carrying letter no. {mine} already; one at a time."
                )));
            }
            note(w, &ctx.actor, no, "taken", None)?;
            let holder = super::canonical_name(w, &l.to_actor).unwrap_or_else(|| l.to.clone());
            Ok(Outcome::new(
                format!(
                    "You take letter no. {no} out of its pigeonhole: \"{}\" It is for whoever holds the name {holder} now. Stand with them and `carry` hands it over.",
                    clip(&l.word.text, 120)
                ),
                json!({"letter": no, "taken": true, "for": holder}),
            ))
        }),
    );
}
