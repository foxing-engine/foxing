//! The operator's channel. Notices (an arrival worth knowing about) are posted to a
//! Discord webhook from a thread of their own; the world never waits on the network.
//! Without a webhook they go to stderr, so a local run still shows them.

use std::sync::mpsc;

/// Start the notifier; returns the sender the tick loop feeds.
pub fn start(webhook: Option<String>, world: &str) -> mpsc::Sender<String> {
    let (tx, rx) = mpsc::channel::<String>();
    let world = world.to_string();
    std::thread::Builder::new()
        .name("foxing-notify".into())
        .spawn(move || {
            let send = |line: &str| match &webhook {
                Some(url) => post(url, &world, line),
                None => eprintln!("notice: {line}"),
            };
            let mut crowd = Crowd::default();
            loop {
                match rx.recv_timeout(std::time::Duration::from_secs(1)) {
                    Ok(line) => {
                        if let Some(out) = crowd.take(line, std::time::Instant::now()) {
                            send(&out);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
                if let Some(out) = crowd.due(std::time::Instant::now()) {
                    send(&out);
                }
            }
        })
        .expect("notifier thread");
    tx
}

/// Arrivals said one by one are welcome until they come as a crowd. The first
/// [`SINGLY`] in any [`WINDOW`] go out as they are; the rest of that window are held
/// and said in one notice when it closes, so a load test or a busy evening is a line
/// a minute and not a line a name.
pub const SINGLY: usize = 3;
pub const WINDOW: std::time::Duration = std::time::Duration::from_secs(60);
/// How many names a crowd's notice spells out before it only counts.
pub const NAMED: usize = 12;
const ARRIVED: &str = " arrived for the first time";
/// What the world writes on an arrival from outside the operator's own networks.
const UNKNOWN: &str = "not from the lab";

#[derive(Default)]
struct Crowd {
    /// When the window opened, and how many arrivals it has passed singly.
    opened: Option<std::time::Instant>,
    passed: usize,
    held: Vec<String>,
}

impl Crowd {
    /// A line in: what to send now, if anything. Only arrivals are ever held.
    fn take(&mut self, line: String, now: std::time::Instant) -> Option<String> {
        // An unknown visitor is never one of a crowd: each is said, and said at once.
        if !line.contains(ARRIVED) || line.contains(UNKNOWN) {
            return Some(line);
        }
        if self.opened.is_none() {
            self.opened = Some(now);
            self.passed = 0;
        }
        if self.passed < SINGLY {
            self.passed += 1;
            return Some(line);
        }
        let name = line
            .split(ARRIVED)
            .next()
            .unwrap_or(&line)
            .trim()
            .to_string();
        self.held.push(name);
        None
    }

    /// The window's close: the held names as one notice. A window that held any opens
    /// the next one already full, so a steady crowd stays one line a window.
    fn due(&mut self, now: std::time::Instant) -> Option<String> {
        let opened = self.opened?;
        if now.duration_since(opened) < WINDOW {
            return None;
        }
        if self.held.is_empty() {
            self.opened = None;
            return None;
        }
        let held = std::mem::take(&mut self.held);
        self.opened = Some(now);
        self.passed = SINGLY;
        let named = held
            .iter()
            .take(NAMED)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let more = match held.len().saturating_sub(NAMED) {
            0 => String::new(),
            n => format!(", and {n} more"),
        };
        Some(if held.len() == 1 {
            format!("{named} arrived for the first time.")
        } else {
            format!(
                "{} more arrived for the first time in the last minute: {named}{more}.",
                held.len()
            )
        })
    }
}

/// How a notice is dressed for the channel (David, 11-28: "highlight when someone
/// unknown visits and any other improvements to readability"). Three weights:
/// an **embed** with a colored bar and a title for what the operator must not miss
/// (an unknown visitor, the world unable to write); a plain line led by a mark of
/// its kind for the world's own news; and small grey text for the lab's comings and
/// goings, which are most of the lines and the least of the news.
#[derive(Debug, PartialEq)]
struct Dressed {
    content: String,
    /// `(title, description, color)`.
    embed: Option<(String, String, u32)>,
}

const ORANGE: u32 = 0xE6_7E_22;
const YELLOW: u32 = 0xF1_C4_0F;
const RED: u32 = 0xE7_4C_3C;
const GREEN: u32 = 0x2E_CC_71;

fn dress(line: &str) -> Dressed {
    let embed = |title: &str, desc: String, color: u32| Dressed {
        content: String::new(),
        embed: Some((title.to_string(), desc, color)),
    };
    let plain = |content: String| Dressed {
        content,
        embed: None,
    };
    // The address, when the line ends with one in brackets or names one.
    let code_ip = |text: &str| -> String {
        text.split(|c: char| !(c.is_ascii_hexdigit() || c == '.' || c == ':'))
            .map(|w| w.trim_end_matches(['.', ':']))
            .filter(|w| w.parse::<std::net::IpAddr>().is_ok())
            .fold(text.to_string(), |acc, ip| {
                acc.replacen(ip, &format!("`{ip}`"), 1)
            })
    };
    if line.contains(UNKNOWN) {
        if line.starts_with("A visitor looked in") {
            let desc = code_ip(
                &line
                    .replace(", not from the lab", "")
                    .replace("A visitor looked in", "Someone looked in"),
            );
            return embed("\u{1f440} Unknown visitor at the door", desc, YELLOW);
        }
        // "**Name** arrived for the first time, via mcp — **not from the lab** (ip)."
        let (what, ip) = line
            .split_once(" — **not from the lab**")
            .unwrap_or((line, ""));
        let ip = ip.trim().trim_end_matches('.');
        let ip = ip.trim_start_matches('(').trim_end_matches(')');
        let title = if what.contains(ARRIVED) {
            "\u{1f7e0} Unknown visitor: a new name, not from the lab"
        } else {
            "\u{1f7e0} Unknown visitor is back, not from the lab"
        };
        let from = if ip.is_empty() {
            String::new()
        } else {
            format!("\nfrom `{ip}`")
        };
        return embed(title, format!("{what}.{from}"), ORANGE);
    }
    if let Some(rest) = line.strip_prefix("\u{26a0}\u{fe0f} ") {
        return embed("\u{1f6a8} The world cannot write", rest.to_string(), RED);
    }
    if let Some(rest) = line.strip_prefix("\u{2705} ") {
        return embed(
            "\u{2705} The world is writing again",
            rest.to_string(),
            GREEN,
        );
    }
    // The lab's own, and a crowd's count: small and grey.
    let coming = line.contains(ARRIVED) || line.contains(" is back");
    if coming {
        return plain(format!("-# \u{1f9ea} {line}"));
    }
    let mark = [
        ("Court of Lethe", "\u{2696}\u{fe0f}"),
        ("Conclave", "\u{1f3db}\u{fe0f}"),
        ("enacted by", "\u{1f3db}\u{fe0f}"),
        ("Ossuary", "\u{1f56f}\u{fe0f}"),
        ("lost its name", "\u{1f32b}\u{fe0f}"),
        ("forgotten its name", "\u{1f32b}\u{fe0f}"),
        ("The world forgot", "\u{1f32b}\u{fe0f}"),
        ("was born", "\u{1f423}"),
        ("riddle", "\u{2753}"),
        ("Purse of the Exergue", "\u{1f4b0}"),
        ("wrote a recipe", "\u{1f4dc}"),
        ("traded", "\u{1f91d}"),
        (" gave ", "\u{1f381}"),
        ("Bindery", "\u{1f4da}"),
        ("Namery", "\u{1f4da}"),
        (" named ", "\u{1f3f7}\u{fe0f}"),
        ("calling", "\u{1f9ed}"),
        ("room, is **", "\u{1f3e0}"),
    ]
    .iter()
    .find(|(key, _)| line.contains(key))
    .map(|(_, m)| *m)
    .unwrap_or("\u{25ab}\u{fe0f}");
    plain(format!("{mark} {line}"))
}

fn post(url: &str, world: &str, line: &str) {
    let d = dress(line);
    let embeds = match &d.embed {
        Some((title, description, color)) => serde_json::json!([{
            "title": title,
            "description": description,
            "color": color,
        }]),
        None => serde_json::json!([]),
    };
    let body = serde_json::json!({
        "username": format!("{world} (the world)"),
        "content": d.content,
        "embeds": embeds,
        "allowed_mentions": {"parse": []},
    });
    match ureq::post(url)
        .set("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send_string(&body.to_string())
    {
        Ok(_) => {}
        Err(ureq::Error::Status(code, _)) => eprintln!("notice: webhook answered {code}: {line}"),
        Err(e) => eprintln!("notice: webhook failed: {e}: {line}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_crowd_of_arrivals_is_one_notice_a_minute_and_a_few_are_said_singly() {
        let t0 = std::time::Instant::now();
        let mut c = Crowd::default();
        let line = |n: usize| format!("**Gn{n}** arrived for the first time, via mcp.");
        for n in 0..SINGLY {
            assert_eq!(c.take(line(n), t0), Some(line(n)));
        }
        for n in SINGLY..SINGLY + 20 {
            assert_eq!(c.take(line(n), t0), None, "held past the threshold");
        }
        assert_eq!(
            c.take("the log refused a write".into(), t0).as_deref(),
            Some("the log refused a write")
        );
        assert_eq!(c.due(t0 + WINDOW / 2), None);
        let said = c.due(t0 + WINDOW).expect("the window closes");
        assert!(
            said.starts_with("20 more arrived for the first time in the last minute: **Gn3**, ")
        );
        assert!(said.ends_with(", and 8 more."), "{said}");
        // The crowd goes on: nothing singly in the window after a full one.
        assert_eq!(c.take(line(99), t0 + WINDOW), None);
        let said = c.due(t0 + WINDOW * 2).expect("the next window");
        assert_eq!(said, "**Gn99** arrived for the first time.");
        // A quiet window ends it, and arrivals are said singly again.
        assert_eq!(c.due(t0 + WINDOW * 3), None);
        assert_eq!(c.take(line(100), t0 + WINDOW * 3), Some(line(100)));
    }

    #[test]
    fn an_unknown_visitor_is_an_embed_and_the_labs_own_are_small() {
        let d = dress("**Pennywren** arrived for the first time, via mcp — **not from the lab** (192.0.2.44).");
        let (title, desc, color) = d.embed.expect("an embed");
        assert!(title.contains("Unknown visitor: a new name"), "{title}");
        assert_eq!(
            desc,
            "**Pennywren** arrived for the first time, via mcp.\nfrom `192.0.2.44`"
        );
        assert_eq!(color, ORANGE);
        assert!(d.content.is_empty());
        let d = dress("**Pennywren** is back, via http — **not from the lab** (2001:db8::7).");
        let (title, desc, _) = d.embed.expect("an embed");
        assert!(title.contains("is back"), "{title}");
        assert!(desc.ends_with("from `2001:db8::7`"), "{desc}");
        let d = dress("A visitor looked in (cloak) from a new address, 192.0.2.9, not from the lab. No name yet.");
        let (title, desc, color) = d.embed.expect("an embed");
        assert!(title.contains("at the door"));
        assert_eq!(
            desc,
            "Someone looked in (cloak) from a new address, `192.0.2.9`. No name yet."
        );
        assert_eq!(color, YELLOW);
        // The lab's own, and a crowd, are small grey lines.
        let d = dress("**Gnaw** arrived for the first time, via mcp (lab).");
        assert_eq!(d.embed, None);
        assert!(d.content.starts_with("-# "), "{}", d.content);
        assert!(
            dress("20 more arrived for the first time in the last minute: **A**, **B**.")
                .content
                .starts_with("-# ")
        );
        // The world's own news is a plain line led by its mark; trouble is red.
        let d = dress("**The Court of Lethe** is sitting on case 4.");
        assert!(d.content.starts_with("\u{2696}"), "{}", d.content);
        assert_eq!(
            dress("\u{26a0}\u{fe0f} **The world cannot write.** x")
                .embed
                .unwrap()
                .2,
            RED
        );
        // And an unknown arrival is never held in a crowd.
        let t0 = std::time::Instant::now();
        let mut c = Crowd::default();
        for n in 0..SINGLY + 2 {
            let _ = c.take(
                format!("**Gn{n}** arrived for the first time, via mcp (lab)."),
                t0,
            );
        }
        let line =
            "**Stranger** arrived for the first time, via mcp — **not from the lab** (192.0.2.1).";
        assert_eq!(c.take(line.into(), t0).as_deref(), Some(line));
    }
}
