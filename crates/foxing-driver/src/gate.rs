//! The Gate: where an emancipated agent speaks CLEAR to the world.
//!
//! A message is checked by the same checker the agent can run itself. A passing
//! `REQUEST` compiles each `[DO]` whose actor is the sender into one registry command,
//! run in order, each accepted or refused on its own. A passing `QUERY` compiles each
//! `[ASK]` through the vocabulary's question patterns into an observation. The reply is
//! a `REPORT` in the world's voice: `[FACT]` lines cite event ids, the narrator's prose
//! arrives as `[QUOTE]`, a refusal cites the rule or the precondition that failed, and
//! the world never writes `[INFER]` or `[DO]`. Every reply passes the checker too; a test
//! holds the gate to that.
//!
//! One passing message costs one unit of an hourly budget; a failed one costs a
//! fraction. Legibility is public standing, not a cost.
//!
//! CLEAR is the only way to act in the world. A human's terminal line (`attack wisp`)
//! is compiled into a one-statement REQUEST by the text port, the HTTP command
//! endpoint or the session, and goes through the same gate: checked by the same rules,
//! logged word for word as `gate.message` and `gate.reply`, run by the same code. Only
//! messages that arrive raw at `/v0/gate` are charged to the hourly budget; the budget
//! is the knob on agent traffic, not a tax on typing. Observation is a read, not
//! speech, and stays outside the gate.

use crate::world::World;
use foxing_clear::{merge, parse_raw, CheckResult, Checker, Message, Statement};
use foxing_query::{Limits, QueryError};
use foxing_registry::{ActorKind, CallContext, CommandError};
use foxing_store::Viewer;
use foxing_time::Timestamp;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

const BASE_VOCAB: &str = include_str!("../../../clear/vocab/base.yaml");
const DISREMEMBERED_VOCAB: &str = include_str!("../../../clear/vocab/disremembered.yaml");

/// Disremembered's vocabulary, as the site publishes it: the base vocabulary with the
/// world's own layered on.
pub fn disremembered_vocabulary() -> Value {
    let base = parse_raw(BASE_VOCAB).expect("base vocabulary parses");
    let world = parse_raw(DISREMEMBERED_VOCAB).expect("disremembered vocabulary parses");
    merge(&merge(&foxing_clear::default_vocab(), &base), &world)
}

/// The world's checker, compiled once.
pub fn checker() -> Checker {
    Checker::new(&disremembered_vocabulary(), "disremembered")
}

/// The world's checker, shared: for commands that test a name against the reserved
/// words without compiling the vocabulary again.
pub fn world_checker() -> &'static Checker {
    static C: std::sync::OnceLock<Checker> = std::sync::OnceLock::new();
    C.get_or_init(checker)
}

/// The hourly message budget. A passing message costs one; a failed one costs a
/// fraction, so a broken agent cannot spend the hour on refusals but does pay for them.
pub struct HourBudget {
    pub per_hour: f64,
    pub fail_cost: f64,
    used: HashMap<String, (Timestamp, f64)>,
}

const HOUR_MS: i64 = 3_600_000;

/// The most a message may be, in bytes, by any door: eight kilobytes, which is the
/// checker's own limit at `/v0/check` and twenty statements of a hundred words.
pub const MAX_MESSAGE_BYTES: usize = 8 * 1024;

/// What one compiled line costs over `POST /v0/command`: a quarter of a message, so
/// the hour holds 240 lines against the gate's 1,200 statements, and the gate is the
/// better deal for anything that is not a person typing.
pub const COMPILED_HTTP_COST: f64 = 0.25;
/// What one compiled line costs from the play page's session: a twentieth, which no
/// one types fast enough to notice and a loop on the socket does.
pub const COMPILED_SESSION_COST: f64 = 0.05;

impl HourBudget {
    pub fn new(per_hour: f64, fail_cost: f64) -> Self {
        HourBudget {
            per_hour,
            fail_cost,
            used: HashMap::new(),
        }
    }

    /// Whether `actor` has `cost` left this hour, and if not, when the hour turns.
    pub fn check_cost(&mut self, actor: &str, now: Timestamp, cost: f64) -> Result<(), Timestamp> {
        let entry = self.used.entry(actor.to_string()).or_insert((now, 0.0));
        if now.since(entry.0).0 >= HOUR_MS {
            *entry = (now, 0.0);
        }
        if entry.1 + cost > self.per_hour + 1e-9 {
            return Err(Timestamp(entry.0 .0 + HOUR_MS));
        }
        Ok(())
    }

    /// Charge for a message after it was checked.
    pub fn charge(&mut self, actor: &str, now: Timestamp, passed: bool) {
        let cost = if passed { 1.0 } else { self.fail_cost };
        self.charge_cost(actor, now, cost);
    }

    /// Charge `cost` units.
    pub fn charge_cost(&mut self, actor: &str, now: Timestamp, cost: f64) {
        let entry = self.used.entry(actor.to_string()).or_insert((now, 0.0));
        entry.1 += cost;
    }

    /// When `actor`'s rolling hour frees, if anything is spent in it.
    pub fn frees_at(&self, actor: &str, now: Timestamp) -> Option<Timestamp> {
        match self.used.get(actor) {
            Some((start, spent)) if now.since(*start).0 < HOUR_MS && *spent > 0.0 => {
                Some(Timestamp(start.0 + HOUR_MS))
            }
            _ => None,
        }
    }

    /// What `actor` has left this hour.
    pub fn remaining(&self, actor: &str, now: Timestamp) -> f64 {
        match self.used.get(actor) {
            Some((start, spent)) if now.since(*start).0 < HOUR_MS => {
                (self.per_hour - spent).max(0.0)
            }
            _ => self.per_hour,
        }
    }
}

/// One line of a reply.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum Line {
    /// A fact, cited.
    Fact {
        /// One sentence, in CLEAR.
        text: String,
        /// Its source: an event id, a rule id, or a system name.
        src: String,
    },
    /// Carried text: the narrator's prose, or something another actor said.
    Quote {
        /// The text, word for word.
        text: String,
        /// Who said it.
        src: String,
    },
}

/// A reply from the world, before rendering.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub from: String,
    pub to: String,
    pub r#ref: Option<String>,
    pub lines: Vec<Line>,
    /// Indexes into `lines` that are receipts: what ran, which rule, which
    /// precondition. In the CLEAR, not in the prose.
    #[serde(skip)]
    receipts: Vec<usize>,
}

/// The most lines a reply carries; the vocabulary's statement limit less room for a
/// continuation note.
const MAX_REPLY_LINES: usize = 38;
/// In a message of several statements, the most lines one statement's answer takes.
const PER_STATEMENT_LINES: usize = 12;

impl Report {
    pub(crate) fn new(world: &str, to: &str, r#ref: Option<&str>) -> Self {
        Report {
            from: world.into(),
            to: to.into(),
            r#ref: r#ref.map(str::to_string),
            lines: Vec::new(),
            receipts: Vec::new(),
        }
    }

    pub fn fact(&mut self, text: impl Into<String>, src: impl Into<String>) {
        self.lines.push(Line::Fact {
            text: text.into(),
            src: src.into(),
        });
    }

    /// A fact that is a receipt for a human: kept in the CLEAR, left out of the prose.
    pub fn receipt(&mut self, text: impl Into<String>, src: impl Into<String>) {
        self.receipts.push(self.lines.len());
        self.fact(text, src);
    }

    /// A summary with line breaks (the map's grid) becomes one QUOTE per line: a
    /// statement is one line in CLEAR, and the grid flattened to one was unreadable.
    pub fn quote(&mut self, text: impl Into<String>, src: impl Into<String>) {
        self.quote_capped(text, src, usize::MAX);
    }

    /// A quote capped at `cap` lines (R64: a `log 40` first in a batch filled the
    /// reply and the other statements' answers were omitted); the rest is counted.
    pub fn quote_capped(&mut self, text: impl Into<String>, src: impl Into<String>, cap: usize) {
        let text: String = text.into();
        let src: String = src.into();
        let parts: Vec<&str> = text.split('\n').filter(|l| !l.trim().is_empty()).collect();
        if parts.is_empty() {
            self.lines.push(Line::Quote { text, src });
            return;
        }
        let shown = parts.len().min(cap.max(1));
        for part in &parts[..shown] {
            self.lines.push(Line::Quote {
                text: part.to_string(),
                src: src.clone(),
            });
        }
        if parts.len() > shown {
            self.lines.push(Line::Quote {
                text: format!(
                    "… and {} more lines of this; send it alone to read it whole.",
                    parts.len() - shown
                ),
                src,
            });
        }
    }

    /// The reply as CLEAR text.
    pub fn render(&self) -> String {
        let mut out = format!("FROM: {}\nTO: {}\nINTENT: REPORT\n", self.from, self.to);
        if let Some(r) = &self.r#ref {
            out.push_str(&format!("REF: {r}\n"));
        }
        out.push('\n');
        // Over the limit, the receipts go first (what ran, which rule): what the
        // world said is what the sender wrote for. A room-maker's ten digs lost
        // their last two replies to "Statement N ran dig" lines.
        let mut keep: Vec<usize> = (0..self.lines.len()).collect();
        if keep.len() > MAX_REPLY_LINES {
            let over = keep.len() - MAX_REPLY_LINES;
            let mut dropped = 0;
            keep.retain(|i| {
                if dropped < over && self.receipts.contains(i) {
                    dropped += 1;
                    false
                } else {
                    true
                }
            });
        }
        let omitted_receipts = self.lines.len() - keep.len();
        let shown = keep.len().min(MAX_REPLY_LINES);
        for &i in &keep[..shown] {
            match &self.lines[i] {
                Line::Fact { text, src } => {
                    out.push_str(&format!("[FACT] {} [src:{src}]\n", sentence(text)))
                }
                Line::Quote { text, src } => {
                    out.push_str(&format!("[QUOTE] {} [src:{src}]\n", one_line(text)))
                }
            }
        }
        let omitted = omitted_receipts + (keep.len() - shown);
        if omitted > 0 {
            out.push_str(&format!(
                "[FACT] The world omitted {omitted} statements from this reply under the statement limit{}. [src:gate]\n",
                if omitted_receipts > 0 { ", the receipts first" } else { "" }
            ));
        }
        out
    }
}

impl Report {
    /// The reply for a human: quotes as they are, and the world's own statements;
    /// the receipts (event ids, which rule, which precondition) stay in the CLEAR form.
    pub fn prose(&self) -> String {
        let mut out: Vec<String> = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            match line {
                Line::Quote { text, .. } => out.push(text.clone()),
                Line::Fact { text, .. } if !self.receipts.contains(&i) => out.push(sentence(text)),
                Line::Fact { .. } => {}
            }
        }
        out.join("\n")
    }
}

/// Make sure a FACT ends with a period and holds no line breaks.
fn sentence(text: &str) -> String {
    let t = one_line(text);
    if t.ends_with('.') {
        t
    } else {
        format!("{t}.")
    }
}

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Milliseconds since the epoch as `YYYY-MM-DDTHH:MMZ`.
/// `2026-11-03T19:16:42Z`: to the second, for a wait the reader will time.
pub fn iso_second(t: Timestamp) -> String {
    let secs = t.0.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

pub fn iso_minute(t: Timestamp) -> String {
    let secs = t.0.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// What a world binds into the gate: how a question pattern becomes an answer.
pub trait AskBinding {
    /// Answer a matched `[ASK]` for `actor`, appending FACT lines to `report`. Return
    /// `false` if this world has no answer for the pattern.
    fn answer(&self, world: &World, actor: &str, st: &Statement, report: &mut Report) -> bool;
}

/// A DO's parsed arguments (the checker's `args`, v0.4) as positional command
/// arguments: a quoted string and a name as strings, a number as a number (its unit,
/// if any, is dropped; the registry's types carry none).
pub fn compile_arguments(args: &[Value]) -> Vec<Value> {
    args.iter()
        .map(|a| match a.get("kind").and_then(Value::as_str) {
            Some("number") => a.get("value").cloned().unwrap_or(Value::Null),
            _ => Value::String(
                a.get("value")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ),
        })
        .collect()
}

/// The outcome of one message through the gate.
#[derive(Debug, Clone, Serialize)]
pub struct Passage {
    /// The checker's verdict on the message.
    pub verdict: String,
    /// Its Legibility score.
    pub score: u32,
    /// Whether the world acted on it (a passing REQUEST or QUERY).
    pub acted: bool,
    /// The reply, rendered as CLEAR.
    pub reply: String,
    /// The checker's findings on the message.
    pub findings: Vec<foxing_clear::Finding>,
    /// Budget left this hour, in messages.
    pub budget_remaining: f64,
    /// Whether every statement the world ran succeeded (and the message passed).
    pub ok: bool,
    /// The reply as prose for a human: the narrator's quotes and the world's own
    /// statements, without the event receipts.
    pub prose: String,
    /// What each DO the world ran came to.
    pub outcomes: Vec<StatementOutcome>,
    /// The message as the checker read it (for a compiled line, what the line became).
    pub message: String,
}

/// One DO's outcome.
#[derive(Debug, Clone, Serialize)]
pub struct StatementOutcome {
    /// The statement's index in the message.
    pub statement: usize,
    /// The verb it named.
    pub verb: String,
    /// Whether the command ran.
    pub ok: bool,
    /// The command's result, when it ran.
    pub result: Value,
}

/// Where a message came in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    /// Raw CLEAR at `/v0/gate`; budgeted.
    Gate,
    /// Compiled from a terminal line on the text port.
    Text,
    /// Compiled from `POST /v0/command`.
    Http,
    /// Compiled from a session `command` frame.
    Session,
}

impl Via {
    fn name(self) -> &'static str {
        match self {
            Via::Gate => "gate",
            Via::Text => "text",
            Via::Http => "http",
            Via::Session => "session",
        }
    }
}

/// Whether a line is a CLEAR message rather than a command: it starts with the header.
pub fn is_message(line: &str) -> bool {
    line.trim_start().starts_with("FROM:")
}

/// A CLEAR message flattened onto one line by a single-line input gets its line
/// breaks back: before each header field after FROM, and before each statement tag,
/// with the blank line the header ends with.
pub fn reflow(line: &str) -> String {
    let mut out = String::new();
    let mut first_tag = true;
    for word in line.split_whitespace() {
        let header = matches!(word, "TO:" | "INTENT:" | "REF:" | "CC:" | "DATE:");
        let tag = ["[DO]", "[ASK]", "[FACT]", "[INFER", "[QUOTE]"]
            .iter()
            .any(|t| word.starts_with(t));
        if !out.is_empty() {
            if tag && first_tag {
                out.push_str("\n\n");
                first_tag = false;
            } else if header || tag {
                out.push('\n');
            } else {
                out.push(' ');
            }
        }
        out.push_str(word);
    }
    out.push('\n');
    out
}

/// The narrative addressed to `actor` since their last message through the Gate (or
/// since their last arrival, if they have not written before).
fn since_last_message(world: &World, actor: &str) -> Vec<String> {
    let recent = world.recent_events(2000);
    let last = recent
        .iter()
        .rev()
        .find(|e| {
            // After the previous reply: what the previous message itself caused (a
            // creature's answer to your tell) was in that reply already.
            (e.kind == "gate.reply" && e.payload["to"] == actor)
                || (e.kind == crate::world::ARRIVED && e.payload["actor"] == actor)
        })
        .map(|e| e.seq.0)
        .unwrap_or(0);
    recent
        .iter()
        .filter(|e| e.seq.0 > last)
        .filter(|e| e.kind == crate::disremembered::NARRATIVE && e.payload["to"] == actor)
        .filter_map(|e| e.payload["text"].as_str().map(str::to_string))
        .collect()
}

/// Whether `actor` went down since their last message through the Gate: a death on
/// the clock (venom, a strike) lands at the top of the next reply, and the message's
/// statements were written for a room they are no longer in.
fn died_since_last_message(world: &World, actor: &str) -> bool {
    let recent = world.recent_events(2000);
    let last = recent
        .iter()
        .rev()
        .find(|e| e.kind == "gate.reply" && e.payload["to"] == actor)
        .map(|e| e.seq.0)
        .unwrap_or(0);
    recent.iter().any(|e| {
        e.seq.0 > last && e.kind == crate::disremembered::DIED && e.payload["citizen"] == actor
    })
}

/// The verbs a message may still run after a death the sender has not read yet: the
/// ones that only read.
/// Verbs that change nothing: a message of these alone costs no budget.
const READ_VERBS: &[&str] = &[
    "look",
    "chunks",
    "map",
    "time",
    "rest",
    "board",
    "statutes",
    "palace",
    "satchel",
    "log",
    "recipes",
    "help",
    "commands",
    "read",
    "wants",
    "mine",
    "where",
    "check",
    "session",
    "messages",
    "petitions",
];

const AFTER_DEATH: &[&str] = &[
    "look", "chunks", "map", "time", "rest", "board", "statutes", "palace", "satchel", "log",
    "recipes", "help", "commands", "read", "wants", "mine", "where", "check", "session",
    "messages",
];

/// Compile a terminal line (`attack the wisp`) into a REQUEST from `actor` to `world`:
/// one DO, MUST, the line's first word as the verb. What a human types is what the
/// checker reads; a line the checker refuses comes back with the rule it broke. Where
/// the verb's signature has a text parameter (`say hello there`, `tell Pell who are
/// you`, `dig north chunk-5 The Salt Stair`), everything from that position on is
/// quoted for the typist, as the grammar wants it; text already in quotes is left alone.
/// Straight quotes inside a text argument, alternately opening and closing, as
/// curly ones: the checker reads `"` as the argument's edge.
fn curl_inner_quotes(text: &str) -> String {
    let mut open = true;
    text.chars()
        .map(|c| {
            if c == '"' {
                open = !open;
                if open {
                    '\u{201d}'
                } else {
                    '\u{201c}'
                }
            } else {
                c
            }
        })
        .collect()
}

pub fn compile(actor: &str, world: &str, line: &str, now: Timestamp, checker: &Checker) -> String {
    let full = one_line(line);
    let body = full.trim_end_matches('.');
    let (verb, _) = body
        .split_once(' ')
        .map(|(v, r)| (v, r.trim()))
        .unwrap_or((body, ""));
    // What follows the text parameter is the text itself, period and all.
    let rest = full.split_once(' ').map(|(_, r)| r.trim()).unwrap_or("");
    let text_at = checker.signature(verb).and_then(|params| {
        params
            .iter()
            .position(|p| p.get("type").and_then(Value::as_str) == Some("text"))
    });
    let quoted;
    // A room's name typed after `travel` or `where` ("Mortar and Pestle", "Moonpool,
    // Described") is quoted whole: read as words, its "and" or comma tripped the
    // checker or split the name, and one message in four of an explorer's failed.
    let multiword_place = matches!(verb, "travel" | "where")
        && rest.split_whitespace().count() > 1
        && !rest.starts_with('"')
        && !rest.starts_with('\u{201c}')
        // A door and its quoted word (`travel wicket "linger"`) are two arguments.
        && !rest.contains('"')
        && !rest.to_ascii_lowercase().starts_with("to ");
    // A thing's name typed after `take`, `drop` or `read` ("dab of ink", "a spare
    // minute") is quoted whole too: "of" read as a second argument voided the
    // message the world had just said "take it" in.
    let multiword_thing = matches!(verb, "take" | "drop" | "read")
        && rest.split_whitespace().count() > 1
        && !rest.starts_with('"')
        && !rest.starts_with('\u{201c}');
    let body = match text_at {
        _ if multiword_place || multiword_thing => {
            quoted = format!(
                "{verb} \"{}\"",
                rest.trim_end_matches('.').trim_end_matches('"')
            );
            quoted.as_str()
        }
        Some(i) if !rest.is_empty() => {
            let tokens: Vec<&str> = rest.split_whitespace().collect();
            if tokens.len() > i && !tokens[i].starts_with('"') && !tokens[i].starts_with('“') {
                let text = tokens[i..].join(" ");
                let head = tokens[..i].join(" ");
                // R30: a quote inside the text (`say He said "hi" to me`) is written
                // curly, so the outer quoting holds and the message is not voided.
                let text = curl_inner_quotes(text.trim_end_matches('"'));
                quoted = if head.is_empty() {
                    format!("{verb} \"{text}\"")
                } else {
                    format!("{verb} {head} \"{text}\"")
                };
                quoted.as_str()
            } else if tokens.len() > i && tokens[i].starts_with('"') {
                // Typed with its own quotes and a quote inside them: the inside
                // goes curly too.
                let text = tokens[i..].join(" ");
                let head = tokens[..i].join(" ");
                let inner = text
                    .trim_start_matches('"')
                    .trim_end_matches('.')
                    .trim_end_matches('"');
                if inner.contains('"') {
                    let text = curl_inner_quotes(inner);
                    quoted = if head.is_empty() {
                        format!("{verb} \"{text}\"")
                    } else {
                        format!("{verb} {head} \"{text}\"")
                    };
                    quoted.as_str()
                } else {
                    body
                }
            } else {
                body
            }
        }
        _ => body,
    };
    format!(
        "FROM: {actor}\nTO: {world}\nINTENT: REQUEST\nREF: {actor}-{}\n\n[DO] {actor} MUST {body}.\n",
        iso_minute(now)
    )
}

/// Run one message through the gate for an authenticated actor. `kind` is what the
/// channel authenticated the actor as; raw gate messages are always citizens.
#[allow(clippy::too_many_arguments)]
pub fn pass(
    world: &mut World,
    checker: &Checker,
    budget: &mut HourBudget,
    asks: &dyn AskBinding,
    actor: &str,
    kind: ActorKind,
    via: Via,
    compiled: bool,
    text: &str,
) -> Passage {
    let world_name = world.id.0.clone();
    let now = world.now();
    // One cap on what a message may be, whichever door it came in by: the checker
    // is regular expressions over the text and the log keeps every byte.
    if text.len() > MAX_MESSAGE_BYTES {
        let short = text.chars().take(200).collect::<String>();
        let result = checker.check(&short);
        let mut report = Report::new(&world_name, actor, None);
        report.fact(
            format!(
                "The message is {} bytes; the world reads at most {MAX_MESSAGE_BYTES}",
                text.len()
            ),
            "gate",
        );
        budget.charge_cost(actor, now, if compiled { 0.0 } else { budget.fail_cost });
        return finish(
            world,
            short,
            result,
            report,
            false,
            budget.remaining(actor, now),
            Vec::new(),
        );
    }
    let result = checker.check(text);
    let msg = &result.message;
    let r#ref = msg.field("REF").filter(|r| !r.is_empty());
    let mut report = Report::new(&world_name, actor, r#ref);
    // What happened to the sender since they last wrote, for the channels that have
    // no other way to hear it: an ambush, a strike, words left for them. The text port
    // prints these itself and the session pushes them.
    if matches!(via, Via::Gate | Via::Http) {
        for line in since_last_message(world, actor) {
            report.quote(line, "narrator");
        }
    }
    // Where this message's own statements begin in the record: what they cause to be
    // said to the sender (a creature's answer to a story) goes in this reply, not
    // the next one. ("Every reply says 'and it answers' but never gives an answer.")
    let began = world
        .log
        .head(&world.id)
        .ok()
        .flatten()
        .map(|s| s.0)
        .unwrap_or(0);
    let mut outcomes = Vec::new();

    // The budget is on messages written as messages, one unit each. A compiled line
    // is typing, and typing is cheaper, but it is not free on a port an agent can
    // drive: a player ran several hundred `/v0/command` lines an hour against a
    // budget that never moved (#2). The text port is local and the operator's.
    // A message of reads alone (look, chunks, rest, wants…) is free: a tester polling
    // their hit points spent the hour on it and was cut off before `leave`.
    let reads_only = {
        let dos: Vec<&Statement> = msg
            .statements
            .iter()
            .filter(|s| s.r#type.as_deref() == Some("DO"))
            .collect();
        !dos.is_empty()
            && dos.iter().all(|s| {
                s.verb
                    .as_deref()
                    .is_some_and(|v| READ_VERBS.contains(&v.to_lowercase().as_str()))
            })
    };
    let cost = match (compiled, via) {
        (false, _) if reads_only => 0.0,
        (false, _) => 1.0,
        (true, Via::Http) => COMPILED_HTTP_COST,
        (true, Via::Session) => COMPILED_SESSION_COST,
        (true, Via::Text | Via::Gate) => 0.0,
    };
    if cost > 0.0 {
        if let Err(resets) = budget.check_cost(actor, now, cost) {
            // To the second, and as a wait (#7: truncated to the minute, the reset
            // could read as the minute it was refused at, and the hour is a rolling
            // one from the first charged message, not the clock's).
            let wait = (resets.0 - now.0).max(0) / 1000;
            report.fact(
                format!(
                    "{actor} has spent the rolling hour's sixty messages; the budget frees at {} ({} after {})",
                    iso_second(resets),
                    if wait >= 90 { format!("{}m {}s", wait / 60, wait % 60) } else { format!("{wait}s") },
                    iso_second(now)
                ),
                "budget",
            );
            return finish(
                world,
                text.to_string(),
                result,
                report,
                false,
                budget.remaining(actor, now),
                outcomes,
            );
        }
    }

    let passed = result.verdict == "PASS";
    if !compiled && !(reads_only && passed) {
        budget.charge(actor, now, passed);
    } else if cost > 0.0 {
        budget.charge_cost(actor, now, cost);
    }
    // Every message that crosses the Gate is in the log, word for word, as the
    // invitation promises. Humans read it like any other event.
    let _ = world.note(
        foxing_log::Actor::Citizen(actor.to_string()),
        "gate.message",
        serde_json::json!({"from": actor, "text": text, "verdict": result.verdict, "score": result.score, "via": via.name(), "compiled": compiled}),
    );

    // The sender must be who they say they are, and must be talking to this world.
    let from = msg.field("FROM").unwrap_or("");
    let to = msg.field("TO").unwrap_or("");
    if passed && from != actor {
        report.fact(
            format!("The message names {from} as FROM, and the world authenticated {actor}"),
            "gate",
        );
        report.fact(
            "The world acts only on messages whose FROM is the authenticated sender",
            "gate",
        );
        return finish(
            world,
            text.to_string(),
            result,
            report,
            false,
            budget.remaining(actor, now),
            outcomes,
        );
    }
    if passed && to != world_name {
        report.fact(
            format!("The message is addressed to {to}, and this world is {world_name}"),
            "gate",
        );
        report.fact(
            "The world does not relay messages between actors in this version",
            "gate",
        );
        return finish(
            world,
            text.to_string(),
            result,
            report,
            false,
            budget.remaining(actor, now),
            outcomes,
        );
    }

    if !passed {
        rejection(&result, &mut report);
        return finish(
            world,
            text.to_string(),
            result,
            report,
            false,
            budget.remaining(actor, now),
            outcomes,
        );
    }

    let intent = msg.field("INTENT").unwrap_or("");
    match intent {
        "REQUEST" => run_request(world, msg, actor, kind, &mut report, &mut outcomes),
        "QUERY" => run_query(world, msg, actor, asks, &mut report),
        "PROPOSE" => {
            report.fact("The world does not take proposals in this version", "gate");
            report.fact(
                "A PROPOSE will create an offer entity when the economy exists",
                "gate",
            );
        }
        _ => {
            report.fact(
                format!("The world received the {intent} message and recorded nothing from it"),
                "gate",
            );
            report.fact(
                "The world never takes a visitor's FACT as a fact about the world",
                "gate",
            );
        }
    }
    let acted = matches!(intent, "REQUEST" | "QUERY");
    // F2: a message every statement of which the world refused (a precondition, not
    // a rule) did nothing, and costs what a refused message costs, not a whole one.
    if !compiled
        && !reads_only
        && passed
        && intent == "REQUEST"
        && !outcomes.is_empty()
        && outcomes.iter().all(|o| !o.ok)
    {
        budget.charge_cost(actor, now, budget.fail_cost - 1.0);
        report.fact(
            "Nothing in this message ran, so it cost what a refused one costs, not a whole message",
            "budget",
        );
    }
    if matches!(via, Via::Gate | Via::Http) && !compiled {
        let left = budget.remaining(actor, now);
        // R47: the hour is a rolling one from the first charged message, and the
        // figure climbs back when it frees (54 then 59 read as jitter); say when,
        // and what a refused message costs.
        let frees = budget
            .frees_at(actor, now)
            .map(|t| format!(", which frees at {}", iso_second(t)))
            .unwrap_or_default();
        report.fact(
            format!(
                "Budget: {left:.0} of {:.0} messages left in the budget's hour{frees}{}",
                budget.per_hour,
                if reads_only {
                    "; a message of reads alone is free, as this one was"
                } else {
                    "; a message of reads alone (look, chunks, rest, wants) is free"
                },
            ),
            "budget",
        );
        if result.verdict != "PASS" {
            report.fact(
                format!(
                    "A message refused at the door costs {} of one, not one",
                    match budget.fail_cost {
                        c if (c - 0.25).abs() < 1e-9 => "a quarter".to_string(),
                        c if (c - 0.5).abs() < 1e-9 => "a half".to_string(),
                        c => format!("{c}"),
                    }
                ),
                "budget",
            );
        }
    }
    if matches!(via, Via::Gate | Via::Http) {
        let said: Vec<String> = world
            .recent_events(500)
            .iter()
            .filter(|e| e.seq.0 > began)
            .filter(|e| e.kind == crate::disremembered::NARRATIVE && e.payload["to"] == actor)
            .filter_map(|e| e.payload["text"].as_str().map(str::to_string))
            .collect();
        for line in said {
            // Not a second time: a creature's answer to a story is the statement's
            // own summary and also narrated to the teller, and a visitor read every
            // reply twice.
            // The same words in a different frame count too: the statement's own
            // line says `Vellic says, "…"` and the narrator `Vellic says to you, "…"`.
            let quoted: Option<&str> = line.find('"').and_then(|a| {
                line.rfind('"')
                    .filter(|b| *b > a + 12)
                    .map(|b| &line[a + 1..b])
            });
            let already = report.lines.iter().any(|l| match l {
                Line::Quote { text, .. } => {
                    text.contains(line.trim())
                        || line.contains(text.trim())
                        || quoted.is_some_and(|q| text.contains(q))
                }
                Line::Fact { text, .. } => quoted.is_some_and(|q| text.contains(q)),
            });
            if !already {
                report.quote(line, "narrator");
            }
        }
    }
    finish(
        world,
        text.to_string(),
        result,
        report,
        acted,
        budget.remaining(actor, now),
        outcomes,
    )
}

fn finish(
    world: &mut World,
    message: String,
    result: CheckResult,
    report: Report,
    acted: bool,
    budget_remaining: f64,
    outcomes: Vec<StatementOutcome>,
) -> Passage {
    let reply = report.render();
    let _ = world.note(
        foxing_log::Actor::Driver,
        "gate.reply",
        serde_json::json!({"to": report.to, "text": reply, "acted": acted}),
    );
    let ok = acted && outcomes.iter().all(|o| o.ok);
    Passage {
        message,
        verdict: result.verdict,
        score: result.score,
        acted,
        prose: report.prose(),
        reply,
        findings: result.findings,
        budget_remaining,
        ok,
        outcomes,
    }
}

/// A rejection cites every error by rule.
fn rejection(result: &CheckResult, report: &mut Report) {
    let r#ref = result
        .message
        .field("REF")
        .filter(|r| !r.is_empty())
        .unwrap_or("without REF")
        .to_string();
    for f in result
        .findings
        .iter()
        .filter(|f| f.severity == foxing_clear::ERROR)
    {
        let place = match f.r#where.as_str() {
            "header" => "The header".to_string(),
            "message" => "The message".to_string(),
            w => {
                let n = w.rsplit(' ').next().unwrap_or("0");
                format!("Statement {n}")
            }
        };
        // The rule by number and by what it says: a visitor met "failed rule CL-I5"
        // and had no way to know the verb was the problem.
        // The reason for the verb rule only, first clause, since a visitor met
        // "failed rule CL-I5" with no way to know the verb was the problem. Not for
        // the others: quoting the hedge or the relative time re-offends in the
        // world's own FACT, and the reply obeys the same checker.
        let why: String = if !matches!(f.rule.as_str(), "CL-I5" | "CL-I6" | "CL-I1") {
            String::new()
        } else {
            f.message
                .split(['.', ';'])
                .next()
                .unwrap_or("")
                .split_whitespace()
                .take(12)
                .collect::<Vec<_>>()
                .join(" ")
        };
        report.fact(
            if why.is_empty() {
                format!("{place} of message {ref} failed rule {}", f.rule)
            } else {
                format!("{place} of message {ref} failed rule {} ({why})", f.rule)
            },
            "clear_check",
        );
    }
    report.fact(
        format!(
            "The message scored {} Legibility and the world ran none of it",
            result.score
        ),
        "clear_check",
    );
    if result.findings.iter().any(|f| f.rule == "CL-I5") {
        report.fact(
            "The `commands` tool lists the verbs this world runs",
            "gate",
        );
    }
}

fn run_request(
    world: &mut World,
    msg: &Message,
    actor: &str,
    kind: ActorKind,
    report: &mut Report,
    outcomes: &mut Vec<StatementOutcome>,
) {
    let dos: Vec<&Statement> = msg
        .statements
        .iter()
        .filter(|s| s.r#type.as_deref() == Some("DO"))
        .collect();
    if dos.is_empty() {
        report.fact(
            "The REQUEST carried no DO statement for the world to run",
            "gate",
        );
        return;
    }
    // A death the sender has not read yet: the statements were written for another
    // room (a delver's "You go down… You go west" walked from the Threshold into a
    // digging), so only the reads run; the rest wait for a message written after.
    // The same holds for the statements after a death inside one message: they too
    // were written for the room the sender was in (an `attack` written for the
    // Stacks once hit a Threshold wisp, where nothing hunts).
    let mut woke = died_since_last_message(world, actor);
    // Which statement of this message the fall was at, when it was in this one.
    let mut fell_at: Option<usize> = None;
    // The last statement that was a travel and arrived: a look right after is carried.
    let mut last_travel_ok: Option<usize> = None;
    let batch = dos.len() > 1;
    for st in dos {
        if woke {
            let verb = st.verb.clone().unwrap_or_default();
            if !AFTER_DEATH.contains(&verb.as_str()) {
                report.fact(
                    match fell_at {
                        Some(k) => format!("Statement {} did not run: you woke at the Threshold; read the room first. {actor} went down at statement {k}, in another room", st.index),
                        None => format!("Statement {} did not run: you woke at the Threshold; read the room first. {actor} went down since the last message, in another room", st.index),
                    },
                    "gate",
                );
                continue;
            }
        }
        let n = st.index;
        let is_sender =
            matches!(st.actor.as_deref(), Some(a) if a == actor || a == "I" || a == "i");
        if !is_sender {
            report.fact(
                format!("Statement {n} names {} as its actor and the world runs only the sender's own actions", st.actor.as_deref().unwrap_or("nobody")),
                "gate",
            );
            continue;
        }
        if matches!(st.modal.as_deref(), Some("MUST NOT") | Some("SHOULD NOT")) {
            report.fact(
                format!("Statement {n} is a prohibition and the world recorded nothing from it"),
                "gate",
            );
            continue;
        }
        let verb = st.verb.clone().unwrap_or_default();
        let args = Value::Array(compile_arguments(&st.args));
        // `travel` says the room it arrives in; a `look` right after it in the same
        // message is carried by it (the reply once quoted the room twice and the
        // checker warned on the pair).
        if verb.eq_ignore_ascii_case("look")
            && args.as_array().map_or(true, |a| {
                a.is_empty()
                    || a.iter()
                        .all(|v| v.as_str().is_some_and(|s| s.trim().is_empty()))
            })
            && last_travel_ok == Some(n.saturating_sub(1))
        {
            report.fact(
                format!("Statement {n} (look) is carried by statement {}: travel says the room it reaches", n - 1),
                "gate",
            );
            outcomes.push(StatementOutcome {
                statement: n,
                verb: verb.clone(),
                ok: true,
                result: serde_json::json!({"carried_by": n - 1}),
            });
            continue;
        }
        let before = world
            .log
            .head(&world.id)
            .ok()
            .flatten()
            .map(|s| s.0)
            .unwrap_or(0);
        let ctx = CallContext {
            actor: actor.to_string(),
            kind,
        };
        match world.call(&ctx, &verb, &args) {
            Ok(out) => {
                if out.result["died"] == Value::Bool(true) {
                    woke = true;
                    fell_at = Some(n);
                }
                if verb.eq_ignore_ascii_case("travel") && out.result["died"] != Value::Bool(true) {
                    last_travel_ok = Some(n);
                }
                outcomes.push(StatementOutcome {
                    statement: n,
                    verb: verb.clone(),
                    ok: true,
                    result: out.result.clone(),
                });
                let after = world
                    .log
                    .head(&world.id)
                    .ok()
                    .flatten()
                    .map(|s| s.0)
                    .unwrap_or(0);
                let src = format!("ev-{after}");
                let at = iso_minute(world.now());
                if after > before {
                    report.receipt(format!("Statement {n} ran {verb} for {actor} at {at} and recorded events ev-{} to ev-{after}", before + 1), &src);
                } else {
                    report.receipt(
                        format!("Statement {n} ran {verb} for {actor} at {at} and changed nothing"),
                        &src,
                    );
                }
                // R64: in a batch, one statement's answer is capped so the others fit.
                let cap = if batch {
                    PER_STATEMENT_LINES
                } else {
                    usize::MAX
                };
                report.quote_capped(out.summary, &world.id.0, cap);
            }
            Err(e) => {
                outcomes.push(StatementOutcome {
                    statement: n,
                    verb: verb.clone(),
                    ok: false,
                    result: serde_json::to_value(&e).unwrap_or(Value::Null),
                });
                let (reason, src) = match &e {
                    CommandError::Unknown { .. } => ("names no command this world has", "registry"),
                    CommandError::NotAllowed { .. } => {
                        ("names a command the sender may not call", "registry")
                    }
                    CommandError::MissingParam { .. } | CommandError::BadParam { .. } => {
                        ("gives arguments the command does not take", "registry")
                    }
                    CommandError::Precondition { .. } => {
                        ("was refused by the world as a precondition", "precondition")
                    }
                    CommandError::Internal { .. } => ("failed inside the world", "driver"),
                };
                report.receipt(format!("Statement {n} {reason}"), src);
                report.quote(e.to_string(), &world.id.0);
            }
        }
        // A name through the gate is struck once a statement while something fights
        // it (its clock is the message), narrated into this reply with the rest.
        match crate::disremembered::gate_round(
            world,
            &crate::disremembered::Curves::from_env(),
            actor,
            &verb,
        ) {
            Ok(Some(said)) => report.quote(said, "narrator"),
            Ok(None) => {}
            Err(e) => report.fact(
                format!("The round after statement {n} failed inside the world: {e}"),
                "driver",
            ),
        }
        if !woke
            && world.recent_events(30).iter().any(|e| {
                e.kind == crate::disremembered::DIED
                    && e.payload["citizen"] == actor
                    && e.seq.0 > before
            })
        {
            woke = true;
            fell_at = Some(n);
        }
    }
}

fn run_query(
    world: &World,
    msg: &Message,
    actor: &str,
    asks: &dyn AskBinding,
    report: &mut Report,
) {
    let questions: Vec<&Statement> = msg
        .statements
        .iter()
        .filter(|s| s.r#type.as_deref() == Some("ASK"))
        .collect();
    if questions.is_empty() {
        report.fact(
            "The QUERY carried no ASK statement for the world to answer",
            "gate",
        );
        return;
    }
    for st in questions {
        if st.pattern.is_none() {
            report.fact(
                format!(
                    "Question {} matched no question pattern and the world cannot compile it",
                    st.index
                ),
                "clear_check",
            );
            continue;
        }
        if !asks.answer(world, actor, st, report) {
            report.fact(
                format!(
                    "Question {} matched a pattern this world does not answer yet",
                    st.index
                ),
                "gate",
            );
            report.fact(
                "The world answers: where is X; what is the level, hit points, attack or room of X; a count of creatures, citizens or chunks",
                "gate",
            );
        }
    }
}

/// Observe as `actor`, for ask bindings.
pub fn observe(
    world: &World,
    actor: &str,
    query: Value,
) -> Result<crate::observe::Observed, QueryError> {
    let req: crate::observe::Request =
        serde_json::from_value(query).map_err(|e| QueryError::BadFilter {
            message: e.to_string(),
        })?;
    world.observe(
        &Viewer::actor(actor),
        ActorKind::Citizen,
        &req,
        &Limits::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disremembered::{self, ask::Asks};
    use crate::world::LogBacking;
    use foxing_log::WorldId;
    use foxing_store::Store;
    use foxing_time::Duration;

    const PASS: &str = "a-passphrase-of-eight";

    fn world() -> World {
        let mut w = World::open(
            WorldId("disremembered".into()),
            Store::new(disremembered::registry()),
            LogBacking::Memory,
        )
        .unwrap();
        w.fixed_now = Some(Timestamp(1_793_100_000_000));
        w.seed(3);
        disremembered::ensure_world(&mut w).unwrap();
        disremembered::register(
            &mut w,
            disremembered::Curves::with_half_life(Duration::seconds(600)),
        );
        w.tick();
        disremembered::login(&mut w, "lantern-7", "test", PASS).unwrap();
        disremembered::login(&mut w, "spacepants", "test", PASS).unwrap();
        w
    }

    fn send(
        w: &mut World,
        checker: &Checker,
        budget: &mut HourBudget,
        actor: &str,
        text: &str,
    ) -> Passage {
        send_via(w, checker, budget, actor, Via::Gate, text)
    }

    fn send_via(
        w: &mut World,
        checker: &Checker,
        budget: &mut HourBudget,
        actor: &str,
        via: Via,
        text: &str,
    ) -> Passage {
        let p = pass(
            w,
            checker,
            budget,
            &Asks,
            actor,
            ActorKind::Citizen,
            via,
            via != Via::Gate,
            text,
        );
        // The world's own replies pass the same checker, always.
        let verdict = checker.check(&p.reply);
        assert_eq!(
            verdict.errors,
            0,
            "the world's reply fails its own checker:\n{}\n{}",
            p.reply,
            verdict.format_report()
        );
        p
    }

    fn msg(from: &str, intent: &str, body: &str) -> String {
        format!("FROM: {from}\nTO: disremembered\nINTENT: {intent}\nREF: {from}-test-1\n\n{body}\n")
    }

    #[test]
    fn a_message_written_before_a_death_runs_only_its_reads() {
        let mut w = world();
        let checker = checker();
        let mut budget = HourBudget::new(60.0, 0.25);
        let text = msg("lantern-7", "REQUEST", "[DO] lantern-7 MUST look.");
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        assert!(p.ok, "{}", p.prose);
        // Something took them down between messages (the record says so).
        w.note(
            foxing_log::Actor::Driver,
            crate::disremembered::DIED,
            serde_json::json!({"citizen": "lantern-7", "level": 0.0, "share": 0.0, "lost": []}),
        )
        .unwrap();
        let text = msg(
            "lantern-7",
            "REQUEST",
            "[DO] lantern-7 MUST travel to the east.\n[DO] lantern-7 MUST look.",
        );
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        assert!(
            p.reply
                .contains("Statement 1 did not run: you woke at the Threshold; read the room first. lantern-7 went down since the last message"),
            "{}",
            p.reply
        );
        assert!(p.reply.contains("Statement 2 ran look"), "{}", p.reply);
        // The next message runs as written.
        let text = msg(
            "lantern-7",
            "REQUEST",
            "[DO] lantern-7 MUST travel to the east.",
        );
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        assert!(p.reply.contains("Statement 1 ran travel"), "{}", p.reply);
    }

    #[test]
    fn a_name_through_the_gate_is_struck_once_a_statement_not_on_the_clock() {
        let mut w = world();
        let checker = checker();
        let mut budget = HourBudget::new(60.0, 0.25);
        disremembered::login_as(&mut w, "lantern-7", "gate", PASS, false).unwrap();
        assert!(disremembered::combat::through_the_gate(&w, "lantern-7"));
        let text = msg(
            "lantern-7",
            "REQUEST",
            "[DO] lantern-7 MUST travel to the east.\n[DO] lantern-7 MUST travel to the north.",
        );
        send(&mut w, &checker, &mut budget, "lantern-7", &text);
        disremembered::grown(&mut w, "lantern-7");
        // The ambush is the first round, on the clock.
        w.fixed_now = Some(w.now().plus(Duration::seconds(9)));
        w.tick();
        let f = disremembered::find_citizen(&w, "lantern-7").unwrap();
        let after_ambush = disremembered::combat::hp(&w, f.citizen);
        assert!(after_ambush < 30.0, "{after_ambush}");
        // A minute of ticks: no strike comes on the clock for a gate name.
        for i in 1..=6 {
            w.fixed_now = Some(w.now().plus(Duration::seconds(10 * i)));
            w.tick();
        }
        let quiet = disremembered::combat::hp(&w, f.citizen);
        assert!(
            quiet >= after_ambush - 0.01,
            "struck on the clock: {after_ambush} -> {quiet}"
        );
        // Each statement is a round: three looks, three strikes, in the reply.
        let text = msg(
            "lantern-7",
            "REQUEST",
            "[DO] lantern-7 MUST look.\n[DO] lantern-7 MUST look.\n[DO] lantern-7 MUST look.",
        );
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        // The ambush's line and three rounds, each after its statement.
        let hits = p.reply.matches("hits you for").count();
        assert_eq!(hits, 4, "{}", p.reply);
        let first_look = p.reply.find("Statement 1 ran look").unwrap();
        let second_look = p.reply.find("Statement 2 ran look").unwrap();
        let round_one = p.reply[first_look..]
            .find("The hollow hits you for")
            .unwrap()
            + first_look;
        assert!(round_one < second_look, "{}", p.reply);
        assert!(disremembered::combat::hp(&w, f.citizen) < quiet);
    }

    #[test]
    fn a_reply_through_the_gate_carries_what_happened_since_you_last_wrote() {
        let mut w = world();
        let checker = checker();
        let mut budget = HourBudget::new(60.0, 0.25);
        // Into the Shallows, where hollows ambush a few seconds after you arrive.
        let text = msg(
            "lantern-7",
            "REQUEST",
            "[DO] lantern-7 MUST travel to the east.\n[DO] lantern-7 MUST travel to the north.",
        );
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        assert!(p.ok, "{}", p.prose);
        assert!(
            !p.reply.contains("stands up out of the water"),
            "nothing has happened yet"
        );
        crate::disremembered::grown(&mut w, "lantern-7");
        w.fixed_now = Some(w.now().plus(Duration::seconds(9)));
        w.tick();
        // The next message's reply says what the hollow did, as the narrator's quote.
        let text = msg("lantern-7", "REQUEST", "[DO] lantern-7 MUST look.");
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        assert!(
            p.reply
                .contains("[QUOTE] The hollow stands up out of the water and comes at you.")
                && p.reply.contains("[src:narrator]"),
            "{}",
            p.reply
        );
        assert!(
            p.prose.contains("stands up out of the water"),
            "{}",
            p.prose
        );
        // And not twice.
        let text = msg("lantern-7", "REQUEST", "[DO] lantern-7 MUST retreat.");
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        assert!(
            !p.reply.contains("stands up out of the water"),
            "{}",
            p.reply
        );
        // A session or the text port hears these another way: not repeated there.
        w.fixed_now = Some(w.now().plus(Duration::seconds(15)));
        w.tick();
        let p = send_via(
            &mut w,
            &checker,
            &mut budget,
            "lantern-7",
            Via::Session,
            "look",
        );
        assert!(
            !p.reply.contains("[src:narrator]") || !p.reply.contains("hits you"),
            "{}",
            p.reply
        );
    }

    #[test]
    fn a_request_runs_each_do_in_order_and_reports_each_on_its_own() {
        let mut w = world();
        let checker = checker();
        let mut budget = HourBudget::new(60.0, 0.25);
        let text = msg(
            "lantern-7",
            "REQUEST",
            "[FACT] lantern-7 stands at the Threshold as of 2026-10-27T02:00Z. [src:ev-1]\n[DO] lantern-7 MUST travel to the east.\n[DO] lantern-7 MUST attack the wisp.\n[DO] lantern-7 MUST NOT retreat.\n[DO] spacepants SHOULD rest.\n[DO] lantern-7 MAY shelve chunk-999.",
        );
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        assert_eq!(p.verdict, "PASS", "{:?}", p.findings);
        assert!(p.acted);
        let reply = &p.reply;
        assert!(
            reply.starts_with(
                "FROM: disremembered\nTO: lantern-7\nINTENT: REPORT\nREF: lantern-7-test-1\n\n"
            ),
            "{reply}"
        );
        assert!(
            reply.contains("[FACT] Statement 2 ran travel for lantern-7 at 2026-10-27T11:20Z"),
            "{reply}"
        );
        assert!(
            reply.contains("[QUOTE] You go east. The Fogbank."),
            "{reply}"
        );
        assert!(
            reply.contains("[FACT] Statement 3 ran attack for lantern-7"),
            "{reply}"
        );
        assert!(
            reply.contains("[QUOTE] You hit the wisp")
                || reply.contains("[QUOTE] You swing at the wisp"),
            "{reply}"
        );
        assert!(reply.contains("[FACT] Statement 4 is a prohibition and the world recorded nothing from it. [src:gate]"), "{reply}");
        assert!(reply.contains("[FACT] Statement 5 names spacepants as its actor and the world runs only the sender's own actions. [src:gate]"), "{reply}");
        assert!(
            reply.contains(
                "[FACT] Statement 6 was refused by the world as a precondition. [src:precondition]"
            ),
            "{reply}"
        );
        // (Mid-fight, the shelf is refused before the chunk is looked for.)
        assert!(
            reply.contains("[QUOTE] You are in a fight; nothing goes on the shelf until it is over. [src:disremembered]"),
            "{reply}"
        );
        assert!((p.budget_remaining - 59.0).abs() < 1e-9);
        // The world moved: lantern-7 is in the Fogbank, in a fight.
        let look = w
            .call(
                &CallContext {
                    actor: "lantern-7".into(),
                    kind: ActorKind::Citizen,
                },
                "look",
                &Value::Array(vec![]),
            )
            .unwrap();
        assert_eq!(look.result["room"], "The Fogbank");
    }

    #[test]
    fn a_query_is_answered_from_observations_with_event_citations() {
        let mut w = world();
        let checker = checker();
        let mut budget = HourBudget::new(60.0, 0.25);
        w.call(
            &CallContext {
                actor: "spacepants".into(),
                kind: ActorKind::Admin,
            },
            "earn",
            &serde_json::json!(["hydra", 100]),
        )
        .unwrap();
        let text = msg(
            "lantern-7",
            "QUERY",
            "[ASK] Where is spacepants?\n[ASK] What is the level of spacepants?\n[ASK] What is the level of me?\n[ASK] How many chunks does spacepants hold?\n[ASK] How many creatures are in the fogbank?\n[ASK] What is the hit points of spacepants?\n[ASK] Where is nobody-here?\n[ASK] Who governs the fogbank?",
        );
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        assert_eq!(p.verdict, "PASS", "{:?}", p.findings);
        let r = &p.reply;
        assert!(
            r.contains("[FACT] spacepants is in The Threshold as of 2026-10-27T"),
            "{r}"
        );
        assert!(
            r.contains("[FACT] The level of spacepants is not visible to lantern-7. [src:ev-"),
            "{r}"
        );
        assert!(
            r.contains("[FACT] The level of lantern-7 is 0.00 XP as of"),
            "{r}"
        );
        assert!(
            r.contains("[FACT] spacepants holds 1 chunks in the active window as of"),
            "{r}"
        );
        assert!(
            r.contains(" creatures are in The Fogbank as of")
                || r.contains(" creature is in The Fogbank as of")
                || r.contains("0 creatures are in The Fogbank"),
            "{r}"
        );
        assert!(
            r.contains("[FACT] spacepants has 30 HP of 30 HP as of"),
            "{r}"
        );
        // A window past the engine's breadth (a hundred) still answers the level
        // question and the chunks readout: both come from the store. (#1)
        for _ in 0..120 {
            w.call(
                &CallContext {
                    actor: "lantern-7".into(),
                    kind: ActorKind::Admin,
                },
                "earn",
                &serde_json::json!(["moth", 2]),
            )
            .unwrap();
        }
        let text = msg("lantern-7", "QUERY", "[ASK] What is the level of me?");
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        assert!(
            p.reply
                .contains("[FACT] The level of lantern-7 is 240.00 XP as of"),
            "{}",
            p.reply
        );
        let out = w
            .call(
                &CallContext {
                    actor: "lantern-7".into(),
                    kind: ActorKind::Citizen,
                },
                "chunks",
                &Value::Array(vec![]),
            )
            .unwrap();
        assert!(out.summary.starts_with("Level 240.00"), "{}", out.summary);
        assert!(
            out.summary
                .contains("… and 20 smaller, worth 40.00 together"),
            "{}",
            out.summary
        );
        assert_eq!(out.result["chunks"].as_array().unwrap().len(), 120);
        // The board counts lantern-7's lines through the Gate: two written
        // messages of eight and one ASK lines (an ASK counts one) and the one DO.
        let (lines, _) = crate::disremembered::board::board(&w);
        let l = lines.iter().find(|l| l.name == "lantern-7").unwrap();
        assert!(l.actions >= 2, "{}", l.actions);
        assert!((l.per_action - l.holding / l.actions as f64).abs() < 1e-9);
        assert!(
            r.contains("[FACT] The world knows no citizen or named creature called nobody-here"),
            "{r}"
        );
        assert!(
            r.contains(
                "[FACT] Question 8 matched a pattern this world does not answer yet. [src:gate]"
            ) && r.contains("[FACT] The world answers: where is X;"),
            "{r}"
        );
    }

    #[test]
    fn a_failing_message_is_refused_by_rule_and_costs_a_fraction() {
        let mut w = world();
        let checker = checker();
        let mut budget = HourBudget::new(60.0, 0.25);
        let text = msg(
            "lantern-7",
            "REQUEST",
            "[FACT] It probably rained yesterday.\n[DO] lantern-7 MUST nuke the hydra.",
        );
        let before = w.log.head(&w.id).unwrap().unwrap().0;
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &text);
        assert_eq!(p.verdict, "FAIL");
        assert!(!p.acted);
        assert!(p.reply.contains("[FACT] Statement 1 of message lantern-7-test-1 failed rule CL-C2. [src:clear_check]"), "{}", p.reply);
        assert!(p.reply.contains("failed rule CL-R1."), "{}", p.reply);
        assert!(p.reply.contains("failed rule CL-R2."), "{}", p.reply);
        // The verb rule says why; a visitor met "failed rule CL-I5" and nothing more.
        assert!(p.reply.contains("[FACT] Statement 2 of message lantern-7-test-1 failed rule CL-I5 (Verb 'nuke' is not an approved action verb for the sender). [src:clear_check]"), "{}", p.reply);
        assert!(
            p.reply.contains("Legibility and the world ran none of it"),
            "{}",
            p.reply
        );
        assert!((p.budget_remaining - 59.75).abs() < 1e-9);
        // Nothing happened in the world beyond the record of the exchange itself.
        let events = w
            .log
            .read_all(&w.id, foxing_log::Cursor { after: before })
            .unwrap();
        let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
        assert_eq!(kinds, vec!["gate.message", "gate.reply"]);
        assert_eq!(events[0].payload["text"], text);
        assert_eq!(events[1].payload["text"], p.reply);
    }

    #[test]
    fn the_gate_acts_only_for_the_authenticated_sender_addressing_this_world() {
        let mut w = world();
        let checker = checker();
        let mut budget = HourBudget::new(60.0, 0.25);
        let p = send(
            &mut w,
            &checker,
            &mut budget,
            "lantern-7",
            &msg("spacepants", "REQUEST", "[DO] spacepants MUST travel east."),
        );
        assert!(!p.acted);
        assert!(
            p.reply
                .contains("names spacepants as FROM, and the world authenticated lantern-7"),
            "{}",
            p.reply
        );
        let elsewhere = "FROM: lantern-7\nTO: spacepants\nINTENT: INFORM\n\n[FACT] The fog is thick as of 2026-10-27T02:00Z. [src:ev-1]\n";
        let p = send(&mut w, &checker, &mut budget, "lantern-7", elsewhere);
        assert!(!p.acted);
        assert!(
            p.reply
                .contains("addressed to spacepants, and this world is disremembered"),
            "{}",
            p.reply
        );
        let inform = msg(
            "lantern-7",
            "INFORM",
            "[FACT] The fog is thick as of 2026-10-27T02:00Z. [src:ev-1]",
        );
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &inform);
        assert!(!p.acted);
        assert!(p.reply.contains("recorded nothing from it"), "{}", p.reply);
    }

    #[test]
    fn the_hourly_budget_runs_out_and_comes_back() {
        let mut w = world();
        let checker = checker();
        let mut budget = HourBudget::new(2.0, 0.25);
        let q = msg("lantern-7", "QUERY", "[ASK] Where is me?");
        send(&mut w, &checker, &mut budget, "lantern-7", &q);
        send(&mut w, &checker, &mut budget, "lantern-7", &q);
        let p = send(&mut w, &checker, &mut budget, "lantern-7", &q);
        assert!(!p.acted);
        assert!(
            p.reply
                .contains("has spent the rolling hour's sixty messages"),
            "{}",
            p.reply
        );
        assert!(
            p.reply.contains(
                "the budget frees at 2026-10-27T12:20:00Z (60m 0s after 2026-10-27T11:20:00Z)"
            ),
            "{}",
            p.reply
        );
        // Another agent is unaffected; an hour later, so is lantern-7.
        assert!(
            send(
                &mut w,
                &checker,
                &mut budget,
                "spacepants",
                &msg("spacepants", "QUERY", "[ASK] Where is me?")
            )
            .acted
        );
        w.fixed_now = Some(w.now().plus(Duration::seconds(3601)));
        assert!(send(&mut w, &checker, &mut budget, "lantern-7", &q).acted);
    }

    #[test]
    fn arguments_compile_from_the_parse_and_times_render_as_iso_minutes() {
        let checker = checker();
        let r = checker.check("FROM: a\nTO: disremembered\nINTENT: REQUEST\n\n[DO] a MUST say \"hello, there\".\n[DO] a MUST earn hydra 100 0.5.\n");
        assert_eq!(
            compile_arguments(&r.message.statements[0].args),
            vec![Value::String("hello, there".into())]
        );
        assert_eq!(
            compile_arguments(&r.message.statements[1].args),
            vec![
                Value::String("hydra".into()),
                Value::from(100),
                Value::from(0.5)
            ]
        );
        assert_eq!(iso_minute(Timestamp(0)), "1970-01-01T00:00Z");
        assert_eq!(
            iso_minute(Timestamp(1_793_100_000_000)),
            "2026-10-27T11:20Z"
        );
    }

    #[test]
    fn a_terminal_line_is_a_message_through_the_same_gate() {
        let mut w = world();
        let checker = checker();
        let mut budget = HourBudget::new(60.0, 0.25);
        let now = w.now();

        // What a human types compiles to one DO, and the prose is the narrator's.
        let text = compile(
            "spacepants",
            "disremembered",
            "travel to the east",
            now,
            &checker,
        );
        assert!(
            text.contains("[DO] spacepants MUST travel to the east."),
            "{text}"
        );
        let p = send_via(
            &mut w,
            &checker,
            &mut budget,
            "spacepants",
            Via::Text,
            &text,
        );
        assert!(p.ok, "{}", p.reply);
        assert!(p.prose.starts_with("You go east."), "{}", p.prose);
        assert!(
            !p.prose.contains("Statement 1"),
            "receipts stay in CLEAR: {}",
            p.prose
        );
        assert_eq!(p.outcomes.len(), 1);
        assert_eq!(p.outcomes[0].verb, "travel");

        // The message and the reply are in the log word for word, marked by channel,
        // and typing is not charged to the agent budget.
        let events = w.log.read_all(&w.id, foxing_log::Cursor::START).unwrap();
        let sent = events
            .iter()
            .rev()
            .find(|e| e.kind == "gate.message")
            .expect("the line is in the log");
        assert_eq!(sent.payload["via"], "text");
        assert_eq!(sent.payload["text"], text);
        assert!(events.iter().rev().any(|e| e.kind == "gate.reply"));
        assert_eq!(budget.remaining("spacepants", now), 60.0);
        // Typing on the local port is free; the same line over HTTP costs a quarter
        // of a message, and from the play page's session a twentieth, so the budget
        // the world card promises holds on every door an agent can drive (#2).
        let text = compile("spacepants", "disremembered", "look", now, &checker);
        let p = send_via(
            &mut w,
            &checker,
            &mut budget,
            "spacepants",
            Via::Http,
            &text,
        );
        assert!(p.ok, "{}", p.reply);
        assert!(
            (p.budget_remaining - 59.75).abs() < 1e-9,
            "{}",
            p.budget_remaining
        );
        let p = send_via(
            &mut w,
            &checker,
            &mut budget,
            "spacepants",
            Via::Session,
            &text,
        );
        assert!(
            (p.budget_remaining - 59.70).abs() < 1e-9,
            "{}",
            p.budget_remaining
        );
        for _ in 0..238 {
            send_via(
                &mut w,
                &checker,
                &mut budget,
                "spacepants",
                Via::Http,
                &text,
            );
        }
        let p = send_via(
            &mut w,
            &checker,
            &mut budget,
            "spacepants",
            Via::Http,
            &text,
        );
        assert!(!p.ok);
        assert!(
            p.reply
                .contains("has spent the rolling hour's sixty messages"),
            "{}",
            p.reply
        );
        budget.used.clear();

        // A message past the byte cap is refused at the door, by every door, for a
        // quarter unit written or nothing typed.
        let long = format!("FROM: spacepants\nTO: disremembered\nINTENT: REQUEST\nREF: x\n\n[DO] spacepants MUST say \"{}\".\n", "a".repeat(MAX_MESSAGE_BYTES));
        let p = send_via(
            &mut w,
            &checker,
            &mut budget,
            "spacepants",
            Via::Gate,
            &long,
        );
        assert!(!p.ok);
        assert!(
            p.reply.contains("the world reads at most 8192"),
            "{}",
            p.reply
        );
        assert!(
            (p.budget_remaining - 59.75).abs() < 1e-9,
            "{}",
            p.budget_remaining
        );
        budget.used.clear();

        // A refused command is not ok, and the prose says why in the world's words.
        let text = compile(
            "spacepants",
            "disremembered",
            "shelve chunk-99",
            now,
            &checker,
        );
        let p = send_via(
            &mut w,
            &checker,
            &mut budget,
            "spacepants",
            Via::Text,
            &text,
        );
        assert!(!p.ok);
        assert!(p.acted);
        assert!(!p.prose.is_empty());

        // A word the world has no verb for fails the checker, and the reply says what
        // to do, in CLEAR that passes the checker (send_via asserts that).
        let text = compile("spacepants", "disremembered", "east", now, &checker);
        let p = send_via(
            &mut w,
            &checker,
            &mut budget,
            "spacepants",
            Via::Text,
            &text,
        );
        assert!(!p.ok && !p.acted);
        assert!(p.prose.contains("CL-I5"), "{}", p.prose);
        assert!(
            p.prose.contains("lists the verbs this world runs"),
            "{}",
            p.prose
        );

        // Text after other arguments is quoted from where the signature says it starts.
        let text = compile(
            "spacepants",
            "disremembered",
            "tell tyler who are you?",
            now,
            &checker,
        );
        assert!(
            text.contains("[DO] spacepants MUST tell tyler \"who are you?\"."),
            "{text}"
        );
        let text = compile(
            "spacepants",
            "disremembered",
            "dig north chunk-5 The Salt Stair.",
            now,
            &checker,
        );
        assert!(
            text.contains("[DO] spacepants MUST dig north chunk-5 \"The Salt Stair.\"."),
            "{text}"
        );
        let text = compile(
            "spacepants",
            "disremembered",
            "dig north chunk-5",
            now,
            &checker,
        );
        assert!(
            text.contains("[DO] spacepants MUST dig north chunk-5."),
            "{text}"
        );
        let text = compile(
            "spacepants",
            "disremembered",
            "tell tyler \"already quoted\"",
            now,
            &checker,
        );
        assert!(
            text.contains("[DO] spacepants MUST tell tyler \"already quoted\"."),
            "{text}"
        );

        // Speech: the typist's words are quoted for them, recorded as said, and heard.
        disremembered::login(&mut w, "tyler", "test", PASS).unwrap();
        let tyler = CallContext {
            actor: "tyler".into(),
            kind: ActorKind::Citizen,
        };
        w.call(&tyler, "travel", &serde_json::json!(["east"]))
            .expect("tyler joins spacepants in the Fogbank");
        let text = compile(
            "spacepants",
            "disremembered",
            "say the fog is thinner north",
            now,
            &checker,
        );
        assert!(
            text.contains("[DO] spacepants MUST say \"the fog is thinner north\"."),
            "{text}"
        );
        let p = send_via(
            &mut w,
            &checker,
            &mut budget,
            "spacepants",
            Via::Text,
            &text,
        );
        assert!(p.ok, "{}", p.reply);
        assert!(
            p.prose.starts_with("You say, \"the fog is thinner north\""),
            "{}",
            p.prose
        );
        let events = w.log.read_all(&w.id, foxing_log::Cursor::START).unwrap();
        let said = events
            .iter()
            .rev()
            .find(|e| e.kind == disremembered::speech::SAY)
            .expect("recorded");
        assert_eq!(said.payload["text"], "the fog is thinner north");
        let heard = events
            .iter()
            .rev()
            .find(|e| e.kind == disremembered::NARRATIVE && e.payload["to"] == "tyler")
            .expect("tyler hears it");
        assert!(heard.payload["text"]
            .as_str()
            .unwrap()
            .contains("spacepants says, \"the fog is thinner north\""));

        // Too many arguments for the verb is the checker's refusal, not the registry's.
        let text = compile("spacepants", "disremembered", "look around", now, &checker);
        let p = send_via(
            &mut w,
            &checker,
            &mut budget,
            "spacepants",
            Via::Text,
            &text,
        );
        assert!(!p.acted && p.prose.contains("CL-I6"), "{}", p.prose);
    }

    #[test]
    fn a_pasted_message_gets_its_lines_back_and_is_not_compiled() {
        let flat = "FROM: lantern-7 TO: disremembered INTENT: QUERY REF: lantern-7-scout-1 [ASK] Where is spacepants? [ASK] How many creatures are in the fogbank?";
        assert!(is_message(flat));
        assert!(!is_message("attack the wisp"));
        let text = reflow(flat);
        assert_eq!(
            text,
            "FROM: lantern-7\nTO: disremembered\nINTENT: QUERY\nREF: lantern-7-scout-1\n\n[ASK] Where is spacepants?\n[ASK] How many creatures are in the fogbank?\n"
        );
        let mut w = world();
        let checker = checker();
        let mut budget = HourBudget::new(60.0, 0.25);
        let p = pass(
            &mut w,
            &checker,
            &mut budget,
            &Asks,
            "lantern-7",
            ActorKind::Citizen,
            Via::Text,
            false,
            &text,
        );
        assert!(p.acted, "{}", p.reply);
        assert!(p.reply.contains("spacepants is"), "{}", p.reply);
        // Written as a message, so it is budgeted like one, whichever port it came by.
        assert_eq!(budget.remaining("lantern-7", w.now()), 59.0);
    }
}

#[cfg(test)]
mod quoting_tests {
    use super::*;

    #[test]
    fn a_things_name_after_take_is_quoted_whole_and_a_door_word_is_left_alone() {
        let checker = checker();
        let now = Timestamp(1_000_000);
        let c = |line: &str| compile("spacepants", "disremembered", line, now, &checker);
        assert!(
            c("take dab of ink.").contains("take \"dab of ink\""),
            "{}",
            c("take dab of ink.")
        );
        assert!(
            c("drop river weed").contains("drop \"river weed\""),
            "{}",
            c("drop river weed")
        );
        assert!(
            c("read a spare minute.").contains("read \"a spare minute\""),
            "{}",
            c("read a spare minute.")
        );
        assert!(
            c("take item-12.").contains("take item-12"),
            "{}",
            c("take item-12.")
        );
        // A door and its word, quoted or not, stay two arguments.
        let t = c("travel wicket \"linger\".");
        assert!(t.contains("travel wicket \"linger\""), "{t}");
        let t = c("travel The Lookout.");
        assert!(t.contains("travel \"The Lookout\""), "{t}");
        // A quote inside the text goes curly, so the message is not voided (R30),
        // typed with or without its own quotes; the checker passes it.
        let t = c("say He said \"hi\" to me.");
        assert!(
            t.contains("say \"He said \u{201c}hi\u{201d} to me.\""),
            "{t}"
        );
        let t = c("tell docent \"what is a \"wisp\"?\"");
        assert!(
            t.contains("tell docent \"what is a \u{201c}wisp\u{201d}?\""),
            "{t}"
        );
        let r = checker.check(&t);
        assert_eq!(r.verdict, "PASS", "{:?}", r.findings);
    }
}
