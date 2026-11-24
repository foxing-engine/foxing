//! D33: the second and third words by meaning.
//!
//! With the switch on (`sense meanings`, the operator's), a named creature's second and
//! third rungs hold a meaning, not a word ("something lent and never returned"), and
//! a story finds it by being about that, in any words. The first rung stays a word.
//!
//! Nothing here asks a model. A story told to a creature at a meaning rung is paid
//! as a story at once, and put on the Docent's record to be weighed; the tool outside
//! (`tools/docent.py`, the same that answers questions and sits on the Court's bench)
//! reads `GET /v0/docent`, asks a small model one question under a fixed rubric (is
//! this story about this meaning: 0 to 3, and one line in the creature's voice), and
//! says the result with the operator's verb `weigh`. The verdict reaches the teller
//! the way the Docent's answers do, a little later, wherever they stand. A 3 finds
//! the meaning: the bonus is paid then, the strike is public, and the creature keeps
//! the next one.
//!
//! What a late judge costs, and how each is met:
//! - It cannot be predicted, so no `check` and no reading it off the record: a
//!   strike names the meaning found, and the creature has moved on to another.
//! - A story will address the judge. The story goes in as data; only a number and a
//!   short line come out, and the world checks both. A line that says the meaning
//!   short of a 3 is thrown away for a plain one.
//! - Two like stories may be weighed apart, so the score is coarse and the creature
//!   says what it made of it.
//! - When the tool is not listening the rung sleeps: the story is told so, is paid as
//!   a story, and nothing guessable stands in. A verdict that never comes is said to
//!   have never come.
//! - One story an hour from a name is weighed by a creature (it hears one an hour).

use super::mind::{self, MIND};
use super::Curves;
use crate::world::World;
use foxing_log::Actor;
use foxing_registry::{ActorKind, Args, CommandError, CommandSpec, Outcome, ParamType};
use foxing_store::{EntityId, Field};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A story was weighed against a meaning: `{id, from, creature, name, level, score,
/// model}`; with `meaning` when it was found. Public.
pub const WEIGHED: &str = "disremembered.meaning_weighed";
const JOB_SLEPT: &str = "disremembered.meaning_slept";
/// A verdict not here this long after the story is said to have never come.
pub const SLEPT_AFTER_S: i64 = 240;
/// The tool is listening if it read the record this lately.
const AWAKE_MS: i64 = 90_000;
const KEPT: usize = 60;
/// The creature's line, at most.
pub const LINE_MAX: usize = 200;
/// What goes to the judge of a story, at most.
const STORY_MAX: usize = 1500;

/// A meaning a creature may keep: its key (what the mind stores), the meaning as it
/// is said when found, and three hints, each plainer than the last.
pub struct Theme {
    pub key: &'static str,
    pub meaning: &'static str,
    pub hints: [&'static str; 3],
}

macro_rules! themes {
    ($( $k:literal, $m:literal, $a:literal, $b:literal, $c:literal; )*) => {
        &[ $( Theme { key: $k, meaning: $m, hints: [$a, $b, $c] }, )* ]
    };
}

pub const THEMES: &[Theme] = themes![
    "lent", "something lent and never returned", "Something changed hands.", "It went from one hand to another and was meant to come back.", "Someone is still owed it.";
    "late", "arriving too late for the thing one came for", "The hour mattered.", "Somebody hurried, and it did not help.", "The door was shut, or the bed empty, by the time.";
    "kept-promise", "a promise kept long after anyone would have minded it broken", "Something was said once, long before.", "Nobody was holding them to it any more.", "It was done anyway, years on, for someone who had stopped expecting it.";
    "wrong-name", "being called by a name that is not one's own, and answering", "Somebody turned round.", "Somebody was mistaken about who, and was not corrected.", "It was easier to turn round than to explain.";
    "last-of", "being the last one who remembers how a thing was done", "It is about hands that know.", "When this one goes, the knack goes.", "There is nobody left to ask whether it is being done right.";
    "unsent", "a letter written and never sent", "Somebody found the words.", "It was meant for someone and stayed in a drawer.", "Every word of it was finished; only the going was not.";
    "trade-down", "giving up something good for something worse, and being glad of it", "Two things were weighed against each other.", "The bargain was a bad one and nobody regretted it.", "What was given was worth more than what was got, and that was the point.";
    "borrowed-light", "finding the way by a light that belongs to someone else", "It was dark, for a while.", "They could see, and it was not their doing.", "Somebody else's window, or lamp, or fire, and a road by it.";
    "small-theft", "a small theft that nobody ever noticed", "Only one person knows.", "Something went missing and was never missed.", "The one who took it is the only one who knows.";
    "wait", "waiting for someone who is not coming", "Somebody stayed.", "A place was kept, and stayed kept.", "The one who waits half knows, and waits.";
    "mended", "a thing mended so often that nothing of the first one is left", "It was looked after.", "It is the same one, and none of it is.", "Each part was replaced in turn, and nobody ever called it new.";
    "inherit", "inheriting a thing one does not know how to use", "It came from before them.", "It came down to them, and they could only keep it.", "The one who knew what it was for is not there to say.";
    "first-lie", "a kind lie told to spare someone, and believed", "Something was said out of kindness.", "It was not true, and it was the gentlest thing said that day.", "They never found out, and were happier.";
    "return", "coming home and finding it smaller than it was", "Somebody went back.", "The place had not changed, and it did not fit.", "Everything was where it had been left, and lower, and nearer together.";
    "overheard", "learning something important by overhearing what was not meant for one", "Somebody was listening.", "It was said to somebody else.", "A door was not quite shut, and it changed things.";
    "shared-secret", "a secret that two people keep by never speaking of it", "There are two in it.", "Both of them know, and neither has said.", "It holds only as long as nobody mentions it.";
    "wrong-turn", "a wrong turning that turned out better than the right one", "It did not go as planned.", "They were lost, and it was the best thing that happened.", "The road they meant to take would have been worse.";
    "counting", "counting something over and over because one is afraid of the number", "There is a number in it.", "The sum does not change, and is taken again.", "As long as it is being counted it has not been lost.";
    "gift-refused", "a gift that was refused", "Somebody offered.", "It was held out, and not taken.", "The giver carried it home again.";
    "namesake", "being named after someone one never met", "It is about what one is called.", "The name was somebody's before, and they are only a story.", "People look for the other one in the face.";
    "watch", "keeping watch over something that no longer needs it", "Somebody did not leave their place.", "The danger passed long ago, and the post was never stood down.", "It is guarded out of habit, or love.";
    "forgiven", "being forgiven by someone who never knew what for", "Something was let go.", "The pardon was real, and the offense was never told.", "They said it was nothing, and did not know how much it was.";
    "unfinished", "work left unfinished on purpose", "It was nearly done.", "It could have been done, and was set down one step short.", "Finishing it would have meant it was over.";
    "echo", "hearing one's own words come back from someone else, changed", "Something was said twice.", "They said it once, and it returned a stranger.", "Somebody repeated it, and it was not what was meant any more.";
    "spare-key", "keeping a key to a door one will never open again", "Something small was kept.", "It still fits, and it is never tried.", "The house is someone else's now, and the thing that opens it is in a pocket.";
    "empty-chair", "setting a place at the table for someone who is gone", "It happens at a meal.", "There is one more than there are people.", "The plate is laid out every evening and nobody sits to it.";
    "taught", "being outdone by someone one taught", "There are an elder and a younger in it.", "The lesson took better than was meant.", "The pupil does it finer now, and the teacher watches.";
    "false-map", "following directions that were wrong, given in good faith", "Somebody asked the way.", "They were told kindly, and told wrong.", "The one who pointed believed it, and the road went nowhere.";
    "stayed-night", "a stranger taken in for one night who is remembered for years", "Somebody knocked.", "They were gone by morning, and not from the talk of the house.", "One night under the roof, and the family still speaks of them.";
    "last-time", "doing something for the last time without knowing it was the last", "It was an ordinary day.", "Nobody marked it, because nobody knew.", "They would have done it more slowly if they had known there would be no other.";
    "held-back", "a skill kept hidden so as not to shame someone else", "Somebody held back.", "They could have done it better, and did not let on.", "They lost on purpose, or stayed quiet, so another could be the one who was good at it.";
    "wrong-grave", "mourning at the wrong grave", "There is grief in it.", "The grief was true, and the place was not.", "The flowers went to a stranger for years.";
    "saved-for", "saving something for a special day that never came", "Something was put away.", "It was too good for an ordinary day.", "It was found afterwards, unused, still in its paper.";
    "dream-told", "telling a dream to someone and finding they had it too", "It began in sleep.", "Somebody said: I know that place.", "Two people who had never spoken of it had walked the same one at night.";
    "credit", "letting someone else take the credit for one's work", "Something was made.", "The praise went to another, and the maker said nothing.", "They watched somebody thanked for it, and let it stand.";
    "tree-planted", "planting something one will not live to see grown", "Somebody dug.", "It was for later than them.", "The shade will fall on people they will never meet.";
    "wrong-house", "walking into the wrong house and being made welcome", "A door was opened.", "It was a mistake, and nobody said so.", "They were given supper before anyone asked who they were.";
    "carried", "carrying something heavy a long way for someone who did not need it", "It was a long way.", "The trouble taken was not wanted.", "They arrived with it on their back and were told there had been no need.";
    "name-forgotten", "forgetting the name of someone one loved", "Somebody is remembered.", "The face is there, and the voice, and one thing is not.", "They can see them plainly and cannot say what they were called.";
    "two-versions", "two people remembering the same day differently, and both sure", "There was a day.", "Each tells it, and the tellings do not meet.", "Neither is lying, and it cannot have been both.";
    "outgrown", "keeping a child's thing long after the child is grown", "Something small is in a cupboard.", "The one it fitted is taller than the door now.", "A shoe, or a toy, kept by the parent and forgotten by the child.";
    "apology-late", "an apology made to someone who can no longer hear it", "Somebody was sorry.", "It was said aloud, to nobody.", "The words came years after the one they were for was gone.";
    "weather", "a whole life steered by one day's weather", "It rained, or it did not.", "Had the sky been otherwise, so would everything after.", "They stayed in because of the storm, and that was where it all began.";
    "unsung", "a song that everyone knows and no one will sing", "There is a tune.", "It is known by heart and kept behind the teeth.", "It belonged to a time or a person, and singing it would bring them back.";
    "stand-in", "taking someone's place and being preferred to them", "One came instead of another.", "It was only meant to be for a while.", "When the first one returned, nobody wanted the change undone.";
    "fear-small", "being afraid of a thing for years and finding it was small", "Somebody finally looked.", "It had been larger in the dark.", "The dreaded thing, met at last, was hardly anything.";
    "unthanked", "a kindness from a stranger one never got to thank", "Someone helped.", "They were gone before anything could be said.", "The thanks are owed to a face with no name, and cannot be given.";
    "other-road", "wondering about the life one would have had by choosing otherwise", "There was a choice.", "The other one is still thought about.", "Somewhere a different self took the other road, and is imagined often.";
    "burned", "destroying something precious so that no one else can have it", "Something was ended on purpose.", "Better gone than in another's hands.", "They put it in the fire themselves.";
    "false-brave", "being thought brave when one was only unable to run", "Somebody stood their ground.", "It looked like courage from outside.", "They were praised for staying, and had only been too frightened to move.";
    "nickname", "a nickname whose reason everyone has forgotten", "Somebody is called something.", "Nobody remembers why.", "The joke it came from is lost and the name stuck.";
    "came-back-better", "something borrowed that came back better than it went", "Something came back.", "It was not as it had been when it left.", "It was returned cleaned, or mended, or filled.";
    "smell", "a smell that brings back a whole lost place", "It came in on the air.", "For a moment they were somewhere else entirely.", "One breath of it and the kitchen, or the street, was all there again.";
    "older-than", "living to be older than one's parent ever was", "There is a birthday in it.", "They passed a mark that someone else never reached.", "From this year on they have had more years than their mother, or father, ever got.";
    "strangers-picture", "a picture of strangers that someone keeps as if they were family", "Something is kept on a shelf.", "Nobody in it is known.", "The faces were bought, or found, and are loved anyway.";
    "outlast", "an object outlasting everyone who cared about it", "It is still there.", "Those it mattered to are not.", "It sits in a shop or a ditch, and no one living knows what it meant.";
    "found-after", "losing a thing and finding it only after getting another", "Something was looked for.", "It turned up when it was no longer needed.", "Now there are two, and the first was behind the dresser all along.";
    "go-between", "softening someone's words while carrying them to another", "Somebody stood between two.", "What arrived was not quite what was sent.", "The messenger left out the cruel part, and peace was kept.";
    "rival", "missing a rival once they are gone", "There were two who did not get on.", "Winning was no good without them.", "The enemy left, or died, and the days were flatter for it.";
    "practice", "practicing long for a moment that passes in an instant", "There was a great deal of getting ready.", "It was over almost before it began.", "Months of rehearsal for a thing that took one breath.";
    "wrong-praise", "being loved for the thing one likes least in oneself", "Somebody was admired.", "It was not for what they would have chosen.", "The part of themselves they would hide is the part others came for.";
    "on-the-way", "being more at home on the way than at either end", "There is travelling in it.", "Arriving was the lesser part.", "They were happiest between the two places, and at neither.";
    "token", "keeping a worthless thing because of who gave it", "It is worth nothing.", "It would not be sold for anything.", "A pebble, or a button, kept for the hand it came from.";
    "storm-truce", "enemies sheltering together and parting without a word", "The weather drove them in.", "They would not have shared a roof by choice.", "For one night the quarrel was set down, and picked up again at the door.";
    "voice", "hearing a dead person's voice in a living one", "Somebody spoke.", "For a moment it was another who was speaking.", "The grandchild laughed, and it was the grandfather's laugh exactly.";
    "old-letter", "reading a letter meant for someone long dead", "Something was opened.", "It had waited a long time for other eyes.", "The one it was written to is long buried, and a stranger read it.";
    "guide-lost", "leading others with confidence while being lost oneself", "Others followed.", "The one in front did not know either.", "They walked ahead as if sure, and prayed for a landmark.";
    "unseen-work", "careful work done where no one will ever see it", "It was done well.", "No eye will fall on it.", "The back of the carving is as finished as the front.";
    "wrong-words", "singing the wrong words for years and preferring them", "Something was misheard.", "The mistake was better.", "When they learned the true words they kept their own.";
    "threshold", "standing at a door and not knocking", "Somebody came a long way.", "The last step was not taken.", "They stood outside with a hand raised, and went away again.";
    "spared", "being spared by chance where others were not, and never knowing why", "Something terrible happened to many.", "One was left out of it.", "They were away that day, or late, and have asked why ever since.";
    "gone-wild", "a tended place left to go wild, and lovelier for it", "Somebody stopped.", "What came after was not planned.", "The beds went to seed and to weeds, and people stop at the wall to look.";
    "cost", "learning what something cost only long after it was given", "It was taken lightly.", "The giver had gone without.", "Years later they heard what had been sold to pay for it.";
    "newcomer", "a stranger knowing a place better than those born there", "Somebody came from elsewhere.", "They saw what the others walked past.", "The newcomer could name every lane and the natives could not.";
    "true-joke", "a true thing said as a joke so that it could be said", "People laughed.", "It was meant.", "It was the only way to say it, and nobody heard it as true.";
    "early", "getting up before everyone to have the world alone", "It was very early.", "Nobody else was about, and that was the point.", "An hour taken before the house woke, kept to oneself and kept daily.";
    "already-knew", "confessing something and finding the other already knew", "Somebody finally said it.", "It was not news.", "They had dreaded the telling for years, and were answered: I know.";
    "appearances", "keeping up appearances for neighbors who are doing the same", "It is about how things look.", "Both sides of the fence are acting.", "Each house thinks the other is doing well, and neither is.";
    "animal-knows", "an animal knowing something before the people do", "There is a creature in it.", "It was uneasy first.", "The dog would not go in, or the birds left, a day before anyone understood.";
    "shortcut", "a shortcut that took longer", "Somebody thought to save time.", "The nearer way was not.", "They would have been there hours sooner by the long road.";
    "last-coin", "spending one's last coin on something that is not needed", "There was very little left.", "It went on something that feeds nobody.", "The last of the money bought flowers, or a ribbon, or music.";
    "name-carved", "finding one's own name written somewhere by a hand one does not know", "Something was read.", "It was the reader's own, and they had not put it there.", "Cut in a bench or written in a margin, by whom they never learned.";
    "ferry", "helping others across and never crossing oneself", "Somebody takes people over.", "They always come back to the same side.", "Everyone they carry goes on, and they turn the boat round.";
    "advice", "understanding a parent's advice only at the parent's age", "Something was said long ago.", "It made no sense then.", "The words came clear decades on, standing where the speaker had stood.";
    "unbelieved", "a warning that was right and was not believed", "Somebody said so beforehand.", "They were not listened to.", "Afterwards nobody remembered who had told them.";
    "mask", "playing a part so long it stops being a part", "It began as pretending.", "There came a day when there was nothing underneath it.", "They put the manner on for work, or safety, and could not find the way out of it.";
    "winter-store", "sharing a store that was barely enough, and coming through anyway", "There was not enough.", "It was divided all the same.", "They gave from what would hardly do for themselves, and saw the spring.";
    "runaway", "running away and finding nobody noticed", "Somebody left.", "They were not looked for.", "They came back by supper and it had not been remarked.";
    "old-quarrel", "a quarrel kept up after both have forgotten its cause", "Two do not speak.", "Neither could say what began it.", "It is kept out of habit, and pride, and the reason is long gone.";
    "handwriting", "knowing someone only by their handwriting", "There are notes, or letters.", "The two have never met.", "They would know the hand anywhere, and not the face.";
    "enemy-kindness", "a kindness done by the one who had least cause to", "It was not expected from that quarter.", "The one who did it had no cause to.", "The one who liked them least was the one who quietly helped.";
    "recipe", "making a dish from memory that never tastes as it did", "Somebody is cooking.", "It is nearly right, every time.", "Whatever the missing thing was went with the one who used to make it.";
    "summit", "reaching the goal and feeling nothing", "Something long wanted was got.", "The joy did not come with it.", "They stood at the top at last and wondered what to do with the afternoon.";
    "stray", "an animal that chose its person, not the other way about", "It turned up one day.", "Nobody went looking for it.", "It walked in, lay down, and was theirs from then on.";
    "other-speech", "rehearsing what to say and saying something else", "The words were ready.", "Other ones came out.", "The practiced speech went unsaid, and what was said instead was truer.";
    "light-left", "leaving a light on for oneself in an empty house", "A lamp is burning.", "Nobody is in to need it.", "They leave it lit when they go out, so as to come home to something.";
];

fn theme(key: &str) -> Option<&'static Theme> {
    THEMES.iter().find(|t| t.key == key)
}

fn hash(s: &str) -> usize {
    s.bytes()
        .fold(5381usize, |h, b| h.wrapping_mul(33) ^ b as usize)
}

/// The meaning for a creature at a rung, on its `turn`th finding (the top rung moves
/// on each time it is found), and not `not`.
pub fn pick(creature: EntityId, level: u8, turn: usize, not: &str) -> &'static Theme {
    let start = hash(&format!("meaning-{}-{level}-{turn}", creature.0));
    (0..THEMES.len())
        .map(|i| &THEMES[(start + i) % THEMES.len()])
        .find(|t| t.key != not)
        .unwrap_or(&THEMES[0])
}

/// Whether the switch is on.
pub fn on(world: &World) -> bool {
    super::docent::field(world, "sense").is_some_and(|v| v == true)
}

fn secret(world: &World, creature: EntityId) -> Option<Value> {
    match world
        .store
        .entity(creature)?
        .components
        .get(MIND)?
        .fields
        .get("secret")?
    {
        Field::Plain(v) if v.is_object() => Some(v.clone()),
        _ => None,
    }
}

/// The meaning `creature` keeps now, if its rung is one.
pub fn kept(world: &World, creature: EntityId) -> Option<&'static Theme> {
    let v = secret(world, creature)?;
    (v["from"] == "meaning")
        .then(|| theme(v["word"].as_str().unwrap_or("")))
        .flatten()
}

// ---- whether the tool is listening ----
#[cfg(not(test))]
static LISTENED: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
#[cfg(test)]
thread_local! { static LISTENED: std::cell::Cell<i64> = const { std::cell::Cell::new(0) }; }

/// The tool read the record just now (`GET /v0/docent`).
pub fn listened(world: &World) {
    let now = world.now().0;
    #[cfg(not(test))]
    LISTENED.store(now, std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    LISTENED.with(|l| l.set(now));
}

fn awake(world: &World) -> bool {
    #[cfg(not(test))]
    let at = LISTENED.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    let at = LISTENED.with(|l| l.get());
    at > 0 && world.now().0 - at < AWAKE_MS
}

fn weighing(world: &World) -> Vec<Value> {
    super::docent::field(world, "weighing")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

/// How long a creature rests from a name that found its third meaning before it
/// keeps another for that name (D54, David 11-24: "c").
pub const REST_MS: i64 = 86_400_000;

/// How many of `creature`'s third meanings `actor` has found.
fn finds(v: &Value, actor: &str) -> usize {
    v["struck"]
        .as_array()
        .map_or(0, |a| a.iter().filter(|s| *s == actor).count())
}

/// If `actor` found this creature's third meaning within the day: how long until it
/// keeps another for them.
fn resting(v: &Value, actor: &str, now: i64) -> Option<i64> {
    if finds(v, actor) == 0 {
        return None;
    }
    let left = REST_MS - (now - v["finds"][actor].as_i64().unwrap_or(0));
    (left > 0).then_some(left)
}

fn hours(ms: i64) -> String {
    let h = (ms + 3_599_999) / 3_600_000;
    if h <= 1 {
        "an hour".to_string()
    } else {
        format!("{h} hours")
    }
}

fn nth(level: u8) -> &'static str {
    ["first", "second", "third"][(level.clamp(1, 3) - 1) as usize]
}

/// After a story has been heard and paid as one: if the creature's rung is a meaning
/// the teller has not found, the story goes to be weighed. Returns what to tell the
/// teller of it (empty when the rung is a word, or the switch is off).
pub fn ask(
    world: &mut World,
    actor: &str,
    creature: EntityId,
    text: &str,
    paid: f64,
) -> Result<String, CommandError> {
    if !on(world) {
        return Ok(String::new());
    }
    let Some(v) = secret(world, creature) else {
        return Ok(String::new());
    };
    let Some(t) = kept(world, creature) else {
        return Ok(String::new());
    };
    let level = v["level"].as_u64().unwrap_or(2) as u8;
    let name = mind::display(world, creature);
    let now = world.now().0;
    if let Some(left) = resting(&v, actor, now) {
        return Ok(format!(
            " You found what it kept under the others, and it rests from you a day after: it keeps another meaning for you in {}, so this one was heard as a story.",
            hours(left)
        ));
    }
    let mut list = weighing(world);
    if let Some(m) = wait(world, actor, creature) {
        return Ok(format!(
            " Its {} word is a meaning, and it weighs one story an hour from a name against it; it has weighed one of yours within the hour, so this one was heard as a story. It weighs your next in {m} minute{}.",
            nth(level),
            if m == 1 { "" } else { "s" }
        ));
    }
    if !awake(world) {
        return Ok(format!(
            " Its {} word is a meaning, not a word, and what weighs a meaning is asleep tonight: the story was heard as a story, and nothing of the word was spent. (`tell {} \"what is your word?\"` says what it will say of it.)",
            nth(level),
            mind::name(world, creature).unwrap_or_default()
        ));
    }
    let id = list
        .iter()
        .filter_map(|j| j["id"].as_u64())
        .max()
        .unwrap_or(0)
        + 1;
    let story: String = text.trim().chars().take(STORY_MAX).collect();
    list.push(json!({"id": id, "from": actor, "creature": creature.0, "name": name, "level": level, "key": t.key,
                     "story": story, "paid": paid, "at": now, "done": Value::Null}));
    if list.len() > KEPT {
        list.drain(..list.len() - KEPT);
    }
    let mut changes = BTreeMap::new();
    changes.insert("weighing".into(), json!(list));
    super::docent::write(world, changes).map_err(CommandError::precondition)?;
    let due = world
        .now()
        .plus(foxing_time::Duration::seconds(SLEPT_AFTER_S));
    world
        .schedule_once(JOB_SLEPT, due, json!({"id": id}), |p| p["id"] == id)
        .map_err(CommandError::precondition)?;
    Ok(format!(
        " Its {} word is a meaning, not a word: it is turning your story over to see whether that is what it was about, and will say what it made of it in a few minutes, wherever you are (`messages` keeps it).",
        nth(level)
    ))
}

/// What the creature says of the meaning it keeps, plainer with each story of this
/// name's it has weighed and found short.
pub fn hint(world: &World, actor: &str, creature: EntityId) -> Option<String> {
    let t = kept(world, creature)?;
    let v = secret(world, creature)?;
    let level = v["level"].as_u64().unwrap_or(2) as u8;
    if let Some(left) = resting(&v, actor, world.now().0) {
        return Some(format!(
            "You found what I keep under the others. What I keep now is for whoever comes next; I will keep another for you in {}, and it will be worth a third less than the last.",
            hours(left)
        ));
    }
    let short = weighing(world)
        .iter()
        .filter(|j| {
            j["from"] == actor
                && j["creature"] == creature.0
                && j["key"] == t.key
                && j["score"].as_u64().is_some_and(|s| s < 3)
        })
        .count();
    let said = t.hints[short.min(2)];
    let sleeping = if awake(world) {
        ""
    } else {
        " (What weighs it is asleep tonight; it will keep.)"
    };
    let again =
        match finds(&v, actor) {
            0 => String::new(),
            n => {
                format!(
            " You have found {} of mine before, so this one is worth less by a third for each.",
            if n == 1 { "one".to_string() } else { n.to_string() }
        )
            }
        };
    let when = match wait(world, actor, creature) {
        Some(m) => format!(
            " I weigh your next in {m} minute{}.",
            if m == 1 { "" } else { "s" }
        ),
        None => String::new(),
    };
    Some(format!(
        "{} What I keep under it now is not a word but a meaning, the {} of three.{again} {said} Tell me a story that is about that, in any words you like, and I will know it; I weigh one story an hour from a name, and say what I made of it a little after.{when}{sleeping}",
        if level == 2 { "The first was found." } else { "Two were found." },
        nth(level),
    ))
}

/// Minutes until `creature` weighs another story of `actor`'s, if it has weighed one
/// within the hour (the breaker of 11-24 spent a story on a creature `wants` called
/// ready: the hour it pays by and the hour it weighs by are two hours).
pub fn wait(world: &World, actor: &str, creature: EntityId) -> Option<i64> {
    let now = world.now().0;
    weighing(world)
        .iter()
        .filter(|j| j["from"] == actor && j["creature"] == creature.0)
        .filter_map(|j| j["at"].as_i64())
        .map(|at| 3_600_000 - (now - at))
        .filter(|ms| *ms > 0)
        .max()
        .map(|ms| (ms + 59_999) / 60_000)
}

/// The switch thrown, or a creature come to a rung: its secret is made to match. A
/// word at the second or third rung becomes a meaning while the switch is on, and a
/// meaning becomes a word again when it is off. Returns whether the secret is a
/// meaning now.
pub fn settle(world: &mut World, creature: EntityId) -> Result<bool, CommandError> {
    let Some(v) = secret(world, creature) else {
        return Ok(false);
    };
    let level = v["level"].as_u64().unwrap_or(1) as u8;
    let is = v["from"] == "meaning";
    let should = on(world) && level >= 2;
    if is == should {
        return Ok(is);
    }
    let struck = v["struck"].clone();
    let turn = struck.as_array().map_or(0, Vec::len);
    let next = if should {
        json!({"word": pick(creature, level, turn, "").key, "level": level, "struck": struck, "from": "meaning", "finds": v["finds"]})
    } else {
        let (word, from) = super::secret::word_again(world, creature, level);
        json!({"word": word, "level": level, "struck": struck, "from": from})
    };
    let mut changes = BTreeMap::new();
    changes.insert("secret".into(), Field::Plain(next));
    mind::set_mind_pub(world, Actor::Driver, creature, changes)?;
    Ok(should)
}

fn line_ok(line: &str, t: &Theme, score: u64) -> bool {
    let n = line.chars().count();
    if !(6..=LINE_MAX).contains(&n)
        || line.contains(['{', '}', '`', '<', '>', '\n'])
        || line.to_lowercase().contains("http")
    {
        return false;
    }
    if score >= 3 {
        return true;
    }
    // Short of a find, the line does not say the meaning: three of its longer words
    // in one line is saying it.
    let low = line.to_lowercase();
    let said = t
        .meaning
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 5)
        .filter(|w| low.contains(&w.to_lowercase()))
        .count();
    said < 3
}

const PLAIN: [&str; 4] = [
    "I turned it over. It is not about what I keep.",
    "There is a thing in it that brushes mine, and no more than brushes.",
    "Near. What I keep is in your story, and your story is about something else.",
    "That is it. That is what I keep.",
];

/// The verdict: said to the teller, and at 3 the meaning is found.
fn weigh(
    world: &mut World,
    by: &str,
    id: u64,
    score: u64,
    line: &str,
    model: &str,
    curves: &Curves,
) -> Result<String, CommandError> {
    if score > 3 {
        return Err(CommandError::precondition("A weighing is 0, 1, 2 or 3."));
    }
    let mut list = weighing(world);
    let Some(j) = list.iter_mut().find(|j| j["id"] == id) else {
        return Err(CommandError::precondition(format!(
            "No story {id} is being weighed."
        )));
    };
    if !j["done"].is_null() && j["done"] != false {
        return Err(CommandError::precondition(format!(
            "Story {id} was weighed already."
        )));
    }
    let late = j["done"] == false;
    let (from, creature, level, key, paid) = (
        j["from"].as_str().unwrap_or("").to_string(),
        EntityId(j["creature"].as_u64().unwrap_or(0)),
        j["level"].as_u64().unwrap_or(2) as u8,
        j["key"].as_str().unwrap_or("").to_string(),
        j["paid"].as_f64().unwrap_or(0.0),
    );
    let gist = super::exchange::gist(j["story"].as_str().unwrap_or(""));
    let Some(t) = theme(&key) else {
        return Err(CommandError::precondition(
            "That meaning is not one the world keeps.",
        ));
    };
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    let line = if line_ok(&line, t, score) {
        line
    } else {
        PLAIN[score as usize].to_string()
    };
    // The creature may have come apart, or another name found this meaning first.
    let still = kept(world, creature).is_some_and(|k| k.key == key)
        && secret(world, creature).is_some_and(|v| resting(&v, &from, world.now().0).is_none());
    let score = if score == 3 && !still { 2 } else { score };
    j["done"] = json!(world.now().0);
    j["score"] = json!(score);
    let name = j["name"].as_str().unwrap_or("it").to_string();
    let mut changes = BTreeMap::new();
    changes.insert("weighing".into(), json!(list));
    super::docent::write(world, changes).map_err(CommandError::precondition)?;
    world
        .cancel(JOB_SLEPT, |p| p["id"] == id)
        .map_err(CommandError::precondition)?;
    let after = if late { ", late" } else { "" };
    let mut found = Value::Null;
    let told = if score == 3 {
        let v = secret(world, creature).unwrap_or(json!({}));
        let mut struck: Vec<String> = v["struck"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let top = level >= super::secret::LEVELS;
        // D54: each third meaning a name finds from one creature is worth a third
        // less than the last.
        let before = if top { finds(&v, &from) } else { 0 };
        let less = (2.0f64 / 3.0).powi(before as i32);
        let mut found_at = if v["finds"].is_object() {
            v["finds"].clone()
        } else {
            json!({})
        };
        if top {
            found_at[&from] = json!(world.now().0);
        }
        struck.push(from.clone());
        let next_level = (level + 1).min(super::secret::LEVELS);
        let next = pick(
            creature,
            next_level,
            if top { struck.len() } else { 0 },
            t.key,
        );
        let mut changes = BTreeMap::new();
        changes.insert(
            "secret".into(),
            Field::Plain(json!({"word": next.key, "level": next_level, "struck": if top { struck.clone() } else { Vec::new() }, "from": "meaning", "finds": if top { found_at } else { Value::Null }})),
        );
        mind::set_mind_pub(world, Actor::Driver, creature, changes)?;
        let bonus = super::secret::BONUS[(level.clamp(1, 3) - 1) as usize];
        let extra = (paid * (bonus - 1.0) * less).round().max(1.0);
        let earned = match super::find_citizen(world, &from) {
            Some(f) => Some(super::memory::earn_chunk(
                world,
                &from,
                f.window,
                &format!("finding what {name} kept under its others"),
                extra,
                0.8,
                curves,
            )?),
            None => None,
        };
        super::bard::did(world, &from, "word");
        let _ = world.note(
            Actor::Citizen(from.clone()),
            super::secret::STRUCK,
            json!({"from": from, "creature": creature.0, "name": name, "word": t.meaning, "level": level, "bonus": bonus, "meaning": true}),
        );
        world.notice(format!("**{from}** told {name} a story about what it kept under its others: *{}* (level {level}).", t.meaning));
        found = json!(t.meaning);
        format!(
            "{name} has turned your story over (\"{gist}\"){after}: \"{line}\" It was about what it kept under the others, the {} of three: {}. {}{}",
            nth(level),
            t.meaning,
            match earned {
                Some(c) if before == 0 => format!("You remember it: chunk-{} worth {extra:.1} XP, {} what the story paid, and vivid.", c.0, match ((bonus - 1.0) * 10.0).round() as i64 { 5 => "half", 10 => "as much again as", _ => "one and a half times" }),
                Some(c) => format!("You remember it: chunk-{} worth {extra:.1} XP, and vivid (a third less for each of its meanings you had found before).", c.0),
                None => String::new(),
            },
            if top { " That was the last of its three. What it keeps now is for whoever comes next; it will keep another for you in a day, worth a third less." } else { " There is another under that." }
        )
    } else {
        format!(
            "{name} has turned your story over (\"{gist}\"){after}: \"{line}\" {} Its hint is plainer for it: `tell {} \"what is your word?\"`.",
            ["Not about what it keeps.", "Far from what it keeps: a likeness on the surface.", "Near what it keeps, and not about it."][score as usize],
            mind::name(world, creature).unwrap_or(name.clone())
        )
    };
    super::combat::narrate(world, &from, told);
    let _ = world.note(
        Actor::Citizen(by.to_string()),
        WEIGHED,
        json!({"id": id, "from": from, "creature": creature.0, "name": name, "level": level, "score": score, "model": model, "meaning": found}),
    );
    Ok(from)
}

/// For the tool: the stories waiting to be weighed, each with the meaning and who
/// keeps it.
pub fn pending(world: &World) -> Vec<Value> {
    let now = world.now().0;
    weighing(world)
        .iter()
        .filter(|j| j["done"].is_null() || j["done"] == false)
        .filter(|j| j["at"].as_i64().is_some_and(|at| now - at < if j["done"] == false { 1_800_000 } else { SLEPT_AFTER_S * 1000 }))
        .filter_map(|j| {
            let t = theme(j["key"].as_str()?)?;
            let creature = EntityId(j["creature"].as_u64()?);
            Some(json!({
                "id": j["id"], "meaning": t.meaning, "story": j["story"],
                "creature": {"name": j["name"], "kind": super::plain_str(world, creature, super::CREATURE, "name")},
            }))
        })
        .collect()
}

pub fn register(world: &mut World, curves: Curves) {
    world.on_job(
        JOB_SLEPT,
        Box::new(|w: &mut World, job| {
            let id = job.payload["id"].as_u64().unwrap_or(0);
            let mut list = weighing(w);
            let Some(j) = list.iter_mut().find(|j| j["id"] == id) else {
                return Ok(());
            };
            if !j["done"].is_null() {
                return Ok(());
            }
            j["done"] = json!(false);
            let (to, name) = (j["from"].as_str().unwrap_or("").to_string(), j["name"].as_str().unwrap_or("It").to_string());
            // Not counted against the hour: the next story may be weighed.
            j["at"] = json!(0);
            let mut changes = BTreeMap::new();
            changes.insert("weighing".into(), json!(list));
            super::docent::write(w, changes)?;
            super::combat::narrate(
                w,
                &to,
                format!("{name} has not weighed your story after all: what weighs a meaning fell asleep over it. Nothing of its word was spent; the next story you tell it may be weighed."),
            );
            Ok(())
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.weigh",
            "Operator. The verdict on a story being weighed against a creature's meaning (GET /v0/docent lists them under weighing): 0 not about it, 1 a surface likeness, 2 near, 3 about it; and one line the creature says, at most 200 characters. Said to the teller wherever they stand; a 3 finds the meaning.",
        )
        .param("id", ParamType::Integer, true, "the story's id")
        .param("score", ParamType::Integer, true, "0 to 3")
        .param("line", ParamType::Text, true, "what the creature says of it, quoted")
        .param("model", ParamType::String, false, "what weighed it, for the record")
        .allowed(&[ActorKind::Admin]),
        Box::new(move |w: &mut World, ctx, args: &Args| {
            let id = args.int("id").unwrap_or(0).max(0) as u64;
            let score = args.int("score").unwrap_or(-1);
            if score < 0 {
                return Err(CommandError::precondition("A weighing is 0, 1, 2 or 3."));
            }
            let line = args.str("line").unwrap_or("").to_string();
            let model = args.str("model").unwrap_or("").trim().to_string();
            let to = weigh(w, &ctx.actor, id, score as u64, &line, &model, &curves)?;
            Ok(Outcome::new(format!("Weighed: story {id}, {score} of 3, said to {to}."), json!({"id": id, "score": score, "to": to})))
        }),
    );
    world.register(
        CommandSpec::new(
            "disremembered.sense",
            "Operator. Whether a named creature's second and third words are meanings, weighed by the Docent's model (`sense meanings`), or words, as the first is (`sense words`). Bare: which it is, whether the model is listening, and what is being weighed. Creatures change over as they are next told a story or asked their word.",
        )
        .param("as", ParamType::String, false, "meanings or words")
        .allowed(&[ActorKind::Admin]),
        Box::new(|w: &mut World, _ctx, args: &Args| {
            // (Said as `meanings` and `words`: the gate reads a bare `on` as a
            // preposition, and `sense on` arrived as `sense`.)
            let to = args.str("as").unwrap_or("").trim().to_lowercase();
            match to.as_str() {
                "meanings" | "words" | "on" | "off" => {
                    let mut changes = BTreeMap::new();
                    changes.insert("sense".into(), json!(to == "meanings" || to == "on"));
                    super::docent::write(w, changes).map_err(CommandError::precondition)?;
                    // Every named creature at a second or third rung changes over now.
                    let named: Vec<EntityId> = w.store.of_kind("creature").map(|e| e.id).filter(|c| mind::name(w, *c).is_some()).collect();
                    for c in named {
                        settle(w, c)?;
                    }
                }
                "" => {}
                _ => return Err(CommandError::precondition("`sense meanings` or `sense words`.")),
            }
            let open = weighing(w).iter().filter(|j| j["done"].is_null()).count();
            Ok(Outcome::new(
                format!(
                    "The second and third words are {}. The model is {}. {open} being weighed, {} themes.",
                    if on(w) { "meanings" } else { "words" },
                    if awake(w) { "listening" } else { "not listening" },
                    THEMES.len()
                ),
                json!({"on": on(w), "awake": awake(w), "open": open}),
            ))
        }),
    );
}

#[cfg(test)]
pub fn wake_for_test(world: &World) {
    listened(world);
}
