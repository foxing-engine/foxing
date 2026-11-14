//! C5: a year in the world. The Conclave keeps its holidays (`life::NAMED_DAYS`);
//! this is the calendar everything else reads: a table of seasons by date (UTC), each
//! of which changes one thing while it is on, and the `almanac` that says what is on
//! and what comes next. A first cut of three:
//!
//! - **the Fog Month** (November): the fog is heavier. It thickens before anything
//!   else happens in the world, and brings one wisp more than the Fogbank keeps.
//! - **the Ripe Week** (16 to 22 November): the Orchard's hours ripen, the only time
//!   its fruit is a material. A ripe hour falls now and then; two things are made
//!   of it and nothing else.
//! - **the Night of One Story** (30 November, the fog's last night): every named
//!   creature in the world wants the same story.
//!
//! A season is a row in [`SEASONS`]; what it does is wherever the thing it changes
//! lives (`life::happen`, `weather::line`, `errand::of`), each asking [`on`].

use super::me;
use crate::world::World;
use foxing_registry::{Args, CommandSpec, Outcome};
use serde_json::json;

pub struct Season {
    pub key: &'static str,
    pub name: &'static str,
    /// First and last day, month and day (UTC), both inclusive; never across the
    /// year's end.
    pub from: (u32, u32),
    pub to: (u32, u32),
    /// What it does, for the almanac.
    pub does: &'static str,
}

pub const FOG_MONTH: &str = "fog-month";
pub const RIPE_WEEK: &str = "ripe-week";
pub const ONE_STORY: &str = "one-story";

pub const SEASONS: &[Season] = &[
    Season {
        key: FOG_MONTH,
        name: "the Fog Month",
        from: (11, 1),
        to: (11, 30),
        does: "the fog is heavier: it thickens at the Fogbank before anything else happens in the world, twice an hour, and brings a wisp more than the room keeps",
    },
    Season {
        key: RIPE_WEEK,
        name: "the Ripe Week",
        from: (11, 16),
        to: (11, 22),
        does: "the Orchard's hours ripen, the one week its fruit is a material: a ripe hour falls there about twice an hour, three lying at most (`take` it), and an orchard cordial is made of one and hollow water (`recipes`)",
    },
    Season {
        key: ONE_STORY,
        name: "the Night of One Story",
        from: (11, 30),
        to: (11, 30),
        does: "every named creature in the world wants the same story, a lantern lit against the fog, and pays double for it as for any want",
    },
];

/// The one story, and the word a telling is known by.
pub const THE_STORY: &str = "a lantern lit against the fog";
pub const THE_STORY_WORD: &str = "lantern";
pub const RIPE_HOUR: &str = "a ripe hour";
/// How many ripe hours lie in the Orchard at most.
pub const RIPE_LYING: usize = 3;

#[cfg(test)]
thread_local! {
    /// A test's date, month and day, in place of the clock's.
    pub static DATE: std::cell::Cell<Option<(u32, u32)>> = const { std::cell::Cell::new(None) };
}

/// Today, month and day (UTC). `FOXING_ALMANAC_DATE=MM-DD` sets it for a local
/// world, so a season can be walked through out of season.
pub fn today(world: &World) -> Option<(u32, u32)> {
    #[cfg(test)]
    if let Some(d) = DATE.with(|c| c.get()) {
        return Some(d);
    }
    if let Some((m, d)) = std::env::var("FOXING_ALMANAC_DATE").ok().and_then(|v| {
        let (m, d) = v.split_once('-')?;
        Some((m.trim().parse().ok()?, d.trim().parse().ok()?))
    }) {
        return Some((m, d));
    }
    let now = world.now().0;
    if now < super::life::CALENDAR_FROM_MS {
        return None;
    }
    let (_, m, d, _) = super::life::civil(now);
    Some((m, d))
}

fn within(s: &Season, md: (u32, u32)) -> bool {
    s.from <= md && md <= s.to
}

/// Whether a season is on.
pub fn on(world: &World, key: &str) -> bool {
    today(world).is_some_and(|md| SEASONS.iter().any(|s| s.key == key && within(s, md)))
}

/// The seasons that are on.
pub fn now_on(world: &World) -> Vec<&'static Season> {
    match today(world) {
        Some(md) => SEASONS.iter().filter(|s| within(s, md)).collect(),
        None => Vec::new(),
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const DAYS_IN: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn day_of_year(md: (u32, u32)) -> i64 {
    DAYS_IN[..(md.0 as usize - 1).min(11)].iter().sum::<u32>() as i64 + md.1 as i64
}

fn date_said(md: (u32, u32)) -> String {
    format!("{} {}", md.1, MONTHS[(md.0 as usize - 1).min(11)])
}

fn span_said(s: &Season) -> String {
    if s.from == s.to {
        date_said(s.from)
    } else if s.from.0 == s.to.0 {
        format!("{} to {}", s.from.1, date_said(s.to))
    } else {
        format!("{} to {}", date_said(s.from), date_said(s.to))
    }
}

/// Days from `md` until `to` comes round, 0 for today.
fn days_until(md: (u32, u32), to: (u32, u32)) -> i64 {
    (day_of_year(to) - day_of_year(md)).rem_euclid(365)
}

/// The almanac, as a paragraph: the date, what is on, what comes next.
pub fn said(world: &World) -> (String, serde_json::Value) {
    let Some(md) = today(world) else {
        return (
            "The almanac has not been opened yet: the calendar begins on 6 November 2026.".into(),
            json!({"today": null}),
        );
    };
    let mut lines = vec![format!(
        "The almanac, {} (the world keeps UTC).",
        date_said(md)
    )];
    let mut on_now = Vec::new();
    let mut ahead: Vec<(i64, String, serde_json::Value)> = Vec::new();
    let mut ahead_json: Vec<serde_json::Value> = Vec::new();
    for s in SEASONS {
        if within(s, md) {
            let left = days_until(md, s.to);
            lines.push(format!(
                "It is {}, {}: {}. {}",
                s.name,
                span_said(s),
                s.does,
                match left {
                    0 => "Today is its last day.".to_string(),
                    1 => "It ends tomorrow.".to_string(),
                    n => format!("It ends in {n} days, with {}.", date_said(s.to)),
                }
            ));
            on_now.push(json!({"key": s.key, "name": s.name, "days_left": left}));
        } else {
            let n = days_until(md, s.from);
            ahead.push((
                n,
                format!(
                    "{} ({}; in {n} day{}): {}",
                    s.name,
                    span_said(s),
                    if n == 1 { "" } else { "s" },
                    s.does
                ),
                json!({"key": s.key, "name": s.name, "in_days": n}),
            ));
        }
    }
    // The Conclave's holidays stand on the same calendar: listed after the
    // seasons, the rule said once.
    let mut days: Vec<(i64, String)> = super::life::NAMED_DAYS
        .iter()
        .map(|(m, d, name)| {
            let n = days_until(md, (*m, *d));
            ahead_json.push(json!({"key": "holiday", "name": name, "in_days": n}));
            (
                n,
                if n == 0 {
                    format!("{name} (today)")
                } else {
                    format!(
                        "{name} ({}; in {n} day{})",
                        date_said((*m, *d)),
                        if n == 1 { "" } else { "s" }
                    )
                },
            )
        })
        .collect();
    days.sort_by_key(|(n, _)| *n);
    ahead.sort_by_key(|(n, _, _)| *n);
    if on_now.is_empty() {
        lines.push("Nothing is in season today.".into());
    }
    if !ahead.is_empty() {
        lines.push(format!(
            "Ahead: {}.",
            ahead
                .iter()
                .map(|(_, s, _)| s.clone())
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    lines.push(format!(
        "The Conclave's holidays, on each of which it does not sit and something is left at the Threshold: {}. Every Sunday (UTC) is the Clerks' Sabbath, and it does not sit then either.",
        days.iter().map(|(_, s)| s.clone()).collect::<Vec<_>>().join("; ")
    ));
    (
        lines.join(" "),
        json!({
            "today": {"month": md.0, "day": md.1},
            "on": on_now,
            "ahead": ahead.into_iter().map(|(_, _, v)| v).chain(ahead_json).collect::<Vec<_>>(),
        }),
    )
}

/// One line for `time` and the Threshold: what is in season, if anything.
pub fn line(world: &World) -> Option<String> {
    let on = now_on(world);
    if on.is_empty() {
        return None;
    }
    Some(format!(
        "In season: {} (`almanac`).",
        on.iter().map(|s| s.name).collect::<Vec<_>>().join(", ")
    ))
}

pub fn register(world: &mut World) {
    world.register(
        CommandSpec::new(
            "disremembered.almanac",
            "The world's calendar: today's date (UTC), what is in season and what it changes, and what comes next with the days until it. Seasons are few and each changes one thing: the Fog Month, the Ripe Week, the Night of One Story; the Conclave's holidays stand on the same calendar.",
        ),
        Box::new(|w: &mut World, ctx, _: &Args| {
            me(w, ctx)?;
            let (text, data) = said(w);
            Ok(Outcome::new(text, data))
        }),
    );
}
